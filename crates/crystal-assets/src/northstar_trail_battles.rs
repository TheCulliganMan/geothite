//! Campaign directions, city opponents and sight battles at two occupied doors.
use anyhow::{Result, ensure};

use crate::CompiledGamePack;

#[cfg(test)]
#[path = "northstar_cave_escape_tests.rs"]
mod cave_escape_tests;

#[cfg(test)]
#[path = "northstar_trail_battles_tests.rs"]
mod tests;

/// Revises the two Fen Trail editions using their existing opponents and events.
///
/// # Errors
/// Rejects a different parent, a changed trainer or map, or an archive that fails verification.
pub fn revise_trail_battles(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        base.report.manifests.last().map(String::as_str) == Some("northstar-fen-trail-v1"),
        "campaign battles require the Fen Trail edition"
    );
    let mut pack = base.clone();
    crate::northstar_city_parties::apply(&mut pack)?;
    crate::northstar_trainer_lanes::apply(&mut pack)?;
    crate::northstar_badge_handoffs::apply(&mut pack)?;
    crate::northstar_fen_bank::apply(&mut pack)?;
    crate::northstar_fen_turnoff::apply(&mut pack)?;
    crate::northstar_bank_directions::apply(&mut pack)?;
    crate::northstar_fen_circuit::apply(&mut pack)?;
    pack.report
        .manifests
        .push("northstar-trail-battles-v1".into());
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
