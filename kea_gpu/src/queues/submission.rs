use crate::{commands::RecordedCommandBuffer, sync::Semaphore};
use ash::vk;

pub struct Wait<'a> {
    pub semaphore: &'a Semaphore,
    pub stage: vk::PipelineStageFlags2,
    /// Timeline value to wait for. Ignored for binary semaphores.
    pub value: u64,
}

pub struct Signal<'a> {
    pub semaphore: &'a Semaphore,
    pub stage: vk::PipelineStageFlags2,
    /// Timeline value to signal. Ignored for binary semaphores.
    pub value: u64,
}

#[derive(Default)]
pub struct Submission<'a> {
    pub wait: &'a [Wait<'a>],
    pub commands: &'a [RecordedCommandBuffer],
    pub signal: &'a [Signal<'a>],
}
