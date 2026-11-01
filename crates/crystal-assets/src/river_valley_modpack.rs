//! Native publication of the individually authored river route.
use anyhow::Result;

use crate::CompiledGamePack;

/// Compiles Minnesota River Valley, both footbridges and its cave pocket.
///
/// Retains original maps, assets, audio, HOME and recovery. The actual Mankato
/// and Minneapolis ports and earned second-badge story remain regional work.
///
/// # Errors
/// Returns an error for invalid authoring, missing sources, identifier
/// collisions or a failed native pack validation.
pub fn build_river_valley_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    dossier_json: &str,
) -> Result<CompiledGamePack> {
    crate::authored_route_modpack::build_authored_route(base, layout_json, dossier_json, "MN_R03")
}
