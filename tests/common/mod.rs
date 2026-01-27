// tests/common/mod.rs
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use lsystem_explorer::core::config::*;
use lsystem_explorer::visuals::turtle::TurtleRenderState;

/// Creates a minimal headless Bevy app with necessary resources and plugins
pub fn setup_headless_app() -> App {
    let mut app = App::new();

    // Use MinimalPlugins for core loop/schedule/tasks (Headless)
    app.add_plugins(MinimalPlugins);

    // Add Asset infrastructure
    app.add_plugins(AssetPlugin::default());

    // Register Assets manually to avoid needing the full RenderPlugin stack (WGPU/Window)
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    app.init_asset::<Image>();

    // Initialize L-System Explorer Resources
    app.init_resource::<LSystemConfig>()
        .init_resource::<LSystemEngine>()
        .init_resource::<DerivationStatus>()
        .init_resource::<DerivationDebounce>()
        .init_resource::<DerivationTask>()
        .init_resource::<DirtyFlags>()
        .init_resource::<LSystemAnalysis>()
        .init_resource::<PropConfig>()
        .init_resource::<MaterialSettingsMap>()
        .init_resource::<ExportConfig>()
        .init_resource::<TurtleRenderState>();

    // Mock the asset setup usually done in main.rs
    // run_system_once takes the function directly
    app.world_mut()
        .run_system_once(bevy_symbios::materials::setup_material_assets)
        .expect("Failed to run setup material assets");
    app.world_mut()
        .run_system_once(lsystem_explorer::visuals::assets::setup_prop_assets)
        .expect("Failed to run setup prop assets");

    app
}
