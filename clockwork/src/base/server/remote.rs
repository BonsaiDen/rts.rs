// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::time::Instant;
use std::net::SocketAddr;


// External Dependencies ------------------------------------------------------
use bincode::{serialize, deserialize, Infinite, Bounded};


// Internal Dependencies ------------------------------------------------------
use base::client::ClientTick;
use base::server::ServerTick;

use ::{Config, ConnectionID, Options, Input};
use base::{RemoteInput, RemoteConnection, RemoteOptions, base_packet};


/// Server side remote abstraction.
pub struct ServerRemote<O, I> {

    /// Current client clock tick.
    tick: ServerTick,

    /// The local sequence number for InputStates.
    sequence: u8,

    /// Last received input from the remote.
    input: Option<RemoteInput<I>>,

    /// Last received options from the remote.
    options: Option<RemoteOptions<O>>,

    /// Connection information of the remote.
    connection: RemoteConnection,

    /// Last time data was received from this remote.
    last_receive_time: Instant,

    /// Wether this remote was previously disconnected and should be dropped
    /// quickly by any given server.
    was_disconnected: bool

}

impl<O, I> ServerRemote<O, I> where O: Options, I: Input {

    pub fn new(
        id: ConnectionID,
        address: SocketAddr,
        connection: Option<RemoteConnection>,
        tick: Option<ServerTick>,
        was_disconnected: bool

    ) -> Self {
        Self {
            tick: tick.unwrap_or_else(ServerTick::default),
            sequence: 0,
            input: None,
            options: None,
            connection: connection.unwrap_or_else(|| RemoteConnection::new(id, address, 0)),
            last_receive_time: Instant::now(),
            was_disconnected: was_disconnected
        }
    }

    pub fn connection(&self) -> &RemoteConnection {
        &self.connection
    }

    pub fn sequence(&self) -> u8 {
        self.sequence
    }

    pub fn tick(&self) -> ServerTick {
        self.tick
    }

    pub fn input(&self) -> Option<&RemoteInput<I>> {
        self.input.as_ref()
    }

    pub fn options(&self) -> Option<&RemoteOptions<O>> {
        self.options.as_ref()
    }

    pub fn timed_out(&self, config: &Config) -> bool {
        self.last_receive_time.elapsed() > config.remote_timeout_threshold
    }

    pub fn disconnected(&self) -> bool {
        // TODO also implement clean disconnect / shutdown
        self.was_disconnected
    }

    pub fn set_order(&mut self, order: u8) {
        self.connection.set_order(order);
    }

    pub fn set_address(&mut self, address: SocketAddr) {
        self.connection.set_address(address);
    }

    pub fn send(
        &self,
        config: &Config,
        tick: ServerTick,
        sequence: u8,
        ticks_per_sequence: u8,
        host_id: ConnectionID,
        connections: &[&RemoteConnection],
        options: &[&RemoteOptions<O>],
        inputs: &[&RemoteInput<I>]

    ) -> Vec<u8> {

        let mut packet = base_packet(self.connection.id(), config, self.tick as u8);

        match tick {
            ServerTick::WaitForClients | ServerTick::Migrate => {
                if let Ok(mut bytes) = serialize(&host_id, Infinite) {
                    packet.append(&mut bytes);
                }
            },
            ServerTick::ConfirmOptions => {
                packet.push(options.len() as u8);
                for option in options {
                    if let Ok(mut bytes) = serialize(option, Infinite) {
                        packet.append(&mut bytes);
                    }
                }
            },
            ServerTick::Initialize => {
                packet.push(connections.len() as u8);
                for connection in connections {
                    if let Ok(mut bytes) = serialize(connection, Infinite) {
                        packet.append(&mut bytes);
                    }
                }
            },
            ServerTick::AwaitInput => {

                packet.push(sequence);
                packet.push(ticks_per_sequence);
                packet.push(inputs.len() as u8);

                for input in inputs {
                    if let Ok(mut bytes) = serialize(input, Bounded(255)) {
                        // TODO work around input length limitation
                        packet.push(bytes.len() as u8);
                        packet.append(&mut bytes);
                    }
                }

            },
            _ => {}
        }

        packet

    }

    pub fn receive(&mut self, sequence: u8, id: ConnectionID, packet: Vec<u8>) -> bool {

        if id != self.connection.id() || packet.len() < 9 {
            return false;
        }

        match (self.tick, ClientTick::from_u8(packet[8])) {

            // Wait for client to send options
            (ServerTick::WaitForClients, ClientTick::WaitForServer) |

            // Wait for client to confirm ready after migration
            (ServerTick::Migrate, ClientTick::WaitForServer) |

            // Wait for client to confirm migrations
            (ServerTick::InitializeMigrate, ClientTick::WaitForServer) |
            (ServerTick::InitializeMigrate, ClientTick::Ready) => {},

            (ServerTick::WaitForClients, ClientTick::SendOptions) => {
                self.tick = ServerTick::AwaitOptions;
            },

            // Receive client options
            (ServerTick::AwaitOptions, ClientTick::SendOptions) |
            (ServerTick::ConfirmOptions, ClientTick::SendOptions) => if packet.len() >= 10 {
                if let Ok(options) = deserialize::<RemoteOptions<O>>(&packet[9..]) {
                    self.tick = ServerTick::ConfirmOptions;
                    self.options = Some(options);
                }
            },

            // Wait for client to be ready
            (ServerTick::ConfirmOptions, ClientTick::Ready) |
            (ServerTick::Initialize, ClientTick::Ready) => {
                self.sequence = 0;
                self.tick = ServerTick::Initialize;
            },

            (ServerTick::Migrate, ClientTick::Ready) => {
                self.sequence = 0;
                self.tick = ServerTick::InitializeMigrate;
            },

            // Wait for client to confirm sync
            (ServerTick::Initialize, ClientTick::SyncConfirm) |
            (ServerTick::InitializeMigrate, ClientTick::SyncConfirm) |
            (ServerTick::AwaitInput, ClientTick::SyncConfirm) => {
                self.tick = ServerTick::AwaitInput;
            },

            // Wait for client to send inputs
            (ServerTick::AwaitInput, ClientTick::SendInput) => if packet.len() >= 11 {
                if let Ok(input) = deserialize::<RemoteInput<I>>(&packet[10..]) {
                    if input.sequence == sequence && input.sequence == self.sequence {
                        self.input = Some(input);
                        self.sequence = self.sequence.wrapping_add(1);
                    }
                }
            },
            (_, _) => {
                println!("Unknown packet {:?}/{:?}", self.tick, ClientTick::from_u8(packet[8]));
                return false;
            }

        }

        self.last_receive_time = Instant::now();
        true

    }

}

