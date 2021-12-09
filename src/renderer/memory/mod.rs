mod allocated_buffer;
mod allocated_image;
mod allocating;
mod queries;

use allocating::*;


use ash::vk;
use crate::renderer::vk_types::*;


/// Memory
#[derive(PartialEq, Eq)]
pub enum MemType {
    Persistent,
    Dynamic,
}

// ------------------------
pub enum MemoryUsage {
    CpuToGpu, // writable by CPU, readable by GPU
    GpuOnly,  // only accessible GPU-side
}

#[derive(Debug)]
/// The data required to map and write to some allocated memory
pub struct AllocatedMemoryInfo {
    pub size: vk::DeviceSize,
    pub map_flags: vk::MemoryMapFlags,
    pub alignment: u64,
}


#[derive(Debug)]
pub struct AllocatedBuffer {
    pub handle: vk::Buffer, // handle to gpu-side buffer
    memory: vk::DeviceMemory,
    pub info: AllocatedMemoryInfo, // info for the base allocation, dynamic allocations will need their own memoryinfo when binding
}

pub struct AllocatedBufferCreateInfo<'a, T> {
    pub pd_memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub initial_data: &'a [T],
    pub buffer_usage: vk::BufferUsageFlags,
    pub memory_usage: MemoryUsage,
    //pub alignment: Alignment,

    //pub buffer_size: u64, //Alignment, // if auto, same as size_of_val(T)
    //pub base_alignment: Alignment, // alignment / Range for the const-size part of the buffer, not for the dynamic bindings. The dynamic bindings alignments are specified when binding the buffer
    pub sharing_mode: vk::SharingMode,
    pub memory_map_flags: vk::MemoryMapFlags,
}




pub struct AllocatedImage {
    pub image: vk::Image,
    memory: vk::DeviceMemory,
    memory_size: vk::DeviceSize,
}




impl MemoryUsage {
    /// Finds the best memory flags for the intended usage
    pub(crate) fn memory_property_flags(&self) -> vk::MemoryPropertyFlags {
        match self {
            MemoryUsage::CpuToGpu => {
                // writable from CPU
                vk::MemoryPropertyFlags::HOST_VISIBLE |
                    vk::MemoryPropertyFlags::HOST_COHERENT  // ensure mapped memory always match contents of allocated memory (no need for explicit flushing)
            }
            MemoryUsage::GpuOnly => vk::MemoryPropertyFlags::DEVICE_LOCAL,
            // ..
        }
    }
}





pub fn packed_range_from_min_align<T>(min_align: u64) -> u64 {
    let mem_range = std::mem::size_of::<T>() as u64;
    let mut packed_range = 0;
    while packed_range < mem_range && packed_range < min_align {
        packed_range += min_align;
    }
    packed_range
}




