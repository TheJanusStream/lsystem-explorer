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
    // Target Y=50.0 approximates the mid-section of the first trunk segment (100 units)
    commands.spawn((
        Transform::from_xyz(0.0, 80.0, 180.0).looking_at(Vec3::new(0.0, 50.0, 0.0), Vec3::Y),
        PanOrbitCamera {
            button_orbit: MouseButton::Middle,
            button_pan: MouseButton::Right,
            ..default()
        },
        Camera3d::default(),
    ));
}
