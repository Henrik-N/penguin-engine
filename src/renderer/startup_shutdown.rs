use ash::vk;
use penguin_app::ecs::*;
use penguin_app::window::Window;
use crate::math_vk_format::{Vec3};
use crate::renderer::frame_data::{FrameData, FrameDataContainer};
use crate::renderer::resources::{MaterialsResource, MeshesResource, RenderObjectsResource};
use crate::renderer::memory::{AllocatedBuffer, AllocatedBufferCreateInfo, MemoryUsage, UploadContext};
use crate::renderer::gpu_data::{GPUCameraData, GPUObjectDataNew, GPUObjectDataOld};
use crate::renderer::render_objects::{RenderObject, Vertex};
use crate::renderer::vk_types::descriptor::{DescriptorSetContainer, DescriptorSetLayoutContainer};
use crate::renderer::vk_types::*;
use crate::renderer::vk_types::resources::DescriptorSetsResource;


#[system]
pub fn renderer_startup(
    cmd: &mut legion::systems::CommandBuffer,
    #[resource] window: &Window,
    #[resource] meshes: &mut MeshesResource,
    #[resource] materials: &mut MaterialsResource,
    #[resource] render_objects: &mut RenderObjectsResource,
    #[resource] descriptor_sets_resource: &mut DescriptorSetsResource,
) {
    log::trace!("RENDERER STARTUP STARTED!");
    // /------------------ CONTEXT  -----------------------------------------------------
    let context = VkContext::init(window, window.logger_level);
    // ///////////////////////////////////////

    let upload_context = UploadContext::init(&context);

    // /------------------ OTHER RENDERER STRUCTS----------------------------------------


    let VkComponents {
        swapchain,
        depth_image,
        render_pass,
        frame_buffers,
        //descriptor_pool,
    } = init_vk_components(window, &context);
    // ///////////////////////////////////////


    descriptor_sets_resource.init_pool(DescriptorPool::from_sizes(&context, 20, &[
        vk::DescriptorPoolSize { descriptor_count: 10, ty: vk::DescriptorType::UNIFORM_BUFFER },
        //vk::DescriptorPoolSize { descriptor_count: 10, ty: vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC },
        vk::DescriptorPoolSize { descriptor_count: 10, ty: vk::DescriptorType::STORAGE_BUFFER },
    ]));
    let descriptor_pool = descriptor_sets_resource.pool;



    /////////////////////


    // * descriptor set layouts
    //
    let uniform_buffer_descriptor_set_layout = DescriptorSetLayout::builder(&context)
        .layout_binding(vk::DescriptorSetLayoutBinding::builder()
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .binding(0)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
        )
        .layout_binding(vk::DescriptorSetLayoutBinding::builder()
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .binding(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
        )
        .build();

    let storage_buffer_descriptor_set_layout = DescriptorSetLayout::builder(&context)
            .layout_binding(vk::DescriptorSetLayoutBinding::builder()
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .binding(0)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
            )
            .build();

    // * descriptor sets
    //
    let uniform_buffer_set_layouts = [
        uniform_buffer_descriptor_set_layout.handle,
    ];
    let uniform_buffer_descriptor_set = unsafe {
        context.device.allocate_descriptor_sets(&vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool.handle)
            .set_layouts(&uniform_buffer_set_layouts)
        )
    }.expect("couldn't allocate uniform descriptor set")[0];


    let storage_buffer_descriptor_set_layouts = [storage_buffer_descriptor_set_layout.handle];
    let storage_buffer_descriptor_set = unsafe {
        context.device.allocate_descriptor_sets(&vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool.handle)
            .set_layouts(&storage_buffer_descriptor_set_layouts)
        )
    }.expect("couldn't allocate storage descriptor set")[0];


    // * pipeline layout(s)
    //
    let set_layouts = [
        uniform_buffer_descriptor_set_layout.handle,
        storage_buffer_descriptor_set_layout.handle,
    ];

    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(&set_layouts)
        .build();

    let pipeline_layout = unsafe {
        context.device.create_pipeline_layout(&pipeline_layout_create_info, None)
    }.expect("couldn't create pipeline layout");

    // * graphics pipeline
    //
    let vertex_input_bindings = Vertex::create_binding_descriptions(0);
    let vertex_input_attribute_descriptions = Vertex::create_attribute_descriptions(0);
    let pipeline = Pipeline::builder(
        &context,
        swapchain.extent,
        render_pass.handle,
        vk::PipelineBindPoint::GRAPHICS,
    )
        .shaders(&["simple.vert", "simple.frag"])
        .vertex_input(&vertex_input_bindings, &vertex_input_attribute_descriptions)
        //.wireframe_mode()
        .pipeline_layout(pipeline_layout)
        //.add_push_constants::<MeshPushConstants>()
        .build();

    // allocate buffers for uniform descriptor set bindings
    let (b0_buffer, b1_buffer) = {
        let b0_packed_range = context.packed_uniform_buffer_range::<GPUCameraData>();
        let b0_size = b0_packed_range * (penguin_config::vk_config::MAX_FRAMES_COUNT) as u64;
        let b0_buffer = AllocatedBuffer::create_buffer(
            &context,
            AllocatedBufferCreateInfo {
                initial_data: &(0..b0_size).map(|_| 0_u8).collect::<Vec<u8>>(),
                buffer_size: b0_size,
                buffer_usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
                memory_usage: MemoryUsage::GpuMemCpuWritable,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                memory_map_flags: vk::MemoryMapFlags::empty(),
            }
        );

        // scene data
        let b1_packed_range = context.packed_uniform_buffer_range::<GPUObjectDataOld>();
        let b1_size = b1_packed_range * (penguin_config::vk_config::MAX_FRAMES_COUNT) as u64;
        let b1_buffer = AllocatedBuffer::create_buffer(
            &context,
            AllocatedBufferCreateInfo {
                initial_data: &(0..b1_size).map(|_| 0_u8).collect::<Vec<u8>>(),
                buffer_size: b1_size,
                buffer_usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
                memory_usage: MemoryUsage::GpuMemCpuWritable,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                memory_map_flags: vk::MemoryMapFlags::empty(),
            }
        );

        (b0_buffer, b1_buffer)
    };

    let s3_buffer = {
        let s3_size = std::mem::size_of::<GPUObjectDataNew>() * penguin_config::vk_config::MAX_OBJECTS;
        let s3_buffer = AllocatedBuffer::create_buffer(
            &context,
            AllocatedBufferCreateInfo {
                initial_data: &(0..s3_size).map(|_| 0_u8).collect::<Vec<u8>>(),
                buffer_size: s3_size as _,
                buffer_usage: vk::BufferUsageFlags::STORAGE_BUFFER,
                memory_usage: MemoryUsage::GpuMemCpuWritable,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                memory_map_flags: vk::MemoryMapFlags::empty(),
            }
        );
        s3_buffer
    };

    let uniform_buffer_descriptor_set_container = DescriptorSetContainer {
        handle: uniform_buffer_descriptor_set,
        layout: DescriptorSetLayoutContainer {
            descriptor_set: uniform_buffer_descriptor_set_layout,
            pipeline: pipeline_layout,
        },
        allocated_buffers: vec![b0_buffer, b1_buffer],
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
    };

    let storage_buffer_descriptor_set_container = DescriptorSetContainer {
        handle: storage_buffer_descriptor_set,
        layout: DescriptorSetLayoutContainer {
            descriptor_set: storage_buffer_descriptor_set_layout,
            pipeline: pipeline_layout,
        },
        allocated_buffers: vec![s3_buffer],
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
    };


    descriptor_sets_resource.sets = vec![
        // 0
        uniform_buffer_descriptor_set_container,
        // 1
        storage_buffer_descriptor_set_container
    ];


    let command_pool = context.alloc_command_pool(
        context.physical_device.graphics_queue_index,
        vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

    let command_buffers = context.allocate_command_buffers(
        command_pool,
        penguin_config::vk_config::MAX_FRAMES_COUNT as _);




    //let upload_fence = context.create_fence(vk::FenceCreateFlags::empty());

    let frame_datas = FrameDataContainer::new(
        command_pool,
        command_buffers.into_iter().enumerate()
            .map(|(frame_index, command_buffer)| {
                let frame_index = frame_index as u64;

                // fence --------------
                let render_fence = context.create_fence(vk::FenceCreateFlags::SIGNALED);


                // semaphores --------------
                let rendering_complete_semaphore = context.create_semaphore(vk::SemaphoreCreateFlags::empty());
                let presenting_complete_semaphore = context.create_semaphore(vk::SemaphoreCreateFlags::empty());

                let uniform_buffer_infos = [
                    // set 0, binding 0
                    vk::DescriptorBufferInfo::builder()
                        .buffer(descriptor_sets_resource.get_set(0).allocated_buffers[0].handle)
                        .range(std::mem::size_of::<GPUCameraData>() as _)
                        .offset(
                            (context.packed_uniform_buffer_range::<GPUCameraData>() * frame_index) as _
                        )
                        .build(),
                    // set 0, binding 1
                    vk::DescriptorBufferInfo::builder()
                        .buffer(descriptor_sets_resource.get_set(0).allocated_buffers[1].handle)
                        .range(std::mem::size_of::<GPUObjectDataOld>() as _)
                        .offset(
                            (context.packed_uniform_buffer_range::<GPUObjectDataOld>() * frame_index) as _
                        )
                        .build(),
                ];

                let storage_buffer_infos = [
                    // set 1, binding 0
                    vk::DescriptorBufferInfo::builder()
                        .buffer(descriptor_sets_resource.get_set(1).allocated_buffers[0].handle)
                        .range(
                            (std::mem::size_of::<GPUObjectDataNew>() * penguin_config::vk_config::MAX_OBJECTS) as _
                        )
                        .offset(0)
                        .build(),
                ];

                let uniform_buffer_write_sets = [
                    // set 0, binding 0
                    vk::WriteDescriptorSet::builder()
                        .dst_set(descriptor_sets_resource.get_set(0).handle)
                        .dst_binding(0)
                        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                        .buffer_info(&uniform_buffer_infos[0..1])
                        .build(),
                    // set 0, binding 1
                    vk::WriteDescriptorSet::builder()
                        .dst_set(descriptor_sets_resource.get_set(0).handle)
                        .dst_binding(1)
                        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                        .buffer_info(&uniform_buffer_infos[1..2])
                        .build(),
                ];

                let storage_buffer_write_sets = [
                    vk::WriteDescriptorSet::builder()
                        .dst_set(descriptor_sets_resource.get_set(1).handle)
                        .dst_binding(0)
                        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                        .buffer_info(&storage_buffer_infos[0..1])
                        .build()
                ];

                let write_sets = uniform_buffer_write_sets.into_iter()
                    .chain(storage_buffer_write_sets.into_iter())
                    .collect::<Vec<vk::WriteDescriptorSet>>();

                unsafe {
                    context.device.update_descriptor_sets(&write_sets, &[]);
                }

                FrameData {
                    command_buffer,
                    render_complete_fence: render_fence,
                    rendering_complete_semaphore,
                    presenting_complete_semaphore,
                    uniform_buffer_descriptor_set,
                    frame_index,
                }
            }).collect::<Vec<FrameData>>(),
    );

    // /------------------ MESHES  -----------------------------------------------------
    meshes.insert_from_file(&context, &upload_context, ("monkey", "bunny.obj"));
    materials.insert(("default", pipeline));


    let render_object = RenderObject {
        material: materials.get("default").clone(),
        mesh: meshes.get("monkey").clone(),
        translation: Vec3::new(0., 0., 0.),
        name: "bunny".to_owned()
    };
    render_objects.render_objects.push(render_object);


    let _renderer_entity: Entity = cmd.push((
        context,
        swapchain,
        depth_image,
        render_pass,
        frame_buffers,
        descriptor_pool,
        //
        frame_datas,
        //
        upload_context,
    ));
}


#[system]
pub fn renderer_shutdown(
    world: &mut SubWorld,
    query: &mut Query<(
        &mut VkContext,
        &mut Swapchain,
        &mut FrameBuffers,
        &mut RenderPass,
        &mut DepthImage,
        //&mut DescriptorPool,
        //
        &mut FrameDataContainer,
        &mut UploadContext,
    )>,
    #[resource] meshes: &mut MeshesResource,
    #[resource] materials: &mut MaterialsResource,
    #[resource] descriptor_sets: &mut DescriptorSetsResource,
) {
    log::info!("RENDERER SHUTDOWN STARTED!");

    query.iter_mut(world).for_each(
        |(context, swapchain, frame_buffers, render_pass, depth_image,
             //descriptor_pool,
             frame_datas,
            upload_context
         ): (
            &mut VkContext,
            &mut Swapchain,
            &mut FrameBuffers,
            &mut RenderPass,
            &mut DepthImage,
            //&mut DescriptorPool,
            &mut FrameDataContainer,
            //
            &mut UploadContext
        )| {
            // wait for device idle..
            context.wait_for_device_idle();

            frame_datas.destroy(context);

            frame_buffers.destroy(context);

            render_pass.destroy(context);


            swapchain.destroy(context);

            depth_image.destroy(context);

            // ------------- RESOURCES ----------
            meshes.destroy(context);
            materials.destroy(context);
            descriptor_sets.destroy(context);

            // ------------- END OF RESOURCES ----------

            upload_context.destroy(context);


            context.destroy();

            log::info!("Renderer finished!");
        },
    );
}


