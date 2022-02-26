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
