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
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::swapchain::{Surface, Swapchain, ColorSpace, SupportedPresentModes, PresentMode, SurfaceTransform, CompositeAlpha, FullscreenExclusive, acquire_next_image, AcquireError, SwapchainCreationError};
use vulkano::image::{SwapchainImage, ImageUsage};
use vulkano::format::Format;
use vulkano::sync;
use vulkano::sync::{SharingMode, GpuFuture, FlushError};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::vertex::{BufferlessDefinition, BufferlessVertices, SingleBufferDefinition};
use vulkano::command_buffer::{DynamicState, AutoCommandBuffer, AutoCommandBufferBuilder, SubpassContents};
use vulkano::framebuffer::{RenderPassAbstract, Subpass, FramebufferAbstract, Framebuffer};
use vulkano::single_pass_renderpass;
use vulkano::descriptor::PipelineLayoutAbstract;
use winit::window::{Window, WindowBuilder};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::event::{WindowEvent, Event};
use winit::dpi::LogicalSize;
use vulkano_win::VkSurfaceBuild;

const VALIDATION_LAYERS: &[&str] = &[
    //"VK_LAYER_LUNARG_standard_validation"
    "VK_LAYER_KHRONOS_validation"
];

#[cfg(all(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;

fn main() {
    let vulkan_instance = create_vulkan_instance();
    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .with_title("Vulkan App")
        .with_inner_size(LogicalSize::new(f64::from(WIDTH), f64::from(HEIGHT)))
        .build_vk_surface(&event_loop, vulkan_instance.clone())
        .expect("Failed to create window surface!");
    let _debug_callback = create_debug_callback(&vulkan_instance);
    let physical_device = select_device(&vulkan_instance, &surface);
    let (device, graphics_queue, presentation_queue) = create_logical_device(physical_device);
    let (mut swapchain, swapchain_images) =
        create_swapchain(&surface,
                         physical_device,
                         &device,
                         &graphics_queue,
                         &presentation_queue);
    let render_pass = create_render_pass(&device, swapchain.format());
    let graphics_pipeline = create_graphics_pipeline(&device, swapchain.dimensions(), &render_pass);

    let mut dynamic_state = DynamicState {
        line_width: None,
        viewports: None,
        scissors: None,
        compare_mask: None,
        write_mask: None,
        reference: None,
    };

    let mut swapchain_framebuffers = create_framebuffers(&swapchain_images, &render_pass, &mut dynamic_state);
    let command_buffers = create_command_buffers(&device, &graphics_queue, &swapchain_framebuffers, &graphics_pipeline, &mut dynamic_state);

    let mut need_to_recreate_swapchain = false;
    let mut previous_frame_end = Some(sync::now(device.clone()).boxed());

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed!");
                *control_flow = ControlFlow::Exit
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                need_to_recreate_swapchain = true;
            }
            Event::RedrawEventsCleared => {
                previous_frame_end.as_mut().unwrap().cleanup_finished();

                if need_to_recreate_swapchain {
                    let dimensions: [u32; 2] = surface.window().inner_size().into();
                    let (new_swapchain, new_images) =
                        match swapchain.recreate_with_dimensions(dimensions) {
                            Ok(r) => r,
                            Err(SwapchainCreationError::UnsupportedDimensions) => return,
                            Err(e) => panic!("Failed to recreate swapchain! {:?}", e)
                        };
                    swapchain = new_swapchain;
                    swapchain_framebuffers = create_framebuffers(&new_images, &render_pass.clone(), &mut dynamic_state);
                    need_to_recreate_swapchain = false;
                }

                let (image_index, suboptimal, acquire_future) =
                    match acquire_next_image(swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            need_to_recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image! {:?}", e)
                    };

                let command_buffer = command_buffers[image_index].clone();

                let future = previous_frame_end
                    .take()
                    .expect("Failed to take!")
                    .join(acquire_future)
                    .then_execute(graphics_queue.clone(), command_buffer)
                    .expect("Failed to execute!")
                    .then_swapchain_present(presentation_queue.clone(), swapchain.clone(), image_index)
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => {
                        previous_frame_end = Some(future.boxed());
                    }
                    Err(FlushError::OutOfDate) => {
                        need_to_recreate_swapchain = true;
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }
                    Err(e) => {
                        println!("Failed to flush future: {:?}", e);
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }
                }
            }
            _ => ()
        }
    });


}

fn create_vulkan_instance() -> Arc<Instance> {
    let validation_layers_supported = check_validation_layer_support();
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

    let required_extensions = get_required_instance_extensions();

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

fn select_device<'a>(instance: &'a Arc<Instance>, surface: &'a Arc<Surface<Window>>) -> PhysicalDevice<'a> {
    let device = PhysicalDevice::enumerate(&instance)
        .find(|device| is_vulkan_compatible(device, &surface))
        .expect("Failed to find a Vulkan-compatible device");

    println!(
        "Using device: {} (type: {:?})",
        device.name(),
        device.ty()
    );
    device
}

fn is_vulkan_compatible(device: &PhysicalDevice, surface: &Arc<Surface<Window>>) -> bool {
    for (_, family) in device.queue_families().enumerate() {
        if family.supports_graphics() && surface.is_supported(family).unwrap_or(false) { return true }
    }
    false
}

fn create_logical_device(physical_device: PhysicalDevice) -> (Arc<Device>, Arc<Queue>, Arc<Queue>) {
    let queue_family = physical_device.queue_families().find(|queue| {
        queue.supports_graphics()
    })
        .expect("Couldn't find a graphical queue family!");

    let queue_priority = 1.0;
    let required_extensions = &get_required_device_extensions();

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

fn get_required_device_extensions() -> DeviceExtensions {
    DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::none()
    }
}

fn create_swapchain(
    surface: &Arc<Surface<Window>>,
    physical_device: PhysicalDevice,
    logical_device: &Arc<Device>,
    graphics_queue: &Arc<Queue>,
    present_queue: &Arc<Queue>
) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
    let capabilities = surface.capabilities(physical_device)
        .expect("Failed to get capabilities from device");
    let surface_format = select_swap_surface_format(&capabilities.supported_formats);
    let present_mode = select_swap_present_mode(capabilities.present_modes);
    let extent = select_swap_extent(&surface);

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

fn create_render_pass(device: &Arc<Device>, color_format: Format) -> Arc<dyn RenderPassAbstract + Send + Sync> {
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

type ConcreteGraphicsPipeline = GraphicsPipeline<
    BufferlessDefinition,
    Box<dyn PipelineLayoutAbstract + Send + Sync + 'static>,
    Arc<dyn RenderPassAbstract + Send + Sync + 'static>
>;

fn create_graphics_pipeline(device: &Arc<Device>,
                            swap_chain_extent: [u32; 2],
                            render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>
) -> Arc<ConcreteGraphicsPipeline> {
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

    Arc::new(GraphicsPipeline::start()
        .vertex_input(BufferlessDefinition {})
        .vertex_shader(vert_shader_module.main_entry_point(), ())
        .triangle_list()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(frag_shader_module.main_entry_point(), ())
        .depth_clamp(false)
        .polygon_mode_line()
        .line_width(1.0)
        .cull_mode_back()
        .front_face_clockwise()
        .blend_pass_through()
        .render_pass(Subpass::from(render_pass.clone(), 0)
            .expect("Failed to create subpass!"))
        .build(device.clone())
        .expect("Failed to create graphics pipeline!")
    )
}

fn create_framebuffers(swapchain_images: &[Arc<SwapchainImage<Window>>],
                       render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
                       dynamic_state: &mut DynamicState
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let dimensions = swapchain_images[0].dimensions();

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0..1.0
    };
    dynamic_state.viewports = Some(vec![viewport]);

    swapchain_images.iter()
        .map(|image| {
            Arc::new(Framebuffer::start(render_pass.clone())
                .add(image.clone()).expect("Failed to add image!")
                .build().expect("Failed to build")
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        })
        .collect::<Vec<_>>()
}

fn create_command_buffers(device: &Arc<Device>,
                          graphics_queue: &Arc<Queue>,
                          swapchain_framebuffers: &Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
                          graphics_pipeline: &Arc<ConcreteGraphicsPipeline>,
                          dynamic_state: &mut DynamicState
) -> Vec<Arc<AutoCommandBuffer>> {
    let queue_family = graphics_queue.family();
    swapchain_framebuffers.iter()
        .map(|framebuffer| {
            let vertices = BufferlessVertices { vertices: 3, instances: 1 };
            let mut builder = AutoCommandBufferBuilder::primary_simultaneous_use(device.clone(), queue_family)
                .expect("Failed to create auto command buffer builder");
            builder
                .begin_render_pass(framebuffer.clone(),
                                   SubpassContents::Inline,
                                   vec![[0.0, 0.0, 0.0, 1.0].into()])
                .expect("Failed to begin render pass!")
                .draw(graphics_pipeline.clone(), dynamic_state, vertices, (), ())
                .expect("Failed to draw!")
                .end_render_pass()
                .expect("Failed to end render pass!");
            Arc::new(builder.build()
                .expect("Failed to build auto command buffer")
            )
        })
        .collect()
}
