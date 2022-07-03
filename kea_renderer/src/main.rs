use env_logger::Env;
use kea_gpu::debug::DebugFeature;
use kea_gpu::presentation::Window;
use kea_gpu::ray_tracing::RayTracingFeature;
use kea_gpu::Kea;
use path_tracer::PathTracer;

mod path_tracer;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    let size = (1280, 720);
    let window = Window::new(size.0, size.1);
    let kea = Kea::new(
        &window,
        size,
        vec![
            Box::new(RayTracingFeature::new()),
            Box::new(DebugFeature::new()),
        ],
    );
    let path_tracer = PathTracer::new(kea);

    window.event_loop(move || path_tracer.draw())
}
