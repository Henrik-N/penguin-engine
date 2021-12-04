use crate::engine::renderer::vk_context::VkContext;
/// ------------------------- VK COMPONENTS ----------------------------------
// most vulkan components, except for the context
use ash::vk;
use crate::engine::descriptor_sets::DescriptorSet;
use crate::engine::renderer::uniform_buffer::UniformBufferGlobalData;


pub struct VkComponents {
    pub swapchain: Swapchain,
    pub depth_image: DepthImage,
    pub render_pass: RenderPass,
    pub frame_buffers: FrameBuffers,
    pub descriptor_pool: DescriptorPool,
}


pub fn init(context: &VkContext) -> VkComponents {
    log::trace!("Creating swapchain.");
    let swapchain = Swapchain::init(context);
    // ///////////////////////////////////////
    log::trace!("Creating depth image.");
    let depth_image = DepthImage::init(context, &swapchain);
    // ///////////////////////////////////////
    log::trace!("Creating render pass.");
    let render_pass = RenderPass::init(context, &swapchain);
    // ///////////////////////////////////////
    log::trace!("Creating frame buffers.");
    let frame_buffers = FrameBuffers::init(context, &swapchain, &depth_image, &render_pass);
    // ///////////////////////////////////////
    log::trace!("Creating descriptor pool.");
    let descriptor_pool = DescriptorPool::init(context);
    // ///////////////////////////////////////

    VkComponents {
        swapchain,
        depth_image,
        render_pass,
        frame_buffers,
        descriptor_pool,
    }

}





pub struct Swapchain {
    pub loader: ash::extensions::khr::Swapchain,
    pub handle: vk::SwapchainKHR,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
}
impl Swapchain {

    pub fn acquire_next_swapchain_image(&self, semaphore: vk::Semaphore, fence: vk::Fence, timeout: std::time::Duration) -> u32 {
        log::trace!("Acquiring next swapchain image");

        // todo: Handle suboptimal case?
        let (image_index, _is_suboptimal) = unsafe {
            self.loader.acquire_next_image(
                self.handle,
                // timeout 1 sec, specified in nanoseconds
                timeout.as_nanos() as _,
                semaphore,
                fence,
            )
        }.expect("Couldn't acquire next swapchain image");
        log::trace!("Swapchain image {} aquired!", image_index);

        image_index
    }

}



pub struct DepthImage {
    pub image: crate::engine::buffers::AllocatedImage,
    pub image_view: vk::ImageView,
    pub image_format: vk::Format,
}

pub struct RenderPass {
    pub handle: vk::RenderPass,
    pub attachment_count: usize,
}


pub struct DescriptorPool {
    pub descriptor_pool: vk::DescriptorPool,
    //pub global_set_layout: vk::DescriptorSetLayout,
}
impl DescriptorPool {
    const MAX_UNIFORM_BUFFER_COUNT: u32 = 10;
    const MAX_DESCRIPTOR_SET_COUNT: u32 = 10;

    pub fn create_pool(device: &ash::Device) -> Self {
        //let global_set_layout =
        //UniformBufferGlobalData::create_descriptor_set_layout(&device);
        //let global_set_layout = Self::create_global_set_layout(&device);
        let global_set_layout = UniformBufferGlobalData::create_descriptor_set_layout(&device);

        let descriptor_pool_size = [vk::DescriptorPoolSize::builder()
            .descriptor_count(Self::MAX_UNIFORM_BUFFER_COUNT) // 10 uniform buffers
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .build()];

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(Self::MAX_DESCRIPTOR_SET_COUNT)
            .pool_sizes(&descriptor_pool_size);

        let descriptor_pool =
            unsafe { device.create_descriptor_pool(&descriptor_pool_create_info, None) }
                .expect("Couldn't create descriptor pool");

        Self {
            descriptor_pool,
        }
    }

    //fn binding_index() -> u32 {
   //     1_u32
   // }

    //fn create_descriptor_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
    //    let binding_index = Self::binding_index();
    //    descriptor_set_layout(
    //        device,
    //        binding_index,
    //        vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
    //        vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
    //    )
    //}

    //fn descriptor_set_binding_flags() -> vk::DescriptorPoolCreateFlags {
    //    DescriptorSetBindingFrequency::PerFrame.descriptor_flag_bits()
    //}
}



pub struct FrameBuffers {
    pub frame_buffers: Vec<vk::Framebuffer>,
}
impl FrameBuffers {
    pub fn get(&self, image_index: usize) -> vk::Framebuffer {
        self.frame_buffers.get(image_index).expect(&format!("no frame buffer for the given index {}", image_index)).clone()
    }
}

impl DepthImage {
    #[allow(unused)]
    pub fn find_depth_image_format(context: &VkContext) -> vk::Format {
        context
            .find_supported_format(
                &[
                    vk::Format::D32_SFLOAT,
                    vk::Format::D32_SFLOAT_S8_UINT,
                    vk::Format::D24_UNORM_S8_UINT,
                ],
                vk::ImageTiling::OPTIMAL,
                vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
            )
            .expect("couldn't find suitable depth format")
    }
}




mod init_destroy {
    use crate::engine::buffers::AllocatedImage;
    use crate::engine::pe;
    use ash::vk;

    use super::{DepthImage, DescriptorPool, FrameBuffers, RenderPass, Swapchain, VkContext};

    impl Swapchain {
        pub fn destroy(&mut self, context: &VkContext) {
            log::debug!("Destroying swapchain image views");
            self.image_views.iter().for_each(|&image_view| unsafe {
                context.device.handle.destroy_image_view(image_view, None);
            });
        }

        pub fn init(context: &VkContext) -> Self {
            log::trace!("Quering device for swapchain support");
            let swapchain_support_details = pe::device::query_swapchain_support(
                context.physical_device.handle,
                context.surface.handle,
                &context.surface.loader,
            );

            log::trace!("Creating swapchain");
            let swapchain = pe::swapchain::PSwapchain::create(
                &context.instance.handle,
                &context.device.handle,
                context.surface.handle,
                swapchain_support_details,
                context.physical_device.queue_index,
            );

            let pe::swapchain::PSwapchain {
                swapchain_loader,
                swapchain,
                swapchain_format,
                swapchain_extent,
                swapchain_images,
                swapchain_image_views,
            } = swapchain;

            Self {
                loader: swapchain_loader,
                handle: swapchain,
                format: swapchain_format,
                extent: swapchain_extent,
                images: swapchain_images,
                image_views: swapchain_image_views,
            }
        }
    }
    impl DepthImage {
        pub fn destroy(&mut self, context: &VkContext) {
            unsafe {
                context
                    .device
                    .handle
                    .destroy_image_view(self.image_view, None)
            };
            self.image.destroy(&context.device.handle);
        }

        pub fn init(context: &VkContext, swapchain: &Swapchain) -> Self {
            // depth images
            // NOTE: Hardcoded for now, also hardcoded in the render pass
            //let depth_format = vk::Format::D32_SFLOAT;
            let depth_image_format = vk::Format::D16_UNORM;
            //let depth_image_format = Self::find_depth_image_format(&context);
            log::debug!("depth format: {:?}", depth_image_format);

            let depth_image_create_info = vk::ImageCreateInfo::builder()
                .image_type(vk::ImageType::TYPE_2D)
                .format(depth_image_format)
                .extent(vk::Extent3D {
                    width: swapchain.extent.width,
                    height: swapchain.extent.height,
                    depth: 1,
                })
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let depth_image = AllocatedImage::create(
                &context.device.handle,
                context.pd_mem_properties(),
                &depth_image_create_info,
            );

            let image_view_create_info = vk::ImageViewCreateInfo::builder()
                .format(depth_image_format)
                .image(depth_image.image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .subresource_range(
                    vk::ImageSubresourceRange::builder()
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1)
                        .aspect_mask(vk::ImageAspectFlags::DEPTH)
                        .build(),
                );

            let depth_image_view = unsafe {
                context
                    .device
                    .handle
                    .create_image_view(&image_view_create_info, None)
            }
            .expect("Couldn't create depth image view");

            Self {
                image: depth_image,
                image_view: depth_image_view,
                image_format: depth_image_format,
            }
        }
    }
    impl RenderPass {
        pub fn destroy(&mut self, context: &VkContext) {
            unsafe {
                context.device.handle.destroy_render_pass(self.handle, None);
            }
        }

        pub fn init(context: &VkContext, swapchain: &Swapchain) -> Self {
            use crate::engine::pe::render_pass::PRenderPass;

            let (render_pass, attachment_count) =
                PRenderPass::create_default_render_pass(&context.device.handle, &swapchain.format);
            Self {
                handle: render_pass,
                attachment_count,
            }
        }
    }
    impl DescriptorPool {
        pub fn destroy(&mut self, context: &VkContext) {
            unsafe {
                context
                    .device
                    .handle
                    .destroy_descriptor_pool(self.descriptor_pool, None);
            }
        }

        pub fn init(context: &VkContext) -> Self {
            //let global_descriptor_set_layout =
            //UniformBuffer::create_descriptor_set_layout::<UniformBufferGlobalData>(&device);
            //UniformBufferGlobalData::create_descriptor_set_layout(&device);

            let descriptor_pool_size = [
                vk::DescriptorPoolSize::builder()
                    // reserve 1 handle
                    .descriptor_count(10) // 10 uniform buffers
                    .ty(vk::DescriptorType::UNIFORM_BUFFER)
                    .build(),
                vk::DescriptorPoolSize::builder()
                    .descriptor_count(10) // 10 dynamic uniform buffers
                    .ty(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                    .build(),
            ];

            let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::builder()
                .max_sets(10 as u32)
                .pool_sizes(&descriptor_pool_size);

            let descriptor_pool = unsafe {
                context
                    .device
                    .handle
                    .create_descriptor_pool(&descriptor_pool_create_info, None)
            }
            .expect("Couldn't create descriptor pool");

            Self {
                descriptor_pool,
                //global_set_layout: Default::default()
            }
        }
    }
    impl FrameBuffers {
        pub fn destroy(&mut self, context: &VkContext) {
            log::debug!("Destroying frame buffers");
            self.frame_buffers.iter().for_each(|&framebuffer| {
                unsafe { context.device.handle.destroy_framebuffer(framebuffer, None) };
            });
        }

        pub fn init(
            context: &VkContext,
            swapchain: &Swapchain,
            depth_image: &DepthImage,
            render_pass: &RenderPass,
        ) -> Self {
            // frame buffers --------------
            let frame_buffers: Vec<vk::Framebuffer> = swapchain
                .image_views
                .iter()
                .map(|&image_view| {
                    let attachments = [image_view, depth_image.image_view];
                    let create_info = vk::FramebufferCreateInfo::builder()
                        .render_pass(render_pass.handle)
                        .attachments(&attachments)
                        .width(swapchain.extent.width)
                        .height(swapchain.extent.height)
                        .layers(1);

                    unsafe { context.device.handle.create_framebuffer(&create_info, None) }
                        .expect("Couldn't create framebuffer")
                })
                .collect();
            Self { frame_buffers }
        }
    }
}
