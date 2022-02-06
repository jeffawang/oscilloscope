// mod pipeline;
mod camera;
mod instancing;
mod texture;

use bytemuck::{self};

use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

const NUM_INSTANCES_PER_ROW: u32 = 1;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    // NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0 * NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    0.0 * NUM_INSTANCES_PER_ROW as f32 * 0.5,
    // NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

mod vertex {
    #[repr(C)]
    #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
    pub(crate) struct Vertex {
        pub(crate) position: [f32; 3],
        pub(crate) color: [f32; 4],
    }

    impl Vertex {
        pub(crate) fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x3,
                    },
                    wgpu::VertexAttribute {
                        offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                        shader_location: 1,
                        format: wgpu::VertexFormat::Float32x4,
                    },
                ],
            }
        }
    }
}

mod lightning {
    use core::num;
    use std::f32::consts::PI;

    use crate::vertex::Vertex;

    use super::vertex;

    use cgmath::vec3;
    use cgmath::InnerSpace;

    use cgmath::Vector3;
    use rand::Rng;

    pub(crate) const NUM_VERTS: usize = 32;

    pub(crate) const CYAN: [f32; 4] = [0.0, 1.0, 1.0, 1.0];

    pub(crate) const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
    pub(crate) const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];

    const CIRCLE_RADIUS: f32 = 0.5;

    pub(crate) fn lightning_step() -> cgmath::Vector3<f32> {
        cgmath::vec3(
            rand::thread_rng().gen_range(-0.4..0.4),
            0.3 - rand::thread_rng().gen_range(0.2..0.3),
            0.0,
        )
    }

    pub(crate) const LINE_THICCNESS: f32 = 0.01;

    pub(crate) fn calc_verts() -> Vec<vertex::Vertex> {
        let mut vectors = vec![vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 0.0)];
        let mut verts = vec![];
        let mut lines = vec![];
        // vectors[1] = cgmath::vec3(0.0, 0.0, 0.0);

        verts.extend((0..=NUM_VERTS).map(|i| {
            let theta = 2.0 * PI * (i as f32 / NUM_VERTS as f32);
            let x = CIRCLE_RADIUS * theta.cos();
            let y = CIRCLE_RADIUS * theta.sin();
            Vertex {
                color: YELLOW,
                position: [x, y, 0.0],
            }
        }));

        for i in 2..NUM_VERTS {
            let prev = vectors[i - 2];
            let curr = vectors[i - 1];
            let next = vec3(curr[0], curr[1], curr[2]) + lightning_step();

            vectors.push(next);
            // verts.push(Vertex {
            //     color: CYAN,
            //     position: next.into(),
            // });
            lines.extend(elbow(prev, curr, next).iter().map(|&v| vertex::Vertex {
                color: RED,
                position: v.into(),
            }));
        }
        verts.extend(lines);
        verts
    }

    pub(crate) fn elbow(
        prev: Vector3<f32>,
        curr: Vector3<f32>,
        next: Vector3<f32>,
    ) -> Vec<Vector3<f32>> {
        let e1 = (prev - curr)
            .normalize()
            .cross(vec3(0.0, 0.0, 1.0))
            .normalize_to(LINE_THICCNESS);
        let e2 = (next - curr)
            .normalize()
            .cross(vec3(0.0, 0.0, 1.0))
            .normalize_to(LINE_THICCNESS);
        vec![curr + e1, curr - e1, curr - e2, curr + e2]
    }

    pub(crate) fn calc_indices(verts: &Vec<vertex::Vertex>) -> Vec<u16> {
        (0..verts.len() as u16)
            .collect::<Vec<u16>>()
            .try_into()
            .unwrap()
    }
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    color: wgpu::Color,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,

    use_complex: bool,

    diffuse_bind_group: wgpu::BindGroup,

    instances: Vec<instancing::Instance>,
    instance_buffer: wgpu::Buffer,

    depth_texture: texture::Texture,
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
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
                    label: None,
                    features: wgpu::Features::POLYGON_MODE_LINE,
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
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
        let (texture_bind_group_layout, diffuse_bind_group) = texture_bind_group(&device, &queue);

        let color = wgpu::Color::BLACK;

        // alternatively, can use include_wgsl! macro
        // let shader = device.create_shader_module(&include_wgsl!("shader.wgsl"));
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // let camera = camera::new_camera(&config, &device);

        let (instances, instance_buffer) =
            instancing::new(&device, NUM_INSTANCES_PER_ROW, INSTANCE_DISPLACEMENT);

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex::Vertex::desc(), instancing::InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill
                // requires Features::NON_FILL_POLYGON_MODE
                // polygon_mode: wgpu::PolygonMode::Line,
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let verts = lightning::calc_verts();
        let indices = lightning::calc_indices(&verts);
        let num_indices = indices.len() as u32;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let use_complex = false;

        Self {
            surface,
            device,
            queue,
            config,
            size,
            color,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            use_complex,
            diffuse_bind_group,
            instances,
            instance_buffer,
            depth_texture,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.color = wgpu::Color {
                    r: 0.0,
                    g: 0.05 * position.x / (self.size.width as f64),
                    b: 0.05 * position.y / (self.size.height as f64),
                    a: 1.0,
                };
                false
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
                self.use_complex = *state == ElementState::Pressed;
                true
            }
            _ => false,
        }
    }

    fn update(&mut self) {
        // self.camera
        //     .controller
        //     .update_camera(&mut self.camera.camera);
        // self.camera.uniform.update_view_proj(&self.camera.camera);
        // self.queue.write_buffer(
        //     &self.camera.buffer,
        //     0,
        //     bytemuck::cast_slice(&[self.camera.uniform]),
        // );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.color),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_pipeline(&self.render_pipeline);

            let data = (&self.vertex_buffer, &self.index_buffer, self.num_indices);

            let bind_group = &self.diffuse_bind_group;
            render_pass.set_bind_group(0, bind_group, &[]);
            // render_pass.set_bind_group(1, &self.camera.bind_group, &[]);

            render_pass.set_vertex_buffer(0, data.0.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

            render_pass.set_index_buffer(data.1.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.draw_indexed(0..data.2, 0, 0..self.instances.len() as _);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn texture_bind_group(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    let diffuse_bytes = include_bytes!("happy-tree.png");
    let diffuse_texture =
        texture::Texture::from_bytes(device, queue, diffuse_bytes, "happy-tree.png").unwrap();
    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(
                        // SamplerBindingType::Comparison is only for TextureSampleType::Depth
                        // SamplerBindingType::Filtering if the sample_type of the texture is:
                        //     TextureSampleType::Float { filterable: true }
                        // Otherwise you'll get an error.
                        wgpu::SamplerBindingType::Filtering,
                    ),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });
    let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
            },
        ],
        label: Some("diffuse_bind_group"),
    });
    (texture_bind_group_layout, diffuse_bind_group)
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = pollster::block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !state.input(event) {
                match event {
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
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &&mut so we have to dereference it twice
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            state.update();
            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
        _ => {}
    });
}
