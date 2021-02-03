extern crate vulkano;
extern crate winit;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    dpi::LogicalSize,
};
use vulkano::instance::{Instance, InstanceExtensions, ApplicationInfo, Version};
use std::sync::Arc;

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;

struct App {
    event_loop: EventLoop<()>,
    instance: Arc<Instance>,
}

impl App {
    pub fn new() -> Self {
        let instance = Self::create_vulkan_instance();
        let event_loop = Self::create_event_loop();

        Self {
            event_loop,
            instance
        }
    }

    fn create_event_loop() -> EventLoop<()> {
        let event_loop = EventLoop::new();
        let _window = WindowBuilder::new()
            .with_title("Vulkan")
            .with_inner_size(LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT)))
            .build(&event_loop);
        event_loop
    }

    fn create_vulkan_instance() -> Arc<Instance> {
        let supported_extensions = InstanceExtensions::supported_by_core()
            .expect("Failed to retrieve supported extensions");
        println!("Supported extensions: {:?}", supported_extensions);

        let app_info = ApplicationInfo {
            application_name: Some("Vulkan demo".into()),
            application_version: Some( Version { major: 0, minor: 1, patch: 0}),
            engine_name: None,
            engine_version: None,
        };

        let required_extensions = vulkano_win::required_extensions();
        Instance::new(Some(&app_info), &required_extensions, None)
            .expect("Failed to created Vulkan instance")
    }

}

fn main() {
    let app = App::new();
    app.event_loop.run(move |event, _, control_flow| {
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

