// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::hash::Hasher;
use std::net::SocketAddr;
use std::marker::PhantomData;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;


// External Dependencies ------------------------------------------------------
use rand;


// Internal Dependencies ------------------------------------------------------
mod remote;

use ::socket::Socket;
use self::remote::{ClientRemote, ClientEvent};
use super::{RemoteConnection, connection_id_from_packet};
use ::{Config, ConnectionID, HostID, Event, Error, State, Options, Input};


/// Client Tick States
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Ord, PartialOrd)]
pub enum ClientTick {
    WaitForServer = 0,
    SendOptions = 1,
    Ready = 2,
    SyncConfirm = 3,
    SendInput = 4,
    Unknown = 255
}

impl ClientTick {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => ClientTick::WaitForServer,
            1 => ClientTick::SendOptions,
            2 => ClientTick::Ready,
            3 => ClientTick::SyncConfirm,
            4 => ClientTick::SendInput,
            _ => ClientTick::Unknown
        }
    }
}

impl Default for ClientTick {
    fn default() -> Self {
        ClientTick::WaitForServer
    }
}


/// Implementation of a lock step protocol client.
pub struct ClockworkClient<S, O, I, R> {

    /// Socket this clients uses for communication with the host.
    socket: Socket,

    /// State shared across all remotes.
    state: S,

    /// Local client state.
    local: ClientRemote<S, O, I, R>,

    /// All known remote connections from the host this client is currently
    /// connected to.
    connections: HashMap<ConnectionID, RemoteConnection>,

    /// ConnectionID of the current host this client is connected to.
    host_id: HostID,

    /// A list of ConnectionID's for hosts that were already used during the
    /// migration process
    used_host_migrations: Vec<ConnectionID>,

    /// Whether or not the client was disconnected from its host.
    network_status: NetworkStatus,

    /// Hash for options change detection.
    last_options_hash: Option<u64>,

    /// Local input queue of yet to be confirmed inputs.
    input_queue: Vec<I>,

    /// Reference type
    refs: PhantomData<R>

}

impl<S, O, I, R> ClockworkClient<S, O, I, R> where S: State<O, I, R>, O: Options, I: Input {

    pub fn new(config: &Config) -> Result<Self, Error> {
        Ok(Self {
            socket: {
                Socket::new("0.0.0.0:0", config.packet_max_size)?
            },
            state: S::default(),
            local: ClientRemote::<S, O, I, R>::new(
                ConnectionID::new(rand::random()),
                config.remote_addr
            ),
            connections: HashMap::new(),
            host_id: HostID::new(ConnectionID::new(0)),
            used_host_migrations: Vec::new(),
            network_status: NetworkStatus::Connecting,
            last_options_hash: None,
            input_queue: Vec::new(),
            refs: PhantomData
        })
    }

    pub fn state(&mut self) -> &mut S {
        &mut self.state
    }

    pub fn set_options(&mut self, options: O) {
        self.local.set_options(options);
    }

    pub fn queue_input(&mut self, input: I) {
        self.input_queue.push(input);
    }

    pub fn receive(&mut self, config: &Config, t: u64, refs: &mut R) -> Result<Vec<Event>, Error> {

        if self.local.timed_out(config) {
            self.network_status = NetworkStatus::Disconnected;
            Err(Error::RemoteTimeout)

        } else if self.local.disconnected() {
            self.network_status = NetworkStatus::Disconnected;
            Err(Error::Disconnected)

        } else {

            let mut network_events = Vec::new();
            while let Ok((addr, packet)) = self.socket.try_recv() {
                if addr == config.remote_addr {
                    if let Some(id) = connection_id_from_packet(config, &packet) {
                        if let Some(mut received) = self.local.receive(id, packet) {
                            network_events.append(&mut received);
                        }
                    }
                }
            }

            self.apply_network_events(config, network_events, t, refs)

        }

    }

    pub fn send(&mut self, config: &Config) -> u64 {

        if self.network_status == NetworkStatus::Disconnected {
            return config.high_tick_rate
        }

        // TODO do not send everything?
        let packet = self.local.send(config, self.input_queue.clone());
        self.socket.send_to(&packet[..], config.remote_addr).ok();

        // TODO return low tick rate during game pause
        config.high_tick_rate

    }

    pub fn reconnect(&mut self) {
        self.network_status = NetworkStatus::Reconnecting;
        self.local.reset();
    }

    pub fn migrate(&mut self) -> Option<(Option<SocketAddr>, HostID, ConnectionID)> {

        // If there is one extract and return it
        if let Some(host_id) = self.get_next_host() {

            // Spin up a new local server in case we're the next host
            if host_id == self.local.connection().id() {
                Some((None, host_id, self.local.connection().id()))

            // Connect to the next remote in line to host the game
            } else {

                // Mark that remote as used, so we don't try it again in case
                // it times out too
                let id = host_id.into();
                self.used_host_migrations.push(id);

                let state = &self.connections[&id];
                Some((Some(state.address()), host_id, self.local.connection().id()))
            }

        } else {
            None
        }

    }

    pub fn migration(&self) -> Migration {
        Migration {
            previous_host: self.host_id,
            local_host: self.local.connection().id(),
            connection: self.connections.iter().map(|(&id, state)| (id, state.clone())).collect()
        }
    }

    // Internal API -----------------------------------------------------------
    fn get_next_host(&self) -> Option<HostID> {

        let mut connections: Vec<(&ConnectionID, &RemoteConnection)> = self.connections.iter().filter(|&(id, _)| {
            !self.used_host_migrations.contains(id)

        }).collect();

        connections.sort_by(|a, b| {
            a.1.order().cmp(&b.1.order())
        });

        if connections.is_empty() {
            None

        } else {
            Some(HostID::new(*connections[0].0))
        }

    }

    fn apply_network_events(
        &mut self,
        config: &Config,
        network_events: Vec<ClientEvent<O, I>>,
        t: u64,
        refs: &mut R

    ) -> Result<Vec<Event>, Error> {

        // TODO clean up
        let mut events: Vec<Event> = Vec::new();
        for event in network_events {
            match event {

                ClientEvent::Connected(id) => {
                    self.host_id = HostID::new(id);
                    events.push(Event::HostConnect(
                        config.remote_addr,
                        self.host_id,
                        self.local.connection().id()
                    ));
                },

                ClientEvent::Options(options) => {

                    let mut hasher = DefaultHasher::new();
                    let options: Vec<(ConnectionID, O)> = options.into_iter().map(|o| {
                        o.data.hash(&mut hasher);
                        (o.id, o.data)

                    }).collect();

                    // Detect options changes
                    let new_options_hash = hasher.finish();
                    let options_changed = if let Some(last_hash) = self.last_options_hash {
                        new_options_hash != last_hash

                    } else {
                        true
                    };

                    if options_changed {
                        self.last_options_hash = Some(new_options_hash);
                        self.state.apply_options(self.host_id, &options[..]);
                        if self.state.is_ready() {
                            self.local.set_tick(ClientTick::Ready);
                        }
                        events.push(Event::RemoteOptions);
                    }

                },

                ClientEvent::Ready(connections) => {

                    if let Some(connections) = connections {

                        if self.network_status == NetworkStatus::Connecting {
                            let connections = self.connections();
                            self.state.init(
                                self.host_id,
                                &connections[..],
                                refs
                            );
                        }

                        for connection in connections {
                            if !self.connections.contains_key(&connection.id()) {
                                events.push(Event::RemoteJoined(connection.address(), connection.id()))
                            }
                            self.connections.insert(connection.id(), connection.clone());
                        }

                    }

                    if self.network_status != NetworkStatus::Connected {
                        self.used_host_migrations.clear();
                        self.network_status = NetworkStatus::Connected;
                        events.push(Event::HostReady(
                            config.remote_addr,
                            self.host_id,
                            self.local.connection().id()
                        ));
                    }

                },

                ClientEvent::Inputs(ticks, inputs) => {

                    // Apply inputs
                    let received_inputs: Vec<ConnectionID> = inputs.into_iter().map(|input| {

                        // Remove confirmed inputs from the local input queue
                        if input.id == self.local.connection().id() && !input.data.is_empty() {
                            self.input_queue = self.input_queue.split_off(input.data.len());
                        }

                        for i in input.data {
                            self.state.apply_input(self.host_id, input.id, i);
                        }

                        input.id

                    }).collect();

                    // Check for any connections which are no longer being send by the host
                    self.connections.retain(|id, conn| {
                        if !received_inputs.contains(id) {
                            events.push(Event::RemoteLeft(conn.address(), *id));
                            false

                        } else {
                            true
                        }
                    });

                    // Tick state
                    let connections = self.connections();
                    for _ in 0..ticks {
                        self.state.tick(t, self.host_id, &connections[..]);
                    }

                }

            }

        }

        Ok(events)

    }

    fn connections(&self) -> Vec<(ConnectionID, SocketAddr)> {
        self.connections.iter().map(|(id, conn)| {
            (*id, conn.address())

        }).collect()
    }

}

// Helpers --------------------------------------------------------------------
pub struct Migration {
    pub previous_host: HostID,
    pub local_host: ConnectionID,
    pub connection: Vec<(ConnectionID, RemoteConnection)>
}

#[derive(Debug, Eq, PartialEq)]
enum NetworkStatus {
    Connecting,
    Reconnecting,
    Connected,
    Disconnected
}

