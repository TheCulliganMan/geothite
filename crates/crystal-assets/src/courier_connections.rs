//! Current Badge departure and reciprocal Minneapolis/R04 walking seam.
use anyhow::{Context, Result, ensure};
use crystal_core::map::MapConnection;
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;
use serde_json::{Value, json};

use crate::authored_checkpoint::{Checkpoint, install as checkpoint};
use crate::authored_town_geometry::{kind, reachable};
use crate::courier_story::Chapter;
use crate::{CompiledGamePack, MapModule};

pub(crate) fn install(
    base: &CompiledGamePack,
    chapter: &Chapter,
    route_json: &str,
) -> Result<Vec<MapModule>> {
    let source: Value = serde_json::from_str(route_json)?;
    ensure!(
        source["id"] == "MN_R04"
            && source["required_flags"] == json!(["BADGE_3"])
            && chapter.town == "MnMinneapolis"
            && chapter.route == "MnR04C00R00"
            && chapter.offset == 3
            && chapter.mouth_metatile == [31, 12]
            && chapter.town_edge == [[63, 24], [63, 25]]
            && chapter.route_edge == [[0, 18], [0, 19]],
        "canonical Current-only R04 seam required"
    );
    let mut city = base
        .data
        .maps
        .get(&chapter.town)
        .context("Minneapolis")?
        .clone();
    let mut road = base
        .data
        .maps
        .get(&chapter.route)
        .context("Twin Cities route")?
        .clone();
    ensure!(
        [city.attributes.width, city.attributes.height] == [32, 24]
            && [road.attributes.width, road.attributes.height] == [28, 18]
            && city.blocks[12 * 32 + 31] == 5,
        "closed current city mouth required"
    );
    city.blocks[12 * 32 + 31] = 1;
    for (map, side, target, offset, edges) in [
        (&mut city, "east", "MnR04C00R00", 3, chapter.town_edge),
        (&mut road, "west", "MnMinneapolis", -3, chapter.route_edge),
    ] {
        ensure!(
            !map.attributes
                .connections
                .iter()
                .any(|c| c.direction == side),
            "duplicate R04 city seam"
        );
        for tile in edges {
            ensure!(
                kind(base, map, tile)? == "FLOOR",
                "R04 native mouth blocked"
            );
        }
        map.attributes.connections.push(MapConnection {
            direction: side.into(),
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
    }
    let ids = [
        "MN_MP_EAST_WARDEN_NORTH".into(),
        "MN_MP_EAST_WARDEN_SOUTH".into(),
    ];
    checkpoint(
        base,
        &mut city,
        &Checkpoint {
            identifiers: &ids,
            bodies: [[61, 24], [61, 25]],
            inside: [[60, 24], [60, 25]],
            outside: [[62, 24], [62, 25]],
            root: "MnMPEastCheck",
            incoming_facing: "LEFT",
            outward: "east",
            required_event: "",
            badge_flags: &["ENGINE_PLAINBADGE"],
            badge_lines: &[
                "Earn AVRIN's".into(),
                "CURRENT BADGE.".into(),
                "Then ST PAUL lies".into(),
                "across the bridge.".into(),
            ],
            evidence_lines: &[],
        },
    )?;
    for event in &mut city.events.coord_events {
        if event.script_name.starts_with("MnMPEastCheck") {
            event.scene_id = "-1".into();
        }
    }
    let closed = reachable(base, &city, [32, 40], true)?;
    ensure!(
        !closed.contains(&[63, 24]) && !closed.contains(&[63, 25]),
        "Current gate has walking/Surf bypass"
    );
    let mut open = city.clone();
    open.objects.retain(|o| {
        !ids.iter()
            .any(|id| o.object_identifier.as_ref() == Some(id))
    });
    crate::prairie_story_connections::perimeter(
        base,
        &open,
        [32, 40],
        &[[32, 47], [33, 47], [63, 24], [63, 25]],
    )?;
    crate::prairie_story_connections::perimeter(
        base,
        &road,
        [4, 18],
        &[[0, 18], [0, 19], [55, 18], [55, 19]],
    )?;
    Ok(vec![city, road])
}

pub(crate) fn verify(pack: &CompiledGamePack) -> Result<()> {
    for (id, target, edge, expected) in [
        ("MnMinneapolis", "MnR04C00R00", [64, 24], [0, 18]),
        ("MnMinneapolis", "MnR04C00R00", [64, 25], [0, 19]),
        ("MnR04C00R00", "MnMinneapolis", [-1, 18], [63, 24]),
        ("MnR04C00R00", "MnMinneapolis", [-1, 19], [63, 25]),
    ] {
        let map = &pack.data.maps[id];
        let native = OverworldMapData::from_attributes(id, &map.attributes, map.blocks.clone());
        let t = connection_for_tile_checked(&native, TilePosition::new(edge[0], edge[1]))?
            .context("native R04 edge")?;
        let destination = pack.data.resolve_connection_transition(&t)?.destination;
        ensure!(
            destination.map_name == target
                && destination.tile == TilePosition::new(expected[0], expected[1]),
            "native R04 projection differs"
        );
    }
    Ok(())
}
