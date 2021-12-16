//pub fn old_draw(&mut self, delta_time: f32) {
//    let _ = delta_time;
//
//    self.frame_num += 1;
//    let frame_index = self.frame_num % penguin_config::vk_config::MAX_FRAMES_COUNT;
//
//    self.context.submit_render_commands(
//        &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
//        frame_index,
//        |device, command_buffer, frame_buffer, frame_data| {
//            //let flash = f32::abs(f32::sin(self.frame_num as f32 / 120_f32));
//            //let color = [0.0_f32, 0.0_f32, flash, 1.0_f32];
//            let color = [0.0_f32, 0., 0., 1.];
//
//            let color_clear = vk::ClearColorValue { float32: color };
//
//            let depth_clear = vk::ClearDepthStencilValue::builder().depth(1.0_f32).build();
//
//            let clear_values = [
//                vk::ClearValue { color: color_clear },
//                vk::ClearValue {
//                    depth_stencil: depth_clear,
//                },
//            ];
//
//            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
//                .render_pass(self.context.render_pass)
//                .framebuffer(frame_buffer)
//                .render_area(vk::Rect2D {
//                    offset: vk::Offset2D { x: 0, y: 0 },
//                    extent: self.context.swapchain_extent,
//                })
//                .clear_values(&clear_values);
//
//            unsafe {
//                device.cmd_begin_render_pass(
//                    command_buffer,
//                    &render_pass_begin_info,
//                    vk::SubpassContents::INLINE,
//                );
//
//                self.draw_render_objects(
//                    &self.context.device,
//                    command_buffer,
//                    frame_data,
//                    self.frame_num,
//                );
//
//                device.cmd_end_render_pass(command_buffer);
//            }
//        },
//    );
//}

