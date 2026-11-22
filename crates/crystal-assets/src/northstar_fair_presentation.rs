//! Combine the reviewed Fair dialogue and trainer art with the public save lineage.

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::{CompiledGamePack, LoadedCompiledGamePack};

const POLICY: &str = "data/northstar-save-compatibility.json";
const RELEASE: &str = "northstar-fair-presentation-v1";

/// Builds both Fair presentation editions from the exact public pack pair and
/// the reviewed native trainer art pack.
///
/// # Errors
/// Rejects an unexpected input release, changes outside the trainer art bank,
/// divergent public save policies, or any edition that fails pack validation.
pub fn build_northstar_fair_presentation(
    fair_raw: &LoadedCompiledGamePack,
    fair_hosted: &LoadedCompiledGamePack,
    portraits: &CompiledGamePack,
    story: &str,
) -> Result<[CompiledGamePack; 2]> {
    let raw = fair_raw.save_modpack_identity()?;
    let hosted = fair_hosted.save_modpack_identity()?;
    ensure!(
        raw.hash() == "18003b91597173f1d98937a0f6cc2fa82cfd2c5b3612c9803cd5f615acad2a73"
            && hosted.hash() == "9fbfa371972f6b3cdbd9d0ca9689aa5a66ee43d1d17e7650982da88d4e57f58e",
        "expected the published Fair archives"
    );
    ensure!(
        portraits.identity()?.content_hash
            == "b3de9604cfa8c8a681e32847e9f84088b6393fae799423f8c4a80b1a2eb57b46"
            && portraits.data == fair_raw.pack().data,
        "expected the reviewed Fair full-cast artwork"
    );
    let fair_files = &fair_raw.pack().runtime_files;
    let inherited = fair_files.get(POLICY).context("Fair save policy")?;
    ensure!(
        fair_hosted.pack().runtime_files.get(POLICY) == Some(inherited),
        "public save policies differ"
    );
    let inherited: Value = serde_json::from_slice(inherited)?;
    let policy = serde_json::to_vec(&json!({
        "schema_version": 10,
        "sources": [
            { "modpack_id": raw.id(), "modpack_hash": raw.hash(), "content_hash": fair_raw.pack_content_hash()? },
            { "modpack_id": hosted.id(), "modpack_hash": hosted.hash(), "content_hash": fair_hosted.pack_content_hash()? },
        ],
        "inherited": inherited,
    }))?;
    let artwork: Vec<_> = portraits
        .runtime_files
        .iter()
        .filter(|(path, bytes)| fair_files.get(*path) != Some(*bytes))
        .collect();
    ensure!(
        artwork
            .iter()
            .all(|(path, _)| path.starts_with("gfx/trainers/"))
            && artwork
                .iter()
                .any(|(path, _)| *path == crate::TRAINER_PORTRAIT_OVERRIDES_PATH),
        "unexpected changes in the full-cast pack"
    );
    let build = |base: &CompiledGamePack| -> Result<CompiledGamePack> {
        let mut pack = crate::revise_northstar_dialogue_clarity(base, story)?;
        // The preview marker is replaced by the public migration target.
        pack.report
            .manifests
            .pop()
            .context("dialogue preview marker")?;
        pack.report.manifests.push(RELEASE.into());
        for (path, bytes) in &artwork {
            ensure!(
                pack.runtime_files.get(*path) == fair_files.get(*path),
                "portrait conflict at {path}"
            );
            pack.runtime_files.insert((*path).clone(), (*bytes).clone());
        }
        pack.runtime_files.insert(POLICY.into(), policy.clone());
        pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
            pack.format_version,
            &pack.data,
            &pack.audio_manifest,
            &pack.runtime_files,
            &pack.report,
        )?;
        crate::verify_compiled_game_pack_for_runtime(&pack)?;
        Ok(pack)
    };
    let raw = build(fair_raw.pack())?;
    let all = crate::build_all_pokemon_modpack(&raw)?;
    let battles = crate::build_default_battle_modpack(&all)?;
    let clock = crate::build_realtime_clock_modpack(&battles)?;
    let hosted = crate::build_player_customization_modpack(&clock)?;
    Ok([raw, hosted])
}
