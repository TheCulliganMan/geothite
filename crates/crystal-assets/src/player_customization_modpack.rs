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
