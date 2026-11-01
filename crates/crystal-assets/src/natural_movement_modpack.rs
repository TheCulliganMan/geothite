//! Enable continuous controller locomotion without changing content or audio.
use crate::{
    CompiledGamePack, derive_compiled_game_pack_identity_from_manifest,
    verify_compiled_game_pack_for_runtime,
};
use anyhow::{Result, ensure};

pub const NATURAL_MOVEMENT_MANIFEST_ID: &str = "natural-movement";

pub fn build_natural_movement_modpack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base
            .report
            .manifests
            .iter()
            .any(|id| id == NATURAL_MOVEMENT_MANIFEST_ID),
        "compiled pack already includes natural movement"
    );
    let mut pack = base.clone();
    pack.data.natural_movement = Some(true);
    pack.report
        .manifests
        .push(NATURAL_MOVEMENT_MANIFEST_ID.into());
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn continuous_step_commands_preserve_legacy_record_shape() {
        // Use the production enum serializer so this test also covers the
        // actual button and divider representations used by old recordings.
        let command = crate::RuntimeOverworldInputCommand {
            buttons: vec![crystal_core::input::buttons::GameButton::Right],
            continuous_step: false,
            divider_trace: crate::RuntimeDividerTrace::new([]),
        };
        let value = serde_json::to_value(&command).unwrap();
        assert!(value.get("continuous_step").is_none());
        let decoded: crate::RuntimeOverworldInputCommand = serde_json::from_value(value).unwrap();
        assert!(!decoded.continuous_step);
        let mut continuous = command;
        continuous.continuous_step = true;
        let value = serde_json::to_value(continuous).unwrap();
        assert_eq!(value["continuous_step"], true);
    }
    #[test]
    fn natural_movement_pack_preserves_content_and_audio() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../content-packs/text-tui.crystalpack")
            .canonicalize()
            .expect("external pack path");
        let base = crate::read_verified_compiled_game_pack(&path).expect("external pack");
        let pack = build_natural_movement_modpack(&base).unwrap();
        assert_eq!(pack.audio_manifest, base.audio_manifest);
        assert_eq!(pack.runtime_files, base.runtime_files);
        assert_eq!(pack.compiled_audio, base.compiled_audio);
        assert_eq!(pack.audio_compression, base.audio_compression);
        let mut data = pack.data.clone();
        data.natural_movement = base.data.natural_movement;
        assert_eq!(data, base.data);
        assert_ne!(pack.identity, base.identity);
        assert!(build_natural_movement_modpack(&pack).is_err());
    }
}
