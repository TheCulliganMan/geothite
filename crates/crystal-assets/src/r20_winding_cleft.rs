//! Rebuild the two-mouth Voyageurs cleft without changing its exterior ports.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::authored_route_design::Interior;
use crate::grand_marais_modpack::authoring;

const ROOM: &str = "MnR20Cave1F";
const OPPONENT: &str = "MN_EXTRA_R20_WAYFINDER";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Contract {
    schema_version: u8,
    map: String,
    native_module_sha256: String,
    preserved_warps: Value,
    purpose: String,
}

/// Rebuilds the complete Voyageurs cleft with two relocated interior mouths,
/// native stairways, a westward drop and one optional regional opponent.
/// Both exterior maps/warp indices, original actors, reward, programs, native
/// identities, art, audio and all other maps remain unchanged.
///
/// # Errors
/// Rejects stale native/source bindings, changed exterior pairings or original
/// content, unsafe terrain/actor placement, invalid regional teams and bad packs.
pub fn rebuild_r20_winding_cleft(
    base: &CompiledGamePack,
    before: &str,
    after: &str,
    contract: &str,
    trainers: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let old: Interior = serde_json::from_str(before)?;
    let old_value: Value = serde_json::from_str(before)?;
    let mut design: Value = serde_json::from_str(after)?;
    let contract: Contract = serde_json::from_str(contract)?;
    let original = base
        .data
        .maps
        .get(ROOM)
        .context("installed Voyageurs cleft")?;
    ensure!(
        contract.schema_version == 1
            && contract.map == ROOM
            && !contract.purpose.is_empty()
            && contract.native_module_sha256
                == format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(&serde_json::to_value(original)?)?)
                )
            && contract.preserved_warps == serde_json::to_value(&original.events.warps)?
            && original.attributes.tileset_name == "cave"
            && old.id == ROOM
            && old.dimensions_metatiles == [9, 8]
            && original.blocks == crate::authored_cave_revision::blocks(&old)?
            && original.events.warps.len() == 2
            && original.events.coord_events.is_empty()
            && original.events.bg_events.is_empty(),
        "complete Voyageurs cleft preimage differs"
    );
    for field in [
        "id",
        "template",
        "exit_doors",
        "exit_collision",
        "border_block",
        "residents",
        "items",
        "signs",
        "mandatory",
    ] {
        ensure!(
            old_value[field] == design[field],
            "retained cleft field differs: {field}"
        );
    }
    ensure!(
        design["dimensions_metatiles"] == json!([14, 12])
            && design["exits"] == json!([[5, 21], [23, 21]])
            && design["clear_arrival"] == json!([5, 20])
            && design["proposed_actors"]
                == json!([{"id":OPPONENT,"sprite":"SPRITE_COOLTRAINER_M","tile":[21,17]}]),
        "complete authored cleft and two interior mouth positions required"
    );
    let placements: Value = serde_json::from_str(trainers)?;
    let additions = placements["trainers"]
        .as_array()
        .context("cleft wayfinder")?;
    ensure!(
        placements["schema_version"] == 1
            && additions.len() == 1
            && additions[0]["map"] == ROOM
            && additions[0]["id"] == OPPONENT
            && additions[0]["tile"] == json!([21, 17])
            && additions[0]["sprite"] == "SPRITE_COOLTRAINER_M"
            && additions[0]["optional_battle"] == true,
        "one optional native cleft wayfinder required"
    );
    let terrain: Interior = serde_json::from_value(design.clone())?;
    let mut room = original.clone();
    room.blocks = crate::authored_cave_revision::blocks(&terrain)?;
    room.attributes.width = 14;
    room.attributes.height = 12;
    for (warp, [x, y]) in room.events.warps.iter_mut().zip([[5, 21], [23, 21]]) {
        warp.x = x;
        warp.y = y;
    }
    authoring::refresh_event_section(&mut room);
    let mut pack = base.clone();
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
        "minnesota-r20-winding-cleft-v1",
        base.data.minnesota_trainer_scaling,
    )?;
    // The scout is installed by the interior-life overlay rather than the old
    // terrain source. Certify all actual bodies in the geometry projection.
    let residents = design["residents"]
        .as_array_mut()
        .context("cleft residents")?;
    for actor in &original.objects {
        if actor.script != "MN_R20_ITEM_01" {
            residents.push(json!({"id":actor.script,"sprite":actor.sprite,
                "tile":[actor.x,actor.y],"facing":"south","lines":["Retain native dialogue."]}));
        }
    }
    let trainer = &additions[0];
    residents.push(json!({"id":trainer["id"],"sprite":trainer["sprite"],
        "tile":trainer["tile"],"facing":trainer["facing"],"lines":trainer["before_lines"]}));
    let geometry: Interior = serde_json::from_value(design)?;
    let revised = &pack.data.maps[ROOM];
    crate::authored_route_modpack::validate_room(&pack, revised, &geometry)?;
    crate::authored_cave_revision::validate_terrain(&pack, revised, &geometry)?;
    let mut expected_events = original.events.clone();
    for (warp, [x, y]) in expected_events.warps.iter_mut().zip([[5, 21], [23, 21]]) {
        warp.x = x;
        warp.y = y;
    }
    ensure!(
        revised.events == expected_events,
        "cleft exterior pairing changed"
    );
    ensure!(
        revised.objects.starts_with(&original.objects),
        "original cleft bodies changed"
    );
    for (label, body) in &original.scripts {
        ensure!(
            revised.scripts.get(label) == Some(body),
            "original cleft program changed: {label}"
        );
    }
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    #[ignore = "requires CRYSTAL_R20_CLEFT_PARENT and authored sources at runtime"]
    fn winding_cleft_keeps_exterior_pairings_and_rejects_stale_unsafe_or_foreign_content() {
        let parent = std::env::var_os("CRYSTAL_R20_CLEFT_PARENT").expect("current native parent");
        let base = crate::read_verified_compiled_game_pack(PathBuf::from(parent)).expect("parent");
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../planning/minnesota/caves/r20-winding-cleft-v01");
        let read = |name: &str| std::fs::read_to_string(root.join(name)).expect("cleft source");
        let before = read("before.json");
        let after = read("after.json");
        let contract = read("contract.json");
        let trainers = read("trainers.json");
        let build = |after: &str, contract: &str, trainers: &str| {
            super::rebuild_r20_winding_cleft(&base, &before, after, contract, trainers)
        };
        let revised = build(&after, &contract, &trainers).expect("native winding cleft");
        for (name, map) in &base.data.maps {
            if name != super::ROOM {
                assert_eq!(
                    serde_json::to_value(map).expect("old"),
                    serde_json::to_value(&revised.data.maps[name]).expect("new"),
                    "{name}"
                );
            }
        }
        assert_eq!(base.runtime_files, revised.runtime_files);
        assert_eq!(base.audio_manifest, revised.audio_manifest);
        let mut stale: serde_json::Value = serde_json::from_str(&contract).expect("contract");
        stale["native_module_sha256"] = "0".into();
        assert!(build(&after, &stale.to_string(), &trainers).is_err());
        let mut broken: serde_json::Value = serde_json::from_str(&after).expect("terrain");
        broken["exits"][1] = serde_json::json!([5, 21]);
        assert!(build(&broken.to_string(), &contract, &trainers).is_err());
        let foreign = trainers.replace("WHIRLNUT", "PIDGEOT");
        assert!(build(&after, &contract, &foreign).is_err());
    }
}
