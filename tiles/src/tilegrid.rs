// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::cmp;


// External Dependencies ------------------------------------------------------
use gfx;
use gfx::Factory;
use gfx::traits::FactoryExt;
use gfx::state::Rasterizer;
use gfx::texture::{SamplerInfo, FilterMethod, WrapMode};
use gfx_device_gl;


// Internal Dependencies ------------------------------------------------------
use ::terrain::Terrain;
use ::tileset::TileSet;
use ::tiledata::TileData;


// Tilegrid Abstraction -------------------------------------------------------
#[derive(Debug)]
pub struct TileGrid {
    tileset: TileSet,
    tiledata: TileData,
    vertices: Vec<Vertex>,
    transform: Transform,
    vertex_buffer: gfx::handle::Buffer<gfx_device_gl::Resources, Vertex>,
    pso: gfx::PipelineState<gfx_device_gl::Resources, grid::Meta>,
    data: grid::Data<gfx_device_gl::Resources>,
    slice: gfx::Slice<gfx_device_gl::Resources>,
    draw_size: u32,
    dirty: bool,
    border: u32,
    scale_x: f32,
    scale_y: f32,
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

impl TileGrid {

    pub fn new(
        factory: &mut gfx_device_gl::Factory,
        output_color: gfx::handle::RenderTargetView<gfx_device_gl::Resources, (gfx::format::R8_G8_B8_A8, gfx::format::Srgb)>,
        width: u32,
        height: u32,
        draw_size: u32,
        tileset: TileSet

    ) -> Self {

        let border = 4;
        let ts = draw_size as f32;
        let (w, h) = (width / draw_size, height / draw_size);
        let (cols, rows) = (w + border * 2, h + border * 2);

        let (bx, by) = (
            -(width as f32 / 2.0) - ts * border as f32,
            (height as f32 / 2.0) + (ts * border as f32 - ts).max(0.0)
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

        let scale_x = 2.0 / width as f32;
        let scale_y = 2.0 / height as f32;
        let vertex_count = vertices.len();

        // Tile Map Texture
        let texture = tileset.texture().bind();
        let sampler = factory.create_sampler(
            SamplerInfo::new(FilterMethod::Scale, WrapMode::Tile)
        );

        // Create buffers
        let locals_buffer = factory.create_constant_buffer(1);
        let vertex_buffer = factory.create_buffer::<Vertex>(
            vertex_count * 4,
            gfx::buffer::Role::Vertex,
            gfx::memory::Usage::Dynamic,
            gfx::Bind::empty()

        ).expect("Could not create `vertex_buffer`");

        // Create Shaders and Pipeline
        let shader_program = factory.link_program(
            VERTEX_SHADER_150,
            FRAGMENT_SHADER_150

        ).expect("TileGrid: Failed to link shader program.");

        let mut r = Rasterizer::new_fill();
        //r.cull_face = gfx::state::CullFace::Back;
        //r.method = gfx::state::RasterMethod::Line(1);
        r.samples = None;
        let pso = factory.create_pipeline_from_program(
            &shader_program,
            gfx::Primitive::TriangleList,
            r,
            grid::Init {
                vbuf: (),
                transform: "Transform",
                tex: "t_Texture",
                out: "o_Color"
            }

        ).expect("TileGrid: PSO init failed.");

        Self {
            tileset: tileset,
            tiledata: TileData::default(),
            vertices: vertices,
            transform: Transform {
                transform: [[ scale_x, 0.0, 0.0, 0.0],
                            [0.0,  scale_y, 0.0, 0.0],
                            [0.0, 0.0, 1.0, 1.0],
                            [0.0, 0.0, 0.0, 1.0]]
            },
            vertex_buffer: vertex_buffer.clone(),
            //locals_buffer: locals_buffer.clone(),
            pso: pso,
            data: grid::Data {
                vbuf: vertex_buffer,
                transform: locals_buffer,
                tex: (texture, sampler),
                out: output_color,
            },
            slice: gfx::Slice {
                instances: None,
                start: 0,
                end: vertex_count as u32,
                buffer: gfx::IndexBuffer::Auto,
                base_vertex: 0
            },
            draw_size: draw_size,
            dirty: true,
            border: border,
            scale_x: scale_x,
            scale_y: scale_y,
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

    pub fn consume_tile(&mut self, x: i32, y: i32) -> bool {
        if let Some(index) = self.tiledata.get_tile_index(x, y) {
            if let Some(terrain) = self.tileset.get_tile_terrain(index) {
                if terrain.consume_tile(&mut self.tiledata, x, y) {
                    self.dirty = true;
                    true

                } else {
                    false
                }

            } else {
                false
            }

        } else {
            false
        }
    }

    pub fn set_tile_index(&mut self, x: i32, y: i32, index: u32) {
        if self.tiledata.set_tile_index(x, y, index) {
            self.dirty = true;
        }
    }

    pub fn get_tile_index(&self, x: i32, y: i32) -> Option<u32> {
        self.tiledata.get_tile_index(x, y)
    }

    pub fn get_tile_terrain(&self, x: i32, y: i32) -> Option<&Terrain> {
        if let Some(index) = self.tiledata.get_tile_index(x, y) {
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

    pub fn set_tiledata(&mut self, data: TileData) {
        self.tiledata = data;
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
        self.transform.transform[3][0] = -((scroll_x % (self.draw_size * self.border)) as f32 * self.scale_x);
        self.transform.transform[3][1] = (scroll_y % (self.draw_size * self.border)) as f32 * self.scale_y;

        (scroll_x as i32, scroll_y as i32)

    }

    pub fn draw(&mut self, encoder: &mut gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>) {

        if self.dirty {
            self.dirty = false;
            self.update_tiles();
            encoder.update_buffer(&self.vertex_buffer, &self.vertices, 0).ok();
        }

        encoder.update_buffer(&self.data.transform, &[self.transform], 0).ok();
        encoder.draw(&self.slice, &self.pso, &self.data);

    }

    fn update_tiles(&mut self) {

        let ox = (self.gx * self.border) as isize - self.border as isize;
        let oy = (self.gy * self.border) as isize - self.border as isize;

        let w = self.tiledata.width as isize;
        let m = w * self.tiledata.height as isize;

        for y in 0..self.rows as isize {
            for x in 0..self.cols as isize {

                let offset = (oy + y) * w + (ox + x as isize);
                if offset >= 0 && offset < m {
                    let i = self.tiledata.indices[offset as usize];
                    self.set_tile(x as u32, y as u32, i);
                }

            }
        }
    }

    fn set_tile(&mut self, x: u32, y: u32, i: u32) {
        let uvs = self.tileset.uvs(i);
        let index = ((x + y * self.cols) * 6) as usize;
        let vertices = &mut self.vertices[index..index + 6];
        vertices[0].uv = uvs[0];
        vertices[1].uv = uvs[1];
        vertices[2].uv = uvs[2];
        vertices[3].uv = uvs[1];
        vertices[4].uv = uvs[3];
        vertices[5].uv = uvs[2];
    }

    fn limit(&self, x: i32, y: i32) -> (u32, u32) {
        (
            cmp::min(cmp::max(x, 0) as u32, (cmp::max(self.tiledata.width, self.width) - self.width) * self.draw_size),
            cmp::min(cmp::max(y, 0) as u32, (cmp::max(self.tiledata.height, self.height) - self.height) * self.draw_size)
        )
    }

}


// Data -----------------------------------------------------------------------
gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "pos",
        uv: [f32; 2] = "uv",
    }

    constant Transform {
        transform: [[f32; 4]; 4] = "u_View",
    }

    pipeline grid {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        transform: gfx::ConstantBuffer<Transform> = "Transform",
        tex: gfx::TextureSampler<[f32; 4]> = "t_Texture",
        out: gfx::RenderTarget<gfx::format::Srgba8> = "Target0",
    }
}

pub static VERTEX_SHADER_150: &'static [u8] = br#"
    #version 150 core
    in vec2 pos;
    in vec2 uv;

    out vec2 v_Uv;

    uniform Transform {
        mat4 u_View;
    };

    void main() {
        gl_Position = u_View * vec4(pos, 0.0, 1.0);
        v_Uv = uv;
    }
"#;

pub static FRAGMENT_SHADER_150: &'static [u8] = br#"
    #version 150 core

    uniform sampler2D t_Texture;
    in vec2 v_Uv;
    out vec4 o_Color;

    void main() {
        o_Color = texture(t_Texture, v_Uv);
    }
"#;

