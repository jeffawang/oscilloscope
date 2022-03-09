use std::{cmp::max, num::NonZeroU64, time::Instant};

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
    time: f32,
    prev_time: f32,
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

    offset: u64,

    wav_streamer: WavStreamer,
    rb: RingBuffer<(i32, i32)>,
}

// TODO: parameterize these
// TODO: Set COMPUTE_BUFFER_FACTOR > 1
pub const SAMPLE_RENDER_COUNT: usize = 128000;
pub const SAMPLE_BUFFER_SIZE: usize = 20000;
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

        let rb = RingBuffer::new(vec![(0, 0); SAMPLE_BUFFER_SIZE]);

        Self {
            frame: 0,
            uniforms: Uniforms::default(),
            start_time: Instant::now(),
            time: 0.0,
            prev_time: 0.0,
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

            offset: 0,

            wav_streamer,
            rb,
        }
    }

    pub fn update_uniforms(&mut self) {
        self.frame += 1;
        self.prev_time = self.time;
        self.time = Instant::now().duration_since(self.start_time).as_secs_f32();
        self.uniforms.time = self.time;
        self.uniforms.frame = self.frame;
    }

    pub fn update_instances(&mut self, queue: &wgpu::Queue) {
        let hz = self.wav_streamer.spec.sample_rate as f32;
        let dt = self.time - self.prev_time;
        let sample_count = (hz * dt) as usize;

        let data = self
            .wav_streamer
            .iter()
            .take(sample_count)
            .map(|(x, y)| [x as f32 / i16::MAX as f32, y as f32 / i16::MAX as f32])
            .collect_vec();

        let curr_offset = self.offset;
        let next_offset = (curr_offset + sample_count as u64) % SAMPLE_BUFFER_SIZE as u64;
        self.offset = next_offset;

        if next_offset < curr_offset {
            println!("====================");
            let cutoff = (SAMPLE_BUFFER_SIZE as u64 - curr_offset) as usize;
            queue.write_buffer(
                &self.instance_buffer,
                curr_offset * std::mem::size_of::<[f32; 2]>() as u64,
                bytemuck::cast_slice(&data[..cutoff]),
            );
            queue.write_buffer(
                &self.instance_buffer,
                0,
                bytemuck::cast_slice(&data[cutoff..]),
            );
        } else {
            queue.write_buffer(
                &self.instance_buffer,
                curr_offset * std::mem::size_of::<[f32; 2]>() as u64,
                bytemuck::cast_slice(&data),
            );
        }
    }

    pub fn write_queue(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&self.uniforms));
    }
}
