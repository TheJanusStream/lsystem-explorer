use crate::core::config::{DerivationStatus, LSystemAnalysis, LSystemConfig, LSystemEngine};
use bevy::prelude::*;
use symbios::System;

pub fn derive_l_system(
    mut config: ResMut<LSystemConfig>,
    mut engine: ResMut<LSystemEngine>,
    mut status: ResMut<DerivationStatus>,
    mut analysis: ResMut<LSystemAnalysis>,
) {
    if !config.recompile_requested {
        return;
    }
    config.recompile_requested = false;

    status.error = None;

    analysis.uses_implicit_step = false;
    analysis.uses_implicit_angle = false;
    analysis.uses_explicit_width = false;

    let sys = &mut engine.0;
    *sys = System::new();

    let source = config.source_code.clone();
    let lines: Vec<&str> = source.lines().collect();
    let mut axiom_set = false;

    let mut check_module = |symbol: &str, param_count: usize| {
        let step_syms = ["F", "f"];
        let turn_syms = ["+", "-", "&", "^", "/", "\\", "|"];

        if symbol == "!" {
            analysis.uses_explicit_width = true;
        }

        if param_count == 0 {
            if step_syms.contains(&symbol) {
                analysis.uses_implicit_step = true;
            } else if turn_syms.contains(&symbol) {
                analysis.uses_implicit_angle = true;
            }
        }
    };

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let line_num = i + 1;

        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        if trimmed.starts_with("#") {
            if let Err(e) = sys.add_directive(trimmed) {
                status.error = Some(format!("Line {}: {}", line_num, e));
                return;
            }
            continue;
        }

        if trimmed.starts_with("omega:") {
            let axiom_src = trimmed.trim_start_matches("omega:").trim();

            let mut remaining = axiom_src;
            while !remaining.is_empty() {
                if let Ok((rest, module)) = symbios::parser::parse_module(remaining) {
                    check_module(&module.symbol, module.params.len());
                    remaining = rest.trim();
                } else {
                    break;
                }
            }

            if let Err(e) = sys.set_axiom(axiom_src) {
                status.error = Some(format!("Line {}: Axiom error: {}", line_num, e));
                return;
            }
            axiom_set = true;
            continue;
        }

        match symbios::parser::parse_rule(trimmed) {
            Ok((_, rule_ast)) => {
                for succ in &rule_ast.successors {
                    check_module(&succ.symbol, succ.params.len());
                }

                if let Err(e) = sys.add_rule(trimmed) {
                    status.error = Some(format!("Line {}: Rule error: {}", line_num, e));
                    return;
                }
            }
            Err(e) => {
                status.error = Some(format!("Line {}: Parse error: {}", line_num, e));
                return;
            }
        }
    }

    if axiom_set {
        if let Err(e) = sys.derive(config.iterations) {
            status.error = Some(format!("Derivation error: {}", e));
        }
    } else {
        status.error = Some("No axiom defined".to_string());
    }
}
