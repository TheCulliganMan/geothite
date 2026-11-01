//! River Valley's reciprocal Mankato/Minneapolis mouths and real badge2 gate.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_core::map::MapConnection;
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::authored_checkpoint::{Checkpoint, install as checkpoint};
use crate::authored_town_geometry::kind;
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const BASE: &str = "f9853ad3269c91f53871f50ce766e4d59b8228bcfdbbeded44243ccad48981be";
const MANIFEST: &str = "minnesota-river-city-links-v2";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Chapter {
    schema_version: u8,
    manifest: String,
    base_content_hash: String,
    links: Vec<Link>,
    route_edge_closures: Vec<crate::river_city_geometry::Closure>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Link {
    town: String,
    route: String,
    town_direction: String,
    route_direction: String,
    offset: i32,
    mouth_metatile: [u16; 2],
    town_edge: [[u16; 2]; 2],
    route_edge: [[u16; 2]; 2],
}

/// Connects Mankato to Minneapolis through the existing two River Valley chunks.
///
/// Leaving Mankato northward requires its actual second badge. Inbound travel
/// and returning south remain free; the optional notebook is not a gate. Only
/// the two forest mouths, six south-bank boundary caps, reciprocal connection
/// records and the Mankato guard scripts/objects change. Other terrain,
/// encounters, every gym, existing story,
/// numeric events, audio, HOME and recovery are preserved. No progress is granted.
///
/// # Errors
/// Returns an error for stale/duplicate input, changed authoring contracts,
/// occupied mouths/guards, unsafe geometry, mismatched native projections,
/// script parsing failure or failed runtime verification. The input is unchanged
/// on failure; no partial pack is published.
pub fn connect_river_cities(
    base: &CompiledGamePack,
    chapter_json: &str,
    route_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let chapter: Chapter = serde_json::from_str(chapter_json)?;
    let route: Value = serde_json::from_str(route_json)?;
    ensure!(
        chapter.schema_version == 1
            && chapter.manifest == MANIFEST
            && chapter.base_content_hash == BASE
            && base.identity.content_hash == BASE
            && chapter.links.len() == 2
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "exact Prairie Road chapter required; stale/duplicate River Valley links"
    );
    ensure!(
        route["id"] == "MN_R03"
            && route["required_flags"] == json!(["BADGE_2"])
            && route["dimensions_metatiles"] == json!([24, 32]),
        "canonical River Valley badge and extent contract"
    );
    let mut maps = ["MnMankato", "MnMinneapolis", "MnR03C00R00", "MnR03C00R01"]
        .into_iter()
        .map(|id| {
            Ok((
                id.to_owned(),
                base.data
                    .maps
                    .get(id)
                    .context("installed River Valley module")?
                    .clone(),
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    crate::river_city_geometry::seal(base, &mut maps, &chapter.route_edge_closures)?;
    for (link, (town, road, side, reverse, offset, mouth, town_edge, route_edge, size)) in
        chapter.links.iter().zip([
            (
                "MnMankato",
                "MnR03C00R01",
                "north",
                "south",
                3,
                [12, 0],
                [[24, 0], [25, 0]],
                [[18, 15], [19, 15]],
                [24, 18],
            ),
            (
                "MnMinneapolis",
                "MnR03C00R00",
                "south",
                "north",
                6,
                [16, 23],
                [[32, 47], [33, 47]],
                [[20, 0], [21, 0]],
                [32, 24],
            ),
        ])
    {
        ensure!(
            link.town == town
                && link.route == road
                && link.town_direction == side
                && link.route_direction == reverse
                && link.offset == offset
                && link.mouth_metatile == mouth
                && link.town_edge == town_edge
                && link.route_edge == route_edge,
            "fixed River Valley mouths and offsets required"
        );
        let city = maps.get_mut(town).context("city")?;
        ensure!(
            [city.attributes.width, city.attributes.height] == size
                && !city
                    .attributes
                    .connections
                    .iter()
                    .any(|c| c.direction == side),
            "city mouth already connected or stale extents"
        );
        let at = usize::from(mouth[1]) * usize::from(city.attributes.width) + usize::from(mouth[0]);
        ensure!(
            city.blocks.get(at) == Some(&5),
            "River Valley city mouth must be the closed native forest block"
        );
        city.blocks[at] = 1;
        city.attributes.connections.push(MapConnection {
            direction: side.into(),
            target_map: road.into(),
            offset,
        });
        flags(city);
        let road = maps.get_mut(road).context("route chunk")?;
        ensure!(
            !road
                .attributes
                .connections
                .iter()
                .any(|c| c.direction == reverse),
            "route mouth already connected"
        );
        for tile in route_edge {
            ensure!(
                kind(base, road, tile)? == "FLOOR",
                "River Valley edge must be native floor"
            );
        }
        road.attributes.connections.push(MapConnection {
            direction: reverse.into(),
            target_map: town.into(),
            offset: -offset,
        });
        flags(road);
    }
    let city = maps
        .get_mut("MnMankato")
        .context("Prairie Badge checkpoint")?;
    let ids = [
        "MN_MK_RIVER_WARDEN_WEST".into(),
        "MN_MK_RIVER_WARDEN_EAST".into(),
    ];
    checkpoint(
        base,
        city,
        &Checkpoint {
            identifiers: &ids,
            bodies: [[24, 2], [25, 2]],
            inside: [[24, 3], [25, 3]],
            outside: [[24, 1], [25, 1]],
            root: "MnMKRiverCheck",
            incoming_facing: "DOWN",
            outward: "north",
            required_event: "",
            badge_flags: &["ENGINE_HIVEBADGE"],
            badge_lines: &[
                "Earn TORVEN's".into(),
                "PRAIRIE BADGE.".into(),
                "Then take the".into(),
                "valley road north.".into(),
            ],
            evidence_lines: &[],
        },
    )?;
    for event in &mut city.events.coord_events {
        if event.script_name.starts_with("MnMKRiverCheck") {
            event.scene_id = "-1".into();
        }
    }
    crate::river_city_geometry::validate(base, &maps)?;
    let city = maps.get("MnMankato").context("Mankato")?;
    let mut open = city.clone();
    open.objects.retain(|o| {
        !ids.iter()
            .any(|id| o.object_identifier.as_ref() == Some(id))
    });
    crate::prairie_story_connections::perimeter(
        base,
        &open,
        [40, 18],
        &[[47, 18], [47, 19], [24, 0], [25, 0]],
    )?;
    crate::prairie_story_connections::perimeter(
        base,
        &maps["MnMinneapolis"],
        [32, 40],
        &[[32, 47], [33, 47]],
    )?;
    let mut pack = base.clone();
    for (_, mut map) in maps {
        authoring::finish(&pack, &mut map)?;
        map.script_runtime_commands = crate::parse_script_runtime_commands(&map.id, &map.scripts)?;
        map.script_object_commands = crate::parse_script_object_commands(&map.id, &map.scripts)?;
        map.script_variable_commands =
            crate::parse_script_variable_commands(&map.id, &map.scripts)?;
        map.script_movements =
            crate::parse_script_movements(&map.id, &map.scripts, &map.script_object_commands)?;
        crate::authored_town_publication::replace(&mut pack, map)?;
    }
    verify(&pack, &chapter.links)?;
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
fn flags(map: &mut MapModule) {
    map.attributes.connection_flags = Some(
        map.attributes
            .connections
            .iter()
            .map(|c| c.direction.to_ascii_uppercase())
            .collect::<Vec<_>>()
            .join(" | "),
    );
}
fn verify(pack: &CompiledGamePack, links: &[Link]) -> Result<()> {
    for link in links {
        for (id, side, edges, target, arrivals) in [
            (
                &link.town,
                &link.town_direction,
                link.town_edge,
                &link.route,
                link.route_edge,
            ),
            (
                &link.route,
                &link.route_direction,
                link.route_edge,
                &link.town,
                link.town_edge,
            ),
        ] {
            let map = &pack.data.maps[id];
            let native = OverworldMapData::from_attributes(id, &map.attributes, map.blocks.clone());
            for (edge, expected) in edges.into_iter().zip(arrivals) {
                let crossing = TilePosition::new(
                    i16::try_from(edge[0])?,
                    if side == "north" {
                        -1
                    } else {
                        i16::try_from(edge[1] + 1)?
                    },
                );
                let transition = connection_for_tile_checked(&native, crossing)?
                    .context("native River Valley transition")?;
                let arrival = pack.data.resolve_connection_transition(&transition)?;
                ensure!(
                    arrival.destination.map_name == *target
                        && arrival.destination.tile
                            == TilePosition::new(
                                i16::try_from(expected[0])?,
                                i16::try_from(expected[1])?
                            ),
                    "River Valley native projection differs"
                );
            }
        }
    }
    Ok(())
}
