//! First-visit navigation through the six gyms outside St Paul and Duluth.
use anyhow::{Result, ensure};

use crate::CompiledGamePack;

/// Installs the reviewed stairs, controls, swimming beach and garden directions.
///
/// # Errors
/// Returns an error if the parent or any expected gym feature differs, or if
/// the result fails the runtime's pack verifier.
pub fn revise_gym_walk(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        base.report.manifests.last().map(String::as_str) == Some("northstar-badge-trails-v1"),
        "Gym Walk requires a Badge Trails parent"
    );
    let mut pack = base.clone();
    crate::northstar_gym_walk_rochester::apply(&mut pack)?;
    crate::northstar_gym_walk_relays::apply(&mut pack)?;
    crate::northstar_gym_walk_north::apply(&mut pack)?;
    crate::northstar_gym_walk_pools::apply(&mut pack)?;
    pack.report.manifests.push("northstar-gym-walk-v1".into());
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
