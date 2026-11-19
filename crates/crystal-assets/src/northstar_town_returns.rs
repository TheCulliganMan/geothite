//! Rochester, Duluth and the State Fair on a return visit.
use anyhow::{Result, ensure};

use crate::CompiledGamePack;

#[cfg(test)]
#[path = "northstar_town_returns_tests.rs"]
mod tests;

/// Revises the resident conversations, ring-toss host and earned Duluth ice exit.
///
/// # Errors
/// Returns an error for an incompatible parent, changed characters or quests,
/// or an archive that fails runtime verification.
pub fn revise_town_returns(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        base.report.manifests.last().map(String::as_str) == Some("northstar-trail-discoveries-v1"),
        "town returns require the Trail Discoveries edition"
    );
    let mut pack = base.clone();
    crate::northstar_return_visits::apply(&mut pack)?;
    crate::northstar_crew_book::apply(&mut pack)?;
    crate::northstar_fair_and_residents::apply(&mut pack)?;
    crate::northstar_harbor_gym_exit::apply(&mut pack)?;
    pack.report
        .manifests
        .push("northstar-town-returns-v1".into());
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
