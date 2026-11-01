//! Publish the individually authored granite route using production map APIs.
use anyhow::Result;

use crate::CompiledGamePack;

/// Compiles the three clipped League Approach maps and Cedar Shelter.
///
/// Preserves source maps, art, recovery destinations and audio. The Boundary
/// Waters and Ely hub ports and Q08 gate remain separate regional work.
///
/// # Errors
/// Returns an error for invalid authoring, missing sources, identifier collisions
/// or a failed native pack validation.
pub fn build_league_approach_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    dossier_json: &str,
) -> Result<CompiledGamePack> {
    crate::authored_route_modpack::build_authored_route(base, layout_json, dossier_json, "MN_R13")
}
