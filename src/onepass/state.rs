use std::time::Instant;

use bytemuck::{Pod, Zeroable};

use super::wgpu_resources::{UniformBinder, WgpuResources};

#[repr(C)]
#[derive(Pod, Copy, Zeroable, Clone)]
pub struct Uniforms {
    time: f32,
}

pub struct State {
    start_time: Instant,
    pub uniforms: Uniforms,

    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    pub uniform_bind_group_layout: wgpu::BindGroupLayout,
}

impl State {
    pub fn new(wgpu_resources: &WgpuResources) -> Self {
        let uniform_binder = UniformBinder::<Uniforms>::new(wgpu_resources);

        let uniform_buffer = uniform_binder.new_uniform_buffer();
        let uniform_bind_group_layout = uniform_binder.bind_group_layout();
        let uniform_bind_group =
            uniform_binder.bind_group(&uniform_bind_group_layout, &uniform_buffer);

        Self {
            uniforms: Uniforms { time: 0.0 },
            start_time: Instant::now(),
            uniform_buffer,
            uniform_bind_group_layout,
            uniform_bind_group,
        }
    }

    pub fn update(&mut self) {
        self.uniforms.time = Instant::now().duration_since(self.start_time).as_secs_f32();
    }

    pub fn write_queue(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&self.uniforms));
    }
}
