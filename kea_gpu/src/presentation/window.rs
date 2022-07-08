use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};

pub struct Window {
    window: winit::window::Window,
    event_loop: winit::event_loop::EventLoop<()>,
}

impl Window {
    pub fn new(width: u32, height: u32, fullscreen: bool) -> Window {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("kea")
            .with_inner_size(PhysicalSize::new(width, height))
            .with_resizable(false)
            .with_fullscreen(if fullscreen {
                Some(Fullscreen::Borderless(None))
            } else {
                None
            })
            .build(&event_loop)
            .expect("Failed to create window");

        Window { window, event_loop }
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    pub fn event_loop<F: 'static + Fn() -> ()>(self, draw: F) {
        self.event_loop
            .run(move |event, _, control_flow| match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::MainEventsCleared => draw(),
                _ => (),
            });
    }
}
