use ash::vk;
use std::{ffi::CStr, os::raw::c_char, sync::Arc};

use super::device::PhysicalDevice;

pub struct VulkanInstance {
    entry: ash::Entry,
    raw: ash::Instance,
    ext: Extensions,
}

pub struct Extensions {
    pub surface: ash::extensions::khr::Surface,
}

impl VulkanInstance {
    pub fn new(extension_names: &[*const i8]) -> Arc<VulkanInstance> {
        let entry = ash::Entry::linked();
        let raw = Self::create_instance(&entry, extension_names);
        let ext = Extensions {
            surface: ash::extensions::khr::Surface::new(&entry, &raw),
        };

        Arc::new(VulkanInstance { entry, raw, ext })
    }

    fn create_instance(entry: &ash::Entry, extension_names: &[*const i8]) -> ash::Instance {
        let app_info = vk::ApplicationInfo::builder().api_version(vk::API_VERSION_1_3);

        let layer_names = unsafe {
            [CStr::from_bytes_with_nul_unchecked(
                b"VK_LAYER_KHRONOS_validation\0",
            )]
        };

        let layers_names_raw: Vec<*const c_char> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(extension_names)
            .enabled_layer_names(&layers_names_raw);

        unsafe { entry.create_instance(&create_info, None).unwrap() }
    }

    pub unsafe fn raw(&self) -> &ash::Instance {
        &self.raw
    }

    pub unsafe fn entry(&self) -> &ash::Entry {
        &self.entry
    }

    pub unsafe fn ext(&self) -> &Extensions {
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
