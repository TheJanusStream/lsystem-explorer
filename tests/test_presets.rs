use lsystem_explorer::core::presets::PRESETS;
use symbios::System;

#[test]
fn test_presets_validity() {
    for preset in PRESETS {
        println!("Testing preset: {}", preset.name);

        let mut sys = System::new();
        let source = preset.code;

        // 1. Simulate Parsing
        let mut axiom_set = false;
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            if trimmed.starts_with('#') {
                assert!(
                    sys.add_directive(trimmed).is_ok(),
                    "Failed to add directive in {}",
                    preset.name
                );
                continue;
            }

            if trimmed.starts_with("omega:") {
                let axiom_src = trimmed.trim_start_matches("omega:").trim();
                assert!(
                    sys.set_axiom(axiom_src).is_ok(),
                    "Failed to set axiom in {}",
                    preset.name
                );
                axiom_set = true;
                continue;
            }

            assert!(
                sys.add_rule(trimmed).is_ok(),
                "Failed to add rule in {}",
                preset.name
            );
        }

        assert!(axiom_set, "Preset {} has no axiom", preset.name);

        // 2. Simulate Derivation
        // Use a small iteration count to verify logic without burning CPU
        let result = sys.derive(1);
        assert!(
            result.is_ok(),
            "Derivation failed for {}: {:?}",
            preset.name,
            result.err()
        );

        // 3. Verify Output
        assert!(
            !sys.state.is_empty(),
            "Preset {} produced empty state",
            preset.name
        );
    }
}
