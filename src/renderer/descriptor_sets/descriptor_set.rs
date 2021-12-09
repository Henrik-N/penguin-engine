use ash::vk;
use crate::renderer::memory;
use crate::renderer::memory::MemoryUsage;
use crate::renderer::vk_types::*;

// a bundle of all related allocations to a descriptor set
pub struct DescriptorSet {
    pub binding_index: u32,
    pub layout: vk::DescriptorSetLayout,
    pub set: vk::DescriptorSet,
    pub packed_range_single: vk::DeviceSize,
    pub data_multiplier: u32,
}

#[derive(Debug, Clone)]
pub struct DescriptorSetCreateInfo {
    pub binding_index: u32,
    pub stage_flags: vk::ShaderStageFlags,
    pub descriptor_pool: DescriptorPool,
    pub descriptor_variant: DescriptorVariant,
    pub data_multiplier: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum DescriptorVariant {
    UniformStatic(vk::ShaderStageFlags),
    UniformDynamic(vk::ShaderStageFlags),
    Storage,
    StorageDynamic,
}
impl DescriptorVariant {
    pub fn stage_flags(&self) -> vk::ShaderStageFlags {
        match self {
            DescriptorVariant::UniformStatic(flags) => *flags,
            DescriptorVariant::UniformDynamic(flags) => *flags,
            _ => vk::ShaderStageFlags::empty(),
        }
    }

    pub fn descriptor_type(&self) -> vk::DescriptorType {
        match self {
            DescriptorVariant::UniformStatic(_) => vk::DescriptorType::UNIFORM_BUFFER,
            DescriptorVariant::UniformDynamic(_) => vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
            DescriptorVariant::Storage => vk::DescriptorType::STORAGE_BUFFER,
            DescriptorVariant::StorageDynamic => vk::DescriptorType::STORAGE_BUFFER_DYNAMIC,
        }
    }
    pub fn buffer_usage(&self) -> vk::BufferUsageFlags {
        match self {
            DescriptorVariant::UniformStatic(_) => vk::BufferUsageFlags::UNIFORM_BUFFER,
            DescriptorVariant::UniformDynamic(_) => vk::BufferUsageFlags::UNIFORM_BUFFER,
            DescriptorVariant::Storage => vk::BufferUsageFlags::STORAGE_BUFFER,
            DescriptorVariant::StorageDynamic => vk::BufferUsageFlags::STORAGE_BUFFER,
        }
    }
    pub fn memory_usage(&self) -> MemoryUsage {
        match self {
            DescriptorVariant::UniformStatic(_) => MemoryUsage::CpuToGpu,
            DescriptorVariant::UniformDynamic(_) => MemoryUsage::CpuToGpu,
            _ => {log::error!("not yet implemented"); panic!();}
        }
    }
}




impl DescriptorSet {
    pub fn destroy(&mut self, context: &VkContext) {
        unsafe {
            context.device.handle.destroy_descriptor_set_layout(self.layout, None);
        }
    }

    // creates a descriptor set with the size of this type
    pub fn new_from_type<T>(context: &VkContext, create_info: DescriptorSetCreateInfo) -> Self {
        // With the minimum offset alignment we can define the offsets of the
        // the different subtypes
        let min_ubuffer_offset_align= context.min_uniform_buffer_offset_alignment();

        let packed_range_single = memory::packed_range_from_min_align::<T>(min_ubuffer_offset_align);

        println!("packed size: {}", packed_range_single);

        // create layout -----------
        let descriptor_layout: vk::DescriptorSetLayout = Self::create_layout(context, &create_info, 1);
        let descriptor_set: vk::DescriptorSet = context.alloc_descriptor_set(create_info.descriptor_pool, descriptor_layout);
        //allocate_descriptor_set(context, &create_info, descriptor_layout);

        Self {
            binding_index: create_info.binding_index,
            layout: descriptor_layout,
            set: descriptor_set,
            packed_range_single,
            data_multiplier: create_info.data_multiplier,
        }
    }


    fn create_layout(context: &VkContext, create_info: &DescriptorSetCreateInfo, descriptor_sets_count: u32) -> vk::DescriptorSetLayout {
        let layout_bindings = [vk::DescriptorSetLayoutBinding::builder()
            .binding(create_info.binding_index)
            .descriptor_count(descriptor_sets_count)
            .descriptor_type(create_info.descriptor_variant.descriptor_type())
            .stage_flags(create_info.stage_flags)
            //.immutable_samplers()
            .build()
        ];
        let layout_create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&layout_bindings);

        unsafe { context.device.handle
            .create_descriptor_set_layout(&layout_create_info, None) }
            .expect("Couldn't create descriptor set layout")
    }

}
