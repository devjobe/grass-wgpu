mod file_watcher;
mod grass;
mod input;
mod perspective_camera;
mod pipeline;
mod quad;
mod texture;

use file_watcher::FileWatcher;
use grass::GrassPipeline;
use perspective_camera::PerspectiveCamera;
use quad::QuadPipeline;
use std::{
    collections::HashSet,
    iter,
    path::PathBuf,
    time::{Duration, Instant},
};
use texture::Texture;
use ultraviolet::{Mat4, Vec3, Vec4};
use wgpu::util::DeviceExt;
use winit::{
    dpi::LogicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

const BACKGROUND_CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.2,
    g: 0.5,
    b: 1.0,
    a: 1.0,
};

pub trait PipelineHandler {
    fn render_bundle(&mut self, state: &State) -> Option<&wgpu::RenderBundle>;
    fn files_changed(&mut self, state: &mut State, changed: &HashSet<PathBuf>);
}

pub struct State {
    keyboard_input: input::Input<VirtualKeyCode>,

    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,

    perspective_camera: PerspectiveCamera,
    multisampled_framebuffer: Option<wgpu::TextureView>,
    depth_texture: Texture,

    uniform_bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: Mat4,
    view_position: Vec4,
    time: f32,
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let multisampled_framebuffer = if Texture::MSAA_SAMPLES > 1 {
            Some(create_multisampled_framebuffer(
                &device,
                &sc_desc,
                Texture::MSAA_SAMPLES,
            ))
        } else {
            None
        };
        let depth_texture = Texture::create_depth_texture(&device, &sc_desc);

        let perspective_camera = PerspectiveCamera {
            eye: (0.0, 1.0, 4.0).into(),
            at: (0.0, 0.0, 0.0).into(),
            up: Vec3::unit_y(),
            vertical_fov: std::f32::consts::PI / 4.0,
            aspect_ratio: sc_desc.width as f32 / sc_desc.height as f32,
            z_near: 0.1,
            z_far: 100.0,
        };

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::bytes_of(&Uniforms {
                view_proj: perspective_camera.compute_matrix(),
                view_position: perspective_camera.eye.into_homogeneous_vector(),
                time: 0.0f32,
            }),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        Self {
            keyboard_input: Default::default(),
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            perspective_camera,
            multisampled_framebuffer,
            depth_texture,

            uniform_bind_group_layout,
            uniform_buffer,
            uniform_bind_group,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);

        if Texture::MSAA_SAMPLES > 1 {
            self.multisampled_framebuffer = Some(create_multisampled_framebuffer(
                &self.device,
                &self.sc_desc,
                Texture::MSAA_SAMPLES,
            ));
        }
        self.depth_texture = Texture::create_depth_texture(&self.device, &self.sc_desc);

        self.perspective_camera.aspect_ratio = new_size.width as f32 / new_size.height as f32;
    }

    fn keyboard_input_event(&mut self, event: &KeyboardInput) {
        if let KeyboardInput {
            virtual_keycode: Some(key),
            state,
            ..
        } = *event
        {
            match state {
                ElementState::Pressed => self.keyboard_input.activate(key),
                ElementState::Released => self.keyboard_input.deactivate(key),
            }
        }
    }

    fn update(&mut self, delta: Duration, absolute_time: Duration) {
        let difference: Vec3 = self.perspective_camera.at - self.perspective_camera.eye;
        let forward: Vec3 = (difference).normalized();

        let right = forward.cross(self.perspective_camera.up);

        let elapsed_seconds = delta.as_secs_f32();

        if self.keyboard_input.pressed(VirtualKeyCode::W) {
            if difference.mag_sq() > 1.0 {
                self.perspective_camera.eye += forward * elapsed_seconds;
            }
        }

        if self.keyboard_input.pressed(VirtualKeyCode::S) {
            self.perspective_camera.eye -= forward * elapsed_seconds;
        }
        if self.keyboard_input.pressed(VirtualKeyCode::A) {
            self.perspective_camera.eye -= right * elapsed_seconds;
        }
        if self.keyboard_input.pressed(VirtualKeyCode::D) {
            self.perspective_camera.eye += right * elapsed_seconds;
        }
        if self.keyboard_input.pressed(VirtualKeyCode::Q) {
            self.perspective_camera.eye -= self.perspective_camera.up * elapsed_seconds;
        }
        if self.keyboard_input.pressed(VirtualKeyCode::E) {
            self.perspective_camera.eye += self.perspective_camera.up * elapsed_seconds;
        }

        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::bytes_of(&Uniforms {
                view_proj: self.perspective_camera.compute_matrix(),
                view_position: self.perspective_camera.eye.into_homogeneous_vector(),
                time: absolute_time.as_secs_f32(),
            }),
        );

        self.keyboard_input.update();
    }

    fn files_changed(&mut self, _changed: &HashSet<PathBuf>) {}

    fn render(
        &mut self,
        pipelines: &mut Vec<Box<dyn PipelineHandler>>,
    ) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: {
                        if let Some(ref multisampled_frametexture) = self.multisampled_framebuffer {
                            multisampled_frametexture
                        } else {
                            &frame.view
                        }
                    },
                    resolve_target: {
                        if self.multisampled_framebuffer.is_some() {
                            Some(&frame.view)
                        } else {
                            None
                        }
                    },
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(BACKGROUND_CLEAR_COLOR),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass
                .execute_bundles(pipelines.iter_mut().filter_map(|p| p.render_bundle(&self)));
        }

        self.queue.submit(iter::once(encoder.finish()));

        Ok(())
    }
}

fn create_multisampled_framebuffer(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
    sample_count: u32,
) -> wgpu::TextureView {
    let multisampled_texture_extent = wgpu::Extent3d {
        width: sc_desc.width,
        height: sc_desc.height,
        depth: 1,
    };
    let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
        size: multisampled_texture_extent,
        mip_level_count: 1,
        sample_count: sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: sc_desc.format,
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        label: None,
    };

    device
        .create_texture(multisampled_frame_descriptor)
        .create_view(&wgpu::TextureViewDescriptor::default())
}

fn main() {
    let startup_time = Instant::now();
    let mut last_update_time = None;
    env_logger::init();
    let file_watcher = FileWatcher::default();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Grass-wgpu")
        .with_inner_size(LogicalSize::new(1024, 720))
        .build(&event_loop)
        .unwrap();

    use futures::executor::block_on;

    let mut state = block_on(State::new(&window));
    let mut pipelines = Vec::new();

    {
        let instance_pipeline: Box<dyn PipelineHandler> = Box::new(QuadPipeline::create(&state));
        pipelines.push(instance_pipeline);
    }

    {
        let instance_pipeline: Box<dyn PipelineHandler> = Box::new(GrassPipeline::create(&state));
        pipelines.push(instance_pipeline);
    }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            //Event::DeviceEvent { ref event, .. } => {}
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => {
                    state.keyboard_input_event(input);
                    match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => {}
                    }
                }
                WindowEvent::Resized(size) => state.resize(*size),
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.resize(**new_inner_size)
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                let now = Instant::now();
                let delta = now - last_update_time.unwrap_or(now);
                last_update_time = Some(now);
                state.update(delta, now - startup_time);
                match state.render(&mut pipelines) {
                    Ok(_) => {}
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(wgpu::SwapChainError::Outdated) => {}
                    Err(wgpu::SwapChainError::Timeout) => {}
                }

                if let Some(changed) = file_watcher.collect_modified() {
                    state.files_changed(&changed);
                    for pipeline in pipelines.iter_mut() {
                        pipeline.files_changed(&mut state, &changed);
                    }
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}
