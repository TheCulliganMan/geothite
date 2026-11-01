// Shared graphical and renderer-neutral reward presentation clock.
fn advance_visible_battle_reward_frame(runtime_shell: &mut BevyRuntimeShell) -> Result<bool> {
    let mut segment_finished = false;
    let mut animation_finished = false;
    let mut consumed = false;
    if let Some(tween) = runtime_shell.battle_exp_tween.as_mut()
        && tween.started
    {
        consumed = true;
        if tween.frames_until_step > 0 {
            tween.frames_until_step -= 1;
        } else {
            if tween.pixels < tween.target_pixels {
                tween.pixels += 1;
                tween.steps_in_segment += 1;
                tween.frames_until_step = if tween.steps_in_segment <= 2 {
                    2
                } else if tween.steps_in_segment <= 4 {
                    1
                } else {
                    0
                };
            }
            if tween.pixels == tween.target_pixels {
                segment_finished = !tween.remaining_targets.is_empty();
                animation_finished = !segment_finished;
                if segment_finished {
                    tween.level = tween.level.saturating_add(1).min(100);
                }
                tween.started = false;
            }
        }
        mark_runtime_snapshot_dirty(runtime_shell);
    }
    if segment_finished {
        queue_visible_shell_sound_effect(runtime_shell, "SFX_HIT_END_OF_EXP_BAR")?;
    }
    if animation_finished {
        runtime_shell.battle_exp_tween = runtime_shell.pending_battle_exp_tweens.pop_front();
        if let Some(stats) = runtime_shell.battle_level_stats.front_mut()
            && stats.triggered
        {
            stats.active = true;
            stats.frames_before_input = 31;
            mark_runtime_snapshot_dirty(runtime_shell);
        }
        finish_visible_empty_battle_reward_presentation(runtime_shell)?;
    }
    if (segment_finished || animation_finished)
        && runtime_shell.battle_fanfare_messages.front()
            .is_some_and(|fanfare| runtime_shell.battle_messages.front() == Some(fanfare))
    {
        runtime_shell.battle_fanfare_messages.pop_front();
        queue_visible_shell_sound_effect(runtime_shell, "SFX_DEX_FANFARE_50_79")?;
    }
    if let Some(stats) = runtime_shell.battle_level_stats.front_mut()
        && stats.active
        && stats.frames_before_input > 0
    {
        stats.frames_before_input -= 1;
        consumed = true;
    }
    Ok(consumed)
}
