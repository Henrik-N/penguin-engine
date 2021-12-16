use ash::vk;
use crate::renderer::vk_types::{Instance, PhysicalDevice};


pub struct Device {
    pub handle: ash::Device,
    pub graphics_queue_handle: vk::Queue,
}
impl std::ops::Deref for Device {
    type Target = ash::Device;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}



impl super::Device {
    pub(crate) fn init(instance: &Instance, physical_device: &PhysicalDevice) -> Self {
        log::trace!("Queue index: {}", physical_device.graphics_queue_index);

        log::trace!("Creating logical device");
        let device: ash::Device = create_logical_device(
            &instance.handle,
            physical_device.handle,
            physical_device.graphics_queue_index,
        );

        log::trace!("Getting graphics queue handle");
        let queue_handle: vk::Queue =
            get_graphics_queue_handle(&device, physical_device.graphics_queue_index);

        Self {
            handle: device,
            graphics_queue_handle: queue_handle,
        }
    }
}




pub fn create_logical_device(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    graphics_queue_index: u32,
) -> ash::Device {
    init::create_logical_device(&instance, physical_device, graphics_queue_index)
}


pub fn get_graphics_queue_handle(
    logical_device: &ash::Device,
    graphics_queue_index: u32,
) -> vk::Queue {
    // todo: Ensure safety

    unsafe { logical_device.get_device_queue(graphics_queue_index, 0) }
}

mod init {
    use std::ffi::CString;
    use std::ptr;
    use ash::vk;

    // ------------------- LOGICAL DEVICE ---------------------------------
    fn required_device_features() -> vk::PhysicalDeviceFeatures {
        // TODO: Support separate depth stencil layouts if feature is available. This allows for optimal tiling rather than linear (render pass create info -> pAttachemnts[1].finalLayout

        vk::PhysicalDeviceFeatures {
            fill_mode_non_solid: 1, // for wireframe mode
            geometry_shader: 1,
            tessellation_shader: 1,
            shader_float64: 1,
            ..Default::default()
        }
    }

    pub(crate) fn create_logical_device(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        graphics_queue_index: u32,
    ) -> ash::Device {
        let priorities = [1.0_f32];

        let graphics_queue_create_info = vk::DeviceQueueCreateInfo {
            queue_family_index: graphics_queue_index,
            queue_count: 1,
            p_queue_priorities: priorities.as_ptr(),
            ..Default::default()
        };

        // Specify device features to use
        let physical_device_features = required_device_features();


        let enable_extension_names = [ash::extensions::khr::Swapchain::name().as_ptr()];

        // validation layers
        let enabled_validation_layers_raw: Vec<CString> = penguin_config::vk_config::VK_VALIDATION
            .required_validation_layers
            .iter()
            .map(|name| CString::new(*name).expect("Couldn't unwrap layer name ptr"))
            .collect();

        let enabled_validation_layers: Vec<*const std::os::raw::c_char> =
            enabled_validation_layers_raw
                .iter()
                .map(|name| name.as_ptr())
                .collect();

        // Create logical device info
        let create_info = vk::DeviceCreateInfo {
            queue_create_info_count: 1,
            p_queue_create_infos: &graphics_queue_create_info,
            p_enabled_features: &physical_device_features,
            enabled_extension_count: enable_extension_names.len() as u32,
            pp_enabled_extension_names: enable_extension_names.as_ptr(),
            enabled_layer_count: if penguin_config::vk_config::VK_VALIDATION.is_enabled {
                enabled_validation_layers_raw.len() as u32
            } else {
                0 as u32
            },
            pp_enabled_layer_names: if penguin_config::vk_config::VK_VALIDATION.is_enabled {
                enabled_validation_layers.as_ptr()
            } else {
                ptr::null()
            },
            ..Default::default()
        };

        unsafe {
            instance
                .create_device(physical_device, &create_info, None)
                .expect("Couldn't create logical device.")
        }
    }
}
