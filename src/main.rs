use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use bevy_panorbit_camera::PanOrbitCameraPlugin;

use lsystem_explorer::core::config::{
    DerivationDebounce, DerivationStatus, DerivationTask, DirtyFlags, ExportConfig,
    LSystemAnalysis, LSystemConfig, LSystemEngine, MaterialSettingsMap, PropConfig,
};
use lsystem_explorer::visuals::turtle::{PropMaterialCache, TurtleRenderState};
use lsystem_explorer::{logic, ui, visuals};

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
        .init_resource::<DerivationTask>()
        .init_resource::<DirtyFlags>()
        .init_resource::<LSystemAnalysis>()
        .init_resource::<PropConfig>()
        .init_resource::<MaterialSettingsMap>()
        .init_resource::<ExportConfig>()
        .init_resource::<TurtleRenderState>()
        .init_resource::<PropMaterialCache>()
        // Startup
        .add_systems(
            Startup,
            (
                visuals::scene::setup_scene,
                bevy_symbios::materials::setup_material_assets,
                visuals::assets::setup_prop_assets,
            ),
        )
        // UI
        .add_systems(EguiPrimaryContextPass, ui::editor::ui_system)
        // Logic & Render Loop
        .add_systems(
            Update,
            (
                logic::derivation::start_derivation,
                logic::derivation::poll_derivation,
                logic::derivation::ensure_material_palette_size,
                visuals::turtle::render_turtle,
                bevy_symbios::materials::sync_material_properties,
                visuals::turtle::sync_prop_materials,
                visuals::export::batch_export_system,
            )
                .chain(),
        )
        .run();
}
