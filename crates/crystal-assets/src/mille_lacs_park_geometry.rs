//! Exact park terrain contracts and native reciprocal road projections.
use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{Direction, OverworldMapData, TilePosition};
use crystal_core::world::movement::{
    LedgeJumpOutcome, PlayerMovementState, StepOptions, StepOutcome,
    attempt_ledge_jump_with_occupied_tiles, attempt_step_with_occupied_tiles,
};
use crystal_core::world::session::connection_for_tile_checked;

use crate::authored_town_geometry::{kind, reachable};
use crate::{CompiledGamePack, MapModule};

use super::{MAP, ROUTE};

pub(super) fn validate(pack: &CompiledGamePack, map: &MapModule) -> Result<()> {
    let walk = reachable(pack, map, [0, 12], false)?;
    let surf = reachable(pack, map, [0, 12], true)?;
    let bodies = map
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    ensure!(
        bodies.len() == 6,
        "park residents/reward must have distinct tiles"
    );
    for y in 0..40 {
        for x in 0..48 {
            let at = [x, y];
            let k = kind(pack, map, at)?;
            ensure!(
                k != "CUT_TREE",
                "ordinary park trees cannot be incidental Cut gates"
            );
            if matches!(k, "FLOOR" | "TALL_GRASS") && !bodies.contains(&at) {
                ensure!(surf.contains(&at), "stranded park floor at {at:?}");
                ensure!(
                    walk.contains(&at) || ((34..=37).contains(&x) && (28..=31).contains(&y)),
                    "only the offshore roost may require Surf"
                );
            }
            if x == 0 || x == 47 || y == 0 || y == 39 {
                ensure!(
                    k == "WALL" || (x == 0 && (y == 12 || y == 13) && k == "FLOOR"),
                    "park has an undeclared open walking/water edge"
                );
            }
        }
    }
    ensure!(
        surf.contains(&[37, 29]) && !walk.contains(&[37, 29]),
        "roost requires Surf"
    );
    let grassless = walk
        .iter()
        .copied()
        .filter(|at| kind(pack, map, *at).is_ok_and(|k| k != "TALL_GRASS"))
        .collect::<BTreeSet<_>>();
    ensure!(
        distance([0, 12], [16, 16], &grassless).is_none(),
        "bird lookout has a dry bypass around its grass neck"
    );
    ensure!(
        distance([16, 20], [16, 17], &walk).is_some_and(|n| n >= 9),
        "ledge requires a useful ordinary uphill return"
    );
    for x in 18..28 {
        ensure!(
            kind(pack, map, [x, 20])? == "WALL",
            "missing upper dock rail"
        );
        ensure!(
            kind(pack, map, [x, 25])? == "WALL",
            "missing lower dock rail"
        );
        ensure!(walk.contains(&[x, 22]), "dock has no passing lane");
    }
    for y in 20..26 {
        ensure!(kind(pack, map, [28, y])? == "WALL", "dock end is open");
    }
    for y in 12..36 {
        ensure!(
            kind(pack, map, [45, y])? == "BUOY",
            "missing eastern lake buoy"
        );
    }
    for x in 22..44 {
        ensure!(
            kind(pack, map, [x, 37])? == "BUOY",
            "missing southern lake buoy"
        );
    }
    let native = OverworldMapData::from_attributes(MAP, &map.attributes, map.blocks.clone());
    let tileset = pack.data.tileset_collision("mn_headwaters")?;
    for x in [16, 17] {
        ensure!(
            walk.contains(&[x, 17]) && walk.contains(&[x, 20]),
            "ledge approaches disconnected"
        );
        for at in [[x, 19], [15, 19], [18, 19]] {
            ensure!(kind(pack, map, at)? == "WALL", "ledge reverse/side bypass");
        }
        let mut player = PlayerMovementState::new(TilePosition::new(x as i16, 18));
        ensure!(
            matches!(attempt_ledge_jump_with_occupied_tiles(&mut player, Direction::Down,
                &native, &tileset, StepOptions::default(), &[]),
                LedgeJumpOutcome::Jumped {to, ..} if to == TilePosition::new(x as i16, 20)),
            "native lookout ledge landing differs"
        );
        player.facing = Direction::Up;
        ensure!(
            matches!(
                attempt_step_with_occupied_tiles(
                    &mut player,
                    Direction::Up,
                    &native,
                    &tileset,
                    StepOptions::default(),
                    &[]
                ),
                StepOutcome::Blocked { .. }
            ),
            "lookout ledge permits reverse movement"
        );
    }
    Ok(())
}

fn distance(start: [u16; 2], target: [u16; 2], floor: &BTreeSet<[u16; 2]>) -> Option<usize> {
    let mut seen = BTreeSet::from([start]);
    let mut todo = VecDeque::from([(start, 0)]);
    while let Some(([x, y], steps)) = todo.pop_front() {
        if [x, y] == target {
            return Some(steps);
        }
        for at in [
            x.checked_sub(1).map(|v| [v, y]),
            Some([x + 1, y]),
            y.checked_sub(1).map(|v| [x, v]),
            Some([x, y + 1]),
        ]
        .into_iter()
        .flatten()
        {
            if floor.contains(&at) && seen.insert(at) {
                todo.push_back((at, steps + 1));
            }
        }
    }
    None
}

pub(super) fn connect(pack: &mut CompiledGamePack) -> Result<()> {
    let route = &pack.data.maps[ROUTE];
    ensure!(
        [route.attributes.width, route.attributes.height] == [8, 24]
            && route.attributes.tileset_name == "mn_headwaters"
            && route.attributes.connections.len() == 1
            && route.attributes.connections[0].direction == "west"
            && route.attributes.connections[0].target_map == "MnR18C00R00"
            && route.attributes.connections[0].offset == 0,
        "retain the existing R18 western seam"
    );
    for (name, direction, target, offset) in [(ROUTE, "east", MAP, -1), (MAP, "west", ROUTE, 1)] {
        let mut map = pack.data.maps[name].clone();
        crate::lake_country_geometry::connect(&mut map, direction, target, offset)?;
        pack.data
            .map_attributes
            .insert(name.into(), map.attributes.clone());
        pack.data.maps.insert(name.into(), map);
    }
    for (name, target, at, outside, wanted) in [
        (ROUTE, MAP, [15, 10], [16, 10], [0, 12]),
        (ROUTE, MAP, [15, 11], [16, 11], [0, 13]),
        (MAP, ROUTE, [0, 12], [-1, 12], [15, 10]),
        (MAP, ROUTE, [0, 13], [-1, 13], [15, 11]),
    ] {
        let map = &pack.data.maps[name];
        ensure!(
            kind(pack, map, at)? == "FLOOR"
                && kind(pack, &pack.data.maps[target], wanted)? == "FLOOR",
            "both park seam lanes require native FLOOR"
        );
        let native = OverworldMapData::from_attributes(name, &map.attributes, map.blocks.clone());
        let transition =
            connection_for_tile_checked(&native, TilePosition::new(outside[0], outside[1]))?
                .context("park reciprocal road")?;
        let destination = pack
            .data
            .resolve_connection_transition(&transition)?
            .destination;
        ensure!(
            destination.map_name == target
                && destination.tile == TilePosition::new(wanted[0] as i16, wanted[1] as i16),
            "park native connection projection differs"
        );
    }
    Ok(())
}
