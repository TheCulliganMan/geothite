//! Checked release continuity for native graphical default save locations.

use std::path::Path;

use anyhow::Result;
use crystal_core::save::SAVE_EXTENSION;
use crystal_runtime::{CrystalRuntime, prepare_runtime_save_slot};

/// Explicit paths receive same-pack recovery only. Default slots may copy a
/// fully validated source from the installed or previous portable location.
pub(crate) fn prepare_native_save(
    runtime: &CrystalRuntime,
    destination: &Path,
    pack_directory: &Path,
    default_slot: bool,
) -> Result<bool> {
    if !default_slot {
        return prepare_runtime_save_slot(runtime, destination, |_| destination.to_path_buf());
    }
    let directory = destination.parent().unwrap_or(pack_directory);
    // Check all known layouts without selecting an invalid newer candidate over
    // a valid older backup. Every source path still passes exact identity and
    // complete target-state validation in the shared selector.
    for layout in 0..3 {
        let found = prepare_runtime_save_slot(runtime, destination, |source| {
            let bounded = format!(
                "{}.{}",
                crate::desktop_paths::save_stem(
                    &source.modpack_id,
                    &source.modpack_hash,
                    &source.content_hash,
                ),
                SAVE_EXTENSION
            );
            match layout {
                0 => directory.join(bounded),
                1 => pack_directory.join("saves").join(bounded),
                _ => pack_directory.join("saves").join(format!(
                    "{}-{}-{}.{}",
                    source.modpack_id, source.modpack_hash, source.content_hash, SAVE_EXTENSION
                )),
            }
        })?;
        if found {
            return Ok(true);
        }
    }
    Ok(false)
}
