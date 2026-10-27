//! Native collision checks for town doors, room returns and protected shores.
use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};

use super::design::Town;
use crate::{CompiledGamePack, MapModule};

pub(super) fn kind<'a>(
    pack: &'a CompiledGamePack,
    module: &MapModule,
    [x, y]: [u16; 2],
) -> Result<&'a str> {
    ensure!(
        x < module.attributes.width * 2 && y < module.attributes.height * 2,
        "{} coordinate out of bounds",
        module.id
    );
    let block = module.blocks
        [usize::from(y / 2) * usize::from(module.attributes.width) + usize::from(x / 2)];
    Ok(pack
        .data
        .tilesets
        .get(&module.attributes.tileset_name)
        .context("Duluth native tileset")?
        .collision
        .get(&format!("{block:02x}"))
        .context("Duluth source collision")?[usize::from(y % 2 * 2 + x % 2)]
    .as_str())
}

pub(super) fn neighbors([x, y]: [u16; 2]) -> impl Iterator<Item = [u16; 2]> {
    [
        x.checked_sub(1).map(|x| [x, y]),
        y.checked_sub(1).map(|y| [x, y]),
        x.checked_add(1).map(|x| [x, y]),
        y.checked_add(1).map(|y| [x, y]),
    ]
    .into_iter()
    .flatten()
}

pub(super) fn reachable(
    pack: &CompiledGamePack,
    module: &MapModule,
    start: [u16; 2],
    surf: bool,
) -> Result<BTreeSet<[u16; 2]>> {
    let occupied = module
        .objects
        .iter()
        .map(|a| [a.x, a.y])
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::from([start]);
    let mut todo = VecDeque::from([start]);
    while let Some(p) = todo.pop_front() {
        for q in neighbors(p) {
            if q[0] >= module.attributes.width * 2
                || q[1] >= module.attributes.height * 2
                || occupied.contains(&q)
                || seen.contains(&q)
            {
                continue;
            }
            let collision = kind(pack, module, q)?;
            if matches!(collision, "FLOOR" | "DOOR" | "WARP_CARPET_DOWN")
                || (surf && collision == "WATER")
            {
                seen.insert(q);
                if collision != "DOOR" {
                    todo.push_back(q);
                }
            }
        }
    }
    Ok(seen)
}

pub(super) fn validate(pack: &CompiledGamePack, town: &Town, modules: &[MapModule]) -> Result<()> {
    let exterior = modules
        .iter()
        .find(|m| m.id == town.id)
        .context("Duluth exterior")?;
    let seen = reachable(pack, exterior, [6, 24], false)?;
    let surf = reachable(pack, exterior, [6, 24], true)?;
    let doors = town.doors.iter().map(|d| d.tile).collect::<BTreeSet<_>>();
    let actors = exterior
        .objects
        .iter()
        .map(|a| [a.x, a.y])
        .collect::<BTreeSet<_>>();
    for y in 0..48 {
        for x in 0..64 {
            let p = [x, y];
            let collision = kind(pack, exterior, p)?;
            ensure!(
                collision != "CUT_TREE",
                "Duluth has incidental Cut collision"
            );
            ensure!(
                (collision == "DOOR") == doors.contains(&p),
                "Duluth has unbound/false facade door {p:?}"
            );
            if matches!(collision, "FLOOR" | "DOOR") {
                ensure!(
                    seen.contains(&p) || actors.contains(&p),
                    "disconnected Duluth floor {p:?}"
                );
            }
            if x == 0 || x == 63 || y == 0 || y == 47 {
                ensure!(
                    collision != "WATER"
                        && (!surf.contains(&p)
                            || matches!(p, [0, 24] | [0, 25] | [32, 0] | [33, 0])),
                    "Duluth has open outer walking/Surf edge {p:?}"
                );
            }
            if collision == "HOP_DOWN" {
                ensure!(
                    y == 20
                        && (12..=17).contains(&x)
                        && kind(pack, exterior, [x, y - 1])? == "FLOOR"
                        && kind(pack, exterior, [x, y + 1])? == "WALL"
                        && kind(pack, exterior, [x, y + 2])? == "FLOOR",
                    "changed Duluth native downhill ledge"
                );
            }
        }
    }
    let allowed = BTreeSet::from([
        ([28, 37], [28, 38]),
        ([29, 37], [29, 38]),
        ([30, 37], [30, 38]),
        ([31, 36], [32, 36]),
        ([31, 37], [31, 38]),
        ([31, 37], [32, 37]),
    ]);
    let mut banks = BTreeSet::new();
    for &p in &seen {
        if kind(pack, exterior, p)? != "FLOOR" {
            continue;
        }
        for q in neighbors(p) {
            if q[0] < 64 && q[1] < 48 && kind(pack, exterior, q)? == "WATER" {
                banks.insert((p, q));
            }
        }
    }
    ensure!(
        banks == allowed,
        "Duluth Surf access must remain in its one authored low cove"
    );
    for port in &town.ports {
        for p in [port.gate_metatile, port.landing_metatile] {
            ensure!(
                seen.contains(&p.map(|v| v * 2)),
                "disconnected reserved Duluth port"
            );
        }
    }
    for door in &town.doors {
        ensure!(
            seen.contains(&door.tile)
                && seen.contains(&door.return_tile)
                && kind(pack, exterior, door.return_tile)? == "FLOOR",
            "unsafe exterior door/return"
        );
        ensure!(
            door.arrival_tile == door.return_tile
                && door.return_tile == [door.tile[0], door.tile[1] + 1]
                && door.target_warp == 3,
            "Duluth requires passive FLOOR room arrival3"
        );
    }
    for module in modules {
        let start = if module.id == town.id {
            [6, 24]
        } else {
            town.interiors
                .iter()
                .find(|r| r.id == module.id)
                .context("Duluth authored room")?
                .passive_arrival
                .tile
        };
        let reachable = reachable(pack, module, start, false)?;
        ensure!(
            kind(pack, module, start)? == "FLOOR",
            "{} passive arrival not FLOOR",
            module.id
        );
        let residents = if module.id == town.id {
            &town.residents
        } else {
            &town
                .interiors
                .iter()
                .find(|r| r.id == module.id)
                .context("Duluth authored residents")?
                .residents
        };
        for actor in &module.objects {
            let counter_approach = residents
                .iter()
                .find(|source| actor.object_identifier.as_deref() == Some(source.id.as_str()))
                .and_then(|source| source.counter_tile.zip(source.approach))
                .is_some_and(|(counter, approach)| {
                    reachable.contains(&approach)
                        && kind(pack, module, counter).is_ok_and(|collision| collision == "COUNTER")
                        && actor.x.abs_diff(approach[0]) + actor.y.abs_diff(approach[1]) == 2
                        && (actor.x == approach[0] || actor.y == approach[1])
                        && u32::from(actor.x) + u32::from(approach[0]) == u32::from(counter[0]) * 2
                        && u32::from(actor.y) + u32::from(approach[1]) == u32::from(counter[1]) * 2
                });
            ensure!(
                kind(pack, module, [actor.x, actor.y])? == "FLOOR"
                    && (neighbors([actor.x, actor.y]).any(|p| reachable.contains(&p))
                        || counter_approach),
                "{} actor off FLOOR or inaccessible",
                module.id
            );
        }
        for sign in &module.events.bg_events {
            ensure!(
                kind(pack, module, [sign.x, sign.y])? == "WALL"
                    && neighbors([sign.x, sign.y]).any(|p| reachable.contains(&p)),
                "{} sign off WALL or inaccessible",
                module.id
            );
        }
        if module.id != town.id {
            ensure!(
                module.events.warps.len() == 3,
                "{} needs two carpets and passive3",
                module.id
            );
            for warp in &module.events.warps[..2] {
                ensure!(
                    kind(pack, module, [warp.x, warp.y])? == "WARP_CARPET_DOWN"
                        && reachable.contains(&[warp.x, warp.y]),
                    "{} unsafe carpet exit",
                    module.id
                );
            }
            let source = town
                .interiors
                .iter()
                .find(|r| r.id == module.id)
                .context("authored room")?;
            for actor in &source.residents {
                if let Some(counter) = actor.counter_tile {
                    ensure!(
                        kind(pack, module, counter)? == "COUNTER"
                            && actor.approach.is_some_and(|p| reachable.contains(&p)),
                        "{} blocked counter service",
                        actor.id
                    );
                }
            }
            if let Some(pc) = &source.pc {
                ensure!(
                    kind(pack, module, pc.tile)? == "PC" && reachable.contains(&pc.approach),
                    "clinic PC tile/approach changed"
                );
            }
        }
    }
    Ok(())
}
