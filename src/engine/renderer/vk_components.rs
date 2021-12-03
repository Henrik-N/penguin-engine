/// ------------------------- VK COMPONENTS ----------------------------------

// most vulkan components, except for the context

use ash::vk;
use crate::engine::renderer::vk_context::VkContext;

pub struct Swapchain {
    pub loader: ash::extensions::khr::Swapchain,
    pub handle: vk::SwapchainKHR,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
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
    pub global_descriptor_pool: vk::DescriptorPool,
}

pub struct FrameBuffers {
    pub frame_buffers: Vec<vk::Framebuffer>,
}



impl DepthImage {
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
    use ash::vk;
    use crate::engine::buffers::AllocatedImage;
    use crate::engine::pe;

    use super::{
        VkContext,
        RenderPass,
        Swapchain,
        DepthImage,
        DescriptorPool,
        FrameBuffers};


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
            unsafe { context.device.handle.destroy_descriptor_pool(self.global_descriptor_pool, None); }
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

            let global_descriptor_pool =
                unsafe { context.device.handle.create_descriptor_pool(&descriptor_pool_create_info, None) }
                    .expect("Couldn't create descriptor pool");

            Self { global_descriptor_pool }
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