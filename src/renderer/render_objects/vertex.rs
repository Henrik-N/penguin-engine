use ash::vk;
use crate::math_vk_format::{Vec3, VkFormat};


#[derive(Clone, Copy, Default)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Vec3,
}
impl Vertex {
    pub fn create_binding_descriptions(binding: u32) -> [vk::VertexInputBindingDescription; 1] {
        [vk::VertexInputBindingDescription {
            binding,
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    pub fn create_attribute_descriptions(binding: u32) -> [vk::VertexInputAttributeDescription; 3] {
        let offset0 = 0;
        let offset1 = std::mem::size_of::<Vec3>();
        let offset2 = offset1 + std::mem::size_of::<Vec3>();

        [
            vk::VertexInputAttributeDescription {
                binding,
                location: 0,
                format: Vec3::vk_format(),
                offset: offset0 as u32,
            },
            vk::VertexInputAttributeDescription {
                binding,
                location: 1,
                format: Vec3::vk_format(),
                offset: offset1 as u32,
            },
            vk::VertexInputAttributeDescription {
                binding,
                location: 2,
                format: Vec3::vk_format(),
                offset: offset2 as u32,
            },
        ]
    }
}

