use crate::core::presets::PRESETS;
use bevy::prelude::*;
use symbios::System;

#[derive(Resource)]
pub struct LSystemConfig {
    pub source_code: String,
    pub iterations: usize,
    pub default_angle: f32,
    pub step_size: f32,
    // New fields for Tropism/Gravity
    pub tropism: Option<Vec3>,
    pub elasticity: f32,
    pub recompile_requested: bool,
    pub auto_update: bool,
}

impl Default for LSystemConfig {
    fn default() -> Self {
        // Load the "Monopodial Tree" as the application default
        let default_preset = &PRESETS[0];

        Self {
            source_code: default_preset.code.to_string(),
            iterations: 5,
            default_angle: 45.0,
            step_size: 0.5,
            // Default to no tropism
            tropism: None,
            elasticity: 0.0,
            recompile_requested: true,
            auto_update: true,
        }
    }
}

// ... (Rest of file remains unchanged: LSystemEngine, DerivationStatus, DerivationDebounce) ...
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
