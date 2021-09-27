use shaderc::{OptimizationLevel, CompilationArtifact};
use std::io::Read;
use ash::vk;
use ash::vk::ShaderModuleCreateInfo;
use crate::pe;

const SHADERS_FOLDER_PATH: &'static str = "src/shaders/";

const SHADERS: &[&'static str; 2] = &["simple.vert", "simple.frag"];

static ENTRY_NAME: &str = concat!("main", "\0");

pub struct Shader {
    binary_code: shaderc::CompilationArtifact,
    pub shader_stage: vk::ShaderStageFlags,
    pub shader_module: vk::ShaderModule,
}

impl Shader {
    pub fn drop(&self, device: &pe::Device) {
        unsafe {
           device.logical_device.destroy_shader_module(self.shader_module, None);
        }
    }

    #[no_mangle]
    pub extern "C" fn get_entry_name() -> *const std::os::raw::c_char {
        ENTRY_NAME.as_ptr() as *const std::os::raw::c_char
    }

    pub fn create_and_compile(device: &pe::Device, file_name: &str) -> Shader {
        let mut compiler = shaderc::Compiler::new().unwrap();
        let mut options = shaderc::CompileOptions::new().unwrap();
        options.set_optimization_level(OptimizationLevel::Zero); // OptimizationLevel::Performance
        options.set_generate_debug_info();

        let file_path = String::from(SHADERS_FOLDER_PATH.clone().to_string() + file_name);

        // read file content into source variable
        let mut source = String::new();
        match std::fs::File::open(&file_path) {
            Ok(mut file) => {
                file.read_to_string(&mut source)
                    .expect("Shader file content couldn't be read.");
            },
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

        assert_eq!(Some(&0x07230203), binary_result.as_binary().first());


        let shader_module = Self::create_shader_module(&binary_result, &device);


        Shader {
            binary_code: binary_result,
            shader_stage: match shader_kind {
                shaderc::ShaderKind::Vertex => vk::ShaderStageFlags::VERTEX,
                shaderc::ShaderKind::Fragment => vk::ShaderStageFlags::FRAGMENT,
                shaderc::ShaderKind::Compute => vk::ShaderStageFlags::COMPUTE,
                _ => panic!("Shader type not implemented"),
            },
            shader_module
        }
    }

    fn create_shader_module(binary_code: &CompilationArtifact, device: &pe::Device) -> vk::ShaderModule {
        let create_info = vk::ShaderModuleCreateInfo::builder()
            .code(&binary_code.as_binary());

        unsafe {
            device.logical_device.create_shader_module(&create_info, None)
        }.expect("Couldn't create ShaderModule")
    }

    fn shader_kind_from_file_name(file_name: &str) -> shaderc::ShaderKind {
        let file_path = String::from(SHADERS_FOLDER_PATH.clone().to_string() + file_name);
        let file_extension = std::path::Path::new(&file_path).extension().and_then(std::ffi::OsStr::to_str).expect("Shader: Couldn't receive file extension.");
        match file_extension {
            "vert" => shaderc::ShaderKind::Vertex,
            "frag" => shaderc::ShaderKind::Fragment,
            "comp" => shaderc::ShaderKind::Compute,
            _ => panic!("Undefined shader file extension.")
        }
    }
}


// pub struct Shaders {
//     pub vertex_shaders: std::collections::HashMap<String, vk::ShaderModuleCreateInfo>,
//     pub fragment_shaders: std::collections::HashMap<String, vk::ShaderModuleCreateInfo>,
// }
//
// impl Shaders {
//     pub fn get_shader_module_info(&self, shader_file_name: &str) -> &ShaderModuleCreateInfo {
//         let shader_kind = Self::shader_kind_from_file_name(shader_file_name);
//
//         match shader_kind {
//             shaderc::ShaderKind::Vertex => self.vertex_shaders.get(shader_file_name),
//             shaderc::ShaderKind::Fragment => self.fragment_shaders.get(shader_file_name),
//             _ => panic!("Shaders of that type not yet implemented"),
//         }.expect("Couldn't receive shader")
//     }
//
//     pub fn init() -> Shaders {
//         let mut compiler = shaderc::Compiler::new().unwrap();
//         let mut options = shaderc::CompileOptions::new().unwrap();
//         options.set_optimization_level(OptimizationLevel::Zero); // OptimizationLevel::Performance
//         options.set_generate_debug_info();
//
//         let (vertex_shaders, fragment_shaders) = Self::compile_shaders(
//             &mut compiler,
//             &mut options,
//         );
//
//         Shaders {
//             vertex_shaders,
//             fragment_shaders,
//         }
//     }
//
//     fn compile_shaders(
//         compiler: &mut shaderc::Compiler,
//         options: &mut shaderc::CompileOptions,
//     ) ->
//         (std::collections::HashMap<String, vk::ShaderModuleCreateInfo>,
//          std::collections::HashMap<String, vk::ShaderModuleCreateInfo>)
//     {
//         let mut vertex_shaders = std::collections::HashMap::new();
//         let mut fragment_shaders = std::collections::HashMap::new();
//
//         for &file_name in SHADERS {
//             let file_path = String::from(SHADERS_FOLDER_PATH.clone().to_string() + file_name);
//
//             // read file content into source variable
//             let mut source = String::new();
//             match std::fs::File::open(&file_path) {
//                 Ok(mut file) => {
//                     file.read_to_string(&mut source)
//                         .expect("Shader file content couldn't be read.");
//                 }
//                 Err(error) => println!("Error opening file {}: {}", file_name, error),
//             }
//
//             // figure out the shader kind
//             let shader_kind = Self::shader_kind_from_file_name(&file_name);
//
//             // compile source into binary SPIR-V code
//             let binary_result = compiler
//                 .compile_into_spirv(
//                     source.as_str(),
//                     shader_kind,
//                     file_path.as_str(),
//                     "main",
//                     Some(&options),
//                 )
//                 .expect("Couldn't compile shader source code.");
//
//             assert_eq!(Some(&0x07230203), binary_result.as_binary().first());
//
//             // create shader module
//             let module_create_info = vk::ShaderModuleCreateInfo {
//                 code_size: binary_result.len(),
//                 p_code: binary_result.as_binary().as_ptr(),
//                 ..Default::default()
//             };
//
//             match shader_kind {
//                 shaderc::ShaderKind::Vertex => vertex_shaders.insert(file_name.to_string(), module_create_info),
//                 shaderc::ShaderKind::Fragment => fragment_shaders.insert(file_name.to_string(), module_create_info),
//                 _ => panic!("ShaderKind not implemented")
//             };
//         }
//         (vertex_shaders, fragment_shaders)
//     }
//
//     fn shader_kind_from_file_name(file_name: &str) -> shaderc::ShaderKind {
//         let file_path = String::from(SHADERS_FOLDER_PATH.clone().to_string() + file_name);
//         let file_extension = std::path::Path::new(&file_path).extension().and_then(std::ffi::OsStr::to_str).expect("Shader: Couldn't receive file extension.");
//         match file_extension {
//             "vert" => shaderc::ShaderKind::Vertex,
//             "frag" => shaderc::ShaderKind::Fragment,
//             "comp" => shaderc::ShaderKind::Compute,
//             _ => panic!("Undefined shader file extension.")
//         }
//     }
// }
