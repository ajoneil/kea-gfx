use std::sync::Arc;

use ash::vk;
use env_logger::Env;
use gpu::{Device, RasterizationPipeline, Surface, Swapchain, Vulkan};

use window::Window;

mod gpu;
mod window;

struct KeaApp {
    _vulkan: Arc<Vulkan>,
    device: Arc<Device>,
    _surface: Arc<Surface>,
    swapchain: Swapchain,
    pipeline: RasterizationPipeline,
    framebuffers: Vec<vk::Framebuffer>,
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
    image_available_semaphore: vk::Semaphore,
    render_finished_semaphore: vk::Semaphore,
    in_flight_fence: vk::Fence,
}

impl KeaApp {
    pub fn new(window: &Window) -> KeaApp {
        let vulkan = Arc::new(Vulkan::new(window.required_extensions()));
        let surface = Arc::new(Surface::from_window(&vulkan, &window));
        let device = Arc::new(Device::new(&vulkan, &surface));

        let swapchain = Swapchain::new(&device, &surface);

        let pipeline = RasterizationPipeline::new(&device, swapchain.format);

        let framebuffers =
            Self::create_framebuffers(&device, pipeline.render_pass, &swapchain.image_views);

        let command_pool = Self::create_command_pool(&device, device.queue_family_index);
        let command_buffer = Self::create_command_buffer(&device, command_pool);

        let (image_available_semaphore, render_finished_semaphore, in_flight_fence) =
            Self::create_sync_objects(&device);

        KeaApp {
            _vulkan: vulkan,
            _surface: surface,
            device,
            swapchain,
            pipeline,
            framebuffers,
            command_pool,
            command_buffer,
            image_available_semaphore,
            render_finished_semaphore,
            in_flight_fence,
        }
    }

    fn create_framebuffers(
        device: &Device,
        render_pass: vk::RenderPass,
        image_views: &[vk::ImageView],
    ) -> Vec<vk::Framebuffer> {
        image_views
            .iter()
            .map(|image_view| {
                let attachments = [*image_view];
                let framebuffer = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass)
                    .attachments(&attachments)
                    .width(1920)
                    .height(1080)
                    .layers(1);

                unsafe { device.device.create_framebuffer(&framebuffer, None) }.unwrap()
            })
            .collect()
    }

    fn create_command_pool(device: &Device, queue_family_index: u32) -> vk::CommandPool {
        let command_pool = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_index);
        unsafe { device.device.create_command_pool(&command_pool, None) }.unwrap()
    }

    fn create_command_buffer(device: &Device, command_pool: vk::CommandPool) -> vk::CommandBuffer {
        let command_buffer = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        unsafe { device.device.allocate_command_buffers(&command_buffer) }.unwrap()[0]
    }

    fn record_command_buffer(&self, image_index: u32) {
        let begin_command_buffer = vk::CommandBufferBeginInfo::builder();
        unsafe {
            self.device
                .device
                .begin_command_buffer(self.command_buffer, &begin_command_buffer)
        }
        .unwrap();

        let begin_render_pass = vk::RenderPassBeginInfo::builder()
            .render_pass(self.pipeline.render_pass)
            .framebuffer(self.framebuffers[image_index as usize])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: 1920,
                    height: 1080,
                },
            })
            .clear_values(&[vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            }]);

        unsafe {
            self.device.device.cmd_begin_render_pass(
                self.command_buffer,
                &begin_render_pass,
                vk::SubpassContents::INLINE,
            );

            self.device.device.cmd_bind_pipeline(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            );
            self.device.device.cmd_draw(self.command_buffer, 3, 1, 0, 0);
            self.device.device.cmd_end_render_pass(self.command_buffer);
            self.device.device.end_command_buffer(self.command_buffer)
        }
        .unwrap();
    }

    fn create_sync_objects(device: &Device) -> (vk::Semaphore, vk::Semaphore, vk::Fence) {
        let image_available_semaphore = unsafe {
            device
                .device
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
        }
        .unwrap();
        let render_finished_semaphore = unsafe {
            device
                .device
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
        }
        .unwrap();
        let in_flight_fence = unsafe {
            device.device.create_fence(
                &vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED),
                None,
            )
        }
        .unwrap();

        (
            image_available_semaphore,
            render_finished_semaphore,
            in_flight_fence,
        )
    }

    pub fn draw(&self) {
        unsafe {
            self.device
                .device
                .wait_for_fences(&[self.in_flight_fence], true, u64::MAX)
                .unwrap();
            self.device
                .device
                .reset_fences(&[self.in_flight_fence])
                .unwrap();

            let (image_index, _) = self
                .device
                .ext
                .swapchain
                .acquire_next_image(
                    self.swapchain.swapchain,
                    u64::MAX,
                    self.image_available_semaphore,
                    vk::Fence::null(),
                )
                .unwrap();

            self.device
                .device
                .reset_command_buffer(self.command_buffer, vk::CommandBufferResetFlags::empty())
                .unwrap();

            self.record_command_buffer(image_index);

            let submits = [vk::SubmitInfo::builder()
                .wait_semaphores(&[self.image_available_semaphore])
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .command_buffers(&[self.command_buffer])
                .signal_semaphores(&[self.render_finished_semaphore])
                .build()];

            self.device
                .device
                .queue_submit(self.device.queue, &submits, self.in_flight_fence)
                .unwrap();

            let present = vk::PresentInfoKHR::builder()
                .wait_semaphores(&[self.render_finished_semaphore])
                .swapchains(&[self.swapchain.swapchain])
                .image_indices(&[image_index])
                .build();

            self.device
                .ext
                .swapchain
                .queue_present(self.device.queue, &present)
                .unwrap();
        }
    }
}

impl Drop for KeaApp {
    fn drop(&mut self) {
        unsafe {
            self.device.device.device_wait_idle().unwrap();

            self.device
                .device
                .destroy_semaphore(self.image_available_semaphore, None);
            self.device
                .device
                .destroy_semaphore(self.render_finished_semaphore, None);
            self.device.device.destroy_fence(self.in_flight_fence, None);

            self.device
                .device
                .destroy_command_pool(self.command_pool, None);

            for &framebuffer in self.framebuffers.iter() {
                self.device.device.destroy_framebuffer(framebuffer, None);
            }
        }
    }
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let window = Window::new(1920, 1080);
    let app = KeaApp::new(&window);

    window.event_loop(move || app.draw())
}
