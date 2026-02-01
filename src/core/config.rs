use crate::core::presets::PRESETS;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use symbios::System;

// Re-export material and export types from bevy_symbios for convenience.
pub use bevy_symbios::export::ExportFormat;
pub use bevy_symbios::materials::{MaterialSettings, MaterialSettingsMap, TextureType};

/// Geometry dirty flag for split reactivity.
/// Geometry dirty = requires derivation + remesh.
#[derive(Resource, Default)]
pub struct DirtyFlags {
    pub geometry: bool,
}

/// Available prop mesh types for prop IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PropMeshType {
    #[default]
    Leaf,
    Sphere,
    Cone,
    Cylinder,
    Cube,
}

impl PropMeshType {
    pub const ALL: &'static [PropMeshType] = &[
        PropMeshType::Leaf,
        PropMeshType::Sphere,
        PropMeshType::Cone,
        PropMeshType::Cylinder,
        PropMeshType::Cube,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            PropMeshType::Leaf => "Leaf",
            PropMeshType::Sphere => "Sphere",
            PropMeshType::Cone => "Cone",
            PropMeshType::Cylinder => "Cylinder",
            PropMeshType::Cube => "Cube",
        }
    }
}

/// Configuration for prop meshes mapped to prop IDs
#[derive(Resource)]
pub struct PropConfig {
    pub prop_meshes: HashMap<u16, PropMeshType>,
    pub prop_scale: f32,
}

impl Default for PropConfig {
    fn default() -> Self {
        let mut prop_meshes = HashMap::new();
        prop_meshes.insert(0, PropMeshType::Leaf);
        prop_meshes.insert(1, PropMeshType::Sphere);
        Self {
            prop_meshes,
            prop_scale: 1.0,
        }
    }
}

#[derive(Resource)]
pub struct LSystemConfig {
    pub source_code: String,
    /// Finalization/decomposition code for two-pass derivation.
    /// Applied after the main growth phase completes.
    pub finalization_code: String,
    pub iterations: usize,
    pub default_angle: f32,
    pub step_size: f32,
    pub default_width: f32,

    pub tropism: Option<Vec3>,
    pub elasticity: f32,

    /// Random seed for stochastic L-systems.
    pub seed: u64,

    /// Resolution of procedural tube meshes (vertices per ring).
    pub mesh_resolution: u32,

    pub recompile_requested: bool,
    pub auto_update: bool,
}

impl Default for LSystemConfig {
    fn default() -> Self {
        let default_preset = &PRESETS[3];
        let (growth, finalization) = split_source_code(default_preset.code);

        Self {
            source_code: growth,
            finalization_code: finalization,
            iterations: 5,
            default_angle: 45.0,
            step_size: 0.5,
            default_width: 0.1,

            tropism: None,
            elasticity: 0.0,

            seed: 42,

            mesh_resolution: 8,

            recompile_requested: true,
            auto_update: true,
        }
    }
}

/// Separator used to split growth and finalization code in preset strings.
pub const DECOMPOSITION_SEPARATOR: &str = "/// DECOMPOSITION ///";

/// Splits a combined source code string into (growth, finalization) parts.
/// If the separator is not found, returns the full string as growth with empty finalization.
pub fn split_source_code(full: &str) -> (String, String) {
    if let Some(idx) = full.find(DECOMPOSITION_SEPARATOR) {
        let growth = full[..idx].trim_end().to_string();
        let finalization = full[idx + DECOMPOSITION_SEPARATOR.len()..]
            .trim_start()
            .to_string();
        (growth, finalization)
    } else {
        (full.to_string(), String::new())
    }
}

/// Joins growth and finalization code into a single string with separator.
/// If finalization is empty, returns just the growth code.
pub fn join_source_code(growth: &str, finalization: &str) -> String {
    if finalization.trim().is_empty() {
        growth.to_string()
    } else {
        format!(
            "{}\n{}\n{}",
            growth.trim_end(),
            DECOMPOSITION_SEPARATOR,
            finalization.trim_start()
        )
    }
}

#[derive(Resource, Default, Clone)]
pub struct LSystemAnalysis {
    pub uses_implicit_step: bool,
    pub uses_implicit_angle: bool,
    pub uses_explicit_width: bool,
    /// Maximum material ID referenced in the source code.
    pub max_material_id: u8,
}

/// The persistent Symbios engine
#[derive(Resource)]
pub struct LSystemEngine(pub System);

impl Default for LSystemEngine {
    fn default() -> Self {
        Self(System::new())
    }
}

/// Tracks the result of the last compilation attempt
#[derive(Resource, Default)]
pub struct DerivationStatus {
    /// None = Success, Some(String) = Error Message
    pub error: Option<String>,
    /// True while an async derivation task is running
    pub generating: bool,
}

/// Debounce timer for auto-updates
#[derive(Resource)]
pub struct DerivationDebounce {
    pub timer: Timer,
    pub pending: bool,
}

impl Default for DerivationDebounce {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Once),
            pending: false,
        }
    }
}

/// Result from an async derivation task
pub struct DerivationResult {
    pub system: System,
    pub analysis: LSystemAnalysis,
    pub derivation_time_ms: f32,
}

/// Type alias for the shared async derivation result container.
pub type SharedDerivationResult = Arc<Mutex<Option<Result<DerivationResult, String>>>>;

/// Shared cancellation flag for async derivation tasks.
pub type CancellationFlag = Arc<AtomicBool>;

/// Holds a reference to a pending async derivation result.
/// The background task writes into the shared Arc<Mutex<Option<...>>> when complete.
#[derive(Resource, Default)]
pub struct DerivationTask {
    pub shared: Option<SharedDerivationResult>,
    /// Cancellation flag for the current task. Set to false to cancel.
    pub cancel_flag: Option<CancellationFlag>,
}

/// Configuration for batch export
#[derive(Resource)]
pub struct ExportConfig {
    pub base_filename: String,
    pub variation_count: usize,
    pub format: ExportFormat,
    pub export_requested: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            base_filename: "LSystem_Variant".to_string(),
            variation_count: 5,
            format: ExportFormat::Obj,
            export_requested: false,
        }
    }
}
