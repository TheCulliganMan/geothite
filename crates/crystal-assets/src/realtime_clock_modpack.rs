pub const REALTIME_CLOCK_MANIFEST_ID: &str = "realtime-clock";

use anyhow::{Result, ensure};

use crate::{
    CompiledGamePack, derive_compiled_game_pack_identity_from_manifest,
    verify_compiled_game_pack_for_runtime,
};

/// Add the standard shared server clock to an already verified game pack.
pub fn build_realtime_clock_modpack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base
            .report
            .manifests
            .iter()
            .any(|manifest| manifest == REALTIME_CLOCK_MANIFEST_ID),
        "compiled pack already includes the real-time clock modpack"
    );
    ensure!(
        !base.data.server_clock,
        "compiled base pack already enables server clock"
    );

    let mut pack = base.clone();
    pack.data.server_clock = true;
    pack.report
        .manifests
        .push(REALTIME_CLOCK_MANIFEST_ID.to_string());
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
