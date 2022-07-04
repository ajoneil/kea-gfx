use crate::device::Device;
use ash::{util::read_spv, vk};
use log::info;
use spirv_builder::{MetadataPrintout, SpirvBuilder};
use std::{collections::HashMap, fs::File, sync::Arc};

pub struct ShaderModule {
    device: Arc<Device>,
    raw: vk::ShaderModule,
    entry_points: Vec<String>,
}

impl ShaderModule {
    pub fn new(device: Arc<Device>, shader_crate_path: &str) -> Arc<ShaderModule> {
        let (entry_points, compiled_shaders) = Self::compile_shaders(&shader_crate_path);

        let shader_create_info = vk::ShaderModuleCreateInfo::builder().code(&compiled_shaders);

        let raw = unsafe { device.raw().create_shader_module(&shader_create_info, None) }.unwrap();

        Arc::new(ShaderModule {
            raw,
            device,
            entry_points,
        })
    }

    pub fn new_multimodule(
        device: &Arc<Device>,
        shader_crate_path: &str,
    ) -> HashMap<String, Arc<ShaderModule>> {
        let compiled_shaders = Self::compile_shaders_multimodule(&shader_crate_path);

        compiled_shaders
            .into_iter()
            .map(|(entry_point, compiled_shader)| {
                let shader_create_info =
                    vk::ShaderModuleCreateInfo::builder().code(&compiled_shader);
                let raw = unsafe { device.raw().create_shader_module(&shader_create_info, None) }
                    .unwrap();

                (
                    entry_point.clone(),
                    Arc::new(ShaderModule {
                        raw,
                        device: device.clone(),
                        entry_points: vec![entry_point],
                    }),
                )
            })
            .collect()
    }

    fn compile_shaders_multimodule(shader_crate_path: &str) -> HashMap<String, Vec<u32>> {
        let compile_result = SpirvBuilder::new(shader_crate_path, "spirv-unknown-vulkan1.2")
            .capability(spirv_builder::Capability::RayTracingKHR)
            .extension("SPV_KHR_ray_tracing")
            .print_metadata(MetadataPrintout::None)
            .multimodule(true)
            .build()
            .unwrap();

        info!("Shader entry points: {:?}", compile_result.entry_points);

        compile_result
            .module
            .unwrap_multi()
            .into_iter()
            .map(|(entry_point, path)| {
                (
                    entry_point.clone(),
                    read_spv(&mut File::open(path.to_path_buf()).unwrap()).unwrap(),
                )
            })
            .collect()
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

    pub fn entry_point(self: &Arc<Self>, entry_point_name: &str) -> ShaderEntryPoint {
        ShaderEntryPoint {
            module: self.clone(),
            name: self
                .entry_points
                .iter()
                .find(|name| *name == entry_point_name)
                .unwrap()
                .clone(),
        }
    }

    // pub fn entry_points(self: &Arc<Self>) -> impl Iterator<Item = ShaderEntryPoint> {
    //     self.entry_points.iter().map(|name| ShaderEntryPoint {
    //         module: self.clone(),
    //         name: name.clone(),
    //     })
    // }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_shader_module(self.raw, None);
        }
    }
}

pub struct ShaderEntryPoint {
    module: Arc<ShaderModule>,
    name: String,
}

impl ShaderEntryPoint {
    pub fn module(&self) -> &ShaderModule {
        &self.module
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
