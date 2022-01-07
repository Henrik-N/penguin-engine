use crate::renderer::vk_types::vk_context::VkContext;
/// ------------------------- VK COMPONENTS ----------------------------------
use crate::renderer::vk_types::{DepthImage, FrameBuffers, RenderPass, Swapchain};

pub struct VkComponents {
    pub swapchain: Swapchain,
    pub depth_image: DepthImage,
    pub render_pass: RenderPass,
    pub frame_buffers: FrameBuffers,
}

pub fn init_vk_components(
    window: &penguin_app::window::Window,
    context: &VkContext,
) -> VkComponents {
    log::trace!("Creating swapchain.");
    let swapchain = Swapchain::init(window, context);
    // ///////////////////////////////////////
    log::trace!("Creating depth image.");
    let depth_image = DepthImage::init(context, &swapchain);
    // ///////////////////////////////////////
    log::trace!("Creating render pass.");
    let render_pass = RenderPass::init(context, &swapchain);
    // ///////////////////////////////////////
    log::trace!("Creating frame buffers.");
    let frame_buffers = FrameBuffers::init(context, &swapchain, &depth_image, &render_pass);
    // ///////////////////////////////////////

    VkComponents {
        swapchain,
        depth_image,
        render_pass,
        frame_buffers,
    }
}
