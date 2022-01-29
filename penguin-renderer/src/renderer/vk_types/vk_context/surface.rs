use crate::renderer::vk_types::Instance;
use anyhow::*;
use ash::vk;

use crate::impl_deref;

pub struct Surface {
    pub handle: vk::SurfaceKHR,
    pub loader: ash::extensions::khr::Surface,
}
impl_deref!(Surface, handle, vk::SurfaceKHR);

impl Surface {
    pub(crate) fn init(instance: &Instance, window: &penguin_app::window::Window) -> Result<Self> {
        Ok(Self {
            handle: unsafe {
                ash_window::create_surface(&instance.entry, &instance.handle, &window.handle, None)?
            },
            loader: ash::extensions::khr::Surface::new(&instance.entry, &instance.handle),
        })
    }
}
