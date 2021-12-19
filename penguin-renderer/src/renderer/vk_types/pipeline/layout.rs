use ash::vk;
use crate::renderer::vk_types::{DescriptorSetLayout, VkContext};

pub struct PipelineLayout {
    pub handle: vk::PipelineLayout,
}
impl std::ops::Deref for PipelineLayout {
    type Target = vk::PipelineLayout;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}
impl PipelineLayout {
    pub fn destroy(&mut self, context: &VkContext) {
        unsafe {
            context.device.destroy_pipeline_layout(self.handle, None);
        }
    }

    pub fn new(context: &VkContext, layouts: &[DescriptorSetLayout]) -> Self {
        let layout_handles = layouts.iter().map(|layout| layout.handle).collect::<Vec<_>>();

        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(layout_handles.as_slice())
            .build();

        let pipeline_layout = unsafe {
            context.device.create_pipeline_layout(&pipeline_layout_create_info, None)
        }.expect("couldn't create pipeline layout");

        Self { handle: pipeline_layout }
    }

    pub fn builder() -> PipelineLayoutBuilder {
        PipelineLayoutBuilder::default()
    }
}

#[derive(Default)]
pub struct PipelineLayoutBuilder {
    set_layouts: Vec<vk::DescriptorSetLayout>,
}
impl PipelineLayoutBuilder {
    pub fn add_layout(mut self, layout: vk::DescriptorSetLayout) -> Self {
        self.set_layouts.push(layout);
        self
    }

    pub fn build(self, context: &VkContext) -> PipelineLayout {
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(self.set_layouts.as_slice())
            .build();

        let pipeline_layout = unsafe {
            context.device.create_pipeline_layout(&pipeline_layout_create_info, None)
        }.expect("couldn't create pipeline layout");

        PipelineLayout { handle: pipeline_layout }
    }
}
