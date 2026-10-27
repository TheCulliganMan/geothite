//! Native publication of the individually authored starter route.
use anyhow::Result;

use crate::CompiledGamePack;

/// Compiles Driftless Trail, its small limestone hollow and Nelson farm.
///
/// Retains original maps, assets, audio, HOME and recovery. The actual Winona
/// and Rochester ports and earned starter/parcel story remain regional work.
///
/// # Errors
/// Returns an error for invalid authoring, missing sources, identifier
/// collisions or a failed native pack validation.
pub fn build_driftless_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    dossier_json: &str,
) -> Result<CompiledGamePack> {
    crate::authored_route_modpack::build_authored_route(base, layout_json, dossier_json, "MN_R01")
}
