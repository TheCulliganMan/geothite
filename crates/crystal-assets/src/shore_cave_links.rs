//! Check the actual entry step, interior destination and return events for each outdoor cave.
use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::{StepOptions, StepOutcome};

use crate::CompiledGamePack;
use crate::authored_town_geometry::kind;

pub(super) fn verify(base: &CompiledGamePack, pack: &CompiledGamePack) -> Result<()> {
    for (name, old) in &base.data.maps {
        if !name.starts_with("Mn")
            || !matches!(
                old.attributes.environment.as_deref(),
                Some("ROUTE" | "TOWN")
            )
        {
            continue;
        }
        let room = &pack.data.maps[name];
        for (before, door) in old.events.warps.iter().zip(&room.events.warps) {
            if kind(base, old, [before.x, before.y])? != "CAVE" {
                continue;
            }
            let at = TilePosition::new(i16::try_from(door.x)?, i16::try_from(door.y)?);
            let south = at.moved(Direction::Down);
            ensure!(
                room.objects.iter().all(|o| [o.x, o.y] != [door.x, door.y]
                    && [i32::from(o.x), i32::from(o.y)]
                        != [i32::from(south.x), i32::from(south.y)]),
                "occupied cave approach on {name}"
            );
            let mut session = pack.data.overworld_session(name, south, 0)?;
            let entered = session.step_and_check_warp_checked(
                Direction::Up,
                StepOptions {
                    force_step_after_turn: true,
                    ..Default::default()
                },
            )?;
            ensure!(
                matches!(entered.outcome,StepOutcome::Moved{to,..} if to==at),
                "blocked cave on {name} at {at:?}"
            );
            let trigger = entered
                .warp
                .context("cave did not trigger its entrance warp")?;
            ensure!(
                trigger.warp == *door
                    && before.target_map_constant == door.target_map_constant
                    && before.target_warp_id == door.target_warp_id,
                "cave destination changed on {name}"
            );
            let destination = pack.data.resolve_warp_transition(&trigger)?.destination;
            let inside = &pack.data.maps[&destination.map_name];
            let mut returns = 0;
            for exit in &inside.events.warps {
                if old.attributes.map_constant.as_deref() != Some(exit.target_map_constant.as_str())
                {
                    continue;
                }
                let inside_tile = TilePosition::new(i16::try_from(exit.x)?, i16::try_from(exit.y)?);
                let mut inside_session = pack.data.overworld_session(&inside.id, inside_tile, 0)?;
                inside_session.player.facing = Direction::Down;
                // Some caves keep a passive floor warp as their interior arrival anchor.
                if let Some(trigger) = inside_session.check_warp_checked()? {
                    let arrival = pack.data.resolve_warp_transition(&trigger)?.destination;
                    let expected = usize::try_from(exit.target_warp_id)
                        .ok()
                        .and_then(|id| id.checked_sub(1))
                        .and_then(|id| room.events.warps.get(id))
                        .context("missing exterior return index")?;
                    ensure!(
                        arrival.map_name == *name && arrival.warp == *expected,
                        "wrong cave return from {}",
                        inside.id
                    );
                    returns += 1;
                }
            }
            ensure!(
                returns > 0,
                "missing active cave exit to {name} from {}",
                inside.id
            );
        }
    }
    Ok(())
}
