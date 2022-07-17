use super::Scene;
use glam::{vec3, vec3a, Quat, Vec3A};
use kea_gpu::device::Device;
use kea_renderer_shaders::materials::Material;
use std::sync::Arc;

// pub fn basic_shapes(device: Arc<Device>) -> Scene {
//     let mut scene = Scene::new(device);

//     scene.add_sphere(vec3(0.0, -1000.0, -1.5), 1000.0);
//     scene.add_sphere(vec3(0.0, 0.8, -2.8), 0.8);
//     scene.add_sphere(vec3(-0.9, 0.4, -2.0), 0.4);
//     scene.add_sphere(vec3(0.7, 1.3, -1.9), 0.3);
//     scene.add_box(vec3(1.1, 0.5, -1.8), vec3(1.0, 1.0, 1.0));
//     scene.add_box(vec3(-0.1, 0.3, -1.7), vec3(0.6, 0.6, 0.6));

//     scene.build_scene();

//     scene
// }

pub fn cornell_box(device: Arc<Device>) -> Scene {
    let mut scene = Scene::new(device);

    let red = Material {
        diffuse: vec3a(1.0, 0.0, 0.0),
        emit: Vec3A::ZERO,
    };
    let green = Material {
        diffuse: vec3a(0.0, 1.0, 0.0),
        emit: Vec3A::ZERO,
    };

    let white = Material {
        diffuse: Vec3A::ONE,
        emit: Vec3A::ZERO,
    };

    let grey = Material {
        diffuse: vec3a(0.7, 0.7, 0.7),
        emit: Vec3A::ZERO,
    };

    let light = Material {
        diffuse: Vec3A::ONE,
        emit: Vec3A::ONE,
    };

    // Walls
    scene.add_box(
        vec3(-1.0, 1.0, -1.0),
        vec3(0.001, 2.0, 2.0),
        Quat::IDENTITY,
        red,
    );
    scene.add_box(
        vec3(1.0, 1.0, -1.0),
        vec3(0.001, 2.0, 2.0),
        Quat::IDENTITY,
        green,
    );
    scene.add_box(
        vec3(0.0, 1.0, -2.0),
        vec3(2.0, 2.0, 0.001),
        Quat::IDENTITY,
        white,
    );
    // Floor
    scene.add_box(
        vec3(0.0, 0.0, -1.0),
        vec3(2.0, 0.001, 2.0),
        Quat::IDENTITY,
        white,
    );
    // Ceiling
    scene.add_box(
        vec3(0.0, 2.0, -1.0),
        vec3(2.0, 0.001, 2.0),
        Quat::IDENTITY,
        white,
    );
    // Light
    scene.add_box(
        vec3(0.0, 2.0, -1.0),
        vec3(1.0, 0.1, 0.5),
        Quat::IDENTITY,
        light,
    );

    // Some items in the room
    scene.add_box(
        vec3(-0.3, 0.65, -1.3),
        vec3(0.6, 1.3, 0.6),
        Quat::from_rotation_y(0.4),
        grey,
    );
    scene.add_sphere(vec3(0.4, 0.25, -0.5), 0.25, grey);

    scene.build_scene();

    scene
}
