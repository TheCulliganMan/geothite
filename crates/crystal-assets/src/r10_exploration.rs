//! Replace the survey cave's terrain and add its authored optional encounters.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::authored_route_design::Interior;
use crate::grand_marais_modpack::authoring;

const ROOM: &str = "MnR10Cave1F";
const CACHE: &str = "MN_R10_ESCAPE_ROPE";
const FLAG: &str = "EVENT_MN_R10_ESCAPE_ROPE";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Contract {
    schema_version: u8,
    map: String,
    native_module_sha256: String,
    design: String,
    preserved_warps: Value,
}

/// Rebuilds the installed R10 survey cave around its retained exterior mouth,
/// reader and Revive. Adds one Escape Rope and two optional regional opponents.
/// Native art, all other maps, existing programs and outside connections remain.
///
/// # Errors
/// Rejects a stale full module/source binding, changed retained events, unsafe
/// floor/ledge geometry, overlapping actors, invalid regional teams or pack data.
pub fn rebuild_r10_exploration(
    base: &CompiledGamePack,
    before: &str,
    after: &str,
    contract: &str,
    trainers: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let old: Interior = serde_json::from_str(before)?;
    let mut design: Value = serde_json::from_str(after)?;
    let contract: Contract = serde_json::from_str(contract)?;
    let original = base.data.maps.get(ROOM).context("installed survey cave")?;
    ensure!(
        contract.schema_version == 1
            && contract.map == ROOM
            && !contract.design.is_empty()
            && contract.native_module_sha256
                == format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(&serde_json::to_value(original)?)?)
                )
            && contract.preserved_warps == serde_json::to_value(&original.events.warps)?
            && old.id == ROOM
            && old.dimensions_metatiles == [18, 14]
            && original.blocks == crate::authored_cave_revision::blocks(&old)?,
        "survey cave full preimage differs"
    );
    let old_value: Value = serde_json::from_str(before)?;
    for field in [
        "id",
        "template",
        "exits",
        "clear_arrival",
        "exit_collision",
        "border_block",
        "residents",
        "items",
        "signs",
    ] {
        ensure!(
            old_value[field] == design[field],
            "retained survey field differs: {field}"
        );
    }
    ensure!(
        design["dimensions_metatiles"] == json!([24, 18])
            && design["proposed_items"]
                == json!([{
                    "id":CACHE,"item":"ESCAPE_ROPE","quantity":1,"tile":[39,5],"flag":FLAG
                }]),
        "complete exploration extent and declared supply cache required"
    );
    let placements: Value = serde_json::from_str(trainers)?;
    let additions = placements["trainers"]
        .as_array()
        .context("two survey opponents")?;
    ensure!(
        placements["schema_version"] == 1 && additions.len() == 2,
        "two authored opponents required"
    );
    for (placement, (id, tile)) in additions.iter().zip([
        ("MN_EXTRA_R10_SURVEY", [18, 15]),
        ("MN_EXTRA_R10_SPRING", [33, 24]),
    ]) {
        ensure!(
            placement["map"] == ROOM
                && placement["id"] == id
                && placement["tile"] == json!(tile)
                && placement["optional_battle"] == true,
            "declared optional survey opponent differs"
        );
    }
    let mut room = original.clone();
    let terrain: Interior = serde_json::from_value(design.clone())?;
    room.blocks = crate::authored_cave_revision::blocks(&terrain)?;
    room.attributes.width = 24;
    room.attributes.height = 18;
    let mut pack = base.clone();
    crate::northstar_lair_authoring::flag(&mut pack, FLAG)?;
    crate::northstar_lair_authoring::actor(&pack, &mut room, CACHE, "SPRITE_POKE_BALL", [39, 5])?;
    let item = room.objects.last_mut().context("survey supply cache")?;
    item.object_type = "OBJECTTYPE_ITEMBALL".into();
    item.sprite_has_facings = false;
    item.event_flag = FLAG.into();
    ensure!(
        room.scripts
            .insert(
                CACHE.into(),
                json!([authoring::command("itemball", &["ESCAPE_ROPE", "1"])])
            )
            .is_none(),
        "occupied survey cache script"
    );
    room.script_field_pickups =
        crate::parse_script_field_pickups(ROOM, &room.scripts, &room.objects)?;
    authoring::refresh_event_section(&mut room);
    crate::authored_town_publication::replace(&mut pack, room)?;
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    let pack = crate::minnesota_trainers::build_trainers(
        &pack,
        trainers,
        "minnesota-r10-exploration-trainers-v1",
        base.data.minnesota_trainer_scaling,
    )?;
    // Geometry uses every real body, including optional challengers. Keep the
    // exact original reader/item programs while validating their new routes.
    for placement in additions {
        design["residents"]
            .as_array_mut()
            .context("survey residents")?
            .push(json!({
                "id":placement["id"],"sprite":placement["sprite"],"tile":placement["tile"],
                "facing":placement["facing"],"lines":placement["before_lines"]
            }));
    }
    let cache_source = design["proposed_items"][0].clone();
    design["items"]
        .as_array_mut()
        .context("survey items")?
        .push(cache_source);
    let geometry: Interior = serde_json::from_value(design)?;
    let revised = &pack.data.maps[ROOM];
    crate::authored_route_modpack::validate_room(&pack, revised, &geometry)?;
    crate::authored_cave_revision::validate_terrain(&pack, revised, &geometry)?;
    ensure!(
        revised.events == original.events,
        "survey mouth/events changed"
    );
    for (label, body) in &original.scripts {
        ensure!(
            revised.scripts.get(label) == Some(body),
            "original survey program changed: {label}"
        );
    }
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    #[ignore = "requires CRYSTAL_R10_EXPLORATION_PARENT and the authored survey sources"]
    fn exploration_keeps_the_mouth_and_rejects_stale_unsafe_or_foreign_content() {
        let parent = std::env::var_os("CRYSTAL_R10_EXPLORATION_PARENT").expect("current parent");
        let base =
            crate::read_verified_compiled_game_pack(PathBuf::from(parent)).expect("native parent");
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../planning/minnesota/caves/r10-exploration-v01");
        let read =
            |name: &str| std::fs::read_to_string(root.join(name)).expect("authored cave source");
        let before = read("before.json");
        let after = read("after.json");
        let contract = read("contract.json");
        let trainers = read("trainers.json");
        let revised = super::rebuild_r10_exploration(&base, &before, &after, &contract, &trainers)
            .expect("complete native exploration cave");
        assert_eq!(base.data.maps.len(), revised.data.maps.len());
        for (name, map) in &base.data.maps {
            if name != super::ROOM {
                assert!(map == &revised.data.maps[name], "unowned map {name}");
            }
        }
        assert!(base.runtime_files == revised.runtime_files);
        assert!(base.compiled_audio == revised.compiled_audio);
        assert!(base.audio_manifest == revised.audio_manifest);
        let mut bad: serde_json::Value = serde_json::from_str(&contract).expect("contract");
        bad["native_module_sha256"] = serde_json::json!("stale");
        assert!(
            super::rebuild_r10_exploration(&base, &before, &after, &bad.to_string(), &trainers)
                .is_err()
        );
        let mut bad: serde_json::Value = serde_json::from_str(&after).expect("layout");
        bad["proposed_items"][0]["quantity"] = serde_json::json!(99);
        assert!(
            super::rebuild_r10_exploration(&base, &before, &bad.to_string(), &contract, &trainers)
                .is_err()
        );
        let mut bad: serde_json::Value = serde_json::from_str(&trainers).expect("trainers");
        bad["trainers"][0]["party"][0]["species"] = serde_json::json!("GEODUDE");
        assert!(
            super::rebuild_r10_exploration(&base, &before, &after, &contract, &bad.to_string())
                .is_err()
        );
    }
}
