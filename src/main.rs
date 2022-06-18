use env_logger::Env;
use gpu::{device::Device, surface::Surface, swapchain::Swapchain, vulkan::Vulkan};
use presenter::Presenter;
use rasterizer::Rasterizer;
use std::sync::Arc;
use window::Window;

mod gpu;
mod presenter;
mod rasterizer;
mod window;

struct KeaApp {
    presenter: Presenter,
    rasterizer: Rasterizer,
}

impl KeaApp {
    pub fn new(window: &Window) -> KeaApp {
        let vulkan = Arc::new(Vulkan::new(window.required_extensions()));
        let device = Arc::new(Device::new(&vulkan, Surface::from_window(&vulkan, &window)));
        let swapchain = Swapchain::new(&device);
        let rasterizer = Rasterizer::new(&device, swapchain.format);
        let presenter = Presenter::new(swapchain);

        KeaApp {
            presenter,
            rasterizer,
        }
    }

    pub fn draw(&self) {
        self.presenter.draw(|cmd, image_view| {
            self.rasterizer.draw(cmd, image_view);
        });
    }
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let window = Window::new(1920, 1080);
    let app = KeaApp::new(&window);

    window.event_loop(move || app.draw())
}
