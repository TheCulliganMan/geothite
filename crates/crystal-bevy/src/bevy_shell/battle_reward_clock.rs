// Shared by the graphical update clock and renderer-neutral controller settling.
// This advances presentation only; reward state was committed by the battle engine.
fn advance_visible_battle_reward_frame(runtime_shell: &mut BevyRuntimeShell) {
    let mut exp_segment_finished = false;
    let mut exp_animation_finished = false;
    let mut exp_pixels_changed = false;
    if let Some(tween) = runtime_shell.battle_exp_tween.as_mut()
        && tween.started
    {
        if tween.frames_until_step > 0 {
            tween.frames_until_step -= 1;
        } else if tween.pixels < tween.target_pixels {
            tween.pixels += 1;
            tween.steps_in_segment += 1;
            tween.frames_until_step = if tween.steps_in_segment <= 2 {
                2
            } else if tween.steps_in_segment <= 4 {
                1
            } else {
                0
            };
            exp_pixels_changed = true;
            if tween.pixels == tween.target_pixels {
                exp_segment_finished = !tween.remaining_targets.is_empty();
                exp_animation_finished = tween.remaining_targets.is_empty();
                if exp_segment_finished {
                    tween.level = tween.level.saturating_add(1).min(100);
                }
                tween.started = false;
            }
        } else {
            exp_segment_finished = !tween.remaining_targets.is_empty();
            exp_animation_finished = tween.remaining_targets.is_empty();
            if exp_segment_finished {
                tween.level = tween.level.saturating_add(1).min(100);
            }
            tween.started = false;
        }
    }
    if exp_pixels_changed {
        mark_runtime_snapshot_dirty(runtime_shell);
    }
    if exp_segment_finished {
        if let Err(error) =
            queue_visible_shell_sound_effect(runtime_shell, "SFX_HIT_END_OF_EXP_BAR")
        {
            record_visible_runtime_error(runtime_shell, &error);
        }
        if runtime_shell
            .battle_fanfare_messages
            .front()
            .is_some_and(|fanfare| runtime_shell.battle_messages.front() == Some(fanfare))
        {
            runtime_shell.battle_fanfare_messages.pop_front();
            if let Err(error) =
                queue_visible_shell_sound_effect(runtime_shell, "SFX_DEX_FANFARE_50_79")
            {
                record_visible_runtime_error(runtime_shell, &error);
            }
        }
    }
    if exp_animation_finished {
        runtime_shell.battle_exp_tween = runtime_shell.pending_battle_exp_tweens.pop_front();
        if let Some(stats) = runtime_shell.battle_level_stats.front_mut()
            && stats.triggered
        {
            stats.active = true;
            // This activation occurs before the per-frame countdown below.
            stats.frames_before_input = 31;
            mark_runtime_snapshot_dirty(runtime_shell);
        }
        if let Err(error) = finish_visible_empty_battle_reward_presentation(runtime_shell) {
            record_visible_runtime_error(runtime_shell, &error);
        }
        if runtime_shell
            .battle_fanfare_messages
            .front()
            .is_some_and(|fanfare| runtime_shell.battle_messages.front() == Some(fanfare))
        {
            runtime_shell.battle_fanfare_messages.pop_front();
            if let Err(error) =
                queue_visible_shell_sound_effect(runtime_shell, "SFX_DEX_FANFARE_50_79")
            {
                record_visible_runtime_error(runtime_shell, &error);
            }
        }
    }
    if let Some(stats) = runtime_shell.battle_level_stats.front_mut()
        && stats.active
        && stats.frames_before_input > 0
    {
        stats.frames_before_input -= 1;
    }
}
