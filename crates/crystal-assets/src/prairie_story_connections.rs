//! Native two-way Prairie Road seams, Care checkpoint and bounded outer edges.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::map::MapConnection;
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;

use crate::authored_checkpoint::{Checkpoint, install as checkpoint};
use crate::authored_town_geometry::{kind, reachable};
use crate::prairie_story::Link;
use crate::{CompiledGamePack, MapModule};

pub(super) fn install(
    base: &CompiledGamePack,
    maps: &mut BTreeMap<String, MapModule>,
    links: &[Link],
) -> Result<()> {
    ensure!(
        links.len() == 2,
        "both reciprocal Prairie Road town seams required"
    );
    for (link, (town, direction, reverse, offset, mouth, town_x, route_x, route_y)) in
        links.iter().zip([
            ("MnRochester", "west", "east", 2, [0, 9], 0, 59, 14),
            ("MnMankato", "east", "west", -6, [23, 9], 47, 0, 30),
        ])
    {
        ensure!(
            link.town == town
                && link.town_direction == direction
                && link.route_direction == reverse
                && link.offset == offset
                && link.mouth_metatile == mouth
                && link.town_edge == [[town_x, 18], [town_x, 19]]
                && link.route_edge == [[route_x, route_y], [route_x, route_y + 1]],
            "fixed R02 mouths and offsets required"
        );
        let m = maps.get_mut(town).context("town")?;
        ensure!(
            !m.attributes
                .connections
                .iter()
                .any(|c| c.direction == direction),
            "town mouth already bound"
        );
        let index = usize::from(mouth[1]) * usize::from(m.attributes.width) + usize::from(mouth[0]);
        ensure!(
            m.blocks.get(index) == Some(&5),
            "stale closed Prairie Road town mouth"
        );
        m.blocks[index] = 1;
        m.attributes.connections.push(MapConnection {
            direction: direction.into(),
            target_map: "MnR02C00R00".into(),
            offset,
        });
        flags(m);
        let route = maps.get_mut("MnR02C00R00").context("Prairie Road")?;
        ensure!(
            !route
                .attributes
                .connections
                .iter()
                .any(|c| c.direction == reverse),
            "route mouth already bound"
        );
        for tile in link.route_edge {
            ensure!(
                kind(base, route, tile)? == "FLOOR",
                "R02 endpoint is blocked"
            );
        }
        route.attributes.connections.push(MapConnection {
            direction: reverse.into(),
            target_map: town.into(),
            offset: -offset,
        });
        flags(route);
    }
    let town = maps.get_mut("MnRochester").context("Care checkpoint")?;
    let ids = [
        "MN_RC_PRAIRIE_WARDEN_NORTH".into(),
        "MN_RC_PRAIRIE_WARDEN_SOUTH".into(),
    ];
    checkpoint(
        base,
        town,
        &Checkpoint {
            identifiers: &ids,
            bodies: [[2, 18], [2, 19]],
            inside: [[3, 18], [3, 19]],
            outside: [[1, 18], [1, 19]],
            root: "MnRCPrairieCheck",
            incoming_facing: "RIGHT",
            outward: "west",
            required_event: "",
            badge_flags: &["ENGINE_ZEPHYRBADGE"],
            badge_lines: &[
                "Earn VAREL's CARE".into(),
                "BADGE first.".into(),
                "Then take the road".into(),
                "to MANKATO.".into(),
            ],
            evidence_lines: &[],
        },
    )?;
    for event in &mut town.events.coord_events {
        if event.script_name.starts_with("MnRCPrairieCheck") {
            event.scene_id = "-1".into();
        }
    }
    let closed = reachable(base, town, [32, 18], true)?;
    ensure!(
        !closed.contains(&[0, 18]) && !closed.contains(&[0, 19]),
        "Care checkpoint has a walking/Surf side bypass"
    );
    let mut open = town.clone();
    open.objects.retain(|o| {
        !ids.iter()
            .any(|id| o.object_identifier.as_ref() == Some(id))
    });
    perimeter(
        base,
        &open,
        [32, 18],
        &[[0, 18], [0, 19], [39, 18], [39, 19]],
    )?;
    perimeter(
        base,
        maps.get("MnMankato").context("Mankato")?,
        [40, 18],
        &[[47, 18], [47, 19]],
    )?;
    let road = maps.get("MnR02C00R00").context("R02")?;
    for start in [[54, 14], [2, 30]] {
        perimeter(base, road, start, &[[59, 14], [59, 15], [0, 30], [0, 31]])?;
    }
    Ok(())
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

pub(crate) fn perimeter(
    pack: &CompiledGamePack,
    map: &MapModule,
    start: [u16; 2],
    allowed: &[[u16; 2]],
) -> Result<()> {
    let bodies = map
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    ensure!(
        kind(pack, map, start)? == "FLOOR" && !bodies.contains(&start),
        "safe chapter arrival required"
    );
    let mut seen = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some([x, y]) = queue.pop_front() {
        if x == 0
            || y == 0
            || x == map.attributes.width * 2 - 1
            || y == map.attributes.height * 2 - 1
        {
            ensure!(
                allowed.contains(&[x, y]),
                "{} unbound walking/Surf perimeter at {x},{y}",
                map.id
            );
        }
        for at in [
            x.checked_sub(1).map(|a| [a, y]),
            y.checked_sub(1).map(|a| [x, a]),
            x.checked_add(1).map(|a| [a, y]),
            y.checked_add(1).map(|a| [x, a]),
        ]
        .into_iter()
        .flatten()
        {
            if at[0] >= map.attributes.width * 2
                || at[1] >= map.attributes.height * 2
                || bodies.contains(&at)
                || seen.contains(&at)
            {
                continue;
            }
            let k = kind(pack, map, at)?;
            if matches!(
                k,
                "FLOOR" | "TALL_GRASS" | "WATER" | "LADDER" | "DOOR" | "CAVE" | "WARP_CARPET_DOWN"
            ) {
                seen.insert(at);
                if !matches!(k, "DOOR" | "CAVE" | "WARP_CARPET_DOWN") {
                    queue.push_back(at);
                }
            }
        }
    }
    for at in allowed {
        ensure!(
            seen.contains(at),
            "{} bound mouth is unreachable: {at:?}",
            map.id
        );
    }
    Ok(())
}

pub(super) fn verify(pack: &CompiledGamePack, links: &[Link]) -> Result<()> {
    for link in links {
        for (name, direction, edges, target, destinations) in [
            (
                link.town.as_str(),
                link.town_direction.as_str(),
                link.town_edge,
                "MnR02C00R00",
                link.route_edge,
            ),
            (
                "MnR02C00R00",
                link.route_direction.as_str(),
                link.route_edge,
                link.town.as_str(),
                link.town_edge,
            ),
        ] {
            let m = &pack.data.maps[name];
            let native = OverworldMapData::from_attributes(name, &m.attributes, m.blocks.clone());
            for (at, expected) in edges.into_iter().zip(destinations) {
                let crossing = TilePosition::new(
                    if direction == "west" {
                        -1
                    } else {
                        i16::try_from(at[0] + 1)?
                    },
                    i16::try_from(at[1])?,
                );
                let transition = connection_for_tile_checked(&native, crossing)?
                    .context("native R02 transition")?;
                let resolved = pack.data.resolve_connection_transition(&transition)?;
                ensure!(
                    resolved.destination.map_name == target
                        && resolved.destination.tile
                            == TilePosition::new(
                                i16::try_from(expected[0])?,
                                i16::try_from(expected[1])?
                            ),
                    "native R02 arrival differs"
                );
            }
        }
    }
    Ok(())
}
