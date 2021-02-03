use vulkano::instance::{
    Instance,
    InstanceExtensions,
    ApplicationInfo,
    Version,
    layers_list,
    debug::{DebugCallback, MessageType}
};
use std::sync::Arc;
use vulkano::instance::debug::MessageSeverity;

const VALIDATION_LAYERS: &[&str] = &[
    "VK_LAYER_KHRONOS_validation"
];

#[cfg(all(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

pub struct App {
    instance: Arc<Instance>,
    debug_callback: Option<DebugCallback>
}

impl App {
    pub fn new() -> Self {
        let instance = Self::create_vulkan_instance();
        let debug_callback = Self::create_debug_callback(&instance);

        Self {
            instance,
            debug_callback
        }
    }

    fn create_vulkan_instance() -> Arc<Instance> {
        let validation_layers_supported = Self::check_validation_layer_support();
        if ENABLE_VALIDATION_LAYERS && !validation_layers_supported {
            println!("Validation layers requested but not available!")
        }

        let supported_extensions = InstanceExtensions::supported_by_core()
            .expect("Failed to retrieve supported extensions");
        println!("Supported extensions: {:?}", supported_extensions);

        let app_info = ApplicationInfo {
            application_name: Some("Vulkan demo".into()),
            application_version: Some( Version { major: 0, minor: 1, patch: 0}),
            engine_name: None,
            engine_version: None
        };

        let required_extensions = Self::get_required_extensions();

        if ENABLE_VALIDATION_LAYERS && validation_layers_supported {
            Instance::new(Some(&app_info), &required_extensions, VALIDATION_LAYERS.iter().cloned())
                .expect("Failed to created Vulkan instance")
        } else {
            Instance::new(Some(&app_info), &required_extensions, None)
                .expect("Failed to created Vulkan instance")
        }
    }

    fn check_validation_layer_support() -> bool {
        let layers: Vec<_> = layers_list().unwrap().map(|item| item.name().to_owned()).collect();
        println!("Validation layers supported: {:?}", layers);
        VALIDATION_LAYERS.iter()
            .all(|layer_name| layers.contains(&layer_name.to_string()))
    }

    fn get_required_extensions() -> InstanceExtensions {
        let mut required_extensions = vulkano_win::required_extensions();
        if ENABLE_VALIDATION_LAYERS {
            required_extensions.ext_debug_utils = true;
        }
        required_extensions
    }

    fn create_debug_callback(instance: &Arc<Instance>) -> Option<DebugCallback> {
        if !ENABLE_VALIDATION_LAYERS {
            return None;
        }

        let msg_types = MessageType::all();
        let severity = MessageSeverity {
            error: true,
            warning: true,
            information: true,
            verbose: true
        };
        DebugCallback::new(&instance, severity,msg_types, |msg| {
            println!("Validation layer: {:?}", msg.description);
        }).ok()
    }
}
