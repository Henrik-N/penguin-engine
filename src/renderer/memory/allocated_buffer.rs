use ash::util::Align;
use ash::vk;
use crate::renderer::memory::{AllocatedBuffer, AllocatedBufferCreateInfo, AllocatedMemoryInfo, MemoryUsage, queries};
use crate::renderer::vk_types::VkContext;
use crate::render_objects::Vertex;


impl AllocatedBuffer {
    pub fn destroy(&mut self, context: &VkContext) {
        unsafe {
            context.device.handle.destroy_buffer(self.handle, None);
            context.device.handle.free_memory(self.memory, None);
        }
    }

    pub fn create_buffer<T: Copy>(context: &VkContext, create_info: &AllocatedBufferCreateInfo<T>) -> Self {
        println!(
            "BUFFER SIZE {}",
            std::mem::size_of_val(create_info.initial_data) as u64
        );

        let buffer_info = vk::BufferCreateInfo::builder()
            .size(std::mem::size_of_val(create_info.initial_data) as u64) // NOTE: Ensure it's not the size of the address
            .usage(create_info.buffer_usage)
            .sharing_mode(create_info.sharing_mode);

        let buffer = unsafe { context.device.create_buffer(&buffer_info, None) }
            .expect("Couldn't create index buffer");

        let memory_requirements: vk::MemoryRequirements =
            unsafe { context.device.get_buffer_memory_requirements(buffer) };

        log::info!("MEMORY SIZE: {}", memory_requirements.size);

        let mem_info = AllocatedMemoryInfo {
            size: memory_requirements.size, // NOTE: How this is vk::DeviceSize, not u64
            map_flags: create_info.memory_map_flags,
            alignment: std::mem::size_of::<T>() as u64,
        };

        // allocate device memory
        //
        let memory_type_index = queries::find_memory_type_index(
            &memory_requirements,
            &create_info.pd_memory_properties,
            create_info.memory_usage.memory_property_flags(),
        )
            .expect("Index buffer creation: Couldn't find a suitable memory type.");

        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_info.size)
            .memory_type_index(memory_type_index)
            .build();

        let memory = context.alloc_memory(&allocate_info);

        // memcpy
        Self::write_memory_inner(&context.device, memory, create_info.initial_data, &mem_info, 0);

        // associate buffer with memory
        context.bind_buffer_memory(buffer, memory);

        Self {
            handle: buffer,
            memory,
            info: mem_info,
        }
    }

    pub fn write_memory<T: Copy>(&self, device: &ash::Device, data: &[T], offset: u64) {
        Self::write_memory_inner(device, self.memory, data, &self.info, offset);
    }

    fn write_memory_inner<T: Copy>(
        device: &ash::Device,
        //buffer: vk::Buffer,
        memory: vk::DeviceMemory,
        data: &[T],
        mem_info: &AllocatedMemoryInfo,
        offset: u64,
    ) {
        // map memory
        //let ptr_to_memory =
        //    unsafe { device.map_memory(memory, offset, mem_info.size, mem_info.map_flags) }
        //        .expect("Couldn't get pointer to memory");
        let ptr_to_memory =
            unsafe { device.map_memory(memory, offset, std::mem::size_of_val(&data) as u64, mem_info.map_flags) }
                .expect("Couldn't get pointer to memory");

        let size = std::mem::size_of_val(data) as u64;
        let alignment = std::mem::align_of::<T>() as u64;

        // align makes it so we can copy a correctly aligned slice of &[T]
        // directly into memory without an extra allocation
        let mut memory_slice: Align<T> = unsafe {
            Align::new(
                ptr_to_memory,
                alignment,
                size,
                // info.alignment,
                // std::mem::size_of_val(data) as u64
                //info.size
            )
        };

        memory_slice.copy_from_slice(data);

        unsafe {
            // unmap memory
            device.unmap_memory(memory);
        };
    }

    pub fn create_vertex_buffer(
        context: &VkContext,
        vertices: &[Vertex],
    ) -> AllocatedBuffer {
        let create_info = AllocatedBufferCreateInfo {
            pd_memory_properties: context.pd_mem_properties(),
            initial_data: vertices,
            buffer_usage: vk::BufferUsageFlags::VERTEX_BUFFER,
            memory_usage: MemoryUsage::CpuToGpu,
            memory_map_flags: vk::MemoryMapFlags::empty(),
            //size: Alignment::Auto,
            sharing_mode: vk::SharingMode::EXCLUSIVE, // will only be used from one queue, the graphics queue
        };
        Self::create_buffer(context, &create_info)
    }
}
