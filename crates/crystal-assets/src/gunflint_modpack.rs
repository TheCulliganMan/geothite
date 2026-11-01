//! Publish the hand-authored Gunflint route through production map APIs.
use crate::CompiledGamePack;
use anyhow::Result;

/// Compiles all six Gunflint chunks, Raven Rock and the ranger hut.
///
/// Retains original maps, spawns, catalogs and audio. Regional hub ports and Q07
/// approach gates must be implemented with the hubs before progression is proven.
///
/// # Errors
/// Returns an error for invalid authoring, missing sources, identifier collisions
/// or failed native pack validation.
pub fn build_gunflint_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    dossier_json: &str,
) -> Result<CompiledGamePack> {
    crate::authored_route_modpack::build_authored_route(base, layout_json, dossier_json, "MN_R12")
}
