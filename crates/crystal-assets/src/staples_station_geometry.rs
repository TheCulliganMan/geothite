//! Native station silhouette, public footing, forest spur and paired projections.
use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;

use crate::authored_town_design::{Actor, Room};
use crate::authored_town_geometry::{kind, reachable};
use crate::staples_station::{JUNCTION, ROAD, ROOM};
use crate::{CompiledGamePack, MapModule};

pub(crate) fn residents(exterior: &[Actor], room: &Room) -> Result<()> {
    ensure!(
        exterior.len() == 1 && room.residents.len() == 4,
        "five Staples residents required"
    );
    for (actor, (id, sprite, tile, approach, service)) in
        exterior.iter().chain(&room.residents).zip([
            (
                "MN_STAPLES_JUNCTION_WALKER",
                "SPRITE_FISHER",
                [18, 14],
                [17, 14],
                None,
            ),
            (
                "MN_STAPLES_CONDUCTOR",
                "SPRITE_OFFICER",
                [11, 9],
                [11, 10],
                None,
            ),
            (
                "MN_STAPLES_SIGNAL_KEEPER",
                "SPRITE_GRAMPS",
                [3, 11],
                [3, 12],
                None,
            ),
            (
                "MN_STAPLES_PARCEL_CLERK",
                "SPRITE_CLERK",
                [5, 13],
                [5, 14],
                Some("pokemart"),
            ),
            (
                "MN_STAPLES_WAITING_STUDENT",
                "SPRITE_LASS",
                [13, 14],
                [12, 14],
                None,
            ),
        ])
    {
        ensure!(
            actor.id == id
                && actor.sprite == sprite
                && actor.tile == tile
                && actor.approach == Some(approach)
                && actor.service.as_deref() == service
                && actor.counter_tile.is_none()
                && (service.is_some() || !actor.lines.is_empty())
                && actor
                    .lines
                    .iter()
                    .all(|line| !line.is_empty() && line.chars().count() <= 18),
            "Staples requires its local encounters, safe approaches and readable native dialogue"
        );
    }
    ensure!(
        room.residents[2].stock == ["REPEL", "SUPER_POTION", "PARLYZ_HEAL", "FLOWER_MAIL"],
        "parcel counter needs its four useful native supplies"
    );
    ensure!(
        room.signs.iter().all(|sign| !sign.lines.is_empty()
            && sign
                .lines
                .iter()
                .all(|line| !line.is_empty() && line.chars().count() <= 18)),
        "signal ledger must fit the native text box"
    );
    Ok(())
}

pub(crate) fn native(
    pack: &CompiledGamePack,
    old: &MapModule,
    road: &MapModule,
    junction: &MapModule,
    station: &MapModule,
) -> Result<()> {
    let mut preserved = road.clone();
    preserved.blocks = old.blocks.clone();
    preserved.attributes = old.attributes.clone();
    ensure!(
        &preserved == old,
        "all original Central Lakes programs and events must remain exact"
    );
    let mut attributes = road.attributes.clone();
    attributes.connections = old.attributes.connections.clone();
    attributes.connection_flags = old.attributes.connection_flags.clone();
    attributes.fishing_group = old.attributes.fishing_group.clone();
    ensure!(
        attributes == old.attributes,
        "the spur may only append its northern connection and local Minnesota fishing binding"
    );
    for y in 0..36 {
        for x in 0..64 {
            let before = kind(pack, old, [x, y])?;
            let after = kind(pack, road, [x, y])?;
            if before != "WALL" {
                ensure!(
                    before == after,
                    "Staples cannot change existing grass, water or field gates"
                );
            }
            if x == 0 || x == 63 || y == 35 || y == 0 {
                ensure!(
                    (y == 0 && [26, 27].contains(&x) && after == "FLOOR") || before == after,
                    "the spur may open only its two paired northern walking lanes"
                );
            }
        }
    }
    let public = reachable(pack, road, [26, 0], false)?;
    ensure!(
        [[27, 0], [18, 9], [18, 10], [19, 10]]
            .iter()
            .all(|p| public.contains(p)),
        "both northern lanes must meet the existing grass approach"
    );
    // Compare every old dry component, with real actor bodies blocking. The
    // new leaf cannot silently make required encounter grass avoidable.
    let mut unexamined = BTreeSet::new();
    for y in 0..36 {
        for x in 0..64 {
            if kind(pack, old, [x, y])? == "FLOOR" {
                unexamined.insert([x, y]);
            }
        }
    }
    while let Some(&start) = unexamined.first() {
        let before = dry(pack, old, start)?;
        let after = dry(pack, road, start)?;
        for p in &after {
            ensure!(
                kind(pack, old, *p)? != "FLOOR" || before.contains(p),
                "Staples spur creates a dry bypass between original grass-separated areas"
            );
        }
        unexamined.retain(|p| !before.contains(p));
    }
    for map in [junction, station] {
        let tileset = &pack.data.tilesets[&map.attributes.tileset_name];
        ensure!(
            map.blocks
                .iter()
                .all(|b| tileset
                    .collision
                    .get(&format!("{b:02x}"))
                    .is_some_and(|quadrants| !quadrants
                        .iter()
                        .any(|k| matches!(k.as_str(), "CUT_TREE" | "WATER" | "BUOY")))),
            "Staples must have deliberate dry native terrain without incidental Cut gates"
        );
    }
    for y in 0..20 {
        for x in 0..32 {
            if x == 0 || x == 31 || y == 0 || y == 19 {
                let expected = if [[6, 19], [7, 19]].contains(&[x, y]) {
                    "FLOOR"
                } else {
                    "WALL"
                };
                ensure!(
                    kind(pack, junction, [x, y])? == expected,
                    "the entire outpost boundary must be visibly closed except its paired spur"
                );
            }
        }
    }
    // The complete six-block facade comes directly from Goldenrod's station.
    let goldenrod = &pack.data.maps["GoldenrodCity"];
    for dy in 0..2 {
        for dx in 0..3 {
            ensure!(
                junction.blocks[(2 + dy) * 16 + 6 + dx] == goldenrod.blocks[(5 + dy) * 20 + 4 + dx],
                "keep Goldenrod's complete native station roof, wall and actual door"
            );
        }
    }
    for x in 2..14 {
        ensure!(
            junction.blocks[16 + x] == 0x7e,
            "native raised freight track must remain blocking"
        );
    }
    for (row, expected) in [
        (2, [0x0f, 0x14, 0x10, 0x11, 0x15, 0x12, 0x13, 0x14, 0x0f]),
        (4, [0x0f, 0x06, 0x06, 0x06, 0x06, 0x07, 0x06, 0x06, 0x0f]),
        (8, [0x0f, 0x0f, 0x0f, 0x0f, 0x09, 0x0f, 0x0f, 0x0f, 0x0f]),
    ] {
        ensure!(
            station.blocks[row * 9..(row + 1) * 9] == expected,
            "native train nose/tail, single boarding throat and exit shell must remain intact"
        );
    }
    ensure!(
        kind(pack, station, [4, 10])? == "WALL"
            && kind(pack, station, [4, 11])? == "FLOOR"
            && station
                .events
                .bg_events
                .iter()
                .any(|e| e.script == "MN_STAPLES_SIGNAL_BOOK" && [e.x, e.y] == [4, 10]),
        "the signal ledger must sit on its wall panel above a safe floor approach"
    );
    let public = reachable(pack, station, [9, 15], false)?;
    let mut unguarded = station.clone();
    unguarded
        .objects
        .retain(|o| o.script != "MN_STAPLES_CONDUCTOR");
    let boarding = reachable(pack, &unguarded, [11, 10], false)?;
    for tile in [[6, 5], [11, 5], [11, 7]] {
        ensure!(
            !public.contains(&tile) && boarding.contains(&tile),
            "the real conductor must be the only barrier to the full native platform"
        );
    }
    ensure!(
        kind(pack, station, [6, 5])? == "DOOR"
            && kind(pack, station, [11, 5])? == "DOOR"
            && kind(pack, station, [11, 11])? == "FLOOR"
            && station
                .events
                .warps
                .iter()
                .all(|w| [[8, 17], [9, 17], [9, 15]].contains(&[w.x, w.y]))
            && station.events.coord_events.is_empty(),
        "no public train-door warp or unsolicited boarding trigger may bypass service"
    );
    Ok(())
}

fn dry(pack: &CompiledGamePack, map: &MapModule, start: [u16; 2]) -> Result<BTreeSet<[u16; 2]>> {
    let bodies = map
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::from([start]);
    let mut todo = VecDeque::from([start]);
    while let Some([x, y]) = todo.pop_front() {
        for q in [
            x.checked_sub(1).map(|x| [x, y]),
            y.checked_sub(1).map(|y| [x, y]),
            x.checked_add(1).map(|x| [x, y]),
            y.checked_add(1).map(|y| [x, y]),
        ]
        .into_iter()
        .flatten()
        {
            if q[0] < map.attributes.width * 2
                && q[1] < map.attributes.height * 2
                && !bodies.contains(&q)
                && !seen.contains(&q)
                && kind(pack, map, q)? == "FLOOR"
            {
                seen.insert(q);
                todo.push_back(q);
            }
        }
    }
    Ok(seen)
}

pub(crate) fn seams(pack: &CompiledGamePack) -> Result<()> {
    ensure!(
        pack.data.maps[JUNCTION].attributes.connections.len() == 1,
        "Staples must remain a single walking leaf without a new regional shortcut"
    );
    for lane in 0..2 {
        for (name, at, outside, target, expected) in [
            (
                ROAD,
                [26 + lane, 0],
                [26 + lane, -1],
                JUNCTION,
                [6 + lane, 19],
            ),
            (
                JUNCTION,
                [6 + lane, 19],
                [6 + lane, 20],
                ROAD,
                [26 + lane, 0],
            ),
        ] {
            let map = &pack.data.maps[name];
            let unsigned = |p: [i16; 2]| -> Result<[u16; 2]> {
                Ok([u16::try_from(p[0])?, u16::try_from(p[1])?])
            };
            ensure!(
                kind(pack, map, unsigned(at)?)? == "FLOOR"
                    && kind(pack, &pack.data.maps[target], unsigned(expected)?)? == "FLOOR",
                "both paired Staples lanes need real native Floor"
            );
            let native =
                OverworldMapData::from_attributes(name, &map.attributes, map.blocks.clone());
            let transition =
                connection_for_tile_checked(&native, TilePosition::new(outside[0], outside[1]))?
                    .context("Staples walking seam")?;
            let arrival = pack
                .data
                .resolve_connection_transition(&transition)?
                .destination;
            ensure!(
                arrival.map_name == target
                    && arrival.tile == TilePosition::new(expected[0], expected[1]),
                "Staples lane must project onto its exact reciprocal arrival"
            );
        }
    }
    ensure!(
        pack.data.maps[ROOM].events.warps.len() == 3,
        "only two carpets and one passive hall arrival"
    );
    Ok(())
}
