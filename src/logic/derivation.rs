use crate::core::config::{
    CancellationFlag, DerivationResult, DerivationStatus, DerivationTask, DirtyFlags,
    LSystemAnalysis, LSystemConfig, LSystemEngine, MaterialSettingsMap,
};
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use symbios::System;

/// Spawns an async derivation task when a recompile is requested.
/// If a previous task is still running, it is signaled to cancel.
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

    // Signal any in-progress task to cancel
    if let Some(old_flag) = &task.cancel_flag {
        old_flag.store(false, Ordering::Relaxed);
    }

    // Create new shared result and cancellation flag
    let shared: Arc<Mutex<Option<Result<DerivationResult, String>>>> = Arc::new(Mutex::new(None));
    let cancel_flag: CancellationFlag = Arc::new(std::sync::atomic::AtomicBool::new(true));

    task.shared = Some(shared.clone());
    task.cancel_flag = Some(cancel_flag.clone());

    let source = config.source_code.clone();
    let iterations = config.iterations;
    let seed = config.seed;

    let pool = AsyncComputeTaskPool::get();
    pool.spawn(async move {
        let result = perform_derivation(&source, iterations, seed, &cancel_flag);
        // Only store result if not cancelled
        if cancel_flag.load(Ordering::Relaxed)
            && let Ok(mut guard) = shared.lock()
        {
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

/// Ensures the MaterialSettingsMap has slots for all material IDs up to max_material_id.
/// Adds default entries for any missing slots.
pub fn ensure_material_palette_size(
    analysis: Res<LSystemAnalysis>,
    mut material_settings: ResMut<MaterialSettingsMap>,
) {
    if !analysis.is_changed() {
        return;
    }

    for id in 0..=analysis.max_material_id {
        material_settings.settings.entry(id).or_default();
    }
}

/// Performs L-system parsing and derivation. Runs on a background thread.
/// Checks the cancellation flag periodically and aborts early if cancelled.
fn perform_derivation(
    source: &str,
    iterations: usize,
    seed: u64,
    cancel_flag: &CancellationFlag,
) -> Result<DerivationResult, String> {
    let start_time = std::time::Instant::now();
    let mut sys = System::new();
    sys.set_seed(seed);
    let mut analysis = LSystemAnalysis::default();
    let mut axiom_set = false;

    // Helper to check if we should abort
    let is_cancelled = || !cancel_flag.load(Ordering::Relaxed);

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

    // Scan source for material ID usage: ,(N) pattern
    analysis.max_material_id = scan_max_material_id(source);

    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        // Check cancellation periodically during parsing
        if is_cancelled() {
            return Err("Cancelled".to_string());
        }

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
        // Check cancellation before expensive derivation
        if is_cancelled() {
            return Err("Cancelled".to_string());
        }

        // Derive one iteration at a time to allow cancellation checks
        for _ in 0..iterations {
            if is_cancelled() {
                return Err("Cancelled".to_string());
            }
            sys.derive(1)
                .map_err(|e| format!("Derivation error: {}", e))?;
        }
    } else {
        return Err("No axiom defined".to_string());
    }

    Ok(DerivationResult {
        system: sys,
        analysis,
        derivation_time_ms: start_time.elapsed().as_secs_f32() * 1000.0,
    })
}

/// Scans source code for material ID usage patterns: `,(N)` where N is a number.
/// Returns the maximum material ID found, or 0 if none.
fn scan_max_material_id(source: &str) -> u8 {
    let mut max_id: u8 = 0;
    let bytes = source.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Look for `,` followed by `(`
        if bytes[i] == b',' && i + 1 < bytes.len() && bytes[i + 1] == b'(' {
            i += 2; // Skip `,(`

            // Parse the number
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }

            if let Some(num) = std::str::from_utf8(&bytes[start..i])
                .ok()
                .and_then(|s| s.parse::<u8>().ok())
            {
                max_id = max_id.max(num);
            }
        } else {
            i += 1;
        }
    }

    max_id
}
