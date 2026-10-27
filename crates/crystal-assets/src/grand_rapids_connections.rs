//! Guarded native walking from Iron Range West through the survey-depot town.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_core::map::MapConnection;
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;
use serde::Deserialize;
use serde_json::Value;

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const TOWN: &str = "MnGrandRapids";
const MANIFEST: &str = "minnesota-grand-rapids-connections-v1";

#[derive(Deserialize)]
struct Source {
    manifest: String,
    patches: Vec<Patch>,
    passages: Vec<Passage>,
}
#[derive(Deserialize)]
struct Patch {
    map: String,
    metatile: [u16; 2],
    expected: String,
    block: String,
}
#[derive(Deserialize)]
struct Passage {
    route: String,
    outward: String,
    reverse: String,
    offset: i32,
    town_edge: [[u16; 2]; 2],
    route_edge: [[u16; 2]; 2],
    bodies: [[u16; 2]; 2],
    inside: [[u16; 2]; 2],
    outside: [[u16; 2]; 2],
    identifiers: [String; 2],
    root: String,
    incoming_facing: String,
    required_event: String,
    badge_lines: Vec<String>,
    evidence_lines: Vec<String>,
}

/// Opens the two reserved mouths and binds reciprocal native walking seams.
///
/// Westbound travel requires badge 7 and the headwaters report. Eastbound
/// travel requires badge 7 and the completed depot handover; neither foreman
/// defeat nor meter recovery can replace it. Inbound travelers are escorted
/// through the guards to free care. Native scripts own checks, movement and
/// temporary actor visibility. No quest flag, badge, item or money is awarded.
///
/// Six explicitly authored metatiles fit the two mouths and narrow the east
/// guard throat. Existing actors, room/script behavior, other terrain, audio,
/// HOME and recovery records are preserved. Native controller proof of the
/// resulting journey remains separate from publication.
///
/// # Errors
/// Returns an error for stale source/geometry, changed prerequisites, duplicate
/// installation, conflicting native labels/anchors, an unsealed walking/Surf
/// checkpoint, mismatched offsets, or parser/runtime validation failures.
/// The input pack and source remain unchanged on success and failure.
pub fn connect_grand_rapids(base: &CompiledGamePack, town_json: &str) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let raw: Value = serde_json::from_str(town_json).context("Grand Rapids connection source")?;
    ensure!(
        raw["id"] == TOWN,
        "connections require the authored Grand Rapids town"
    );
    let source: Source = serde_json::from_value(raw["regional_connections"].clone())?;
    ensure!(
        source.manifest == MANIFEST
            && source.passages.len() == 2
            && !base.report.manifests.iter().any(|s| s == MANIFEST),
        "Grand Rapids connections already installed or changed"
    );
    validate_source(&source)?;
    for name in [
        BADGE_ENGINE_FLAGS[6],
        "EVENT_MN_HEADWATERS_REPORT",
        "EVENT_MN_DEPOT_CLEARED",
        "VAR_FACING",
        "LEFT",
        "RIGHT",
    ] {
        ensure!(
            base.data
                .story_event_script_constants
                .global
                .contains_key(name),
            "missing native checkpoint constant {name}"
        );
    }
    let mut pack = base.clone();
    let mut maps = [TOWN, "MnR09C01R00", "MnR10C00R00"]
        .into_iter()
        .map(|name| {
            Ok((
                name.to_owned(),
                base.data
                    .maps
                    .get(name)
                    .with_context(|| format!("installed {name}"))?
                    .clone(),
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    ensure!(
        [maps[TOWN].attributes.width, maps[TOWN].attributes.height] == [24, 24]
            && [
                maps["MnR09C01R00"].attributes.width,
                maps["MnR09C01R00"].attributes.height
            ] == [32, 24]
            && [
                maps["MnR10C00R00"].attributes.width,
                maps["MnR10C00R00"].attributes.height
            ] == [32, 20],
        "changed native midgame extents"
    );
    for patch in &source.patches {
        let map = maps
            .get_mut(&patch.map)
            .context("patch outside declared midgame maps")?;
        let [x, y] = patch.metatile;
        ensure!(
            x < map.attributes.width && y < map.attributes.height,
            "midgame patch outside map"
        );
        let index = usize::from(y) * usize::from(map.attributes.width) + usize::from(x);
        ensure!(
            map.blocks[index] == u16::from_str_radix(&patch.expected, 16)?,
            "stale native mouth at {} {:?}",
            patch.map,
            patch.metatile
        );
        map.blocks[index] = u16::from_str_radix(&patch.block, 16)?;
    }
    let mut town = maps.remove(TOWN).context("native town")?;
    validate_geometry(&pack, &town, &maps, &source.passages)?;
    for passage in &source.passages {
        add_guards(&pack, &mut town, passage)?;
        add_connection(&mut town, &passage.outward, &passage.route, passage.offset)?;
        let route = maps
            .get_mut(&passage.route)
            .context("native neighboring route")?;
        add_connection(route, &passage.reverse, TOWN, -passage.offset)?;
    }
    authoring::finish(&pack, &mut town)?;
    town.script_runtime_commands = crate::parse_script_runtime_commands(TOWN, &town.scripts)?;
    town.script_object_commands = crate::parse_script_object_commands(TOWN, &town.scripts)?;
    town.script_movements =
        crate::parse_script_movements(TOWN, &town.scripts, &town.script_object_commands)?;
    town.script_variable_commands = crate::parse_script_variable_commands(TOWN, &town.scripts)?;
    maps.insert(TOWN.into(), town);
    for (_, map) in maps {
        install_module(&mut pack, map)?;
    }
    validate_connections(&pack, &source.passages)?;
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

fn validate_source(source: &Source) -> Result<()> {
    let patches = source
        .patches
        .iter()
        .map(|p| {
            (
                p.map.as_str(),
                p.metatile,
                p.expected.as_str(),
                p.block.as_str(),
            )
        })
        .collect::<BTreeSet<_>>();
    ensure!(
        patches.len() == 6
            && source.patches.len() == 6
            && patches
                == BTreeSet::from([
                    (TOWN, [0, 12], "05", "01"),
                    (TOWN, [23, 12], "05", "01"),
                    (TOWN, [22, 11], "02", "05"),
                    (TOWN, [22, 13], "02", "05"),
                    (TOWN, [22, 14], "02", "05"),
                    ("MnR10C00R00", [0, 5], "68", "01"),
                ]),
        "retain the six explicit reserved-mouth and guard-throat patches"
    );
    for (
        p,
        (
            route,
            outward,
            reverse,
            offset,
            body,
            inside,
            outside,
            edge,
            route_x,
            route_y,
            event,
            root,
            side,
        ),
    ) in source.passages.iter().zip([
        (
            "MnR09C01R00",
            "west",
            "east",
            0,
            2,
            3,
            1,
            0,
            63,
            24,
            "EVENT_MN_HEADWATERS_REPORT",
            "MnGrandRapidsWestCheck",
            "WEST",
        ),
        (
            "MnR10C00R00",
            "east",
            "west",
            7,
            45,
            44,
            46,
            47,
            0,
            10,
            "EVENT_MN_DEPOT_CLEARED",
            "MnGrandRapidsEastCheck",
            "EAST",
        ),
    ]) {
        let column = |x, y| [[x, y], [x, y + 1]];
        ensure!(
            p.route == route
                && p.outward == outward
                && p.reverse == reverse
                && p.offset == offset
                && p.town_edge == column(edge, 24)
                && p.route_edge == column(route_x, route_y)
                && p.bodies == column(body, 24)
                && p.inside == column(inside, 24)
                && p.outside == column(outside, 24)
                && p.required_event == event
                && p.root == root
                && p.incoming_facing == if outward == "west" { "RIGHT" } else { "LEFT" }
                && p.identifiers
                    == [
                        format!("MN_GRAND_RAPIDS_{side}_GUARD_NORTH"),
                        format!("MN_GRAND_RAPIDS_{side}_GUARD_SOUTH")
                    ],
            "retain exact badge/report/depot route checkpoints and paired seams"
        );
        for lines in [&p.badge_lines, &p.evidence_lines] {
            ensure!(
                !lines.is_empty()
                    && lines.iter().all(|s| !s.is_empty()
                        && s.chars().count() <= 18
                        && s.is_ascii()
                        && !s.contains('"')),
                "checkpoint text must fit native pages"
            );
        }
    }
    Ok(())
}

fn add_connection(map: &mut MapModule, direction: &str, target: &str, offset: i32) -> Result<()> {
    ensure!(
        !map.attributes
            .connections
            .iter()
            .any(|c| c.direction == direction),
        "{} already has a {direction} connection",
        map.id
    );
    map.attributes.connections.push(MapConnection {
        direction: direction.into(),
        target_map: target.into(),
        offset,
    });
    map.attributes.connection_flags = Some(
        map.attributes
            .connections
            .iter()
            .map(|c| c.direction.to_ascii_uppercase())
            .collect::<Vec<_>>()
            .join(" | "),
    );
    Ok(())
}

fn install_module(pack: &mut CompiledGamePack, map: MapModule) -> Result<()> {
    crate::insert_compiled_map_split_payloads(
        &mut pack.data.map_scripts,
        &mut pack.data.npcs,
        &map.id,
        map.attributes
            .map_scripts_label
            .as_deref()
            .context("native script section")?,
        map.attributes
            .map_events_label
            .as_deref()
            .context("native event section")?,
        &map,
    )?;
    pack.data
        .map_attributes
        .insert(map.id.clone(), map.attributes.clone());
    pack.data.map_blocks.insert(
        map.attributes
            .blocks_label
            .clone()
            .context("native block label")?,
        crate::encode_base64_bytes(
            &map.blocks
                .iter()
                .copied()
                .map(u8::try_from)
                .collect::<std::result::Result<Vec<_>, _>>()
                .context("native connection block exceeds Crystal byte range")?,
        ),
    );
    pack.data.maps.insert(map.id.clone(), map);
    Ok(())
}

fn floor(pack: &CompiledGamePack, map: &MapModule, p: [u16; 2]) -> Result<()> {
    ensure!(
        crate::authored_route_modpack::collision(
            pack,
            &map.blocks,
            map.attributes.width,
            p,
            &map.attributes.tileset_name
        )? == "FLOOR",
        "{} passage {p:?} is not native FLOOR",
        map.id
    );
    Ok(())
}

fn validate_geometry(
    pack: &CompiledGamePack,
    town: &MapModule,
    maps: &BTreeMap<String, MapModule>,
    passages: &[Passage],
) -> Result<()> {
    let mut inner = BTreeSet::new();
    for p in passages {
        for tile in p
            .town_edge
            .into_iter()
            .chain(p.bodies)
            .chain(p.inside)
            .chain(p.outside)
        {
            floor(pack, town, tile)?;
            ensure!(
                !town.objects.iter().any(|a| [a.x, a.y] == tile)
                    && !town.events.coord_events.iter().any(|e| [e.x, e.y] == tile)
                    && !town.events.warps.iter().any(|w| [w.x, w.y] == tile),
                "midgame checkpoint overlaps original interaction"
            );
        }
        for tile in p.route_edge {
            floor(pack, &maps[&p.route], tile)?;
        }
        for tile in p.inside {
            let x = if p.incoming_facing == "RIGHT" {
                tile[0] + 1
            } else {
                tile[0] - 1
            };
            floor(pack, town, [x, tile[1]])?;
            ensure!(
                !town.objects.iter().any(|a| [a.x, a.y] == [x, tile[1]]),
                "checkpoint refusal hits original actor"
            );
        }
        let blocked = BTreeSet::from(p.inside);
        let reached =
            crate::authored_route_surface::reachable_map_with_surf(pack, town, [27, 14], &blocked)?;
        ensure!(
            p.town_edge.iter().all(|t| !reached.contains(t)),
            "{} has a walking/Surf checkpoint bypass",
            p.route
        );
        inner.extend(p.inside);
    }
    let public =
        crate::authored_route_surface::reachable_map_with_surf(pack, town, [27, 14], &inner)?;
    for tile in [[9, 12], [39, 16], [27, 14], [6, 24], [40, 24], [30, 32]] {
        ensure!(
            public.contains(&tile),
            "closed checkpoints deny town service/landing {tile:?}"
        );
    }
    for y in 0..48 {
        for x in 0..48 {
            if x != 0 && x != 47 && y != 0 && y != 47 {
                continue;
            }
            let p = [x, y];
            let allowed = passages.iter().any(|s| s.town_edge.contains(&p));
            let collision = crate::authored_route_modpack::collision(
                pack,
                &town.blocks,
                24,
                p,
                &town.attributes.tileset_name,
            )?;
            ensure!(
                allowed || !matches!(collision, "FLOOR" | "WATER" | "TALL_GRASS"),
                "unpaired town walking/Surf boundary {p:?}"
            );
        }
    }
    Ok(())
}

fn add_guards(pack: &CompiledGamePack, town: &mut MapModule, p: &Passage) -> Result<()> {
    crate::authored_checkpoint::install(
        pack,
        town,
        &crate::authored_checkpoint::Checkpoint {
            identifiers: &p.identifiers,
            bodies: p.bodies,
            inside: p.inside,
            outside: p.outside,
            root: &p.root,
            incoming_facing: &p.incoming_facing,
            outward: &p.outward,
            required_event: &p.required_event,
            badge_flags: &[BADGE_ENGINE_FLAGS[6]],
            badge_lines: &p.badge_lines,
            evidence_lines: &p.evidence_lines,
        },
    )
}

fn validate_connections(pack: &CompiledGamePack, passages: &[Passage]) -> Result<()> {
    for p in passages {
        for (source, target, direction, edge, expected) in [
            (
                TOWN,
                p.route.as_str(),
                p.outward.as_str(),
                p.town_edge,
                p.route_edge,
            ),
            (
                p.route.as_str(),
                TOWN,
                p.reverse.as_str(),
                p.route_edge,
                p.town_edge,
            ),
        ] {
            let module = &pack.data.maps[source];
            let map = OverworldMapData::from_attributes(
                source,
                &module.attributes,
                module.blocks.clone(),
            );
            for (from, to) in edge.into_iter().zip(expected) {
                let mut crossing = native_tile(from)?;
                if direction == "east" {
                    crossing.x += 1;
                } else {
                    crossing.x -= 1;
                }
                let trigger = connection_for_tile_checked(&map, crossing)?
                    .context("native midgame boundary trigger")?;
                ensure!(
                    trigger.connection.target_map == target
                        && trigger.connection.direction == direction,
                    "wrong native midgame seam"
                );
                let arrival = pack.data.resolve_connection_transition(&trigger)?;
                ensure!(
                    arrival.destination.map_name == target
                        && arrival.destination.tile == native_tile(to)?,
                    "native midgame offset does not align"
                );
                floor(pack, &pack.data.maps[target], to)?;
            }
        }
    }
    Ok(())
}
fn native_tile([x, y]: [u16; 2]) -> Result<TilePosition> {
    Ok(TilePosition::new(i16::try_from(x)?, i16::try_from(y)?))
}
