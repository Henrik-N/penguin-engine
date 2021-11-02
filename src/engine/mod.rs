pub mod buffers;
pub mod math;
mod pe;
pub mod push_constants;
pub mod resources;

use math::prelude::*;
use resources::prelude::*;

#[allow(unused_imports)]
use push_constants::prelude::*;

use crate::core::config;
use crate::engine::render_backend::Core;
use anyhow::*;
use ash::vk;
use log;
use pe::pipeline::PPipelineBuilder;
use render_backend::RenderContext;

use crate::engine::buffers::prelude::*;
use crate::engine::resources::prelude::*;

use render_backend::FrameData;

// TODO
struct _Texture;
struct _RenderGraph;

// 2 == double buffering
pub const MAX_FRAMES_COUNT: usize = 2;

// Drop order: https://github.com/rust-lang/rfcs/blob/246ff86b320a72f98ed2df92805e8e3d48b402d6/text/1857-stabilize-drop-order.md
pub struct Renderer {
    // data
    // render_objects: Vec<RenderObject>,
    #[allow(dead_code)]
    meshes: MeshResource,
    #[allow(dead_code)]
    materials: HashResource<Material>,

    context: RenderContext,
    _core: Core,

    //
    frame_num: usize,
    wireframe_mode: bool,
    // descriptor_pool: vk::DescriptorPool,
}

impl Drop for Renderer {
    fn drop(&mut self) {
        log::debug!("Destroying renderer");

        log::debug!("Renderer: waiting for device idle..");
        unsafe {
            self.context
                .device
                .device_wait_idle()
                .expect("Device: couldn't wait for idle");
            log::debug!("Finished waiting for device");
        }
        log::debug!("Renderer: device now idle");
    }
}
impl Renderer {
    pub fn shutdown(&mut self) {
        // The call to this function call ensures the renderer doesn't get dropped until the event loop has ended
        log::debug!("Shutting down.");
    }

    pub(crate) fn create(window: &winit::window::Window) -> Result<Self> {
        let core = Core::create(&window)?;

        let context = RenderContext::create(&core, MAX_FRAMES_COUNT)?;

        let mut meshes =
            MeshResource::new(context.device_rc(), core.physical_device_memory_properties);

        meshes.insert_from_file("monkey", "monkey.obj");

        let vertex_input_bindings = Vertex::get_binding_descriptions();
        let vertex_attribute_descriptions = Vertex::get_attribute_descriptions();

        let descriptor_set_layouts = vec![context.global_descriptor_set_layout];

        // ***
        // Pipelines creation
        // ***
        let pipeline = PPipelineBuilder::default(
            &context.device,
            context.swapchain_extent,
            context.render_pass,
            vk::PipelineBindPoint::GRAPHICS,
        )
        .shaders(&["simple.vert", "simple.frag"])
        .vertex_input(&vertex_input_bindings, &vertex_attribute_descriptions)
        //.wireframe_mode()
        .descriptor_set_layouts(descriptor_set_layouts)
        //.add_push_constants::<MeshPushConstants>()
        .build();

        let mut materials = HashResource::new();
        materials.insert(
            "default",
            Material::from_pipeline(context.device_rc(), pipeline),
        );

        // let render_objects = vec![RenderObject::new(
        //     meshes.get_rc("monkey"),
        //     materials.get_rc("default"),
        //     Mat4::IDENTITY,
        // )];

        Ok(Self {
            _core: core,
            context,
            frame_num: 0,
            wireframe_mode: false,
            materials,
            meshes,
            //frame_data: outer_frame_data,
            //descriptor_pool,
            //render_objects,
        })
    }

    pub fn toggle_wireframe_mode(&mut self) {
        self.wireframe_mode = !self.wireframe_mode;
    }

    fn draw_render_objects(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        frame_data: &FrameData,
        frame_num: usize,
    ) {
        // create mvp matrix
        // let camera_loc = Vec3::new(0.0, 0.0, 3.0);
        // let camera_forward = Vec3::new(0.0, 0.0, -1.0);
        // let camera_up = Vec3::new(0.0, 1.0, 0.0);

        // let view = Mat4::look_at_rh(camera_loc, camera_loc + camera_forward, camera_up);

        // let model = Mat4::IDENTITY;

        // let vec = Vec4::new(1., 0., 0., 1.);
        //let translation_matrix = Mat4::from_translation(Vec3::new(1., 1., 0.));

        //let translation_matrix = Mat4::from_translation(Vec3::new(0., 0., 0.));

        //log::debug!("Vec: {:?}", vec);

        let fov_y_radians = 70.0_f32.to_radians();
        let aspect_ratio = config::WIDTH as f32 / config::HEIGHT as f32;
        let (z_near, z_far) = (0.1_f32, 200.0_f32);
        let mut projection = Mat4::perspective_rh(fov_y_radians, aspect_ratio, z_near, z_far);

        //projection[1][1] *= -1;
        log::debug!("Projection matrix: {:#?}", projection);


        let mat = self.materials.get_rc("default");
        let mesh = self.meshes.get_rc("monkey");


        //let cam_pos = Vec3::new(0., 0., -3.);
        //let view = Mat4::from_translation(cam_pos);


        // create mvp matrix
        let camera_loc = Vec3::new(0.0, 0.0, 3.0);
        let camera_forward = Vec3::new(0.0, 0.0, -1.0);
        let camera_up = Vec3::new(0.0, 1.0, 0.0);

        let view = Mat4::look_at_rh(camera_loc, camera_loc + camera_forward, camera_up);

        
        let fov_y = 70.0_f32.to_radians();
       

        let aspect_ratio = config::WIDTH as f32 / config::HEIGHT as f32;
        let (z_near, z_far) = (0.1_f32, 200.0_f32);
        let projection = Mat4::perspective_rh(
            fov_y,
            aspect_ratio,
            z_near,
            z_far);
        //let model = Mat4::from_translation(Vec3::new(0., 0.5, 0.));
        let spin = (frame_num as f32 * 0.4).to_radians();
        let model = Mat4::from_rotation_x(spin);
        
        let mesh_matrix = projection * view * model;

        let uniform_buffer_data = UniformBufferGlobalData {
            data: Vec4::default(),
            render_matrix: mesh_matrix,
        };



        frame_data.update_global_uniform_buffer(uniform_buffer_data);



        // bind pipeline
        mat.bind(command_buffer);

        // bind global descriptor sets
        frame_data.bind_global_descriptor_set(
            &self.context.device,
            command_buffer,
            mat.pipeline.pipeline_layout,
        );

        // TODO: bind descriptor sets corresponding to pipeline
        //

        unsafe {
            // bind vertex buffers
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[mesh.vertex_buffer.handle], &[0]);

            // draw mesh
            device.cmd_draw(command_buffer, mesh.vertex_count as u32, 1, 0, 0);
            //device.cmd_draw(command_buffer, mesh.vertices.len() as u32, 1, 0, 1);
        }

        // self.render_objects.iter().for_each(|obj| {
        //     // -----------------------------------

        //     // bind material if different from previous
        //     if let Some(material) = &bound_material {
        //         if material != &obj.material {
        //             obj.material.bind(command_buffer);
        //             bound_material = Some(Rc::clone(&obj.material));
        //         }
        //     } else {
        //         obj.material.bind(command_buffer);
        //     }

        //     // Bind mesh if different from previous
        //     if let Some(mesh) = &bound_mesh {
        //         if mesh != &obj.mesh {
        //             // bind vertex buffer
        //             unsafe {
        //                 device.cmd_bind_vertex_buffers(
        //                     command_buffer,
        //                     0,
        //                     &[obj.mesh.vertex_buffer.handle],
        //                     &[0],
        //                 );
        //             }
        //             bound_mesh = Some(Rc::clone(&obj.mesh));

        //             // bind index buffer
        //             unsafe {
        //                 device.cmd_bind_index_buffer(
        //                     command_buffer,
        //                     obj.mesh.index_buffer.handle,
        //                     0,
        //                     vk::IndexType::UINT16,
        //                 );
        //             }
        //         }
        //     } else {
        //         // bind mesh
        //         unsafe {
        //             device.cmd_bind_vertex_buffers(
        //                 command_buffer,
        //                 0,
        //                 &[obj.mesh.vertex_buffer.handle],
        //                 &[0],
        //             );
        //         }

        //         // bind index buffer
        //         unsafe {
        //             device.cmd_bind_index_buffer(
        //                 command_buffer,
        //                 obj.mesh.index_buffer.handle,
        //                 0,
        //                 vk::IndexType::UINT16,
        //             );
        //         }
        //     }

        //     // -----------------------------------

        //     unsafe {
        //         device.cmd_draw_indexed(command_buffer, obj.mesh.indices.len() as u32, 1, 0, 0, 1);
        //     }

        //     // unsafe {
        //     //     //device.cmd_bind_vertex_buffers(command_buffer, 0, &buffers, &device_sizes);
        //     //     device.cmd_draw(command_buffer, 3_u32, 1_u32, 0_u32, 0_u32);
        //     // }

        //     // ---
        //     //let model = obj.transform;

        //     //let mvp = projection * view * model;

        //     // let constants = MeshPushConstants {
        //     //     data: Vec4::new(1.0, 0.5, 0.0, 0.6), //some random data
        //     //     render_matrix: mvp,
        //     // };

        //     // device.cmd_push_constants(
        //     //     command_buffer,
        //     //     self.pipeline.pipeline_layout,
        //     //     MeshPushConstants::shader_stage(),
        //     //     std::mem::size_of::<MeshPushConstants>() as u32,
        //     //     constants.as_u8_slice(),
        //     // );

        //     //let buffers = [self.triangle.mesh.vertex_buffer.buffer_handle];
        //     //let device_sizes = [0_u64];
        // });
    }

    pub fn draw(&mut self, delta_time: f32) {
        let _ = delta_time;

        self.frame_num += 1;
        let frame_index = self.frame_num % MAX_FRAMES_COUNT;

        self.context.submit_render_commands(
            &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
            frame_index,
            |device, command_buffer, frame_buffer, frame_data| {
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
                    .render_pass(self.context.render_pass)
                    .framebuffer(frame_buffer)
                    .render_area(vk::Rect2D {
                        offset: vk::Offset2D { x: 0, y: 0 },
                        extent: self.context.swapchain_extent,
                    })
                    .clear_values(&clear_values);

                unsafe {
                    device.cmd_begin_render_pass(
                        command_buffer,
                        &render_pass_begin_info,
                        vk::SubpassContents::INLINE,
                    );

                    self.draw_render_objects(&self.context.device, command_buffer, frame_data, self.frame_num);

                    device.cmd_end_render_pass(command_buffer);
                }
            },
        );
    }
}

pub mod render_backend {
    use crate::core::logger;
    use crate::engine::buffers::prelude::*;
    use crate::engine::pe;
    use crate::engine::pe::command_buffers::record_submit_command_buffer;
    use crate::engine::pe::render_pass::PRenderPass;
    use crate::engine::resources::prelude::*;
    use anyhow::*;
    use ash::vk;
    use std::rc::Rc;

    pub struct Core {
        debug_utils_loader: ash::extensions::ext::DebugUtils,
        debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
        pub entry: ash::Entry,
        pub instance: ash::Instance,
        pub surface: vk::SurfaceKHR,
        pub surface_loader: ash::extensions::khr::Surface,
        pub physical_device: vk::PhysicalDevice,
        pub physical_device_memory_properties: vk::PhysicalDeviceMemoryProperties,
        pub queue_index: u32,
    }

    impl Core {
        pub fn create(window: &winit::window::Window) -> Result<Self> {
            let entry = unsafe { ash::Entry::new() }?;
            let required_surface_extensions = ash_window::enumerate_required_extensions(window)?;

            log::trace!("Creating Vulkan instance.");
            let instance: ash::Instance =
                pe::instance::create_ash_instance(&entry, &required_surface_extensions)?;

            let (debug_utils_loader, debug_messenger) =
                logger::init::init_vk_debug_messenger(&entry, &instance)?;

            let surface = unsafe { ash_window::create_surface(&entry, &instance, window, None)? };
            let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);

            log::trace!("Selecting physical device");
            let (physical_device, queue_index) =
                pe::device::select_physical_device(&instance, surface, &surface_loader)?;

            let physical_device_memory_properties =
                unsafe { instance.get_physical_device_memory_properties(physical_device) };

            Ok(Self {
                debug_utils_loader,
                debug_messenger,
                entry,
                instance,
                surface,
                surface_loader,
                physical_device,
                physical_device_memory_properties,
                queue_index,
            })
        }
    }
    impl Drop for Core {
        fn drop(&mut self) {
            unsafe {
                log::debug!("Dropping core");

                log::debug!("Destroying surface..");
                self.surface_loader.destroy_surface(self.surface, None);
                log::debug!("Surface destroyed!");

                log::debug!("Destroying debug messenger..");
                if let Some(debug_messenger) = self.debug_messenger {
                    self.debug_utils_loader
                        .destroy_debug_utils_messenger(debug_messenger, None);
                }
                log::debug!("Debug messenger destroyed!");

                log::debug!("Destroying instance!");
                self.instance.destroy_instance(None);
                log::debug!("Instance destroyed!");
            }
        }
    }

    pub(crate) struct FrameData {
        pub(super) command_pool: vk::CommandPool,
        pub(super) command_buffer: vk::CommandBuffer,

        pub(super) render_fence: vk::Fence,
        pub(super) rendering_complete_semaphore: vk::Semaphore,
        pub(super) presenting_complete_semaphore: vk::Semaphore,

        global_uniform_buffer: AllocatedBuffer,

        //global_descriptor_pool: vk::DescriptorPool,
        global_descriptor_set: vk::DescriptorSet,
    }
    impl FrameData {
        pub fn update_global_uniform_buffer(&self, buffer_data: UniformBufferGlobalData) {
            let buffer_data = [buffer_data];
            self.global_uniform_buffer.update_memory(&buffer_data);
        }

        pub fn bind_global_descriptor_set(
            &self,
            device: &ash::Device,
            command_buffer: vk::CommandBuffer,
            pipeline_layout: vk::PipelineLayout,
        ) {
            let first_set = 0;
            let descriptor_sets = [self.global_descriptor_set];
            let dynamic_offsets = [];

            unsafe {
                device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline_layout,
                    first_set,
                    &descriptor_sets,
                    &dynamic_offsets,
                );
            }
        }

        pub fn new(
            device: Rc<ash::Device>,
            queue_index: u32,
            pd_memory_properties: vk::PhysicalDeviceMemoryProperties,
            descriptor_pool: vk::DescriptorPool,
            global_descriptor_set_layout: vk::DescriptorSetLayout,
        ) -> Self {
            // command pool and command buffer ---------
            let (command_pool, command_buffer) =
                pe::command_buffers::init::create_command_pool_and_buffer(&device, queue_index);

            // fences ---------
            let render_fence_create_info =
                vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED); // start signaled, to wait for it before the first gpu command

            let render_fence = unsafe { device.create_fence(&render_fence_create_info, None) }
                .expect("Failed to create render fence.");

            // semaphores --------------
            let semaphore_create_info = vk::SemaphoreCreateInfo::default();

            let rendering_complete_semaphore =
                unsafe { device.create_semaphore(&semaphore_create_info, None) }
                    .expect("Failed to create semaphore");
            let presenting_complete_semaphore =
                unsafe { device.create_semaphore(&semaphore_create_info, None) }
                    .expect("Failed to create semaphore");

            //Decscriptor buffer
            let initial_data = [UniformBufferGlobalData::default()];

            let global_uniform_buffer = AllocatedBuffer::create_buffer_updateable(
                Rc::clone(&device),
                pd_memory_properties,
                &initial_data,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                MemoryUsage::CpuToGpu,
            );

            // descriptor sets
            //
            let descriptor_set_layouts = [global_descriptor_set_layout];

            assert_eq!(descriptor_set_layouts.len(), 1);

            let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(descriptor_pool)
                //.descriptor_set_count(1)
                .set_layouts(&descriptor_set_layouts);

            log::debug!("Allocating a descriptor set.");
            let allocated_descriptor_sets =
                unsafe { device.allocate_descriptor_sets(&descriptor_set_allocate_info) }
                    .expect("Couldn't allocate global descriptor set");

            log::debug!("Descriptor set count {}", allocated_descriptor_sets.len());

            // point descriptor set to buffer
            let buffer_info = [vk::DescriptorBufferInfo::builder()
                .buffer(global_uniform_buffer.handle)
                .offset(0)
                .range(std::mem::size_of::<UniformBufferGlobalData>() as u64)
                .build()];

            let set_write = [vk::WriteDescriptorSet::builder()
                // writing to binding 0
                .dst_binding(UniformBufferGlobalData::binding_index())
                .dst_set(allocated_descriptor_sets[0])
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_info)
                .build()];

            unsafe {
                device.update_descriptor_sets(&set_write, &[]);
            }

            Self {
                command_pool,
                command_buffer,
                render_fence,
                rendering_complete_semaphore,
                presenting_complete_semaphore,
                global_uniform_buffer,
                global_descriptor_set: allocated_descriptor_sets[0],
            }
        }

        pub unsafe fn destroy(&mut self, device: &ash::Device) {
            device.destroy_semaphore(self.presenting_complete_semaphore, None);
            device.destroy_semaphore(self.rendering_complete_semaphore, None);
            device.destroy_fence(self.render_fence, None);
            device.destroy_command_pool(self.command_pool, None);
        }
    }



    enum DescriptorSetUsageFrequency {
        Global,
        PerPass,
        PerFrame,
    }
    

    pub struct DescriptorPool {
        descriptor_pool: vk::DescriptorPool,
        // global resources, bound once per frame
        global_layout: vk::DescriptorSetLayout,
        global_set: vk::DescriptorSetLayout,
        // pass resources, bound once per render pass
        pass_set: vk::DescriptorSet,
        // render resources, bound once per render object (at most)
        material_set: vk::DescriptorSet,
        object_set: vk::DescriptorSet,
    }



    /// Struct containing most Vulkan object handles and global states.
    #[allow(dead_code)]
    pub(super) struct RenderContext {
        pub(super) device: Rc<ash::Device>,
        pub(super) queue_handle: vk::Queue,

        pub(super) swapchain_loader: ash::extensions::khr::Swapchain,
        pub(super) swapchain: vk::SwapchainKHR,
        pub(super) swapchain_format: vk::Format,
        pub(super) swapchain_extent: vk::Extent2D,

        pub(super) swapchain_images: Vec<vk::Image>,
        pub(super) swapchain_image_views: Vec<vk::ImageView>,

        pub(super) depth_image: AllocatedImage,
        pub(super) depth_image_view: vk::ImageView,

        pub(super) global_descriptor_set_layout: vk::DescriptorSetLayout,
        pub(super) global_descriptor_pool: vk::DescriptorPool,
        // contains frame data
        pub(super) frame_data: Vec<FrameData>,

        pub(super) render_pass: vk::RenderPass,
        pub(super) frame_buffers: Vec<vk::Framebuffer>,
    }

    impl RenderContext {
        /// Create a new reference to the same allocation as ash::Device
        pub fn device_rc(&self) -> Rc<ash::Device> {
            Rc::clone(&self.device)
        }

        pub fn submit_render_commands<
            RenderPassFn: FnOnce(&ash::Device, vk::CommandBuffer, vk::Framebuffer, &FrameData),
        >(
            &self,
            pipeline_stage_flags: &[vk::PipelineStageFlags],
            frame_index: usize,
            render_pass_fn: RenderPassFn,
        ) {
            let frame = &self.frame_data[frame_index];

            let wait_semaphores = [frame.presenting_complete_semaphore];

            // request new image from swapchain
            let (image_index, _is_suboptimal) = unsafe {
                log::trace!("Aquiring next swapchain image");
                // timeout 1 sec, specified in nanoseconds
                self.swapchain_loader.acquire_next_image(
                    self.swapchain,
                    1000000000,
                    wait_semaphores[0],
                    vk::Fence::null(),
                )
            }
            .expect("Couldn't acquire next swapchain image");
            log::trace!("Swapchain image {} aquired!", image_index);

            let frame_buffer = self
                .frame_buffers
                .get(image_index as usize)
                .expect("Couldn't get frame buffer at the given index");

            record_submit_command_buffer(
                &self.device,
                frame.command_buffer,
                frame.render_fence,
                self.queue_handle,
                pipeline_stage_flags,
                &wait_semaphores,
                &[frame.rendering_complete_semaphore],
                *frame_buffer,
                frame,
                render_pass_fn,
            );

            // after commands are submitted, wait for rending to complete and then display the image to the screen
            let swapchains = [self.swapchain];
            let wait_semaphores = [frame.rendering_complete_semaphore];
            let image_indices = [image_index];
            let present_info = vk::PresentInfoKHR::builder()
                .swapchains(&swapchains)
                .wait_semaphores(&wait_semaphores)
                .image_indices(&image_indices);

            unsafe {
                self.swapchain_loader
                    .queue_present(self.queue_handle, &present_info)
                    .expect("Couldn't submit to present queue");
            }
        }

        pub fn create(core: &Core, max_frames_count: usize) -> Result<Self> {
            log::trace!("Queue index: {}", core.queue_index);

            log::trace!("Creating logical device");
            let device = Rc::new(pe::device::create_logical_device(
                &core.instance,
                core.physical_device,
                core.queue_index,
            ));

            log::trace!("Getting graphics queue handle");
            let queue_handle: vk::Queue =
                pe::device::get_graphics_queue_handle(&device, core.queue_index);

            log::trace!("Quering device for swapchain support");
            let swapchain_support_details = pe::device::query_swapchain_support(
                core.physical_device,
                core.surface,
                &core.surface_loader,
            );

            log::trace!("Creating swapchain");
            let swapchain = pe::swapchain::PSwapchain::create(
                &core.instance,
                &device,
                core.surface,
                swapchain_support_details,
                core.queue_index,
            );

            let pe::swapchain::PSwapchain {
                swapchain_loader,
                swapchain,
                swapchain_format,
                swapchain_extent,
                swapchain_images,
                swapchain_image_views,
            } = swapchain;

            // depth images

            // NOTE: Hardcoded for now, also hardcoded in the render pass
            //let depth_format = vk::Format::D32_SFLOAT;
            let depth_format = vk::Format::D16_UNORM;

            let depth_image_create_info = vk::ImageCreateInfo::builder()
                .image_type(vk::ImageType::TYPE_2D)
                .format(depth_format)
                .extent(vk::Extent3D {
                    width: swapchain_extent.width,
                    height: swapchain_extent.height,
                    depth: 1,
                })
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let depth_image = AllocatedImage::create(
                &device,
                core.physical_device_memory_properties,
                &depth_image_create_info,
            );

            let image_view_create_info = vk::ImageViewCreateInfo::builder()
                .format(depth_format)
                .image(depth_image.image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .subresource_range(
                    vk::ImageSubresourceRange::builder()
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1)
                        .aspect_mask(vk::ImageAspectFlags::DEPTH)
                        .build(),
                );

            let depth_image_view =
                unsafe { device.create_image_view(&image_view_create_info, None) }
                    .expect("Couldn't create depth image view");

            // render pass --------------

            let (render_pass, _attachment_count) =
                PRenderPass::create_default_render_pass(&device, &swapchain_format);

            // frame buffers --------------
            let frame_buffers: Vec<vk::Framebuffer> = swapchain_image_views
                .iter()
                .map(|&image_view| {
                    let attachments = [image_view, depth_image_view];
                    let create_info = vk::FramebufferCreateInfo::builder()
                        .render_pass(render_pass)
                        .attachments(&attachments)
                        .width(swapchain_extent.width)
                        .height(swapchain_extent.height)
                        .layers(1);

                    unsafe { device.create_framebuffer(&create_info, None) }
                        .expect("Couldn't create framebuffer")
                })
                .collect();

            // descriptor pool ------------------------
            let global_descriptor_set_layout =
                UniformBufferGlobalData::create_descriptor_set_layout(&device);

            let descriptor_pool_size = [vk::DescriptorPoolSize::builder()
                // reserve 1 handle
                .descriptor_count(10) // 10 uniform buffers
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .build()];

            let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::builder()
                .max_sets(10 as u32)
                .pool_sizes(&descriptor_pool_size);

            let global_descriptor_pool =
                unsafe { device.create_descriptor_pool(&descriptor_pool_create_info, None) }
                    .expect("Couldn't create descriptor pool");

            unsafe { device.device_wait_idle() }.unwrap();

            let frame_data: Vec<FrameData> = (0..max_frames_count)
                .map(|_| {
                    FrameData::new(
                        Rc::clone(&device),
                        core.queue_index,
                        core.physical_device_memory_properties,
                        global_descriptor_pool,
                        global_descriptor_set_layout,
                    )
                })
                .collect();

            Ok(Self {
                device,
                queue_handle,
                swapchain,
                swapchain_loader,
                swapchain_format,
                swapchain_extent,
                swapchain_images,
                swapchain_image_views,
                global_descriptor_set_layout,
                global_descriptor_pool,
                frame_data,
                render_pass,
                frame_buffers,
                depth_image,
                depth_image_view,
            })
        }
    }
    impl Drop for RenderContext {
        fn drop(&mut self) {
            unsafe {
                log::debug!("Dropping render context");
                self.device
                    .device_wait_idle()
                    .expect("Device: couldn't wait for idle");
                log::debug!("Finished waiting for device");

                log::debug!("Render context: dropping framebuffers");
                self.frame_buffers.iter().for_each(|&frame_buffer| {
                    self.device.destroy_framebuffer(frame_buffer, None);
                });

                log::debug!("Render context: dropping render pass");
                self.device.destroy_render_pass(self.render_pass, None);



                self.device
                    .destroy_descriptor_pool(self.global_descriptor_pool, None);



                log::debug!("Render context: dropping frame data");
                self.frame_data.iter_mut().for_each(|frame| {
                    frame.destroy(&self.device);
                });


                log::debug!("Destroying swapchain image views");
                self.swapchain_image_views.iter().for_each(|&image_view| {
                    self.device.destroy_image_view(image_view, None);
                });


                // depth image
                self.device.destroy_image_view(self.depth_image_view, None);
                self.depth_image.destroy(&self.device);


                log::debug!("Destroying swapchain loader");
                self.swapchain_loader
                    .destroy_swapchain(self.swapchain, None);

                log::debug!("Destroying device");

                log::debug!("Rc count: {}", Rc::strong_count(&self.device));
                self.device.destroy_device(None);
            }
        }
    }
}
