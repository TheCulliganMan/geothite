pub const PLAYER_CUSTOMIZATION_MANIFEST_ID: &str = "player-customization";

use anyhow::{Result, ensure};

use crate::{
    CompiledGamePack, derive_compiled_game_pack_identity_from_manifest,
    verify_compiled_game_pack_for_runtime,
};

/// Enable name, sprite, and online handle personalization to an already verified game pack.
pub fn build_player_customization_modpack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base
            .report
            .manifests
            .iter()
            .any(|manifest| manifest == PLAYER_CUSTOMIZATION_MANIFEST_ID),
        "compiled pack already includes the player customization modpack"
    );
    ensure!(
        !base.data.player_customization,
        "compiled base pack already enables player customization"
    );

    let mut pack = base.clone();
    pack.data.player_customization = true;
    pack.report
        .manifests
        .push(PLAYER_CUSTOMIZATION_MANIFEST_ID.to_string());
    pack.identity = derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

/// Reconstruct only the exact pre-personalization pack, so existing browser saves
/// can be copied into the new slot without accepting unrelated pack identities.
pub fn player_customization_base_save_identity(pack: &CompiledGamePack)
    -> Result<Option<(crystal_core::save::SaveModpackIdentity, String)>> {
    if !pack.data.player_customization { return Ok(None); }
    verify_compiled_game_pack_for_runtime(pack)?;
    ensure!(pack.report.manifests.last().map(String::as_str) == Some(PLAYER_CUSTOMIZATION_MANIFEST_ID),
        "player customization must be the final extension for save migration");
    let mut base = pack.clone();
    base.data.player_customization = false;
    base.report.manifests.pop();
    base.identity = derive_compiled_game_pack_identity_from_manifest(
        base.format_version, &base.data, &base.audio_manifest, &base.runtime_files, &base.report,
    )?;
    verify_compiled_game_pack_for_runtime(&base)?;
    let bytes = crate::serialized_compiled_game_pack_bytes(&base)?;
    let identity = crystal_core::save::SaveModpackIdentity::from_compiled_pack_bytes(
        base.runtime_modpack_id()?, &bytes,
    )?;
    Ok(Some((identity, base.identity.content_hash)))
}
