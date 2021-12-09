use ash::util::Align;
use ash::vk;
use crate::renderer::memory::AllocatedImage;
use crate::renderer::vk_types::{DescriptorPool, VkContext};


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
            self.device.bind_image_memory(image, memory, 0);
        }
    }

    pub fn map_memory(&self,
                      memory: vk::DeviceMemory,
                      memory_size: vk::DeviceSize,
                      memory_map_flags: Option<vk::MemoryMapFlags>,
    ) -> *mut core::ffi::c_void {

        let map_flags = match memory_map_flags {
            Some(flags) => flags,
            None => vk::MemoryMapFlags::empty(),
        };

        // map memory
        let offset = 0;
        let ptr_to_memory = unsafe { self.device.map_memory(memory, offset, memory_size, map_flags) }
            .expect("Couldn't get pointer to memory");

        ptr_to_memory
    }

    fn unmap_memory(&self, memory: vk::DeviceMemory) {
        unsafe {
            self.unmap_memory(memory);
        }
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



impl VkContext {
    pub fn alloc_descriptor_set(
        &self,
        descriptor_pool: DescriptorPool,
        layout: vk::DescriptorSetLayout
    ) -> vk::DescriptorSet {
        let descriptor_set_layouts = [layout];

        let descriptor_sets_allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool.into())
            .set_layouts(&descriptor_set_layouts);

        // descriptor sets
        log::debug!("Allocating descriptor sets.");
        let allocated_descriptor_sets =
            unsafe { self.device.allocate_descriptor_sets(&descriptor_sets_allocate_info) }
                .expect("Couldn't allocate global descriptor set");

        allocated_descriptor_sets[0]
    }

    pub fn alloc_command_pool_with_buffers(&self, buffer_count: u32) -> (vk::CommandPool, Vec<vk::CommandBuffer>) {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(self.physical_device.queue_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER); // allows resetting of individual buffers from this pool

        log::trace!("Creating command pool");
        let command_pool = unsafe { self.device.create_command_pool(&command_pool_create_info, None) }
            .expect("Command pool couldn't be created.");

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .command_buffer_count(buffer_count)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers =
            unsafe { self.device.allocate_command_buffers(&command_buffer_allocate_info) }
                .expect("Command buffer couldn't be created");


        (command_pool, command_buffers)
    }
}
