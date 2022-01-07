/// The required extensions for the physical device that we will be selecting.
pub const REQUIRED_DEVICE_EXTENSIONS: [&'static str; 1] = ["VK_KHR_swapchain"];

#[cfg(all(debug_assertions))]
const DEBUG_ENABLED: bool = true;
#[cfg(not(debug_assertions))]
const DEBUG_ENABLED: bool = false;

pub const VK_VALIDATION: ValidationInfo = ValidationInfo {
    is_enabled: DEBUG_ENABLED,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};

// -------------
pub struct ValidationInfo {
    pub is_enabled: bool,
    pub required_validation_layers: [&'static str; 1],
}

/// Weather to use verbose vulkan validation layer logging
pub const VK_VERBOSE_LOGGING_ENABLE: bool = false;

// 2 == double buffering
pub const MAX_FRAMES_COUNT: usize = 2;

pub const MAX_OBJECTS: usize = 10_000;
