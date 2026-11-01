//! Three native former shores and a separately bounded fictional inner basin.
use anyhow::{Context, Result, ensure};
use crystal_core::map::MapConnection;
use serde_json::Value;

use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::northstar_lair_authoring as a;
use crate::northstar_rival_arc as arc;
use crate::{CompiledGamePack, MapModule};

const PARENT: &str = "MnR16C00R00";
const RIDGE: &str = "MnFormerShores";
const BASIN: &str = "MnInnerBasin";
const RIDGE_CONSTANT: &str = "MN_FORMER_SHORES";
const BASIN_CONSTANT: &str = "MN_INNER_BASIN";
const MARKER: &str = "minnesota-strandrak-former-shores-v1";

/// Adds an optional shore survey and native cave control puzzle off R16.
/// Three exterior terraces have ordinary returns. A separate cave contains
/// three independently acknowledged indicators and a lever. Its solution opens
/// exactly eight local water blocks and one rock sill; all existing Minnesota
/// waters stay exact. Tiles callbacks restore the acknowledged cave state on
/// Continue and entry. A voluntary level-60 encounter follows the solution.
/// Original route actors/events, species, art and audio remain unchanged.
/// The caller must accept production travel, controls, storage, recovery, MCP
/// and native presentation. The borrowed pack and disk remain unchanged.
///
/// # Errors
/// Rejects changed parent approaches, duplicate publication, noncanonical
/// geometry/anchors, missing original species/art/habitat, incidental Cut/buoy
/// or pit terrain, a bypassable sill, unsafe travel and invalid resulting packs.
pub fn build_strandrak_discovery_pack(
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
            && source["id"] == "minnesota-strandrak-former-shores-site-v1"
            && source["parent"] == PARENT
            && source["ridge"]["map"] == RIDGE
            && source["basin"]["map"] == BASIN
            && source["ridge"]["dimensions_metatiles"] == serde_json::json!([18, 14])
            && source["basin"]["dimensions_metatiles"] == serde_json::json!([14, 14])
            && source["parent_opening"]["south_offset"] == 14
            && source["ridge"]["north_offset"] == -14,
        "reviewed former-shores source required"
    );
    ensure!(
        !base.data.maps.contains_key(RIDGE)
            && !base.data.maps.contains_key(BASIN)
            && !base.report.manifests.iter().any(|m| m == MARKER),
        "former shores already installed"
    );
    ensure!(
        base.data.pokemon.contains_key("STRANDRAK")
            && base
                .data
                .story_event_script_constants
                .global
                .contains_key("MN_DEX_MEMBER_STRANDRAK")
            && base
                .runtime_files
                .contains_key("gfx/pokemon/strandrak/overworld.2bpp"),
        "active original Strandrak and native field bank required"
    );
    let mut outside = arc::map(base, PARENT)?;
    ensure!(
        outside.attributes.width == 32
            && outside.attributes.height == 24
            && outside.attributes.tileset_name == "mn_prairie_headwaters"
            && outside.blocks[23 * 32 + 21] == 5
            && outside.blocks[22 * 32 + 21] == 0x84
            && !outside
                .attributes
                .connections
                .iter()
                .any(|c| c.direction.eq_ignore_ascii_case("south")),
        "reviewed prairie south preimage required"
    );
    for x in [42, 43] {
        for y in [44, 45, 46, 47] {
            vacant(&outside, [x, y])?;
        }
    }
    let encounters = base
        .data
        .wild_encounters
        .get(PARENT)
        .context("original prairie habitat required")?;
    validate_species(base, &serde_json::to_value(encounters)?)?;
    let mut programs = crate::build_strandrak_discovery_programs(quests)?;
    ensure!(
        programs.terrain_hook == "MnBasinOpenInnerOutlet",
        "native basin terrain hook required"
    );
    let mut pack = base.clone();
    for flag in &programs.event_flags {
        a::flag(&mut pack, flag)?;
    }
    let mut ridge = authoring::blank(base, PARENT, RIDGE, RIDGE_CONSTANT)?;
    let mut basin = authoring::blank(base, PARENT, BASIN, BASIN_CONSTANT)?;
    for (map, key, width, expected) in [
        (&mut ridge, "ridge", 18, reviewed_ridge_rows()),
        (&mut basin, "basin", 14, reviewed_basin_rows()),
    ] {
        let rows = source[key]["rows"]
            .as_array()
            .context("former-shores rows")?
            .iter()
            .map(|r| r.as_str().context("native row"))
            .collect::<Result<Vec<_>>>()?;
        ensure!(rows == expected, "reviewed native terrain: {key}");
        authoring::blocks(map, width, 14, &rows.join("\n"))?;
    }
    ridge.attributes.border_block = 5;
    ridge.attributes.connections.push(MapConnection {
        direction: "north".into(),
        target_map: PARENT.into(),
        offset: -14,
    });
    ridge.attributes.connection_flags = Some("NORTH".into());
    basin.attributes.tileset_name = "cave".into();
    basin.attributes.border_block = 0x1d;
    let native_cave = base
        .data
        .maps
        .get("SlowpokeWellB1F")
        .context("native cave presentation template")?;
    basin.attributes.environment = native_cave.attributes.environment.clone();
    basin.attributes.music = native_cave.attributes.music.clone();
    basin.attributes.palette = native_cave.attributes.palette.clone();
    ensure!(
        basin.attributes.palette.as_deref() == Some("PALETTE_NITE"),
        "readable native cave palette required"
    );
    for (key, label, at, approach) in [
        ("upper_shore", "MnBasinHighShore", [16, 8], [16, 9]),
        ("middle_shore", "MnBasinMiddleShore", [20, 14], [20, 15]),
        ("lower_shore", "MnBasinLowShore", [12, 20], [12, 21]),
    ] {
        ensure!(
            point(&source["ridge"]["reads"][key]["at"])? == at
                && point(&source["ridge"]["reads"][key]["approach"])? == approach,
            "reviewed shore anchor: {key}"
        );
        ensure!(
            crate::authored_town_geometry::kind(base, &ridge, at)? == "UP_WALL"
                && crate::authored_town_geometry::kind(base, &ridge, approach)? == "FLOOR",
            "native shore face and approach"
        );
        a::read(&mut ridge, at, label);
    }
    for (key, label, at, approach) in [
        ("upper", "MnBasinUpperDial", [9, 10], [10, 10]),
        ("middle", "MnBasinCenterDial", [18, 14], [17, 14]),
        ("lower", "MnBasinLowerDial", [11, 18], [12, 18]),
        ("lever", "MnBasinLever", [14, 20], [15, 20]),
    ] {
        ensure!(
            point(&source["basin"]["controls"][key]["at"])? == at
                && point(&source["basin"]["controls"][key]["approach"])? == approach,
            "reviewed outlet control: {key}"
        );
        ensure!(
            crate::authored_town_geometry::kind(base, &basin, at)? == "WALL"
                && crate::authored_town_geometry::kind(base, &basin, approach)? == "FLOOR",
            "native outlet read/approach"
        );
        a::read(&mut basin, at, label);
    }
    ensure!(
        point(&source["ridge"]["surveyor"])? == [17, 5]
            && point(&source["basin"]["creature"])? == [14, 2],
        "reviewed surveyor/creature anchors"
    );
    a::actor(
        base,
        &mut ridge,
        "MnBasinSurveyor",
        "SPRITE_SCIENTIST",
        [17, 5],
    )?;
    a::actor(
        base,
        &mut basin,
        "MN_BASIN_STRANDRAK",
        "SPRITE_STRANDRAK",
        [14, 2],
    )?;
    for (map, at, kind) in [
        (&ridge, [18, 23], "CAVE"),
        (&ridge, [18, 24], "FLOOR"),
        (&basin, [3, 25], "LADDER"),
        (&basin, [2, 25], "FLOOR"),
    ] {
        ensure!(
            crate::authored_town_geometry::kind(base, map, at)? == kind,
            "native active/passive cave travel"
        );
    }
    authoring::warp(&mut ridge, (18, 23), BASIN_CONSTANT, 2);
    authoring::warp(&mut ridge, (18, 24), RIDGE_CONSTANT, 2);
    authoring::warp(&mut basin, (3, 25), RIDGE_CONSTANT, 2);
    authoring::warp(&mut basin, (2, 25), BASIN_CONSTANT, 2);
    let changes = [
        ([7, 3], 0x1d),
        ([4, 1], 0x0f),
        ([5, 1], 0x0f),
        ([9, 1], 0x0f),
        ([10, 1], 0x0f),
        ([4, 2], 0x0f),
        ([5, 2], 0x0f),
        ([9, 2], 0x0f),
        ([10, 2], 0x0f),
    ];
    let mut restore = vec![
        c("checkevent", &["EVENT_MN_BASIN_INNER_OUTLET_ACKNOWLEDGED"]),
        c("iffalse", &["MnBasinTerrainEnd"]),
    ];
    let authored = source["basin"]["terrain_changes"]
        .as_array()
        .context("inner terrain changes")?;
    ensure!(
        authored.len() == changes.len(),
        "only nine inner basin changes permitted"
    );
    let mut opened = basin.clone();
    for (([x, y], before), entry) in changes.into_iter().zip(authored) {
        ensure!(
            point(&entry["metatile"])? == [x, y]
                && entry["before"] == format!("{before:02x}")
                && entry["after"] == "02"
                && basin.blocks[usize::from(y) * 14 + usize::from(x)] == before,
            "reviewed local outlet preimage"
        );
        opened.blocks[usize::from(y) * 14 + usize::from(x)] = 2;
        restore.push(c(
            "changeblock",
            &[&(x * 2).to_string(), &(y * 2).to_string(), "2"],
        ));
    }
    restore.push(c("end", &[]));
    for (label, commands) in [
        ("MnBasinRestoreTerrain", restore),
        ("MnBasinTerrainEnd", vec![c("end", &[])]),
        (
            "MnBasinOpenInnerOutlet",
            vec![
                c("scall", &["MnBasinRestoreTerrain"]),
                c("refreshmap", &[]),
                c("end", &[]),
            ],
        ),
        (
            "MnBasinTiles",
            vec![
                c("scall", &["MnBasinRestoreTerrain"]),
                c("endcallback", &[]),
            ],
        ),
    ] {
        ensure!(
            programs
                .scripts
                .insert(label.into(), Value::Array(commands))
                .is_none(),
            "reserved basin terrain label"
        );
    }
    a::callback(&mut basin, "MAPCALLBACK_OBJECTS", &programs.callbacks[0]);
    a::callback(&mut basin, "MAPCALLBACK_NEWMAP", &programs.callbacks[1]);
    a::callback(&mut basin, "MAPCALLBACK_TILES", "MnBasinTiles");
    let reach = crate::authored_town_geometry::reachable(base, &ridge, [14, 0], false)?;
    // The native CAVE tile above the passive landing is a terminal warp,
    // separately checked above. The dry-floor flood must reach its adjacent
    // landing without treating the cave event as ordinary traversable floor.
    for at in [[17, 6], [16, 9], [20, 15], [12, 21], [18, 24]] {
        ensure!(
            reach.contains(&at),
            "ordinary terrace/cave return must remain open at {at:?}"
        );
    }
    let reach = crate::authored_town_geometry::reachable(base, &basin, [2, 25], true)?;
    for at in [[10, 10], [17, 14], [12, 18], [15, 20], [3, 25]] {
        ensure!(reach.contains(&at), "unreachable basin control/return");
    }
    ensure!(
        !reach.contains(&[14, 3]),
        "Surf must not bypass the closed sill"
    );
    ensure!(
        crate::authored_town_geometry::reachable(base, &opened, [2, 25], false)?.contains(&[14, 3]),
        "drained inner basin must connect the creature approach"
    );
    for map in [&ridge, &basin, &opened] {
        for y in 0..map.attributes.height * 2 {
            for x in 0..map.attributes.width * 2 {
                let kind = crate::authored_town_geometry::kind(base, map, [x, y])?;
                ensure!(
                    !matches!(kind, "CUT_TREE" | "BUOY" | "PIT"),
                    "unsupported incidental basin terrain"
                );
                if (x == 0
                    || y == 0
                    || x == map.attributes.width * 2 - 1
                    || y == map.attributes.height * 2 - 1)
                    && !(map.id == RIDGE && y == 0 && (x == 14 || x == 15))
                {
                    ensure!(
                        kind == "WALL",
                        "closed basin/terrace outer boundary required"
                    );
                }
            }
        }
    }
    for (map, roots) in [
        (
            &mut ridge,
            vec![
                "MnBasinSurveyor",
                "MnBasinHighShore",
                "MnBasinMiddleShore",
                "MnBasinLowShore",
            ],
        ),
        (
            &mut basin,
            vec![
                "MnBasinUpperDial",
                "MnBasinCenterDial",
                "MnBasinLowerDial",
                "MnBasinLever",
                "MN_BASIN_STRANDRAK",
                "MnBasinObjects",
                "MnBasinNewVisit",
                "MnBasinTiles",
            ],
        ),
    ] {
        let roots = roots.into_iter().map(String::from).collect::<Vec<_>>();
        for label in crate::discovery_dialogue::script_closure(&programs.scripts, &roots)? {
            ensure!(!map.scripts.contains_key(&label), "occupied basin program");
            map.scripts
                .insert(label.clone(), programs.scripts[&label].clone());
        }
        map.scripted_wild_battles = crate::parse_scripted_wild_battles(&map.id, &map.scripts)?;
    }
    outside.blocks[23 * 32 + 21] = 2;
    outside.attributes.connections.push(MapConnection {
        direction: "south".into(),
        target_map: RIDGE.into(),
        offset: 14,
    });
    let flags = outside
        .attributes
        .connection_flags
        .as_ref()
        .context("original prairie connections")?;
    outside.attributes.connection_flags = Some(format!("{flags} | SOUTH"));
    arc::publish(&mut pack, outside)?;
    a::publish(&mut pack, ridge, PARENT, [14, 0])?;
    a::publish(&mut pack, basin, PARENT, [2, 25])?;
    let mut encounters = encounters.clone();
    encounters.map_name = RIDGE.into();
    pack.data.wild_encounters.insert(RIDGE.into(), encounters);
    for (name, map) in &base.data.maps {
        if name != PARENT {
            ensure!(
                pack.data.maps.get(name) == Some(map),
                "unrelated shore map changed: {name}"
            );
        }
    }
    ensure!(
        pack.runtime_files == base.runtime_files
            && pack.audio_manifest == base.audio_manifest
            && pack.compiled_audio == base.compiled_audio
            && pack.data.pokemon == base.data.pokemon,
        "original assets/audio/species must remain exact"
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

fn reviewed_ridge_rows() -> [&'static str; 14] {
    [
        "05 05 05 05 05 05 05 01 05 05 05 05 05 05 05 05 05 05",
        "05 05 05 05 05 02 02 01 02 02 02 05 05 05 05 05 05 05",
        "05 05 05 05 02 02 03 03 02 02 02 02 02 02 05 05 05 05",
        "05 05 05 02 02 02 03 03 02 02 02 02 02 02 02 05 05 05",
        "05 05 05 02 02 70 70 70 70 70 70 70 02 02 02 05 05 05",
        "05 05 05 02 02 02 02 03 02 02 02 02 02 02 02 05 05 05",
        "05 05 05 05 02 02 02 03 03 02 02 02 02 02 02 05 05 05",
        "05 05 05 70 70 70 70 70 70 70 70 70 02 02 02 02 05 05",
        "05 05 05 02 02 02 03 03 02 02 02 02 02 02 02 02 05 05",
        "05 05 05 05 02 02 03 03 02 02 02 02 02 02 02 05 05 05",
        "05 05 05 02 02 02 70 70 70 70 70 70 70 70 02 02 05 05",
        "05 05 05 02 02 03 03 02 02 73 02 02 02 05 05 05 05 05",
        "05 05 05 05 02 02 02 02 02 02 02 02 02 05 05 05 05 05",
        "05 05 05 05 05 05 05 05 05 05 05 05 05 05 05 05 05 05",
    ]
}

fn reviewed_basin_rows() -> [&'static str; 14] {
    [
        "1d 1d 1d 1d 1d 1d 1d 1d 1d 1d 1d 1d 1d 1d",
        "1d 02 02 02 0f 0f 02 02 02 0f 0f 02 02 1d",
        "1d 02 02 02 0f 0f 02 02 02 0f 0f 02 02 1d",
        "1d 1d 1d 1d 1d 1d 1d 1d 1d 1d 1d 1d 1d 1d",
        "1d 02 02 02 02 02 02 02 02 02 02 02 02 1d",
        "1d 02 02 02 1d 02 02 1d 1d 1d 1d 02 02 1d",
        "1d 02 02 1d 1d 1d 02 02 02 02 1d 02 02 1d",
        "1d 02 02 02 02 02 02 02 02 1d 1d 02 02 1d",
        "1d 02 02 1d 1d 1d 1d 1d 02 02 1d 02 02 1d",
        "1d 02 02 02 02 1d 02 02 02 02 02 02 02 1d",
        "1d 02 02 02 1d 1d 1d 0c 1d 1d 1d 02 02 1d",
        "1d 02 02 02 02 02 02 02 02 02 02 02 02 1d",
        "1d 0b 02 02 02 02 02 02 02 02 02 02 02 1d",
        "1d 1d 1d 1d 1d 1d 1d 1d 1d 1d 1d 1d 1d 1d",
    ]
}
fn point(value: &Value) -> Result<[u16; 2]> {
    serde_json::from_value(value.clone()).context("basin coordinate")
}
fn vacant(map: &MapModule, at: [u16; 2]) -> Result<()> {
    ensure!(
        !map.objects.iter().any(|o| [o.x, o.y] == at)
            && !map.events.warps.iter().any(|e| [e.x, e.y] == at)
            && !map.events.coord_events.iter().any(|e| [e.x, e.y] == at)
            && !map.events.bg_events.iter().any(|e| [e.x, e.y] == at),
        "occupied basin approach: {at:?}"
    );
    Ok(())
}
fn validate_species(pack: &CompiledGamePack, value: &Value) -> Result<()> {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                if key == "species" {
                    let species = value.as_str().context("prairie habitat species")?;
                    ensure!(
                        pack.data.pokemon.contains_key(species)
                            && pack
                                .data
                                .story_event_script_constants
                                .global
                                .contains_key(&format!("MN_DEX_MEMBER_{species}")),
                        "active Minnesota shore encounter: {species}"
                    );
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
