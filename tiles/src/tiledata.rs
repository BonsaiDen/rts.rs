// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::fs::File;
use std::path::Path;


// External Dependencies ------------------------------------------------------
use serde_xml_rs::deserialize;


// Tiledata Abstraction -------------------------------------------------------
#[derive(Debug)]
pub struct TileData {
    pub width: u32,
    pub height: u32,
    pub indices: Vec<u32>
}

impl TileData {

    pub fn new(path: &Path) -> Self {

        let file = File::open(path).expect("TileData: Failed to open map file.");
        let map: Map = deserialize(file).expect("TileData: Failed to parse map file.");
        let base = &map.layer[0];

        Self {
            width: base.width.parse().unwrap_or(0),
            height: base.height.parse().unwrap_or(0),
            indices: base.data.split(',').map(|i| i.trim().parse::<u32>().unwrap_or(1) - 1).collect()
        }

    }

    pub fn set_tile_index(&mut self, x: i32, y: i32, index: u32) -> bool {
        let i = y * self.width as i32 + x;
        if i >= 0 && i < (self.width * self.height) as i32 {
            self.indices[i as usize] = index;
            true

        } else {
            false
        }
    }

    pub fn get_tile_index(&self, x: i32, y: i32) -> Option<u32> {
        let w = self.width as i32;
        let h = self.height as i32;
        if x >= 0 && x < w && y >= 0 && y < h {
            Some(self.indices[(y * w + x) as usize])

        } else {
            None
        }
    }

    pub fn get_offset_border(&self, x: i32, y: i32, distance: i32) -> Vec<Option<(i32, i32)>> {
        let mut offsets = Vec::new();
        for px in x - distance..x + distance + 1 {
            for py in y - distance..y + distance + 1 {
                if px == x - distance || px == x + distance || py == y - distance || py == y + distance  {
                    offsets.push(self.limit_offset(px, py));
                }
            }
        }
        offsets
    }

    pub fn get_offset_area(&self, x: i32, y: i32, distance: i32) -> Vec<Option<(i32, i32)>> {
        let mut offsets = Vec::new();
        for px in x - distance..x + distance + 1 {
            for py in y - distance..y + distance + 1 {
                offsets.push(self.limit_offset(px, py));
            }
        }
        offsets
    }

    pub fn get_surrounding_indices(&self, x: i32, y: i32) -> [u32; 8] {
        [
            self.get_tile_index(x - 1, y - 1).unwrap_or(4096),
            self.get_tile_index(x,     y - 1).unwrap_or(4096),
            self.get_tile_index(x + 1, y - 1).unwrap_or(4096),

            self.get_tile_index(x - 1, y).unwrap_or(4096),

            self.get_tile_index(x + 1, y).unwrap_or(4096),

            self.get_tile_index(x - 1, y + 1).unwrap_or(4096),
            self.get_tile_index(x,     y + 1).unwrap_or(4096),
            self.get_tile_index(x + 1, y + 1).unwrap_or(4096)
        ]
    }

    fn limit_offset(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        let w = self.width as i32;
        let h = self.height as i32;
        if x >= 0 && x < w && y >= 0 && y < h {
            Some((x, y))

        } else {
            None
        }
    }

}

impl Default for TileData {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            indices: Vec::new()
        }
    }
}

#[derive(Debug, Deserialize)]
struct Layer {
    width: String,
    height: String,
    data: String
}

#[derive(Debug, Deserialize)]
struct Map {
    layer: Vec<Layer>
}

