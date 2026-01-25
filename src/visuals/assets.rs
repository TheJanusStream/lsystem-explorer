// lsystem-explorer/src/visuals/assets.rs

use crate::core::config::{PropMeshType, TextureType};
use bevy::image::{ImageSampler, ImageSamplerDescriptor};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::{platform::collections::HashMap, prelude::*};

#[derive(Resource)]
pub struct MaterialPalette {
    pub materials: HashMap<u8, Handle<StandardMaterial>>,
    pub primary_material: Handle<StandardMaterial>, // For UI binding
}

/// Stores procedural textures for material customization
#[derive(Resource)]
pub struct ProceduralTextures {
    pub textures: HashMap<TextureType, Handle<Image>>,
}

/// Stores base meshes for each PropMeshType
#[derive(Resource)]
pub struct PropMeshAssets {
    pub meshes: HashMap<PropMeshType, Handle<Mesh>>,
}

/// Generate a grid pattern texture
fn generate_grid_texture(size: u32, line_width: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let on_grid = (x % (size / 8) < line_width) || (y % (size / 8) < line_width);
            let val = if on_grid { 255 } else { 180 };
            data.extend_from_slice(&[val, val, val, 255]);
        }
    }
    data
}

/// Generate a noise pattern texture using simple pseudo-random
fn generate_noise_texture(size: u32, seed: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            // Simple hash-based noise
            let hash = ((x.wrapping_mul(374761393))
                ^ (y.wrapping_mul(668265263))
                ^ seed.wrapping_mul(1013904223))
            .wrapping_mul(1664525);
            let val = ((hash >> 24) & 0xFF) as u8;
            let blended = 128 + (val as i32 - 128) / 2; // Reduce contrast
            data.extend_from_slice(&[blended as u8, blended as u8, blended as u8, 255]);
        }
    }
    data
}

/// Generate a checker pattern texture
fn generate_checker_texture(size: u32, tile_size: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let checker = ((x / tile_size) + (y / tile_size)).is_multiple_of(2);
            let val = if checker { 220 } else { 160 };
            data.extend_from_slice(&[val, val, val, 255]);
        }
    }
    data
}

/// Create a Bevy Image from raw RGBA data
fn create_image(data: Vec<u8>, size: u32) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: bevy::image::ImageAddressMode::Repeat,
        address_mode_v: bevy::image::ImageAddressMode::Repeat,
        ..default()
    });
    image
}

pub fn setup_turtle_assets(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Generate procedural textures
    const TEX_SIZE: u32 = 256;
    let mut proc_textures = HashMap::new();

    let grid_data = generate_grid_texture(TEX_SIZE, 2);
    proc_textures.insert(
        TextureType::Grid,
        images.add(create_image(grid_data, TEX_SIZE)),
    );

    let noise_data = generate_noise_texture(TEX_SIZE, 42);
    proc_textures.insert(
        TextureType::Noise,
        images.add(create_image(noise_data, TEX_SIZE)),
    );

    let checker_data = generate_checker_texture(TEX_SIZE, 32);
    proc_textures.insert(
        TextureType::Checker,
        images.add(create_image(checker_data, TEX_SIZE)),
    );

    commands.insert_resource(ProceduralTextures {
        textures: proc_textures,
    });

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
