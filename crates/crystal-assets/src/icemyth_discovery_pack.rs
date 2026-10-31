//! Guarded exhibition-hall extension without adding to the full fairgrounds.
use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;
use crate::northstar_lair_authoring as a;
use crate::northstar_rival_arc as arc;

const HALL: &str = "MnStPaulFairExhibitionHall";
const SERVICE: &str = "MnIcemythServicePassage";
const CONSTANT: &str = "MN_ICEMYTH_SERVICE_PASSAGE";
const MARKER: &str = "minnesota-icemyth-exhibition-v1";

/// Adds the optional exhibit investigation and a native service-room staircase.
/// The existing fairgrounds receives no new actors. The hall keeps its original
/// actors, scripts, statue read and three warps; one reviewed floor block gains
/// native stairs. A new service room holds the optional level-50 encounter.
/// All original runtime assets and audio stay unchanged. The borrowed pack and
/// disk remain unchanged. Production travel, capture, storage, return, Continue
/// and actual native presentation still require acceptance by the caller.
///
/// # Errors
/// Rejects duplicate publication, noncanonical layout/anchors, missing active
/// species/art, occupied arrivals/reads, changed native geometry, script-name
/// collisions, invalid map publication or an invalid resulting runtime pack.
pub fn build_icemyth_discovery_pack(
    base: &CompiledGamePack,
    quests: &str,
    site: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    ensure!(
        !base.data.maps.contains_key(SERVICE) && !base.report.manifests.iter().any(|m| m == MARKER),
        "Icemyth exhibit already installed"
    );
    let source: Value = serde_json::from_str(site)?;
    ensure!(
        source["schema_version"] == 1
            && source["id"] == "minnesota-icemyth-exhibition-site-v1"
            && source["hall"] == HALL
            && source["hall_dimensions_metatiles"] == serde_json::json!([8, 6])
            && source["service"]["map"] == SERVICE
            && source["service"]["constant"] == CONSTANT,
        "canonical exhibit source required"
    );
    ensure!(
        base.data.pokemon.contains_key("ICEMYTH")
            && base
                .data
                .story_event_script_constants
                .global
                .contains_key("MN_DEX_MEMBER_ICEMYTH")
            && base
                .runtime_files
                .contains_key("gfx/pokemon/icemyth/overworld.2bpp"),
        "active original Icemyth and native field bank required"
    );
    let mut pack = base.clone();
    let programs = crate::build_icemyth_discovery_programs(quests)?;
    for flag in &programs.event_flags {
        a::flag(&mut pack, flag)?;
    }
    let mut hall = arc::map(base, HALL)?;
    ensure!(
        hall.attributes.tileset_name == "house"
            && hall.attributes.width == 8
            && hall.attributes.height == 6
            && hall.events.warps.len() == 3
            && hall.objects.len() == 2,
        "reviewed exhibit hall required"
    );
    let hall_constant = hall
        .attributes
        .map_constant
        .clone()
        .context("hall constant")?;
    ensure!(
        point(&source["hall_stair"]["metatile"])? == [6, 4]
            && source["hall_stair"]["before"] == "0f"
            && source["hall_stair"]["after"] == "25"
            && hall.blocks[4 * 8 + 6] == 0x0f,
        "reviewed floor staircase preimage required"
    );
    ensure!(
        point(&source["hall_stair"]["active"])? == [12, 8]
            && point(&source["hall_stair"]["arrival"])? == [13, 8]
            && point(&source["hall_stair"]["approach"])? == [13, 9],
        "reviewed hall stair quadrants required"
    );
    for at in [[12, 8], [13, 8], [13, 9]] {
        vacant(&hall, at)?;
        ensure!(
            crate::authored_town_geometry::kind(base, &hall, at)? == "FLOOR",
            "hall stair/arrival must begin on floor"
        );
    }
    let hall_actors = [
        (
            "MnIcemythAttendant",
            "SPRITE_POKEFAN_M",
            "attendant",
            [10, 8],
        ),
        (
            "MnIcemythDelivery",
            "SPRITE_SAILOR",
            "delivery_worker",
            [7, 8],
        ),
        ("MnIcemythVisitor", "SPRITE_LASS", "visitor", [4, 8]),
        (
            "MN_EXHIBIT_FROZEN_CASE",
            "SPRITE_ICEMYTH",
            "frozen_case",
            [10, 7],
        ),
    ];
    for (label, sprite, key, at) in hall_actors {
        ensure!(
            point(&source["hall_actors"][key])? == at,
            "reviewed exhibit actor anchor: {key}"
        );
        vacant(&hall, at)?;
        ensure!(
            crate::authored_town_geometry::kind(base, &hall, at)? == "FLOOR",
            "exhibit actor floor: {key}"
        );
        a::actor(base, &mut hall, label, sprite, at)?;
    }
    for (key, label, at, approach) in [
        ("latch", "MnIcemythLatch", [9, 8], [9, 7]),
        ("tray", "MnIcemythTray", [13, 7], [13, 8]),
    ] {
        ensure!(
            point(&source["hall_observations"][key]["at"])? == at
                && point(&source["hall_observations"][key]["approach"])? == approach,
            "reviewed exhibit read: {key}"
        );
        vacant(&hall, at)?;
        vacant(&hall, approach)?;
        ensure!(
            crate::authored_town_geometry::kind(base, &hall, at)? == "WALL"
                && crate::authored_town_geometry::kind(base, &hall, approach)? == "FLOOR",
            "solid exhibit read and dry approach required: {key}"
        );
        a::read(&mut hall, at, label);
    }
    hall.blocks[4 * 8 + 6] = 0x25;
    ensure!(
        crate::authored_town_geometry::kind(base, &hall, [12, 8])? == "LADDER"
            && crate::authored_town_geometry::kind(base, &hall, [13, 8])? == "FLOOR",
        "native active/passive hall stair quadrants required"
    );
    authoring::warp(&mut hall, (12, 8), CONSTANT, 2);
    authoring::warp(&mut hall, (13, 8), &hall_constant, 5);
    a::callback(&mut hall, "MAPCALLBACK_OBJECTS", "MnIcemythHallObjects");
    let mut room = authoring::blank(&pack, HALL, SERVICE, CONSTANT)?;
    let service = &source["service"];
    ensure!(
        service["dimensions_metatiles"] == serde_json::json!([7, 7])
            && service["tileset"] == "house",
        "reviewed service-room dimensions/art required"
    );
    let rows = service["rows"]
        .as_array()
        .context("service rows")?
        .iter()
        .map(|r| r.as_str().context("native row"))
        .collect::<Result<Vec<_>>>()?;
    ensure!(
        rows.len() == 7 && rows.iter().all(|r| r.split_whitespace().count() == 7),
        "service row extent required"
    );
    authoring::blocks(&mut room, 7, 7, &rows.join("\n"))?;
    let expected = [
        "0a 0a 0a 0a 0a 0a 0a",
        "0a 08 0f 04 0f 08 0a",
        "0a 0f 04 0f 0f 0f 0a",
        "0a 0f 0f 0f 0f 1f 0a",
        "0a 04 0f 0f 0f 0f 0a",
        "0a 25 0f 0f 0f 0f 0a",
        "0a 0a 0a 0a 0a 0a 0a",
    ]
    .join("\n");
    let mut reviewed = room.clone();
    authoring::blocks(&mut reviewed, 7, 7, &expected)?;
    ensure!(
        room.blocks == reviewed.blocks,
        "reviewed source service terrain required"
    );
    room.attributes.border_block = 0x0a;
    for (key, at) in [
        ("active_stair", [2, 10]),
        ("arrival", [3, 11]),
        ("icemyth", [9, 5]),
        ("icemyth_approach", [9, 6]),
    ] {
        ensure!(
            point(&service[key])? == at,
            "reviewed service anchor: {key}"
        );
    }
    ensure!(
        point(&service["case_read"]["at"])? == [5, 4]
            && point(&service["case_read"]["approach"])? == [6, 4],
        "reviewed empty-case approach required"
    );
    ensure!(
        crate::authored_town_geometry::kind(base, &room, [2, 10])? == "LADDER",
        "native return stair required"
    );
    for at in [[3, 11], [9, 5], [9, 6], [6, 4]] {
        ensure!(
            crate::authored_town_geometry::kind(base, &room, at)? == "FLOOR",
            "service floor anchor required: {at:?}"
        );
    }
    ensure!(
        crate::authored_town_geometry::kind(base, &room, [5, 4])? == "WALL",
        "solid empty case required"
    );
    for y in 0..14 {
        for x in 0..14 {
            if x == 0 || y == 0 || x == 13 || y == 13 {
                ensure!(
                    crate::authored_town_geometry::kind(base, &room, [x, y])? == "WALL",
                    "closed service perimeter required"
                );
            }
        }
    }
    a::actor(
        base,
        &mut room,
        "MN_EXHIBIT_ICEMYTH",
        "SPRITE_ICEMYTH",
        [9, 5],
    )?;
    a::read(&mut room, [5, 4], "MnIcemythCase");
    authoring::warp(&mut room, (2, 10), &hall_constant, 5);
    authoring::warp(&mut room, (3, 11), CONSTANT, 2);
    a::callback(&mut room, "MAPCALLBACK_OBJECTS", &programs.callbacks[0]);
    a::callback(&mut room, "MAPCALLBACK_NEWMAP", &programs.callbacks[1]);
    for (map, roots) in [
        (
            &mut hall,
            vec![
                "MnIcemythAttendant",
                "MnIcemythDelivery",
                "MnIcemythVisitor",
                "MN_EXHIBIT_FROZEN_CASE",
                "MnIcemythLatch",
                "MnIcemythTray",
                "MnIcemythHallObjects",
            ],
        ),
        (
            &mut room,
            vec![
                "MN_EXHIBIT_ICEMYTH",
                "MnIcemythCase",
                "MnIcemythObjects",
                "MnIcemythNewVisit",
            ],
        ),
    ] {
        let roots = roots.into_iter().map(String::from).collect::<Vec<_>>();
        for label in crate::discovery_dialogue::script_closure(&programs.scripts, &roots)? {
            ensure!(
                !map.scripts.contains_key(&label),
                "occupied exhibit script: {label}"
            );
            map.scripts
                .insert(label.clone(), programs.scripts[&label].clone());
        }
        map.scripted_wild_battles = crate::parse_scripted_wild_battles(&map.id, &map.scripts)?;
    }
    arc::publish(&mut pack, hall)?;
    a::publish(&mut pack, room, HALL, [3, 11])?;
    for (name, original) in &base.data.maps {
        if name != HALL {
            ensure!(
                pack.data.maps.get(name) == Some(original),
                "unrelated exhibit map: {name}"
            );
        }
    }
    ensure!(
        pack.runtime_files == base.runtime_files
            && pack.audio_manifest == base.audio_manifest
            && pack.compiled_audio == base.compiled_audio,
        "exhibit must preserve original art/audio exactly"
    );
    pack.report.manifests.push(MARKER.into());
    pack.report.maps = pack.data.maps.len();
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::verify_northstar_publication_geometry(&pack)?;
    Ok(pack)
}

fn point(value: &Value) -> Result<[u16; 2]> {
    serde_json::from_value(value.clone()).context("two-coordinate authored exhibit anchor")
}
fn vacant(map: &crate::MapModule, at: [u16; 2]) -> Result<()> {
    ensure!(
        !map.objects.iter().any(|o| [o.x, o.y] == at)
            && !map.events.warps.iter().any(|e| [e.x, e.y] == at)
            && !map.events.coord_events.iter().any(|e| [e.x, e.y] == at)
            && !map.events.bg_events.iter().any(|e| [e.x, e.y] == at),
        "occupied exhibit anchor/approach: {at:?}"
    );
    Ok(())
}
