//! Native landmarks and directions for the Layered Fen and the Lake Pepin shore.
use anyhow::{Result, ensure};

use crate::CompiledGamePack;

/// Updates the optional lake investigations in either Town Returns edition.
///
/// # Errors
/// Returns an error if the parent or landmark data differs from the reviewed
/// edition, or if the revised archive fails runtime verification.
pub fn revise_fen_trail(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        base.report.manifests.last().map(String::as_str) == Some("northstar-town-returns-v1"),
        "the fen trail requires a Town Returns archive"
    );
    let mut pack = base.clone();
    crate::northstar_fen_observations::apply(&mut pack)?;
    crate::northstar_fen_landmarks::apply(&mut pack)?;
    crate::northstar_pepin_observations::apply(&mut pack)?;
    crate::northstar_field_margin_reads::apply(&mut pack)?;
    pack.report.manifests.push("northstar-fen-trail-v1".into());
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
