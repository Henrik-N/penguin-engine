use ash::vk;
use crate::renderer::vk_types::VkContext;


#[derive(Debug, Copy, Clone, Default)]
pub struct DescriptorPool {
    pub handle: vk::DescriptorPool,
}
impl std::ops::Deref for DescriptorPool {
    type Target = vk::DescriptorPool;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}
impl From<DescriptorPool> for vk::DescriptorPool {
    fn from(p: DescriptorPool) -> Self {
        p.handle
    }
}

impl DescriptorPool {
    const MAX_UNIFORM_BUFFER_COUNT: u32 = 10;
    const MAX_DESCRIPTOR_SET_COUNT: u32 = 10;

    pub fn from_sizes(context: &VkContext, max_descriptor_sets: u32, pool_sizes: &[vk::DescriptorPoolSize])
                      -> Self {
        log::trace!("Creating descriptor pool.");

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(max_descriptor_sets)
            .pool_sizes(pool_sizes);

        let descriptor_pool =
            unsafe { context.device.create_descriptor_pool(&descriptor_pool_create_info, None) }
                .expect("Couldn't create descriptor pool");

        Self { handle: descriptor_pool }
    }

    pub fn create_pool(device: &ash::Device) -> Self {
        log::trace!("Creating descriptor pool.");

        let descriptor_pool_size = [vk::DescriptorPoolSize::builder()
            .descriptor_count(Self::MAX_UNIFORM_BUFFER_COUNT) // 10 uniform buffers
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .build()];

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(Self::MAX_DESCRIPTOR_SET_COUNT)
            .pool_sizes(&descriptor_pool_size);

        let descriptor_pool =
            unsafe { device.create_descriptor_pool(&descriptor_pool_create_info, None) }
                .expect("Couldn't create descriptor pool");

        Self {
            handle: descriptor_pool,
        }
    }
}

impl DescriptorPool {
    pub fn destroy(&mut self, context: &VkContext) {
        unsafe { context.device.destroy_descriptor_pool(self.handle, None); }
    }

    pub fn init(context: &VkContext) -> Self {
        let descriptor_pool_size = [
            vk::DescriptorPoolSize::builder()
                // reserve 1 handle
                .descriptor_count(10) // 10 uniform buffers
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .build(),
            vk::DescriptorPoolSize::builder()
                .descriptor_count(10) // 10 dynamic uniform buffers
                .ty(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                .build(),
            vk::DescriptorPoolSize::builder()
                .descriptor_count(10)
                .ty(vk::DescriptorType::STORAGE_BUFFER_DYNAMIC)
                .build(),
        ];

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(10 as u32)
            .pool_sizes(&descriptor_pool_size);

        let descriptor_pool = unsafe {
            context
                .device
                .handle
                .create_descriptor_pool(&descriptor_pool_create_info, None)
        }
            .expect("Couldn't create descriptor pool");

        Self {
            handle: descriptor_pool,
        }
    }
}
