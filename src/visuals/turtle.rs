use crate::core::config::{DirtyFlags, LSystemConfig, LSystemEngine, PropConfig, PropMeshType};
use crate::visuals::assets::PropMeshAssets;
use bevy::platform::collections::HashMap;
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy_symbios::LSystemMeshBuilder;
use bevy_symbios::materials::MaterialPalette;
use symbios_turtle_3d::{TurtleConfig, TurtleInterpreter};

#[derive(Component)]
pub struct LSystemMeshTag;

#[derive(Component)]
pub struct LSystemPropTag;

/// Component storing the tint data for a prop, enabling material reactivity.
#[derive(Component)]
pub struct PropTint {
    pub material_id: u8,
    pub color: Vec4,
}

/// Cache key for prop materials: (material_id, color as [u8; 4]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PropMaterialKey {
    pub material_id: u8,
    pub color_rgba: [u8; 4],
}

impl PropMaterialKey {
    pub fn new(material_id: u8, color: Vec4) -> Self {
        Self {
            material_id,
            color_rgba: [
                (color.x.clamp(0.0, 1.0) * 255.0) as u8,
                (color.y.clamp(0.0, 1.0) * 255.0) as u8,
                (color.z.clamp(0.0, 1.0) * 255.0) as u8,
                (color.w.clamp(0.0, 1.0) * 255.0) as u8,
            ],
        }
    }
}

/// Creates or retrieves a cached prop material.
fn get_or_create_prop_material(
    cache: &mut PropMaterialCache,
    materials: &mut Assets<StandardMaterial>,
    palette: &MaterialPalette,
    key: PropMaterialKey,
    material_id: u8,
    color: Vec4,
) -> Handle<StandardMaterial> {
    if let Some(handle) = cache.cache.get(&key) {
        return handle.clone();
    }

    let base_handle = palette
        .materials
        .get(&material_id)
        .unwrap_or(&palette.primary_material);

    let base_mat = materials.get(base_handle).cloned().unwrap_or_default();

    let base_srgba = base_mat.base_color.to_srgba();
    let blended = Color::srgba(
        base_srgba.red * color.x,
        base_srgba.green * color.y,
        base_srgba.blue * color.z,
        base_srgba.alpha * color.w,
    );

    let prop_material = materials.add(StandardMaterial {
        base_color: blended,
        ..base_mat
    });

    cache.cache.insert(key, prop_material.clone());
    prop_material
}

/// Resource caching prop materials by (material_id, color) to avoid duplication.
#[derive(Resource, Default)]
pub struct PropMaterialCache {
    pub cache: HashMap<PropMaterialKey, Handle<StandardMaterial>>,
}

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
    mut prop_material_cache: ResMut<PropMaterialCache>,
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

    // 1. Cleanup (including prop material cache)
    prop_material_cache.cache.clear();
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
        max_stack_depth: 1024,
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

    // 5. Spawn Props (with inherited material ID and color, using cache)
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

            let key = PropMaterialKey::new(prop.material_id, prop.color);
            let prop_material = get_or_create_prop_material(
                &mut prop_material_cache,
                &mut materials,
                &palette,
                key,
                prop.material_id,
                prop.color,
            );

            commands.spawn((
                Mesh3d(handle.clone()),
                MeshMaterial3d(prop_material),
                Transform {
                    translation: prop.position,
                    rotation: prop.rotation,
                    scale: prop.scale * prop_config.prop_scale,
                },
                LSystemPropTag,
                PropTint {
                    material_id: prop.material_id,
                    color: prop.color,
                },
            ));
        }
    }

    render_state.total_vertices = total_verts;
    render_state.meshing_time_ms = start_time.elapsed().as_secs_f32() * 1000.0;
}

/// System that updates prop materials when the MaterialPalette changes.
/// Regenerates cached materials and updates all prop handles.
pub fn sync_prop_materials(
    palette: Res<MaterialPalette>,
    mut prop_material_cache: ResMut<PropMaterialCache>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut props: Query<(&PropTint, &mut MeshMaterial3d<StandardMaterial>), With<LSystemPropTag>>,
) {
    if !palette.is_changed() || props.is_empty() {
        return;
    }

    // Clear cache and regenerate all prop materials
    prop_material_cache.cache.clear();

    for (tint, mut mat_handle) in &mut props {
        let key = PropMaterialKey::new(tint.material_id, tint.color);
        let new_handle = get_or_create_prop_material(
            &mut prop_material_cache,
            &mut materials,
            &palette,
            key,
            tint.material_id,
            tint.color,
        );
        mat_handle.0 = new_handle;
    }
}
