use ash::vk;
use std::io::Read;

use std::ffi::CString;
use crate::renderer::vk_types::VkContext;

const SHADERS_FOLDER_PATH: &str = "penguin-renderer/src/shaders/";

pub struct Shader<'a> {
    context: &'a VkContext,
    entry_function_name: CString,
    pub shader_stage: vk::ShaderStageFlags,
    pub shader_module: vk::ShaderModule,
}

impl<'a> Drop for Shader<'a> {
    fn drop(&mut self) {
        unsafe {
            self.context.device.destroy_shader_module(self.shader_module, None);
        }
    }
}

impl<'a> Shader<'a> {
    pub fn new(context: &'a VkContext, file_name: &str) -> Self {
        let mut compiler = shaderc::Compiler::new().unwrap();

        let mut options = shaderc::CompileOptions::new().unwrap();
        options.set_optimization_level(shaderc::OptimizationLevel::Zero); // OptimizationLevel::Performance
        options.set_generate_debug_info();

        let file_path = String::from(SHADERS_FOLDER_PATH.to_string() + file_name);

        // read file content into source variable
        let mut source = String::new();
        match std::fs::File::open(&file_path) {
            Ok(mut file) => {
                file.read_to_string(&mut source)
                    .expect("Shader file content couldn't be read.");
            }
            Err(error) => println!("Error opening file {}: {}", file_name, error),
        };

        let shader_kind = Self::shader_kind_from_file_name(&file_name);

        let binary_result = compiler
            .compile_into_spirv(
                source.as_str(),
                shader_kind,
                file_path.as_str(),
                "main",
                Some(&options),
            )
            .expect("Couldn't compile shader source code.");

        let shader_stage = match shader_kind {
            shaderc::ShaderKind::Vertex => vk::ShaderStageFlags::VERTEX,
            shaderc::ShaderKind::Fragment => vk::ShaderStageFlags::FRAGMENT,
            shaderc::ShaderKind::Compute => vk::ShaderStageFlags::COMPUTE,
            _ => panic!("Shader type not implemented"),
        };

        let entry_function_name = CString::new("main").unwrap();

        let module_create_info =
            vk::ShaderModuleCreateInfo::builder().code(&binary_result.as_binary());

        let shader_module = unsafe { context.device.create_shader_module(&module_create_info, None) }
            .expect("Couldn't create shader module");

        Self {
            context,
            shader_stage,
            entry_function_name,
            shader_module,
        }
    }

    pub fn shader_stage_create_info(&self) -> vk::PipelineShaderStageCreateInfo {
        vk::PipelineShaderStageCreateInfo {
            module: self.shader_module,
            p_name: self.entry_function_name.as_ptr(),
            stage: self.shader_stage,
            ..Default::default()
        }
    }

    fn shader_kind_from_file_name(file_name: &str) -> shaderc::ShaderKind {
        let file_path = String::from(SHADERS_FOLDER_PATH.to_owned() + file_name);
        let file_extension = std::path::Path::new(&file_path)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .expect("Shader: Couldn't receive file extension.");
        match file_extension {
            "vert" => shaderc::ShaderKind::Vertex,
            "frag" => shaderc::ShaderKind::Fragment,
            "comp" => shaderc::ShaderKind::Compute,
            _ => panic!("Undefined shader file extension."),
        }
    }
}
