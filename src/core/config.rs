use crate::core::presets::PRESETS;
use bevy::prelude::*;
use symbios::System;

#[derive(Resource)]
pub struct LSystemConfig {
    pub source_code: String,
    pub iterations: usize,
    pub default_angle: f32,
    pub step_size: f32,
    pub default_width: f32,

    pub tropism: Option<Vec3>,
    pub elasticity: f32,

    pub material_color: [f32; 3],
    pub emission_color: [f32; 3],
    pub emission_strength: f32,

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

            material_color: [0.2, 0.8, 0.2],
            emission_color: [0.5, 1.0, 0.5],
            emission_strength: 0.0,

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
