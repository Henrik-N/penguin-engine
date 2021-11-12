use super::config;
use anyhow::Result;
use chrono;
use fern::colors::{Color, ColoredLevelConfig};

pub mod prelude {
    pub use log::{debug, error, info, trace, warn};
}

/// Initializes the fern logger.
pub fn init_logger() -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::new()
        .trace(Color::Cyan)
        .debug(Color::Magenta)
        .info(Color::Green)
        .warn(Color::Yellow)
        .error(Color::Red);

    let time = chrono::Local::now().format("%H:%M:%S").to_string();

    let line_colors = colors.clone().info(Color::Green);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color_arg} {time} | {level} | {color_arg}[{module}] \x1B[0m{message}",
                color_arg = {
                    format_args!(
                        "\x1B[{}m",
                        line_colors.get_color(&record.level()).to_fg_str()
                    )
                },
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
    use crate::core::config;
    use anyhow::*;
    use ash::vk;
    use log::LevelFilter;
    use std::ffi::CStr;
    use std::os::raw::c_void;

    pub fn init_vk_debug_messenger(
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> Result<(
        ash::extensions::ext::DebugUtils,
        Option<vk::DebugUtilsMessengerEXT>,
    )> {
        log::trace!("Creating Vulkan utility messenger");

        let debug_utils_loader = ash::extensions::ext::DebugUtils::new(entry, instance);

        let messenger_create_info = debug_messenger_create_info();

        let utils_messenger = if config::DEBUG.is_enabled {
            unsafe {
                Some(
                    debug_utils_loader
                        .create_debug_utils_messenger(&messenger_create_info, None)?,
                )
            }
        } else {
            None
        };

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
            }
            LevelFilter::Trace => {
                vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
            }
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

        // TODO: no need to init a static if no debug messenger (in non-debug mode)
        static mut LAST_MSG: &str = "";
        static mut IDENTICAL_MSG_COUNT: usize = 0;

        match message_severity {
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
                let msg = message.to_str().expect("Failed to convert &CStr to &str.");
                let val_err = parser::parse_vk_validation_error_message(msg);

                let mut context_objs: String = String::new();
                val_err.context_objects.iter().for_each(|obj| {
                    context_objs +=
                        &format!("\t| {} | {}: {}\n", obj.index, obj.vk_type, obj.handle);
                });

                // don't spam the identical error message over and over, just provide what's new
                let same_msg = LAST_MSG == msg;

                let err_string = if same_msg {
                    // same message
                    IDENTICAL_MSG_COUNT += 1;
                    format!("\n{vk_message_type} ({count})\n\t{vulkan_id}\n{context_objects}\n_______________\n",
                                vk_message_type = "[IDENTICAL]",
                                count = IDENTICAL_MSG_COUNT,
                                vulkan_id = val_err.vulkan_id,
                                context_objects = context_objs,
                                )
                } else {
                    // new message
                    LAST_MSG = &msg;
                    IDENTICAL_MSG_COUNT = 0;

                    format!("\n{vk_message_type}\n\t{vulkan_id}\n{context_objects}\nMSG: {message}\n\nISSUE: {spec_info}\n\nSPEC: {spec_link}\n_______________\n",
                            vk_message_type = vk_message_type,
                            vulkan_id = val_err.vulkan_id,
                            context_objects = context_objs,
                            message = val_err.message,
                            spec_info = val_err.spec_info,
                            spec_link = val_err.spec_link,
                            )
                };

                log::error!("{}", err_string);
            }
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
                log::warn!("{} [{:?}]", vk_message_type, message)
            }
            vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
                let msg = message.to_str().expect("Failed to convert &Cstr to &str.");
                let msg = parser::parse_vk_general_message(msg);
                log::info!("{} {}", vk_message_type, msg);
            }
            vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
                if config::VK_VERBOSE_LOGGING_ENABLE {
                    log::trace!("{} [{:?}]", vk_message_type, message);
                }
            }
            _ => log::error!("Unknown message severity. This code should never be reached."),
        };

        ash::vk::FALSE
    }

    mod parser {
        pub(super) struct ContextObj {
            pub index: String,
            pub vk_type: String,
            pub handle: String,
        }

        pub(super) struct ValidationError {
            pub vulkan_id: String,
            pub context_objects: Vec<ContextObj>,
            pub message: String,
            pub spec_info: String, // If there is a "The Vulkan spec states": -> issue, otherwise empty
            pub spec_link: String,
        }
        impl ValidationError {
            fn simple_msg(msg: &str) -> Self {
                Self {
                    vulkan_id: "".to_owned(),
                    context_objects: vec![ContextObj {
                        index: "".to_owned(),
                        vk_type: "".to_owned(),
                        handle: "".to_owned(),
                    }],
                    message: msg.to_owned(),
                    spec_info: "".to_owned(),
                    spec_link: "".to_owned(),
                }
            }
        }

        pub fn parse_vk_general_message(input: &str) -> &str {
            &input
        }

        pub(super) fn parse_vk_validation_error_message(input: &str) -> ValidationError {
            let sections: Vec<String> = input
                .clone()
                .split("|")
                .map(|str| str.to_string())
                .collect();

            let context_section = sections.iter().find_map(|s| {
                if s.contains("[ ") && s.contains(" ]") {
                    Some(s)
                } else {
                    None
                }
            });

            if let Some(context) = context_section {
                // find vulkan id (name of vk object/function that failed)
                let (vuid, end_index) = find_vuid(&context);

                let context_objs: Vec<ContextObj> = context[end_index..]
                    .split("Object")
                    .filter_map(|object| {
                        if let Some((index, rest)) = object.split_once(":") {
                            let index = index.replace(" ", "");

                            if let Some((handle, vk_type)) = rest.split_once(",") {
                                let handle = handle
                                    .split_once("= ")
                                    .unwrap_or(("", "Unknown handle"))
                                    .1
                                    .replace(" ", "");
                                let vk_type = vk_type
                                    .split_once("= ")
                                    .unwrap_or(("", "Unknown type"))
                                    .1
                                    .replace(" ", "")
                                    .replace(";", "");

                                let err = ContextObj {
                                    index,
                                    vk_type,
                                    handle,
                                };
                                Some(err)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();

                let vuid = vuid.split_once("VUID-").unwrap_or(("", vuid)).1;
                let vuid = vuid.rsplit_once("-").unwrap_or((vuid, "")).0;

                let message = sections.iter().find_map(|s| {
                    if s.contains("(") {
                        Some(s.to_owned())
                    } else {
                        None
                    }
                });

                if let Some(msg) = message {
                    // let adresses: Vec<String> = context_objs.iter().map(|cxt_obj| {
                    //     println!("MEMORY ADRESS INDEX {}", cxt_obj.handle);
                    // }).collect();
                    let mut msg = msg.to_owned();

                    for (i, obj) in context_objs.iter().enumerate() {
                        let index: String = "|".to_owned() + &i.to_string() + "|"; // "|i|"
                        msg = msg.replace(&(obj.handle.to_owned() + "[]"), &index);
                        msg = msg.replace(&obj.handle, &index);
                    }

                    let (msg, spec_info) = msg
                        .split_once("The Vulkan spec states: ")
                        .unwrap_or((&msg, ""));

                    // get link part
                    let link_pattern = "(https://".to_owned();
                    let (spec_info, spec_link) = spec_info
                        .split_once(&link_pattern)
                        .unwrap_or((spec_info, ""));

                    // trim trailing whitespace and add .
                    let mut spec_info = spec_info.to_owned().trim_end().to_string();
                    spec_info += ".";

                    // add back split off link pattern
                    let mut spec_link = spec_link.to_owned();
                    if !spec_link.is_empty() {
                        spec_link = link_pattern + &spec_link;
                    }

                    ValidationError {
                        vulkan_id: vuid.to_owned(),
                        context_objects: context_objs,
                        message: msg.to_owned(),
                        spec_info,
                        spec_link,
                    }
                } else {
                    // failed to parse
                    ValidationError::simple_msg(input)
                }
            } else {
                // failed to parse
                ValidationError::simple_msg(input)
            }
        }

        // Helper functions --------------------------

        /// Finds the VUID section of the given context section of a vulkan validation error
        /// string. Returns a readable version of the VUID along with the index in the context
        /// string at which the VUID section ends.
        fn find_vuid(context_section: &str) -> (&str, usize) {
            let (vuid, end_index) = {
                let from = context_section.find(|c| c == '[').unwrap();
                let to = context_section.find(|c| c == ']').unwrap();
                (&context_section[from + 2..to], to)
            };
            (vuid, end_index)
        }
    }
}
