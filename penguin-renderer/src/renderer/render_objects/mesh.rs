use ash::vk;
use crate::math_vk_format::Vec3;
use crate::renderer::memory::{AllocatedBuffer, AllocatedBufferCreateInfo, DeviceMemoryWriteInfo, MemoryUsage, UploadContext};
use crate::renderer::render_objects::Vertex;
use crate::renderer::vk_types::VkContext;

const MESHES_FOLDER_PATH: &'static str = "penguin-renderer/assets/meshes/";

#[derive(Clone)]
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

    pub fn from_obj(context: &VkContext, upload_context: &UploadContext, file_name: &str) -> Self {
        let file_path = String::from(MESHES_FOLDER_PATH.clone().to_string() + file_name);

        let (vertices, vertex_count) = Self::load_verts_indices_from_obj(&file_path);

        let size = std::mem::size_of::<Vertex>() * vertices.len();
        //
        let mut staging_buffer = AllocatedBuffer::create_buffer(context, AllocatedBufferCreateInfo::<Vertex> {
            buffer_size: size as _,
            buffer_usage: vk::BufferUsageFlags::TRANSFER_SRC,
            memory_usage: MemoryUsage::CpuMemGpuVisible,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        });

        staging_buffer.write_memory(context, DeviceMemoryWriteInfo {
            data: &vertices,
            size: size as _,
            offset: 0,
            alignment: std::mem::align_of::<Vertex>() as _,
        });

        let gpu_buffer = AllocatedBuffer::create_buffer(context, AllocatedBufferCreateInfo::<Vertex> {
            buffer_size: size as _,
            buffer_usage: vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            memory_usage: MemoryUsage::GpuOnly,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        });

        upload_context.immediate_submit(context, |cmd_buffer| {
            let copy = vk::BufferCopy {
                src_offset: 0,
                dst_offset: 0,
                size: size as _,
            };
            let regions = [copy];

            unsafe {
                context.device.cmd_copy_buffer(
                    cmd_buffer,
                    staging_buffer.handle,
                    gpu_buffer.handle,
                    &regions,
                )
            }
        });

        staging_buffer.destroy(context);

        Self {
            vertex_count,
            vertex_buffer: gpu_buffer,
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
