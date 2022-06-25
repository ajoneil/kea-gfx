use env_logger::Env;
use kea::Kea;
use path_tracer::PathTracer;
use window::Window;

mod gpu;
mod kea;
mod path_tracer;
mod presenter;
mod window;

struct KeaApp {
    kea: Kea,
    path_tracer: PathTracer,
}

impl KeaApp {
    pub fn new(window: &Window) -> KeaApp {
        let kea = Kea::new(window);
        let path_tracer = PathTracer::new(&kea);

        KeaApp { kea, path_tracer }
    }

    pub fn draw(&self) {
        self.kea.presenter().draw(|cmd, image_view| {
            self.path_tracer.draw(cmd, image_view);
        });
    }
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let window = Window::new(1920, 1080);
    let app = KeaApp::new(&window);

    window.event_loop(move || app.draw())
}
