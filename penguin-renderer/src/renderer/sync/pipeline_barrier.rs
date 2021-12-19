use ash::vk;
use crate::renderer::vk_types::{VkContext};

// https://gpuopen.com/learn/vulkan-barriers-explained/

#[derive(Default)]
pub struct PipelineBarrierBuilder<'a> {
    src_stage_mask: vk::PipelineStageFlags,
    dst_stage_mask: vk::PipelineStageFlags,
    dependency_flags: vk::DependencyFlags,

    memory_barriers: &'a [vk::MemoryBarrier],
    buffer_memory_barriers: &'a [vk::BufferMemoryBarrier],
    image_memory_barriers: &'a [vk::ImageMemoryBarrier]
}

impl<'a> PipelineBarrierBuilder<'a> {

    pub fn builder() -> Self {
        Self::default()
    }

    pub fn src_stage_mask(mut self, stage: vk::PipelineStageFlags) -> Self {
        self.src_stage_mask = stage;
        self
    }

    pub fn dst_stage_mask(mut self, stage: vk::PipelineStageFlags) -> Self {
        self.dst_stage_mask = stage;
        self
    }

    pub fn dependency_flags(mut self, flags: vk::DependencyFlags) -> Self {
        self.dependency_flags = flags;
        self
    }

    #[allow(unused)]
    pub fn memory_barriers(mut self, memory_barriers: &'a [vk::MemoryBarrier]) -> Self {
        self.memory_barriers = memory_barriers;
        self
    }

    #[allow(unused)]
    pub fn buffer_memory_barriers(mut self, buffer_memory_barriers: &'a [vk::BufferMemoryBarrier]) -> Self {
        self.buffer_memory_barriers = buffer_memory_barriers;
        self
    }

    #[allow(unused)]
    pub fn image_memory_barriers(mut self, image_memory_barriers: &'a [vk::ImageMemoryBarrier]) -> Self {
        self.image_memory_barriers = image_memory_barriers;
        self
    }

    pub fn build_exec(self, context: &VkContext, command_buffer: vk::CommandBuffer) {
       unsafe {
           context.device.cmd_pipeline_barrier(
               command_buffer,
               self.src_stage_mask,
               self.dst_stage_mask,
               self.dependency_flags,
               self.memory_barriers,
               self.buffer_memory_barriers,
               self.image_memory_barriers,
           );
       }
    }
}

