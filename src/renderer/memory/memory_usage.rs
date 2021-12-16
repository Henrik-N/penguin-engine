use ash::vk;

// https://zeux.io/2020/02/27/writing-an-efficient-vulkan-renderer/


#[derive(Eq, PartialEq, Clone, Copy)]
pub enum MemoryUsage {
    /// GPU memory directly writable by from the CPU. For uniform & dynamic vertex/index and smaller data (generally up to 256MB).
    GpuMemCpuWritable,
    /// Only accessible GPU-side.
    GpuOnly,
    /// Reads over PCI-bus. use for staging buffers used to allocate static resources or if GpuMemCpuWritable is unsupported
    CpuMemGpuVisible,
    /// Save VRAM by lazily allocating memory for big targets that are never stored to, like MSAA & depth images.
    GpuOnlyLazy,
}

impl MemoryUsage {
    /// Finds the best memory flags for the intended usage
    pub fn memory_property_flags(&self) -> vk::MemoryPropertyFlags {
        match self {
            MemoryUsage::GpuMemCpuWritable => {
                vk::MemoryPropertyFlags::DEVICE_LOCAL |
                    vk::MemoryPropertyFlags::HOST_COHERENT  // ensure mapped memory always match contents of allocated memory (no need for explicit flushing)
            }
            MemoryUsage::GpuOnly => vk::MemoryPropertyFlags::DEVICE_LOCAL,
            MemoryUsage::CpuMemGpuVisible => vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            MemoryUsage::GpuOnlyLazy => vk::MemoryPropertyFlags::DEVICE_LOCAL | vk::MemoryPropertyFlags::LAZILY_ALLOCATED,
        }
    }
}
