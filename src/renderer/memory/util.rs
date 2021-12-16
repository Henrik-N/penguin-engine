// https://stackoverflow.com/questions/28127165/how-to-convert-struct-to-u8

pub fn packed_range_from_min_align_manual(mem_range: u64, min_align: u64) -> u64 {
    let mut packed_range = 0;
    while packed_range < mem_range && packed_range < min_align {
        packed_range += min_align;
    }
    packed_range
}


pub fn packed_range_from_min_align<T>(min_align: u64) -> u64 {
    let mem_range = std::mem::size_of::<T>() as u64;
    let mut packed_range = 0;
    while packed_range < mem_range && packed_range < min_align {
        packed_range += min_align;
    }
    packed_range
}

#[allow(unused)]
pub unsafe fn as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    std::slice::from_raw_parts((p as *const T) as *const u8, std::mem::size_of::<T>())
}

use ash::vk;

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
