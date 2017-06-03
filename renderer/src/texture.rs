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
use gfx;
use gfx::Factory;
use gfx_device_gl;
use image;


// Texture Abstraction --------------------------------------------------------
#[derive(Debug)]
pub struct Texture {
    view: gfx::handle::ShaderResourceView<gfx_device_gl::Resources, [f32; 4]>,
    size: (u32, u32)
}

impl Texture {

    pub fn new(factory: &mut gfx_device_gl::Factory, path: &Path) -> Result<Self, Box<Error>> {

        let img = image::open(path)?.to_rgba();
        let (width, height) = img.dimensions();
        let kind = gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);
        let (_, view) = factory.create_texture_immutable_u8::<gfx::format::Srgba8>(kind, &[&img])?;

        Ok(Self {
            view: view,
            size: (width, height)
        })
    }

    pub fn bind(&self) -> gfx::handle::ShaderResourceView<gfx_device_gl::Resources, [f32; 4]> {
        self.view.clone()
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

}

