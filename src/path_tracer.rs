use crate::gpu::{
    buffer::{AllocatedBuffer, Buffer},
    device::Device,
    rt::acceleration_structure::{Aabb, Blas, Geometry},
};
use ash::vk;
use glam::{vec3, Vec3};
use gpu_allocator::MemoryLocation;
use std::{mem, sync::Arc};

pub struct PathTracer {}

struct Sphere {
    position: Vec3,
    radius: f32,
}

impl Sphere {
    pub fn aabb(&self) -> Aabb {
        let Sphere { position, radius } = self;

        Aabb {
            min: vec3(
                position.x - radius,
                position.y - radius,
                position.z - radius,
            ),
            max: vec3(
                position.x + radius,
                position.y + radius,
                position.z + radius,
            ),
        }
    }
}

impl PathTracer {
    pub fn new(device: &Arc<Device>) -> PathTracer {
        Self::build_acceleration_structure(device);
        PathTracer {}
    }

    fn build_acceleration_structure(device: &Arc<Device>) {
        let spheres = [Sphere {
            position: vec3(0.0, 0.0, 1.0),
            radius: 0.5,
        }];

        let (spheres_buffer, aabbs_buffer) = Self::create_buffers(device, &spheres);
        let geometries = [Geometry::aabbs(&aabbs_buffer)];
        let blas = Blas::new(&geometries);
    }

    fn create_buffers(
        device: &Arc<Device>,
        spheres: &[Sphere],
    ) -> (AllocatedBuffer, AllocatedBuffer) {
        let spheres_buffer = Buffer::new(
            device,
            (mem::size_of::<Sphere>() * spheres.len()) as u64,
            vk::BufferUsageFlags::STORAGE_BUFFER,
        );

        let spheres_buffer = spheres_buffer.allocate("spheres", MemoryLocation::CpuToGpu, true);
        spheres_buffer.fill(spheres);

        let aabbs: Vec<Aabb> = spheres.iter().map(|s: &Sphere| s.aabb()).collect();
        let aabbs_buffer = Buffer::new(
            device,
            (mem::size_of::<Aabb>() * aabbs.len()) as u64,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        );

        let aabbs_buffer = aabbs_buffer.allocate("vertices", MemoryLocation::CpuToGpu, true);
        aabbs_buffer.fill(&aabbs);

        (spheres_buffer, aabbs_buffer)
    }
}
