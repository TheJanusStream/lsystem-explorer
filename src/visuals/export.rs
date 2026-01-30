use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::core::config::{ExportConfig, ExportFormat, LSystemConfig, MaterialSettingsMap, PropConfig};
use crate::visuals::assets::PropMeshAssets;

use bevy_symbios::LSystemMeshBuilder;
use bevy_symbios::export::{mesh_to_obj, meshes_to_glb};
use symbios::System;
use symbios_turtle_3d::{SkeletonProp, TurtleConfig, TurtleInterpreter};

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
// Prop Mesh Merging
// ---------------------------------------------------------------------------

/// Merges a prop's transformed geometry into the appropriate material bucket.
///
/// Takes the source prop mesh, transforms vertices by the prop's position/rotation/scale,
/// tints vertex colors by the prop's color, and appends to the bucket matching the prop's material_id.
fn merge_prop_into_bucket(
    buckets: &mut HashMap<u8, Mesh>,
    source_mesh: &Mesh,
    prop: &SkeletonProp,
    prop_scale: f32,
) {
    let Some(VertexAttributeValues::Float32x3(src_positions)) =
        source_mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return;
    };

    let src_normals = source_mesh
        .attribute(Mesh::ATTRIBUTE_NORMAL)
        .and_then(|a| match a {
            VertexAttributeValues::Float32x3(v) => Some(v),
            _ => None,
        });

    let src_colors = source_mesh
        .attribute(Mesh::ATTRIBUTE_COLOR)
        .and_then(|a| match a {
            VertexAttributeValues::Float32x4(v) => Some(v),
            _ => None,
        });

    // Transform vertices
    let combined_scale = prop.scale * prop_scale;
    let mut transformed_positions: Vec<[f32; 3]> = Vec::with_capacity(src_positions.len());
    let mut transformed_normals: Vec<[f32; 3]> = Vec::with_capacity(src_positions.len());
    let mut transformed_colors: Vec<[f32; 4]> = Vec::with_capacity(src_positions.len());

    for (i, pos) in src_positions.iter().enumerate() {
        // Apply scale, rotation, then translation
        let scaled = Vec3::new(
            pos[0] * combined_scale.x,
            pos[1] * combined_scale.y,
            pos[2] * combined_scale.z,
        );
        let rotated = prop.rotation * scaled;
        let final_pos = rotated + prop.position;
        transformed_positions.push([final_pos.x, final_pos.y, final_pos.z]);

        // Transform normal (rotation only, no scale/translation)
        if let Some(normals) = src_normals {
            let norm = Vec3::new(normals[i][0], normals[i][1], normals[i][2]);
            let rotated_norm = (prop.rotation * norm).normalize();
            transformed_normals.push([rotated_norm.x, rotated_norm.y, rotated_norm.z]);
        }

        // Tint vertex color by prop color
        if let Some(colors) = src_colors {
            transformed_colors.push([
                colors[i][0] * prop.color.x,
                colors[i][1] * prop.color.y,
                colors[i][2] * prop.color.z,
                colors[i][3] * prop.color.w,
            ]);
        } else {
            // No source colors - use prop color directly
            transformed_colors.push([prop.color.x, prop.color.y, prop.color.z, prop.color.w]);
        }
    }

    // Get or create the bucket mesh for this material
    let bucket = buckets.entry(prop.material_id).or_insert_with(|| {
        // Create empty mesh using a zero-sized primitive, then replace attributes
        let mut mesh = Mesh::from(Cuboid::new(0.0, 0.0, 0.0));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<[f32; 3]>::new());
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, Vec::<[f32; 4]>::new());
        mesh.remove_attribute(Mesh::ATTRIBUTE_UV_0);
        mesh.insert_indices(Indices::U32(Vec::new()));
        mesh
    });

    // Get current vertex count for index offset
    let vertex_offset = bucket.count_vertices() as u32;

    // Append positions
    if let Some(VertexAttributeValues::Float32x3(bucket_positions)) =
        bucket.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    {
        bucket_positions.extend(transformed_positions);
    }

    // Append normals
    if let Some(VertexAttributeValues::Float32x3(bucket_normals)) =
        bucket.attribute_mut(Mesh::ATTRIBUTE_NORMAL)
    {
        bucket_normals.extend(transformed_normals);
    }

    // Append colors
    if let Some(VertexAttributeValues::Float32x4(bucket_colors)) =
        bucket.attribute_mut(Mesh::ATTRIBUTE_COLOR)
    {
        bucket_colors.extend(transformed_colors);
    }

    // Append indices with offset
    if let (Some(src_indices), Some(Indices::U32(bucket_indices))) =
        (source_mesh.indices(), bucket.indices_mut())
    {
        match src_indices {
            Indices::U16(idx) => {
                bucket_indices.extend(idx.iter().map(|&i| i as u32 + vertex_offset));
            }
            Indices::U32(idx) => {
                bucket_indices.extend(idx.iter().map(|&i| i + vertex_offset));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Batch Export System
// ---------------------------------------------------------------------------

/// System that handles batch export when requested
#[allow(clippy::too_many_arguments)]
pub fn batch_export_system(
    mut export_config: ResMut<ExportConfig>,
    lsystem_config: Res<LSystemConfig>,
    material_settings: Res<MaterialSettingsMap>,
    prop_config: Res<PropConfig>,
    prop_assets: Res<PropMeshAssets>,
    mesh_assets: Res<Assets<Mesh>>,
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
        let mut mesh_buckets = builder.build(&skeleton);

        // Merge props into the mesh buckets
        for prop in &skeleton.props {
            let mesh_type = prop_config
                .prop_meshes
                .get(&prop.prop_id)
                .copied()
                .unwrap_or_default();

            if let Some(mesh_handle) = prop_assets.meshes.get(&mesh_type)
                && let Some(source_mesh) = mesh_assets.get(mesh_handle)
            {
                merge_prop_into_bucket(
                    &mut mesh_buckets,
                    source_mesh,
                    prop,
                    prop_config.prop_scale,
                );
            }
        }

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
