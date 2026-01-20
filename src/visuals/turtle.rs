use crate::core::config::{LSystemConfig, LSystemEngine};
use crate::visuals::assets::{SurfaceAssets, TurtleMaterialHandle};
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
    config: Res<LSystemConfig>,
    mat_handle: Res<TurtleMaterialHandle>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !config.is_changed() {
        return;
    }

    if let Some(mat) = materials.get_mut(&mat_handle.0) {
        mat.base_color = Color::srgb_from_array(config.material_color);
        let emission_linear =
            Color::srgb_from_array(config.emission_color).to_linear() * config.emission_strength;
        mat.emissive = emission_linear;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render_turtle(
    mut commands: Commands,
    engine: Res<LSystemEngine>,
    config: Res<LSystemConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mat_handle: Res<TurtleMaterialHandle>,
    surface_assets: Res<SurfaceAssets>, // Access our new assets
    mut render_state: ResMut<TurtleRenderState>,
    old_meshes: Query<Entity, With<LSystemMeshTag>>,
    old_props: Query<Entity, With<LSystemPropTag>>,
) {
    let sys = &engine.0;

    if !engine.is_changed() {
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

    let initial_width = sys.constants.get("width").map(|&w| w as f32).unwrap_or(0.1);
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

    // 4. Mesh Branches
    let builder = LSystemMeshBuilder::new().with_resolution(8);
    let final_mesh = builder.build(&skeleton);
    render_state.total_vertices = final_mesh.count_vertices();

    commands.spawn((
        Mesh3d(meshes.add(final_mesh)),
        MeshMaterial3d(mat_handle.0.clone()),
        Transform::IDENTITY,
        LSystemMeshTag,
    ));

    // 5. Spawn Props
    for prop in &skeleton.props {
        let mesh_handle = surface_assets
            .meshes
            .get(&prop.surface_id)
            .or_else(|| surface_assets.meshes.get(&0));

        if let Some(handle) = mesh_handle {
            commands.spawn((
                Mesh3d(handle.clone()),
                MeshMaterial3d(mat_handle.0.clone()),
                Transform {
                    translation: prop.position,
                    rotation: prop.rotation,
                    scale: prop.scale,
                },
                LSystemPropTag,
            ));
        }
    }

    render_state.generation_time_ms = start_time.elapsed().as_secs_f32() * 1000.0;
}
