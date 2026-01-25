// lsystem-explorer/src/visuals/assets.rs

use crate::core::config::PropMeshType;
use bevy::{platform::collections::HashMap, prelude::*};

#[derive(Resource)]
pub struct MaterialPalette {
    pub materials: HashMap<u8, Handle<StandardMaterial>>,
    pub primary_material: Handle<StandardMaterial>, // For UI binding
}

/// Stores base meshes for each PropMeshType
#[derive(Resource)]
pub struct PropMeshAssets {
    pub meshes: HashMap<PropMeshType, Handle<Mesh>>,
}

pub fn setup_turtle_assets(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut palette = HashMap::new();

    // --- Material 0: The "Trunk" / Primary ---
    // Base color White to allow Vertex Colors to shine through un-tinted.
    // High Metallic for the "Tech-Tree" look.
    let mat_0 = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.2,
        metallic: 0.8,
        reflectance: 0.5,
        ..default()
    });
    palette.insert(0, mat_0.clone());

    // --- Material 1: The "Energy" / Leaves ---
    // Emissive, Low Roughness (Glassy).
    let mat_1 = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.1,
        metallic: 0.0,
        emissive: LinearRgba::rgb(0.0, 2.0, 2.0), // Cyan Glow default
        ..default()
    });
    palette.insert(1, mat_1);

    // --- Material 2: Matte / Structure ---
    let mat_2 = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.5, 0.5),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    });
    palette.insert(2, mat_2);

    commands.insert_resource(MaterialPalette {
        materials: palette,
        primary_material: mat_0,
    });

    // 2. Prop Mesh Assets - one mesh per PropMeshType
    let mut prop_meshes = HashMap::new();

    // Leaf: Flattened cuboid
    prop_meshes.insert(PropMeshType::Leaf, meshes.add(Cuboid::new(0.5, 0.8, 0.02)));

    // Sphere: Ico-sphere
    prop_meshes.insert(
        PropMeshType::Sphere,
        meshes.add(Sphere::new(0.2).mesh().ico(2).unwrap()),
    );

    // Cone: Approximated with a narrow cylinder (Bevy doesn't have built-in cone)
    prop_meshes.insert(
        PropMeshType::Cone,
        meshes.add(Cone::new(0.15, 0.4).mesh().resolution(8)),
    );

    // Cylinder
    prop_meshes.insert(
        PropMeshType::Cylinder,
        meshes.add(Cylinder::new(0.1, 0.5).mesh().resolution(8)),
    );

    // Cube
    prop_meshes.insert(PropMeshType::Cube, meshes.add(Cuboid::new(0.3, 0.3, 0.3)));

    commands.insert_resource(PropMeshAssets {
        meshes: prop_meshes,
    });
}
