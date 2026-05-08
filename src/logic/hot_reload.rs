//! Hot-reload of `.lsys` and `.matpalette.json` files.
//!
//! Opt-in via the `--lsys <PATH>` and `--matpalette <PATH>` CLI flags. When a
//! path is provided, the asset is loaded through Bevy's asset server (with the
//! `file_watcher` feature) so external edits trigger an `AssetEvent::Modified`
//! and we re-apply the file's content to the editor state.
//!
//! The user can still edit the source in the editor; in-editor edits override
//! whatever is on disk until the next `Modified` event arrives. This is by
//! design — hot-reload is a developer convenience, not authoritative state.

use crate::core::config::{
    DirtyFlags, LSystemConfig, MaterialSettingsChanged, MaterialSettingsMap,
};
use bevy::prelude::*;
use bevy_symbios::loader::{LSystemSource, MaterialSettingsSource};

/// Holds opt-in handles to externally-loaded grammar / palette files.
///
/// Populated by [`load_cli_assets`] from `--lsys` and `--matpalette` CLI args.
#[derive(Resource, Default)]
pub struct WatchedAssets {
    pub lsys: Option<Handle<LSystemSource>>,
    pub palette: Option<Handle<MaterialSettingsSource>>,
}

/// Parses CLI args (`--lsys <path>` / `--matpalette <path>`) and starts loading
/// the matching files. Files outside Bevy's asset directory must be referenced
/// by absolute or workspace-relative paths.
pub fn load_cli_assets(asset_server: Res<AssetServer>, mut watched: ResMut<WatchedAssets>) {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--lsys" => {
                if let Some(path) = args.next() {
                    info!("Loading L-system grammar from {path}");
                    watched.lsys = Some(asset_server.load(path));
                }
            }
            "--matpalette" => {
                if let Some(path) = args.next() {
                    info!("Loading material palette from {path}");
                    watched.palette = Some(asset_server.load(path));
                }
            }
            _ => {}
        }
    }
}

/// Applies an `.lsys` asset to [`LSystemConfig`] whenever the underlying file
/// changes. The current grammar and finalization are overwritten with the
/// reconstructed source from the parsed `System`.
pub fn apply_lsys_reload(
    watched: Res<WatchedAssets>,
    mut events: MessageReader<AssetEvent<LSystemSource>>,
    sources: Res<Assets<LSystemSource>>,
    mut config: ResMut<LSystemConfig>,
) {
    let Some(handle) = &watched.lsys else {
        events.clear();
        return;
    };

    for event in events.read() {
        let id = match event {
            AssetEvent::Added { id }
            | AssetEvent::Modified { id }
            | AssetEvent::LoadedWithDependencies { id } => *id,
            _ => continue,
        };
        if id != handle.id() {
            continue;
        }
        let Some(source) = sources.get(handle) else {
            continue;
        };
        config.source_code = source.0.to_source();
        config.finalization_code.clear();
        config.recompile_requested = true;
        info!("Hot-reloaded grammar from .lsys");
    }
}

/// Applies a `.matpalette.json` asset to [`MaterialSettingsMap`] whenever the
/// underlying file changes, then triggers [`MaterialSettingsChanged`] so the
/// observer re-applies handles.
pub fn apply_palette_reload(
    mut commands: Commands,
    watched: Res<WatchedAssets>,
    mut events: MessageReader<AssetEvent<MaterialSettingsSource>>,
    sources: Res<Assets<MaterialSettingsSource>>,
    mut palette: ResMut<MaterialSettingsMap>,
    mut dirty: ResMut<DirtyFlags>,
) {
    let Some(handle) = &watched.palette else {
        events.clear();
        return;
    };

    for event in events.read() {
        let id = match event {
            AssetEvent::Added { id }
            | AssetEvent::Modified { id }
            | AssetEvent::LoadedWithDependencies { id } => *id,
            _ => continue,
        };
        if id != handle.id() {
            continue;
        }
        let Some(source) = sources.get(handle) else {
            continue;
        };
        palette.settings.clear();
        for (slot, settings) in &source.0 {
            palette.settings.insert(*slot, settings.clone());
        }
        commands.trigger(MaterialSettingsChanged);
        dirty.geometry = true;
        info!("Hot-reloaded material palette from .matpalette.json");
    }
}
