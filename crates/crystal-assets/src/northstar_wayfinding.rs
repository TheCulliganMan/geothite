//! Native Minnesota cave junctions, underground waterways and trail discoveries.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::{CompiledGamePack, LoadedCompiledGamePack};

const MANIFEST: &str = "northstar-wayfinding-v1";
const POLICY: &str = "data/northstar-save-compatibility.json";

/// Installs the wayfinding revision on the released regional design pack.
///
/// # Errors
/// Returns an error if either parent differs from the released pair, an edit
/// fails its terrain or event checks, or the resulting game data is invalid.
pub fn revise_northstar_wayfinding(
    raw: &LoadedCompiledGamePack,
    hosted: &LoadedCompiledGamePack,
) -> Result<CompiledGamePack> {
    let raw_id = raw.save_modpack_identity()?;
    let hosted_id = hosted.save_modpack_identity()?;
    ensure!(
        raw_id.hash() == "ea89ae2a3df712b16018c3f7d89f96cfef5486f24778476cff5f7df490af74dd"
            && hosted_id.hash()
                == "19caa20f4616a72a378f757cbef7052c5bf2a91d079782e267a7b43f6b91a64e",
        "expected the exact released Minnesota regional design pair"
    );
    let mut pack = raw.pack().clone();
    crate::northstar_carrier_ledge::install_fossil_bluff(&mut pack)?;
    crate::northstar_mine_landmarks::install(&mut pack)?;
    crate::northstar_small_water_caves::install(&mut pack)?;
    crate::northstar_trail_caches::install(&mut pack)?;
    crate::northstar_cargo_lookouts::install(&mut pack)?;
    crate::northstar_route_passages::install(&mut pack)?;
    crate::northstar_mille_lacs_erratic::install(&mut pack)?;
    crate::northstar_maze_passages::install(&mut pack)?;
    crate::northstar_northern_rockfall::install(&mut pack)?;
    crate::northstar_park_passage::install(&mut pack)?;
    crate::northstar_gallery_tm::install(&mut pack)?;

    let inherited: Value = serde_json::from_slice(
        pack.runtime_files
            .get(POLICY)
            .context("regional design save policy")?,
    )?;
    ensure!(
        inherited["schema_version"] == 5
            && inherited["target_manifest"] == "northstar-regional-design-v1",
        "unexpected parent save policy"
    );
    pack.report.manifests.push(MANIFEST.into());
    let id = pack.runtime_modpack_id()?;
    let policy = json!({
        "schema_version": 6, "target_manifest": MANIFEST,
        "target_modpack_ids": [id, format!("{id}+realtime-clock+player-customization")],
        "sources": [
            {"modpack_id":raw_id.id(), "modpack_hash":raw_id.hash(), "content_hash":raw.pack_content_hash()?},
            {"modpack_id":hosted_id.id(), "modpack_hash":hosted_id.hash(), "content_hash":hosted.pack_content_hash()?}
        ],
        "inherited": inherited
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
