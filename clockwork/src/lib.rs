// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Crates ---------------------------------------------------------------------
extern crate rand;
extern crate serde;
extern crate bincode;
#[macro_use] extern crate serde_derive;


// STD Dependencies -----------------------------------------------------------
use std::io;
use std::time::Duration;
use std::net::SocketAddr;
use std::marker::PhantomData;
use std::collections::VecDeque;
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::thread::{self, JoinHandle};


// Modules --------------------------------------------------------------------
mod base;
mod config;
mod traits;
mod socket;


// Internal Dependencies ------------------------------------------------------
use base::{ClockworkClient, ClockworkServer, Migration};


// Re-Exports -----------------------------------------------------------------
pub use config::Config;
pub use base::{ConnectionID, HostID};
pub use traits::{State, Input, Options};

/// Enumeration of all possible clockwork client events.
pub enum Event {
    HostConnect(SocketAddr, HostID, ConnectionID),
    HostReady(SocketAddr, HostID, ConnectionID),
    HostReconnect(SocketAddr, HostID, ConnectionID),
    RemoteJoined(SocketAddr, ConnectionID),
    RemoteOptions,
    RemoteLeft(SocketAddr, ConnectionID),
    // TODO Waiting For Player Event in case a player is having connection problems
    // TODO Server needs to handle that by sending the event as long as the issues is in progress:
    // Also drop to the low tick rate during that time?
    // TODO "Resume" event after connection problems have been fixed
    Error(Error)
}


/// Enumeration of all possible clockwork client errors.
#[derive(Debug)]
pub enum Error {
    RemoteTimeout,
    Disconnected,
    Io(io::Error)
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}


/// A server / client lockstep protocol implementation with automatic host
/// migration.
pub struct Clockwork<S, O, I, R> {

    /// Network configuration.
    config: Config,

    /// The clockwork client.
    client: ClockworkClient<S, O, I, R>,

    /// The clockwork server's join handle, if applicable.
    server: Option<(Sender<()>, JoinHandle<()>)>,

    /// Internal queue of the last received events.
    events: VecDeque<Event>,

    /// Internal mode toggle for event queueing.
    mode: Mode,

    /// Reference type
    refs: PhantomData<R>

}

impl<S, O, I, R> Clockwork<S, O, I, R> where S: State<O, I, R> + 'static,
                                             O: Options + 'static,
                                             I: Input + 'static {

    pub fn connect(config: Config) -> Result<Self, Error> {
        Ok(Self {
            config: config,
            client: {
                ClockworkClient::new(&config)?
            },
            server: None,
            events: VecDeque::new(),
            mode: Mode::Receive,
            refs: PhantomData
        })
    }

    pub fn state(&mut self) -> &mut S {
        self.client.state()
    }

    pub fn set_options(&mut self, options: O) {
        self.client.set_options(options);
    }

    pub fn queue_input(&mut self, input: I) {
        self.client.queue_input(input);
    }

    pub fn try_recv(&mut self, t: u64, refs: &mut R) -> Result<Event, TryRecvError> {

        // Initial receive after last send call
        if self.mode == Mode::Receive {
            let config = &self.config;
            match self.client.receive(config, t, refs) {
                Ok(events) => {
                    for event in events {
                        self.events.push_back(event);
                    }
                },
                Err(err) => self.events.push_back(Event::Error(err))
            }
            self.mode = Mode::Send;
        }

        // Return received events one by one
        if let Some(mut event) = self.events.pop_front() {

            // In case of a timeout...
            if let Event::Error(Error::RemoteTimeout) = event {

                // Get the next host in line...
                match self.client.migrate() {

                    // If it's us, start a local server and point ourselfs to
                    // our localhost
                    Some((None, host_id, local_id)) => {

                        // Bootstrap the local server with the last known state
                        if let Ok(server) = create_server::<O, I>(
                            self.config,
                            self.client.migration(),
                            true

                        ) {
                            self.server = Some(server);
                            self.config.remote_addr = self.config.local_server_addr();
                            self.client.reconnect();
                            event = Event::HostReconnect(
                                self.config.local_server_addr(),
                                host_id,
                                local_id
                            );
                        }

                    },

                    // If it's another client, try to connect to that one instead
                    Some((Some(address), host_id, local_id)) => {
                        self.config.remote_addr = self.config.remote_host_addr(address);
                        self.client.reconnect();
                        event = Event::HostReconnect(
                            self.config.remote_addr,
                            host_id,
                            local_id
                        );
                    },

                    // If there are no further hosts left, exit
                    _ => {}
                }

            }

            Ok(event)

        } else {
            Err(TryRecvError::Empty)
        }

    }

    pub fn send(&mut self) -> u64 {
        if self.mode == Mode::Send {
            let config = &self.config;
            self.mode = Mode::Receive;
            self.client.send(config)

        } else {
            self.config.high_tick_rate
        }
    }

    pub fn disconnect(&mut self) {
        if let Some((sender, server)) = self.server.take() {
            sender.send(()).ok();
            server.join().ok();
        }
    }

    pub fn with_server(mut self) -> Result<Self, Error> {
        self.server = Some(create_server::<O, I>(
            self.config,
            self.client.migration(),
            false
        )?);
        Ok(self)
    }

}


// Helpers --------------------------------------------------------------------
#[derive(Eq, PartialEq)]
enum Mode {
    Receive,
    Send
}

fn create_server<O, I>(
    config: Config,
    migration: Migration,
    was_started: bool

) -> Result<(Sender<()>, JoinHandle<()>), Error> where O: Options + 'static,
                                                       I: Input + 'static

{

    let mut server = ClockworkServer::<O, I>::new(
        &config,
        migration.local_host,
        was_started

    )?.with_migrated_states(migration.previous_host, migration.connection);

    let (sender, receiver) = channel::<()>();
    Ok((sender, thread::spawn(move || {
        loop {

            if receiver.try_recv().is_ok() {
                break;
            }

            let tick_rate = server.tick(&config);
            thread::sleep(Duration::from_millis(1000 / tick_rate));

        }
    })))

}

