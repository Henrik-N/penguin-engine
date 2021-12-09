/// ------------------------- VK CONTEXT ----------------------------------
use ash::vk;
use crate::renderer::vk_types::Pipeline;

pub struct VkContext {
    pub instance: Instance,
    pub debug_messenger: DebugMessenger,
    pub surface: Surface,
    pub physical_device: PhysicalDevice,
    pub device: Device,
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
    pub graphics_queue_handle: vk::Queue,
}
impl std::ops::Deref for Device {
    type Target = ash::Device;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}



impl VkContext {
    pub fn bind_pipeline(&self, pipeline: &Pipeline, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device.cmd_bind_pipeline(
                command_buffer,
                pipeline.pipeline_bind_point,
                pipeline.handle,
            );
        }
    }
}




impl VkContext {
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


    // command buffers -------------
    pub fn reset_command_buffer(&self, command_buffer: vk::CommandBuffer, reset_flags: vk::CommandBufferResetFlags) {
        unsafe {
            self.device.handle
                .reset_command_buffer(command_buffer, reset_flags) //  vk::CommandBufferResetFlags::RELEASE_RESOURCES
                .expect("Failed to reset command buffer");
        }
    }

    pub fn begin_command_buffer(&self, command_buffer: vk::CommandBuffer, usage_flags: vk::CommandBufferUsageFlags) {
        let cmd_buffer_begin_info =
            vk::CommandBufferBeginInfo::builder().flags(usage_flags);

        unsafe {
            log::trace!("Beginning command buffer");
            self.device.handle
                .begin_command_buffer(command_buffer, &cmd_buffer_begin_info)
                .expect("Couldn't begin command buffer");
        }
    }

    pub fn end_command_buffer(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            log::trace!("Ending command buffer");
            self.device.handle
                .end_command_buffer(command_buffer)
                .expect("Couldn't end command buffer");
        }
    }

    pub fn submit_to_graphics_queue(&self, submit_info: vk::SubmitInfoBuilder, fence: vk::Fence) {
        unsafe {
            // the render fence will block until the graphics commands finish execution
            self.device.handle
                .queue_submit(self.device.graphics_queue_handle, &[submit_info.build()], fence)
                .expect("Couldn't submit command queue");
        }
    }
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



mod init_vk_context {
    use super::{DebugMessenger, Instance, PhysicalDevice, Surface};
    use crate::renderer::renderer_internal;
    use anyhow::*;
    use ash::vk;
    use crate::renderer::vk_types::VkContext;

    impl Instance {
        pub(crate) fn init(window: &penguin_app::window::Window) -> Result<Self> {
            let entry = unsafe { ash::Entry::new() }?;
            let required_surface_extensions = ash_window::enumerate_required_extensions(&window.handle)?;

            log::trace!("Creating Vulkan instance.");
            let instance: ash::Instance =
                renderer_internal::instance::create_ash_instance(&entry, &required_surface_extensions)?;

            Ok(Self {
                entry,
                handle: instance,
            })
        }
    }
    impl DebugMessenger {
        pub(crate) fn init(instance: &Instance, log_level_filter: log::LevelFilter) -> Result<Self> {
            let (debug_utils_loader, debug_messenger) =
                crate::renderer::debug::debug_messenger::init_vk_debug_messenger(
                    &instance.entry,
                    &instance.handle,
                    log_level_filter,
                )?;
            Ok(Self {
                debug_utils_loader,
                handle_option: debug_messenger,
            })
        }
    }
    impl Surface {
        pub(crate) fn init(instance: &Instance, window: &penguin_app::window::Window) -> Result<Self> {
            Ok(Self {
                handle: unsafe {
                    ash_window::create_surface(&instance.entry, &instance.handle, &window.handle, None)?
                },
                loader: ash::extensions::khr::Surface::new(&instance.entry, &instance.handle),
            })
        }
    }
    impl PhysicalDevice {
        pub(crate) fn init(instance: &Instance, surface: &Surface) -> Result<Self> {
            let (physical_device, queue_index) = renderer_internal::device::select_physical_device(
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
            let device: ash::Device = renderer_internal::device::create_logical_device(
                &instance.handle,
                physical_device.handle,
                physical_device.queue_index,
            );

            log::trace!("Getting graphics queue handle");
            let queue_handle: vk::Queue =
                renderer_internal::device::get_graphics_queue_handle(&device, physical_device.queue_index);

            Self {
                handle: device,
                graphics_queue_handle: queue_handle,
            }
        }
    }
}
