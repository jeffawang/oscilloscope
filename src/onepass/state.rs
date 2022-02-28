use std::{num::NonZeroU64, time::Instant};

use bytemuck::{Pod, Zeroable};
use itertools::Itertools;

use crate::{ringbuffer::RingBuffer, sound::WavStreamer};

use super::wgpu_resources::{UniformBinder, WavStreamBinder, WgpuResources};

#[repr(C)]
#[derive(Pod, Copy, Zeroable, Clone)]
pub struct Uniforms {
    frame: u32,
    time: f32,
    line_thickness: f32,
    count: f32,
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            frame: 0,
            time: Default::default(),
            line_thickness: 0.0075,
            count: SAMPLE_BUFFER_SIZE as f32,
        }
    }
}

pub struct State {
    pub frame: u32,
    start_time: Instant,
    pub uniforms: Uniforms,

    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    pub uniform_bind_group_layout: wgpu::BindGroupLayout,

    pub sample_buffer_size: usize,
    pub compute_buffer_factor: usize,
    pub compute_buffer_size: usize,
    pub compute_buffer: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
    pub wav_stream_bind_groups: Vec<wgpu::BindGroup>,
    pub wav_stream_bind_group_layout: wgpu::BindGroupLayout,

    wav_streamer: WavStreamer,
    rb: RingBuffer<(f32, f32)>,
}

// TODO: parameterize these
// TODO: Set COMPUTE_BUFFER_FACTOR > 1
pub const SAMPLE_RENDER_COUNT: usize = 32000;
pub const SAMPLE_BUFFER_SIZE: usize = 5 * 44100;
pub const COMPUTE_BUFFER_FACTOR: usize = 1;

impl State {
    pub fn new(wgpu_resources: &WgpuResources, filename: &str) -> Self {
        let uniform_binder = UniformBinder::<Uniforms>::new(wgpu_resources);

        let uniform_buffer = uniform_binder.new_uniform_buffer();
        let uniform_bind_group_layout = uniform_binder.bind_group_layout();
        let uniform_bind_group =
            uniform_binder.bind_group(&uniform_bind_group_layout, &uniform_buffer);

        let wav_streamer = WavStreamer::new(filename);
        let wav_stream_binder = WavStreamBinder::new(
            wgpu_resources,
            &wav_streamer.spec,
            SAMPLE_BUFFER_SIZE,
            COMPUTE_BUFFER_FACTOR,
        );
        let wav_stream_bind_group_layout = wav_stream_binder.bind_group_layout();
        let (compute_buffer, instance_buffer) = wav_stream_binder.new_buffers();
        // TODO
        let wav_stream_bind_groups = wav_stream_binder.bind_groups(
            &wav_stream_bind_group_layout,
            &compute_buffer,
            &instance_buffer,
        );

        let rb = RingBuffer::new(vec![(0.0, 0.0); SAMPLE_BUFFER_SIZE]);

        Self {
            frame: 0,
            uniforms: Uniforms::default(),
            start_time: Instant::now(),
            uniform_buffer,
            uniform_bind_group_layout,
            uniform_bind_group,

            sample_buffer_size: wav_stream_binder.sample_buffer_size,
            compute_buffer_factor: wav_stream_binder.compute_buffer_factor,
            compute_buffer_size: wav_stream_binder.compute_buffer_size,
            compute_buffer,
            instance_buffer,
            wav_stream_bind_groups,
            wav_stream_bind_group_layout,

            wav_streamer,
            rb,
        }
    }

    pub fn update_uniforms(&mut self) {
        self.frame += 1;
        self.uniforms.time = Instant::now().duration_since(self.start_time).as_secs_f32();
        self.uniforms.frame = self.frame;
    }

    pub fn update_instances(&mut self, queue: &wgpu::Queue) {
        // TODO: set sample count somewhere
        // self.wav_streamer
        //     .iter()
        //     .take(samples as usize)
        //     .for_each(|v| self.rb.push(v));

        // let verts = self.rb.iter().map(|(x, y)| [x, y]).collect::<Vec<_>>();
        let samples = SAMPLE_RENDER_COUNT;

        let verts = self
            .wav_streamer
            .iter()
            .take(samples)
            .map(|(x, y)| [x, y])
            .collect_vec();

        queue.write_buffer(&self.compute_buffer, 0, bytemuck::cast_slice(&verts));
    }

    pub fn write_queue(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&self.uniforms));
    }
}
