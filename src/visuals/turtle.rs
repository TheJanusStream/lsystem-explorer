use crate::core::config::{DirtyFlags, LSystemConfig, LSystemEngine, PropConfig, PropMeshType};
use crate::visuals::assets::PropMeshAssets;
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy_symbios::LSystemMeshBuilder;
use bevy_symbios::materials::MaterialPalette;
use symbios_turtle_3d::{TurtleConfig, TurtleInterpreter};

// sync_material_properties is now provided by bevy_symbios::materials.

#[derive(Component)]
pub struct LSystemMeshTag;

#[derive(Component)]
pub struct LSystemPropTag;

#[derive(Resource, Default)]
pub struct TurtleRenderState {
    pub total_vertices: usize,
    pub meshing_time_ms: f32,
    pub derivation_time_ms: f32,
}

#[allow(clippy::too_many_arguments)]
pub fn render_turtle(
    mut commands: Commands,
    mut dirty: ResMut<DirtyFlags>,
    engine: Res<LSystemEngine>,
    config: Res<LSystemConfig>,
    prop_config: Res<PropConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    palette: Res<MaterialPalette>,
    prop_assets: Res<PropMeshAssets>,
    mut render_state: ResMut<TurtleRenderState>,
    old_meshes: Query<Entity, With<LSystemMeshTag>>,
    old_props: Query<Entity, With<LSystemPropTag>>,
) {
    if !dirty.geometry {
        return;
    }
    dirty.geometry = false;

    let sys = &engine.0;

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

    let turtle_config = TurtleConfig {
        default_step,
        default_angle,
        initial_width,
        tropism: config.tropism,
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

    for (material_id, mesh) in mesh_buckets {
        total_verts += mesh.count_vertices();

        let material = palette
            .materials
            .get(&material_id)
            .unwrap_or(&palette.primary_material)
            .clone();

        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material),
            Transform::IDENTITY,
            LSystemMeshTag,
        ));
    }

    // 5. Spawn Props (with inherited material ID and color)
    for prop in &skeleton.props {
        let mesh_type = prop_config
            .prop_meshes
            .get(&prop.prop_id)
            .copied()
            .unwrap_or(PropMeshType::Leaf);

        let mesh_handle = prop_assets.meshes.get(&mesh_type);

        if let Some(handle) = mesh_handle {
            if let Some(mesh) = meshes.get(handle) {
                total_verts += mesh.count_vertices();
            }
            let base_handle = palette
                .materials
                .get(&prop.material_id)
                .unwrap_or(&palette.primary_material);

            let base_mat = materials.get(base_handle).cloned().unwrap_or_default();

            let base_srgba = base_mat.base_color.to_srgba();
            let blended = Color::srgba(
                base_srgba.red * prop.color.x,
                base_srgba.green * prop.color.y,
                base_srgba.blue * prop.color.z,
                base_srgba.alpha * prop.color.w,
            );
            let prop_material = materials.add(StandardMaterial {
                base_color: blended,
                ..base_mat
            });

            commands.spawn((
                Mesh3d(handle.clone()),
                MeshMaterial3d(prop_material),
                Transform {
                    translation: prop.position,
                    rotation: prop.rotation,
                    scale: prop.scale * prop_config.prop_scale,
                },
                LSystemPropTag,
            ));
        }
    }

    render_state.total_vertices = total_verts;
    render_state.meshing_time_ms = start_time.elapsed().as_secs_f32() * 1000.0;
}
