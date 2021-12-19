use ash::vk;
use crate::renderer::memory::{AllocatedBuffer, AllocatedBufferCreateInfo, AllocatedImage, AllocatedImageCreateInfo, MemoryUsage, UploadContext};
use crate::renderer::sync::PipelineBarrierBuilder;
use crate::renderer::vk_types::VkContext;


const IMAGES_FOLDER_PATH: &str = "penguin-renderer/assets/images/";

pub struct Texture {
    pub image: AllocatedImage,
    pub image_view: vk::ImageView,
}
impl Texture {
    pub fn destroy(&mut self, context: &VkContext) {
        unsafe { context.device.destroy_image_view(self.image_view, None) };
        self.image.destroy(context);
    }
}

impl Texture {
    pub fn from_image_file(context: &VkContext, upload_context: &UploadContext, image_file_name: &str) -> Self {
        let (allocated_image, image_format, subresource_range) = AllocatedImage::from_image_file(
            context,
            upload_context,
            &(IMAGES_FOLDER_PATH.to_owned() + image_file_name));

        let image_view_create_info = vk::ImageViewCreateInfo::builder()
            .image(allocated_image.handle)
            .format(image_format)
            .subresource_range(subresource_range)
            .view_type(vk::ImageViewType::TYPE_2D);

        let image_view = unsafe {
            context.device.create_image_view(&image_view_create_info, None)
        }.expect("couldn't create image view");

        Self {
            image: allocated_image,
            image_view,
        }
    }
}


impl AllocatedImage {
    fn from_image_file(context: &VkContext, upload_context: &UploadContext, file_path: &str)
        -> (Self, vk::Format, vk::ImageSubresourceRange) {

        let file = std::fs::File::open(file_path).expect(&format!("couldn't open file: {}", file_path));
        let mut reader = std::io::BufReader::new(file);

        let vk_format = vk::Format::R8G8B8A8_SRGB;
        let (image_info, pixels) = stb::image::stbi_load_from_reader(
            &mut reader, stb::image::Channels::RgbAlpha,
        ).expect(&format!("couldn't read image {} as RBGA", file_path));

        let size = pixels.size(); // aka pixel count / height * width * channel_count)

        let mut staging_buffer = AllocatedBuffer::create_buffer(context, AllocatedBufferCreateInfo::<u8> {
            buffer_size: size as _,
            buffer_usage: vk::BufferUsageFlags::TRANSFER_SRC,
            memory_usage: MemoryUsage::CpuMemGpuVisible,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        });

        let image_extent = vk::Extent3D::builder()
            .height(image_info.height as _)
            .width(image_info.width as _)
            .depth(1)
            .build();

        let allocated_image = AllocatedImage::create(
            context,
            AllocatedImageCreateInfo {
                image_create_info: {
                    vk::ImageCreateInfo::builder()
                        .image_type(vk::ImageType::TYPE_2D)
                        .mip_levels(1)
                        .array_layers(1)
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .tiling(vk::ImageTiling::OPTIMAL)
                        //
                        .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST)
                        .format(vk_format)
                        .extent(image_extent)
                },
                memory_usage: MemoryUsage::GpuOnly
            }
        );

        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1)
            .build();

        let subresource_layers = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1)
            .build();


        upload_context.immediate_submit(context, |cmd_buffer| {
            // perform layout transition to prepare image to be ready to be a destination
            // for memory transfers
            PipelineBarrierBuilder::builder()
                .src_stage_mask(vk::PipelineStageFlags::TOP_OF_PIPE)
                .dst_stage_mask(vk::PipelineStageFlags::TRANSFER)
                .image_memory_barriers(&[
                    vk::ImageMemoryBarrier::builder()
                        .image(allocated_image.handle)
                        .old_layout(vk::ImageLayout::UNDEFINED)
                        .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                        .subresource_range(subresource_range)
                        // prepare image layout to be ready to be a destination for memory transfers
                        .src_access_mask(vk::AccessFlags::empty())
                        .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                        .build()
                ])
                .build_exec(context, cmd_buffer);

            {
                let copy_region = vk::BufferImageCopy::builder()
                    .buffer_offset(0)
                    .buffer_row_length(0)
                    .buffer_image_height(0)
                    .image_subresource(subresource_layers)
                    //.image_offset()
                    .image_extent(image_extent)
                    .build();

                unsafe {
                    context.device.cmd_copy_buffer_to_image(
                        cmd_buffer,
                        staging_buffer.handle,
                        allocated_image.handle,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &[copy_region],
                    )
                }
            }

            //// perform layout transition to prepare image to be ready to read from shaders
            PipelineBarrierBuilder::builder()
                .src_stage_mask(vk::PipelineStageFlags::TRANSFER)
                .dst_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
                .image_memory_barriers(&[
                    vk::ImageMemoryBarrier::builder()
                        .image(allocated_image.handle)
                        .subresource_range(subresource_range)
                        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                        .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                        .dst_access_mask(vk::AccessFlags::SHADER_READ)
                        .build()
                ])
                .build_exec(context, cmd_buffer);
        });

        staging_buffer.destroy(context);

        log::trace!("Image {} loaded successfully!", file_path);

        (allocated_image, vk_format, subresource_range)
    }
}
