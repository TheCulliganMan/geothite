//! First-visit cave clues, Duluth supplies and League retreat directions.
use anyhow::{Result, ensure};

use crate::CompiledGamePack;

/// Installs the Route 3 investigation clues, dock stock and League advice.
///
/// # Errors
/// Returns an error when the source edition or expected maps differ, or the
/// revised content fails pack verification.
pub fn revise_cave_trip(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        base.report.manifests.last().map(String::as_str) == Some("northstar-gym-walk-v1"),
        "Cave Returns requires a Gym Walk parent"
    );
    let mut pack = base.clone();
    crate::northstar_cave_returns::apply(&mut pack)?;
    crate::northstar_league_provisions::apply(&mut pack)?;
    crate::northstar_league_retreat_advice::apply(&mut pack)?;
    pack.report
        .manifests
        .push("northstar-cave-returns-v1".into());
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
