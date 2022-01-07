use crate::renderer::vk_types::{
DescriptorPool, DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutBuilder, VkContext,
};
use anyhow::*;
use ash::prelude::VkResult;
use ash::vk;
use stb::image::Channels;



type PoolMaxDescriptorSetCount = usize;
const POOL_SIZES: (PoolMaxDescriptorSetCount, [(vk::DescriptorPoolSize); 3]) = (
    30,
    [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 10,
        },
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::STORAGE_BUFFER,
            descriptor_count: 10,
        },
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 10,
        },
    ],
);

#[derive(Default)]
pub struct DescriptorAllocator {
    current_pool: Option<DescriptorPool>,
    used_pools: Vec<DescriptorPool>,
    free_pools: Vec<DescriptorPool>,
}
impl DescriptorAllocator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Destroy all pools
    pub fn destroy_pools(&self, context: &VkContext) {
        self.free_pools
            .iter()
            .for_each(|pool| pool.destroy(context));
        self.used_pools
            .iter()
            .for_each(|pool| pool.destroy(context));
    }

    fn take_pool(&mut self, context: &VkContext) -> DescriptorPool {
        let new_pool = if let Some(pool) = self.free_pools.pop() {
            pool
        } else {
            todo!("create pool")
            //DescriptorPool::create_pool2(
            //    context,
            //    &self.pool_sizes,
            //    1000,
            //    vk::DescriptorPoolCreateFlags::empty(),
            //)
        };
        self.used_pools.push(new_pool);
        new_pool
    }

    pub fn allocate_set(
        &mut self,
        context: &VkContext,
        layout: DescriptorSetLayout,
    ) -> vk::DescriptorSet {
        let current_pool = {
            // get current pool if any
            if let Some(pool) = self.current_pool {
                pool
            } else {
                // if no current pool, get a new one
                let new_pool = self.take_pool(context);
                self.current_pool = Some(new_pool);
                new_pool
            }
        };

        // try to allocate set in pool
        let allocated_set = match context.alloc_descriptor_set(current_pool, layout.handle) {
            // if success, use the set
            VkResult::Ok(allocated_set) => allocated_set,
            // if memory error, try allocating a new pool
            VkResult::Err(err)
            if err == vk::Result::ERROR_FRAGMENTED_POOL || err == vk::Result::ERROR_OUT_OF_POOL_MEMORY =>
                {
                    let desc_pool = self.take_pool(context);

                    context
                        .alloc_descriptor_set(desc_pool, layout.handle)
                        .expect("descriptor set allocator received a bad pool even after retry,\
                            this shouldn't be able to happen")

                }
            // if any other error, print it and panic
            VkResult::Err(err) => {
                let err_msg = format!("unknown error {:?}", err);
                log::error!("{}", err_msg);
                panic!("{}", err_msg);
            }
        };

        allocated_set
    }

    pub fn reset_used_pools(&mut self) {
        self.free_pools.extend(self.used_pools.iter());
        self.used_pools.clear();
        self.current_pool = None;
    }

}

//pub struct PoolSizes {
//    // (type, descriptor count multiplier)
//    sizes: Vec<(vk::DescriptorType, usize)>
//}
//impl Default for PoolSizes {
//    fn default() -> Self {
//        Self {
//            sizes: vec![
//                (vk::DescriptorType::UNIFORM_BUFFER, 1),
//                (vk::DescriptorType::STORAGE_BUFFER, 1),
//                (vk::DescriptorType::COMBINED_IMAGE_SAMPLER, 2)
//            ]
//        }
//    }
//}
//// for initializing the list above
//impl From<Vec<(vk::DescriptorType, usize)>> for PoolSizes {
//    fn from(sizes: Vec<(vk::DescriptorType, usize)>) -> Self {
//        Self { sizes }
//    }
//}

//impl PoolSizes {
//    fn vk_pool_sizes_multiplied(&self, descriptor_count: usize) -> Vec<vk::DescriptorPoolSize> {
//        self.sizes
//            .iter()
//            .map(|(ty, ty_desc_count_multiplier)| {
//                vk::DescriptorPoolSize::builder()
//                    .ty(*ty)
//                    .descriptor_count((descriptor_count * ty_desc_count_multiplier) as _)
//                    .build()
//            })
//            .collect()
//    }
//}

//impl DescriptorPool {
//    pub fn create_pool2(
//        context: &VkContext,
//        pool_sizes: &PoolSizes,
//        decsriptor_set_count: usize,
//        create_flags: vk::DescriptorPoolCreateFlags,
//    ) -> Self {
//        let pool_sizes: Vec<vk::DescriptorPoolSize> =
//            pool_sizes.vk_pool_sizes_multiplied(decsriptor_set_count);

//        let create_info = vk::DescriptorPoolCreateInfo::builder()
//            .max_sets(decsriptor_set_count as _)
//            .pool_sizes(&pool_sizes)
//            .flags(create_flags);

//        let descriptor_pool =
//            unsafe { context.device.create_descriptor_pool(&create_info, None) }
//                .expect("couldn't create descriptor pool");

//        DescriptorPool {
//            handle: descriptor_pool,
//        }
//    }
//}
