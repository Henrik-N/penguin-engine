use ash::vk;
use penguin_app::ecs::*;
use crate::renderer::frame_data::{
    FrameData,
    FrameDataContainer
};
use crate::renderer::vk_types::{BindDescriptorSetsInfo, DescriptorSetsResource, FrameBuffers, RenderPass, Swapchain, VkContext};
use crate::renderer::resources::*;


#[system(for_each)]
pub fn render(
    context: &VkContext,
    frame_datas: &mut FrameDataContainer,
    swapchain: &Swapchain,
    frame_buffers: &FrameBuffers,
    render_pass: &RenderPass,

    // things that draw need
    #[resource] window: &penguin_app::window::Window, // for aspect ratio
    #[resource] materials: &MaterialsResource,
    #[resource] meshes: &MeshesResource,
    #[resource] render_objects: &RenderObjectsResource,
    #[resource] descriptor_sets: &DescriptorSetsResource,
) {
    frame_datas.increment_frame();
    let frame_data: &FrameData = frame_datas.get_current();

    let (swapchain_image_index, frame_buffer) = GetNextFrameBuffer {
        swapchain,
        frame_buffers,
        signal_semaphore: frame_data.presenting_complete_semaphore,
        signal_fence: vk::Fence::null(),
    }.exec();

    RecordCommandBuffer {
        command_buffer: frame_data.command_buffer,
        // Wait for previous command buffer for this frame data.
        //  In other words, wait until the GPU finished rendering the previous frame
        //  associated with this frame data.
        wait_for_fence: frame_data.render_complete_fence,
        reset_before_begin_flags: vk::CommandBufferResetFlags::empty(),
        usage_flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
    }.exec(context,
           ||
               {
                   RecordRenderPass {
                       command_buffer: frame_data.command_buffer,
                       render_pass,
                       frame_buffer,
                       swapchain_extent: swapchain.extent,
                   }.exec(context, ||
                       {
                           RecordDrawCommands {
                               params: DrawParams {
                                   aspect_ratio: aspect_ratio(window.dimensions.width, window.dimensions.height),
                                   fov_y: 70.0_f32.to_radians(),
                                   frame_data,
                                   frame_count: frame_datas.frame_count(),
                               },
                               resources: DrawResourceAccess {
                                   materials,
                                   meshes,
                                   descriptor_sets,
                                   render_objects,
                               }
                           }.exec(context);
                       });
               }
    );

    SubmitCommandBufferToGraphicsQueue {
        command_buffer: frame_data.command_buffer,
        pipeline_wait_stages: &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
        wait_for_fence: frame_data.render_complete_fence,
        wait_for_semaphore: frame_data.presenting_complete_semaphore,
        // after this command buffer is complete, signal the rendering complete semaphore
        signal_semaphore: frame_data.rendering_complete_semaphore,
    }.exec(context);


    PresentImageToScreen {
        swapchain,
        wait_for_semaphore: frame_data.rendering_complete_semaphore,
        swapchain_image_index,
    }.exec(context);
}






struct GetNextFrameBuffer<'a> {
    swapchain: &'a Swapchain,
    frame_buffers: &'a FrameBuffers,
    signal_semaphore: vk::Semaphore,
    signal_fence: vk::Fence,
}
impl<'a> GetNextFrameBuffer<'a> {
    fn exec(self) -> (u32, vk::Framebuffer) {
        // find next image
        let swapchain_image_index = self.swapchain.acquire_next_swapchain_image(
            self.signal_semaphore,
            self.signal_fence,
            std::time::Duration::from_secs(1),
        );

        (swapchain_image_index, self.frame_buffers.get(swapchain_image_index as _))
    }
}

struct RecordCommandBuffer {
    command_buffer: vk::CommandBuffer,
    wait_for_fence: vk::Fence,
    reset_before_begin_flags: vk::CommandBufferResetFlags,
    usage_flags: vk::CommandBufferUsageFlags,
}

impl RecordCommandBuffer {
    fn exec<F: FnOnce()>(
        self,
        context: &VkContext,
        commands: F,
    ) {
        // begin
        context.wait_for_fence(self.wait_for_fence, std::time::Duration::MAX);
        context.reset_command_buffer(self.command_buffer, self.reset_before_begin_flags);
        context.begin_command_buffer(self.command_buffer, self.usage_flags);

        commands();

        // end
        context.end_command_buffer(self.command_buffer);
    }
}


struct DrawParams<'a> {
    aspect_ratio: f32,
    fov_y: f32,
    frame_data: &'a FrameData,
    frame_count: usize,
}

struct DrawResourceAccess<'a> {
    materials: &'a MaterialsResource,
    meshes: &'a MeshesResource,
    descriptor_sets: &'a DescriptorSetsResource,
    render_objects: &'a RenderObjectsResource,
}

struct RecordDrawCommands<'a> {
    params: DrawParams<'a>,
    resources: DrawResourceAccess<'a>,
}





use macaw::{Affine3A, Quat, Vec3};
use crate::math_vk_format::{Mat4, Vec4};
use crate::renderer::gpu_data::{GPUCameraData, GPUObjectData, SomeGPUData};
use crate::renderer::memory::{AllocatedBuffer, AllocatedBufferCreateInfo, DeviceMemoryWriteInfo, MemoryUsage};
use crate::renderer::render_objects::Vertex;

impl<'a> RecordDrawCommands<'a> {

    // todo
    fn _exec2(self, context: &VkContext) {
        let world_transform = Affine3A::from_scale_rotation_translation(
            Vec3::ONE, // scale
            Quat::IDENTITY, // rotation
            Vec3::ZERO // translation
        );

        context.bind_descriptor_sets(BindDescriptorSetsInfo {
            command_buffer: self.params.frame_data.command_buffer,
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            pipeline_layout: self.resources.descriptor_sets.get_set(0).pipeline_layout,
            first_set: 0,
            descriptor_set_handles: &self.resources.descriptor_sets.get_set_handles(&[0, 1 /*2*/]),
        });

        let buffer_data = [SomeGPUData {
            data: world_transform,
        }];


        let alignment = context.packed_uniform_buffer_range::<SomeGPUData>();
        let offset = (self.params.frame_data.frame_index) as u64 * alignment;
        self.resources.descriptor_sets.get_set(0).allocated_buffers[0].write_memory(
            &context,
            DeviceMemoryWriteInfo {
                data: &buffer_data,
                size: alignment,
                offset,
                alignment,
            },
        );

        // bind pipeline
        self.resources.render_objects.render_objects[0]
            .material
            .bind(&context, self.params.frame_data.command_buffer);


        let max_commands_count = 10;


        let vertices = [
            Vertex {
                position: Default::default(),
                normal: Default::default(),
                color: Default::default(),
                uv: Default::default()
            }
        ];


        //let cmd = vk::DrawIndirectCommand::builder()
        //    .vertex_count()



        let buffer = AllocatedBuffer::create_buffer(
            context,
            AllocatedBufferCreateInfo::<vk::DrawIndexedIndirectCommand> {
                initial_data: &[],
                buffer_size: (max_commands_count * std::mem::size_of::<vk::DrawIndexedIndirectCommand>()) as _,
                buffer_usage: vk::BufferUsageFlags::INDIRECT_BUFFER |
                    vk::BufferUsageFlags::TRANSFER_DST | // writable
                    vk::BufferUsageFlags::STORAGE_BUFFER, // just because it may be nice to have
                memory_usage: MemoryUsage::GpuMemCpuWritable,
                sharing_mode: Default::default(),
                memory_map_flags: Default::default()
            });


        unsafe {
            let cmd_buffer = self.params.frame_data.command_buffer;

            // draw indirect
        }

    }

    fn exec(self, context: &VkContext) {
        //let material = resources.materials.get("default");
        let mesh = self.resources.meshes.get("monkey");

        // create mvp matrix
        let camera_loc = Vec3::new(0.0, 10., -2.0);
        let camera_forward = Vec3::new(0.0, 0.0, 1.0);
        let camera_up = Vec3::new(0.0, 1.0, 0.0);

        let view = Mat4::look_at_rh(camera_loc, camera_loc + camera_forward, camera_up);

        let (z_near, z_far) = (0.1_f32, 200.0_f32);
        let projection = Mat4::perspective_rh(self.params.fov_y, self.params.aspect_ratio, z_near, z_far);

        context.bind_descriptor_sets(BindDescriptorSetsInfo {
            command_buffer: self.params.frame_data.command_buffer,
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            pipeline_layout: self.resources.descriptor_sets.get_set(0).pipeline_layout,
            first_set: 0,
            descriptor_set_handles: &self.resources.descriptor_sets.get_set_handles(&[0, 1 /*2*/]),
        });

        let buffer_data = [GPUCameraData {
            data: Vec4::default(),
            proj_view: projection * view,
        }];

        let alignment = context.packed_uniform_buffer_range::<GPUCameraData>();
        let offset = (self.params.frame_data.frame_index) as u64 * alignment;
        self.resources.descriptor_sets.get_set(0).allocated_buffers[0].write_memory(
            &context,
            DeviceMemoryWriteInfo {
                data: &buffer_data,
                size: alignment,
                offset,
                alignment,
            },
        );

        let spin = (self.params.frame_count as f32 * 0.4).to_radians();
        //let spin = 0.0_f32.to_radians();

        let buffer_data = [GPUObjectData {
            transform: Mat4::from_rotation_x(spin),
        }];

        let alignment = std::mem::align_of::<GPUObjectData>() as _;
        self.resources.descriptor_sets.get_set(1).allocated_buffers[0].write_memory(
            &context,
            DeviceMemoryWriteInfo {
                data: &buffer_data,
                size: alignment,
                offset: 0,
                alignment,
            },
        );

        // bind pipeline
        self.resources.render_objects.render_objects[0]
            .material
            .bind(&context, self.params.frame_data.command_buffer);

        unsafe {
            // bind vertex buffers
            context.device.cmd_bind_vertex_buffers(
                self.params.frame_data.command_buffer,
                0,
                &[self.resources.render_objects[0].mesh.vertex_buffer.handle],
                &[0],
            );

            let instance_id = 0;

            // draw mesh
            context.device.cmd_draw(
                self.params.frame_data.command_buffer,
                mesh.vertex_count as u32,
                1,
                0,
                instance_id,
            );
        }
    }
}





struct SubmitCommandBufferToGraphicsQueue<'a> {
    command_buffer: vk::CommandBuffer,
    pipeline_wait_stages: &'a [vk::PipelineStageFlags],
    wait_for_fence: vk::Fence,
    wait_for_semaphore: vk::Semaphore,
    signal_semaphore: vk::Semaphore,
}
impl<'a> SubmitCommandBufferToGraphicsQueue<'a> {
    fn exec(self, context: &VkContext) {
        let wait_semaphores = [self.wait_for_semaphore];
        let signal_semaphores = [self.signal_semaphore];
        let command_buffers = [self.command_buffer];

        let submit_info = [vk::SubmitInfo::builder()
            .wait_dst_stage_mask(self.pipeline_wait_stages)
            .wait_semaphores(&wait_semaphores)
            .signal_semaphores(&signal_semaphores)
            .command_buffers(&command_buffers)
            .build()];

        unsafe {
            context.device.queue_submit(
                context.device.graphics_queue_handle,
                &submit_info,
                self.wait_for_fence)
                .expect("Couldn't submit command buffer to graphics queue");
        }
    }
}

struct PresentImageToScreen<'a> {
    swapchain: &'a Swapchain,
    wait_for_semaphore: vk::Semaphore,
    swapchain_image_index: u32,
}
impl<'a> PresentImageToScreen<'a> {
    fn exec(self, context: &VkContext) {
        let swapchains = [self.swapchain.handle];
        let wait_semaphores = [self.wait_for_semaphore];
        let image_indices = [self.swapchain_image_index];

        let present_info = vk::PresentInfoKHR::builder()
            .swapchains(&swapchains)
            .wait_semaphores(&wait_semaphores)
            .image_indices(&image_indices);

        unsafe {
            self.swapchain.loader.queue_present(
                context.device.graphics_queue_handle,
                &present_info)
                .expect("Couldn't submit to present queue");
        }
    }
}




fn aspect_ratio(width: u32, height: u32) -> f32 {
    width as f32 / height as f32
}


struct RecordRenderPass<'a> {
    command_buffer: vk::CommandBuffer,
    render_pass: &'a RenderPass,
    frame_buffer: vk::Framebuffer,
    swapchain_extent: vk::Extent2D,
}
impl<'a> RecordRenderPass<'a> {
    fn exec<F: FnOnce()>(self, context: &VkContext, draw: F) {
        self.begin(context);

        draw();

        self.end(context);
    }

    const fn clear_values() -> [vk::ClearValue; 2] {
        let color = [0.0_f32, 0., 0., 1.];
        let color_clear = vk::ClearColorValue { float32: color };

        let depth_clear = vk::ClearDepthStencilValue {
            depth: 1.0_f32,
            stencil: 0
        };

        [
            vk::ClearValue { color: color_clear },
            vk::ClearValue {
                depth_stencil: depth_clear,
            },
        ]
    }

    fn begin(&self, context: &VkContext) {
        const CLEAR_VALUES: [vk::ClearValue; 2] = RecordRenderPass::clear_values();

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass.handle)
            .framebuffer(self.frame_buffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_extent,
            })
            .clear_values(&CLEAR_VALUES);

        unsafe {
            context.device.cmd_begin_render_pass(
                self.command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
        }
    }

    fn end(&self, context: &VkContext) {
        unsafe {
            context.device.cmd_end_render_pass(self.command_buffer);
        }
    }
}
