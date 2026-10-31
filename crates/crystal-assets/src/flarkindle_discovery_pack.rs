//! Native, reversible-travel fen branch and acknowledged terrain restoration.
use anyhow::{Context, Result, ensure};
use crystal_core::map::MapConnection;
use serde_json::Value;

use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::northstar_lair_authoring as a;
use crate::northstar_rival_arc as arc;
use crate::{CompiledGamePack, MapModule};

const PARENT: &str = "MnBemidjiNorthwoods";
const FEN: &str = "MnLayeredFen";
const CONSTANT: &str = "MN_LAYERED_FEN";
const MARKER: &str = "minnesota-flarkindle-layered-fen-v1";

/// Adds the optional root/pool/peat investigation beside the northwoods corridor.
/// The original north/south corridor and any eastern tall-tale branch remain
/// intact. A western walking connection reaches six distinct observation
/// anchors. Acknowledging the ordered sequence opens only the hollow's native
/// terrain; both ordinary walking and Surf retain a safe way home. The tiles
/// callback restores that opening on Continue and physical return. No art,
/// audio, existing species or existing encounter table is changed.
/// The caller owns controller, recovery, storage, MCP and native visual review.
/// The borrowed pack and disk remain unchanged.
///
/// # Errors
/// Rejects duplicate installation, changed/occupied parent approaches, foreign
/// species, unreviewed rows or anchors, incidental Cut/buoy terrain, bypassable
/// gates, unconnected edges, program collisions and invalid resulting packs.
pub fn build_flarkindle_discovery_pack(
    base: &CompiledGamePack,
    quests: &str,
    site: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    crate::validate_compact_superior_geography(base)?;
    let source: Value = serde_json::from_str(site)?;
    ensure!(
        source["schema_version"] == 1
            && source["id"] == "minnesota-flarkindle-fen-site-v1"
            && source["parent"] == PARENT
            && source["fen"]["map"] == FEN
            && source["fen"]["constant"] == CONSTANT
            && source["fen"]["dimensions_metatiles"] == serde_json::json!([20, 13])
            && source["parent_opening"]["west_offset"] == 0
            && source["fen"]["east_offset"] == 0,
        "reviewed fen source required"
    );
    ensure!(
        !base.data.maps.contains_key(FEN) && !base.report.manifests.iter().any(|m| m == MARKER),
        "fen already installed"
    );
    ensure!(
        base.data.pokemon.contains_key("FLARKINDLE")
            && base
                .data
                .story_event_script_constants
                .global
                .contains_key("MN_DEX_MEMBER_FLARKINDLE")
            && base
                .runtime_files
                .contains_key("gfx/pokemon/flarkindle/overworld.2bpp"),
        "active original Flarkindle/native field bank required"
    );
    let mut outside = arc::map(base, PARENT)?;
    ensure!(
        outside.attributes.width == 8
            && outside.attributes.height == 14
            && outside.attributes.tileset_name == "johto"
            && outside.blocks[8 * 8..8 * 8 + 3] == [5, 5, 1]
            && !outside
                .attributes
                .connections
                .iter()
                .any(|c| c.direction.eq_ignore_ascii_case("west")),
        "reviewed west corridor preimage required"
    );
    for x in 0..6 {
        for y in [16, 17] {
            vacant(&outside, [x, y])?;
        }
    }
    let original_encounters = base
        .data
        .wild_encounters
        .get(PARENT)
        .context("native northwoods habitat required")?;
    validate_encounter_species(base, &serde_json::to_value(original_encounters)?)?;
    let mut programs = crate::build_flarkindle_discovery_programs(quests)?;
    ensure!(
        programs.terrain_hook == "MnFenOpenPath",
        "native fen hook identity"
    );
    for (label, commands) in [
        (
            "MnFenRestoreTerrain",
            vec![
                c("checkevent", &["EVENT_MN_FEN_FOLDED_PATH_REVEALED"]),
                c("iffalse", &["MnFenTerrainEnd"]),
                c("changeblock", &["26", "8", "1"]),
                c("end", &[]),
            ],
        ),
        ("MnFenTerrainEnd", vec![c("end", &[])]),
        (
            "MnFenOpenPath",
            vec![
                c("scall", &["MnFenRestoreTerrain"]),
                c("refreshmap", &[]),
                c("end", &[]),
            ],
        ),
        (
            "MnFenTiles",
            vec![c("scall", &["MnFenRestoreTerrain"]), c("endcallback", &[])],
        ),
    ] {
        ensure!(
            programs
                .scripts
                .insert(label.into(), Value::Array(commands))
                .is_none(),
            "reserved terrain hook collision"
        );
    }
    let mut pack = base.clone();
    for flag in &programs.event_flags {
        a::flag(&mut pack, flag)?;
    }
    let mut fen = authoring::blank(base, PARENT, FEN, CONSTANT)?;
    let expected = reviewed_rows();
    let rows = source["fen"]["rows"]
        .as_array()
        .context("fen rows")?
        .iter()
        .map(|r| r.as_str().context("fen row"))
        .collect::<Result<Vec<_>>>()?;
    ensure!(rows == expected, "reviewed native fen rows required");
    authoring::blocks(&mut fen, 20, 13, &rows.join("\n"))?;
    fen.attributes.border_block = 5;
    fen.attributes.connections.push(MapConnection {
        direction: "east".into(),
        target_map: PARENT.into(),
        offset: 0,
    });
    fen.attributes.connection_flags = Some("EAST".into());
    for (key, label, at, approach, kind) in [
        ("root", "MnFenReadRoot", [12, 12], [11, 12], "HEADBUTT_TREE"),
        ("pool", "MnFenReadPool", [12, 18], [11, 18], "WATER"),
        ("rib", "MnFenReadRib", [30, 12], [29, 12], "WALL"),
        (
            "choose_root",
            "MnFenChooseRoot",
            [12, 15],
            [11, 15],
            "HEADBUTT_TREE",
        ),
        (
            "choose_pool",
            "MnFenChoosePool",
            [12, 20],
            [11, 20],
            "WATER",
        ),
        ("choose_rib", "MnFenChooseRib", [28, 14], [27, 14], "WALL"),
    ] {
        ensure!(
            point(&source["fen"]["reads"][key]["at"])? == at
                && point(&source["fen"]["reads"][key]["approach"])? == approach,
            "reviewed fen read: {key}"
        );
        ensure!(
            crate::authored_town_geometry::kind(base, &fen, at)? == kind
                && crate::authored_town_geometry::kind(base, &fen, approach)? == "FLOOR",
            "native fen clue collision: {key}"
        );
        a::read(&mut fen, at, label);
    }
    ensure!(
        point(&source["fen"]["observer"])? == [36, 17]
            && point(&source["fen"]["creature"])? == [32, 5]
            && point(&source["fen"]["gate"]["metatile"])? == [13, 4],
        "reviewed fen actors/gate required"
    );
    a::actor(
        base,
        &mut fen,
        "MnFenObserver",
        "SPRITE_SCIENTIST",
        [36, 17],
    )?;
    a::actor(
        base,
        &mut fen,
        "MN_FEN_FLARKINDLE",
        "SPRITE_FLARKINDLE",
        [32, 5],
    )?;
    a::callback(&mut fen, "MAPCALLBACK_OBJECTS", &programs.callbacks[0]);
    a::callback(&mut fen, "MAPCALLBACK_NEWMAP", &programs.callbacks[1]);
    a::callback(&mut fen, "MAPCALLBACK_TILES", "MnFenTiles");
    let closed = crate::authored_town_geometry::reachable(base, &fen, [39, 16], false)?;
    for at in [
        [35, 17],
        [11, 12],
        [11, 18],
        [29, 12],
        [11, 15],
        [11, 20],
        [27, 14],
    ] {
        ensure!(
            closed.contains(&at),
            "unreachable ordinary fen read/observer"
        );
    }
    ensure!(!closed.contains(&[32, 6]), "hollow must begin gated");
    let closed_surf = crate::authored_town_geometry::reachable(base, &fen, [39, 16], true)?;
    ensure!(
        !closed_surf.contains(&[32, 6]),
        "Surf cannot bypass the hollow gate"
    );
    let mut open = fen.clone();
    open.blocks[4 * 20 + 13] = 1;
    ensure!(
        crate::authored_town_geometry::reachable(base, &open, [39, 16], false)?.contains(&[32, 6]),
        "acknowledged opening must connect the hollow"
    );
    for map in [&fen, &open] {
        for y in 0..map.attributes.height * 2 {
            for x in 0..map.attributes.width * 2 {
                let kind = crate::authored_town_geometry::kind(base, map, [x, y])?;
                ensure!(
                    kind != "CUT_TREE" && kind != "BUOY",
                    "no incidental Cut/buoy terrain"
                );
                if (x == 0
                    || x + 1 == map.attributes.width * 2
                    || y == 0
                    || y + 1 == map.attributes.height * 2)
                    && !(x == 39 && (y == 16 || y == 17))
                {
                    ensure!(kind == "WALL", "closed fen perimeter required");
                }
            }
        }
    }
    let roots = [
        "MnFenObserver",
        "MnFenReadRoot",
        "MnFenReadPool",
        "MnFenReadRib",
        "MnFenChooseRoot",
        "MnFenChoosePool",
        "MnFenChooseRib",
        "MN_FEN_FLARKINDLE",
        "MnFenObjects",
        "MnFenNewVisit",
        "MnFenTiles",
    ]
    .into_iter()
    .map(String::from)
    .collect::<Vec<_>>();
    for label in crate::discovery_dialogue::script_closure(&programs.scripts, &roots)? {
        ensure!(!fen.scripts.contains_key(&label), "occupied fen program");
        fen.scripts
            .insert(label.clone(), programs.scripts[&label].clone());
    }
    fen.scripted_wild_battles = crate::parse_scripted_wild_battles(FEN, &fen.scripts)?;
    for x in [0, 1] {
        outside.blocks[8 * 8 + x] = 1;
    }
    outside.attributes.connections.push(MapConnection {
        direction: "west".into(),
        target_map: FEN.into(),
        offset: 0,
    });
    let flags = outside
        .attributes
        .connection_flags
        .as_ref()
        .context("existing corridor connections")?;
    outside.attributes.connection_flags = Some(format!("{flags} | WEST"));
    arc::publish(&mut pack, outside)?;
    a::publish(&mut pack, fen, PARENT, [39, 16])?;
    let mut encounters = original_encounters.clone();
    encounters.map_name = FEN.into();
    pack.data.wild_encounters.insert(FEN.into(), encounters);
    if let Some(original) = base.data.field_encounters.get(PARENT) {
        let mut encounters = original.clone();
        encounters.map_name = FEN.into();
        pack.data.field_encounters.insert(FEN.into(), encounters);
    }
    for (name, map) in &base.data.maps {
        if name != PARENT {
            ensure!(
                pack.data.maps.get(name) == Some(map),
                "unrelated fen map: {name}"
            );
        }
    }
    ensure!(
        pack.runtime_files == base.runtime_files
            && pack.audio_manifest == base.audio_manifest
            && pack.compiled_audio == base.compiled_audio
            && pack.data.pokemon == base.data.pokemon,
        "original art/audio/species must remain exact"
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

fn reviewed_rows() -> [&'static str; 13] {
    // The loop stays inside the empty pocket above R14, without moving any
    // existing world map or changing the six clue/creature/entry anchors.
    [
        "05 05 05 05 05 05 05 05 05 05 05 05 05 05 05 05 05 05 05 05",
        "05 05 05 05 05 05 05 05 05 05 05 05 05 05 02 02 02 02 05 05",
        "05 05 05 05 05 05 05 05 05 05 05 05 05 05 04 04 04 04 05 05",
        "05 05 05 05 05 05 05 05 05 05 05 05 05 05 04 04 04 04 05 05",
        "05 05 05 05 05 05 05 05 05 05 05 01 01 05 01 01 01 01 05 05",
        "05 05 05 05 05 05 02 02 02 05 05 02 02 05 05 05 05 05 05 05",
        "05 05 05 05 03 02 61 02 02 05 05 02 02 02 02 68 02 05 05 05",
        "05 05 05 02 03 02 61 02 02 02 02 02 02 02 68 02 02 05 05 05",
        "05 05 05 02 02 02 02 02 02 01 01 01 01 01 01 01 01 01 01 01",
        "05 05 05 02 02 02 54 55 02 02 02 02 05 02 02 02 02 05 05 05",
        "05 05 05 02 02 02 58 59 02 02 02 02 05 02 02 03 02 05 05 05",
        "05 05 05 02 02 02 02 02 02 02 02 02 01 02 02 03 02 05 05 05",
        "05 05 05 05 05 05 05 05 05 05 05 05 05 05 05 05 05 05 05 05",
    ]
}
fn point(value: &Value) -> Result<[u16; 2]> {
    serde_json::from_value(value.clone()).context("fen coordinate")
}
fn vacant(map: &MapModule, at: [u16; 2]) -> Result<()> {
    ensure!(
        !map.objects.iter().any(|o| [o.x, o.y] == at)
            && !map.events.warps.iter().any(|e| [e.x, e.y] == at)
            && !map.events.coord_events.iter().any(|e| [e.x, e.y] == at)
            && !map.events.bg_events.iter().any(|e| [e.x, e.y] == at),
        "occupied fen approach: {at:?}"
    );
    Ok(())
}

fn validate_encounter_species(pack: &CompiledGamePack, value: &Value) -> Result<()> {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                if key == "species" {
                    let species = value.as_str().context("native encounter species")?;
                    ensure!(
                        pack.data.pokemon.contains_key(species)
                            && pack
                                .data
                                .story_event_script_constants
                                .global
                                .contains_key(&format!("MN_DEX_MEMBER_{species}")),
                        "active Minnesota fen encounter required: {species}"
                    );
                } else {
                    validate_encounter_species(pack, value)?;
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                validate_encounter_species(pack, value)?;
            }
        }
        _ => {}
    }
    Ok(())
}
