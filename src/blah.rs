use std::{borrow::Cow, mem};

use bytemuck::Zeroable;
use itertools::Itertools;
use wgpu::util::DeviceExt;

pub struct Vertex {
    position: [f32; 2],
    normal: [f32; 3],
    color: [f32; 4],
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
    // work_group_count: u32,
    frame_num: usize,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct OsciUniforms {}

const NUM_PARTICLES: u32 = 1500;
struct Particle {}

impl Oscilloscope {
    fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) -> Self {
        let osci_param_data = OsciUniforms {};
        let osci_param_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Oscilloscope Parameter buffer"),
            contents: bytemuck::cast_slice(&[osci_param_data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let (compute_pipeline, compute_bind_group_layout) = Self::new_compute_pipeline(device);
        let render_pipeline = Self::new_render_pipeline(device, config);

        let initial_particle_data = vec![0.0f32]; // TODO: make this more of a thing.
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
                            resource: particle_buffers[i].as_entire_binding(),
                        },
                    ],
                })
            })
            .collect();

        let vertex_buffer_data = [-1.0f32, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::bytes_of(&vertex_buffer_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            compute_pipeline,
            render_pipeline,
            particle_bind_groups,
            particle_buffers,
            vertex_buffer,
            frame_num: 0,
        }
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
                        attributes: &wgpu::vertex_attr_array![0 => Float32, 1 => Float32],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: 2 * 4, // TODO: set this array stride...
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![2 => Float32x2],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &draw_shader,
                entry_point: "main_fs",
                targets: &[config.format.into()],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        })
    }
}
