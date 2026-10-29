//! Bind the shore-watch shelter to the approved compact winter Cedar road.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::authored_town_design::{Room, Town};
use crate::authored_town_publication::{self, CityIdentity};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, authored_town_scripts, shore_gate_geometry};

const ROOM: &str = "MnGrandMaraisShoreGate";
const CONSTANT: &str = "MN_GRAND_MARAIS_SHORE_GATE";
const EXTERIOR: &str = "MnMaraisCedarApproach";
const MANIFEST: &str = "minnesota-compact-shore-gate-v1";

#[derive(Deserialize)]
struct Source {
    #[serde(flatten)]
    room: Room,
    base_content_hash: String,
    passive_arrivals: Vec<Arrival>,
    exterior_binding: Value,
}

#[derive(Deserialize)]
struct Arrival {
    index: u16,
    tile: [u16; 2],
    target: String,
    target_warp: i16,
}

/// Adds a public two-sided shore shelter to the existing Cedar Approach.
///
/// The authentic Ilex gate has two residents, paired passive arrivals and no
/// reward, recovery, story unlock or progression shortcut. Eight bounded native
/// blocks divide the existing road; all original connections and actors remain.
///
/// # Errors
/// Returns an error for a different parent/source contract, occupied identity,
/// changed facade or room, unsafe/blocked arrivals, a bypass or broken road link,
/// malformed native programs, or invalid pack publication.
pub fn build_shore_gate_modpack(
    base: &CompiledGamePack,
    source_json: &str,
) -> Result<CompiledGamePack> {
    crate::minnesota_interior_lineage::require_compact_coast(base)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let source: Source = serde_json::from_str(source_json).context("shore gate source")?;
    ensure!(
        source.base_content_hash == base.identity.content_hash
            && !base.data.maps.contains_key(ROOM)
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "the bound compact parent and uninstalled shore gate are required"
    );
    let room = &source.room;
    ensure!(
        room.id == ROOM
            && room.native_constant == CONSTANT
            && room.template == "IlexForestAzaleaGate"
            && room.tileset == "gate"
            && room.dimensions_metatiles == [5, 4]
            && room.passive_arrival.index == 5
            && room.passive_arrival.tile == [1, 4]
            && room.residents.len() == 2
            && room.exits.len() == 4
            && room.signs.is_empty()
            && room.pc.is_none()
            && source.passive_arrivals.len() == 4,
        "complete authentic gate floorplan, both mouths and two residents required"
    );
    for (actor, id, tile, approach) in [
        (
            &room.residents[0],
            "MNGRANDMARAISSHOREGATE_NPC1",
            [5, 2],
            [5, 4],
        ),
        (&room.residents[1], "MN_MARAIS_GATE_PADDLER", [7, 5], [7, 4]),
    ] {
        ensure!(
            actor.id == id
                && actor.tile == tile
                && actor.approach == Some(approach)
                && actor.service.is_none()
                && actor.lines.len() >= 5
                && actor
                    .lines
                    .iter()
                    .all(|line| !line.is_empty() && line.len() <= 18),
            "shore shelter residents need their native footing, counter approach and readable story"
        );
    }
    let binding = &source.exterior_binding;
    let patches = json!([
        {"position":[9,6],"before":"8b","after":"83"},
        {"position":[10,6],"before":"8b","after":"08"},
        {"position":[11,6],"before":"8b","after":"09"},
        {"position":[12,6],"before":"8b","after":"83"},
        {"position":[9,7],"before":"83","after":"0e"},
        {"position":[10,7],"before":"83","after":"10"},
        {"position":[11,7],"before":"83","after":"11"},
        {"position":[12,7],"before":"83","after":"0b"}
    ]);
    ensure!(
        binding["map"] == EXTERIOR
            && binding["expected_dimensions_metatiles"] == json!([20, 12])
            && binding["tileset"] == "mn_winter_north_shore"
            && binding["patches"] == patches
            && binding["west_road_lanes"] == json!([[0, 14], [0, 15]])
            && binding["south_road_lanes"] == json!([[26, 23], [27, 23]]),
        "gate must occupy its exact compact native Cedar road frontage"
    );
    ensure!(
        binding["sides"]
            == json!([
                {"name":"gunflint","entrances":[[19,14],[19,15]],"returns":[[18,14],[18,15]],"active_indices":[1,2],"passive_indices":[3,4],"room_passive_indices":[5,6]},
                {"name":"grand-marais","entrances":[[24,14],[24,15]],"returns":[[25,14],[25,15]],"active_indices":[5,6],"passive_indices":[7,8],"room_passive_indices":[7,8]}
            ]),
        "both compact mouths must keep exact native active/passive pairing"
    );
    let mut pack = base.clone();
    let mut exterior = pack
        .data
        .maps
        .get(EXTERIOR)
        .context("installed Cedar Approach")?
        .clone();
    ensure!(
        exterior.attributes.width == 20
            && exterior.attributes.height == 12
            && exterior.attributes.tileset_name == "mn_winter_north_shore"
            && exterior.events.warps.is_empty()
            && serde_json::to_value(&exterior.attributes.connections)?
                == binding["retained_connections"],
        "unchanged Cedar extent, art, existing connections and unoccupied warp list required"
    );
    for patch in patches.as_array().context("fixed patches")? {
        let x = patch["position"][0].as_u64().context("x")? as usize;
        let y = patch["position"][1].as_u64().context("y")? as usize;
        let before = u16::from_str_radix(patch["before"].as_str().context("before")?, 16)?;
        let after = u16::from_str_radix(patch["after"].as_str().context("after")?, 16)?;
        ensure!(
            exterior.blocks[y * 20 + x] == before,
            "changed native gate frontage preimage"
        );
        exterior.blocks[y * 20 + x] = after;
    }
    let exterior_constant = exterior
        .attributes
        .map_constant
        .clone()
        .context("Cedar constant")?;
    for (side, (entry, return_x)) in [(19, 18), (24, 25)].into_iter().enumerate() {
        for lane in 0..2 {
            authoring::warp(
                &mut exterior,
                (entry, 14 + lane),
                CONSTANT,
                5 + i16::try_from(side)? * 2 + i16::try_from(lane)?,
            );
        }
        for lane in 0..2 {
            authoring::warp(
                &mut exterior,
                (return_x, 14 + lane),
                &exterior_constant,
                3 + i16::try_from(side)? * 4 + i16::try_from(lane)?,
            );
        }
    }
    let mut gate = authoring::blank(&pack, &room.template, ROOM, CONSTANT)?;
    authoring::blocks(&mut gate, 5, 4, &room.rows.join(" "))?;
    ensure!(
        gate.blocks == pack.data.maps["IlexForestAzaleaGate"].blocks
            && room.border_block == gate.attributes.border_block,
        "the original Ilex gate shell and native border remain exact"
    );
    for (i, exit) in room.exits.iter().enumerate() {
        ensure!(
            exit.index == i as u16 + 1
                && exit.tile == [if i < 2 { 0 } else { 9 }, 4 + (i % 2) as u16]
                && exit.target == EXTERIOR
                && exit.target_warp == [3, 4, 7, 8][i],
            "gate exit pairing changed"
        );
        authoring::warp(
            &mut gate,
            (exit.tile[0], exit.tile[1]),
            &exterior_constant,
            exit.target_warp,
        );
    }
    for (i, arrival) in source.passive_arrivals.iter().enumerate() {
        ensure!(
            arrival.index == i as u16 + 5
                && arrival.tile == [if i < 2 { 1 } else { 8 }, 4 + (i % 2) as u16]
                && arrival.target == EXTERIOR
                && arrival.target_warp == [3, 4, 7, 8][i],
            "passive gate arrival pairing changed"
        );
        authoring::warp(
            &mut gate,
            (arrival.tile[0], arrival.tile[1]),
            &exterior_constant,
            arrival.target_warp,
        );
    }
    authored_town_scripts::ordinary(&pack, &mut gate, &room.residents, &[], "", "")?;
    let town: Town = serde_json::from_value(
        json!({"id":EXTERIOR,"native_constant":exterior_constant,
        "compiled_tileset":"mn_winter_north_shore","dimensions_metatiles":[20,12],"ports":[],"residents":[],"signs":[],"doors":[],"interiors":[]}),
    )?;
    authored_town_scripts::finish(&pack, &mut gate, &town)?;
    shore_gate_geometry::validate(&pack, &exterior, &gate)?;
    let location = pack.data.maps["MnGrandMarais"]
        .attributes
        .location
        .clone()
        .context("Grand Marais area")?;
    authored_town_publication::publish(
        &mut pack,
        vec![(gate, [1, 4])],
        CityIdentity {
            group: "MINNESOTA_SHORE_GATE",
            group_id: 90,
            landmark: "LANDMARK_MN_SHORE_GATE",
            fallback_landmark: Some(&location),
            constant: CONSTANT,
            name: "SHORE WATCH",
            position: [87, 19],
            manifest: MANIFEST,
            town: EXTERIOR,
            clinic: "",
            gym: "",
        },
    )?;
    authoring::refresh_event_section(&mut exterior);
    authored_town_publication::replace(&mut pack, exterior)?;
    pack.report.manifests.push(MANIFEST.into());
    pack.report.maps = pack.data.maps.len();
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::validate_compact_superior_geography(&pack)?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}
