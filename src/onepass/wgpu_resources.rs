use std::{marker::PhantomData, mem};

use wgpu::util::DeviceExt;
use winit::window::Window;

/// WgpuResources holds the information needed to set up shader pipeline and whatnot.
pub struct WgpuResources {
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl WgpuResources {
    pub fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap(); // TODO: handle this unwrap

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Device Descriptor"),
                features: wgpu::Features::POLYGON_MODE_LINE,
                limits: wgpu::Limits::default(),
            },
            None,
        ))
        .unwrap(); // TODO: handle this unwrap

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(), // TODO: handle this unwrap
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &config);

        WgpuResources {
            surface,
            config,
            adapter,
            device,
            queue,
        }
    }
    pub fn frame(&self) -> (wgpu::SurfaceTexture) {
        match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => {
                self.surface.configure(&self.device, &self.config);
                self.surface
                    .get_current_texture()
                    .expect("Failed to acquire next surface texture!")
            }
        }
    }
}

pub struct UniformBinder<'a, T> {
    uniform_type: PhantomData<T>,
    wgpu_resources: &'a WgpuResources,
}

impl<'a, T> UniformBinder<'a, T> {
    pub fn new(wgpu_resources: &'a WgpuResources) -> Self {
        Self {
            uniform_type: PhantomData,
            wgpu_resources,
        }
    }

    pub fn bind_group_layout(&self) -> wgpu::BindGroupLayout {
        self.wgpu_resources
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX, // TODO: parameterize this?
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(mem::size_of::<T>() as _),
                    },
                    count: None,
                }],
            })
    }

    pub fn bind_group(
        &self,
        layout: &wgpu::BindGroupLayout,
        uniform_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        self.wgpu_resources
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Uniform BindGroup"),
                layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
            })
    }

    pub fn new_uniform_buffer(&self) -> wgpu::Buffer {
        self.wgpu_resources
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("Uniform Buffer"),
                mapped_at_creation: false,
                size: std::mem::size_of::<T>() as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
    }
}
