use ash::vk;
use crate::renderer::memory::{DeviceMemoryWriteInfo, MemoryUsage};
use crate::renderer::memory::device_memory::{DeviceMemory, DeviceMemoryCreateInfoFromBuffer};
use crate::renderer::vk_types::VkContext;



#[derive(Debug, Clone)]
pub struct AllocatedBuffer {
    pub handle: vk::Buffer,
    memory: DeviceMemory,
}
impl AllocatedBuffer {
    pub fn destroy(&mut self, context: &VkContext) {
        unsafe {
            context.device.destroy_buffer(self.handle, None);
            self.memory.destroy(context);
        }
    }
}


impl AllocatedBuffer {
    /// write to allocated gpu memory
    pub fn write_memory<T: Copy>(
        &self,
        context: &VkContext,
        write_memory_info: DeviceMemoryWriteInfo<T>) {

        self.memory.write_memory(context, write_memory_info);
    }
}



pub struct AllocatedBufferCreateInfo<'a, T> {
    pub initial_data: &'a [T],
    pub buffer_size: u64,
    pub buffer_usage: vk::BufferUsageFlags,
    pub memory_usage: MemoryUsage,

    pub sharing_mode: vk::SharingMode,
    pub memory_map_flags: vk::MemoryMapFlags,
}
impl<'a, T> Default for AllocatedBufferCreateInfo<'a, T> {
    fn default() -> Self {
        Self {
            initial_data: &[],
            buffer_size: 0,
            buffer_usage: vk::BufferUsageFlags::empty(),
            memory_usage: MemoryUsage::CpuMemGpuVisible,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            memory_map_flags: vk::MemoryMapFlags::empty(),
        }
    }
}



impl AllocatedBuffer {
    pub fn create_buffer<T: Copy>(context: &VkContext, create_info: AllocatedBufferCreateInfo<T>) -> Self {
        println!(
            "BUFFER SIZE {}",
            std::mem::size_of_val(create_info.initial_data) as u64
        );

        let buffer = context.create_buffer(vk::BufferCreateInfo::builder()
            .size(create_info.buffer_size)
            .usage(create_info.buffer_usage)
            .sharing_mode(create_info.sharing_mode)
        );

        let memory_requirements: vk::MemoryRequirements =
            unsafe { context.device.get_buffer_memory_requirements(buffer) };

        log::info!("MEMORY SIZE: {}", memory_requirements.size);

        // allocate device memory
        //
        let device_memory = DeviceMemory::new_from_buffer(
            context,
            DeviceMemoryCreateInfoFromBuffer {
                buffer,
                memory_usage: create_info.memory_usage,
                map_flags: create_info.memory_map_flags,
            }
        );

        if create_info.initial_data.len() > 0 {
            // memcpy
            device_memory.write_memory(
                &context,
                DeviceMemoryWriteInfo {
                    data: create_info.initial_data,
                    size: (std::mem::size_of::<T>() * create_info.initial_data.len()) as _,
                    offset: 0,
                    alignment: std::mem::align_of::<T>() as _,
                },
            );
        }

        // associate buffer with memory
        context.bind_buffer_memory(buffer, device_memory.handle);

        Self {
            handle: buffer,
            memory: device_memory,
        }
    }
}
