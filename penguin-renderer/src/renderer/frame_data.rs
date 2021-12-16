use ash::vk;
use crate::renderer::vk_types::VkContext;

pub struct FrameData {
    pub command_buffer: vk::CommandBuffer,

    pub render_complete_fence: vk::Fence, // render commands finished execution
    pub rendering_complete_semaphore: vk::Semaphore,
    pub presenting_complete_semaphore: vk::Semaphore,

    pub uniform_buffer_descriptor_set: vk::DescriptorSet,
    pub frame_index: u64,
}
impl FrameData {
    pub fn destroy(&mut self, context: &VkContext) {
        unsafe {
            context.device.destroy_semaphore(self.presenting_complete_semaphore, None);
            context.device.destroy_semaphore(self.rendering_complete_semaphore, None);
            context.device.destroy_fence(self.render_complete_fence, None);
        }
    }
}

pub struct FrameDataContainer {
    frame_count: usize,
    frame_index: usize,
    pub command_pool: vk::CommandPool,
    pub frame_datas: Vec<FrameData>,
}

impl FrameDataContainer {
    pub fn new(command_pool: vk::CommandPool, frame_datas: Vec<FrameData>) -> Self {
        Self {
            frame_count: 0,
            frame_index: 0,
            command_pool,
            frame_datas
        }
    }

    pub fn get_current(&self) -> &FrameData {
        &self.frame_datas[self.frame_index]
    }

    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    pub fn update_current_to_next_frame(&mut self) {
        self.increment_frame_count();
        self.update_frame_index();
    }

    fn increment_frame_count(&mut self) {
        if self.frame_count + 1 < usize::MAX {
            self.frame_count += 1;
        } else {
            self.frame_count = 0;
        }
    }

    fn update_frame_index(&mut self) {
        self.frame_index = (self.frame_index + 1) % crate::config::MAX_FRAMES_COUNT;
    }

    pub fn destroy(&mut self, context: &VkContext) {
        self.frame_datas.iter_mut().for_each(|frame_data| frame_data.destroy(context));
        unsafe {context.device.destroy_command_pool(self.command_pool, None)};
    }
}
