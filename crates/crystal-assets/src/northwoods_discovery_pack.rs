//! Current-parent map binding for the optional northwoods discovery review.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_core::world::encounters::{
    FieldEncounterData, FieldEncounterEntry, FieldEncounterTable, WildEncounter, WildEncounterData,
    WildEncounterTable,
};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;
use crate::northstar_lair_authoring as a;
use crate::northstar_rival_arc as arc;

#[path = "northwoods_gatehouse.rs"]
mod gatehouse;

const MAP: &str = "MnTallTaleTrail";
const OUTSIDE: &str = "MnBemidjiNorthwoods";
const MANIFEST: &str = "minnesota-northwoods-tall-tales-v1";

/// Builds an unselected review child with the native northwoods quest programs.
/// Installs an original Ilex gate with both passage lanes while preserving the
/// original north/south corridor. The roomy trail is a separate outdoor scene,
/// so it cannot overlap the neighboring roads in the connected world. The caller owns visual acceptance, prop art,
/// actual travel/battle/blackout/Continue tests and any later world selection.
/// It does not write files or alter its borrowed parent.
///
/// # Errors
/// Rejects occupied identities/seams, changed branch terrain, malformed source,
/// missing original Minnesota species/art, unsafe anchors or invalid output.
pub fn build_northwoods_discovery_pack(
    base: &CompiledGamePack,
    quests: &str,
    site: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    crate::validate_compact_superior_geography(base)?;
    ensure!(
        !base.data.maps.contains_key(MAP) && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "northwoods discovery already installed"
    );
    let source: Value = serde_json::from_str(site)?;
    ensure!(
        source["schema_version"] == 1 && source["map"] == MAP && source["tileset"] == "johto",
        "canonical northwoods site required"
    );
    ensure!(
        source["dimensions_metatiles"] == serde_json::json!([20, 20]),
        "reviewed trail dimensions required"
    );
    let programs = crate::build_northwoods_discovery_programs(quests)?;
    let mut outside = arc::map(base, OUTSIDE)?;
    ensure!(
        outside.attributes.tileset_name == "johto"
            && outside.attributes.width == 8
            && outside.attributes.height == 14,
        "reviewed northwoods corridor required"
    );
    ensure!(
        !outside
            .attributes
            .connections
            .iter()
            .any(|c| c.direction.eq_ignore_ascii_case("east")),
        "occupied east seam"
    );
    ensure!(
        outside.blocks[5 * 8 + 5] == 1
            && outside.blocks[5 * 8 + 6] == 5
            && outside.blocks[5 * 8 + 7] == 5,
        "northwoods branch preimage differs"
    );
    ensure!(
        !outside
            .objects
            .iter()
            .any(|o| (12..16).contains(&o.x) && (10..12).contains(&o.y))
            && !outside
                .events
                .coord_events
                .iter()
                .any(|e| (12..16).contains(&e.x) && (10..12).contains(&e.y)),
        "occupied branch approach"
    );
    for species in [
        "BABEHEMOTH",
        "BUNJANT",
        "HATCHIT",
        "PINIT",
        "PITCHWICK",
        "NOTCHBILL",
        "DENSY",
        "DOZEBEAR",
        "CARTICK",
    ] {
        ensure!(
            base.data.pokemon.contains_key(species)
                && base
                    .data
                    .story_event_script_constants
                    .global
                    .contains_key(&format!("MN_DEX_MEMBER_{species}")),
            "active original species required: {species}"
        );
    }
    for species in ["babehemoth", "bunjant"] {
        ensure!(
            base.runtime_files
                .contains_key(&format!("gfx/pokemon/{species}/overworld.2bpp")),
            "reviewed native walking bank required: {species}"
        );
    }
    let mut pack = base.clone();
    for flag in &programs.event_flags {
        a::flag(&mut pack, flag)?;
    }
    let mut room = authoring::blank(&pack, OUTSIDE, MAP, "MN_TALL_TALE_TRAIL")?;
    let mut rows = Vec::new();
    for row in source["rows"].as_array().context("trail rows")? {
        let row = row
            .as_str()
            .context("terrain row")?
            .split_whitespace()
            .map(|t| source["legend"][t].as_str().context("native terrain token"))
            .collect::<Result<Vec<_>>>()?;
        ensure!(row.len() == 20, "trail row width");
        rows.push(row.join(" "));
    }
    ensure!(rows.len() == 20, "trail height");
    authoring::blocks(&mut room, 20, 20, &rows.join("\n"))?;
    ensure!(
        room.blocks.iter().all(|block| *block < 128),
        "northwoods discovery must use original Crystal blocks only"
    );
    room.attributes.border_block = 5;
    room.attributes.tileset_name = "johto".into();
    room.attributes.connections.clear();
    room.attributes.connection_flags = None;
    let gate = gatehouse::bind(&pack, &mut outside, &mut room, &source["gatehouse"])?;
    room.scripts = programs.scripts;
    for (id, sprite, key) in [
        ("MnTaleWorker", "SPRITE_FISHER", "worker_candidate"),
        (
            "MN_TALE_BABEHEMOTH",
            "SPRITE_BABEHEMOTH",
            "babehemoth_candidate",
        ),
        ("MN_TALE_BUNJANT", "SPRITE_BUNJANT", "bunjant_candidate"),
    ] {
        let at = point(&source, key)?;
        ensure!(
            crate::authored_town_geometry::kind(&pack, &room, at)? == "FLOOR",
            "unsafe actor anchor {id}"
        );
        a::actor(&pack, &mut room, id, sprite, at)?;
    }
    for (label, key) in [
        ("MnTaleCulvert", "culvert_observation_candidate"),
        ("MnTalePrints", "hoofprint_observation_candidate"),
        ("MnTaleStack", "windthrow_observation_candidate"),
        ("MnTaleLog", "marked_log_candidate"),
    ] {
        a::read(&mut room, point(&source, key)?, label);
    }
    a::read(&mut room, [28, 16], "MnTalePath");
    a::read(&mut room, [30, 12], "MnTaleCampRecord");
    a::callback(&mut room, "MAPCALLBACK_OBJECTS", &programs.callbacks[0]);
    a::callback(&mut room, "MAPCALLBACK_NEWMAP", &programs.callbacks[1]);
    a::callback(&mut room, "MAPCALLBACK_TILES", &programs.callbacks[2]);
    let reach = crate::authored_town_geometry::reachable(&pack, &room, [3, 34], false)?;
    for at in [[3, 34], [4, 34], [28, 17]] {
        ensure!(reach.contains(&at), "unreachable quest seam/path {at:?}");
    }
    for key in [
        "culvert_observation_candidate",
        "hoofprint_observation_candidate",
        "windthrow_observation_candidate",
        "marked_log_candidate",
    ] {
        let at = point(&source, key)?;
        if key == "marked_log_candidate" {
            ensure!(
                !reach.contains(&[30, 9]),
                "camp must remain gated before clearance"
            );
        } else {
            ensure!(
                has_approach(&reach, at),
                "unreachable authored observation {key}"
            );
        }
    }
    let mut opened = room.clone();
    ensure!(
        opened.blocks[8 * 20 + 14] == 0x5d,
        "reviewed timber throat required"
    );
    opened.blocks[8 * 20 + 14] = 1;
    let after = crate::authored_town_geometry::reachable(&pack, &opened, [3, 34], false)?;
    ensure!(
        after.contains(&[30, 9]) && has_approach(&after, point(&source, "marked_log_candidate")?),
        "cleared throat must reach camp and marked wood"
    );
    for y in 0..40 {
        for x in 0..40 {
            let kind = crate::authored_town_geometry::kind(&pack, &room, [x, y])?;
            ensure!(
                kind != "BUOY",
                "buoy terrain forbidden in the northwoods pool: {x},{y}"
            );
            if x == 0 || x == 39 || y == 0 || y == 39 {
                let expected = "WALL";
                ensure!(kind == expected, "unreviewed outer opening {x},{y}");
            }
        }
    }
    room.scripted_wild_battles = crate::parse_scripted_wild_battles(MAP, &room.scripts)?;
    a::publish(&mut pack, gate, OUTSIDE, [1, 4])?;
    a::publish(&mut pack, room, OUTSIDE, [3, 34])?;
    arc::publish(&mut pack, outside)?;
    let slots = [
        ("HATCHIT", 28),
        ("PINIT", 28),
        ("PITCHWICK", 32),
        ("NOTCHBILL", 30),
        ("DENSY", 28),
        ("DOZEBEAR", 32),
        ("CARTICK", 30),
    ]
    .into_iter()
    .map(|(species, level)| WildEncounter {
        species: species.into(),
        level,
    })
    .collect::<Vec<_>>();
    pack.data.wild_encounters.insert(
        MAP.into(),
        WildEncounterData {
            map_name: MAP.into(),
            grass_rates: Some(BTreeMap::from([
                ("morning".into(), 20),
                ("day".into(), 20),
                ("night".into(), 20),
            ])),
            grass: Some(WildEncounterTable {
                morning: slots.clone(),
                day: slots.clone(),
                night: slots,
            }),
            ..Default::default()
        },
    );
    let tree_slots = |entries: &[(&str, u8, u8)]| {
        entries
            .iter()
            .map(|&(species, level, weight)| FieldEncounterEntry {
                species: species.into(),
                level,
                weight,
                sleep_turns_by_time: BTreeMap::new(),
            })
            .collect()
    };
    pack.data.field_encounters.insert(
        MAP.into(),
        FieldEncounterData {
            map_name: MAP.into(),
            tables: BTreeMap::from([(
                "headbutt".into(),
                FieldEncounterTable {
                    common: tree_slots(&[
                        ("NOTCHBILL", 30, 50),
                        ("PINIT", 28, 30),
                        ("HATCHIT", 28, 20),
                    ]),
                    rare: tree_slots(&[
                        ("DOZEBEAR", 32, 50),
                        ("PITCHWICK", 32, 30),
                        ("CARTICK", 30, 20),
                    ]),
                },
            )]),
        },
    );
    pack.report.manifests.push(MANIFEST.into());
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
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}

fn point(source: &Value, key: &str) -> Result<[u16; 2]> {
    let at = source["design"][key]
        .as_array()
        .context("authored interaction anchor")?;
    ensure!(at.len() == 2, "two-coordinate anchor {key}");
    Ok([
        u16::try_from(at[0].as_u64().context("anchor x")?)?,
        u16::try_from(at[1].as_u64().context("anchor y")?)?,
    ])
}

fn has_approach(reach: &BTreeSet<[u16; 2]>, [x, y]: [u16; 2]) -> bool {
    [
        x.checked_sub(1).map(|a| [a, y]),
        x.checked_add(1).map(|a| [a, y]),
        y.checked_sub(1).map(|b| [x, b]),
        y.checked_add(1).map(|b| [x, b]),
    ]
    .into_iter()
    .flatten()
    .any(|at| reach.contains(&at))
}
