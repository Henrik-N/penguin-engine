

mod render_commands;
mod frame_data;
pub(crate) mod memory;
pub(crate) mod vk_types;
pub mod sync;
mod resources;
mod render_objects;
mod renderer_internal;
mod debug;
mod startup_shutdown;
pub mod descriptor_sets;

use penguin_app::ecs::*;
use std::ops::Deref;
use ash::vk;
use penguin_app::window::Window;

use vk_types::vk_components::*;
use vk_types::vk_context::*;
use crate::renderer::vk_types::VkContext;
use crate::renderer::frame_data::{FrameData, FrameDataContainer};
use crate::renderer::render_commands::{RenderPassParams, SubmitRenderCommandsParams};
use crate::render_objects::{Material, Vertex};

use penguin_app::time_plugin::PTime;

use resources::*;


pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn startup(&mut self, resources: &mut Resources) -> Vec<Step> {

        resources.insert(MeshesResource::default());
        resources.insert(MaterialsResource::default());

        Schedule::builder()
            .add_thread_local(startup_shutdown::renderer_startup_system())
            .build()
            .into_vec()
    }

    fn run() -> Vec<Step> {
        Schedule::builder()
            .add_thread_local(render_system(0, 0))
            .build().into_vec()
    }

    fn shutdown() -> Vec<Step> {
        Schedule::builder()
            .add_thread_local(startup_shutdown::renderer_shutdown_system())
            .build()
            .into_vec()
    }
}


fn draw(
    window: &Window,
    context: &VkContext,
    frame_data: &FrameData,
    uniform_buffers: &UniformBuffers,
    materials: &MaterialsResource,
    meshes: &MeshesResource,
    frame_count: usize) {


    let material = materials.get("default");
    let mesh = meshes.get("monkey");

    // create mvp matrix
    let camera_loc = Vec3::new(0.0, 0.0, -3.0);
    let camera_forward = Vec3::new(0.0, 0.0, 1.0);
    let camera_up = Vec3::new(0.0, 1.0, 0.0);

    let view = Mat4::look_at_rh(camera_loc, camera_loc + camera_forward, camera_up);

    let fov_y = 70.0_f32.to_radians();

    let aspect_ratio = window.dimensions.width as f32 / window.dimensions.height as f32;
    let (z_near, z_far) = (0.1_f32, 200.0_f32);
    let projection = Mat4::perspective_rh(fov_y, aspect_ratio, z_near, z_far);
    //let model = Mat4::from_translation(Vec3::new(0., 0.5, 0.));
    let spin = (frame_count as f32 * 0.4).to_radians();
    let model = Mat4::from_rotation_x(spin);

    let mesh_matrix = projection * view * model;


    use descriptor_sets::uniform_buffers::UniformBufferGlobalData;



    let global_buff= &uniform_buffers.global;
    let per_frame_buff = &uniform_buffers.per_frame;


    //uniform_buffers.bind_descriptor_sets(
    //    context,
    //    frame_data.command_buffer,
    //    &material,
    //    frame_data.frame_index);


    //let new_frame_data = UniformBufferFrameData {
    //    ambient_color: Vec4::new(f32::sin(frame_data.frame_index as f32),
    //                             0., f32::cos(frame_data.frame_index as f32),
    //                             1.),
    //};
    //per_frame_buff.bind_descriptor_set(context, frame_data.command_buffer, &material, 1, 0);
    //per_frame_buff.write_memory(context, new_frame_data, 0);


    let new_global_data = UniformBufferGlobalData {
        data: Vec4::default(),
        render_matrix: mesh_matrix,
    };
    global_buff.bind_descriptor_set(context, frame_data.command_buffer, &material, 0, 0);
    global_buff.write_memory(context, new_global_data, 0);

    // bind pipeline
    material.bind(&context, frame_data.command_buffer);

    unsafe {
        // bind vertex buffers
        context.device.handle.cmd_bind_vertex_buffers(frame_data.command_buffer, 0, &[mesh.vertex_buffer.handle], &[0]);

        // draw mesh
        context.device.handle.cmd_draw(frame_data.command_buffer, mesh.vertex_count as u32, 1, 0, 0);
    }

}







#[system(for_each)]
fn render(
    #[state] frame_index: &mut usize,
    #[state] frame_counter: &mut usize,
    #[resource] window: &Window,
    #[resource] time: &PTime,
    #[resource] materials: &MaterialsResource,
    #[resource] meshes: &MeshesResource,
    context: &VkContext,
    swapchain: &Swapchain,
    frame_buffers: &FrameBuffers,
    frame_datas: &FrameDataContainer,
    render_pass: &RenderPass,
    uniform_buffers: &UniformBuffers,
) {
    *frame_counter += 1;


    *frame_index = (*frame_index + 1) % penguin_config::vk_config::MAX_FRAMES_COUNT;

    let frame_data = frame_datas.get(*frame_index);

    render_commands::submit_render_commands(
        SubmitRenderCommandsParams {
            context,
            swapchain,
            frame_buffers,
            pipeline_wait_stage_flags: &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
            frame_data,
        },
        |render_pass_per_frame_params| {
            render_pass_func(
                *frame_counter,
                window,
                context,
                swapchain,
                render_pass,
                render_pass_per_frame_params,
                uniform_buffers,
                materials,
                meshes,
            );
        }
    )
}

// once per frame
fn render_pass_func(
    frame_counter: usize,
    window: &Window,
    context: &VkContext,
    swapchain: &Swapchain,
    render_pass: &RenderPass,
    render_pass_per_frame_params: RenderPassParams,
    uniform_buffers: &UniformBuffers,
    materials: &MaterialsResource,
    meshes: &MeshesResource,
) {
    let render_commands::RenderPassParams {
        frame_buffer,
        frame_data
    } = render_pass_per_frame_params;


    let flash = f32::abs(f32::sin(frame_counter as f32 / 120_f32));
    //let color = [0.0_f32, 0.0_f32, flash, 1.0_f32];
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
        .render_pass(render_pass.handle)
        .framebuffer(frame_buffer)
        .render_area(vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain.extent,
        })
        .clear_values(&clear_values);

    unsafe {
        context.device.handle.cmd_begin_render_pass(
            frame_data.command_buffer,
            &render_pass_begin_info,
            vk::SubpassContents::INLINE,
        );

        draw(
            window,
            context,
            frame_data,
            uniform_buffers,
            materials,
            meshes,
            frame_counter
        );

        context.device.handle.cmd_end_render_pass(frame_data.command_buffer);
    }
}

use crate::math_vk_format::*;
use descriptor_sets::uniform_buffers::{UniformBufferFrameData, UniformBufferGlobalData, UniformBuffers};

