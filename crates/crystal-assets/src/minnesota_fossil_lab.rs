//! A real Rochester laboratory for the existing paired-fossil service.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde::Deserialize;
use serde_json::Value;

use crate::grand_marais_modpack::authoring;
use crate::grand_marais_modpack::authoring::command as c;
use crate::{CompiledGamePack, CompiledMapExtension, MapModule, RuntimeSpawnPoint};

const MANIFEST: &str = "minnesota-fossil-lab-v1";
const LAB: &str = "MnRochesterFossilLab";
const CONSTANT: &str = "MN_ROCHESTER_FOSSIL_LAB";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    schema_version: u8,
    id: String,
    parent_content_hash: String,
    native_group_id: u16,
    native_map_id: u16,
    cells: Vec<Cell>,
    door: [u16; 2],
    porch: [u16; 2],
    approach: [u16; 2],
    sign: [u16; 2],
    rows: Vec<String>,
    researcher: [u16; 2],
    assistant: [u16; 2],
    bench: [u16; 2],
    arrival: [u16; 2],
    exits: [[u16; 2]; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Cell {
    metatile: [u16; 2],
    before: String,
    after: String,
}

/// Moves fossil restoration into a signed, enterable lab beside the clinic.
///
/// The existing native programs remain the source of badge, party-space,
/// gift-success, item-consumption and once-only rules. They are relocated under
/// unique local labels; their saved global flags are unchanged. The bluff
/// directions point to the lab. Unrelated terrain, programs, art and audio are
/// preserved, and the borrowed parent is unchanged on failure.
///
/// # Errors
/// Rejects stale parents/preimages, occupied native identities, missing fossil
/// programs, obstructed approaches, malformed rooms or invalid compiled output.
pub fn build_minnesota_fossil_lab(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_northstar_publication_geometry(base)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let plan: Plan = serde_json::from_str(source)?;
    ensure!(
        plan.schema_version == 1
            && plan.id == MANIFEST
            && plan.parent_content_hash == base.identity()?.content_hash,
        "reviewed fossil-lab parent required"
    );
    ensure!(
        !base.data.maps.contains_key(LAB)
            && !base
                .data
                .runtime_map_metadata
                .values()
                .any(|m| m.group_id == plan.native_group_id && m.map_id == plan.native_map_id),
        "fossil lab identity is occupied"
    );
    let mut pack = base.clone();
    let mut city = base
        .data
        .maps
        .get("MnRochester")
        .context("Rochester")?
        .clone();
    ensure!(
        city.attributes.width == 20
            && city.attributes.height == 16
            && city.attributes.tileset_name == "johto_modern",
        "reviewed Rochester geometry required"
    );
    let old_researcher = city
        .objects
        .iter()
        .position(|o| o.script == "MnFossilResearcher")
        .context("outdoor fossil researcher")?;
    ensure!(
        [
            city.objects[old_researcher].x,
            city.objects[old_researcher].y
        ] == [20, 10],
        "outdoor fossil researcher moved"
    );
    city.objects.remove(old_researcher);

    let mut lab = authoring::blank(base, "ElmsLab", LAB, CONSTANT)?;
    authoring::blocks(&mut lab, 5, 5, &plan.rows.join(" "))?;
    lab.attributes.location = city.attributes.location.clone();
    lab.attributes.map_group_constant = city.attributes.map_group_constant.clone();
    lab.attributes.fishing_group = Some("FISHGROUP_NONE".into());
    ensure!(
        plan.researcher != plan.assistant
            && plan.researcher != plan.arrival
            && plan.assistant != plan.arrival
            && plan.exits[0] != plan.exits[1],
        "lab residents and passive arrival must have distinct positions"
    );
    for tile in plan.exits {
        ensure!(
            crate::authored_town_geometry::kind(base, &lab, tile)? == "WARP_CARPET_DOWN",
            "lab exits must be real native carpets"
        );
    }
    let fossil_labels: Vec<_> = city
        .scripts
        .keys()
        .filter(|label| label.starts_with("MnFossil"))
        .cloned()
        .collect();
    ensure!(
        fossil_labels.iter().any(|s| s == "MnFossilRestoreTRILUME")
            && fossil_labels.iter().any(|s| s == "MnFossilRestoreSIPHLET"),
        "both restoration programs required"
    );
    for label in fossil_labels {
        let mut body = city.scripts[&label].clone();
        rename_local_labels(&mut body);
        lab.scripts
            .insert(label.replacen("MnFossil", "MnLabFossil", 1), body);
        // Common helpers shared with the bluff retain their identical native
        // definitions. The old gift entry points themselves leave the city.
        if !base
            .data
            .maps
            .values()
            .any(|m| m.id != city.id && m.scripts.contains_key(&label))
        {
            city.scripts.remove(&label);
        }
    }
    let mut seen = BTreeSet::new();
    for cell in &plan.cells {
        let [x, y] = cell.metatile;
        ensure!(
            x < city.attributes.width && y < city.attributes.height && seen.insert([x, y]),
            "bounded unique lab cells required"
        );
        ensure!(
            !city.objects.iter().any(|o| [o.x / 2, o.y / 2] == [x, y])
                && !city
                    .events
                    .warps
                    .iter()
                    .any(|w| [w.x / 2, w.y / 2] == [x, y])
                && !city
                    .events
                    .bg_events
                    .iter()
                    .any(|e| [e.x / 2, e.y / 2] == [x, y]),
            "lab patch overlaps an existing event"
        );
        let index = usize::from(y) * 20 + usize::from(x);
        ensure!(
            city.blocks[index] == u16::from_str_radix(&cell.before, 16)?,
            "lab terrain preimage differs"
        );
        city.blocks[index] = u16::from_str_radix(&cell.after, 16)?;
    }
    for (tile, kind) in [
        (plan.door, "DOOR"),
        (plan.porch, "FLOOR"),
        (plan.approach, "FLOOR"),
        (plan.sign, "WALL"),
    ] {
        ensure!(
            crate::authored_town_geometry::kind(base, &city, tile)? == kind,
            "lab exterior collision differs at {tile:?}"
        );
    }
    let outside = crate::authored_town_geometry::reachable(base, &city, plan.approach, false)?;
    ensure!(
        outside.contains(&plan.door)
            && outside.contains(&plan.porch)
            && outside.contains(&[plan.sign[0], plan.sign[1] + 1]),
        "lab and sign must connect to the public road"
    );
    let return_index = i16::try_from(city.events.warps.len() + 2)?;
    authoring::warp(&mut city, (plan.door[0], plan.door[1]), CONSTANT, 3);
    authoring::warp(
        &mut city,
        (plan.porch[0], plan.porch[1]),
        "MN_ROCHESTER",
        return_index,
    );
    sign(
        &mut city,
        plan.sign,
        "MnRochesterFossilLabSign",
        &["FOSSIL LAB", "Ancient partners", "restored here."],
    );
    crate::northstar_rival_arc::publish(&mut pack, city)?;

    for tile in plan.exits.into_iter().chain([plan.arrival]) {
        authoring::warp(&mut lab, (tile[0], tile[1]), "MN_ROCHESTER", return_index);
    }
    authoring::npc(
        base,
        &mut lab,
        "MnLabFossilResearcher",
        "SPRITE_SCIENTIST",
        i16::try_from(plan.researcher[0])?,
        i16::try_from(plan.researcher[1])?,
    )?;
    authoring::speech(
        base,
        &mut lab,
        "Assistant",
        "SPRITE_SCIENTIST",
        (
            i16::try_from(plan.assistant[0])?,
            i16::try_from(plan.assistant[1])?,
        ),
        &[
            "Bring a LENS or",
            "SPIRE FOSSIL from",
            "DRIFTLESS TRAIL.",
            "Our scientist can",
            "restore it.",
        ],
    )?;
    sign(
        &mut lab,
        plan.bench,
        "MnLabFossilBench",
        &["RESTORATION BENCH", "LENS: TRILUME", "SPIRE: SIPHLET"],
    );
    crate::ely_league_rooms::finish(&pack, &mut lab)?;
    for tile in [plan.researcher, plan.assistant, plan.arrival] {
        ensure!(
            crate::authored_town_geometry::kind(base, &lab, tile)? == "FLOOR",
            "lab actor/arrival must use native floor"
        );
    }
    ensure!(
        crate::authored_town_geometry::kind(base, &lab, plan.bench)? == "WALL",
        "bench reader needs a solid target"
    );
    let indoors = crate::authored_town_geometry::reachable(base, &lab, plan.arrival, false)?;
    ensure!(
        plan.exits.iter().all(|tile| indoors.contains(tile))
            && indoors.contains(&[plan.researcher[0], plan.researcher[1] + 1])
            && indoors.contains(&[plan.bench[0], plan.bench[1] + 1]),
        "lab exits, researcher and bench must be reachable"
    );
    install_room(&mut pack, lab, &plan)?;

    let mut bluff = pack
        .data
        .maps
        .get("MnR01C00R00")
        .context("fossil bluff")?
        .clone();
    authoring::text(
        &mut bluff,
        "MnFossilDirectionsText",
        &[
            "Bring your fossil",
            "to ROCHESTER's",
            "FOSSIL LAB beside",
            "the clinic.",
            "Earn the CARE BADGE",
            "and leave room for",
            "your new friend.",
        ],
    );
    crate::northstar_rival_arc::publish(&mut pack, bluff)?;
    pack.report.maps = pack.data.maps.len();
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

fn rename_local_labels(value: &mut Value) {
    match value {
        Value::String(text) if text.starts_with("MnFossil") => {
            *text = text.replacen("MnFossil", "MnLabFossil", 1)
        }
        Value::Array(values) => {
            for value in values {
                rename_local_labels(value);
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                rename_local_labels(value);
            }
        }
        _ => {}
    }
}

fn sign(map: &mut MapModule, tile: [u16; 2], label: &str, lines: &[&str]) {
    let text = format!("{label}Text");
    authoring::text(map, &text, lines);
    map.scripts
        .insert(label.into(), serde_json::json!([c("jumptext", &[&text])]));
    map.events.bg_events.push(BackgroundEvent {
        x: tile[0],
        y: tile[1],
        event_type: "BGEVENT_READ".into(),
        script: label.into(),
    });
}

fn install_room(pack: &mut CompiledGamePack, room: MapModule, plan: &Plan) -> Result<()> {
    let mut metadata = pack.data.runtime_map_metadata["MN_ROCHESTER"].clone();
    ensure!(
        metadata.group_id == plan.native_group_id,
        "Rochester group changed"
    );
    metadata.constant = CONSTANT.into();
    metadata.name = LAB.into();
    metadata.map_id = plan.native_map_id;
    metadata.width = 5;
    metadata.height = 5;
    metadata.environment = room
        .attributes
        .environment
        .clone()
        .context("lab environment")?;
    metadata.phone_service = room.attributes.phone_service;
    let spawn = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|s| s.identifier)
        .max()
        .context("spawn table")?
        .checked_add(1)
        .context("spawn overflow")?;
    crate::insert_compiled_map_extension(
        pack,
        CompiledMapExtension {
            manifest_id: MANIFEST.into(),
            map_name: LAB.into(),
            map_constant: CONSTANT.into(),
            spawn_key: spawn.to_string(),
            spawn: RuntimeSpawnPoint {
                identifier: spawn,
                map_constant: CONSTANT.into(),
                map_name: LAB.into(),
                group_name: metadata.group_name.clone(),
                group_id: i16::try_from(plan.native_group_id)?,
                map_id: i16::try_from(plan.native_map_id)?,
                tile_x: i16::try_from(plan.arrival[0])?,
                tile_y: i16::try_from(plan.arrival[1])?,
                metatile_x: i16::try_from(plan.arrival[0] / 2)?,
                metatile_y: i16::try_from(plan.arrival[1] / 2)?,
                subtile_x: i16::try_from(plan.arrival[0] % 2)?,
                subtile_y: i16::try_from(plan.arrival[1] % 2)?,
            },
            metadata,
            module: room,
            wild_encounters: None,
            start_new_game_here: false,
        },
    )?;
    pack.data.runtime_spawn_points.remove(&spawn.to_string());
    pack.data
        .pokegear_landmarks
        .map_to_landmark
        .insert(LAB.into(), "LANDMARK_MN_ROCHESTER".into());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::build_minnesota_fossil_lab;

    #[test]
    #[ignore = "requires CRYSTAL_FOSSIL_LAB_PARENT external current Minnesota pack"]
    fn fossil_lab_relocates_only_the_service_and_rejects_bad_contracts() {
        let path = std::env::var_os("CRYSTAL_FOSSIL_LAB_PARENT").expect("external parent");
        let base = crate::read_verified_compiled_game_pack(path).expect("verified parent");
        let source = include_str!("../../../planning/minnesota/landmarks/fossil-lab-v01/lab.json");
        let child = build_minnesota_fossil_lab(&base, source).expect("lab child");
        assert_eq!(child.data.maps.len(), base.data.maps.len() + 1);
        for (name, map) in &base.data.maps {
            if name != "MnRochester" && name != "MnR01C00R00" {
                assert_eq!(map, &child.data.maps[name], "unrelated map {name}");
            }
        }
        assert_eq!(
            child.runtime_files, base.runtime_files,
            "all native art/audio sources"
        );
        let original = serde_json::to_value(&base.data).expect("parent data");
        let mut after = serde_json::to_value(&child.data).expect("child data");
        for key in [
            "maps",
            "map_attributes",
            "map_dimensions",
            "map_blocks",
            "map_scripts",
            "npcs",
            "runtime_map_metadata",
            "pokegear_landmarks",
        ] {
            after[key] = original[key].clone();
        }
        assert_eq!(
            original, after,
            "other catalogs, flags, encounters, audio and recovery remain exact"
        );
        let lab = &child.data.maps["MnRochesterFossilLab"];
        assert_eq!(lab.gift_pokemon_scripts.len(), 2);
        assert!(
            child.data.maps["MnRochester"]
                .gift_pokemon_scripts
                .is_empty(),
            "no obsolete outdoor revival program"
        );
        for (label, body) in base.data.maps["MnRochester"]
            .scripts
            .iter()
            .filter(|(s, _)| s.starts_with("MnFossil"))
        {
            let mut expected = body.clone();
            super::rename_local_labels(&mut expected);
            assert_eq!(
                lab.scripts[&label.replacen("MnFossil", "MnLabFossil", 1)],
                expected,
                "existing retry/claim rules {label}"
            );
        }
        assert!(build_minnesota_fossil_lab(&child, source).is_err());
        for (field, value) in [
            ("parent_content_hash", serde_json::json!("stale")),
            ("native_map_id", serde_json::json!(6)),
            ("researcher", serde_json::json!([0, 0])),
            ("arrival", serde_json::json!([3, 4])),
            ("exits", serde_json::json!([[0, 0], [5, 9]])),
        ] {
            let mut bad: serde_json::Value = serde_json::from_str(source).expect("source");
            bad[field] = value;
            assert!(
                build_minnesota_fossil_lab(&base, &bad.to_string()).is_err(),
                "reject bad {field}"
            );
        }
    }
}
