use std::{ffi::CStr, fs::File, sync::Arc};

use ash::{
    util::read_spv,
    vk::{self, PipelineLayoutCreateInfo},
};
use env_logger::Env;
use gpu::{Device, Surface, Swapchain, Vulkan};
use spirv_builder::{MetadataPrintout, SpirvBuilder};
use window::Window;

mod gpu;
mod window;

struct KeaApp {
    _vulkan: Arc<Vulkan>,
    device: Arc<Device>,
    _surface: Surface,
    swapchain: Swapchain,
    _swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
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
        let surface = Surface::from_window(&vulkan, &window);
        let device = Arc::new(Device::new(&vulkan, &surface));

        let swapchain = Swapchain::new(&device, &surface);
        let swapchain_images = unsafe {
            device
                .ext
                .swapchain
                .get_swapchain_images(swapchain.swapchain)
        }
        .unwrap();
        let swapchain_image_views =
            Self::create_swapchain_image_views(&swapchain_images, swapchain.format, &device);

        let render_pass = Self::create_renderpass(&device, swapchain.format);
        let (pipeline, pipeline_layout) = Self::create_pipeline(&device, render_pass);

        let framebuffers = Self::create_framebuffers(&device, render_pass, &swapchain_image_views);

        let command_pool = Self::create_command_pool(&device, device.queue_family_index);
        let command_buffer = Self::create_command_buffer(&device, command_pool);

        let (image_available_semaphore, render_finished_semaphore, in_flight_fence) =
            Self::create_sync_objects(&device);

        KeaApp {
            _vulkan: vulkan,
            _surface: surface,
            device,
            swapchain,
            _swapchain_images: swapchain_images,
            swapchain_image_views,
            render_pass,
            pipeline_layout,
            pipeline,
            framebuffers,
            command_pool,
            command_buffer,
            image_available_semaphore,
            render_finished_semaphore,
            in_flight_fence,
        }
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
                unsafe {
                    device
                        .device
                        .create_image_view(&imageview_create_info, None)
                }
                .unwrap()
            })
            .collect()
    }

    fn create_renderpass(device: &Device, format: vk::Format) -> vk::RenderPass {
        let attachments = [vk::AttachmentDescription::builder()
            .format(format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build()];

        let color_attachments = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::ATTACHMENT_OPTIMAL,
        }];

        let subpasses = [vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments)
            .build()];

        let dependencies = [vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .build()];

        let create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        unsafe { device.device.create_render_pass(&create_info, None) }.unwrap()
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

        unsafe {
            device
                .device
                .create_shader_module(&shader_create_info, None)
        }
        .unwrap()
    }

    fn create_pipeline(
        device: &Device,
        render_pass: vk::RenderPass,
    ) -> (vk::Pipeline, vk::PipelineLayout) {
        let shader_module = Self::create_shader_module(device);
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(shader_module)
                .name(unsafe { CStr::from_bytes_with_nul_unchecked(b"main_vertex\0") })
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(shader_module)
                .name(unsafe { CStr::from_bytes_with_nul_unchecked(b"main_fragment\0") })
                .build(),
        ];

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder();
        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D {
                width: 1920,
                height: 1080,
            },
        }];

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .min_sample_shading(1.0)
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false);

        let color_blend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ZERO)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .build()];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&color_blend_attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0])
            .build();

        let pipeline_layout = unsafe {
            device
                .device
                .create_pipeline_layout(&PipelineLayoutCreateInfo::builder(), None)
        }
        .unwrap();

        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .color_blend_state(&color_blend_state)
            .render_pass(render_pass)
            .layout(pipeline_layout);

        let pipelines = unsafe {
            device.device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[pipeline_info.build()],
                None,
            )
        }
        .unwrap();

        unsafe {
            device.device.destroy_shader_module(shader_module, None);
        }

        (pipelines[0], pipeline_layout)
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
            .render_pass(self.render_pass)
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
                self.pipeline,
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

            self.device.device.destroy_pipeline(self.pipeline, None);
            self.device
                .device
                .destroy_render_pass(self.render_pass, None);
            self.device
                .device
                .destroy_pipeline_layout(self.pipeline_layout, None);

            for &image_view in self.swapchain_image_views.iter() {
                self.device.device.destroy_image_view(image_view, None);
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
