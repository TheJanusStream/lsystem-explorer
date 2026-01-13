use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

pub fn setup_scene(mut commands: Commands) {
    // Lighting
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(50.0, 100.0, 50.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::PI / 4.),
            ..default()
        },
    ));

    // Camera
    // Positioned to view the tree from a nice angle
    commands.spawn((
        PanOrbitCamera {
            focus: Vec3::new(0.0, 400.0, 0.0),
            yaw: Some(TAU / 5.0),
            pitch: Some(TAU / 64.0),
            radius: Some(1200.0),
            button_orbit: MouseButton::Middle,
            button_pan: MouseButton::Right,
            ..default()
        },
        Camera3d::default(),
    ));
}
