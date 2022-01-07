use super::*;
use crate::renderer::vk_types::VkContext;
use ash::vk;

pub use layout::*;
mod layout {
    use super::*;

    impl VkContext {
        pub fn create_descriptor_set_layout(
            &self,
            bindings: &[vk::DescriptorSetLayoutBinding],
        ) -> vk::DescriptorSetLayout {
            let layout_create_info =
                vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);

            unsafe {
                self.device
                    .create_descriptor_set_layout(&layout_create_info, None)
            }
            .expect("Couldn't create descriptor set layout")
        }
    }
}

pub use sets::*;
mod sets {
    use super::*;
    use ash::prelude::VkResult;

    impl VkContext {
        pub fn alloc_descriptor_set(
            &self,
            descriptor_pool: DescriptorPool,
            layout: vk::DescriptorSetLayout,
        ) -> VkResult<vk::DescriptorSet> {
            let descriptor_set_layouts = [layout];

            let descriptor_sets_allocate_info = vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(descriptor_pool.into())
                .set_layouts(&descriptor_set_layouts);

            // descriptor sets
            log::debug!("Allocating descriptor sets.");
            let allocated_descriptor_sets = unsafe {
                self.device
                    .allocate_descriptor_sets(&descriptor_sets_allocate_info)
            }?;

            Ok(allocated_descriptor_sets[0])
        }
    }

    pub struct BindDescriptorSetsInfo<'a> {
        pub command_buffer: vk::CommandBuffer,
        pub pipeline_bind_point: vk::PipelineBindPoint,
        pub pipeline_layout: vk::PipelineLayout,
        pub first_set: u32,
        pub descriptor_set_handles: &'a [vk::DescriptorSet],
    }

    impl VkContext {
        pub fn bind_descriptor_sets(&self, bind_info: BindDescriptorSetsInfo) {
            let BindDescriptorSetsInfo {
                command_buffer,
                pipeline_bind_point,
                pipeline_layout,
                first_set,
                descriptor_set_handles,
            } = bind_info;

            unsafe {
                self.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    pipeline_bind_point,
                    pipeline_layout,
                    first_set,
                    &descriptor_set_handles,
                    &[],
                )
            }
        }
    }
}
