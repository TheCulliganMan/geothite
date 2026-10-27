//! Reciprocal land seams and Lake-only checkpoints, with free inbound return.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;

use crate::authored_checkpoint::{Checkpoint, install as checkpoint};
use crate::authored_town_geometry::{kind, reachable};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

struct Link {
    town: &'static str,
    route: &'static str,
    direction: &'static str,
    offset: i32,
    town_edge: [[u16; 2]; 2],
    route_edge: [[u16; 2]; 2],
}
const LINKS: [Link; 4] = [
    Link {
        town: "MnBrainerd",
        route: "MnR07C00R01",
        direction: "north",
        offset: 6,
        town_edge: [[32, 0], [33, 0]],
        route_edge: [[20, 47], [21, 47]],
    },
    Link {
        town: "MnLeechLake",
        route: "MnR07C00R00",
        direction: "south",
        offset: 6,
        town_edge: [[32, 47], [33, 47]],
        route_edge: [[20, 0], [21, 0]],
    },
    Link {
        town: "MnLeechLake",
        route: "MnR08C00R01",
        direction: "north",
        offset: 3,
        town_edge: [[32, 0], [33, 0]],
        route_edge: [[26, 47], [27, 47]],
    },
    Link {
        town: "MnBemidji",
        route: "MnR08C00R00",
        direction: "south",
        offset: -8,
        town_edge: [[24, 47], [25, 47]],
        route_edge: [[40, 0], [41, 0]],
    },
];

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let mut maps = [
        "MnBrainerd",
        "MnR07C00R00",
        "MnR07C00R01",
        "MnLeechLake",
        "MnR08C00R00",
        "MnR08C00R01",
        "MnBemidji",
    ]
    .into_iter()
    .map(|name| {
        Ok((
            name.to_owned(),
            pack.data
                .maps
                .get(name)
                .context("installed northern road map")?
                .clone(),
        ))
    })
    .collect::<Result<BTreeMap<String, MapModule>>>()?;
    for (name, [x, y], old) in [
        ("MnR07C00R00", [11, 0], 2),
        ("MnR07C00R01", [11, 23], 2),
        ("MnR08C00R01", [11, 23], 2),
        ("MnR08C00R01", [12, 23], 2),
        ("MnR08C00R01", [14, 23], 2),
        ("MnR08C00R01", [15, 23], 2),
        ("MnR08C00R00", [0, 17], 2),
        ("MnR08C00R00", [0, 18], 2),
        ("MnBemidji", [12, 0], 1),
        ("MnBemidji", [0, 12], 1),
        ("MnBemidji", [23, 12], 1),
    ] {
        let map = maps.get_mut(name).context("visible northern cap")?;
        let index = usize::from(y) * usize::from(map.attributes.width) + usize::from(x);
        ensure!(map.blocks[index] == old, "stale northern cap source");
        map.blocks[index] = 5;
        for tile in [
            [x * 2, y * 2],
            [x * 2 + 1, y * 2],
            [x * 2, y * 2 + 1],
            [x * 2 + 1, y * 2 + 1],
        ] {
            ensure!(
                kind(pack, map, tile)? == "WALL",
                "visible forest must block walking and Surf"
            );
        }
    }
    let brainerd = maps.get_mut("MnBrainerd").context("Brainerd north mouth")?;
    ensure!(
        brainerd.blocks[16] == 5,
        "closed Brainerd north mouth required"
    );
    brainerd.blocks[16] = 1;
    for link in &LINKS {
        let opposite = if link.direction == "north" {
            "south"
        } else {
            "north"
        };
        for (name, target, side, offset, edge) in [
            (
                link.town,
                link.route,
                link.direction,
                link.offset,
                link.town_edge,
            ),
            (
                link.route,
                link.town,
                opposite,
                -link.offset,
                link.route_edge,
            ),
        ] {
            let map = maps.get_mut(name).context("native northern seam")?;
            for tile in edge {
                ensure!(
                    kind(pack, map, tile)? == "FLOOR",
                    "northern seam must use real floor"
                );
            }
            crate::lake_country_geometry::connect(map, side, target, offset)?;
        }
    }
    for (name, root, prefix, lines) in [
        (
            "MnBrainerd",
            "MnBrainerdLakeCheck",
            "MN_BRAINERD_NORTH_WARDEN",
            ["Earn SIVREN's", "LAKE BADGE first.", "Then head north."],
        ),
        (
            "MnLeechLake",
            "MnLeechLakeCheck",
            "MN_LEECH_NORTH_WARDEN",
            ["BEMIDJI is north.", "Earn the LAKE", "BADGE in BRAINERD."],
        ),
    ] {
        let map = maps.get_mut(name).context("Lake checkpoint")?;
        let ids = [format!("{prefix}_WEST"), format!("{prefix}_EAST")];
        checkpoint(
            pack,
            map,
            &Checkpoint {
                identifiers: &ids,
                bodies: [[32, 2], [33, 2]],
                inside: [[32, 3], [33, 3]],
                outside: [[32, 1], [33, 1]],
                root,
                incoming_facing: "DOWN",
                outward: "north",
                required_event: "",
                badge_flags: &["ENGINE_MINERALBADGE"],
                badge_lines: &lines.map(String::from),
                evidence_lines: &[],
            },
        )?;
        for e in &mut map.events.coord_events {
            if e.script_name.starts_with(root) {
                e.scene_id = "-1".into();
            }
        }
        let closed = reachable(pack, map, [32, 44], true)?;
        ensure!(
            !closed.contains(&[32, 0]) && !closed.contains(&[33, 0]),
            "Lake checkpoint has a walking/Surf bypass"
        );
    }
    for (name, start, edges) in [
        (
            "MnBrainerd",
            [32, 44],
            vec![[32, 0], [33, 0], [32, 47], [33, 47]],
        ),
        (
            "MnLeechLake",
            [32, 44],
            vec![[32, 0], [33, 0], [32, 47], [33, 47]],
        ),
        ("MnBemidji", [24, 44], vec![[24, 47], [25, 47]]),
    ] {
        let mut open = maps[name].clone();
        open.objects.retain(|o| {
            !o.object_identifier.as_deref().is_some_and(|id| {
                id.starts_with("MN_BRAINERD_NORTH_WARDEN")
                    || id.starts_with("MN_LEECH_NORTH_WARDEN")
            })
        });
        crate::prairie_story_connections::perimeter(pack, &open, start, &edges)?;
    }
    let names = pack
        .data
        .runtime_map_metadata
        .values()
        .map(|m| (m.constant.clone(), m.name.clone()))
        .collect();
    for (_, mut map) in maps {
        authoring::finish(pack, &mut map)?;
        map.script_runtime_commands = crate::parse_script_runtime_commands(&map.id, &map.scripts)?;
        map.script_object_commands = crate::parse_script_object_commands(&map.id, &map.scripts)?;
        map.script_variable_commands =
            crate::parse_script_variable_commands(&map.id, &map.scripts)?;
        map.script_movements =
            crate::parse_script_movements(&map.id, &map.scripts, &map.script_object_commands)?;
        map.script_map_commands = crate::parse_script_map_commands(&map.id, &map.scripts, &names)?;
        crate::authored_town_publication::replace(pack, map)?;
    }
    crate::authored_region_geometry::route_perimeter(
        pack,
        "MnR07",
        [64, 96],
        &[[20, 0], [21, 0], [20, 95], [21, 95]],
    )?;
    crate::authored_region_geometry::route_perimeter(
        pack,
        "MnR08",
        [80, 96],
        &[[40, 0], [41, 0], [26, 95], [27, 95]],
    )?;
    for link in &LINKS {
        for lane in 0..2 {
            for (name, target, edge, to, north) in [
                (
                    link.town,
                    link.route,
                    link.town_edge[lane],
                    link.route_edge[lane],
                    link.direction == "north",
                ),
                (
                    link.route,
                    link.town,
                    link.route_edge[lane],
                    link.town_edge[lane],
                    link.direction != "north",
                ),
            ] {
                let map = &pack.data.maps[name];
                let native =
                    OverworldMapData::from_attributes(name, &map.attributes, map.blocks.clone());
                let out =
                    TilePosition::new(edge[0] as i16, edge[1] as i16 + if north { -1 } else { 1 });
                let t = connection_for_tile_checked(&native, out)?
                    .context("native lake-road projection")?;
                let d = pack.data.resolve_connection_transition(&t)?.destination;
                ensure!(
                    d.map_name == target && d.tile == TilePosition::new(to[0] as i16, to[1] as i16),
                    "native lake-road destination differs"
                );
            }
        }
    }
    Ok(())
}
