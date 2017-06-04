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
extern crate sprites;
extern crate renderer;
extern crate pathfinding;


// STD Dependencies -----------------------------------------------------------
use std::iter;
use std::time::Instant;
use std::path::{Path, PathBuf};


// External Dependencies ------------------------------------------------------
use rand::Rng;
use audio::AudioQueue;
use sprites::{SpriteSheet, SpriteView, Sprite};
use tiles::{TileData, TileSource, TileGrid, TerrainGrid, TileSet, TileType};
use renderer::{Key, Keyboard, Button, Mouse, Renderable, Encoder};
use pathfinding::astar;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GridCell(i32, i32);


// Example --------------------------------------------------------------------
struct Map {
    terrain_grid: TerrainGrid,
    col_grid: TileGrid<CollisionData>,
    sprite_view: SpriteView,
    audio: AudioQueue,
    scroll: (i32, i32),
    begin: Option<GridCell>,
    path: Vec<Sprite>
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
        self.scroll = self.terrain_grid.scroll_to(self.scroll.0, self.scroll.1);
        self.col_grid.scroll_to(self.scroll.0, self.scroll.1);
        self.sprite_view.scroll_to(self.scroll.0, self.scroll.1);

        if mouse.was_pressed(Button::Left) {

            let (x, y) = mouse.get(Button::Left).position();
            let p = self.terrain_grid.screen_to_grid(x, y);

            if keyboard.is_pressed(Key::B) {
                println!("[Pathfinding] Set beginning");
                self.begin = Some(GridCell(p.0, p.1));

            } else if keyboard.is_pressed(Key::G) {
                println!("[Pathfinding] Set goal");
                let goal = GridCell(p.0, p.1);
                if let Some(ref begin) = self.begin {

                    println!("[Pathfinding] Searching...");

                    let start = Instant::now();
                    let result = astar(begin, |c| {
                        self.col_grid.source().neighbors(c.0, c.1)

                    }, |c| {
                        ((c.0 - goal.0).abs() + (c.1 - goal.1).abs()) as usize

                    }, |c| {
                        *c == goal
                    });

                    println!("[Pathfinding] Completed in 0.{:?}ms", start.elapsed().subsec_nanos() / 100000);
                    if let Some(result) = result {
                        self.path = result.0.into_iter().map(|c| {
                            let mut s = Sprite::new();
                            s.set_position(c.0 as f32 * 32.0, c.1 as f32 * 32.0);
                            s.set_tile(7);
                            s.set_size(32.0, 32.0);
                            s

                        }).collect();
                        println!("[Pathfinding] Path with length {} created", self.path.len());
                    }

                }

            } else if let Some(terrain) = self.terrain_grid.consume_tile(p.0, p.1) {
                self.col_grid.set_tile_index(p.0, p.1, 0);

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
        self.terrain_grid.draw(&mut encoder);
        self.col_grid.draw(&mut encoder);

        for s in &self.path {
            self.sprite_view.draw_sprite(s);
        }

        self.sprite_view.draw(&mut encoder);

    }

}


// Structs --------------------------------------------------------------------
pub struct CollisionData {
    pub width: u32,
    pub height: u32,
    pub indices: Vec<u32>
}

impl CollisionData {

    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width: width,
            height: height,
            indices: iter::repeat(1).take((width * height) as usize).collect()
        }
    }

    pub fn initialize(&mut self, data: &TileData, tileset: &TileSet) {
        for (i, d) in self.indices.iter_mut().zip(data.indices().iter()) {
            *i = match tileset.typ(*d) {
                TileType::Ground => 0,
                TileType::Water => 4,
                TileType::Other => 2
            };
        }
    }

    pub fn neighbors(&self, x: i32, y: i32) -> Vec<(GridCell, usize)> {
        let mut cells = Vec::new();
        for px in x - 1..x + 2 {
            for py in y - 1..y + 2 {
                if px != x || py != y {
                    if self.is_traversable(px, py) {
                        let cost = if (px == x && py != y) || (py == y && px != x) {
                            1
                        } else {
                            2
                        };
                        cells.push((GridCell(px, py), cost));
                    }
                }
            }
        }
        cells
    }

    fn is_traversable(&self, x: i32, y: i32) -> bool {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let i = x + y * self.width as i32;
            self.indices[i as usize] == 0

        } else {
            false
        }
    }

}

impl TileSource for CollisionData {

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn index(&self, index: usize) -> u32 {
        self.indices[index]
    }

    fn indices(&self) -> &[u32] {
        &self.indices
    }

    fn set_tile_index(&mut self, x: i32, y: i32, index: u32) -> bool {
        let i = y * self.width as i32 + x;
        if i >= 0 && i < (self.width * self.height) as i32 {
            self.indices[i as usize] = index;
            true

        } else {
            false
        }
    }

    fn get_tile_index(&self, x: i32, y: i32) -> Option<u32> {
        let w = self.width as i32;
        let h = self.height as i32;
        if x >= 0 && x < w && y >= 0 && y < h {
            Some(self.indices[(y * w + x) as usize])

        } else {
            None
        }
    }

}

impl Default for CollisionData {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            indices: Vec::new()
        }
    }
}


// Demo -----------------------------------------------------------------------
fn main() {
    renderer::run::<Map, _>("Map", 640, 480, 60, 10, |mut target| {

        // Tiles
        let ts = TileSet::new(&mut target.factory, Path::new("../assets/maps/develop.tsx")).unwrap();
        let mut terrain_grid = TerrainGrid::new(&mut target.factory, target.color.clone(), target.width, target.height, 32, ts);

        let m = TileData::new(Path::new("../assets/maps/develop.tmx"));
        terrain_grid.set_source(m);

        // Collision
        let ts = TileSet::new(&mut target.factory, Path::new("../assets/maps/debug.tsx")).unwrap();
        let mut col_grid = TileGrid::new(&mut target.factory, target.color.clone(), target.width, target.height, 32, ts);

        let c = {
            let s = terrain_grid.source();
            CollisionData::new(s.width(), s.height())
        };
        col_grid.set_source(c);

        col_grid.source_mut().initialize(
            terrain_grid.source(),
            terrain_grid.tileset()
        );

        // Sprites
        let sprite_sheet = SpriteSheet::new(&mut target.factory, &Path::new("../assets/textures/debug.png"), 16).unwrap();
        let sprite_view = SpriteView::new(
            &mut target.factory,
            target.color.clone(),
            target.width,
            target.height,
            sprite_sheet,
            128
        );

        Map {
            terrain_grid: terrain_grid,
            col_grid: col_grid,
            sprite_view: sprite_view,
            audio: AudioQueue::new(),
            scroll: (0, 0),
            begin: None,
            path: Vec::new()
        }

    });
}

