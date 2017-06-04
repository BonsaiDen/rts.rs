// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// External Dependencies ------------------------------------------------------
use gfx;
use gfx::Factory;
use gfx::traits::FactoryExt;
use gfx::state::Rasterizer;
use gfx::texture::{SamplerInfo, FilterMethod, WrapMode};
use gfx_device_gl;


// 2D Quad Rendering Implementation -------------------------------------------
#[derive(Debug)]
pub struct QuadView {
    vertices: Vec<Vertex>,
    transform: Transform,
    vertex_buffer: gfx::handle::Buffer<gfx_device_gl::Resources, Vertex>,
    pso: gfx::PipelineState<gfx_device_gl::Resources, grid::Meta>,
    data: grid::Data<gfx_device_gl::Resources>,
    slice: gfx::Slice<gfx_device_gl::Resources>,
    scale_x: f32,
    scale_y: f32,
    view_width: u32,
    view_height: u32,
    dirty: bool,
}

impl QuadView {

    pub fn new(
        factory: &mut gfx_device_gl::Factory,
        color: gfx::handle::RenderTargetView<gfx_device_gl::Resources, (gfx::format::R8_G8_B8_A8, gfx::format::Srgb)>,
        view_width: u32,
        view_height: u32,
        texture: gfx::handle::ShaderResourceView<gfx_device_gl::Resources, [f32; 4]>,
        vertices: Vec<Vertex>

    ) -> Self {

        let vertex_count = vertices.len();

        let locals_buffer = factory.create_constant_buffer(1);
        let vertex_buffer = factory.create_buffer::<Vertex>(
            vertex_count * 4,
            gfx::buffer::Role::Vertex,
            gfx::memory::Usage::Dynamic,
            gfx::Bind::empty()

        ).expect("QuadView: Could not create `vertex_buffer`");

        let sampler = factory.create_sampler(
            SamplerInfo::new(FilterMethod::Scale, WrapMode::Tile)
        );

        let pso = QuadView::create_pipeline(factory, false);
        let scale_x = 2.0 / view_width as f32;
        let scale_y = 2.0 / view_height as f32;

        Self {
            vertices: vertices,
            transform: Transform {
                transform: [[ scale_x, 0.0, 0.0, 0.0],
                            [0.0,  scale_y, 0.0, 0.0],
                            [0.0, 0.0, 1.0, 1.0],
                            [0.0, 0.0, 0.0, 1.0]]
            },
            vertex_buffer: vertex_buffer.clone(),
            pso: pso,
            data: grid::Data {
                vbuf: vertex_buffer,
                transform: locals_buffer,
                blend_target: color.clone(),
                blend_ref: [1.0; 4],
                tex: (texture, sampler),
                out: color
            },
            slice: gfx::Slice {
                instances: None,
                start: 0,
                end: vertex_count as u32,
                buffer: gfx::IndexBuffer::Auto,
                base_vertex: 0
            },
            scale_x: scale_x,
            scale_y: scale_y,
            view_width: view_width,
            view_height: view_height,
            dirty: true
        }

    }

    pub fn set_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn scroll_to(&mut self, scroll_x: f32, scroll_y: f32) {
        self.transform.transform[3][0] = scroll_x * self.scale_x;
        self.transform.transform[3][1] = scroll_y * self.scale_y;
    }

    pub fn vertices_mut(&mut self, index: usize) -> &mut [Vertex] {
        &mut self.vertices[index..index + 6]
    }

    pub fn draw(&mut self, encoder: &mut gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>, vertices_limit: Option<u32>) {

        if self.dirty {
            self.dirty = false;
            encoder.update_buffer(&self.vertex_buffer, &self.vertices, 0).ok();
        }

        encoder.update_buffer(&self.data.transform, &[self.transform], 0).ok();

        if let Some(limit) = vertices_limit {
            self.slice.end = limit;
        }

        encoder.draw(&self.slice, &self.pso, &self.data);

    }

    pub fn set_wireframe(&mut self, factory: &mut gfx_device_gl::Factory, active: bool) {
        self.pso = QuadView::create_pipeline(factory, active);
    }

    fn create_pipeline(
        factory: &mut gfx_device_gl::Factory,
        wireframe: bool

    ) -> gfx::PipelineState<gfx_device_gl::Resources, grid::Meta> {

        // Create Shaders and Pipeline
        let shader_program = factory.link_program(
            VERTEX_SHADER_150,
            FRAGMENT_SHADER_150

        ).expect("QuadView: Failed to link shader program.");

        let mut r = Rasterizer::new_fill();
        if wireframe {
            r.method = gfx::state::RasterMethod::Line(1);
        }
        r.samples = None;

        factory.create_pipeline_from_program(
            &shader_program,
            gfx::Primitive::TriangleList,
            r,
            grid::Init {
                vbuf: (),
                transform: "Transform",
                blend_target: ("o_Color", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
                tex: "t_Texture",
                out: "o_Color",
                blend_ref: ()
            }

        ).expect("QuadView: Failed to create pipeline state object.")

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
        blend_target: gfx::BlendTarget<gfx::format::Srgba8> = ("o_Color", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
        blend_ref: gfx::BlendRef = (),
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

