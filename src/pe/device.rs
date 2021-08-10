// -------------------------- DEVICE --------------------------
use ash::version::DeviceV1_0;
use ash::vk;

pub struct QueueFamilyIndices {
    pub graphics_queue: Option<u32>,
    pub present_queue: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn is_complete(&self) -> bool {
        self.graphics_queue.is_some()
    }
}

pub struct SurfaceDetails {
    pub surface_loader: ash::extensions::khr::Surface,
    pub surface: vk::SurfaceKHR,
}

pub struct SwapchainSupportDetails {
    pub surface_capabilities: vk::SurfaceCapabilitiesKHR,
    pub surface_color_formats: Vec<vk::SurfaceFormatKHR>,
    pub surface_present_modes: Vec<vk::PresentModeKHR>,
}

/// DEVICE STRUCT
pub struct Device {
    pub physical_device: vk::PhysicalDevice,
    pub surface_details: SurfaceDetails,
    pub logical_device: ash::Device,
    pub queue_family_indices: QueueFamilyIndices,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
}

impl Device {
    pub fn init(pe_instance: &crate::pe::Instance, window: &winit::window::Window) -> Self {
        init::init_device(&pe_instance.entry, &pe_instance.instance, &window)
    }

    pub fn drop(&self) {
        unsafe {
            self.logical_device.destroy_device(None);
            self.surface_details
                .surface_loader
                .destroy_surface(self.surface_details.surface, None);
        }
    }

    pub fn query_swapchain_support(&self) -> SwapchainSupportDetails {
        let surface_capabilities = unsafe {
            self.surface_details
                .surface_loader
                .get_physical_device_surface_capabilities(
                    self.physical_device,
                    self.surface_details.surface,
                )
        }
        .expect("Couldn't get surface capabilities");

        let surface_color_formats = unsafe {
            self.surface_details
                .surface_loader
                .get_physical_device_surface_formats(
                    self.physical_device,
                    self.surface_details.surface,
                )
        }
        .expect("Couldn't get surface formats.");

        let surface_present_modes = unsafe {
            self.surface_details
                .surface_loader
                .get_physical_device_surface_present_modes(
                    self.physical_device,
                    self.surface_details.surface,
                )
        }
        .expect("Couldn't get surface presenet modes.");

        SwapchainSupportDetails {
            surface_capabilities,
            surface_color_formats,
            surface_present_modes,
        }
    }
}

mod init {
    use ash::version::{DeviceV1_0, InstanceV1_0};
    use ash::vk;

    pub fn init_device(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &winit::window::Window,
    ) -> super::Device {
        let surface_details = create_surface(&entry, &instance, &window);
        let physical_device = select_physical_device(&instance, &surface_details);
        let queue_family_indices =
            find_queue_families(&instance, physical_device, &surface_details);
        let ash_device = create_logical_device(&instance, physical_device, &queue_family_indices);
        let (graphics_queue, present_queue) =
            get_device_queue_handles(&ash_device, &queue_family_indices);

        super::Device {
            physical_device,
            surface_details,
            logical_device: ash_device,
            queue_family_indices,
            graphics_queue,
            present_queue,
        }
    }

    // --------------------- SURFACE -----------------------
    fn create_surface(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &winit::window::Window,
    ) -> crate::pe::device::SurfaceDetails {
        // note: Windows specific implementation
        //use ash::extensions::khr::Surface;
        use ash::extensions::khr::Win32Surface;

        // let required_extension_names = vec![
        //     Surface::name().as_ptr(),
        //     Win32Surface::name().as_ptr(),
        //     DebugUtils::name().as_ptr(), // todo Add the actual debug stuff
        // ];

        use std::os::raw::c_void;
        use std::ptr;
        use winapi::shared::windef::HWND;
        use winapi::um::libloaderapi::GetModuleHandleW;
        use winit::platform::windows::WindowExtWindows;

        let hwnd = window.hwnd() as HWND;
        let hinstance = unsafe { GetModuleHandleW(ptr::null()) as *const c_void };
        let win32_create_info = vk::Win32SurfaceCreateInfoKHR {
            hinstance,
            hwnd: hwnd as *const c_void,
            ..Default::default()
        };

        let win32_surface_loader = Win32Surface::new(entry, instance);
        let vk_surface =
            unsafe { win32_surface_loader.create_win32_surface(&win32_create_info, None) }
                .expect("Failed to create vk::SurfaceKHR");

        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);

        crate::pe::device::SurfaceDetails {
            surface_loader,
            surface: vk_surface,
        }
    }

    // --------------------- PHYSICAL DEVICE -----------------------

    pub fn select_physical_device(
        instance: &ash::Instance,
        surface_details: &crate::pe::device::SurfaceDetails,
    ) -> vk::PhysicalDevice {
        // Find devices with vulkan support
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Couldn't find any physical devices with vulkan support.")
        };

        println!(
            "Found {} physical devices with Vulkan support",
            physical_devices.len()
        );

        // Select suitable physical device
        let mut suitable_device = None;

        for &physical_device in physical_devices.iter() {
            if is_physical_device_suitable(&instance, physical_device, surface_details) {
                if suitable_device.is_none() {
                    suitable_device = Some(physical_device);
                    break;
                }
            }
        }

        match suitable_device {
            None => panic!("Couldn't find a suitable physical device."),
            Some(physical_device) => physical_device,
        }
    }

    fn is_physical_device_suitable(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface_details: &crate::pe::device::SurfaceDetails,
    ) -> bool {
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        //let features = unsafe { instance.get_physical_device_features(physical_device) };

        let device_type = match properties.device_type {
            vk::PhysicalDeviceType::CPU => "Cpu",
            vk::PhysicalDeviceType::INTEGRATED_GPU => "Integrated GPU",
            vk::PhysicalDeviceType::DISCRETE_GPU => "Discrete GPU",
            vk::PhysicalDeviceType::VIRTUAL_GPU => "Virtual GPU",
            vk::PhysicalDeviceType::OTHER => "Unknown",
            _ => panic!("Couldn't find physical device type"),
        };
        println!("Found a physical device: {}", device_type);
        println!(
            "Current Vulkan API version: {}.{}.{}.",
            vk::version_major(properties.api_version),
            vk::version_minor(properties.api_version),
            vk::version_patch(properties.api_version)
        );

        let queue_family_indices = find_queue_families(&instance, physical_device, surface_details);

        let are_required_extensions_supported =
            check_required_extensions_supported(&instance, physical_device);

        queue_family_indices.is_complete() && are_required_extensions_supported
    }

    fn check_required_extensions_supported(
        _instance: &ash::Instance,
        _physical_device: vk::PhysicalDevice,
    ) -> bool {
        // enumerate available physical device extensions
        // let available_extensions =
        //     unsafe { instance.enumerate_device_extension_properties(physical_device) }.unwrap();

        // todo check that required extensions are available

        true
    }

    fn find_queue_families(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface_details: &crate::pe::device::SurfaceDetails,
    ) -> crate::pe::device::QueueFamilyIndices {
        // Query physical device for which queue families it supports
        let available_queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut queue_family_indices = crate::pe::device::QueueFamilyIndices {
            graphics_queue: None,
            present_queue: None,
        };

        let mut queue_family_index = 0;
        for queue_family in available_queue_families.iter() {
            // Check if queue family support a graphics queue
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                queue_family_indices.graphics_queue = Some(queue_family_index);
            }

            // Check if queue family support a present queue
            let is_present_queue_supported = unsafe {
                surface_details
                    .surface_loader
                    .get_physical_device_surface_support(
                        physical_device,
                        queue_family_index as u32,
                        surface_details.surface,
                    )
            }
            .unwrap();

            if queue_family.queue_count > 0 && is_present_queue_supported {
                queue_family_indices.present_queue = Some(queue_family_index);
            }

            // Break loop if all queues are found
            if queue_family_indices.is_complete() {
                break;
            }

            queue_family_index += 1;
        }

        queue_family_indices
    }

    // ------------------- LOGICAL DEVICE ---------------------------------
    fn create_logical_device(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        queue_family_indices: &crate::pe::device::QueueFamilyIndices,
    ) -> ash::Device {
        let mut unique_queue_family_indices = std::collections::HashSet::new();
        unique_queue_family_indices.insert(queue_family_indices.graphics_queue.unwrap());
        unique_queue_family_indices.insert(queue_family_indices.present_queue.unwrap());

        let queue_priorities = [1.0_f32];
        let mut queue_create_infos = vec![];

        // Fill out info for device queue family
        for queue_family_index in unique_queue_family_indices {
            let queue_create_info = vk::DeviceQueueCreateInfo {
                queue_family_index,
                queue_count: queue_priorities.len() as u32,
                p_queue_priorities: queue_priorities.as_ptr(),
                ..Default::default()
            };
            queue_create_infos.push(queue_create_info);
        }

        // Specify device features to use
        let physical_device_features = vk::PhysicalDeviceFeatures {
            ..Default::default()
        }; // todo: Support separate depth stencil layouts if feature is available. This allows for optimal tiling rather than linear (render pass create info -> pAttachemnts[1].finalLayout

        let enable_extension_names = [ash::extensions::khr::Swapchain::name().as_ptr()];

        // Create logical device info
        let create_info = vk::DeviceCreateInfo {
            queue_create_info_count: 1,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            p_enabled_features: &physical_device_features,
            enabled_extension_count: enable_extension_names.len() as u32,
            pp_enabled_extension_names: enable_extension_names.as_ptr(),
            ..Default::default()
        };

        unsafe {
            instance
                .create_device(physical_device, &create_info, None)
                .expect("Couldn't create logical device.")
        }
    }

    fn get_device_queue_handles(
        logical_device: &ash::Device,
        queue_family_indices: &crate::pe::device::QueueFamilyIndices,
    ) -> (vk::Queue, vk::Queue) {
        // todo: Ensure safety

        let graphics_queue = unsafe {
            logical_device.get_device_queue(queue_family_indices.graphics_queue.unwrap(), 0)
        };

        let present_queue = unsafe {
            logical_device.get_device_queue(queue_family_indices.present_queue.unwrap(), 0)
        };

        (graphics_queue, present_queue)
    }
}
