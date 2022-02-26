use super::{wgpu_resources::WgpuResources, Shaderer};

pub struct Oscilloscope<'a> {
    wgpu_resources: &'a WgpuResources,
}

impl<'a> Shaderer<'a> for Oscilloscope<'a> {
    fn new(wgpu_resources: &'a WgpuResources) -> Oscilloscope<'a> {
        Self { wgpu_resources }
    }

    fn render(&mut self, view: &wgpu::TextureView) {}
}
