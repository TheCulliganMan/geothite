//! First-challenge preparation and team selection in Halden's League room.
#[path = "northstar_ely_counter.rs"]
mod counter;
#[path = "northstar_ely_party.rs"]
mod party;

use anyhow::{Result, ensure};

use crate::CompiledGamePack;

/// Lets a challenger prepare for Halden and gives him a distinct first team.
///
/// # Errors
/// Returns an error for an unexpected parent, a changed League script or team,
/// or a child archive that fails runtime verification.
pub fn revise_ely_return(mut pack: CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        pack.report.manifests.last().map(String::as_str) == Some("northstar-finder-lesson-v1"),
        "Ely Return requires the Finder Lesson edition"
    );
    counter::apply(&mut pack)?;
    party::apply(&mut pack)?;
    pack.report.manifests.push("northstar-ely-return-v1".into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}
