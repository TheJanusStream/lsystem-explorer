use std::f32::consts::TAU;

use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

pub fn setup_scene(mut commands: Commands) {
    // Directional Light (Sunlight)
    commands.spawn((
        DirectionalLight {
            illuminance: 8000.0,
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.95, 0.9), // Warm sunlight
            ..default()
        },
        Transform {
            translation: Vec3::new(50.0, 100.0, 50.0),
            // Angled down and slightly from the side
            rotation: Quat::from_rotation_x(-std::f32::consts::PI / 4.)
                .mul_quat(Quat::from_rotation_y(-std::f32::consts::PI / 6.)),
            ..default()
        },
    ));

    // Camera with Bloom
    commands.spawn((
        PanOrbitCamera {
            focus: Vec3::new(0.0, 400.0, 0.0),
            yaw: Some(TAU / 5.0),
            pitch: Some(TAU / 64.0),
            radius: Some(1200.0),
            button_orbit: MouseButton::Right,
            button_pan: MouseButton::Middle,
            ..default()
        },
        Camera3d::default(),
        Bloom::NATURAL, // Enable Bloom
    ));
}
