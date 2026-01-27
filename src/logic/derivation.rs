use crate::core::config::{
    DerivationResult, DerivationStatus, DerivationTask, DirtyFlags, LSystemAnalysis, LSystemConfig,
    LSystemEngine,
};
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use std::sync::{Arc, Mutex};
use symbios::System;

/// Spawns an async derivation task when a recompile is requested.
/// If a previous task is still running, it is abandoned (its result will be ignored).
pub fn start_derivation(
    mut config: ResMut<LSystemConfig>,
    mut task: ResMut<DerivationTask>,
    mut status: ResMut<DerivationStatus>,
) {
    if !config.recompile_requested {
        return;
    }
    config.recompile_requested = false;
    status.error = None;
    status.generating = true;

    // Abandon any in-progress task by dropping the old shared reference
    let shared: Arc<Mutex<Option<Result<DerivationResult, String>>>> = Arc::new(Mutex::new(None));
    task.shared = Some(shared.clone());

    let source = config.source_code.clone();
    let iterations = config.iterations;

    let pool = AsyncComputeTaskPool::get();
    pool.spawn(async move {
        let result = perform_derivation(&source, iterations);
        if let Ok(mut guard) = shared.lock() {
            *guard = Some(result);
        }
    })
    .detach();
}

/// Polls the async derivation task for completion.
/// When done, updates the engine state and sets the geometry dirty flag.
pub fn poll_derivation(
    mut engine: ResMut<LSystemEngine>,
    mut task: ResMut<DerivationTask>,
    mut status: ResMut<DerivationStatus>,
    mut analysis: ResMut<LSystemAnalysis>,
    mut dirty: ResMut<DirtyFlags>,
    mut render_state: ResMut<crate::visuals::turtle::TurtleRenderState>,
) {
    let Some(shared) = &task.shared else {
        return;
    };
    let Ok(mut guard) = shared.lock() else {
        return;
    };
    let Some(result) = guard.take() else {
        return;
    };
    drop(guard);
    task.shared = None;
    status.generating = false;

    match result {
        Ok(derivation) => {
            engine.0 = derivation.system;
            *analysis = derivation.analysis;
            render_state.derivation_time_ms = derivation.derivation_time_ms;
            dirty.geometry = true;
        }
        Err(err) => {
            status.error = Some(err);
        }
    }
}

/// Performs L-system parsing and derivation. Runs on a background thread.
fn perform_derivation(source: &str, iterations: usize) -> Result<DerivationResult, String> {
    let start_time = std::time::Instant::now();
    let mut sys = System::new();
    let mut analysis = LSystemAnalysis::default();
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

    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let line_num = i + 1;

        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        if trimmed.starts_with("#") {
            if let Err(e) = sys.add_directive(trimmed) {
                return Err(format!("Line {}: {}", line_num, e));
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
                return Err(format!("Line {}: Axiom error: {}", line_num, e));
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
                    return Err(format!("Line {}: Rule error: {}", line_num, e));
                }
            }
            Err(e) => {
                return Err(format!("Line {}: Parse error: {}", line_num, e));
            }
        }
    }

    if axiom_set {
        sys.derive(iterations)
            .map_err(|e| format!("Derivation error: {}", e))?;
    } else {
        return Err("No axiom defined".to_string());
    }

    Ok(DerivationResult {
        system: sys,
        analysis,
        derivation_time_ms: start_time.elapsed().as_secs_f32() * 1000.0,
    })
}
