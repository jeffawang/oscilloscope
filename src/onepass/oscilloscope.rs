use std::{borrow::Cow, time::Instant};

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use super::{wgpu_resources::WgpuResources, Shaderer};

struct Uniforms {
    time: Instant,
}

pub struct Oscilloscope {
    pub wgpu_resources: WgpuResources,

    render_pipeline: wgpu::RenderPipeline,
    instance_buffer: wgpu::Buffer,

    uniforms: Uniforms,
}

#[repr(C)]
#[derive(Pod, Copy, Zeroable, Clone)]
struct Vertex([f32; 2]);

impl Oscilloscope {
    fn new(wgpu_resources: WgpuResources) -> Self {
        Self {
            render_pipeline: Oscilloscope::new_render_pipeline(&wgpu_resources),
            instance_buffer: Oscilloscope::new_instance_buffer(&wgpu_resources),
            wgpu_resources,
            uniforms: Uniforms {
                time: Instant::now(),
            },
        }
    }

    fn new_instance_buffer(wgpu_resources: &WgpuResources) -> wgpu::Buffer {
        let data = [
            Vertex([1.0, 0.0]),
            Vertex([0.1, 0.1]),
            Vertex([0.0, 0.0]),
            Vertex([0.0, 1.0]),
        ];
        wgpu_resources
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::bytes_of(&data),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            })
    }

    fn new_render_pipeline(wgpu_resources: &WgpuResources) -> wgpu::RenderPipeline {
        let WgpuResources { device, config, .. } = wgpu_resources;
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("render.wgsl"))),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[], // TODO: use these maybe??
            push_constant_ranges: &[],
        });

        let vertex = wgpu::VertexState {
            module: &shader,
            entry_point: "main_vs",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2],
            }], // example: blah.rs:381
        };

        let fragment = wgpu::FragmentState {
            module: &shader,
            entry_point: "main_fs",
            targets: &[config.format.into()],
        };

        let primitive = wgpu::PrimitiveState {
            polygon_mode: wgpu::PolygonMode::Line,
            // polygon_mode: wgpu::PolygonMode::Fill,
            ..Default::default()
        };

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            layout: Some(&layout),
            fragment: Some(fragment),
            vertex,
            primitive,
        })
    }
    fn rpass(&self, command_encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let color_attachments = [wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::default()),
                store: true,
            },
        }];

        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &color_attachments,
            depth_stencil_attachment: None,
        };

        command_encoder.push_debug_group("Render Pass");
        {
            let mut rpass = command_encoder.begin_render_pass(&render_pass_descriptor);
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_vertex_buffer(0, self.instance_buffer.slice(..)); // TODO: fill in this buffer
            rpass.draw(0..4, 0..4); // TODO: more instances
        }
        command_encoder.pop_debug_group();
    }
    pub fn update(&mut self) {
        self.uniforms.time = Instant::now();
    }
}

impl Shaderer for Oscilloscope {
    fn new(wgpu_resources: WgpuResources) -> Self {
        Oscilloscope::new(wgpu_resources)
    }

    fn render(&self, view: &wgpu::TextureView) {
        let WgpuResources { device, queue, .. } = &self.wgpu_resources;

        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Command Encoder"),
        });
        self.rpass(&mut command_encoder, view);
        queue.submit(Some(command_encoder.finish()));
    }
}
