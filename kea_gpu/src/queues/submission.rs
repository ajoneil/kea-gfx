use crate::{commands::RecordedCommandBuffer, sync::Semaphore};
use ash::vk;

pub struct Wait<'a> {
    pub semaphore: &'a Semaphore,
    pub stage: vk::PipelineStageFlags,
}

#[derive(Default)]
pub struct Submission<'a> {
    pub wait: &'a [Wait<'a>],
    pub commands: &'a [RecordedCommandBuffer],
    pub signal_semaphores: &'a [Semaphore],
}
