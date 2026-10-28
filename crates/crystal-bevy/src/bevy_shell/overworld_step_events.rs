//! Production step-event presentation for renderer-neutral controller hosts.
use anyhow::Result;
use crystal_runtime::RuntimeOverworldFrame;

use super::{
    BevyRuntimeShell, VisibleEggHatchPhase, advance_visible_egg_hatch,
    advance_visible_poison_flash, present_visible_step_event,
};

pub(super) fn present(shell: &mut BevyRuntimeShell, frame: &RuntimeOverworldFrame) -> Result<()> {
    // Match the graphical update's higher-priority arrival/battle owners.
    if frame.coord_event.is_some()
        || frame.trainer_sight.is_some()
        || frame.warp.is_some()
        || frame.connection.is_some()
        || frame.wild_battle.is_some()
        || frame.phone_call.is_some()
        || frame.interaction.is_some()
    {
        return Ok(());
    }
    let Some(event) = frame.step_events.as_ref().filter(|event| {
        event.repel_expired.is_some() || event.egg_hatched || event.poison_result.is_some()
    }) else {
        return Ok(());
    };
    let blackout = event
        .poison_result
        .as_ref()
        .is_some_and(|poison| !poison.fainted_names.is_empty())
        && !shell
            .shell
            .session()
            .state()
            .storage
            .party
            .pokemon
            .iter()
            .flatten()
            .any(|pokemon| !pokemon.is_egg && pokemon.species.id != "EGG" && pokemon.hp > 0);
    present_visible_step_event(shell, event)?;
    if blackout {
        shell.pending_poison_blackout = true;
    }
    Ok(())
}

/// Advances the existing finite presentation clocks without acknowledging text.
pub(super) fn advance(shell: &mut BevyRuntimeShell) -> Result<bool> {
    if advance_visible_poison_flash(shell, 1)? {
        return Ok(true);
    }
    let hatch_owns_frame = shell.visible_egg_hatch.as_ref().is_some_and(|hatch| {
        matches!(
            hatch.phase,
            VisibleEggHatchPhase::EggHold
                | VisibleEggHatchPhase::Wobble
                | VisibleEggHatchPhase::Shell
        ) || (hatch.phase == VisibleEggHatchPhase::Reveal
            && shell.visible_frontpic_animation.is_none())
    });
    if hatch_owns_frame {
        advance_visible_egg_hatch(shell)?;
    }
    Ok(hatch_owns_frame)
}
