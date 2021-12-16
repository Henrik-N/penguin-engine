use ash::vk;
use crate::renderer::memory::AllocatedImage;
use crate::renderer::vk_types::{Swapchain, VkContext};

pub struct DepthImage {
    pub image: AllocatedImage,
    pub image_view: vk::ImageView,
    pub image_format: vk::Format,
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

impl DepthImage {
    pub fn destroy(&mut self, context: &VkContext) {
        unsafe {context.device.destroy_image_view(self.image_view, None)};
        self.image.destroy(&context);
    }

    pub fn init(context: &VkContext, swapchain: &Swapchain) -> Self {
        // depth images
        // NOTE: Hardcoded for now, also hardcoded in the render pass
        //let depth_format = vk::Format::D32_SFLOAT;
        let depth_image_format = vk::Format::D16_UNORM;
        //let depth_image_format = Self::find_depth_image_format(&context);
        log::debug!("depth format: {:?}", depth_image_format);

        let depth_image = AllocatedImage::create(
            context,
            vk::ImageCreateInfo::builder()
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
                .sharing_mode(vk::SharingMode::EXCLUSIVE),
        );

        let image_view_create_info = vk::ImageViewCreateInfo::builder()
            .image(depth_image.handle)
            .format(depth_image_format)
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

        let depth_image_view= unsafe {
            context.device.create_image_view(&image_view_create_info, None)
        }.expect("couldn't create image view");

        Self {
            image: depth_image,
            image_view: depth_image_view,
            image_format: depth_image_format,
        }
    }
}
