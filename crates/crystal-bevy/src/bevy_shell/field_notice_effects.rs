//! Shared native field-effect clocks for graphical and renderer-neutral hosts.
use anyhow::Result;

use super::{
    BevyRuntimeShell, VisibleWaitPlaySfxCompletion, mark_runtime_snapshot_dirty,
    play_pending_field_notice_sound, prepare_visible_battle_entry,
    settle_visible_battle_after_action, visible_field_dialogue_is_fully_revealed,
};

pub(super) fn advance(
    runtime_shell: &mut BevyRuntimeShell,
    elapsed_input_ticks: u32,
) -> Result<bool> {
    if runtime_shell.field_notice.is_some() {
        return Ok(false);
    }
    let Some(frames) = runtime_shell.pending_field_notice_effect_frames else {
        return Ok(false);
    };
    let frames = frames.saturating_sub(elapsed_input_ticks.min(u32::from(u8::MAX)) as u8);
    runtime_shell.pending_field_notice_effect_frames = Some(frames);
    let mut field_object_effect_advanced = false;
    if let Some(cut) = runtime_shell.visible_cut_animation.as_mut() {
        cut.frame = 34_u8.saturating_sub(frames);
        field_object_effect_advanced = true;
    }
    if let Some(headbutt) = runtime_shell.visible_headbutt_animation.as_mut() {
        headbutt.frame = 32_u8.saturating_sub(frames);
        field_object_effect_advanced = true;
    }
    if let Some(flash) = runtime_shell.visible_flash_animation.as_mut() {
        let previous_frame = flash.frame;
        flash.frame = 16_u8.saturating_sub(frames);
        field_object_effect_advanced = true;
        if previous_frame < 8 && flash.frame >= 8 {
            runtime_shell.field_notice_scene = None;
        }
    }
    if field_object_effect_advanced {
        mark_runtime_snapshot_dirty(runtime_shell);
    }
    if frames == 0 {
        runtime_shell.pending_field_notice_effect_frames = None;
        runtime_shell.field_notice_scene = None;
        runtime_shell.visible_earthquake = None;
        runtime_shell.visible_cut_animation = None;
        runtime_shell.pending_whirlpool_sound_wait = false;
        runtime_shell.visible_headbutt_animation = None;
        runtime_shell.visible_flash_animation = None;
        runtime_shell.pending_surf_start_from = None;
        if std::mem::take(&mut runtime_shell.pending_field_battle_entry) {
            prepare_visible_battle_entry(runtime_shell)?;
            settle_visible_battle_after_action(runtime_shell)?;
        } else if let Some(next) = runtime_shell.field_notice_queue.pop_front() {
            runtime_shell.field_notice = Some(next);
        }
        mark_runtime_snapshot_dirty(runtime_shell);
    }
    Ok(true)
}

pub(super) fn start_flash_sound(runtime_shell: &mut BevyRuntimeShell) -> Result<bool> {
    if runtime_shell.visible_flash_animation.is_none() || runtime_shell.field_notice.is_none() {
        return Ok(false);
    }
    let snapshot = runtime_shell.shell.presentation_snapshot()?;
    if !visible_field_dialogue_is_fully_revealed(runtime_shell, &snapshot) {
        return Ok(false);
    }
    runtime_shell.field_notice = None;
    play_pending_field_notice_sound(runtime_shell)?;
    runtime_shell.visible_wait_sfx_boundary = true;
    runtime_shell.wait_play_sfx_completion = Some(VisibleWaitPlaySfxCompletion::FlashFieldMove);
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(true)
}
