// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Crates ---------------------------------------------------------------------
extern crate rand;
extern crate serde;
extern crate clockwork;
#[macro_use]
extern crate serde_derive;

extern crate audio;
extern crate tiles;
extern crate client;
extern crate renderer;


// External Dependencies ------------------------------------------------------
use rand::Rng;


// Internal Dependencies ------------------------------------------------------
mod core;
mod game;
use game::Game;
use core::GameOptions;


// Main -----------------------------------------------------------------------
pub fn main() {

    client::start(|config, min_players, client| {
        renderer::run::<Game, _>("RTS", 640, 480, 60, config.high_tick_rate as u32, move |refs| {

            // Create a seed for the RNG
            let mut seed = [0u8; 4];
            rand::thread_rng().fill_bytes(&mut seed[..]);
            println!("Seed is: {:?}", seed);

            let options = GameOptions {
                min_players: min_players,
                random_seed: seed
            };

            Game::new(client, options, refs)

        });

    }).expect("[Game] Failed to start client.");

}

