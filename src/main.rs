use std::ffi::CString;

use ash::{vk, Entry, Instance};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

struct KeaApp {
    entry: Entry,
    instance: Instance,
}

impl KeaApp {
    pub fn new() -> KeaApp {
        let entry = Entry::linked();
        let instance = Self::create_instance(&entry);

        KeaApp { entry, instance }
    }

    fn create_instance(entry: &Entry) -> Instance {
        let app_info = vk::ApplicationInfo {
            api_version: vk::API_VERSION_1_3,
            ..Default::default()
        };
        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            ..Default::default()
        };

        unsafe { entry.create_instance(&create_info, None).unwrap() }
    }

    pub fn run(mut self, event_loop: EventLoop<()>, window: Window) {
        event_loop.run(|event, _, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        });
    }
}

impl Drop for KeaApp {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("kea")
        .with_inner_size(LogicalSize::new(1920 as u32, 1080 as u32))
        .build(&event_loop)
        .expect("Failed to create window");

    let app = KeaApp::new();
    app.run(event_loop, window);
}
