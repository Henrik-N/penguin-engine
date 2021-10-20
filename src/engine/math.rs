pub mod prelude {
    pub use super::glam_types::*;
    pub use super::vk_formats::VkFormat;
}

pub mod glam_types {
    // f32
    pub use glam::{Affine2, Affine3A, Mat2, Mat3, Mat3A, Mat4, Quat, Vec2, Vec3, Vec3A, Vec4};
    // f64
    pub use glam::{DAffine2, DAffine3, DMat2, DMat3, DMat4, DQuat, DVec2, DVec3, DVec4};
    // i32
    pub use glam::{IVec2, IVec3, IVec4};
    // u32
    pub use glam::{UVec2, UVec3, UVec4};
    // bool
    pub use glam::{BVec2, BVec3, BVec4};
}

pub mod vk_formats {
    use super::glam_types::*;
    use ash::vk;

    pub trait VkFormat {
        fn vk_format() -> vk::Format;
    }

    //
    // vectors
    //

    // f32
    impl VkFormat for f32 {
        fn vk_format() -> vk::Format {
            vk::Format::R32_SFLOAT
        }
    }
    impl VkFormat for Vec2 {
        fn vk_format() -> vk::Format {
            vk::Format::R32G32_SFLOAT
        }
    }
    impl VkFormat for Vec3 {
        fn vk_format() -> vk::Format {
            vk::Format::R32G32B32_SFLOAT
        }
    }
    impl VkFormat for Vec3A {
        fn vk_format() -> vk::Format {
            vk::Format::R32G32B32_SFLOAT
        }
    }
    impl VkFormat for Vec4 {
        fn vk_format() -> vk::Format {
            vk::Format::R32G32B32A32_SFLOAT
        }
    }

    // f64
    impl VkFormat for f64 {
        fn vk_format() -> vk::Format {
            vk::Format::R64_SFLOAT
        }
    }
    impl VkFormat for DVec2 {
        fn vk_format() -> vk::Format {
            vk::Format::R64G64_SFLOAT
        }
    }
    impl VkFormat for DVec3 {
        fn vk_format() -> vk::Format {
            vk::Format::R64G64B64_SFLOAT
        }
    }
    impl VkFormat for DVec4 {
        fn vk_format() -> vk::Format {
            vk::Format::R64G64B64A64_SFLOAT
        }
    }

    // i32
    impl VkFormat for i32 {
        fn vk_format() -> vk::Format {
            vk::Format::R32_SINT
        }
    }
    impl VkFormat for IVec2 {
        fn vk_format() -> vk::Format {
            vk::Format::R32G32_SINT
        }
    }
    impl VkFormat for IVec3 {
        fn vk_format() -> vk::Format {
            vk::Format::R32G32B32_SINT
        }
    }
    impl VkFormat for IVec4 {
        fn vk_format() -> vk::Format {
            vk::Format::R32G32B32A32_SINT
        }
    }

    // u32
    impl VkFormat for u32 {
        fn vk_format() -> vk::Format {
            vk::Format::R32_UINT
        }
    }
    impl VkFormat for UVec2 {
        fn vk_format() -> vk::Format {
            vk::Format::R32G32_UINT
        }
    }
    impl VkFormat for UVec3 {
        fn vk_format() -> vk::Format {
            vk::Format::R32G32B32_UINT
        }
    }
    impl VkFormat for UVec4 {
        fn vk_format() -> vk::Format {
            vk::Format::R32G32B32A32_UINT
        }
    }

    // ...
}
