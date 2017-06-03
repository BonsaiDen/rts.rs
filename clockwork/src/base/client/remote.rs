// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::time::Instant;
use std::net::SocketAddr;
use std::marker::PhantomData;


// External Dependencies ------------------------------------------------------
use bincode::{serialize, deserialize, Infinite};


// Internal Dependencies ------------------------------------------------------
use base::client::ClientTick;
use base::server::ServerTick;

use ::{Config, ConnectionID, State, Options, Input};
use base::{RemoteInput, RemoteConnection, RemoteOptions, base_packet};

pub enum ClientEvent<O, I> {
    Connected(ConnectionID),
    Options(Vec<RemoteOptions<O>>),
    Ready(Option<Vec<RemoteConnection>>),
    Inputs(u8, Vec<RemoteInput<I>>)
}


/// Client side remote abstraction.
pub struct ClientRemote<S, O, I, R> {

    /// Current client clock tick.
    tick: ClientTick,

    /// The local sequence number for InputStates.
    sequence: u8,

    /// Last received options from the remote.
    options: Option<RemoteOptions<O>>,

    /// Connection information of the remote.
    connection: RemoteConnection,

    /// Last time data was received from this remote.
    last_receive_time: Instant,

    /// State implementation type.
    state: PhantomData<S>,

    /// Input implementation type.
    input: PhantomData<I>,

    /// Refs implementation type.
    refs: PhantomData<R>

}

impl<S, O, I, R> ClientRemote<S, O, I, R> where S: State<O, I, R>, O: Options, I: Input {

    pub fn new(id: ConnectionID, address: SocketAddr) -> Self {
        Self {
            tick: ClientTick::default(),
            sequence: 0,
            options: None,
            connection: RemoteConnection::new(id, address, 0),
            last_receive_time: Instant::now(),
            state: PhantomData,
            input: PhantomData,
            refs: PhantomData
        }
    }

    pub fn connection(&self) -> &RemoteConnection {
        &self.connection
    }

    pub fn timed_out(&self, config: &Config) -> bool {
        self.last_receive_time.elapsed() > config.remote_timeout_threshold
    }

    pub fn disconnected(&self) -> bool {
        // TODO also implement clean disconnect / shutdown
        false
    }

    pub fn set_tick(&mut self, tick: ClientTick) {
        self.tick = tick;
    }

    pub fn set_options(&mut self, options: O) {
        self.options = Some(RemoteOptions::new(
            self.connection.id(),
            options
        ));
    }

    pub fn reset(&mut self) {
        self.tick = ClientTick::default();
        self.last_receive_time = Instant::now();
    }

    pub fn send(&self, config: &Config, inputs: Vec<I>) -> Vec<u8> {

        let mut packet = base_packet(self.connection.id(), config, self.tick as u8);
        match self.tick {
            ClientTick::SendOptions => if let Some(ref options) = self.options {
                if let Ok(mut bytes) = serialize(options, Infinite) {
                    packet.append(&mut bytes);
                }
            },
            ClientTick::SendInput => {

                let input = RemoteInput::new(
                    self.connection.id(),
                    self.sequence,
                    inputs
                );

                packet.push(self.sequence);

                if let Ok(mut bytes) = serialize(&input, Infinite) {
                    packet.append(&mut bytes);
                }

            },
            _ => {}
        }

        packet

    }

    pub fn receive(
        &mut self,
        id: ConnectionID,
        packet: Vec<u8>

    ) -> Option<Vec<ClientEvent<O, I>>> {

        if id != self.connection.id() || packet.len() < 9 {
            return None;
        }

        // TODO move down
        self.last_receive_time = Instant::now();

        // TODO optimize return structure
        match (self.tick, ServerTick::from_u8(packet[8])) {

            (ClientTick::WaitForServer, ServerTick::Migrate) => if packet.len() >= 10 {
                if let Ok(id) = deserialize::<ConnectionID>(&packet[9..]) {
                    self.sequence = 0;
                    self.tick = ClientTick::Ready;
                    return Some(vec![
                        ClientEvent::Connected(id)
                    ]);
                }
            },

            (ClientTick::WaitForServer, ServerTick::WaitForClients) => if packet.len() >= 10 {
                if let Ok(id) = deserialize::<ConnectionID>(&packet[9..]) {
                    self.sequence = 0;
                    self.tick = ClientTick::SendOptions;
                    return Some(vec![
                        ClientEvent::Connected(id)
                    ]);
                }
            },

            // Waiting for server to drop any failed connections before migrating
            (ClientTick::WaitForServer, ServerTick::InitializeMigrate) |

            // Waiting for server to confirm our sent options
            (ClientTick::SendOptions, ServerTick::AwaitOptions) |

            // Server is still waiting for options from other clients
            (ClientTick::SendOptions, ServerTick::WaitForClients) |

            // wait for server to confirm migration
            (ClientTick::Ready, ServerTick::Migrate) |

            // Waiting for server to accepts inputs
            (ClientTick::SyncConfirm, ServerTick::InitializeMigrate) => {
            },

            (ClientTick::SendOptions, ServerTick::ConfirmOptions) => if packet.len() >= 10 {

                let count = packet[9] as usize;
                let option_size = packet.len() - 10;
                if option_size > 0 {
                    let bytes_per_option = option_size / count;

                    let mut options: Vec<RemoteOptions<O>> = Vec::with_capacity(count);
                    for i in 0..count {
                        let offset = 10 + i * bytes_per_option;
                        // TODO verify length
                        let bytes = &packet[offset..offset + bytes_per_option];
                        if let Ok(option) = deserialize::<RemoteOptions<O>>(bytes) {
                            options.push(option);
                        }
                    }

                    return Some(vec![
                        ClientEvent::Options(options)
                    ]);
                }

            },

            (ClientTick::Ready, ServerTick::Initialize) => if packet.len() >= 10 {

                self.tick = ClientTick::SyncConfirm;

                // Calculate state size
                let count = packet[9] as usize;
                let conn_bytes = packet.len() - 10;
                let bytes_per_conn = conn_bytes / count;

                let mut connections: Vec<RemoteConnection> = Vec::with_capacity(count);
                for i in 0..count {
                    let offset = 10 + i * bytes_per_conn;
                    // TODO verify length
                    let bytes = &packet[offset..offset + bytes_per_conn];
                    if let Ok(state) = deserialize::<RemoteConnection>(bytes) {
                        connections.push(state);
                    }
                }

                return Some(vec![
                    ClientEvent::Ready(Some(connections))
                ]);

            },

            (ClientTick::Ready, ServerTick::InitializeMigrate) => {
                self.tick = ClientTick::SyncConfirm;
                return Some(vec![
                    ClientEvent::Ready(None)
                ]);
            }
            (ClientTick::SyncConfirm, ServerTick::AwaitInput) => {
                self.tick = ClientTick::SendInput;
            },
            (ClientTick::SendInput, ServerTick::AwaitInput) => if packet.len() >= 12 {

                let next_sequence = packet[9];
                let sequence_ticks = packet[10];

                let count = packet[11] as usize;
                if count > 0 {

                    let mut inputs: Vec<RemoteInput<I>> = Vec::with_capacity(count);
                    let mut offset = 12;
                    for _ in 0..count {

                        let length = packet[offset] as usize;
                        let bytes = &packet[offset + 1..offset + length + 1];

                        if let Ok(input) = deserialize::<RemoteInput<I>>(bytes) {
                            // Skip any mismatching sequences
                            if input.sequence == self.sequence {
                                inputs.push(input);
                            }
                        }

                        offset += length + 1;

                    }

                    // We only every send our next sequence number after we
                    // received the full server inputs for the current one
                    if inputs.len() == count && next_sequence == self.sequence.wrapping_add(1) {
                        self.sequence = self.sequence.wrapping_add(1);
                        return Some(vec![
                            ClientEvent::Inputs(sequence_ticks, inputs)
                        ]);
                    }

                }

            },
            (_, _) => {
                println!("Unknown packet {:?}/{:?}", self.tick, ServerTick::from_u8(packet[8]));
                return None;
            }
        }

        None

    }

}

