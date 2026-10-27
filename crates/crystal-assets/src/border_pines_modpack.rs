//! Publish the hand-authored Border Pines forest passage and its two rooms.
use anyhow::Result;

use crate::CompiledGamePack;

/// Compiles three clipped forest chunks, the granite den and ranger hut.
///
/// Uses unchanged native forest art and exact imported timber rails. The Cut
/// shortcut serves the nearby cache/watcher; its local saving is distinct from
/// the two-step regional saving. Existing maps, source files, catalogs, audio,
/// HOME and recovery remain unchanged. Earned SQ03 progression is separate.
///
/// # Errors
/// Returns an error for invalid terrain/events, unsupported source art, unsafe
/// room returns, identifier collisions or failed native pack validation.
pub fn build_border_pines_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    dossier_json: &str,
) -> Result<CompiledGamePack> {
    crate::authored_route_modpack::build_authored_route(base, layout_json, dossier_json, "MN_R17")
}
