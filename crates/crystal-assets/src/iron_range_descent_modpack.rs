//! Native publication of the authored carrier bench, mine gallery and shore road.
use anyhow::Result;

use crate::CompiledGamePack;

/// Compiles Iron Range Descent's two clipped exterior maps and optional gallery.
///
/// Retains source maps, files, audio, HOME and recovery. The original Johto
/// tileset supplies every exterior block; no derived art is required. Regional
/// gates and earned depot progression remain separate integration work.
///
/// # Errors
/// Returns an error for invalid authoring, missing source content, identifier
/// collisions or a failed native pack validation.
pub fn build_iron_range_descent_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    dossier_json: &str,
) -> Result<CompiledGamePack> {
    crate::authored_route_modpack::build_authored_route(base, layout_json, dossier_json, "MN_R10")
}
