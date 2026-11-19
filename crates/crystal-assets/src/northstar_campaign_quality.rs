//! Restore the missing campaign lessons and repair directions on both cave-face editions.
use std::path::Path;

use anyhow::{Result, ensure};

use crate::CompiledGamePack;
use crate::northstar_outdoor_contours::install_reviewed_dialogue;

/// Publishes the Minnesota journey dialogue, gym rewards, and hidden-item clues on a reviewed parent.
///
/// # Errors
/// Rejects the wrong edition, stale sign or encounter data, and invalid runtime content.
pub fn revise_campaign_quality(base: &CompiledGamePack, dir: &Path) -> Result<CompiledGamePack> {
    ensure!(
        base.report
            .manifests
            .last()
            .is_some_and(|m| m == "northstar-crystal-cave-faces-v1"),
        "expected the reviewed Crystal cave face revision"
    );
    let mut pack = base.clone();
    crate::northstar_gym_lessons::complete(&mut pack)?;
    crate::northstar_story_landmarks::complete(&mut pack)?;
    crate::install_minnesota_hidden_route_clues(&mut pack)?;
    crate::northstar_intake_guidance::install(&mut pack)?;
    install_reviewed_dialogue(&mut pack, dir, "road-signs.json")?;
    pack.report
        .manifests
        .push("northstar-campaign-quality-v1".into());
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
