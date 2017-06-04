// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::path::Path;
use std::error::Error;


// External Dependencies ------------------------------------------------------
use renderer::{Factory, Texture};


// Internal Dependencies ------------------------------------------------------
use ::animation::Animation;


// Spritesheet Abstraction ----------------------------------------------------
#[derive(Debug)]
pub struct SpriteSheet {
    animations: Vec<Animation>,
    cols: u32,
    rows: u32,
    texture: Texture
}

impl SpriteSheet {

    pub fn new(factory: &mut Factory, path: &Path, tile_size: u32) -> Result<Self, Box<Error>> {
        let texture = Texture::new(factory, path)?;
        let size = texture.size();
        Ok(Self {
            animations: Vec::new(),
            cols: size.0 / tile_size,
            rows: size.1 / tile_size,
            texture: texture,
        })
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn uvs(&self, index: u32, size: (f32, f32)) -> [[f32; 2]; 4] {
        let (w, h) = (1.0 / self.cols as f32, 1.0 / self.rows as f32);
        let (x, y) = (
            (index % self.cols) as f32 * w,
            (index / self.cols) as f32 * h
        );
        [
            [x, y + h * size.1],
            [x + w * size.0, y + h * size.1],
            [x, y],
            [x + w * size.0, y]
        ]
    }

}

