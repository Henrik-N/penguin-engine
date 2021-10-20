use std::ffi::CStr;

/// Checks if the machine supports the required validation layers.
pub fn check_validation_layer_support(entry: &ash::Entry) {
    for required_layer in crate::core::config::DEBUG.required_validation_layers.iter() {
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

pub fn raw_c_string_to_string(c_string: &[std::os::raw::c_char]) -> String {
    let raw_c_string = unsafe {
        let ptr = c_string.as_ptr();
        CStr::from_ptr(ptr)
    };
    raw_c_string
        .to_str()
        .expect("Couldn't convert c string.")
        .to_owned()
}

/// Throws an early anyhow::Result error and logs the given provided string.
macro_rules! pbail {
    ($msg:literal $(,)?) => {
        log::error!($msg);
        anyhow::bail!("Exited due to error");
    };
    ($err:expr $(,)?) => {
        log::error!($err);
        anyhow::bail!("^Exited due to error");
    };
    ($fmt:expr, $($arg:tt)*) => {
        log::error!($fmt, $($arg)*);
        anyhow::bail!("Exited due to error");
    };
}

/// Same as panic!, but it also logs to the console.
#[allow(unused_macros)]
macro_rules! ppanic {
    ($msg:literal $(,)?) => {
        log::error!($msg);
        panic!("Paniced due to latest error.");
    };
    ($fmt:expr, $($arg:tt)*) => {
        log::error!($fmt, $($arg)*);
        panic!("Paniced due to latest error.");
    };
}

pub(crate) use pbail;
#[allow(unused_imports)]
pub(crate) use ppanic;
