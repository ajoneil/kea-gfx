use std::{ffi::CStr, fs::File, os::raw::c_char};

use ash::{
    extensions::khr::{Surface, Swapchain},
    util::read_spv,
    vk, Device, Entry, Instance,
};
use env_logger::Env;
use log::info;
use spirv_builder::{MetadataPrintout, SpirvBuilder};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

struct KeaApp {
    _entry: Entry,
    instance: Instance,
    surface_loader: Surface,
    surface: vk::SurfaceKHR,
    _physical_device: vk::PhysicalDevice,
    device: Device,
    _present_queue: vk::Queue,
    swapchain_loader: Swapchain,
    swapchain: vk::SwapchainKHR,
    _swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
    shader_module: vk::ShaderModule,
}

impl KeaApp {
    pub fn new(window: &Window) -> KeaApp {
        let entry = Entry::linked();
        let instance = Self::create_instance(&entry, window);

        let surface_loader = Surface::new(&entry, &instance);
        let surface =
            unsafe { ash_window::create_surface(&entry, &instance, window, None) }.unwrap();

        let (physical_device, queue_family_index) =
            Self::select_physical_device(&instance, surface, &surface_loader);
        let (device, present_queue) =
            Self::create_logical_device_with_queue(&instance, physical_device, queue_family_index);

        let swapchain_loader = Swapchain::new(&instance, &device);
        let (swapchain, format) =
            Self::create_swapchain(surface, physical_device, &swapchain_loader, &surface_loader);
        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }.unwrap();
        let swapchain_image_views =
            Self::create_swapchain_image_views(&swapchain_images, format, &device);

        let shader_module = Self::create_shader_module(&device);

        KeaApp {
            _entry: entry,
            instance,
            surface_loader,
            surface,
            _physical_device: physical_device,
            device,
            _present_queue: present_queue,
            swapchain_loader,
            swapchain,
            _swapchain_images: swapchain_images,
            swapchain_image_views,
            shader_module,
        }
    }

    fn create_instance(entry: &Entry, window: &Window) -> Instance {
        let app_info = vk::ApplicationInfo::builder().api_version(vk::API_VERSION_1_3);
        let extension_names = Self::instance_extension_names(window);

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

    fn instance_extension_names(window: &Window) -> Vec<*const i8> {
        ash_window::enumerate_required_extensions(window)
            .unwrap()
            .to_vec()
    }

    fn device_extension_names() -> Vec<*const i8> {
        vec![Swapchain::name().as_ptr()]
    }

    fn select_physical_device(
        instance: &Instance,
        surface: vk::SurfaceKHR,
        surface_loader: &Surface,
    ) -> (vk::PhysicalDevice, u32) {
        let devices = unsafe { instance.enumerate_physical_devices() }.unwrap();
        let (device, queue_family_index) = devices
            .into_iter()
            .find_map(|device| {
                match Self::find_queue_family_index(instance, device, surface, surface_loader) {
                    Some(index) => Some((device, index)),
                    None => None,
                }
            })
            .unwrap();

        let props = unsafe { instance.get_physical_device_properties(device) };
        info!("Selected physical device: {:?}", unsafe {
            CStr::from_ptr(props.device_name.as_ptr())
        });

        (device, queue_family_index)
    }

    fn find_queue_family_index(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        surface_loader: &Surface,
    ) -> Option<u32> {
        let props =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        props
            .iter()
            .enumerate()
            .find(|(index, family)| {
                family.queue_count > 0
                    && family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                    && unsafe {
                        surface_loader.get_physical_device_surface_support(
                            physical_device,
                            *index as u32,
                            surface,
                        )
                    }
                    .unwrap()
            })
            .map(|(index, _)| index as _)
    }

    fn create_logical_device_with_queue(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> (Device, vk::Queue) {
        let queue_create_infos = [vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&[1.0])
            .build()];
        let extension_names = Self::device_extension_names();
        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&extension_names);

        let device =
            unsafe { instance.create_device(physical_device, &create_info, None) }.unwrap();
        let present_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        (device, present_queue)
    }

    fn create_swapchain(
        surface: vk::SurfaceKHR,
        physical_device: vk::PhysicalDevice,
        swapchain_loader: &Swapchain,
        surface_loader: &Surface,
    ) -> (vk::SwapchainKHR, vk::Format) {
        let surface_capabilities = unsafe {
            surface_loader.get_physical_device_surface_capabilities(physical_device, surface)
        }
        .unwrap();

        let image_count = surface_capabilities.min_image_count + 1;
        let image_count = if surface_capabilities.max_image_count > 0 {
            image_count.min(surface_capabilities.max_image_count)
        } else {
            image_count
        };

        let surface_format =
            unsafe { surface_loader.get_physical_device_surface_formats(physical_device, surface) }
                .unwrap()[0];

        let present_mode = unsafe {
            surface_loader.get_physical_device_surface_present_modes(physical_device, surface)
        }
        .unwrap()
        .iter()
        .cloned()
        .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
        .unwrap_or(vk::PresentModeKHR::FIFO);

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(vk::Extent2D {
                width: 1920,
                height: 1080,
            })
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .image_array_layers(1)
            .present_mode(present_mode);

        let swapchain =
            unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None) }.unwrap();
        (swapchain, surface_format.format)
    }

    fn create_swapchain_image_views(
        swapchain_images: &[vk::Image],
        format: vk::Format,
        device: &Device,
    ) -> Vec<vk::ImageView> {
        swapchain_images
            .iter()
            .map(|&image| {
                let imageview_create_info = vk::ImageViewCreateInfo::builder()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });
                unsafe { device.create_image_view(&imageview_create_info, None) }.unwrap()
            })
            .collect()
    }

    fn compile_shaders() -> Vec<u32> {
        let compiled_shader_path = SpirvBuilder::new("src/shaders", "spirv-unknown-vulkan1.2")
            .print_metadata(MetadataPrintout::None)
            .build()
            .unwrap()
            .module
            .unwrap_single()
            .to_path_buf();

        read_spv(&mut File::open(compiled_shader_path).unwrap()).unwrap()
    }

    fn create_shader_module(device: &Device) -> vk::ShaderModule {
        let compiled_shaders = Self::compile_shaders();
        let shader_create_info = vk::ShaderModuleCreateInfo::builder().code(&compiled_shaders);

        unsafe { device.create_shader_module(&shader_create_info, None) }.unwrap()
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
            self.device.destroy_shader_module(self.shader_module, None);
            for &image_view in self.swapchain_image_views.iter() {
                self.device.destroy_image_view(image_view, None);
            }

            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("kea")
        .with_inner_size(PhysicalSize::new(1920 as u32, 1080 as u32))
        .with_resizable(false)
        .build(&event_loop)
        .expect("Failed to create window");

    let app = KeaApp::new(&window);
    app.run(event_loop, window);
}
