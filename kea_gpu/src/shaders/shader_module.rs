use crate::device::Device;
use ash::{util::read_spv, vk};
use std::{collections::HashMap, io::Cursor, sync::Arc};

pub struct ShaderModule {
    device: Arc<Device>,
    raw: vk::ShaderModule,
    entry_points: Vec<String>,
}

impl ShaderModule {
    pub fn from_spirv_bytes(
        device: Arc<Device>,
        bytes: &[u8],
        entry_points: Vec<String>,
    ) -> Arc<ShaderModule> {
        let words = read_spv(&mut Cursor::new(bytes)).unwrap();
        let create_info = vk::ShaderModuleCreateInfo::default().code(&words);
        let raw = unsafe { device.raw().create_shader_module(&create_info, None) }.unwrap();
        Arc::new(ShaderModule {
            raw,
            device,
            entry_points,
        })
    }

    pub fn load_modules(
        device: &Arc<Device>,
        modules: &[(&str, &[u8])],
    ) -> HashMap<String, Arc<ShaderModule>> {
        modules
            .iter()
            .map(|(entry_point, bytes)| {
                let entry_point = entry_point.to_string();
                let module = Self::from_spirv_bytes(
                    device.clone(),
                    bytes,
                    vec![entry_point.clone()],
                );
                (entry_point, module)
            })
            .collect()
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
