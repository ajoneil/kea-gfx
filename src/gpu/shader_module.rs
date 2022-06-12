use std::{fs::File, sync::Arc};

use ash::{util::read_spv, vk};
use spirv_builder::{MetadataPrintout, SpirvBuilder};

use super::Device;

pub struct ShaderModule {
    pub module: vk::ShaderModule,
    device: Arc<Device>,
}

impl ShaderModule {
    pub fn new(device: &Arc<Device>) -> ShaderModule {
        let compiled_shaders = Self::compile_shaders();

        let shader_create_info = vk::ShaderModuleCreateInfo::builder().code(&compiled_shaders);

        let module = unsafe {
            device
                .device
                .create_shader_module(&shader_create_info, None)
        }
        .unwrap();

        ShaderModule {
            module,
            device: device.clone(),
        }
    }

    fn compile_shaders() -> Vec<u32> {
        let compiled_shader_path = SpirvBuilder::new("src/shaders", "spirv-unknown-vulkan1.2")
            .print_metadata(MetadataPrintout::None)
            .build()
            .unwrap()
            .module
            .unwrap_single()
            .to_path_buf();

        read_spv(&mut File::open(compiled_shader_path).unwrap()).unwrap()
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device.device.destroy_shader_module(self.module, None);
        }
    }
}
