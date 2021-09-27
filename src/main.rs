use anyhow::*;
use ash::vk;

mod core;

mod pe;
mod resources;
mod events;

use log::{debug, error, info, trace, warn};


/// Per-frame data.
struct FrameData {
    device: ash::Device,
    // handle
    queue_submit_fence: vk::Fence,
    command_pool_primary: vk::CommandBuffer,
    command_buffer_primary: vk::CommandBuffer,
    swapchain_acquire_semaphore: vk::Semaphore,
    swapchain_release_semaphore: vk::Semaphore,
    queue_index: u32,
}

/// Struct containing most Vulkan object handles and global states.
struct Context {
    instance: ash::Instance,
    surface: vk::SurfaceKHR,
    gpu: vk::PhysicalDevice,
    device: ash::Device,

    // gpu: vk::PhysicalDevice,
    // device: ash::Device,
    // queue_index: Option<u32>,
    // // graphics queue, queue index to submit graphics to
    // swapchain: vk::SwapchainKHR,
    // swapchain_loader: ash::extensions::khr::Swapchain,
    // swapchain_dimensions: SwapchainDimensions,
    // surface: vk::SurfaceKHR,
    // swapchain_image_views: Vec<vk::ImageView>,
    // // imageview for each image in the swapchain
    // swapchain_framebuffers: Vec<vk::Framebuffer>,
    // // framebuffers for each image view
    // render_pass: vk::RenderPass,
    // // renderpass description
    // pipeline: vk::Pipeline,
    // // graphics pipeline
    // pipeline_layout: vk::PipelineLayout,
    // // pipeline layout for resources
    // recycled_semaphores: Vec<vk::Semaphore>,
    // // semaphore objects that can be reused
    // per_frame_data: Vec<FrameData>,
}



use crate::resources::time::{PTime, PTimeResource};

/// Interface with
struct Engine {
    debug_utils_loader: Option<ash::extensions::ext::DebugUtils>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
    surface_fn: ash::extensions::khr::Surface,
    context: Context,
}

use crate::core::{utility, config, logger};

impl Engine {
    fn init_engine_app(window: &winit::window::Window) -> Result<Self> {
        let entry = unsafe { ash::Entry::new() }?;
        let required_surface_extensions = ash_window::enumerate_required_extensions(window)?;

        log::trace!("Creating Vulkan instance.");
        let instance: ash::Instance = pe::instance::create_ash_instance(&entry, &required_surface_extensions)?;

        let (debug_utils_loader, debug_messenger) = if config::DEBUG.is_enabled {
            log::trace!("Initializing vulkan utility messenger.");
            let (loader, messenger) = logger::init::init_vk_debug_messenger(&entry, &instance)?;
            (Some(loader), Some(messenger))
        } else {
            (None, None)
        };

        log::trace!("Creating surface");
        let surface = unsafe { ash_window::create_surface(&entry, &instance, window, None)? };
        let surface_fn = ash::extensions::khr::Surface::new(&entry, &instance);


        log::trace!("Selecting physical device");
        let physical_device = pe::device::init::select_physical_device(&instance, surface, &surface_fn)?;

        let queue_family_indices = pe::device::init::find_queue_families(&instance, physical_device, surface, &surface_fn);

        log::trace!("Creating logical device");
        let device = pe::device::init::create_logical_device(&instance, physical_device, &queue_family_indices);


        // log::trace!("Finding queue families");
        // let queue_family_indices = pe::device::init::find_queue_families(&instance, physical_device, surface, &surface_fn);
        //
        // log::trace!("Creating logical device");
        // let device = pe::device::init::create_logical_device(&instance, physical_device, &queue_family_indices);
        //
        // log::trace!("Initialing queue handles");
        // let (graphics_queue, present_queue) = pe::device::init::get_device_queue_handles(&device, &queue_family_indices);



        let context = Context {
            instance,
            surface,
            gpu: physical_device,
            device,
        };

        Ok(Self {
            debug_utils_loader,
            debug_messenger,
            surface_fn,
            context,
            // surface
        })
    }

    fn render_frame(&mut self, _time: &PTimeResource, _window: &winit::window::Window) {}

    fn destroy(&mut self) {

    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        // unsafe { self.surface_fn.destroy_surface(self.surface, None); };
        // unsafe { self.context.device.destroy_device(None)};
        // unsafe { self.context.instance.destroy_instance(None)};
        log::trace!("Destroying Vulkan Instance"); // apparently it destroys itself
    }
}





use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use crate::pe::instance::create_ash_instance;


// use crate::application::{ApplicationConfig, Application};

mod game_types;
mod entry;
mod ecs;


use crate::core::application::{Application, ApplicationConfig};



use game_types::GameInstance;

fn main() -> Result<()> {

    let config = ApplicationConfig {
        start_x: 100_u16,
        start_y: 100_u16,
        start_width: 1024_u16,
        start_height: 768_u16
    };
    //
    // // request game instance from app
    // let mut game_instance = GameInstance::create(config);
    //
    // let mut app = Application::create(&mut game_instance);
    //
    // // app.start()?;
    //
    // match app.run() {
    //     Ok(()) => log::info!("App exited successfully!"),
    //     Err(e) => log::error!("{}", e),
    // };


    crate::core::logger::init_logger()?;

    // app init
    let event_loop = events::init_event_loop();

    let window = WindowBuilder::new()
        .with_title("penguin engine")
        .with_inner_size(winit::dpi::PhysicalSize::new(config.start_width as u32, config.start_height as u32))
        .build(&event_loop)?;


    // let window = pe::window::init_window(&event_loop);

    // todo: The resources are not supposed to be contained by the systems. Setup ECS!
    let mut time = PTime::create_system();

    let mut app = Engine::init_engine_app(&window)?;


    // let mut destroying = false; // true when started the process of destroying vulkan instances
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            // Window events ------------
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    log::info!("Close requested, closing...");
                    *control_flow = ControlFlow::Exit;
                    app.destroy();
                }
                WindowEvent::KeyboardInput { input, .. } => match input {
                    KeyboardInput {
                        virtual_keycode,
                        state,
                        ..
                    } => match (virtual_keycode, state) {
                        (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                            *control_flow = ControlFlow::Exit
                        }
                        _ => {}
                    },
                },
                _ => {}
            },
            // Other events ------------
            // render next frame if app is not being destroyed
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_window_id) => {
                time.tick();

                app.render_frame(&time.resource(), &window);
            }
            _ => (),
        }
    });







    //let shaders = pe::shaders::Shaders::init();

    // let mut engine = pe::PenguinEngine::init_engine(&window);
    // let device = &engine.device;
    //
    // let pipeline = device.build_graphics_pipeline().build();
    // pipeline.drop(&device);
}
