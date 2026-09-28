pub const TEXT_TUI_MANIFEST_ID: &str = "text-tui";

use anyhow::{Result, ensure};

use crate::{
    CompiledGamePack, derive_compiled_game_pack_identity_from_manifest,
    verify_compiled_game_pack_for_runtime,
};

/// Mark a verified game pack for use with the native text renderer.
///
/// The renderer consumes the normal typed runtime catalogs, so no game content
/// is duplicated or changed. The manifest entry gives the resulting pack its
/// own verified identity while preserving the base pack's storage format.
pub fn build_text_tui_modpack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base
            .report
            .manifests
            .iter()
            .any(|manifest| manifest == TEXT_TUI_MANIFEST_ID),
        "compiled pack already includes the text TUI modpack"
    );

    let mut pack = base.clone();
    pack.report.manifests.push(TEXT_TUI_MANIFEST_ID.to_string());
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
