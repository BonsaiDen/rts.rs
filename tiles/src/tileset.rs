// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::fs::File;
use std::path::Path;
use std::error::Error;
use std::collections::HashMap;


// External Dependencies ------------------------------------------------------
use gfx_device_gl;
use serde_xml_rs::deserialize;


// Internal Dependencies ------------------------------------------------------
use ::texture::Texture;
use ::terrain::Terrain;


// Tileset Abstraction --------------------------------------------------------
#[derive(Debug)]
pub struct TileSet {
    cols: u32,
    rows: u32,
    texture: Texture,
    terrains: HashMap<String, Terrain>
}

impl TileSet {

    pub fn new(factory: &mut gfx_device_gl::Factory, path: &Path) -> Result<Self, Box<Error>> {

        let file = File::open(path)?;
        let set: Set = deserialize(file)?;
        let tex_path = path.canonicalize()?
                           .parent().unwrap()
                           .join(Path::new(&set.image.source))
                           .canonicalize()?;

        let texture = Texture::new(factory, tex_path.as_path())?;
        let size = texture.size();

        // Extract terrains from tiles
        let mut tiles: Vec<(u32, u32, u32)> = set.tile.into_iter().filter_map(|t| {
            if let Ok(tile_index) = t.id.parse::<u32>() {

                // Parse edge data
                let edges: Vec<Option<u32>> = t.terrain.split(",").map(|e| {
                    e.parse::<u32>().ok()

                }).collect();

                // Extract terrain index from edges
                if let Some(terrain_index) = edges.clone().into_iter().filter_map(|e| e).min() {

                    // Construct tile index from edge data
                    let terrain_offset = edges.into_iter().enumerate().map(|(index, edge)| {
                        edge.map(|_| 1 << index).unwrap_or(0) as u32

                    }).sum();

                    Some((terrain_index, terrain_offset, tile_index))

                } else {
                    None
                }

            } else {
                None
            }

        }).collect();

        // Sort by terrain index
        tiles.sort();

        // Generate terrains
        let mut terrain_data: Vec<TerrainData> = Vec::new();
        for tile in tiles.into_iter().rev() {

            if terrain_data.is_empty() || terrain_data.first().unwrap().id != tile.0 {
                let typ = &set.terraintypes.terrain[tile.0 as usize];
                let standalone = typ.properties.property.iter().find(|p| p.name == "standalone");
                let reduced = typ.properties.property.iter().find(|p| p.name == "reduced");
                terrain_data.insert(0, TerrainData {
                    id: tile.0,
                    name: typ.name.to_string(),
                    standalone: standalone.map(|p| {
                        p.value.split(",").filter_map(|v| {
                            v.parse::<u32>().ok()

                        }).collect()

                    }).unwrap_or_else(Vec::new),
                    reduced: reduced.map(|p| {
                        p.value.split(",").filter_map(|v| {
                            v.parse::<u32>().ok()

                        }).collect()

                    }).unwrap_or_else(Vec::new),
                    group: vec![tile.2]
                });

            } else {
                terrain_data.first_mut().unwrap().group.push(tile.2);
            }

        }

        let mut terrains = HashMap::new();
        for terrain in terrain_data {
            let g = terrain.group;
            terrains.insert(terrain.name.clone(), Terrain::new(terrain.name, [
                // Group A
                g[6], g[3], g[9],
                g[5], g[0], g[8],
                g[11], g[10], g[12],

                // Group B
                g[7], g[4],
                g[2], g[1]

            ], terrain.standalone, terrain.reduced));
        }

        Ok(Self {
            cols: size.0 / set.tilewidth.parse::<u32>()?,
            rows: size.1 / set.tileheight.parse::<u32>()?,
            texture: texture,
            terrains: terrains
        })

    }

    pub fn terrain(&self, name: &str) -> Option<&Terrain> {
        self.terrains.get(name)
    }

    pub fn get_tile_terrain(&self, tile: u32) -> Option<&Terrain> {
        for terrain in self.terrains.values() {
            if terrain.has_tile(tile) {
                return Some(terrain);
            }
        }
        None
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn uvs(&self, index: u32) -> [[f32; 2]; 4] {
        let (w, h) = (1.0 / self.cols as f32, 1.0 / self.rows as f32);
        let (x, y) = (
            (index % self.cols) as f32 * w,
            (index / self.cols) as f32 * h
        );
        [
            [x, y + h],
            [x + w, y + h],
            [x, y],
            [x + w, y]
        ]
    }

}

#[derive(Debug)]
pub struct TerrainData {
    id: u32,
    name: String,
    standalone: Vec<u32>,
    reduced: Vec<u32>,
    group: Vec<u32>
}


// Deserialization Structures -------------------------------------------------
#[derive(Debug, Deserialize)]
struct Set {
    image: Image,
    tilewidth: String,
    tileheight: String,
    terraintypes: TerrainTypes,
    tile: Vec<Tile>
}

#[derive(Debug, Deserialize)]
struct Image {
    source: String
}

#[derive(Debug, Deserialize)]
struct TerrainTypes {
    terrain: Vec<TerrainType>
}

#[derive(Debug, Deserialize)]
struct TerrainType {
    name: String,
    #[serde(default)]
    properties: TerrainProperties
}

#[derive(Debug, Default, Deserialize)]
struct TerrainProperties {
    property: Vec<TerrainProperty>
}

#[derive(Debug, Deserialize)]
struct TerrainProperty {
    name: String,
    value: String
}

#[derive(Debug, Deserialize)]
struct Tile {
    id: String,
    terrain: String
}

