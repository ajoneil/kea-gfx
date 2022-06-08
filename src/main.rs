use std::{ffi::CStr, os::raw::c_char};

use ash::{vk, Device, Entry, Instance};
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
    _device: Device,
    _gfx_queue: vk::Queue,
}

impl KeaApp {
    pub fn new(window: &Window) -> KeaApp {
        let entry = Entry::linked();
        let instance = Self::create_instance(&entry, window);
        let (physical_device, gfx_queue_family_idx) = Self::select_physical_device(&instance);
        let (device, gfx_queue) = Self::create_logical_device_with_queue(
            &instance,
            physical_device,
            gfx_queue_family_idx,
        );

        KeaApp {
            _entry: entry,
            instance,
            _physical_device: physical_device,
            _device: device,
            _gfx_queue: gfx_queue,
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

    fn select_physical_device(instance: &Instance) -> (vk::PhysicalDevice, u32) {
        let devices = unsafe { instance.enumerate_physical_devices() }.unwrap();
        let (device, gfx_queue_family_idx) = devices
            .into_iter()
            .find_map(
                |device| match Self::find_gfx_queue_family_idx(instance, device) {
                    Some(idx) => Some((device, idx)),
                    None => None,
                },
            )
            .unwrap();

        let props = unsafe { instance.get_physical_device_properties(device) };
        info!("Selected physical device: {:?}", unsafe {
            CStr::from_ptr(props.device_name.as_ptr())
        });

        (device, gfx_queue_family_idx)
    }

    fn find_gfx_queue_family_idx(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
    ) -> Option<u32> {
        let props =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        props
            .iter()
            .enumerate()
            .find(|(_, family)| {
                family.queue_count > 0 && family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            })
            .map(|(index, _)| index as _)
    }

    fn create_logical_device_with_queue(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        gfx_queue_family_idx: u32,
    ) -> (Device, vk::Queue) {
        let queue_create_infos = [vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(gfx_queue_family_idx)
            .queue_priorities(&[1.0])
            .build()];

        let create_info = vk::DeviceCreateInfo::builder().queue_create_infos(&queue_create_infos);
        let device =
            unsafe { instance.create_device(physical_device, &create_info, None) }.unwrap();
        let graphics_queue = unsafe { device.get_device_queue(gfx_queue_family_idx, 0) };

        (device, graphics_queue)
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
