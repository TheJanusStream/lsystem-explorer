use crate::core::config::{LSystemConfig, LSystemEngine};
use crate::visuals::assets::TurtleMaterialHandle;
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy_symbios::LSystemMeshBuilder;
use symbios_turtle_3d::{TurtleConfig, TurtleInterpreter};

#[derive(Component)]
pub struct LSystemMeshTag;

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

pub fn render_turtle(
    mut commands: Commands,
    engine: Res<LSystemEngine>,
    config: Res<LSystemConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mat_handle: Res<TurtleMaterialHandle>,
    mut render_state: ResMut<TurtleRenderState>,
    old_meshes: Query<Entity, With<LSystemMeshTag>>,
) {
    let sys = &engine.0;

    if !engine.is_changed() {
        return;
    }

    // Clean up previous generation
    for entity in &old_meshes {
        commands.entity(entity).despawn();
    }

    if sys.state.is_empty() {
        return;
    }

    let start_time = Instant::now();

    // 1. Configure the Sovereign Interpreter
    // We prioritize #define constants from the L-System, falling back to UI config
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

    let tropism_compat = config.tropism.map(|v| bevy::math::Vec3::new(v.x, v.y, v.z));

    let turtle_config = TurtleConfig {
        default_step,
        default_angle,
        initial_width,
        tropism: tropism_compat,
        elasticity: config.elasticity,
    };

    let mut interpreter = TurtleInterpreter::new(turtle_config);

    // 2. Populate Symbols
    // The interpreter handles standard ABOP mapping automatically
    interpreter.populate_standard_symbols(&sys.interner);

    // 3. Generate Geometric Skeleton (Engine Agnostic)
    let skeleton = interpreter.build_skeleton(&sys.state);

    // 4. Generate Mesh (Bevy Specific)
    let builder = LSystemMeshBuilder::new().with_resolution(8);
    let final_mesh = builder.build(&skeleton);

    render_state.total_vertices = final_mesh.count_vertices();
    let mesh_handle = meshes.add(final_mesh);

    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(mat_handle.0.clone()),
        Transform::IDENTITY,
        LSystemMeshTag,
    ));

    render_state.generation_time_ms = start_time.elapsed().as_secs_f32() * 1000.0;
}
