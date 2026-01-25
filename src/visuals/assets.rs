// lsystem-explorer/src/visuals/assets.rs

use bevy::{platform::collections::HashMap, prelude::*};

#[derive(Resource)]
pub struct MaterialPalette {
    pub materials: HashMap<u8, Handle<StandardMaterial>>,
    pub primary_material: Handle<StandardMaterial>, // For UI binding
}

/// Maps a Surface ID (from L-System ~) to a 3D Mesh handle.
#[derive(Resource, Default)]
pub struct SurfaceAssets {
    pub meshes: HashMap<u16, Handle<Mesh>>,
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

    // 2. Surface Assets (Procedural Placeholders)
    let mut surface_map = HashMap::new();

    // --- ID 0: Leaf (Flattened Diamond/Cube) ---
    // Scale: Wide X, Thin Y, Long Z
    let leaf_mesh = Cuboid::new(0.5, 0.8, 0.02);
    surface_map.insert(0, meshes.add(leaf_mesh));

    // --- ID 1: Flower (Bud/Sphere) ---
    let flower_mesh = Sphere::new(0.2).mesh().ico(1).unwrap();
    surface_map.insert(1, meshes.add(flower_mesh));

    commands.insert_resource(SurfaceAssets {
        meshes: surface_map,
    });
}
