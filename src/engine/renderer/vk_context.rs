/// ------------------------- VK CONTEXT ----------------------------------
use ash::vk;


pub struct VkContext {
    pub instance: Instance,
    pub debug_messenger: DebugMessenger,
    pub surface: Surface,
    pub physical_device: PhysicalDevice,
    pub device: Device,
}
impl VkContext {
    pub fn pd_mem_properties(&self) -> vk::PhysicalDeviceMemoryProperties {
        unsafe {
            self.instance
                .handle
                .get_physical_device_memory_properties(self.physical_device.handle)
        }
    }

    pub fn pd_device_properties(&self) -> vk::PhysicalDeviceProperties {
        unsafe {
            self.instance
                .handle
                .get_physical_device_properties(self.physical_device.handle)
        }
    }

    pub fn min_uniform_buffer_offset_alignment(&self) -> vk::DeviceSize {
        self.pd_device_properties()
            .limits
            .min_uniform_buffer_offset_alignment
    }

    pub fn find_supported_format(
        &self,
        possible_formats: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> Option<vk::Format> {
        let supported_format = possible_formats.into_iter().find_map(|&format| {
            let format_properties = unsafe {
                self.instance
                    .handle
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

    pub fn wait_for_device_idle(&self) {
        log::debug!("Renderer: waiting for device idle..");
        unsafe {
            self
                .device
                .handle
                .device_wait_idle()
                .expect("Device: couldn't wait for idle");
        }
        log::debug!("Renderer: device now idle");
    }
}


pub struct Instance {
    entry: ash::Entry,
    pub handle: ash::Instance,
}
pub struct DebugMessenger {
    pub debug_utils_loader: ash::extensions::ext::DebugUtils,
    pub handle_option: Option<vk::DebugUtilsMessengerEXT>,
}
pub struct Surface {
    pub handle: vk::SurfaceKHR,
    pub loader: ash::extensions::khr::Surface,
}
pub struct PhysicalDevice {
    pub handle: vk::PhysicalDevice,
    pub queue_index: u32,
}
pub struct Device {
    pub handle: ash::Device,
    // graphics queue only for now
    pub queue_handle: vk::Queue,
}




impl VkContext {
    pub fn destroy(&mut self) {
        unsafe {
            log::debug!("Dropping vk context!");

            self.device.handle.destroy_device(None);

            log::debug!("Destroying surface..");
            self.surface
                .loader
                .destroy_surface(self.surface.handle, None);
            log::debug!("Surface destroyed!");

            log::debug!("Destroying debug messenger..");
            if let Some(handle) = self.debug_messenger.handle_option {
                self.debug_messenger
                    .debug_utils_loader
                    .destroy_debug_utils_messenger(handle, None);
            }
            log::debug!("Debug messenger destroyed!");

            log::debug!("Destroying instance!");
            self.instance.handle.destroy_instance(None);
            //self.instance.destroy_instance(None);
            log::debug!("Instance destroyed!");
        }
    }

    pub fn init(window: &winit::window::Window) -> Self {
        log::trace!("Constructing VkContext...");

        use init_context::*;
        log::trace!("Creating instance.");
        let instance = Instance::init(window).expect("couldn't init vk instance");
        log::trace!("Creating vk debug messenger.");
        let debug_messenger =
            DebugMessenger::init(&instance).expect("couldn't init vk debug messenger");
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





mod init_context {
    use super::{DebugMessenger, Instance, PhysicalDevice, Surface};
    use crate::engine::pe;
    use anyhow::*;
    use ash::vk;

    impl Instance {
        pub(super) fn init(window: &winit::window::Window) -> Result<Self> {
            let entry = unsafe { ash::Entry::new() }?;
            let required_surface_extensions = ash_window::enumerate_required_extensions(window)?;

            log::trace!("Creating Vulkan instance.");
            let instance: ash::Instance =
                pe::instance::create_ash_instance(&entry, &required_surface_extensions)?;

            Ok(Self {
                entry,
                handle: instance,
            })
        }
    }
    impl DebugMessenger {
        pub(super) fn init(instance: &Instance) -> Result<Self> {
            let (debug_utils_loader, debug_messenger) =
                crate::core::logger::init::init_vk_debug_messenger(
                    &instance.entry,
                    &instance.handle,
                )?;
            Ok(Self {
                debug_utils_loader,
                handle_option: debug_messenger,
            })
        }
    }
    impl Surface {
        pub(super) fn init(instance: &Instance, window: &winit::window::Window) -> Result<Self> {
            Ok(Self {
                handle: unsafe {
                    ash_window::create_surface(&instance.entry, &instance.handle, window, None)?
                },
                loader: ash::extensions::khr::Surface::new(&instance.entry, &instance.handle),
            })
        }
    }
    impl PhysicalDevice {
        pub(super) fn init(instance: &Instance, surface: &Surface) -> Result<Self> {
            let (physical_device, queue_index) = pe::device::select_physical_device(
                &instance.handle,
                surface.handle,
                &surface.loader,
            )?;

            Ok(Self {
                handle: physical_device,
                queue_index,
            })
        }
    }
    impl super::Device {
        pub(crate) fn init(instance: &Instance, physical_device: &PhysicalDevice) -> Self {
            log::trace!("Queue index: {}", physical_device.queue_index);

            log::trace!("Creating logical device");
            let device = pe::device::create_logical_device(
                &instance.handle,
                physical_device.handle,
                physical_device.queue_index,
            );

            log::trace!("Getting graphics queue handle");
            let queue_handle: vk::Queue =
                pe::device::get_graphics_queue_handle(&device, physical_device.queue_index);

            Self {
                handle: device,
                queue_handle,
            }
        }
    }
}
