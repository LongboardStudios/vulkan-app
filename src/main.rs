mod app;

use crate::app::App;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder}
};
use vulkano_win::VkSurfaceBuild;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

fn main() {
    let vulkan_instance = App::create_vulkan_instance();

    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .with_title("Vulkan App")
        .with_inner_size(LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT)))
        .build_vk_surface(&event_loop, vulkan_instance.clone())
        .expect("Failed to create window surface!");

    let mut app = App::new(&vulkan_instance, &surface);

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
                &app.draw_frame();
            },
            _ => ()
        }
    });
}
