use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::core::config::{ExportConfig, ExportFormat, LSystemConfig, MaterialSettings};

use bevy_symbios::LSystemMeshBuilder;
use symbios::System;
use symbios_turtle_3d::{TurtleConfig, TurtleInterpreter};

/// Convert a Bevy Mesh to OBJ format string with vertex index offset for combining meshes
fn mesh_to_obj_with_offset(mesh: &Mesh, object_name: &str, vertex_offset: u32) -> String {
    let mut obj = String::new();
    obj.push_str(&format!("o {}\n", object_name));

    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|attr| match attr {
            VertexAttributeValues::Float32x3(v) => Some(v),
            _ => None,
        });

    let normals = mesh
        .attribute(Mesh::ATTRIBUTE_NORMAL)
        .and_then(|attr| match attr {
            VertexAttributeValues::Float32x3(v) => Some(v),
            _ => None,
        });

    if let Some(positions) = positions {
        for pos in positions {
            obj.push_str(&format!("v {} {} {}\n", pos[0], pos[1], pos[2]));
        }
    }

    if let Some(normals) = normals {
        for norm in normals {
            obj.push_str(&format!("vn {} {} {}\n", norm[0], norm[1], norm[2]));
        }
    }

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

// ---------------------------------------------------------------------------
// GLB (Binary glTF) Export
// ---------------------------------------------------------------------------

/// Build a GLB binary from mesh buckets and material settings
fn build_glb(
    mesh_buckets: &HashMap<u8, Mesh>,
    material_settings: &HashMap<u8, MaterialSettings>,
) -> Vec<u8> {
    let mut bin_buffer: Vec<u8> = Vec::new();
    let mut buffer_views = Vec::new();
    let mut accessors = Vec::new();
    let mut gltf_meshes = Vec::new();
    let mut gltf_nodes = Vec::new();
    let mut gltf_materials = Vec::new();

    // Sorted material IDs for deterministic output
    let mut mat_ids: Vec<u8> = mesh_buckets.keys().copied().collect();
    mat_ids.sort();

    // Build GLTF materials
    for &mat_id in &mat_ids {
        let defaults = MaterialSettings::default();
        let s = material_settings.get(&mat_id).unwrap_or(&defaults);
        let em_r = s.emission_color[0] * s.emission_strength;
        let em_g = s.emission_color[1] * s.emission_strength;
        let em_b = s.emission_color[2] * s.emission_strength;
        // Clamp emissive to [0,1] for GLTF spec
        let em_r = em_r.min(1.0);
        let em_g = em_g.min(1.0);
        let em_b = em_b.min(1.0);

        gltf_materials.push(format!(
            concat!(
                "{{",
                "\"name\":\"Material_{}\",",
                "\"pbrMetallicRoughness\":{{",
                "\"baseColorFactor\":[{:.4},{:.4},{:.4},1.0],",
                "\"metallicFactor\":{:.4},",
                "\"roughnessFactor\":{:.4}",
                "}},",
                "\"emissiveFactor\":[{:.4},{:.4},{:.4}]",
                "}}"
            ),
            mat_id,
            s.base_color[0],
            s.base_color[1],
            s.base_color[2],
            s.metallic,
            s.roughness,
            em_r,
            em_g,
            em_b,
        ));
    }

    // Build mesh data
    for (mesh_idx, &mat_id) in mat_ids.iter().enumerate() {
        let mesh = &mesh_buckets[&mat_id];

        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|a| match a {
                VertexAttributeValues::Float32x3(v) => Some(v),
                _ => None,
            });

        let normals = mesh
            .attribute(Mesh::ATTRIBUTE_NORMAL)
            .and_then(|a| match a {
                VertexAttributeValues::Float32x3(v) => Some(v),
                _ => None,
            });

        let Some(positions) = positions else {
            continue;
        };
        let vertex_count = positions.len();
        if vertex_count == 0 {
            continue;
        }

        // Compute position bounds (required by GLTF spec for POSITION accessor)
        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];
        for pos in positions {
            for i in 0..3 {
                min[i] = min[i].min(pos[i]);
                max[i] = max[i].max(pos[i]);
            }
        }

        let mut attr_entries = Vec::new();

        // --- Positions ---
        let pos_accessor_idx = accessors.len();
        attr_entries.push(format!("\"POSITION\":{}", pos_accessor_idx));

        let pos_offset = bin_buffer.len();
        for pos in positions {
            bin_buffer.extend_from_slice(&pos[0].to_le_bytes());
            bin_buffer.extend_from_slice(&pos[1].to_le_bytes());
            bin_buffer.extend_from_slice(&pos[2].to_le_bytes());
        }
        let pos_length = bin_buffer.len() - pos_offset;

        buffer_views.push(format!(
            "{{\"buffer\":0,\"byteOffset\":{},\"byteLength\":{},\"target\":34962}}",
            pos_offset, pos_length
        ));
        accessors.push(format!(
            concat!(
                "{{\"bufferView\":{},\"componentType\":5126,\"count\":{},\"type\":\"VEC3\",",
                "\"min\":[{:.6},{:.6},{:.6}],\"max\":[{:.6},{:.6},{:.6}]}}"
            ),
            buffer_views.len() - 1,
            vertex_count,
            min[0],
            min[1],
            min[2],
            max[0],
            max[1],
            max[2],
        ));

        // --- Normals ---
        if let Some(normals) = normals {
            let norm_accessor_idx = accessors.len();
            attr_entries.push(format!("\"NORMAL\":{}", norm_accessor_idx));

            let norm_offset = bin_buffer.len();
            for norm in normals {
                bin_buffer.extend_from_slice(&norm[0].to_le_bytes());
                bin_buffer.extend_from_slice(&norm[1].to_le_bytes());
                bin_buffer.extend_from_slice(&norm[2].to_le_bytes());
            }
            let norm_length = bin_buffer.len() - norm_offset;

            buffer_views.push(format!(
                "{{\"buffer\":0,\"byteOffset\":{},\"byteLength\":{},\"target\":34962}}",
                norm_offset, norm_length
            ));
            accessors.push(format!(
                "{{\"bufferView\":{},\"componentType\":5126,\"count\":{},\"type\":\"VEC3\"}}",
                buffer_views.len() - 1,
                vertex_count,
            ));
        }

        // --- Vertex Colors ---
        let colors = mesh.attribute(Mesh::ATTRIBUTE_COLOR).and_then(|a| match a {
            VertexAttributeValues::Float32x4(v) => Some(v.as_slice()),
            _ => None,
        });
        if let Some(colors) = colors {
            let col_accessor_idx = accessors.len();
            attr_entries.push(format!("\"COLOR_0\":{}", col_accessor_idx));

            let col_offset = bin_buffer.len();
            for col in colors {
                bin_buffer.extend_from_slice(&col[0].to_le_bytes());
                bin_buffer.extend_from_slice(&col[1].to_le_bytes());
                bin_buffer.extend_from_slice(&col[2].to_le_bytes());
                bin_buffer.extend_from_slice(&col[3].to_le_bytes());
            }
            let col_length = bin_buffer.len() - col_offset;

            buffer_views.push(format!(
                "{{\"buffer\":0,\"byteOffset\":{},\"byteLength\":{},\"target\":34962}}",
                col_offset, col_length
            ));
            accessors.push(format!(
                "{{\"bufferView\":{},\"componentType\":5126,\"count\":{},\"type\":\"VEC4\"}}",
                buffer_views.len() - 1,
                vertex_count,
            ));
        }

        // --- Indices ---
        let mut indices_accessor_str = String::new();
        if let Some(indices) = mesh.indices() {
            let idx_accessor_idx = accessors.len();
            indices_accessor_str = format!(",\"indices\":{}", idx_accessor_idx);

            let idx_offset = bin_buffer.len();
            let index_count = match indices {
                Indices::U16(idx) => {
                    for &i in idx {
                        bin_buffer.extend_from_slice(&(i as u32).to_le_bytes());
                    }
                    idx.len()
                }
                Indices::U32(idx) => {
                    for &i in idx {
                        bin_buffer.extend_from_slice(&i.to_le_bytes());
                    }
                    idx.len()
                }
            };
            let idx_length = bin_buffer.len() - idx_offset;

            buffer_views.push(format!(
                "{{\"buffer\":0,\"byteOffset\":{},\"byteLength\":{},\"target\":34963}}",
                idx_offset, idx_length
            ));
            accessors.push(format!(
                "{{\"bufferView\":{},\"componentType\":5125,\"count\":{},\"type\":\"SCALAR\"}}",
                buffer_views.len() - 1,
                index_count,
            ));
        }

        // Build mesh primitive JSON
        let attrs_json = attr_entries.join(",");
        gltf_meshes.push(format!(
            "{{\"name\":\"mesh_mat{}\",\"primitives\":[{{\"attributes\":{{{}}}{},\"material\":{}}}]}}",
            mat_id, attrs_json, indices_accessor_str, mesh_idx
        ));

        gltf_nodes.push(format!(
            "{{\"name\":\"node_mat{}\",\"mesh\":{}}}",
            mat_id, mesh_idx
        ));
    }

    // Handle empty meshes
    if gltf_nodes.is_empty() {
        return build_empty_glb();
    }

    // Assemble JSON
    let node_indices: String = (0..gltf_nodes.len())
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let json = format!(
        concat!(
            "{{",
            "\"asset\":{{\"version\":\"2.0\",\"generator\":\"L-System Explorer\"}},",
            "\"scene\":0,",
            "\"scenes\":[{{\"name\":\"LSystem\",\"nodes\":[{}]}}],",
            "\"nodes\":[{}],",
            "\"meshes\":[{}],",
            "\"materials\":[{}],",
            "\"accessors\":[{}],",
            "\"bufferViews\":[{}],",
            "\"buffers\":[{{\"byteLength\":{}}}]",
            "}}"
        ),
        node_indices,
        gltf_nodes.join(","),
        gltf_meshes.join(","),
        gltf_materials.join(","),
        accessors.join(","),
        buffer_views.join(","),
        bin_buffer.len(),
    );

    pack_glb(&json, &bin_buffer)
}

/// Build an empty but valid GLB file
fn build_empty_glb() -> Vec<u8> {
    let json = r#"{"asset":{"version":"2.0","generator":"L-System Explorer"},"scene":0,"scenes":[{"name":"Empty"}]}"#;
    pack_glb(json, &[])
}

/// Pack JSON + binary data into the GLB container format
fn pack_glb(json: &str, bin_data: &[u8]) -> Vec<u8> {
    let json_bytes = json.as_bytes();
    let json_padded_len = (json_bytes.len() + 3) & !3;

    let bin_padded_len = (bin_data.len() + 3) & !3;

    let has_bin = !bin_data.is_empty();
    let bin_chunk_size = if has_bin { 8 + bin_padded_len } else { 0 };
    let total_length = 12 + 8 + json_padded_len + bin_chunk_size;

    let mut glb = Vec::with_capacity(total_length);

    // GLB Header
    glb.extend_from_slice(&0x46546C67u32.to_le_bytes()); // magic "glTF"
    glb.extend_from_slice(&2u32.to_le_bytes()); // version
    glb.extend_from_slice(&(total_length as u32).to_le_bytes());

    // JSON Chunk
    glb.extend_from_slice(&(json_padded_len as u32).to_le_bytes());
    glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // "JSON"
    glb.extend_from_slice(json_bytes);
    // Pad with spaces to 4-byte alignment
    glb.resize(glb.len() + json_padded_len - json_bytes.len(), b' ');

    // BIN Chunk (only if there's data)
    if has_bin {
        glb.extend_from_slice(&(bin_padded_len as u32).to_le_bytes());
        glb.extend_from_slice(&0x004E4942u32.to_le_bytes()); // "BIN\0"
        glb.extend_from_slice(bin_data);
        // Pad with zeros to 4-byte alignment
        glb.resize(glb.len() + bin_padded_len - bin_data.len(), 0);
    }

    glb
}

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
    material_settings: Res<crate::core::config::MaterialSettingsMap>,
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
                    combined_obj.push_str(&mesh_to_obj_with_offset(
                        mesh,
                        &object_name,
                        vertex_offset,
                    ));
                    vertex_offset += mesh.count_vertices() as u32;
                }

                save_file(&filename, &combined_obj);
            }
            ExportFormat::Glb => {
                let glb_data = build_glb(&mesh_buckets, &material_settings.settings);
                save_file_binary(&filename, &glb_data);
            }
        }
    }

    info!("Batch export complete!");
}
