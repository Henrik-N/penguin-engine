use ash::vk;
use crate::math_vk_format::{Mat4, Vec4};
use crate::renderer::descriptor_sets::{DescriptorVariant, GpuBuffer, GpuBufferBuilder};
use crate::renderer::vk_types::{DescriptorPool, VkContext};
use crate::render_objects::Material;

/// Data to send to and from the gpu through the Uniform buffer
/// Global uniform data, bound once per frame
#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct UniformBufferGlobalData {
    pub data: Vec4,
    pub render_matrix: Mat4,
}

/// Data to send to and from the gpu through the Uniform buffer
#[derive(Default, Clone, Copy)]
#[repr(C)]
//#[repr(align(256))]
pub struct UniformBufferFrameData {
    //pub fog_color: Vec4,
    //pub fog_distances: Vec4,
    pub ambient_color: Vec4,
    //pub sunlight_direction: Vec4,
    //pub sunlight_color: Vec4,
}


// component
pub struct UniformBuffers {
    pub global: GpuBuffer<UniformBufferGlobalData>,
    pub per_frame: GpuBuffer<UniformBufferFrameData>,
}
impl UniformBuffers {
    pub fn bind_descriptor_sets(
        &self,
        context: &VkContext,
        command_buffer: vk::CommandBuffer,
        material: &Material,
        frame_index: usize,
    ) {
        let first_set = 0;
        let descriptor_sets = [self.global.descriptor_set.set, self.per_frame.descriptor_set.set];
        let dynamic_offsets = [(std::mem::size_of::<UniformBufferFrameData>() * frame_index) as u32];

        unsafe {
           context.device.handle.cmd_bind_descriptor_sets(
               command_buffer,
               material.pipeline.pipeline_bind_point,
               material.pipeline.pipeline_layout,
               first_set,
               &descriptor_sets,
               &dynamic_offsets,
           )
        }
    }


    pub fn destroy(&mut self, context: &VkContext) {
        self.per_frame.destroy(context);
        self.global.destroy(context);
    }

    pub fn init(context: &VkContext, descriptor_pool: DescriptorPool) -> Self {
        let global_uniform_buffer = GpuBufferBuilder::builder(&context)
            .descriptor_variant(DescriptorVariant::UniformStatic(vk::ShaderStageFlags::VERTEX))
            .descriptor_pool(descriptor_pool)
            .binding_index(0)
            .data_multiplier(1)
            .build::<UniformBufferGlobalData>();

        let per_frame_uniform_buffer = GpuBufferBuilder::builder(&context)
            .descriptor_variant(
                DescriptorVariant::UniformDynamic(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            )
            .descriptor_pool(descriptor_pool)
            .binding_index(1)
            .data_multiplier(penguin_config::vk_config::MAX_FRAMES_COUNT)
            .build::<UniformBufferFrameData>();

        Self {
            global: global_uniform_buffer,
            per_frame: per_frame_uniform_buffer,
        }
    }
}
