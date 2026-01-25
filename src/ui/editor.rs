use crate::core::config::{
    DerivationDebounce, DerivationStatus, ExportConfig, LSystemAnalysis, LSystemConfig,
    LSystemEngine, MaterialSettingsMap, PropConfig, PropMeshType, TextureType,
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
    mut material_settings: ResMut<MaterialSettingsMap>,
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
                    let material_names = ["Mat 0 (Primary)", "Mat 1 (Energy)", "Mat 2 (Matte)"];

                    for mat_id in 0u8..3 {
                        // Read current values into locals
                        let Some(current) = material_settings.settings.get(&mat_id).cloned() else {
                            continue;
                        };

                        let mut local_base_color = current.base_color;
                        let mut local_emission_color = current.emission_color;
                        let mut local_emission_strength = current.emission_strength;
                        let mut local_roughness = current.roughness;
                        let mut local_metallic = current.metallic;
                        let mut local_texture = current.texture;

                        let mut mat_changed = false;

                        ui.collapsing(material_names[mat_id as usize], |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Base Color:");
                                mat_changed |=
                                    ui.color_edit_button_rgb(&mut local_base_color).changed();
                            });
                            ui.horizontal(|ui| {
                                ui.label("Emission:");
                                mat_changed |= ui
                                    .color_edit_button_rgb(&mut local_emission_color)
                                    .changed();
                            });
                            mat_changed |= ui
                                .add(
                                    egui::Slider::new(&mut local_emission_strength, 0.0..=10.0)
                                        .text("Glow"),
                                )
                                .changed();
                            mat_changed |= ui
                                .add(
                                    egui::Slider::new(&mut local_roughness, 0.0..=1.0)
                                        .text("Roughness"),
                                )
                                .changed();
                            mat_changed |= ui
                                .add(
                                    egui::Slider::new(&mut local_metallic, 0.0..=1.0)
                                        .text("Metallic"),
                                )
                                .changed();

                            ui.horizontal(|ui| {
                                ui.label("Texture:");
                                egui::ComboBox::from_id_salt(format!("mat_tex_{}", mat_id))
                                    .selected_text(local_texture.name())
                                    .show_ui(ui, |ui| {
                                        for tex_type in TextureType::ALL {
                                            if ui
                                                .selectable_label(
                                                    local_texture == *tex_type,
                                                    tex_type.name(),
                                                )
                                                .clicked()
                                            {
                                                local_texture = *tex_type;
                                                mat_changed = true;
                                            }
                                        }
                                    });
                            });
                        });

                        // Only write back if changed
                        if mat_changed
                            && let Some(settings) = material_settings.settings.get_mut(&mat_id)
                        {
                            settings.base_color = local_base_color;
                            settings.emission_color = local_emission_color;
                            settings.emission_strength = local_emission_strength;
                            settings.roughness = local_roughness;
                            settings.metallic = local_metallic;
                            settings.texture = local_texture;
                        }
                    }
                });

                ui.collapsing("Prop Settings", |ui| {
                    // Read current values into locals to avoid marking resource as changed
                    let mut local_prop_scale = prop_config.prop_scale;
                    let scale_changed = ui
                        .add(egui::Slider::new(&mut local_prop_scale, 0.1..=5.0).text("Prop Scale"))
                        .changed();

                    ui.separator();
                    ui.label("Surface ID Mappings:");

                    // Track mesh mapping changes
                    let mut mesh_changes: Vec<(u16, PropMeshType)> = Vec::new();

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
                                            mesh_changes.push((surface_id, *mesh_type));
                                        }
                                    }
                                });
                        });
                    }

                    // Only mutate prop_config if something actually changed
                    if scale_changed {
                        prop_config.prop_scale = local_prop_scale;
                    }
                    for (surface_id, mesh_type) in mesh_changes {
                        prop_config.surface_meshes.insert(surface_id, mesh_type);
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
