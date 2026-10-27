//! Native publication of the individually authored farm route.
use anyhow::Result;

use crate::CompiledGamePack;

/// Compiles Prairie Farm Road, Lind Farm and the eastern tool shed.
///
/// Retains original maps, assets, audio, HOME and recovery. The actual Rochester
/// and Mankato ports and earned first-badge story remain regional work.
///
/// # Errors
/// Returns an error for invalid authoring, missing sources, identifier
/// collisions or a failed native pack validation.
pub fn build_prairie_farm_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    dossier_json: &str,
) -> Result<CompiledGamePack> {
    crate::authored_route_modpack::build_authored_route(base, layout_json, dossier_json, "MN_R02")
}
