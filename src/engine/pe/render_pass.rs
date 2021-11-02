use ash::vk;

pub(crate) struct PRenderPass;

impl PRenderPass {
    pub fn create_default_render_pass(
        device: &ash::Device,
        swapchain_format: &vk::Format,
    ) -> (vk::RenderPass, usize) {
        // description of image for writing render commands into
        let render_pass_attachments = [
            // color attachment
            vk::AttachmentDescription::builder()
                .format(*swapchain_format)
                // 1 sample, no MSAA
                .samples(vk::SampleCountFlags::TYPE_1)
                // clear image on attachment load
                .load_op(vk::AttachmentLoadOp::CLEAR)
                // store image for being read later
                .store_op(vk::AttachmentStoreOp::STORE)
                // no stencil
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                // starting layout doesn't matter
                .initial_layout(vk::ImageLayout::UNDEFINED)
                // layout ready for display
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .build(),
            // depth attachment
            vk::AttachmentDescription::builder()
                //.format(vk::Format::D32_SFLOAT) // TODO: Find supported format
                .format(vk::Format::D16_UNORM) // TODO: Find supported format
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::CLEAR)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .build(),
        ];

        let color_attachment_ref = [vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build()]; // layout optimal to be written into by rendering commands

        let depth_attachment_ref = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        let subpass = [vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_ref)
            .depth_stencil_attachment(&depth_attachment_ref)
            .build()];

        //any dependencies here...
        let dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ..Default::default()
        }];
        // let dependencies = [vk::SubpassDependency::builder()
        //     .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT |
        //                     vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
        //     .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT |
        //                     vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
        //     .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE |
        //                      vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
        //     .build()
        // ];

        let render_pass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&render_pass_attachments)
            .subpasses(&subpass)
            .dependencies(&dependencies);

        (
            unsafe { device.create_render_pass(&render_pass_create_info, None) }
                .expect("Couldn't create render pass!"),
            render_pass_attachments.len(),
        )
    }
}
