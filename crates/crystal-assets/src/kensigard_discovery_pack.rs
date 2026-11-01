//! Guarded, native prairie field-margin branch for the moving-stone mystery.
use anyhow::{Context, Result, ensure};
use crystal_core::map::{CoordEvent, MapConnection};
use serde_json::Value;

use crate::grand_marais_modpack::authoring;
use crate::northstar_lair_authoring as a;
use crate::northstar_rival_arc as arc;
use crate::{CompiledGamePack, MapModule};

const PARENT: &str = "MnR16C01R00";
const FIELD: &str = "MnFieldMargin";
const CONSTANT: &str = "MN_FIELD_MARGIN";
const MARKER: &str = "minnesota-kensigard-field-margin-v1";

/// Adds a voluntary field investigation reached by walking north from R16.
/// Existing route travel, encounters and actors remain intact. The new branch
/// uses the parent's native art and grass encounter table. The caretaker stays
/// outside, making the changed-grooves revisit an actual journey. Only capture
/// resolves the optional encounter; ordinary return travel remains open.
/// The borrowed pack and disk remain unchanged. The caller must verify native
/// traversal, clue acknowledgment, Continue, recovery, storage and presentation.
///
/// # Errors
/// Rejects duplicate installation, changed parent geometry, occupied anchors,
/// foreign or retired encounters, missing native art, unreviewed field rows,
/// incidental Cut gates, open borders, broken connections or invalid output.
pub fn build_kensigard_discovery_pack(
    base: &CompiledGamePack,
    quests: &str,
    site: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    crate::validate_compact_superior_geography(base)?;
    ensure!(
        !base.data.maps.contains_key(FIELD) && !base.report.manifests.iter().any(|m| m == MARKER),
        "field branch already installed"
    );
    let source: Value = serde_json::from_str(site)?;
    ensure!(
        source["schema_version"] == 1
            && source["id"] == "minnesota-kensigard-field-margin-site-v1"
            && source["parent"] == PARENT
            && source["field"]["map"] == FIELD
            && source["field"]["constant"] == CONSTANT
            && source["field"]["dimensions_metatiles"] == serde_json::json!([14, 16])
            && source["parent_opening"]["north_offset"] == 3
            && source["field"]["south_offset"] == -3,
        "reviewed field source required"
    );
    active(base, "KENSIGARD")?;
    ensure!(
        base.runtime_files
            .contains_key("gfx/pokemon/kensigard/overworld.2bpp"),
        "original field sprite required"
    );
    let mut outside = arc::map(base, PARENT)?;
    ensure!(
        outside.attributes.width == 16
            && outside.attributes.height == 24
            && outside.attributes.tileset_name == "mn_prairie_headwaters"
            && !outside
                .attributes
                .connections
                .iter()
                .any(|c| c.direction.eq_ignore_ascii_case("north"))
            && outside.blocks[11] == 5
            && outside.blocks[16 + 11] == 2,
        "reviewed R16 branch preimage required"
    );
    ensure!(
        point(&source["parent_opening"]["metatile"])? == [11, 0]
            && source["parent_opening"]["before"] == "05"
            && source["parent_opening"]["after"] == "02"
            && point(&source["caretaker"]["at"])? == [23, 5]
            && point(&source["caretaker"]["approach"])? == [23, 6],
        "reviewed outside anchors required"
    );
    for at in [[22, 2], [23, 2], [23, 5], [23, 6]] {
        vacant(&outside, at)?;
        ensure!(
            crate::authored_town_geometry::kind(base, &outside, at)? == "FLOOR",
            "dry outside approach"
        );
    }
    let encounters = base
        .data
        .wild_encounters
        .get(PARENT)
        .context("R16 native encounters")?;
    let encounter_value = serde_json::to_value(encounters)?;
    validate_species(base, &encounter_value)?;
    let programs = crate::build_kensigard_discovery_programs(quests)?;
    let mut pack = base.clone();
    for flag in &programs.event_flags {
        a::flag(&mut pack, flag)?;
    }
    let mut field = authoring::blank(base, PARENT, FIELD, CONSTANT)?;
    let expected = [
        "05 05 05 05 05 05 05 05 05 05 05 05 05 05",
        "05 05 05 05 02 02 02 05 05 05 05 05 05 05",
        "05 05 05 02 02 02 02 02 05 05 05 05 05 05",
        "05 05 02 02 0a 02 02 02 02 05 05 05 05 05",
        "05 05 02 02 02 02 68 71 02 02 05 05 05 05",
        "05 05 02 02 0a 02 71 71 02 02 02 05 05 05",
        "05 05 05 02 02 02 71 71 02 02 02 05 05 05",
        "05 05 05 02 84 84 02 02 84 84 84 02 05 05",
        "05 05 02 02 84 84 02 02 84 84 84 02 05 05",
        "05 05 02 02 84 84 02 02 84 84 84 02 05 05",
        "05 05 02 02 84 84 02 71 71 02 02 02 05 05",
        "05 05 05 02 02 02 02 68 71 02 02 02 05 05",
        "05 05 05 05 02 02 02 71 71 02 02 05 05 05",
        "05 05 05 05 05 02 02 02 02 02 05 05 05 05",
        "05 05 05 05 05 05 05 02 02 05 05 05 05 05",
        "05 05 05 05 05 05 05 05 02 05 05 05 05 05",
    ];
    let rows = source["field"]["rows"]
        .as_array()
        .context("field rows")?
        .iter()
        .map(|r| r.as_str().context("field row"))
        .collect::<Result<Vec<_>>>()?;
    ensure!(rows == expected, "reviewed native field rows required");
    authoring::blocks(&mut field, 14, 16, &rows.join("\n"))?;
    field.attributes.border_block = 5;
    field.attributes.connections.push(MapConnection {
        direction: "south".into(),
        target_map: PARENT.into(),
        offset: -3,
    });
    field.attributes.connection_flags = Some("SOUTH".into());
    for (key, label, at, approach, kind) in [
        ("soil", "MnKensigardSoil", [12, 8], [11, 8], "WALL"),
        ("buried_edge", "MnKensigardEdge", [8, 7], [8, 8], "WALL"),
        ("grooves", "MnKensigardGrooves", [14, 22], [13, 22], "WALL"),
        ("boundary", "MnKensigardBoundary", [9, 10], [10, 10], "WALL"),
    ] {
        ensure!(
            point(&source["field"]["reads"][key]["at"])? == at
                && point(&source["field"]["reads"][key]["approach"])? == approach,
            "reviewed field read: {key}"
        );
        ensure!(
            crate::authored_town_geometry::kind(base, &field, at)? == kind
                && crate::authored_town_geometry::kind(base, &field, approach)? == "FLOOR",
            "native field read collision: {key}"
        );
        a::read(&mut field, at, label);
    }
    ensure!(
        point(&source["field"]["creature"])? == [13, 5]
            && point(&source["field"]["reveal_coordinate"])? == [13, 6],
        "reviewed creature/reveal required"
    );
    a::actor(
        base,
        &mut field,
        "MN_MARGIN_KENSIGARD",
        "SPRITE_KENSIGARD",
        [13, 5],
    )?;
    field.events.coord_events.push(CoordEvent {
        x: 13,
        y: 6,
        scene_id: "-1".into(),
        script_name: "MnKensigardReveal".into(),
    });
    a::callback(&mut field, "MAPCALLBACK_OBJECTS", &programs.callbacks[0]);
    a::callback(&mut field, "MAPCALLBACK_NEWMAP", &programs.callbacks[1]);
    a::actor(
        base,
        &mut outside,
        "MnKensigardCaretaker",
        "SPRITE_POKEFAN_M",
        [23, 5],
    )?;
    outside.blocks[11] = 2;
    outside.attributes.connections.push(MapConnection {
        direction: "north".into(),
        target_map: FIELD.into(),
        offset: 3,
    });
    let flags = outside
        .attributes
        .connection_flags
        .as_ref()
        .context("existing route connections")?;
    outside.attributes.connection_flags = Some(format!("{flags} | NORTH"));
    for (map, roots) in [
        (&mut outside, vec!["MnKensigardCaretaker"]),
        (
            &mut field,
            vec![
                "MnKensigardSoil",
                "MnKensigardEdge",
                "MnKensigardGrooves",
                "MnKensigardBoundary",
                "MnKensigardReveal",
                "MN_MARGIN_KENSIGARD",
                "MnKensigardObjects",
                "MnKensigardNewVisit",
            ],
        ),
    ] {
        let roots = roots.into_iter().map(String::from).collect::<Vec<_>>();
        for label in crate::discovery_dialogue::script_closure(&programs.scripts, &roots)? {
            ensure!(
                !map.scripts.contains_key(&label),
                "occupied field program: {label}"
            );
            map.scripts
                .insert(label.clone(), programs.scripts[&label].clone());
        }
        map.scripted_wild_battles = crate::parse_scripted_wild_battles(&map.id, &map.scripts)?;
    }
    for y in 0..32 {
        for x in 0..28 {
            let kind = crate::authored_town_geometry::kind(base, &field, [x, y])?;
            ensure!(
                kind != "CUT_TREE" && kind != "WATER" && kind != "BUOY",
                "unsupported field terrain"
            );
            if (x == 0 || x == 27 || y == 0 || y == 31) && !(y == 31 && (x == 16 || x == 17)) {
                ensure!(kind == "WALL", "closed field border required");
            }
        }
    }
    arc::publish(&mut pack, outside)?;
    a::publish(&mut pack, field, PARENT, [16, 31])?;
    let mut field_encounters = encounters.clone();
    field_encounters.map_name = FIELD.into();
    pack.data
        .wild_encounters
        .insert(FIELD.into(), field_encounters);
    for (name, map) in &base.data.maps {
        if name != PARENT {
            ensure!(
                pack.data.maps.get(name) == Some(map),
                "unrelated field map changed: {name}"
            );
        }
    }
    ensure!(
        pack.runtime_files == base.runtime_files
            && pack.audio_manifest == base.audio_manifest
            && pack.compiled_audio == base.compiled_audio
            && pack.data.pokemon == base.data.pokemon,
        "original species/art/audio must remain exact"
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
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}

fn active(pack: &CompiledGamePack, species: &str) -> Result<()> {
    ensure!(
        pack.data.pokemon.contains_key(species)
            && pack
                .data
                .story_event_script_constants
                .global
                .contains_key(&format!("MN_DEX_MEMBER_{species}")),
        "active original Minnesota species required: {species}"
    );
    Ok(())
}
fn validate_species(pack: &CompiledGamePack, value: &Value) -> Result<()> {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                if key == "species" {
                    active(pack, value.as_str().context("encounter species")?)?;
                } else {
                    validate_species(pack, value)?;
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                validate_species(pack, value)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn point(value: &Value) -> Result<[u16; 2]> {
    serde_json::from_value(value.clone()).context("field coordinate")
}
fn vacant(map: &MapModule, at: [u16; 2]) -> Result<()> {
    ensure!(
        !map.objects.iter().any(|o| [o.x, o.y] == at)
            && !map.events.warps.iter().any(|e| [e.x, e.y] == at)
            && !map.events.coord_events.iter().any(|e| [e.x, e.y] == at)
            && !map.events.bg_events.iter().any(|e| [e.x, e.y] == at),
        "occupied field anchor: {at:?}"
    );
    Ok(())
}
