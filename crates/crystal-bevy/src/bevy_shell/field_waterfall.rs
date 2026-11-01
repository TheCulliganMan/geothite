//! Shared native Waterfall clock for graphical and renderer-neutral hosts.
use std::sync::Arc;

use anyhow::{Context, Result, ensure};
use crystal_core::world::map::TilePosition;

use super::{
    BevyRuntimeShell, WALK_FRAME_HOLD_TICKS, execute_visible_pending_waterfall_step,
    mark_runtime_snapshot_dirty,
};

/// Returns whether the acknowledged climb owns this frame's presentation.
pub(super) fn advance(shell: &mut BevyRuntimeShell, ticks: u32) -> Result<bool> {
    if shell.field_notice.is_some() || shell.visible_waterfall_animation.is_none() {
        return Ok(false);
    }
    for _ in 0..ticks {
        let Some(animation) = shell.visible_waterfall_animation else {
            break;
        };
        let total_frames = animation
            .steps
            .checked_mul(4)
            .context("WATERFALL visual duration overflows its native frame counter")?;
        if animation.frame >= total_frames {
            shell.visible_waterfall_animation = None;
            shell.field_notice_scene = None;
            shell.player_walk_from = None;
            shell.player_walk_frame_ticks = 0;
            shell.player_walk_total_ticks = WALK_FRAME_HOLD_TICKS;
            shell.player_walk_stride = false;
            shell.player_walk_mirror_stride = false;
        } else {
            let step_index = animation.frame / 4;
            let phase = (animation.frame % 4) as u8;
            if phase == 0 {
                execute_visible_pending_waterfall_step(shell, step_index, animation.steps)?;
            }
            let offset = i16::try_from(step_index)
                .context("WATERFALL step exceeds runtime tile coordinates")?;
            let from_y = animation
                .from_tile
                .y
                .checked_sub(offset)
                .context("WATERFALL visual origin underflows")?;
            let to_y = from_y
                .checked_sub(1)
                .context("WATERFALL visual destination underflows")?;
            let from = TilePosition {
                x: animation.from_tile.x,
                y: from_y,
            };
            let to = TilePosition { x: from.x, y: to_y };
            ensure!(
                step_index + 1 != animation.steps || to == animation.to_tile,
                "WATERFALL visual path ended at {to:?} instead of authoritative {:?}",
                animation.to_tile
            );
            if let Some(scene) = shell.field_notice_scene.as_mut() {
                Arc::make_mut(scene).overworld.tile = to;
            }
            shell.player_walk_from = Some(from);
            shell.player_walk_total_ticks = WALK_FRAME_HOLD_TICKS;
            shell.player_walk_frame_ticks =
                WALK_FRAME_HOLD_TICKS.saturating_sub(phase.saturating_mul(2));
            shell.player_walk_stride = step_index.is_multiple_of(2);
            shell.player_walk_mirror_stride = step_index % 4 >= 2;
            if let Some(animation) = shell.visible_waterfall_animation.as_mut() {
                animation.frame = animation.frame.saturating_add(1);
            }
        }
        mark_runtime_snapshot_dirty(shell);
    }
    Ok(true)
}
