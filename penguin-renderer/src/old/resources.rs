use crate::render_objects::Mesh;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

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

    #[allow(dead_code)]
    pub fn insert(&mut self, name: &str, mesh: Mesh) {
        self.resource.insert(name, mesh);
    }

    pub fn get_rc(&self, name: &str) -> Rc<Mesh> {
        self.resource.get_rc(name)
    }
}
