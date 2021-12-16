use crate::math_vk_format::*;
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




//use crate::engine::renderer::pipeline::PPipelineBuilder;

//impl<'a> PPipelineBuilder<'a> {
//
//    #[allow(dead_code)]
//    pub fn add_push_constants<PushConstantType: PushConstants>(mut self) -> Self {
//        let size = std::mem::size_of::<PushConstantType>() as u32;
//
//        match PushConstantType::shader_stage() {
//            vk::ShaderStageFlags::VERTEX => {
//                self.add_vertex_shader_push_constants::<PushConstantType>(size);
//            }
//            vk::ShaderStageFlags::FRAGMENT => {
//                self.add_fragment_shader_push_constants::<PushConstantType>(size);
//            }
//            _ => {
//                panic!(
//                    "Push constants for that shader stage are not yet
//                         implemented in the pipeline builder."
//                )
//            }
//        }
//
//        self
//    }
//}
