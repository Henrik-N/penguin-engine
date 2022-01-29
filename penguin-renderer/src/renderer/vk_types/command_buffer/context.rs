use crate::renderer::vk_types::VkContext;
use ash::vk;

impl VkContext {
    pub fn reset_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        reset_flags: vk::CommandBufferResetFlags,
    ) {
        unsafe {
            self.device
                .reset_command_buffer(command_buffer, reset_flags) //  vk::CommandBufferResetFlags::RELEASE_RESOURCES
                .expect("Failed to reset command buffer");
        }
    }

    pub fn reset_command_pool(
        &self,
        command_pool: vk::CommandPool,
        reset_flags: vk::CommandPoolResetFlags,
    ) {
        unsafe {
            self.device
                .reset_command_pool(command_pool, reset_flags)
                .expect("failed to reset command pool");
        }
    }

    pub fn begin_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        usage_flags: vk::CommandBufferUsageFlags,
    ) {
        let cmd_buffer_begin_info = vk::CommandBufferBeginInfo::builder().flags(usage_flags);

        unsafe {
            log::trace!("Beginning command buffer");
            self.device
                .handle
                .begin_command_buffer(command_buffer, &cmd_buffer_begin_info)
                .expect("Couldn't begin command buffer");
        }
    }

    pub fn end_command_buffer(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            log::trace!("Ending command buffer");
            self.device
                .handle
                .end_command_buffer(command_buffer)
                .expect("Couldn't end command buffer");
        }
    }

    pub fn submit_to_graphics_queue(&self, submit_info: vk::SubmitInfo, fence: vk::Fence) {
        let submit_info = [submit_info];

        unsafe {
            // the render fence will block until the graphics commands finish execution
            self.device
                .queue_submit(self.device.graphics_queue_handle, &submit_info, fence)
                .expect("Couldn't submit command queue");
        }
    }
}

pub use alloc::*;
mod alloc {
    use super::*;

    impl VkContext {
        pub fn alloc_command_pool(
            &self,
            queue_family_index: u32,
            pool_flags: vk::CommandPoolCreateFlags,
        ) -> vk::CommandPool {
            log::trace!("Creating command pool");
            let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
                .queue_family_index(queue_family_index)
                .flags(pool_flags);

            log::trace!("Creating command pool");
            unsafe {
                self.device
                    .create_command_pool(&command_pool_create_info, None)
            }
            .expect("Command pool couldn't be created.")
        }

        pub fn allocate_command_buffers(
            &self,
            command_pool: vk::CommandPool,
            count: u32,
        ) -> Vec<vk::CommandBuffer> {
            let create_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(command_pool)
                .command_buffer_count(count)
                .level(vk::CommandBufferLevel::PRIMARY)
                .build();

            unsafe { self.device.allocate_command_buffers(&create_info) }
                .expect("couldn't allocate command buffers")
        }

        pub fn alloc_command_pool_with_buffers(
            &self,
            buffer_count: u32,
            queue_family_index: u32,
            pool_flags: vk::CommandPoolCreateFlags,
        ) -> (vk::CommandPool, Vec<vk::CommandBuffer>) {
            let command_pool = self.alloc_command_pool(queue_family_index, pool_flags);
            let command_buffers = self.allocate_command_buffers(command_pool, buffer_count);

            (command_pool, command_buffers)
        }
    }
}
