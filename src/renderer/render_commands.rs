use ash::vk;
use crate::renderer::vk_types::*;
use super::{FrameData, FrameDataContainer};


pub struct SubmitRenderCommandsParams<'a> {
    pub context: &'a VkContext,
    pub swapchain: &'a Swapchain,
    pub frame_buffers: &'a FrameBuffers,
    pub pipeline_wait_stage_flags: &'a [vk::PipelineStageFlags],
    //
    pub frame_data: &'a FrameData,
}


pub struct RenderPassParams<'a> {
    //pub context: &'a VkContext,
    //pub command_buffer: vk::CommandBuffer,
    pub frame_buffer: vk::Framebuffer,
    pub frame_data: &'a FrameData,
}

pub fn submit_render_commands
<RenderPassFn: FnOnce(RenderPassParams)>(
    submit_render_commands_params: SubmitRenderCommandsParams,
    render_pass_fn: RenderPassFn,
) {
    let SubmitRenderCommandsParams {
        context,
        swapchain,
        frame_buffers,
        frame_data,
        pipeline_wait_stage_flags
    } = submit_render_commands_params;

    // find next image
    let swapchain_image_index = swapchain.acquire_next_swapchain_image(
        frame_data.presenting_complete_semaphore,
        vk::Fence::null(),
        std::time::Duration::from_secs(1)
    );

    let frame_buffer = frame_buffers.get(swapchain_image_index as _);


    //  record & submit command buffer to graphics queue
    {
        // ------ record command buffer ------
        {
            // wait for fence / previous command buffer
            // (aka wait until the GPU finished rendering the last frame in this case)
            context.wait_for_fence(frame_data.render_fence, std::time::Duration::MAX);

            context.reset_command_buffer(frame_data.command_buffer, vk::CommandBufferResetFlags::empty()); //  vk::CommandBufferResetFlags::RELEASE_RESOURCES
            context.begin_command_buffer(frame_data.command_buffer, vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            render_pass_fn(RenderPassParams {
                frame_buffer,
                frame_data
            });

            context.end_command_buffer(frame_data.command_buffer);
        }

        // ------ submit command buffer to graphics queue --------
        {
            let wait_semaphores = [frame_data.presenting_complete_semaphore];
            let signal_semaphores = [frame_data.rendering_complete_semaphore];
            let command_buffers = [frame_data.command_buffer];

            let submit_info = vk::SubmitInfo::builder()
                .wait_dst_stage_mask(pipeline_wait_stage_flags)
                .wait_semaphores(&wait_semaphores)
                .signal_semaphores(&signal_semaphores)
                .command_buffers(&command_buffers);

            context.submit_to_graphics_queue(submit_info, frame_data.render_fence);
        }
    }


    // after commands are submitted, wait for rending to complete and then display the image to the screen
    {
        let swapchains = [swapchain.handle];
        let wait_semaphores = [frame_data.rendering_complete_semaphore];
        let image_indices = [swapchain_image_index];

        let present_info = vk::PresentInfoKHR::builder()
            .swapchains(&swapchains)
            .wait_semaphores(&wait_semaphores)
            .image_indices(&image_indices);

        unsafe {
            swapchain.loader
                .queue_present(context.device.graphics_queue_handle, &present_info)
                .expect("Couldn't submit to present queue");
        }
    }
}
