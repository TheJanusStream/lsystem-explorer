use crate::core::config::{DerivationDebounce, DerivationStatus, LSystemConfig};
use crate::core::presets::PRESETS;
use crate::visuals::turtle::TurtleRenderState; // Import RenderState
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

pub fn ui_system(
    mut contexts: EguiContexts,
    mut config: ResMut<LSystemConfig>,
    mut debounce: ResMut<DerivationDebounce>,
    status: Res<DerivationStatus>,
    render_state: Res<TurtleRenderState>, // Add this resource
    time: Res<Time>,
) {
    // Tick debounce timer
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
                // ... (Presets UI) ...
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
                                        config.recompile_requested = true;
                                        debounce.pending = false;
                                    }
                                }
                            });
                    },
                );

                ui.add_space(5.0);

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

                // Material UI
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
                ui.separator();

                ui.add_space(5.0);

                // Status Indicator
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
                    ui.vertical(|ui| {
                        ui.colored_label(egui::Color32::GREEN, "✅ Mesh Ready");
                        ui.label(format!("Vertices: {}", render_state.total_vertices));
                        ui.label(format!(
                            "Gen Time: {:.2}ms",
                            render_state.generation_time_ms
                        ));
                    });
                }

                ui.add_space(5.0);

                // Iterations
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

                ui.add_space(5.0);
                ui.checkbox(&mut config.auto_update, "Live Update");

                if !config.auto_update && ui.button("▶ Run / Recompile").clicked() {
                    config.recompile_requested = true;
                    debounce.pending = false;
                }
            });
    }
}
