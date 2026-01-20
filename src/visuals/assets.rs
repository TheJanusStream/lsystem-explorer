use bevy::{platform::collections::HashMap, prelude::*};

#[derive(Resource)]
pub struct TurtleMaterialHandle(pub Handle<StandardMaterial>);

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
    // 1. Branch Material
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.8, 0.2),
        perceptual_roughness: 0.8,
        metallic: 0.0,
        ..default()
    });
    commands.insert_resource(TurtleMaterialHandle(material));

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
