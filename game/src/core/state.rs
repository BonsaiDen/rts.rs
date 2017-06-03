// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::path::Path;
use std::net::SocketAddr;


// External Dependencies ------------------------------------------------------
use clockwork::{ConnectionID, HostID, State};
use renderer::RenderTarget;
use tiles::{TileData, TileGrid, TileSet};


// Internal Dependencies ------------------------------------------------------
use core::{GameInput, GameOptions};


// Game State Abstraction -----------------------------------------------------
pub struct GameState {
    is_ready: bool,
    options: GameOptions,
    pub tile_grid: Option<TileGrid>
}

impl State<GameOptions, GameInput, RenderTarget> for GameState {

    fn is_ready(&self) -> bool {
        self.is_ready
    }

    fn init(&mut self, host_id: HostID, _: &[(ConnectionID, SocketAddr)], target: &mut RenderTarget) {

        println!("[GameState] (Host {:?}) Loading map...", host_id);
        let ts = TileSet::new(&mut target.factory, Path::new("../assets/maps/develop.tsx")).unwrap();
        let mut tile_grid = TileGrid::new(
            &mut target.factory,
            target.color.clone(),
            target.width,
            target.height,
            32,
            ts
        );

        let m = TileData::new(Path::new("../assets/maps/develop.tmx"));
        tile_grid.set_tiledata(m);

        self.tile_grid = Some(tile_grid);

        println!("[GameState] (Host {:?}) Initialized", host_id);
        self.is_ready = true;

    }

    fn tick(&mut self, _: HostID, _: &[(ConnectionID, SocketAddr)]) {
        println!("[GameState] Tick");
    }

    fn apply_options(&mut self, host_id: HostID, options: &[(ConnectionID, GameOptions)]) {
        for &(id, ref o) in options {
            if id == host_id && options.len() == o.min_players as usize {
                self.is_ready = true;
                self.options = o.clone();
                break;
            }
        }
    }

    fn apply_input(&mut self, _: HostID, id: ConnectionID, input: GameInput) {
        println!("[GameState] [Input] [#{:?}] {:?}", id, input);
        if let Some(ref mut tile_grid) = self.tile_grid {
            match input {
                GameInput::LeftClick(x, y) => {
                    tile_grid.consume_tile(x as i32, y as i32);
                },
                GameInput::Idle => {}
            }
        }
    }

}


impl Default for GameState {
    fn default() -> Self {
        Self {
            is_ready: false,
            options: GameOptions::default(),
            tile_grid: None
        }
    }
}

