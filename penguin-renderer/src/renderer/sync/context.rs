use ash::vk;
use crate::renderer::vk_types::VkContext;

impl VkContext {
    pub fn wait_for_device_idle(&self) {
        log::debug!("Renderer: waiting for device idle..");
        unsafe { self.device.device_wait_idle().expect("Device: couldn't wait for idle") };
        log::debug!("Renderer: device now idle");
    }

    /// waits for a fence and then resets it
    pub fn wait_for_fence(&self, fence: vk::Fence, timeout: std::time::Duration) {
        self.wait_for_fences(&[fence], timeout)
    }


    /// waits for fences and then resets them
    pub fn wait_for_fences(&self, fences: &[vk::Fence], timeout: std::time::Duration) {
        unsafe {
            log::trace!("Waiting for fences...");
            self.device.wait_for_fences(fences, true, timeout.as_nanos() as _)
                .expect("Couldn't wait for fences. Timed out?");

            log::trace!("Resetting fence.");
            self.device.reset_fences(fences).expect("Couldn't reset fences.");
        }
    }

    pub fn create_fence(&self, flags: vk::FenceCreateFlags) -> vk::Fence {
        let create_info = vk::FenceCreateInfo::builder().flags(flags);

        unsafe { self.device.create_fence(&create_info, None)
        }.expect("failed to create fence")
    }


    pub fn create_semaphore(&self, flags: vk::SemaphoreCreateFlags) -> vk::Semaphore {
        // semaphores --------------
        let semaphore_create_info = vk::SemaphoreCreateInfo::builder().flags(flags);

        unsafe { self.device.create_semaphore(&semaphore_create_info, None) }
            .expect("Failed to create semaphore")
    }
}
