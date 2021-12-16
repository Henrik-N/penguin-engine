use crate::math_vk_format::Vec3;
use crate::renderer::render_objects::{Material, Mesh};

pub struct RenderObject {
    pub material: Material,
    pub mesh: Mesh,
    pub translation: Vec3,
    pub name: String,
}
