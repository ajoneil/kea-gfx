use super::vulkan::VulkanInstance;
use crate::window::Window;
use ash::vk;
use std::sync::Arc;

pub struct Surface {
    raw: vk::SurfaceKHR,
    vulkan: Arc<VulkanInstance>,
}

impl Surface {
    pub fn from_window<'a>(vulkan: Arc<VulkanInstance>, window: &Window) -> Surface {
        let raw = unsafe {
            ash_window::create_surface(&vulkan.entry(), &vulkan.raw(), window.window(), None)
        }
        .unwrap();

        Surface {
            raw,
            vulkan: vulkan,
        }
    }

    pub unsafe fn raw(&self) -> vk::SurfaceKHR {
        self.raw
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe { self.vulkan.ext().surface.destroy_surface(self.raw, None) };
    }
}
