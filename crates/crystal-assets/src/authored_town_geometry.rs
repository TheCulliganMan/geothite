//! Native door, actor, service, return and closed-boundary checks.
use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};

use crate::authored_town_design::Town;
use crate::{CompiledGamePack, MapModule};

pub(crate) fn kind<'a>(
    pack: &'a CompiledGamePack,
    map: &MapModule,
    [x, y]: [u16; 2],
) -> Result<&'a str> {
    ensure!(
        x < map.attributes.width * 2 && y < map.attributes.height * 2,
        "{} tile outside map",
        map.id
    );
    let b = map.blocks[usize::from(y / 2) * usize::from(map.attributes.width) + usize::from(x / 2)];
    Ok(pack
        .data
        .tilesets
        .get(&map.attributes.tileset_name)
        .context("authored town original tileset")?
        .collision
        .get(&format!("{b:02x}"))
        .context("authored town native collision")?[usize::from(y % 2 * 2 + x % 2)]
    .as_str())
}

fn neighbors([x, y]: [u16; 2]) -> impl Iterator<Item = [u16; 2]> {
    [
        x.checked_sub(1).map(|x| [x, y]),
        y.checked_sub(1).map(|y| [x, y]),
        x.checked_add(1).map(|x| [x, y]),
        y.checked_add(1).map(|y| [x, y]),
    ]
    .into_iter()
    .flatten()
}

pub(crate) fn reachable(
    pack: &CompiledGamePack,
    map: &MapModule,
    start: [u16; 2],
    surf: bool,
) -> Result<BTreeSet<[u16; 2]>> {
    let bodies = map
        .objects
        .iter()
        .map(|a| [a.x, a.y])
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::from([start]);
    let mut todo = VecDeque::from([start]);
    while let Some(p) = todo.pop_front() {
        for q in neighbors(p) {
            if q[0] >= map.attributes.width * 2
                || q[1] >= map.attributes.height * 2
                || bodies.contains(&q)
                || seen.contains(&q)
            {
                continue;
            }
            let k = kind(pack, map, q)?;
            if matches!(
                k,
                "FLOOR"
                    | "TALL_GRASS"
                    | "LADDER"
                    | "DOOR"
                    | "WARP_CARPET_DOWN"
                    | "WARP_CARPET_LEFT"
                    | "WARP_CARPET_RIGHT"
            ) || (surf && k == "WATER")
            {
                seen.insert(q);
                if !matches!(
                    k,
                    "DOOR" | "WARP_CARPET_DOWN" | "WARP_CARPET_LEFT" | "WARP_CARPET_RIGHT"
                ) {
                    todo.push_back(q);
                }
            }
        }
    }
    Ok(seen)
}

pub(super) fn validate(
    pack: &CompiledGamePack,
    town: &Town,
    modules: &[(MapModule, [u16; 2])],
) -> Result<()> {
    validate_with_edges(pack, town, modules, &BTreeSet::new())
}

/// The caller must install and verify a reciprocal connection for every allowed
/// exterior edge before publishing the resulting pack.
pub(crate) fn validate_with_edges(
    pack: &CompiledGamePack,
    town: &Town,
    modules: &[(MapModule, [u16; 2])],
    bound_edges: &BTreeSet<[u16; 2]>,
) -> Result<()> {
    let mut labels = BTreeSet::new();
    for (map, start) in modules {
        ensure!(
            kind(pack, map, *start)? == "FLOOR",
            "{} arrival must be FLOOR",
            map.id
        );
        let seen = reachable(pack, map, *start, false)?;
        let bodies = map
            .objects
            .iter()
            .map(|o| [o.x, o.y])
            .collect::<BTreeSet<_>>();
        ensure!(
            bodies.len() == map.objects.len() && !bodies.contains(start),
            "authored town overlapping actor/arrival"
        );
        let actors = if map.id == town.id {
            &town.residents
        } else {
            &town
                .interiors
                .iter()
                .find(|r| r.id == map.id)
                .context("authored town source room")?
                .residents
        };
        for actor in actors {
            ensure!(
                labels.insert(actor.id.as_str()) && kind(pack, map, actor.tile)? == "FLOOR",
                "{}: duplicate or blocked actor {} at {:?} ({})",
                map.id,
                actor.id,
                actor.tile,
                kind(pack, map, actor.tile)?
            );
            let mut approaches = neighbors(actor.tile).collect::<Vec<_>>();
            if let (Some(counter), Some(approach)) = (actor.counter_tile, actor.approach) {
                ensure!(
                    kind(pack, map, counter)? == "COUNTER"
                        && counter[0].abs_diff(actor.tile[0]) + counter[1].abs_diff(actor.tile[1])
                            == 1
                        && counter[0].abs_diff(approach[0]) + counter[1].abs_diff(approach[1]) == 1,
                    "authored town counter interaction changed"
                );
                approaches.push(approach);
            }
            ensure!(
                approaches.iter().any(|p| seen.contains(p)),
                "authored town actor cannot be approached: {}",
                actor.id
            );
            if let Some(p) = actor.approach {
                ensure!(seen.contains(&p), "authored town reserved approach blocked");
            }
        }
        ensure!(
            map.events
                .warps
                .iter()
                .map(|w| [w.x, w.y])
                .collect::<BTreeSet<_>>()
                .len()
                == map.events.warps.len(),
            "authored town warp overlap"
        );
        for bg in &map.events.bg_events {
            ensure!(
                neighbors([bg.x, bg.y]).any(|p| seen.contains(&p)),
                "authored town sign/PC has no approach: {} ({},{})",
                map.id,
                bg.x,
                bg.y
            );
        }
        if map.id == town.id {
            for door in &town.doors {
                ensure!(
                    seen.contains(&door.tile)
                        && kind(pack, map, door.tile)? == "DOOR"
                        && seen.contains(&door.return_tile)
                        && kind(pack, map, door.return_tile)? == "FLOOR",
                    "authored town unsafe native facade/return"
                );
            }
            let paired = town.doors.iter().map(|d| d.tile).collect::<BTreeSet<_>>();
            for p in &seen {
                ensure!(
                    kind(pack, map, *p)? != "DOOR" || paired.contains(p),
                    "authored town unpaired facade"
                );
            }
            for port in &town.ports {
                ensure!(
                    seen.contains(&port.landing_metatile.map(|n| n * 2)),
                    "authored town port disconnected"
                );
            }
            for p in reachable(pack, map, *start, true)? {
                ensure!(
                    bound_edges.contains(&p)
                        || (p[0] > 0
                            && p[0] < map.attributes.width * 2 - 1
                            && p[1] > 0
                            && p[1] < map.attributes.height * 2 - 1),
                    "authored town unbound walking/Surf edge must remain closed"
                );
            }
        } else {
            let room = town
                .interiors
                .iter()
                .find(|r| r.id == map.id)
                .context("authored town room contract")?;
            for exit in &room.exits {
                ensure!(
                    seen.contains(&exit.tile) && kind(pack, map, exit.tile)? == "WARP_CARPET_DOWN",
                    "authored town carpet/return disconnected"
                );
            }
            if let Some(pc) = &room.pc {
                ensure!(
                    seen.contains(&pc.approach) && kind(pack, map, pc.tile)? == "PC",
                    "authored town native PC inaccessible"
                );
            }
        }
        ensure!(
            map.attributes.connections.is_empty(),
            "authored town regional binding remains separate"
        );
    }
    Ok(())
}
