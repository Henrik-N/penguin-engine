use ash::vk;
use crate::renderer::memory::AllocatedBuffer;
use crate::renderer::vk_types::{DescriptorSetLayoutContainer, VkContext};

#[derive(Default)]
pub struct DescriptorSetContainer {
    pub handle: vk::DescriptorSet,
    pub layout: DescriptorSetLayoutContainer,
    pub allocated_buffers: Vec<AllocatedBuffer>,
    pub pipeline_bind_point: vk::PipelineBindPoint,
}

impl DescriptorSetContainer {
    pub fn destroy(&mut self, context: &VkContext) {
        self.allocated_buffers.iter_mut().for_each(|buffer| buffer.destroy(context));
        self.layout.destroy(context);
    }

    pub fn bind(&self, context: &VkContext, command_buffer: vk::CommandBuffer) {
        unsafe {
            context.device.cmd_bind_descriptor_sets(
                command_buffer,
                self.pipeline_bind_point,
                self.layout.pipeline,
                0,
                &[self.handle],
                &[],
            )
        }
    }
}
