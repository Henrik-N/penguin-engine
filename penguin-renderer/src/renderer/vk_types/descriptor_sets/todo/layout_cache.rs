use std::collections::HashMap;

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


//#[derive(Default)]
//pub struct DescriptorSetLayoutCache {
//    layouts_cache: HashMap<DescriptorSetLayoutBuilder, vk::DescriptorSetLayout>,
//}
//impl DescriptorSetLayoutCache {
//    pub fn destroy_layouts(&self, context: &VkContext) {
//        self.layouts_cache
//            .iter()
//            .for_each(|(_builder, &desc_set_layout)| unsafe {
//                context
//                    .device
//                    .destroy_descriptor_set_layout(desc_set_layout, None);
//            });
//    }
//
//    pub fn new() -> Self {
//        Self::default()
//    }
//
//    pub fn create_descriptor_layout(
//        &mut self,
//        context: &VkContext,
//        layout_builder: DescriptorSetLayoutBuilder,
//    ) -> vk::DescriptorSetLayout {
//        if let Some(&layout) = self.layouts_cache.get(&layout_builder) {
//            // return if in cache
//            layout
//        } else {
//            // otherwise cache a new one and return that
//            let layout = layout_builder.clone().build_vk_type(context);
//            self.layouts_cache.insert(layout_builder, layout);
//            layout
//        }
//    }
//}

