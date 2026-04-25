use std::sync::Arc;

pub struct Window {
    window: Arc<winit::window::Window>,
}

impl Window {
    pub fn new(window: Arc<winit::window::Window>) -> Window {
        Window { window }
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }
}
