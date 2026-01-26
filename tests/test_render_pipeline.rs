mod common;
use bevy::prelude::*;
use common::setup_headless_app;
use lsystem_explorer::core::config::{DirtyFlags, LSystemEngine};
use lsystem_explorer::visuals::turtle::{LSystemMeshTag, render_turtle};
use symbios::System;

#[test]
fn test_mesh_generation() {
    let mut app = setup_headless_app();

    // 1. Manually inject a derived state into the Engine
    // This skips the async derivation step to test the renderer in isolation
    let mut sys = System::new();
    sys.set_axiom("F(10)").unwrap();
    sys.derive(0).unwrap(); // State = [Module(F, [10])]

    app.world_mut().resource_mut::<LSystemEngine>().0 = sys;

    // 2. Set Dirty Flag to trigger renderer
    app.world_mut().resource_mut::<DirtyFlags>().geometry = true;

    // 3. Run Render System
    app.add_systems(Update, render_turtle);
    app.update();

    // 4. Verify Entity Spawn
    let mut query = app
        .world_mut()
        .query_filtered::<&Mesh3d, With<LSystemMeshTag>>();
    let mesh_handle = query.single(app.world());

    assert!(
        mesh_handle.is_ok(),
        "Should spawn exactly one mesh entity for a single material"
    );

    // 5. Verify Mesh Data
    let mesh_assets = app.world().resource::<Assets<Mesh>>();
    let mesh = mesh_assets
        .get(&mesh_handle.unwrap().0)
        .expect("Mesh asset should exist");

    // A single cylinder segment (resolution 8) + caps should have vertices
    let count = mesh.count_vertices();
    assert!(count > 0, "Generated mesh should have vertices");
}
