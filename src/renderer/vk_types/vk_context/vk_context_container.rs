/// ------------------------- VK CONTEXT ----------------------------------
use ash::vk;
use crate::renderer::vk_types::{Instance, DebugMessenger, Surface, PhysicalDevice, Device};

pub struct VkContext {
    pub instance: Instance,
    pub debug_messenger: DebugMessenger,
    pub surface: Surface,
    pub physical_device: PhysicalDevice,
    pub device: Device,
}

impl VkContext {
    #[allow(unused)]
    pub fn pd_device_properties(&self) -> vk::PhysicalDeviceProperties {
        unsafe { self.instance.get_physical_device_properties(self.physical_device.handle) }
    }


    #[allow(unused)]
    pub fn find_supported_format(
        &self,
        possible_formats: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> Option<vk::Format> {
        let supported_format = possible_formats.into_iter().find_map(|&format| {
            let format_properties = unsafe {
                self.instance
                    .get_physical_device_format_properties(self.physical_device.handle, format)
            };
            if tiling == vk::ImageTiling::LINEAR
                && format_properties.linear_tiling_features.contains(features)
            {
                return Some(format.clone());
            } else if tiling == vk::ImageTiling::OPTIMAL
                && format_properties.optimal_tiling_features.contains(features)
            {
                return Some(format.clone());
            } else {
                None
            }
        });
        supported_format
    }
}


impl VkContext {
    pub fn destroy(&mut self) {
        unsafe {
            log::trace!("Dropping vk context!");

            self.device.destroy_device(None);

            log::trace!("Destroying surface..");
            self.surface
                .loader
                .destroy_surface(self.surface.handle, None);
            log::trace!("Surface destroyed!");

            log::trace!("Destroying debug messenger..");
            if let Some(handle) = self.debug_messenger.handle_option {
                self.debug_messenger
                    .debug_utils_loader
                    .destroy_debug_utils_messenger(handle, None);
            }
            log::trace!("Debug messenger destroyed!");

            log::trace!("Destroying instance!");
            self.instance.destroy_instance(None);
            log::trace!("Instance destroyed!");
        }
    }

    pub fn init(window: &penguin_app::window::Window, log_level_filter: log::LevelFilter) -> Self {
        log::trace!("Constructing VkContext...");

        log::trace!("Creating instance.");
        let instance = Instance::init(window).expect("couldn't init vk instance");
        log::trace!("Creating vk debug messenger.");
        let debug_messenger =
            DebugMessenger::init(&instance, log_level_filter).expect("couldn't init vk debug messenger");
        log::trace!("Creating surface.");
        let surface = Surface::init(&instance, window).expect("couldn't init vk surface");
        log::trace!("Selecting physical device and caching it's properties.");
        let physical_device =
            PhysicalDevice::init(&instance, &surface).expect("couldn't init vk physical device");

        log::trace!("Creating logical device.");
        let device = Device::init(&instance, &physical_device);


        Self {
            instance,
            debug_messenger,
            surface,
            physical_device,
            device,
        }
    }
}
