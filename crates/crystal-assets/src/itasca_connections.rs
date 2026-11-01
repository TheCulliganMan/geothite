//! Public reference outing and report-gated Iron Range, with free returns.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;
use serde_json::Value;

use crate::authored_checkpoint::{Checkpoint, install as checkpoint};
use crate::authored_town_design::Town;
use crate::authored_town_geometry::{kind, reachable};
use crate::itasca_events::REPORT;
use crate::{CompiledGamePack, MapModule};

struct Link {
    town: &'static str,
    route: &'static str,
    side: &'static str,
    offset: i32,
    town_edge: [[u16; 2]; 2],
    route_edge: [[u16; 2]; 2],
}
const LINKS: [Link; 2] = [
    Link {
        town: "MnBemidji",
        route: "MnR14C01R00",
        side: "west",
        offset: -1,
        town_edge: [[0, 24], [0, 25]],
        route_edge: [[31, 26], [31, 27]],
    },
    Link {
        town: "MnBemidji",
        route: "MnR09C00R01",
        side: "east",
        offset: 11,
        town_edge: [[47, 24], [47, 25]],
        route_edge: [[0, 2], [0, 3]],
    },
];

pub(crate) fn install(pack: &mut CompiledGamePack, town: &Town, story: &Value) -> Result<()> {
    let mut maps = [
        "MnBemidji",
        "MnLakeItasca",
        "MnR14C01R00",
        "MnR14C00R00",
        "MnR09C00R01",
    ]
    .into_iter()
    .map(|name| {
        Ok((
            name.to_owned(),
            pack.data
                .maps
                .get(name)
                .context("installed reference map")?
                .clone(),
        ))
    })
    .collect::<Result<BTreeMap<String, MapModule>>>()?;
    let city = maps.get_mut("MnBemidji").context("Bemidji")?;
    for [x, y] in [[0, 12], [23, 12]] {
        let i = y * 24 + x;
        ensure!(city.blocks[i] == 5, "closed Bemidji reservation required");
        city.blocks[i] = 1;
    }
    crate::itasca_events::officer(city, story)?;
    for link in &LINKS {
        let reverse = if link.side == "west" { "east" } else { "west" };
        for (name, target, side, offset, edge) in [
            (
                link.town,
                link.route,
                link.side,
                link.offset,
                link.town_edge,
            ),
            (
                link.route,
                link.town,
                reverse,
                -link.offset,
                link.route_edge,
            ),
        ] {
            let map = maps.get_mut(name).context("reference seam")?;
            for tile in edge {
                ensure!(
                    kind(pack, map, tile)? == "FLOOR",
                    "reference seam requires native floor"
                );
            }
            crate::lake_country_geometry::connect(map, side, target, offset)?;
        }
    }
    let city = maps.get_mut("MnBemidji").context("Bemidji checkpoints")?;
    for (root, ids, bodies, inside, outside, facing, outward, flags, event, lines, evidence) in [
        (
            "MnBemidjiItascaCheck",
            [
                "MN_BEMIDJI_WEST_WARDEN_NORTH".into(),
                "MN_BEMIDJI_WEST_WARDEN_SOUTH".into(),
            ],
            [[2, 24], [2, 25]],
            [[3, 24], [3, 25]],
            [[1, 24], [1, 25]],
            "RIGHT",
            "west",
            vec!["ENGINE_MINERALBADGE"],
            "",
            vec![
                "Earn SIVREN's".into(),
                "LAKE BADGE first.".into(),
                "Then visit ITASCA.".into(),
            ],
            vec![],
        ),
        (
            "MnBemidjiIronCheck",
            [
                "MN_BEMIDJI_EAST_WARDEN_NORTH".into(),
                "MN_BEMIDJI_EAST_WARDEN_SOUTH".into(),
            ],
            [[45, 24], [45, 25]],
            [[44, 24], [44, 25]],
            [[46, 24], [46, 25]],
            "LEFT",
            "east",
            vec!["ENGINE_GLACIERBADGE"],
            REPORT,
            vec!["Earn ORLIS's".into(), "PINE BADGE first.".into()],
            vec![
                "Read the marker".into(),
                "at LAKE ITASCA.".into(),
                "Bring the reading".into(),
                "to the officer.".into(),
            ],
        ),
    ] {
        checkpoint(
            pack,
            city,
            &Checkpoint {
                identifiers: &ids,
                bodies,
                inside,
                outside,
                root,
                incoming_facing: facing,
                outward,
                required_event: event,
                badge_flags: &flags,
                badge_lines: &lines,
                evidence_lines: &evidence,
            },
        )?;
        for e in &mut city.events.coord_events {
            if e.script_name.starts_with(root) {
                e.scene_id = "-1".into();
            }
        }
    }
    let closed = reachable(pack, city, [24, 44], true)?;
    ensure!(
        [[0, 24], [0, 25], [47, 24], [47, 25]]
            .iter()
            .all(|p| !closed.contains(p)),
        "reference/report checkpoint has walking/Surf bypass"
    );
    let mut open = city.clone();
    open.objects.retain(|o| {
        !o.object_identifier.as_deref().is_some_and(|id| {
            id.starts_with("MN_BEMIDJI_WEST_WARDEN") || id.starts_with("MN_BEMIDJI_EAST_WARDEN")
        })
    });
    crate::prairie_story_connections::perimeter(
        pack,
        &open,
        [24, 44],
        &[[0, 24], [0, 25], [47, 24], [47, 25], [24, 47], [25, 47]],
    )?;
    for (_, mut map) in maps {
        crate::authored_town_scripts::finish(pack, &mut map, town)?;
        crate::authored_town_publication::replace(pack, map)?;
    }
    crate::authored_region_geometry::route_perimeter(
        pack,
        "MnR14",
        [96, 64],
        &[[5, 42], [5, 43], [95, 26], [95, 27]],
    )?;
    crate::authored_region_geometry::route_perimeter(
        pack,
        "MnR09",
        [128, 72],
        &[[0, 50], [0, 51], [127, 24], [127, 25]],
    )?;
    for link in &LINKS {
        for lane in 0..2 {
            for (name, target, edge, to, left) in [
                (
                    link.town,
                    link.route,
                    link.town_edge[lane],
                    link.route_edge[lane],
                    link.side == "west",
                ),
                (
                    link.route,
                    link.town,
                    link.route_edge[lane],
                    link.town_edge[lane],
                    link.side != "west",
                ),
            ] {
                let map = &pack.data.maps[name];
                let native =
                    OverworldMapData::from_attributes(name, &map.attributes, map.blocks.clone());
                let t = connection_for_tile_checked(
                    &native,
                    TilePosition::new(edge[0] as i16 + if left { -1 } else { 1 }, edge[1] as i16),
                )?
                .context("native reference projection")?;
                let d = pack.data.resolve_connection_transition(&t)?.destination;
                ensure!(
                    d.map_name == target && d.tile == TilePosition::new(to[0] as i16, to[1] as i16),
                    "native reference projection differs"
                );
            }
        }
    }
    Ok(())
}
