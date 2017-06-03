// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::cmp;
use std::time::Instant;
use std::net::SocketAddr;
use std::collections::HashMap;


// Internal Dependencies ------------------------------------------------------
mod remote;

use ::socket::Socket;
use self::remote::ServerRemote;
use super::{RemoteConnection, RemoteInput, RemoteOptions, connection_id_from_packet};
use ::{Config, ConnectionID, HostID, Error, Options, Input};


/// Server Tick States
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Ord, PartialOrd)]
pub enum ServerTick {
    WaitForClients = 0,
    AwaitOptions = 1,
    ConfirmOptions = 2,
    Migrate = 3,
    Initialize = 4,
    InitializeMigrate = 5,
    AwaitInput = 6,
    Unknown = 255
}

impl ServerTick {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => ServerTick::WaitForClients,
            1 => ServerTick::AwaitOptions,
            2 => ServerTick::ConfirmOptions,
            3 => ServerTick::Migrate,
            4 => ServerTick::Initialize,
            5 => ServerTick::InitializeMigrate,
            6 => ServerTick::AwaitInput,
            _ => ServerTick::Unknown
        }
    }
}

impl Default for ServerTick {
    fn default() -> Self {
        ServerTick::WaitForClients
    }
}


/// Implementation of a lock step protocol server.
pub struct ClockworkServer<O, I> {

    /// Socket this server is listening on.
    socket: Socket,

    /// Whether the server has already started.
    started: bool,

    sequence: u8,

    /// Currently connected clients on this server.
    remotes: HashMap<ConnectionID, ServerRemote<O, I>>,

    /// Last known remote address for each connection.
    addresses: HashMap<ConnectionID, SocketAddr>,

    /// The ConnectionID of the local client which started the server.
    host_id: ConnectionID,

    last_tick_time: Instant,
    tick_buffer: u32,
    ticks_per_sequence: u8

}

impl<O, I> ClockworkServer<O, I> where O: Options, I: Input {

    pub fn new(
        config: &Config,
        host_id: ConnectionID,
        was_started: bool

    ) -> Result<Self, Error> {
        Ok(Self {
            socket: {
                Socket::new(config.server_addr, config.packet_max_size)?
            },
            started: was_started,
            sequence: 0,
            remotes: HashMap::new(),
            addresses: HashMap::new(),
            host_id: host_id,
            last_tick_time: Instant::now(),
            tick_buffer: 0,
            ticks_per_sequence: 1
        })
    }

    pub fn with_migrated_states(
        mut self,
        previous_host_id: HostID,
        connections: Vec<(ConnectionID, RemoteConnection)>

    ) -> Self {
        for (id, connection) in connections {
            let address = connection.address();
            let remote = ServerRemote::<O, I>::new(
                id, address,
                Some(connection),
                Some(ServerTick::Migrate),
                id == previous_host_id
            );
            self.remotes.insert(id, remote);
            self.addresses.insert(id, address);
        }
        self
    }

    #[cfg_attr(feature = "cargo-clippy", allow(map_entry))]
    pub fn tick(&mut self, config: &Config) -> u64 {

        // Handle incoming connections and receive packets
        let mut new_connection = false;
        while let Ok((addr, packet)) = self.socket.try_recv() {
            if let Some(id) = connection_id_from_packet(config, &packet) {

                if self.remotes.contains_key(&id) {

                    // Check if the packet was actually consumed by the connection.
                    //
                    // If it was, see if the address we received the packet from
                    // differs from the last known address of the connection.
                    //
                    // If it does, we re-map the remotes address to the new one,
                    // effectively tracking the clients sending / receiving port.
                    //
                    // This is done so that when the client's NAT decides to switch
                    // sending the port, the connection doesn't end up sending
                    // packets into the void.
                    let remote = self.remotes.get_mut(&id).unwrap();
                    if remote.receive(self.sequence, id, packet) && addr != remote.connection().address() {
                        remote.set_address(addr);
                        self.addresses.insert(id, addr);
                    }

                // Only allow new clients to connect if the game has not been started yet
                } else if !self.started && self.remotes.len() < config.server_max_clients as usize {
                    let mut remote = ServerRemote::<O, I>::new(id, addr, None, None, false);
                    if remote.receive(self.sequence, id, packet) {
                        self.remotes.insert(id, remote);
                        self.addresses.insert(id, addr);
                        new_connection = true;
                    }
                }

            }
        }

        // Check for remote timeouts or disconnects
        let mut disconnected = Vec::new();
        self.remotes.retain(|id, remote| {
            if remote.timed_out(config) || remote.disconnected() {
                disconnected.push(*id);
                false

            } else {
                true
            }
        });

        // Remove any disconnected addresses and reset remotes in case of any
        // dis-/connect
        if !disconnected.is_empty() {
            for id in disconnected {
                self.addresses.remove(&id);
            }
        }

        if new_connection {
            for (order, (id, remote)) in self.remotes.iter_mut().enumerate() {

                // The client that also hosts the server should always go last
                // in the host migration order to avoid superfluous reconnection
                // attempts.
                if *id == self.host_id {
                    remote.set_order(255);

                } else {
                    remote.set_order(order as u8);
                }

            }
        }

        // Calculate number of ticks to be executed by clients with the next
        // input sequence
        if self.started && !self.remotes.is_empty() {

            let mut all_match = true;
            for remote in self.remotes.values() {
                if remote.sequence() != self.sequence.wrapping_add(1) {
                    all_match = false;
                }
            }

            if all_match {

                // In case the actual network latency is greater than the desired
                // tick rate we need to introduce additional non-input
                // ticks to keep up the simulation speed.
                //
                // This will of course increase the actual response time.
                let tick_ms = (1000 / config.high_tick_rate) as u32;
                let ms = self.last_tick_time.elapsed().subsec_nanos() / 1000000;

                self.tick_buffer += ms.saturating_sub(tick_ms);
                self.ticks_per_sequence = if self.tick_buffer >= tick_ms {
                    let additional_ticks = cmp::min(self.tick_buffer / tick_ms, 16);
                    self.tick_buffer -= additional_ticks * tick_ms;
                    1 + additional_ticks as u8

                } else {
                    1
                };

                self.last_tick_time = Instant::now();
                self.sequence = self.sequence.wrapping_add(1);

            }

        }

        // Find lowest ServerTick shared across all remotes
        let lowest_tick = self.remotes.values().min_by_key(|r| r.tick()).map(|r| r.tick());

        // Send current tick data for all remotes
        if let Some(tick) = lowest_tick {

            if !self.started && tick == ServerTick::Initialize {
                self.last_tick_time = Instant::now();
                self.started = true;
            }

            // We pre-sort by the connection ID to prevent any possible de-sync
            let mut remotes: Vec<&ServerRemote<O, I>> = self.remotes.values().collect();
            remotes.sort_by(|a, b| {
                a.connection().id().cmp(&b.connection().id())
            });

            let connections: Vec<&RemoteConnection> = remotes.iter().map(|r| r.connection()).collect();
            let options: Vec<&RemoteOptions<O>> = remotes.iter().filter_map(|r| r.options()).collect();
            let inputs: Vec<&RemoteInput<I>> = remotes.iter().filter_map(|r| r.input()).collect();

            for remote in self.remotes.values() {
                let packet = remote.send(
                    config,
                    tick,
                    self.sequence,
                    self.ticks_per_sequence,
                    self.host_id,
                    &connections[..],
                    &options[..],
                    &inputs[..]
                );
                self.socket.send_to(
                    &packet[..],
                    remote.connection().address()
                ).ok();
            }

            if tick >= ServerTick::Initialize {
                 config.high_tick_rate

            } else {
                 config.low_tick_rate
            }

        } else {
            config.low_tick_rate
        }

    }

}

