//! Solid native rock and one optional cache in the installed iron crosscut.
use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::authored_town_geometry::kind;
use crate::grand_marais_modpack::authoring;
use crate::grand_marais_modpack::authoring::command;

const ROOM: &str = "MnIronDrillApproach";
const CACHE: &str = "MN_IRON_APPROACH_ETHER";
const FLAG: &str = "EVENT_MN_IRON_APPROACH_ETHER";

#[derive(Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Terrain {
    schema_version: u8,
    id: String,
    dimensions_metatiles: [u16; 2],
    tileset: String,
    palette_group: String,
    border_block: u8,
    rows: Vec<String>,
}

impl Terrain {
    fn blocks(&self) -> Result<Vec<u16>> {
        ensure!(self.rows.len() == 13, "complete crosscut rows required");
        let mut blocks = Vec::new();
        for row in &self.rows {
            let cells: Vec<_> = row.split_whitespace().collect();
            ensure!(cells.len() == 16, "complete crosscut columns required");
            for cell in cells {
                blocks.push(u16::from_str_radix(cell, 16).context("native cave block")?);
            }
        }
        Ok(blocks)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Contract {
    schema_version: u8,
    map: String,
    native_module_sha256: String,
    reward: Reward,
    preserved_warps: Value,
    native_actor_anchors: Value,
    purpose: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reward {
    id: String,
    flag: String,
    tile: [u16; 2],
    item: String,
    quantity: u16,
}

/// Fills the crosscut's black wall cells with existing solid cave rock and adds
/// an optional Ether to its existing upper side loop. Both stair pairs and all
/// original collision quadrants, actors, programs, teams and palettes remain.
///
/// # Errors
/// Rejects stale modules, altered walking geometry or native identities, unsafe
/// cache placement, occupied script/flag names, or an invalid resulting pack.
pub fn revise_iron_drill_approach(
    base: &CompiledGamePack,
    before: &str,
    after: &str,
    contract: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let before: Terrain = serde_json::from_str(before)?;
    let mut after: Terrain = serde_json::from_str(after)?;
    let contract: Contract = serde_json::from_str(contract)?;
    let original = base
        .data
        .maps
        .get(ROOM)
        .context("installed iron crosscut")?;
    let canonical = serde_json::to_vec(&serde_json::to_value(original)?)?;
    ensure!(
        contract.schema_version == 1
            && contract.map == ROOM
            && !contract.purpose.is_empty()
            && contract.native_module_sha256 == format!("{:x}", Sha256::digest(canonical))
            && before.schema_version == 1
            && before.id == ROOM
            && before.dimensions_metatiles == [16, 13]
            && before.tileset == "cave"
            && before.palette_group == "nite"
            && before.border_block == 1
            && original.attributes.tileset_name == "cave"
            && original.attributes.time_of_day.as_deref() == Some("nite")
            && original.attributes.border_block == 1
            && before.blocks()? == original.blocks,
        "crosscut module/source preimage differs"
    );
    let replacement = after.blocks()?;
    ensure!(
        after.border_block == 0x2e,
        "solid native cave border required"
    );
    after.border_block = before.border_block;
    after.rows.clone_from(&before.rows);
    ensure!(before == after, "retained crosscut source fields changed");
    ensure!(
        replacement
            == original
                .blocks
                .iter()
                .map(|&b| if b == 1 { 0x2e } else { b })
                .collect::<Vec<_>>(),
        "only black wall infill may change"
    );
    let reward = contract.reward;
    ensure!(
        reward.id == CACHE
            && reward.flag == FLAG
            && reward.tile == [5, 5]
            && reward.item == "ETHER"
            && reward.quantity == 1
            && base.data.items.contains_key(&reward.item)
            && !original.scripts.contains_key(CACHE),
        "one optional native crosscut Ether required"
    );
    let warps: Vec<_> = original.events.warps.iter().map(|w| json!({
        "index":w.index,"tile":[w.x,w.y],"target":w.target_map,"target_warp":w.target_warp_id,
    })).collect();
    let actors: Vec<_> = original
        .objects
        .iter()
        .map(|o| {
            json!({
                "id":o.object_identifier,"tile":[o.x,o.y],
            })
        })
        .collect();
    ensure!(
        contract.preserved_warps == json!(warps)
            && contract.native_actor_anchors == json!(actors)
            && original.events.warps.len() == 4,
        "two original stair/arrival pairs and actor anchors required"
    );
    let mut room = original.clone();
    room.blocks = replacement;
    room.attributes.border_block = 0x2e;
    for y in 0..26 {
        for x in 0..32 {
            ensure!(
                kind(base, original, [x, y])? == kind(base, &room, [x, y])?,
                "crosscut collision changed at {x},{y}"
            );
        }
    }
    let occupied: BTreeSet<_> = room
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .chain(std::iter::once(reward.tile))
        .collect();
    ensure!(
        occupied.len() == room.objects.len() + 1
            && kind(base, &room, reward.tile)? == "FLOOR"
            && !room.events.warps.iter().any(|w| [w.x, w.y] == reward.tile),
        "clear native cache site required"
    );
    let mut seen = BTreeSet::from([[6_u16, 21]]);
    let mut pending = VecDeque::from([[6_u16, 21]]);
    while let Some([x, y]) = pending.pop_front() {
        for point in [
            x.checked_sub(1).map(|x| [x, y]),
            y.checked_sub(1).map(|y| [x, y]),
            x.checked_add(1).map(|x| [x, y]),
            y.checked_add(1).map(|y| [x, y]),
        ]
        .into_iter()
        .flatten()
        {
            if point[0] < 32
                && point[1] < 26
                && !occupied.contains(&point)
                && !seen.contains(&point)
                && kind(base, &room, point)? == "FLOOR"
            {
                seen.insert(point);
                pending.push_back(point);
            }
        }
    }
    for y in 0..26 {
        for x in 0..32 {
            if kind(base, &room, [x, y])? == "FLOOR" && !occupied.contains(&[x, y]) {
                ensure!(
                    seen.contains(&[x, y]),
                    "new cache blocks an ordinary floor branch"
                );
            }
        }
    }
    ensure!(
        seen.contains(&[4, 5]) && seen.contains(&[24, 5]),
        "cache approach and upper stair return"
    );
    let mut pack = base.clone();
    crate::northstar_lair_authoring::flag(&mut pack, FLAG)?;
    crate::northstar_lair_authoring::actor(
        &pack,
        &mut room,
        CACHE,
        "SPRITE_POKE_BALL",
        reward.tile,
    )?;
    let object = room.objects.last_mut().context("new native cache")?;
    object.object_type = "OBJECTTYPE_ITEMBALL".into();
    object.sprite_has_facings = false;
    object.event_flag = FLAG.into();
    room.scripts
        .insert(CACHE.into(), json!([command("itemball", &["ETHER", "1"])]));
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
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    #[ignore = "requires CRYSTAL_IRON_APPROACH_PARENT and authored crosscut sources"]
    fn iron_approach_keeps_story_and_rejects_collision_reward_and_preimage_drift() {
        let path = std::env::var_os("CRYSTAL_IRON_APPROACH_PARENT").expect("crosscut parent pack");
        let pack = crate::read_verified_compiled_game_pack(PathBuf::from(path))
            .expect("verified crosscut parent");
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../planning/minnesota/caves/current-iron-approach-v01");
        let before = std::fs::read_to_string(source.join("before.json")).expect("before terrain");
        let after = std::fs::read_to_string(source.join("after.json")).expect("after terrain");
        let contract = std::fs::read_to_string(source.join("contract.json")).expect("room binding");
        let child = super::revise_iron_drill_approach(&pack, &before, &after, &contract)
            .expect("solid native rock and optional resource");
        let old = &pack.data.maps[super::ROOM];
        let new = &child.data.maps[super::ROOM];
        assert_eq!(old.events, new.events);
        assert_eq!(old.script_map_commands, new.script_map_commands);
        assert_eq!(old.script_object_commands, new.script_object_commands);
        assert_eq!(new.objects.len(), old.objects.len() + 1);
        assert_eq!(&new.objects[..old.objects.len()], old.objects);
        assert_eq!(
            new.script_field_pickups.len(),
            old.script_field_pickups.len() + 1
        );
        for (label, body) in &old.scripts {
            assert_eq!(
                new.scripts.get(label),
                Some(body),
                "retained program {label}"
            );
        }
        for (id, map) in &pack.data.maps {
            if id != super::ROOM {
                assert_eq!(child.data.maps[id], *map, "unrelated map {id}");
            }
        }
        assert_eq!(child.data.trainers, pack.data.trainers);
        assert_eq!(child.runtime_files, pack.runtime_files);
        assert_eq!(child.audio_manifest, pack.audio_manifest);
        let mut unsafe_terrain: serde_json::Value =
            serde_json::from_str(&after).expect("terrain JSON");
        let row = unsafe_terrain["rows"][2].as_str().expect("native row");
        unsafe_terrain["rows"][2] = row.replacen("02", "2e", 1).into();
        assert!(
            super::revise_iron_drill_approach(
                &pack,
                &before,
                &unsafe_terrain.to_string(),
                &contract
            )
            .is_err()
        );
        for (field, value) in [
            ("native_module_sha256", serde_json::json!("stale")),
            ("preserved_warps", serde_json::json!([])),
        ] {
            let mut invalid: serde_json::Value =
                serde_json::from_str(&contract).expect("contract JSON");
            invalid[field] = value;
            assert!(
                super::revise_iron_drill_approach(&pack, &before, &after, &invalid.to_string())
                    .is_err()
            );
        }
        let mut invalid: serde_json::Value =
            serde_json::from_str(&contract).expect("contract JSON");
        invalid["reward"]["item"] = "RARE_CANDY".into();
        assert!(
            super::revise_iron_drill_approach(&pack, &before, &after, &invalid.to_string())
                .is_err()
        );
    }
}
