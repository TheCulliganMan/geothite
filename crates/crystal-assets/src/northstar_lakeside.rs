//! Publish Brainerd's swimming beach, gym lessons, regional directions and reviewed portraits.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::path::Path;

use crate::{CompiledGamePack, LoadedCompiledGamePack};

pub(super) const MANIFEST: &str = "northstar-lakeside-v1";
const POLICY: &str = "data/northstar-save-compatibility.json";

/// Installs the Lakeside revision and declares its exact raw and hosted save parents.
///
/// # Errors
/// Returns an error if either parent is different from the released Crossings pack,
/// the authored maps, teams or art differ from their expected inputs, or pack validation fails.
pub fn revise_northstar_lakeside(
    raw: &LoadedCompiledGamePack,
    hosted: &LoadedCompiledGamePack,
    pokemon_root: &Path,
    native_root: &Path,
) -> Result<CompiledGamePack> {
    let raw_id = raw.save_modpack_identity()?;
    let hosted_id = hosted.save_modpack_identity()?;
    ensure!(
        raw_id.hash() == "caecd60747ee26510651509c6567a3490d919c0d6c9f8c2528d9445de2ad1599"
            && hosted_id.hash()
                == "966e3a8ac71190642addc67a010ffd4e59cb289401fc748ddd36594de4820ae3",
        "expected the released Crossings pair"
    );
    let mut pack = raw.pack().clone();
    crate::northstar_reviewed_portraits::install(&mut pack, pokemon_root, native_root)?;
    crate::northstar_brainerd_lakeside::install(&mut pack)?;
    crate::northstar_gym_lessons::install(&mut pack)?;
    crate::northstar_story_landmarks::install(&mut pack)?;
    let inherited: Value = serde_json::from_slice(
        pack.runtime_files
            .get(POLICY)
            .context("Crossings compatibility file")?,
    )?;
    ensure!(
        inherited["schema_version"] == 7
            && inherited["target_manifest"] == "northstar-crossings-v1",
        "expected Crossings save policy"
    );
    pack.report.manifests.push(MANIFEST.into());
    let id = pack.runtime_modpack_id()?;
    ensure!(
        id == format!("{}+{MANIFEST}", raw_id.id()),
        "Lakeside must directly extend the verified Crossings identity"
    );
    let policy = json!({
        "schema_version": 8, "target_manifest": MANIFEST,
        "target_modpack_ids": [id, format!("{id}+realtime-clock+player-customization")],
        "sources": [
            {"modpack_id":raw_id.id(), "modpack_hash":raw_id.hash(), "content_hash":raw.pack_content_hash()?},
            {"modpack_id":hosted_id.id(), "modpack_hash":hosted_id.hash(), "content_hash":hosted.pack_content_hash()?}
        ], "inherited": inherited
    });
    pack.runtime_files
        .insert(POLICY.into(), serde_json::to_vec(&policy)?);
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
