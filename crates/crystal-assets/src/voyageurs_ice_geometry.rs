//! Native slide graph validation, independent of room composition.
use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::world::collision::{
    is_direction_blocked, is_direction_blocked_leaving, permissions,
};
use crystal_core::world::map::Direction;

use super::Room;
use crate::authored_town_geometry::kind;
use crate::{CompiledGamePack, MapModule};

const DIRECTIONS: [Direction; 4] = [
    Direction::Left,
    Direction::Right,
    Direction::Up,
    Direction::Down,
];

pub(super) fn validate(pack: &CompiledGamePack, map: &MapModule, room: &Room) -> Result<()> {
    let bodies = map
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    let border = pack
        .data
        .tilesets
        .get("ice_path")
        .context("native Ice Path tileset")?
        .collision
        .get(&format!("{:02x}", map.attributes.border_block))
        .context("native ice border collision")?;
    ensure!(border.len() == 4, "four native border quadrants");
    for at in [[0_u16, 0], [1, 0], [0, 1], [1, 1]] {
        ensure!(
            border[usize::from(at[1]) * 2 + usize::from(at[0])] == "WALL",
            "solid native ice border"
        );
    }
    ensure!(kind(pack, map, room.arrival)? == "FLOOR", "dry ice arrival");
    for y in 0..map.attributes.height * 2 {
        for x in 0..map.attributes.width * 2 {
            if x == 0
                || y == 0
                || x == map.attributes.width * 2 - 1
                || y == map.attributes.height * 2 - 1
            {
                ensure!(kind(pack, map, [x, y])? == "WALL", "closed ice perimeter");
            }
        }
    }
    let states = reachable(pack, map, room.arrival, &bodies)?;
    for warp in &room.warps {
        ensure!(
            states.contains(&warp.tile),
            "{} unreachable ice warp {:?}",
            room.id,
            warp.tile
        );
    }
    for object in &map.objects {
        let at = [object.x, object.y];
        ensure!(
            kind(pack, map, at)? == "FLOOR",
            "ice actor needs dry ground"
        );
        ensure!(
            DIRECTIONS
                .iter()
                .filter_map(|&d| neighbor(map, at, d))
                .any(|p| states.contains(&p)),
            "{} unreachable ice actor {}",
            room.id,
            object.script
        );
    }
    // An item ball can act as a physical stopping body before collection.
    // Prove every reachable stop can return after all pickups disappear.
    let residents = room
        .residents
        .iter()
        .map(|r| Ok([u16::try_from(r.tile[0])?, u16::try_from(r.tile[1])?]))
        .collect::<Result<BTreeSet<_>>>()?;
    let clear = reachable(pack, map, room.arrival, &residents)?;
    let mut returning = BTreeSet::from([room.arrival]);
    loop {
        let before = returning.len();
        for &at in &clear {
            for d in DIRECTIONS {
                if returning.contains(&slide(pack, map, at, d, &residents)?) {
                    returning.insert(at);
                }
            }
        }
        if returning.len() == before {
            break;
        }
    }
    ensure!(
        returning == clear,
        "{} contains an ice trap after collection",
        room.id
    );
    Ok(())
}

fn neighbor(map: &MapModule, at: [u16; 2], d: Direction) -> Option<[u16; 2]> {
    let (dx, dy) = d.delta();
    let x = u16::try_from(i32::from(at[0]) + i32::from(dx)).ok()?;
    let y = u16::try_from(i32::from(at[1]) + i32::from(dy)).ok()?;
    (x < map.attributes.width * 2 && y < map.attributes.height * 2).then_some([x, y])
}

pub(super) fn slide(
    pack: &CompiledGamePack,
    map: &MapModule,
    mut at: [u16; 2],
    d: Direction,
    bodies: &BTreeSet<[u16; 2]>,
) -> Result<[u16; 2]> {
    while let Some(next) = neighbor(map, at, d) {
        let terrain = kind(pack, map, next)?;
        if (kind(pack, map, at)? == "UP_WALL"
            && is_direction_blocked_leaving(permissions::UP_WALL, d))
            || (terrain == "UP_WALL" && is_direction_blocked(permissions::UP_WALL, d))
        {
            return Ok(at);
        }
        if bodies.contains(&next)
            || !matches!(terrain, "FLOOR" | "ICE" | "ICE_2B" | "LADDER" | "UP_WALL")
        {
            return Ok(at);
        }
        at = next;
        if !matches!(terrain, "ICE" | "ICE_2B") {
            return Ok(at);
        }
    }
    Ok(at)
}

pub(super) fn reachable(
    pack: &CompiledGamePack,
    map: &MapModule,
    start: [u16; 2],
    bodies: &BTreeSet<[u16; 2]>,
) -> Result<BTreeSet<[u16; 2]>> {
    ensure!(!bodies.contains(&start), "clear ice arrival");
    let mut seen = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(at) = queue.pop_front() {
        for d in DIRECTIONS {
            let next = slide(pack, map, at, d, bodies)
                .with_context(|| format!("ice slide from {at:?}"))?;
            if seen.insert(next) {
                queue.push_back(next);
            }
        }
    }
    Ok(seen)
}
