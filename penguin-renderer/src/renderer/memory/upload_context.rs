use ash::vk;
use crate::renderer::vk_types::VkContext;


// Context for memory transfers
#[derive(Copy, Clone)]
pub struct UploadContext {
    upload_fence: vk::Fence,
    command_pool: vk::CommandPool,
}

impl UploadContext {
    pub fn destroy(&mut self, context: &VkContext) {
        unsafe {
            context.device.destroy_fence(self.upload_fence, None);
            context.device.destroy_command_pool(self.command_pool, None);
        };
    }

    pub fn init(context: &VkContext) -> Self {
        Self {
            upload_fence: context.create_fence(vk::FenceCreateFlags::empty()),
            command_pool: context.alloc_command_pool(
                // todo: Check for and use a transfer queue index
                context.physical_device.graphics_queue_index,
                vk::CommandPoolCreateFlags::empty(),
            ),
        }
    }

    /// this makes it possible to submit commands immediately, without having to wait for the
    /// render loop sync. This can also be used to submit commands from a thread separate from
    /// the render loop thread.
    #[allow(unused)]
    pub fn immediate_submit<F: FnOnce(vk::CommandBuffer)>(&self, context: &VkContext, f: F) {
        let command_buffer = context.allocate_command_buffers(self.command_pool, 1);
        let command_buffer = command_buffer[0];

        context.begin_command_buffer(command_buffer, vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        f(command_buffer);

        context.end_command_buffer(command_buffer);

        let command_buffers = [command_buffer];
        let submit_info = vk::SubmitInfo::builder().command_buffers(&command_buffers);

        context.submit_to_graphics_queue(submit_info, self.upload_fence);

        // TODO: Connect it to the render semaphores?
        context.wait_for_fence(self.upload_fence, std::time::Duration::from_secs(10));

        context.reset_command_pool(self.command_pool, vk::CommandPoolResetFlags::empty());
    }
}
