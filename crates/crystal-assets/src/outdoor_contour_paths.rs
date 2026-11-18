//! Check outdoor stair flights and ledges on the installed blocks, without opening a session.
use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{Direction, OverworldMapData, TilePosition};
use crystal_core::world::movement::{
    LedgeJumpOutcome, OccupiedTile, PlayerMovementState, StepOptions, StepOutcome,
    attempt_ledge_jump_with_occupied_tiles, attempt_step_with_occupied_tiles,
};

use crate::CompiledGamePack;
use crate::authored_route_surface::walking_map_distances;

pub(super) fn verify(pack: &CompiledGamePack) -> Result<()> {
    for (name, start) in [
        ("MnR01C00R00", [24, 16]),
        ("MnR05C00R00", [37, 40]),
        ("MnR05C00R01", [37, 28]),
        ("MnR10C01R00", [20, 22]),
        ("MnR12C00R00", [48, 6]),
        ("MnR13C00R00", [39, 24]),
    ] {
        check_drop(pack, name, start, Direction::Down)?;
    }
    for (name, bottom, top, limit) in [
        ("MnBrainerd", [47, 20], [47, 13], 7),
        ("MnR01C00R00", [20, 34], [12, 34], 8),
        ("MnR15C00R00", [42, 14], [34, 14], 8),
        ("MnR01C00R00", [23, 14], [23, 11], 3),
        ("MnR01C00R00", [39, 20], [39, 17], 3),
        ("MnR09C00R00", [36, 38], [36, 34], 4),
        ("MnR09C00R00", [49, 42], [49, 39], 3),
        ("MnR10C01R00", [20, 24], [20, 19], 13),
        ("MnR13C00R00", [39, 26], [39, 24], 18),
        ("MnR13C00R01", [14, 6], [20, 6], 6),
    ] {
        check_walk(pack, name, bottom, top, limit)?;
    }
    Ok(())
}

pub(super) fn check_drop(
    pack: &CompiledGamePack,
    name: &str,
    start: [i16; 2],
    direction: Direction,
) -> Result<()> {
    let room = pack.data.maps.get(name).context("missing outdoor ledge")?;
    let map = OverworldMapData::from_attributes(name, &room.attributes, room.blocks.clone());
    let collisions = pack.data.tileset_collision(&room.attributes.tileset_name)?;
    let objects = room
        .objects
        .iter()
        .map(|o| {
            Ok(OccupiedTile {
                tile: TilePosition::new(i16::try_from(o.x)?, i16::try_from(o.y)?),
                object_identifier: o.object_identifier.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let mut player = PlayerMovementState::new(TilePosition::new(start[0], start[1]));
    let expected = player.tile.moved(direction).moved(direction);
    let reverse = match direction {
        Direction::Down => Direction::Up,
        Direction::Up => Direction::Down,
        Direction::Left => Direction::Right,
        Direction::Right => Direction::Left,
    };
    let options = StepOptions {
        force_step_after_turn: true,
        ..Default::default()
    };
    ensure!(
        matches!(attempt_ledge_jump_with_occupied_tiles(&mut player, direction, &map, &collisions, options, &objects), LedgeJumpOutcome::Jumped {to,..} if to == expected),
        "missing {direction:?} drop on {name} at {start:?}"
    );
    ensure!(
        !matches!(
            attempt_step_with_occupied_tiles(
                &mut player,
                reverse,
                &map,
                &collisions,
                options,
                &objects
            ),
            StepOutcome::Moved { .. }
        ),
        "climbable cliff face on {name} at {expected:?}"
    );
    Ok(())
}

pub(super) fn check_walk(
    pack: &CompiledGamePack,
    name: &str,
    bottom: [u16; 2],
    top: [u16; 2],
    limit: u32,
) -> Result<()> {
    let room = pack.data.maps.get(name).context("missing outdoor stair")?;
    let objects = room.objects.iter().map(|o| [o.x, o.y]).collect();
    for (from, to) in [(bottom, top), (top, bottom)] {
        let walked = walking_map_distances(pack, room, from, &objects)?;
        ensure!(
            walked.get(&to).is_some_and(|&n| n <= limit),
            "broken outdoor approach on {name} from {from:?} to {to:?} (distance {:?})",
            walked.get(&to)
        );
    }
    Ok(())
}
