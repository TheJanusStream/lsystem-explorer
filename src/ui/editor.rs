use crate::core::config::{DerivationDebounce, DerivationStatus, LSystemConfig};
use crate::core::presets::PRESETS;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

pub fn ui_system(
    mut contexts: EguiContexts,
    mut config: ResMut<LSystemConfig>,
    mut debounce: ResMut<DerivationDebounce>,
    status: Res<DerivationStatus>,
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
                                        // Cancel debounce on explicit load
                                        debounce.pending = false;
                                    }
                                }
                            });
                    },
                );

                ui.add_space(5.0);

                egui::ScrollArea::vertical()
                    .min_scrolled_height(300.0)
                    .show(ui, |ui| {
                        let response = ui.add(
                            egui::TextEdit::multiline(&mut config.source_code)
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                                .desired_width(f32::INFINITY),
                        );

                        if response.changed() {
                            if config.auto_update {
                                debounce.timer.reset();
                                debounce.pending = true;
                            }
                        }
                    });

                ui.add_space(5.0);

                // Status Indicator
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
                    ui.colored_label(egui::Color32::GREEN, "✅ System Valid");
                }

                ui.add_space(5.0);

                // Iterations: Stepper Buttons (Safety vs Slider)
                ui.horizontal(|ui| {
                    ui.label("Iterations:");

                    if ui.button("➖").clicked() && config.iterations > 0 {
                        config.iterations -= 1;
                        config.recompile_requested = true;
                        debounce.pending = false;
                    }

                    // Display count
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

                if !config.auto_update {
                    if ui.button("▶ Run / Recompile").clicked() {
                        config.recompile_requested = true;
                        debounce.pending = false;
                    }
                }
            });
    }
}
