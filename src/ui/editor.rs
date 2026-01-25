use crate::core::config::{
    DerivationDebounce, DerivationStatus, ExportConfig, LSystemAnalysis, LSystemConfig,
    LSystemEngine, PropConfig, PropMeshType,
};
use crate::core::presets::PRESETS;
use crate::visuals::turtle::TurtleRenderState;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

#[allow(clippy::too_many_arguments)]
pub fn ui_system(
    mut contexts: EguiContexts,
    mut config: ResMut<LSystemConfig>,
    engine: ResMut<LSystemEngine>,
    mut prop_config: ResMut<PropConfig>,
    mut export_config: ResMut<ExportConfig>,
    mut debounce: ResMut<DerivationDebounce>,
    status: Res<DerivationStatus>,
    analysis: Res<LSystemAnalysis>,
    render_state: Res<TurtleRenderState>,
    time: Res<Time>,
) {
    // Handle Debounce
    if debounce.pending {
        debounce.timer.tick(time.delta());
        if debounce.timer.is_finished() {
            config.recompile_requested = true;
            debounce.pending = false;
        }
    }

    if let Ok(ctx) = contexts.ctx_mut() {
        egui::Window::new("Symbios Lab")
            .default_width(350.0)
            .show(ctx, |ui| {
                // --- PRESETS ---
                egui::containers::Sides::new().show(
                    ui,
                    |ui| {
                        ui.heading("Grammar:");
                    },
                    |ui| {
                        egui::ComboBox::from_id_salt("preset_combo")
                            .selected_text("Presets...")
                            .show_ui(ui, |ui| {
                                for preset in PRESETS {
                                    if ui.selectable_label(false, preset.name).clicked() {
                                        config.source_code = preset.code.to_string();
                                        config.iterations = preset.iterations;
                                        config.default_angle = preset.angle;
                                        config.step_size = preset.step;
                                        config.default_width = preset.width;
                                        config.elasticity = preset.elasticity;
                                        config.tropism = preset.tropism;
                                        config.recompile_requested = true;
                                        debounce.pending = false;
                                    }
                                }
                            });
                    },
                );

                ui.add_space(5.0);

                // --- EDITOR ---
                egui::ScrollArea::vertical()
                    .min_scrolled_height(200.0)
                    .id_salt("source_scroll")
                    .show(ui, |ui| {
                        let response = ui.add(
                            egui::TextEdit::multiline(&mut config.source_code)
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                                .desired_width(f32::INFINITY),
                        );
                        if response.changed() && config.auto_update {
                            debounce.timer.reset();
                            debounce.pending = true;
                        }
                    });

                ui.add_space(5.0);
                ui.separator();

                // --- DEFINED CONSTANTS ---
                let sys = &engine.0;
                if !sys.constants.is_empty() {
                    ui.heading("Defined Constants:");

                    let mut keys: Vec<String> = sys.constants.keys().cloned().collect();
                    keys.sort();

                    let mut constants_changed = false;

                    egui::Grid::new("constants_grid")
                        .num_columns(2)
                        .striped(true)
                        .show(ui, |ui| {
                            for key in keys {
                                if let Some(&current_val) = sys.constants.get(&key) {
                                    ui.label(format!("{}:", key));

                                    let mut val_f32 = current_val as f32;
                                    let speed = dynamic_drag_speed(val_f32);

                                    if ui
                                        .add(egui::DragValue::new(&mut val_f32).speed(speed))
                                        .changed()
                                    {
                                        let new_source = update_define_in_source(
                                            &config.source_code,
                                            &key,
                                            val_f32,
                                        );
                                        config.source_code = new_source;
                                        constants_changed = true;
                                    }
                                    ui.end_row();
                                }
                            }
                        });

                    if constants_changed {
                        config.recompile_requested = true;
                        debounce.pending = false;
                    }

                    ui.add_space(5.0);
                    ui.separator();
                }

                // --- INTERPRETATION SETTINGS ---
                ui.heading("Interpretation:");

                if analysis.uses_implicit_step
                    || analysis.uses_implicit_angle
                    || !analysis.uses_explicit_width
                {
                    ui.horizontal(|ui| {
                        if analysis.uses_implicit_step {
                            ui.label("Step:");
                            if ui
                                .add(
                                    egui::DragValue::new(&mut config.step_size)
                                        .speed(0.1)
                                        .range(0.1..=100.0),
                                )
                                .changed()
                            {
                                config.recompile_requested = true;
                            }
                        }
                        if analysis.uses_implicit_angle {
                            ui.label("Angle:");
                            if ui
                                .add(
                                    egui::DragValue::new(&mut config.default_angle)
                                        .speed(1.0)
                                        .range(0.0..=180.0),
                                )
                                .changed()
                            {
                                config.recompile_requested = true;
                            }
                        }
                        if !analysis.uses_explicit_width {
                            ui.label("Width:");
                            if ui
                                .add(
                                    egui::DragValue::new(&mut config.default_width)
                                        .speed(0.01)
                                        .range(0.001..=10.0),
                                )
                                .changed()
                            {
                                config.recompile_requested = true;
                            }
                        }
                    });
                }

                ui.horizontal(|ui| {
                    ui.label("Iterations:");
                    if ui.button("➖").clicked() && config.iterations > 0 {
                        config.iterations -= 1;
                        config.recompile_requested = true;
                        debounce.pending = false;
                    }
                    ui.label(
                        egui::RichText::new(format!("{}", config.iterations))
                            .strong()
                            .size(16.0),
                    );
                    if ui.button("➕").clicked() {
                        config.iterations += 1;
                        config.recompile_requested = true;
                        debounce.pending = false;
                    }
                });

                ui.collapsing("Physics & Tropism", |ui| {
                    if ui
                        .add(
                            egui::Slider::new(&mut config.elasticity, 0.0..=1.0).text("Elasticity"),
                        )
                        .changed()
                    {
                        config.recompile_requested = true;
                    }

                    let mut tropism_active = config.tropism.is_some();
                    if ui.checkbox(&mut tropism_active, "Enable Tropism").changed() {
                        config.tropism = if tropism_active {
                            Some(Vec3::NEG_Y)
                        } else {
                            None
                        };
                        config.recompile_requested = true;
                    }

                    // FIX: Track changes in a boolean to avoid holding mutable borrow of `config`
                    let mut tropism_changed = false;
                    if let Some(t) = &mut config.tropism {
                        ui.horizontal(|ui| {
                            ui.label("Vec:");
                            tropism_changed |=
                                ui.add(egui::DragValue::new(&mut t.x).speed(0.1)).changed();
                            tropism_changed |=
                                ui.add(egui::DragValue::new(&mut t.y).speed(0.1)).changed();
                            tropism_changed |=
                                ui.add(egui::DragValue::new(&mut t.z).speed(0.1)).changed();
                        });
                    }
                    // Apply change after borrow ends
                    if tropism_changed {
                        config.recompile_requested = true;
                    }
                });

                ui.add_space(5.0);
                ui.separator();

                ui.collapsing("Material Settings", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Base Color:");
                        ui.color_edit_button_rgb(&mut config.material_color);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Emission:");
                        ui.color_edit_button_rgb(&mut config.emission_color);
                    });
                    ui.add(
                        egui::Slider::new(&mut config.emission_strength, 0.0..=10.0)
                            .text("Glow Strength"),
                    );
                });

                ui.collapsing("Prop Settings", |ui| {
                    ui.add(
                        egui::Slider::new(&mut prop_config.prop_scale, 0.1..=5.0)
                            .text("Prop Scale"),
                    );

                    ui.separator();
                    ui.label("Surface ID Mappings:");

                    // Show mappings for surface IDs 0-3
                    for surface_id in 0u16..4 {
                        ui.horizontal(|ui| {
                            ui.label(format!("~{}", surface_id));

                            let current = prop_config
                                .surface_meshes
                                .get(&surface_id)
                                .copied()
                                .unwrap_or(PropMeshType::Leaf);

                            egui::ComboBox::from_id_salt(format!("prop_mesh_{}", surface_id))
                                .selected_text(current.name())
                                .show_ui(ui, |ui| {
                                    for mesh_type in PropMeshType::ALL {
                                        if ui
                                            .selectable_label(
                                                current == *mesh_type,
                                                mesh_type.name(),
                                            )
                                            .clicked()
                                        {
                                            prop_config
                                                .surface_meshes
                                                .insert(surface_id, *mesh_type);
                                        }
                                    }
                                });
                        });
                    }
                });

                ui.collapsing("Batch Export", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Base Name:");
                        ui.text_edit_singleline(&mut export_config.base_filename);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Variations:");
                        ui.add(
                            egui::DragValue::new(&mut export_config.variation_count)
                                .range(1..=100)
                                .speed(0.5),
                        );
                    });

                    ui.add_space(5.0);

                    if ui.button("Export OBJ Files").clicked() {
                        export_config.export_requested = true;
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    ui.label(
                        egui::RichText::new("Files saved to ./exports/")
                            .small()
                            .color(egui::Color32::GRAY),
                    );

                    #[cfg(target_arch = "wasm32")]
                    ui.label(
                        egui::RichText::new("Files download via browser")
                            .small()
                            .color(egui::Color32::GRAY),
                    );
                });

                ui.add_space(5.0);

                // --- STATUS ---
                if let Some(err) = &status.error {
                    ui.group(|ui| {
                        ui.colored_label(egui::Color32::RED, "❌ Parse Error:");
                        ui.label(
                            egui::RichText::new(err)
                                .color(egui::Color32::from_rgb(255, 100, 100))
                                .small(),
                        );
                    });
                } else if debounce.pending {
                    ui.colored_label(egui::Color32::YELLOW, "⏳ Typing...");
                } else {
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::GREEN, "✅ Mesh Ready");
                        ui.label(format!(
                            "| {} Verts | {:.2}ms",
                            render_state.total_vertices, render_state.generation_time_ms
                        ));
                    });
                }

                ui.checkbox(&mut config.auto_update, "Live Update");
                if !config.auto_update && ui.button("▶ Run / Recompile").clicked() {
                    config.recompile_requested = true;
                    debounce.pending = false;
                }
            });
    }
}

/// Calculate appropriate drag speed based on value magnitude using log10.
/// Returns a speed that provides ~1% change per pixel of drag.
fn dynamic_drag_speed(value: f32) -> f64 {
    let abs_val = value.abs();
    if abs_val < 0.0001 {
        return 0.001; // Minimum speed for near-zero values
    }
    let magnitude = abs_val.log10().floor();
    (10.0_f32.powf(magnitude - 1.0)) as f64
}

/// Helper to update a #define value in the source string.
fn update_define_in_source(source: &str, key: &str, new_value: f32) -> String {
    let mut new_lines = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#define") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            // Expected parts: ["#define", "KEY", "VALUE", ...]
            if parts.len() >= 2 && parts[1] == key {
                // Reconstruct the line
                new_lines.push(format!("#define {} {}", key, new_value));
                continue;
            }
        }
        new_lines.push(line.to_string());
    }

    new_lines.join("\n")
}
