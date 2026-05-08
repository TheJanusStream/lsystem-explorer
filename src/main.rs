use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use bevy_panorbit_camera::PanOrbitCameraPlugin;

use bevy_symbios::MeshCache;
use bevy_symbios::loader::LSystemAssetPlugin;
use lsystem_explorer::core::config::{
    DerivationDebounce, DerivationStatus, DerivationTask, DirtyFlags, ExportConfig,
    LSystemAnalysis, LSystemConfig, LSystemEngine, MaterialSettingsMap, PropConfig,
};
use lsystem_explorer::logic::hot_reload::WatchedAssets;
use lsystem_explorer::ui::nursery::{NurseryState, PopulationMeshCache};
use lsystem_explorer::visuals::export::ExportStatus;
use lsystem_explorer::visuals::nursery_render::{
    NurseryDerivationTask, NurseryFoliageTextureTasks,
};
use lsystem_explorer::visuals::turtle::{PropMaterialCache, TurtleRenderState};
use lsystem_explorer::{core, logic, ui, visuals};

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
            LSystemAssetPlugin,
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
        .init_resource::<ExportStatus>()
        .init_resource::<TurtleRenderState>()
        .init_resource::<PropMaterialCache>()
        .init_resource::<MeshCache>()
        .init_resource::<NurseryState>()
        .init_resource::<PopulationMeshCache>()
        .init_resource::<NurseryDerivationTask>()
        .init_resource::<NurseryFoliageTextureTasks>()
        .init_resource::<WatchedAssets>()
        // Startup
        .add_systems(
            Startup,
            (
                visuals::scene::setup_scene,
                bevy_symbios::materials::setup_material_assets,
                visuals::assets::setup_prop_assets,
                core::config::apply_startup_preset,
                visuals::nursery_render::setup_nursery_materials,
                logic::hot_reload::load_cli_assets,
            )
                .chain(),
        )
        // Observer: re-syncs MaterialPalette whenever code triggers MaterialSettingsChanged.
        .add_observer(bevy_symbios::materials::on_material_settings_changed)
        // UI
        .add_systems(EguiPrimaryContextPass, ui::editor::ui_system)
        // Logic & Render Loop
        .add_systems(
            Update,
            (
                logic::hot_reload::apply_lsys_reload,
                logic::hot_reload::apply_palette_reload,
                logic::derivation::start_derivation,
                logic::derivation::poll_derivation,
                logic::derivation::ensure_material_palette_size,
                bevy_symbios::materials::apply_foliage_textures,
                visuals::turtle::render_turtle,
                visuals::turtle::toggle_editor_visibility,
                visuals::nursery_render::rebuild_nursery_cache,
                visuals::nursery_render::poll_nursery_derivation,
                visuals::nursery_render::render_nursery_population,
                visuals::nursery_render::apply_nursery_foliage_textures,
                visuals::nursery_render::sync_nursery_selection_visuals,
                visuals::nursery_render::handle_panel_clicks,
                visuals::turtle::sync_prop_materials,
                visuals::export::batch_export_system,
                visuals::export::poll_export_status,
            )
                .chain(),
        )
        .run();
}
