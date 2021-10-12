use std::fmt::{Debug, Display, Formatter};
use chrono;
use fern::colors::{Color, ColoredLevelConfig};
use super::config;
use anyhow::Result;

use log::{info, trace, warn, debug, error};

pub mod prelude {
    pub use log::{info, trace, warn, debug, error};
}

/// Initializes the fern logger.
pub fn init_logger() -> Result<(), fern::InitError> {
    let mut colors = ColoredLevelConfig::new()
        .trace(Color::Cyan)
        .debug(Color::Magenta)
        .info(Color::Green)
        .warn(Color::Yellow) // testa bright yellow
        .error(Color::Red); // testa bright red

    let time = chrono::Local::now().format("%H:%M:%S").to_string();
    let date = chrono::Local::now().format("[%Y-%m-%d]");

    let line_colors = colors.clone().info(Color::Green);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color_arg} {time} | {level} | {color_arg}[{module}] \x1B[0m\n\t{message}\n",
                color_arg = format_args!(
                    "\x1B[{}m",
                    line_colors.get_color(&record.level()).to_fg_str()
                ),
                time = time,
                level = colors.color(record.level()),
                message = message,
                module = record.target(),
            ))
        })
        .level(config::DEBUG_MESSAGE_SEVERITY)
        .chain(std::io::stdout())
        .chain(fern::log_file(format!("logs/output.log"))?)
        .apply()?;

    Ok(())
}


pub(crate) mod init {
    //////////////
    use ash::vk;
    use anyhow::*;
    use std::ffi::CStr;
    use std::os::raw::c_void;
    use log::LevelFilter;
    use crate::core::config;

    // /// Sets up VkDebugUtilsMessengerEXT https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkDebugUtilsMessengerEXT.html.
    // pub fn setup_vk_debug_utils(
    //     entry: &ash::Entry,
    //     instance: &ash::Instance,
    // ) -> Result<(ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT)> {
    //     let debug_utils_loader = ash::extensions::ext::DebugUtils::new(entry, instance);
    //
    //     // let messenger_create_info = vk_debug_messenger_create_info();
    //     let messenger_create_info = debug_messenger_create_info();
    //
    //     let utils_messenger =
    //         unsafe { debug_utils_loader.create_debug_utils_messenger(&messenger_create_info, None)? };
    //     Ok((debug_utils_loader, utils_messenger))
    // }

    /// Initializes VkDebugUtilsMessengerEXT https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkDebugUtilsMessengerEXT.html/.
    pub fn init_vk_debug_messenger(entry: &ash::Entry, instance: &ash::Instance) -> Result<(ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT)> {
        log::trace!("Creating Vulkan utility messenger");

        let debug_utils_loader = ash::extensions::ext::DebugUtils::new(entry, instance);

        let messenger_create_info = debug_messenger_create_info();

        let utils_messenger =
            unsafe { debug_utils_loader.create_debug_utils_messenger(&messenger_create_info, None)? };
        Ok((debug_utils_loader, utils_messenger))
    }

    fn debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
        let message_severity = match config::DEBUG_MESSAGE_SEVERITY {
            LevelFilter::Off => vk::DebugUtilsMessageSeverityFlagsEXT::empty(),
            LevelFilter::Error => vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            LevelFilter::Warn => {
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
            }
            LevelFilter::Info | LevelFilter::Debug => {
                vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
            },
            LevelFilter::Trace => {
                vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
            },
        };

        vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(message_severity)
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            )
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
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[VULKAN GENERAL]",
            vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[VULKAN PERFORMANCE]",
            vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[VULKAN VALIDATION]",
            _ => "[VULKAN UNKNOWN]",
        };

        let message = CStr::from_ptr((*p_callback_data).p_message);

        let output = match message_severity {
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
                let msg = message.to_str().expect("Failed to convert &CStr to &str.");
                let tokens = parser::parse_vk_validation_error_message(msg);

                let mut context_objs: String = String::new();
                tokens.context_objects.iter().for_each(|obj| { context_objs += &format!("\t| {} | {}: {}\n", obj.index, obj.vk_type, obj.handle); });

                let string =
                    format!("\nCONTEXT:\n\t{vulkan_id}\n{context_objects}\nMSG: {message}\n\n_______________\n",
                            vulkan_id = tokens.vulkan_id,
                            context_objects = context_objs,
                            message = tokens.message);

                log::error!("{}", string);
            }
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => log::warn!("{} [{:?}]", vk_message_type, message),
            vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
                let msg = message.to_str().expect("Failed to convert &Cstr to &str.");
                let msg = parser::parse_vk_general_message(msg);
                log::info!("{} {}", vk_message_type, msg);
            }
            vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
                if config::VK_VERBOSE_LOGGING_ENABLE {
                    log::trace!("{} [{:?}]", vk_message_type, message);
                }
            },
            _ => log::error!("Unknown message severity. This code should never be reached."),
        };


        ash::vk::FALSE
    }

    mod parser {
        pub struct ContextObj {
            pub index: String,
            pub vk_type: String,
            pub handle: String,
        }

        pub struct ValidationError {
            pub vulkan_id: String,
            pub context_objects: Vec<ContextObj>,
            pub message: String,
        }

        pub fn parse_vk_general_message(input: &str) -> &str {
            &input
        }

        pub fn parse_vk_validation_error_message(input: &str) -> ValidationError {

            let mut sections: Vec<String> =
                input
                    .clone()
                    .split("|")
                    .map(|str| str.to_string())
                    .collect();

            let context_section = sections.iter().enumerate().find_map(|(i, s)| {
                if s.contains("[ ") && s.contains(" ]") {
                    Some(s)
                } else { None }
            });

            if let Some(context) = context_section {
                let (vuid, end_index) = {
                    let from = context.find(|c| c == '[').unwrap();
                    let to = context.find(|c| c == ']').unwrap();
                    (&context[from + 2..to], to)
                };

                let processed_context_objs: Vec<ContextObj> = context[end_index..]
                    .split("Object")
                    .filter_map(|object| {
                        if let Some((index, rest)) = object.split_once(":") {
                            let index = index.replace(" ", "");

                            if let Some((handle, vk_type)) = rest.split_once(",") {
                                let handle = handle.split_once("= ").unwrap_or(("", "Unknown handle")).1.replace(" ", "");
                                let vk_type = vk_type.split_once("= ").unwrap_or(("", "Unknown type")).1.replace(" ", "").replace(";", "");

                                let err = ContextObj {
                                    index,
                                    vk_type,
                                    handle,
                                };
                                Some(err)
                            } else { None }
                        } else { None }
                    }).collect();


                let vuid = vuid.split_once("VUID-").unwrap_or(("", vuid)).1;
                let vuid = vuid.rsplit_once("-").unwrap_or((vuid, "")).0;


                let msg = sections.iter().enumerate()
                    .find_map(|(i, s)| {
                        if s.contains("(") {
                            Some(s.to_owned())
                        } else { None }
                    });

                if let Some(msg) = msg {

                    ValidationError {
                        vulkan_id: vuid.to_string(),
                        context_objects: processed_context_objs,
                        message: msg,
                    }
                } else {
                    // failed to parse
                    ValidationError {
                        vulkan_id: "".to_string(),
                        context_objects: vec![ContextObj {
                            index: "".to_string(),
                            vk_type: "".to_string(),
                            handle: "".to_string()
                        }],
                        message: input.to_string(),
                    }
                }
            } else {
                // failed to parse
                ValidationError {
                    vulkan_id: "".to_string(),
                    context_objects: vec![ContextObj {
                        index: "".to_string(),
                        vk_type: "".to_string(),
                        handle: "".to_string()
                    }],
                    message: input.to_string(),
                }
            }
        }
    }
}
