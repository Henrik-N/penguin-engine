use crate::renderer::vk_types::vk_context::instance::Instance;
use crate::renderer::vk_types::Surface;
use anyhow::*;
use ash::vk;

pub struct PhysicalDevice {
    pub handle: vk::PhysicalDevice,
    pub graphics_queue_index: u32,
}
impl std::ops::Deref for PhysicalDevice {
    type Target = vk::PhysicalDevice;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl PhysicalDevice {
    pub(crate) fn init(instance: &Instance, surface: &Surface) -> Result<Self> {
        let (handle, queue_index) = init::select_physical_device(instance, surface)?;

        Ok(Self {
            handle,
            graphics_queue_index: queue_index,
        })
    }
}

pub struct SwapchainSupportDetails {
    pub surface_capabilities: vk::SurfaceCapabilitiesKHR,
    pub surface_color_formats: Vec<vk::SurfaceFormatKHR>,
    pub surface_present_modes: Vec<vk::PresentModeKHR>,
}

impl PhysicalDevice {
    pub fn query_swapchain_support(&self, surface: &Surface) -> SwapchainSupportDetails {
        query_swapchain_support(self.handle, surface)
    }
}

fn query_swapchain_support(pd: vk::PhysicalDevice, surface: &Surface) -> SwapchainSupportDetails {
    let surface_capabilities = unsafe {
        surface
            .loader
            .get_physical_device_surface_capabilities(pd, surface.handle)
    }
    .expect("Couldn't get surface capabilities");

    let surface_color_formats = unsafe {
        surface
            .loader
            .get_physical_device_surface_formats(pd, surface.handle)
    }
    .expect("Couldn't get surface formats.");

    let surface_present_modes = unsafe {
        surface
            .loader
            .get_physical_device_surface_present_modes(pd, surface.handle)
    }
    .expect("Couldn't get surface presenet modes.");

    SwapchainSupportDetails {
        surface_capabilities,
        surface_color_formats,
        surface_present_modes,
    }
}

mod init {
    // --------------------- PHYSICAL DEVICE -----------------------
    use anyhow::Result;

    type PhysicalDeviceQueueIndex = u32;

    pub fn select_physical_device(
        instance: &Instance,
        surface: &Surface,
    ) -> Result<(vk::PhysicalDevice, PhysicalDeviceQueueIndex)> {
        // Find devices with vulkan support

        log::trace!("Enumerating physical devices...");
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Couldn't find any physical devices with vulkan support.")
        };

        log::debug!(
            "Found {} physical devices with Vulkan support",
            physical_devices.len()
        );

        // Select suitable physical device
        let mut suitable_device = None;

        for &physical_device in physical_devices.iter() {
            let device_info = check_device_suitablity_info(&instance, physical_device, &surface);

            if device_info.is_suitable() {
                if suitable_device.is_none() {
                    suitable_device = Some((physical_device, device_info));
                    break;
                }
            }
        }

        match suitable_device {
            None => {
                log::error!("Couldn't find a suitable physical device.");
                panic!();
            }
            Some((physical_device, info)) => {
                let queue_index = info.graphics_queue_index.expect("No graphics queue index");

                Ok((physical_device, queue_index))
            }
        }
    }

    struct PhysicalDeviceInfo {
        graphics_queue_index: Option<PhysicalDeviceQueueIndex>,
        required_extensions_supported: bool,
        swapchain_supported: bool,
        geometry_shader_support: bool,
    }
    impl PhysicalDeviceInfo {
        fn is_suitable(&self) -> bool {
            self.graphics_queue_index.is_some()
                && self.required_extensions_supported
                && self.swapchain_supported
                && self.geometry_shader_support
        }
    }

    /// Checks if the given physical device supports the required device extensions.
    fn check_device_suitablity_info(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface: &Surface,
    ) -> PhysicalDeviceInfo {
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };

        let features = unsafe { instance.get_physical_device_features(physical_device) };

        let geometry_shader_support = if features.geometry_shader == vk::FALSE {
            false
        } else {
            true
        };

        let device_type = match properties.device_type {
            vk::PhysicalDeviceType::CPU => "Cpu",
            vk::PhysicalDeviceType::INTEGRATED_GPU => "Integrated GPU",
            vk::PhysicalDeviceType::DISCRETE_GPU => "Discrete GPU",
            vk::PhysicalDeviceType::VIRTUAL_GPU => "Virtual GPU",
            vk::PhysicalDeviceType::OTHER => "Unknown",
            _ => panic!("Couldn't find physical device type"),
        };
        log::info!("Physical device: {}", device_type);

        let graphics_queue_index = find_graphics_queue_family(&instance, physical_device, surface);

        let are_required_extensions_supported =
            check_required_extensions_supported(&instance, physical_device);

        let is_swapchain_supported = if are_required_extensions_supported {
            let swapchain_support = super::query_swapchain_support(physical_device, surface);

            !swapchain_support.surface_color_formats.is_empty()
                && !swapchain_support.surface_present_modes.is_empty()
        } else {
            false
        };

        PhysicalDeviceInfo {
            graphics_queue_index,
            required_extensions_supported: are_required_extensions_supported,
            swapchain_supported: is_swapchain_supported,
            geometry_shader_support,
        }
    }

    use crate::renderer::vk_types::{Instance, Surface};
    use ash::vk;

    /// Checks if the listed device extensions are supported on the given physical device.
    fn check_required_extensions_supported(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
    ) -> bool {
        log::trace!("Checking extensions supported:");
        let supported_extensions =
            unsafe { instance.enumerate_device_extension_properties(physical_device) }
                .expect("Physical device: Couldn't get device extensions");

        let supported_extensions_found: Vec<String> = supported_extensions
            .into_iter()
            .map(|extension| crate::util::raw_c_string_to_string(&extension.extension_name)) // converts each raw string to strings
            .filter(|extension_name| {
                crate::config::REQUIRED_DEVICE_EXTENSIONS.contains(&extension_name.as_str())
            }) // filters out any extensions that aren't also in the DEVICE_EXTENSIONS array
            .collect();

        supported_extensions_found
            .iter()
            .for_each(|name| log::debug!("Required extension: {} is supported.", name));

        supported_extensions_found.len() == crate::config::REQUIRED_DEVICE_EXTENSIONS.len()
    }

    pub(crate) fn find_graphics_queue_family(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface: &Surface,
    ) -> Option<u32> {
        log::trace!("Querying physical device for queue family support");
        // Query physical device for which queue families it supports
        let available_queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        log::trace!("Trying to find graphics queue family");

        let mut queues_support = "Available queue families:\n".to_owned();
        //log::info!("QUEUES SUPPORT -----------------------");
        available_queue_families
            .iter()
            .enumerate()
            .for_each(|(index, queue_family_property)| {
                let has_graphics_queue = queue_family_property
                    .queue_flags
                    .contains(vk::QueueFlags::GRAPHICS)
                    as u32;
                let has_compute_queue = queue_family_property
                    .queue_flags
                    .contains(vk::QueueFlags::COMPUTE)
                    as u32;
                let has_transfer_queue = queue_family_property
                    .queue_flags
                    .contains(vk::QueueFlags::TRANSFER)
                    as u32;

                queues_support += &format!(
                    "\tqueue family {}: graphics: {} compute: {} transfer: {}\n",
                    index, has_graphics_queue, has_compute_queue, has_transfer_queue
                );
            });
        log::info!("{}", queues_support);

        let queue_family_index = available_queue_families.iter().enumerate().find_map(
            |(index, queue_family_property)| {
                let graphics_support = queue_family_property
                    .queue_flags
                    .contains(vk::QueueFlags::GRAPHICS);

                if !graphics_support {
                    return None;
                }

                let present_support = unsafe {
                    surface.loader.get_physical_device_surface_support(
                        physical_device,
                        index as u32,
                        surface.handle,
                    )
                }
                .expect("Returned vk false.");

                if present_support {
                    return Some(index as u32);
                }
                None
            },
        );

        queue_family_index
    }
}
