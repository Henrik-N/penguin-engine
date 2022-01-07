use crate::renderer::vk_types::{Pipeline, VkContext};
use ash::vk;

impl VkContext {
    pub fn bind_pipeline(&self, pipeline: &Pipeline, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device.cmd_bind_pipeline(
                command_buffer,
                pipeline.pipeline_bind_point,
                pipeline.handle,
            );
        }
    }
}
