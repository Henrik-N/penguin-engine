use ash::util::Align;
use ash::vk;
use crate::renderer::memory;
use crate::renderer::vk_types::VkContext;


impl VkContext {
    pub fn copy_buffer(&self, command_buffer: vk::CommandBuffer, src: vk::Buffer, dst: vk::Buffer, size: usize) {
        let copy = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: size as _,
        };
        let regions = [copy];

        unsafe {
            self.device.cmd_copy_buffer(
                command_buffer,
                src,
                dst,
                &regions,
            )
        }
    }
}


impl VkContext {
    pub fn packed_uniform_buffer_range<T>(&self) -> u64 {
        let min_offset_align = self.min_uniform_buffer_offset_alignment();
        let packed_range = memory::util::packed_range_from_min_align::<T>(min_offset_align);
        packed_range
    }
}

impl VkContext {
    pub fn pd_mem_properties(&self) -> vk::PhysicalDeviceMemoryProperties {
        unsafe {
            self.instance
                .get_physical_device_memory_properties(self.physical_device.handle)
        }
    }

    #[allow(unused)]
    pub fn min_uniform_buffer_offset_alignment(&self) -> vk::DeviceSize {
        self.pd_device_properties()
            .limits
            .min_uniform_buffer_offset_alignment
    }
}


impl VkContext {
    pub fn create_buffer(&self, create_info: vk::BufferCreateInfoBuilder) -> vk::Buffer {
        unsafe { self.device.create_buffer(&create_info, None) }
            .expect("Couldn't create index buffer")
    }
}


impl VkContext {
    /// Allocates gpu memory
    pub fn alloc_memory(&self, alloc_info: &vk::MemoryAllocateInfo) -> vk::DeviceMemory {
        unsafe { self.device.allocate_memory(alloc_info, None) }.expect("Couldn't allocate memory")
    }

    /// Associates a buffer handle with gpu memory
    pub fn bind_buffer_memory(&self, buffer: vk::Buffer, memory: vk::DeviceMemory) {
        unsafe {
            self.device.bind_buffer_memory(buffer, memory, 0)
        }.expect("Couldn't bind memory buffer");
    }

    /// Associates an image handle with gpu memory
    pub fn bind_image_memory(&self, image: vk::Image, memory: vk::DeviceMemory) {
        unsafe {
            self.device.bind_image_memory(image, memory, 0)
        }.expect("couldn't bind image memory");
    }

    pub fn map_memory(&self,
                      memory: vk::DeviceMemory,
                      offset: vk::DeviceSize,
                      memory_size: vk::DeviceSize,
                      map_flags: vk::MemoryMapFlags,
    ) -> *mut core::ffi::c_void {
        // map memory
        let ptr_to_memory = unsafe { self.device.map_memory(memory, offset, memory_size, map_flags) }
            .expect("Couldn't get pointer to memory");

        ptr_to_memory
    }

    pub fn unmap_memory(&self, memory: vk::DeviceMemory) {
        unsafe { self.device.unmap_memory(memory) };
    }

    // updating
    pub fn copy_to_mapped_memory<T: Copy>(
        &self,
        ptr_to_memory: *mut core::ffi::c_void,
        memory_size: vk::DeviceSize,
        new_data: &[T],
    ) {
        // align makes it so we can copy a correctly aligned slice of &[T]
        // directly into memory without an extra allocation
        let mut memory_slice: Align<T> =
            unsafe { Align::new(ptr_to_memory, std::mem::align_of::<T>() as u64, memory_size) };

        memory_slice.copy_from_slice(&new_data);
    }
}




//impl VkContext {
//fn allocate_vk_buffer(
//    &self,
//    buffer_info: vk::BufferCreateInfoBuilder) -> vk::Buffer {
//    unsafe { self.device.create_buffer(&buffer_info, None) }
//        .expect("Couldn't create buffer")
//}

//fn allocate_vk_device_memory(
//    &self,
//    buffer: vk::Buffer,
//    memory_usage: &MemoryUsage,
//    memory_map_flags: vk::MemoryMapFlags
//)
//    -> (vk::DeviceMemory, AllocatedMemoryInfo) {

//    let memory_requirements: vk::MemoryRequirements =
//        unsafe { self.device.get_buffer_memory_requirements(buffer) };

//    log::info!("MEMORY SIZE: {}", memory_requirements.size);

//    let mem_info = AllocatedMemoryInfo {
//        size: memory_requirements.size, // NOTE: How this is vk::DeviceSize, not u64
//        map_flags: memory_map_flags,
//        //alignment: std::mem::size_of::<T>() as u64,
//    };

//    // allocate device memory
//    //
//    let memory_type_index = queries::find_memory_type_index(
//        &memory_requirements,
//        &self.pd_mem_properties(),
//        memory_usage.memory_property_flags(),
//    )
//        .expect("Index buffer creation: Couldn't find a suitable memory type.");

//    let allocate_info = vk::MemoryAllocateInfo::builder()
//        .allocation_size(mem_info.size)
//        .memory_type_index(memory_type_index)
//        .build();

//    (self.alloc_memory(&allocate_info), mem_info)
//}
//}

