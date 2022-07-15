use ash::vk;
use glam::vec3;
use gpu_allocator::MemoryLocation;
use kea_gpu::{
    device::Device,
    ray_tracing::scenes::{Geometry, GeometryInstance, GeometryType, Scene},
    storage::buffers::Buffer,
};
use kea_gpu_shaderlib::Aabb;
use kea_renderer_shaders::{boxes::Boxo, spheres::Sphere};
use std::sync::Arc;

pub fn basic_shapes(device: Arc<Device>) -> Scene {
    let mut scene = Scene::new(device.clone(), "test scene".to_string());

    let spheres = [
        Sphere::new(vec3(0.0, -1000.0, -1.5), 1000.0),
        Sphere::new(vec3(0.0, 0.8, -2.8), 0.8),
        Sphere::new(vec3(-0.9, 0.4, -2.0), 0.4),
        Sphere::new(vec3(0.7, 1.3, -1.9), 0.3),
    ];

    let (spheres_buffer, aabbs_buffer) = create_buffers(&device, &spheres);
    let mut geometry = Geometry::new(
        device.clone(),
        "spheres".to_string(),
        GeometryType::Aabbs(aabbs_buffer),
        Some(Arc::new(spheres_buffer)),
    );
    geometry.build();
    let geometry_instance = GeometryInstance::new(Arc::new(geometry), 1);
    scene.add_instance(geometry_instance);

    let boxes = [
        Boxo::new(vec3(1.1, 0.5, -1.8), vec3(1.0, 1.0, 1.0)),
        Boxo::new(vec3(-0.1, 0.3, -1.7), vec3(0.6, 0.6, 0.6)),
    ];
    let (boxes_buffer, aabbs_buffer) = create_boxes_buffers(&device, &boxes);
    let mut geometry = Geometry::new(
        device,
        "boxes".to_string(),
        GeometryType::Aabbs(aabbs_buffer),
        Some(Arc::new(boxes_buffer)),
    );
    geometry.build();
    let geometry_instance = GeometryInstance::new(Arc::new(geometry), 2);
    scene.add_instance(geometry_instance);

    scene.build();

    scene
}

fn create_buffers(device: &Arc<Device>, spheres: &[Sphere]) -> (Buffer, Buffer) {
    let spheres_buffer = Buffer::new_from_data(
        device.clone(),
        spheres,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        "spheres".to_string(),
        MemoryLocation::GpuOnly,
        None,
    );
    log::info!("spheres data {:?}", spheres);

    let aabbs: Vec<Aabb> = spheres.iter().map(|s: &Sphere| s.aabb()).collect();
    log::debug!("Aabbs: {:?}", aabbs);
    let aabbs_buffer = Buffer::new_from_data(
        device.clone(),
        &aabbs,
        vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        "aabbs".to_string(),
        MemoryLocation::GpuOnly,
        None,
    );

    (spheres_buffer, aabbs_buffer)
}

fn create_boxes_buffers(device: &Arc<Device>, boxes: &[Boxo]) -> (Buffer, Buffer) {
    let boxes_buffer = Buffer::new_from_data(
        device.clone(),
        boxes,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        "boxes".to_string(),
        MemoryLocation::GpuOnly,
        None,
    );
    log::info!("boxes data {:?}", boxes);

    let aabbs: Vec<Aabb> = boxes.iter().map(|b: &Boxo| b.aabb()).collect();
    log::debug!("Aabbs: {:?}", aabbs);
    let aabbs_buffer = Buffer::new_from_data(
        device.clone(),
        &aabbs,
        vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        "aabbs".to_string(),
        MemoryLocation::GpuOnly,
        None,
    );

    (boxes_buffer, aabbs_buffer)
}
