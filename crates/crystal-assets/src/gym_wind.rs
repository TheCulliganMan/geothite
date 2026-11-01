//! Safe native directional walk lanes and their resting-position graph.
use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::world::collision::{TilesetCollision, permissions, sample_collision};
use crystal_core::world::map::{Direction, OverworldMapData, TilePosition};
use crystal_core::world::movement::{
    OccupiedTile, PlayerMovementState, StepOptions, StepOutcome, attempt_step_with_occupied_tiles,
};

use crate::gym_design::Gym;
use crate::{CompiledGamePack, MapModule};

fn kind(map: &OverworldMapData, collision: &TilesetCollision, at: [u16; 2]) -> Result<u8> {
    Ok(sample_collision(
        map,
        collision,
        TilePosition::new(i16::try_from(at[0])?, i16::try_from(at[1])?),
    )
    .context("wind outside room")?
    .permission)
}

fn direction(permission: u8) -> Option<Direction> {
    match permission {
        permissions::WALK_RIGHT => Some(Direction::Right),
        permissions::WALK_LEFT => Some(Direction::Left),
        _ => None,
    }
}

/// The source direct-step routine skips terrain checks. Reject unsafe landings
/// before publication instead of relying on a wall to stop a gust.
fn settle(
    map: &OverworldMapData,
    collision: &TilesetCollision,
    mut at: [u16; 2],
    occupied: &BTreeSet<[u16; 2]>,
) -> Result<[u16; 2]> {
    let mut seen = BTreeSet::new();
    while let Some(wind) = direction(kind(map, collision, at)?) {
        ensure!(seen.insert(at), "wind lane cycles at {at:?}");
        let x = match wind {
            Direction::Right => at[0].checked_add(1),
            Direction::Left => at[0].checked_sub(1),
            _ => unreachable!("horizontal wind direction"),
        }
        .context("wind crosses room edge")?;
        at = [x, at[1]];
        ensure!(!occupied.contains(&at), "wind hits an actor at {at:?}");
    }
    ensure!(
        kind(map, collision, at)? == permissions::FLOOR,
        "wind needs an ordinary floor landing at {at:?}"
    );
    Ok(at)
}

/// Publication planning only; controller tests must prove actual forced input.
pub(crate) fn reachable(
    pack: &CompiledGamePack,
    module: &MapModule,
    gym: &Gym,
    occupied: &BTreeSet<[u16; 2]>,
) -> Result<BTreeSet<[u16; 2]>> {
    ensure!(gym.ledges.is_empty(), "wind bridge rooms do not mix ledges");
    let map =
        OverworldMapData::from_attributes(&module.id, &module.attributes, module.blocks.clone());
    let collision = pack
        .data()
        .tileset_collision(&module.attributes.tileset_name)?;
    for symbol in gym.winds.keys() {
        ensure!(
            gym.rows.iter().any(|row| row.contains(*symbol)),
            "unused wind symbol"
        );
    }
    for y in 0..module.attributes.height * 2 {
        for x in 0..module.attributes.width * 2 {
            if direction(kind(&map, &collision, [x, y])?).is_some() {
                settle(&map, &collision, [x, y], occupied)?;
            }
        }
    }
    let bodies = occupied
        .iter()
        .map(|at| {
            Ok(OccupiedTile {
                tile: TilePosition::new(i16::try_from(at[0])?, i16::try_from(at[1])?),
                object_identifier: None,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let mut reached = BTreeSet::from([gym.arrival_core]);
    let mut todo = VecDeque::from([gym.arrival_core]);
    while let Some(at) = todo.pop_front() {
        for dir in [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ] {
            let mut player = PlayerMovementState::new(TilePosition::new(
                i16::try_from(at[0])?,
                i16::try_from(at[1])?,
            ));
            if let StepOutcome::Moved { to, .. } = attempt_step_with_occupied_tiles(
                &mut player,
                dir,
                &map,
                &collision,
                StepOptions {
                    force_step_after_turn: true,
                    ..Default::default()
                },
                &bodies,
            ) {
                let mut target = [u16::try_from(to.x)?, u16::try_from(to.y)?];
                if direction(kind(&map, &collision, target)?).is_some() {
                    target = settle(&map, &collision, target, occupied)?;
                }
                if reached.insert(target) {
                    todo.push_back(target);
                }
            }
        }
    }
    Ok(reached)
}
