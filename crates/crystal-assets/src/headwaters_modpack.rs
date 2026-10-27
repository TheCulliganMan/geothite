//! Publish the hand-authored Headwaters route through production map APIs.
use anyhow::Result;

use crate::CompiledGamePack;

/// Compiles all four clipped Headwaters chunks and its native wooden-rail crossing.
///
/// Preserves source maps, art, recovery destinations and audio. Real Bemidji
/// and Lake Itasca ports and the earned Q05 journey remain regional work.
///
/// # Errors
/// Returns an error for invalid authoring, missing sources, identifier collisions
/// or failed native pack validation.
pub fn build_headwaters_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    dossier_json: &str,
) -> Result<CompiledGamePack> {
    crate::authored_route_modpack::build_authored_route(base, layout_json, dossier_json, "MN_R14")
}
