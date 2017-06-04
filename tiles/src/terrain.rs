// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Internal Dependencies ------------------------------------------------------
use ::tiledata::TileData;


// Terrain Abstraction --------------------------------------------------------
#[derive(Debug)]
pub struct Terrain {
    pub name: String,
    group: [u32; 13],
    standalone: Vec<u32>,
    reduced: Vec<u32>,
}

impl Terrain {

    pub fn new(
        name: String,
        group: [u32; 13],
        standalone: Vec<u32>,
        reduced: Vec<u32>

    ) -> Self {
        Self {
            name: name,
            group: group,
            standalone: standalone,
            reduced: reduced
        }
    }

    pub fn consume_tile(&self, data: &mut TileData, x: i32, y: i32) -> bool {

        if self.reduced.is_empty() {
            false

        } else if let Some(tile) = data.get_tile_index(x, y) {
            if self.is_group_tile(tile) || self.is_standalone_tile(tile) {

                let tree = self.reduced_tile(x as u32, y as u32);
                data.set_tile_index(x, y, tree);

                // 1. Go outwards and convert all tiles which do not form a
                //    2x2 group into standalone tiles
                let mut distance = 1;
                loop {

                    let mut standalone = 0;
                    for offset in data.get_offset_border(x, y, distance) {

                        if let Some(o) = offset {

                            // Check if surrounding tile is part of terrain too
                            let tile = data.get_tile_index(o.0, o.1).unwrap();
                            if self.is_group_tile(tile) {

                                // Get surrounding tiles
                                let flags = self.to_flags(data.get_surrounding_indices(o.0, o.1));

                                // Convert into standalone
                                if let Some(tile) = self.flags_to_standalone(o.0 as u32, o.1 as u32, flags) {
                                    data.set_tile_index(o.0, o.1, tile);
                                    standalone += 1;
                                }

                            }
                        }

                    }

                    if standalone == 0 {
                        break;

                    } else {
                        distance += 1;
                    }

                }

                // 2. We now reflow all bordering tiles within the adjusted area
                for offset in data.get_offset_area(x, y, distance + 1) {
                    if let Some(o) = offset {
                        let tile = data.get_tile_index(o.0, o.1).unwrap();
                        if self.is_group_tile(tile) {
                            let indices = data.get_surrounding_indices(o.0, o.1);
                            let tile = self.flags_to_grouped(self.to_flags(indices));
                            data.set_tile_index(o.0, o.1, tile);
                        }
                    }
                }

                true

            } else {
                false
            }

        } else {
            false
        }

    }

    pub fn has_tile(&self, tile: u32) -> bool {
        self.is_group_tile(tile)
            || self.is_standalone_tile(tile)
            || self.reduced.iter().any(|v| *v == tile)
    }

    fn is_standalone_tile(&self, tile: u32) -> bool {
        self.standalone.iter().any(|v| *v == tile)
    }

    fn is_group_tile(&self, tile: u32) -> bool {
        self.group.iter().any(|v| *v == tile)
    }

    fn reduced_tile(&self, x: u32, y: u32) -> u32 {
        let index = (x * 233 + y * 107 - y / 3) * 41 % (self.reduced.len() as u32);
        self.reduced[index as usize]
    }

    fn standalone_tile(&self, x: u32, y: u32) -> u32 {
        let index = (x * 67 + y * 293 + x / 5) * 13 % (self.standalone.len() as u32);
        self.standalone[index as usize]
    }

    fn to_flags(&self, tiles: [u32; 8]) -> [bool; 8] {
        let mut flags = [false; 8];
        for i in 0..8 {
            flags[i] = tiles[i] == 4096 || self.is_group_tile(tiles[i])
        }
        flags
    }

    fn flags_to_standalone(&self, x: u32, y: u32, flags: [bool; 8]) -> Option<u32> {

        // Calculate adjacents tile borders
        let mut group = 0;
        if flags[1] {
            group += 1;
        }

        if flags[4] {
            group += 4;
        }

        if flags[6]  {
            group += 16;
        }

        if flags[3] {
            group += 64;
        }

        if flags[2] {
            group += 2;
        }

        if flags[7] {
            group += 8;
        }

        if flags[5] {
            group += 32;
        }

        if flags[0] {
            group += 128;
        }

        // Possible connected groups of 4
        let a = 1 + 2 + 4;
        let b = 4 + 8 + 16;
        let c = 16 + 32 + 64;
        let d = 64 + 128 + 1;

        // We need at least a group of 4 connected tiles...
        if group & a == a || group & b == b || group & c == c || group & d == d  {
            None

        } else {
            Some(self.standalone_tile(x, y))
        }

    }

    fn flags_to_grouped(&self, flags: [bool; 8]) -> u32 {

        let mut border = 0;
        if !flags[1] {
            border += 1;
        }

        if !flags[4] {
            border += 4;
        }

        if !flags[6]  {
            border += 16;
        }

        if !flags[3] {
            border += 64;
        }

        if !flags[2] {
            border += 2;
        }

        if !flags[7] {
            border += 8;
        }

        if !flags[5] {
            border += 32;
        }

        if !flags[0] {
            border += 128;
        }

        // Diagonal top right
        if border & (1 + 4) == 1 + 4 {
            self.group[2]

        // Diagonal bottom right
        } else if border & (4 + 16) == 4 + 16 {
            self.group[8]

        // Diagonal bottom left
        } else if border & (16 + 64) == 16 + 64 {
            self.group[6]

        // Diagonal top right
        } else if border & (64 + 1) == 64 + 1 {
            self.group[0]

        // Top
        } else if border & 1 == 1 {
            self.group[1]

        // Right
        } else if border & 4 == 4 {
            self.group[5]

        // Bottom
        } else if border & 16 == 16 {
            self.group[7]

        // Left
        } else if border & 64 == 64 {
            self.group[3]

        // Center fallback
        } else {
            self.group[4]
        }

    }

}

