// ----------------- RESOURCES -----------------
use std::collections::HashMap;
use crate::renderer::memory::UploadContext;
use crate::renderer::render_objects::{Material, Mesh, RenderObject};
use crate::renderer::vk_types::{VkContext, Pipeline};


#[derive(Default)]
pub struct RenderObjectsResource {
    pub render_objects: Vec<RenderObject>,
}
impl std::ops::Deref for RenderObjectsResource {
    type Target = Vec<RenderObject>;

    fn deref(&self) -> &Self::Target {
        &self.render_objects
    }
}


#[derive(Default)]
pub struct MeshesResource {
    meshes: HashMap<String, Mesh>,
}
impl MeshesResource {
    pub fn destroy(&mut self, context: &VkContext) {
        self.meshes.iter_mut().for_each(|(_name, mesh)| mesh.destroy(&context));
    }

    pub fn insert_from_file(&mut self, context: &VkContext, upload_context: &UploadContext, (name, file_name): (&str, &str)) {
        self.meshes.insert(name.to_owned(), Mesh::from_obj(
            context,
            upload_context,
            file_name,
        ));
    }

    pub fn get(&self, name: &str) -> &Mesh {
        let name = name.to_owned();
        self.meshes.get(&name).expect(&format!("no mesh called {}", name))
    }
}

#[derive(Default)]
pub struct MaterialsResource {
    materials: HashMap<String, Material>,
}
impl MaterialsResource {
    pub fn destroy(&mut self, context: &VkContext) {
        self.materials.iter_mut().for_each(|(_name, material)| material.destroy(context));
    }

    pub fn insert(&mut self, (name, pipeline): (&str, Pipeline)) {
        self.materials.insert(name.to_owned(), Material::from_pipeline(pipeline));
    }

    pub fn get(&self, name: &str) -> &Material {
        let name = name.to_owned();
        self.materials.get(&name).expect(&format!("no material called {}", name))
    }
}

// ----------------- END OF RESOURCES -----------------
