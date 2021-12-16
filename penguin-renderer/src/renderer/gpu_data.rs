pub use uniforms::*;
/// Data to send to and from the gpu through Uniform buffers
mod uniforms {
    use crate::math_vk_format::{Mat4, Vec4};

    #[derive(Default, Clone, Copy)]
    #[repr(C)]
    pub struct GPUCameraData {
        pub data: Vec4,
        pub proj_view: Mat4,
    }

    #[derive(Default, Clone, Copy)]
    #[repr(C)]
    pub struct GPUObjectDataOld {
        pub transform: Mat4,
    }
}

pub use buffers::*;
mod buffers {
    use crate::math_vk_format::Mat4;

    #[derive(Default, Clone, Copy)]
    #[repr(C)]
    pub struct GPUObjectDataNew {
        pub transform: Mat4,
    }
}
