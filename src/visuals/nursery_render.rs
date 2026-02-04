//! 3D rendering for the nursery population grid.
//!
//! This module provides systems to render the 9-individual population
//! as a 3D grid when nursery mode is active.

use crate::core::config::LSystemConfig;
use crate::core::genotype::PlantGenotype;
use crate::ui::nursery::{
    CachedGenotypeMesh, GRID_COLS, GRID_SPACING, NurseryLabelTag, NurseryMeshTag, NurseryMode,
    NurseryPropTag, NurseryState, POPULATION_SIZE, PopulationMeshCache,
};
use bevy::prelude::*;
use bevy_symbios::LSystemMeshBuilder;
use bevy_symbios::materials::MaterialPalette;
use symbios::System;
use symbios_genetics::Evolver;
use symbios_turtle_3d::{TurtleConfig, TurtleInterpreter};

/// Derives a PlantGenotype into a System with full state.
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

    // Get population data
    let Some(evolver) = &mut nursery.evolver else {
        return;
    };

    let population: Vec<(PlantGenotype, f32)> = evolver
        .population()
        .iter()
        .map(|p| (p.genotype.clone(), p.fitness))
        .collect();

    // Derive each genotype
    for (i, (genotype, fitness)) in population.into_iter().enumerate() {
        if let Some(system) = derive_genotype(&genotype, &config) {
            cache
                .entries
                .insert(i, CachedGenotypeMesh { system, fitness });
        }
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    palette: Res<MaterialPalette>,
    // Queries for existing nursery entities
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
    let grid_offset = (GRID_COLS as f32 - 1.0) * GRID_SPACING / 2.0;

    // Spawn meshes for each cached genotype
    for i in 0..POPULATION_SIZE {
        let Some(cached) = cache.entries.get(&i) else {
            continue;
        };

        // Calculate grid position (3x3 in XZ plane)
        let row = i / GRID_COLS;
        let col = i % GRID_COLS;
        let x = col as f32 * GRID_SPACING - grid_offset;
        let z = row as f32 * GRID_SPACING - grid_offset;
        let grid_pos = Vec3::new(x, 0.0, z);

        // Configure turtle interpreter
        let default_step = cached
            .system
            .constants
            .get("step")
            .map(|&s| s as f32)
            .unwrap_or(config.step_size);

        let default_angle = cached
            .system
            .constants
            .get("angle")
            .map(|&a| a as f32)
            .unwrap_or(config.default_angle)
            .to_radians();

        let initial_width = cached
            .system
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
        interpreter.populate_standard_symbols(&cached.system.interner);

        // Build skeleton and meshes
        let skeleton = interpreter.build_skeleton(&cached.system.state);
        let builder = LSystemMeshBuilder::new().with_resolution(config.mesh_resolution);
        let mesh_buckets = builder.build(&skeleton);

        // Spawn branch meshes
        for (material_id, mesh) in mesh_buckets {
            let material = palette
                .materials
                .get(&material_id)
                .unwrap_or(&palette.primary_material)
                .clone();

            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material),
                Transform::from_translation(grid_pos),
                NurseryMeshTag { index: i },
            ));
        }

        // Calculate approximate height for the label based on props or use a default
        let max_height = skeleton
            .props
            .iter()
            .map(|p| p.position.y)
            .fold(default_step * 50.0, |a, b| a.max(b));
        let label_height = max_height + 20.0;

        let is_selected = nursery.selected == Some(i);

        // Create a colored sphere indicator above each plant
        let indicator_size = if is_selected { 6.0 } else { 4.0 };
        let indicator_color = if is_selected {
            Color::srgb(0.2, 1.0, 0.2) // Bright green for selected
        } else {
            Color::srgb(0.6, 0.6, 0.7) // Gray for unselected
        };

        let indicator_mesh = meshes.add(Sphere::new(indicator_size));
        let indicator_material = materials.add(StandardMaterial {
            base_color: indicator_color,
            emissive: if is_selected {
                indicator_color.into()
            } else {
                LinearRgba::BLACK
            },
            ..default()
        });

        commands.spawn((
            Mesh3d(indicator_mesh),
            MeshMaterial3d(indicator_material),
            Transform::from_translation(grid_pos + Vec3::new(0.0, label_height, 0.0)),
            NurseryLabelTag { index: i },
        ));
    }
}

/// System to update selection indicators when selection changes.
pub fn update_nursery_selection(
    nursery: Res<NurseryState>,
    mut cache: ResMut<PopulationMeshCache>,
) {
    if nursery.is_changed() && nursery.mode == NurseryMode::Enabled {
        // Mark cache dirty to refresh selection indicators
        if nursery.selected.is_some() {
            cache.dirty = true;
        }
    }
}
