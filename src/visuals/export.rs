use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;

use crate::core::config::{
    ExportConfig, ExportFormat, LSystemConfig, MaterialSettingsMap, PropConfig, PropMeshType,
};
use crate::visuals::assets::PropMeshAssets;

use bevy_symbios::LSystemMeshBuilder;
use bevy_symbios::export::{mesh_to_obj, meshes_to_glb};
use bevy_symbios::materials::MaterialSettings;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use symbios::System;
use symbios_turtle_3d::{SkeletonProp, TurtleConfig, TurtleInterpreter};

// ---------------------------------------------------------------------------
// Platform-specific file I/O
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
pub fn save_file(filename: &str, content: &str) -> Result<(), String> {
    use wasm_bindgen::JsCast;

    let window = web_sys::window().ok_or("No browser window available")?;
    let document = window.document().ok_or("No document available")?;

    let blob_parts = js_sys::Array::new();
    blob_parts.push(&wasm_bindgen::JsValue::from_str(content));

    let mut options = web_sys::BlobPropertyBag::new();
    options.type_("text/plain");

    let blob = web_sys::Blob::new_with_str_sequence_and_options(&blob_parts, &options)
        .map_err(|e| format!("Failed to create blob: {:?}", e))?;

    let url = web_sys::Url::create_object_url_with_blob(&blob)
        .map_err(|e| format!("Failed to create URL: {:?}", e))?;

    let anchor: web_sys::HtmlAnchorElement = document
        .create_element("a")
        .map_err(|e| format!("Failed to create anchor: {:?}", e))?
        .dyn_into()
        .map_err(|_| "Element is not an anchor".to_string())?;

    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();

    let _ = web_sys::Url::revoke_object_url(&url);
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_file(filename: &str, content: &str) -> Result<(), String> {
    use std::fs;
    use std::path::Path;

    let export_dir = Path::new("exports");
    if !export_dir.exists() {
        fs::create_dir_all(export_dir)
            .map_err(|e| format!("Failed to create exports directory: {}", e))?;
    }

    let path = export_dir.join(filename);
    fs::write(&path, content).map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    info!("Exported: {}", path.display());
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn save_file_binary(filename: &str, content: &[u8]) -> Result<(), String> {
    use wasm_bindgen::JsCast;

    let window = web_sys::window().ok_or("No browser window available")?;
    let document = window.document().ok_or("No document available")?;

    let uint8arr = js_sys::Uint8Array::new_with_length(content.len() as u32);
    uint8arr.copy_from(content);

    let parts = js_sys::Array::new();
    parts.push(&uint8arr);

    let mut options = web_sys::BlobPropertyBag::new();
    options.type_("application/octet-stream");

    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options)
        .map_err(|e| format!("Failed to create blob: {:?}", e))?;

    let url = web_sys::Url::create_object_url_with_blob(&blob)
        .map_err(|e| format!("Failed to create URL: {:?}", e))?;

    let anchor: web_sys::HtmlAnchorElement = document
        .create_element("a")
        .map_err(|e| format!("Failed to create anchor: {:?}", e))?
        .dyn_into()
        .map_err(|_| "Element is not an anchor".to_string())?;

    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();

    let _ = web_sys::Url::revoke_object_url(&url);
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_file_binary(filename: &str, content: &[u8]) -> Result<(), String> {
    use std::fs;
    use std::path::Path;

    let export_dir = Path::new("exports");
    if !export_dir.exists() {
        fs::create_dir_all(export_dir)
            .map_err(|e| format!("Failed to create exports directory: {}", e))?;
    }

    let path = export_dir.join(filename);
    fs::write(&path, content).map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    info!("Exported: {}", path.display());
    Ok(())
}

/// Tracks the result and progress of export operations for UI feedback.
#[derive(Resource, Default)]
pub struct ExportStatus {
    /// None = no export attempted or success, Some = error message.
    pub error: Option<String>,
    /// Number of files successfully exported in the last batch.
    pub last_export_count: usize,
    /// Whether a background export is currently running.
    pub exporting: bool,
    /// Progress counter shared with background thread.
    pub progress: Option<Arc<AtomicUsize>>,
    /// Total number of variants being exported.
    pub total: usize,
    /// Shared result container for the background export task.
    pending_result: Option<Arc<Mutex<Option<ExportResult>>>>,
}

/// Result from a background batch export.
struct ExportResult {
    count: usize,
    error: Option<String>,
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

    let src_uvs = source_mesh
        .attribute(Mesh::ATTRIBUTE_UV_0)
        .and_then(|a| match a {
            VertexAttributeValues::Float32x2(v) => Some(v),
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
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, Vec::<[f32; 2]>::new());
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

    // Append UVs (use source UVs or zero placeholders to maintain vertex count alignment)
    if let Some(VertexAttributeValues::Float32x2(bucket_uvs)) =
        bucket.attribute_mut(Mesh::ATTRIBUTE_UV_0)
    {
        if let Some(uvs) = src_uvs {
            bucket_uvs.extend(uvs.iter().copied());
        } else {
            bucket_uvs.extend(std::iter::repeat_n([0.0, 0.0], src_positions.len()));
        }
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

/// Captures all data needed for a batch export, cloned from ECS resources
/// so the export can run on a background thread.
struct BatchExportParams {
    source_code: String,
    iterations: usize,
    seed: u64,
    step_size: f32,
    default_angle: f32,
    default_width: f32,
    tropism: Option<Vec3>,
    elasticity: f32,
    variation_count: usize,
    base_filename: String,
    format: ExportFormat,
    material_settings: HashMap<u8, MaterialSettings>,
    prop_meshes: HashMap<u16, PropMeshType>,
    prop_scale: f32,
    /// Pre-extracted prop mesh data (cloned from Assets<Mesh>), keyed by PropMeshType.
    extracted_prop_meshes: HashMap<PropMeshType, Mesh>,
}

/// System that dispatches batch export to a background thread when requested.
#[allow(clippy::too_many_arguments)]
pub fn batch_export_system(
    mut export_config: ResMut<ExportConfig>,
    mut export_status: ResMut<ExportStatus>,
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

    if export_status.exporting {
        return; // Already exporting
    }

    export_status.error = None;
    export_status.last_export_count = 0;
    export_status.exporting = true;
    export_status.total = export_config.variation_count;

    // Pre-extract prop mesh data from assets so the background thread has it
    let mut extracted_prop_meshes = HashMap::new();
    for (_, mesh_type) in prop_config.prop_meshes.iter() {
        if !extracted_prop_meshes.contains_key(mesh_type)
            && let Some(handle) = prop_assets.meshes.get(mesh_type)
            && let Some(mesh) = mesh_assets.get(handle)
        {
            extracted_prop_meshes.insert(*mesh_type, mesh.clone());
        }
    }

    let params = BatchExportParams {
        source_code: lsystem_config.source_code.clone(),
        iterations: lsystem_config.iterations,
        seed: lsystem_config.seed,
        step_size: lsystem_config.step_size,
        default_angle: lsystem_config.default_angle,
        default_width: lsystem_config.default_width,
        tropism: lsystem_config.tropism,
        elasticity: lsystem_config.elasticity,
        variation_count: export_config.variation_count,
        base_filename: export_config.base_filename.clone(),
        format: export_config.format,
        material_settings: material_settings.settings.clone(),
        prop_meshes: prop_config.prop_meshes.clone(),
        prop_scale: prop_config.prop_scale,
        extracted_prop_meshes,
    };

    let progress = Arc::new(AtomicUsize::new(0));
    let result: Arc<Mutex<Option<ExportResult>>> = Arc::new(Mutex::new(None));

    export_status.progress = Some(progress.clone());
    export_status.pending_result = Some(result.clone());

    info!(
        "Starting async batch export: {} variations as {}",
        params.variation_count,
        params.format.name()
    );

    let pool = AsyncComputeTaskPool::get();
    pool.spawn(async move {
        let export_result = perform_batch_export(&params, &progress);
        if let Ok(mut guard) = result.lock() {
            *guard = Some(export_result);
        }
    })
    .detach();
}

/// Performs the full batch export on a background thread.
fn perform_batch_export(params: &BatchExportParams, progress: &Arc<AtomicUsize>) -> ExportResult {
    let mut count = 0usize;

    for variant_idx in 0..params.variation_count {
        let mut sys = System::new();
        let variant_seed = if variant_idx == 0 {
            // First variant uses the editor's exact seed for an identical result
            params.seed
        } else {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            params.seed.hash(&mut hasher);
            variant_idx.hash(&mut hasher);
            hasher.finish()
        };
        sys.set_seed(variant_seed);

        let mut axiom_set = false;

        for line in params.source_code.lines() {
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
            progress.fetch_add(1, Ordering::Relaxed);
            continue;
        }

        if sys.derive(params.iterations).is_err() {
            progress.fetch_add(1, Ordering::Relaxed);
            continue;
        }

        // Configure turtle interpreter
        let default_step = sys
            .constants
            .get("step")
            .map(|&s| s as f32)
            .unwrap_or(params.step_size);

        let default_angle = sys
            .constants
            .get("angle")
            .map(|&a| a as f32)
            .unwrap_or(params.default_angle)
            .to_radians();

        let initial_width = sys
            .constants
            .get("width")
            .map(|&w| w as f32)
            .unwrap_or(params.default_width);

        let turtle_config = TurtleConfig {
            default_step,
            default_angle,
            initial_width,
            tropism: params.tropism,
            elasticity: params.elasticity,
            max_stack_depth: 1024,
        };

        let mut interpreter = TurtleInterpreter::new(turtle_config);
        interpreter.populate_standard_symbols(&sys.interner);

        let skeleton = interpreter.build_skeleton(&sys.state);
        let builder = LSystemMeshBuilder::new().with_resolution(8);
        let mut mesh_buckets = builder.build(&skeleton);

        // Merge props using pre-extracted mesh data
        for prop in &skeleton.props {
            let mesh_type = params
                .prop_meshes
                .get(&prop.prop_id)
                .copied()
                .unwrap_or_default();

            if let Some(source_mesh) = params.extracted_prop_meshes.get(&mesh_type) {
                merge_prop_into_bucket(&mut mesh_buckets, source_mesh, prop, params.prop_scale);
            }
        }

        let filename = format!(
            "{}_{:02}.{}",
            params.base_filename,
            variant_idx + 1,
            params.format.extension()
        );

        let save_result = match params.format {
            ExportFormat::Obj => {
                let mut combined_obj = String::new();
                combined_obj.push_str("# Exported from L-System Explorer\n");
                combined_obj.push_str(&format!(
                    "# Variant {} of {}\n\n",
                    variant_idx + 1,
                    params.variation_count
                ));

                let mut vertex_offset = 0u32;
                for (material_id, mesh) in &mesh_buckets {
                    let object_name = format!(
                        "{}_{:02}_mat{}",
                        params.base_filename,
                        variant_idx + 1,
                        material_id
                    );
                    combined_obj.push_str(&mesh_to_obj(mesh, &object_name, vertex_offset));
                    vertex_offset += mesh.count_vertices() as u32;
                }

                save_file(&filename, &combined_obj)
            }
            ExportFormat::Glb => {
                let glb_data = meshes_to_glb(&mesh_buckets, &params.material_settings);
                save_file_binary(&filename, &glb_data)
            }
        };

        match save_result {
            Ok(()) => {
                count += 1;
            }
            Err(e) => {
                progress.fetch_add(1, Ordering::Relaxed);
                return ExportResult {
                    count,
                    error: Some(e),
                };
            }
        }

        progress.fetch_add(1, Ordering::Relaxed);
    }

    ExportResult { count, error: None }
}

/// System that polls for completed background export tasks.
pub fn poll_export_status(mut export_status: ResMut<ExportStatus>) {
    if !export_status.exporting {
        return;
    }

    let Some(result_arc) = &export_status.pending_result else {
        return;
    };

    let Ok(mut guard) = result_arc.lock() else {
        return;
    };

    let Some(result) = guard.take() else {
        return; // Not done yet
    };

    drop(guard);

    export_status.last_export_count = result.count;
    export_status.error = result.error;
    export_status.exporting = false;
    export_status.pending_result = None;
    export_status.progress = None;

    if export_status.error.is_none() {
        info!(
            "Batch export complete: {} files",
            export_status.last_export_count
        );
    }
}
