use super::resources::Vertex;
use ash::util::Align;
use ash::vk;
use std::rc::Rc;
use crate::engine::renderer::vk_context::VkContext;

struct MemoryMapping {
    bytes: u64,
    align: u64,
}

struct MemoryMappings {
    aligns: Vec<u64>, // one entry of byte count / mapping
}
impl MemoryMappings {}

trait DeviceMemoryOperations {
    // allocation
    fn allocate_memory_p(&self, allocate_info: &vk::MemoryAllocateInfo) -> vk::DeviceMemory;
    // binding
    fn bind_buffer_memory_p(&self, buffer: vk::Buffer, memory: vk::DeviceMemory);
    fn bind_image_memory_p(&self, image: vk::Image, memory: vk::DeviceMemory);
    // mapping
    fn map_memory_p(
        &self,
        memory: vk::DeviceMemory,
        memory_size: vk::DeviceSize,
        memory_map_flags: Option<vk::MemoryMapFlags>,
    ) -> *mut core::ffi::c_void;
    fn unmap_memory_p(&self, memory: vk::DeviceMemory);

    // updating
    fn copy_to_mapped<T: Copy>(
        &self,
        ptr_to_memory: *mut core::ffi::c_void,
        memory_size: vk::DeviceSize,
        new_data: &[T],
    );
    fn copy_to_mapped_image_memory_p<T: Copy>(
        &self,
        ptr_to_memory: *mut core::ffi::c_void,
        image: AllocatedImage,
        new_data: &[T],
    );
}
impl DeviceMemoryOperations for ash::Device {
    // Allocates gpu memory
    fn allocate_memory_p(&self, allocate_info: &vk::MemoryAllocateInfo) -> vk::DeviceMemory {
        unsafe { self.allocate_memory(allocate_info, None) }.expect("Couldn't allocate memory")
    }

    /// Associates a buffer handle with gpu memory
    fn bind_buffer_memory_p(&self, buffer: vk::Buffer, memory: vk::DeviceMemory) {
        unsafe {
            // associate memory with buffer
            self.bind_buffer_memory(buffer, memory, 0)
        }
        .expect("Couldn't bind memory buffer");
    }

    /// Associates an image handle with gpu memory
    fn bind_image_memory_p(&self, image: vk::Image, memory: vk::DeviceMemory) {
        unsafe {
            self.bind_image_memory(image, memory, 0);
        }
    }

    fn unmap_memory_p(&self, memory: vk::DeviceMemory) {
        unsafe {
            self.unmap_memory(memory);
        }
    }

    fn copy_to_mapped<T: Copy>(
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

    // Maps image memory
    fn map_memory_p(
        &self,
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
        let ptr_to_memory = unsafe { self.map_memory(memory, offset, memory_size, map_flags) }
            .expect("Couldn't get pointer to memory");

        ptr_to_memory
    }

    fn copy_to_mapped_image_memory_p<T: Copy>(
        &self,
        ptr_to_memory: *mut core::ffi::c_void,
        image: AllocatedImage,
        new_data: &[T],
    ) {
        // align makes it so we can copy a correctly aligned slice of &[T]
        // directly into memory without an extra allocation
        let mut memory_slice: Align<T> = unsafe {
            Align::new(
                ptr_to_memory,
                std::mem::align_of::<T>() as u64,
                image.memory_size,
            )
        };

        memory_slice.copy_from_slice(&new_data);
    }
}

pub mod prelude {
    pub use super::{AllocatedBuffer, AllocatedImage, MemoryUsage};
}

pub struct AllocatedImage {
    pub image: vk::Image,
    memory: vk::DeviceMemory,
    memory_size: vk::DeviceSize,
}
impl AllocatedImage {
    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_image(self.image, None);
            device.free_memory(self.memory, None);
        }
    }

    pub fn create(
        device: &ash::Device,
        pd_memory_properties: vk::PhysicalDeviceMemoryProperties,
        image_create_info: &vk::ImageCreateInfoBuilder,
    ) -> Self {
        let depth_image =
            unsafe { device.create_image(image_create_info, None) }.expect("Couldn't create image");

        let image_memory_requirements =
            unsafe { device.get_image_memory_requirements(depth_image) };

        println!("mem type requirements: {:?}", image_memory_requirements);

        let image_memory_index = find_memory_type_index(
            &image_memory_requirements,
            &pd_memory_properties,
            MemoryUsage::GpuOnly.memory_property_flags(),
        )
        .expect("Couldn't find suitable memory index for image");

        let depth_image_allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(image_memory_requirements.size)
            .memory_type_index(image_memory_index);

        // allocate memory
        let memory = device.allocate_memory_p(&depth_image_allocate_info);
        device.bind_image_memory_p(depth_image, memory);

        Self {
            image: depth_image,
            memory,
            memory_size: image_memory_requirements.size,
        }
    }
}

// ------------------------
pub enum MemoryUsage {
    CpuToGpu, // writable by CPU, readable by GPU
    GpuOnly,  // only accessible GPU-side
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

#[derive(Debug)]
pub struct AllocatedBuffer {
    pub handle: vk::Buffer, // handle to gpu-side buffer
    memory: vk::DeviceMemory,
    pub info: MemoryInfo, // info for the base allocation, dynamic allocations will need their own memoryinfo when binding
}


#[derive(Debug)]
/// The data required to map and write to some allocated memory
pub struct MemoryInfo {
    pub size: vk::DeviceSize,
    pub map_flags: vk::MemoryMapFlags,
    pub alignment: u64,
}

pub struct AllocatedBufferCreateInfo<'a, T> {
    pub device: &'a ash::Device,
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
//impl Drop for AllocatedBuffer {
//    fn drop(&mut self, context: &VkContext) {
//        unsafe {
//            context.device.destroy_buffer(self.handle, None);
//            context.device.free_memory(self.memory, None);
//        }
//    }
//}

impl AllocatedBuffer {
    pub fn destroy(&mut self, context: &VkContext) {
        unsafe {
            context.device.handle.destroy_buffer(self.handle, None);
            context.device.handle.free_memory(self.memory, None);
        }
    }

    // pub fn destroy(&self) {
    //     unsafe {
    //         self.device.destroy_buffer(self.handle, None);
    //         self.device.free_memory(self.memory, None);
    //     }
    // }

    pub fn write_memory<T: Copy>(&self, device: &ash::Device, data: &[T], offset: u64) {
        Self::write_memory_inner(device, self.memory, data, &self.info, offset);
    }

    pub fn create_buffer<'a, T: Copy>(create_info: &'a AllocatedBufferCreateInfo<T>) -> Self {
        let device = create_info.device;

        println!(
            "BUFFER SIZE {}",
            std::mem::size_of_val(create_info.initial_data) as u64
        );

        // let alignment = match create_info.alignment {
        //     Alignment::Auto => std::size_of_val(create_info.initial_data as u64),
        //         //std::mem::size_of::<T>() as u64,
        //     Alignment::Bytes(bytes) => bytes,
        // };

        let buffer_info = vk::BufferCreateInfo::builder()
            .size(std::mem::size_of_val(create_info.initial_data) as u64) // NOTE: Ensure it's not the size of the address
            .usage(create_info.buffer_usage)
            .sharing_mode(create_info.sharing_mode);

        let buffer = unsafe { device.create_buffer(&buffer_info, None) }
            .expect("Couldn't create index buffer");

        let memory_requirements: vk::MemoryRequirements =
            unsafe { device.get_buffer_memory_requirements(buffer) };

        log::info!("MEMORY SIZE: {}", memory_requirements.size);

        let mem_info = MemoryInfo {
            size: memory_requirements.size, // NOTE: How this is vk::DeviceSize, not u64
            map_flags: create_info.memory_map_flags,
            alignment: std::mem::size_of::<T>() as u64,
        };

        // allocate device memory
        //
        let memory_type_index = find_memory_type_index(
            &memory_requirements,
            &create_info.pd_memory_properties,
            create_info.memory_usage.memory_property_flags(),
        )
        .expect("Index buffer creation: Couldn't find a suitable memory type.");

        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_info.size)
            .memory_type_index(memory_type_index)
            .build();

        let memory = device.allocate_memory_p(&allocate_info);

        // memcpy
        Self::write_memory_inner(&device, memory, create_info.initial_data, &mem_info, 0);

        // associate buffer with memory
        device.bind_buffer_memory_p(buffer, memory);

        Self {
            handle: buffer,
            memory,
            info: mem_info,
        }
    }

    fn write_memory_inner<T: Copy>(
        device: &ash::Device,
        //buffer: vk::Buffer,
        memory: vk::DeviceMemory,
        data: &[T],
        mem_info: &MemoryInfo,
        offset: u64,
    ) {
        // map memory
        let ptr_to_memory =
            unsafe { device.map_memory(memory, offset, mem_info.size, mem_info.map_flags) }
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
        device: &ash::Device,
        vertices: &[Vertex],
        pd_memory_properties: vk::PhysicalDeviceMemoryProperties,
    ) -> AllocatedBuffer {
        let create_info = AllocatedBufferCreateInfo {
            device,
            pd_memory_properties,
            initial_data: vertices,
            buffer_usage: vk::BufferUsageFlags::VERTEX_BUFFER,
            memory_usage: MemoryUsage::CpuToGpu,
            memory_map_flags: vk::MemoryMapFlags::empty(),
            //size: Alignment::Auto,
            sharing_mode: vk::SharingMode::EXCLUSIVE, // will only be used from one queue, the graphics queue
        };
        Self::create_buffer(&create_info)
    }
}

//pub
/// GPUs have different types of memory to allocate from.
/// This function finds the right type of memory to use based on the provided
/// parameters.
/// https://github.com/MaikKlein/ash/blob/e10bbf3063d9b84b9d8c04e6e2baae7d4881cce4/examples/src/lib.rs#L120-L133
pub fn find_memory_type_index(
    memory_requirements: &vk::MemoryRequirements,
    memory_properties: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_properties.memory_types[..memory_properties.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_requirements.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}
