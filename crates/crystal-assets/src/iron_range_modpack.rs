//! Native publication of the hand-authored Iron Range West quarry route.
use anyhow::Result;

use crate::CompiledGamePack;

/// Compiles four clipped quarry-route chunks and the survey mine interior.
///
/// Uses the native Iron Range rail art, collision-backed optional Cut area,
/// habitat choices and one-way spoil return. The mine's native entrance and
/// separate passive FLOOR return preserve the authored safe landing. Existing
/// maps, catalogs, source assets, audio, HOME and recovery remain unchanged.
///
/// Includes the Q06 foreman battle and separate saved instrument recovery in
/// the mine. Grand Rapids handover and the earned regional journey remain
/// separate story work; mine victory alone does not complete the depot quest.
///
/// # Errors
/// Returns an error for invalid terrain/events, broken traversal or room returns,
/// missing source content, identifier collisions or failed pack validation.
pub fn build_iron_range_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    dossier_json: &str,
) -> Result<CompiledGamePack> {
    let pack = crate::authored_route_modpack::build_authored_route(
        base,
        layout_json,
        dossier_json,
        "MN_R09",
    )?;
    crate::build_iron_range_story_modpack(
        &pack,
        dossier_json,
        include_str!("../../../planning/minnesota/story.json"),
    )
}
