// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Crates ---------------------------------------------------------------------
extern crate rand;
extern crate audio;
extern crate tiles;
extern crate renderer;


// STD Dependencies -----------------------------------------------------------
use std::path::{Path, PathBuf};


// External Dependencies ------------------------------------------------------
use rand::Rng;
use audio::AudioQueue;
use tiles::{TileData, TileGrid, TileSet};
use renderer::{Key, Keyboard, Button, Mouse, Renderable, Encoder};


// Example --------------------------------------------------------------------
struct Map {
    tile_grid: TileGrid,
    audio: AudioQueue,
    scroll: (i32, i32)
}

impl Renderable for Map {

    fn tick(&mut self) where Self: Sized {

    }

    fn draw(&mut self, mut encoder: &mut Encoder, keyboard: &Keyboard, mouse: &Mouse) where Self: Sized {

        // Scrolling
        if keyboard.is_pressed(Key::A) {
            self.scroll.0 -= 12;
        }

        if keyboard.is_pressed(Key::D) {
            self.scroll.0 += 12;
        }

        if keyboard.is_pressed(Key::W) {
            self.scroll.1 -= 12;
        }

        if keyboard.is_pressed(Key::S) {
            self.scroll.1 += 12;
        }

        // TODO limit diagonal scroll speed
        self.scroll = self.tile_grid.scroll_to(self.scroll.0, self.scroll.1);

        if mouse.was_pressed(Button::Left) {
            let (x, y) = mouse.get(Button::Left).position();
            let p = self.tile_grid.screen_to_grid(x, y);


            if let Some(terrain) = self.tile_grid.consume_tile(p.0, p.1) {
                let mut rng = rand::thread_rng();
                let speed: f32 = rng.gen_range(0.8, 1.0);
                if terrain.name == "Forest" {
                    self.audio.play_effect(PathBuf::from("../assets/sounds/woodaxe.flac"), Some(speed));

                } else if terrain.name == "Rocks" {
                    self.audio.play_effect(PathBuf::from("../assets/sounds/pickaxe.flac"), Some(speed));
                }
            }

        }

        // Draw test map
        self.tile_grid.draw(&mut encoder);

    }

}

// Demo -----------------------------------------------------------------------
fn main() {
    renderer::run::<Map, _>("Map", 640, 480, 60, 10, |mut target| {

        let ts = TileSet::new(&mut target.factory, Path::new("../assets/maps/develop.tsx")).unwrap();
        let mut tile_grid = TileGrid::new(&mut target.factory, target.color, target.width, target.height, 32, ts);

        let m = TileData::new(Path::new("../assets/maps/develop.tmx"));
        tile_grid.set_tiledata(m);

        Map {
            tile_grid: tile_grid,
            audio: AudioQueue::new(),
            scroll: (0, 0)
        }

    });
}

