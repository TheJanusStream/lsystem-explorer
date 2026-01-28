use crate::core::config::PropMeshType;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

// Material-related resources (MaterialPalette, ProceduralTextures, texture generation)
// are now provided by bevy_symbios::materials.

/// Stores base meshes for each PropMeshType
#[derive(Resource)]
pub struct PropMeshAssets {
    pub meshes: HashMap<PropMeshType, Handle<Mesh>>,
}

pub fn setup_prop_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mut prop_meshes = HashMap::new();

    // Leaf: Flattened cuboid
    prop_meshes.insert(PropMeshType::Leaf, meshes.add(Cuboid::new(0.5, 0.8, 0.0)));

    // Sphere: Ico-sphere
    prop_meshes.insert(
        PropMeshType::Sphere,
        meshes.add(Sphere::new(0.2).mesh().ico(2).unwrap()),
    );

    // Cone
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
