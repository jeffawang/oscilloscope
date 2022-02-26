use std::time::Instant;

use bytemuck::{Pod, Zeroable};

use crate::{ringbuffer::RingBuffer, sound::WavStreamer};

use super::wgpu_resources::{UniformBinder, WgpuResources};

#[repr(C)]
#[derive(Pod, Copy, Zeroable, Clone)]
pub struct Uniforms {
    time: f32,
    line_thickness: f32,
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            time: Default::default(),
            line_thickness: 0.01,
        }
    }
}

pub struct State {
    start_time: Instant,
    pub uniforms: Uniforms,

    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    pub uniform_bind_group_layout: wgpu::BindGroupLayout,

    wav_streamer: WavStreamer,
    rb: RingBuffer<(f32, f32)>,
}

pub const SAMPLES: usize = 2 * 44100;

impl State {
    pub fn new(wgpu_resources: &WgpuResources, filename: &str) -> Self {
        let uniform_binder = UniformBinder::<Uniforms>::new(wgpu_resources);

        let uniform_buffer = uniform_binder.new_uniform_buffer();
        let uniform_bind_group_layout = uniform_binder.bind_group_layout();
        let uniform_bind_group =
            uniform_binder.bind_group(&uniform_bind_group_layout, &uniform_buffer);

        let wav_streamer = WavStreamer::new(filename);
        let rb = RingBuffer::new(vec![(0.0, 0.0); SAMPLES]);

        Self {
            uniforms: Uniforms::default(),
            start_time: Instant::now(),
            uniform_buffer,
            uniform_bind_group_layout,
            uniform_bind_group,

            wav_streamer,
            rb,
        }
    }

    pub fn update_uniforms(&mut self) {
        self.uniforms.time = Instant::now().duration_since(self.start_time).as_secs_f32();
    }

    pub fn update_instances(&mut self, queue: &wgpu::Queue, instance_buffer: &wgpu::Buffer) {
        let samples = 22000;
        self.wav_streamer
            .iter()
            .take(samples as usize)
            .for_each(|v| self.rb.push(v));

        let verts = self
            .rb
            .iter()
            .map(|(x, y)| super::oscilloscope::Vertex([x, y]))
            .collect::<Vec<_>>();

        queue.write_buffer(&instance_buffer, 0, bytemuck::cast_slice(&verts));
    }

    pub fn write_queue(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&self.uniforms));
    }
}
