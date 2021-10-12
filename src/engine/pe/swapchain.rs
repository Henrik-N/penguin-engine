// // -------------------------- SWAPCHAIN --------------------------
use ash::vk;
use crate::engine::pe;
use crate::engine::pe::device::{SwapchainSupportDetails};

pub struct SwapchainImage {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
}

pub struct PSwapchain {
    pub swapchain_loader: ash::extensions::khr::Swapchain, // todo can probably make these private by moving some things
    pub swapchain: vk::SwapchainKHR, // can probably make these private by moving some things
    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,
}
//
impl PSwapchain {
    pub fn create(instance: &ash::Instance, device: &ash::Device, surface: vk::SurfaceKHR, swapchain_support_details: SwapchainSupportDetails, graphics_queue_index: u32) -> PSwapchain {
        init::init_swapchain(instance, device, surface, swapchain_support_details, graphics_queue_index)
    }
}

pub(crate) mod init {
    use ash::vk;
    use crate::engine::pe;
    use crate::engine::pe::device::{SwapchainSupportDetails};

    pub(super) fn init_swapchain(
        instance: &ash::Instance,
        device: &ash::Device,
        surface: vk::SurfaceKHR,
        swapchain_support_details: SwapchainSupportDetails,
        graphics_queue_index: u32
    ) -> super::PSwapchain {
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

        let extent = select_swapchain_extent(&swapchain_support_details.surface_capabilities);

        let (image_sharing_mode, _queue_family_index_count, queue_family_indices) =
                // Use exclusive mode if same, as it is more performant
                (vk::SharingMode::EXCLUSIVE, 1, vec![graphics_queue_index]);

        // let graphics_queue_family_index = [graphics_queue_index];


        // Swapchain create info
        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count as u32)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            // .queue_family_indices(&graphics_queue_family_index)
            .pre_transform(swapchain_support_details.surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let swapchain_loader =
            ash::extensions::khr::Swapchain::new(instance, &device);

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

        let swapchain_image_views = create_swapchain_images(
            &device,
            surface_format.format,
            &swapchain_images,
        );

        super::PSwapchain {
            swapchain_loader,
            swapchain,
            swapchain_format: surface_format.format,
            swapchain_extent: extent,
            swapchain_images,
            swapchain_image_views,
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

    fn select_swapchain_extent(surface_capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        // Translate the screen coordinates into pixel resolution if they are not the same. (On high DPI-displays for example, sometimes they differ).

        if surface_capabilities.current_extent.width != u32::MAX {
            // value is set to UINT32_MAX if they differ
            return surface_capabilities.current_extent;
        }

        let mut true_extent: vk::Extent2D = vk::Extent2D {
            width: crate::core::config::WIDTH,
            height: crate::core::config::HEIGHT,
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
        let image_views: Vec<vk::ImageView> = swapchain_images.iter().map(|&image| {
            let image_view_create_info= vk::ImageViewCreateInfo::builder()
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
                    layer_count: 1
                })
                .image(image);

                unsafe { logical_device.create_image_view(&image_view_create_info, None) }
                    .expect("Couldn't create image view")
        }).collect();

        image_views
    }
}