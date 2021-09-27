use chrono;
use fern::colors::{Color, ColoredLevelConfig};
use super::config;

use log::{debug, error, info, trace, warn};


/// Initializes the fern logger.
pub fn init_logger() -> Result<(), fern::InitError> {
    let mut colors = ColoredLevelConfig::new()
        .trace(Color::Cyan)
        .debug(Color::Magenta)
        .info(Color::Green)
        .warn(Color::Yellow) // testa bright yellow
        .error(Color::Red); // testa bright red

    // let date = chrono::Local::now().format("[%Y-%m-%d]").to_string();
    let time = chrono::Local::now().format("%H:%M:%S").to_string();
    let date = chrono::Local::now().format("[%Y-%m-%d]");

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} | {} {} \t\t\t\t Module: {}",
                time,
                colors.color(record.level()),
                message,
                record.target(),
            ))
        })
        .level(config::DEBUG_MESSAGE_SEVERITY)
        .chain(std::io::stdout())
        .chain(fern::log_file(format!("logs/LOG{}.log", date.to_string()))?)
        .apply()?;

    log::trace!("Logger initialized.");
    Ok(())
}


pub(crate) mod init {
    //////////////
    use ash::vk;
    use anyhow::*;
    use std::ffi::CStr;
    use std::os::raw::c_void;
    use super::config;

    /// Sets up VkDebugUtilsMessengerEXT https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkDebugUtilsMessengerEXT.html.
    pub fn setup_vk_debug_utils(
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> Result<(ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT)> {
        let debug_utils_loader = ash::extensions::ext::DebugUtils::new(entry, instance);

        let messenger_create_info = vk_debug_messenger_create_info();

        let utils_messenger =
            unsafe { debug_utils_loader.create_debug_utils_messenger(&messenger_create_info, None)? };
        Ok((debug_utils_loader, utils_messenger))
    }



    /// Initializes VkDebugUtilsMessengerEXT https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkDebugUtilsMessengerEXT.html/.
    pub fn init_vk_debug_messenger(entry: &ash::Entry, ash_instance: &ash::Instance) -> Result<(ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT)> {
        log::trace!("Creating Vulkan utility messenger");
        let debug_utils_loader = ash::extensions::ext::DebugUtils::new(entry, ash_instance);

        if !config::DEBUG.is_enabled {
            Ok((debug_utils_loader, ash::vk::DebugUtilsMessengerEXT::null()))
        } else {
            let debug_messenger_create_info = vk_debug_messenger_create_info();

            let debug_utils_messenger = unsafe {
                debug_utils_loader.create_debug_utils_messenger(&debug_messenger_create_info, None)
                    .expect("Couldn't create utility utils messenger")
            };

            Ok((debug_utils_loader, debug_utils_messenger))
        }
    }

    use log::LevelFilter;

    fn vk_debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
        vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(config::VK_VALIDATION_LAYERS_MESSAGE_SEVERITY)
            .message_type(
                ash::vk::DebugUtilsMessageTypeFlagsEXT::GENERAL |
                    ash::vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION |
                    ash::vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE)
            .pfn_user_callback(Some(vulkan_debug_utils_callback))
            .build()
    }

    /// The callback function used in Debug Utils.
    unsafe extern "system" fn vulkan_debug_utils_callback(
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _p_user_data: *mut c_void,
    ) -> vk::Bool32 {
        let vk_message_type = match message_type {
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "Vulkan, General:",
            vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "Vulkan, Performance:",
            vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "Vulkan, Validation:",
            _ => "[Unknown]",
        };

        let message = CStr::from_ptr((*p_callback_data).p_message);

        match message_severity {
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => log::error!("{} [{:?}]", vk_message_type, message),
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => log::warn!("{} [{:?}]", vk_message_type, message),
            vk::DebugUtilsMessageSeverityFlagsEXT::INFO => log::info!("{} [{:?}]", vk_message_type, message),
            vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => log::trace!("{} [{:?}]", vk_message_type, message),
            _ => log::error!("Unknown message severity. This code should never be reached."),
        };

        vk::FALSE
    }
}
