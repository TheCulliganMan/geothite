//! Publish the hand-authored North Shore through the common native compiler.
use crate::CompiledGamePack;
use anyhow::{Result, ensure};

/// Compiles all three North Shore chunks and its keeper room.
///
/// Original maps, spawns and audio are preserved. Regional hub ports remain
/// unconnected; this overlay does not establish a connected new-game journey.
///
/// # Errors
/// Returns an error for invalid authoring, missing sources, identifier collisions
/// or failed native pack validation.
pub fn build_north_shore_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    dossier_json: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        crate::authored_route_layout::Layout::parse(layout_json)?.dimensions_metatiles == [32, 72],
        "expanded North Shore must use the installed-component expansion API to preserve numeric IDs"
    );
    crate::authored_route_modpack::build_authored_route(base, layout_json, dossier_json, "MN_R11")
}
