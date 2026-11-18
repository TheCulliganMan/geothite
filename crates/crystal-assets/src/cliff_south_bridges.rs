//! Keep the Red River and prairie drainage crossings usable above and below the boards.
use anyhow::{Result, ensure};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::{MovementMode, StepOptions, StepOutcome};

use crate::CompiledGamePack;

const WEST: &str = "MnR16C00R00";
const EAST: &str = "MnR16C01R00";

fn passage(
    pack: &CompiledGamePack,
    start: (&str, i16, i16),
    direction: Direction,
    mode: MovementMode,
    steps: u16,
    end: (&str, i16, i16),
) -> Result<()> {
    let mut session =
        pack.data
            .overworld_session(start.0, TilePosition::new(start.1, start.2), 0)?;
    session.player.mode = mode;
    let options = StepOptions {
        force_step_after_turn: true,
        ..Default::default()
    };
    for _ in 0..steps {
        ensure!(
            matches!(
                session.step_checked(direction, options)?,
                StepOutcome::Moved { .. }
            ) && session.player.mode == mode,
            "broken {mode:?} passage from {start:?} toward {direction:?} at {:?}",
            session.player.tile
        );
        if let Some(trigger) = session.check_connection_checked()? {
            let destination = pack
                .data
                .resolve_connection_transition(&trigger)?
                .destination;
            session = pack
                .data
                .overworld_session(&destination.map_name, destination.tile, 0)?;
            session.player.mode = mode;
        }
    }
    ensure!(
        session.map.name == end.0 && session.player.tile == TilePosition::new(end.1, end.2),
        "prairie bridge from {start:?} did not reach {end:?}"
    );
    Ok(())
}

pub(super) fn verify(pack: &CompiledGamePack) -> Result<()> {
    for y in [23, 24] {
        passage(
            pack,
            (WEST, 9, y),
            Direction::Right,
            MovementMode::Normal,
            18,
            (WEST, 27, y),
        )?;
        passage(
            pack,
            (WEST, 27, y),
            Direction::Left,
            MovementMode::Normal,
            18,
            (WEST, 9, y),
        )?;
    }
    for y in [29, 30] {
        passage(
            pack,
            (WEST, 59, y),
            Direction::Right,
            MovementMode::Normal,
            9,
            (EAST, 4, y),
        )?;
        passage(
            pack,
            (EAST, 4, y),
            Direction::Left,
            MovementMode::Normal,
            9,
            (WEST, 59, y),
        )?;
    }
    for (name, x, top) in [(WEST, 16, 21), (WEST, 63, 27), (EAST, 0, 27)] {
        passage(
            pack,
            (name, x, top),
            Direction::Down,
            MovementMode::Surf,
            6,
            (name, x, top + 6),
        )?;
        passage(
            pack,
            (name, x, top + 6),
            Direction::Up,
            MovementMode::Surf,
            6,
            (name, x, top),
        )?;
        for (mode, direction) in [
            (MovementMode::Normal, Direction::Up),
            (MovementMode::Surf, Direction::Left),
            (MovementMode::Surf, Direction::Right),
        ] {
            let middle = TilePosition::new(x, top + 2);
            let mut session = pack.data.overworld_session(name, middle, 0)?;
            session.player.mode = mode;
            let outcome = session.step_checked(
                direction,
                StepOptions {
                    force_step_after_turn: true,
                    ..Default::default()
                },
            )?;
            ensure!(
                matches!(outcome, StepOutcome::Blocked { .. }) && session.player.tile == middle,
                "bridge layer leak at {name} {middle:?} moving {direction:?} in {mode:?}"
            );
        }
    }
    Ok(())
}
