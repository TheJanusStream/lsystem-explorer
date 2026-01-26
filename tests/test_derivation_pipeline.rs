mod common;
use bevy::prelude::*;
use common::setup_headless_app;
use lsystem_explorer::core::config::{DerivationStatus, DirtyFlags, LSystemConfig, LSystemEngine};
use lsystem_explorer::logic::derivation::{poll_derivation, start_derivation};

#[test]
fn test_async_derivation_flow() {
    let mut app = setup_headless_app();

    // 1. Configure the app with a simple grammar
    let mut config = app.world_mut().resource_mut::<LSystemConfig>();
    config.source_code = "omega: F\np1: F -> F+F".to_string();
    config.iterations = 2;
    config.recompile_requested = true;

    // Add the derivation systems
    app.add_systems(Update, (start_derivation, poll_derivation).chain());

    // 2. First Update: Should Trigger Start
    app.update();

    // Verify task started
    let status = app.world().resource::<DerivationStatus>();
    assert!(
        status.generating,
        "Derivation should be generating after first update"
    );
    assert!(status.error.is_none(), "Should be no error initially");

    // 3. Subsequent Updates: Wait for Async Task
    // We loop briefly to allow the thread pool to finish the simple derivation
    let mut done = false;
    for _ in 0..100 {
        app.update();
        let status = app.world().resource::<DerivationStatus>();
        if !status.generating {
            done = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    assert!(done, "Derivation timed out");

    // 4. Verify Results
    let engine = app.world().resource::<LSystemEngine>();
    let dirty = app.world().resource::<DirtyFlags>();

    // F -> F+F (iter 1) -> F+F+F+F (iter 2) ... roughly
    // Just checking it's not empty
    assert!(
        !engine.0.state.is_empty(),
        "Engine state should be populated"
    );
    assert!(
        dirty.geometry,
        "Geometry dirty flag should be set after derivation"
    );
}
