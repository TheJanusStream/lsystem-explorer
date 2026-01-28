use crate::core::config::{
    DerivationDebounce, DerivationStatus, DirtyFlags, ExportConfig, ExportFormat, LSystemAnalysis,
    LSystemConfig, LSystemEngine, MaterialSettingsMap, PropConfig, PropMeshType,
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
    mut dirty: ResMut<DirtyFlags>,
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
                // --- GRAMMAR (Collapsible) ---
                egui::CollapsingHeader::new("Grammar")
                    .default_open(true)
                    .show(ui, |ui| {
                        // Presets dropdown aligned right
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
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
                        });

                        ui.add_space(5.0);

                        // Editor with full available width
                        egui::ScrollArea::vertical()
                            .min_scrolled_height(200.0)
                            .id_salt("source_scroll")
                            .show(ui, |ui| {
                                let response = ui.add(
                                    egui::TextEdit::multiline(&mut config.source_code)
                                        .code_editor()
                                        .desired_width(f32::INFINITY)
                                        .layouter(&mut |ui, text, wrap_width| {
                                            let font_id =
                                                egui::TextStyle::Monospace.resolve(ui.style());
                                            let mut job = highlight_lsystem(text.as_str(), font_id);
                                            job.wrap.max_width = wrap_width;
                                            ui.ctx().fonts_mut(|f| f.layout_job(job))
                                        }),
                                );
                                if response.changed() && config.auto_update {
                                    debounce.timer.reset();
                                    debounce.pending = true;
                                }
                            });
                    });

                ui.add_space(5.0);

                // --- DEFINED CONSTANTS (Collapsible) ---
                let sys = &engine.0;
                if !sys.constants.is_empty() {
                    egui::CollapsingHeader::new("Defined Constants")
                        .default_open(true)
                        .show(ui, |ui| {
                            let mut keys: Vec<String> = sys.constants.keys().cloned().collect();
                            keys.sort();

                            let mut constants_changed = false;
                            let available_width = ui.available_width();

                            for key in keys {
                                if let Some(&current_val) = sys.constants.get(&key) {
                                    let mut val_f32 = current_val as f32;
                                    let (lo, hi) = smart_slider_range(val_f32);

                                    ui.horizontal(|ui| {
                                        ui.set_min_width(available_width);
                                        if ui
                                            .add_sized(
                                                [available_width, ui.spacing().interact_size.y],
                                                egui::Slider::new(&mut val_f32, lo..=hi)
                                                    .text(&key)
                                                    .clamping(egui::SliderClamping::Never),
                                            )
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
                                    });
                                }
                            }

                            if constants_changed {
                                config.recompile_requested = true;
                                debounce.pending = false;
                            }
                        });

                    ui.add_space(5.0);
                }

                // --- INTERPRETATION SETTINGS ---
                ui.heading("Interpretation:");

                if analysis.uses_implicit_step
                    && ui
                        .add(
                            egui::Slider::new(&mut config.step_size, 0.1..=100.0)
                                .text("Step")
                                .logarithmic(true),
                        )
                        .changed()
                {
                    config.recompile_requested = true;
                }
                if analysis.uses_implicit_angle
                    && ui
                        .add(
                            egui::Slider::new(&mut config.default_angle, 0.0..=180.0).text("Angle"),
                        )
                        .changed()
                {
                    config.recompile_requested = true;
                }
                if !analysis.uses_explicit_width
                    && ui
                        .add(
                            egui::Slider::new(&mut config.default_width, 0.001..=10.0)
                                .text("Width")
                                .logarithmic(true),
                        )
                        .changed()
                {
                    config.recompile_requested = true;
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
                    if tropism_changed {
                        config.recompile_requested = true;
                    }
                });

                ui.add_space(5.0);
                ui.separator();

                // --- MATERIAL PALETTE ---
                ui.collapsing("Material Palette", |ui| {
                    bevy_symbios::ui::material_palette_editor(ui, &mut material_settings.settings);
                });

                ui.collapsing("Prop Settings", |ui| {
                    let mut local_prop_scale = prop_config.prop_scale;
                    let scale_changed = ui
                        .add(egui::Slider::new(&mut local_prop_scale, 0.1..=5.0).text("Prop Scale"))
                        .changed();

                    ui.separator();
                    ui.label("Prop ID Mappings:");

                    let mut mesh_changes: Vec<(u16, PropMeshType)> = Vec::new();

                    for prop_id in 0u16..4 {
                        ui.horizontal(|ui| {
                            ui.label(format!("~{}", prop_id));

                            let current = prop_config
                                .prop_meshes
                                .get(&prop_id)
                                .copied()
                                .unwrap_or(PropMeshType::Leaf);

                            egui::ComboBox::from_id_salt(format!("prop_mesh_{}", prop_id))
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
                                            mesh_changes.push((prop_id, *mesh_type));
                                        }
                                    }
                                });
                        });
                    }

                    if scale_changed {
                        prop_config.prop_scale = local_prop_scale;
                        dirty.geometry = true;
                    }
                    for (prop_id, mesh_type) in mesh_changes {
                        prop_config.prop_meshes.insert(prop_id, mesh_type);
                        dirty.geometry = true;
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

                    ui.horizontal(|ui| {
                        ui.label("Format:");
                        egui::ComboBox::from_id_salt("export_format")
                            .selected_text(export_config.format.name())
                            .show_ui(ui, |ui| {
                                for fmt in ExportFormat::ALL {
                                    if ui
                                        .selectable_label(export_config.format == *fmt, fmt.name())
                                        .clicked()
                                    {
                                        export_config.format = *fmt;
                                    }
                                }
                            });
                    });

                    ui.add_space(5.0);

                    if ui
                        .button(format!("Export {} Files", export_config.format.name()))
                        .clicked()
                    {
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
                if status.generating {
                    ui.colored_label(egui::Color32::YELLOW, "⏳ Generating...");
                } else if let Some(err) = &status.error {
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
                        let total_ms =
                            render_state.derivation_time_ms + render_state.meshing_time_ms;
                        ui.label(format!(
                            "| {} Verts | {:.1}ms (D:{:.1} M:{:.1})",
                            render_state.total_vertices,
                            total_ms,
                            render_state.derivation_time_ms,
                            render_state.meshing_time_ms,
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

/// Compute a slider range centered on the current value.
///
/// For zero or near-zero: [-1, 1].
/// For negative values: [2*val, 0] (or [2*val, -2*val] if very negative).
/// For positive values: [0, 2*val].
fn smart_slider_range(value: f32) -> (f32, f32) {
    let abs_val = value.abs();
    if abs_val < 0.001 {
        return (-1.0, 1.0);
    }
    let extent = abs_val * 2.0;
    if value < 0.0 {
        (-extent, extent)
    } else {
        (0.0, extent)
    }
}

/// Helper to update a #define value in the source string.
fn update_define_in_source(source: &str, key: &str, new_value: f32) -> String {
    let mut new_lines = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#define") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == key {
                new_lines.push(format!("#define {} {}", key, new_value));
                continue;
            }
        }
        new_lines.push(line.to_string());
    }

    new_lines.join("\n")
}

// --- Syntax Highlighting ---

const HL_COMMENT: egui::Color32 = egui::Color32::from_rgb(0x6A, 0x99, 0x55);
const HL_DIRECTIVE: egui::Color32 = egui::Color32::from_rgb(0xC5, 0x86, 0xC0);
const HL_KEYWORD: egui::Color32 = egui::Color32::from_rgb(0x56, 0x9C, 0xD6);
const HL_RULE_LABEL: egui::Color32 = egui::Color32::from_rgb(0x4E, 0xC9, 0xB0);
const HL_NUMBER: egui::Color32 = egui::Color32::from_rgb(0xB5, 0xCE, 0xA8);
const HL_ARROW: egui::Color32 = egui::Color32::from_rgb(0xD4, 0xD4, 0xD4);
const HL_BRACKET: egui::Color32 = egui::Color32::from_rgb(0xDA, 0xDA, 0x6E);
const HL_SYMBOL: egui::Color32 = egui::Color32::from_rgb(0x9C, 0xDC, 0xFE);
const HL_SPECIAL: egui::Color32 = egui::Color32::from_rgb(0xCE, 0x91, 0x78);
const HL_DEFAULT: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);

fn highlight_lsystem(text: &str, font_id: egui::FontId) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob {
        text: text.to_string(),
        ..Default::default()
    };

    let mut pos = 0;
    for line in text.split_inclusive('\n') {
        let line_end = pos + line.len();
        let trimmed = line.trim();
        let ws = line.len() - line.trim_start().len();
        let content_start = pos + ws;

        if trimmed.is_empty() {
            push_hl(&mut job, pos, line_end, HL_DEFAULT, &font_id);
        } else if trimmed.starts_with("//") {
            push_hl(&mut job, pos, line_end, HL_COMMENT, &font_id);
        } else if trimmed.starts_with('#') {
            if ws > 0 {
                push_hl(&mut job, pos, content_start, HL_DEFAULT, &font_id);
            }
            let kw_end = trimmed
                .find(|c: char| c == ':' || c.is_ascii_whitespace())
                .unwrap_or(trimmed.len());
            push_hl(
                &mut job,
                content_start,
                content_start + kw_end,
                HL_DIRECTIVE,
                &font_id,
            );
            highlight_body(&mut job, text, content_start + kw_end, line_end, &font_id);
        } else if trimmed.starts_with("omega:") {
            if ws > 0 {
                push_hl(&mut job, pos, content_start, HL_DEFAULT, &font_id);
            }
            let kw_len = "omega:".len();
            push_hl(
                &mut job,
                content_start,
                content_start + kw_len,
                HL_KEYWORD,
                &font_id,
            );
            highlight_body(&mut job, text, content_start + kw_len, line_end, &font_id);
        } else if let Some(colon) = trimmed.find(':') {
            // Check for rule label pattern: pN:
            let prefix = &trimmed[..colon];
            if prefix.starts_with('p')
                && prefix.len() > 1
                && prefix[1..].chars().all(|c| c.is_ascii_digit())
            {
                if ws > 0 {
                    push_hl(&mut job, pos, content_start, HL_DEFAULT, &font_id);
                }
                let label_len = colon + 1;
                push_hl(
                    &mut job,
                    content_start,
                    content_start + label_len,
                    HL_RULE_LABEL,
                    &font_id,
                );
                highlight_body(
                    &mut job,
                    text,
                    content_start + label_len,
                    line_end,
                    &font_id,
                );
            } else {
                if ws > 0 {
                    push_hl(&mut job, pos, content_start, HL_DEFAULT, &font_id);
                }
                highlight_body(&mut job, text, content_start, line_end, &font_id);
            }
        } else {
            if ws > 0 {
                push_hl(&mut job, pos, content_start, HL_DEFAULT, &font_id);
            }
            highlight_body(&mut job, text, content_start, line_end, &font_id);
        }

        pos = line_end;
    }

    // Handle text not ending with newline (split_inclusive still yields it, but
    // ensure we haven't missed trailing content).
    if pos < text.len() {
        push_hl(&mut job, pos, text.len(), HL_DEFAULT, &font_id);
    }

    job
}

/// Token-level highlighting for rule/axiom body content.
fn highlight_body(
    job: &mut egui::text::LayoutJob,
    text: &str,
    start: usize,
    end: usize,
    font_id: &egui::FontId,
) {
    if start >= end {
        return;
    }

    let bytes = text.as_bytes();
    let mut i = start;

    while i < end {
        let b = bytes[i];

        // Arrow ->
        if b == b'-' && i + 1 < end && bytes[i + 1] == b'>' {
            push_hl(job, i, i + 2, HL_ARROW, font_id);
            i += 2;
            continue;
        }

        // Numbers
        if b.is_ascii_digit() {
            let s = i;
            while i < end
                && (bytes[i].is_ascii_digit()
                    || (bytes[i] == b'.' && i + 1 < end && bytes[i + 1].is_ascii_digit()))
            {
                i += 1;
            }
            push_hl(job, s, i, HL_NUMBER, font_id);
            continue;
        }

        // Brackets
        if b == b'[' || b == b']' {
            push_hl(job, i, i + 1, HL_BRACKET, font_id);
            i += 1;
            continue;
        }

        // Turtle symbols
        if b"Ff+-&^/\\|$".contains(&b) {
            push_hl(job, i, i + 1, HL_SYMBOL, font_id);
            i += 1;
            continue;
        }

        // Prop / material / color / width symbols
        if b"~,';!".contains(&b) {
            push_hl(job, i, i + 1, HL_SPECIAL, font_id);
            i += 1;
            continue;
        }

        // Default run: accumulate until next token
        let s = i;
        while i < end {
            let c = bytes[i];
            if c == b'-' && i + 1 < end && bytes[i + 1] == b'>' {
                break;
            }
            if c.is_ascii_digit() || b"[]Ff+-&^/\\|$~,';!".contains(&c) {
                break;
            }
            i += 1;
        }
        if s < i {
            push_hl(job, s, i, HL_DEFAULT, font_id);
        }
    }
}

fn push_hl(
    job: &mut egui::text::LayoutJob,
    start: usize,
    end: usize,
    color: egui::Color32,
    font_id: &egui::FontId,
) {
    if start >= end {
        return;
    }
    job.sections.push(egui::text::LayoutSection {
        leading_space: 0.0,
        byte_range: start..end,
        format: egui::TextFormat::simple(font_id.clone(), color),
    });
}
