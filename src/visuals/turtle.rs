use crate::core::config::{
    LSystemConfig, LSystemEngine, MaterialSettingsMap, PropConfig, PropMeshType, TextureType,
};
use crate::visuals::assets::{MaterialPalette, ProceduralTextures, PropMeshAssets};
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy_symbios::LSystemMeshBuilder;
use symbios_turtle_3d::{TurtleConfig, TurtleInterpreter};

#[derive(Component)]
pub struct LSystemMeshTag;

// Tag for discrete objects (Leaves, Flowers) to allow cleanup
#[derive(Component)]
pub struct LSystemPropTag;

#[derive(Resource, Default)]
pub struct TurtleRenderState {
    pub total_vertices: usize,
    pub generation_time_ms: f32,
}

pub fn sync_material_properties(
    material_settings: Res<MaterialSettingsMap>,
    palette: Res<MaterialPalette>,
    proc_textures: Res<ProceduralTextures>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !material_settings.is_changed() {
        return;
    }

    // Sync UI settings to all materials in the palette
    for (mat_id, settings) in &material_settings.settings {
        let Some(handle) = palette.materials.get(mat_id) else {
            continue;
        };
        let Some(mat) = materials.get_mut(handle) else {
            continue;
        };

        mat.base_color = Color::srgb_from_array(settings.base_color);
        mat.perceptual_roughness = settings.roughness;
        mat.metallic = settings.metallic;

        let emission_linear = Color::srgb_from_array(settings.emission_color).to_linear()
            * settings.emission_strength;
        mat.emissive = emission_linear;

        // Apply texture based on TextureType
        mat.base_color_texture = match settings.texture {
            TextureType::None => None,
            other => proc_textures.textures.get(&other).cloned(),
        };
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render_turtle(
    mut commands: Commands,
    engine: Res<LSystemEngine>,
    config: Res<LSystemConfig>,
    prop_config: Res<PropConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    palette: Res<MaterialPalette>,
    prop_assets: Res<PropMeshAssets>,
    mut render_state: ResMut<TurtleRenderState>,
    old_meshes: Query<Entity, With<LSystemMeshTag>>,
    old_props: Query<Entity, With<LSystemPropTag>>,
) {
    let sys = &engine.0;

    // Re-render when L-system state OR prop configuration changes
    if !engine.is_changed() && !prop_config.is_changed() {
        return;
    }

    // 1. Cleanup
    for entity in &old_meshes {
        commands.entity(entity).despawn();
    }
    for entity in &old_props {
        commands.entity(entity).despawn();
    }

    if sys.state.is_empty() {
        return;
    }

    let start_time = Instant::now();

    // 2. Configure Interpreter
    let default_step = sys
        .constants
        .get("step")
        .map(|&s| s as f32)
        .unwrap_or(config.step_size);

    let default_angle = sys
        .constants
        .get("angle")
        .map(|&a| a as f32)
        .unwrap_or(config.default_angle)
        .to_radians();

    let initial_width = sys
        .constants
        .get("width")
        .map(|&w| w as f32)
        .unwrap_or(config.default_width);

    let tropism_compat = config.tropism;

    let turtle_config = TurtleConfig {
        default_step,
        default_angle,
        initial_width,
        tropism: tropism_compat,
        elasticity: config.elasticity,
    };

    let mut interpreter = TurtleInterpreter::new(turtle_config);
    interpreter.populate_standard_symbols(&sys.interner);

    // 3. Build Skeleton (Geometry + Props)
    let skeleton = interpreter.build_skeleton(&sys.state);

    // 4. Mesh Branches (Multi-Material Support)
    let builder = LSystemMeshBuilder::new().with_resolution(8);
    let mesh_buckets = builder.build(&skeleton);

    let mut total_verts = 0;

    // Iterate over all generated material buckets
    for (material_id, mesh) in mesh_buckets {
        total_verts += mesh.count_vertices();

        // Resolve Material ID to Handle
        let material = palette
            .materials
            .get(&material_id)
            .unwrap_or(&palette.primary_material) // Fallback to Mat 0
            .clone();

        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material),
            Transform::IDENTITY,
            LSystemMeshTag,
        ));
    }

    render_state.total_vertices = total_verts;

    // 5. Spawn Props
    for prop in &skeleton.props {
        // Look up mesh type from PropConfig, default to Leaf
        let mesh_type = prop_config
            .surface_meshes
            .get(&prop.surface_id)
            .copied()
            .unwrap_or(PropMeshType::Leaf);

        let mesh_handle = prop_assets.meshes.get(&mesh_type);

        if let Some(handle) = mesh_handle {
            commands.spawn((
                Mesh3d(handle.clone()),
                MeshMaterial3d(palette.primary_material.clone()),
                Transform {
                    translation: prop.position,
                    rotation: prop.rotation,
                    scale: prop.scale * prop_config.prop_scale,
                },
                LSystemPropTag,
            ));
        }
    }

    render_state.generation_time_ms = start_time.elapsed().as_secs_f32() * 1000.0;
}
