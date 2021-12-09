use crate::math_vk_format::*;
use ash::vk;
use std::collections::hash_map::HashMap;
use std::rc::{Rc, Weak};
use crate::renderer::memory::AllocatedBuffer;
use crate::renderer::vk_types::{Pipeline, VkContext};

pub mod prelude {
    pub use super::{Material, Mesh, RenderObject, Vertex};
}

// **
// RenderObject
// **
pub struct RenderObject {
    pub mesh: Rc<Mesh>,
    pub material: Rc<Material>,
    pub transform: Mat4,
}
impl RenderObject {
    pub fn new(mesh: Rc<Mesh>, material: Rc<Material>, transform: Mat4) -> Self {
        Self {
            mesh,
            material,
            transform,
        }
    }

    pub fn _change_material(&mut self, material: Rc<Material>) {
        self.material = material;
    }
}


// **
// Material
// **
pub struct Material {
    pub pipeline: Pipeline,
}
impl PartialEq for Material {
    fn eq(&self, other: &Self) -> bool {
        self.pipeline == other.pipeline
    }
}
impl Eq for Material {}

impl Material {
    pub fn destroy(&mut self, context: &VkContext) {
        self.pipeline.destroy(&context);
    }

    pub fn from_pipeline(pipeline: Pipeline) -> Self {
        Self { pipeline }
    }

    pub fn bind(&self, context: &VkContext, command_buffer: vk::CommandBuffer) {
        context.bind_pipeline(&self.pipeline, command_buffer);
    }
}

// **
// Mesh
// **
#[derive(Clone, Copy, Default)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
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

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
        let offset0 = 0;
        let offset1 = std::mem::size_of::<Vec3>();
        let offset2 = offset1 + std::mem::size_of::<Vec3>();

        [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: Vec3::vk_format(),
                offset: offset0 as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: Vec3::vk_format(),
                offset: offset1 as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: Vec3::vk_format(),
                offset: offset2 as u32,
            },
        ]
    }
}

const MESHES_FOLDER_PATH: &'static str = "penguin-renderer/assets/meshes/";


pub struct Mesh {
    pub vertex_count: usize,
    pub vertex_buffer: AllocatedBuffer,
}
impl PartialEq for Mesh {
    fn eq(&self, other: &Self) -> bool {
        self.vertex_buffer.handle == other.vertex_buffer.handle
    }
}
impl Eq for Mesh {}

impl Mesh {
    pub fn destroy(&mut self, context: &VkContext) {
        self.vertex_buffer.destroy(&context);
    }

    pub fn from_obj(context: &VkContext, file_name: &str) -> Self {
        let file_path = String::from(MESHES_FOLDER_PATH.clone().to_string() + file_name);

        let (vertices, vertex_count) = Self::load_verts_indices_from_obj(&file_path);

        let vertex_buffer = AllocatedBuffer::create_vertex_buffer(
            context,
            &vertices,
        );

        Self {
            vertex_count,
            vertex_buffer,
        }
    }

    // NOTE: Only supports triangulated meshes, not the entire OBJ spec.
    fn load_verts_indices_from_obj(file_path: &str) -> (Vec<Vertex>, usize) {
        let model = wavefront::Obj::from_file(file_path).expect("Couldn't load obj file");

        let mut verts: Vec<Vertex> = Vec::new();

        let (red, green, blue) = (
            Vec3::new(1., 0., 0.),
            Vec3::new(0., 1., 0.),
            Vec3::new(0., 0., 1.),
        );

        for [a, b, c] in model.triangles() {
            let a = a.position();
            let b = b.position();
            let c = c.position();

            let a = Vec3::new(a[0], a[1], a[2]);
            let b = Vec3::new(b[0], b[1], b[2]);
            let c = Vec3::new(c[0], c[1], c[2]);

            verts.push(Vertex {
                position: a,
                color: red,
                normal: red,
            });
            verts.push(Vertex {
                position: b,
                color: green,
                normal: green,
            });
            verts.push(Vertex {
                position: c,
                color: blue,
                normal: blue,
            });
        }

        let verts_count = verts.len();

        (verts, verts_count)
    }
}
