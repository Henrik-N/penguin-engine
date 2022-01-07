// -------------------------- INSTANCE ----------------------------
use anyhow::*;
use ash::vk;
use std::ffi::{CStr, CString};

pub struct Instance {
    pub(super) entry: ash::Entry,
    pub handle: ash::Instance,
}

impl std::ops::Deref for Instance {
    type Target = ash::Instance;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Instance {
    pub fn init(window: &penguin_app::window::Window) -> Result<Self> {
        let entry = unsafe { ash::Entry::new() }?;
        let required_surface_extensions =
            ash_window::enumerate_required_extensions(&window.handle)?;

        log::trace!("Creating Vulkan instance.");
        let instance: ash::Instance = create_ash_instance(&entry, &required_surface_extensions)?;

        Ok(Self {
            entry,
            handle: instance,
        })
    }
}

fn create_ash_instance(
    entry: &ash::Entry,
    surface_extensions: &Vec<&CStr>,
) -> Result<ash::Instance, ash::InstanceError> {
    log::info!("Using Vulkan version 1.2.186");
    let app_info = vk::ApplicationInfo::builder()
        .application_name(CString::new("penguin application").unwrap().as_c_str())
        // .application_version(vk::make_version(0, 1, 0))
        .engine_name(CString::new("penguin engine").unwrap().as_c_str())
        .api_version(vk::make_api_version(0, 1, 2, 186))
        .build();

    let mut extension_names = surface_extensions
        .iter()
        .map(|extension| extension.as_ptr())
        .collect::<Vec<_>>();

    if crate::config::VK_VALIDATION.is_enabled {
        extension_names.push(ash::extensions::ext::DebugUtils::name().as_ptr());
    }

    let instance_desc = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&extension_names);

    if !crate::config::VK_VALIDATION.is_enabled {
        unsafe { entry.create_instance(&instance_desc, None) }
    } else {
        let layer_names = crate::config::VK_VALIDATION
            .required_validation_layers
            .iter()
            .map(|name| CString::new(*name).expect("Failed to build CString"))
            .collect::<Vec<_>>();

        let layer_names_pointers = layer_names
            .iter()
            .map(|name| name.as_ptr())
            .collect::<Vec<_>>();

        crate::renderer::debug::validation_layers::check_validation_layer_support(&entry);

        let instance_desc = instance_desc.enabled_layer_names(&layer_names_pointers);
        unsafe { entry.create_instance(&instance_desc, None) }
    }
}
