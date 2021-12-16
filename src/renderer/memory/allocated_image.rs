use ash::vk;
use crate::renderer::memory;
use crate::renderer::memory::MemoryUsage;
use crate::renderer::memory::device_memory::DeviceMemory;
use crate::renderer::vk_types::VkContext;


pub struct AllocatedImage {
    pub handle: vk::Image,
    memory: DeviceMemory,
}


impl AllocatedImage {
    pub fn destroy(&mut self, context: &VkContext) {
        unsafe {
            context.device.destroy_image(self.handle, None);
        }
        self.memory.destroy(context);
    }

    pub fn create(
        context: &VkContext,
        image_create_info: vk::ImageCreateInfoBuilder,
    ) -> Self {
        let image =
            unsafe { context.device.create_image(&image_create_info, None) }.expect("Couldn't create image");

        let image_memory_requirements =
            unsafe { context.device.get_image_memory_requirements(image) };

        log::info!("image memory requirements: {:?}", image_memory_requirements);

        let image_memory_index = memory::util::find_memory_type_index(
            &image_memory_requirements,
            &context.pd_mem_properties(),
            MemoryUsage::GpuOnly.memory_property_flags(),
        ).expect("Couldn't find suitable memory index for image");

        let depth_image_allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(image_memory_requirements.size)
            .memory_type_index(image_memory_index);

        // allocate memory
        let memory = context.alloc_memory(&depth_image_allocate_info);

        context.bind_image_memory(image, memory);

        Self {
            handle: image,
            memory: DeviceMemory {
                handle: memory,
                size: image_memory_requirements.size,
                map_flags: vk::MemoryMapFlags::empty(),
            },
        }
    }
}

