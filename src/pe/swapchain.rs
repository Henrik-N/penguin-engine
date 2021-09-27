// // -------------------------- SWAPCHAIN --------------------------
// use ash::version::DeviceV1_0;
// use ash::vk;
//
// pub struct SwapchainImage {
//     pub image: vk::Image,
//     pub image_view: vk::ImageView,
// }
//
// pub struct Swapchain {
//     pub swapchain_loader: ash::extensions::khr::Swapchain, // todo can probably make these private by moving some things
//     pub swapchain: vk::SwapchainKHR, // can probably make these private by moving some things
//     pub swapchain_format: vk::Format,
//     pub swapchain_extent: vk::Extent2D,
//     pub images: Vec<SwapchainImage>,
// }
//
// impl Swapchain {
//     pub fn drop(&self, device: &crate::pe::Device) {
//         unsafe {
//             for image_view in self.images.iter() {
//                 device
//                     .logical_device
//                     .destroy_image_view(image_view.image_view, None);
//             }
//             self.swapchain_loader
//                 .destroy_swapchain(self.swapchain, None);
//         }
//     }
//
//     pub fn init(instance: &crate::pe::Instance, device: &crate::pe::Device) -> Self {
//         init::init_swapchain(&instance.instance, &device)
//     }
// }
//
// mod init {
//     use ash::version::DeviceV1_0;
//     use ash::vk;
//
//     pub fn init_swapchain(
//         instance: &ash::Instance,
//         device: &crate::pe::Device,
//     ) -> super::Swapchain {
//         // Physical device swapchain support info
//         let swapchain_support_details = device.query_swapchain_support();
//
//         // Decide how many images we should have in the swapchain
//         // At least the minimum value + 1, since using the minimum value can mean having to wait for the driver to complete operations before providing an image to render == lag..
//         let mut image_count = swapchain_support_details
//             .surface_capabilities
//             .min_image_count
//             + 1;
//
//         // Clamp value
//         // A value of 0 means that there is no limit on the number of images
//         if swapchain_support_details
//             .surface_capabilities
//             .max_image_count
//             > 0
//             && image_count
//                 < swapchain_support_details
//                     .surface_capabilities
//                     .max_image_count
//         {
//             image_count = swapchain_support_details
//                 .surface_capabilities
//                 .max_image_count; // if not exceeding the maximum, set the maximum
//         }
//
//         let surface_format =
//             select_swapchain_surface_format(&swapchain_support_details.surface_color_formats);
//
//         let present_mode =
//             select_swapchain_present_mode(&swapchain_support_details.surface_present_modes);
//
//         let extent = select_swapchain_extent(&swapchain_support_details.surface_capabilities);
//
//         let (image_sharing_mode, _queue_family_index_count, queue_family_indices) =
//             if device.queue_family_indices.graphics_queue
//                 == device.queue_family_indices.present_queue
//             {
//                 // Use exclusive mode if same, as it is more performant
//                 (vk::SharingMode::EXCLUSIVE, 0, vec![])
//             } else {
//                 (
//                     vk::SharingMode::CONCURRENT,
//                     2,
//                     vec![
//                         device.queue_family_indices.graphics_queue.unwrap(), // todo: Does this move them here?
//                         device.queue_family_indices.present_queue.unwrap(),
//                     ],
//                 )
//             };
//
//         // Swapchain create info
//         let create_info = vk::SwapchainCreateInfoKHR {
//             surface: device.surface_details.surface,
//             min_image_count: image_count as u32,
//             image_format: surface_format.format,
//             image_color_space: Default::default(),
//             image_extent: extent,
//             image_array_layers: 1,
//             image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
//             image_sharing_mode,
//             queue_family_index_count: 0,
//             p_queue_family_indices: queue_family_indices.as_ptr(),
//             pre_transform: swapchain_support_details
//                 .surface_capabilities
//                 .current_transform,
//             composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
//             present_mode,
//             clipped: vk::TRUE,
//             ..Default::default()
//         };
//
//         let swapchain_loader =
//             ash::extensions::khr::Swapchain::new(instance, &device.logical_device);
//
//         let swapchain = unsafe {
//             swapchain_loader
//                 .create_swapchain(&create_info, None)
//                 .expect("Couldn't create swapchain.")
//         };
//
//         let swapchain_images = unsafe {
//             swapchain_loader
//                 .get_swapchain_images(swapchain)
//                 .expect("Coudln't get swapchain images")
//         };
//
//         let swapchain_image_structs = create_swapchain_image_structs(
//             &device.logical_device,
//             surface_format.format,
//             &swapchain_images,
//         );
//
//         // todo MAKE IT CLEANER, CREATE BOTH VECS AT THE SAME TIME INSTEAD
//
//         super::Swapchain {
//             swapchain_loader,
//             swapchain,
//             swapchain_format: surface_format.format,
//             swapchain_extent: extent,
//             images: swapchain_image_structs,
//         }
//     }
//
//     fn select_swapchain_surface_format(
//         available_surface_formats: &Vec<vk::SurfaceFormatKHR>,
//     ) -> vk::SurfaceFormatKHR {
//         // Use SRG if available
//         for surface_format in available_surface_formats {
//             let supports_srgb = surface_format.format == vk::Format::B8G8R8_SRGB;
//             let supports_non_linear_color_space =
//                 surface_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR;
//
//             if supports_srgb && supports_non_linear_color_space {
//                 return surface_format.clone();
//             }
//         }
//
//         // If SRGB is not available, pick any
//         available_surface_formats.first().unwrap().clone()
//     }
//
//     fn select_swapchain_present_mode(
//         _available_present_modes: &Vec<vk::PresentModeKHR>,
//     ) -> vk::PresentModeKHR {
//         // todo have a more detailed selection
//
//         vk::PresentModeKHR::FIFO_RELAXED
//     }
//
//     fn select_swapchain_extent(surface_capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
//         // Translate the screen coordinates into pixel resolution if they are not the same. (On high DPI-displays for example, sometimes they differ).
//
//         if surface_capabilities.current_extent.width != u32::MAX {
//             // value is set to UINT32_MAX if they differ
//             return surface_capabilities.current_extent;
//         }
//
//         let mut true_extent: vk::Extent2D = vk::Extent2D {
//             width: crate::pe::window::WINDOW_WIDTH,
//             height: crate::pe::window::WINDOW_HEIGHT,
//         };
//
//         // Clamp
//         true_extent.width = std::cmp::max(
//             std::cmp::min(
//                 true_extent.width,
//                 surface_capabilities.min_image_extent.width,
//             ),
//             surface_capabilities.max_image_extent.width,
//         );
//         true_extent.height = std::cmp::max(
//             std::cmp::min(
//                 true_extent.width,
//                 surface_capabilities.min_image_extent.height,
//             ),
//             surface_capabilities.max_image_extent.height,
//         );
//
//         true_extent
//     }
//
//     fn create_swapchain_image_structs(
//         logical_device: &ash::Device,
//         swapchain_format: vk::Format,
//         swapchain_images: &Vec<vk::Image>,
//     ) -> Vec<super::SwapchainImage> {
//         let mut struct_images: Vec<super::SwapchainImage> = vec![];
//
//         for &image in swapchain_images.iter() {
//             let image_view_create_info1 =
//                 image_view_create_info(swapchain_format, image, vk::ImageAspectFlags::COLOR);
//
//             let image_view =
//                 unsafe { logical_device.create_image_view(&image_view_create_info1, None) }
//                     .expect("Couldn't create image view");
//
//             let struct_image = super::SwapchainImage { image, image_view };
//
//             struct_images.push(struct_image);
//         }
//
//         struct_images
//     }
//
//     fn image_view_create_info(
//         format: vk::Format,
//         image: vk::Image,
//         aspect_mask: vk::ImageAspectFlags,
//     ) -> vk::ImageViewCreateInfo {
//         vk::ImageViewCreateInfo {
//             // Image data interpretation
//             view_type: vk::ImageViewType::TYPE_2D,
//             image,
//             format,
//
//             // Color channel mapping
//             components: vk::ComponentMapping {
//                 r: vk::ComponentSwizzle::IDENTITY,
//                 g: vk::ComponentSwizzle::IDENTITY,
//                 b: vk::ComponentSwizzle::IDENTITY,
//                 a: vk::ComponentSwizzle::IDENTITY,
//             },
//
//             // Info about where the image points to
//             subresource_range: vk::ImageSubresourceRange {
//                 base_mip_level: 0,
//                 level_count: 1,
//                 base_array_layer: 0,
//                 layer_count: 1, // texture layer
//                 aspect_mask,    // basically the image usage
//             },
//             ..Default::default()
//         }
//     }
// }
