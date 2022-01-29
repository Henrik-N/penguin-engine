use macaw::Affine3A;
use crate::impl_deref;

#[derive(Default)]
pub struct Transform {
    pub matrix: Affine3A,
}
impl_deref!(mut Transform, matrix, Affine3A);
