//! Build the Central Lakes, Former Shores and Hall of Fame revision of Minnesota.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::{CompiledGamePack, LoadedCompiledGamePack};

const MANIFEST: &str = "northstar-crossings-v1";
const POLICY: &str = "data/northstar-save-compatibility.json";

/// Installs the Crossings revision and declares its exact raw and hosted save parents.
///
/// # Errors
/// Returns an error if either parent is different from the released Wayfinding pack,
/// the authored map or field-move preconditions fail, or the produced pack is invalid.
pub fn revise_northstar_crossings(
    raw: &LoadedCompiledGamePack,
    hosted: &LoadedCompiledGamePack,
) -> Result<CompiledGamePack> {
    let raw_id = raw.save_modpack_identity()?;
    let hosted_id = hosted.save_modpack_identity()?;
    ensure!(
        raw_id.hash() == "e0638a9327e3296b214cdb8d3fdb6e25c33616cdacd2adac583bdd4eb70958bd"
            && hosted_id.hash()
                == "e17218b24ad2b8292d17658b1176e7e093851f7502b2b5a6177f67edf27d97c4",
        "expected the released Wayfinding pair"
    );
    let mut pack = raw.pack().clone();
    crate::northstar_crossings_rocks::install(&mut pack)?;
    crate::northstar_shiner_narrows::install(&mut pack)?;
    crate::northstar_ely_register::install(&mut pack)?;
    crate::northstar_former_shores::install(&mut pack)?;
    crate::northstar_gallery_strength::install(&mut pack)?;
    let inherited: Value = serde_json::from_slice(
        pack.runtime_files
            .get(POLICY)
            .context("Wayfinding compatibility file")?,
    )?;
    ensure!(
        inherited["schema_version"] == 6
            && inherited["target_manifest"] == "northstar-wayfinding-v1",
        "expected Wayfinding save policy"
    );
    pack.report.manifests.push(MANIFEST.into());
    let id = pack.runtime_modpack_id()?;
    let policy = json!({
        "schema_version": 7, "target_manifest": MANIFEST,
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
