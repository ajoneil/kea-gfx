use super::vulkan::VulkanInstance;
use crate::window::Window;
use ash::vk;
use std::sync::Arc;

pub struct Surface {
    pub surface: vk::SurfaceKHR,
    vulkan: Arc<VulkanInstance>,
}

impl Surface {
    pub fn from_window<'a>(vulkan: Arc<VulkanInstance>, window: &Window) -> Surface {
        let surface = unsafe {
            ash_window::create_surface(&vulkan.entry(), &vulkan.raw(), window.window(), None)
        }
        .unwrap();

        Surface {
            surface,
            vulkan: vulkan,
        }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.vulkan
                .ext()
                .surface
                .destroy_surface(self.surface, None)
        };
    }
}
