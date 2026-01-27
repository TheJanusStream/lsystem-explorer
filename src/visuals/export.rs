use bevy::prelude::*;

use crate::core::config::{ExportConfig, ExportFormat, LSystemConfig, MaterialSettingsMap};

use bevy_symbios::LSystemMeshBuilder;
use bevy_symbios::export::{mesh_to_obj, meshes_to_glb};
use symbios::System;
use symbios_turtle_3d::{TurtleConfig, TurtleInterpreter};

// ---------------------------------------------------------------------------
// Platform-specific file I/O
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
pub fn save_file(filename: &str, content: &str) {
    use wasm_bindgen::JsCast;

    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    let blob_parts = js_sys::Array::new();
    blob_parts.push(&wasm_bindgen::JsValue::from_str(content));

    let mut options = web_sys::BlobPropertyBag::new();
    options.type_("text/plain");

    let blob = web_sys::Blob::new_with_str_sequence_and_options(&blob_parts, &options)
        .expect("failed to create blob");

    let url = web_sys::Url::create_object_url_with_blob(&blob).expect("failed to create URL");

    let anchor: web_sys::HtmlAnchorElement = document
        .create_element("a")
        .expect("failed to create anchor")
        .dyn_into()
        .expect("not an anchor");

    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();

    web_sys::Url::revoke_object_url(&url).expect("failed to revoke URL");
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_file(filename: &str, content: &str) {
    use std::fs;
    use std::path::Path;

    let export_dir = Path::new("exports");
    if !export_dir.exists() {
        fs::create_dir_all(export_dir).expect("failed to create exports directory");
    }

    let path = export_dir.join(filename);
    fs::write(&path, content).expect("failed to write file");
    info!("Exported: {}", path.display());
}

#[cfg(target_arch = "wasm32")]
pub fn save_file_binary(filename: &str, content: &[u8]) {
    use wasm_bindgen::JsCast;

    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    let uint8arr = js_sys::Uint8Array::new_with_length(content.len() as u32);
    uint8arr.copy_from(content);

    let parts = js_sys::Array::new();
    parts.push(&uint8arr);

    let mut options = web_sys::BlobPropertyBag::new();
    options.type_("application/octet-stream");

    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options)
        .expect("failed to create blob");

    let url = web_sys::Url::create_object_url_with_blob(&blob).expect("failed to create URL");

    let anchor: web_sys::HtmlAnchorElement = document
        .create_element("a")
        .expect("failed to create anchor")
        .dyn_into()
        .expect("not an anchor");

    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();

    web_sys::Url::revoke_object_url(&url).expect("failed to revoke URL");
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_file_binary(filename: &str, content: &[u8]) {
    use std::fs;
    use std::path::Path;

    let export_dir = Path::new("exports");
    if !export_dir.exists() {
        fs::create_dir_all(export_dir).expect("failed to create exports directory");
    }

    let path = export_dir.join(filename);
    fs::write(&path, content).expect("failed to write file");
    info!("Exported: {}", path.display());
}

// ---------------------------------------------------------------------------
// Batch Export System
// ---------------------------------------------------------------------------

/// System that handles batch export when requested
pub fn batch_export_system(
    mut export_config: ResMut<ExportConfig>,
    lsystem_config: Res<LSystemConfig>,
    material_settings: Res<MaterialSettingsMap>,
) {
    if !export_config.export_requested {
        return;
    }
    export_config.export_requested = false;

    info!(
        "Starting batch export: {} variations as {}",
        export_config.variation_count,
        export_config.format.name()
    );

    for variant_idx in 0..export_config.variation_count {
        let mut sys = System::new();

        let source = &lsystem_config.source_code;
        let mut axiom_set = false;

        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            if trimmed.starts_with('#') {
                let _ = sys.add_directive(trimmed);
                continue;
            }

            if trimmed.starts_with("omega:") {
                let axiom_src = trimmed.trim_start_matches("omega:").trim();
                if sys.set_axiom(axiom_src).is_ok() {
                    axiom_set = true;
                }
                continue;
            }

            let _ = sys.add_rule(trimmed);
        }

        if !axiom_set {
            warn!("Export variant {}: No axiom found", variant_idx);
            continue;
        }

        if sys.derive(lsystem_config.iterations).is_err() {
            warn!("Export variant {}: Derivation failed", variant_idx);
            continue;
        }

        // Configure turtle interpreter
        let default_step = sys
            .constants
            .get("step")
            .map(|&s| s as f32)
            .unwrap_or(lsystem_config.step_size);

        let default_angle = sys
            .constants
            .get("angle")
            .map(|&a| a as f32)
            .unwrap_or(lsystem_config.default_angle)
            .to_radians();

        let initial_width = sys
            .constants
            .get("width")
            .map(|&w| w as f32)
            .unwrap_or(lsystem_config.default_width);

        let turtle_config = TurtleConfig {
            default_step,
            default_angle,
            initial_width,
            tropism: lsystem_config.tropism,
            elasticity: lsystem_config.elasticity,
        };

        let mut interpreter = TurtleInterpreter::new(turtle_config);
        interpreter.populate_standard_symbols(&sys.interner);

        let skeleton = interpreter.build_skeleton(&sys.state);
        let builder = LSystemMeshBuilder::new().with_resolution(8);
        let mesh_buckets = builder.build(&skeleton);

        let filename = format!(
            "{}_{:02}.{}",
            export_config.base_filename,
            variant_idx + 1,
            export_config.format.extension()
        );

        match export_config.format {
            ExportFormat::Obj => {
                let mut combined_obj = String::new();
                combined_obj.push_str("# Exported from L-System Explorer\n");
                combined_obj.push_str(&format!(
                    "# Variant {} of {}\n\n",
                    variant_idx + 1,
                    export_config.variation_count
                ));

                let mut vertex_offset = 0u32;
                for (material_id, mesh) in &mesh_buckets {
                    let object_name = format!(
                        "{}_{:02}_mat{}",
                        export_config.base_filename,
                        variant_idx + 1,
                        material_id
                    );
                    combined_obj.push_str(&mesh_to_obj(mesh, &object_name, vertex_offset));
                    vertex_offset += mesh.count_vertices() as u32;
                }

                save_file(&filename, &combined_obj);
            }
            ExportFormat::Glb => {
                let glb_data = meshes_to_glb(&mesh_buckets, &material_settings.settings);
                save_file_binary(&filename, &glb_data);
            }
        }
    }

    info!("Batch export complete!");
}
