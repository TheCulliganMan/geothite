//! Keep the Red River and prairie drainage crossings usable above and below the boards.
use anyhow::{Result, ensure};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::{MovementMode, StepOptions, StepOutcome};

use crate::CompiledGamePack;
use crate::outdoor_contour_paths::check_passage;

const WEST: &str = "MnR16C00R00";
const EAST: &str = "MnR16C01R00";

pub(super) fn verify(pack: &CompiledGamePack) -> Result<()> {
    for y in [23, 24] {
        check_passage(
            pack,
            (WEST, 9, y),
            Direction::Right,
            MovementMode::Normal,
            18,
            (WEST, 27, y),
        )?;
        check_passage(
            pack,
            (WEST, 27, y),
            Direction::Left,
            MovementMode::Normal,
            18,
            (WEST, 9, y),
        )?;
    }
    for y in [29, 30] {
        check_passage(
            pack,
            (WEST, 59, y),
            Direction::Right,
            MovementMode::Normal,
            9,
            (EAST, 4, y),
        )?;
        check_passage(
            pack,
            (EAST, 4, y),
            Direction::Left,
            MovementMode::Normal,
            9,
            (WEST, 59, y),
        )?;
    }
    for (name, x, top) in [(WEST, 16, 21), (WEST, 63, 27), (EAST, 0, 27)] {
        check_passage(
            pack,
            (name, x, top),
            Direction::Down,
            MovementMode::Surf,
            6,
            (name, x, top + 6),
        )?;
        check_passage(
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
