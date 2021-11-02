use super::resources::{UniformBufferGlobalData, Vertex};
use ash::util::Align;
use ash::vk;
use std::rc::Rc;

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
        let depth_image = unsafe { device.create_image(&image_create_info, None) }
            .expect("Couldn't create image");

        let image_memory_requirements =
            unsafe { device.get_image_memory_requirements(depth_image) };

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
    fn memory_property_flags(&self) -> vk::MemoryPropertyFlags {
        match self {
            MemoryUsage::CpuToGpu => {
                // writable from CPU
                vk::MemoryPropertyFlags::HOST_VISIBLE | 
              // ensure mapped memory always match contents of allocated memory (no need for explicit flushing)
              vk::MemoryPropertyFlags::HOST_COHERENT
            }
            MemoryUsage::GpuOnly => vk::MemoryPropertyFlags::DEVICE_LOCAL,
            // ..
        }
    }
}

/// Information required to update a specific buffer,
/// cached
#[derive(Clone, Copy)]
struct CachedUpdateMemoryInfo {
    memory_requirements: vk::MemoryRequirements,
    memory_map_flags: vk::MemoryMapFlags,
}

pub struct AllocatedBuffer {
    device: Rc<ash::Device>,
    pub handle: vk::Buffer, // handle to gpu-side buffer
    memory: vk::DeviceMemory,
    memory_size: vk::DeviceSize,

    // information required to update the buffer
    update_memory_info: Option<CachedUpdateMemoryInfo>,
}
impl Drop for AllocatedBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_buffer(self.handle, None);
            self.device.free_memory(self.memory, None);
        }
    }
}
impl AllocatedBuffer {
    // pub fn destroy(&self) {
    //     unsafe {
    //         self.device.destroy_buffer(self.handle, None);
    //         self.device.free_memory(self.memory, None);
    //     }
    // }


    pub fn update_memory<DataType: Copy>(&self, data: &[DataType]) {
        Self::update_memory_inner(
            &self.device,
            self.handle,
            self.memory,
            &self
                .update_memory_info
                .expect("Tried to update allocated memory that is not defined as updateable"),
            data,
        );
    }

    fn update_memory_inner<DataType: Copy>(
        device: &ash::Device,
        buffer: vk::Buffer,
        memory: vk::DeviceMemory,
        memory_info: &CachedUpdateMemoryInfo,
        data: &[DataType],
    ) {
        // map memory
        let offset = 0;
        let ptr_to_memory = unsafe {
            device.map_memory(
                memory,
                offset,
                memory_info.memory_requirements.size,
                memory_info.memory_map_flags,
            )
        }
        .expect("Couldn't get pointer to memory");

        // align makes it so we can copy a correctly aligned slice of &[DataType]
        // directly into memory without an extra allocation
        let mut memory_slice: Align<DataType> = unsafe {
            Align::new(
                ptr_to_memory,
                std::mem::align_of::<DataType>() as u64,
                memory_info.memory_requirements.size,
            )
        };

        memory_slice.copy_from_slice(&data);

        unsafe {
            // unmap memory
            device.unmap_memory(memory);
        };
    }

    pub fn create_empty_unbound_buffer(
        device: Rc<ash::Device>,
        alloc_size: u64,
        pd_memory_properties: vk::PhysicalDeviceMemoryProperties,
        buffer_usage: vk::BufferUsageFlags,
        memory_usage: MemoryUsage,
    ) -> AllocatedBuffer {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(alloc_size)
            .usage(buffer_usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE); // TODO: is this the best?

        let buffer =
            unsafe { device.create_buffer(&buffer_info, None) }.expect("Couldn't create buffer");

        let memory_requirements: vk::MemoryRequirements =
            unsafe { device.get_buffer_memory_requirements(buffer) };

        let memory_type_index = find_memory_type_index(
            &memory_requirements,
            &pd_memory_properties,
            memory_usage.memory_property_flags(),
        )
        .expect("Index buffer creation: Couldn't find a suitable memory type.");

        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type_index)
            .build();

        let memory = device.allocate_memory_p(&allocate_info);

        Self {
            device: Rc::clone(&device),
            handle: buffer,
            memory,
            update_memory_info: None,
            memory_size: memory_requirements.size,
        }
    }

    pub fn create_buffer<T: Copy>(
        device: Rc<ash::Device>,
        alloc_size: u64,
        data: &[T],
        pd_memory_properties: vk::PhysicalDeviceMemoryProperties,
        buffer_usage: vk::BufferUsageFlags,
        memory_usage: MemoryUsage,
    ) -> Self {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(alloc_size)
            .usage(buffer_usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE); // TODO: is this the best?

        let buffer =
            unsafe { device.create_buffer(&buffer_info, None) }.expect("Couldn't create buffer");

        let memory_requirements: vk::MemoryRequirements =
            unsafe { device.get_buffer_memory_requirements(buffer) };

        let memory_type_index = find_memory_type_index(
            &memory_requirements,
            &pd_memory_properties,
            memory_usage.memory_property_flags(),
        )
        .expect("Index buffer creation: Couldn't find a suitable memory type.");

        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type_index)
            .build();

        let memory = device.allocate_memory_p(&allocate_info);

        let memory_map_flags = None;
        let ptr_to_memory = device.map_memory_p(memory, memory_requirements.size, memory_map_flags);

        device.copy_to_mapped(ptr_to_memory, memory_requirements.size, data);

        device.unmap_memory_p(memory);

        device.bind_buffer_memory_p(buffer, memory);

        Self {
            device: Rc::clone(&device),
            handle: buffer,
            memory,
            update_memory_info: None,
            memory_size: memory_requirements.size,
        }
    }

    pub fn create_buffer_updateable<DataType: Copy>(
        device: Rc<ash::Device>,
        pd_memory_properties: vk::PhysicalDeviceMemoryProperties,
        initial_data: &[DataType],
        buffer_usage: vk::BufferUsageFlags,
        memory_usage: MemoryUsage,
    ) -> Self {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(std::mem::size_of_val(initial_data) as u64)
            .usage(buffer_usage); //.sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { device.create_buffer(&buffer_info, None) }
            .expect("Couldn't create index buffer");

        let memory_requirements: vk::MemoryRequirements =
            unsafe { device.get_buffer_memory_requirements(buffer) };

        let memory_type_index = find_memory_type_index(
            &memory_requirements,
            &pd_memory_properties,
            memory_usage.memory_property_flags(),
        )
        .expect("Index buffer creation: Couldn't find a suitable memory type.");

        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type_index)
            .build();

        let memory_map_flags = vk::MemoryMapFlags::empty();

        let memory = device.allocate_memory_p(&allocate_info);

        let update_memory_info = CachedUpdateMemoryInfo {
            memory_requirements,
            memory_map_flags,
        };

        Self::update_memory_inner(&device, buffer, memory, &update_memory_info, &initial_data);

        device.bind_buffer_memory_p(buffer, memory);

        Self {
            device: Rc::clone(&device),
            handle: buffer,
            memory,
            update_memory_info: Some(update_memory_info),
            memory_size: memory_requirements.size,
        }
    }

    fn bind_memory(device: &ash::Device, buffer: vk::Buffer, memory: vk::DeviceMemory) {
        unsafe {
            // associate memory with buffer
            device.bind_buffer_memory(buffer, memory, 0)
        }
        .expect("Couldn't bind memory buffer");
    }

    pub fn new_vertex_buffer(
        device: Rc<ash::Device>,
        physical_device_memory_properties: vk::PhysicalDeviceMemoryProperties,
        vertices: &Vec<Vertex>,
    ) -> Self {
        let buffer = Self::create_vertex_buffer(&device, &vertices);

        let memory_requirements: vk::MemoryRequirements =
            unsafe { device.get_buffer_memory_requirements(buffer) };

        let allocated_memory = Self::allocate_vertex_buffer_memory(
            &device,
            buffer,
            &vertices,
            memory_requirements,
            physical_device_memory_properties,
        );

        Self {
            device: Rc::clone(&device),
            handle: buffer,
            memory: allocated_memory,
            update_memory_info: None,
            memory_size: memory_requirements.size,
        }
    }

    fn allocate_index_buffer_memory(
        device: &ash::Device,
        index_buffer: vk::Buffer,
        indices: &[u16],
        memory_requirements: vk::MemoryRequirements,
        physical_device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    ) -> vk::DeviceMemory {
        let memory_property_flags = 
              // writable from CPU
              vk::MemoryPropertyFlags::HOST_VISIBLE | 
              // ensure mapped memory always match contents of allocated memory (no need for explicit flushing)
              vk::MemoryPropertyFlags::HOST_COHERENT;

        let memory_type_index = find_memory_type_index(
            &memory_requirements,
            &physical_device_memory_properties,
            memory_property_flags,
        )
        .expect("Index buffer creation: Couldn't find a suitable memory type.");

        let allocate_info = vk::MemoryAllocateInfo {
            allocation_size: memory_requirements.size,
            memory_type_index,
            ..Default::default()
        };

        // allocate device memory
        let allocated_memory = unsafe { device.allocate_memory(&allocate_info, None) }
            .expect("Couldn't allocate memory");

        // map memory
        let offset = 0;
        let ptr_to_memory = unsafe {
            device.map_memory(
                allocated_memory,
                offset,
                memory_requirements.size,
                vk::MemoryMapFlags::empty(),
            )
        }
        .expect("Couldn't get pointer to memory");

        // align makes it so we can copy a correctly aligned slice of &[u32]
        //    directly into memory without an extra allocation
        let mut memory_slice: Align<u16> = unsafe {
            Align::new(
                ptr_to_memory,
                std::mem::align_of::<u16>() as u64,
                memory_requirements.size,
            )
        };

        memory_slice.copy_from_slice(indices);

        unsafe {
            // unmap memory
            device.unmap_memory(allocated_memory);

            // associate memory with buffer
            device
                .bind_buffer_memory(index_buffer, allocated_memory, 0)
                .expect("Couldn't bind memory buffer")
        };

        allocated_memory
    }



    fn create_index_buffer(device: &ash::Device, indices: &[u16]) -> vk::Buffer {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(std::mem::size_of_val(indices) as u64)
            .usage(vk::BufferUsageFlags::INDEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        unsafe { device.create_buffer(&buffer_info, None) }.expect("Couldn't create index buffer")
    }


    fn create_vertex_buffer(
        device: &ash::Device,
        vertices: &[Vertex],
    ) -> vk::Buffer {
        let buffer_info = vk::BufferCreateInfo {
            size: std::mem::size_of_val(vertices) as u64, // NOTE: The 3 here
            usage: vk::BufferUsageFlags::VERTEX_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE, // will only be used from one queue, the graphics queue
            ..Default::default()
        };

        unsafe { device.create_buffer(&buffer_info, None) }.expect("Couldn't create vertex buffer")
    }

    fn allocate_vertex_buffer_memory(
        device: &ash::Device,
        buffer: vk::Buffer,
        vertices: &Vec<Vertex>,
        memory_requirements: vk::MemoryRequirements,
        physical_device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    ) -> vk::DeviceMemory {
        let memory_property_flags = 
                // writable from CPU
                vk::MemoryPropertyFlags::HOST_VISIBLE | 
                // ensure mapped memory always match contents of allocated memory (no need for explicit flushing)
                vk::MemoryPropertyFlags::HOST_COHERENT;

        let memory_type_index = find_memory_type_index(
            &memory_requirements,
            &physical_device_memory_properties,
            memory_property_flags,
        )
        .expect("Vertex buffer creation: Couldn't find a suitable memory type.");

        let allocate_info = vk::MemoryAllocateInfo {
            allocation_size: memory_requirements.size,
            memory_type_index,
            ..Default::default()
        };

        // allocate device memory
        let allocated_memory = unsafe { device.allocate_memory(&allocate_info, None) }
            .expect("Couldn't allocate memory");

        // map memory
        let offset = 0;
        let ptr_to_memory = unsafe {
            device.map_memory(
                allocated_memory,
                offset,
                memory_requirements.size,
                vk::MemoryMapFlags::empty(),
            )
        }
        .expect("Couldn't get pointer to memory");

        // align makes it so we can copy a correctly aligned slice of &[Vertex]
        // directly into memory without an extra allocation
        let mut memory_slice: Align<Vertex> = unsafe {
            Align::new(
                ptr_to_memory,
                std::mem::align_of::<Vertex>() as u64,
                memory_requirements.size,
            )
        };

        memory_slice.copy_from_slice(&vertices);

        unsafe {
            // unmap memory
            device.unmap_memory(allocated_memory);

            // associate memory with buffer
            device
                .bind_buffer_memory(buffer, allocated_memory, 0)
                .expect("Couldn't bind memory buffer")
        };

        allocated_memory
    }

    // pub fn new_vertex_buffer_old(
    //     device: Rc<ash::Device>,
    //     physical_device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    //     vertices: &[Vertex],
    // ) -> Self {
    //     // create buffer and get buffer handle
    //     let buffer = Self::create_vertex_buffer(&device, &vertices);

    //     // prerequisites
    //     let memory_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    //     let allocated_memory = Self::allocate_vertex_buffer_memory(
    //         &device,
    //         &vertices,
    //         memory_requirements,
    //         physical_device_memory_properties);

    //     Self {
    //         device: Rc::clone(&device),
    //         handle: buffer,
    //         memory: allocated_memory,
    //     }
    // }
}

//pub
/// GPUs have different types of memory to allocate from.
/// This function finds the right type of memory to use based on the provided
/// parameters.
/// https://github.com/MaikKlein/ash/blob/e10bbf3063d9b84b9d8c04e6e2baae7d4881cce4/examples/src/lib.rs#L120-L133
fn find_memory_type_index(
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
