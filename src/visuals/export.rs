use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::*;

/// Convert a Bevy Mesh to OBJ format string with vertex index offset for combining meshes
fn mesh_to_obj_with_offset(mesh: &Mesh, object_name: &str, vertex_offset: u32) -> String {
    let mut obj = String::new();
    obj.push_str(&format!("o {}\n", object_name));

    // Extract positions
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|attr| match attr {
            VertexAttributeValues::Float32x3(v) => Some(v),
            _ => None,
        });

    // Extract normals
    let normals = mesh
        .attribute(Mesh::ATTRIBUTE_NORMAL)
        .and_then(|attr| match attr {
            VertexAttributeValues::Float32x3(v) => Some(v),
            _ => None,
        });

    // Write vertices
    if let Some(positions) = positions {
        for pos in positions {
            obj.push_str(&format!("v {} {} {}\n", pos[0], pos[1], pos[2]));
        }
    }

    // Write normals
    if let Some(normals) = normals {
        for norm in normals {
            obj.push_str(&format!("vn {} {} {}\n", norm[0], norm[1], norm[2]));
        }
    }

    // Write faces from indices (with offset applied)
    if let Some(indices) = mesh.indices() {
        let has_normals = normals.is_some();
        match indices {
            Indices::U16(idx) => {
                for tri in idx.chunks(3) {
                    if tri.len() == 3 {
                        let (a, b, c) = (
                            tri[0] as u32 + 1 + vertex_offset,
                            tri[1] as u32 + 1 + vertex_offset,
                            tri[2] as u32 + 1 + vertex_offset,
                        );
                        if has_normals {
                            obj.push_str(&format!("f {}//{} {}//{} {}//{}\n", a, a, b, b, c, c));
                        } else {
                            obj.push_str(&format!("f {} {} {}\n", a, b, c));
                        }
                    }
                }
            }
            Indices::U32(idx) => {
                for tri in idx.chunks(3) {
                    if tri.len() == 3 {
                        let (a, b, c) = (
                            tri[0] + 1 + vertex_offset,
                            tri[1] + 1 + vertex_offset,
                            tri[2] + 1 + vertex_offset,
                        );
                        if has_normals {
                            obj.push_str(&format!("f {}//{} {}//{} {}//{}\n", a, a, b, b, c, c));
                        } else {
                            obj.push_str(&format!("f {} {} {}\n", a, b, c));
                        }
                    }
                }
            }
        }
    }

    obj
}

/// Convert a Bevy Mesh to OBJ format string
#[allow(dead_code)]
pub fn mesh_to_obj(mesh: &Mesh, object_name: &str) -> String {
    let mut obj = String::new();
    obj.push_str("# Exported from L-System Explorer\n");
    obj.push_str(&format!("o {}\n", object_name));

    // Extract positions
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|attr| match attr {
            VertexAttributeValues::Float32x3(v) => Some(v),
            _ => None,
        });

    // Extract normals
    let normals = mesh
        .attribute(Mesh::ATTRIBUTE_NORMAL)
        .and_then(|attr| match attr {
            VertexAttributeValues::Float32x3(v) => Some(v),
            _ => None,
        });

    // Write vertices
    if let Some(positions) = positions {
        for pos in positions {
            obj.push_str(&format!("v {} {} {}\n", pos[0], pos[1], pos[2]));
        }
    }

    // Write normals
    if let Some(normals) = normals {
        for norm in normals {
            obj.push_str(&format!("vn {} {} {}\n", norm[0], norm[1], norm[2]));
        }
    }

    // Write faces from indices
    if let Some(indices) = mesh.indices() {
        let has_normals = normals.is_some();
        match indices {
            Indices::U16(idx) => {
                for tri in idx.chunks(3) {
                    if tri.len() == 3 {
                        let (a, b, c) = (tri[0] as u32 + 1, tri[1] as u32 + 1, tri[2] as u32 + 1);
                        if has_normals {
                            obj.push_str(&format!("f {}//{} {}//{} {}//{}\n", a, a, b, b, c, c));
                        } else {
                            obj.push_str(&format!("f {} {} {}\n", a, b, c));
                        }
                    }
                }
            }
            Indices::U32(idx) => {
                for tri in idx.chunks(3) {
                    if tri.len() == 3 {
                        let (a, b, c) = (tri[0] + 1, tri[1] + 1, tri[2] + 1);
                        if has_normals {
                            obj.push_str(&format!("f {}//{} {}//{} {}//{}\n", a, a, b, b, c, c));
                        } else {
                            obj.push_str(&format!("f {} {} {}\n", a, b, c));
                        }
                    }
                }
            }
        }
    }

    obj
}

/// Platform-specific file download/save
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

use crate::core::config::{ExportConfig, LSystemConfig};
use bevy_symbios::LSystemMeshBuilder;
use symbios::System;
use symbios_turtle_3d::{TurtleConfig, TurtleInterpreter};

/// System that handles batch export when requested
pub fn batch_export_system(
    mut export_config: ResMut<ExportConfig>,
    lsystem_config: Res<LSystemConfig>,
) {
    if !export_config.export_requested {
        return;
    }
    export_config.export_requested = false;

    info!(
        "Starting batch export: {} variations",
        export_config.variation_count
    );

    for variant_idx in 0..export_config.variation_count {
        // Create a fresh system for each variant (stochastic rules will produce different results)
        let mut sys = System::new();

        // Parse the source code
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

        // Derive with current iterations
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

        // Build skeleton and mesh
        let skeleton = interpreter.build_skeleton(&sys.state);
        let builder = LSystemMeshBuilder::new().with_resolution(8);
        let mesh_buckets = builder.build(&skeleton);

        // Export meshes - combine all material buckets into one file
        let mut combined_obj = String::new();
        combined_obj.push_str("# Exported from L-System Explorer\n");
        combined_obj.push_str(&format!(
            "# Variant {} of {}\n\n",
            variant_idx + 1,
            export_config.variation_count
        ));

        let mut vertex_offset = 0u32;
        for (material_id, mesh) in mesh_buckets {
            let object_name = format!(
                "{}_{:02}_mat{}",
                export_config.base_filename,
                variant_idx + 1,
                material_id
            );
            combined_obj.push_str(&mesh_to_obj_with_offset(&mesh, &object_name, vertex_offset));

            // Update vertex offset for next mesh
            vertex_offset += mesh.count_vertices() as u32;
        }

        let filename = format!(
            "{}_{:02}.{}",
            export_config.base_filename,
            variant_idx + 1,
            export_config.format.extension()
        );
        save_file(&filename, &combined_obj);
    }

    info!("Batch export complete!");
}
