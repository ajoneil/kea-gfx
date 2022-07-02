use super::buffers::{AlignedBuffer, Buffer};
use crate::core::command::CommandBufferRecorder;
use ash::vk;
use std::slice;

impl<'a> CommandBufferRecorder<'a> {
    pub fn copy_buffer(&self, source: &'a Buffer, destination: &'a Buffer) {
        assert!(destination.size() >= source.size());

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

    pub fn copy_buffer_aligned(&self, source: &'a Buffer, destination: &'a AlignedBuffer) {
        assert!(destination.size() >= source.size() as _);

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
}
