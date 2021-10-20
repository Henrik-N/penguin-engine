use ash::vk;

pub(crate) mod init {
    use ash::vk;

    pub fn create_command_pool_and_buffer(
        device: &ash::Device,
        graphics_queue_index: u32,
    ) -> (vk::CommandPool, vk::CommandBuffer) {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(graphics_queue_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER); // allows resetting of individual buffers from this pool

        log::trace!("Creating command pool");
        let command_pool = unsafe { device.create_command_pool(&command_pool_create_info, None) }
            .expect("Command pool couldn't be created.");

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .command_buffer_count(1)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers =
            unsafe { device.allocate_command_buffers(&command_buffer_allocate_info) }
                .expect("Command buffer couldn't be created");

        let command_buffer = command_buffers[0];

        (command_pool, command_buffer)
    }
}

pub fn record_submit_command_buffer<
    RenderPassFn: FnOnce(&ash::Device, vk::CommandBuffer, vk::Framebuffer),
>(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    // fence to wait for before resetting command buffer (and then to set to the newly queued command)
    fence: vk::Fence,
    submit_queue: vk::Queue,
    pipeline_wait_stage: &[vk::PipelineStageFlags],
    // semaphores the gpu should wait for before executing these commands
    wait_semaphores: &[vk::Semaphore],
    // signals the gpu should send when finished with these commands
    signal_samaphores: &[vk::Semaphore],
    frame_buffer: vk::Framebuffer,
    render_pass_fn: RenderPassFn,
) {
    // wait for fence / previous command buffer (aka wait until the GPU finished rendering the last frame in this case)
    unsafe {
        log::trace!("Waiting for fences...");
        device
            .wait_for_fences(&[fence], true, u64::MAX)
            .expect("Couldn't wait for fences. Timed out?");
        log::trace!("Resetting fence.");
        device
            .reset_fences(&[fence])
            .expect("Couldn't reset fences.");
    }

    // reset command buffer
    unsafe {
        device
            .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty()) //  vk::CommandBufferResetFlags::RELEASE_RESOURCES)
            .expect("Failed to reset command buffer");
    }

    let cmd_buffer_begin_info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe {
        log::trace!("Beginning command buffer");
        device
            .begin_command_buffer(command_buffer, &cmd_buffer_begin_info)
            .expect("Couldn't begin command buffer");
    }

    /////////////////////////////////////////// RENDER PASS BUFFER

    render_pass_fn(device, command_buffer, frame_buffer); // todo Image index not really needed but yeah

    ////////////////////////////////////////// END OF RENDER PASS

    unsafe {
        log::trace!("Ending command buffer");
        device
            .end_command_buffer(command_buffer)
            .expect("Couldn't end command buffer");
    }

    /*
    Prepare submission to render queue
    */

    let command_buffer = [command_buffer];

    let submit_info = vk::SubmitInfo::builder()
        .wait_dst_stage_mask(pipeline_wait_stage)
        .wait_semaphores(wait_semaphores)
        .signal_semaphores(signal_samaphores)
        .command_buffers(&command_buffer);

    unsafe {
        // the render fence will block until the graphics commands finish execution
        device.queue_submit(submit_queue, &[submit_info.build()], fence).expect("Couldn't submit command queue");
    }
}
