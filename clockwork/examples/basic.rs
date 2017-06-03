// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Crates ---------------------------------------------------------------------
#[macro_use] extern crate clap;
extern crate rand;
extern crate serde;
extern crate clockwork;
#[macro_use] extern crate serde_derive;


// STD Dependencies -----------------------------------------------------------
use std::thread;
use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};
use std::time::Duration;


// External Dependencies ------------------------------------------------------
use clockwork::{
    Clockwork, Config, ConnectionID, HostID,
    Event, Error,
    State, Input, Options
};


// Internals ------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize)]
struct GameState {
    is_ready: bool,
    tick: u8,
    state: u8
}

impl State<GameOptions, PlayerInput, InitData> for GameState {

    fn is_ready(&self) -> bool {
        self.is_ready
    }

    fn init(&mut self, host_id: HostID, _: &[(ConnectionID, SocketAddr)], _: &mut InitData) {
        println!("[GameState] (Host {:?}) Initialized", host_id);
        self.is_ready = true;
    }

    fn tick(&mut self, _: HostID, _: &[(ConnectionID, SocketAddr)]) {
        println!("[GameState] Tick #{} | {}", self.tick, self.state);
        self.state %= 85;
        self.tick = self.tick.wrapping_add(1);
    }

    fn apply_options(&mut self, host_id: HostID, options: &[(ConnectionID, GameOptions)]) {
        for &(id, ref o) in options {
            if id == host_id && options.len() == o.min_players as usize {
                self.is_ready = true;
                break;
            }
        }
    }

    fn apply_input(&mut self, _: HostID, _: ConnectionID, input: PlayerInput) {
        self.state = self.state.wrapping_add(input.buttons);
    }

}

impl Default for GameState {
    fn default() -> Self {
        Self {
            is_ready: false,
            tick: 0,
            state: 0
        }
    }
}

#[derive(Debug, Hash, Serialize, Deserialize)]
struct GameOptions {
    min_players: u8
}

impl Options for GameOptions {}

impl Default for GameOptions {
    fn default() -> Self {
        Self {
            min_players: 3
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
struct PlayerInput {
    buttons: u8
}

impl Input for PlayerInput {}

impl Default for PlayerInput {
    fn default() -> Self {
        Self {
            buttons: 0
        }
    }
}


pub struct InitData;

// Mainloop -------------------------------------------------------------------
fn main() {

    let args = clap::App::new("clockwork")
        .version(crate_version!())
        .author("Ivo Wetzel <ivo.wetzel@googlemail.com>")
        .about("Clockwork Test Tool")
        .arg(clap::Arg::with_name("port")
            .help("Port to use for client/server communication")
            .index(1)

        ).arg(clap::Arg::with_name("address")
            .short("a")
            .long("addr")
            .takes_value(true)
            .help("Server address to connect to.")

        ).arg(clap::Arg::with_name("min_players")
            .short("m")
            .long("min_players")
            .takes_value(true)
            .help("Required number of clients to start (Default is 2).")

        ).get_matches();

    run(
        value_t!(args.value_of("port"), u16).ok(),
        value_t!(args.value_of("address"), Ipv4Addr).ok(),
        value_t!(args.value_of("min_players"), u8).ok()

    ).expect("Failed to start server / client.");

}

fn run(port: Option<u16>, addr: Option<Ipv4Addr>, min_players: Option<u8>) -> Result<(), Error> {

    let port = port.unwrap_or(7156);
    let remote_addr = addr.unwrap_or_else(|| Ipv4Addr::new(127, 0, 0, 1));

    let config = Config {
        low_tick_rate: 10,
        high_tick_rate: 10,
        server_addr: SocketAddr::V4(
            SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port)
        ),
        remote_addr: SocketAddr::V4(
            SocketAddrV4::new(remote_addr, port)
        ),
        .. Config::default()
    };

    let mut c = Clockwork::<GameState, GameOptions, PlayerInput, InitData>::connect(config, GameState::default())?;
    if addr.is_none() {
        println!("[Info] Starting server on local port {}...", port);
        c = c.with_server()?;
    }

    println!("[Info] Connecting to server at {}:{}...", remote_addr, port);

    let mut time_passed = 0;
    let mut init = InitData;

    'main: loop {
        {

            while let Ok(event) = c.try_recv(&mut init) {
                match event {
                    Event::HostConnect(address, host_id, local_id) => {
                        println!("[Info] Connected to host {:?}({:?}) as {:?}, but not yet ready...", address, host_id, local_id);
                        c.set_options(GameOptions {
                            min_players: min_players.unwrap_or(2)
                        });
                    },
                    Event::HostReady(address, host_id, local_id) => {
                        println!("[Info] Connected to host {:?}({:?}) as {:?}, now ready!", address, host_id, local_id)
                    },
                    Event::HostReconnect(address, host_id, local_id) => {
                        println!("[Info] Reconnecting to host {:?}({:?}) as {:?}...", address, host_id, local_id);
                    },
                    Event::RemoteJoined(address, id) => {
                        println!("[Info] Remote {:?}{:?} joined", address, id);
                    },
                    Event::RemoteOptions => {
                        println!("[Info] Options have changed");
                    },
                    Event::RemoteLeft(address, id) => {
                        println!("[Info] Remote {:?}{:?} left", address, id);
                    },
                    Event::Error(err) => {
                        println!("[Info] Error: {:?}", err);
                        break 'main;
                    }
                }
            }

        }

        if c.state().is_ready() {
            let r: u8 = rand::random();
            if r > 96 {
                c.queue_input(PlayerInput {
                    buttons: rand::random()
                });
            }
        }

        let tick_rate = c.send();
        thread::sleep(Duration::from_millis(1000 / tick_rate));

        time_passed += 1000 / tick_rate;
        if time_passed >= 1000 {
            println!("[Info] One Second passed");
            time_passed -= 1000;
        }

    }

    c.disconnect();

    Ok(())

}

