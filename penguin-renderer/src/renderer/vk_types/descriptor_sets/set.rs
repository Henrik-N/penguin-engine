use crate::renderer::memory::AllocatedBuffer;
use crate::renderer::vk_types::{DescriptorPool, DescriptorSetLayout, VkContext};
use ash::vk;

use anyhow::*;

#[derive(Default)]
pub struct DescriptorSetBuilder {
    handle: Option<vk::DescriptorSet>,
    layout: Option<DescriptorSetLayout>,
}
impl DescriptorSetBuilder {
    fn builder() -> Self {
        Self::default()
    }

    pub fn layout(mut self, layout: DescriptorSetLayout) -> Self {
        self.layout = Some(layout);
        self
    }

    pub fn build(
        self,
        context: &VkContext,
        descriptor_pool: &DescriptorPool,
    ) -> Result<DescriptorSet> {
        let layout = self
            .layout
            .expect("trying to build descriptor set without layout");

        let descriptor_set = unsafe {
            context.device.allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::builder()
                    .descriptor_pool(descriptor_pool.handle)
                    .set_layouts(&[layout.handle]),
            )
        }?[0];

        Ok(DescriptorSet {
            handle: descriptor_set,
            layout,
        })
    }
}

#[derive(Default, Clone)]
pub struct DescriptorSet {
    pub handle: vk::DescriptorSet,
    pub layout: DescriptorSetLayout,
}
impl DescriptorSet {
    pub fn destroy(&mut self, context: &VkContext) {
        self.layout.destroy(context);
    }
    pub fn builder() -> DescriptorSetBuilder {
        DescriptorSetBuilder::builder()
    }
}

#[derive(Default)]
pub struct DescriptorSetContainer {
    pub set: DescriptorSet,
    pub pipeline_layout: vk::PipelineLayout,
    pub allocated_buffers: Vec<AllocatedBuffer>,
    pub pipeline_bind_point: vk::PipelineBindPoint,
}

impl DescriptorSetContainer {
    pub fn handle(&self) -> vk::DescriptorSet {
        self.set.handle
    }

    pub fn destroy(&mut self, context: &VkContext) {
        self.allocated_buffers
            .iter_mut()
            .for_each(|buffer| buffer.destroy(context));
        self.set.destroy(context);
    }

    pub fn bind(&self, context: &VkContext, command_buffer: vk::CommandBuffer) {
        unsafe {
            context.device.cmd_bind_descriptor_sets(
                command_buffer,
                self.pipeline_bind_point,
                self.pipeline_layout,
                0,
                &[self.set.handle],
                &[],
            )
        }
    }
}
