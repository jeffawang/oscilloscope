use std::borrow::Cow;

use bytemuck::{Pod, Zeroable};

use wgpu::util::DeviceExt;

use super::{state, wgpu_resources::WgpuResources, Shaderer};

pub struct Oscilloscope {
    pub wgpu_resources: WgpuResources,

    render_pipeline: wgpu::RenderPipeline,
    compute_pipeline: wgpu::ComputePipeline,

    state: state::State,
}

#[repr(C)]
#[derive(Pod, Copy, Zeroable, Clone)]
pub struct Vertex(pub [f32; 2]);

impl Oscilloscope {
    fn new(wgpu_resources: WgpuResources) -> Self {
        let state = state::State::new(&wgpu_resources, "music/03 Blocks.wav");
        Self {
            render_pipeline: Oscilloscope::new_render_pipeline(&wgpu_resources, &state),
            compute_pipeline: Oscilloscope::new_compute_pipeline(&wgpu_resources, &state),
            wgpu_resources,
            state,
        }
    }

    fn new_render_pipeline(
        wgpu_resources: &WgpuResources,
        state: &state::State,
    ) -> wgpu::RenderPipeline {
        let WgpuResources { device, config, .. } = wgpu_resources;
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/render.wgsl"))),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&state.uniform_bind_group_layout], // TODO: use these maybe??
            push_constant_ranges: &[],
        });

        let vertex = wgpu::VertexState {
            module: &shader,
            entry_point: "main_vs",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
            }], // example: blah.rs:381
        };

        let fragment = wgpu::FragmentState {
            module: &shader,
            entry_point: "main_fs",
            targets: &[config.format.into()],
        };

        let primitive = wgpu::PrimitiveState {
            // topology: wgpu::PrimitiveTopology::TriangleStrip,
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            // polygon_mode: wgpu::PolygonMode::Line,
            polygon_mode: wgpu::PolygonMode::Fill,
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

    fn new_compute_pipeline(
        wgpu_resources: &WgpuResources,
        state: &state::State,
    ) -> wgpu::ComputePipeline {
        let WgpuResources { device, .. } = wgpu_resources;
        let compute_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/compute.wgsl"))),
        });
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[&state.wav_stream_bind_group_layout],
                push_constant_ranges: &[],
            });
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
        })
    }

    fn cpass(&self, command_encoder: &mut wgpu::CommandEncoder) {
        let compute_pass_descriptor = wgpu::ComputePassDescriptor {
            label: Some("Compute Pass"),
        };

        command_encoder.push_debug_group("Compute Pass");
        {
            let mut cpass = command_encoder.begin_compute_pass(&compute_pass_descriptor);
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &self.state.wav_stream_bind_groups[0], &[]);
            cpass.dispatch(64 as u32, 1, 1);
        }
        command_encoder.pop_debug_group();
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
            rpass.set_vertex_buffer(0, self.state.instance_buffer.slice(..)); // TODO: fill in this buffer
            rpass.set_bind_group(0, &self.state.uniform_bind_group, &[]);
            rpass.draw(0..4, 0..(state::SAMPLE_BUFFER_SIZE as u32)); // NOTE: this is one less than instance_buffer len because the last element wouldn't have a pair
        }
        command_encoder.pop_debug_group();
    }
}

impl Shaderer for Oscilloscope {
    fn new(wgpu_resources: WgpuResources) -> Self {
        Oscilloscope::new(wgpu_resources)
    }

    fn update(&mut self) {
        self.state.update_uniforms();
        self.state.update_instances(&self.wgpu_resources.queue);
        self.state.write_queue(&self.wgpu_resources.queue);
    }

    fn render(&self, view: &wgpu::TextureView) {
        let WgpuResources { device, queue, .. } = &self.wgpu_resources;

        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Command Encoder"),
        });

        // println!("cpass");
        // self.cpass(&mut command_encoder);

        println!("rpass ({})", self.state.frame);
        self.rpass(&mut command_encoder, view);
        queue.submit(Some(command_encoder.finish()));
    }
}
