use vulkano::instance::{Instance, InstanceExtensions, ApplicationInfo, Version};
use std::sync::Arc;

pub struct App {
    instance: Arc<Instance>,
}

impl App {
    pub fn new() -> Self {
        let instance = Self::create_vulkan_instance();

        Self {
            instance
        }
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
