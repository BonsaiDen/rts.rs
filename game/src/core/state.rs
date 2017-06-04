// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::net::SocketAddr;
use std::path::{Path, PathBuf};


// External Dependencies ------------------------------------------------------
use rand::{XorShiftRng, SeedableRng, Rng};
use audio::AudioQueue;
use renderer::RenderTarget;
use tiles::{TileData, TileGrid, TileSet};
use clockwork::{ConnectionID, HostID, State};


// Internal Dependencies ------------------------------------------------------
use core::{GameInput, GameOptions};


// Game State Abstraction -----------------------------------------------------
pub struct GameState {
    is_ready: bool,
    options: GameOptions,
    rng: XorShiftRng,
    audio: AudioQueue,
    pub tile_grid: Option<TileGrid>
}

impl State<GameOptions, GameInput, RenderTarget> for GameState {

    fn is_ready(&self) -> bool {
        self.is_ready
    }

    fn init(&mut self, host_id: HostID, _: &[(ConnectionID, SocketAddr)], target: &mut RenderTarget) {

        // Seed RNG
        println!("[GameState] (Host {:?}) Seeding rng with {:?}", host_id, self.options.random_seed);
        self.rng.reseed([
            self.options.random_seed[0] as u32,
            self.options.random_seed[1] as u32,
            self.options.random_seed[2] as u32,
            self.options.random_seed[3] as u32
        ]);

        // Setup Map rendering
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
                println!("take options {:?}", options);
                self.options = o.clone();
                break;
            }
        }
    }

    fn apply_input(&mut self, _: HostID, id: ConnectionID, input: GameInput) {
        println!("[GameState] [Input] [#{:?}] {:?}", id, input);
        match input {
            GameInput::LeftClick(x, y) => self.consume_tile(x as i32, y as i32),
            GameInput::Idle => {}
        }
    }

}

impl GameState {

    fn consume_tile(&mut self, x: i32, y: i32) {

        let effect = if let Some(ref mut tile_grid) = self.tile_grid {
            if let Some(terrain) = tile_grid.consume_tile(x, y) {
                if terrain.name == "Forest" {
                    Some(PathBuf::from("../assets/sounds/woodaxe.flac"))

                } else if terrain.name == "Rocks" {
                    Some(PathBuf::from("../assets/sounds/pickaxe.flac"))

                } else {
                    None
                }

            } else {
                None
            }

        } else {
            None
        };

        if let Some(effect) = effect {
            self.play_effect_at(x, y, effect, true);
        }

    }

    fn play_effect_at(&mut self, tx: i32, ty: i32, path: PathBuf, vary_speed: bool) {

        // DE-SYNC: Must always be called
        let speed: Option<f32> = if vary_speed {
            Some(self.rng.gen_range(0.8, 1.0))

        } else {
            None
        };

        // Only play effect when it is within the screen bounds
        if let Some(ref tile_grid) = self.tile_grid {
            if tile_grid.tile_within_screen_grid(tx, ty, 1) {
                self.audio.play_effect(path, speed);
            }
        }

    }

}


impl Default for GameState {
    fn default() -> Self {
        Self {
            is_ready: false,
            options: GameOptions::default(),
            rng: XorShiftRng::new_unseeded(),
            audio: AudioQueue::new(),
            tile_grid: None
        }
    }
}

