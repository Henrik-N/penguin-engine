use ash::vk;

// 2 == double buffering
pub const MAX_FRAMES_COUNT: usize = 2;

pub const WIDTH: u32 = 640;
pub const HEIGHT: u32 = 400;

/// The required extensions for the physical device that we will be selecting.
pub(crate) const REQUIRED_DEVICE_EXTENSIONS: [&'static str; 1] = ["VK_KHR_swapchain"];

pub(crate) fn required_device_features() -> vk::PhysicalDeviceFeatures {
    // TODO: Support separate depth stencil layouts if feature is available. This allows for optimal tiling rather than linear (render pass create info -> pAttachemnts[1].finalLayout

    vk::PhysicalDeviceFeatures {
        fill_mode_non_solid: 1, // for wireframe mode
        ..Default::default()
    }
}

/// **********************
/// DEBUG CONFIGURATION
/// **********************
#[cfg(all(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

pub const DEBUG: ValidationInfo = ValidationInfo {
    is_enabled: ENABLE_VALIDATION_LAYERS,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};

/// Weather to use verbose vulkan validation layer logging
pub const VK_VERBOSE_LOGGING_ENABLE: bool = false;

/// Filter for which messages to log.
pub const DEBUG_MESSAGE_SEVERITY: log::LevelFilter =
    // * OPTIONS:
    //     log::LevelFilter::Off;
    //     log::LevelFilter::Error;
    //     log::LevelFilter::Warn;
    //     log::LevelFilter::Info;
    log::LevelFilter::Debug;
    //log::LevelFilter::Trace;

// -------------
pub struct ValidationInfo {
    pub is_enabled: bool,
    pub required_validation_layers: [&'static str; 1],
}
