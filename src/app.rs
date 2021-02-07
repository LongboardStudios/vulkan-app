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
use vulkano::swapchain::{Surface, Swapchain, ColorSpace, SupportedPresentModes, PresentMode, Capabilities, SurfaceTransform, CompositeAlpha, FullscreenExclusive};
use winit::window::Window;
use vulkano::image::{SwapchainImage, ImageUsage};
use vulkano::format::Format;
use vulkano::sync::SharingMode;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::vertex::BufferlessDefinition;
use vulkano::command_buffer::DynamicState;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::single_pass_renderpass;

const VALIDATION_LAYERS: &[&str] = &[
    //"VK_LAYER_LUNARG_standard_validation"
    "VK_LAYER_KHRONOS_validation"
];

#[cfg(all(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

#[allow(unused)]
pub struct App<'a> {
    vulkan_instance: &'a Arc<Instance>,
    debug_callback: Option<DebugCallback>,
    physical_device: PhysicalDevice<'a>,
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    presentation_queue: Arc<Queue>,
    swapchain: Arc<Swapchain<Window>>,
    swapchain_images: Vec<Arc<SwapchainImage<Window>>>,
    render_pass: Arc<RenderPassAbstract + Send + Sync>
}

impl<'a> App<'a> {
    pub fn new(vulkan_instance: &'a Arc<Instance>, surface: &'a Arc<Surface<Window>>) -> Self {
        let debug_callback = Self::create_debug_callback(vulkan_instance);
        let physical_device = Self::select_device(vulkan_instance, surface);
        let (device, graphics_queue, presentation_queue) = Self::create_logical_device(vulkan_instance, physical_device);
        let (swapchain, swapchain_images) =
            Self::create_swapchain(vulkan_instance,
                                   surface,
                                   physical_device,
                                   &device,
                                   &graphics_queue,
                                   &presentation_queue);
        let render_pass = Self::create_render_pass(&device, swapchain.format());
        Self::create_graphics_pipeline(&device, swapchain.dimensions());

        Self {
            vulkan_instance: &vulkan_instance,
            debug_callback,
            physical_device,
            device,
            graphics_queue,
            presentation_queue,
            swapchain,
            swapchain_images,
            render_pass
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

        let required_extensions = Self::get_required_instance_extensions();

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

    fn get_required_instance_extensions() -> InstanceExtensions {
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

    fn select_device(instance: &'a Arc<Instance>, surface: &'a Arc<Surface<Window>>) -> PhysicalDevice<'a> {
        let device = PhysicalDevice::enumerate(&instance)
            .find(|device| Self::is_vulkan_compatible(device, &surface))
            .expect("Failed to find a Vulkan-compatible device");

        println!(
            "Using device: {} (type: {:?})",
            device.name(),
            device.ty()
        );
        device
    }

    fn is_vulkan_compatible(device: &PhysicalDevice, surface: &'a Arc<Surface<Window>>) -> bool {
        for (_, family) in device.queue_families().enumerate() {
            if family.supports_graphics() && surface.is_supported(family).unwrap_or(false) { return true }
        }
        false
    }

    fn create_logical_device(instance: &Arc<Instance>, physical_device: PhysicalDevice) -> (Arc<Device>, Arc<Queue>, Arc<Queue>) {
        let queue_family = physical_device.queue_families().find(|queue| {
            queue.supports_graphics()
        })
        .expect("Couldn't find a graphical queue family!");

        let queue_priority = 1.0;
        let required_extensions = &Self::get_required_device_extensions(&physical_device);

        let (device, mut queues) = Device::new(
            physical_device,
            physical_device.supported_features(),
            required_extensions,
            [(queue_family, queue_priority)].iter().cloned())
            .expect("Failed to create logical device!");

        let graphics_queue = queues.next().unwrap();
        let presentation_queue = queues.next().unwrap_or_else(|| graphics_queue.clone());
        (device, graphics_queue, presentation_queue)
    }

    fn get_required_device_extensions(physical_device: &PhysicalDevice) -> DeviceExtensions {
        DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        }
    }

    fn create_swapchain(
        instance: &Arc<Instance>,
        surface: &Arc<Surface<Window>>,
        physical_device: PhysicalDevice,
        logical_device: &Arc<Device>,
        graphics_queue: &Arc<Queue>,
        present_queue: &Arc<Queue>
    ) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
        let capabilities = surface.capabilities(physical_device)
            .expect("Failed to get capabilities from device");
        let surface_format = Self::select_swap_surface_format(&capabilities.supported_formats);
        let present_mode = Self::select_swap_present_mode(capabilities.present_modes);
        let extent = Self::select_swap_extent(&surface);

        let mut image_count = capabilities.min_image_count + 1;
        let max_image_count = capabilities.max_image_count.expect("Failed to get max image count!");
        if image_count > max_image_count {
            image_count = max_image_count;
        }

        let image_usage = ImageUsage {
            color_attachment: true,
            .. ImageUsage::none()
        };

        let sharing_mode: SharingMode = vec![graphics_queue, present_queue].as_slice().into();

        let (swapchain, images) = Swapchain::new(
            logical_device.clone(),
            surface.clone(),
            image_count,
            surface_format.0,
            extent,
            1,
            image_usage,
            sharing_mode,
            SurfaceTransform::Identity,
            CompositeAlpha::Opaque,
            present_mode,
            FullscreenExclusive::Default,
            true,
            surface_format.1
        ).expect("Failed to create swapchain!");

        (swapchain, images)
    }

    fn select_swap_surface_format(formats: &[(Format, ColorSpace)]) -> (Format, ColorSpace) {
        *formats.iter().find(|(format, color_space)|
            *format == Format::B8G8R8A8Srgb && *color_space == ColorSpace::SrgbNonLinear
        ).unwrap_or_else(|| &formats.first().expect("No surface formats found!"))
    }

    fn select_swap_present_mode(available_modes: SupportedPresentModes) -> PresentMode {
        if available_modes.mailbox {
            PresentMode::Mailbox
        } else if available_modes.immediate {
            PresentMode::Immediate
        } else {
            PresentMode::Fifo
        }
    }

    fn select_swap_extent(surface: &Arc<Surface<Window>>) -> [u32; 2] {
        surface.window().inner_size().into()
    }

    fn create_graphics_pipeline(device: &Arc<Device>, swap_chain_extent: [u32; 2]) {
        mod vertex_shader {
            vulkano_shaders::shader! {
                ty: "vertex",
                path: "src/static_triangle.vert"
            }
        }

        mod fragment_shader {
            vulkano_shaders::shader! {
                ty: "fragment",
                path: "src/vertex_colors.frag"
            }
        }

        let vert_shader_module = vertex_shader::Shader::load(device.clone())
            .expect("Failed to create vertex shader module!");

        let frag_shader_module = fragment_shader::Shader::load(device.clone())
            .expect("Failed to create fragment shader module!");

        let dimensions = [swap_chain_extent[0] as f32, swap_chain_extent[1] as f32];
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions,
            depth_range: 0.0 .. 1.0
        };

        let _pipeline_builder = Arc::new(GraphicsPipeline::start()
            .vertex_input(BufferlessDefinition {})
            .vertex_shader(vert_shader_module.main_entry_point(), ())
            .triangle_list()
            .primitive_restart(false)
            .viewports(vec![viewport])
            .fragment_shader(frag_shader_module.main_entry_point(), ())
            .depth_clamp(false)
            .polygon_mode_fill()
            .line_width(1.0)
            .cull_mode_back()
            .front_face_clockwise()
            .blend_pass_through()
        );

        let mut dynamic_state = DynamicState {
            line_width: None,
            viewports: None,
            scissors: None,
            compare_mask: None,
            write_mask: None,
            reference: None
        };
    }

    fn create_render_pass(device: &Arc<Device>, color_format: Format) -> Arc<RenderPassAbstract + Send + Sync> {
        Arc::new(single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: color_format,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        ).expect("Failed to create render pass!"))
    }
}
