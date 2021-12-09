use ash::util::Align;
use ash::vk;
use crate::renderer::memory::{AllocatedImage, MemoryUsage, queries};
use crate::renderer::vk_types::VkContext;

impl AllocatedImage {
    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_image(self.image, None);
            device.free_memory(self.memory, None);
        }
    }

    pub fn create(
        context: &VkContext,
        image_create_info: &vk::ImageCreateInfoBuilder,
    ) -> Self {
        let depth_image =
            unsafe { context.device.create_image(image_create_info, None) }.expect("Couldn't create image");

        let image_memory_requirements =
            unsafe { context.device.get_image_memory_requirements(depth_image) };

        println!("mem type requirements: {:?}", image_memory_requirements);

        let image_memory_index = queries::find_memory_type_index(
            &image_memory_requirements,
            &context.pd_mem_properties(),
            MemoryUsage::GpuOnly.memory_property_flags(),
        )
            .expect("Couldn't find suitable memory index for image");

        let depth_image_allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(image_memory_requirements.size)
            .memory_type_index(image_memory_index);

        // allocate memory
        let memory = context.alloc_memory(&depth_image_allocate_info);

        context.bind_image_memory(depth_image, memory);

        Self {
            image: depth_image,
            memory,
            memory_size: image_memory_requirements.size,
        }
    }
}

