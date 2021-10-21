use super::math::prelude::*;
use crate::core::utility;
use ash::vk;

pub mod prelude {
    pub use super::MeshPushConstants;
    pub use super::PushConstants;
}

// Push constants minimum size == 128 bytes
//

pub trait PushConstants {
    fn shader_stage() -> vk::ShaderStageFlags;
    fn as_u8_slice(&self) -> &[u8];
}

#[allow(dead_code)]
pub struct MeshPushConstants {
    pub data: Vec4,
    pub render_matrix: Mat4,
}

impl PushConstants for MeshPushConstants {
    fn shader_stage() -> vk::ShaderStageFlags {
        vk::ShaderStageFlags::VERTEX
    }

    fn as_u8_slice(&self) -> &[u8] {
        unsafe { utility::as_u8_slice(self) }
    }
}
