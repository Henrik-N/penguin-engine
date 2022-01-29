use macaw::Affine3A;
use crate::math_vk_format::Vec3;
use crate::renderer::render_objects::{Material, Mesh};

pub struct ModelMatrix {
    matrix: Affine3A,
}


pub struct RenderObjectNew {
    model_matrix: ModelMatrix,
}

pub struct ProjectionMatrix {

}


pub struct RenderObject {
    pub material: Material,
    pub mesh: Mesh,
    pub translation: Vec3,
    pub name: String,
}


