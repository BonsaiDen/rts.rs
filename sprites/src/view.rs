// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// External Dependencies ------------------------------------------------------
use renderer::{ColorBuffer, Encoder, Factory, QuadView, Vertex};


// Internal Dependencies ------------------------------------------------------
use ::animation::Animation;
use ::sheet::SpriteSheet;


// Sprite Abstraction ---------------------------------------------------------
#[derive(Debug)]
pub struct Sprite {
    position: (f32, f32),
    size: (f32, f32),
    tile: u32,
    tile_size: (f32, f32),
    animation: Option<Animation>
}

impl Sprite {

    pub fn new() -> Self {
        Self {
            position: (0.0, 0.0),
            size: (0.0, 0.0),
            tile: 0,
            tile_size: (1.0, 1.0),
            animation: None
        }
    }

    pub fn size(&self) -> (f32, f32) {
        self.size
    }

    pub fn position(&self) -> (f32, f32) {
        self.position
    }

    pub fn set_size(&mut self, w: f32, h: f32) {
        self.size.0 = w;
        self.size.1 = h;
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position.0 = x;
        self.position.1 = y;
    }

    pub fn set_tile(&mut self, index: u32) {
        self.tile = index;
    }

    pub fn set_tile_size(&mut self, x: u8, y: u8) {
        self.tile_size = (x as f32, y as f32);
    }

    pub fn hit(&self, x: f32, y: f32) -> bool {
        x >= self.position.0
        && y >= self.position.1
        && x < self.position.0 + self.size.0
        && y < self.position.1 + self.size.1
    }

}


// SpriteView Implementation --------------------------------------------------
#[derive(Debug)]
pub struct SpriteView {
    sheet: SpriteSheet,
    sprite_draw_index: usize,
    sprite_max: usize,
    quad_view: QuadView,
    view_width: f32,
    view_height: f32,
    dirty: bool,
}

impl SpriteView {

    pub fn new(
        factory: &mut Factory,
        color: ColorBuffer,
        view_width: u32,
        view_height: u32,
        sheet: SpriteSheet,
        max_sprites: usize

    ) -> Self {

        let mut vertices = Vec::with_capacity(max_sprites);
        for _ in 0..max_sprites {

            let uvs = sheet.uvs(0, (1.0, 1.0));

            // Top left
            vertices.push(Vertex { pos: [-100000.0, -100000.0], uv: uvs[0] });

            // Top right
            vertices.push(Vertex { pos: [-100000.0, -100000.0], uv: uvs[1] });

            // Bottom left
            vertices.push(Vertex { pos: [-100000.0, -100000.0], uv: uvs[2] });

            // Top right
            vertices.push(Vertex { pos: [-100000.0, -100000.0], uv: uvs[1] });

            // bottom right
            vertices.push(Vertex { pos: [-100000.0, -100000.0], uv: uvs[3] });

            // Top bottom left
            vertices.push(Vertex { pos: [-100000.0, -100000.0], uv: uvs[2] });

        }

        let quad_view = QuadView::new(
            factory,
            color,
            view_width,
            view_height,
            sheet.texture().bind(),
            vertices
        );

        Self {
            sheet: sheet,
            sprite_draw_index: 0,
            sprite_max: max_sprites,
            quad_view: quad_view,
            view_width: view_width as f32,
            view_height: view_height as f32,
            dirty: true
        }
    }

    pub fn scroll_to(&mut self, scroll_x: i32, scroll_y: i32) {
        self.quad_view.scroll_to(-scroll_x as f32, scroll_y as f32);
    }

    pub fn draw(&mut self, encoder: &mut Encoder) {

        if self.dirty {
            self.dirty = false;
            self.quad_view.set_dirty();
        }

        self.quad_view.draw(encoder, Some(self.sprite_draw_index as u32 * 6));
        self.sprite_draw_index = 0;

    }

    pub fn draw_sprite(&mut self, sprite: &Sprite) {

        if self.sprite_draw_index == self.sprite_max {
            return;
        }

        let vertices = self.quad_view.vertices_mut(self.sprite_draw_index * 6);
        let uvs = self.sheet.uvs(sprite.tile, sprite.tile_size);
        let (w, h) = sprite.size;
        let (x, y) = (
            (sprite.position.0 - self.view_width / 2.0),
            -(sprite.position.1 - self.view_height / 2.0 + h)
        );

        // Top left
        vertices[0].pos = [x, y];
        vertices[0].uv = uvs[0];

        // Top right
        vertices[1].pos = [x + w, y];
        vertices[1].uv = uvs[1];

        // Bottom left
        vertices[2].pos = [x, y + h];
        vertices[2].uv = uvs[2];

        // Top right
        vertices[3].pos = [x + w, y];
        vertices[3].uv = uvs[1];

        // bottom right
        vertices[4].pos = [x + w, y + h];
        vertices[4].uv = uvs[3];

        // Top bottom left
        vertices[5].pos = [x, y + h];
        vertices[5].uv = uvs[2];

        self.sprite_draw_index += 1;
        self.dirty = true;

    }

}

