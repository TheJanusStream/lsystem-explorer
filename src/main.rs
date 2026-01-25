use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use bevy_panorbit_camera::PanOrbitCameraPlugin;

mod core;
mod logic;
mod ui;
mod visuals;

use core::config::{
    DerivationDebounce, DerivationStatus, ExportConfig, LSystemConfig, LSystemEngine, PropConfig,
};

use crate::{core::config::LSystemAnalysis, visuals::turtle::TurtleRenderState};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Symbios L-System Explorer".into(),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            }),
            EguiPlugin::default(),
            PanOrbitCameraPlugin,
        ))
        // Core State
        .init_resource::<LSystemConfig>()
        .init_resource::<LSystemEngine>()
        .init_resource::<DerivationStatus>()
        .init_resource::<DerivationDebounce>()
        .init_resource::<LSystemAnalysis>()
        .init_resource::<PropConfig>()
        .init_resource::<ExportConfig>()
        .init_resource::<TurtleRenderState>()
        // Startup
        .add_systems(
            Startup,
            (
                visuals::scene::setup_scene,
                visuals::assets::setup_turtle_assets,
            ),
        )
        // UI
        .add_systems(EguiPrimaryContextPass, ui::editor::ui_system)
        // Logic & Render Loop
        .add_systems(
            Update,
            (
                logic::derivation::derive_l_system,
                visuals::turtle::render_turtle,
                visuals::turtle::sync_material_properties,
                visuals::export::batch_export_system,
            )
                .chain(),
        )
        .run();
}
