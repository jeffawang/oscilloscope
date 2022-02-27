mod oscilloscope;
mod state;
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

    let mut paused = false;

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
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Space),
                        ..
                    },
                ..
            } => {
                paused = !paused;
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

            let frame = oscilloscope.wgpu_resources.frame();
            let view = &frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            oscilloscope.update();
            oscilloscope.render(view);
            frame.present();
        }
        Event::MainEventsCleared => {
            if !paused {
                window.request_redraw();
            }
        }
        _ => {}
    })
}

pub trait Shaderer {
    fn new(wgpu_resources: WgpuResources) -> Self;
    fn update(&mut self);
    fn render(&self, view: &wgpu::TextureView);
}
