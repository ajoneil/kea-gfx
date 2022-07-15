use super::Scene;
use glam::vec3;
use kea_gpu::device::Device;
use std::sync::Arc;

pub fn basic_shapes(device: Arc<Device>) -> Scene {
    let mut scene = Scene::new(device);

    scene.add_sphere(vec3(0.0, -1000.0, -1.5), 1000.0);
    scene.add_sphere(vec3(0.0, 0.8, -2.8), 0.8);
    scene.add_sphere(vec3(-0.9, 0.4, -2.0), 0.4);
    scene.add_sphere(vec3(0.7, 1.3, -1.9), 0.3);
    scene.add_box(vec3(1.1, 0.5, -1.8), vec3(1.0, 1.0, 1.0));
    scene.add_box(vec3(-0.1, 0.3, -1.7), vec3(0.6, 0.6, 0.6));

    scene.build_scene();

    scene
}

pub fn cornell_box(device: Arc<Device>) -> Scene {
    let mut scene = Scene::new(device);

    // Walls
    scene.add_box(vec3(-2.0, 1.0, -1.0), vec3(2.0, 2.0, 2.0));
    scene.add_box(vec3(2.0, 1.0, -1.0), vec3(2.0, 2.0, 2.0));
    scene.add_box(vec3(0.0, 1.0, -3.0), vec3(2.0, 2.0, 2.0));
    // Floor
    scene.add_box(vec3(0.0, -1.0, -1.0), vec3(2.0, 2.0, 2.0));
    // Ceiling
    scene.add_box(vec3(0.0, 3.0, -1.0), vec3(2.0, 2.0, 2.0));
    // Light
    scene.add_box(vec3(0.0, 2.0, -1.0), vec3(1.0, 0.1, 0.5));

    // Some items in the room
    scene.add_sphere(vec3(-0.2, 0.2, -0.6), 0.2);
    scene.add_sphere(vec3(-0.5, 0.3, -1.2), 0.3);
    scene.add_box(vec3(0.5, 0.2, -0.8), vec3(0.4, 0.4, 0.4));
    scene.add_box(vec3(0.2, 0.35, -1.3), vec3(0.7, 0.7, 0.7));
    scene.add_sphere(vec3(0.1, 0.15, -0.2), 0.15);

    scene.build_scene();

    scene
}
