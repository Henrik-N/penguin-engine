use crate::engine::buffers::prelude::*;
use crate::engine::math::prelude::*;
use crate::engine::pe::pipeline::PPipeline;
use ash::vk;
use std::collections::hash_map::HashMap;
use std::rc::{Rc, Weak};
use crate::engine::renderer::vk_types::VkContext;

pub mod prelude {
    pub use super::{HashResource, Material, Mesh, MeshResource, RenderObject, Vertex};
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
            data: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: &str, resource: T) {
        self.data.insert(name.to_string(), Rc::new(resource));
    }

    pub fn get_rc(&self, name: &str) -> Rc<T> {
        let resource: &Rc<T> = self
            .data
            .get(&name.to_string())
            .expect(&format!("Couldn't find resource: {}", name));

        Rc::clone(&resource)
    }

    #[allow(dead_code)]
    pub fn get_weak(&self, name: &str) -> Weak<T> {
        let resource: &Rc<T> = self
            .data
            .get(&name.to_string())
            .expect(&format!("Couldn't find resource: {}", name));

        Rc::downgrade(&resource)
    }
}

pub struct MeshResource {
    device: Rc<ash::Device>,
    pd_memory_properties: vk::PhysicalDeviceMemoryProperties,
    resource: HashResource<Mesh>,
}

impl Drop for MeshResource {
    fn drop(&mut self) {
        // self.resource.data.iter().for_each(|(name, mesh)| {
        //     log::trace!("Destroying mesh: {}", name);
        //     mesh.destroy();
        // });
    }
}

impl MeshResource {
    pub fn new(
        device: Rc<ash::Device>,
        pd_memory_properties: vk::PhysicalDeviceMemoryProperties,
    ) -> Self {
        Self {
            device,
            pd_memory_properties,
            resource: HashResource::new(),
        }
    }

    //pub fn insert_from_file(&mut self, name: &str, file_name: &str) {
    //    self.resource.insert(
    //        name,
    //        Mesh::from_obj(
    //            file_name,
    //            &self.device,
    //            self.pd_memory_properties,
    //        ),
    //    );
    //}

    #[allow(dead_code)]
    pub fn insert(&mut self, name: &str, mesh: Mesh) {
        self.resource.insert(name, mesh);
    }

    pub fn get_rc(&self, name: &str) -> Rc<Mesh> {
        self.resource.get_rc(name)
    }
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
    pub pipeline: PPipeline,
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

    pub fn from_pipeline(pipeline: PPipeline) -> Self {
        Self { pipeline }
    }

    pub fn bind(&self, context: &VkContext, command_buffer: vk::CommandBuffer) {
        unsafe {
            context.device.handle.cmd_bind_pipeline(
                command_buffer,
                self.pipeline.pipeline_bind_point,
                self.pipeline.pipeline,
            );
        }
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

const MESHES_FOLDER_PATH: &'static str = "assets/meshes/";
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

    #[allow(dead_code)]
    pub fn from_vertices(
        device: Rc<ash::Device>,
        pd_memory_properties: vk::PhysicalDeviceMemoryProperties,
        vertices: Vec<Vertex>,
    ) -> Self {
        let vertex_buffer = AllocatedBuffer::create_vertex_buffer(
            &device,
            &vertices,
            pd_memory_properties,
        );

        //AllocatedBuffer::new_vertex_buffer(Rc::clone(&device), pd_memory_properties, &vertices);

        Self {
            vertex_count: vertices.len(),
            vertex_buffer,
        }
    }

    pub fn from_obj(
        context: &VkContext,
        file_name: &str,
        //device: &ash::Device,
        //pd_memory_properties: vk::PhysicalDeviceMemoryProperties,
    ) -> Self {
        let file_path = String::from(MESHES_FOLDER_PATH.clone().to_string() + file_name);

        let (vertices, vertex_count) = Self::load_verts_indices_from_obj(&file_path);

        let vertex_buffer = AllocatedBuffer::create_vertex_buffer(
            &context.device.handle,
            &vertices,
            context.pd_mem_properties(),
        );
        //AllocatedBuffer::new_vertex_buffer(Rc::clone(&device), pd_memory_properties, &vertices);

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

    //
    // fn load_verts_from_obj(file_path: &str) -> Vec<Vertex> {
    //     let (models, _materials) = tobj::load_obj(file_path, &tobj::LoadOptions::default())
    //         .expect("Failed to load obj file.");

    //     log::debug!("Obj loader: Number of models          = {}", models.len());

    //     let mut vertices: Vec<Vertex> = Vec::new();

    //     for (index, model) in models.iter().enumerate() {
    //         let mesh: &tobj::Mesh = &model.mesh;

    //         println!("model[{}].name = \'{}\'", index, model.name);
    //         println!("model[{}].mesh.material_id = {:?}", index, mesh.material_id);

    //         println!(
    //             "Size of model[{}].face_arities: {}",
    //             index,
    //             mesh.face_arities.len()
    //         );

    //         // Normals and texture coordinates are also loaded, but not printed in this example
    //         println!("model[{}].vertices: {}", index, mesh.positions.len() / 3);
    //         println!("normal[{}].normals: {}", index, mesh.normals.len() / 3);

    //         assert!(mesh.positions.len() % 3 == 0);
    //         assert!(mesh.normals.len() % 3 == 0);
    //         for v in 0..mesh.positions.len() / 3 {
    //             let v_pos = Vec3::new(
    //                 mesh.positions[3 * v],
    //                 mesh.positions[3 * v + 1],
    //                 mesh.positions[3 * v + 2],
    //             );

    //             let v_normal = Vec3::new(
    //                 mesh.normals[3 * v],
    //                 mesh.normals[3 * v + 1],
    //                 mesh.normals[3 * v + 2],
    //             );

    //             // println!(
    //             //     "    v[{}] = pos:({}, {}, {}), norm:({}, {}, {})",
    //             //     v,
    //             //     v_pos.x,
    //             //     v_pos.y,
    //             //     v_pos.z,
    //             //     v_normal.x,
    //             //     v_normal.y,
    //             //     v_normal.z,
    //             // );

    //             vertices.push(Vertex {
    //                 position: v_pos,
    //                 normal: v_normal,
    //                 color: v_normal, // setting color to normal for now
    //             });
    //         }
    //     }

    //     vertices
    // }

    // fn from_obj_old(
    //     file_name: &str,
    //     device: Rc<ash::Device>,
    //     pd_memory_properties: vk::PhysicalDeviceMemoryProperties,
    // ) -> Self {
    //     let file_path = String::from(MESHES_FOLDER_PATH.clone().to_string() + file_name);

    //     let input = BufReader::new(File::open(&file_path).expect("Couldn't open obj file"));
    //     let model: Obj = load_obj(input).expect("Couldn't create Obj file");

    //     log::debug!("Vertices: {:?}", model.vertices);
    //     log::debug!("Indices: {:?}", model.indices);
    //

    //     // model.vertices is obj::Vertex { position: Vec3, normal: Vec3 }
    //     let vertices: Vec<Vertex> = model.vertices.into_iter().enumerate().map(|(i, obj_vertex)| {
    //         let face_index = i % 3;
    //         let color = match face_index  {
    //             0 => Vec3::new(1.0, 0.0, 0.0),
    //             1 => Vec3::new(0.0, 1.0, 0.0),
    //             2 => Vec3::new(0.0, 0.0, 1.0),
    //             _ => panic!("Shouldn't be able to reach this!"),
    //         };

    //         let pos = obj_vertex.position;
    //         let norm = obj_vertex.normal;

    //         Vertex {
    //             position: Vec3::new(pos[0], pos[1], pos[2]),
    //             normal: Vec3::new(norm[0], norm[1], norm[2]),
    //             color,
    //         }
    //     }).collect();

    //     let vertex_buffer = AllocatedBuffer::new_vertex_buffer_obj(
    //         Rc::clone(&device),
    //         pd_memory_properties,
    //         &vertices
    //     );

    //     let indices = model.indices;

    //     let index_buffer = AllocatedBuffer::new_index_buffer_obj(
    //         Rc::clone(&device),
    //         pd_memory_properties,
    //         &indices);

    //     Self {
    //         vertices,
    //         indices,
    //         vertex_buffer,
    //         index_buffer,
    //     }
    // }

    // pub fn create_triangle_mesh(
    //     device: Rc<ash::Device>,
    //     pd_memory_properties: vk::PhysicalDeviceMemoryProperties,
    // ) -> Self {
    //     let vertices = [
    //         Vertex {
    //             position: Vec2::new(0.0, -0.5),
    //             color: Vec3::new(1.0, 0.0, 0.0),
    //         },
    //         Vertex {
    //             position: Vec2::new(0.5, 0.5),
    //             color: Vec3::new(0.0, 1.0, 0.0),
    //         },
    //         Vertex {
    //             position: Vec2::new(-0.5, 0.5),
    //             color: Vec3::new(0.0, 0.0, 1.0),
    //         },
    //     ];

    //     let vertex_buffer =
    //         AllocatedBuffer::new_vertex_buffer(Rc::clone(&device), pd_memory_properties, &vertices);

    //     Self {
    //         vertices: vertices.to_vec(),
    //         vertex_buffer,
    //     }
    // }
}
