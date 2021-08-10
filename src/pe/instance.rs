// -------------------------- INSTANCE ----------------------------
use ash::version::InstanceV1_0;
use ash::vk;

pub struct Instance {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    //_debug_utils: ash::extensions::ext::DebugUtils, // todo: Fix
    // debug_messenger: vk::DebugUtilsMessengerEXT,
}

impl Instance {
    pub fn drop(&self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }

    pub fn init() -> Self {
        let entry = init::init_ash_entry();
        let ash_instance = init::init_ash_instance(&entry);

        // let (debug_utils, debug_messenger) = init::init_debug_messenger(&entry, &ash_instance);

        Self {
            entry,
            instance: ash_instance,
            // debug_utils,
            // debug_messenger,
        }
    }
}

mod init {
    fn debug_utils_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
        vk::DebugUtilsMessengerCreateInfoEXT {
            s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
            p_next: std::ptr::null(),
            flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
                // vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
                // vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            pfn_user_callback: Some(vulkan_debug_utils_callback),
            p_user_data: std::ptr::null_mut(),
        }
    }

    pub fn init_debug_messenger(entry: &ash::Entry, ash_instance: &ash::Instance) -> (ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT) {
        let debug_utils_loader = ash::extensions::ext::DebugUtils::new(entry, ash_instance);

        if !ENABLE_VALIDATION_LAYERS {
            (debug_utils_loader, ash::vk::DebugUtilsMessengerEXT::null())
        } else {
            let debug_messenger_create_info = debug_utils_create_info();

            let debug_utils_messenger = unsafe {
                debug_utils_loader.create_debug_utils_messenger(&debug_messenger_create_info, None)
                    .expect("Couldn't create debug utils messenger")
            };

            (debug_utils_loader, debug_utils_messenger)
        }
    }


    #[cfg(all(debug_assertions))]
    pub const ENABLE_VALIDATION_LAYERS: bool = true;
    #[cfg(not(debug_assertions))]
    const ENABLE_VALIDATION_LAYERS: bool = false;

    const REQUIRED_INSTANCE_LAYERS: &[&'static str; 1] = &["VK_LAYER_KHRONOS_validation"];

    use ash::version::EntryV1_0;
    use ash::vk;
    use std::ffi::{CStr, CString};

    use ash::extensions::khr::{Surface, Win32Surface};
    use winapi::shared::evntrace::EnableTraceEx;

    // windows specific
    fn required_extension_names() -> Vec<*const i8> {
        vec![Surface::name().as_ptr(), Win32Surface::name().as_ptr()]
    }

    pub fn init_ash_entry() -> ash::Entry {
        unsafe { ash::Entry::new().expect("Couldn't create entry") }
    }

    pub fn init_ash_instance(entry: &ash::Entry) -> ash::Instance {
        let debug_utils_info = self::debug_utils_create_info();

        let application_info = vk::ApplicationInfo::builder()
            .application_name(CString::new("penguin application").unwrap().as_c_str())
            .application_version(vk::make_version(0, 1, 0))
            .engine_name(CString::new("penguin engine").unwrap().as_c_str())
            .api_version(vk::make_version(1, 2, 0))
            .build();

        let extension_names = required_extension_names();

        let layer_names = REQUIRED_INSTANCE_LAYERS
            .iter()
            .map(|name| CString::new(*name).expect("Failed to build CString"))
            .collect::<Vec<_>>();

        let layer_names_pointers = layer_names
            .iter()
            .map(|name| name.as_ptr())
            .collect::<Vec<_>>();

        let mut instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_extension_names(&extension_names)
            .enabled_layer_names(&layer_names_pointers);

        if ENABLE_VALIDATION_LAYERS {
            self::check_validation_layer_support(&entry);
            println!("Validation layers available!");
            instance_create_info = instance_create_info.enabled_layer_names(&layer_names_pointers);
        }

        unsafe {
            entry
                .create_instance(&instance_create_info, None)
                .expect("Couldn't create instance")
        }
    }

    fn check_validation_layer_support(entry: &ash::Entry) {
        for required_layer in REQUIRED_INSTANCE_LAYERS.iter() {
            let found = entry
                .enumerate_instance_layer_properties()
                .unwrap()
                .iter()
                .any(|layer| {
                    let name = unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) };
                    let name = name.to_str().expect("Failed to get layer name pointer");
                    required_layer == &name
                });

            if !found {
                panic!("Validation layer not supported {}", required_layer);
            }
        }
    }


    // https://github.com/unknownue/vulkan-tutorial-rust/blob/master/src/tutorials/02_validation_layers.rs
    unsafe extern "system" fn vulkan_debug_utils_callback(
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _p_user_data: *mut std::ffi::c_void,
    ) -> vk::Bool32 {
        let severity = match message_severity {
            vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]",
            vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[Info]",
            _ => "[Unknown]",
        };
        let types = match message_type {
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
            vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
            vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
            _ => "[Unknown]",
        };
        let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);
        println!("[Debug]{}{}{:?}", severity, types, message);

        vk::FALSE
    }
}
