use std::{ffi::CStr, os::raw::c_char};

use ash::vk;

pub struct Vulkan {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub ext: Extensions,
}

pub struct Extensions {
    pub surface: ash::extensions::khr::Surface,
}

impl Vulkan {
    pub fn new(extension_names: &[*const i8]) -> Vulkan {
        let entry = ash::Entry::linked();
        let instance = Self::create_instance(&entry, extension_names);

        let surface = ash::extensions::khr::Surface::new(&entry, &instance);

        Vulkan {
            entry,
            instance,
            ext: { Extensions { surface } },
        }
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
}

impl Drop for Vulkan {
    fn drop(&mut self) {
        unsafe { self.instance.destroy_instance(None) };
    }
}
