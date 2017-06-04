// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::iter;


// External Dependencies ------------------------------------------------------
use renderer::{ColorBuffer, Encoder, Factory, QuadView, Vertex};


// Internal Dependencies ------------------------------------------------------
use ::animation::Animation;
use ::sheet::SpriteSheet;


// Sprite Abstraction ---------------------------------------------------------
#[derive(Debug)]
pub struct Sprite {
    id: usize,
    bucket: usize,

    position: (f32, f32),
    size: (f32, f32),
    tile: u32,
    animation: Option<Animation>
}

impl Sprite {

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

}


// SpriteView Implementation --------------------------------------------------
#[derive(Debug)]
pub struct SpriteView {
    sheet: SpriteSheet,
    sprite_buckets: Vec<Option<usize>>,
    sprite_max: usize,
    sprite_count: usize,
    sprite_id: usize,
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

            let uvs = sheet.uvs(0);

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
            sprite_buckets: iter::repeat(None).take(max_sprites).collect(),
            sprite_max: max_sprites,
            sprite_count: 0,
            sprite_id: 0,
            quad_view: quad_view,
            view_width: view_width as f32,
            view_height: view_height as f32,
            dirty: true
        }
    }

    pub fn create_sprite(&mut self) -> Option<Sprite> {
        if self.sprite_count < self.sprite_max {

            self.sprite_count += 1;
            self.sprite_id += 1;

            let mut index = 0;
            for (i, bucket) in self.sprite_buckets.iter_mut().enumerate() {
                if bucket.is_none() {
                    *bucket = Some(self.sprite_id);
                    index = i;
                    break;
                }
            }

            Some(Sprite {
                id: self.sprite_id,
                bucket: index,

                position: (0.0, 0.0),
                size: (0.0, 0.0),
                tile: 0,
                animation: None
            })

        } else {
            None
        }
    }

    pub fn update_sprite(&mut self, sprite: &Sprite) {

        let vertices = self.quad_view.vertices_mut((self.sprite_max - 1) * 6 - sprite.bucket * 6);
        let uvs = self.sheet.uvs(sprite.tile);
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

        self.dirty = true;

    }

    pub fn destroy_sprite(&mut self, mut sprite: Sprite) {
        sprite.position.0 = -100000.0;
        sprite.position.1 = -100000.0;
        self.update_sprite(&sprite);
        self.sprite_buckets[sprite.bucket] = None;
        self.sprite_count -= 1;
    }

    pub fn scroll_to(&mut self, scroll_x: i32, scroll_y: i32) {
        self.quad_view.scroll_to(-scroll_x as f32, scroll_y as f32);
    }

    pub fn draw(&mut self, encoder: &mut Encoder) {

        if self.dirty {
            self.dirty = false;
            self.quad_view.set_dirty();
        }

        self.quad_view.draw(encoder);

    }

}

