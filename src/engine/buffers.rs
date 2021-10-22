use super::Vertex;
use ash::util::Align;
use ash::vk;
use std::rc::Rc;

// ------------------------
pub struct AllocatedBuffer {
    device: Rc<ash::Device>,
    pub buffer_handle: vk::Buffer, // handle to gpu-side buffer
    buffer_memory: vk::DeviceMemory,
}
impl Drop for AllocatedBuffer {
  fn drop(&mut self) {
      unsafe {
          self.device.destroy_buffer(self.buffer_handle, None);
          self.device.free_memory(self.buffer_memory, None);
      }
  }
}
impl AllocatedBuffer {
    pub fn new_vertex_buffer(
        device: Rc<ash::Device>,
        physical_device_memory_properties: vk::PhysicalDeviceMemoryProperties,
        vertices: &[Vertex],
    ) -> AllocatedBuffer {
        let buffer_info = vk::BufferCreateInfo {
            size: std::mem::size_of_val(&vertices) as u64,
            usage: vk::BufferUsageFlags::VERTEX_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE, // will only be used from one queue, the graphics queue
            ..Default::default()
        };

        // create buffer and get buffer handle
        let buffer =
            unsafe { device.create_buffer(&buffer_info, None) }.expect("Couldn't create buffer");

        // prerequisites
        let memory_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

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

        let offset = 0;

        // map memory
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

        AllocatedBuffer {
            device,
            buffer_handle: buffer,
            buffer_memory: allocated_memory,
        }
    }
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
