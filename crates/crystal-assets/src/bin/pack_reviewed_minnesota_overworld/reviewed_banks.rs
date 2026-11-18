//! Verify that the standalone sprite pack builder preserves unrelated content.
use anyhow::{Result, ensure};
use crystal_assets::CompiledGamePack;
use crystal_assets::minnesota_overworld_sprites::OverworldCompilation;

pub(super) fn verify_preservation(
    base: &CompiledGamePack,
    published: &CompiledGamePack,
    banks: &OverworldCompilation,
) -> Result<()> {
    ensure!(
        serde_json::to_vec(base.data())? == serde_json::to_vec(published.data())?,
        "game data changed during art publication"
    );
    ensure!(
        base.compiled_audio() == published.compiled_audio()
            && base.audio_manifest()? == published.audio_manifest()?,
        "audio changed during art publication"
    );
    let mut expected = base.runtime_files().clone();
    for (key, bytes) in &banks.runtime_files {
        if let Some(old) = expected.insert(key.clone(), bytes.clone()) {
            ensure!(old == *bytes, "existing asset conflict {key}");
        }
    }
    ensure!(
        &expected == published.runtime_files(),
        "unrelated runtime content changed"
    );
    Ok(())
}
