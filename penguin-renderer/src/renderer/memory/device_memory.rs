use crate::renderer::memory;
use crate::renderer::memory::MemoryUsage;
use crate::renderer::vk_types::VkContext;
use ash::util::Align;
use ash::vk;

#[derive(Debug, Clone)]
pub struct DeviceMemory {
    pub handle: vk::DeviceMemory,
    pub size: vk::DeviceSize,
    pub map_flags: vk::MemoryMapFlags, // flags to use when mapping the memory
}
impl std::ops::Deref for DeviceMemory {
    type Target = vk::DeviceMemory;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}
impl DeviceMemory {
    pub fn destroy(&mut self, context: &VkContext) {
        unsafe {
            context.device.free_memory(self.handle, None);
        }
    }
}

pub struct DeviceMemoryCreateInfoFromBuffer {
    pub buffer: vk::Buffer,
    pub memory_usage: MemoryUsage,
    pub map_flags: vk::MemoryMapFlags,
}
impl DeviceMemory {
    pub fn new_from_buffer(
        context: &VkContext,
        create_info: DeviceMemoryCreateInfoFromBuffer,
    ) -> Self {
        let memory_requirements: vk::MemoryRequirements = unsafe {
            context
                .device
                .get_buffer_memory_requirements(create_info.buffer)
        };

        log::info!("MEMORY SIZE: {}", memory_requirements.size);

        // allocate device memory
        //
        let memory_type_index = memory::util::find_memory_type_index(
            &memory_requirements,
            &context.pd_mem_properties(),
            create_info.memory_usage.memory_property_flags(),
        )
        .expect("Index buffer creation: Couldn't find a suitable memory type.");

        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type_index)
            .build();

        let memory = context.alloc_memory(&allocate_info);

        Self {
            handle: memory,
            size: memory_requirements.size,
            map_flags: create_info.map_flags,
        }
    }
}

pub struct DeviceMemoryWriteInfo<'a, T: Copy> {
    pub data: &'a [T],
    pub size: u64,
    pub offset: u64,
    pub alignment: u64,
}
impl DeviceMemory {
    pub fn write_memory<T: Copy>(&self, context: &VkContext, write_info: DeviceMemoryWriteInfo<T>) {
        // todo: Is this memory alignment only necessary when mapping uniform buffers?
        //let min_offset_align = context.min_uniform_buffer_offset_alignment();
        //let size = crate::renderer::memory::util::packed_range_from_min_align_manual(
        //    std::mem::size_of_val(write_info.data) as _,
        //    min_offset_align);

        // map memory
        let ptr_to_memory =
            //context.map_memory(self.handle, write_info.offset, size, self.map_flags);
        context.map_memory(self.handle, write_info.offset, write_info.size, self.map_flags);
        //context.map_memory(self.handle, write_info.offset, self.size, self.map_flags);

        // align makes it so we can copy a correctly aligned slice of &[T]
        // directly into memory without an extra allocation
        let mut memory_slice: Align<T> = unsafe {
            Align::new(
                ptr_to_memory,
                write_info.alignment,
                write_info.size,
                //size,
                //self.size,
            )
        };

        memory_slice.copy_from_slice(write_info.data);

        context.unmap_memory(self.handle);
    }
}
