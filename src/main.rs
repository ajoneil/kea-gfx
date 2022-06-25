use env_logger::Env;
use gpu::{
    device::Device, physical_device::PhysicalDevice, surface::Surface, swapchain::Swapchain,
    vulkan::Vulkan,
};
use path_tracer::PathTracer;
use presenter::Presenter;
use rasterizer::Rasterizer;
use shaders::ray_for_pixel;
use std::sync::Arc;
use window::Window;

mod gpu;
mod path_tracer;
mod presenter;
mod rasterizer;
mod window;

struct KeaApp {
    presenter: Presenter,
    // rasterizer: Rasterizer,
    path_tracer: PathTracer,
}

impl KeaApp {
    pub fn new(window: &Window) -> KeaApp {
        let vulkan = Arc::new(Vulkan::new(window.required_extensions()));
        let surface = Surface::from_window(vulkan.clone(), &window);
        let device_selection = PhysicalDevice::select_physical_device(&vulkan, &surface);
        let device = Device::new(vulkan.clone(), device_selection.clone(), surface);
        let swapchain = Swapchain::new(&device, &device_selection.physical_device);
        // let rasterizer = Rasterizer::new(device.clone(), swapchain.format);
        let path_tracer = PathTracer::new(
            device,
            swapchain.format,
            &device_selection
                .physical_device
                .ray_tracing_pipeline_properties(),
            &device_selection
                .physical_device
                .acceleration_structure_properties(),
        );
        let presenter = Presenter::new(swapchain);

        KeaApp {
            presenter,
            // rasterizer,
            path_tracer,
        }
    }

    pub fn draw(&self) {
        self.presenter.draw(|cmd, image_view| {
            // self.rasterizer.draw(cmd, image_view);
            self.path_tracer.draw(cmd, image_view);
        });
    }
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let window = Window::new(1920, 1080);
    let app = KeaApp::new(&window);

    println!("ray: {:?}", ray_for_pixel(1000, 700));

    window.event_loop(move || app.draw())
}
