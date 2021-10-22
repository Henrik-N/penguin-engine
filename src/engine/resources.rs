use ash::vk;
use crate::engine::buffers::AllocatedBuffer;
use std::rc::{Rc, Weak};
use std::collections::hash_map::HashMap;
use crate::engine::pe::pipeline::PPipeline;
use crate::engine::math::prelude::*;

pub mod prelude {
    pub use super::{
        Vertex,
        Mesh,
        Material,
        RenderObject,
        HashResource,
    };
}

// **
// HashResource
// **
pub struct HashResource<T> {
    pub data: HashMap<String, Rc<T>>,
}
impl<T> HashResource<T> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new()
        }
    }

    pub fn insert(&mut self, name: &str, resource: T) {
        self.data.insert(name.to_string(), Rc::new(resource));
    }

    pub fn get_rc(&self, name: &str) -> Rc<T> {
        let resource: &Rc<T> =
            self.data.get(&name.to_string())
            .expect(&format!("Couldn't find resource: {}", name));

        Rc::clone(&resource)
    }

    pub fn get_weak(&self, name: &str) -> Weak<T> {
        let resource: &Rc<T> = 
            self.data.get(&name.to_string())
            .expect(&format!("Couldn't find resource: {}", name));

        Rc::downgrade(&resource)
    }
}



// ***
// Vertex
// ***
#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: Vec2,
    pub color: Vec3,
}
impl Vertex {
    pub fn get_binding_descriptions() -> [vk::VertexInputBindingDescription; 1] {
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        let offset0 = 0;
        let offset1 = std::mem::size_of::<Vec2>();

        [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: Vec2::vk_format(),
                offset: offset0 as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: Vec3::vk_format(),
                offset: offset1 as u32,
            },
        ]
    }
}


// **
// RenderObject
// **
pub struct RenderObject {
    pub mesh: Rc<Mesh>,
    pub material: Rc<Material>,
    pub transform: Mat4
}
impl RenderObject {
    pub fn new(mesh: Rc<Mesh>, material: Rc<Material>, transform: Mat4) -> Self {
        Self {mesh, material, transform}
    }

    pub fn _change_material(&mut self, material: Rc<Material>) {
        self.material = material;
    }
}


// **
// Material
// **
pub struct Material {
    device: Rc<ash::Device>,
    pipeline: PPipeline,
}
impl Drop for Material {
    fn drop(&mut self) {
        self.pipeline.destroy(&self.device);
    }
}
impl PartialEq for Material {
    fn eq(&self, other: &Self) -> bool {
        self.pipeline == other.pipeline
    }
}
impl Eq for Material {}
impl Material {
    pub fn from_pipeline(device: Rc<ash::Device>, pipeline: PPipeline) -> Self {
        Self { device, pipeline, }
    }

    pub fn bind(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device.cmd_bind_pipeline(command_buffer, self.pipeline.pipeline_bindpoint, self.pipeline.pipeline);
        }
    }
}




// **
// Mesh
// **
pub struct Mesh {
    #[allow(dead_code)]
    vertices: Vec<Vertex>,
    pub vertex_buffer: AllocatedBuffer,
}
// impl Drop for Mesh {
//     fn drop(&mut self) {
//         self.vertex_buffer.destroy();
//     }
// }
impl PartialEq for Mesh {
    fn eq(&self, other: &Self) -> bool {
        self.vertex_buffer.buffer_handle == other.vertex_buffer.buffer_handle
    }
}
impl Eq for Mesh {}
impl Mesh {
    // pub fn destroy(&mut self) {
    //     self.vertex_buffer.destroy();
    // }


    pub fn create_triangle_mesh(
        device: Rc<ash::Device>,
        pd_memory_properties: vk::PhysicalDeviceMemoryProperties,
    ) -> Self {
        let vertices = [
            Vertex {
                position: Vec2::new(0.0, -0.5),
                color: Vec3::new(1.0, 0.0, 0.0),
            },
            Vertex {
                position: Vec2::new(0.5, 0.5),
                color: Vec3::new(0.0, 1.0, 0.0),
            },
            Vertex {
                position: Vec2::new(-0.5, 0.5),
                color: Vec3::new(0.0, 0.0, 1.0),
            },
        ];

        let vertex_buffer =
            AllocatedBuffer::new_vertex_buffer(Rc::clone(&device), pd_memory_properties, &vertices);

        Self {
            vertices: vertices.to_vec(),
            vertex_buffer,
        }
    }
}

