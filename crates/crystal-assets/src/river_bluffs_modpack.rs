//! Native publication of the individually authored ridge route.
use anyhow::Result;

use crate::CompiledGamePack;

/// Compiles River Bluffs North, its ledge returns and two-ended rock cut.
///
/// Retains original maps, assets, audio, HOME and recovery. The actual Minneapolis
/// and Saint Cloud ports and earned fourth-badge story remain regional work.
///
/// # Errors
/// Returns an error for invalid authoring, missing sources, identifier
/// collisions or a failed native pack validation.
pub fn build_river_bluffs_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    dossier_json: &str,
) -> Result<CompiledGamePack> {
    crate::authored_route_modpack::build_authored_route(base, layout_json, dossier_json, "MN_R05")
}
