use std::{borrow::Cow, path::Path};

use shaderc::CompileOptions;
use wgpu::VertexBufferLayout;

use crate::texture::Texture;

pub fn compile_shader<'a>(
    source_text: &str,
    shader_kind: shaderc::ShaderKind,
    input_file_name: &str,
    entry_point_name: &str,
    additional_options: Option<&CompileOptions>,
) -> Option<wgpu::ShaderSource<'a>> {
    let mut compiler = shaderc::Compiler::new()?;

    let result = compiler.compile_into_spirv(
        source_text,
        shader_kind,
        input_file_name,
        entry_point_name,
        additional_options,
    );

    match result {
        Ok(artifact) => {
            log::info!(
                "Compiled \'{}\' with {} warnings.",
                input_file_name,
                artifact.get_num_warnings()
            );
            let vec: Vec<u32> = artifact.as_binary().into();
            Some(wgpu::ShaderSource::SpirV(Cow::from(vec)))
        }
        Err(error) => {
            log::error!("{:?}", error);
            None
        }
    }
}

pub fn create_default_pipeline<P: AsRef<Path>>(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    vertex_buffers_layout: &[VertexBufferLayout],
    vs_path: P,
    fs_path: P,
) -> Option<wgpu::RenderPipeline> {
    let vs_src = std::fs::read_to_string(&vs_path).ok()?;
    let vs_data = compile_shader(
        &vs_src,
        shaderc::ShaderKind::Vertex,
        vs_path.as_ref().to_str().unwrap_or("vertex_shader.unknown"),
        "main",
        None,
    )?;

    let fs_src = std::fs::read_to_string(&fs_path).ok()?;
    let fs_data = compile_shader(
        &fs_src,
        shaderc::ShaderKind::Fragment,
        fs_path
            .as_ref()
            .to_str()
            .unwrap_or("fragment_shader.unknown"),
        "main",
        None,
    )?;

    let vs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Vertex Shader"),
        source: vs_data,
        flags: wgpu::ShaderFlags::default(),
    });

    let fs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Fragment Shader"),
        source: fs_data,
        flags: wgpu::ShaderFlags::default(),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts,
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vs_module,
            entry_point: "main",
            buffers: vertex_buffers_layout,
        },
        fragment: Some(wgpu::FragmentState {
            // 3.
            module: &fs_module,
            entry_point: "main",
            targets: &[wgpu::ColorTargetState {
                // 4.
                format: sc_desc.format,
                alpha_blend: wgpu::BlendState::REPLACE,
                color_blend: wgpu::BlendState::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList, // 1.
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw, // 2.
            cull_mode: wgpu::CullMode::None,
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less, // 1.
            stencil: wgpu::StencilState::default(),     // 2.
            bias: wgpu::DepthBiasState::default(),
            // Setting this to true requires Features::DEPTH_CLAMPING
            clamp_depth: false,
        }),
        multisample: wgpu::MultisampleState {
            count: Texture::MSAA_SAMPLES,
            ..Default::default()
        },
    });

    Some(pipeline)
}

#[allow(dead_code)]
pub fn create_transparent_pipeline<P: AsRef<Path>>(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    vertex_buffers_layout: &[VertexBufferLayout],
    vs_path: P,
    fs_path: P,
) -> Option<wgpu::RenderPipeline> {
    let vs_src = std::fs::read_to_string(&vs_path).ok()?;
    let vs_data = compile_shader(
        &vs_src,
        shaderc::ShaderKind::Vertex,
        vs_path.as_ref().to_str().unwrap_or("vertex_shader.unknown"),
        "main",
        None,
    )?;

    let fs_src = std::fs::read_to_string(&fs_path).ok()?;
    let fs_data = compile_shader(
        &fs_src,
        shaderc::ShaderKind::Fragment,
        fs_path
            .as_ref()
            .to_str()
            .unwrap_or("fragment_shader.unknown"),
        "main",
        None,
    )?;

    let vs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Vertex Shader"),
        source: vs_data,
        flags: wgpu::ShaderFlags::default(),
    });

    let fs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("Fragment Shader"),
        source: fs_data,
        flags: wgpu::ShaderFlags::default(),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts,
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vs_module,
            entry_point: "main",
            buffers: vertex_buffers_layout,
        },
        fragment: Some(wgpu::FragmentState {
            // 3.
            module: &fs_module,
            entry_point: "main",
            targets: &[wgpu::ColorTargetState {
                // 4.
                format: sc_desc.format,
                color_blend: wgpu::BlendState {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha_blend: wgpu::BlendState {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Max,
                },
                write_mask: wgpu::ColorWrite::ALL,
            }],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList, // 1.
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw, // 2.
            cull_mode: wgpu::CullMode::None,
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less, // 1.
            stencil: wgpu::StencilState::default(),     // 2.
            bias: wgpu::DepthBiasState::default(),
            // Setting this to true requires Features::DEPTH_CLAMPING
            clamp_depth: false,
        }),
        multisample: wgpu::MultisampleState {
            count: Texture::MSAA_SAMPLES,
            ..Default::default()
        },
    });

    Some(pipeline)
}
