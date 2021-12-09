use ash::vk;
use penguin_app::ecs::*;
use penguin_app::window::Window;
use crate::render_objects::Vertex;
use crate::renderer::frame_data::{FrameData, FrameDataContainer};
use crate::renderer::resources::{MaterialsResource, MeshesResource};
use crate::renderer::descriptor_sets::uniform_buffers::UniformBuffers;
use crate::renderer::vk_types::*;
//use crate::renderer::vk_types::{Pipeline, vk_components, VkComponents, VkContext};

#[system]
pub fn renderer_startup(cmd: &mut legion::systems::CommandBuffer,
                        #[resource] window: &Window,
                        #[resource] meshes: &mut MeshesResource,
                        #[resource] materials: &mut MaterialsResource) {
    log::info!("RENDERER STARTUP STARTED!");
    // /------------------ CONTEXT  -----------------------------------------------------
    let context = VkContext::init(window, window.logger_level);
    // ///////////////////////////////////////


    // /------------------ OTHER RENDERER STRUCTS----------------------------------------


    let VkComponents {
        swapchain,
        depth_image,
        render_pass,
        frame_buffers,
        descriptor_pool
    } = vk_components::init_vk_components(window, &context);
    // ///////////////////////////////////////



    let uniform_buffers = UniformBuffers::init(&context, descriptor_pool);





    let frame_datas = FrameDataContainer {
        frame_datas: (0..penguin_config::vk_config::MAX_FRAMES_COUNT)
            .map(|frame_index| FrameData::new(&context, frame_index)).collect()
    };



    let vertex_input_bindings = Vertex::get_binding_descriptions();
    let vertex_input_attribute_descriptions = Vertex::get_attribute_descriptions();

    let pipeline = Pipeline::builder(
        &context.device.handle,
        swapchain.extent,
        render_pass.handle,
        vk::PipelineBindPoint::GRAPHICS,
    )
        .shaders(&["simple.vert", "simple.frag"])
        .vertex_input(&vertex_input_bindings, &vertex_input_attribute_descriptions)
        //.wireframe_mode()
        //.descriptor_set_layouts(gpu_uniform_buffer.get_descriptor_set_layouts())

        .descriptor_set_layouts(vec![
            uniform_buffers.global.get_descriptor_set_layout(),
            uniform_buffers.per_frame.get_descriptor_set_layout(),
        ])

        //.add_push_constants::<MeshPushConstants>()
        .build();




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
        uniform_buffers,
    ));
}


#[system]
pub fn renderer_shutdown(
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
        &mut UniformBuffers,
        //&mut GpuBuffer<UniformBufferGlobalData>,
    )>,
    #[resource] meshes: &mut MeshesResource,
    #[resource] materials: &mut MaterialsResource,
) {
    log::info!("RENDERER SHUTDOWN STARTED!");

    query.iter_mut(world).for_each(
        |(context, swapchain, frame_buffers, render_pass, depth_image, descriptor_pool, frame_datas,
             uniform_buffers): (
            &mut VkContext,
            &mut Swapchain,
            &mut FrameBuffers,
            &mut RenderPass,
            &mut DepthImage,
            &mut DescriptorPool,
            &mut FrameDataContainer,
            &mut UniformBuffers,
        )| {
            // wait for device idle..
            context.wait_for_device_idle();

            frame_datas.destroy(&context);

            frame_buffers.destroy(&context);

            render_pass.destroy(&context);


            uniform_buffers.destroy(&context);


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


