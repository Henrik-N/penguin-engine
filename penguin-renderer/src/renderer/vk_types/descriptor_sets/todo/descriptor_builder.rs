use ash::vk;

pub struct DescriptorBuilderBindParams {
    pub binding: u32,
    pub ty: vk::DescriptorType,
    pub shader_stages: vk::ShaderStageFlags,
}
impl DescriptorBuilderBindParams {
    fn descriptor_set_layout_binding(&self) -> vk::DescriptorSetLayoutBinding {
        vk::DescriptorSetLayoutBinding::builder()
            .descriptor_count(1)
            .descriptor_type(self.ty)
            .binding(self.binding)
            .stage_flags(self.shader_stages)
            //.p_immutable_samplers() );
            .build()
    }
}

//mod new {
//    use ash::vk;
//
//    #[derive(Default)]
//    struct DescriptorBuilder {
//        set_bindings: Vec<vk::DescriptorSetLayoutBinding>,
//        write_sets: Vec<vk::WriteDescriptorSet>,
//    }
//    impl<'a> DescriptorBuilder {
//        pub fn builder() -> Self {
//            Self::default()
//        }
//
//        pub fn bind_buffer(mut self, binding: u32, buffer_info: &'a [vk::DescriptorBufferInfo], ty: vk::DescriptorType, stages: vk::ShaderStageFlags) -> Self {
//            self.set_bindings.push(
//                vk::DescriptorSetLayoutBinding::builder()
//                    .descriptor_count(1)
//                    .descriptor_type(ty)
//                    .binding(binding)
//                    .stage_flags(stages)
//                    //.p_immutable_samplers() );
//                    .build()
//            );
//            self.write_sets.push(vk::WriteDescriptorSet::builder()
//                .descriptor_type(ty)
//                .dst_binding(binding)
//                .buffer_info(buffer_info)
//                .build()
//            );
//            self
//        }
//    }
//
//
//
//
//    fn bindings_eq(b0: vk::DescriptorSetLayoutBinding, b1: vk::DescriptorSetLayoutBinding) -> bool {
//        if b0.binding == b1.binding &&
//            b0.descriptor_type == b1.descriptor_type &&
//            b0.descriptor_count == b1.descriptor_count &&
//            b0.stage_flags == b1.stage_flags
//            {
//            return true;
//        }
//        false
//    }
//
//    struct DescriptorLayout {
//        bindings: Vec<vk::DescriptorSetLayoutBinding>,
//    }
//    impl Eq for DescriptorLayout {}
//    impl PartialEq<Self> for DescriptorLayout {
//        fn eq(&self, other: &Self) -> bool {
//
//            if self.bindings.len() != other.bindings.len() {
//                return false;
//            }
//
//            let pair_that_is_different = self.bindings.iter()
//                .zip(other.bindings.iter())
//                .find(|(&self_binding, &other_binding)| {
//                    bindings_eq(self_binding, other_binding)
//            });
//
//            pair_that_is_different.is_none()
//        }
//    }
//}

//#[derive(Default)]
//struct DescriptorBuilder<'a> {
//    allocator: &'a DescriptorAllocator,
//    layout_cache: &'a DescriptorSetLayoutCache,
//    //
//    set_bindings: Vec<vk::DescriptorSetLayoutBinding>,
//    write_sets: Vec<vk::WriteDescriptorSet>,
//}
//impl<'a> DescriptorBuilder<'a> {
//    pub fn builder(
//        allocator: &'a DescriptorAllocator,
//        layout_cache: &'a DescriptorSetLayoutCache,
//    ) -> Self {
//        todo!()
//        //Self { allocator, layout_cache, ..Default::default() }
//    }
//
//    pub fn bind_buffer(
//        mut self,
//        buffer_info: &'a [vk::DescriptorBufferInfo],
//        bind_params: DescriptorBuilderBindParams,
//    ) -> Self {
//        self.set_bindings
//            .push(bind_params.descriptor_set_layout_binding());
//
//        self.write_sets.push(
//            vk::WriteDescriptorSet::builder()
//                .descriptor_type(bind_params.ty)
//                .dst_binding(bind_params.binding)
//                .buffer_info(buffer_info)
//                .build(),
//        );
//        self
//    }
//
//    pub fn bind_image(
//        mut self,
//        image_info: &'a [vk::DescriptorImageInfo],
//        bind_params: DescriptorBuilderBindParams,
//    ) -> Self {
//        self.set_bindings
//            .push(bind_params.descriptor_set_layout_binding());
//
//        self.write_sets.push(
//            vk::WriteDescriptorSet::builder()
//                .descriptor_type(bind_params.ty)
//                .dst_binding(bind_params.binding)
//                .image_info(image_info)
//                .build(),
//        );
//        self
//    }
//
//    pub fn build(self, context: &VkContext) -> (vk::DescriptorSet, vk::DescriptorSetLayout) {
//        todo!()
//        //let desc_set_layout = self.layout_cache.create_descriptor_layout(context, )
//    }
//}
