use crate::renderer::vk_types::{DescriptorSetContainer, Pipeline, VkContext};
use ash::vk;

#[derive(Clone)]
pub struct Material {
    pub pipeline: Pipeline,
    //pub descriptor_set: DescriptorSetContainer,
    //pub pipeline_layout: vk::PipelineLayout,
}
impl PartialEq for Material {
    fn eq(&self, other: &Self) -> bool {
        self.pipeline == other.pipeline
    }
}
impl Eq for Material {}

impl Material {
    pub fn destroy(&mut self, context: &VkContext) {
        self.pipeline.destroy(&context);
    }

    pub fn from_pipeline(pipeline: Pipeline) -> Self {
        Self { pipeline }
    }

    pub fn bind(&self, context: &VkContext, command_buffer: vk::CommandBuffer) {
        context.bind_pipeline(&self.pipeline, command_buffer);
    }
}
