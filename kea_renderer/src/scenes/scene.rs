use ash::vk;
use bevy_ecs::prelude::*;
use glam::{vec3a, Affine3A, Vec3, Vec3A};
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
    meshes: Option<Arc<Buffer>>,
    vertices: Option<Arc<Buffer>>,
    indices: Option<Arc<Buffer>>,
}

#[derive(Component)]
pub struct Position(pub Vec3);

#[derive(Component)]
pub struct Scale(pub Vec3);

#[derive(Component)]
pub struct Sphere {
    radius: f32,
}

#[derive(Component)]
pub struct Material(pub kea_renderer_shaders::materials::Material);

#[derive(Component)]
pub struct Mesh {
    vertices: Vec<Vec3A>,
    indices: Vec<[u32; 3]>,
}

impl Scene {
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            device,
            world: World::new(),
            gpu_scene: None,
            spheres: None,
            meshes: None,
            vertices: None,
            indices: None,
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
        let vertices = vec![
            vec3a(0.5, -0.5, 0.5),
            vec3a(0.5, -0.5, -0.5),
            vec3a(0.5, 0.5, -0.5),
            vec3a(0.5, 0.5, 0.5),
            vec3a(-0.5, -0.5, 0.5),
            vec3a(-0.5, -0.5, -0.5),
            vec3a(-0.5, 0.5, -0.5),
            vec3a(-0.5, 0.5, 0.5),
        ];
        let indices = vec![
            [4, 0, 3],
            [4, 3, 7],
            [0, 1, 2],
            [0, 2, 3],
            [1, 5, 6],
            [1, 6, 2],
            [5, 4, 7],
            [5, 7, 6],
            [7, 3, 2],
            [7, 2, 6],
            [0, 5, 1],
            [0, 4, 5],
        ];

        self.world
            .spawn()
            .insert(Position(position))
            .insert(Scale(scale))
            .insert(Material(material))
            .insert(Mesh { vertices, indices });
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
            let geometry_instance =
                GeometryInstance::new(Arc::new(geometry), 1, Affine3A::IDENTITY, 0);
            scene.add_instance(geometry_instance);
        }

        let mut meshes: Vec<kea_renderer_shaders::triangles::Mesh> = vec![];
        let mut all_vertices: Vec<Vec3A> = vec![];
        let mut all_indices: Vec<[u32; 3]> = vec![];

        for (mesh, position, scale, material) in self
            .world
            .query::<(&Mesh, &Position, &Scale, &Material)>()
            .iter(&self.world)
        {
            let vertices = Buffer::new_from_data(
                self.device.clone(),
                &mesh.vertices,
                vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
                    | vk::BufferUsageFlags::STORAGE_BUFFER,
                "vertices".to_string(),
                MemoryLocation::GpuOnly,
                None,
            );
            //let vertices_address = vertices.device_address();
            let vertices_offset = all_vertices.len();
            all_vertices.extend(&mesh.vertices);

            let indices = Buffer::new_from_data(
                self.device.clone(),
                &mesh.indices,
                vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
                    | vk::BufferUsageFlags::STORAGE_BUFFER,
                "indices".to_string(),
                MemoryLocation::GpuOnly,
                None,
            );
            //let indices_address = indices.device_address();
            let indices_offset = all_indices.len();
            all_indices.extend(&mesh.indices);

            let mut geometry = Geometry::new(
                self.device.clone(),
                "triangle mesh".to_string(),
                GeometryType::Triangles { vertices, indices },
            );

            geometry.build();

            let transform = Affine3A::from_translation(position.0) * Affine3A::from_scale(scale.0);
            let geometry_instance =
                GeometryInstance::new(Arc::new(geometry), 0, transform, meshes.len() as _);
            scene.add_instance(geometry_instance);

            meshes.push(kea_renderer_shaders::triangles::Mesh {
                // vertices_address,
                // indices_address,
                vertices_offset: vertices_offset as _,
                indices_offset: indices_offset as _,
                material: material.0,
            });
        }

        if !meshes.is_empty() {
            self.meshes = Some(Arc::new(Buffer::new_from_data(
                self.device.clone(),
                &meshes,
                vk::BufferUsageFlags::STORAGE_BUFFER,
                "meshes".to_string(),
                MemoryLocation::GpuOnly,
                None,
            )));

            self.vertices = Some(Arc::new(Buffer::new_from_data(
                self.device.clone(),
                &all_vertices,
                vk::BufferUsageFlags::STORAGE_BUFFER,
                "all vertices".to_string(),
                MemoryLocation::GpuOnly,
                None,
            )));

            self.indices = Some(Arc::new(Buffer::new_from_data(
                self.device.clone(),
                &all_indices,
                vk::BufferUsageFlags::STORAGE_BUFFER,
                "all indices".to_string(),
                MemoryLocation::GpuOnly,
                None,
            )));
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

        if let Some(meshes) = self.meshes.as_ref() {
            slot_bindings.bind_buffer(SlotId::Meshes, meshes.clone());
            slot_bindings.bind_buffer(SlotId::Vertices, self.vertices.as_ref().unwrap().clone());
            slot_bindings.bind_buffer(SlotId::Indices, self.indices.as_ref().unwrap().clone());
        }
    }
}
