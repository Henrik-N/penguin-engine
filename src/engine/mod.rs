mod pe;

mod front {
    use ash::vk;

    struct Mesh;

    struct Texture;

    struct Material;

    pub(crate) struct RenderPass;

    struct RenderGraph;

    // configurable pipeline


    impl RenderPass {
        pub fn create_default_render_pass(device: &ash::Device, swapchain_format: &vk::Format) -> (vk::RenderPass, usize) {

            // description of image for writing render commands into
            let render_pass_attachments = [
                // color attachment
                vk::AttachmentDescription::builder()
                    .format(*swapchain_format)
                    // 1 sample, no MSAA
                    .samples(vk::SampleCountFlags::TYPE_1)
                    // clear image on attachment load
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    // store image for being read later
                    .store_op(vk::AttachmentStoreOp::STORE)
                    // no stencil
                    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                    // starting layout doesn't matter
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    // layout ready for display
                    .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                    .build(),
            ];

            let color_attachment_ref = [vk::AttachmentReference::builder()
                .attachment(0)
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .build()]; // layout optimal to be written into by rendering commands


            let subpass = [vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&color_attachment_ref)
                .build()];


            let render_pass_create_info = vk::RenderPassCreateInfo::builder()
                .attachments(&render_pass_attachments)
                .subpasses(&subpass);


            (unsafe {
                device.create_render_pass(&render_pass_create_info, None)
            }.expect("Couldn't create render pass!"),
             render_pass_attachments.len())

            /*
            // let renderpass_attachments = [
            //     // vk::AttachmentDescription {
            //     //     format: swapchain_format,
            //     //     samples: vk::SampleCountFlags::TYPE_1,
            //     //     load_op: vk::AttachmentLoadOp::CLEAR,
            //     //     store_op: vk::AttachmentStoreOp::STORE,
            //     //     final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            //     //     ..Default::default()
            //     // },
            //     // vk::AttachmentDescription {
            //     //     format: vk::Format::D16_UNORM,
            //     //     samples: vk::SampleCountFlags::TYPE_1,
            //     //     load_op: vk::AttachmentLoadOp::CLEAR,
            //     //     initial_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            //     //     final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            //     //     ..Default::default()
            //     // },
            // ];

            // let color_attachment_refs = [vk::AttachmentReference {
            //     attachment: 0,
            //     layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            // }];
            // let depth_attachment_ref = vk::AttachmentReference {
            //     attachment: 1,
            //     layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            // };
            // let dependencies = [vk::SubpassDependency {
            //     src_subpass: vk::SUBPASS_EXTERNAL,
            //     src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            //     dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
            //         | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            //     dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            //     ..Default::default()
            // }];
            //
            // let subpasses = [vk::SubpassDescription::builder()
            //     .color_attachments(&color_attachment_refs)
            //     .depth_stencil_attachment(&depth_attachment_ref)
            //     .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            //     .build()];
            //
            // let render_pass_create_info = vk::RenderPassCreateInfo::builder()
            //     .attachments(&renderpass_attachments)
            //     .subpasses(&subpasses)
            //     .dependencies(&dependencies);
            //
            // unsafe {
            //     device.logical_device
            //         .create_render_pass(&render_pass_create_info, None)
            //         .unwrap()
            // }
             */
        }
    }
}


use ash::vk;
use anyhow::*;
use crate::core;
use self::core::logger;
use pe::device::{SurfaceDetails};
use pe::swapchain::PSwapchain;
use pe::swapchain::SwapchainImage;

/// Per-frame data.
struct FrameData {
    device: ash::Device,
    // gpu->cpu communication, notifies cpu when gpu is done
    queue_submit_fence: vk::Fence,
    command_pool_primary: vk::CommandPool,
    command_buffer_primary: vk::CommandBuffer,
    swapchain_acquire_semaphore: vk::Semaphore,
    swapchain_release_semaphore: vk::Semaphore,
    queue_index: u32,
}

/// Struct containing most Vulkan object handles and global states.
struct Context {
    instance: ash::Instance,
    gpu: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
    device: ash::Device,
    graphics_queue_index: u32,
    graphics_queue_handle: vk::Queue,

    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    swapchain_images: Vec<SwapchainImage>,

    // // imageview for each image in the swapchain

    // framebuffers for each image view
    swapchain_framebuffers: Vec<vk::Framebuffer>,
    // // renderpass description
    render_pass: vk::RenderPass,

    // pipeline: vk::Pipeline,
    // // graphics pipeline
    // pipeline_layout: vk::PipelineLayout,
    // // pipeline layout for resources
    // recycled_semaphores: Vec<vk::Semaphore>,
    // // semaphore objects that can be reused
    // per_frame_data: Vec<FrameData>,
}


pub struct PenguinEngine {
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,
    surface_loader: ash::extensions::khr::Surface,
    context: Context,
    frame_data: Vec<FrameData>,
}


impl PenguinEngine {
    pub fn create(window: &winit::window::Window) -> Result<Self> {
        let entry = unsafe { ash::Entry::new() }?;
        let required_surface_extensions = ash_window::enumerate_required_extensions(window)?;

        log::trace!("Creating Vulkan instance.");
        let instance: ash::Instance = pe::instance::create_ash_instance(&entry, &required_surface_extensions)?;

        let (debug_utils_loader, debug_messenger) = {
            let (loader, messenger) = logger::init::init_vk_debug_messenger(&entry, &instance)?;

            if core::config::DEBUG.is_enabled {
                log::trace!("Initializing vulkan debug messenger.");
                (loader, messenger)
            } else {
                log::trace!("Vulkan debug messenger disabled.");
                (loader, vk::DebugUtilsMessengerEXT::null())
            }
        };


        let surface = unsafe { ash_window::create_surface(&entry, &instance, window, None)? };
        let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);

        log::trace!("Selecting physical device");
        let physical_device = pe::device::select_physical_device(&instance, surface, &surface_loader)?;


        log::trace!("Finding queue family indices");
        let graphics_queue_index = pe::device::find_graphics_queue_family(&instance, physical_device, surface, &surface_loader);

        let graphics_queue_index = match graphics_queue_index {
            Some(queue) => queue,
            None => {
                log::error!("Graphics queue index was None");
                panic!("Graphics queue index was None");
            }
        };


        log::trace!("Creating logical device");
        let device = pe::device::create_logical_device(&instance, physical_device, graphics_queue_index);

        log::trace!("Getting graphics queue handle");
        let graphics_queue_handle: vk::Queue = pe::device::get_graphics_queue_handle(&device, graphics_queue_index);


        log::trace!("Quering device for swapchain support");
        let swapchain_support_details = pe::device::query_swapchain_support(physical_device, surface, &surface_loader);

        log::trace!("Creating swapchain");
        let swapchain = pe::swapchain::PSwapchain::create(&instance, &device, surface, swapchain_support_details, graphics_queue_index);

        let PSwapchain {
            swapchain_loader,
            swapchain,
            swapchain_format,
            swapchain_extent,
            swapchain_images
        } = swapchain;


        // init commnads
        log::trace!("Creating default render pass");
        let (render_pass, attachment_count) = front::RenderPass::create_default_render_pass(&device, &swapchain_format);

        // create framebuffers
        let mut framebuffer_create_info = vk::FramebufferCreateInfo {
            render_pass,
            attachment_count: 1,
            width: core::config::WIDTH,
            height: core::config::HEIGHT,
            layers: 1,
            ..Default::default()
        };

        let swapchain_image_count = swapchain_images.len();


        log::trace!("Creating framebuffers");
        let frame_buffers: Vec<vk::Framebuffer> = swapchain_images.iter().map(|image| {
            let image_view = &[image.image_view];

            let mut framebuffer_create_info = vk::FramebufferCreateInfo {
                render_pass,
                attachment_count: 1,
                p_attachments: image_view.as_ptr(),
                width: core::config::WIDTH,
                height: core::config::HEIGHT,
                layers: 1,
                ..Default::default()
            };

            let frame_buffer = unsafe {
                device.create_framebuffer(&framebuffer_create_info, None)
            }.expect("Couldn't create framebuffer");

            frame_buffer
        }).collect();


        let context = Context {
            instance,
            device,
            gpu: physical_device,
            graphics_queue_index,
            graphics_queue_handle,
            surface,
            swapchain,
            swapchain_loader,
            swapchain_format,
            swapchain_extent,
            swapchain_images,
            render_pass,
            swapchain_framebuffers: frame_buffers,
        };

        Ok(Self {
            debug_utils_loader,
            debug_messenger,
            context,
            surface_loader,
            frame_data: Vec::new(),
        })
    }

    pub fn init_commands(&mut self) {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(self.context.graphics_queue_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER); // allows resetting of individual buffers from this pool

        log::trace!("Creating command pool");
        let command_pool = unsafe {
            self.context.device
                .create_command_pool(&command_pool_create_info, None)
        }.expect("Command pool couldn't be created.");


        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .command_buffer_count(1)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = unsafe { self.context.device.allocate_command_buffers(&command_buffer_allocate_info) }
            .expect("Command buffer couldn't be created");


        let frame_data = FrameData {
            device: self.context.device.clone(),
            queue_submit_fence: Default::default(),
            command_pool_primary: command_pool,
            command_buffer_primary: Default::default(),
            swapchain_acquire_semaphore: Default::default(),
            swapchain_release_semaphore: Default::default(),
            queue_index: self.context.graphics_queue_index,
        };

    }

    pub fn update(&mut self, delta: f32) {
        // queuing commands here
    }


    pub fn render(&mut self, delta: f32) {}

    pub fn shutdown(&mut self) {
        self.frame_data.iter().for_each(|frame_data| {
            unsafe { frame_data.device.destroy_command_pool(frame_data.command_pool_primary, None) };
        })
    }
}

impl Drop for PenguinEngine {
    fn drop(&mut self) {
        unsafe {

            self.context.swapchain_framebuffers.iter().for_each(|&fb| {
                self.context.device.destroy_framebuffer(fb, None);
            });

            self.context.device.destroy_render_pass(self.context.render_pass, None);

            self.context.swapchain_images.iter().for_each(|swapchain_image| {
                self.context.device.destroy_image_view(swapchain_image.image_view, None);
            });

            self.context.swapchain_loader.destroy_swapchain(self.context.swapchain, None);


            self.context.device.destroy_device(None);

            self.surface_loader.destroy_surface(self.context.surface, None);


            if core::config::DEBUG.is_enabled {
                self.debug_utils_loader.destroy_debug_utils_messenger(self.debug_messenger, None);
            }

            self.context.instance.destroy_instance(None);
        }
    }
}
