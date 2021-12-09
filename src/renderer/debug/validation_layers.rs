use std::ffi::CStr;

/// Checks if the machine supports the required validation layers.
pub fn check_validation_layer_support(entry: &ash::Entry) {
    for required_layer in penguin_config::vk_config::VK_VALIDATION.required_validation_layers.iter() {
        let found = entry
            .enumerate_instance_layer_properties()
            .unwrap()
            .iter()
            .any(|layer| {
                let name = unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) };
                let name = name.to_str().expect("Failed to get layer name pointer");
                required_layer == &name
            });

        // pbail!("One of the validation layers, [{}], is listed in config but not supported on your machine. \
        //     Did you install the LunarG SDK? https://www.lunarg.com/vulkan-sdk/.", required_layer);
        if !found {
            log::error!(
                "One of the required validation layers, [{}], is not supported. \
            Did you install the LunarG SDK? https://www.lunarg.com/vulkan-sdk/.",
                required_layer
            );
        } else {
            log::debug!(
                "Required validation layer [{}] is supported",
                required_layer
            );
        }
    }
}

