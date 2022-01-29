mod layout_cache {
    use crate::renderer::vk_types::{DescriptorSetLayout, VkContext};
    use ash::vk;

    #[derive(Default, Clone)]
    pub struct DescriptorSetLayoutCreateInfo {
        pub layout_bindings: Vec<vk::DescriptorSetLayoutBinding>,
    }
    impl DescriptorSetLayoutCreateInfo {
        pub fn builder() -> Self {
            Self::default()
        }

        pub fn layout_binding(
            mut self,
            layout_binding: vk::DescriptorSetLayoutBindingBuilder,
        ) -> Self {
            self.layout_bindings.push(layout_binding.build());
            self
        }

        pub fn create_descriptor_layout(&self, context: &VkContext) -> DescriptorSetLayout {
            let layout_create_info =
                vk::DescriptorSetLayoutCreateInfo::builder().bindings(&self.layout_bindings);

            let layout = unsafe {
                context
                    .device
                    .create_descriptor_set_layout(&layout_create_info, None)
            }
            .expect("Couldn't create descriptor set layout");

            DescriptorSetLayout { handle: layout }
        }
    }
}
