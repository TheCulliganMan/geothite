//! Shared native Fly clock for graphical and renderer-neutral hosts.
use anyhow::Result;

use super::{
    BevyRuntimeShell, VisibleFlyAnimation, VisibleFlyAnimationPhase, mark_runtime_snapshot_dirty,
    queue_visible_sound_effect, settle_visible_overworld_travel,
};

/// Returns whether departure or arrival owns this frame's presentation.
pub(super) fn advance(shell: &mut BevyRuntimeShell, ticks: u32) -> Result<bool> {
    if shell.visible_fly_animation.is_none() {
        return Ok(false);
    }
    for _ in 0..ticks {
        let Some(animation) = shell.visible_fly_animation else {
            break;
        };
        let counter = match animation.phase {
            VisibleFlyAnimationPhase::From => 128_u8.saturating_sub(animation.frame),
            VisibleFlyAnimationPhase::To => 64_u8.saturating_sub(animation.frame),
        };
        if animation.frame > 0 && counter >= 0x40 && counter & 7 == 0 {
            let BevyRuntimeShell {
                shell,
                pending_audio,
                last_audio_events,
                ..
            } = &mut *shell;
            queue_visible_sound_effect(
                shell.runtime().audio(),
                pending_audio,
                last_audio_events,
                "SFX_FLY",
            )?;
        }
        match animation.phase {
            VisibleFlyAnimationPhase::From => {
                if animation.frame >= 128 {
                    shell.field_notice_scene = None;
                    settle_visible_overworld_travel(shell)?;
                    shell.visible_fly_animation = Some(VisibleFlyAnimation {
                        phase: VisibleFlyAnimationPhase::To,
                        frame: 0,
                        actor_party_index: animation.actor_party_index,
                    });
                    let BevyRuntimeShell {
                        shell,
                        pending_audio,
                        last_audio_events,
                        ..
                    } = &mut *shell;
                    queue_visible_sound_effect(
                        shell.runtime().audio(),
                        pending_audio,
                        last_audio_events,
                        "SFX_FLY",
                    )?;
                } else if let Some(animation) = shell.visible_fly_animation.as_mut() {
                    animation.frame = animation.frame.saturating_add(1);
                }
            }
            VisibleFlyAnimationPhase::To => {
                if animation.frame >= 64 {
                    shell.visible_fly_animation = None;
                } else if let Some(animation) = shell.visible_fly_animation.as_mut() {
                    animation.frame = animation.frame.saturating_add(1);
                }
            }
        }
        mark_runtime_snapshot_dirty(shell);
    }
    Ok(true)
}
