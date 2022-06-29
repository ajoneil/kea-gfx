use crate::device::Device;
use ash::{util::read_spv, vk};
use log::info;
use spirv_builder::{MetadataPrintout, SpirvBuilder};
use std::{fs::File, sync::Arc};

pub struct ShaderModule {
    device: Arc<Device>,
    raw: vk::ShaderModule,
    entry_points: Vec<String>,
}

impl ShaderModule {
    pub fn new(device: Arc<Device>, shader_crate_path: &str) -> ShaderModule {
        let (entry_points, compiled_shaders) = Self::compile_shaders(&shader_crate_path);

        let shader_create_info = vk::ShaderModuleCreateInfo::builder().code(&compiled_shaders);

        let raw = unsafe { device.raw().create_shader_module(&shader_create_info, None) }.unwrap();

        ShaderModule {
            raw,
            device,
            entry_points,
        }
    }

    fn compile_shaders(shader_crate_path: &str) -> (Vec<String>, Vec<u32>) {
        let compile_result = SpirvBuilder::new(shader_crate_path, "spirv-unknown-vulkan1.2")
            .capability(spirv_builder::Capability::RayTracingKHR)
            .extension("SPV_KHR_ray_tracing")
            .print_metadata(MetadataPrintout::None)
            .build()
            .unwrap();

        info!("Shader entry points: {:?}", compile_result.entry_points);

        let compiled_shader_path = compile_result.module.unwrap_single().to_path_buf();

        (
            compile_result.entry_points,
            read_spv(&mut File::open(compiled_shader_path).unwrap()).unwrap(),
        )
    }

    pub unsafe fn raw(&self) -> vk::ShaderModule {
        self.raw
    }

    pub fn entry_point(&self, entry_point_name: &str) -> ShaderEntryPoint {
        self.entry_points()
            .find(|ep| ep.name == entry_point_name)
            .unwrap()
    }

    pub fn entry_points(&self) -> impl Iterator<Item = ShaderEntryPoint> {
        self.entry_points
            .iter()
            .map(|name| ShaderEntryPoint { module: self, name })
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_shader_module(self.raw, None);
        }
    }
}

pub struct ShaderEntryPoint<'a> {
    module: &'a ShaderModule,
    name: &'a String,
}

impl<'a> ShaderEntryPoint<'a> {
    pub fn module(&self) -> &ShaderModule {
        self.module
    }

    pub fn name(&self) -> &String {
        self.name
    }
}
