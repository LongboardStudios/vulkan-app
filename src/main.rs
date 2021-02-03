mod app;

use crate::app::App;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder},
};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

fn main() {
    let event_loop = EventLoop::new();
    let _window = WindowBuilder::new()
        .with_title("Vulkan")
        .with_inner_size(LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT)))
        .build(&event_loop);

    let _app = App::new();

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
