use crate::math_vk_format::Vec3;
use crate::renderer::frame_data::{FrameData, FrameDataContainer};
use crate::renderer::gpu_data::{GPUCameraData, GPUObjectData};
use crate::renderer::memory::{
    AllocatedBuffer, AllocatedBufferCreateInfo, MemoryUsage, UploadContext,
};
use crate::renderer::render_objects::{RenderObject, Texture, Vertex};
use crate::renderer::resources::{
    MaterialsResource, MeshesResource, RenderObjectsResource, TexturesResource,
};
use crate::renderer::vk_types::descriptor_sets::DescriptorSetContainer;
use crate::renderer::vk_types::resources::DescriptorSetsResource;
use crate::renderer::vk_types::*;
use ash::vk;
use ash::vk::DescriptorSetLayoutBinding;
use penguin_app::ecs::*;
use penguin_app::window::Window;
use stb::image::Channels::Default;

use uniform_buffer::*;
mod uniform_buffer {
    use super::*;

    pub fn uniform_buffer_desc_set(context: &VkContext, pool: &DescriptorPool) -> DescriptorSet {
        DescriptorSet::builder()
            .layout(
                DescriptorSetLayout::builder()
                    .layout_binding(
                        vk::DescriptorSetLayoutBinding::builder()
                            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                            .descriptor_count(1)
                            .binding(0)
                            .stage_flags(vk::ShaderStageFlags::VERTEX),
                    )
                    .build(context),
            )
            .build(context, pool)
            .expect("couldn't alloc uniform buffer set")
    }

    pub fn update_ubuffer_desc_info(
        context: &VkContext,
        desc_buffer_info: &mut vk::DescriptorBufferInfo,
        frame_index: usize,
    ) {
        desc_buffer_info.offset =
            (context.packed_uniform_buffer_range::<GPUCameraData>() * (frame_index as u64)) as _;
    }

    pub fn uniform_buffer_write_set(
        resource: &DescriptorSetsResource,
        desc_buffer_info: &[vk::DescriptorBufferInfo],
    ) -> vk::WriteDescriptorSet {
        vk::WriteDescriptorSet::builder()
            .dst_set(resource.get_set(0).handle())
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(&desc_buffer_info)
            .build()
    }

    pub fn uniform_buffer(context: &VkContext) -> (AllocatedBuffer, vk::DescriptorBufferInfo) {
        let packed_range = context.packed_uniform_buffer_range::<GPUCameraData>();
        let size = packed_range * (crate::config::MAX_FRAMES_COUNT) as u64;
        let buffer = AllocatedBuffer::create_buffer(
            &context,
            AllocatedBufferCreateInfo {
                initial_data: &(0..size).map(|_| 0_u8).collect::<Vec<u8>>(),
                buffer_size: size,
                buffer_usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
                memory_usage: MemoryUsage::GpuMemCpuWritable,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                memory_map_flags: vk::MemoryMapFlags::empty(),
            },
        );

        let desc_buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(buffer.handle)
            .range(std::mem::size_of::<GPUCameraData>() as _)
            // the offset depends on the frame, see when initiating the frame data
            .build();

        (buffer, desc_buffer_info)
    }
}

use storage_buffer::*;
mod storage_buffer {
    use super::*;

    pub fn storage_buffer_desc_set(context: &VkContext, pool: &DescriptorPool) -> DescriptorSet {
        DescriptorSet::builder()
            .layout(
                DescriptorSetLayout::builder()
                    .layout_binding(
                        vk::DescriptorSetLayoutBinding::builder()
                            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                            .descriptor_count(1)
                            .binding(0)
                            .stage_flags(vk::ShaderStageFlags::VERTEX),
                    )
                    .build(context),
            )
            .build(context, pool)
            .expect("couldn't allo storage set")
    }

    pub fn storage_buffer_write_set(
        resource: &DescriptorSetsResource,
        desc_buffer_info: &[vk::DescriptorBufferInfo],
    ) -> vk::WriteDescriptorSet {
        vk::WriteDescriptorSet::builder()
            .dst_set(resource.get_set(1).handle())
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(desc_buffer_info)
            .build()
    }

    pub fn storage_buffer(context: &VkContext) -> (AllocatedBuffer, vk::DescriptorBufferInfo) {
        let size = std::mem::size_of::<GPUObjectData>() * crate::config::MAX_OBJECTS;
        let buffer = AllocatedBuffer::create_buffer(
            &context,
            AllocatedBufferCreateInfo {
                initial_data: &(0..size).map(|_| 0_u8).collect::<Vec<u8>>(),
                buffer_size: size as u64,
                buffer_usage: vk::BufferUsageFlags::STORAGE_BUFFER,
                memory_usage: MemoryUsage::GpuMemCpuWritable,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                memory_map_flags: vk::MemoryMapFlags::empty(),
            },
        );

        let storage_buffer_desc_info = vk::DescriptorBufferInfo::builder()
            .buffer(buffer.handle)
            .range((std::mem::size_of::<GPUObjectData>() * crate::config::MAX_OBJECTS) as _)
            .offset(0)
            .build();

        (buffer, storage_buffer_desc_info)
    }
}

use image_sampler::*;
mod image_sampler {
    use super::*;

    pub fn single_texture_desc_set(context: &VkContext, pool: &DescriptorPool) -> DescriptorSet {
        DescriptorSet::builder()
            .layout(
                DescriptorSetLayout::builder()
                    .layout_binding(
                        vk::DescriptorSetLayoutBinding::builder()
                            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                            .descriptor_count(1)
                            .binding(0)
                            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
                    )
                    .build(context),
            )
            .build(context, pool)
            .expect("couldn't alloc single texture set")
    }

    pub fn blocky_sampler(context: &VkContext) -> vk::Sampler {
        let filter = vk::Filter::NEAREST;
        let address_mode = vk::SamplerAddressMode::REPEAT;
        let sampler_create_info = vk::SamplerCreateInfo::builder()
            .mag_filter(filter)
            .min_filter(filter)
            .address_mode_u(address_mode)
            .address_mode_v(address_mode)
            .address_mode_w(address_mode);

        unsafe { context.device.create_sampler(&sampler_create_info, None) }
            .expect("couldn't create blocky sampler")
    }
}

fn textured_pipeline(
    context: &VkContext,
    swapchain: &Swapchain,
    render_pass: &RenderPass,
    pipeline_layout: &PipelineLayout,
) -> Pipeline {
    Pipeline::builder(
        &context,
        swapchain.extent,
        render_pass.handle,
        vk::PipelineBindPoint::GRAPHICS,
    )
    .shaders(&["simple.vert", "textured.frag"])
    .vertex_input(
        &Vertex::create_binding_descriptions(0),
        &Vertex::create_attribute_descriptions(0),
    )
    .pipeline_layout(pipeline_layout.handle)
    .build()
}

#[system]
pub fn renderer_startup(
    cmd: &mut legion::systems::CommandBuffer,
    #[resource] window: &Window,
    #[resource] meshes: &mut MeshesResource,
    #[resource] materials: &mut MaterialsResource,
    #[resource] render_objects: &mut RenderObjectsResource,
    #[resource] descriptor_sets_resource: &mut DescriptorSetsResource,
    #[resource] textures: &mut TexturesResource,
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

    descriptor_sets_resource.init_pool(DescriptorPool::from_sizes(
        &context,
        20,
        vk::DescriptorPoolCreateFlags::empty(),
        &[
            vk::DescriptorPoolSize {
                descriptor_count: 10,
                ty: vk::DescriptorType::UNIFORM_BUFFER,
            },
            vk::DescriptorPoolSize {
                descriptor_count: 10,
                ty: vk::DescriptorType::STORAGE_BUFFER,
            },
            vk::DescriptorPoolSize {
                descriptor_count: 10,
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            },
        ],
    ));
    let descriptor_pool = descriptor_sets_resource.pool;

    /////////////////////

    // * descriptor set layouts
    //
    let uniform_buffer_desc_set = uniform_buffer_desc_set(&context, &descriptor_pool);
    let storage_buffer_desc_set = storage_buffer_desc_set(&context, &descriptor_pool);
    let image_sampler_desc_set = single_texture_desc_set(&context, &descriptor_pool);

    let pipeline_layout = PipelineLayout::builder()
        .add_layout(uniform_buffer_desc_set.layout.handle)
        .add_layout(storage_buffer_desc_set.layout.handle)
        .add_layout(image_sampler_desc_set.layout.handle)
        .build(&context);

    // textured pipeline
    let pipeline = textured_pipeline(&context, &swapchain, &render_pass, &pipeline_layout);

    ////////////////////////////////////////////
    let (uniform_buffer, mut uniform_desc_buffer_info) = uniform_buffer(&context);

    let uniform_buffer_descriptor_set_container = DescriptorSetContainer {
        set: uniform_buffer_desc_set.clone(),
        pipeline_layout: pipeline_layout.handle,
        allocated_buffers: vec![uniform_buffer],
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
    };

    ////////////////////////////////////////////
    let (storage_buffer, storage_buffer_desc_info) = storage_buffer(&context);

    let storage_buffer_descriptor_set_container = DescriptorSetContainer {
        set: storage_buffer_desc_set.clone(),
        pipeline_layout: pipeline_layout.handle,
        allocated_buffers: vec![storage_buffer],
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
    };

    ////////////////////////////////////////////

    // /------------------ RESOURCES  -----------------------------------------------------
    meshes.insert_from_file(&context, &upload_context, ("monkey", "lost_empire.obj"));
    materials.insert(("default", pipeline));
    textures.insert_from_file(&context, &upload_context, ("lost_emp", "dusk.jpeg"));

    let sampler = blocky_sampler(&context);
    let single_texture_desc_set = single_texture_desc_set(&context, &descriptor_pool);
    let texture = textures.get("lost_emp");

    let desc_image_info = vk::DescriptorImageInfo::builder()
        .sampler(sampler)
        .image_view(texture.image_view)
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .build();

    let single_texture_write_set = vk::WriteDescriptorSet::builder()
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .dst_set(single_texture_desc_set.handle)
        .image_info(&[desc_image_info])
        .dst_binding(0)
        .build();

    // todo make better
    let single_texture_desc_set_container = DescriptorSetContainer {
        set: single_texture_desc_set.clone(),
        pipeline_layout: pipeline_layout.handle,
        allocated_buffers: vec![],
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
    };

    descriptor_sets_resource.sets = vec![
        uniform_buffer_descriptor_set_container, // 0
        storage_buffer_descriptor_set_container, // 1
        single_texture_desc_set_container,       // 2
    ];

    let command_pool = context.alloc_command_pool(
        context.physical_device.graphics_queue_index,
        vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
    );

    let command_buffers =
        context.allocate_command_buffers(command_pool, crate::config::MAX_FRAMES_COUNT as _);

    let frame_datas = FrameDataContainer::new(
        command_pool,
        command_buffers
            .into_iter()
            .enumerate()
            .map(|(frame_index, command_buffer)| {
                // fence --------------
                let render_fence = context.create_fence(vk::FenceCreateFlags::SIGNALED);

                // semaphores --------------
                let rendering_complete_semaphore =
                    context.create_semaphore(vk::SemaphoreCreateFlags::empty());
                let presenting_complete_semaphore =
                    context.create_semaphore(vk::SemaphoreCreateFlags::empty());

                // write sets
                // *
                //update_ubuffer_desc_info(&context, &mut uniform_desc_buffer_info, frame_index);

                let write_sets = [
                    // set 0, binding 0
                    uniform_buffer_write_set(
                        &descriptor_sets_resource,
                        &[uniform_desc_buffer_info],
                    ),
                    // set 1, binding 0
                    storage_buffer_write_set(
                        &descriptor_sets_resource,
                        &[storage_buffer_desc_info],
                    ),
                    single_texture_write_set.clone(),
                ];

                unsafe {
                    context.device.update_descriptor_sets(&write_sets, &[]);
                }

                FrameData {
                    command_buffer,
                    render_complete_fence: render_fence,
                    rendering_complete_semaphore,
                    presenting_complete_semaphore,
                    uniform_buffer_descriptor_set: uniform_buffer_desc_set.handle,
                    frame_index: frame_index as _,
                }
            })
            .collect::<Vec<FrameData>>(),
    );

    let render_object = RenderObject {
        material: materials.get("default").clone(),
        mesh: meshes.get("monkey").clone(),
        translation: Vec3::new(0., 0., 0.),
        name: "bunny".to_owned(),
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
        //
        &mut FrameDataContainer,
        &mut UploadContext,
    )>,
    #[resource] meshes: &mut MeshesResource,
    #[resource] materials: &mut MaterialsResource,
    #[resource] descriptor_sets: &mut DescriptorSetsResource,
    #[resource] textures: &mut TexturesResource,
) {
    log::info!("RENDERER SHUTDOWN STARTED!");

    query.iter_mut(world).for_each(
        |(
            context,
            swapchain,
            frame_buffers,
            render_pass,
            depth_image,
            //descriptor_pool,
            frame_datas,
            upload_context,
        ): (
            &mut VkContext,
            &mut Swapchain,
            &mut FrameBuffers,
            &mut RenderPass,
            &mut DepthImage,
            &mut FrameDataContainer,
            //
            &mut UploadContext,
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
            textures.destroy(context);
            // ------------- END OF RESOURCES ----------

            upload_context.destroy(context);

            context.destroy();

            log::info!("Renderer finished!");
        },
    );
}
