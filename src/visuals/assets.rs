use crate::core::config::PropMeshType;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

// Material-related resources (MaterialPalette, ProceduralTextures, texture generation)
// are now provided by bevy_symbios::materials.

/// Stores base meshes for each PropMeshType
#[derive(Resource)]
pub struct PropMeshAssets {
    pub meshes: HashMap<PropMeshType, Handle<Mesh>>,
}

/// Creates a billboard quad mesh with the pivot/origin at the base centre.
///
/// The card extends from y=0 (base, V=1 in texture) to y=`height` (tip, V=0).
/// Width spans -`width`/2 to +`width`/2 in X.  The surface faces +Z.
/// UV layout matches the foliage card convention used by `bevy_symbios_texture`
/// (V=1 at the stem attachment, V=0 at the leaf tip).
fn create_foliage_card(width: f32, height: f32) -> Mesh {
    let positions: Vec<[f32; 3]> = vec![
        [-width / 2.0, 0.0, 0.0],    // 0: bottom-left  (base, V=1)
        [width / 2.0, 0.0, 0.0],     // 1: bottom-right (base, V=1)
        [width / 2.0, height, 0.0],  // 2: top-right    (tip,  V=0)
        [-width / 2.0, height, 0.0], // 3: top-left     (tip,  V=0)
    ];
    let normals: Vec<[f32; 3]> = vec![[0.0, 0.0, 1.0]; 4];
    let uvs: Vec<[f32; 2]> = vec![
        [0.0, 1.0], // bottom-left
        [1.0, 1.0], // bottom-right
        [1.0, 0.0], // top-right
        [0.0, 0.0], // top-left
    ];
    // Two triangles: (0,1,2) and (0,2,3)
    let indices = Indices::U32(vec![0, 1, 2, 0, 2, 3]);

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(indices);
    let _ = mesh.generate_tangents();
    mesh
}

pub fn setup_prop_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mut prop_meshes = HashMap::new();

    // Leaf: billboard card, pivot at base (V=1), tip pointing up (V=0)
    prop_meshes.insert(
        PropMeshType::Leaf,
        meshes.add(create_foliage_card(0.5, 0.8)),
    );

    // Twig: wider billboard card for composite twig cards
    prop_meshes.insert(
        PropMeshType::Twig,
        meshes.add(create_foliage_card(0.7, 1.0)),
    );

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
