use crate::core::config::{DerivationStatus, LSystemConfig, LSystemEngine};
use bevy::prelude::*;
use symbios::System;

pub fn derive_l_system(
    mut config: ResMut<LSystemConfig>,
    mut engine: ResMut<LSystemEngine>,
    mut status: ResMut<DerivationStatus>,
) {
    if !config.recompile_requested {
        return;
    }
    config.recompile_requested = false;

    // Reset status to Success initially
    status.error = None;

    // Reset Config defaults (important so removing lines resets behavior)
    config.tropism = None;
    config.elasticity = 0.0;

    // Reset Engine
    let sys = &mut engine.0;
    *sys = System::new();

    // Clone source to avoid immutable borrow of 'config' preventing mutation later
    let source = config.source_code.clone();
    let lines: Vec<&str> = source.lines().collect();
    let mut axiom_set = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let line_num = i + 1;

        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        // Directives
        if trimmed.starts_with("#define") || trimmed.starts_with("#ignore") {
            if let Err(e) = sys.add_directive(trimmed) {
                status.error = Some(format!("Line {}: {}", line_num, e));
                return;
            }
            continue;
        }

        // Axiom
        if trimmed.starts_with("omega:") {
            let axiom = trimmed.trim_start_matches("omega:").trim();
            if let Err(e) = sys.set_axiom(axiom) {
                status.error = Some(format!("Line {}: Axiom error: {}", line_num, e));
                return;
            }
            axiom_set = true;
            continue;
        }

        // CONFIG OVERRIDES: "config: key value"
        if trimmed.starts_with("config:") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 3 {
                match parts[1] {
                    "iterations" => {
                        if let Ok(i) = parts[2].parse::<usize>() {
                            config.iterations = i;
                        }
                    }
                    "angle" => {
                        if let Ok(a) = parts[2].parse::<f32>() {
                            config.default_angle = a;
                        }
                    }
                    "step" => {
                        if let Ok(s) = parts[2].parse::<f32>() {
                            config.step_size = s;
                        }
                    }
                    "elasticity" => {
                        if let Ok(e) = parts[2].parse::<f32>() {
                            config.elasticity = e;
                        }
                    }
                    "tropism" => {
                        if parts.len() >= 5 {
                            let x = parts[2].parse().unwrap_or(0.0);
                            let y = parts[3].parse().unwrap_or(-1.0);
                            let z = parts[4].parse().unwrap_or(0.0);
                            config.tropism = Some(Vec3::new(x, y, z));
                        }
                    }
                    _ => {}
                }
            }
            continue;
        }

        // ... (Legacy syntax support if needed) ...

        // Rules
        if let Err(e) = sys.add_rule(trimmed) {
            status.error = Some(format!("Line {}: Rule error: {}", line_num, e));
            return;
        }
    }

    // ... (Constants mapping if needed) ...

    if axiom_set {
        if let Err(e) = sys.derive(config.iterations) {
            status.error = Some(format!("Derivation error: {}", e));
        }
    } else {
        status.error = Some("No axiom defined (start with 'omega: ...')".to_string());
    }
}
