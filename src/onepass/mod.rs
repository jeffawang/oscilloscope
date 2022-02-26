mod oscilloscope;
mod wgpu_resources;

use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use self::{oscilloscope::Oscilloscope, wgpu_resources::WgpuResources};

pub fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let wgpu_resources = wgpu_resources::WgpuResources::new(&window);

    let mut oscilloscope = Oscilloscope::new(wgpu_resources);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(_physical_size) => {
                // TODO
                todo!();
            }
            WindowEvent::ScaleFactorChanged {
                new_inner_size: _, ..
            } => {
                // TODO
                todo!()
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: _,
                        virtual_keycode: Some(VirtualKeyCode::Space),
                        ..
                    },
                ..
            } => {
                // TODO
                todo!();
            }
            _ => {}
        },
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            let WgpuResources {
                surface,
                device,
                config,
                ..
            } = &oscilloscope.wgpu_resources;
            let frame = match surface.get_current_texture() {
                Ok(frame) => frame,
                Err(_) => {
                    surface.configure(&device, &config);
                    surface
                        .get_current_texture()
                        .expect("Failed to acquire next surface texture!")
                }
            };
            let view = &frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            oscilloscope.render(view);
            frame.present();
        }
        Event::MainEventsCleared => {
            // TODO
        }
        _ => {}
    })
}

pub trait Shaderer {
    fn new(wgpu_resources: WgpuResources) -> Self;
    fn render(&mut self, view: &wgpu::TextureView);
}