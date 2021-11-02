use ash::vk;
use super::buffers::*;
use super::math::prelude::*;
use std::rc::Rc;

/// Trait with useful functions for vk::DescriptorSet
pub trait DescriptorSet {
    fn create_descriptor_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout;
    fn binding_index() -> u32;
    fn descriptor_set_binding_flags() -> vk::DescriptorPoolCreateFlags;
}

/// Helper function to create a descriptor set layout
fn descriptor_set_layout(device: &ash::Device, 
                                    binding_index: u32,
                                    shader_stage_flags: vk::ShaderStageFlags) -> vk::DescriptorSetLayout {
    let layout_bindings = [vk::DescriptorSetLayoutBinding::builder()
        .binding(binding_index)
        .descriptor_count(1)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .stage_flags(shader_stage_flags)
        //.immutable_samplers()
        .build()];

    let create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&layout_bindings);

    unsafe { device.create_descriptor_set_layout(&create_info, None) }
        .expect("Couldn't create descriptor set layout")
}

/// Enum for helping to simplify which flags fit for which descriptor set's use case
#[allow(dead_code)]
enum DescriptorSetBindingFrequency {
        Global,
        PerPass,
        PerFrame,
}
impl DescriptorSetBindingFrequency {
    fn descriptor_flag_bits(&self) -> vk::DescriptorPoolCreateFlags {
        match self {
            Self::Global => vk::DescriptorPoolCreateFlags::empty(),
            Self::PerPass => vk::DescriptorPoolCreateFlags::empty(),
            // don't allow freeing of the set, can increase performance when
            // the set is used this frequently
            Self::PerFrame => vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET,
        }
    }
}


/// Gpu-side uniform buffer with all buffer data
pub struct UniformBuffer {
    device: Rc<ash::Device>,
    pub global_layout: vk::DescriptorSetLayout,
    pub global_set: vk::DescriptorSet,
    pub buffer: AllocatedBuffer,
}
/// Cpu-side buffer to allocate data for UniformBuffer
#[derive(Default, Clone, Copy)]
#[repr(C)] // ensure compiler doesn't reorder properties
struct UniformBufferData {
    _global_data: UniformBufferGlobalData,
    //pub data: Vec4,
    //pub render_matrix: Mat4,
}


/// Global uniform data, bound once per frame
#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct UniformBufferGlobalData {
    pub data: Vec4,
    pub render_matrix: Mat4,
}
impl DescriptorSet for UniformBufferGlobalData {
    fn binding_index() -> u32 {0_u32}

    fn create_descriptor_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
        let binding_index = Self::binding_index();
        descriptor_set_layout(device, binding_index, vk::ShaderStageFlags::VERTEX)
    }

    fn descriptor_set_binding_flags() -> vk::DescriptorPoolCreateFlags {
        DescriptorSetBindingFrequency::Global.descriptor_flag_bits()
    }
}


impl UniformBuffer {
    pub fn create_descriptor_set_layout<T: DescriptorSet>(device: &ash::Device) 
    -> vk::DescriptorSetLayout {
        T::create_descriptor_set_layout(device)
    }

    pub fn bind_global_set(&self, 
                           command_buffer: vk::CommandBuffer,
                           pipeline_layout: vk::PipelineLayout,
                           ) {
        let first_set = 0;

        let descriptor_sets = [self.global_set];
        let dynamic_offsets = [];

        unsafe {
            self.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                first_set,
                &descriptor_sets,
                &dynamic_offsets,
            );
        }
    }

    pub fn new(device: Rc<ash::Device>, 
               pd_memory_properties: vk::PhysicalDeviceMemoryProperties,
               descriptor_pool: vk::DescriptorPool) -> Self {
        let initial_data = [UniformBufferData::default()];

        // TODO: Need to recreate the allocation part to be able to have 
        let uniform_buffer = AllocatedBuffer::create_buffer_updateable(
            Rc::clone(&device),
            pd_memory_properties,
            &initial_data,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            MemoryUsage::CpuToGpu,
        );

        // descriptor layouts
        let global_layout = UniformBufferGlobalData::create_descriptor_set_layout(&device);
        let descriptor_set_layouts = [global_layout];

        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&descriptor_set_layouts);

        log::debug!("Allocating descriptor sets.");
        let allocated_descriptor_sets =
            unsafe { device.allocate_descriptor_sets(&descriptor_set_allocate_info) }
                .expect("Couldn't allocate global descriptor set");

        // point descriptor set to buffer
        let buffer_info = [vk::DescriptorBufferInfo::builder()
            .buffer(uniform_buffer.handle)
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
            device: Rc::clone(&device),
            global_layout,
            global_set: allocated_descriptor_sets[0],
            buffer: uniform_buffer,
        }
    }

    pub fn update_global_memory(&self, new_data: UniformBufferGlobalData) {
        let buffer_data = [new_data];
        self.buffer.update_memory(&buffer_data);
    }

}



// pub struct UniformGlobalData {
//     pub data: Vec4,
//     pub render_matrix: Mat4,
// }
// impl UniformGlobalData {
//     
// }


/// Uniform buffer intended for global data, i.e data that will be bound once per frame.
/// TODO: Change to only camera data,
/// model data will be seperate
//#[derive(Clone, Copy, Default)]
//pub struct UniformBufferGlobalData {
//    pub data: Vec4,
//    pub render_matrix: Mat4,
//    // pub model: Mat4,
//    // pub view: Mat4,
//    // pub projection: Mat4,
//
//    // **
//    // GPU camera data
//    // **
//    //view: Mat4, // camera transform
//    //projection: Mat4, // perspective matrix
//    // view_proj: Mat4, // both of the above multiplied together,
//    // to avoid having to multiply them in the shader
//}
//impl DescriptorSet for UniformBufferGlobalData {
//    fn descriptor_set_binding_frequency() -> vk::DescriptorPoolCreateFlags {
//        DescriptorSetBindingFrequency::Global.descriptor_flag_bits()
//    }
//
//    fn binding_index() -> u32 {
//        0_u32
//    }
//
//    fn create_descriptor_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
//        let layout_bindings = [vk::DescriptorSetLayoutBinding::builder()
//            .binding(Self::binding_index())
//            .descriptor_count(1)
//            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
//            .stage_flags(vk::ShaderStageFlags::VERTEX)
//            //.immutable_samplers()
//            .build()];
//
//        let create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&layout_bindings);
//
//        unsafe { device.create_descriptor_set_layout(&create_info, None) }
//            .expect("Couldn't create descriptor set layout")
//    }
//}



/// Uniform buffer intended for data that only needs to be bound
/// once per pass
//pub struct PassData {}

// Data bound in the inner render loops
//pub struct MaterialData {}
// Data bound in the inner render loops
//pub struct ObjectData {}



pub struct DescriptorPool {
    descriptor_pool: vk::DescriptorPool,
    // global resources, bound once per frame
    //global_layout: vk::DescriptorSetLayout,
    //global_set: vk::DescriptorSet,
    
    global_set_layout: vk::DescriptorSetLayout,
    // pass resources, bound once per render pass
    //render_pass_set_layout: vk::DescriptorSetLayout,
    // render resources, bound once per render object (at most)
    //material_set_layout: vk::DescriptorSet,
    //object_set_layout: vk::DescriptorSet,
}
impl DescriptorPool {
    const MAX_UNIFORM_BUFFER_COUNT: u32 = 10;
    const MAX_DESCRIPTOR_SET_COUNT: u32 = 10;

    pub fn create_pool(device: &ash::Device) -> Self {
        //let global_set_layout =
            //UniformBufferGlobalData::create_descriptor_set_layout(&device);
        let global_set_layout = Self::create_global_set_layout(&device);

        let descriptor_pool_size = [vk::DescriptorPoolSize::builder()
            .descriptor_count(Self::MAX_UNIFORM_BUFFER_COUNT) // 10 uniform buffers
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .build()];

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(Self::MAX_DESCRIPTOR_SET_COUNT)
            .pool_sizes(&descriptor_pool_size);

        let descriptor_pool =
            unsafe { device.create_descriptor_pool(&descriptor_pool_create_info, None) }
                .expect("Couldn't create descriptor pool");

        Self {
            descriptor_pool,
            global_set_layout,
            //global_layout,
        }
    }

    fn create_global_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
        let layout_bindings = [vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            //.immutable_samplers()
            .build()];

        let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&layout_bindings);

        Self::create_set_layout(device, &create_info)
    }

    fn create_set_layout(device: &ash::Device, create_info: &vk::DescriptorSetLayoutCreateInfoBuilder) -> vk::DescriptorSetLayout {
        unsafe { device.create_descriptor_set_layout(&create_info, None) }
            .expect("Couldn't create descriptor set layout")
    }

    fn descriptor_buffer(
        &self,
        device: Rc<ash::Device>,
        queue_index: u32,
        pd_memory_properties: vk::PhysicalDeviceMemoryProperties,
        descriptor_pool: vk::DescriptorPool,
        global_descriptor_set_layout: vk::DescriptorSetLayout,
        ) {

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
    }
}
