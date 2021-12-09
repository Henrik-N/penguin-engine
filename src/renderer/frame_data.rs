use ash::vk;
use crate::renderer::renderer_internal;
use crate::renderer::vk_types::VkContext;

pub struct FrameDataContainer {
    pub frame_datas: Vec<FrameData>,
}
impl FrameDataContainer {
    pub fn destroy(&mut self, context: &VkContext) {
        self.frame_datas.iter_mut().for_each(|frame_data| frame_data.destroy(context));
    }

    pub fn get(&self, index: usize) -> &FrameData {
        self.frame_datas.get(index).expect("yeet")
    }
}


pub struct FrameData {
    pub command_pool: vk::CommandPool,
    pub command_buffer: vk::CommandBuffer,

    pub render_fence: vk::Fence,
    pub rendering_complete_semaphore: vk::Semaphore,
    pub presenting_complete_semaphore: vk::Semaphore,

    pub frame_index: usize,
}
impl FrameData {
    pub fn destroy(&mut self, context: &VkContext) {
        unsafe {
            context.device.handle.destroy_semaphore(self.presenting_complete_semaphore, None);
            context.device.handle.destroy_semaphore(self.rendering_complete_semaphore, None);
            context.device.handle.destroy_fence(self.render_fence, None);
            context.device.handle.destroy_command_pool(self.command_pool, None);
        }
    }

    pub fn new(context: &VkContext, frame_index: usize) -> Self {
        // command pool and command buffer ---------
        let (command_pool, command_buffers) =
            context.alloc_command_pool_with_buffers(1);

        let command_buffer = command_buffers[0];


        // fences ---------
        let render_fence_create_info =
            vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED); // start signaled, to wait for it before the first gpu command

        let render_fence = unsafe { context.device.handle.create_fence(&render_fence_create_info, None) }
            .expect("Failed to create render fence.");

        // semaphores --------------
        let semaphore_create_info = vk::SemaphoreCreateInfo::default();

        let rendering_complete_semaphore =
            unsafe { context.device.handle.create_semaphore(&semaphore_create_info, None) }
                .expect("Failed to create semaphore");
        let presenting_complete_semaphore =
            unsafe { context.device.handle.create_semaphore(&semaphore_create_info, None) }
                .expect("Failed to create semaphore");

        Self {
            command_pool,
            command_buffer,
            render_fence,
            rendering_complete_semaphore,
            presenting_complete_semaphore,
            frame_index,
        }
    }
}
