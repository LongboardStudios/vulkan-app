extern crate vulkano;
extern crate winit;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit::window::Window;

const WIDTH: u32 = 800;
const SIZE: u32 = 600;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed!");
                *control_flow = ControlFlow::Exit
            },
            Event::MainEventsCleared => {
                // render here
            },
            _ => ()
        }

    });
}
