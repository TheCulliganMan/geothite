//! Route 18 and Mille Lacs Shore Watch revision.
use anyhow::{Result, ensure};

use crate::CompiledGamePack;

/// Builds the Shore Watch revision of either League Journey edition.
///
/// # Errors
/// Returns an error if the parent, shoreline, or quest differs from the expected
/// edition, or if the resulting archive fails runtime validation.
pub fn revise_shore_watch(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        base.report.manifests.last().map(String::as_str) == Some("northstar-league-journey-v1"),
        "Shore Watch requires a League Journey edition"
    );
    let mut pack = base.clone();
    crate::northstar_shore_watch_route::apply(&mut pack)?;
    crate::northstar_mille_lacs_shore::apply(&mut pack)?;
    crate::northstar_shore_watch_reports::apply(&mut pack)?;
    pack.report.manifests.push("northstar-shore-watch-v1".into());
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
