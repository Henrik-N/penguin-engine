use crate::renderer::vk_types::{Pipeline, Shader, VkContext};
use ash::vk;

impl Pipeline {
    pub fn builder(
        context: &VkContext,
        swapchain_extent: vk::Extent2D,
        render_pass: vk::RenderPass,
        pipeline_bindpoint: vk::PipelineBindPoint,
    ) -> PipelineBuilder {
        PipelineBuilder::default(context, swapchain_extent, render_pass, pipeline_bindpoint)
    }
}

pub struct PipelineBuilder<'a> {
    context: &'a VkContext,
    render_pass: vk::RenderPass,
    pipeline_bindpoint: vk::PipelineBindPoint,

    shaders: Vec<Shader<'a>>,
    vertex_input: vk::PipelineVertexInputStateCreateInfoBuilder<'a>,
    input_assembly: vk::PipelineInputAssemblyStateCreateInfoBuilder<'a>,
    viewports: Vec<vk::ViewportBuilder<'a>>,
    scissors: Vec<vk::Rect2DBuilder<'a>>,
    rasterization: vk::PipelineRasterizationStateCreateInfoBuilder<'a>,
    multisampling: vk::PipelineMultisampleStateCreateInfoBuilder<'a>,

    front_stencil_op: vk::StencilOpStateBuilder<'a>,
    back_stencil_op: vk::StencilOpStateBuilder<'a>,
    depth_stencil: vk::PipelineDepthStencilStateCreateInfoBuilder<'a>,

    color_blend_attachments: Vec<vk::PipelineColorBlendAttachmentStateBuilder<'a>>,
    color_blending: vk::PipelineColorBlendStateCreateInfoBuilder<'a>,

    #[allow(unused)] // todo
    vertex_shader_push_constants_byte_offset: Option<u32>,
    #[allow(unused)] // todo
    fragment_shader_push_constants_byte_offset: Option<u32>,

    pipeline_layout: Option<vk::PipelineLayout>,
}

impl<'a> PipelineBuilder<'a> {
    pub fn default(
        context: &'a VkContext,
        swapchain_extent: vk::Extent2D,
        render_pass: vk::RenderPass,
        pipeline_bindpoint: vk::PipelineBindPoint,
    ) -> Self {
        let shaders = Vec::new();

        let vertex_input = vk::PipelineVertexInputStateCreateInfo::builder()
            // .vertex_attribute_descriptions()
            // .vertex_binding_descriptions()
            ;

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .primitive_restart_enable(false)
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let viewports = vec![vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(swapchain_extent.width as f32)
            .height(swapchain_extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0)];

        let scissors = vec![vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(swapchain_extent)];

        let rasterization = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            //
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .line_width(1.0_f32)
            //
            .depth_bias_enable(false)
            .depth_bias_clamp(0.0_f32)
            .depth_bias_constant_factor(0.0_f32)
            .depth_bias_slope_factor(0.0_f32);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .min_sample_shading(1.0_f32)
            // .sample_mask()
            .alpha_to_one_enable(false)
            .alpha_to_coverage_enable(false);

        let front_stencil_op = vk::StencilOpState::builder()
            .fail_op(vk::StencilOp::KEEP)
            .pass_op(vk::StencilOp::KEEP)
            .depth_fail_op(vk::StencilOp::KEEP)
            .compare_op(vk::CompareOp::ALWAYS)
            .compare_mask(0_u32)
            .write_mask(0_u32)
            .reference(0_u32);

        let back_stencil_op = vk::StencilOpState::builder()
            .fail_op(vk::StencilOp::KEEP)
            .pass_op(vk::StencilOp::KEEP)
            .depth_fail_op(vk::StencilOp::KEEP)
            .compare_op(vk::CompareOp::ALWAYS)
            .compare_mask(0_u32)
            .write_mask(0_u32)
            .reference(0_u32);

        // let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::builder()
        //     .depth_test_enable(false)
        //     .depth_write_enable(false)
        //     .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
        //     .depth_bounds_test_enable(false)
        //     //
        //     .stencil_test_enable(false)
        //     //.front(stencil_state.clone())
        //     //.back(stencil_state)
        //     .max_depth_bounds(1.0_f32)
        //     .min_depth_bounds(1.0_f32);

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL) //LESS_OR_EQUAL)
            .depth_bounds_test_enable(false)
            //
            .stencil_test_enable(false)
            //.front(stencil_state.clone())
            //.back(stencil_state)
            .min_depth_bounds(0.0_f32)
            .max_depth_bounds(1.0_f32);

        let color_blend_attachments = vec![
            vk::PipelineColorBlendAttachmentState::builder()
                .blend_enable(false)
                .color_write_mask(vk::ColorComponentFlags::all()) // RGBA
                // RGB
                .src_color_blend_factor(vk::BlendFactor::ONE)
                .dst_color_blend_factor(vk::BlendFactor::ZERO)
                .color_blend_op(vk::BlendOp::ADD)
                // A
                .src_alpha_blend_factor(vk::BlendFactor::ONE)
                .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                .alpha_blend_op(vk::BlendOp::ADD), //
                                                   //.build()
        ];

        let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            //.attachments(&color_blend_attachment_states)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        Self {
            context,
            render_pass,
            pipeline_bindpoint,
            shaders,
            vertex_input,
            input_assembly,
            viewports,
            scissors,
            rasterization,
            multisampling,
            front_stencil_op,
            back_stencil_op,
            depth_stencil,
            color_blend_attachments,
            color_blending,
            vertex_shader_push_constants_byte_offset: None,
            fragment_shader_push_constants_byte_offset: None,
            pipeline_layout: None,
            //descriptor_set_layouts: vec![],
        }
    }

    pub fn pipeline_layout(mut self, pipeline_layout: vk::PipelineLayout) -> Self {
        self.pipeline_layout = Some(pipeline_layout);
        self
    }

    pub fn vertex_input(
        mut self,
        vertex_binding_descriptions: &'a [vk::VertexInputBindingDescription],
        vertex_attribute_descriptions: &'a [vk::VertexInputAttributeDescription],
    ) -> Self {
        self.vertex_input = self
            .vertex_input
            .vertex_binding_descriptions(&vertex_binding_descriptions)
            .vertex_attribute_descriptions(&vertex_attribute_descriptions);
        self
    }

    #[allow(dead_code)]
    pub fn wireframe_mode(mut self) -> Self {
        self.rasterization = self.rasterization.polygon_mode(vk::PolygonMode::LINE);
        self
    }

    /// Specify a list of shaders to be compiled at runtime.
    /// Searches in src/shaders/ for the listed shaders and compiles them.
    ///
    /// The function detects the shader type based on the file type of the
    /// passed file (.frag/.vert/.comp).
    /// # Arguments
    /// * `shader_names` - A list of shaders in the path src/shaders.
    ///
    pub fn shaders(mut self, shader_names: &[&str]) -> Self {
        self.shaders = shader_names
            .into_iter()
            .map(|shader_name| Shader::new(self.context, shader_name))
            .collect();

        self
    }

    #[allow(unused)]
    fn add_vertex_shader_push_constants<PushConstantType>(&mut self, size: u32) {
        log::trace!("Adding vertex shader push constants of size: {}", size);
        log::debug!("Adding vertex shader push constants of size: {}", size);

        if let Some(offset) = &mut self.vertex_shader_push_constants_byte_offset {
            *offset += size;
        } else {
            self.vertex_shader_push_constants_byte_offset = Some(size);
        }
    }

    #[allow(unused)]
    fn add_fragment_shader_push_constants<PushConstantType>(&mut self, size: u32) {
        log::trace!("Adding fragment shader push constants of size: {}", size);
        log::debug!("Adding fragment shader push constants of size: {}", size);

        if let Some(offset) = &mut self.fragment_shader_push_constants_byte_offset {
            *offset += size;
        } else {
            self.fragment_shader_push_constants_byte_offset = Some(size);
        }
    }

    /// Creates the pipeline and returns the compiled shaders for reuse if
    pub fn build(self) -> Pipeline {
        let shader_stages: Vec<vk::PipelineShaderStageCreateInfo> = self
            .shaders
            .iter()
            .map(|shader| shader.shader_stage_create_info())
            .collect();

        // Vertex input
        let vertex_input = self.vertex_input;

        // Input assembly
        let input_assembly = self.input_assembly;

        // Tesselation
        //

        // Viewport state
        let viewports: Vec<vk::Viewport> =
            self.viewports.into_iter().map(|vp| vp.build()).collect();

        let scissors: Vec<vk::Rect2D> = self
            .scissors
            .into_iter()
            .map(|scissor| scissor.build())
            .collect();

        let viewport = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);

        // Rasterization
        let rasterization = self.rasterization;

        // Multisampling
        let multisampling = self.multisampling;

        // Depth stencil
        let back_op = self.back_stencil_op.build();
        let front_op = self.front_stencil_op.build();

        let depth_stencil = self.depth_stencil.front(front_op).back(back_op);

        // Color blending
        let color_blend_attachments: Vec<vk::PipelineColorBlendAttachmentState> = self
            .color_blend_attachments
            .into_iter()
            .map(|color_attachment| color_attachment.build())
            .collect();

        let color_blending = self.color_blending.attachments(&color_blend_attachments);

        // Pipeline layout
        //

        // let byte_offsets: &[Option<u32>] = &[
        //     self.vertex_shader_push_constants_byte_offset,
        //     self.fragment_shader_push_constants_byte_offset,
        // ];

        // FIXME: Using push constants results in a validation error that
        // says that vk::PushConstantRange doesn't contain vk::ShaderStageFlags::VERTEX,
        // even though I am providing it.
        // I'm guessing some memory is getting cleaned up somewhere it shouldn't.

        //let mut current_offset = 0;
        //let push_constant_ranges: Vec<vk::PushConstantRange> = byte_offsets
        //    .iter()
        //    .enumerate()
        //    .filter_map(|(i, &size)| {
        //        if let Some(size) = size {
        //            log::debug!(
        //                "Push constant index {} has size {} and offset {}.",
        //                i,
        //                size,
        //                current_offset
        //            );

        //            let stage_flag = match i {
        //                0 => vk::ShaderStageFlags::VERTEX,
        //                _ => vk::ShaderStageFlags::FRAGMENT,
        //            };
        //            log::debug!("Shader stage: {:?}", stage_flag);

        //            let range = vk::PushConstantRange::builder()
        //                .offset(current_offset)
        //                .size(size)
        //                .stage_flags(stage_flag)
        //                .build();
        //            current_offset += size;
        //            Some(range)
        //        } else {
        //            None
        //        }
        //    })
        //    .collect();

        //push_constant_ranges.iter().for_each(|pc| {
        //    log::debug!(
        //        "Push constant with flag: {:#?} has range offset: {}",
        //        pc.stage_flags,
        //        pc.offset
        //    );
        //});

        //let descriptor_set_layouts = self.descriptor_set_layouts;

        //let pipeline_layout_create_info = self
        //    .pipeline_layout
        //    .set_layouts(&descriptor_set_layouts)
        //    //.push_constant_ranges(&push_constant_ranges)
        //    ;

        //let pipeline_layout = unsafe {
        //    self.device
        //        .create_pipeline_layout(&pipeline_layout_create_info, None)
        //}
        //    .expect("Couldn't create pipeline layout");

        let pipeline_layout = self
            .pipeline_layout
            .expect("no pipeline layout provided to pipeline builder");

        // Render pass
        let render_pass = self.render_pass;

        let graphics_pipeline_create_infos = [vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly)
            // .tessellation_state()
            .viewport_state(&viewport)
            .rasterization_state(&rasterization)
            .multisample_state(&multisampling)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&color_blending)
            // .dynamic_state()
            .layout(pipeline_layout)
            //
            .render_pass(render_pass)
            .subpass(0)
            // .base_pipeline_handle()
            .base_pipeline_index(-1)
            //
            .build()];

        let graphics_pipelines = unsafe {
            self.context.device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                &graphics_pipeline_create_infos,
                None,
            )
        }
        .expect("Couldn't create graphics pipeline");

        Pipeline {
            handle: graphics_pipelines[0],
            pipeline_layout,
            pipeline_bind_point: self.pipeline_bindpoint,
            //descriptor_set_layouts,
        }
    }
}
