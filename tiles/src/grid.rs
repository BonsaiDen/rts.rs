// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::cmp;


// External Dependencies ------------------------------------------------------
use renderer::{ColorBuffer, Encoder, Factory, QuadView, Vertex};


// Internal Dependencies ------------------------------------------------------
use ::data::TileData;
use ::terrain::Terrain;
use ::tileset::TileSet;
use ::source::TileSource;


// Tilegrid Abstraction -------------------------------------------------------
#[derive(Debug)]
pub struct TileGrid<S> {
    tileset: TileSet,
    source: S,
    quad_view: QuadView,
    draw_size: u32,
    dirty: bool,
    border: u32,
    ox: i32,
    oy: i32,
    gx: u32,
    gy: u32,
    tx: u32,
    ty: u32,
    width: u32,
    height: u32,
    rows: u32,
    cols: u32
}

impl<S> TileGrid<S> where S: TileSource {

    pub fn new(
        factory: &mut Factory,
        color: ColorBuffer,
        view_width: u32,
        view_height: u32,
        draw_size: u32,
        tileset: TileSet

    ) -> Self {

        let border = 4;
        let ts = draw_size as f32;
        let (w, h) = (view_width / draw_size, view_height / draw_size);
        let (cols, rows) = (w + border * 2, h + border * 2);

        let (bx, by) = (
            -(view_width as f32 / 2.0) - ts * border as f32,
            (view_height as f32 / 2.0) + (ts * border as f32 - ts).max(0.0)
        );

        let mut vertices = Vec::with_capacity((cols * rows) as usize);
        for y in 0..rows {
            for x in 0..cols {

                let uvs = tileset.uvs(0);
                let (x, y) = (x as f32 * ts, y as f32 * ts);

                let tr = ts;

                // Top left
                vertices.push(Vertex { pos: [bx + x, by - y], uv: uvs[0] });

                // Top right
                vertices.push(Vertex { pos: [bx + x + tr, by - y ], uv: uvs[1] });

                // Bottom left
                vertices.push(Vertex { pos: [bx + x, by - y + tr ], uv: uvs[2] });

                // Top right
                vertices.push(Vertex { pos: [bx + x + tr, by - y ], uv: uvs[1] });

                // bottom right
                vertices.push(Vertex { pos: [bx + x + tr, by - y + tr], uv: uvs[3] });

                // Top bottom left
                vertices.push(Vertex { pos: [bx + x, by - y + tr], uv: uvs[2] });

            }
        }

        let quad_view = QuadView::new(
            factory,
            color,
            view_width,
            view_height,
            tileset.texture().bind(),
            vertices
        );

        Self {
            tileset: tileset,
            source: S::default(),
            quad_view: quad_view,
            draw_size: draw_size,
            dirty: true,
            border: border,
            ox: 0,
            oy: 0,
            gx: 0,
            gy: 0,
            tx: 0,
            ty: 0,
            width: w,
            height: h,
            rows: rows,
            cols: cols
        }

    }

    pub fn set_tile_index(&mut self, x: i32, y: i32, index: u32) {
        if self.source.set_tile_index(x, y, index) {
            self.dirty = true;
        }
    }

    pub fn get_tile_index(&self, x: i32, y: i32) -> Option<u32> {
        self.source.get_tile_index(x, y)
    }

    pub fn get_tile_terrain(&self, x: i32, y: i32) -> Option<&Terrain> {
        if let Some(index) = self.source.get_tile_index(x, y) {
            self.tileset.get_tile_terrain(index)

        } else {
            None
        }
    }

    pub fn screen_to_grid(&self, x: i32, y: i32) -> (i32, i32) {
        let (x, y) = self.limit(x + self.ox, y + self.oy);
        (
            (self.tx + x / self.draw_size) as i32,
            (self.ty + y / self.draw_size) as i32
        )
    }

    pub fn tile_within_screen_grid(&self, x: i32, y: i32, border: i32) -> bool {
        let ox = (self.gx * self.border) as i32 - border;
        let oy = (self.gy * self.border) as i32 - border;
        x >= ox && x < ox + self.width as i32 + border * 2 && y >= oy && y < oy + self.height as i32 + border * 2
    }

    pub fn tileset(&self) -> &TileSet {
        &self.tileset
    }

    pub fn source(&self) -> &S {
        &self.source
    }

    pub fn source_mut(&mut self) -> &mut S {
        &mut self.source
    }

    pub fn set_source(&mut self, source: S) {
        self.source = source;
        self.dirty = true;
    }

    pub fn scroll_to(&mut self, scroll_x: i32, scroll_y: i32) -> (i32, i32) {

        let (scroll_x, scroll_y) = self.limit(scroll_x, scroll_y);

        self.tx = scroll_x / self.draw_size;
        self.ty = scroll_y / self.draw_size;
        self.ox = (scroll_x % self.draw_size) as i32;
        self.oy = (scroll_y % self.draw_size) as i32;

        // Redraw every "border" tiles
        let gx = self.tx / self.border;
        let gy = self.ty / self.border;
        if gx != self.gx || gy != self.gy {
            self.gx = gx;
            self.gy = gy;
            self.dirty = true;
        }

        // Scroll offset
        self.quad_view.scroll_to(
            -((scroll_x % (self.draw_size * self.border)) as f32),
            (scroll_y % (self.draw_size * self.border)) as f32
        );

        (scroll_x as i32, scroll_y as i32)

    }

    pub fn draw(&mut self, encoder: &mut Encoder) {

        if self.dirty {
            self.dirty = false;
            self.update_tiles();
            self.quad_view.set_dirty();
        }

        self.quad_view.draw(encoder, None);

    }

    fn update_tiles(&mut self) {

        let ox = (self.gx * self.border) as isize - self.border as isize;
        let oy = (self.gy * self.border) as isize - self.border as isize;

        let w = self.source.width() as isize;
        let m = w * self.source.height() as isize;

        for y in 0..self.rows as isize {
            for x in 0..self.cols as isize {

                let offset = (oy + y) * w + (ox + x as isize);
                if offset >= 0 && offset < m {
                    let i = self.source.index(offset as usize);
                    self.set_tile(x as u32, y as u32, i);
                }

            }
        }
    }

    fn set_tile(&mut self, x: u32, y: u32, i: u32) {
        let index = ((x + y * self.cols) * 6) as usize;
        let uvs = self.tileset.uvs(i);
        let vertices = self.quad_view.vertices_mut(index);
        vertices[0].uv = uvs[0];
        vertices[1].uv = uvs[1];
        vertices[2].uv = uvs[2];
        vertices[3].uv = uvs[1];
        vertices[4].uv = uvs[3];
        vertices[5].uv = uvs[2];
    }

    fn limit(&self, x: i32, y: i32) -> (u32, u32) {
        (
            cmp::min(cmp::max(x, 0) as u32, (cmp::max(self.source.width(), self.width) - self.width) * self.draw_size),
            cmp::min(cmp::max(y, 0) as u32, (cmp::max(self.source.height(), self.height) - self.height) * self.draw_size)
        )
    }

}


// Terrain Specific Grid Implementation ---------------------------------------
pub type TerrainGrid = TileGrid<TileData>;

impl TerrainGrid {

    pub fn consume_tile(&mut self, x: i32, y: i32) -> Option<&Terrain> {
        if let Some(index) = self.source.get_tile_index(x, y) {
            if let Some(terrain) = self.tileset.get_tile_terrain(index) {
                if terrain.consume_tile(&mut self.source, x, y) {
                    self.dirty = true;
                    Some(terrain)

                } else {
                    None
                }

            } else {
                None
            }

        } else {
            None
        }
    }

}

