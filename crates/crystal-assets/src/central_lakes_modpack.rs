//! Publish the hand-authored Central Lakes route and its real lake cabin.
use anyhow::Result;

use crate::CompiledGamePack;

/// Compiles one compact Central Lakes map and its lakeside cabin.
///
/// Preserves source maps, art, audio and recovery destinations. Real Brainerd
/// and Detroit Lakes ports and the earned SQ02 journey remain regional work.
///
/// # Errors
/// Returns an error for invalid authoring, missing sources, identifier collisions
/// or a failed native pack validation.
pub fn build_central_lakes_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    dossier_json: &str,
) -> Result<CompiledGamePack> {
    crate::authored_route_modpack::build_authored_route(base, layout_json, dossier_json, "MN_R15")
}
