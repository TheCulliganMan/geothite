//! Publish the optional Moorhead prairie circuit and repair two rescue journeys.
use anyhow::{Result, ensure};

use crate::CompiledGamePack;

#[cfg(test)]
#[path = "northstar_floodwatch_tests.rs"]
mod tests;

/// Builds the Floodwatch revision from either reviewed League and Fair edition.
///
/// # Errors
/// Rejects an unexpected parent, stale scripts or parties, or an invalid result.
pub fn revise_floodwatch(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        base.report.manifests.last().map(String::as_str) == Some("northstar-league-and-fair-v1"),
        "expected Minnesota's League and Fair revision"
    );
    let mut pack = base.clone();
    crate::northstar_prairie_run::apply(&mut pack)?;
    crate::northstar_ice_handoff::apply(&mut pack)?;
    crate::northstar_mine_handoff::apply(&mut pack)?;
    crate::northstar_porch_garden::apply(&mut pack)?;
    pack.report.manifests.push("northstar-floodwatch-v1".into());
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
