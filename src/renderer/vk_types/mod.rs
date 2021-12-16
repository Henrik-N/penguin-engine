// -- containers --
pub mod vk_context;
pub use vk_context::*;

mod vk_components;
pub use vk_components::*;
pub use vk_components::init_vk_components;
// -- end of containers --



mod depth_image;
pub use depth_image::*;


pub mod descriptor;
pub use descriptor::*;



mod command_buffer;
pub use command_buffer::*;

mod pipeline;
pub use pipeline::*;

mod render_pass;
pub use render_pass::*;

pub mod resources {
    pub use super::descriptor::resource::*;
}

pub use crate::renderer::shader::*;

mod swapchain;
pub use swapchain::*;

