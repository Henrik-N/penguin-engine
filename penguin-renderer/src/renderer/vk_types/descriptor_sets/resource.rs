use ash::vk;
use crate::renderer::vk_types::{DescriptorPool, DescriptorSetContainer, VkContext};

pub struct DescriptorSetsResource {
    pub pool: DescriptorPool,
    pub sets: Vec<DescriptorSetContainer>,
}
impl Default for DescriptorSetsResource {
    fn default() -> Self {
        Self { pool: DescriptorPool::default(), sets: Vec::with_capacity(4) }
    }
}
impl DescriptorSetsResource {
    pub fn init_pool(&mut self, pool: DescriptorPool) {
        self.pool = pool;
    }

    pub fn destroy(&mut self, context: &VkContext) {
        self.sets.iter_mut().for_each(|set| set.destroy(context));
        self.pool.destroy(context);
    }

    pub fn get_set_handles(&self, set_ids: &[usize]) -> Vec<vk::DescriptorSet> {
        set_ids.iter().map(|&set_id| {
            self.get_set(set_id).set.handle
        }).collect::<Vec<vk::DescriptorSet>>()
    }

    pub fn get_set(&self, set: usize) -> &DescriptorSetContainer {
        &self.sets[set]
    }
}
