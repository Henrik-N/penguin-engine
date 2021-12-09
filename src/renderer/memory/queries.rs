use ash::vk;
use crate::renderer::vk_types::VkContext;

impl VkContext {
    pub fn pd_mem_properties(&self) -> vk::PhysicalDeviceMemoryProperties {
        unsafe {
            self.instance
                .handle
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

//pub
/// GPUs have different types of memory to allocate from.
/// This function finds the right type of memory to use based on the provided
/// parameters.
/// https://github.com/MaikKlein/ash/blob/e10bbf3063d9b84b9d8c04e6e2baae7d4881cce4/examples/src/lib.rs#L120-L133
pub(super) fn find_memory_type_index(
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

