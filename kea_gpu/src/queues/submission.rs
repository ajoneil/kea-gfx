use crate::{commands::RecordedCommandBuffer, sync::Semaphore};
use ash::vk;

pub struct Wait<'a> {
    pub semaphore: &'a Semaphore,
    pub stage: vk::PipelineStageFlags2,
}

pub struct Signal<'a> {
    pub semaphore: &'a Semaphore,
    pub stage: vk::PipelineStageFlags2,
}

#[derive(Default)]
pub struct Submission<'a> {
    pub wait: &'a [Wait<'a>],
    pub commands: &'a [RecordedCommandBuffer],
    pub signal: &'a [Signal<'a>],
}
