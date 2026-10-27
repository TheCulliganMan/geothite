//! Native publication of the hand-authored Northern Lake Circuit route.
use crate::CompiledGamePack;
use anyhow::Result;

/// Compiles Northern Lake Circuit and its collision-backed optional Cut area.
///
/// Preserves source maps, assets, audio, HOME and recovery. Regional hub ports,
/// earned progression and final visual acceptance remain separate work.
///
/// # Errors
/// Returns an error for invalid terrain/events, a bypassable Cut gate, missing
/// source data, identifier collisions or a failed native pack validation.
pub fn build_northern_lakes_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    dossier_json: &str,
) -> Result<CompiledGamePack> {
    crate::authored_route_modpack::build_authored_route(base, layout_json, dossier_json, "MN_R08")
}
