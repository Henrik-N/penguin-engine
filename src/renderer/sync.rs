use ash::vk;
use crate::renderer::vk_types::VkContext;

impl VkContext {
    pub fn wait_for_device_idle(&self) {
        log::debug!("Renderer: waiting for device idle..");
        unsafe {
            self.device
                .handle
                .device_wait_idle()
                .expect("Device: couldn't wait for idle");
        }
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
            self.device.handle
                .wait_for_fences(fences, true, timeout.as_nanos() as _).expect("Couldn't wait for fences. Timed out?");

            log::trace!("Resetting fence.");
            self.device.handle
                .reset_fences(fences).expect("Couldn't reset fences.");
        }
    }
}
