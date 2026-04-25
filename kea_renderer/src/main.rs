use env_logger::Env;
use kea_gpu::debug::DebugFeature;
use kea_gpu::presentation::Window;
use kea_gpu::ray_tracing::RayTracingFeature;
use kea_gpu::Kea;
use path_tracer::PathTracer;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Fullscreen, WindowId};

mod path_tracer;
mod scenes;

struct InitConfig {
    size: (u32, u32),
    fullscreen: bool,
}

struct State {
    window: Arc<winit::window::Window>,
    path_tracer: PathTracer,
}

struct App {
    init: InitConfig,
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let attributes = winit::window::Window::default_attributes()
            .with_title("kea")
            .with_inner_size(PhysicalSize::new(self.init.size.0, self.init.size.1))
            .with_resizable(false)
            .with_fullscreen(if self.init.fullscreen {
                Some(Fullscreen::Borderless(None))
            } else {
                None
            });

        let window = Arc::new(event_loop.create_window(attributes).unwrap());
        let kea_window = Window::new(window.clone());
        let kea = Kea::new(
            &kea_window,
            self.init.size,
            vec![
                Box::new(RayTracingFeature::new()),
                Box::new(DebugFeature::new()),
            ],
        );
        let path_tracer = PathTracer::new(kea);

        window.request_redraw();
        self.state = Some(State { window, path_tracer });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = self.state.as_ref() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                state.path_tracer.draw();
                state.window.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let event_loop = EventLoop::new().unwrap();
    let mut app = App {
        init: InitConfig {
            size: (1280, 720),
            fullscreen: false,
        },
        state: None,
    };
    event_loop.run_app(&mut app).unwrap();
}
