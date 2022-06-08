use std::{ffi::CStr, os::raw::c_char};

use ash::{vk, Entry, Instance};
use env_logger::Env;
use log::info;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

struct KeaApp {
    _entry: Entry,
    instance: Instance,
    _physical_device: vk::PhysicalDevice,
}

impl KeaApp {
    pub fn new(window: &Window) -> KeaApp {
        let entry = Entry::linked();
        let instance = Self::create_instance(&entry, window);
        let physical_device = Self::select_physical_device(&instance);

        KeaApp {
            _entry: entry,
            instance,
            _physical_device: physical_device,
        }
    }

    fn create_instance(entry: &Entry, window: &Window) -> Instance {
        let app_info = vk::ApplicationInfo::builder().api_version(vk::API_VERSION_1_3);
        let extension_names = Self::extension_names(window);

        let layer_names = unsafe {
            [CStr::from_bytes_with_nul_unchecked(
                b"VK_LAYER_KHRONOS_validation\0",
            )]
        };

        let layers_names_raw: Vec<*const c_char> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names)
            .enabled_layer_names(&layers_names_raw);

        unsafe { entry.create_instance(&create_info, None).unwrap() }
    }

    fn extension_names(window: &Window) -> Vec<*const i8> {
        ash_window::enumerate_required_extensions(window)
            .unwrap()
            .to_vec()
    }

    fn select_physical_device(instance: &Instance) -> vk::PhysicalDevice {
        let devices = unsafe { instance.enumerate_physical_devices() }.unwrap();
        let device = devices[0];

        let props = unsafe { instance.get_physical_device_properties(device) };
        info!("Selected physical device: {:?}", unsafe {
            CStr::from_ptr(props.device_name.as_ptr())
        });

        device
    }

    pub fn run(self, event_loop: EventLoop<()>, _window: Window) {
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
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("kea")
        .with_inner_size(LogicalSize::new(1920 as u32, 1080 as u32))
        .build(&event_loop)
        .expect("Failed to create window");

    let app = KeaApp::new(&window);
    app.run(event_loop, window);
}
