use crate::renderer::vk_types::VkContext;
use ash::vk;

pub struct Swapchain {
    pub loader: ash::extensions::khr::Swapchain,
    pub handle: vk::SwapchainKHR,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
}

impl Swapchain {
    pub fn acquire_next_swapchain_image(
        &self,
        signal_semaphore: vk::Semaphore,
        signal_fence: vk::Fence,
        timeout: std::time::Duration,
    ) -> u32 {
        log::trace!("Acquiring next swapchain image");

        // todo: Handle suboptimal case?
        let (image_index, _is_suboptimal) = unsafe {
            self.loader.acquire_next_image(
                self.handle,
                // timeout 1 sec, specified in nanoseconds
                timeout.as_nanos() as _,
                signal_semaphore,
                signal_fence,
            )
        }
        .expect("Couldn't acquire next swapchain image");
        log::trace!("Swapchain image {} aquired!", image_index);

        image_index
    }
}

impl Swapchain {
    pub fn destroy(&mut self, context: &VkContext) {
        log::debug!("Destroying swapchain image views");
        self.image_views.iter().for_each(|&image_view| unsafe {
            context.device.handle.destroy_image_view(image_view, None);
        });
    }

    pub fn init(window: &penguin_app::window::Window, context: &VkContext) -> Self {
        log::trace!("Querying device for swapchain support");
        let swapchain_support_details = context
            .physical_device
            .query_swapchain_support(&context.surface);

        log::trace!("Creating swapchain");
        init::init_swapchain(
            context,
            swapchain_support_details,
            window.dimensions.width,
            window.dimensions.height,
        )
    }
}

mod init {
    use crate::renderer::vk_types::{Swapchain, SwapchainSupportDetails, VkContext};
    use ash::vk;

    pub(crate) fn init_swapchain(
        context: &VkContext,
        swapchain_support_details: SwapchainSupportDetails,
        window_width: u32,
        window_height: u32,
    ) -> Swapchain {
        // Physical device swapchain support info

        // Decide how many images we should have in the swapchain
        // At least the minimum value + 1, since using the minimum value can mean having to wait for the driver to complete operations before providing an image to render == lag..
        let mut image_count = swapchain_support_details
            .surface_capabilities
            .min_image_count
            + 1;

        // Clamp value
        // A value of 0 means that there is no limit on the number of images
        if swapchain_support_details
            .surface_capabilities
            .max_image_count
            > 0
            && image_count
                < swapchain_support_details
                    .surface_capabilities
                    .max_image_count
        {
            image_count = swapchain_support_details
                .surface_capabilities
                .max_image_count; // if not exceeding the maximum, set the maximum
        }

        let surface_format =
            select_swapchain_surface_format(&swapchain_support_details.surface_color_formats);

        let present_mode =
            select_swapchain_present_mode(&swapchain_support_details.surface_present_modes);
        let extent = select_swapchain_extent(
            &swapchain_support_details.surface_capabilities,
            window_width,
            window_height,
        );

        let (image_sharing_mode, _queue_family_index_count, queue_family_indices) =
            // Use exclusive mode if same, as it is more performant
            (vk::SharingMode::EXCLUSIVE, 1, vec![context.physical_device.graphics_queue_index]);

        // let graphics_queue_family_index = [graphics_queue_index];

        // Swapchain create info
        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(context.surface.handle)
            .min_image_count(image_count as u32)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_indices) // NOTE: Seems like this entry almost isn't needed?
            .pre_transform(
                swapchain_support_details
                    .surface_capabilities
                    .current_transform,
            )
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let swapchain_loader =
            ash::extensions::khr::Swapchain::new(&context.instance.handle, &context.device.handle);

        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&create_info, None)
                .expect("Couldn't create swapchain.")
        };

        let swapchain_images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("Coudln't get swapchain images")
        };

        let swapchain_image_views =
            create_swapchain_images(&context.device, surface_format.format, &swapchain_images);

        Swapchain {
            loader: swapchain_loader,
            handle: swapchain,
            format: surface_format.format,
            extent: extent,
            images: swapchain_images,
            image_views: swapchain_image_views,
        }
    }

    fn select_swapchain_surface_format(
        available_surface_formats: &Vec<vk::SurfaceFormatKHR>,
    ) -> vk::SurfaceFormatKHR {
        // Use SRG if available
        for surface_format in available_surface_formats {
            let supports_srgb = surface_format.format == vk::Format::B8G8R8_SRGB;
            let supports_non_linear_color_space =
                surface_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR;

            if supports_srgb && supports_non_linear_color_space {
                return surface_format.clone();
            }
        }

        // If SRGB is not available, pick any
        available_surface_formats.first().unwrap().clone()
    }

    fn select_swapchain_present_mode(
        _available_present_modes: &Vec<vk::PresentModeKHR>,
    ) -> vk::PresentModeKHR {
        // todo have a more detailed selection

        vk::PresentModeKHR::FIFO_RELAXED
    }

    fn select_swapchain_extent(
        surface_capabilities: &vk::SurfaceCapabilitiesKHR,
        window_width: u32,
        window_height: u32,
    ) -> vk::Extent2D {
        // Translate the screen coordinates into pixel resolution if they are not the same. (On high DPI-displays for example, sometimes they differ).

        if surface_capabilities.current_extent.width != u32::MAX {
            // value is set to UINT32_MAX if they differ
            return surface_capabilities.current_extent;
        }

        let mut true_extent: vk::Extent2D = vk::Extent2D {
            width: window_width,
            height: window_height,
        };

        // Clamp
        true_extent.width = std::cmp::max(
            std::cmp::min(
                true_extent.width,
                surface_capabilities.min_image_extent.width,
            ),
            surface_capabilities.max_image_extent.width,
        );
        true_extent.height = std::cmp::max(
            std::cmp::min(
                true_extent.width,
                surface_capabilities.min_image_extent.height,
            ),
            surface_capabilities.max_image_extent.height,
        );

        true_extent
    }

    fn create_swapchain_images(
        logical_device: &ash::Device,
        swapchain_format: vk::Format,
        swapchain_images: &Vec<vk::Image>,
    ) -> Vec<vk::ImageView> {
        let image_views: Vec<vk::ImageView> = swapchain_images
            .iter()
            .map(|&image| {
                let image_view_create_info = vk::ImageViewCreateInfo::builder()
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(swapchain_format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::R,
                        g: vk::ComponentSwizzle::G,
                        b: vk::ComponentSwizzle::B,
                        a: vk::ComponentSwizzle::A,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    })
                    .image(image);

                unsafe { logical_device.create_image_view(&image_view_create_info, None) }
                    .expect("Couldn't create image view")
            })
            .collect();

        image_views
    }
}
