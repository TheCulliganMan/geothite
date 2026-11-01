//! Native publication of the individually authored civic park route.
use anyhow::Result;

use crate::CompiledGamePack;

/// Compiles the Twin Cities park crossing and its battle/grass alternative.
///
/// Retains original maps, assets, audio, HOME and recovery. The actual Minneapolis
/// and Saint Paul ports and earned third-badge story remain regional work.
///
/// # Errors
/// Returns an error for invalid authoring, missing sources, identifier
/// collisions or a failed native pack validation.
pub fn build_twin_cities_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    dossier_json: &str,
) -> Result<CompiledGamePack> {
    crate::authored_route_modpack::build_authored_route(base, layout_json, dossier_json, "MN_R04")
}
