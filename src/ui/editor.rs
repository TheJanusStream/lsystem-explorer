use crate::core::config::{DerivationDebounce, DerivationStatus, LSystemAnalysis, LSystemConfig};
use crate::core::presets::PRESETS;
use crate::visuals::turtle::TurtleRenderState;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

pub fn ui_system(
    mut contexts: EguiContexts,
    mut config: ResMut<LSystemConfig>,
    mut debounce: ResMut<DerivationDebounce>,
    status: Res<DerivationStatus>,
    analysis: Res<LSystemAnalysis>,
    render_state: Res<TurtleRenderState>,
    time: Res<Time>,
) {
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

                // Editor
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

                // --- DYNAMIC PARAMETERS ---
                ui.heading("Interpretation:");

                if analysis.uses_implicit_step || analysis.uses_implicit_angle {
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
                    });
                } else {
                    ui.label(
                        egui::RichText::new("Fully Parametric System (No Defaults Needed)")
                            .small()
                            .italics(),
                    );
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
                        if tropism_active {
                            config.tropism = Some(Vec3::NEG_Y);
                        } else {
                            config.tropism = None;
                        }
                        config.recompile_requested = true;
                    }

                    let mut tropism_changed = false;
                    if let Some(t) = &mut config.tropism {
                        ui.horizontal(|ui| {
                            ui.label("Vec:");
                            if ui.add(egui::DragValue::new(&mut t.x).speed(0.1)).changed() {
                                tropism_changed = true;
                            }
                            if ui.add(egui::DragValue::new(&mut t.y).speed(0.1)).changed() {
                                tropism_changed = true;
                            }
                            if ui.add(egui::DragValue::new(&mut t.z).speed(0.1)).changed() {
                                tropism_changed = true;
                            }
                        });
                    }
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

                ui.add_space(5.0);

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
