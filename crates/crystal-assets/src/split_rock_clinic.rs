//! The public south-bank Split Rock clinic on the protected interior lineage.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::authored_route_design::Sign;
use crate::authored_town_design::{Room, Town};
use crate::authored_town_publication::{self, CityIdentity};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, authored_town_geometry, authored_town_scripts};

const BASE: &str = "2a23b30e96a5c25be19bce8a0686d08ab7f1fa8b95e4e63c4087f1e64b82f29d";
const ROOM: &str = "MnSplitRockPokecenter1F";
const CONSTANT: &str = "MN_SPLIT_ROCK_POKECENTER_1F";
const SHORE: &str = "MnR11C00R02";
const MANIFEST: &str = "minnesota-split-rock-clinic-v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    schema_version: u16,
    id: String,
    base_content_hash: String,
    room: Room,
    exterior_binding: Binding,
    integration_scope: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    map: String,
    dimensions_metatiles: [u16; 2],
    tileset: String,
    patches: Vec<Patch>,
    door: [u16; 2],
    return_tile: [u16; 2],
    entry_index: u16,
    return_index: u16,
    sign: Sign,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Patch {
    position: [u16; 2],
    before: u16,
    after: u16,
}

/// Adds Split Rock's free shore clinic, PC and two local waiting-room residents.
///
/// Fifteen native facade/forecourt cells, two warps and a placard are appended
/// to the existing shore. Original roads, water, field gates, actors, story,
/// interiors, catalogs and audio remain intact. The standard nurse binds one
/// new recovery point. This does not install Ely's separate back trail or snow,
/// rebase the canonical coast, migrate saves or establish earned progression.
///
/// # Errors
/// Rejects a stale parent/source, occupied identity, changed native floorplan,
/// unsafe facade/door/arrival, disconnected public approach, missing resident or
/// service, malformed native scripts or failed pack verification.
pub fn build_split_rock_clinic_pack(
    base: &CompiledGamePack,
    source_json: &str,
) -> Result<CompiledGamePack> {
    crate::minnesota_interior_lineage::require_compact_coast(base)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let value: Value = serde_json::from_str(source_json)?;
    let source: Source = serde_json::from_value(value.clone()).context("Split Rock source")?;
    ensure!(
        source.schema_version == 1
            && source.id == "MN_SPLIT_ROCK_CLINIC_INTERIORS_V1"
            && source.base_content_hash == BASE
            && base.identity.content_hash == BASE
            && !source.integration_scope.is_empty()
            && !base.data.maps.contains_key(ROOM)
            && !base.report.manifests.iter().any(|id| id == MANIFEST),
        "preserved fishing-home parent and uninstalled Split Rock clinic required"
    );
    let room = &source.room;
    let binding = &source.exterior_binding;
    ensure!(
        room.id == ROOM
            && room.native_constant == CONSTANT
            && room.template == "CherrygrovePokecenter1F"
            && room.tileset == "pokecenter"
            && room.dimensions_metatiles == [5, 4]
            && room.border_block == 0
            && room.passive_arrival.index == 3
            && room.passive_arrival.tile == [3, 5]
            && room.exits.len() == 2
            && room.residents.len() == 3
            && room.signs.is_empty()
            && room
                .pc
                .as_ref()
                .is_some_and(|pc| pc.tile == [9, 1] && pc.approach == [9, 2])
            && binding.map == SHORE
            && binding.dimensions_metatiles == [32, 24]
            && binding.tileset == "mn_north_shore"
            && binding.door == [33, 33]
            && binding.return_tile == [33, 34]
            && binding.entry_index == 3
            && binding.return_index == 4
            && binding.sign.id == "MN_SPLIT_ROCK_CLINIC_SIGN"
            && binding.sign.tile == [30, 34],
        "complete native shore clinic, services and indexed door pair required"
    );
    let mut pack = base.clone();
    let original = base
        .data
        .maps
        .get(SHORE)
        .context("installed Split Rock shore")?;
    ensure!(
        original.attributes.width == 32
            && original.attributes.height == 24
            && original.attributes.tileset_name == binding.tileset
            && original.events.warps.len() == 2,
        "unchanged shore shell and lighthouse warps required"
    );
    let mut shore = original.clone();
    ensure!(
        binding.patches.len() == 15,
        "fifteen bounded clinic cells required"
    );
    let mut seen = BTreeSet::new();
    for patch in &binding.patches {
        let [x, y] = patch.position;
        let expected = match [x, y] {
            [16, 15] => 0x18,
            [17, 15] => 0x19,
            [16, 16] => 0x1a,
            [17, 16] => 0x1b,
            [15, 17] => 0x45,
            _ => 0x01,
        };
        ensure!(
            (15..=18).contains(&x)
                && (15..=18).contains(&y)
                && [x, y] != [17, 18]
                && patch.after == expected
                && seen.insert([x, y]),
            "clinic must keep its complete native facade and dry forecourt"
        );
        let index = usize::from(y) * 32 + usize::from(x);
        ensure!(
            shore.blocks[index] == patch.before,
            "clinic cell preimage differs"
        );
        let old = &base.data.tilesets[&binding.tileset].collision[&format!("{:02x}", patch.before)];
        ensure!(
            old.iter()
                .all(|kind| matches!(kind.as_str(), "FLOOR" | "TALL_GRASS" | "WALL"))
                && !original
                    .objects
                    .iter()
                    .any(|o| [o.x / 2, o.y / 2] == [x, y])
                && !original
                    .events
                    .warps
                    .iter()
                    .any(|w| [w.x / 2, w.y / 2] == [x, y])
                && !original
                    .events
                    .coord_events
                    .iter()
                    .any(|e| [e.x / 2, e.y / 2] == [x, y])
                && !original
                    .events
                    .bg_events
                    .iter()
                    .any(|e| [e.x / 2, e.y / 2] == [x, y]),
            "clinic must preserve water, cliffs, field gates and existing events"
        );
        shore.blocks[index] = patch.after;
    }
    authoring::warp(&mut shore, (33, 33), CONSTANT, 3);
    authoring::warp(&mut shore, (33, 34), "MN_R11_C00R02", 4);
    authored_town_scripts::ordinary(
        &pack,
        &mut shore,
        &[],
        std::slice::from_ref(&binding.sign),
        "",
        "",
    )?;
    let town: Town = serde_json::from_value(json!({
        "id":SHORE,"native_constant":"MN_R11_C00R02","compiled_tileset":"mn_north_shore",
        "dimensions_metatiles":[32,24],"ports":[],"residents":[],"signs":[],"doors":[],
        "interiors":[value["room"].clone()]
    }))?;
    authored_town_scripts::finish(&pack, &mut shore, &town)?;
    let mut clinic = authoring::blank(&pack, &room.template, ROOM, CONSTANT)?;
    authoring::blocks(&mut clinic, 5, 4, &room.rows.join(" "))?;
    let mut expected = base.data.maps[&room.template].blocks.clone();
    expected[15] = 0x04;
    ensure!(
        clinic.blocks == expected,
        "native clinic shell except unused stairs must stay exact"
    );
    for (index, exit) in room.exits.iter().enumerate() {
        ensure!(
            exit.index == u16::try_from(index)? + 1
                && exit.tile == [[3, 7], [4, 7]][index]
                && exit.target == SHORE
                && exit.target_warp == 4,
            "both native clinic carpets must return to their shore forecourt"
        );
        authoring::warp(
            &mut clinic,
            (exit.tile[0], exit.tile[1]),
            "MN_R11_C00R02",
            4,
        );
    }
    authoring::warp(&mut clinic, (3, 5), "MN_R11_C00R02", 4);
    authored_town_scripts::ordinary(&pack, &mut clinic, &room.residents, &[], CONSTANT, "")?;
    crate::split_rock_clinic_geometry::residents(room)?;
    clinic.scripts.insert(
        "MnSplitRockNurse".into(),
        json!([
            authoring::command("blackoutmod", &[CONSTANT]),
            authoring::command("jumpstd", &["PokecenterNurseScript"])
        ]),
    );
    clinic.scripts.insert(
        "MnSplitRockPC".into(),
        json!([
            authoring::command("opentext", &[]),
            authoring::command("special", &["PokemonCenterPC"]),
            authoring::command("closetext", &[]),
            authoring::command("end", &[])
        ]),
    );
    clinic.events.bg_events.push(serde_json::from_value(json!({
        "x":9,"y":1,"event_type":"BGEVENT_READ","script":"MnSplitRockPC"
    }))?);
    authored_town_scripts::finish(&pack, &mut clinic, &town)?;
    let modules = vec![(clinic, [3, 5])];
    authored_town_geometry::validate(&pack, &town, &modules)?;
    crate::split_rock_clinic_geometry::shore(&pack, original, &shore)?;
    authored_town_publication::publish(
        &mut pack,
        modules,
        CityIdentity {
            group: "MINNESOTA_SPLIT_ROCK_CLINIC",
            group_id: 91,
            landmark: "LANDMARK_MN_SPLIT_ROCK_CLINIC",
            fallback_landmark: Some("LANDMARK_MN_NORTH_SHORE"),
            constant: CONSTANT,
            name: "SPLIT ROCK CLINIC",
            position: [82, 22],
            manifest: MANIFEST,
            town: ROOM,
            clinic: ROOM,
            gym: "",
        },
    )?;
    authored_town_publication::replace(&mut pack, shore)?;
    pack.report.maps = pack.data.maps.len();
    pack.report.manifests.push(MANIFEST.into());
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
#[path = "split_rock_clinic_tests.rs"]
mod tests;
