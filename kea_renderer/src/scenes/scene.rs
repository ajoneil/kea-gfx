use ash::vk;
use bevy_ecs::prelude::*;
use glam::Vec3;
use gpu_allocator::MemoryLocation;
use kea_gpu::{
    device::Device,
    ray_tracing::scenes::{Geometry, GeometryInstance, GeometryType},
    slots::SlotBindings,
    storage::buffers::Buffer,
};
use kea_gpu_shaderlib::Aabb;
use kea_renderer_shaders::SlotId;
use std::sync::Arc;

pub struct Scene {
    device: Arc<Device>,
    world: World,
    gpu_scene: Option<kea_gpu::ray_tracing::scenes::Scene>,
    spheres: Option<Arc<Buffer>>,
    boxes: Option<Arc<Buffer>>,
}

#[derive(Component)]
pub struct Position(pub Vec3);

#[derive(Component)]
pub struct Sphere {
    radius: f32,
}

#[derive(Component)]
pub struct Boxo {
    scale: Vec3,
}

#[derive(Component)]
pub struct Material(pub kea_renderer_shaders::materials::Material);

impl Scene {
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            device,
            world: World::new(),
            gpu_scene: None,
            spheres: None,
            boxes: None,
        }
    }

    pub fn add_sphere(
        &mut self,
        position: Vec3,
        radius: f32,
        material: kea_renderer_shaders::materials::Material,
    ) {
        self.world
            .spawn()
            .insert(Position(position))
            .insert(Sphere { radius })
            .insert(Material(material));
    }

    pub fn add_box(
        &mut self,
        position: Vec3,
        scale: Vec3,
        material: kea_renderer_shaders::materials::Material,
    ) {
        self.world
            .spawn()
            .insert(Position(position))
            .insert(Boxo { scale })
            .insert(Material(material));
    }

    pub fn build_scene(&mut self) {
        let mut scene = kea_gpu::ray_tracing::scenes::Scene::new(
            self.device.clone(),
            "kea renderer scene".to_string(),
        );

        let spheres: Vec<kea_renderer_shaders::spheres::Sphere> = self
            .world
            .query::<(&Position, &Sphere, &Material)>()
            .iter(&self.world)
            .map(|(position, sphere, material)| {
                kea_renderer_shaders::spheres::Sphere::new(position.0, sphere.radius, material.0)
            })
            .collect();

        if spheres.len() > 0 {
            let spheres_buffer = Buffer::new_from_data(
                self.device.clone(),
                &spheres,
                vk::BufferUsageFlags::STORAGE_BUFFER,
                "spheres".to_string(),
                MemoryLocation::GpuOnly,
                None,
            );
            log::info!("spheres data {:?}", spheres);

            let aabbs: Vec<Aabb> = spheres
                .iter()
                .map(|s: &kea_renderer_shaders::spheres::Sphere| s.aabb())
                .collect();
            log::debug!("Aabbs: {:?}", aabbs);
            let aabbs_buffer = Buffer::new_from_data(
                self.device.clone(),
                &aabbs,
                vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
                "aabbs".to_string(),
                MemoryLocation::GpuOnly,
                None,
            );

            self.spheres = Some(Arc::new(spheres_buffer));

            let mut geometry = Geometry::new(
                self.device.clone(),
                "spheres".to_string(),
                GeometryType::Aabbs(aabbs_buffer),
            );
            geometry.build();
            let geometry_instance = GeometryInstance::new(Arc::new(geometry), 1);
            scene.add_instance(geometry_instance);
        }

        let boxes: Vec<kea_renderer_shaders::boxes::Boxo> = self
            .world
            .query::<(&Position, &Boxo, &Material)>()
            .iter(&self.world)
            .map(|(position, boxo, material)| {
                kea_renderer_shaders::boxes::Boxo::new(position.0, boxo.scale, material.0)
            })
            .collect();

        if boxes.len() > 0 {
            let boxes_buffer = Buffer::new_from_data(
                self.device.clone(),
                &boxes,
                vk::BufferUsageFlags::STORAGE_BUFFER,
                "boxes".to_string(),
                MemoryLocation::GpuOnly,
                None,
            );
            log::info!("boxes data {:?}", boxes);

            let aabbs: Vec<Aabb> = boxes.iter().map(|b| b.aabb()).collect();
            log::debug!("Aabbs: {:?}", aabbs);
            let aabbs_buffer = Buffer::new_from_data(
                self.device.clone(),
                &aabbs,
                vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
                "aabbs".to_string(),
                MemoryLocation::GpuOnly,
                None,
            );

            self.boxes = Some(Arc::new(boxes_buffer));
            let mut geometry = Geometry::new(
                self.device.clone(),
                "boxes".to_string(),
                GeometryType::Aabbs(aabbs_buffer),
            );
            geometry.build();
            let geometry_instance = GeometryInstance::new(Arc::new(geometry), 2);
            scene.add_instance(geometry_instance);
        }

        scene.build();
        self.gpu_scene = Some(scene);
    }

    pub fn bind_data(&self, slot_bindings: &mut SlotBindings<SlotId>) {
        slot_bindings.bind_acceleration_structure(
            SlotId::Scene,
            self.gpu_scene
                .as_ref()
                .unwrap()
                .acceleration_structure()
                .clone(),
        );

        if let Some(spheres) = self.spheres.as_ref() {
            slot_bindings.bind_buffer(SlotId::Spheres, spheres.clone());
        }

        if let Some(boxes) = self.boxes.as_ref() {
            slot_bindings.bind_buffer(SlotId::Boxes, boxes.clone());
        }
    }
}
