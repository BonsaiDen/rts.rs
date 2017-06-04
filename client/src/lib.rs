// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Crates ---------------------------------------------------------------------
#[macro_use]
extern crate clap;
extern crate clockwork;

// STD Dependencies -----------------------------------------------------------
use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};


// External Dependencies ------------------------------------------------------
use clockwork::{Clockwork, Config, Error, State, Options, Input};


// Public Interface -----------------------------------------------------------
pub fn start<S, O, I, R, C: Fn(Config, u8, Clockwork<S, O, I, R>)>(
    callback: C

) -> Result<(), Error> where S: State<O, I, R> + 'static,
                             O: Options + 'static,
                             I: Input + 'static {

    let args = clap::App::new("stratland")
        .version(crate_version!())
        .author("Ivo Wetzel <ivo.wetzel@googlemail.com>")
        .about("Stratland strategy game")
        .arg(clap::Arg::with_name("port")
            .help("Port to use for client / server communication.")
            .index(1)

        ).arg(clap::Arg::with_name("address")
            .short("a")
            .long("addr")
            .takes_value(true)
            .help("Server address for client to connect to.")

        ).arg(clap::Arg::with_name("min_players")
            .short("m")
            .long("min_players")
            .takes_value(true)
            .help("Required number of clients to start the game (Default is 2).")

        ).get_matches();

    run(
        value_t!(args.value_of("port"), u16).unwrap_or(28768),
        value_t!(args.value_of("address"), Ipv4Addr).ok(),
        value_t!(args.value_of("min_players"), u8).ok().unwrap_or(1),
        callback
    )

}

pub fn run<S, O, I, R, C: Fn(Config, u8, Clockwork<S, O, I, R>)> (
    port: u16,
    addr: Option<Ipv4Addr>,
    min_players: u8,
    callback: C

) -> Result<(), Error> where S: State<O, I, R> + 'static,
                             O: Options + 'static,
                             I: Input + 'static {

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

    let mut client = Clockwork::<S, O, I, R>::connect(config)?;
    if addr.is_none() {
        println!("[Client] [Network] Starting server on local port {}...", port);
        client = client.with_server()?;
    }

    println!("[Client] [Network] Connecting to server at {}:{}...", remote_addr, port);

    callback(config, min_players, client);

    Ok(())

}

