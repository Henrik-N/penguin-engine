use std::marker::PhantomData;
use ash::vk;
use crate::render_objects::Material;
use crate::renderer::descriptor_sets::{DescriptorSet, DescriptorSetCreateInfo, DescriptorVariant};
use crate::renderer::memory::{AllocatedBuffer, AllocatedBufferCreateInfo};
use crate::renderer::vk_types::{DescriptorPool, VkContext};

pub struct GpuBuffer<T: Copy> {
    pub allocated_buffer: AllocatedBuffer,
    pub descriptor_set: DescriptorSet,
    descriptor_variant: DescriptorVariant,
    data_type: PhantomData<T>,
}


pub struct GpuBufferBuilder<'a> {
    context: &'a VkContext,
    descriptor_pool: Option<DescriptorPool>,
    descriptor_variant: Option<DescriptorVariant>,
    data_multiplier: Option<u32>,
    binding_index: Option<u32>,
}



impl<T: Copy> GpuBuffer<T> {
    pub fn destroy(&mut self, context: &VkContext) {
        self.allocated_buffer.destroy(context);
        self.descriptor_set.destroy(context);
    }

    fn new(context: &VkContext, descriptor_pool: DescriptorPool, descriptor_variant: DescriptorVariant,
           descriptor_bundle: DescriptorSet) -> Self {

        let allocated_buffer = Self::allocate_buffer(
            context,
            descriptor_variant,
            descriptor_bundle.packed_range_single * (descriptor_bundle.data_multiplier) as u64
        );

        let global_descriptor_buffer_info = [
            vk::DescriptorBufferInfo::builder()
                .buffer(allocated_buffer.handle)
                .offset(0)
                .range(descriptor_bundle.packed_range_single * (descriptor_bundle.data_multiplier) as u64)
                .build()
        ];

        let write_sets = [
            vk::WriteDescriptorSet::builder()
                .descriptor_type(descriptor_variant.descriptor_type())
                .dst_set(descriptor_bundle.set)
                .dst_binding(descriptor_bundle.binding_index)
                .buffer_info(&global_descriptor_buffer_info)
                .build()
        ];

        unsafe {
            context.device.handle.update_descriptor_sets(&write_sets, &[]);
        }

        Self { allocated_buffer, descriptor_set: descriptor_bundle, descriptor_variant, data_type: PhantomData }
    }


    fn allocate_buffer(context: &VkContext, descriptor_variant: DescriptorVariant, packed_size: u64) -> AllocatedBuffer {
        let initial_data: Vec<u8> = (0..packed_size).map(|_| 0_u8).collect();
        let buffer_create_info = AllocatedBufferCreateInfo {
            pd_memory_properties: context.pd_mem_properties(),
            initial_data: &initial_data,
            buffer_usage: descriptor_variant.buffer_usage(),
            memory_usage: descriptor_variant.memory_usage(),
            memory_map_flags: vk::MemoryMapFlags::empty(),
            sharing_mode: vk::SharingMode::EXCLUSIVE,
        };
        AllocatedBuffer::create_buffer(context, &buffer_create_info)
    }

    pub fn get_descriptor_set_layout(&self) -> vk::DescriptorSetLayout {
        self.descriptor_set.layout
    }

    // offset_count is number of T items in, not bytes
    pub fn write_memory(&self, context: &VkContext, new_data: T, offset_count: u64) {
        let offset = std::mem::size_of::<T>() as u64 * offset_count;

        let buffer_data = [new_data];

        self.allocated_buffer.write_memory(&context.device.handle, &buffer_data, offset);
    }

    pub fn bind_descriptor_set(&self, context: &VkContext, command_buffer: vk::CommandBuffer,
                               material: &Material, dyn_instance_num: u32, first_set: u32) {

        //let first_set = 0; // self.descriptor_set.binding_index;
        let descriptor_sets = [self.descriptor_set.set];


        let dynamic_offsets = match self.descriptor_variant {
            DescriptorVariant::UniformStatic(_) => Vec::<u32>::with_capacity(0),
            DescriptorVariant::UniformDynamic(_) => vec![(std::mem::size_of::<T>() as u32) * dyn_instance_num],
            _ => panic!()
        };

        unsafe {
            context.device.handle.cmd_bind_descriptor_sets(
                command_buffer,
                material.pipeline.pipeline_bind_point,
                material.pipeline.pipeline_layout,
                first_set,
                &descriptor_sets,
                &dynamic_offsets,
            );
        }
    }
}




impl<'a> GpuBufferBuilder<'a> {
    const MISSING: &'static str = "GpuBufferBuilder not complete";

    pub fn builder(context: &'a VkContext) -> Self {
        Self {
            context,
            descriptor_pool: None,
            descriptor_variant: None,
            //descriptor_set: None,
            data_multiplier: None,
            binding_index: None,
        }
    }

    pub fn descriptor_pool(mut self, descriptor_pool: DescriptorPool) -> Self {
        self.descriptor_pool = Some(descriptor_pool);
        self
    }

    pub fn descriptor_variant(mut self, descriptor_variant: DescriptorVariant) -> Self {
        self.descriptor_variant = Some(descriptor_variant);
        self
    }

    pub fn binding_index(mut self, binding_index: u32) -> Self {
        self.binding_index = Some(binding_index);
        self
    }

    // std::mem::size_of::<T>() * data_multiplier will be allocated
    pub fn data_multiplier(mut self, count: usize) -> Self {
        self.data_multiplier = Some(count as u32);
        self
    }


    pub fn build<T: Copy>(mut self) -> GpuBuffer<T> {
        let descriptor_variant = self.descriptor_variant.expect(Self::MISSING);
        let descriptor_pool = self.descriptor_pool.expect(Self::MISSING);
        let binding_index = self.binding_index.expect(Self::MISSING);
        let data_multiplier = self.data_multiplier.expect(Self::MISSING);

        let global_uniform_buffer =
            DescriptorSet::new_from_type::<T>(
                &self.context,
                DescriptorSetCreateInfo {
                    binding_index,
                    stage_flags: descriptor_variant.stage_flags(),
                    descriptor_variant,
                    descriptor_pool,
                    data_multiplier,
                });

        GpuBuffer::new(self.context, descriptor_pool, descriptor_variant, global_uniform_buffer)
    }
}





