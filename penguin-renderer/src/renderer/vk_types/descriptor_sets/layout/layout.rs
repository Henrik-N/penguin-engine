use ash::vk;
use crate::renderer::vk_types::VkContext;


#[derive(Default, Clone)]
pub struct DescriptorSetLayout {
    pub handle: vk::DescriptorSetLayout,
}
impl std::ops::Deref for DescriptorSetLayout {
    type Target = vk::DescriptorSetLayout;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}
impl DescriptorSetLayout {
    pub fn builder(context: &VkContext) -> DescriptorSetLayoutBuilder {
        DescriptorSetLayoutBuilder::builder(context)
    }

    pub fn destroy(&self, context: &VkContext) {
        unsafe { context.device.destroy_descriptor_set_layout(self.handle, None) }
    }
}


pub struct DescriptorSetLayoutBuilder<'a> {
    context: &'a VkContext,
    layout_bindings: Vec<vk::DescriptorSetLayoutBinding>,
}
impl<'a> DescriptorSetLayoutBuilder<'a> {
    fn builder(context: &'a VkContext) -> Self {
        Self {
            context,
            layout_bindings: vec![],
        }
    }

    pub fn layout_binding(mut self, layout_binding: vk::DescriptorSetLayoutBindingBuilder) -> Self {
        self.layout_bindings.push(layout_binding.build());
        self
    }

    pub fn build(self) -> DescriptorSetLayout {
        // create layout
        let layout_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&self.layout_bindings);

        let layout = unsafe {
            self.context.device
                .create_descriptor_set_layout(&layout_create_info, None) }
            .expect("Couldn't create descriptor set layout");

        DescriptorSetLayout { handle: layout }
    }
}

