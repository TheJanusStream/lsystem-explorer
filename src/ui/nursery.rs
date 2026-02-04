//! Breeding Nursery UI for evolutionary L-system exploration.
//!
//! This module provides a grid-based interface for visualizing and evolving
//! populations of plant genotypes using genetic algorithms.

use crate::core::config::{LSystemConfig, MaterialSettingsMap};
use crate::core::genotype::PlantGenotype;
use bevy::prelude::*;
use bevy_egui::egui;
use rand::SeedableRng;
use rand_pcg::Pcg64;
use symbios_genetics::{Evaluator, Evolver, Genotype, Phenotype, algorithms::simple::SimpleGA};

/// Number of individuals in the nursery population.
pub const POPULATION_SIZE: usize = 9;

/// Grid dimensions for display (3x3).
pub const GRID_COLS: usize = 3;

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
    /// The evolutionary algorithm managing the population.
    pub evolver: Option<SimpleGA<PlantGenotype>>,
    /// Currently selected individual index (for loading into editor).
    pub selected: Option<usize>,
    /// Mutation rate for breeding operations.
    pub mutation_rate: f32,
    /// RNG seed for reproducibility.
    pub seed: u64,
    /// Generation counter.
    pub generation: usize,
}

impl Default for NurseryState {
    fn default() -> Self {
        Self {
            mode: NurseryMode::Disabled,
            evolver: None,
            selected: None,
            mutation_rate: 0.15,
            seed: 42,
            generation: 0,
        }
    }
}

impl NurseryState {
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

        // Create initial population with variations
        let mut rng = Pcg64::seed_from_u64(self.seed);
        let mut initial_pop: Vec<PlantGenotype> = Vec::with_capacity(POPULATION_SIZE);

        // First individual is the original
        initial_pop.push(base.clone());

        // Rest are mutated variants
        for i in 1..POPULATION_SIZE {
            let mut variant = base.clone();
            variant.seed = self.seed + i as u64;
            variant.mutate(&mut rng, self.mutation_rate);
            initial_pop.push(variant);
        }

        // Create the evolutionary algorithm
        self.evolver = Some(SimpleGA::new(
            initial_pop,
            self.mutation_rate,
            2, // elitism: keep top 2
            self.seed,
        ));
        self.generation = 0;
        self.selected = Some(0);
    }

    /// Returns the current population as phenotypes.
    pub fn population(&mut self) -> Option<&[Phenotype<PlantGenotype>]> {
        self.evolver.as_mut().map(|e| e.population())
    }

    /// Runs one generation of evolution.
    pub fn evolve_step(&mut self) {
        if let Some(evolver) = &mut self.evolver {
            evolver.step(&PlantEvaluator);
            self.generation += 1;
        }
    }

    /// Mutates all individuals in the population (except elites).
    pub fn mutate_all(&mut self) {
        let Some(evolver) = &mut self.evolver else {
            return;
        };

        let mut rng = Pcg64::seed_from_u64(self.seed.wrapping_add(self.generation as u64));

        // Get elitism count to preserve top individuals
        let elitism = evolver.elitism();

        // Clone and mutate genotypes
        let mutated: Vec<PlantGenotype> = evolver
            .population()
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let mut g = p.genotype.clone();
                if i >= elitism {
                    g.mutate(&mut rng, self.mutation_rate);
                }
                g
            })
            .collect();

        // Recreate evolver with mutated population
        self.evolver = Some(SimpleGA::new(
            mutated,
            self.mutation_rate,
            elitism,
            self.seed.wrapping_add(self.generation as u64),
        ));
    }

    /// Gets the genotype at the specified index.
    pub fn get_genotype(&mut self, index: usize) -> Option<PlantGenotype> {
        self.population()
            .and_then(|pop| pop.get(index))
            .map(|p| p.genotype.clone())
    }
}

/// Simple fitness evaluator for plant genotypes.
///
/// Currently uses a basic fitness based on rule count and parameter complexity.
/// This can be extended with actual rendering metrics in the future.
struct PlantEvaluator;

impl Evaluator<PlantGenotype> for PlantEvaluator {
    fn evaluate(&self, genotype: &PlantGenotype) -> (f32, Vec<f32>, Vec<f32>) {
        // Simple fitness: more rules = more complex = potentially more interesting
        let rule_count = genotype
            .source_code
            .lines()
            .filter(|l| l.contains("->"))
            .count();

        // Material variety bonus
        let material_count = genotype.materials.len();

        // Compute fitness (higher is better)
        let fitness = (rule_count as f32 * 10.0) + (material_count as f32 * 5.0);

        (fitness, vec![fitness], vec![])
    }
}

/// Renders the nursery UI panel.
pub fn nursery_ui(
    ui: &mut egui::Ui,
    nursery: &mut NurseryState,
    config: &mut LSystemConfig,
    materials: &mut MaterialSettingsMap,
) {
    ui.horizontal(|ui| {
        let mode_text = match nursery.mode {
            NurseryMode::Disabled => "🌱 Open Nursery",
            NurseryMode::Enabled => "🌿 Close Nursery",
        };

        if ui.button(mode_text).clicked() {
            nursery.mode = match nursery.mode {
                NurseryMode::Disabled => {
                    // Initialize population when opening
                    nursery.initialize_from_editor(config, materials);
                    NurseryMode::Enabled
                }
                NurseryMode::Enabled => NurseryMode::Disabled,
            };
        }
    });

    if nursery.mode == NurseryMode::Disabled {
        return;
    }

    ui.separator();

    // Controls
    ui.horizontal(|ui| {
        ui.label(format!("Generation: {}", nursery.generation));
        ui.separator();

        if ui
            .button("🧬 Breed")
            .on_hover_text("Run one generation of evolution")
            .clicked()
        {
            nursery.evolve_step();
        }

        if ui
            .button("🎲 Mutate")
            .on_hover_text("Mutate all non-elite individuals")
            .clicked()
        {
            nursery.mutate_all();
        }

        if ui
            .button("🔄 Reset")
            .on_hover_text("Reset population from current editor")
            .clicked()
        {
            nursery.initialize_from_editor(config, materials);
        }
    });

    ui.horizontal(|ui| {
        ui.label("Mutation Rate:");
        ui.add(egui::Slider::new(&mut nursery.mutation_rate, 0.01..=0.5));
    });

    ui.separator();

    // Population Grid - extract data first to avoid borrow issues
    let pop_data: Option<Vec<(PlantGenotype, f32)>> = nursery.evolver.as_mut().map(|evolver| {
        evolver
            .population()
            .iter()
            .map(|p| (p.genotype.clone(), p.fitness))
            .collect()
    });

    if let Some(pop) = pop_data {
        let cell_size = 80.0;

        egui::Grid::new("nursery_grid")
            .num_columns(GRID_COLS)
            .spacing([4.0, 4.0])
            .show(ui, |ui| {
                for (i, (_, fitness)) in pop.iter().enumerate() {
                    let is_selected = nursery.selected == Some(i);

                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(cell_size, cell_size),
                        egui::Sense::click(),
                    );

                    // Draw cell background
                    let bg_color = if is_selected {
                        egui::Color32::from_rgb(60, 100, 60)
                    } else if response.hovered() {
                        egui::Color32::from_rgb(50, 50, 60)
                    } else {
                        egui::Color32::from_rgb(35, 35, 40)
                    };

                    ui.painter().rect_filled(rect, 4.0, bg_color);

                    // Draw border for selected
                    if is_selected {
                        ui.painter().rect_stroke(
                            rect,
                            4.0,
                            egui::Stroke::new(2.0, egui::Color32::GREEN),
                            egui::StrokeKind::Outside,
                        );
                    }

                    // Draw cell content
                    let center = rect.center();

                    // Display index and fitness
                    ui.painter().text(
                        center - egui::vec2(0.0, 20.0),
                        egui::Align2::CENTER_CENTER,
                        format!("#{}", i + 1),
                        egui::FontId::proportional(14.0),
                        egui::Color32::WHITE,
                    );

                    ui.painter().text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        "🌿",
                        egui::FontId::proportional(24.0),
                        egui::Color32::WHITE,
                    );

                    ui.painter().text(
                        center + egui::vec2(0.0, 22.0),
                        egui::Align2::CENTER_CENTER,
                        format!("f:{:.0}", fitness),
                        egui::FontId::proportional(10.0),
                        egui::Color32::GRAY,
                    );

                    // Handle selection
                    if response.clicked() {
                        nursery.selected = Some(i);
                    }

                    // End row after GRID_COLS items
                    if (i + 1) % GRID_COLS == 0 {
                        ui.end_row();
                    }
                }
            });
    }

    ui.separator();

    // Load selected into editor
    if let Some(selected_idx) = nursery.selected {
        if ui
            .button("📥 Load Selected into Editor")
            .on_hover_text("Replace editor content with selected individual")
            .clicked()
            && let Some(genotype) = nursery.get_genotype(selected_idx)
        {
            // Get materials first before moving fields
            let new_materials = genotype.get_material_settings();

            // Update config
            config.source_code = genotype.source_code;
            config.finalization_code = genotype.finalization_code;
            config.iterations = genotype.iterations;
            config.default_angle = genotype.angle;
            config.step_size = genotype.step;
            config.default_width = genotype.width;
            config.seed = genotype.seed;
            config.recompile_requested = true;

            // Update materials
            materials.settings.clear();
            for (slot, mat) in new_materials {
                materials.settings.insert(slot, mat);
            }
        }

        ui.label(
            egui::RichText::new(format!("Selected: #{}", selected_idx + 1))
                .small()
                .color(egui::Color32::GRAY),
        );
    }
}
