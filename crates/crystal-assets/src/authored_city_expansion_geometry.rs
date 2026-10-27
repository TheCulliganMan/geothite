//! Native movement graphs, real door pairings and walking/Surf boundaries.

use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::{
    LedgeJumpOutcome, MovementMode, OccupiedTile, PlayerMovementState, StepOptions, StepOutcome,
    attempt_ledge_jump_with_occupied_tiles, attempt_step_with_occupied_tiles,
};

use crate::authored_city_expansion_design::Expansion;
use crate::authored_town_geometry::kind;
use crate::{CompiledGamePack, MapModule};

fn graph(
    pack: &CompiledGamePack,
    module: &MapModule,
    start: [u16; 2],
    surf: bool,
    bodies: bool,
    allow_ledges: bool,
) -> Result<BTreeSet<[u16; 2]>> {
    ensure!(
        matches!(kind(pack, module, start)?, "FLOOR" | "TALL_GRASS"),
        "{} arrival must be ordinary floor",
        module.id
    );
    ensure!(
        !module
            .objects
            .iter()
            .any(|object| [object.x, object.y] == start),
        "{} arrival is occupied by an actor/item",
        module.id
    );
    let map = pack.data.overworld_map(&module.id)?;
    let tileset = pack
        .data
        .tileset_collision(&module.attributes.tileset_name)?;
    let occupied = if bodies {
        module
            .objects
            .iter()
            .map(|object| {
                Ok(OccupiedTile {
                    tile: TilePosition::new(i16::try_from(object.x)?, i16::try_from(object.y)?),
                    object_identifier: object.object_identifier.clone(),
                })
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        Vec::new()
    };
    let active = module
        .events
        .warps
        .iter()
        .filter(|warp| {
            kind(pack, module, [warp.x, warp.y]).is_ok_and(|collision| collision != "FLOOR")
        })
        .map(|warp| [warp.x, warp.y])
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some([x, y]) = queue.pop_front() {
        for direction in [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ] {
            let mut player =
                PlayerMovementState::new(TilePosition::new(i16::try_from(x)?, i16::try_from(y)?));
            player.facing = direction;
            if surf {
                player.mode = MovementMode::Surf;
            }
            let step = attempt_step_with_occupied_tiles(
                &mut player,
                direction,
                &map,
                &tileset,
                StepOptions::default(),
                &occupied,
            );
            if !matches!(step, StepOutcome::Moved { .. }) {
                if !allow_ledges {
                    continue;
                }
                let jump = attempt_ledge_jump_with_occupied_tiles(
                    &mut player,
                    direction,
                    &map,
                    &tileset,
                    StepOptions::default(),
                    &occupied,
                );
                if !matches!(jump, LedgeJumpOutcome::Jumped { .. }) {
                    continue;
                }
            }
            let (Ok(tx), Ok(ty)) = (u16::try_from(player.tile.x), u16::try_from(player.tile.y))
            else {
                continue;
            };
            if tx >= module.attributes.width * 2 || ty >= module.attributes.height * 2 {
                continue;
            }
            let tile = [tx, ty];
            if seen.insert(tile) && !active.contains(&tile) {
                queue.push_back(tile);
            }
        }
    }
    Ok(seen)
}

fn adjacent(seen: &BTreeSet<[u16; 2]>, [x, y]: [u16; 2]) -> bool {
    [
        x.checked_sub(1).map(|x| [x, y]),
        y.checked_sub(1).map(|y| [x, y]),
        x.checked_add(1).map(|x| [x, y]),
        y.checked_add(1).map(|y| [x, y]),
    ]
    .into_iter()
    .flatten()
    .any(|tile| seen.contains(&tile))
}

pub(super) fn validate_relocations(
    base: &CompiledGamePack,
    old: &MapModule,
    pack: &CompiledGamePack,
    new: &MapModule,
) -> Result<()> {
    for (before, after) in old.events.warps.iter().zip(&new.events.warps) {
        ensure!(
            kind(base, old, [before.x, before.y])? == kind(pack, new, [after.x, after.y])?,
            "old city warp {} changed collision/trigger behavior",
            before.index
        );
    }
    for (before, after) in old.objects.iter().zip(&new.objects) {
        ensure!(
            kind(base, old, [before.x, before.y])? == kind(pack, new, [after.x, after.y])?,
            "old city object {} changed standing collision",
            before.script
        );
    }
    for (before, after) in old.events.coord_events.iter().zip(&new.events.coord_events) {
        ensure!(
            kind(base, old, [before.x, before.y])? == kind(pack, new, [after.x, after.y])?,
            "old city coordinate event {} changed collision",
            before.script_name
        );
    }
    Ok(())
}

pub(super) fn validate(pack: &CompiledGamePack, plan: &Expansion) -> Result<()> {
    let city = &pack.data.maps[&plan.city];
    let accessible = graph(pack, city, plan.arrival_core, false, true, true)?;
    for warp in &city.events.warps {
        ensure!(
            accessible.contains(&[warp.x, warp.y]),
            "old or new city entrance/return {} is inaccessible",
            warp.index
        );
    }
    for object in &city.objects {
        ensure!(
            matches!(
                kind(pack, city, [object.x, object.y])?,
                "FLOOR" | "TALL_GRASS"
            ),
            "city actor/item stands on blocked terrain: {}",
            object.script
        );
        ensure!(
            adjacent(&accessible, [object.x, object.y]),
            "old or new city object is isolated: {}",
            object.script
        );
    }
    for event in &city.events.bg_events {
        ensure!(
            adjacent(&accessible, [event.x, event.y]),
            "old or new city reading is isolated: {}",
            event.script
        );
    }
    validate_city_hops(pack, city, &accessible)?;
    for actor in &plan.residents {
        ensure!(
            pack.runtime_files.contains_key(&format!(
                "gfx/sprites/{}.png",
                actor
                    .sprite
                    .trim_start_matches("SPRITE_")
                    .to_ascii_lowercase()
            )),
            "missing city sprite"
        );
        let reachable = actor.approach.map_or_else(
            || adjacent(&accessible, actor.tile),
            |tile| accessible.contains(&tile),
        );
        ensure!(
            reachable,
            "city actor approach is unreachable: {}",
            actor.id
        );
    }
    for actor in &plan.trainers {
        ensure!(
            adjacent(&accessible, actor.tile),
            "city trainer inaccessible: {}",
            actor.id
        );
    }
    for item in &plan.items {
        ensure!(
            adjacent(&accessible, item.tile),
            "city item inaccessible: {}",
            item.id
        );
    }
    for sign in &plan.signs {
        ensure!(
            adjacent(&accessible, sign.tile),
            "city sign inaccessible: {}",
            sign.id
        );
    }
    for door in &plan.doors {
        ensure!(
            kind(pack, city, door.tile)? == door.collision && accessible.contains(&door.tile),
            "new facade is inaccessible or wrong collision: {}",
            door.target
        );
        ensure!(
            kind(pack, city, door.return_tile)? == "FLOOR"
                && accessible.contains(&door.return_tile),
            "unsafe city facade return: {}",
            door.target
        );
        let room = pack.data.maps.get(&door.target).context("facade room")?;
        let arrival = room
            .events
            .warps
            .iter()
            .find(|warp| i16::try_from(warp.index).ok() == Some(door.target_warp))
            .context("facade arrival index")?;
        ensure!(
            kind(pack, room, [arrival.x, arrival.y])? == "FLOOR",
            "facade must enter safe passive room floor"
        );
        ensure!(
            door.arrival_tile == door.return_tile,
            "facade exterior arrival must match its passive return"
        );
    }
    let mouths = plan
        .connections
        .iter()
        .flat_map(|link| link.town_edge)
        .collect::<BTreeSet<_>>();
    // Check terrain beyond checkpoints independently of temporarily standing
    // officers. Actual guard denial/removal belongs to controller verification.
    for surf in [false, true] {
        let seen = graph(pack, city, plan.arrival_core, surf, false, true)?;
        ensure!(
            mouths.iter().all(|tile| seen.contains(tile)),
            "city connections need connected dry roads"
        );
        for [x, y] in seen {
            if x == 0
                || y == 0
                || x + 1 == city.attributes.width * 2
                || y + 1 == city.attributes.height * 2
            {
                ensure!(
                    mouths.contains(&[x, y]),
                    "{} unconnected walking/Surf boundary at {x},{y}",
                    city.id
                );
            }
        }
    }
    for room in &plan.rooms {
        let module = &pack.data.maps[&room.id];
        let seen = graph(pack, module, room.arrival_core, false, true, true)?;
        for object in &module.objects {
            ensure!(
                matches!(
                    kind(pack, module, [object.x, object.y])?,
                    "FLOOR" | "TALL_GRASS"
                ),
                "room actor/item stands on blocked terrain: {}",
                object.script
            );
        }
        for warp in &room.warps {
            ensure!(
                seen.contains(&warp.tile),
                "{} warp {} is unreachable",
                room.id,
                warp.index
            );
            if warp.passive {
                ensure!(
                    kind(pack, module, warp.tile)? == "FLOOR",
                    "passive arrival triggers a warp"
                );
                ensure!(
                    !module
                        .objects
                        .iter()
                        .any(|object| [object.x, object.y] == warp.tile),
                    "{} passive arrival {} is occupied",
                    room.id,
                    warp.index
                );
            } else {
                ensure!(
                    kind(pack, module, warp.tile)? != "FLOOR",
                    "active exit has no native trigger"
                );
            }
            let destination = pack
                .data
                .runtime_map_metadata
                .get(&warp.target)
                .context("room target constant")?;
            let target = &pack.data.maps[&destination.name];
            ensure!(
                target
                    .events
                    .warps
                    .iter()
                    .any(|target| i16::try_from(target.index).ok() == Some(warp.target_warp)),
                "room destination warp missing"
            );
        }
        for actor in &room.residents {
            ensure!(
                actor
                    .approach
                    .map_or_else(|| adjacent(&seen, actor.tile), |tile| seen.contains(&tile)),
                "{} actor inaccessible {}",
                room.id,
                actor.id
            );
        }
        for trainer in &room.trainers {
            ensure!(
                adjacent(&seen, trainer.tile),
                "room trainer inaccessible {}",
                trainer.id
            );
        }
        for item in &room.items {
            ensure!(
                adjacent(&seen, item.tile),
                "room item inaccessible {}",
                item.id
            );
        }
        for sign in &room.signs {
            ensure!(
                adjacent(&seen, sign.tile),
                "room sign inaccessible {}",
                sign.id
            );
        }
    }
    Ok(())
}

fn validate_city_hops(
    pack: &CompiledGamePack,
    city: &MapModule,
    accessible: &BTreeSet<[u16; 2]>,
) -> Result<()> {
    let map = pack.data.overworld_map(&city.id)?;
    let tileset = pack.data.tileset_collision(&city.attributes.tileset_name)?;
    let occupied = city
        .objects
        .iter()
        .map(|object| {
            Ok(OccupiedTile {
                tile: TilePosition::new(i16::try_from(object.x)?, i16::try_from(object.y)?),
                object_identifier: object.object_identifier.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    for &[x, y] in accessible {
        let direction = match kind(pack, city, [x, y])? {
            "HOP_DOWN" => Direction::Down,
            "HOP_UP" => Direction::Up,
            "HOP_LEFT" => Direction::Left,
            "HOP_RIGHT" => Direction::Right,
            _ => continue,
        };
        let (dx, dy) = direction.delta();
        let upper = [
            u16::try_from(i16::try_from(x)? - dx)?,
            u16::try_from(i16::try_from(y)? - dy)?,
        ];
        ensure!(
            accessible.contains(&upper),
            "city hop has no reachable upper approach at {x},{y}"
        );
        let mut player =
            PlayerMovementState::new(TilePosition::new(i16::try_from(x)?, i16::try_from(y)?));
        let jump = attempt_ledge_jump_with_occupied_tiles(
            &mut player,
            direction,
            &map,
            &tileset,
            StepOptions::default(),
            &occupied,
        );
        ensure!(
            matches!(jump, LedgeJumpOutcome::Jumped { .. }),
            "city hop lands on blocked/occupied terrain at {x},{y}"
        );
        let landing = [u16::try_from(player.tile.x)?, u16::try_from(player.tile.y)?];
        ensure!(
            matches!(kind(pack, city, landing)?, "FLOOR" | "TALL_GRASS"),
            "city hop needs safe dry landing"
        );
        let reverse = match direction {
            Direction::Down => Direction::Up,
            Direction::Up => Direction::Down,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        };
        player.facing = reverse;
        ensure!(
            !matches!(
                attempt_step_with_occupied_tiles(
                    &mut player,
                    reverse,
                    &map,
                    &tileset,
                    StepOptions::default(),
                    &occupied
                ),
                StepOutcome::Moved { .. }
            ),
            "city hop can climb back directly at {x},{y}"
        );
        let return_course = graph(pack, city, landing, false, true, false)?;
        ensure!(
            return_course.contains(&upper),
            "city hop has no ordinary walking return at {x},{y}"
        );
    }
    Ok(())
}
