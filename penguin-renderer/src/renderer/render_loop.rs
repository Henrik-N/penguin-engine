use ash::vk;
use penguin_app::{
    ecs::*,
    window::Window,
};
use crate::math_vk_format::{Mat4, Vec3, Vec4};

use crate::renderer::{
    frame_data::{FrameData, FrameDataContainer},
    gpu_data::{GPUCameraData, GPUObjectData},
    memory::DeviceMemoryWriteInfo,
    render_commands::{self, SubmitRenderCommandsParams},
    resources::{MaterialsResource, MeshesResource, RenderObjectsResource},
    vk_types::{
        resource::DescriptorSetsResource,
        BindDescriptorSetsInfo, FrameBuffers, RenderPass, Swapchain, VkContext,
    },
};


pub struct DrawParams<'a> {
    pub aspect_ratio: f32,
    pub fov_y: f32,
    pub frame_data: &'a FrameData,
    pub frame_count: usize,
}

pub struct DrawResourceAccess<'a> {
    pub materials: &'a MaterialsResource,
    pub meshes: &'a MeshesResource,
    pub descriptor_sets: &'a DescriptorSetsResource,
    pub render_objects: &'a RenderObjectsResource,
}

pub struct DrawFunctionParams<'a> {
    pub context: &'a VkContext,
    pub params: DrawParams<'a>,
    pub resources: DrawResourceAccess<'a>,
}

fn draw(draw_params: DrawFunctionParams) {
    let DrawFunctionParams {
        context, params, resources
    } = draw_params;


    //let material = resources.materials.get("default");
    let mesh = resources.meshes.get("monkey");

    // create mvp matrix
    let camera_loc = Vec3::new(0.0, 10., -2.0);
    let camera_forward = Vec3::new(0.0, 0.0, 1.0);
    let camera_up = Vec3::new(0.0, 1.0, 0.0);

    let view = Mat4::look_at_rh(camera_loc, camera_loc + camera_forward, camera_up);

    let (z_near, z_far) = (0.1_f32, 200.0_f32);
    let projection = Mat4::perspective_rh(params.fov_y, params.aspect_ratio, z_near, z_far);


    context.bind_descriptor_sets(BindDescriptorSetsInfo {
        command_buffer: params.frame_data.command_buffer,
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
        pipeline_layout: resources.descriptor_sets.get_set(0).pipeline_layout,
        first_set: 0,
        descriptor_set_handles: &resources.descriptor_sets.get_set_handles(&[0, 1, /*2*/]),
    });


    let buffer_data = [
        GPUCameraData {
            data: Vec4::default(),
            proj_view: projection * view,
        }];

    let alignment = context.packed_uniform_buffer_range::<GPUCameraData>();
    let offset = (params.frame_data.frame_index) as u64 * alignment;
    resources.descriptor_sets.get_set(0).allocated_buffers[0].write_memory(
        &context,
        DeviceMemoryWriteInfo {
            data: &buffer_data,
            size: alignment,
            offset,
            alignment
        }
    );

    //let spin = (params.frame_count as f32 * 0.4).to_radians();
    let spin = 0.0_f32.to_radians();

    let buffer_data = [GPUObjectData {
        transform: Mat4::from_rotation_x(spin),
    }];


    let alignment = std::mem::align_of::<GPUObjectData>() as _;
    resources.descriptor_sets.get_set(1).allocated_buffers[0].write_memory(
        &context,
        DeviceMemoryWriteInfo {
            data: &buffer_data,
            size: alignment,
            offset: 0,
            alignment,
        }
    );



    // bind pipeline
    resources.render_objects.render_objects[0].material.bind(&context, params.frame_data.command_buffer);



    unsafe {
        // bind vertex buffers
        context.device.cmd_bind_vertex_buffers(
            params.frame_data.command_buffer,
            0,
            &[resources.render_objects[0].mesh.vertex_buffer.handle],
            &[0]);

        let instance_id = 0;

        // draw mesh
        context.device.cmd_draw(params.frame_data.command_buffer,
                                mesh.vertex_count as u32,
                                1,
                                0,
                                instance_id);
    }
}

#[system(for_each)]
pub fn render(
    #[resource] window: &Window,
    //#[resource] time: &PTime,
    #[resource] materials: &MaterialsResource,
    #[resource] meshes: &MeshesResource,
    #[resource] render_objects: &RenderObjectsResource,
    #[resource] descriptor_sets: &DescriptorSetsResource,
    context: &VkContext,
    swapchain: &Swapchain,
    frame_buffers: &FrameBuffers,
    frame_datas: &mut FrameDataContainer,
    render_pass: &RenderPass,
) {
    frame_datas.update_current_to_next_frame();

    let frame_data = frame_datas.get_current();

    render_commands::submit_render_commands(
        SubmitRenderCommandsParams {
            context,
            swapchain,
            frame_buffers,
            pipeline_wait_stage_flags: &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
            frame_data,
        },
        |frame_buffer: vk::Framebuffer | {
            render_pass_func(
                context,
                RenderPassParams {
                    frame_count: frame_datas.frame_count(),
                    context,
                    swapchain,
                    render_pass,
                    frame_data,
                    frame_buffer
                },
                DrawFunctionParams {
                    context,
                    params: DrawParams {
                        aspect_ratio: window.dimensions.width as f32 / window.dimensions.height as f32,
                        fov_y: 70.0_f32.to_radians(),
                        frame_data,
                        frame_count: frame_datas.frame_count(),
                    },
                    resources: DrawResourceAccess {
                        materials,
                        meshes,
                        descriptor_sets,
                        render_objects,
                    },
                }
            );
        }
    )
}

pub struct RenderPassParams<'a> {
    pub frame_count: usize,
    pub context: &'a VkContext,
    pub swapchain: &'a Swapchain,
    pub render_pass: &'a RenderPass,
    pub frame_data: &'a FrameData,
    pub frame_buffer: vk::Framebuffer,
}

// once per frame
fn render_pass_func(
    context: &VkContext,
    params: RenderPassParams,
    draw_params: DrawFunctionParams,
) {
    //let flash = f32::abs(f32::sin(params.frame_count as f32 / 120_f32));
    let color = [0.0_f32, 0., 0., 1.];

    let color_clear = vk::ClearColorValue { float32: color };

    let depth_clear = vk::ClearDepthStencilValue::builder().depth(1.0_f32).build();

    let clear_values = [
        vk::ClearValue { color: color_clear },
        vk::ClearValue {
            depth_stencil: depth_clear,
        },
    ];

    let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
        .render_pass(params.render_pass.handle)
        .framebuffer(params.frame_buffer)
        .render_area(vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: params.swapchain.extent,
        })
        .clear_values(&clear_values);


    unsafe {
        context.device.cmd_begin_render_pass(
            params.frame_data.command_buffer,
            &render_pass_begin_info,
            vk::SubpassContents::INLINE,
        );
    }

    draw(draw_params);

    unsafe {
        context.device.cmd_end_render_pass(params.frame_data.command_buffer);
    }
}
