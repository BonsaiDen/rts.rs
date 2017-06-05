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


// Example --------------------------------------------------------------------
struct Map {
    terrain_grid: TerrainGrid,
    col_grid: TileGrid<CollisionData>,
    sprite_view: SpriteView,
    audio: AudioQueue,
    scroll: (i32, i32),
    units: Vec<Unit>
}

impl Renderable for Map {

    fn tick(&mut self, time: u64) where Self: Sized {
        for unit in &mut self.units {
            unit.tick(time);
        }
    }

    fn draw(&mut self, time: u64, mut encoder: &mut Encoder, keyboard: &Keyboard, mouse: &Mouse) where Self: Sized {

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

        // Terrain Testing and Unit selection
        if mouse.was_pressed(Button::Left) {

            let (x, y) = mouse.get(Button::Left).position();
            let p = self.terrain_grid.screen_to_grid(x, y);

            if keyboard.is_pressed(Key::C) {
                if let Some(terrain) = self.terrain_grid.consume_tile(p.0, p.1) {
                    self.col_grid.set_tile_index(p.0, p.1, 0);

                    let mut rng = rand::thread_rng();
                    let speed: f32 = rng.gen_range(0.8, 1.0);
                    if terrain.name == "Forest" {
                        self.audio.play_effect(PathBuf::from("../assets/sounds/woodaxe.flac"), Some(speed));

                    } else if terrain.name == "Rocks" {
                        self.audio.play_effect(PathBuf::from("../assets/sounds/pickaxe.flac"), Some(speed));
                    }
                }

            } else {

                let (x, y) = ((x + self.scroll.0) as f32, (y + self.scroll.1) as f32);

                let mut any = false;
                for unit in &mut self.units {
                    unit.set_selected(false);
                    if unit.is_hit(x, y) && !any {
                        unit.set_selected(true);
                        any = true;
                    }
                }

            }

        }

        // Unit move testing
        if mouse.was_pressed(Button::Right) {
            let (x, y) = mouse.get(Button::Right).position();
            let p = self.terrain_grid.screen_to_grid(x, y);

            // TODO generate commands and execute these in the actual game
            // Input::CommandUnitMove(x, y)
            // Input::CommandUnitHarvest(x, y)
            // Input::CommandUnitAttackUnit(x, y) TODO must be able to follow the other unit
            // Input::CommandUnitAttackArea(x, y)
            // TODO check if another unit was clicked
            for unit in &mut self.units {
                if unit.is_selected() {
                    unit.move_to(self.col_grid.source(), p.0, p.1);
                }
            }

        }


        // Draw test map
        self.terrain_grid.draw(&mut encoder);
        //self.col_grid.draw(&mut encoder);

        for unit in &mut self.units {
            unit.draw(time, &mut self.sprite_view);
        }

        self.sprite_view.draw(&mut encoder);

    }

}

pub struct Unit {
    sprite: Sprite,
    selected: bool,
    path: Vec<GridCell>,
    origin: (f32, f32),
    origin_time: u64,
    target: Option<GridCell>,
    target_time: u64,
    move_ticks: usize
}

impl Unit {

    pub fn new(x: f32, y: f32) -> Self {

        let mut sprite = Sprite::new();
        sprite.set_tile_size(1, 1);
        sprite.set_size(32.0, 32.0);
        sprite.set_position(x, y);
        sprite.set_tile(8);

        Self {
            sprite: sprite,
            selected: false,
            path: Vec::new(),
            origin: (x, y),
            origin_time: 0,
            target: None,
            target_time: 0,
            move_ticks: 0
        }

    }

    pub fn tick(&mut self, time: u64) {

        // TODO use a state machine for movement, following, harvesting, attacking etc.
        // TODO movement is done as long there is a target(tile)

        // TODO 1. harvest must go to the last known source location for the specified resource
        // TODO initially this is set by right clicking on a resource
            // TODO 2. the unit must find a nearby resource tile within a radius of X around the
            // source location
            // TODO 2a. resource tile is found
                // TODO the resource node is created upon the first gathering tick on a resource tile
                // TODO every X ticks a certain amount from the node is transferred to the unit
                    // TODO if the units bucket is full go to 3.
                    // TODO if the resource node is exhausted, repeat from 2

            // TODO 2b. no resource tile is found go to 3.

            // TODO 3. return to the nearest headquarter
                // TODO once there offload the resourced and return to 1.
                // TODO if no path to a resource tile specified in 1. is found stay at the
                // headquarter and exit gathering mode


        if self.move_ticks == 0 {

            if let Some(target) = self.target.take() {
                self.sprite.set_position(target.0 as f32 * 32.0, target.1 as f32 * 32.0);
            }

            // TODO periodically re-evaluate the path to the current target
            // TODO when following re-evaluate based on remaining distance
            // TODO the shorter the distance the more often we should re-evaluate
            if !self.path.is_empty() {
                self.origin_time = time;
                self.origin = self.sprite.position();
                self.target = self.path.pop();

                if let Some(ref target) = self.target {
                    let (tx, ty) = (target.0 as f32 * 32.0, target.1 as f32 * 32.0);
                    let (dx, dy) = (tx - self.origin.0, ty - self.origin.1);

                    let d = (dx * dx + dy * dy).sqrt();
                    let duration = (500.0 / 32.0 * d) as u64;

                    self.target_time = time + duration;
                    self.move_ticks = ((duration / 100) as usize).saturating_sub(1);

                }
            }

        } else {
            self.move_ticks -= 1;
        }
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    pub fn set_selected(&mut self, active: bool) {
        self.selected = active;
    }

    pub fn is_hit(&self, x: f32, y: f32) -> bool {
        self.sprite.hit(x, y)
    }

    pub fn move_to(&mut self, collision: &CollisionData, x: i32, y: i32) {

        let goal = GridCell(x, y);
        let (sx, sy) = self.sprite.position();
        let begin = GridCell(sx as i32 / 32, sy as i32 / 32);

        println!("[Pathfinding] Searching...");

        let start = Instant::now();
        let result = astar(&begin, |c| {
            collision.neighbors(c.0, c.1)

        }, |c| {
            // TODO use sqrt instead of manhatten?
            ((c.0 - goal.0).abs() + (c.1 - goal.1).abs()) as usize

        }, |c| {
            *c == goal
        });

        println!("[Pathfinding] Completed in 0.{:?}ms", start.elapsed().subsec_nanos() / 100000);
        if let Some(result) = result {
            self.path = result.0.into_iter().skip(1).take_while(|c| collision.is_traversable(c.0, c.1)).collect();
            self.path.reverse();
            println!("[Pathfinding] Path with length {} created", self.path.len());
        }

    }

    pub fn draw(&mut self, time: u64, view: &mut SpriteView) {

        if let Some(ref target) = self.target {
            let d = self.target_time - self.origin_time;
            let r = self.target_time.saturating_sub(time);

            // TODO remove floats or make sure to clip to
            let lerp = (1.0 - 1.0 / d as f64 * r as f64) as f32;

            let (tx, ty) = (target.0 as f32 * 32.0, target.1 as f32 * 32.0);
            let (dx, dy) = (tx - self.origin.0, ty - self.origin.1);

            self.sprite.set_position(
                self.origin.0 + dx * lerp,
                self.origin.1 + dy * lerp
            );

        }

        let mut s = Sprite::new();
        for p in &self.path {
            s.set_position(p.0 as f32 * 32.0, p.1 as f32 * 32.0);
            s.set_tile(7);
            s.set_size(32.0, 32.0);
            view.draw_sprite(&s);
        }

        if self.selected {
            let (x, y) = self.sprite.position();
            let mut selection = Sprite::new();
            selection.set_tile(9);
            selection.set_size(36.0, 36.0);
            selection.set_position(x - 2.0, y - 2.0);
            view.draw_sprite(&selection);
        }

        view.draw_sprite(&self.sprite);

    }

}


// Collision Structs ----------------------------------------------------------

// TODO clean up and integrate into tiles

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GridCell(i32, i32);


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
                        cells.push((GridCell(px, py), self.distance(x, y, px, py)));

                    } else {
                        cells.push((GridCell(px, py), 4096));
                    }
                }
            }
        }
        cells
    }

    pub fn distance(&self, x: i32, y: i32, tx: i32, ty: i32) -> usize {
        if (tx == x && ty != y) || (ty == y && tx != x) {
            1

        } else {
            2
        }
    }

    pub fn is_traversable(&self, x: i32, y: i32) -> bool {
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

        let units = vec![
            Unit::new(128.0, 128.0),
            Unit::new(128.0, 160.0)
        ];

        Map {
            terrain_grid: terrain_grid,
            col_grid: col_grid,
            sprite_view: sprite_view,
            audio: AudioQueue::new(),
            scroll: (0, 0),
            units: units
        }

    });
}

