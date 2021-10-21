pub mod prelude {
    pub use super::macaw_types::*;
    pub use super::vk_formats::VkFormat;
}

pub mod macaw_types {
    pub use macaw::prelude::*;

    pub use macaw::{
        Affine3A, BoundingBox, ColorRgba8, IVec2, IVec3, IVec4, IsoTransform, Mat2, Mat3, Mat4,
        MeshGen, Plane3, Quat, Ray3, UVec2, UVec3, UVec4, Vec2, Vec3, Vec3A, Vec4,
    };
}

pub mod vk_formats {
    use super::macaw_types::*;
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
