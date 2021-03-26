use std::{collections::HashSet, path::PathBuf};

use ultraviolet::{Mat4, Vec2, Vec3, Vec4};
use wgpu::util::DeviceExt as _;

use crate::{pipeline::create_default_pipeline, texture::Texture, PipelineHandler, State};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: Vec3,
    normal: Vec3,
    tex_coords: Vec2,
}

impl Vertex {
    pub fn attributes() -> [wgpu::VertexAttribute; 3] {
        [
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float3,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 3]>() as _,
                shader_location: 1,
                format: wgpu::VertexFormat::Float3,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 6]>() as _,
                shader_location: 2,
                format: wgpu::VertexFormat::Float2,
            },
        ]
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: Vec3::new(-0.5, 0.0, -0.5),
        tex_coords: Vec2::new(0.0, 0.0),
        normal: Vec3::new(0.0, 1.0, 0.0),
    },
    Vertex {
        position: Vec3::new(0.5, 0.0, 0.5),
        tex_coords: Vec2::new(1.0, 1.0),
        normal: Vec3::new(0.0, 1.0, 0.0),
    },
    Vertex {
        position: Vec3::new(0.5, 0.0, -0.5),
        tex_coords: Vec2::new(1.0, 0.0),
        normal: Vec3::new(0.0, 1.0, 0.0),
    },

    Vertex {
        position: Vec3::new(-0.5, 0.0, -0.5),
        tex_coords: Vec2::new(0.0, 0.0),
        normal: Vec3::new(0.0, 1.0, 0.0),
    },
    Vertex {
        position: Vec3::new(-0.5, 0.0, 0.5),
        tex_coords: Vec2::new(0.0, 1.0),
        normal: Vec3::new(0.0, 1.0, 0.0),
    },
    Vertex {
        position: Vec3::new(0.5, 0.0, 0.5),
        tex_coords: Vec2::new(1.0, 1.0),
        normal: Vec3::new(0.0, 1.0, 0.0),
    },
];

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    mat: Mat4,
    color: Vec4,
}

impl Instance {
    pub fn attributes() -> [wgpu::VertexAttribute; 5] {
        [
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 5,
                format: wgpu::VertexFormat::Float4,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 4]>() as _,
                shader_location: 6,
                format: wgpu::VertexFormat::Float4,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 8]>() as _,
                shader_location: 7,
                format: wgpu::VertexFormat::Float4,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 12]>() as _,
                shader_location: 8,
                format: wgpu::VertexFormat::Float4,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 16]>() as _,
                shader_location: 9,
                format: wgpu::VertexFormat::Float4,
            },
        ]
    }
}


fn create_bundle(
    state: &State,
) -> Option<wgpu::RenderBundle> {

    let device = &state.device;
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&VERTICES),
        usage: wgpu::BufferUsage::VERTEX,
    });

    let instances = [Instance {
        mat: Mat4::from_scale(2.0),
        color: Vec4::new(20.0 / 255.0, 40.0 / 255.0, 0.0, 1.0),
    }];

    let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Instance Buffer"),
        contents: bytemuck::cast_slice(&instances),
        usage: wgpu::BufferUsage::VERTEX,
    });

    let pipeline = create_default_pipeline(
        &state.device,
        &state.sc_desc,
        &[&state.uniform_bind_group_layout],
        &[
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &Vertex::attributes(),
            },
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Instance,
                attributes: &Instance::attributes(),
            },
        ],
        "assets/shaders/quad.vert",
        "assets/shaders/blinn_phong.frag",
    )?;

    let mut encoder =
        state
            .device
            .create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
                label: None,
                color_formats: &[state.sc_desc.format],
                depth_stencil_format: Some(Texture::DEPTH_FORMAT),
                sample_count: Texture::MSAA_SAMPLES,
            });

    encoder.set_pipeline(&pipeline);
    encoder.set_vertex_buffer(0, vertex_buffer.slice(..));
    encoder.set_vertex_buffer(1, instance_buffer.slice(..));
    encoder.set_bind_group(0, &state.uniform_bind_group, &[]);

    encoder.draw(0..VERTICES.len() as _, 0..instances.len() as _);

    Some(encoder.finish(&wgpu::RenderBundleDescriptor {
        label: Some("quad"),
    }))
}


pub struct QuadPipeline {
    render_bundle: Option<wgpu::RenderBundle>,
}

impl QuadPipeline {
    pub fn create(state: &State) -> Self {
        Self {
            render_bundle: create_bundle(state),
        }
    }

}

impl PipelineHandler for QuadPipeline {
    fn files_changed(&mut self, state: &mut State, changed: &HashSet<PathBuf>) {
        if changed.iter().any(|path| {
            path.ends_with("assets/shaders/quad.vert") || path.ends_with("assets/shaders/blinn_phong.frag")
        }) {
            let bundle = create_bundle(state);
            if bundle.is_some() {
                self.render_bundle = bundle;
                log::info!("Quad bundle reloaded.");
            }
        }
    }

    fn render_bundle(&mut self, _state: &State) -> Option<&wgpu::RenderBundle> {
        return self.render_bundle.as_ref();
    }
}
