//! 3D rendering for the nursery population grid.
//!
//! This module provides systems to render the 9-individual population
//! as a 3D grid when nursery mode is active.

use crate::core::config::{LSystemConfig, MaterialSettings, PropConfig, PropMeshType, TextureType};
use crate::core::genotype::PlantGenotype;
use crate::ui::nursery::{
    CachedGenotypeMesh, NurseryLabelTag, NurseryMeshTag, NurseryMode, NurseryPropTag, NurseryState,
    PopulationMeshCache,
};
use crate::visuals::assets::PropMeshAssets;
use bevy::math::{Affine2, Vec2};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_symbios::LSystemMeshBuilder;
use bevy_symbios::materials::ProceduralTextures;
use symbios::System;
use symbios_turtle_3d::{TurtleConfig, TurtleInterpreter};

/// Cached material handles for nursery selection panels.
/// Created once at startup to avoid per-frame allocations.
#[derive(Resource)]
pub struct NurseryMaterials {
    pub normal: Handle<StandardMaterial>,
    pub selected: Handle<StandardMaterial>,
    pub error: Handle<StandardMaterial>,
}

impl NurseryMaterials {
    pub fn new(materials: &mut Assets<StandardMaterial>) -> Self {
        Self {
            normal: materials.add(StandardMaterial {
                base_color: Color::srgba(0.5, 0.5, 0.6, 0.15),
                emissive: LinearRgba::BLACK,
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            }),
            selected: materials.add(StandardMaterial {
                base_color: Color::srgba(0.0, 0.39, 0.0, 0.4),
                emissive: LinearRgba::new(0.06, 0.3, 0.06, 1.0),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            }),
            error: materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.2, 0.2, 0.35),
                emissive: LinearRgba::new(0.3, 0.06, 0.06, 1.0),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            }),
        }
    }

    /// Returns the appropriate material handle for a given panel state.
    pub fn for_state(&self, is_selected: bool, has_error: bool) -> Handle<StandardMaterial> {
        if has_error {
            self.error.clone()
        } else if is_selected {
            self.selected.clone()
        } else {
            self.normal.clone()
        }
    }
}

/// Startup system to create cached nursery panel materials.
pub fn setup_nursery_materials(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    commands.insert_resource(NurseryMaterials::new(&mut materials));
}

/// Creates a StandardMaterial from a MaterialSettings, using procedural textures if available.
fn material_from_settings(
    settings: &MaterialSettings,
    proc_textures: &ProceduralTextures,
) -> StandardMaterial {
    let emission_linear =
        Color::srgb_from_array(settings.emission_color).to_linear() * settings.emission_strength;

    let base_color_texture = match settings.texture {
        TextureType::None => None,
        other => proc_textures.textures.get(&other).cloned(),
    };

    StandardMaterial {
        base_color: Color::srgb_from_array(settings.base_color),
        perceptual_roughness: settings.roughness,
        metallic: settings.metallic,
        emissive: emission_linear,
        base_color_texture,
        uv_transform: Affine2::from_scale(Vec2::splat(settings.uv_scale)),
        ..default()
    }
}

/// Creates per-genotype material handles from the cached material settings.
fn create_genotype_materials(
    cached_materials: &HashMap<u8, MaterialSettings>,
    proc_textures: &ProceduralTextures,
    materials: &mut Assets<StandardMaterial>,
) -> (
    HashMap<u8, Handle<StandardMaterial>>,
    Handle<StandardMaterial>,
) {
    let mut handles = HashMap::new();
    let mut primary = None;

    for (&slot, settings) in cached_materials {
        let handle = materials.add(material_from_settings(settings, proc_textures));
        if primary.is_none() {
            primary = Some(handle.clone());
        }
        handles.insert(slot, handle);
    }

    let fallback = primary.unwrap_or_else(|| {
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.55, 0.27, 0.07),
            perceptual_roughness: 0.8,
            ..default()
        })
    });

    (handles, fallback)
}

/// Derives a PlantGenotype into a System with full state.
///
/// NOTE: Always creates a fresh `System::new()` to guarantee clean derivation state.
/// This prevents cumulative derivation issues where calling `sys.derive(n)` on an
/// already-derived system would result in double-growth.
fn derive_genotype(genotype: &PlantGenotype, _config: &LSystemConfig) -> Option<System> {
    let mut sys = System::new();
    sys.set_seed(genotype.seed);

    let mut axiom_set = false;

    // Parse the source code
    for line in genotype.source_code.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        if trimmed.starts_with("omega:") {
            let axiom = trimmed.strip_prefix("omega:")?.trim();
            sys.set_axiom(axiom).ok()?;
            axiom_set = true;
        } else if trimmed.starts_with('#') {
            sys.add_directive(trimmed).ok()?;
        } else if trimmed.contains("->") {
            sys.add_rule(trimmed).ok()?;
        }
    }

    if !axiom_set {
        return None;
    }

    // Derive growth phase
    sys.derive(genotype.iterations).ok()?;

    // Apply finalization if present
    if !genotype.finalization_code.trim().is_empty() {
        // Clear rules but keep constants and state
        sys.rules.clear();
        sys.ignored_symbols.clear();

        // Parse finalization rules
        for line in genotype.finalization_code.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("omega:") {
                continue;
            }

            if trimmed.starts_with('#') {
                sys.add_directive(trimmed).ok()?;
            } else if trimmed.contains("->") {
                sys.add_rule(trimmed).ok()?;
            }
        }

        // Execute single decomposition pass
        sys.derive(1).ok()?;
    }

    Some(sys)
}

/// System that rebuilds the nursery population mesh cache when needed.
#[allow(clippy::too_many_arguments)]
pub fn rebuild_nursery_cache(
    mut nursery: ResMut<NurseryState>,
    mut cache: ResMut<PopulationMeshCache>,
    config: Res<LSystemConfig>,
) {
    // Only rebuild when explicitly requested and nursery is enabled
    if !nursery.needs_3d_rebuild || nursery.mode != NurseryMode::Enabled {
        return;
    }

    nursery.needs_3d_rebuild = false;
    cache.entries.clear();
    nursery.errors.clear();

    // Get population data directly from NurseryState
    if nursery.population.is_empty() {
        return;
    }

    let population: Vec<(PlantGenotype, f32)> = nursery
        .population
        .iter()
        .map(|p| (p.genotype.clone(), p.fitness))
        .collect();

    // Derive each genotype, capturing errors
    for (i, (genotype, fitness)) in population.into_iter().enumerate() {
        let (system, error) = match derive_genotype(&genotype, &config) {
            Some(sys) => (Some(sys), None),
            None => (
                None,
                Some("Derivation failed: invalid L-system syntax".to_string()),
            ),
        };

        // Store error in NurseryState for UI access
        if let Some(ref err) = error {
            nursery.errors.insert(i, err.clone());
        }

        cache.entries.insert(
            i,
            CachedGenotypeMesh {
                system,
                fitness,
                angle: genotype.angle,
                step: genotype.step,
                width: genotype.width,
                elasticity: genotype.elasticity,
                tropism: genotype.tropism.map(|t| Vec3::new(t[0], t[1], t[2])),
                materials: genotype.get_material_settings(),
                error,
            },
        );
    }

    cache.cached_generation = nursery.generation;
    cache.dirty = true;
}

/// System that spawns/despawns nursery 3D meshes based on cache state.
#[allow(clippy::too_many_arguments)]
pub fn render_nursery_population(
    mut commands: Commands,
    nursery: Res<NurseryState>,
    mut cache: ResMut<PopulationMeshCache>,
    config: Res<LSystemConfig>,
    prop_config: Res<PropConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    proc_textures: Res<ProceduralTextures>,
    prop_assets: Res<PropMeshAssets>,
    // Queries for existing nursery entities
    nursery_materials: Res<NurseryMaterials>,
    old_meshes: Query<Entity, With<NurseryMeshTag>>,
    old_props: Query<Entity, With<NurseryPropTag>>,
    old_labels: Query<Entity, With<NurseryLabelTag>>,
) {
    // Despawn nursery entities when nursery is disabled
    if nursery.mode == NurseryMode::Disabled {
        for entity in old_meshes
            .iter()
            .chain(old_props.iter())
            .chain(old_labels.iter())
        {
            commands.entity(entity).despawn();
        }
        cache.entries.clear();
        return;
    }

    // Only rebuild meshes when cache is dirty
    if !cache.dirty {
        return;
    }
    cache.dirty = false;

    // Despawn old entities
    for entity in old_meshes
        .iter()
        .chain(old_props.iter())
        .chain(old_labels.iter())
    {
        commands.entity(entity).despawn();
    }

    // Calculate grid positions
    let spacing = nursery.grid_spacing;
    let grid_size = nursery.grid_size;
    let pop_size = nursery.population_size();
    let grid_offset = (grid_size as f32 - 1.0) * spacing / 2.0;

    // Spawn meshes for each cached genotype
    for i in 0..pop_size {
        let Some(cached) = cache.entries.get(&i) else {
            continue;
        };

        // Calculate grid position (NxN in XZ plane)
        let row = i / grid_size;
        let col = i % grid_size;
        let x = col as f32 * spacing - grid_offset;
        let z = row as f32 * spacing - grid_offset;
        let grid_pos = Vec3::new(x, 0.0, z);

        let is_selected = nursery.selected.contains(&i);
        let has_error = cached.error.is_some();

        // Only render meshes if derivation succeeded
        if let Some(ref system) = cached.system {
            // Configure turtle interpreter using individual genotype parameters as fallbacks
            let default_step = system
                .constants
                .get("step")
                .map(|&s| s as f32)
                .unwrap_or(cached.step);

            let default_angle = system
                .constants
                .get("angle")
                .map(|&a| a as f32)
                .unwrap_or(cached.angle)
                .to_radians();

            let initial_width = system
                .constants
                .get("width")
                .map(|&w| w as f32)
                .unwrap_or(cached.width);

            let turtle_config = TurtleConfig {
                default_step,
                default_angle,
                initial_width,
                tropism: cached.tropism,
                elasticity: cached.elasticity,
                max_stack_depth: 1024,
            };

            let mut interpreter = TurtleInterpreter::new(turtle_config);
            interpreter.populate_standard_symbols(&system.interner);

            // Build skeleton and meshes
            let skeleton = interpreter.build_skeleton(&system.state);
            let builder = LSystemMeshBuilder::new().with_resolution(config.mesh_resolution);
            let mesh_buckets = builder.build(&skeleton);

            // Create per-genotype material handles from the individual's settings
            let (geno_materials, geno_fallback) =
                create_genotype_materials(&cached.materials, &proc_textures, &mut materials);

            // Spawn branch meshes
            for (material_id, mesh) in mesh_buckets {
                let material = geno_materials
                    .get(&material_id)
                    .unwrap_or(&geno_fallback)
                    .clone();

                commands.spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(material),
                    Transform::from_translation(grid_pos),
                    NurseryMeshTag { index: i },
                ));
            }

            // Spawn props (leaves, flowers, etc.)
            for prop in &skeleton.props {
                let mesh_type = prop_config
                    .prop_meshes
                    .get(&prop.prop_id)
                    .copied()
                    .unwrap_or(PropMeshType::Leaf);

                let mesh_handle = prop_assets.meshes.get(&mesh_type);

                if let Some(handle) = mesh_handle {
                    // Create prop material by blending genotype material with prop color
                    let base_handle = geno_materials
                        .get(&prop.material_id)
                        .unwrap_or(&geno_fallback);
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
                            translation: prop.position + grid_pos,
                            rotation: prop.rotation,
                            scale: prop.scale * prop_config.prop_scale,
                        },
                        NurseryPropTag { index: i },
                    ));
                }
            }
        }

        // Create a translucent horizontal panel below each plant
        let panel_size = spacing * 0.9;
        let panel_mesh = meshes.add(Cuboid::new(panel_size, 2.0, panel_size));
        let panel_material = nursery_materials.for_state(is_selected, has_error);

        commands.spawn((
            Mesh3d(panel_mesh),
            MeshMaterial3d(panel_material),
            Transform::from_translation(grid_pos + Vec3::new(0.0, -1.0, 0.0)),
            NurseryLabelTag { index: i },
        ));
    }
}

/// System to update panel materials in-place when selection changes.
/// This avoids a full scene rebuild by only swapping material handles.
pub fn sync_nursery_selection_visuals(
    nursery: Res<NurseryState>,
    nursery_materials: Res<NurseryMaterials>,
    cache: Res<PopulationMeshCache>,
    mut labels: Query<(&NurseryLabelTag, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    if !nursery.is_changed() || nursery.mode != NurseryMode::Enabled {
        return;
    }

    for (tag, mut mat_handle) in labels.iter_mut() {
        let is_selected = nursery.selected.contains(&tag.index);
        let has_error = cache
            .entries
            .get(&tag.index)
            .is_some_and(|c| c.error.is_some());
        let desired = nursery_materials.for_state(is_selected, has_error);
        if mat_handle.0 != desired {
            mat_handle.0 = desired;
        }
    }
}

/// System that handles clicking on nursery selection panels via ray-plane intersection.
///
/// Uses camera raycasting against the y=0 ground plane to determine which grid cell
/// was clicked, bypassing the picking message pipeline to avoid conflicts with bevy_egui.
pub fn handle_panel_clicks(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut nursery: ResMut<NurseryState>,
    egui_wants: Res<bevy_egui::input::EguiWantsInput>,
) {
    if !mouse.just_pressed(MouseButton::Left) || nursery.mode != NurseryMode::Enabled {
        return;
    }

    // Don't intercept clicks when the pointer is over the egui UI
    if egui_wants.is_pointer_over_area() {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        return;
    };

    // Intersect ray with y=0 ground plane
    let plane_y = -1.0_f32;
    let denom = ray.direction.y;
    if denom.abs() < 1e-6 {
        return; // Ray is parallel to the ground plane
    }
    let t = (plane_y - ray.origin.y) / denom;
    if t < 0.0 {
        return; // Intersection is behind the camera
    }
    let hit_point = ray.origin + *ray.direction * t;

    // Map hit point to grid cell
    let spacing = nursery.grid_spacing;
    let grid_size = nursery.grid_size;
    let grid_offset = (grid_size as f32 - 1.0) * spacing / 2.0;
    let half_panel = spacing * 0.45; // panel is spacing * 0.9 wide

    for i in 0..nursery.population_size() {
        let row = i / grid_size;
        let col = i % grid_size;
        let cx = col as f32 * spacing - grid_offset;
        let cz = row as f32 * spacing - grid_offset;

        if (hit_point.x - cx).abs() <= half_panel && (hit_point.z - cz).abs() <= half_panel {
            nursery.toggle_selection(i);
            return;
        }
    }
}
