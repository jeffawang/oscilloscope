mod oscilloscope;
mod wgpu_resources;

use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use self::wgpu_resources::WgpuResources;

pub fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // TODO: do setup

    let wgpu_resources = wgpu_resources::WgpuResources::new(&window);

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
            WindowEvent::Resized(physical_size) => {
                // TODO
                todo!();
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                // TODO
                todo!()
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
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
            // TODO
        }
        Event::MainEventsCleared => {
            // TODO
        }
        _ => {}
    })
}

pub trait Shaderer<'a> {
    fn new(wgpu_resources: &'a WgpuResources) -> Self;
    fn render(&mut self, view: &wgpu::TextureView);
}
