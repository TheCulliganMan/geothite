//! Publish the revised Minnesota discovery trails from either reviewed edition.
use anyhow::{Result, ensure};

use crate::CompiledGamePack;

/// Installs the optional trail, ranger, trainer and hidden-item changes in Minnesota.
///
/// # Errors
/// Returns an error for an incompatible parent, a changed quest or landmark,
/// or a result that fails the runtime archive checks.
pub fn revise_trail_discoveries(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    const MANIFEST: &str = "northstar-trail-discoveries-v1";
    ensure!(
        base.report
            .manifests
            .iter()
            .any(|s| s == "northstar-headwaters-journey-v1")
            && !base.report.manifests.iter().any(|s| s == MANIFEST),
        "expected an unmodified discovery trail in a Headwaters Journey descendant"
    );
    let mut pack = base.clone();
    crate::northstar_tall_tale_trail::apply(&mut pack)?;
    crate::northstar_safari_trail::apply(&mut pack)?;
    crate::northstar_rare_offers::apply(&mut pack)?;
    crate::northstar_trail_caches::apply(&mut pack)?;
    crate::northstar_reed_partner::apply(&mut pack)?;
    pack.report.manifests.push(MANIFEST.into());
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
