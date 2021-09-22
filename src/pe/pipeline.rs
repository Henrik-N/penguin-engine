use crate::pe;
use ash::version::DeviceV1_0;
use ash::vk;

pub struct Pipeline {
    graphics_pipeline: vk::Pipeline,
    vertex_shader: pe::Shader,
    fragment_shader: pe::Shader,
    pipeline_layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
}

impl Pipeline {
    pub fn drop(&self, device: &pe::Device) {
        self.vertex_shader.drop(&device);
        self.fragment_shader.drop(&device);

        unsafe {
            device.logical_device.destroy_pipeline_layout(self.pipeline_layout, None);
            device.logical_device.destroy_render_pass(self.render_pass, None);
            device.logical_device.destroy_pipeline(self.graphics_pipeline, None);
        }
    }
}


impl pe::Device {
    pub fn build_graphics_pipeline(&self) -> PipelineBuilder {
        PipelineBuilder::default(&self)
    }
}

pub struct PipelineBuilder<'a> {
    device: &'a pe::Device,
    pub input_assembly: vk::PipelineInputAssemblyStateCreateInfoBuilder<'a>,
    pub viewport: vk::ViewportBuilder<'a>,
    pub scissors: vk::Rect2DBuilder<'a>,
    pub rasterization: vk::PipelineRasterizationStateCreateInfoBuilder<'a>,
    pub multisampling: vk::PipelineMultisampleStateCreateInfoBuilder<'a>,
    pub depth_stencil: vk::PipelineDepthStencilStateCreateInfoBuilder<'a>,
    pub color_blend_attachments: vk::PipelineColorBlendAttachmentStateBuilder<'a>,
    pub pipeline_layout: vk::PipelineLayoutCreateInfoBuilder<'a>,
}

impl<'a> PipelineBuilder<'a> {
    fn default(device: &'a pe::Device) -> PipelineBuilder<'a> {
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let viewport = vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(pe::window::WINDOW_WIDTH as f32)
            .min_depth(0.0)
            .max_depth(0.0);

        let scissors = vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(vk::Extent2D {
                width: pe::window::WINDOW_WIDTH,
                height: pe::window::WINDOW_HEIGHT,
            });

        let rasterization = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL) // toggle for wireframe
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE) // backface culling
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .depth_bias_constant_factor(0.0)
            .depth_bias_clamp(0.0)
            .depth_bias_slope_factor(0.0);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false)
            .min_sample_shading(1.0)
            // sample_mask
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false);

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            // .front
            // .back
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0)
            .stencil_test_enable(false);

        let color_blend_attachments =
            vk::PipelineColorBlendAttachmentState::builder()
                .blend_enable(false)
                .src_color_blend_factor(vk::BlendFactor::ONE)
                .dst_color_blend_factor(vk::BlendFactor::ZERO)
                .color_blend_op(vk::BlendOp::ADD)
                .src_alpha_blend_factor(vk::BlendFactor::ONE)
                .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                .alpha_blend_op(vk::BlendOp::ADD)
                .color_write_mask(vk::ColorComponentFlags::all()) //rgba
            ;

        let pipeline_layout = vk::PipelineLayoutCreateInfo::builder();

        PipelineBuilder {
            device,
            input_assembly,
            viewport,
            scissors,
            rasterization,
            multisampling,
            depth_stencil,
            color_blend_attachments,
            pipeline_layout,
        }
    }

    pub fn build(self) -> pe::Pipeline {
        let vertex_shader = pe::Shader::create_and_compile(&self.device, "simple.vert");
        // let vert = vertex_shader.create_shader_module(&device);

        let fragment_shader = pe::Shader::create_and_compile(&self.device, "simple.frag");
        //let frag = fragment_shader.create_shader_module(&device);

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo {
                module: vertex_shader.shader_module,
                p_name: pe::Shader::get_entry_name(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                module: fragment_shader.shader_module,
                p_name: pe::Shader::get_entry_name(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];

        let viewport = [self.viewport.build()];
        let scissors = [self.scissors.build()];
        let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewport)
            .scissors(&scissors);

        let color_blend_attachments_info = [self.color_blend_attachments.build()];

        let color_blend_state_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op(vk::LogicOp::CLEAR)
            .attachments(&color_blend_attachments_info)
            .build();

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_states);


        let pipeline_layout_create_info = self.pipeline_layout.build();
        let pipeline_layout = unsafe {
            self.device.logical_device.create_pipeline_layout(&pipeline_layout_create_info, None)
        }.expect("Failed to create pipeline layout.");

        // todo: Deallocation / allocation callback


        // RENDER PASS
        let render_pass = Self::create_render_pass(&self.device);
        let vert_input = vk::PipelineVertexInputStateCreateInfo::builder().build();

        let graphics_pipeline_create_infos = [vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vert_input)
            .input_assembly_state(&self.input_assembly)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&self.rasterization)
            .multisample_state(&self.multisampling)
            .depth_stencil_state(&self.depth_stencil)
            .color_blend_state(&color_blend_state_info)
            .dynamic_state(&dynamic_state_info)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .build()];

        let graphics_pipelines = unsafe {
            self.device.logical_device
                .create_graphics_pipelines(vk::PipelineCache::null(),
                                           &graphics_pipeline_create_infos,
                                           None)
        }.expect("Failed to create graphics pipeline");

        pe::Pipeline {
            graphics_pipeline: graphics_pipelines[0],
            vertex_shader,
            fragment_shader,
            pipeline_layout,
            render_pass,
        }
    }


    fn create_render_pass(device: &pe::Device) -> vk::RenderPass {
        let renderpass_attachments = [
            vk::AttachmentDescription {
                format: device.query_swapchain_support().surface_color_formats[0].format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                ..Default::default()
            },
            vk::AttachmentDescription {
                format: vk::Format::D16_UNORM,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                initial_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                ..Default::default()
            },
        ];

        let color_attachment_refs = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];
        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };
        let dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ..Default::default()
        }];

        let subpasses = [vk::SubpassDescription::builder()
            .color_attachments(&color_attachment_refs)
            .depth_stencil_attachment(&depth_attachment_ref)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .build()];

        let render_pass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&renderpass_attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        unsafe {
            device.logical_device
                .create_render_pass(&render_pass_create_info, None)
                .unwrap()
        }
    }
}

// #[derive(Debug)]
// pub struct PipelineConfig_old {
//     shader_stage_create_infos: Vec<vk::PipelineShaderStageCreateInfo>,
//     vertex_input_assembly_state_info: vk::PipelineInputAssemblyStateCreateInfo,
//     viewport: vk::Viewport,
//     scissor: vk::Rect2D,
//     rasterization_state_info: vk::PipelineRasterizationStateCreateInfo,
//     multisample_state_info: vk::PipelineMultisampleStateCreateInfo,
//     depth_stencil_state_info: vk::PipelineDepthStencilStateCreateInfo,
//     color_blend_attachment_states: Vec<vk::PipelineColorBlendAttachmentState>,
//     pipeline_layout_create_info: Option<vk::PipelineLayoutCreateInfo>,
//     render_pass: Option<vk::RenderPass>,
//     render_subpass: u32,
// }
//
// impl PipelineConfig_old {
//     fn pipeline_layout_info(mut self, pipeline_layout: vk::PipelineLayoutCreateInfo) -> Self {
//         self.pipeline_layout_create_info = Some(pipeline_layout);
//         self
//     }
//
//     fn render_pass(mut self, render_pass: vk::RenderPass) -> Self {
//         self.render_pass = Some(render_pass);
//         self
//     }
//
//     fn build(self, device: &Device) -> Vec<vk::Pipeline> {
//         let viewport_state_info = vk::PipelineViewportStateCreateInfo {
//             viewport_count: 1,
//             p_viewports: &self.viewport, // Rust will keep variable alive since there's a living reference to it, even though the variable goes out of scope
//             scissor_count: 1,
//             p_scissors: &self.scissor,
//             ..Default::default()
//         };
//
//         let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo {
//             vertex_binding_description_count: 0,
//             // p_vertex_binding_descriptions: (),
//             vertex_attribute_description_count: 0,
//             // p_vertex_attribute_descriptions: ()
//             ..Default::default()
//         };
//
//         let color_blend_state_info = vk::PipelineColorBlendStateCreateInfo::builder()
//             .logic_op_enable(false)
//             .logic_op(vk::LogicOp::COPY)
//             .attachments(&self.color_blend_attachment_states.as_slice())
//             .blend_constants([0.0, 0.0, 0.0, 0.0])
//             .build();
//
//         let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
//         let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder()
//             .dynamic_states(&dynamic_states);
//
//
//         let pipeline_layout = unsafe {
//             device
//                 .logical_device
//                 .create_pipeline_layout(&self.pipeline_layout_create_info, None) // todo: allocation callbacks
//         }
//             .unwrap();
//
//         let graphics_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
//             .stages(&self.shader_stage_create_infos.as_slice())
//             .vertex_input_state(&vertex_input_state_info)
//             .input_assembly_state(&self.vertex_input_assembly_state_info)
//             .viewport_state(&viewport_state_info)
//             .rasterization_state(&self.rasterization_state_info)
//             .multisample_state(&self.multisample_state_info)
//             .depth_stencil_state(&self.depth_stencil_state_info)
//             .color_blend_state(&color_blend_state_info)
//             .dynamic_state(&dynamic_state_info)
//             .layout(pipeline_layout)
//             .render_pass(self.render_pass.expect("No render pass specified for pipeline"));
//
//         unsafe {
//             device
//                 .logical_device
//                 .create_graphics_pipelines(
//                     vk::PipelineCache::null(),
//                     &[graphics_pipeline_info.build()],
//                     None,
//                 )
//         }
//             .expect("Unable to create graphics pipeline")
//
//         // todo: add deallocation
//     }
//
//     fn builder(vertex_shader_module: vk::ShaderModule, fragment_shader_module: vk::ShaderModule) -> Self {
//         let shader_entry_function_name = std::ffi::CString::new("main").unwrap();
//         let shader_stage_create_infos = vec![
//             vk::PipelineShaderStageCreateInfo {
//                 module: vertex_shader_module,
//                 p_name: shader_entry_function_name.as_ptr(), // todo: Will this work as the variable goes out of scope?
//                 stage: vk::ShaderStageFlags::VERTEX,
//                 ..Default::default()
//             },
//             vk::PipelineShaderStageCreateInfo {
//                 module: fragment_shader_module,
//                 p_name: shader_entry_function_name.as_ptr(),
//                 stage: vk::ShaderStageFlags::FRAGMENT,
//                 ..Default::default()
//             },
//         ];
//
//         let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
//             topology: vk::PrimitiveTopology::TRIANGLE_LIST,
//             ..Default::default()
//         };
//
//         let viewport = vk::Viewport {
//             x: 0.0,
//             y: 0.0,
//             width: pe::window::WINDOW_WIDTH as f32,
//             height: pe::window::WINDOW_HEIGHT as f32,
//             min_depth: 0.0,
//             max_depth: 0.0,
//             ..Default::default()
//         };
//
//         let scissor = vk::Rect2D {
//             offset: vk::Offset2D { x: 0, y: 0 },
//             extent: vk::Extent2D {
//                 width: pe::window::WINDOW_WIDTH,
//                 height: pe::window::WINDOW_HEIGHT,
//             },
//             ..Default::default()
//         };
//
//         let rasterization_state_info =
//             vk::PipelineRasterizationStateCreateInfo {
//                 depth_clamp_enable: vk::FALSE,
//                 rasterizer_discard_enable: vk::FALSE,
//                 polygon_mode: vk::PolygonMode::FILL, // toggle for wireframe
//                 line_width: 1.0,
//                 cull_mode: vk::CullModeFlags::NONE, // backface culling
//                 front_face: vk::FrontFace::CLOCKWISE,
//                 depth_bias_enable: vk::FALSE,
//                 depth_bias_constant_factor: 0.0,
//                 depth_bias_clamp: 0.0,
//                 depth_bias_slope_factor: 0.0,
//                 ..Default::default()
//             };
//
//         let multisample_state_info =
//             vk::PipelineMultisampleStateCreateInfo {
//                 rasterization_samples: vk::SampleCountFlags::TYPE_1,
//                 sample_shading_enable: vk::FALSE,
//                 min_sample_shading: 1.0_f32,
//                 // p_sample_mask: (),
//                 alpha_to_coverage_enable: vk::FALSE,
//                 alpha_to_one_enable: vk::FALSE,
//                 ..Default::default()
//             };
//
//         let depth_stencil_state_info = vk::PipelineDepthStencilStateCreateInfo {
//             depth_test_enable: vk::TRUE,
//             depth_write_enable: vk::TRUE,
//             depth_compare_op: vk::CompareOp::LESS,
//             depth_bounds_test_enable: vk::FALSE,
//             front: Default::default(),
//             back: Default::default(),
//             min_depth_bounds: 0.0,
//             max_depth_bounds: 1.0,
//             stencil_test_enable: vk::FALSE,
//             ..Default::default()
//         };
//
//         let color_blend_attachment_states = vec![vk::PipelineColorBlendAttachmentState {
//             blend_enable: vk::FALSE,
//             src_color_blend_factor: vk::BlendFactor::ONE,
//             dst_color_blend_factor: vk::BlendFactor::ZERO,
//             color_blend_op: vk::BlendOp::ADD,
//             src_alpha_blend_factor: vk::BlendFactor::ONE,
//             dst_alpha_blend_factor: vk::BlendFactor::ZERO,
//             alpha_blend_op: vk::BlendOp::ADD,
//             color_write_mask: vk::ColorComponentFlags::R
//                 | vk::ColorComponentFlags::G
//                 | vk::ColorComponentFlags::B
//                 | vk::ColorComponentFlags::A,
//         }];
//
//
//         Self {
//             shader_stage_create_infos,
//             vertex_input_assembly_state_info,
//             viewport,
//             scissor,
//             rasterization_state_info,
//             multisample_state_info,
//             depth_stencil_state_info,
//             color_blend_attachment_states,
//             pipeline_layout_create_info: None,
//             render_pass: None,
//             render_subpass: 0,
//         }
//     }
// }

