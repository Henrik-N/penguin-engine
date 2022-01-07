use crate::renderer::vk_types::VkContext;
use ash::vk;

#[derive(Eq, PartialEq, Clone)]
pub struct Pipeline {
    pub handle: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline_bind_point: vk::PipelineBindPoint,
}
impl std::ops::Deref for Pipeline {
    type Target = vk::Pipeline;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Pipeline {
    pub fn destroy(&mut self, context: &VkContext) {
        log::debug!("Pipeline gets destroyed!");
        unsafe {
            context.device.destroy_pipeline(self.handle, None);
            context
                .device
                .destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}
