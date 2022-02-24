use std::{borrow::Cow, mem, num::NonZeroU32};

use itertools::Itertools;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 2],
}

pub struct Line {
    pos: [f32; 2],
    angle: f32,
    len: f32,
}

struct Oscilloscope {
    compute_pipeline: wgpu::ComputePipeline,

    particle_bind_groups: Vec<wgpu::BindGroup>,
    particle_buffers: Vec<wgpu::Buffer>,

    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    work_group_count: u32,
    texture: wgpu::Texture,
    frame_num: usize,
    blit_pipeline: wgpu::RenderPipeline,
    blit_bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct OsciUniforms {
    frame: u32,
}

const NUM_PARTICLES: u32 = 10;
const PARTICLES_PER_GROUP: u32 = 64;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Particle {
    pos: [f32; 2],
    len: f32,
    angle: f32,
}
/*
   [[location(0)]] pos: vec2<f32>,
   [[location(1)]] len: f32,
   [[location(2)]] angle: f32,
*/

impl Oscilloscope {
    fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) -> Self {
        let osci_uniforms = OsciUniforms { frame: 0 };
        let initial_particle_data = (0..NUM_PARTICLES)
            .map(|i| {
                let theta = 2.0 * 3.14159 * (i as f32 / NUM_PARTICLES as f32);
                Particle {
                    pos: [theta.cos(), theta.sin()],
                    len: 0.15,
                    angle: i as f32,
                }
            })
            .collect_vec();

        // Compute pipeline setup
        let (compute_pipeline, compute_bind_group_layout) = Self::new_compute_pipeline(device);

        let (particle_buffers, particle_bind_groups) = Self::particle_buffers_and_bind_groups(
            device,
            &compute_bind_group_layout,
            &osci_uniforms,
            &initial_particle_data,
        );

        // Rendering pipeline setup
        let render_pipeline = Self::new_render_pipeline(device, config);
        let vertex_buffer_data = [
            Vertex {
                position: [-1.0, 0.0],
            },
            Vertex {
                position: [1.0, 0.0],
            },
            Vertex {
                position: [-1.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0],
            },
        ];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::bytes_of(&vertex_buffer_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let work_group_count =
            ((NUM_PARTICLES as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Render Texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        // Blit pipeline setup
        let blit_pipeline = Self::new_blit_pipeline(device, config);
        let blit_bind_group = Self::blit_bind_group(&blit_pipeline, device, &texture);

        Self {
            compute_pipeline,
            render_pipeline,
            particle_bind_groups,
            particle_buffers,
            vertex_buffer,
            work_group_count,
            blit_pipeline,
            blit_bind_group,
            texture,
            frame_num: 0,
        }
    }

    fn render(
        &mut self,
        view: &wgpu::TextureView,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_texture: &wgpu::Texture,
    ) {
        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Command Encoder"),
        });

        // Compute pass
        self.cpass(&mut command_encoder);

        // Render pass
        self.rpass(
            &mut command_encoder,
            // view
            &self
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
        );

        // Blit pass
        self.bpass(&mut command_encoder, view);

        self.frame_num += 1;

        queue.submit(Some(command_encoder.finish()));
    }

    fn cpass(&mut self, command_encoder: &mut wgpu::CommandEncoder) {
        command_encoder.push_debug_group("compute lines");
        {
            // compute pass
            let mut cpass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
            });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &self.particle_bind_groups[self.frame_num % 2], &[]);
            cpass.dispatch(self.work_group_count, 1, 1);
        }
        command_encoder.pop_debug_group();
    }

    fn rpass(&self, command_encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let color_attachments = [wgpu::RenderPassColorAttachment {
            // view,
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            },
        }];

        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &color_attachments,
            depth_stencil_attachment: None,
        };

        command_encoder.push_debug_group("render stuff");
        {
            let mut rpass = command_encoder.begin_render_pass(&render_pass_descriptor);
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_vertex_buffer(0, self.particle_buffers[(self.frame_num + 1) % 2].slice(..));
            rpass.set_vertex_buffer(1, self.vertex_buffer.slice(..));
            rpass.draw(0..4, 0..NUM_PARTICLES);
        }
        command_encoder.pop_debug_group();
    }

    fn bpass(&self, command_encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let color_attachments = [wgpu::RenderPassColorAttachment {
            // view,
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            },
        }];

        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: Some("Blit Pass"),
            color_attachments: &color_attachments,
            depth_stencil_attachment: None,
        };

        command_encoder.push_debug_group("blit to screen");
        {
            let mut bpass = command_encoder.begin_render_pass(&render_pass_descriptor);
            bpass.set_pipeline(&self.blit_pipeline);
            bpass.set_bind_group(0, &self.blit_bind_group, &[]);
            bpass.draw(0..4, 0..NUM_PARTICLES);
        }
        command_encoder.pop_debug_group();
    }

    fn new_compute_pipeline(
        device: &wgpu::Device,
    ) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
        let compute_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("compute shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/compute.wgsl"))),
        });
        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Compute Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                mem::size_of::<OsciUniforms>() as _
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            // min_binding_size: None,
                            min_binding_size: wgpu::BufferSize::new(
                                (NUM_PARTICLES * mem::size_of::<Particle>() as u32) as _,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            // min_binding_size: None,
                            min_binding_size: wgpu::BufferSize::new(
                                (NUM_PARTICLES * mem::size_of::<Particle>() as u32) as _,
                            ),
                        },
                        count: None,
                    },
                ],
            });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        (
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Compute Pipeline"),
                layout: Some(&compute_pipeline_layout),
                module: &compute_shader,
                entry_point: "main",
            }),
            compute_bind_group_layout,
        )
    }

    fn particle_buffers_and_bind_groups(
        device: &wgpu::Device,
        compute_bind_group_layout: &wgpu::BindGroupLayout,
        osci_uniforms: &OsciUniforms,
        initial_particle_data: &Vec<Particle>,
    ) -> (Vec<wgpu::Buffer>, Vec<wgpu::BindGroup>) {
        let osci_param_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Oscilloscope Parameter buffer"),
            contents: bytemuck::cast_slice(&[*osci_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let particle_buffers = (0..2)
            .map(|i| {
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Particle Buffer {}", i)),
                    contents: bytemuck::cast_slice(&initial_particle_data),
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST,
                })
            })
            .collect_vec();
        let particle_bind_groups = (0..2)
            .map(|i| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("Bind Group {}", i)),
                    layout: &compute_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: osci_param_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: particle_buffers[i].as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: particle_buffers[1 - i].as_entire_binding(),
                        },
                    ],
                })
            })
            .collect();
        (particle_buffers, particle_bind_groups)
    }

    fn new_render_pipeline(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> wgpu::RenderPipeline {
        let draw_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("draw shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/draw.wgsl"))),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &draw_shader,
                entry_point: "main_vs",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Line>() as _, // TODO: revisit this...
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32, 2 => Float32],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as _, // TODO: set this array stride...
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![3 => Float32x2],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &draw_shader,
                entry_point: "main_fs",
                targets: &[config.format.into()],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                // Setting this to anything other than Fill
                // requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // polygon_mode: wgpu::PolygonMode::Line,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        })
    }

    fn new_blit_pipeline(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> wgpu::RenderPipeline {
        let blit_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/blit.wgsl"))),
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blit Pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &blit_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &blit_shader,
                entry_point: "fs_main",
                targets: &[wgpu::TextureFormat::Bgra8UnormSrgb.into()],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        })
    }

    fn blit_bind_group(
        pipeline: &wgpu::RenderPipeline,
        device: &wgpu::Device,
        texture: &wgpu::Texture,
    ) -> wgpu::BindGroup {
        let bind_group_layout = pipeline.get_bind_group_layout(0);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Blit"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Blit View"),
            format: None,
            dimension: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: NonZeroU32::new(1),
            base_array_layer: 0,
            array_layer_count: None,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blit Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        bind_group
    }
}

pub mod main {
    use winit::{
        event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::{Window, WindowBuilder},
    };

    use super::Oscilloscope;

    pub fn main() {
        env_logger::init();
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        let (mut oscilloscope, surface, mut config, device, queue) =
            pollster::block_on(setup_oscilloscope(&event_loop, &window));

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    // state.resize(*physical_size);
                    config.width = physical_size.width;
                    config.height = physical_size.height;
                    surface.configure(&device, &config);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    // new_inner_size is &&mut so we have to dereference it twice
                    // state.resize(**new_inner_size);
                }

                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(VirtualKeyCode::Space),
                            ..
                        },
                    ..
                } => {
                    println!("Space pressed");
                }
                _ => {}
            },
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                // TODO: update state
                // TODO: render
                let frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(_) => {
                        surface.configure(&device, &config);
                        surface
                            .get_current_texture()
                            .expect("Failed to acquire next surface texture!")
                    }
                };
                let dst = &frame.texture;
                let view = dst.create_view(&wgpu::TextureViewDescriptor::default());

                oscilloscope.render(&view, &config, &device, &queue, dst);
                frame.present();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        });
    }

    async fn setup_oscilloscope(
        event_loop: &EventLoop<()>,
        window: &Window,
    ) -> (
        Oscilloscope,
        wgpu::Surface,
        wgpu::SurfaceConfiguration,
        wgpu::Device,
        wgpu::Queue,
    ) {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device Descriptor"),
                    features: wgpu::Features::POLYGON_MODE_LINE,
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &config);

        let frame = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => {
                surface.configure(&device, &config);
                surface
                    .get_current_texture()
                    .expect("Failed to acquire next surface texture!")
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        (
            Oscilloscope::init(&config, &adapter, &device, &queue),
            surface,
            config,
            device,
            queue,
        )
    }
}
