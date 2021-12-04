pub mod vk_components;
pub mod vk_context;
mod render_commands;

use std::collections::HashMap;
use std::mem::swap;

use crate::ecs::*;
use ash_window::create_surface;
use legion::world::SubWorld;
use std::ops::Deref;
use ash::vk;
use crate::core::time::PTime;
use crate::engine::pe;
use crate::engine::pe::pipeline::{PPipeline, PPipelineBuilder};
use crate::engine::renderer::vk_types::VkContext;


pub mod vk_types {
    pub use crate::engine::renderer::vk_context::*;
    pub use crate::engine::renderer::vk_components::*;
}

// ----------------- RESOURCES -----------------

struct Window {
    window: winit::window::Window,
}
impl Deref for Window {
    type Target = winit::window::Window;
    fn deref(&self) -> &Self::Target { &self.window }
}


use crate::engine::resources::Mesh;
#[derive(Default)]
struct MeshesResource {
    meshes: HashMap<String, Mesh>,
}
impl MeshesResource {
    pub fn insert_from_file(&mut self, context: &VkContext, (name, file_name): (&str, &str)) {
        self.meshes.insert(name.to_owned(), Mesh::from_obj(context, file_name));
    }

    pub fn destroy(&mut self, context: &VkContext) {
        self.meshes.iter_mut().for_each(|(_name, mesh)| mesh.destroy(&context));
    }
}

#[derive(Default)]
struct MaterialsResource {
    materials: HashMap<String, Material>,
}
impl MaterialsResource {
    pub fn destroy(&mut self, context: &VkContext) {
        self.materials.iter_mut().for_each(|(_name, material)| material.destroy(context));
    }

    pub fn insert(&mut self, (name, pipeline): (&str, PPipeline)) {
        self.materials.insert(name.to_owned(), Material::from_pipeline(pipeline));
    }
}


// ----------------- END OF RESOURCES -----------------


pub struct RendererPlugin {
    pub window: Option<winit::window::Window>,
}
impl Plugin for RendererPlugin {
    fn startup(&mut self, resources: &mut Resources) -> Vec<Step> {
        // move ownership of window to the resource
        let window = std::mem::replace(&mut self.window, None).expect("couldn't move winit window");

        resources.insert(Window { window });
        resources.insert(MeshesResource::default());
        resources.insert(MaterialsResource::default());

        Schedule::builder()
            .add_thread_local(renderer_startup_system())
            .build()
            .into_vec()
    }

    fn run() -> Vec<Step> {
        Schedule::builder()
            .add_thread_local(draw_system(0))
            .build().into_vec()
    }

    fn shutdown() -> Vec<Step> {
        Schedule::builder()
            .add_thread_local(renderer_shutdown_system())
            .build()
            .into_vec()
    }
}

use crate::engine::renderer::{vk_components::*, vk_context::*};
use crate::engine::renderer::render_commands::{RenderPassParams, SubmitRenderCommandsParams};
use crate::engine::renderer::uniform_buffer::GpuUniformBuffer;
use crate::engine::resources::{HashResource, Material, Vertex};


#[system(for_each)]
fn draw(
    #[state] frame_index: &mut usize,
    #[resource] time: &PTime,
    context: &VkContext,
    swapchain: &Swapchain,
    frame_buffers: &FrameBuffers,
    frame_datas: &FrameDataContainer,
    render_pass: &RenderPass,
) {
    //log::info!("frame: {}, delta: {}", frame_index, time.delta());

    *frame_index = (*frame_index + 1) % crate::config::MAX_FRAMES_COUNT;

    let frame_data = frame_datas.get(*frame_index);

    render_commands::submit_render_commands(
        SubmitRenderCommandsParams {
            context,
            swapchain,
            frame_buffers,
            pipeline_wait_stage_flags: &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
            frame_data,
        },
        |render_pass_per_frame_params| { render_pass_func(
            context,
            swapchain,
            render_pass,
            render_pass_per_frame_params); }
    )
}

fn render_pass_func(
    context: &VkContext,
    swapchain: &Swapchain,
    render_pass: &RenderPass,
    render_pass_per_frame_params: RenderPassParams) {

    let render_commands::RenderPassParams {
        frame_buffer,
        frame_data
    } = render_pass_per_frame_params;


    //let flash = f32::abs(f32::sin(self.frame_num as f32 / 120_f32));
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

        //self.draw_render_objects(
        //    &context.device.handle,
        //    command_buffer,
        //    frame_data,
        //    self.frame_num,
        //);

        context.device.handle.cmd_end_render_pass(frame_data.command_buffer);
    }
}






#[system]
fn renderer_shutdown(
    world: &mut SubWorld,
    mut query: &mut Query<(
        &mut VkContext,
        &mut Swapchain,
        &mut FrameBuffers,
        &mut RenderPass,
        &mut DepthImage,
        &mut DescriptorPool,
        //
        &mut FrameDataContainer,
    )>,
    #[resource] meshes: &mut MeshesResource,
    #[resource] materials: &mut MaterialsResource,
) {
    log::info!("RENDERER SHUTDOWN STARTED!");

    query.iter_mut(world).for_each(
        |(context, swapchain, frame_buffers, render_pass, depth_image, descriptor_pool, frame_datas): (
            &mut VkContext,
            &mut Swapchain,
            &mut FrameBuffers,
            &mut RenderPass,
            &mut DepthImage,
            &mut DescriptorPool,
            &mut FrameDataContainer,
        )| {
            // wait for device idle..
            context.wait_for_device_idle();


            frame_datas.destroy(&context);


            frame_buffers.destroy(&context);

            render_pass.destroy(&context);

            descriptor_pool.destroy(&context);

            swapchain.destroy(&context);

            depth_image.destroy(&context);



            // ------------- RESOURCES ----------

            meshes.destroy(&context);
            materials.destroy(&context);

            // ------------- END OF RESOURCES ----------


            context.destroy();
        },
    );
}




pub struct FrameData {
    pub(super) command_pool: vk::CommandPool,
    pub(super) command_buffer: vk::CommandBuffer,

    pub(super) render_fence: vk::Fence,
    pub(super) rendering_complete_semaphore: vk::Semaphore,
    pub(super) presenting_complete_semaphore: vk::Semaphore,

    //pub(super) uniform_buffer: Rc<UniformBuffer>,

    pub(super) frame_index: usize,
}
impl FrameData {
    pub fn new(
        context: &VkContext,
        //descriptor_pool: vk::DescriptorPool,
        frame_index: usize,
        //uniform_buffer: Rc<UniformBuffer>,
    ) -> Self {
        // command pool and command buffer ---------
        let (command_pool, command_buffer) =
            pe::command_buffers::init::create_command_pool_and_buffer(&context.device.handle, context.physical_device.queue_index);

        // fences ---------
        let render_fence_create_info =
            vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED); // start signaled, to wait for it before the first gpu command

        let render_fence = unsafe { context.device.handle.create_fence(&render_fence_create_info, None) }
            .expect("Failed to create render fence.");

        // semaphores --------------
        let semaphore_create_info = vk::SemaphoreCreateInfo::default();

        let rendering_complete_semaphore =
            unsafe { context.device.handle.create_semaphore(&semaphore_create_info, None) }
                .expect("Failed to create semaphore");
        let presenting_complete_semaphore =
            unsafe { context.device.handle.create_semaphore(&semaphore_create_info, None) }
                .expect("Failed to create semaphore");

        Self {
            command_pool,
            command_buffer,
            render_fence,
            rendering_complete_semaphore,
            presenting_complete_semaphore,
            //uniform_buffer,
            frame_index,
        }
    }
    pub fn destroy(&mut self, context: &VkContext) {
        unsafe {
            context.device.handle.destroy_semaphore(self.presenting_complete_semaphore, None);
            context.device.handle.destroy_semaphore(self.rendering_complete_semaphore, None);
            context.device.handle.destroy_fence(self.render_fence, None);
            context.device.handle.destroy_command_pool(self.command_pool, None);
        }
    }
}


pub struct FrameDataContainer {
    pub frame_datas: Vec<FrameData>,
}
impl FrameDataContainer {
    pub fn destroy(&mut self, context: &VkContext) {
        self.frame_datas.iter_mut().for_each(|frame_data| frame_data.destroy(context));
    }

    pub fn get(&self, index: usize) -> &FrameData {
        self.frame_datas.get(index).expect("yeet")
    }

}




#[system]
fn renderer_startup(cmd: &mut legion::systems::CommandBuffer,
                    #[resource] window: &Window,
                    #[resource] meshes: &mut MeshesResource,
                    #[resource] materials: &mut MaterialsResource) {
    log::info!("RENDERER STARTUP STARTED!");
    // /------------------ CONTEXT  -----------------------------------------------------
    let context = VkContext::init(window);
    // ///////////////////////////////////////



    // /------------------ OTHER RENDERER STRUCTS----------------------------------------
    let VkComponents {
        swapchain,
        depth_image,
        render_pass,
        frame_buffers,
        descriptor_pool
    } = vk_components::init(&context);
    // ///////////////////////////////////////

    let mut gpu_uniform_buffer = GpuUniformBuffer::init(&context, &descriptor_pool);



    // todo: per-frame data ---------, Vec<FrameData>
    let frame_datas = FrameDataContainer {
        frame_datas: (0..crate::config::MAX_FRAMES_COUNT)
            .map(|frame_index| FrameData::new(&context, frame_index)).collect()
    };




    let vertex_input_bindings = Vertex::get_binding_descriptions();
    let vertex_input_attribute_descriptions = Vertex::get_attribute_descriptions();

    let mut pipeline = PPipelineBuilder::default(
        &context.device.handle,
        swapchain.extent,
        render_pass.handle,
        vk::PipelineBindPoint::GRAPHICS,
    )
        .shaders(&["simple.vert", "simple.frag"])
        .vertex_input(&vertex_input_bindings, &vertex_input_attribute_descriptions)
        //.wireframe_mode()
        .descriptor_set_layouts(gpu_uniform_buffer.get_descriptor_set_layouts())
        //.add_push_constants::<MeshPushConstants>()
        .build();


    gpu_uniform_buffer.destroy(&context);


    // /------------------ MESHES  -----------------------------------------------------
    meshes.insert_from_file(&context, ("monkey", "bunny.obj"));
    materials.insert(("default", pipeline));

    //pipeline.destroy(&context);


    let _renderer_entity: Entity = cmd.push((
        context,
        swapchain,
        depth_image,
        render_pass,
        frame_buffers,
        descriptor_pool,
        //
        frame_datas,
    ));


    // //////////////////////////---- OLD --------------------------------------------------------------------------------

    //let uniform_buffer = Rc::new(UniformBuffer::new(
    //    Rc::clone(&context.device),
    //    core.physical_device_properties,
    //    core.physical_device_memory_properties,
    //    global_descriptor_pool,
    //));

    //let frame_data: Vec<FrameData> = (0..max_frames_count)
    //    .map(|frame_index| {
    //        FrameData::new(
    //            Rc::clone(&device),
    //            core.queue_index,
    //            core.physical_device_properties,
    //            core.physical_device_memory_properties,
    //            global_descriptor_pool,
    //            //global_descriptor_set_layout,
    //            frame_index,
    //            Rc::clone(&uniform_buffer),
    //        )
    //    })
    //    .collect();

    ////////////////////////////---- OLD --------------------------------------------------------------------------------
}



//mod descriptor_sets {
//    use crate::engine::descriptor_sets::DescriptorSet;
//    use crate::engine::renderer::vk_types::VkContext;
//
//    pub fn create_descriptor_set_layout<T: DescriptorSet>(context: &VkContext) -> vk::DescriptorSetLayout {
//        T::create_descriptor_set_layout(&context.device.handle)
//    }
//
//}





mod uniform_buffer {
    use ash::vk;
    use crate::config;
    use crate::engine::buffers::{AllocatedBuffer, AllocatedBufferCreateInfo, MemoryUsage};
    use crate::engine::descriptor_sets::{DescriptorSet, DescriptorSetBindingFrequency};
    use crate::engine::renderer::vk_types::*;
    use crate::engine::math::macaw_types::*;


    /// Data to send to and from the gpu through the Uniform buffer
    /// Global uniform data, bound once per frame
    #[derive(Default, Clone, Copy)]
    #[repr(C)]
    #[repr(align(256))] // hardcoded align for now
    pub struct UniformBufferGlobalData {
        pub data: Vec4,
        pub render_matrix: Mat4,
    }
    /// Data to send to and from the gpu through the Uniform buffer
    #[derive(Default, Clone, Copy)]
    #[repr(C)]
    #[repr(align(256))]
    pub struct UniformBufferFrameData {
        pub fog_color: Vec4,
        pub fog_distances: Vec4,
        pub ambient_color: Vec4,
        pub sunlight_direction: Vec4,
        pub sunlight_color: Vec4,
    }









    /// Cpu-side buffer to allocate data for UniformBuffer
    #[derive(Default, Clone, Copy)]
    #[repr(C)] // ensure compiler doesn't reorder properties
    #[repr(align(256))]
    struct UniformBufferData {
        _global_data: UniformBufferGlobalData,
        _frame_data: [UniformBufferFrameData; config::MAX_FRAMES_COUNT],
    }


    #[derive(Debug)]
    pub struct DescriptorSetDetails {
        pub set: vk::DescriptorSet,
        pub layout: vk::DescriptorSetLayout,
    }



    #[derive(Debug)]
    /// Gpu-side uniform buffer with all buffer data
    pub struct GpuUniformBuffer {
        pub buffer: AllocatedBuffer,
        pub global_set: DescriptorSetDetails,
        pub frames_set: DescriptorSetDetails,
    }
    impl GpuUniformBuffer {
        pub fn get_descriptor_set_layouts(&self) -> Vec<vk::DescriptorSetLayout> {
            vec![self.global_set.layout, self.frames_set.layout]
        }
    }

    mod unused_so_far {
        use super::UniformBufferGlobalData;
        use super::GpuUniformBuffer;

        impl GpuUniformBuffer {
            pub fn write_global_memory(&self, new_data: UniformBufferGlobalData) {
                let buffer_data = [new_data];
                panic!()
                //self.global_buffer.write_memory(&buffer_data, 0);
            }
            // todo
            //fn frames_packed_offset<T>(&self, frame_index: u64) -> u64 {
            //    //let mem_range = std::mem::size_of::<T>() as u64;
            //    //let min_align = self.min_ubuffer_offset_align;

            //    let mem_range = super::packed_range_from_min_align::<UniformBufferGlobalData>(
            //        self.min_ubuffer_offset_align,
            //    );

            //    mem_range * frame_index
            //}

        }
    }


    impl GpuUniformBuffer {
        //pub fn bind_descriptor_sets(
        //    &self,
        //    device: &ash::Device,
        //    command_buffer: vk::CommandBuffer,
        //    pipeline_layout: vk::PipelineLayout,
        //    frame_index: usize,
        //) {
        //    let first_set = 0;
        //    let dynamic_offset = std::mem::size_of::<UniformBufferGlobalData>() as u32
        //        + std::mem::size_of::<UniformBufferFrameData>() as u32 * frame_index as u32;

        //    let descriptor_set = [self.global_set, self.frames_set];
        //    let dynamic_offsets = [dynamic_offset];

        //    unsafe {
        //        device.cmd_bind_descriptor_sets(
        //            command_buffer,
        //            vk::PipelineBindPoint::GRAPHICS,
        //            pipeline_layout,
        //            first_set,
        //            &descriptor_set.set,
        //            &dynamic_offsets,
        //        );
        //    }
        //}


        // ///////////////////////-------------------------------

    }



    fn packed_range_from_min_align<T>(min_align: u64) -> u64 {
        let mem_range = std::mem::size_of::<T>() as u64;
        let mut packed_range = 0;
        while packed_range < mem_range && packed_range < min_align {
            packed_range += min_align;
        }
        packed_range
    }





















    impl DescriptorSet for UniformBufferGlobalData {
        fn create_descriptor_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {

        let layout_bindings = [vk::DescriptorSetLayoutBinding::builder()
            .binding(Self::binding_index())
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            //.immutable_samplers()
            .build()];

        let create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&layout_bindings);

        unsafe { device.create_descriptor_set_layout(&create_info, None) }
            .expect("Couldn't create descriptor set layout")
        }

        fn binding_index() -> u32 {
            0_u32
        }

        fn descriptor_set_binding_flags() -> vk::DescriptorPoolCreateFlags {
            DescriptorSetBindingFrequency::Global.descriptor_flag_bits()
        }
    }

    impl DescriptorSet for UniformBufferFrameData {
        fn create_descriptor_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
            let binding_index = Self::binding_index();

            let layout_bindings = [vk::DescriptorSetLayoutBinding::builder()
                .binding(binding_index)
                .descriptor_count(1) // config::MAX_FRAMES_COUNT as u32)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
                //.immutable_samplers()
                .build()];

            let create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&layout_bindings);

            unsafe { device.create_descriptor_set_layout(&create_info, None) }
                .expect("Couldn't create descriptor set layout")
        }

        fn binding_index() -> u32 {
            1_u32
        }

        fn descriptor_set_binding_flags() -> vk::DescriptorPoolCreateFlags {
            DescriptorSetBindingFrequency::PerFrame.descriptor_flag_bits()
        }
    }


    mod init_destroy_uniform_buffer {
        use ash::vk;
        use crate::config;
        use crate::engine::buffers::{AllocatedBuffer, AllocatedBufferCreateInfo, MemoryUsage};
        use crate::engine::descriptor_sets::DescriptorSet;
        use crate::engine::renderer::uniform_buffer::{DescriptorSetDetails, UniformBufferFrameData, UniformBufferGlobalData};
        use crate::engine::renderer::vk_types::*;
        use super::GpuUniformBuffer;
        use super::packed_range_from_min_align;

        impl DescriptorSetDetails {
            pub fn destroy(&mut self, context: &VkContext) {
                unsafe {
                    context
                        .device.handle
                        .destroy_descriptor_set_layout(self.layout, None);
                }
            }
        }

        impl GpuUniformBuffer {
            pub fn destroy(&mut self, context: &VkContext) {
                self.buffer.destroy(context);
                self.global_set.destroy(context);
                self.frames_set.destroy(context);
            }

            pub fn init(context: &VkContext, descriptor_pool: &DescriptorPool) -> Self {

                // With the minimum offset alignment we can define the offsets of the
                // the different subtypes
                let min_ubuffer_offset_align = context.min_uniform_buffer_offset_alignment();



                let gb_packed_align =
                    packed_range_from_min_align::<UniformBufferGlobalData>(min_ubuffer_offset_align);
                let fb_packed_align =
                    packed_range_from_min_align::<UniformBufferFrameData>(min_ubuffer_offset_align);


                // The function create_buffer automatically aligns the memory upon creation
                // based on the given struct's size.
                let packed_size_bytes: u64 =
                    gb_packed_align + fb_packed_align * config::MAX_FRAMES_COUNT as u64;
                let initial_data: Vec<u8> = (0..packed_size_bytes).map(|_| 0_u8).collect();

                let gb_create_info = AllocatedBufferCreateInfo {
                    device: &context.device.handle,
                    pd_memory_properties: context.pd_mem_properties(),
                    initial_data: &initial_data, // size of allocated memory will be size of initial_data, the rest of the size is for dynamic mem
                    buffer_usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
                    memory_usage: MemoryUsage::CpuToGpu,
                    memory_map_flags: vk::MemoryMapFlags::empty(),
                    sharing_mode: vk::SharingMode::EXCLUSIVE,
                };

                let global_buffer = AllocatedBuffer::create_buffer(&gb_create_info);


                // descriptor layouts
                let global_layout = UniformBufferGlobalData::create_descriptor_set_layout(&context.device.handle);
                let frames_layout = UniformBufferFrameData::create_descriptor_set_layout(&context.device.handle);
                let descriptor_set_layouts = [global_layout, frames_layout];


                let descriptor_sets_allocate_info = vk::DescriptorSetAllocateInfo::builder()
                    .descriptor_pool(descriptor_pool.descriptor_pool)
                    .set_layouts(&descriptor_set_layouts);

                // descriptor sets
                log::debug!("Allocating descriptor sets.");
                let allocated_descriptor_sets =
                    unsafe { context.device.handle.allocate_descriptor_sets(&descriptor_sets_allocate_info) }
                        .expect("Couldn't allocate global descriptor set");


                // descriptor buffer infos
                let gb_packed_align =
                    packed_range_from_min_align::<UniformBufferGlobalData>(min_ubuffer_offset_align);

                let global_info = [
                    // global set
                    vk::DescriptorBufferInfo::builder()
                        .buffer(global_buffer.handle)
                        .offset(0)
                        .range(gb_packed_align)
                        //std::mem::size_of::<UniformBufferGlobalData>() as u64)
                        //std::mem::size_of::<
                        //Self::packed_global_range
                        //gb_packed_align)
                        //Self::packed_range_from_min_align::<UniformBufferGlobalData>(min_ubuffer_offset_align))
                        .build(),
                ];


                let fb_packed_align =
                    packed_range_from_min_align::<UniformBufferGlobalData>(min_ubuffer_offset_align);

                let per_frame_info = [
                    // per-frame set
                    vk::DescriptorBufferInfo::builder()
                        .buffer(global_buffer.handle)
                        .offset(0)
                        .range(fb_packed_align as u64)
                        //std::mem::size_of::<UniformBufferFrameData>() as u64)
                        .build(),
                ];


                // write sets
                let global_write_set = [
                    // global set, writing to binding 0
                    vk::WriteDescriptorSet::builder()
                        .dst_binding(UniformBufferGlobalData::binding_index())
                        .dst_set(allocated_descriptor_sets[0])
                        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                        .buffer_info(&global_info)
                        .build(),
                ];
                let frames_write_set = [vk::WriteDescriptorSet::builder()
                    .dst_binding(UniformBufferFrameData::binding_index())
                    .dst_set(allocated_descriptor_sets[1])
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                    .buffer_info(&per_frame_info)
                    .build()];

                let write_sets: Vec<_> = global_write_set
                    .into_iter()
                    .chain(frames_write_set)
                    .collect();

                unsafe {
                    context.device.handle.update_descriptor_sets(&write_sets, &[]);
                }


                Self {
                    buffer: global_buffer,
                    global_set: DescriptorSetDetails {
                        set: allocated_descriptor_sets[0],
                        layout: global_layout,
                    },
                    frames_set: DescriptorSetDetails {
                        set: allocated_descriptor_sets[1],
                        layout: frames_layout,
                    }
                }

            }
        }
    }



}



mod todo_init_destroy {
    //struct FrameDataContainer {
    //    frame_data: Vec<crate::engine::render_backend::FrameData>,
    //}
    //struct UniformBuffer {
    //    uniform_buffer: crate::engine::descriptor_sets::UniformBuffer,
    //    //uniform_buffer: Rc<UniformBuffer>,
    //}
}
