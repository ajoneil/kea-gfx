use std::sync::Arc;

use ash::vk;

use crate::window::Window;

use super::Vulkan;

pub struct Surface {
    pub surface: vk::SurfaceKHR,
    vulkan: Arc<Vulkan>,
}

impl Surface {
    pub fn from_window<'a>(vulkan: &Arc<Vulkan>, window: &Window) -> Surface {
        let surface = unsafe {
            ash_window::create_surface(&vulkan.entry, &vulkan.instance, window.window(), None)
        }
        .unwrap();

        Surface {
            surface,
            vulkan: vulkan.clone(),
        }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe { self.vulkan.ext.surface.destroy_surface(self.surface, None) };
    }
}
