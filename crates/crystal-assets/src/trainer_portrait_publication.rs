//! Add isolated Minnesota battle artwork without reclassifying trainers.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_core::models::TrainerCatalog;
use serde::Deserialize;

use crate::trainer_battle_native::{self, NativeBattlePortrait};
use crate::trainer_portrait_overrides::{TrainerPortraitOverride, TrainerPortraitOverrides};
use crate::{CompiledGamePack, CompiledGamePackIdentity, TRAINER_PORTRAIT_OVERRIDES_PATH};

const SAVE_POLICY: &str = "data/northstar-save-compatibility.json";
const PREVIEW_PREFIX: &str = "northstar-trainer-portraits-v3-";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    schema_version: u8,
    expected_parent_content_hash: Option<String>,
    portraits: Vec<Portrait>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Portrait {
    source: PathBuf,
    trainer_class: String,
    trainer_ids: Vec<String>,
    expected_art: ExpectedArt,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedArt {
    id: String,
    png_sha256: String,
    tiles_sha256: String,
    palette_sha256: String,
}

/// Add battle-only resources and exact trainer-ID overrides to a fresh pack.
///
/// The version-one JSON plan names native `source` paths, `trainer_class`,
/// `trainer_ids`, and `expected_art` with the current class resource `id` and
/// `png_sha256`, `tiles_sha256`, `palette_sha256`. Sources are resolved from the
/// working directory. A plan may pin its intermediate parent with
/// `expected_parent_content_hash`. Each distinct portrait edition has its own
/// save identity. Its parent keeps its existing saves and migration policy.
///
/// # Errors
/// Rejects stale stock art, wrong/missing/duplicate/non-Minnesota trainer IDs,
/// class mismatches, existing bindings/art collisions, invalid sources or packs.
pub fn add_minnesota_trainer_portrait_overrides(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let plan: Plan = serde_json::from_str(source).context("trainer portrait publication plan")?;
    if let Some(expected) = &plan.expected_parent_content_hash {
        ensure!(
            expected == &base.identity.content_hash,
            "portrait plan requires a different parent pack"
        );
    }
    let mut pack = base.clone();
    pack.runtime_files = publication_files(&base.runtime_files, &base.data.trainers, plan)?;
    // Hash the assembled artwork with its parent first. Rebuilding identical
    // art reuses the same slot, while a new image or binding gets a fresh one.
    let artwork = portrait_identity(&pack)?;
    pack.report
        .manifests
        .push(format!("{PREVIEW_PREFIX}{}", artwork.content_hash));
    pack.runtime_files.remove(SAVE_POLICY);
    pack.identity = portrait_identity(&pack)?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn portrait_identity(pack: &CompiledGamePack) -> Result<CompiledGamePackIdentity> {
    if pack.audio_compression.is_some() {
        crate::derive_compiled_game_pack_identity_from_manifest(
            pack.format_version,
            &pack.data,
            &pack.audio_manifest,
            &pack.runtime_files,
            &pack.report,
        )
    } else {
        crate::derive_compiled_game_pack_identity(
            pack.format_version,
            &pack.data,
            &pack.compiled_audio,
            &pack.runtime_files,
            &pack.report,
        )
    }
}

fn publication_files(
    installed: &BTreeMap<String, Vec<u8>>,
    catalog: &TrainerCatalog,
    plan: Plan,
) -> Result<BTreeMap<String, Vec<u8>>> {
    ensure!(
        plan.schema_version == 1 && !plan.portraits.is_empty(),
        "nonempty version-one portrait plan required"
    );
    let mut bindings = TrainerPortraitOverrides::from_runtime_files(installed, catalog)?;
    let mut seen = BTreeSet::new();
    let mut files = installed.clone();
    for portrait in plan.portraits {
        ensure!(
            !portrait.trainer_ids.is_empty(),
            "portrait requires trainer IDs"
        );
        ensure!(
            portrait.expected_art.id == crate::trainer_class_portrait(&portrait.trainer_class),
            "expected art must match the original class resource"
        );
        for (extension, expected) in [
            ("png", &portrait.expected_art.png_sha256),
            ("2bpp", &portrait.expected_art.tiles_sha256),
            ("gbcpal", &portrait.expected_art.palette_sha256),
        ] {
            let path = format!("gfx/trainers/{}.{extension}", portrait.expected_art.id);
            let bytes = installed
                .get(&path)
                .with_context(|| format!("missing stock portrait {path}"))?;
            ensure!(
                sha256(bytes) == *expected,
                "stale stock portrait preimage: {path}"
            );
        }
        let native: NativeBattlePortrait = serde_json::from_slice(
            &fs::read(&portrait.source).with_context(|| portrait.source.display().to_string())?,
        )?;
        ensure!(
            seen.insert(native.id.clone()),
            "duplicate new portrait identity"
        );
        let encoded = trainer_battle_native::encode(&native)?;
        ensure!(
            encoded.keys().all(|key| !installed.contains_key(key)),
            "new portrait resource already exists"
        );
        for id in portrait.trainer_ids {
            ensure!(
                id.starts_with("MN_"),
                "only explicit Minnesota trainer IDs may receive regional art"
            );
            let trainer = catalog
                .get(&id)
                .with_context(|| format!("unknown portrait trainer {id}"))?;
            ensure!(
                trainer.trainer_class == portrait.trainer_class,
                "portrait trainer class mismatch: {id}"
            );
            ensure!(
                !bindings.trainers.contains_key(&id),
                "duplicate or existing portrait trainer binding: {id}"
            );
            bindings.trainers.insert(
                id,
                TrainerPortraitOverride {
                    trainer_class: portrait.trainer_class.clone(),
                    portrait: native.id.clone(),
                },
            );
        }
        files.extend(encoded);
    }
    files.insert(TRAINER_PORTRAIT_OVERRIDES_PATH.into(), bindings.to_json()?);
    Ok(files)
}

fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "trainer_portrait_publication_tests.rs"]
mod tests;
