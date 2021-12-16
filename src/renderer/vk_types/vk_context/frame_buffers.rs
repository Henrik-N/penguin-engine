use ash::vk;
use crate::renderer::vk_types::{DepthImage, RenderPass, Swapchain, VkContext};

pub struct FrameBuffers {
    pub frame_buffers: Vec<vk::Framebuffer>,
}

impl FrameBuffers {
    pub fn get(&self, image_index: usize) -> vk::Framebuffer {
        self.frame_buffers.get(image_index)
            .expect(&format!("no frame buffer for the given index {}", image_index))
            .clone()
    }
}



impl FrameBuffers {
    pub fn destroy(&mut self, context: &VkContext) {
        log::debug!("Destroying frame buffers");
        self.frame_buffers.iter().for_each(|&framebuffer| {
            unsafe { context.device.handle.destroy_framebuffer(framebuffer, None) };
        });
    }

    pub fn init(
        context: &VkContext,
        swapchain: &Swapchain,
        depth_image: &DepthImage,
        render_pass: &RenderPass,
    ) -> Self {
        // frame buffers --------------
        let frame_buffers: Vec<vk::Framebuffer> = swapchain
            .image_views
            .iter()
            .map(|&image_view| {
                let attachments = [image_view, depth_image.image_view];
                let create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass.handle)
                    .attachments(&attachments)
                    .width(swapchain.extent.width)
                    .height(swapchain.extent.height)
                    .layers(1);

                unsafe { context.device.handle.create_framebuffer(&create_info, None) }
                    .expect("Couldn't create framebuffer")
            })
            .collect();
        Self { frame_buffers }
    }
}

