//! Reconnect the lake observations, harbor rescue and League desk to current progress.
use anyhow::{Result, ensure};

use crate::CompiledGamePack;

#[cfg(test)]
#[path = "northstar_league_curve_tests.rs"]
mod league_curve_tests;

/// Builds Minnesota's Lake Observations revision from a reviewed Floodwatch edition.
///
/// # Errors
/// Returns an error when the source edition or a revised landmark disagrees
/// with its native records, or when the completed content fails validation.
pub fn revise_lake_observations(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        base.report.manifests.last().map(String::as_str) == Some("northstar-floodwatch-v1"),
        "expected Minnesota Floodwatch"
    );
    let mut pack = base.clone();
    crate::northstar_observation_loop::apply(&mut pack)?;
    crate::northstar_harbor_return::apply(&mut pack)?;
    crate::northstar_hidden_approaches::apply(&mut pack)?;
    crate::northstar_league_return::apply(&mut pack)?;
    pack.report
        .manifests
        .push("northstar-lake-observations-v1".into());
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
