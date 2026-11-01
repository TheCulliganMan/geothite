//! Add exact winter forest art with Gunflint's existing field-move behavior.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{CompiledGamePack, CompiledTilesetExtension};

const CONTRACT_SHA: &str = "7b36b1fd9b8b6e0ddf55f8b03355bd46cbb774c8391a89b300916ff9d26e3ee6";
const SOURCE: &str = "mn_winter_outdoor";
const TARGET: &str = "mn_winter_gunflint";
const BEHAVIOR: &str = "mn_gunflint";
const MANIFEST: &str = "minnesota-winter-gunflint-art-v1";

#[derive(Deserialize)]
struct Contract {
    schema_version: u16,
    id: String,
    source_tileset: String,
    target_tileset: String,
    behavior_source: String,
    source_definition_sha256: String,
    behavior_definition_sha256: String,
    behavior_replacements_sha256: String,
    source_files_sha256: BTreeMap<String, String>,
    expected_blocks: usize,
    expected_glyphs: usize,
}

/// Adds one exact copy of installed winter forest art without retargeting maps.
///
/// The new private alias inherits Gunflint's native Cut/Whirlpool replacement
/// tables. Existing Border Pines winter behavior remains unchanged. Requires
/// hash-bound source art/behavior and a valid derived actual-parent identity;
/// unrelated approved parent additions are preserved. Only one tileset, six
/// runtime files, inherited replacement entries, one manifest and identity may
/// change. This does not establish winter traversal or native visual acceptance.
///
/// # Errors
/// Rejects stale contracts, identities, source art/behavior, occupied targets,
/// invalid native assets or any mutation outside the declared additive scope.
/// The borrowed parent remains unchanged on both success and failure.
pub fn build_winter_gunflint_alias_pack(
    base: &CompiledGamePack,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        hash(contract_json.as_bytes()) == CONTRACT_SHA,
        "frozen Gunflint alias contract required"
    );
    let contract: Contract =
        serde_json::from_str(contract_json).context("winter Gunflint contract")?;
    ensure!(
        contract.schema_version == 1
            && contract.id == "MN_WINTER_GUNFLINT_ALIAS"
            && contract.source_tileset == SOURCE
            && contract.target_tileset == TARGET
            && contract.behavior_source == BEHAVIOR
            && contract.expected_blocks == 73
            && contract.expected_glyphs == 256
            && contract.source_files_sha256.len() == 6,
        "bounded native winter Gunflint alias required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity == identity(base)?,
        "winter Gunflint parent identity is stale"
    );
    let definition = base
        .data
        .tilesets
        .get(SOURCE)
        .context("winter forest source")?;
    ensure!(
        json_hash(definition)? == contract.source_definition_sha256
            && definition.collision.len() == contract.expected_blocks
            && definition.palette_map.len() == contract.expected_glyphs,
        "winter forest definition preimage differs"
    );
    ensure!(
        json_hash(
            base.data
                .tilesets
                .get(BEHAVIOR)
                .context("Gunflint behavior source")?
        )? == contract.behavior_definition_sha256
            && json_hash(&serde_json::json!({
                "cut": base.data.field_moves.cut.replacements.get(BEHAVIOR),
                "whirlpool": base.data.field_moves.whirlpool.replacements.get(BEHAVIOR),
            }))? == contract.behavior_replacements_sha256,
        "Gunflint native field-move preimage differs"
    );
    for (path, expected) in &contract.source_files_sha256 {
        ensure!(
            path.contains(SOURCE)
                && hash(
                    base.runtime_files
                        .get(path)
                        .context("source winter art file")?
                ) == *expected,
            "winter art preimage differs: {path}"
        );
        ensure!(
            !base.runtime_files.contains_key(&target_path(path)),
            "winter alias file occupied"
        );
    }
    ensure!(
        !base.data.tilesets.contains_key(TARGET)
            && !base.data.field_moves.cut.replacements.contains_key(TARGET)
            && !base
                .data
                .field_moves
                .whirlpool
                .replacements
                .contains_key(TARGET)
            && !base.report.manifests.iter().any(|id| id == MANIFEST),
        "winter alias target occupied"
    );
    let art = |path: &str| {
        base.runtime_files
            .get(path)
            .cloned()
            .context("frozen winter source file")
    };
    let mut output = base.with_tileset_extension(CompiledTilesetExtension {
        manifest_id: MANIFEST.into(),
        tileset_id: TARGET.into(),
        behavior_source_tileset_id: Some(BEHAVIOR.into()),
        definition: definition.clone(),
        metatiles: art(&format!("data/tilesets/{SOURCE}_metatiles.bin"))?,
        tile_graphics_2bpp: art(&format!("gfx/tilesets/{SOURCE}.2bpp"))?,
        tile_graphics_png: art(&format!("gfx/tilesets/{SOURCE}.png"))?,
    })?;
    output.runtime_files.insert(
        format!("data/tilesets/{TARGET}_foregrounds.json"),
        art(&format!("data/tilesets/{SOURCE}_foregrounds.json"))?,
    );
    output.identity = identity(&output)?;
    preserve(base, &output, &contract)?;
    crate::verify_compiled_game_pack_for_runtime(&output)?;
    Ok(output)
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn json_hash(value: &impl serde::Serialize) -> Result<String> {
    Ok(hash(&serde_json::to_vec(&serde_json::to_value(value)?)?))
}

fn identity(pack: &CompiledGamePack) -> Result<crate::CompiledGamePackIdentity> {
    crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )
}

fn target_path(source: &str) -> String {
    source.replace(SOURCE, TARGET)
}

fn remove_behavior<T: PartialEq>(
    base: &BTreeMap<String, T>,
    output: &mut BTreeMap<String, T>,
) -> Result<()> {
    ensure!(
        !base.contains_key(TARGET) && output.remove(TARGET).as_ref() == base.get(BEHAVIOR),
        "new alias field-move behavior differs"
    );
    Ok(())
}

fn preserve(base: &CompiledGamePack, output: &CompiledGamePack, contract: &Contract) -> Result<()> {
    let mut inverse = output.clone();
    ensure!(
        inverse.data.tilesets.remove(TARGET).as_ref() == base.data.tilesets.get(SOURCE),
        "new alias collision/palette differs from exact source"
    );
    for path in contract.source_files_sha256.keys() {
        ensure!(
            inverse.runtime_files.remove(&target_path(path)).as_ref()
                == base.runtime_files.get(path),
            "new alias art differs from exact source: {path}"
        );
    }
    remove_behavior(
        &base.data.field_moves.cut.replacements,
        &mut inverse.data.field_moves.cut.replacements,
    )?;
    remove_behavior(
        &base.data.field_moves.whirlpool.replacements,
        &mut inverse.data.field_moves.whirlpool.replacements,
    )?;
    let mut expected_report = base.report.clone();
    expected_report.manifests.push(MANIFEST.into());
    ensure!(
        output.report == expected_report,
        "winter alias changed unrelated report data"
    );
    inverse.report = base.report.clone();
    inverse.identity = base.identity.clone();
    ensure!(
        inverse == *base,
        "winter alias changed maps, scripts, art, audio or other parent content"
    );
    ensure!(
        output.identity == identity(output)?,
        "winter alias output identity is stale"
    );
    Ok(())
}

#[cfg(test)]
#[path = "winter_gunflint_alias_tests.rs"]
mod tests;
