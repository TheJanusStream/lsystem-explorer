use crate::core::presets::PRESETS;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use symbios::System;

/// Per-material settings for UI editing
#[derive(Clone)]
pub struct MaterialSettings {
    pub base_color: [f32; 3],
    pub emission_color: [f32; 3],
    pub emission_strength: f32,
    pub roughness: f32,
}

impl Default for MaterialSettings {
    fn default() -> Self {
        Self {
            base_color: [1.0, 1.0, 1.0],
            emission_color: [0.0, 0.0, 0.0],
            emission_strength: 0.0,
            roughness: 0.5,
        }
    }
}

/// Resource holding editable settings for each material ID
#[derive(Resource)]
pub struct MaterialSettingsMap {
    pub settings: HashMap<u8, MaterialSettings>,
}

impl Default for MaterialSettingsMap {
    fn default() -> Self {
        let mut settings = HashMap::new();

        // Material 0: Primary/Trunk - White base, metallic look
        settings.insert(
            0,
            MaterialSettings {
                base_color: [0.2, 0.8, 0.2],
                emission_color: [0.5, 1.0, 0.5],
                emission_strength: 0.0,
                roughness: 0.2,
            },
        );

        // Material 1: Energy/Leaves - Emissive cyan glow
        settings.insert(
            1,
            MaterialSettings {
                base_color: [1.0, 1.0, 1.0],
                emission_color: [0.0, 1.0, 1.0],
                emission_strength: 2.0,
                roughness: 0.1,
            },
        );

        // Material 2: Matte/Structure - Gray, high roughness
        settings.insert(
            2,
            MaterialSettings {
                base_color: [0.5, 0.5, 0.5],
                emission_color: [0.0, 0.0, 0.0],
                emission_strength: 0.0,
                roughness: 0.9,
            },
        );

        Self { settings }
    }
}

/// Available prop mesh types for surface IDs
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

/// Configuration for prop meshes mapped to surface IDs
#[derive(Resource)]
pub struct PropConfig {
    pub surface_meshes: HashMap<u16, PropMeshType>,
    pub prop_scale: f32,
}

impl Default for PropConfig {
    fn default() -> Self {
        let mut surface_meshes = HashMap::new();
        surface_meshes.insert(0, PropMeshType::Leaf);
        surface_meshes.insert(1, PropMeshType::Sphere);
        Self {
            surface_meshes,
            prop_scale: 1.0,
        }
    }
}

#[derive(Resource)]
pub struct LSystemConfig {
    pub source_code: String,
    pub iterations: usize,
    pub default_angle: f32,
    pub step_size: f32,
    pub default_width: f32,

    pub tropism: Option<Vec3>,
    pub elasticity: f32,

    pub recompile_requested: bool,
    pub auto_update: bool,
}

impl Default for LSystemConfig {
    fn default() -> Self {
        let default_preset = &PRESETS[3];

        Self {
            source_code: default_preset.code.to_string(),
            iterations: 5,
            default_angle: 45.0,
            step_size: 0.5,
            default_width: 0.1,

            tropism: None,
            elasticity: 0.0,

            recompile_requested: true,
            auto_update: true,
        }
    }
}

#[derive(Resource, Default)]
pub struct LSystemAnalysis {
    pub uses_implicit_step: bool,
    pub uses_implicit_angle: bool,
    pub uses_explicit_width: bool,
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
            // 0.5s delay to prevent freezing while typing
            timer: Timer::from_seconds(0.5, TimerMode::Once),
            pending: false,
        }
    }
}

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportFormat {
    #[default]
    Obj,
}

impl ExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Obj => "obj",
        }
    }
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
