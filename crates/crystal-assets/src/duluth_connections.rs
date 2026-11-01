//! Native guarded walking between the Range, Duluth and the North Shore.
#[path = "duluth_connection_geometry.rs"]
mod geometry;

use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_core::map::MapConnection;
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;
use serde::Deserialize;
use serde_json::Value;

use crate::authored_checkpoint::{Checkpoint, install};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const TOWN: &str = "MnDuluth";
const MANIFEST: &str = "minnesota-duluth-connections-v2";

#[derive(Deserialize)]
struct Source {
    manifest: String,
    patches: Vec<Patch>,
    passages: Vec<Passage>,
    sign_relocation: SignRelocation,
}
#[derive(Deserialize)]
struct SignRelocation {
    script: String,
    original: [u16; 2],
    destination: [u16; 2],
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
    badge_flags: Vec<String>,
    required_event: String,
    badge_lines: Vec<String>,
    evidence_lines: Vec<String>,
}

/// Joins the existing Duluth city to R10 and the southern North Shore chunk.
///
/// Westbound departure requires native badge 7 and Grand Rapids clearance;
/// northbound departure requires all eight badges and the acknowledged harbor
/// signal. Inbound travelers receive free city access through native scripts.
/// No quest result, badge, resource or admission flag is granted here.
///
/// Six declared metatiles fit the mouths and move the west sign two tiles north
/// with its native interaction and unchanged text. Four new Officers
/// and eight coordinate events seal the city lanes. Original actors, scripts,
/// other city terrain, maps/catalogs, art, audio and recovery data are retained.
///
/// # Errors
/// Returns an error for stale source, duplicate installation, conflicting
/// identities, changed prerequisites, blocked anchors, walking/Surf bypasses,
/// mismatched native offsets or parser/pack verification failures. Borrowed
/// inputs remain unchanged on success and failure.
pub fn connect_duluth(base: &CompiledGamePack, town_json: &str) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let raw: Value = serde_json::from_str(town_json).context("Duluth connection source")?;
    ensure!(raw["id"] == TOWN, "connections require Duluth source");
    let source: Source = serde_json::from_value(raw["regional_connections"].clone())?;
    ensure!(
        source.manifest == MANIFEST && !base.report.manifests.iter().any(|s| s == MANIFEST),
        "Duluth connections already installed or changed"
    );
    geometry::validate_source(&source)?;
    for p in &source.passages {
        for name in p.badge_flags.iter().map(String::as_str).chain([
            p.required_event.as_str(),
            "VAR_FACING",
            "LEFT",
            "RIGHT",
            "UP",
            "DOWN",
        ]) {
            ensure!(
                base.data
                    .story_event_script_constants
                    .global
                    .contains_key(name),
                "missing native Duluth checkpoint constant {name}"
            );
        }
    }
    let mut maps = [TOWN, "MnR10C01R00", "MnR11C00R03"]
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
    for patch in &source.patches {
        let map = maps
            .get_mut(&patch.map)
            .context("Duluth patch outside declared maps")?;
        let [x, y] = patch.metatile;
        ensure!(
            x < map.attributes.width && y < map.attributes.height,
            "Duluth patch outside extent"
        );
        let i = usize::from(y) * usize::from(map.attributes.width) + usize::from(x);
        ensure!(
            map.blocks[i] == u16::from_str_radix(&patch.expected, 16)?,
            "stale route mouth {} {x},{y}",
            patch.map
        );
        map.blocks[i] = u16::from_str_radix(&patch.block, 16)?;
    }
    let city = maps.get_mut(TOWN).context("installed city")?;
    let relocation = &source.sign_relocation;
    let signs = city
        .events
        .bg_events
        .iter_mut()
        .filter(|e| e.script == relocation.script)
        .collect::<Vec<_>>();
    ensure!(
        signs.len() == 1 && [signs[0].x, signs[0].y] == relocation.original,
        "stale Duluth west sign anchor"
    );
    let sign = signs.into_iter().next().context("native sign")?;
    [sign.x, sign.y] = relocation.destination;
    geometry::validate_maps(base, &maps, &source)?;
    let mut town = maps.remove(TOWN).context("native city")?;
    for p in &source.passages {
        let badge_flags = p.badge_flags.iter().map(String::as_str).collect::<Vec<_>>();
        install(
            base,
            &mut town,
            &Checkpoint {
                identifiers: &p.identifiers,
                bodies: p.bodies,
                inside: p.inside,
                outside: p.outside,
                root: &p.root,
                incoming_facing: &p.incoming_facing,
                outward: &p.outward,
                required_event: &p.required_event,
                badge_flags: &badge_flags,
                badge_lines: &p.badge_lines,
                evidence_lines: &p.evidence_lines,
            },
        )?;
        connection(&mut town, &p.outward, &p.route, p.offset)?;
        connection(
            maps.get_mut(&p.route).context("neighboring route")?,
            &p.reverse,
            TOWN,
            -p.offset,
        )?;
    }
    authoring::finish(base, &mut town)?;
    town.script_runtime_commands = crate::parse_script_runtime_commands(TOWN, &town.scripts)?;
    town.script_object_commands = crate::parse_script_object_commands(TOWN, &town.scripts)?;
    town.script_movements =
        crate::parse_script_movements(TOWN, &town.scripts, &town.script_object_commands)?;
    town.script_variable_commands = crate::parse_script_variable_commands(TOWN, &town.scripts)?;
    maps.insert(TOWN.into(), town);
    let mut pack = base.clone();
    for (_, map) in maps {
        publish_map(&mut pack, map)?;
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

fn connection(map: &mut MapModule, direction: &str, target: &str, offset: i32) -> Result<()> {
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

fn publish_map(pack: &mut CompiledGamePack, map: MapModule) -> Result<()> {
    crate::insert_compiled_map_split_payloads(
        &mut pack.data.map_scripts,
        &mut pack.data.npcs,
        &map.id,
        map.attributes
            .map_scripts_label
            .as_deref()
            .context("native scripts section")?,
        map.attributes
            .map_events_label
            .as_deref()
            .context("native events section")?,
        &map,
    )?;
    pack.data
        .map_attributes
        .insert(map.id.clone(), map.attributes.clone());
    let bytes = map
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    pack.data.map_blocks.insert(
        map.attributes
            .blocks_label
            .clone()
            .context("native blocks label")?,
        crate::encode_base64_bytes(&bytes),
    );
    pack.data.maps.insert(map.id.clone(), map);
    Ok(())
}

fn native([x, y]: [u16; 2]) -> Result<TilePosition> {
    Ok(TilePosition::new(i16::try_from(x)?, i16::try_from(y)?))
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
                let mut crossing = native(from)?;
                match direction {
                    "west" => crossing.x -= 1,
                    "east" => crossing.x += 1,
                    "north" => crossing.y -= 1,
                    "south" => crossing.y += 1,
                    _ => anyhow::bail!("unknown Duluth seam direction"),
                }
                let trigger = connection_for_tile_checked(&map, crossing)?
                    .context("native Duluth seam trigger")?;
                ensure!(
                    trigger.connection.target_map == target
                        && trigger.connection.direction == direction,
                    "wrong native Duluth seam"
                );
                let arrival = pack.data.resolve_connection_transition(&trigger)?;
                ensure!(
                    arrival.destination.map_name == target
                        && arrival.destination.tile == native(to)?,
                    "Duluth seam offsets do not align"
                );
            }
        }
    }
    Ok(())
}
