// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Crates ---------------------------------------------------------------------
extern crate image;
extern crate serde;
extern crate clockwork;
#[macro_use]
extern crate serde_derive;

extern crate gfx;
extern crate gfx_device_gl;

extern crate client;
extern crate renderer;
extern crate tiles;


// Internal Dependencies ------------------------------------------------------
mod core;
mod game;
use game::Game;
use core::GameOptions;


// Main -----------------------------------------------------------------------
pub fn main() {

    client::start(|config, min_players, client| {
        renderer::run::<Game, _>("RTS", 640, 480, 60, config.high_tick_rate as u32, move |refs| {
            let options = GameOptions {
                min_players: min_players
            };
            Game::new(client, options, refs)
        });

    }).expect("[Game] Failed to start client.");

}

