//! Pine-framed mainland clearings and a useful grassy cabin link.
use anyhow::{Context, Result, ensure};
use crystal_core::world::encounters::WildEncounterZone;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;

#[path = "leech_lake_mainland_geometry.rs"]
mod geometry;
#[path = "leech_lake_mainland_scripts.rs"]
mod scripts;

const LAKE: &str = "MnLeechLake";
const TRAINER: &str = "MN_EXTRA_LEECH_MAINLAND_FLY_TIER";
const ITEM: &str = "MN_LEECH_MAINLAND_REPEL";
const CLAIMED: &str = "EVENT_MN_LEECH_MAINLAND_REPEL";
const MARKER: &str = "minnesota-leech-mainland-pines-v1";

#[cfg(test)]
#[path = "leech_lake_mainland_tests.rs"]
mod tests;

#[derive(Deserialize)]
struct Mainland {
    schema_version: u8,
    id: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    rows: Vec<String>,
    trainer_source: Value,
    mainland_item: Value,
    grass_zone: WildEncounterZone,
    walker_hint: Vec<String>,
}

/// Frames the original Leech Lake cabins with pines and adds a shorter grassy
/// cabin link, an optional regional fly-tier battle and a once-only Super Repel.
/// Preserves the island, current, original services, ports, art and audio.
///
/// # Errors
/// Rejects stale complete-map inputs, altered native anchors, unsafe traversal,
/// bypassed habitat, occupied identities, foreign wildlife or invalid packs.
pub fn rebuild_leech_lake_mainland(
    base: &CompiledGamePack,
    source_json: &str,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let source: Mainland = serde_json::from_str(source_json)?;
    let contract: Value = serde_json::from_str(contract_json)?;
    let original = base.data.maps.get(LAKE).context("current Leech Lake")?;
    ensure!(
        source.schema_version == 1
            && source.id == LAKE
            && source.tileset == "mn_headwaters"
            && source.dimensions_metatiles == [32, 24]
            && contract["schema_version"] == 1
            && contract["map"] == LAKE
            && contract["native_module_sha256"]
                == format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(&serde_json::to_value(original)?)?)
                )
            && contract["preserved_warps"] == serde_json::to_value(&original.events.warps)?
            && contract["preserved_coordinate_events"]
                == serde_json::to_value(&original.events.coord_events)?
            && !base.report.manifests.iter().any(|m| m == MARKER),
        "fresh mainland source and complete current town preimage required"
    );
    let trainers = source.trainer_source["trainers"]
        .as_array()
        .context("one native fly-tier")?;
    ensure!(
        trainers.len() == 1
            && trainers[0]["id"] == TRAINER
            && trainers[0]["map"] == LAKE
            && trainers[0]["tile"] == json!([19, 24])
            && trainers[0]["approach"] == json!([18, 24])
            && trainers[0]["optional_battle"] == true,
        "reviewed optional fly-tier placement required"
    );
    ensure!(
        source.mainland_item
            == json!({
                "id": ITEM, "tile": [19, 26], "item": "SUPER_REPEL",
                "quantity": 1, "flag": CLAIMED
            }),
        "reviewed once-only mainland reward required"
    );
    let mut pack = base.clone();
    let mut lake = original.clone();
    authoring::blocks(&mut lake, 32, 24, &source.rows.join("\n"))?;
    geometry::validate(base, original, &lake)?;
    scripts::discoveries(&mut pack, &mut lake, &source.walker_hint)?;
    crate::authored_town_publication::replace(&mut pack, lake)?;
    scripts::wildlife(&mut pack, source.grass_zone)?;
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    let before_trainer = pack.data.maps[LAKE].clone();
    pack = crate::minnesota_trainers::build_trainers(
        &pack,
        &serde_json::to_string(&source.trainer_source)?,
        MARKER,
        base.data.minnesota_trainer_scaling,
    )?;
    let lake = scripts::preserve_programs(&before_trainer, &pack.data.maps[LAKE])?;
    geometry::validate(base, original, &lake)?;
    crate::authored_town_publication::replace(&mut pack, lake)?;
    for (id, map) in &base.data.maps {
        if id != LAKE {
            ensure!(
                pack.data.maps.get(id) == Some(map),
                "unrelated map changed: {id}"
            );
        }
    }
    ensure!(
        pack.runtime_files == base.runtime_files
            && pack.audio_manifest == base.audio_manifest
            && pack.compiled_audio == base.compiled_audio,
        "original native art, animation and audio must remain exact"
    );
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
