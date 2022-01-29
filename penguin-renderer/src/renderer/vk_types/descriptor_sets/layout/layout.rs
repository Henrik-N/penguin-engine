use crate::renderer::vk_types::VkContext;
use ash::vk;
use std::hash::Hasher;
use crate::impl_deref;

#[derive(Default, Clone, Copy)]
pub struct DescriptorSetLayout {
    pub handle: vk::DescriptorSetLayout,
}
impl_deref!(DescriptorSetLayout, handle, vk::DescriptorSetLayout);

impl DescriptorSetLayout {
    pub fn builder() -> DescriptorSetLayoutBuilder {
        DescriptorSetLayoutBuilder::builder()
    }

    pub fn destroy(&self, context: &VkContext) {
        unsafe {
            context
                .device
                .destroy_descriptor_set_layout(self.handle, None)
        }
    }
}

#[derive(Clone)]
struct DescriptorSetLayoutBinding {
    handle: vk::DescriptorSetLayoutBinding,
}
impl_deref!(DescriptorSetLayoutBinding, handle, vk::DescriptorSetLayoutBinding);

impl std::hash::Hash for DescriptorSetLayoutBinding {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.binding);
        self.descriptor_type.hash(state);
        state.write_u32(self.descriptor_count);
        self.stage_flags.hash(state);
        //state.write_u32(self.stage_flags as _);
        // todo p_immutable_samplers
        state.finish();
    }
}
impl From<vk::DescriptorSetLayoutBinding> for DescriptorSetLayoutBinding {
    fn from(handle: vk::DescriptorSetLayoutBinding) -> Self {
        Self { handle }
    }
}

#[derive(Clone, Hash)]
pub struct DescriptorSetLayoutBuilder {
    layout_bindings: Vec<DescriptorSetLayoutBinding>,
}

impl Eq for DescriptorSetLayoutBuilder {}
impl PartialEq for DescriptorSetLayoutBuilder {
    fn eq(&self, other: &Self) -> bool {
        if self.layout_bindings.len() != other.layout_bindings.len() {
            return false;
        }

        let first_non_equal = self
            .layout_bindings
            .iter()
            .zip(other.layout_bindings.iter())
            .find(|(this, other)| {
                let equal = if this.binding == other.binding &&
                    this.descriptor_type == other.descriptor_type &&
                    this.descriptor_count == other.descriptor_count &&
                    this.stage_flags == other.stage_flags &&
                    // todo can probably be the same even though the pointer is different
                    this.p_immutable_samplers == other.p_immutable_samplers
                {
                    true
                } else {
                    false
                };

                !equal
            });

        first_non_equal.is_none()
    }
}

impl DescriptorSetLayoutBuilder {
    fn builder() -> Self {
        Self {
            layout_bindings: vec![],
        }
    }

    pub fn layout_binding(mut self, layout_binding: vk::DescriptorSetLayoutBindingBuilder) -> Self {
        self.layout_bindings.push(layout_binding.build().into());
        self
    }

    pub fn build_vk_type(self, context: &VkContext) -> vk::DescriptorSetLayout {
        // create layout
        let bindings = self
            .layout_bindings
            .iter()
            .map(|b| b.handle)
            .collect::<Vec<_>>();

        let layout_create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);

        unsafe {
            context
                .device
                .create_descriptor_set_layout(&layout_create_info, None)
        }
        .expect("Couldn't create descriptor set layout")
    }

    pub fn build(self, context: &VkContext) -> DescriptorSetLayout {
        DescriptorSetLayout {
            handle: self.build_vk_type(context),
        }
    }
}
