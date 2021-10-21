use super::math::prelude::*;
use ash::vk;

#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: Vec2,
    pub color: Vec3,
}

impl Vertex {
    pub fn get_binding_descriptions() -> [vk::VertexInputBindingDescription; 1] {
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        let offset0 = 0;
        let offset1 = std::mem::size_of::<Vec2>();

        [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: Vec2::vk_format(),
                offset: offset0 as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: Vec3::vk_format(),
                offset: offset1 as u32,
            },
        ]
    }
}
