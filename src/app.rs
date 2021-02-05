use std::sync::Arc;

use vulkano::instance::{
    Instance,
    InstanceExtensions,
    ApplicationInfo,
    Version,
    layers_list,
    debug::{
        DebugCallback,
        MessageType,
        MessageSeverity
    },
    PhysicalDevice
};
use vulkano::device::{Device, Features, DeviceExtensions, Queue};

const VALIDATION_LAYERS: &[&str] = &[
    "VK_LAYER_KHRONOS_validation"
];

#[cfg(all(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

#[allow(unused)]
pub struct App<'a> {
    vulkan_instance: &'a Arc<Instance>,
    debug_callback: Option<DebugCallback>,
    physical_device: PhysicalDevice<'a>,
    device: Arc<Device>,
    graphics_queue: Arc<Queue>
}

impl<'a> App<'a> {
    pub fn new(vulkan_instance: &'a Arc<Instance>) -> Self {
        let debug_callback = Self::create_debug_callback(vulkan_instance);
        let physical_device = Self::select_device(vulkan_instance);
        let (device, graphics_queue) = Self::create_logical_device(vulkan_instance, physical_device);

        Self {
            vulkan_instance: &vulkan_instance,
            debug_callback,
            physical_device,
            device,
            graphics_queue
        }
    }

    pub fn create_vulkan_instance() -> Arc<Instance> {
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

    fn select_device(instance: &'a Arc<Instance>) -> PhysicalDevice<'a> {
        let device = PhysicalDevice::enumerate(&instance)
            .find(|device| Self::is_vulkan_compatible(device))
            .expect("Failed to find a Vulkan-compatible device");

        println!(
            "Using device: {} (type: {:?})",
            device.name(),
            device.ty()
        );
        device
    }

    fn is_vulkan_compatible(device: &PhysicalDevice) -> bool {
        for (_, family) in device.queue_families().enumerate() {
            if family.supports_graphics() { return true }
        }
        false
    }

    fn create_logical_device(instance: &Arc<Instance>, physical_device: PhysicalDevice) -> (Arc<Device>, Arc<Queue>) {
        let queue_family = physical_device.queue_families().find(|queue| {
            queue.supports_graphics()
        })
        .unwrap();
        let queue_priority = 1.0;

        let (device, mut queues) = Device::new(
            physical_device,
            &Features::none(),
            &DeviceExtensions::none(),
            [(queue_family, queue_priority)].iter().cloned())
            .expect("Failed to create logical device!");

        let graphics_queue = queues.next().unwrap();
        (device, graphics_queue)
    }
}
