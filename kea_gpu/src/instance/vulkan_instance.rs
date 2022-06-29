use ash::vk;
use log::info;
use std::{ffi::CStr, os::raw::c_char, sync::Arc};

use crate::{device::PhysicalDevice, features::Feature};

use super::{extensions::InstanceExtensions, Ext};

pub struct VulkanInstance {
    entry: ash::Entry,
    raw: ash::Instance,
    ext: InstanceExtensions,
}

impl VulkanInstance {
    pub fn new(features: &[Box<dyn Feature + '_>]) -> Arc<VulkanInstance> {
        let entry = ash::Entry::linked();
        let (raw, extensions) = Self::create_instance(&entry, features);
        let ext = InstanceExtensions::new(&entry, &raw, &extensions);
        Arc::new(VulkanInstance { entry, raw, ext })
    }

    fn create_instance(
        entry: &ash::Entry,
        features: &[Box<dyn Feature + '_>],
    ) -> (ash::Instance, Vec<Ext>) {
        let app_info = vk::ApplicationInfo::builder().api_version(vk::API_VERSION_1_3);

        let layer_names = unsafe {
            [CStr::from_bytes_with_nul_unchecked(
                b"VK_LAYER_KHRONOS_validation\0",
            )]
        };

        let mut extensions: Vec<Ext> = vec![];
        for feature in features {
            for ext in feature.instance_extensions() {
                extensions.push(ext);
            }
        }
        let extension_names: Vec<*const c_char> = extensions.iter().map(|ext| ext.name()).collect();
        info!("Requested instance extensions: {:?}", extensions);

        let layers_names_raw: Vec<*const c_char> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names)
            .enabled_layer_names(&layers_names_raw);

        let raw = unsafe { entry.create_instance(&create_info, None).unwrap() };

        (raw, extensions)
    }

    pub unsafe fn raw(&self) -> &ash::Instance {
        &self.raw
    }

    pub unsafe fn entry(&self) -> &ash::Entry {
        &self.entry
    }

    pub unsafe fn ext(&self) -> &InstanceExtensions {
        &self.ext
    }

    pub fn physical_devices(self: &Arc<VulkanInstance>) -> Vec<Arc<PhysicalDevice>> {
        unsafe {
            self.raw
                .enumerate_physical_devices()
                .unwrap()
                .into_iter()
                .map(|physical_device: vk::PhysicalDevice| {
                    Arc::new(PhysicalDevice::from_raw(physical_device, self.clone()))
                })
                .collect()
        }
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe { self.raw.destroy_instance(None) };
    }
}
