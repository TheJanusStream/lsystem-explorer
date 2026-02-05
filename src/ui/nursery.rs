//! Breeding Nursery UI for evolutionary L-system exploration.
//!
//! This module provides a grid-based interface for visualizing and evolving
//! populations of plant genotypes using genetic algorithms.

use crate::core::config::{LSystemConfig, MaterialSettingsMap};
use crate::core::genotype::PlantGenotype;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy_egui::egui;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use symbios::System;
use symbios_genetics::{Genotype, Phenotype};

/// Spacing between plants in the 3D grid (world units).
pub const GRID_SPACING: f32 = 750.0;

/// Component tag for nursery 3D meshes (branches).
#[derive(Component)]
pub struct NurseryMeshTag {
    /// Index in the population (0-8).
    pub index: usize,
}

/// Component tag for nursery 3D props (leaves, etc.).
#[derive(Component)]
pub struct NurseryPropTag {
    /// Index in the population (0-8).
    pub index: usize,
}

/// Component tag for nursery labels (billboard text).
#[derive(Component)]
pub struct NurseryLabelTag {
    /// Index in the population (0-8).
    pub index: usize,
}

/// Cached derived state for a single genotype in the population.
pub struct CachedGenotypeMesh {
    /// The derived L-system state (None if derivation failed).
    pub system: Option<System>,
    /// Fitness value for display.
    pub fitness: f32,
    /// Individual's default turn angle in degrees.
    pub angle: f32,
    /// Individual's step size.
    pub step: f32,
    /// Individual's default branch width.
    pub width: f32,
    /// Error message if derivation failed.
    pub error: Option<String>,
}

/// Resource caching the derived meshes for the nursery population.
/// This prevents re-derivation every frame.
#[derive(Resource, Default)]
pub struct PopulationMeshCache {
    /// Cached systems for each population index.
    pub entries: HashMap<usize, CachedGenotypeMesh>,
    /// Generation number when this cache was built.
    pub cached_generation: usize,
    /// Whether the cache needs to be rebuilt.
    pub dirty: bool,
}

impl PopulationMeshCache {
    /// Marks the cache as dirty, requiring rebuild.
    pub fn invalidate(&mut self) {
        self.dirty = true;
    }

    /// Clears all cached entries.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.dirty = true;
    }
}

/// Nursery mode state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NurseryMode {
    /// Normal editor mode (nursery hidden).
    #[default]
    Disabled,
    /// Nursery mode active (population visible).
    Enabled,
}

/// Manages the evolutionary population of plant genotypes.
#[derive(Resource)]
pub struct NurseryState {
    /// Current nursery mode.
    pub mode: NurseryMode,
    /// The current population of genotypes with fitness values.
    pub population: Vec<Phenotype<PlantGenotype>>,
    /// Currently selected individual indices (champions for breeding).
    pub selected: HashSet<usize>,
    /// Mutation rate for breeding operations.
    pub mutation_rate: f32,
    /// RNG seed for reproducibility.
    pub seed: u64,
    /// Generation counter.
    pub generation: usize,
    /// Flag indicating the 3D nursery view needs to be rebuilt.
    pub needs_3d_rebuild: bool,
    /// Spacing between plants in the 3D grid (world units).
    pub grid_spacing: f32,
    /// Grid size (NxN grid, default 3 for 9 individuals).
    pub grid_size: usize,
    /// Derivation errors by population index (for UI display).
    pub errors: HashMap<usize, String>,
}

impl Default for NurseryState {
    fn default() -> Self {
        Self {
            mode: NurseryMode::Disabled,
            population: Vec::new(),
            selected: HashSet::new(),
            mutation_rate: 0.15,
            seed: 42,
            generation: 0,
            needs_3d_rebuild: false,
            grid_spacing: GRID_SPACING,
            grid_size: 3,
            errors: HashMap::new(),
        }
    }
}

impl NurseryState {
    /// Returns the total population size (grid_size^2).
    pub fn population_size(&self) -> usize {
        self.grid_size * self.grid_size
    }

    /// Initializes the population from the current editor state.
    pub fn initialize_from_editor(
        &mut self,
        config: &LSystemConfig,
        materials: &MaterialSettingsMap,
    ) {
        // Create base genotype from current editor state
        let base = PlantGenotype::new(config.source_code.clone())
            .with_finalization(config.finalization_code.clone())
            .with_materials(&materials.settings)
            .with_params(
                config.iterations,
                config.default_angle,
                config.step_size,
                config.default_width,
            )
            .with_seed(config.seed);

        let pop_size = self.population_size();
        let mut rng = Pcg64::seed_from_u64(self.seed);
        let mut new_population = Vec::with_capacity(pop_size);

        // First individual is the original
        new_population.push(Phenotype {
            genotype: base.clone(),
            fitness: evaluate_genotype(&base),
            objectives: vec![],
            descriptor: vec![],
        });

        // Rest are mutated variants
        for i in 1..pop_size {
            let mut variant = base.clone();
            variant.seed = self.seed + i as u64;
            variant.mutate(&mut rng, self.mutation_rate);
            let fitness = evaluate_genotype(&variant);
            new_population.push(Phenotype {
                genotype: variant,
                fitness,
                objectives: vec![],
                descriptor: vec![],
            });
        }

        self.population = new_population;
        self.generation = 0;
        self.selected.clear();
        self.selected.insert(0);
    }

    /// Resizes the population when grid size changes.
    pub fn resize_population(&mut self, new_size: usize) {
        if new_size == self.grid_size {
            return;
        }

        let old_pop_size = self.population_size();
        self.grid_size = new_size;
        let new_pop_size = self.population_size();

        if new_pop_size > old_pop_size {
            // Fill new slots with mutated variants of existing individuals
            let mut rng = Pcg64::seed_from_u64(self.seed.wrapping_add(self.generation as u64));
            for i in old_pop_size..new_pop_size {
                let source_idx = i % old_pop_size.max(1);
                let source = self
                    .population
                    .get(source_idx)
                    .map(|p| p.genotype.clone())
                    .unwrap_or_else(|| PlantGenotype::new("omega: F\nF -> F".to_string()));
                let mut variant = source;
                variant.seed = self.seed + i as u64;
                variant.mutate(&mut rng, self.mutation_rate);
                let fitness = evaluate_genotype(&variant);
                self.population.push(Phenotype {
                    genotype: variant,
                    fitness,
                    objectives: vec![],
                    descriptor: vec![],
                });
            }
        } else {
            // Trim excess individuals
            self.population.truncate(new_pop_size);
            // Remove invalid selections
            self.selected.retain(|&idx| idx < new_pop_size);
        }
    }

    /// Breeds the next generation using Interactive Evolutionary Computation (IEC).
    /// Champions (selected individuals) are preserved and used as parents.
    pub fn breed(&mut self) {
        if self.population.is_empty() {
            return;
        }

        let pop_size = self.population_size();
        let mut rng = Pcg64::seed_from_u64(self.seed.wrapping_add(self.generation as u64));

        // Identify champions (selected individuals)
        let champions: Vec<usize> = self.selected.iter().copied().collect();

        let mut new_population = Vec::with_capacity(pop_size);

        if champions.is_empty() {
            // Fallback: mutate all individuals randomly
            for (i, phenotype) in self.population.iter().enumerate() {
                let mut offspring = phenotype.genotype.clone();
                offspring.seed = self.seed.wrapping_add(self.generation as u64) + i as u64;
                offspring.mutate(&mut rng, self.mutation_rate);
                let fitness = evaluate_genotype(&offspring);
                new_population.push(Phenotype {
                    genotype: offspring,
                    fitness,
                    objectives: vec![],
                    descriptor: vec![],
                });
            }
        } else {
            // Elitism: preserve champions first
            for &idx in &champions {
                if let Some(phenotype) = self.population.get(idx) {
                    new_population.push(phenotype.clone());
                }
            }

            // Fill remaining slots with offspring from champions
            let remaining = pop_size.saturating_sub(new_population.len());
            for i in 0..remaining {
                // Randomly select two parents from champions
                let parent_a_idx = champions[rng.random_range(0..champions.len())];
                let parent_b_idx = champions[rng.random_range(0..champions.len())];

                let parent_a = &self.population[parent_a_idx].genotype;
                let parent_b = &self.population[parent_b_idx].genotype;

                // Crossover
                let mut offspring = parent_a.crossover(parent_b, &mut rng);

                // Mutation
                offspring.seed =
                    self.seed.wrapping_add(self.generation as u64) + (champions.len() + i) as u64;
                offspring.mutate(&mut rng, self.mutation_rate);

                let fitness = evaluate_genotype(&offspring);
                new_population.push(Phenotype {
                    genotype: offspring,
                    fitness,
                    objectives: vec![],
                    descriptor: vec![],
                });
            }
        }

        self.population = new_population;
        self.generation += 1;

        // Update selection to point to preserved champions (now at start of population)
        self.selected.clear();
        for i in 0..champions.len().min(pop_size) {
            self.selected.insert(i);
        }
    }

    /// Mutates all individuals in the population (except selected champions).
    pub fn mutate_all(&mut self) {
        if self.population.is_empty() {
            return;
        }

        // Increment generation first to guarantee fresh RNG seed
        self.generation += 1;

        let mut rng = Pcg64::seed_from_u64(self.seed.wrapping_add(self.generation as u64));

        for (i, phenotype) in self.population.iter_mut().enumerate() {
            // Skip selected champions
            if self.selected.contains(&i) {
                continue;
            }
            phenotype.genotype.mutate(&mut rng, self.mutation_rate);
            phenotype.fitness = evaluate_genotype(&phenotype.genotype);
        }
    }

    /// Gets the genotype at the specified index.
    pub fn get_genotype(&self, index: usize) -> Option<PlantGenotype> {
        self.population.get(index).map(|p| p.genotype.clone())
    }

    /// Toggles selection state for an individual.
    pub fn toggle_selection(&mut self, index: usize) {
        if self.selected.contains(&index) {
            self.selected.remove(&index);
        } else {
            self.selected.insert(index);
        }
    }

    /// Replaces selected individuals with a new genotype.
    ///
    /// Each selected cell receives a copy of the genotype with a unique seed,
    /// and their fitness is recalculated. This is used for preset injection.
    pub fn replace_selected(&mut self, genotype: PlantGenotype) {
        if self.selected.is_empty() {
            return;
        }

        for (i, &idx) in self.selected.iter().enumerate() {
            if let Some(phenotype) = self.population.get_mut(idx) {
                let mut variant = genotype.clone();
                // Give each variant a unique seed based on its position
                variant.seed =
                    self.seed.wrapping_add(self.generation as u64) + idx as u64 + i as u64;
                phenotype.fitness = evaluate_genotype(&variant);
                phenotype.genotype = variant;
            }
        }

        self.needs_3d_rebuild = true;
    }
}

/// Evaluates a genotype's fitness based on rule complexity and material variety.
fn evaluate_genotype(genotype: &PlantGenotype) -> f32 {
    let rule_count = genotype
        .source_code
        .lines()
        .filter(|l| l.contains("->"))
        .count();
    let material_count = genotype.materials.len();
    (rule_count as f32 * 10.0) + (material_count as f32 * 5.0)
}

/// Renders the nursery UI panel.
/// Returns `true` if nursery mode is currently enabled.
pub fn nursery_ui(
    ui: &mut egui::Ui,
    nursery: &mut NurseryState,
    config: &mut LSystemConfig,
    materials: &mut MaterialSettingsMap,
) -> bool {
    // Only show Open Nursery button when disabled; when enabled, exit via Load buttons
    if nursery.mode == NurseryMode::Disabled {
        ui.horizontal(|ui| {
            let button =
                egui::Button::new(egui::RichText::new("🌱 Open Nursery").size(16.0).strong())
                    .fill(egui::Color32::from_rgb(60, 120, 60))
                    .min_size(egui::vec2(ui.available_width(), 28.0));

            if ui.add(button).clicked() {
                nursery.initialize_from_editor(config, materials);
                nursery.needs_3d_rebuild = true;
                nursery.mode = NurseryMode::Enabled;
            }
        });
        return false;
    }

    ui.separator();

    // Controls
    ui.horizontal(|ui| {
        ui.label(format!("Generation: {}", nursery.generation));
        ui.separator();

        if ui
            .button("🧬 Breed")
            .on_hover_text("Breed next generation from selected champions")
            .clicked()
        {
            nursery.breed();
            nursery.needs_3d_rebuild = true;
        }

        if ui
            .button("🎲 Mutate")
            .on_hover_text("Mutate all non-elite individuals")
            .clicked()
        {
            nursery.mutate_all();
            nursery.needs_3d_rebuild = true;
        }

        if ui
            .button("🔄 Reset")
            .on_hover_text("Reset population from current editor")
            .clicked()
        {
            nursery.initialize_from_editor(config, materials);
            nursery.needs_3d_rebuild = true;
        }
    });

    ui.horizontal(|ui| {
        ui.label("Mutation Rate:");
        ui.add(egui::Slider::new(&mut nursery.mutation_rate, 0.01..=0.5));
    });

    ui.horizontal(|ui| {
        ui.label("Grid Spacing:");
        let old_spacing = nursery.grid_spacing;
        ui.add(egui::Slider::new(&mut nursery.grid_spacing, 50.0..=5000.0));
        if (nursery.grid_spacing - old_spacing).abs() > 0.1 {
            nursery.needs_3d_rebuild = true;
        }
    });

    // Grid size slider
    ui.horizontal(|ui| {
        ui.label("Grid Size:");
        let old_size = nursery.grid_size;
        let mut new_size = old_size as i32;
        ui.add(egui::Slider::new(&mut new_size, 2..=8).suffix("×"));
        if new_size as usize != old_size {
            nursery.resize_population(new_size as usize);
            nursery.needs_3d_rebuild = true;
        }
    });

    ui.separator();

    // Population Grid
    let grid_size = nursery.grid_size;
    let pop_data: Vec<(usize, f32)> = nursery
        .population
        .iter()
        .enumerate()
        .map(|(i, p)| (i, p.fitness))
        .collect();

    if !pop_data.is_empty() {
        let cell_size = 40.0;

        // Show selection count
        let selected_count = nursery.selected.len();
        if selected_count > 0 {
            ui.label(
                egui::RichText::new(format!("Champions: {} selected", selected_count))
                    .small()
                    .color(egui::Color32::from_rgb(100, 200, 100)),
            );
        }

        egui::Grid::new("nursery_grid")
            .num_columns(grid_size)
            .spacing([4.0, 4.0])
            .show(ui, |ui| {
                for (i, _fitness) in &pop_data {
                    let is_selected = nursery.selected.contains(i);
                    let error = nursery.errors.get(i);
                    let has_error = error.is_some();

                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(cell_size, cell_size),
                        egui::Sense::click(),
                    );

                    // Draw cell background (red tint for errors)
                    let bg_color = if has_error {
                        egui::Color32::from_rgb(80, 30, 30)
                    } else if is_selected {
                        egui::Color32::from_rgb(60, 100, 60)
                    } else if response.hovered() {
                        egui::Color32::from_rgb(50, 50, 60)
                    } else {
                        egui::Color32::from_rgb(35, 35, 40)
                    };

                    ui.painter().rect_filled(rect, 4.0, bg_color);

                    // Draw border for selected (champions) or errors
                    if has_error {
                        ui.painter().rect_stroke(
                            rect,
                            4.0,
                            egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 80, 80)),
                            egui::StrokeKind::Outside,
                        );
                    } else if is_selected {
                        ui.painter().rect_stroke(
                            rect,
                            4.0,
                            egui::Stroke::new(2.0, egui::Color32::GREEN),
                            egui::StrokeKind::Outside,
                        );
                    }

                    // Draw cell content
                    let center = rect.center();

                    /*
                    // Display index
                    ui.painter().text(
                        center - egui::vec2(0.0, 20.0),
                        egui::Align2::CENTER_CENTER,
                        format!("#{}", i + 1),
                        egui::FontId::proportional(14.0),
                        egui::Color32::WHITE,
                    );
                    */

                    if has_error {
                        // Show error icon for failed derivation
                        ui.painter().text(
                            center,
                            egui::Align2::CENTER_CENTER,
                            "⚠",
                            egui::FontId::proportional(24.0),
                            egui::Color32::from_rgb(255, 180, 100),
                        );

                        /*
                        ui.painter().text(
                            center + egui::vec2(0.0, 22.0),
                            egui::Align2::CENTER_CENTER,
                            "ERROR",
                            egui::FontId::proportional(10.0),
                            egui::Color32::from_rgb(255, 100, 100),
                        );
                        */

                        // Show tooltip with error message on hover
                        if let Some(err_msg) = error.filter(|_| response.hovered()) {
                            response.show_tooltip_text(err_msg);
                        }
                    } else {
                        // Normal display: icon and fitness
                        ui.painter().text(
                            center,
                            egui::Align2::CENTER_CENTER,
                            if is_selected { "🏆" } else { "🌿" },
                            egui::FontId::proportional(24.0),
                            egui::Color32::WHITE,
                        );

                        /*
                        ui.painter().text(
                            center + egui::vec2(0.0, 22.0),
                            egui::Align2::CENTER_CENTER,
                            format!("f:{:.0}", fitness),
                            egui::FontId::proportional(10.0),
                            egui::Color32::GRAY,
                        );
                        */
                    }

                    // Draw load button overlay in bottom-right corner
                    let load_btn_rect = egui::Rect::from_min_size(
                        rect.right_bottom() - egui::vec2(22.0, 22.0),
                        egui::vec2(20.0, 20.0),
                    );
                    let load_hovered = response.hovered()
                        && ui
                            .input(|i| i.pointer.hover_pos())
                            .map(|p| load_btn_rect.contains(p))
                            .unwrap_or(false);

                    if response.hovered() {
                        let load_bg = if load_hovered {
                            egui::Color32::from_rgb(80, 120, 180)
                        } else {
                            egui::Color32::from_rgb(60, 80, 120)
                        };
                        ui.painter().rect_filled(load_btn_rect, 3.0, load_bg);
                        ui.painter().text(
                            load_btn_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "📥",
                            egui::FontId::proportional(12.0),
                            egui::Color32::WHITE,
                        );
                    }

                    // Handle clicks
                    if response.clicked() {
                        if load_hovered {
                            // Load into editor
                            if let Some(genotype) = nursery.get_genotype(*i) {
                                let new_materials = genotype.get_material_settings();
                                config.source_code = genotype.source_code;
                                config.finalization_code = genotype.finalization_code;
                                config.iterations = genotype.iterations;
                                config.default_angle = genotype.angle;
                                config.step_size = genotype.step;
                                config.default_width = genotype.width;
                                config.seed = genotype.seed;
                                config.recompile_requested = true;
                                materials.settings.clear();
                                for (slot, mat) in new_materials {
                                    materials.settings.insert(slot, mat);
                                }
                                nursery.mode = NurseryMode::Disabled;
                            }
                        } else {
                            // Toggle selection
                            nursery.toggle_selection(*i);
                            nursery.needs_3d_rebuild = true;
                        }
                    }

                    // End row after grid_size items
                    if (i + 1) % grid_size == 0 {
                        ui.end_row();
                    }
                }
            });
    }

    true
}
