#![allow(dead_code)]

use super::buffers::*;
use super::math::prelude::*;
use crate::core::config;
use ash::vk;
use std::rc::Rc;

/// Memory
mod mem {
    #[derive(PartialEq, Eq)]
    pub enum MemType {
        Persistent,
        Dynamic,
    }

    pub fn packed_range_from_min_align<T>(min_align: u64) -> u64 {
        let mem_range = std::mem::size_of::<T>() as u64;
        let mut packed_range = 0;
        while packed_range < mem_range && packed_range < min_align {
            packed_range += min_align;
        }
        packed_range
    }
}

/// Descriptor set layouts cache
///
///
pub struct DescriptorSetLayoutsCache(Vec<vk::DescriptorSetLayout>);
impl DescriptorSetLayoutsCache {
    pub fn destroy(self, device: &ash::Device) {
        self.0.into_iter().for_each(|layout| unsafe {
            device.destroy_descriptor_set_layout(layout, None);
        });
    }

    pub fn get_layout(&self, binding_index: usize) -> vk::DescriptorSetLayout {
        self.0
            .get(binding_index)
            .expect("Binding index out of scope")
            .clone()
    }

    pub fn create(device: &ash::Device) -> Self {
        let layout_bindings = [
            (
                // binding 0, vertex uniform buffer
                [vk::DescriptorSetLayoutBinding::builder()
                    .binding(0)
                    .descriptor_count(1)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                    .stage_flags(vk::ShaderStageFlags::VERTEX)
                    .build()],
                vk::DescriptorSetLayoutCreateFlags::empty(),
            ),
            (
                // binding 1, fragment uniform buffer
                [vk::DescriptorSetLayoutBinding::builder()
                    .binding(1)
                    .descriptor_count(1)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                    .build()],
                vk::DescriptorSetLayoutCreateFlags::empty(),
            ),
        ];

        let descriptor_layouts = layout_bindings
            .into_iter()
            .map(|(layouts, create_flags)| Self::create_set_layout(device, &layouts, create_flags))
            .collect::<Vec<vk::DescriptorSetLayout>>();

        Self(descriptor_layouts)
    }

    fn create_set_layout(
        device: &ash::Device,
        layout_bindings: &[vk::DescriptorSetLayoutBinding],
        create_flags: vk::DescriptorSetLayoutCreateFlags,
    ) -> vk::DescriptorSetLayout {
        let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&layout_bindings)
            .flags(create_flags);

        unsafe { device.create_descriptor_set_layout(&create_info, None) }
            .expect("Couldn't create descriptor set layout")
    }
}

mod descriptor_set_builder {
    use ash::vk;
    //use super::descriptor_set_layout;
    use super::mem;
    use super::DescriptorSetAllocator;
    use super::DescriptorSetLayoutsCache;
    use super::MemoryUsage;
    use std::rc::Rc;

    pub struct BufferCreationData {
        pub pd_properties: vk::PhysicalDeviceProperties,
        pub pd_memory_properties: vk::PhysicalDeviceMemoryProperties,
    }

    pub struct DescriptorSetBuilder<'a> {
        device: Rc<ash::Device>,
        layouts_cache: &'a DescriptorSetLayoutsCache,
        allocator: &'a DescriptorSetAllocator,
        buffer_creation_data: &'a BufferCreationData,
        //
        descriptors: Vec<DescriptorData>,
    }

    struct DescriptorData {
        binding: usize,
        //set_layout: vk::DescriptorSetLayout,
        mem_type: mem::MemType,

        packed_align: u64,
        mem_range: u64,
        mem_size: u64, // mem_range * instances
    }

    impl<'a> DescriptorSetBuilder<'a> {
        pub fn builder(
            device: Rc<ash::Device>,
            layouts_cache: &'a DescriptorSetLayoutsCache,
            allocator: &'a DescriptorSetAllocator,
            buffer_creation_data: &'a BufferCreationData,
        ) -> Self {
            Self {
                device: Rc::clone(&device),
                layouts_cache,
                allocator,
                buffer_creation_data,
                descriptors: Vec::new(),
            }
        }

        pub fn descriptor_set<DescriptorDataType: Sized>(
            self,
            binding: usize,
            instances: usize,
            mem_type: mem::MemType,
        ) {
            let min_ubuffer_offset_align = self
                .buffer_creation_data
                .pd_properties
                .limits
                .min_uniform_buffer_offset_alignment;

            // several instances means allocating at different times, memory needs to be packed for
            // it's buffer region
            let packed_align =
                mem::packed_range_from_min_align::<DescriptorDataType>(min_ubuffer_offset_align);
            let mem_size = packed_align * instances as u64;

            let mem_range = std::mem::size_of::<DescriptorDataType>() as u64;

            let mem_data = DescriptorData {
                binding,
                mem_type,
                packed_align,
                mem_range,
                mem_size,
            };

            todo!()

            //let fb_packed_align = mem::packed_range_from_min_align::<UniformBufferFrameData>(min_ubuffer_offset_align);
            //let packed_size = packed_align * instances
        }

        pub fn build(self) {
            // persistent and dynamic memory should be exclusively aligned
            // (if min align is 256 bytes and persistent is <256, dynamic starts at offset 256)

            let set_layouts = self
                .descriptors
                .iter()
                .map(|desc| self.layouts_cache.get_layout(desc.binding))
                .collect::<Vec<vk::DescriptorSetLayout>>();

            //let both_mem_types = self.descriptors.iter().any(|desc| desc.mem_type == mem::MemType::Persistent)
            &&self
                .descriptors
                .iter()
                .any(|desc| desc.mem_type == mem::MemType::Dynamic);

            //let set_layout = self.layouts_cache.get_layout(binding);

            let persistent_bytes_count = {
                let mut counter = 0;
                for desc in self
                    .descriptors
                    .iter()
                    .filter(|desc| desc.mem_type == mem::MemType::Persistent)
                {
                    counter += desc.packed_align;
                }
                counter
            };

            // TODO ......................

            // The function create_buffer automatically the memory upon creation
            // based on the given struct's size, unless Manual is specified TODO.
            //
            //let packed_size_bytes: u64 = gb_packed_align + fb_packed_align * config::MAX_FRAMES_COUNT as u64;
            // let initial_data: Vec<u8> = (0..packed_size_bytes).map(|_| 0_u8).collect();

            //
            // let gb_create_info = super::AllocatedBufferCreateInfo {
            //     device: Rc::clone(&self.device),
            //     pd_memory_properties: self.pd_memory_properties,
            //     initial_data: &initial_data, // size of allocated memory will be size of initial_data, the rest of the size is for dynamic mem
            //     buffer_usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
            //     memory_usage: MemoryUsage::CpuToGpu,
            //     memory_map_flags: vk::MemoryMapFlags::empty(),
            //     sharing_mode: vk::SharingMode::EXCLUSIVE,
            // };

            // let global_buffer = AllocatedBuffer::create_buffer(&gb_create_info);
        }
    }
}

const POOL_SIZE_MULTIPLIERS: [(vk::DescriptorType, u32); 2] = [
    (vk::DescriptorType::UNIFORM_BUFFER, 10),
    (vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC, 10),
];

pub struct DescriptorSetAllocator {
    device: Rc<ash::Device>,

    pool: vk::DescriptorPool,

    descriptor_layouts: vk::DescriptorSetLayout,
}

// Only as an example
struct DescriptorSets {
    global_per_frame: vk::DescriptorSet,
}
//

struct DescriptorBuilder {}

/// Trait with useful functions for vk::DescriptorSet
pub trait DescriptorSet {
    fn create_descriptor_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout;
    fn binding_index() -> u32;
    fn descriptor_set_binding_flags() -> vk::DescriptorPoolCreateFlags;
}

/// Helper function to create a descriptor set layout
fn descriptor_set_layout(
    device: &ash::Device,
    binding_index: u32,
    shader_stage_flags: vk::ShaderStageFlags,
    descriptor_type: vk::DescriptorType,
) -> vk::DescriptorSetLayout {
    let layout_bindings = [vk::DescriptorSetLayoutBinding::builder()
        .binding(binding_index)
        .descriptor_count(1)
        .descriptor_type(descriptor_type)
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
    min_ubuffer_offset_align: u64,
    pub global_layout: vk::DescriptorSetLayout,
    pub global_set: vk::DescriptorSet,
    pub global_buffer: AllocatedBuffer,
    //
    pub frames_layout: vk::DescriptorSetLayout,
    pub frames_set: vk::DescriptorSet,
    //pub frames_buffer: AllocatedBuffer,
}


/// Cpu-side buffer to allocate data for UniformBuffer
#[derive(Default, Clone, Copy)]
#[repr(C)] // ensure compiler doesn't reorder properties
#[repr(align(256))]
struct UniformBufferData {
    _global_data: UniformBufferGlobalData,
    _frame_data: [UniformBufferFrameData; config::MAX_FRAMES_COUNT],
}

/// Global uniform data, bound once per frame
#[derive(Default, Clone, Copy)]
#[repr(C)]
#[repr(align(256))]
pub struct UniformBufferGlobalData {
    pub data: Vec4,
    pub render_matrix: Mat4,
}
impl DescriptorSet for UniformBufferGlobalData {
    fn binding_index() -> u32 {
        0_u32
    }

    fn create_descriptor_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
        let binding_index = Self::binding_index();
        descriptor_set_layout(
            device,
            binding_index,
            vk::ShaderStageFlags::VERTEX,
            vk::DescriptorType::UNIFORM_BUFFER,
        )
    }

    fn descriptor_set_binding_flags() -> vk::DescriptorPoolCreateFlags {
        DescriptorSetBindingFrequency::Global.descriptor_flag_bits()
    }
}
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
impl DescriptorSet for UniformBufferFrameData {
    fn binding_index() -> u32 {
        1_u32
    }

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

    fn descriptor_set_binding_flags() -> vk::DescriptorPoolCreateFlags {
        DescriptorSetBindingFrequency::PerFrame.descriptor_flag_bits()
    }
}

impl UniformBuffer {
    pub fn create_descriptor_set_layout<T: DescriptorSet>(
        device: &ash::Device,
    ) -> vk::DescriptorSetLayout {
        T::create_descriptor_set_layout(device)
    }

    pub fn write_global_memory(&self, new_data: UniformBufferGlobalData) {
        let buffer_data = [new_data];
        self.global_buffer.write_memory(&buffer_data, 0);
    }

    fn frames_packed_offset<T>(&self, frame_index: u64) -> u64 {
        //let mem_range = std::mem::size_of::<T>() as u64;
        //let min_align = self.min_ubuffer_offset_align;

        let mem_range = Self::packed_range_from_min_align::<UniformBufferGlobalData>(
            self.min_ubuffer_offset_align,
        );

        mem_range * frame_index
    }

    fn packed_range_from_min_align<T>(min_align: u64) -> u64 {
        let mem_range = std::mem::size_of::<T>() as u64;
        let mut packed_range = 0;
        while packed_range < mem_range && packed_range < min_align {
            packed_range += min_align;
        }
        packed_range
    }

    pub fn bind_descriptor_sets(
        &self,
        command_buffer: vk::CommandBuffer,
        pipeline_layout: vk::PipelineLayout,
        frame_index: usize,
    ) {
        let first_set = 0;
        let dynamic_offset = std::mem::size_of::<UniformBufferGlobalData>() as u32
            + std::mem::size_of::<UniformBufferFrameData>() as u32 * frame_index as u32;

        let descriptor_set = [self.global_set, self.frames_set];
        let dynamic_offsets = [dynamic_offset];

        unsafe {
            self.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                first_set,
                &descriptor_set,
                &dynamic_offsets,
            );
        }
    }

    pub fn new(
        device: Rc<ash::Device>,
        pd_properties: vk::PhysicalDeviceProperties,
        pd_memory_properties: vk::PhysicalDeviceMemoryProperties,
        descriptor_pool: vk::DescriptorPool,
    ) -> Self {
        //let gb_packed_align = Self::packed_range_from_min_align::<UniformBufferGlobalData>(min_ubuffer_offset_align); let fb_packed_align = Self::packed_range_from_min_align::<UniformBufferGlobalData>(min_ubuffer_offset_align);

        // With the minimum offset alignment we can define the offsets of the
        // the different subtypes
        let min_ubuffer_offset_align = pd_properties.limits.min_uniform_buffer_offset_alignment;

        let gb_packed_align =
            Self::packed_range_from_min_align::<UniformBufferGlobalData>(min_ubuffer_offset_align);
        let fb_packed_align =
            Self::packed_range_from_min_align::<UniformBufferFrameData>(min_ubuffer_offset_align);

        // The function create_buffer automatically aligns the memory upon creation
        // based on the given struct's size.
        let packed_size_bytes: u64 =
            gb_packed_align + fb_packed_align * config::MAX_FRAMES_COUNT as u64;
        let initial_data: Vec<u8> = (0..packed_size_bytes).map(|_| 0_u8).collect();

        let gb_create_info = AllocatedBufferCreateInfo {
            device: Rc::clone(&device),
            pd_memory_properties,
            initial_data: &initial_data, // size of allocated memory will be size of initial_data, the rest of the size is for dynamic mem
            buffer_usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
            memory_usage: MemoryUsage::CpuToGpu,
            memory_map_flags: vk::MemoryMapFlags::empty(),
            sharing_mode: vk::SharingMode::EXCLUSIVE,
        };

        let global_buffer = AllocatedBuffer::create_buffer(&gb_create_info);

        // descriptor layouts
        let global_layout = UniformBufferGlobalData::create_descriptor_set_layout(&device);
        let frames_layout = UniformBufferFrameData::create_descriptor_set_layout(&device);
        let descriptor_set_layouts = [global_layout, frames_layout];

        let descriptor_sets_allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&descriptor_set_layouts);

        // descriptor sets
        log::debug!("Allocating descriptor sets.");
        let allocated_descriptor_sets =
            unsafe { device.allocate_descriptor_sets(&descriptor_sets_allocate_info) }
                .expect("Couldn't allocate global descriptor set");

        // descriptor buffer infos
        let gb_packed_align =
            Self::packed_range_from_min_align::<UniformBufferGlobalData>(min_ubuffer_offset_align);

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
            Self::packed_range_from_min_align::<UniformBufferGlobalData>(min_ubuffer_offset_align);

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
            device.update_descriptor_sets(&write_sets, &[]);
        }

        Self {
            device: Rc::clone(&device),
            min_ubuffer_offset_align,
            global_layout,
            global_set: allocated_descriptor_sets[0],
            global_buffer,
            frames_layout,
            frames_set: allocated_descriptor_sets[1],
            //frames_buffer,
        }
    }
}

// struct UniformBufferBuilder<'a, T> {
//     device: Rc<ash::Device>,
//     pd_properties: vk::PhysicalDeviceProperties,
//     pd_memory_properties: vk::PhysicalDeviceMemoryProperties,
//     buffer_create_info: Option<AllocatedBufferCreateInfo<'a, T>>,
// }
//
// impl<'a> UniformBufferBuilder<'a, T> {
//     pub fn new(device: Rc<ash::Device>,
//            pd_properties: vk::PhysicalDeviceProperties,
//            pd_memory_properties: vk::PhysicalDeviceMemoryProperties,
//            //descriptor_pool: vk::DescriptorPool
//         ) -> Self {
//
//         Self {
//             device: Rc::clone(&device),
//             pd_properties,
//             pd_memory_properties,
//             buffer_create_info: None,
//         }
//     }
//
//     pub fn data<T: Copy>(mut self, data: &[T]) -> Self {
//         self.buffer_create_info = Some(AllocatedBufferCreateInfo {
//             device: Rc::clone(&self.device),
//             pd_memory_properties: self.pd_memory_properties,
//             initial_data: &data,
//             buffer_usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
//             memory_usage: MemoryUsage::CpuToGpu,
//             memory_map_flags: vk::MemoryMapFlags::empty(),
//             sharing_mode: vk::SharingMode::EXCLUSIVE,
//         });
//     }

// pub fn descriptor_set() {

//     // descriptor layouts
//     let global_layout = UniformBufferGlobalData::create_descriptor_set_layout(&device);
//     let frames_layout = UniformBufferFrameData::create_descriptor_set_layout(&device);
//     let descriptor_set_layouts = [global_layout, frames_layout];

//     let descriptor_sets_allocate_info = vk::DescriptorSetAllocateInfo::builder()
//         .descriptor_pool(descriptor_pool)
//         .set_layouts(&descriptor_set_layouts);

//     // descriptor sets
//     log::debug!("Allocating descriptor sets.");
//     let allocated_descriptor_sets =
//         unsafe { device.allocate_descriptor_sets(&descriptor_sets_allocate_info) }
//             .expect("Couldn't allocate global descriptor set");
// }
// }

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
        //let global_set_layout = Self::create_global_set_layout(&device);
        let global_set_layout = UniformBufferGlobalData::create_descriptor_set_layout(&device);

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

    fn binding_index() -> u32 {
        1_u32
    }

    fn create_descriptor_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
        let binding_index = Self::binding_index();
        descriptor_set_layout(
            device,
            binding_index,
            vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
            vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
        )
    }

    fn descriptor_set_binding_flags() -> vk::DescriptorPoolCreateFlags {
        DescriptorSetBindingFrequency::PerFrame.descriptor_flag_bits()
    }

    // fn create_global_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
    //     let layout_bindings = [vk::DescriptorSetLayoutBinding::builder()
    //         .binding(0)
    //         .descriptor_count(1)
    //         .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
    //         .stage_flags(vk::ShaderStageFlags::VERTEX)
    //         //.immutable_samplers()
    //         .build()];

    //     let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
    //         .bindings(&layout_bindings);

    //     Self::create_set_layout(device, &create_info)
    // }

    // fn create_set_layout(device: &ash::Device, create_info: &vk::DescriptorSetLayoutCreateInfoBuilder) -> vk::DescriptorSetLayout {
    //     unsafe { device.create_descriptor_set_layout(&create_info, None) }
    //         .expect("Couldn't create descriptor set layout")
    // }

    // fn descriptor_buffer(
    //     &self,
    //     device: Rc<ash::Device>,
    //     queue_index: u32,
    //     pd_memory_properties: vk::PhysicalDeviceMemoryProperties,
    //     descriptor_pool: vk::DescriptorPool,
    //     global_descriptor_set_layout: vk::DescriptorSetLayout,
    //     ) {

    //     let initial_data = [UniformBufferGlobalData::default()];

    //     let global_uniform_buffer = AllocatedBuffer::create_buffer_manually_writable(
    //         Rc::clone(&device),
    //         pd_memory_properties,
    //         &initial_data,
    //         vk::BufferUsageFlags::UNIFORM_BUFFER,
    //         MemoryUsage::CpuToGpu,
    //         Alignment::Auto,
    //     );

    //     // descriptor sets
    //     //
    //     let descriptor_set_layouts = [global_descriptor_set_layout];

    //     assert_eq!(descriptor_set_layouts.len(), 1);

    //     let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
    //         .descriptor_pool(descriptor_pool)
    //         //.descriptor_set_count(1)
    //         .set_layouts(&descriptor_set_layouts);

    //     log::debug!("Allocating a descriptor set.");
    //     let allocated_descriptor_sets =
    //         unsafe { device.allocate_descriptor_sets(&descriptor_set_allocate_info) }
    //             .expect("Couldn't allocate global descriptor set");

    //     log::debug!("Descriptor set count {}", allocated_descriptor_sets.len());

    //     // point descriptor set to buffer
    //     let buffer_info = [vk::DescriptorBufferInfo::builder()
    //         .buffer(global_uniform_buffer.handle)
    //         .offset(0)
    //         .range(std::mem::size_of::<UniformBufferGlobalData>() as u64)
    //         .build()];

    //     let set_write = [vk::WriteDescriptorSet::builder()
    //         // writing to binding 0
    //         .dst_binding(UniformBufferGlobalData::binding_index())
    //         .dst_set(allocated_descriptor_sets[0])
    //         .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
    //         .buffer_info(&buffer_info)
    //         .build()];

    //     unsafe {
    //         device.update_descriptor_sets(&set_write, &[]);
    //     }
    // }
}
