use env_logger::Env;
use gpu::{
    device::Device, rasterizer::Rasterizer, surface::Surface, swapchain::Swapchain, vulkan::Vulkan,
};
use std::sync::Arc;
use window::Window;

mod gpu;
mod presenter;
mod window;

struct KeaApp {
    _vulkan: Arc<Vulkan>,
    rasterizer: Rasterizer,
}

impl KeaApp {
    pub fn new(window: &Window) -> KeaApp {
        let vulkan = Arc::new(Vulkan::new(window.required_extensions()));
        let device = Arc::new(Device::new(&vulkan, Surface::from_window(&vulkan, &window)));
        let swapchain = Swapchain::new(&device);
        let rasterizer = Rasterizer::new(swapchain);

        KeaApp {
            _vulkan: vulkan,
            rasterizer,
        }
    }

    pub fn draw(&self) {
        self.rasterizer.draw()
    }
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let window = Window::new(1920, 1080);
    let app = KeaApp::new(&window);

    window.event_loop(move || app.draw())
}
