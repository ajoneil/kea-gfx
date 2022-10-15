use super::{SurfaceExt, Window};
use crate::instance::VulkanInstance;
use ash::vk;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::sync::Arc;

pub struct Surface {
    instance: Arc<VulkanInstance>,
    raw: vk::SurfaceKHR,
}

impl Surface {
    pub fn from_window(instance: Arc<VulkanInstance>, window: &Window) -> Surface {
        let raw = unsafe {
            ash_window::create_surface(
                &instance.entry(),
                &instance.raw(),
                window.window().raw_display_handle(),
                window.window().raw_window_handle(),
                None,
            )
        }
        .unwrap();

        Surface { instance, raw }
    }

    pub unsafe fn raw(&self) -> vk::SurfaceKHR {
        self.raw
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe { self.instance.ext::<SurfaceExt>().destroy_surface(self) };
    }
}
