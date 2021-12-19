use penguin_app::ecs::*;

use crate::renderer::{
    resources::{MaterialsResource, MeshesResource, RenderObjectsResource},
    render_loop,
    startup_shutdown,
};
use crate::renderer::resources::TexturesResource;
use crate::renderer::vk_types::resource::DescriptorSetsResource;

pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn startup(&mut self, resources: &mut Resources) -> Vec<Step> {

        resources.insert(MeshesResource::default());
        resources.insert(MaterialsResource::default());
        resources.insert(TexturesResource::default());
        resources.insert(RenderObjectsResource::default());
        resources.insert(DescriptorSetsResource::default());

        Schedule::builder()
            .add_thread_local(startup_shutdown::renderer_startup_system())
            .build()
            .into_vec()
    }

    fn run() -> Vec<Step> {
        Schedule::builder()
            .add_thread_local(render_loop::render_system())
            .build().into_vec()
    }

    fn shutdown() -> Vec<Step> {
        Schedule::builder()
            .add_thread_local(startup_shutdown::renderer_shutdown_system())
            .build()
            .into_vec()

    }
}
