use super::{buffers::Buffer, images::Image};
use crate::commands::CommandBufferRecorder;
use ash::vk;
use std::slice;

impl<'a> CommandBufferRecorder<'a> {
    pub fn copy_buffer(&self, source: &'a Buffer, destination: &'a Buffer) {
        assert!(destination.size() == source.size());

        let copy = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: source.size() as _,
        };

        unsafe {
            self.device().raw().cmd_copy_buffer(
                self.buffer().raw(),
                source.raw(),
                destination.raw(),
                slice::from_ref(&copy),
            );
        }
    }

    pub fn transition_image_layout(
        &self,
        image: &Image,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
        src_access_mask: vk::AccessFlags,
        dst_access_mask: vk::AccessFlags,
        src_stage_mask: vk::PipelineStageFlags,
        dst_stage_mask: vk::PipelineStageFlags,
    ) {
        let barrier = vk::ImageMemoryBarrier::builder()
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask)
            .old_layout(old_layout)
            .new_layout(new_layout)
            .image(unsafe { image.raw() })
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .level_count(1)
                    .layer_count(1)
                    .build(),
            );

        self.pipeline_barrier(
            src_stage_mask,
            dst_stage_mask,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            slice::from_ref(&barrier),
        );
    }

    pub fn copy_image(&self, from: &Image, to: &Image, region: &vk::ImageCopy) {
        unsafe {
            self.device().raw().cmd_copy_image(
                self.buffer().raw(),
                from.raw(),
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                to.raw(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                std::slice::from_ref(region),
            )
        };
    }
}
