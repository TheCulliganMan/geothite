fn close_visible_noninteractive_runtime_surfaces_until_idle(
    runtime_shell: &mut BevyRuntimeShell,
) -> Result<()> {
    const MAX_NONINTERACTIVE_SURFACE_CLOSES: usize = 64;
    for _ in 0..MAX_NONINTERACTIVE_SURFACE_CLOSES {
        if !close_visible_noninteractive_runtime_surface(runtime_shell)? {
            return Ok(());
        }
    }
    anyhow::bail!(
        "visible shell exceeded noninteractive runtime surface close limit {MAX_NONINTERACTIVE_SURFACE_CLOSES}"
    )
}

fn finish_visible_empty_battle_reward_presentation(
    runtime_shell: &mut BevyRuntimeShell,
) -> Result<bool> {
    if !runtime_shell.battle_messages.is_empty()
        || runtime_shell.battle_exp_tween.is_some()
        || !runtime_shell.pending_battle_exp_tweens.is_empty()
        || !runtime_shell.battle_level_stats.is_empty()
        || runtime_shell.battle_message_scene.is_none()
    {
        return Ok(false);
    }
    let snapshot = runtime_shell.shell.presentation_snapshot()?;
    if snapshot.battle.is_none() {
        reset_visible_battle_exit_state(runtime_shell);
        runtime_shell.battle_message_scene = None;
        runtime_shell.battle_hp_tween = None;
        runtime_shell.battle_fanfare_messages.clear();
        runtime_shell.battle_evolution_cries.clear();
        runtime_shell.battle_evolution_cancellations.clear();
        runtime_shell.battle_sounds_after_messages.clear();
        reset_visible_music_state(runtime_shell);
        queue_visible_current_music(runtime_shell)?;
        if runtime_shell.pending_plain_battle_map_reload {
            begin_visible_plain_battle_map_reload(runtime_shell)?;
        } else {
            continue_visible_script_after_prompt(runtime_shell)?;
        }
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(true);
    }
    let resume_trainer_settlement = runtime_shell.battle_shift_prompt_cursor.is_none()
        && runtime_shell.battle_switch_cursor.is_none()
        && snapshot.battle.as_ref().is_some_and(|battle| {
            matches!(&battle.kind, crate::RuntimeBattleKind::Trainer { .. })
                && battle.enemy_pokemon.hp == 0
                && !battle.enemy_spikes_zero_hp_unchecked
        });
    if resume_trainer_settlement {
        runtime_shell.battle_message_scene = None;
        settle_visible_battle_after_action(runtime_shell)?;
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(true);
    }
    Ok(false)
}

fn restore_visible_cancelled_evolution(
    runtime_shell: &mut BevyRuntimeShell,
    cancellation: &mut VisibleEvolutionCancellation,
) -> Result<String> {
    let source_name = cancellation
        .report
        .cancel_snapshot
        .as_ref()
        .map(|pokemon| pokemon.nickname.clone())
        .context("cancelable evolution is missing its source Pokemon snapshot")?;
    let pending_move_names = cancellation
        .report
        .pending_move_learns
        .iter()
        .map(|learned| learned.name.clone())
        .collect::<HashSet<_>>();
    {
        let state = runtime_shell.shell.session_mut().state_mut();
        let pokemon = state
            .storage
            .party
            .pokemon
            .get_mut(cancellation.party_index)
            .and_then(Option::as_mut)
            .with_context(|| {
                format!(
                    "cancel evolution party index {} is empty",
                    cancellation.party_index
                )
            })?;
        crate::core::systems::evolution::cancel_evolution(pokemon, &mut cancellation.report)
            .context("cancel visible battle evolution")?;

        if state.pending_move_learn.as_ref().is_some_and(|pending| {
            pending.party_index == cancellation.party_index
                && pending_move_names.contains(&pending.learned_move.name)
        }) {
            state.pending_move_learn = None;
        }
        state.pending_move_learn_queue.retain(|pending| {
            pending.party_index != cancellation.party_index
                || !pending_move_names.contains(&pending.learned_move.name)
        });
        crate::core::systems::battle_rewards::promote_next_pending_move_learn(state);
        state.sync_party_from_storage();
        crate::core::systems::battle_rewards::sync_active_combat_player_party_from_storage(state);
    }
    Ok(source_name)
}

fn record_visible_completed_evolution(
    runtime_shell: &mut BevyRuntimeShell,
    party_index: usize,
) -> Result<()> {
    let state = runtime_shell.shell.session_mut().state_mut();
    let evolved = state
        .storage
        .party
        .pokemon
        .get(party_index)
        .and_then(Option::as_ref)
        .with_context(|| format!("completed evolution party index {party_index} is empty"))?
        .clone();
    state.pokedex.record_caught_pokemon(&evolved);
    Ok(())
}

fn visible_evolution_moves_resolved(
    runtime_shell: &BevyRuntimeShell,
    cancellation: &VisibleEvolutionCancellation,
) -> bool {
    let pending_names = cancellation
        .report
        .pending_move_learns
        .iter()
        .map(|learned| learned.name.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let state = &runtime_shell.shell.session().state();
    !state
        .pending_move_learn
        .iter()
        .chain(state.pending_move_learn_queue.iter())
        .any(|pending| {
            pending.party_index == cancellation.party_index
                && pending_names.contains(pending.learned_move.name.as_str())
        })
}

fn complete_visible_accepted_evolution_after_battle_message(
    runtime_shell: &mut BevyRuntimeShell,
    dismissed_message: Option<&str>,
) -> Result<()> {
    let battle_party_index = runtime_shell
        .battle_evolution_cancellations
        .front()
        .filter(|cancellation| {
            cancellation.accepted
                && visible_evolution_moves_resolved(runtime_shell, cancellation)
                && if cancellation.report.pending_move_learns.is_empty() {
                    dismissed_message == Some(cancellation.evolved_message.as_str())
                } else {
                    cancellation
                        .pending_move_messages
                        .last()
                        .is_some_and(|message| dismissed_message == Some(message.as_str()))
                }
        })
        .map(|cancellation| cancellation.party_index);
    if let Some(party_index) = battle_party_index {
        runtime_shell.battle_evolution_cancellations.pop_front();
        record_visible_completed_evolution(runtime_shell, party_index)?;
    }
    let field_party_index = runtime_shell
        .field_evolution_cancellation
        .as_ref()
        .filter(|cancellation| {
            cancellation.accepted
                && !cancellation.report.pending_move_learns.is_empty()
                && visible_evolution_moves_resolved(runtime_shell, cancellation)
                && cancellation
                    .pending_move_messages
                    .last()
                    .is_some_and(|message| dismissed_message == Some(message.as_str()))
        })
        .map(|cancellation| cancellation.party_index);
    if let Some(party_index) = field_party_index {
        runtime_shell.field_evolution_cancellation = None;
        record_visible_completed_evolution(runtime_shell, party_index)?;
    }
    Ok(())
}

fn complete_visible_accepted_evolution_after_special_boundary(
    runtime_shell: &mut BevyRuntimeShell,
    boundary_label: &str,
) -> Result<()> {
    if boundary_label != "LearnedMoveText" {
        return Ok(());
    }
    let battle_party_index = runtime_shell
        .battle_evolution_cancellations
        .front()
        .filter(|cancellation| {
            cancellation.accepted
                && !cancellation.report.pending_move_learns.is_empty()
                && visible_evolution_moves_resolved(runtime_shell, cancellation)
        })
        .map(|cancellation| cancellation.party_index);
    if let Some(party_index) = battle_party_index {
        runtime_shell.battle_evolution_cancellations.pop_front();
        record_visible_completed_evolution(runtime_shell, party_index)?;
    }
    let field_party_index = runtime_shell
        .field_evolution_cancellation
        .as_ref()
        .filter(|cancellation| {
            cancellation.accepted
                && !cancellation.report.pending_move_learns.is_empty()
                && visible_evolution_moves_resolved(runtime_shell, cancellation)
        })
        .map(|cancellation| cancellation.party_index);
    if let Some(party_index) = field_party_index {
        runtime_shell.field_evolution_cancellation = None;
        record_visible_completed_evolution(runtime_shell, party_index)?;
    }
    Ok(())
}

fn cancel_visible_battle_evolution(runtime_shell: &mut BevyRuntimeShell) -> Result<bool> {
    let Some(cancellation) = runtime_shell.battle_evolution_cancellations.front() else {
        return Ok(false);
    };
    let Some(message) = runtime_shell.battle_messages.front() else {
        return Ok(false);
    };
    if cancellation.accepted
        || message != &cancellation.trigger_message
        || !visible_battle_message_is_complete(runtime_shell, message)
    {
        return Ok(false);
    }

    let mut cancellation = runtime_shell
        .battle_evolution_cancellations
        .pop_front()
        .expect("checked pending evolution cancellation");
    let source_name = restore_visible_cancelled_evolution(runtime_shell, &mut cancellation)?;

    let staged_scenes_aligned =
        runtime_shell.battle_message_scenes.len() == runtime_shell.battle_messages.len();
    let stopped_scene = runtime_shell
        .battle_message_scenes
        .front()
        .cloned()
        .or_else(|| runtime_shell.battle_message_scene.clone());
    let mut removed_messages = 0usize;
    if runtime_shell.battle_messages.front() == Some(&cancellation.trigger_message) {
        runtime_shell.battle_messages.pop_front();
        removed_messages += 1;
    }
    if runtime_shell.battle_messages.front() == Some(&cancellation.evolved_message) {
        runtime_shell.battle_messages.pop_front();
        removed_messages += 1;
    }
    for pending_message in &cancellation.pending_move_messages {
        if runtime_shell.battle_messages.front() == Some(pending_message) {
            runtime_shell.battle_messages.pop_front();
            removed_messages += 1;
        }
    }
    let stopped_message = format!("Huh? {}\nstopped evolving!", source_name);
    runtime_shell
        .battle_messages
        .push_front(stopped_message.clone());
    runtime_shell.battle_text_reveal = None;
    if staged_scenes_aligned {
        for _ in 0..removed_messages {
            runtime_shell.battle_message_scenes.pop_front();
        }
        if let Some(scene) = stopped_scene {
            runtime_shell.battle_message_scenes.push_front(scene);
        } else {
            runtime_shell.battle_message_scenes.clear();
        }
    } else {
        runtime_shell.battle_message_scenes.clear();
    }
    runtime_shell
        .battle_evolution_cries
        .retain(|(_, trigger)| trigger != &cancellation.trigger_message);
    runtime_shell
        .battle_sounds_after_messages
        .retain(|(_, trigger)| trigger != &cancellation.trigger_message);
    runtime_shell.last_audio_events.push(format!(
        "battle evolution cancelled party_index={} species={}",
        cancellation.party_index,
        cancellation
            .report
            .cancel_snapshot
            .as_ref()
            .map(|pokemon| pokemon.species.id.as_str())
            .unwrap_or("restored")
    ));
    set_shell_action_status(runtime_shell, "STOPPED EVOLVING");
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(true)
}

fn cancel_visible_field_evolution(runtime_shell: &mut BevyRuntimeShell) -> Result<bool> {
    let Some(cancellation) = runtime_shell.field_evolution_cancellation.as_ref() else {
        return Ok(false);
    };
    if cancellation.accepted
        || runtime_shell.field_notice.as_deref() != Some(cancellation.trigger_message.as_str())
    {
        return Ok(false);
    }
    let snapshot = runtime_shell.shell.presentation_snapshot()?;
    if !visible_field_dialogue_is_fully_revealed(runtime_shell, &snapshot) {
        return Ok(false);
    }

    let mut cancellation = runtime_shell
        .field_evolution_cancellation
        .take()
        .expect("checked field evolution cancellation");
    let source_name = restore_visible_cancelled_evolution(runtime_shell, &mut cancellation)?;
    if runtime_shell.field_notice_queue.front() == Some(&cancellation.evolved_message) {
        runtime_shell.field_notice_queue.pop_front();
    }
    for pending_message in &cancellation.pending_move_messages {
        if runtime_shell.field_notice_queue.front() == Some(pending_message) {
            runtime_shell.field_notice_queue.pop_front();
        }
    }
    runtime_shell.field_notice = Some(format!("Huh? {}\nstopped evolving!", source_name));
    runtime_shell.field_text_reveal = None;
    runtime_shell.pending_field_notice_cry = None;
    runtime_shell.last_audio_events.push(format!(
        "field evolution cancelled party_index={}",
        cancellation.party_index
    ));
    set_shell_action_status(runtime_shell, "STOPPED EVOLVING");
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(true)
}

fn visible_pc_printer_status(runtime_shell: &BevyRuntimeShell) -> bool {
    runtime_shell.pc_notice.as_deref().is_some_and(|notice| {
        notice.starts_with("Printer Error ") && notice.contains("Game Boy\nPrinter Manual.")
    })
}

fn press_visible_a_button(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    if runtime_shell.visible_battle_sliding_intro.is_some() {
        return Ok(());
    }
    if runtime_shell.mailbox_confirmation_response.is_some() {
        return Ok(());
    }
    if runtime_shell.pokegear_exit.is_some() { return Ok(()); }
    if runtime_shell.pokegear_menu_open && runtime_shell.pokegear_page == PokegearPage::Radio {
        // PokegearRadio_Joypad ignores A. PlayRadio instead stops on A/B
        // after its initial DelayFrames, independent of the broadcast line.
        return if runtime_shell.pokegear_map_radio_delay == Some(0) {
            close_visible_map_radio(runtime_shell)
        } else { Ok(()) };
    }
    if runtime_shell.pokegear_phone_call.as_ref().is_some_and(|call| {
        matches!(call.phase, VisiblePokegearPhoneCallPhase::HangingUp { .. })
    }) {
        return Ok(());
    }

    if runtime_shell
        .pokegear_phone_call
        .as_ref()
        .is_some_and(|call| call.phase == VisiblePokegearPhoneCallPhase::NoServicePrompt)
    {
        return dismiss_visible_pokegear_no_service_prompt(runtime_shell);
    }
    if runtime_shell
        .pokegear_phone_call
        .as_ref()
        .is_some_and(|call| call.phase == VisiblePokegearPhoneCallPhase::AwaitHangup)
    {
        return finish_visible_pokegear_phone_call(runtime_shell);
    }
    if runtime_shell
        .special_boundary
        .as_ref()
        .is_some_and(|boundary| boundary.label == "PrinterError2")
    {
        record_visible_runtime_action(runtime_shell, "printer:error:a_ignored")?;
        return Ok(());
    }
    if visible_pc_printer_status(runtime_shell) {
        record_visible_runtime_action(runtime_shell, "printer:error:a_ignored")?;
        return Ok(());
    }
    if runtime_shell.bill_pc_move_save.is_some()
        || runtime_shell.pc_release_sequence.is_some()
        || runtime_shell.pc_transfer_sequence.is_some()
        || runtime_shell.pc_item_move_sequence.is_some()
    {
        return Ok(());
    }
    if runtime_shell.pending_mail_read.is_some() {
        return close_visible_mail_read(runtime_shell);
    }
    if runtime_shell.pending_name_choice.is_some() {
        return confirm_visible_name_choice(runtime_shell);
    }
    if runtime_shell.pending_scene_script.is_some() {
        return take_visible_pending_scene_script(runtime_shell);
    }
    if runtime_shell.visible_diploma.is_some() {
        return close_visible_diploma(runtime_shell);
    }
    if runtime_shell.visible_unown_words.is_some() {
        return close_visible_unown_words(runtime_shell);
    }
    if runtime_shell.visible_heal_machine.is_some() || runtime_shell.visible_magnet_train.is_some()
    {
        return Ok(());
    }
    if runtime_shell
        .battle_exp_tween
        .as_ref()
        .is_some_and(|tween| tween.started)
    {
        return Ok(());
    }
    if let Some(stats) = runtime_shell.battle_level_stats.front()
        && stats.active
    {
        if stats.frames_before_input == 0 {
            runtime_shell.battle_level_stats.pop_front();
            mark_runtime_snapshot_dirty(runtime_shell);
            finish_visible_empty_battle_reward_presentation(runtime_shell)?;
        }
        return Ok(());
    }
    if runtime_shell.visible_card_flip.is_some() {
        return flip_visible_card(runtime_shell);
    }
    if runtime_shell.visible_slot_machine.is_some() {
        return spin_visible_slot_machine(runtime_shell);
    }
    if runtime_shell.visible_unown_puzzle.is_some() {
        return use_visible_unown_puzzle_cell(runtime_shell);
    }
    if runtime_shell.visible_unown_printer.is_some() {
        return print_visible_unown_stamp(runtime_shell);
    }
    if runtime_shell.visible_mom_bank.is_some() {
        return confirm_visible_mom_bank(runtime_shell);
    }
    if runtime_shell.pending_day_of_week.is_some() {
        return confirm_visible_day_of_week(runtime_shell);
    }
    if runtime_shell.kurt_apricorn_cursor.is_some() {
        return resolve_visible_kurt_apricorn_selection(runtime_shell, false);
    }
    if runtime_shell.visible_buena_password.is_some() {
        return resolve_visible_buena_password_selection(runtime_shell);
    }
    if runtime_shell.visible_battle_tower_challenge_menu.is_some() {
        return resolve_visible_battle_tower_challenge_menu(runtime_shell, false);
    }
    if let Some(menu) = runtime_shell.visible_battle_tower_room_menu.as_ref() {
        return match menu.phase.clone() {
            VisibleBattleTowerRoomMenuPhase::PickLevel => {
                resolve_visible_battle_tower_room_level(runtime_shell)
            }
            VisibleBattleTowerRoomMenuPhase::ConfirmCancel { yes_no_index } => {
                resolve_visible_battle_tower_room_cancel(runtime_shell, yes_no_index == 0)
            }
            VisibleBattleTowerRoomMenuPhase::Rejection { .. } => {
                runtime_shell
                    .visible_battle_tower_room_menu
                    .as_mut()
                    .context("Battle Tower room menu disappeared")?
                    .phase = VisibleBattleTowerRoomMenuPhase::PickLevel;
                set_shell_action_status(runtime_shell, "BATTLE ROOM LEVEL");
                mark_runtime_snapshot_dirty(runtime_shell);
                Ok(())
            }
        };
    }
    if runtime_shell.buena_prize_cursor.is_some()
        && runtime_shell.pc_confirmation.is_none()
        && runtime_shell.pc_notice.is_none()
    {
        return resolve_visible_buena_prize_selection(runtime_shell, false);
    }
    if !runtime_shell.battle_messages.is_empty() {
        if runtime_shell
            .battle_hp_tween
            .as_ref()
            .is_some_and(visible_battle_hp_tween_active)
        {
            return Ok(());
        }
        if runtime_shell
            .battle_exp_tween
            .as_ref()
            .is_some_and(|tween| tween.started)
        {
            return Ok(());
        }
        let message = runtime_shell
            .battle_messages
            .front()
            .expect("checked nonempty battle message queue");
        if !visible_battle_message_is_complete(runtime_shell, message) {
            return Ok(());
        }
        if visible_battle_message_has_more_pages(runtime_shell, message) {
            let message = message.clone();
            if advance_visible_battle_message_page(runtime_shell, &message) {
                queue_visible_shell_sound_effect(runtime_shell, "SFX_READ_TEXT_2")?;
                mark_runtime_snapshot_dirty(runtime_shell);
            }
            return Ok(());
        }
        let staged_scenes_aligned =
            runtime_shell.battle_message_scenes.len() == runtime_shell.battle_messages.len();
        runtime_shell.battle_retained_text = visible_battle_message_lines(runtime_shell, runtime_shell.battle_messages.front().unwrap());
        let dismissed_battle_message = runtime_shell.battle_messages.pop_front();
        runtime_shell.battle_text_reveal = None;
        if let Some(animation) = runtime_shell.visible_capture_animation.as_mut()
            && animation.complete
            && animation.caught
            && !animation.sprites_cleared
            && dismissed_battle_message
                .as_deref()
                .is_some_and(|message| message.starts_with("Gotcha! "))
        {
            // PokeBallEffect keeps the caught ball through Gotcha, then calls
            // ClearSprites before either the Pokedex or nickname flow. Core
            // capture mutation is still deferred, so retain the presentation
            // state to keep its authoritative enemy hidden after this clear.
            animation.sprites_cleared = true;
        }
        if dismissed_battle_message.is_some() {
            queue_visible_shell_sound_effect(runtime_shell, "SFX_READ_TEXT_2")?;
            advance_visible_victory_music_boundary(runtime_shell)?;
        }
        if runtime_shell
            .battle_evolution_cries
            .front()
            .is_some_and(|(_, trigger)| {
                dismissed_battle_message.as_deref() == Some(trigger.as_str())
            })
        {
            let (species_id, _) = runtime_shell.battle_evolution_cries.pop_front().unwrap();
            queue_visible_pokemon_cry(runtime_shell, &species_id, "battle_evolution")?;
        }
        if runtime_shell
            .battle_sounds_after_messages
            .front()
            .is_some_and(|(_, trigger)| {
                dismissed_battle_message.as_deref() == Some(trigger.as_str())
            })
        {
            let (sound_id, _) = runtime_shell
                .battle_sounds_after_messages
                .pop_front()
                .unwrap();
            queue_visible_shell_sound_effect(runtime_shell, &sound_id)?;
        }
        if let Some(cancellation) = runtime_shell.battle_evolution_cancellations.front_mut()
            && dismissed_battle_message.as_deref()
                == Some(cancellation.trigger_message.as_str())
        {
            cancellation.accepted = true;
        }
        complete_visible_accepted_evolution_after_battle_message(
            runtime_shell,
            dismissed_battle_message.as_deref(),
        )?;
        let starts_exp_animation = runtime_shell
            .battle_exp_tween
            .as_ref()
            .is_some_and(|tween| {
                !tween.started
                    && (dismissed_battle_message.as_deref() == Some(tween.trigger_message.as_str())
                        || (tween.pixels == tween.target_pixels
                            && !tween.remaining_targets.is_empty()
                            && dismissed_battle_message
                                .as_deref()
                                .is_some_and(|message| message.contains(" grew to\nlevel "))))
            });
        if starts_exp_animation {
            let tween = runtime_shell.battle_exp_tween.as_mut().unwrap();
            if tween.pixels == tween.target_pixels && !tween.remaining_targets.is_empty() {
                tween.pixels = 0;
                tween.target_pixels = tween
                    .remaining_targets
                    .pop_front()
                    .context("multi-level EXP continuation has no next bar target")?;
            }
            tween.steps_in_segment = 0;
            tween.frames_until_step = 9;
            tween.started = true;
            queue_visible_shell_sound_effect(runtime_shell, "SFX_EXP_BAR")?;
        }
        if let Some(stats) = runtime_shell.battle_level_stats.front_mut()
            && dismissed_battle_message.as_deref() == Some(stats.trigger_message.as_str())
        {
            stats.triggered = true;
            stats.active = !starts_exp_animation;
            if stats.active {
                stats.frames_before_input = 30;
            }
        }
        if !starts_exp_animation
            && runtime_shell
                .battle_fanfare_messages
                .front()
                .is_some_and(|fanfare| runtime_shell.battle_messages.front() == Some(fanfare))
        {
            if runtime_shell
                .battle_level_stats
                .front()
                .is_some_and(|stats| {
                    !stats.triggered
                        && runtime_shell.battle_messages.front() == Some(&stats.trigger_message)
                })
            {
                queue_visible_shell_sound_effect(runtime_shell, "SFX_HIT_END_OF_EXP_BAR")?;
            }
            runtime_shell.battle_fanfare_messages.pop_front();
            queue_visible_shell_sound_effect(runtime_shell, "SFX_DEX_FANFARE_50_79")?;
        }
        if starts_exp_animation
            || runtime_shell
                .battle_level_stats
                .front()
                .is_some_and(|stats| stats.active)
        {
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(());
        }
        if runtime_shell
            .battle_messages
            .front()
            .is_some_and(|message| message.contains("was newly added to\nthe POKéDEX."))
        {
            // NewDexDataText plays this command when its page is exposed,
            // after the caught page has been acknowledged.
            queue_visible_shell_sound_effect(runtime_shell, "SFX_SLOT_MACHINE_START")?;
        }
        let entry_messages_before = runtime_shell.battle_entry_messages_remaining;
        let starts_enemy_trainer_exit = entry_messages_before == 3
            && dismissed_battle_message
                .as_deref()
                .is_some_and(|message| message.ends_with("\nwants to battle!"));
        let starts_enemy_send_out = runtime_shell.battle_enemy_send_out_pending
            || dismissed_battle_message
                .as_deref()
                .is_some_and(visible_message_is_enemy_send_out);
        let starts_player_send_out = runtime_shell.battle_player_send_out_pending
            || dismissed_battle_message
                .as_deref()
                .is_some_and(visible_message_is_player_send_out);
        let starts_capture_animation = runtime_shell
            .visible_capture_animation
            .as_ref()
            .is_some_and(|animation| {
                !animation.started
                    && dismissed_battle_message.as_deref()
                        == Some(animation.trigger_message.as_str())
            });
        let starts_capture_pokedex_entry = dismissed_battle_message
            .as_deref()
            .is_some_and(|message| message.contains("was newly added to\nthe POKéDEX."));
        if starts_capture_animation {
            runtime_shell
                .visible_capture_animation
                .as_mut()
                .unwrap()
                .started = true;
            queue_visible_shell_sound_effect(runtime_shell, "SFX_THROW_BALL")?;
        }
        if starts_capture_pokedex_entry {
            let snapshot = runtime_shell.shell.presentation_snapshot()?;
            let species_id = snapshot
                .battle
                .as_ref()
                .context("capture Pokedex entry lost its battle species")?
                .enemy_pokemon
                .species
                .id
                .clone();
            let species_index = snapshot
                .pokemon
                .iter()
                .position(|species| species.species_id == species_id)
                .with_context(|| {
                    format!("captured species {species_id} is absent from the Pokedex catalog")
                })?;
            anyhow::ensure!(
                snapshot
                    .presentation
                    .pokedex_entries
                    .contains_key(&species_id),
                "captured species {species_id} has no compiled Pokedex entry"
            );
            // NewPokedexEntry owns the LCD, but capture mutation is deferred
            // until nickname choice. Retain the cleared capture state so the
            // still-live core enemy remains hidden when the nickname prompt
            // restores the battle background; the cleared state renders no
            // capture objects.
            runtime_shell.battle_message_scene = None;
            runtime_shell.pokedex_cursor = species_index;
            runtime_shell.pokedex_controls = VisiblePokedexControls { mode: runtime_shell.pokedex_controls.mode, ..Default::default() };
            runtime_shell.pokedex_menu_open = true;
            runtime_shell.pokedex_detail_open = true;
            runtime_shell.pokedex_detail_page = 0;
            runtime_shell.pokedex_scripted_entry = true;
            queue_visible_pokemon_cry(runtime_shell, &species_id, "new_pokedex_entry")?;
            set_shell_action_status(runtime_shell, format!("NEW POKEDEX ENTRY {species_id}"));
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(());
        }
        let starts_move_animation =
            runtime_shell
                .visible_move_animations
                .front()
                .is_some_and(|animation| {
                    !animation.started
                        && dismissed_battle_message.as_deref()
                            == Some(animation.trigger_message.as_str())
                });
        if starts_move_animation {
            runtime_shell
                .visible_move_animations
                .front_mut()
                .unwrap()
                .started = true;
        }
        let starts_player_trainer_exit = starts_player_send_out && entry_messages_before == 1;
        let starts_send_out_animation =
            starts_enemy_send_out || (starts_player_send_out && !starts_player_trainer_exit);
        if starts_enemy_trainer_exit || starts_player_trainer_exit {
            runtime_shell.visible_trainer_exit_animation = Some(VisibleTrainerExitAnimation {
                side: if starts_enemy_trainer_exit {
                    crate::core::battle::turn::BattleSide::Enemy
                } else {
                    crate::core::battle::turn::BattleSide::Player
                },
                frame: 0,
                send_out_after: starts_player_trainer_exit,
            });
        }
        if starts_send_out_animation {
            if starts_enemy_send_out {
                runtime_shell.visible_frontpic_animation = None;
            }
            let side = if starts_enemy_send_out {
                crate::core::battle::turn::BattleSide::Enemy
            } else {
                crate::core::battle::turn::BattleSide::Player
            };
            let shiny = visible_send_out_side_is_shiny(runtime_shell, side)?;
            runtime_shell.battle_fainted_hud[usize::from(side == crate::core::battle::turn::BattleSide::Enemy)] = false;
            runtime_shell.visible_send_out_animation = Some(VisibleSendOutAnimation {
                side,
                frame: 0,
                shiny,
            });
            queue_visible_shell_sound_effect(runtime_shell, "SFX_BALL_POOF")?;
        }
        runtime_shell.battle_enemy_send_out_pending = false;
        runtime_shell.battle_player_send_out_pending = false;
        if !starts_send_out_animation
            && runtime_shell
                .pending_battle_cries_after_messages
                .front()
                .is_some_and(|(_, _, trigger_message)| {
                    dismissed_battle_message.as_deref() == Some(trigger_message.as_str())
                })
        {
            let (species_id, reason, _) = runtime_shell
                .pending_battle_cries_after_messages
                .pop_front()
                .unwrap();
            queue_visible_pokemon_cry(runtime_shell, &species_id, &reason)?;
        }
        if !starts_move_animation {
            if staged_scenes_aligned {
                runtime_shell.battle_message_scenes.pop_front();
                if let Some(scene) = runtime_shell.battle_message_scenes.front().cloned() {
                    retarget_visible_battle_hp_tween(runtime_shell, &scene);
                    runtime_shell.battle_message_scene = Some(scene);
                    mark_runtime_snapshot_dirty(runtime_shell);
                }
            } else {
                runtime_shell.battle_message_scenes.clear();
            }
        }
        if !starts_move_animation
            && let Some((trigger_message, scene)) = runtime_shell
                .pending_battle_scenes_after_message
                .pop_front()
        {
            if dismissed_battle_message.as_deref() == Some(trigger_message.as_str()) {
                if runtime_shell.battle_message_scenes.is_empty() {
                    retarget_visible_battle_hp_tween(runtime_shell, &scene);
                    runtime_shell.battle_message_scene = Some(scene);
                }
            } else {
                runtime_shell
                    .pending_battle_scenes_after_message
                    .push_front((trigger_message, scene));
            }
        }
        runtime_shell.battle_entry_messages_remaining = runtime_shell
            .battle_entry_messages_remaining
            .saturating_sub(1);
        if starts_enemy_trainer_exit || starts_player_trainer_exit {
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(());
        }
        if starts_move_animation {
            // TypeScript's animation player blocks the battle state machine
            // here. Keep the pre-hit scene and its queued successors intact;
            // advance_visible_move_animation releases that exact boundary.
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(());
        }
        if runtime_shell.battle_messages.is_empty() {
            runtime_shell.battle_message_scenes.clear();
            if runtime_shell
                .visible_bug_contest_replacement
                .as_ref()
                .is_some_and(|replacement| {
                    replacement.phase == VisibleBugContestReplacementPhase::AlreadyCaughtText
                })
            {
                runtime_shell
                    .visible_bug_contest_replacement
                    .as_mut()
                    .expect("checked Contest replacement")
                    .phase = VisibleBugContestReplacementPhase::StatsPrompt;
                runtime_shell.battle_message_scene = None;
                runtime_shell.visible_capture_animation = None;
                runtime_shell.yes_no_cursor = Some(MenuCursor {
                    surface_id: "ui:yes-no".to_string(),
                    option_index: 0,
                });
                set_shell_action_status(runtime_shell, "BUG CONTEST SWITCH POKEMON?");
                mark_runtime_snapshot_dirty(runtime_shell);
                return Ok(());
            }
            if runtime_shell.visible_blackout_phase == Some(VisibleBlackoutPhase::AwaitText) {
                runtime_shell.visible_blackout_phase = Some(VisibleBlackoutPhase::FadeOut);
                runtime_shell.screen_fade = Some(VisibleScreenFade::new(
                    ScriptFadeColor::White,
                    ScriptFadeDirection::Out,
                    8,
                ));
                mark_runtime_snapshot_dirty(runtime_shell);
                return Ok(());
            }
            if runtime_shell.pending_standard_capture.is_some() {
                // Capture success returns with anim_keepsprites. The source
                // clears the retained ball before the next capture boundary,
                // while the caught battler stays absent. Ordinary captures
                // next ask for a nickname; tutorial and Contest captures
                // return directly after their final authored text instead.
                continue_visible_capture_after_owned_surface(runtime_shell)?;
                return Ok(());
            }
            if runtime_shell
                .visible_capture_animation
                .as_ref()
                .is_some_and(|animation| animation.complete)
            {
                runtime_shell.visible_capture_animation = None;
            }
            if runtime_shell
                .shell
                .presentation_snapshot()?
                .pending_move_learn
                .is_some()
            {
                // Move learning is a retained battle-result surface. Do not
                // expose the overworld between its announcement and the
                // delete/stop decision.
                mark_runtime_snapshot_dirty(runtime_shell);
                return Ok(());
            }
            let automatic_move_slot = runtime_shell
                .shell
                .snapshot()?
                .battle
                .as_ref()
                .filter(|battle| battle.commands.player_turn_automatic)
                .and_then(|battle| battle.commands.player_move_slots.first().copied());
            if let Some(slot) = automatic_move_slot {
                // Recharge and locked multi-turn moves resume immediately
                // after the preceding text. The selected slot is only a
                // structurally valid input; core replaces it with the exact
                // retained move before PP/effect resolution.
                runtime_shell.battle_message_scene = None;
                mark_runtime_snapshot_dirty(runtime_shell);
                return resolve_visible_battle_move(runtime_shell, slot);
            }
            let terminal_scene = runtime_shell.battle_message_scene.is_some()
                && runtime_shell
                    .shell
                    .presentation_snapshot()?
                    .battle
                    .is_none();
            runtime_shell.battle_message_scene = None;
            let resume_trainer_settlement = runtime_shell.battle_shift_prompt_cursor.is_none()
                && runtime_shell.battle_switch_cursor.is_none()
                && runtime_shell
                    .shell
                    .snapshot()?
                    .battle
                    .as_ref()
                    .is_some_and(|battle| {
                        matches!(&battle.kind, crate::RuntimeBattleKind::Trainer { .. })
                            && battle.enemy_pokemon.hp == 0
                            && !battle.enemy_spikes_zero_hp_unchecked
                    });
            if terminal_scene {
                reset_visible_battle_exit_state(runtime_shell);
                runtime_shell.battle_hp_tween = None;
                runtime_shell.battle_exp_tween = None;
                runtime_shell.pending_battle_exp_tweens.clear();
                runtime_shell.battle_fanfare_messages.clear();
                runtime_shell.battle_evolution_cries.clear();
                runtime_shell.battle_evolution_cancellations.clear();
                runtime_shell.battle_sounds_after_messages.clear();
                runtime_shell.battle_level_stats.clear();
                reset_visible_music_state(runtime_shell);
                queue_visible_current_music(runtime_shell)?;
                if runtime_shell.pending_plain_battle_map_reload {
                    begin_visible_plain_battle_map_reload(runtime_shell)?;
                } else {
                    continue_visible_script_after_prompt(runtime_shell)?;
                }
            }
            if resume_trainer_settlement {
                return settle_visible_battle_after_action(runtime_shell);
            }
        }
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(());
    }
    if runtime_shell.intro_screen.is_some() {
        return skip_visible_intro_screen(runtime_shell, GameButton::A);
    }
    if runtime_shell.credits_screen.is_some() {
        return press_visible_credits_a_button(runtime_shell);
    }
    if runtime_shell.pending_delete_save.is_some() {
        return confirm_visible_delete_save_screen(runtime_shell);
    }
    if runtime_shell.pending_clock_reset.is_some() {
        return confirm_visible_clock_reset_screen(runtime_shell);
    }
    if runtime_shell.options_menu_open {
        return confirm_visible_options_selection(runtime_shell);
    }
    if runtime_shell.title_menu.is_some() {
        return press_visible_title_confirm_button(runtime_shell, GameButton::A);
    }
    if runtime_shell.pending_time_set.is_some() {
        return press_visible_time_set_a_button(runtime_shell);
    }
    if runtime_shell.pending_oak_intro.is_some() {
        return press_visible_oak_intro_a_button(runtime_shell);
    }
    if runtime_shell.pending_gender_selection.is_some() {
        return confirm_visible_gender_selection(runtime_shell);
    }
    // Input must observe the authoritative state after the compiled script
    // run. The render cache can still contain the preceding text for one
    // frame, which made Mom's newly published PHONE prompt consume A while
    // ComeHomeForDSTText had not printed yet.
    let snapshot = runtime_shell.shell.snapshot()?;
    let presentation_snapshot = runtime_shell.shell.presentation_snapshot()?;
    if advance_visible_wait_sfx_boundary(runtime_shell, &presentation_snapshot, true)? {
        return Ok(());
    }
    if runtime_shell.pack_toss.is_some() {
        return confirm_visible_pack_toss(runtime_shell);
    }
    if runtime_shell.pc_item_quantity.is_some() {
        if !visible_pc_item_quantity_input_ready(runtime_shell) { return Ok(()); }
        return commit_visible_pc_item_quantity(runtime_shell);
    }
    if runtime_shell.pc_confirmation.is_some() {
        let selected =
            strict_readonly_cursor_index(&runtime_shell.yes_no_cursor, "pc:confirmation", 2)
                .context("PC confirmation requires a valid cursor")?;
        return resolve_visible_pc_confirmation(runtime_shell, selected == 0);
    }
    if runtime_shell.party_mail_take_stage.is_some() {
        let surface = if runtime_shell.party_mail_take_stage == Some(1) {
            "party:mail-send-pc"
        } else {
            "party:mail-lose-message"
        };
        let selected = strict_readonly_cursor_index(&runtime_shell.yes_no_cursor, surface, 2)
            .context("party Mail prompt requires a valid cursor")?;
        return resolve_visible_party_mail_take_prompt(runtime_shell, selected == 0);
    }
    if runtime_shell.pending_contextual_field_move.is_some() {
        let selected =
            strict_readonly_cursor_index(&runtime_shell.yes_no_cursor, "field:move-confirm", 2)
                .context("contextual field-move prompt requires a valid cursor")?;
        return resolve_visible_contextual_field_move_prompt(runtime_shell, selected == 0);
    }
    if runtime_shell.held_item_swap_prompt {
        let selected =
            strict_readonly_cursor_index(&runtime_shell.yes_no_cursor, "party:held-item-swap", 2)
                .context("held-item swap prompt requires a valid cursor")?;
        return resolve_visible_held_item_swap_prompt(runtime_shell, selected == 0);
    }
    // Core may publish YesNoBox as soon as it reaches `yesorno`, while the
    // presentation still has authored pages to print. The prompt owns A only
    // after those pages are fully consumed; otherwise A advances the text.
    if runtime_shell
        .visible_bug_contest_replacement
        .as_ref()
        .is_some_and(|replacement| {
            replacement.phase == VisibleBugContestReplacementPhase::StatsPrompt
        })
    {
        return confirm_visible_pending_yes_no(runtime_shell);
    }
    if snapshot.ui.pending_yes_no.is_some() {
        if snapshot.ui.text.as_ref().map(|text| text.label.as_str())
            != presentation_snapshot
                .ui
                .text
                .as_ref()
                .map(|text| text.label.as_str())
            || !visible_field_dialogue_is_fully_revealed(runtime_shell, &presentation_snapshot)
        {
            return Ok(());
        }
        if advance_visible_completed_field_text_page(runtime_shell, &presentation_snapshot)? {
            return Ok(());
        }
        return confirm_visible_pending_yes_no(runtime_shell);
    }
    if (runtime_shell.field_notice.is_some() || runtime_shell.pc_notice.is_some())
        && !visible_field_dialogue_is_fully_revealed(runtime_shell, &snapshot)
    {
        return Ok(());
    }
    if (runtime_shell.field_notice.is_some() || runtime_shell.pc_notice.is_some())
        && advance_visible_completed_field_text_page(runtime_shell, &snapshot)?
    {
        return Ok(());
    }
    if runtime_shell.tmhm_teach_prompt_cursor.is_some() {
        return resolve_visible_tmhm_teach_prompt(runtime_shell);
    }
    if runtime_shell.tmhm_decision_prompt_cursor.is_some() {
        return resolve_visible_tmhm_decision_prompt(runtime_shell);
    }
    if runtime_shell.field_notice.is_some()
        && runtime_shell.pending_field_travel_delay_frames.is_some()
    {
        return Ok(());
    }
    if runtime_shell.field_notice.is_some() && visible_field_notice_uses_prompt_arrow(runtime_shell)
    {
        queue_visible_shell_sound_effect(runtime_shell, "SFX_READ_TEXT_2")?;
    }
    if let Some(cancellation) = runtime_shell.field_evolution_cancellation.as_mut()
        && runtime_shell.field_notice.as_deref() == Some(cancellation.trigger_message.as_str())
    {
        cancellation.accepted = true;
        runtime_shell.pending_field_notice_cry = cancellation.report.target_species.clone();
    }
    let completed_evolution_party_index = runtime_shell
        .field_evolution_cancellation
        .as_ref()
        .filter(|cancellation| {
            cancellation.accepted
                && cancellation.report.pending_move_learns.is_empty()
                && runtime_shell.field_notice.as_deref()
                    == Some(cancellation.evolved_message.as_str())
        })
        .map(|cancellation| cancellation.party_index);
    if let Some(party_index) = completed_evolution_party_index {
        runtime_shell.field_evolution_cancellation = None;
        record_visible_completed_evolution(runtime_shell, party_index)?;
    }
    if runtime_shell.field_notice.take().is_some() {
        if runtime_shell
            .visible_bug_contest_replacement
            .as_ref()
            .is_some_and(|replacement| {
                replacement.phase == VisibleBugContestReplacementPhase::CaughtText
            })
        {
            let replacement = runtime_shell
                .visible_bug_contest_replacement
                .take()
                .expect("checked Contest replacement");
            runtime_shell.field_notice_scene = None;
            runtime_shell.field_text_reveal = None;
            return finish_visible_wild_battle_exit(
                runtime_shell,
                replacement.scripted_static_wild,
                "bug_contest_capture_replaced",
            );
        }
        if runtime_shell.pending_gift_pokemon_pc_notice {
            return finish_visible_gift_pokemon_pc_notice(runtime_shell);
        }
        if runtime_shell.pending_trainer_intro.is_some() {
            return finish_visible_map_trainer_intro(runtime_shell);
        }
        let field_item_phase = runtime_shell
            .visible_field_item_notice
            .as_ref()
            .map(|notice| notice.phase.clone());
        match field_item_phase {
            Some(VisibleFieldItemPhase::PocketText | VisibleFieldItemPhase::BagFullText) => {
                runtime_shell.visible_field_item_notice = None;
                runtime_shell.field_text_reveal = None;
                runtime_shell.field_notice_scene = None;
                // The item command has returned. Resume before stale Write
                // history can replace its notice with the previous dialogue.
                continue_visible_script_after_prompt(runtime_shell)?;
                mark_runtime_snapshot_dirty(runtime_shell);
                return Ok(());
            }
            Some(VisibleFieldItemPhase::AwaitingPrompt) => {
                let notice = runtime_shell
                    .visible_field_item_notice
                    .as_mut()
                    .expect("prompted verbose-item phase retains its notice");
                runtime_shell.field_notice = Some(notice.pocket_text.clone());
                notice.phase = VisibleFieldItemPhase::PocketText;
                mark_runtime_snapshot_dirty(runtime_shell);
                return Ok(());
            }
            Some(VisibleFieldItemPhase::PromptEachQueuedPage)
                if runtime_shell.field_notice_queue.is_empty() =>
            {
                runtime_shell.visible_field_item_notice = None;
            }
            Some(VisibleFieldItemPhase::BagFullFoundText) => {
                let notice = runtime_shell
                    .visible_field_item_notice
                    .as_mut()
                    .expect("bag-full field-item phase retains its notice");
                runtime_shell.field_notice = Some(notice.pocket_text.clone());
                notice.phase = VisibleFieldItemPhase::BagFullText;
                mark_runtime_snapshot_dirty(runtime_shell);
                return Ok(());
            }
            _ => {}
        }
        let egg_hatch_phase = runtime_shell
            .visible_egg_hatch
            .as_ref()
            .map(|hatch| hatch.phase);
        if egg_hatch_phase == Some(VisibleEggHatchPhase::HuhText) {
            begin_visible_egg_hatch_animation(runtime_shell)?;
            return Ok(());
        }
        if egg_hatch_phase == Some(VisibleEggHatchPhase::HatchText) {
            let hatch = runtime_shell
                .visible_egg_hatch
                .take()
                .context("egg hatch text lost its presentation state")?;
            let default_name = crate::core::models::pokemon_species_display_name(&hatch.species_id);
            let nickname_pages = visible_nickname_prompt_pages(runtime_shell, &default_name, true)?;
            runtime_shell.pending_egg_hatch_nickname = Some(PendingEggHatchNickname {
                party_index: hatch.party_index,
                default_name,
            });
            runtime_shell.pending_name_choice = Some(VisibleNameChoice {
                nickname_pages,
                options: vec!["YES".to_string(), "NO".to_string()],
                selected: 0,
                player_menu: None,
                player_phase: None,
                motion_step: 0,
                motion_frames_remaining: 0,
                pending_player_name: None,
            });
            set_shell_action_status(runtime_shell, "NICKNAME HATCHED POKEMON");
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(());
        }
        if runtime_shell
            .visible_fishing_animation
            .is_some_and(|animation| animation.phase == VisibleFishingPhase::AwaitText)
        {
            runtime_shell.visible_fishing_animation = None;
        }
        play_pending_field_notice_sound(runtime_shell)?;
        if let Some(next) = runtime_shell.field_notice_queue.pop_front() {
            runtime_shell.field_notice = Some(next);
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(());
        }
        if begin_visible_poison_blackout_after_faint_text(runtime_shell)? {
            return Ok(());
        }
        if runtime_shell.pending_tmhm_text_stage.is_some() {
            runtime_shell.field_notice_scene = None;
            advance_visible_tmhm_text_stage(runtime_shell)?;
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(());
        }
        if runtime_shell.pending_field_travel_arrival {
            runtime_shell.pending_field_travel_arrival = false;
            if runtime_shell.visible_field_travel_animation
                == Some(VisibleFieldTravelAnimation::DigOut)
            {
                queue_visible_shell_sound_effect(runtime_shell, "SFX_WARP_TO")?;
                begin_visible_dig_travel_animation(runtime_shell, false)?;
                mark_runtime_snapshot_dirty(runtime_shell);
                return Ok(());
            }
            settle_visible_overworld_travel(runtime_shell)?;
        }
        if begin_pending_field_notice_effect(runtime_shell)? {
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(());
        }
        runtime_shell.field_notice_scene = None;
        if settle_pending_field_battle_entry_after_notice(runtime_shell)? {
            return Ok(());
        }
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(());
    }
    if runtime_shell.pc_notice.is_some() {
        queue_visible_shell_sound_effect(runtime_shell, "SFX_READ_TEXT_2")?;
    }
    if runtime_shell.pc_notice.take().is_some() {
        dismiss_visible_pc_notice(runtime_shell)?;
        return Ok(());
    }
    if visible_pokecenter_pc_text_boundary(runtime_shell).is_some() {
        let snapshot = runtime_shell.shell.presentation_snapshot()?;
        if !visible_field_dialogue_is_fully_revealed(runtime_shell, &snapshot) {
            return Ok(());
        }
        if advance_visible_completed_field_text_page(runtime_shell, &snapshot)? {
            return Ok(());
        }
        return close_visible_special_boundary(runtime_shell);
    }
    // A visible Player PC menu owns A even though the originating script's
    // text/window bookkeeping remains open underneath it.  Handling the
    // generic printer first made A silently close/advance that hidden layer
    // instead of selecting WITHDRAW, exactly matching the live stuck-PC bug.
    if runtime_shell.decoration_menu.is_some() {
        return confirm_visible_decoration_menu(runtime_shell);
    }
    if runtime_shell.player_pc_action_cursor.is_some() {
        return confirm_visible_player_pc_action(runtime_shell);
    }
    // `elevator` is a synchronous script-owned modal. Its cursor deliberately
    // advances past the opcode when the prompt opens, so selecting the visible
    // floor must take precedence over both the still-open field textbox and
    // the underlying active script cursor.
    if runtime_shell.elevator_cursor.is_some() {
        return select_visible_elevator_floor(runtime_shell);
    }
    // A/B accelerate the active printer to one character per frame;
    // they do not reveal a whole page atomically. Only a completed page may
    // advance the script, matching PrintLetterDelay and TypeScript main.
    if visible_field_dialog_pages(&presentation_snapshot, runtime_shell).is_some() {
        if !visible_field_dialogue_is_fully_revealed(runtime_shell, &presentation_snapshot) {
            return Ok(());
        }
        if advance_visible_completed_field_text_page(runtime_shell, &presentation_snapshot)? {
            return Ok(());
        }
    }
    if runtime_shell.pending_phone_prompt.is_some() {
        return confirm_visible_phone_prompt(runtime_shell);
    }
    if runtime_shell.pending_remember_password.is_some() {
        return confirm_visible_remember_password_prompt(runtime_shell);
    }
    if runtime_shell.bill_pc_pokemon_summary.is_some() && !visible_wait_sfx_finished(runtime_shell) {
        return Ok(());
    }
    if let Some(summary) = runtime_shell.bill_pc_pokemon_summary.as_mut() {
        // EggStatsJoypad always exits on A; regular A exits on BLUE_PAGE.
        if summary.page == 3 || visible_pc_pokemon_at(&snapshot, summary.location)?.is_egg {
            runtime_shell.bill_pc_pokemon_summary = None;
        } else {
            summary.page += 1;
        }
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(());
    }
    if runtime_shell.pending_pc_release.is_some() {
        return confirm_visible_pc_release_prompt(runtime_shell);
    }
    if snapshot.ui.pending_text_wait.is_some() {
        return advance_visible_pending_text_wait(runtime_shell);
    }
    if snapshot.pending_shop.is_some() {
        if !runtime_shell.shop_welcome_seen {
            queue_visible_shell_sound_effect(runtime_shell, "SFX_READ_TEXT_2")?;
            runtime_shell.shop_welcome_seen = true;
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(());
        }
        if runtime_shell.shop_notice.is_some() {
            return dismiss_visible_shop_notice(runtime_shell);
        }
        if runtime_shell.shop_quantity.is_some() {
            return confirm_visible_shop_quantity(runtime_shell);
        }
        if runtime_shell.shop_top_cursor.is_some() {
            return confirm_visible_shop_top_menu(runtime_shell);
        }
        if runtime_shell.sell_cursor.is_some() {
            return sell_selected_bag_item(runtime_shell);
        }
        return buy_visible_shop_cursor_item(runtime_shell);
    }
    if visible_menu_has_selectable_options(&snapshot) {
        return select_visible_menu_cursor_option(runtime_shell);
    }
    // PokemonCenterPC owns input until its hub and submenus close. The
    // suspended PCScript cursor points at closetext, not a menu action.
    // Battles likewise own input until their suspended map script resumes.
    if !runtime_shell.pc_hub_session_open && snapshot.battle.is_none() {
        if advance_visible_next_pending_script_request(runtime_shell, &snapshot)? {
            return Ok(());
        };
        if snapshot.ui.window_open {
            return close_active_runtime_surface(runtime_shell);
        }
        if snapshot.ui.active_pokemon_picture.is_some() {
            return close_visible_pokemon_picture(runtime_shell);
        }
        // An open textbox does not imply that the script asked to close it. Once
        // PrintText's pending label is consumed, execute the authored successor
        // (`promptbutton`, `waitbutton`, `yesorno`, or the next command) before
        // considering any host-side text close. Explicit non-text surfaces above
        // still own their canonical close boundary.
        if runtime_shell.active_script_cursor.is_some() {
            return execute_visible_active_script_step(runtime_shell);
        }
        if snapshot.ui.text_window_open {
            return close_visible_text_window(runtime_shell);
        }
        if !snapshot.script_events.command_queue.is_empty() {
            return execute_next_visible_queued_script_command(runtime_shell);
        }
        if snapshot.script_events.next_script.is_some() {
            return take_visible_next_script(runtime_shell);
        }
        if snapshot.script_events.script_ended.is_some() {
            return take_visible_script_end_state(runtime_shell);
        }
        if snapshot.script_events.map_reentry_script.is_some() {
            return take_visible_map_reentry_script(runtime_shell);
        }
        if !snapshot.script_events.deferred_scripts.is_empty() {
            return take_visible_deferred_script(runtime_shell);
        }
        if let Some(flag) = visible_auto_runtime_flag(&snapshot) {
            return consume_visible_runtime_flag_kind(runtime_shell, flag);
        }
    }
    if snapshot.pending_move_learn.is_some() {
        return confirm_visible_pending_move_learn(runtime_shell);
    }
    // Full-screen menus retain input ownership even when their originating
    // battle is still authoritative underneath them. NewPokedexEntry is the
    // canonical case: its internal pages must finish before battle can resume.
    if runtime_shell.pokedex_menu_open {
        return press_visible_pokedex_a_button(runtime_shell);
    }
    if snapshot.battle.is_some() {
        if runtime_shell.battle_shift_prompt_cursor.is_some() {
            return confirm_visible_trainer_shift_prompt(runtime_shell);
        }
        return press_visible_battle_a_button(runtime_shell);
    }
    if runtime_shell.pack_item_switch_origin.is_some() {
        return switch_visible_pack_item(runtime_shell);
    }
    if runtime_shell.pc_item_action == Some(VisiblePlayerPcAction::DepositItem)
        && visible_field_pack_is_open(runtime_shell)
    {
        let pocket = active_visible_field_pack_pocket(runtime_shell);
        if selected_field_pack_cancel_row(&snapshot, runtime_shell, &pocket)? {
            close_visible_pc_item_deposit_pack(runtime_shell);
            return Ok(());
        }
        return begin_visible_pc_item_quantity(runtime_shell);
    }
    if runtime_shell.tmhm_teach_prompt_cursor.is_some() {
        return resolve_visible_tmhm_teach_prompt(runtime_shell);
    }
    if runtime_shell.tmhm_decision_prompt_cursor.is_some() {
        return resolve_visible_tmhm_decision_prompt(runtime_shell);
    }
    if let Some(mode) = runtime_shell.field_pack_target_mode {
        return confirm_visible_field_pack_target(runtime_shell, mode);
    }
    if runtime_shell.field_pack_action_cursor.is_some() {
        return execute_visible_field_pack_action(runtime_shell);
    }
    if runtime_shell.bag_cursor.is_some()
        || runtime_shell.key_item_cursor.is_some()
        || matches!(
            runtime_shell.field_pack_pocket.as_ref(),
            Some(FieldPackPocket::Custom(_))
        )
    {
        return open_visible_field_pack_action_menu(runtime_shell);
    }
    if runtime_shell.tmhm_cursor.is_some() {
        if selected_field_pack_cancel_row(&snapshot, runtime_shell, &FieldPackPocket::TmHm)? {
            return close_visible_field_pack_from_cancel(runtime_shell);
        }
        return open_visible_field_pack_action_menu(runtime_shell);
    }
    if snapshot.battle.is_none() && runtime_shell.ball_cursor.is_some() {
        if selected_field_pack_cancel_row(&snapshot, runtime_shell, &FieldPackPocket::Balls)? {
            return close_visible_field_pack_from_cancel(runtime_shell);
        }
        return open_visible_field_pack_action_menu(runtime_shell);
    }
    if runtime_shell.pokegear_menu_open {
        return inspect_visible_pokegear_selection(runtime_shell);
    }
    if runtime_shell.options_menu_open {
        return confirm_visible_options_selection(runtime_shell);
    }
    if runtime_shell.trainer_card_open {
        return advance_visible_trainer_card(runtime_shell);
    }
    if runtime_shell.save_menu_open {
        return confirm_visible_save_menu(runtime_shell);
    }
    if runtime_shell.special_boundary.is_some() {
        return close_visible_special_boundary(runtime_shell);
    }
    if runtime_shell.party_menu_open {
        if runtime_shell.mailbox_attach_index.is_some() {
            return attach_visible_mailbox_mail(runtime_shell);
        }
        if runtime_shell.pending_script_party_selection.is_some() {
            let snapshot = runtime_shell.shell.presentation_snapshot()?;
            let party_index = snapshot
                .party
                .slots
                .get(runtime_shell.party_cursor)
                .map(|slot| slot.index);
            return resolve_visible_script_party_selection(runtime_shell, party_index);
        }
        if runtime_shell.party_hp_transfer_source.is_some() {
            return confirm_visible_party_hp_transfer_target(runtime_shell);
        }
        if runtime_shell.party_move_reorder_open {
            return confirm_visible_party_move_reorder(runtime_shell);
        }
        if runtime_shell.party_give_take_cursor.is_some() {
            return confirm_visible_party_give_take(runtime_shell);
        }
        if runtime_shell.party_summary_open {
            // StatsScreenWaitCry precedes every Stats joypad path, including
            // the final-page and Egg A exits.
            if !visible_wait_sfx_finished(runtime_shell) {
                return Ok(());
            }
            let snapshot = runtime_shell.shell.presentation_snapshot()?;
            let slot = selected_party_slot_snapshot(&snapshot, runtime_shell.party_cursor)?;
            if slot.pokemon.is_egg || runtime_shell.party_summary_page >= 3 {
                record_visible_runtime_action(runtime_shell, "party:summary:close")?;
                close_visible_party_summary(runtime_shell);
                continue_visible_script_after_prompt(runtime_shell)?;
                return Ok(());
            }
            return cycle_visible_party_summary_page(runtime_shell, 1);
        }
        if runtime_shell.fly_cursor.is_some() {
            return confirm_visible_fly_destination(runtime_shell);
        }
        if runtime_shell.party_switch_cursor.is_some() {
            return confirm_visible_party_switch_target(runtime_shell);
        }
        if runtime_shell.party_action_cursor.is_some() {
            return execute_visible_party_action(runtime_shell);
        }
        return open_visible_party_action_menu(runtime_shell);
    }
    if runtime_shell.bill_pc_action_cursor.is_some() {
        return confirm_visible_bill_pc_action(runtime_shell);
    }
    if runtime_shell.decoration_menu.is_some() {
        return confirm_visible_decoration_menu(runtime_shell);
    }
    if runtime_shell.player_pc_action_cursor.is_some() {
        return confirm_visible_player_pc_action(runtime_shell);
    }
    if runtime_shell.mailbox_action_cursor.is_some() {
        return confirm_visible_mailbox_action(runtime_shell);
    }
    if runtime_shell.mailbox_cursor.is_some() {
        return confirm_visible_mailbox_selection(runtime_shell);
    }
    if runtime_shell.bill_pc_box_action_cursor.is_some() {
        return confirm_visible_bill_pc_box_action(runtime_shell);
    }
    if runtime_shell.bill_pc_box_cursor.is_some() {
        return confirm_visible_bill_pc_box(runtime_shell);
    }
    if runtime_shell.bill_pc_pokemon_action_cursor.is_some() {
        return confirm_visible_bill_pc_pokemon_action(runtime_shell);
    }
    if runtime_shell.pc_hub_cursor.is_some() {
        return confirm_visible_pc_hub(runtime_shell);
    }
    if runtime_shell.storage_cursor.is_some() {
        if runtime_shell.bill_pc_move_open {
            return confirm_visible_bill_pc_move(runtime_shell);
        }
        return open_visible_bill_pc_pokemon_actions(runtime_shell);
    }
    if runtime_shell.pc_item_cursor.is_some() {
        if runtime_shell.pc_item_switch_origin.is_some() { return switch_visible_pc_item(runtime_shell); }
        let selected = strict_readonly_cursor_index(&runtime_shell.pc_item_cursor,
            "pc:items", snapshot.bag.pc_items.len() + 1)
            .context("PC item list requires an item or CANCEL cursor")?;
        if selected == snapshot.bag.pc_items.len() {
            close_visible_pc_item_list(runtime_shell);
            return Ok(());
        }
        return begin_visible_pc_item_quantity(runtime_shell);
    }
    if runtime_shell.start_menu_cursor.is_some() {
        return select_visible_start_menu_option(runtime_shell);
    }
    if !snapshot.script_events.audio_events.is_empty() {
        return drain_visible_audio_events(runtime_shell);
    }
    if has_visible_pending_non_audio_script_events(&snapshot) {
        return drain_visible_non_audio_script_events(runtime_shell);
    }
    if runtime_shell.active_script_cursor.is_some() {
        return execute_visible_active_script_step(runtime_shell);
    }
    if execute_visible_contextual_field_move(runtime_shell)? {
        return Ok(());
    }
    if runtime_shell
        .shell
        .last_frame()
        .and_then(|frame| frame.interaction.as_ref())
        .is_some()
    {
        return execute_last_interaction_script(runtime_shell);
    }
    if runtime_shell
        .shell
        .current_overworld_interaction_checked()?
        .is_some()
    {
        return execute_current_overworld_interaction_script(runtime_shell);
    }
    Ok(())
}

fn has_visible_shell_a_action(runtime_shell: &mut BevyRuntimeShell) -> Result<bool> {
    if runtime_shell.mailbox_cursor.is_some() || runtime_shell.mailbox_action_cursor.is_some() {
        return Ok(true);
    }
    if runtime_shell
        .pokegear_phone_call
        .as_ref()
        .is_some_and(|call| {
            matches!(
                call.phase,
                VisiblePokegearPhoneCallPhase::NoServicePrompt
                    | VisiblePokegearPhoneCallPhase::AwaitHangup
            )
        })
    {
        return Ok(true);
    }
    if runtime_shell.bill_pc_move_save.is_some()
        || runtime_shell.pc_release_sequence.is_some()
        || runtime_shell.pc_transfer_sequence.is_some()
        || runtime_shell.pc_item_move_sequence.is_some()
    {
        return Ok(true);
    }
    if runtime_shell.player_walk_frame_ticks > 0 {
        return Ok(false);
    }
    if !runtime_shell.battle_messages.is_empty()
        || runtime_shell
            .battle_exp_tween
            .as_ref()
            .is_some_and(|tween| tween.started)
        || runtime_shell
            .battle_level_stats
            .front()
            .is_some_and(|stats| stats.active)
    {
        return Ok(true);
    }
    // These text surfaces are owned entirely by the Bevy presentation shell;
    // the authoritative snapshot need not have `ui.window_open` set. Their
    // A/B handlers already implement reveal, queue, prompt, and travel
    // continuation, so route the physical button to that visible owner.
    if runtime_shell.field_notice.is_some() || runtime_shell.pc_notice.is_some() {
        return Ok(true);
    }
    if runtime_shell.visible_diploma.is_some() {
        return Ok(true);
    }
    if runtime_shell.visible_unown_words.is_some() {
        return Ok(true);
    }
    if runtime_shell.visible_heal_machine.is_some() || runtime_shell.visible_magnet_train.is_some()
    {
        return Ok(true);
    }
    if runtime_shell.visible_card_flip.is_some() {
        return Ok(true);
    }
    if runtime_shell.visible_slot_machine.is_some() {
        return Ok(true);
    }
    if runtime_shell.visible_unown_puzzle.is_some() {
        return Ok(true);
    }
    if runtime_shell.visible_unown_printer.is_some() {
        return Ok(true);
    }
    if runtime_shell.visible_mom_bank.is_some() {
        return Ok(true);
    }
    if runtime_shell.pending_day_of_week.is_some() {
        return Ok(true);
    }
    if runtime_shell.kurt_apricorn_cursor.is_some() {
        return Ok(true);
    }
    if runtime_shell.visible_buena_password.is_some() {
        return Ok(true);
    }
    if runtime_shell.visible_battle_tower_challenge_menu.is_some() {
        return Ok(true);
    }
    if runtime_shell.visible_battle_tower_room_menu.is_some() {
        return Ok(true);
    }
    if runtime_shell.buena_prize_cursor.is_some() {
        return Ok(true);
    }
    if runtime_shell.intro_screen.is_some() {
        return Ok(true);
    }
    if runtime_shell.credits_screen.is_some() {
        return Ok(true);
    }
    if runtime_shell.pending_delete_save.is_some() || runtime_shell.pending_clock_reset.is_some() {
        return Ok(true);
    }
    if runtime_shell.title_menu.is_some() {
        return Ok(true);
    }
    if runtime_shell.pending_time_set.is_some() {
        return Ok(true);
    }
    if runtime_shell.pending_oak_intro.is_some() {
        return Ok(true);
    }
    if runtime_shell.pending_gender_selection.is_some() {
        return Ok(true);
    }
    // Input ownership must never be decided from the presentation cache. A
    // room callback can drain its script events after the last rendered
    // snapshot; treating that stale snapshot as modal steals the player's
    // next A press from the authoritative overworld interaction transaction.
    // This path is evaluated for physical input routing, not bitmap rendering.
    let snapshot = runtime_shell.shell.presentation_snapshot()?;
    if snapshot.ui.pending_yes_no.is_some()
        || runtime_shell.pending_phone_prompt.is_some()
        || runtime_shell.pending_remember_password.is_some()
        || snapshot.ui.pending_text_wait.is_some()
        || snapshot.pending_move_learn.is_some()
        || snapshot.pending_shop.is_some()
        || snapshot.script_events.pending_text_label.is_some()
        || snapshot.script_events.pending_map_load.is_some()
        || snapshot.script_events.pending_map_refresh.is_some()
        || snapshot.script_events.pending_music_fade.is_some()
        || snapshot.script_events.pending_screen_fade.is_some()
        || !snapshot.script_events.pending_delays.is_empty()
        || !snapshot.script_events.pending_earthquakes.is_empty()
        || !snapshot.script_events.pending_emotes.is_empty()
        || snapshot.ui.text_window_open
        || snapshot.ui.window_open
        || snapshot.ui.active_pokemon_picture.is_some()
        || snapshot.script_events.pending_script_warp.is_some()
        || !snapshot.script_events.command_queue.is_empty()
        || snapshot.script_events.next_script.is_some()
        || snapshot.script_events.map_reentry_script.is_some()
        || !snapshot.script_events.deferred_scripts.is_empty()
        || snapshot.script_events.script_ended.is_some()
        || !snapshot.script_events.audio_events.is_empty()
        || has_visible_pending_non_audio_script_events(&snapshot)
        || visible_auto_runtime_flag(&snapshot).is_some()
        || runtime_shell.elevator_cursor.is_some()
        || runtime_shell.active_script_cursor.is_some()
        || runtime_shell.bag_cursor.is_some()
        || runtime_shell.key_item_cursor.is_some()
        || runtime_shell.ball_cursor.is_some()
        || runtime_shell.tmhm_cursor.is_some()
        || runtime_shell.custom_item_cursor.is_some()
        || runtime_shell.field_pack_target_mode.is_some()
        || runtime_shell.storage_cursor.is_some()
        || runtime_shell.pc_item_cursor.is_some()
        || runtime_shell.pokedex_menu_open
        || runtime_shell.pokegear_menu_open
        || runtime_shell.trainer_card_open
        || runtime_shell.options_menu_open
        || runtime_shell.save_menu_open
        || runtime_shell.special_boundary.is_some()
        || runtime_shell.party_menu_open
        || runtime_shell.start_menu_cursor.is_some()
        || visible_menu_has_selectable_options(&snapshot)
        || snapshot.battle.is_some()
    {
        return Ok(true);
    }
    // Ordinary map/NPC collisions belong to the authoritative overworld
    // joypad transaction. Claiming them as Bevy-shell A actions prevents the
    // same frame from ever reaching `execute_interaction_script`, producing a
    // sound/no-dialogue no-op after walking. Only modal surfaces above own A.
    Ok(false)
}

fn continue_visible_capture_after_owned_surface(
    runtime_shell: &mut BevyRuntimeShell,
) -> Result<()> {
    let prompt_for_nickname = runtime_shell
        .pending_standard_capture
        .as_ref()
        .context("capture continuation lost its pending completion")?
        .prompt_for_nickname;
    runtime_shell.battle_message_scene = None;
    if prompt_for_nickname {
        runtime_shell.pending_name_choice = Some(VisibleNameChoice {
            nickname_pages: visible_nickname_prompt_pages(
                runtime_shell,
                &runtime_shell.pending_standard_capture.as_ref().unwrap().default_name,
                false,
            )?,
            options: vec!["YES".to_string(), "NO".to_string()],
            selected: 0,
            player_menu: None,
            player_phase: None,
            motion_step: 0,
            motion_frames_remaining: 0,
            pending_player_name: None,
        });
        set_shell_action_status(runtime_shell, "NICKNAME CAUGHT POKEMON");
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(());
    }
    finish_visible_capture_nickname(runtime_shell, None)
}

fn press_visible_pokedex_a_button(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    if pokedex_input_delay_active(runtime_shell) {
        return Ok(());
    }
    if runtime_shell.pokedex_controls.unown_cursor.take().is_some() {
        runtime_shell.pokedex_controls.option_cursor =
            Some(match runtime_shell.pokedex_controls.mode {
                VisiblePokedexMode::New => 0,
                VisiblePokedexMode::Old => 1,
                VisiblePokedexMode::Alphabetical => 2,
            });
        return Ok(());
    }
    if runtime_shell.pokedex_controls.printer_open {
        return Ok(());
    }
    if runtime_shell.pokedex_controls.area_region.take().is_some() {
        return Ok(());
    }
    if runtime_shell.pokedex_detail_open && !runtime_shell.pokedex_scripted_entry {
        match runtime_shell.pokedex_controls.entry_action {
            1 => {
                runtime_shell.pokedex_controls.area_region = Some(false);
                return Ok(());
            }
            2 => {
                let snapshot = runtime_shell.shell.snapshot()?;
                let species =
                    selected_pokedex_catalog_species(&snapshot, runtime_shell.pokedex_cursor)?;
                return queue_visible_pokemon_cry(
                    runtime_shell,
                    &species.species_id,
                    "pokedex_entry",
                );
            }
            3 => {
                runtime_shell.pokedex_controls.printer_open = true;
                return Ok(());
            }
            _ => {}
        }
    }
    if runtime_shell.pokedex_controls.search_cursor.is_some() {
        return press_visible_pokedex_search_a(runtime_shell);
    }
    if let Some(cursor) = runtime_shell.pokedex_controls.option_cursor.take() {
        if cursor == 3 {
            runtime_shell.pokedex_controls.unown_cursor = Some(0);
            return Ok(());
        }
        let mode = match cursor {
            0 => VisiblePokedexMode::New,
            1 => VisiblePokedexMode::Old,
            _ => VisiblePokedexMode::Alphabetical,
        };
        if mode == runtime_shell.pokedex_controls.mode {
            return Ok(());
        }
        runtime_shell.pokedex_controls.mode = mode;
        let snapshot = runtime_shell.shell.snapshot()?;
        let order = visible_pokedex_order(&snapshot, runtime_shell.pokedex_controls.mode);
        if let Some(&index) = order.first() {
            runtime_shell.pokedex_cursor = index;
        }
        runtime_shell.pokedex_scroll = 0;
        return record_visible_runtime_action(runtime_shell, "pokedex:mode");
    }
    if runtime_shell.pokedex_detail_open {
        let snapshot = runtime_shell.shell.presentation_snapshot()?;
        let species = selected_pokedex_catalog_species(&snapshot, runtime_shell.pokedex_cursor)?;
        let entry = snapshot
            .presentation
            .pokedex_entries
            .get(&species.species_id)
            .with_context(|| {
                format!("compiled pack missing Pokedex entry {}", species.species_id)
            })?;
        let page_count = entry.pages.len();
        anyhow::ensure!(
            page_count > 0,
            "compiled Pokedex entry {} has no pages",
            species.species_id
        );
        anyhow::ensure!(
            runtime_shell.pokedex_detail_page < page_count,
            "Pokedex detail page {} is outside {page_count} pages for {}",
            runtime_shell.pokedex_detail_page,
            species.species_id
        );
        if runtime_shell.pokedex_scripted_entry
            && runtime_shell.pokedex_detail_page + 1 >= page_count
        {
            record_visible_runtime_action(runtime_shell, "pokedex:scripted_entry:close")?;
            close_visible_pokedex_menu(runtime_shell);
            if runtime_shell.pending_standard_capture.is_some() {
                continue_visible_capture_after_owned_surface(runtime_shell)?;
                return Ok(());
            }
            continue_visible_script_after_prompt(runtime_shell)?;
            return Ok(());
        }
        runtime_shell.pokedex_detail_page = (runtime_shell.pokedex_detail_page + 1) % page_count;
        let page_number = runtime_shell.pokedex_detail_page + 1;
        record_visible_runtime_action(runtime_shell, format!("pokedex:detail:page:{page_number}"))?;
        return Ok(());
    }
    inspect_visible_pokedex_selection(runtime_shell)
}

fn press_visible_b_button(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let menu_snapshot = runtime_shell.shell.snapshot()?;
    if visible_menu_has_selectable_options(&menu_snapshot) {
        let menu = menu_snapshot.ui.menu.as_ref().expect("selectable menu");
        if menu.menu_2d_requested { return cancel_visible_2d_menu(runtime_shell); }
        if visible_runtime_menu_disables_b(runtime_shell, menu)? { return Ok(()); }
        return close_active_runtime_surface(runtime_shell);
    }
    if runtime_shell.pokedex_menu_open && pokedex_input_delay_active(runtime_shell) { return Ok(()); }
    if runtime_shell.pokedex_menu_open && runtime_shell.pokedex_controls.unown_cursor.is_some() { return press_visible_pokedex_a_button(runtime_shell); }
    if runtime_shell.pokedex_menu_open {
        if std::mem::take(&mut runtime_shell.pokedex_controls.printer_open) { return Ok(()); }
        if runtime_shell.pokedex_controls.area_region.take().is_some() { return Ok(()); }
    }
    if runtime_shell.pokedex_menu_open && !runtime_shell.pokedex_detail_open {
        if runtime_shell.pokedex_controls.search_cursor.take().is_some() { return Ok(()); }
        if runtime_shell.pokedex_controls.search_results.take().is_some() {
            if let Some((cursor, scroll)) = runtime_shell.pokedex_controls.search_backup.take() {
                runtime_shell.pokedex_cursor = cursor;
                runtime_shell.pokedex_scroll = scroll;
            }
            runtime_shell.pokedex_controls.search_cursor = Some(0);
            runtime_shell.pokedex_controls.search_types = [1, 0];
            return Ok(());
        }
    }
    if runtime_shell.pokedex_menu_open && runtime_shell.pokedex_controls.option_cursor.take().is_some() { return Ok(()); }
    if runtime_shell.visible_battle_sliding_intro.is_some() {
        return Ok(());
    }
    if runtime_shell.mailbox_confirmation_response.is_some() {
        return Ok(());
    }
    if runtime_shell.pokegear_exit.is_some() { return Ok(()); }
    if runtime_shell.pokegear_menu_open && runtime_shell.pokegear_map_radio_delay.is_some() {
        return if runtime_shell.pokegear_map_radio_delay == Some(0) {
            close_visible_map_radio(runtime_shell)
        } else { Ok(()) };
    }
    if runtime_shell.pokegear_phone_call.as_ref().is_some_and(|call| {
        matches!(call.phase, VisiblePokegearPhoneCallPhase::HangingUp { .. })
    }) {
        return Ok(());
    }

    if runtime_shell
        .pokegear_phone_call
        .as_ref()
        .is_some_and(|call| call.phase == VisiblePokegearPhoneCallPhase::NoServicePrompt)
    {
        return dismiss_visible_pokegear_no_service_prompt(runtime_shell);
    }
    if runtime_shell
        .pokegear_phone_call
        .as_ref()
        .is_some_and(|call| call.phase == VisiblePokegearPhoneCallPhase::AwaitHangup)
    {
        return finish_visible_pokegear_phone_call(runtime_shell);
    }
    if runtime_shell
        .special_boundary
        .as_ref()
        .is_some_and(|boundary| boundary.label == "PrinterError2")
    {
        return close_visible_special_boundary(runtime_shell);
    }
    if visible_pc_printer_status(runtime_shell) {
        record_visible_runtime_action(runtime_shell, "printer:error:b_cancel")?;
        runtime_shell.pc_notice = None;
        queue_visible_current_music(runtime_shell)?;
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(());
    }
    if runtime_shell.bill_pc_move_save.is_some()
        || runtime_shell.pc_release_sequence.is_some()
        || runtime_shell.pc_transfer_sequence.is_some()
        || runtime_shell.pc_item_move_sequence.is_some()
    {
        return Ok(());
    }
    if runtime_shell.pending_mail_read.is_some() {
        return close_visible_mail_read(runtime_shell);
    }
    if runtime_shell.pending_name_choice.is_some() {
        if advance_visible_nickname_prompt(runtime_shell) {
            return Ok(());
        }
        if runtime_shell
            .pending_name_choice
            .as_ref()
            .is_some_and(|choice| choice.player_menu.is_some())
        {
            // ShowPlayerNamingChoices sets STATICMENU_DISABLE_B. The custom
            // return phases are part of the same blocking NamePlayer call.
            return Ok(());
        }
        runtime_shell.pending_name_choice = None;
        if runtime_shell.pending_egg_hatch_nickname.is_some() {
            return finish_visible_egg_hatch_nickname(runtime_shell, None);
        }
        if runtime_shell.pending_standard_capture.is_some() {
            return finish_visible_capture_nickname(runtime_shell, None);
        }
        if runtime_shell.pending_gift_pokemon_nickname.is_some() {
            return finish_visible_gift_pokemon_nickname(runtime_shell, None);
        }
        return Ok(());
    }
    if runtime_shell.visible_diploma.is_some() {
        return close_visible_diploma(runtime_shell);
    }
    if runtime_shell.visible_unown_words.is_some() {
        return close_visible_unown_words(runtime_shell);
    }
    if runtime_shell.visible_heal_machine.is_some() || runtime_shell.visible_magnet_train.is_some()
    {
        return Ok(());
    }
    if runtime_shell
        .battle_exp_tween
        .as_ref()
        .is_some_and(|tween| tween.started)
    {
        return Ok(());
    }
    if let Some(stats) = runtime_shell.battle_level_stats.front()
        && stats.active
    {
        if stats.frames_before_input == 0 {
            runtime_shell.battle_level_stats.pop_front();
            mark_runtime_snapshot_dirty(runtime_shell);
            finish_visible_empty_battle_reward_presentation(runtime_shell)?;
        }
        return Ok(());
    }
    if runtime_shell.visible_card_flip.is_some() {
        return close_visible_card_flip(runtime_shell);
    }
    if runtime_shell.visible_slot_machine.is_some() {
        return close_visible_slot_machine(runtime_shell);
    }
    if let Some(puzzle) = runtime_shell.visible_unown_puzzle.as_ref() {
        // UnownPuzzleJumptable ignores B; only the solved A/B wait accepts it.
        return if puzzle.solved { use_visible_unown_puzzle_cell(runtime_shell) } else { Ok(()) };
    }
    if runtime_shell.visible_unown_printer.is_some() {
        return close_visible_unown_printer(runtime_shell);
    }
    if runtime_shell.visible_mom_bank.is_some() {
        return cancel_visible_mom_bank(runtime_shell);
    }
    if let Some(prompt) = runtime_shell.pending_day_of_week.as_mut() {
        if prompt.confirming {
            prompt.confirming = false;
            prompt.yes_no_index = 0;
            set_shell_action_status(runtime_shell, "WHAT DAY IS IT?");
            mark_runtime_snapshot_dirty(runtime_shell);
        }
        return Ok(());
    }
    if !runtime_shell.battle_messages.is_empty() {
        if cancel_visible_battle_evolution(runtime_shell)? {
            return Ok(());
        }
        return press_visible_a_button(runtime_shell);
    }
    if runtime_shell.kurt_apricorn_cursor.is_some() {
        if runtime_shell.kurt_apricorn_quantity.take().is_some() {
            set_shell_action_status(runtime_shell, "WHICH APRICORN?");
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(());
        }
        return resolve_visible_kurt_apricorn_selection(runtime_shell, true);
    }
    if runtime_shell.visible_buena_password.is_some() {
        // STATICMENU_DISABLE_B: Buena's live-show password menu cannot be cancelled.
        return Ok(());
    }
    if runtime_shell.visible_battle_tower_challenge_menu.is_some() {
        return resolve_visible_battle_tower_challenge_menu(runtime_shell, true);
    }
    if let Some(menu) = runtime_shell.visible_battle_tower_room_menu.as_ref() {
        return match menu.phase {
            VisibleBattleTowerRoomMenuPhase::PickLevel => {
                runtime_shell
                    .visible_battle_tower_room_menu
                    .as_mut()
                    .context("Battle Tower room menu disappeared")?
                    .phase = VisibleBattleTowerRoomMenuPhase::ConfirmCancel { yes_no_index: 0 };
                set_shell_action_status(runtime_shell, "CANCEL BATTLE ROOM CHALLENGE?");
                mark_runtime_snapshot_dirty(runtime_shell);
                Ok(())
            }
            VisibleBattleTowerRoomMenuPhase::ConfirmCancel { .. } => {
                resolve_visible_battle_tower_room_cancel(runtime_shell, false)
            }
            VisibleBattleTowerRoomMenuPhase::Rejection { .. } => Ok(()),
        };
    }
    if runtime_shell.buena_prize_cursor.is_some()
        && runtime_shell.pc_confirmation.is_none()
        && runtime_shell.pc_notice.is_none()
    {
        return resolve_visible_buena_prize_selection(runtime_shell, true);
    }
    if runtime_shell.intro_screen.is_some() {
        return skip_visible_intro_screen(runtime_shell, GameButton::B);
    }
    if runtime_shell.credits_screen.is_some() {
        return press_visible_credits_b_button(runtime_shell);
    }
    if runtime_shell.pending_delete_save.is_some() {
        return close_visible_delete_save_screen(runtime_shell, "cancel");
    }
    if runtime_shell.pending_clock_reset.is_some() {
        return close_visible_clock_reset_screen(runtime_shell, "cancel");
    }
    if runtime_shell.options_menu_open {
        record_visible_runtime_action(runtime_shell, "options:close")?;
        close_visible_options_menu(runtime_shell);
        continue_visible_script_after_prompt(runtime_shell)?;
        return Ok(());
    }
    if runtime_shell.title_menu.is_some() {
        return Ok(());
    }
    if runtime_shell.pending_time_set.is_some() {
        return press_visible_time_set_b_button(runtime_shell);
    }
    if runtime_shell.pending_oak_intro.is_some() {
        return press_visible_oak_intro_b_button(runtime_shell);
    }
    if runtime_shell.pending_gender_selection.is_some() {
        record_visible_runtime_action(runtime_shell, "gender:b:ignored")?;
        runtime_shell
            .last_audio_events
            .push("gender B ignored".to_string());
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    if runtime_shell.save_menu_open {
        return cancel_visible_save_menu(runtime_shell);
    }
    if runtime_shell.pack_toss.is_some() {
        return cancel_visible_pack_toss(runtime_shell);
    }
    if runtime_shell.pc_item_quantity.is_some() {
        if !visible_pc_item_quantity_input_ready(runtime_shell) { return Ok(()); }
        runtime_shell.pc_item_quantity = None;
        runtime_shell.pc_notice = None;
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(());
    }
    if runtime_shell.pc_confirmation.is_some() {
        return resolve_visible_pc_confirmation(runtime_shell, false);
    }
    if runtime_shell.party_mail_take_stage.is_some() {
        return resolve_visible_party_mail_take_prompt(runtime_shell, false);
    }
    if runtime_shell.pending_contextual_field_move.is_some() {
        return resolve_visible_contextual_field_move_prompt(runtime_shell, false);
    }
    if runtime_shell.held_item_swap_prompt {
        return resolve_visible_held_item_swap_prompt(runtime_shell, false);
    }
    let snapshot = runtime_shell.shell.presentation_snapshot()?;
    if runtime_shell.elevator_cursor.is_some() {
        return cancel_visible_elevator_floor(runtime_shell);
    }
    if (runtime_shell.field_notice.is_some() || runtime_shell.pc_notice.is_some())
        && !visible_field_dialogue_is_fully_revealed(runtime_shell, &snapshot)
    {
        return Ok(());
    }
    if (runtime_shell.field_notice.is_some() || runtime_shell.pc_notice.is_some())
        && advance_visible_completed_field_text_page(runtime_shell, &snapshot)?
    {
        return Ok(());
    }
    if runtime_shell.tmhm_teach_prompt_cursor.is_some() {
        runtime_shell.tmhm_teach_prompt_cursor = Some(MenuCursor {
            surface_id: "pack:tmhm:teach-prompt".to_string(),
            option_index: 1,
        });
        return resolve_visible_tmhm_teach_prompt(runtime_shell);
    }
    if runtime_shell.tmhm_decision_prompt_cursor.is_some() {
        runtime_shell.tmhm_decision_prompt_cursor = Some(MenuCursor {
            surface_id: "pack:tmhm:decision".to_string(),
            option_index: 1,
        });
        return resolve_visible_tmhm_decision_prompt(runtime_shell);
    }
    if runtime_shell.field_notice.is_some()
        && runtime_shell.pending_field_travel_delay_frames.is_some()
    {
        return Ok(());
    }
    if cancel_visible_field_evolution(runtime_shell)? {
        return Ok(());
    }
    if runtime_shell.field_notice.is_some() && visible_field_notice_uses_prompt_arrow(runtime_shell)
    {
        queue_visible_shell_sound_effect(runtime_shell, "SFX_READ_TEXT_2")?;
    }
    if runtime_shell.field_notice.take().is_some() {
        if runtime_shell
            .visible_bug_contest_replacement
            .as_ref()
            .is_some_and(|replacement| {
                replacement.phase == VisibleBugContestReplacementPhase::CaughtText
            })
        {
            let replacement = runtime_shell
                .visible_bug_contest_replacement
                .take()
                .expect("checked Contest replacement");
            runtime_shell.field_notice_scene = None;
            runtime_shell.field_text_reveal = None;
            return finish_visible_wild_battle_exit(
                runtime_shell,
                replacement.scripted_static_wild,
                "bug_contest_capture_replaced",
            );
        }
        if runtime_shell.pending_gift_pokemon_pc_notice {
            return finish_visible_gift_pokemon_pc_notice(runtime_shell);
        }
        if runtime_shell.pending_trainer_intro.is_some() {
            return finish_visible_map_trainer_intro(runtime_shell);
        }
        if runtime_shell
            .visible_fishing_animation
            .is_some_and(|animation| animation.phase == VisibleFishingPhase::AwaitText)
        {
            runtime_shell.visible_fishing_animation = None;
        }
        play_pending_field_notice_sound(runtime_shell)?;
        if let Some(next) = runtime_shell.field_notice_queue.pop_front() {
            runtime_shell.field_notice = Some(next);
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(());
        }
        if begin_visible_poison_blackout_after_faint_text(runtime_shell)? {
            return Ok(());
        }
        if runtime_shell.pending_tmhm_text_stage.is_some() {
            runtime_shell.field_notice_scene = None;
            advance_visible_tmhm_text_stage(runtime_shell)?;
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(());
        }
        if runtime_shell.pending_field_travel_arrival {
            runtime_shell.pending_field_travel_arrival = false;
            if runtime_shell.visible_field_travel_animation
                == Some(VisibleFieldTravelAnimation::DigOut)
            {
                queue_visible_shell_sound_effect(runtime_shell, "SFX_WARP_TO")?;
                begin_visible_dig_travel_animation(runtime_shell, false)?;
                mark_runtime_snapshot_dirty(runtime_shell);
                return Ok(());
            }
            settle_visible_overworld_travel(runtime_shell)?;
        }
        if begin_pending_field_notice_effect(runtime_shell)? {
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(());
        }
        runtime_shell.field_notice_scene = None;
        if settle_pending_field_battle_entry_after_notice(runtime_shell)? {
            return Ok(());
        }
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(());
    }
    if runtime_shell.pc_notice.is_some() {
        queue_visible_shell_sound_effect(runtime_shell, "SFX_READ_TEXT_2")?;
    }
    if runtime_shell.pc_notice.take().is_some() {
        dismiss_visible_pc_notice(runtime_shell)?;
        return Ok(());
    }
    if visible_pokecenter_pc_text_boundary(runtime_shell).is_some() {
        let snapshot = runtime_shell.shell.presentation_snapshot()?;
        if !visible_field_dialogue_is_fully_revealed(runtime_shell, &snapshot) {
            return Ok(());
        }
        if advance_visible_completed_field_text_page(runtime_shell, &snapshot)? {
            return Ok(());
        }
        return close_visible_special_boundary(runtime_shell);
    }
    let snapshot = runtime_shell.shell.presentation_snapshot()?;
    if snapshot.pending_move_learn.is_some() {
        return cancel_visible_pending_move_learn(runtime_shell);
    }
    if runtime_shell.pending_phone_prompt.is_some() {
        return decline_visible_phone_prompt(runtime_shell);
    }
    if runtime_shell.pending_remember_password.is_some() {
        return decline_visible_remember_password_prompt(runtime_shell);
    }
    if runtime_shell.bill_pc_pokemon_summary.is_some() && !visible_wait_sfx_finished(runtime_shell) {
        return Ok(());
    }
    if runtime_shell.bill_pc_pokemon_summary.take().is_some() {
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(());
    }
    if runtime_shell.pending_pc_release.take().is_some() {
        runtime_shell.yes_no_cursor = None;
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(());
    }
    if runtime_shell.bill_pc_pokemon_action_cursor.take().is_some() {
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(());
    }
    // As with A, B must not resolve a prompt using a stale rendered text body.
    let snapshot = runtime_shell.shell.snapshot()?;
    let presentation_snapshot = runtime_shell.shell.presentation_snapshot()?;
    if advance_visible_wait_sfx_boundary(runtime_shell, &presentation_snapshot, true)? {
        return Ok(());
    }
    if runtime_shell
        .visible_bug_contest_replacement
        .as_ref()
        .is_some_and(|replacement| {
            replacement.phase == VisibleBugContestReplacementPhase::StatsPrompt
        })
    {
        return decline_visible_pending_yes_no(runtime_shell);
    }
    if snapshot.ui.pending_yes_no.is_some() {
        if snapshot.ui.text.as_ref().map(|text| text.label.as_str())
            != presentation_snapshot
                .ui
                .text
                .as_ref()
                .map(|text| text.label.as_str())
            || !visible_field_dialogue_is_fully_revealed(runtime_shell, &presentation_snapshot)
        {
            return Ok(());
        }
        if advance_visible_completed_field_text_page(runtime_shell, &presentation_snapshot)? {
            return Ok(());
        }
        return decline_visible_pending_yes_no(runtime_shell);
    }
    if runtime_shell.pokegear_menu_open {
        if let Some(menu) = runtime_shell.pokegear_phone_menu.take() {
            // B from YesNoBox returns through .CancelDelete without printing;
            // B from the contact submenu executes .Cancel and prints AskWhoCall.
            runtime_shell.pokegear_phone_delete_question_retained = menu.delete_confirmation.is_some();
            record_visible_runtime_action(runtime_shell, "pokegear:phone:cancel")?;
            mark_runtime_presentation_dirty(runtime_shell);
            return Ok(());
        }
        record_visible_runtime_action(runtime_shell, "pokegear:close")?;
        if !runtime_shell.pokegear_standalone_map {
            return request_visible_pokegear_exit(runtime_shell);
        }
        // OverworldTownMap retains its originating textbox and core menu
        // beneath the modal. B belongs to the map UI first; closing it then
        // resumes the script at `closetext`/`end`.
        if snapshot.ui.menu.is_some() {
            let _ = runtime_shell.shell.close_active_menu()?;
        }
        close_visible_pokegear_menu(runtime_shell)?;
        if runtime_shell.start_menu_cursor.is_none() {
            continue_visible_script_after_prompt(runtime_shell)?;
        }
        return Ok(());
    }
    // The Player PC action menu is the visible modal owner. Its originating
    // script can retain a completed textbox underneath it; B must close the
    // PC rather than route through that hidden text, just as A selects the
    // visible PC action above the printer.
    if let Some(menu) = runtime_shell.decoration_menu.as_ref() {
        let phase = menu.phase.clone();
        return match phase {
            VisibleDecorationMenuPhase::Categories { .. } => {
                close_visible_decoration_menu(runtime_shell)
            }
            VisibleDecorationMenuPhase::Decorations { category, .. } => {
                return_visible_decoration_to_categories(runtime_shell, category)
            }
            VisibleDecorationMenuPhase::Side {
                category,
                item_cursor_index,
                ..
            } => return_visible_decoration_to_items(runtime_shell, category, item_cursor_index),
        };
    }
    if runtime_shell.player_pc_action_cursor.is_some() {
        return close_visible_player_pc(runtime_shell);
    }
    if runtime_shell.field_text_reveal.is_some()
        && visible_field_dialog_pages(&presentation_snapshot, runtime_shell).is_some()
    {
        // JoyTextDelay, JoyWaitAorB, and PromptButton all accept PAD_A or
        // PAD_B. Higher-priority YES/NO, selection, and cancelable modal
        // surfaces have already handled B above this ordinary text boundary.
        return press_visible_a_button(runtime_shell);
    }
    if snapshot.ui.pending_text_wait.is_some() {
        return advance_visible_pending_text_wait(runtime_shell);
    }
    if snapshot.pending_shop.is_some() {
        if !runtime_shell.shop_welcome_seen {
            queue_visible_shell_sound_effect(runtime_shell, "SFX_READ_TEXT_2")?;
            runtime_shell.shop_welcome_seen = true;
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(());
        }
        if runtime_shell.shop_notice.is_some() {
            return dismiss_visible_shop_notice(runtime_shell);
        }
        if runtime_shell.shop_quantity.take().is_some() {
            mark_runtime_snapshot_dirty(runtime_shell);
            // B cancels only the quantity chooser. The ASM/TypeScript flow
            // returns to the active BUY or SELL list, rather than discarding
            // that list and asking the clerk's top-level question.
            return Ok(());
        }
        if runtime_shell.shop_top_cursor.is_none() {
            runtime_shell.sell_cursor = None;
            runtime_shell.shop_top_cursor = Some(MenuCursor {
                surface_id: "shop:top".to_string(),
                option_index: 0,
            });
            runtime_shell.shop_notice = Some("Can I do anything\nelse for you?".to_string());
            runtime_shell
                .last_audio_events
                .push("returned to shop top menu".to_string());
            trim_event_log(&mut runtime_shell.last_audio_events);
            mark_runtime_snapshot_dirty(runtime_shell);
            return Ok(());
        }
        return close_visible_shop(runtime_shell);
    }
    if runtime_shell.bill_pc_action_cursor.is_some() {
        return close_visible_bill_pc_actions(runtime_shell);
    }
    if runtime_shell.mailbox_action_cursor.is_some() {
        runtime_shell.mailbox_action_cursor = None;
        return Ok(());
    }
    if runtime_shell.mailbox_cursor.is_some() {
        close_visible_mailbox(runtime_shell);
        return Ok(());
    }
    if runtime_shell.pc_item_cursor.is_some() && runtime_shell.pc_item_action.is_some() {
        if runtime_shell.pc_item_switch_origin.take().is_some() {
            mark_runtime_presentation_dirty(runtime_shell);
            return Ok(());
        }
        close_visible_pc_item_list(runtime_shell);
        return Ok(());
    }
    if runtime_shell.pc_item_action == Some(VisiblePlayerPcAction::DepositItem)
        && visible_field_pack_is_open(runtime_shell)
    {
        close_visible_pc_item_deposit_pack(runtime_shell);
        return Ok(());
    }
    if runtime_shell.bill_pc_box_action_cursor.take().is_some() {
        set_shell_action_status(runtime_shell, "CHOOSE A BOX");
        return Ok(());
    }
    if runtime_shell.bill_pc_box_cursor.is_some() {
        runtime_shell.bill_pc_box_cursor = None;
        runtime_shell.bill_pc_action_cursor = Some(MenuCursor {
            surface_id: "pc:bill-actions".to_string(),
            option_index: 2,
        });
        set_shell_action_status(runtime_shell, "BILL'S PC");
        return Ok(());
    }
    if runtime_shell.bill_pc_move_open && runtime_shell.bill_pc_move_source.is_some() {
        return cancel_visible_bill_pc_move_source(runtime_shell);
    }
    if runtime_shell.bill_pc_move_open {
        return close_visible_bill_pc_move_list(runtime_shell);
    }
    if runtime_shell.pc_hub_cursor.is_some() {
        return turn_off_visible_pc_hub(runtime_shell);
    }
    if runtime_shell.bill_pc_session_open
        && (runtime_shell.storage_cursor.is_some() || runtime_shell.pc_item_cursor.is_some())
    {
        return close_visible_pc_surface(runtime_shell);
    }
    if !runtime_shell.pokegear_menu_open
        && !runtime_shell.pc_hub_session_open
        && (snapshot.ui.text_window_open
            || snapshot.ui.window_open
            || snapshot.ui.menu.is_some()
            || snapshot.ui.active_pokemon_picture.is_some())
    {
        if snapshot
            .ui
            .menu
            .as_ref()
            .is_some_and(|menu| menu.menu_2d_requested)
        {
            return cancel_visible_2d_menu(runtime_shell);
        }
        return close_active_runtime_surface(runtime_shell);
    }
    if snapshot.battle.is_none() && visible_field_pack_is_open(runtime_shell) {
        if runtime_shell.pack_item_switch_origin.take().is_some() {
            set_shell_action_status(runtime_shell, "MOVE CANCELLED");
            return Ok(());
        }
        if runtime_shell.tmhm_teach_prompt_cursor.is_some() {
            runtime_shell.tmhm_teach_prompt_cursor = Some(MenuCursor {
                surface_id: "pack:tmhm:teach-prompt".to_string(),
                option_index: 1,
            });
            record_visible_runtime_action(runtime_shell, "pack:tmhm:teach:b")?;
            return resolve_visible_tmhm_teach_prompt(runtime_shell);
        }
        if runtime_shell.tmhm_decision_prompt_cursor.is_some() {
            runtime_shell.tmhm_decision_prompt_cursor = Some(MenuCursor {
                surface_id: "pack:tmhm:decision".to_string(),
                option_index: 1,
            });
            return resolve_visible_tmhm_decision_prompt(runtime_shell);
        }
        if runtime_shell.tmhm_forget_menu_open {
            runtime_shell.tmhm_forget_menu_open = false;
            runtime_shell.party_move_cursor = None;
            return open_visible_tmhm_decision_prompt(
                runtime_shell,
                VisibleTmHmDecision::StopLearning,
            );
        }
        if runtime_shell.field_pack_target_mode.is_some() {
            close_visible_field_pack_target(runtime_shell)?;
            continue_visible_script_after_prompt(runtime_shell)?;
            return Ok(());
        }
        if runtime_shell.field_pack_action_cursor.is_some() {
            record_visible_runtime_action(runtime_shell, "pack:actions:close")?;
            close_visible_field_pack_action_menu(runtime_shell);
            return Ok(());
        }
        if runtime_shell.party_held_item_give_target.is_some() {
            return close_visible_field_pack_from_cancel(runtime_shell);
        }
        record_visible_runtime_action(runtime_shell, "pack:close")?;
        runtime_shell.bag_cursor = None;
        runtime_shell.key_item_cursor = None;
        runtime_shell.ball_cursor = None;
        runtime_shell.tmhm_cursor = None;
        runtime_shell.custom_item_cursor = None;
        runtime_shell.field_pack_action_cursor = None;
        runtime_shell.field_pack_pocket = None;
        runtime_shell.field_pack_target_mode = None;
        runtime_shell
            .last_audio_events
            .push("closed field item cursor".to_string());
        trim_event_log(&mut runtime_shell.last_audio_events);
        continue_visible_script_after_prompt(runtime_shell)?;
        return Ok(());
    }
    if runtime_shell.party_menu_open {
        if let Some(mailbox_index) = runtime_shell.mailbox_attach_index.take() {
            close_visible_party_menu(runtime_shell);
            restore_visible_mailbox_position(runtime_shell, mailbox_index)?;
            return Ok(());
        }
        if runtime_shell.pending_script_party_selection.is_some() {
            return resolve_visible_script_party_selection(runtime_shell, None);
        }
        if runtime_shell.party_hp_transfer_source.is_some() {
            return cancel_visible_party_hp_transfer_target(runtime_shell);
        }
        if runtime_shell.bill_pc_session_open && runtime_shell.storage_cursor.is_some() {
            return close_visible_pc_surface(runtime_shell);
        }
        if runtime_shell.party_move_reorder_open {
            if let Some(origin) = runtime_shell.party_move_reorder_origin.take() {
                let party_index = selected_party_index(runtime_shell)?;
                runtime_shell.party_move_cursor = Some(MenuCursor {
                    surface_id: party_move_reorder_surface_id(party_index),
                    option_index: origin,
                });
                set_shell_action_status(runtime_shell, "MOVE WHERE?");
            } else {
                record_visible_runtime_action(runtime_shell, "party:move_reorder:close")?;
                close_visible_party_move_reorder(runtime_shell);
                set_shell_action_status(runtime_shell, "POKEMON");
            }
            return Ok(());
        }
        if runtime_shell.party_give_take_cursor.is_some() {
            runtime_shell.party_give_take_cursor = None;
            set_shell_action_status(runtime_shell, "POKEMON");
            return Ok(());
        }
        if runtime_shell.party_summary_open {
            if !visible_wait_sfx_finished(runtime_shell) {
                return Ok(());
            }
            record_visible_runtime_action(runtime_shell, "party:summary:close")?;
            close_visible_party_summary(runtime_shell);
            continue_visible_script_after_prompt(runtime_shell)?;
            return Ok(());
        }
        if runtime_shell.fly_cursor.is_some() {
            record_visible_runtime_action(runtime_shell, "party:fly:close")?;
            runtime_shell.fly_cursor = None;
            runtime_shell
                .last_audio_events
                .push("closed Fly destinations".to_string());
            trim_event_log(&mut runtime_shell.last_audio_events);
            continue_visible_script_after_prompt(runtime_shell)?;
            return Ok(());
        }
        if runtime_shell.party_switch_cursor.is_some() {
            record_visible_runtime_action(runtime_shell, "party:switch:close")?;
            runtime_shell.party_switch_cursor = None;
            runtime_shell
                .last_audio_events
                .push("closed party switch".to_string());
            trim_event_log(&mut runtime_shell.last_audio_events);
            continue_visible_script_after_prompt(runtime_shell)?;
            return Ok(());
        }
        if runtime_shell.party_action_cursor.is_some() {
            record_visible_runtime_action(runtime_shell, "party:actions:close")?;
            close_visible_party_action_menu(runtime_shell);
            continue_visible_script_after_prompt(runtime_shell)?;
            return Ok(());
        }
        record_visible_runtime_action(runtime_shell, "party:close")?;
        exit_visible_party_menu(runtime_shell);
        continue_visible_script_after_prompt(runtime_shell)?;
        return Ok(());
    }
    if runtime_shell.pokedex_menu_open {
        if runtime_shell.pokedex_detail_open {
            record_visible_runtime_action(runtime_shell, "pokedex:detail:close")?;
            if runtime_shell.pokedex_scripted_entry {
                let snapshot = runtime_shell.shell.presentation_snapshot()?;
                let species =
                    selected_pokedex_catalog_species(&snapshot, runtime_shell.pokedex_cursor)?;
                let entry = snapshot
                    .presentation
                    .pokedex_entries
                    .get(&species.species_id)
                    .with_context(|| {
                        format!("compiled pack missing Pokedex entry {}", species.species_id)
                    })?;
                let page_count = entry.pages.len();
                anyhow::ensure!(
                    page_count > 0,
                    "compiled Pokedex entry {} has no pages",
                    species.species_id
                );
                anyhow::ensure!(
                    runtime_shell.pokedex_detail_page < page_count,
                    "Pokedex detail page {} is outside {page_count} pages for {}",
                    runtime_shell.pokedex_detail_page,
                    species.species_id
                );
                if runtime_shell.pokedex_detail_page + 1 < page_count {
                    runtime_shell.pokedex_detail_page += 1;
                    let page_number = runtime_shell.pokedex_detail_page + 1;
                    record_visible_runtime_action(
                        runtime_shell,
                        format!("pokedex:scripted_entry:page:{page_number}:b"),
                    )?;
                    mark_runtime_snapshot_dirty(runtime_shell);
                    return Ok(());
                }
                close_visible_pokedex_menu(runtime_shell);
                if runtime_shell.pending_standard_capture.is_some() {
                    continue_visible_capture_after_owned_surface(runtime_shell)?;
                    return Ok(());
                }
                continue_visible_script_after_prompt(runtime_shell)?;
                return Ok(());
            }
            close_visible_pokedex_detail(runtime_shell);
            continue_visible_script_after_prompt(runtime_shell)?;
            return Ok(());
        }
        record_visible_runtime_action(runtime_shell, "pokedex:close")?;
        let return_to_start = runtime_shell.pokedex_controls.return_to_start;
        close_visible_pokedex_menu(runtime_shell);
        queue_visible_shell_sound_effect(runtime_shell, "SFX_READ_TEXT_2")?;
        if return_to_start {
            select_visible_start_menu_option_exact(runtime_shell, StartMenuOption::Pokedex)?;
            set_shell_action_status(runtime_shell, "START MENU");
        } else { continue_visible_script_after_prompt(runtime_shell)?; }
        return Ok(());
    }
    if runtime_shell.options_menu_open {
        record_visible_runtime_action(runtime_shell, "options:close")?;
        close_visible_options_menu(runtime_shell);
        continue_visible_script_after_prompt(runtime_shell)?;
        return Ok(());
    }
    if runtime_shell.trainer_card_open {
        record_visible_runtime_action(runtime_shell, "trainer_card:close")?;
        close_visible_trainer_card(runtime_shell);
        continue_visible_script_after_prompt(runtime_shell)?;
        return Ok(());
    }
    if runtime_shell.save_menu_open {
        return cancel_visible_save_menu(runtime_shell);
    }
    if runtime_shell.special_boundary.is_some() {
        close_visible_special_boundary(runtime_shell)?;
        return Ok(());
    }
    if runtime_shell.start_menu_cursor.is_some() {
        record_visible_runtime_action(runtime_shell, "start_menu:close")?;
        close_visible_start_menu(runtime_shell);
        continue_visible_script_after_prompt(runtime_shell)?;
        return Ok(());
    }
    if runtime_shell.bill_pc_action_cursor.is_some() {
        return close_visible_bill_pc_actions(runtime_shell);
    }
    if runtime_shell.mailbox_action_cursor.is_some() {
        runtime_shell.mailbox_action_cursor = None;
        return Ok(());
    }
    if runtime_shell.mailbox_cursor.is_some() {
        close_visible_mailbox(runtime_shell);
        return Ok(());
    }
    if let Some(menu) = runtime_shell.decoration_menu.as_ref() {
        let phase = menu.phase.clone();
        return match phase {
            VisibleDecorationMenuPhase::Categories { .. } => {
                close_visible_decoration_menu(runtime_shell)
            }
            VisibleDecorationMenuPhase::Decorations { category, .. } => {
                return_visible_decoration_to_categories(runtime_shell, category)
            }
            VisibleDecorationMenuPhase::Side {
                category,
                item_cursor_index,
                ..
            } => return_visible_decoration_to_items(runtime_shell, category, item_cursor_index),
        };
    }
    if runtime_shell.player_pc_action_cursor.is_some() {
        return close_visible_player_pc(runtime_shell);
    }
    if runtime_shell.pc_item_cursor.is_some() && runtime_shell.pc_item_action.is_some() {
        if runtime_shell.pc_item_switch_origin.take().is_some() {
            mark_runtime_presentation_dirty(runtime_shell);
            return Ok(());
        }
        close_visible_pc_item_list(runtime_shell);
        return Ok(());
    }
    if runtime_shell.pc_item_action == Some(VisiblePlayerPcAction::DepositItem)
        && visible_field_pack_is_open(runtime_shell)
    {
        close_visible_pc_item_deposit_pack(runtime_shell);
        return Ok(());
    }
    if runtime_shell.bill_pc_box_action_cursor.take().is_some() {
        set_shell_action_status(runtime_shell, "CHOOSE A BOX");
        return Ok(());
    }
    if runtime_shell.bill_pc_box_cursor.is_some() {
        runtime_shell.bill_pc_box_cursor = None;
        runtime_shell.bill_pc_action_cursor = Some(MenuCursor {
            surface_id: "pc:bill-actions".to_string(),
            option_index: 2,
        });
        set_shell_action_status(runtime_shell, "BILL'S PC");
        return Ok(());
    }
    if runtime_shell.pc_hub_cursor.is_some() {
        return turn_off_visible_pc_hub(runtime_shell);
    }
    if runtime_shell.storage_cursor.is_some() || runtime_shell.pc_item_cursor.is_some() {
        close_visible_pc_surface(runtime_shell)?;
        return Ok(());
    }
    if snapshot.battle.is_some() && runtime_shell.battle_pack_target_mode.is_some() {
        close_visible_battle_pack_target(runtime_shell)?;
        return Ok(());
    }
    if snapshot.battle.is_some() && runtime_shell.field_pack_action_cursor.is_some() {
        record_visible_runtime_action(runtime_shell, "battle:pack:actions:close")?;
        close_visible_field_pack_action_menu(runtime_shell);
        return Ok(());
    }
    if snapshot.battle.is_some()
        && (runtime_shell.ball_cursor.is_some()
            || runtime_shell.bag_cursor.is_some()
            || runtime_shell.key_item_cursor.is_some()
            || runtime_shell.tmhm_cursor.is_some())
    {
        record_visible_runtime_action(runtime_shell, "battle:item_menu:close")?;
        reset_visible_battle_item_cursors(runtime_shell);
        runtime_shell
            .last_audio_events
            .push("closed battle item cursor".to_string());
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    if snapshot.battle.is_some()
        && (runtime_shell.battle_move_cursor.is_some()
            || runtime_shell.battle_switch_cursor.is_some())
    {
        if runtime_shell.battle_party_summary_open
            || runtime_shell.battle_party_action_cursor.is_some()
        {
            return press_visible_battle_b_button(runtime_shell);
        }
        // MoveSelectionScreen returns through ParsePlayerAction's
        // PlayClickSFX even when B canceled it. The party menu likewise owns
        // the ordinary menu-button click. Keep the disabled B input on the
        // main battle command grid silent by limiting this to open submenus.
        queue_visible_shell_sound_effect(runtime_shell, "SFX_READ_TEXT_2")?;
        record_visible_runtime_action(runtime_shell, "battle:submenu:reset")?;
        runtime_shell.battle_move_cursor = None;
        runtime_shell.battle_move_swap_origin = None;
        runtime_shell.battle_shift_prompt_cursor = None;
        runtime_shell.battle_faint_prompt_cursor = None;
        runtime_shell.battle_switch_cursor = None;
        runtime_shell
            .last_audio_events
            .push("reset battle action cursor".to_string());
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    if snapshot.battle.is_some() {
        if runtime_shell.battle_shift_prompt_cursor.is_some() {
            return resolve_visible_trainer_shift_prompt(runtime_shell, false);
        }
        return press_visible_battle_b_button(runtime_shell);
    }
    Ok(())
}

fn confirm_visible_day_of_week(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let Some(prompt) = runtime_shell.pending_day_of_week.as_mut() else {
        return Ok(());
    };
    if !prompt.confirming {
        prompt.confirming = true;
        prompt.yes_no_index = 0;
        set_shell_action_status(runtime_shell, "IS IT?");
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(());
    }
    if prompt.yes_no_index != 0 {
        prompt.confirming = false;
        prompt.yes_no_index = 0;
        set_shell_action_status(runtime_shell, "WHAT DAY IS IT?");
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(());
    }
    let prompt = runtime_shell
        .pending_day_of_week
        .clone()
        .context("weekday prompt disappeared before confirmation")?;
    runtime_shell
        .shell
        .set_script_runtime_variable("wTempDayOfWeek", prompt.selected_day.to_string())?;
    record_visible_runtime_action(
        runtime_shell,
        format!(
            "ui:day_of_week:{}:{}:{}",
            prompt.source_script, prompt.command_index, prompt.selected_day
        ),
    )?;
    let runtime_inputs = explicit_compiled_script_runtime_inputs(
        runtime_shell,
        &prompt.source_script,
        prompt.command_index,
    )?;
    let phone_inputs = explicit_compiled_script_phone_inputs(
        runtime_shell,
        &prompt.source_script,
        prompt.command_index,
    );
    let stepped = runtime_shell.shell.step_compiled_script_command(
        &prompt.origin_map_name,
        &prompt.source_script,
        prompt.command_index,
        runtime_inputs,
        phone_inputs,
    )?;
    integrate_visible_script_mutation_outcome(runtime_shell, &stepped.mutation)?;
    runtime_shell.pending_day_of_week = None;
    trim_event_log(&mut runtime_shell.last_audio_events);
    if activate_visible_script_boundary_after_outcome(runtime_shell, &stepped.mutation)? {
        arm_visible_active_script_cursor_from_run(runtime_shell, stepped.next_cursor);
        return Ok(());
    }
    arm_visible_script_cursor_after_step(runtime_shell, &stepped);
    continue_visible_script_after_prompt(runtime_shell)
}

fn finish_visible_mom_bank(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    runtime_shell.visible_mom_bank = None;
    mark_runtime_snapshot_dirty(runtime_shell);
    continue_visible_script_after_prompt(runtime_shell)
}

fn queue_visible_mom_bank_messages(
    runtime_shell: &mut BevyRuntimeShell,
    messages: &[&str],
    close_after: bool,
) {
    if let Some(bank) = runtime_shell.visible_mom_bank.as_mut() {
        bank.messages = messages
            .iter()
            .map(|message| (*message).to_string())
            .collect();
        bank.close_after_messages = close_after;
    }
    mark_runtime_snapshot_dirty(runtime_shell);
}

fn confirm_visible_mom_bank(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let Some(bank) = runtime_shell.visible_mom_bank.as_mut() else {
        return Ok(());
    };
    if !bank.messages.is_empty() {
        bank.messages.pop_front();
        let finished = bank.messages.is_empty() && bank.close_after_messages;
        mark_runtime_snapshot_dirty(runtime_shell);
        return if finished {
            finish_visible_mom_bank(runtime_shell)
        } else {
            Ok(())
        };
    }
    let phase = bank.phase;
    let accepted = bank.yes_no_index == 0;
    let menu_index = bank.menu_index;
    let amount = bank.amount;
    record_visible_runtime_action(runtime_shell, format!("mom_bank:{phase:?}:confirm"))?;
    match phase {
        VisibleMomBankPhase::InitializeQuestion => {
            runtime_shell
                .shell
                .session_mut()
                .state_mut()
                .mom_saving_some_money = accepted;
            queue_visible_mom_bank_messages(
                runtime_shell,
                if accepted {
                    &[
                        "OK, I'll take care of your money.",
                        "Be careful.\nNow, go on!",
                    ]
                } else {
                    &["Be careful.\nNow, go on!"]
                },
                true,
            );
        }
        VisibleMomBankPhase::AccessQuestion => {
            if accepted {
                let bank = runtime_shell.visible_mom_bank.as_mut().unwrap();
                bank.phase = VisibleMomBankPhase::Menu;
                bank.menu_index = 0;
                mark_runtime_snapshot_dirty(runtime_shell);
            } else {
                queue_visible_mom_bank_messages(runtime_shell, &["Just do what you can."], true);
            }
        }
        VisibleMomBankPhase::Menu => match menu_index {
            0 => {
                let bank = runtime_shell.visible_mom_bank.as_mut().unwrap();
                bank.phase = VisibleMomBankPhase::Withdraw;
                bank.amount = 0;
                bank.digit = 5;
                queue_visible_mom_bank_messages(
                    runtime_shell,
                    &["How much do you want to take?"],
                    false,
                );
            }
            1 => {
                let bank = runtime_shell.visible_mom_bank.as_mut().unwrap();
                bank.phase = VisibleMomBankPhase::Deposit;
                bank.amount = 0;
                bank.digit = 5;
                queue_visible_mom_bank_messages(
                    runtime_shell,
                    &["How much do you want to save?"],
                    false,
                );
            }
            2 => {
                let bank = runtime_shell.visible_mom_bank.as_mut().unwrap();
                bank.phase = VisibleMomBankPhase::ChangeQuestion;
                bank.yes_no_index = 0;
                mark_runtime_snapshot_dirty(runtime_shell);
            }
            _ => queue_visible_mom_bank_messages(runtime_shell, &["Just do what you can."], true),
        },
        VisibleMomBankPhase::Withdraw | VisibleMomBankPhase::Deposit => {
            if amount == 0 {
                queue_visible_mom_bank_messages(runtime_shell, &["Just do what you can."], true);
                return Ok(());
            }
            const MAX_MONEY: u32 = 999_999;
            let state = runtime_shell.shell.session_mut().state_mut();
            let (available, destination) = if phase == VisibleMomBankPhase::Withdraw {
                (state.moms_money, state.money)
            } else {
                (state.money, state.moms_money)
            };
            if available < amount {
                queue_visible_mom_bank_messages(
                    runtime_shell,
                    if phase == VisibleMomBankPhase::Withdraw {
                        &["You haven't saved that much."]
                    } else {
                        &["You don't have that much."]
                    },
                    false,
                );
                return Ok(());
            }
            if destination > MAX_MONEY - amount {
                queue_visible_mom_bank_messages(
                    runtime_shell,
                    if phase == VisibleMomBankPhase::Withdraw {
                        &["You can't take that much."]
                    } else {
                        &["You can't save that much."]
                    },
                    false,
                );
                return Ok(());
            }
            if phase == VisibleMomBankPhase::Withdraw {
                state.moms_money -= amount;
                state.money += amount;
                queue_visible_mom_bank_messages(runtime_shell, &["Here you go!"], true);
            } else {
                state.money -= amount;
                state.moms_money += amount;
                queue_visible_mom_bank_messages(
                    runtime_shell,
                    &["OK, I'll save your money."],
                    true,
                );
            }
        }
        VisibleMomBankPhase::ChangeQuestion => {
            runtime_shell
                .shell
                .session_mut()
                .state_mut()
                .mom_saving_some_money = accepted;
            queue_visible_mom_bank_messages(
                runtime_shell,
                if accepted {
                    &["OK, I'll save your money."]
                } else {
                    &["Just do what you can."]
                },
                true,
            );
        }
    }
    Ok(())
}

fn cancel_visible_mom_bank(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let Some(bank) = runtime_shell.visible_mom_bank.as_ref() else {
        return Ok(());
    };
    if !bank.messages.is_empty() {
        return confirm_visible_mom_bank(runtime_shell);
    }
    match bank.phase {
        VisibleMomBankPhase::InitializeQuestion
        | VisibleMomBankPhase::AccessQuestion
        | VisibleMomBankPhase::ChangeQuestion => {
            runtime_shell
                .visible_mom_bank
                .as_mut()
                .unwrap()
                .yes_no_index = 1;
            confirm_visible_mom_bank(runtime_shell)
        }
        VisibleMomBankPhase::Menu
        | VisibleMomBankPhase::Withdraw
        | VisibleMomBankPhase::Deposit => {
            queue_visible_mom_bank_messages(runtime_shell, &["Just do what you can."], true);
            Ok(())
        }
    }
}

fn move_visible_mom_bank(runtime_shell: &mut BevyRuntimeShell, delta: isize, horizontal: bool) {
    let Some(bank) = runtime_shell.visible_mom_bank.as_mut() else {
        return;
    };
    if !bank.messages.is_empty() {
        return;
    }
    match bank.phase {
        VisibleMomBankPhase::InitializeQuestion
        | VisibleMomBankPhase::AccessQuestion
        | VisibleMomBankPhase::ChangeQuestion => {
            bank.yes_no_index = 1 - bank.yes_no_index.min(1);
        }
        VisibleMomBankPhase::Menu => {
            bank.menu_index = wrapped_index(bank.menu_index, 4, delta);
        }
        VisibleMomBankPhase::Withdraw | VisibleMomBankPhase::Deposit => {
            if horizontal {
                bank.digit = (i16::from(bank.digit) + delta.signum() as i16).clamp(0, 5) as u8;
            } else {
                let place = 10_u32.pow(u32::from(5 - bank.digit));
                if delta < 0 {
                    bank.amount = bank.amount.saturating_add(place).min(999_999);
                } else {
                    bank.amount = bank.amount.saturating_sub(place);
                }
            }
        }
    }
    mark_runtime_snapshot_dirty(runtime_shell);
}

fn press_visible_select_button(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    if runtime_shell.pokedex_menu_open { return press_visible_pokedex_select(runtime_shell); }
    if runtime_shell.pokegear_exit.is_some() { return Ok(()); }
    if runtime_shell
        .special_boundary
        .as_ref()
        .is_some_and(|boundary| boundary.label == "PrinterError2")
    {
        record_visible_runtime_action(runtime_shell, "printer:error:select_ignored")?;
        return Ok(());
    }
    if runtime_shell.bill_pc_move_save.is_some()
        || runtime_shell.pc_release_sequence.is_some()
        || runtime_shell.pc_transfer_sequence.is_some()
        || runtime_shell.pc_item_move_sequence.is_some()
    {
        return Ok(());
    }
    if !runtime_shell.battle_messages.is_empty()
        || runtime_shell
            .battle_exp_tween
            .as_ref()
            .is_some_and(|tween| tween.started)
        || runtime_shell
            .battle_level_stats
            .front()
            .is_some_and(|stats| stats.active)
    {
        return Ok(());
    }
    if runtime_shell.pc_notice.is_some() {
        return Ok(());
    }
    if runtime_shell.visible_heal_machine.is_some()
        || runtime_shell.visible_magnet_train.is_some()
        || runtime_shell.visible_unown_words.is_some()
        || runtime_shell.visible_diploma.is_some()
    {
        return Ok(());
    }
    if runtime_shell.visible_slot_machine.is_some() || runtime_shell.visible_card_flip.is_some() {
        return Ok(());
    }
    if runtime_shell.intro_screen.is_some() {
        return skip_visible_intro_screen(runtime_shell, GameButton::Select);
    }
    if runtime_shell.credits_screen.is_some() {
        record_visible_runtime_action(runtime_shell, "credits:select:ignored")?;
        runtime_shell
            .last_audio_events
            .push("credits Select ignored".to_string());
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    if runtime_shell.pending_delete_save.is_some() || runtime_shell.pending_clock_reset.is_some() {
        record_visible_runtime_action(runtime_shell, "boot_prompt:select:ignored")?;
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    if runtime_shell.pending_time_set.is_some() {
        record_visible_runtime_action(runtime_shell, "time_set:select:ignored")?;
        runtime_shell
            .last_audio_events
            .push("time set Select ignored".to_string());
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    if runtime_shell.pending_oak_intro.is_some() {
        record_visible_runtime_action(runtime_shell, "oak_intro:select:ignored")?;
        runtime_shell
            .last_audio_events
            .push("oak intro Select ignored".to_string());
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    if runtime_shell.pending_gender_selection.is_some() {
        record_visible_runtime_action(runtime_shell, "gender:select:ignored")?;
        runtime_shell
            .last_audio_events
            .push("gender Select ignored".to_string());
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    if runtime_shell.hall_of_fame_pc_index.is_some() {
        record_visible_runtime_action(runtime_shell, "pc:hall_of_fame:select:ignored")?;
        return Ok(());
    }
    if runtime_shell.special_boundary.is_some() {
        return close_visible_special_boundary(runtime_shell);
    }
    if runtime_shell.field_notice.is_some() {
        record_visible_runtime_action(runtime_shell, "field:notice:select:ignored")?;
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    if runtime_shell.pack_toss.is_some() {
        record_visible_runtime_action(runtime_shell, "pack:toss:select_ignored")?;
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    if runtime_shell.held_item_swap_prompt {
        record_visible_runtime_action(runtime_shell, "party:held_item:swap:select_ignored")?;
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    if runtime_shell.pending_pc_release.is_some() {
        record_visible_runtime_action(runtime_shell, "pc:release-confirm:select:ignored")?;
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    if runtime_shell
        .shell
        .presentation_snapshot()?
        .battle
        .is_some()
    {
        if runtime_shell.battle_move_cursor.is_some() {
            return select_visible_battle_move_swap(runtime_shell);
        }
        record_visible_runtime_action(runtime_shell, "battle:select:ignored")?;
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    if runtime_shell.pokegear_menu_open {
        if runtime_shell.pokegear_page == PokegearPage::Clock {
            return request_visible_pokegear_exit(runtime_shell);
        }
        return Ok(());
    }
    if runtime_shell.storage_cursor.is_some() {
        record_visible_runtime_action(runtime_shell, "pc:box:select:ignored")?;
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    if runtime_shell.pc_item_cursor.is_some() {
        return switch_visible_pc_item(runtime_shell);
    }
    if visible_field_pack_is_open(runtime_shell) {
        return switch_visible_pack_item(runtime_shell);
    }
    if runtime_shell.tmhm_cursor.is_some() {
        return open_visible_tmhm_teach_prompt(runtime_shell);
    }
    let snapshot = runtime_shell.shell.presentation_snapshot()?;
    let Some(item_id) = snapshot.progression.registered_key_item.clone() else {
        record_visible_runtime_action(runtime_shell, "pack:key_item:select:none_registered")?;
        retain_visible_field_notice_scene(runtime_shell, &snapshot);
        runtime_shell.field_notice = Some(visible_asm_text(&snapshot, "_MayRegisterItemText")?);
        mark_runtime_snapshot_dirty(runtime_shell);
        set_shell_action_status(runtime_shell, "NO REGISTERED ITEM");
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    };
    if !snapshot
        .bag
        .key_items
        .iter()
        .any(|item| item.item_id == item_id && item.quantity > 0)
    {
        record_visible_runtime_action(
            runtime_shell,
            format!("pack:key_item:select:{item_id}:not_carried"),
        )?;
        runtime_shell
            .last_audio_events
            .push(format!("registered key item {item_id} is not carried"));
        set_shell_action_status(runtime_shell, format!("{item_id} NOT IN BAG"));
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    record_visible_runtime_action(runtime_shell, format!("pack:key_item:select:{item_id}"))?;
    use_visible_field_bag_item_by_id(runtime_shell, item_id)
}

fn switch_visible_pack_item(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let pocket = active_visible_field_pack_pocket(runtime_shell);
    let snapshot = runtime_shell.shell.snapshot()?;
    let pocket_len = match &pocket {
        FieldPackPocket::Items => carried_item_count(&snapshot.bag.items),
        FieldPackPocket::Balls => carried_item_count(&snapshot.bag.balls),
        FieldPackPocket::KeyItems => carried_item_count(&snapshot.bag.key_items),
        FieldPackPocket::TmHm => snapshot.bag.tm_hm.len(),
        FieldPackPocket::Custom(pocket_id) => snapshot
            .bag
            .custom_pockets
            .get(pocket_id)
            .map_or(0, |items| carried_item_count(items)),
    };
    let (cursor, pocket_id) = match &pocket {
        FieldPackPocket::Items => (
            runtime_shell.bag_cursor.as_ref(),
            crate::core::models::ITEM_POCKET_ITEM,
        ),
        FieldPackPocket::Balls => (
            runtime_shell.ball_cursor.as_ref(),
            crate::core::models::ITEM_POCKET_BALL,
        ),
        FieldPackPocket::KeyItems => (
            runtime_shell.key_item_cursor.as_ref(),
            crate::core::models::ITEM_POCKET_KEY_ITEM,
        ),
        FieldPackPocket::TmHm | FieldPackPocket::Custom(_) => {
            record_visible_runtime_action(runtime_shell, "pack:item_switch:unavailable")?;
            return Ok(());
        }
    };
    let selected = cursor
        .context("Pack item switching requires a pocket cursor")?
        .option_index;
    if selected >= pocket_len {
        runtime_shell.pack_item_switch_origin = None;
        set_shell_action_status(runtime_shell, "MOVE CANCELLED");
        return Ok(());
    }
    let Some((origin_pocket, origin)) = runtime_shell.pack_item_switch_origin.take() else {
        runtime_shell.pack_item_switch_origin = Some((pocket, selected));
        set_shell_action_status(runtime_shell, "MOVE ITEM WHERE?");
        return Ok(());
    };
    if origin_pocket != pocket {
        runtime_shell.pack_item_switch_origin = Some((pocket, selected));
        set_shell_action_status(runtime_shell, "MOVE ITEM WHERE?");
        return Ok(());
    }
    let target = runtime_shell
        .shell
        .switch_bag_item_stacks(pocket_id, origin, selected)?;
    match pocket {
        FieldPackPocket::Items => runtime_shell.bag_cursor.as_mut().unwrap().option_index = target,
        FieldPackPocket::Balls => runtime_shell.ball_cursor.as_mut().unwrap().option_index = target,
        FieldPackPocket::KeyItems => {
            runtime_shell.key_item_cursor.as_mut().unwrap().option_index = target
        }
        FieldPackPocket::TmHm | FieldPackPocket::Custom(_) => unreachable!(),
    }
    mark_runtime_snapshot_dirty(runtime_shell);
    set_shell_action_status(runtime_shell, "ITEM MOVED");
    Ok(())
}

fn press_visible_start_button(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    if runtime_shell.pokedex_menu_open { return press_visible_pokedex_start(runtime_shell); }
    if let Some(puzzle) = runtime_shell.visible_unown_puzzle.as_ref() {
        return if puzzle.solved { Ok(()) } else { close_visible_unown_puzzle(runtime_shell) };
    }
    if runtime_shell.pokegear_exit.is_some() { return Ok(()); }
    if runtime_shell.bill_pc_move_save.is_some()
        || runtime_shell.pc_release_sequence.is_some()
        || runtime_shell.pc_transfer_sequence.is_some()
        || runtime_shell.pc_item_move_sequence.is_some()
    {
        return Ok(());
    }
    if runtime_shell
        .battle_exp_tween
        .as_ref()
        .is_some_and(|tween| tween.started)
        || runtime_shell
            .battle_level_stats
            .front()
            .is_some_and(|stats| stats.active)
    {
        return Ok(());
    }
    if runtime_shell.visible_heal_machine.is_some()
        || runtime_shell.visible_magnet_train.is_some()
        || runtime_shell.visible_unown_words.is_some()
        || runtime_shell.visible_diploma.is_some()
    {
        return Ok(());
    }
    if runtime_shell.visible_slot_machine.is_some() || runtime_shell.visible_card_flip.is_some() {
        return Ok(());
    }
    if runtime_shell.pokegear_menu_open {
        if runtime_shell.pokegear_page == PokegearPage::Clock {
            return request_visible_pokegear_exit(runtime_shell);
        }
        return Ok(());
    }
    let snapshot = runtime_shell.shell.presentation_snapshot()?;
    if visible_script_or_dialogue_owns_start_input(runtime_shell, &snapshot) {
        // Text/script execution owns the complete joypad except for the
        // buttons explicitly read by its current ASM command. Start is never
        // a text acknowledgement or a way to pause a running interaction.
        return Ok(());
    }
    if !runtime_shell.battle_messages.is_empty() {
        // ASM PromptButton and PrintLetterDelay consume PAD_A/PAD_B only.
        // Start must not accelerate or dismiss battle dialogue.
        return Ok(());
    }
    if runtime_shell.hall_of_fame_pc_index.is_some() {
        return Ok(());
    }
    if runtime_shell.visible_mom_bank.is_some() {
        return Ok(());
    }
    if runtime_shell.intro_screen.is_some() {
        return skip_visible_intro_screen(runtime_shell, GameButton::Start);
    }
    if runtime_shell.pending_delete_save.is_some() {
        return confirm_visible_delete_save_screen(runtime_shell);
    }
    if runtime_shell.pending_clock_reset.is_some() {
        return confirm_visible_clock_reset_screen(runtime_shell);
    }
    if runtime_shell.pending_time_set.is_some() {
        return press_visible_time_set_a_button(runtime_shell);
    }
    if runtime_shell.pending_oak_intro.is_some() {
        return press_visible_oak_intro_a_button(runtime_shell);
    }
    if runtime_shell.pending_gender_selection.is_some() {
        return Ok(());
    }
    if runtime_shell.options_menu_open {
        record_visible_runtime_action(runtime_shell, "options:close:start")?;
        close_visible_options_menu(runtime_shell);
        continue_visible_script_after_prompt(runtime_shell)?;
        return Ok(());
    }
    if runtime_shell.field_notice.is_some()
        || runtime_shell.pc_notice.is_some()
        || runtime_shell.pack_toss.is_some()
        || runtime_shell.held_item_swap_prompt
    {
        record_visible_runtime_action(runtime_shell, "field:notice:start:ignored")?;
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    record_visible_runtime_action(runtime_shell, "input:Start")?;
    runtime_shell
        .last_audio_events
        .push("pressed Start".to_string());
    trim_event_log(&mut runtime_shell.last_audio_events);
    toggle_visible_start_menu(runtime_shell)
}

fn has_visible_shell_b_action(runtime_shell: &mut BevyRuntimeShell) -> bool {
    if runtime_shell.mailbox_cursor.is_some() || runtime_shell.mailbox_action_cursor.is_some() {
        return true;
    }
    if runtime_shell
        .pokegear_phone_call
        .as_ref()
        .is_some_and(|call| {
            matches!(
                call.phase,
                VisiblePokegearPhoneCallPhase::NoServicePrompt
                    | VisiblePokegearPhoneCallPhase::AwaitHangup
            )
        })
    {
        return true;
    }
    if runtime_shell.bill_pc_move_save.is_some()
        || runtime_shell.pc_release_sequence.is_some()
        || runtime_shell.pc_transfer_sequence.is_some()
        || runtime_shell.pc_item_move_sequence.is_some()
    {
        return true;
    }
    if runtime_shell.player_walk_frame_ticks > 0 {
        return false;
    }
    if !runtime_shell.battle_messages.is_empty()
        || runtime_shell
            .battle_exp_tween
            .as_ref()
            .is_some_and(|tween| tween.started)
        || runtime_shell
            .battle_level_stats
            .front()
            .is_some_and(|stats| stats.active)
    {
        return true;
    }
    if runtime_shell.field_notice.is_some() || runtime_shell.pc_notice.is_some() {
        return true;
    }
    if runtime_shell.visible_diploma.is_some() {
        return true;
    }
    if runtime_shell.visible_unown_words.is_some() {
        return true;
    }
    if runtime_shell.visible_heal_machine.is_some() || runtime_shell.visible_magnet_train.is_some()
    {
        return true;
    }
    if runtime_shell.visible_card_flip.is_some() {
        return true;
    }
    if runtime_shell.visible_slot_machine.is_some() {
        return true;
    }
    if runtime_shell.visible_unown_puzzle.is_some() {
        return true;
    }
    if runtime_shell.visible_unown_printer.is_some() {
        return true;
    }
    if runtime_shell.visible_mom_bank.is_some() {
        return true;
    }
    if runtime_shell.pending_day_of_week.is_some() {
        return true;
    }
    if runtime_shell.kurt_apricorn_cursor.is_some() {
        return true;
    }
    if runtime_shell.visible_buena_password.is_some() {
        return true;
    }
    if runtime_shell.visible_battle_tower_challenge_menu.is_some() {
        return true;
    }
    if runtime_shell.visible_battle_tower_room_menu.is_some() {
        return true;
    }
    if runtime_shell.buena_prize_cursor.is_some() {
        return true;
    }
    if runtime_shell.intro_screen.is_some() {
        return true;
    }
    if runtime_shell.credits_screen.is_some() {
        return true;
    }
    if runtime_shell.pending_delete_save.is_some() || runtime_shell.pending_clock_reset.is_some() {
        return true;
    }
    if runtime_shell.title_menu.is_some() {
        return true;
    }
    if runtime_shell.pending_time_set.is_some() {
        return true;
    }
    if runtime_shell.pending_oak_intro.is_some() {
        return true;
    }
    if runtime_shell.pending_gender_selection.is_some() {
        return true;
    }
    let ownership = cached_runtime_snapshot(runtime_shell).map(|snapshot| {
        snapshot.ui.pending_yes_no.is_some()
            || runtime_shell.pending_phone_prompt.is_some()
            || runtime_shell.pending_remember_password.is_some()
            || snapshot.ui.pending_text_wait.is_some()
            || snapshot.pending_move_learn.is_some()
            || snapshot.pending_shop.is_some()
            || snapshot.ui.text_window_open
            || snapshot.ui.window_open
            || visible_menu_has_selectable_options(&snapshot)
            || snapshot.ui.active_pokemon_picture.is_some()
            || runtime_shell.party_menu_open
            || runtime_shell.pokedex_menu_open
            || runtime_shell.pokegear_menu_open
            || runtime_shell.trainer_card_open
            || runtime_shell.options_menu_open
            || runtime_shell.save_menu_open
            || runtime_shell.special_boundary.is_some()
            || runtime_shell.start_menu_cursor.is_some()
            || runtime_shell.storage_cursor.is_some()
            || runtime_shell.pc_item_cursor.is_some()
            || (snapshot.battle.is_none() && visible_field_pack_is_open(runtime_shell))
            || (snapshot.battle.is_some()
                && (runtime_shell.ball_cursor.is_some()
                    || runtime_shell.bag_cursor.is_some()
                    || runtime_shell.key_item_cursor.is_some()
                    || runtime_shell.tmhm_cursor.is_some()))
            || (snapshot.battle.is_some()
                && (runtime_shell.battle_move_cursor.is_some()
                    || runtime_shell.battle_switch_cursor.is_some()))
            || snapshot.battle.is_some()
    });
    fail_closed_visible_input_ownership(runtime_shell, ownership)
}

fn has_visible_shell_select_action(runtime_shell: &mut BevyRuntimeShell) -> bool {
    if runtime_shell.bill_pc_move_save.is_some()
        || runtime_shell.pc_release_sequence.is_some()
        || runtime_shell.pc_transfer_sequence.is_some()
        || runtime_shell.pc_item_move_sequence.is_some()
    {
        return true;
    }
    if runtime_shell.player_walk_frame_ticks > 0 {
        return false;
    }
    if retained_text_surface_owns_gameplay_input(runtime_shell) {
        return true;
    }
    if runtime_shell.visible_heal_machine.is_some()
        || runtime_shell.visible_magnet_train.is_some()
        || runtime_shell.visible_unown_words.is_some()
        || runtime_shell.visible_diploma.is_some()
    {
        return true;
    }
    if runtime_shell.visible_unown_puzzle.is_some()
        || runtime_shell.visible_unown_printer.is_some()
        || runtime_shell.visible_slot_machine.is_some()
        || runtime_shell.visible_card_flip.is_some()
        || runtime_shell.kurt_apricorn_cursor.is_some()
        || runtime_shell.visible_buena_password.is_some()
        || runtime_shell.visible_battle_tower_challenge_menu.is_some()
        || runtime_shell.visible_battle_tower_room_menu.is_some()
        || runtime_shell.buena_prize_cursor.is_some()
    {
        return false;
    }
    if runtime_shell.pending_delete_save.is_some() || runtime_shell.pending_clock_reset.is_some() {
        return true;
    }
    if runtime_shell.pending_time_set.is_some() {
        return true;
    }
    if runtime_shell.pending_oak_intro.is_some() {
        return true;
    }
    if runtime_shell.pending_gender_selection.is_some() {
        return true;
    }
    if runtime_shell.special_boundary.is_some() {
        return true;
    }
    // PCItemsJoypad owns Select independently of its parent script window.
    if runtime_shell.pc_item_cursor.is_some() {
        return true;
    }
    let ownership = cached_runtime_snapshot(runtime_shell).map(|snapshot| {
        (snapshot.battle.is_some() && runtime_shell.battle_move_cursor.is_some())
            || (snapshot.battle.is_none()
                && snapshot.pending_shop.is_none()
                && !snapshot.ui.text_window_open
                && !snapshot.ui.window_open
                && snapshot.ui.menu.is_none()
                && snapshot.ui.active_pokemon_picture.is_none()
                && snapshot.ui.pending_yes_no.is_none()
                && runtime_shell.pending_phone_prompt.is_none()
                && runtime_shell.pending_remember_password.is_none()
                && snapshot.ui.pending_text_wait.is_none()
                && snapshot.pending_move_learn.is_none()
                && runtime_shell.elevator_cursor.is_none()
                && runtime_shell.special_boundary.is_none()
                && !has_visible_auto_script_action(runtime_shell, &snapshot)
                && (runtime_shell.pokegear_menu_open
                    || runtime_shell.storage_cursor.is_some()
                    || runtime_shell.bag_cursor.is_some()
                    || runtime_shell.ball_cursor.is_some()
                    || matches!(
                        runtime_shell.field_pack_pocket.as_ref(),
                        Some(FieldPackPocket::Custom(_))
                    )
                    || runtime_shell.key_item_cursor.is_some()
                    || !visible_field_pack_is_open(runtime_shell)))
    });
    fail_closed_visible_input_ownership(runtime_shell, ownership)
}

fn has_visible_shell_start_action(runtime_shell: &mut BevyRuntimeShell) -> bool {
    if runtime_shell.pokedex_menu_open {
        return true;
    }
    if runtime_shell.visible_unown_puzzle.is_some() {
        return true;
    }
    if runtime_shell.bill_pc_move_save.is_some()
        || runtime_shell.pc_release_sequence.is_some()
        || runtime_shell.pc_transfer_sequence.is_some()
        || runtime_shell.pc_item_move_sequence.is_some()
    {
        return true;
    }
    if runtime_shell.player_walk_frame_ticks > 0 {
        return false;
    }
    let script_ownership = cached_runtime_snapshot(runtime_shell)
        .map(|snapshot| visible_script_or_dialogue_owns_start_input(runtime_shell, &snapshot));
    if fail_closed_visible_input_ownership(runtime_shell, script_ownership) {
        // Consume the host key so it cannot fall through to overworld input;
        // press_visible_start_button deliberately ignores it.
        return true;
    }
    if retained_text_surface_owns_gameplay_input(runtime_shell) {
        return true;
    }
    if runtime_shell.visible_heal_machine.is_some()
        || runtime_shell.visible_magnet_train.is_some()
        || runtime_shell.visible_unown_words.is_some()
        || runtime_shell.visible_diploma.is_some()
    {
        return true;
    }
    if runtime_shell.hall_of_fame_pc_index.is_some() {
        return true;
    }
    if runtime_shell.visible_unown_puzzle.is_some()
        || runtime_shell.visible_unown_printer.is_some()
        || runtime_shell.visible_slot_machine.is_some()
        || runtime_shell.visible_card_flip.is_some()
        || runtime_shell.kurt_apricorn_cursor.is_some()
        || runtime_shell.visible_buena_password.is_some()
        || runtime_shell.visible_battle_tower_challenge_menu.is_some()
        || runtime_shell.visible_battle_tower_room_menu.is_some()
        || runtime_shell.buena_prize_cursor.is_some()
    {
        return false;
    }
    if runtime_shell.credits_screen.is_some() {
        return false;
    }
    if runtime_shell.pending_delete_save.is_some() || runtime_shell.pending_clock_reset.is_some() {
        return true;
    }
    if runtime_shell.title_menu.is_some() {
        return false;
    }
    if runtime_shell.pending_time_set.is_some() {
        return true;
    }
    if runtime_shell.pending_oak_intro.is_some() {
        return true;
    }
    if runtime_shell.pending_gender_selection.is_some() {
        return true;
    }
    if runtime_shell.special_boundary.is_some() {
        return false;
    }
    let ownership = cached_runtime_snapshot(runtime_shell).map(|snapshot| {
        runtime_shell.start_menu_cursor.is_some()
            || (snapshot.battle.is_none()
                && snapshot.pending_shop.is_none()
                && !snapshot.ui.text_window_open
                && !snapshot.ui.window_open
                && snapshot.ui.menu.is_none()
                && snapshot.ui.active_pokemon_picture.is_none()
                && snapshot.ui.pending_yes_no.is_none()
                && runtime_shell.pending_phone_prompt.is_none()
                && runtime_shell.pending_remember_password.is_none()
                && snapshot.ui.pending_text_wait.is_none()
                && snapshot.pending_move_learn.is_none()
                && runtime_shell.elevator_cursor.is_none()
                && !runtime_shell.party_menu_open
                && !runtime_shell.pokedex_menu_open
                && !runtime_shell.pokegear_menu_open
                && !runtime_shell.trainer_card_open
                && !runtime_shell.options_menu_open
                && !runtime_shell.save_menu_open
                && runtime_shell.special_boundary.is_none()
                && !visible_field_pack_is_open(runtime_shell)
                && !has_visible_auto_script_action(runtime_shell, &snapshot))
    });
    fail_closed_visible_input_ownership(runtime_shell, ownership)
}

fn visible_script_or_dialogue_owns_start_input(
    runtime_shell: &BevyRuntimeShell,
    snapshot: &RuntimeShellSnapshot,
) -> bool {
    runtime_shell.field_text_reveal.is_some()
        || runtime_shell.field_notice.is_some()
        || runtime_shell.pc_notice.is_some()
        || runtime_shell.pending_day_of_week.is_some()
        || runtime_shell.pending_phone_prompt.is_some()
        || runtime_shell.pending_remember_password.is_some()
        || runtime_shell.visible_wait_sfx_boundary
        || runtime_shell.visible_mom_bank.is_some()
        || runtime_shell.visible_script_delay_frames.is_some()
        || runtime_shell.visible_script_movement.is_some()
        || runtime_shell.visible_overworld_emote.is_some()
        || snapshot.ui.text_window_open
        || snapshot.ui.pending_text_wait.is_some()
        || snapshot.ui.pending_yes_no.is_some()
        || snapshot.script_events.pending_text_label.is_some()
        || has_visible_direction_blocking_script_work(runtime_shell, snapshot)
}

fn has_visible_shell_direction_action(runtime_shell: &mut BevyRuntimeShell) -> bool {
    if runtime_shell.mailbox_cursor.is_some() || runtime_shell.mailbox_action_cursor.is_some() {
        return true;
    }
    if retained_text_surface_owns_gameplay_input(runtime_shell) {
        return true;
    }
    if runtime_shell.visible_heal_machine.is_some()
        || runtime_shell.visible_magnet_train.is_some()
        || runtime_shell.visible_unown_words.is_some()
        || runtime_shell.visible_diploma.is_some()
    {
        return true;
    }
    if runtime_shell.hall_of_fame_pc_index.is_some() {
        return true;
    }
    if runtime_shell.visible_card_flip.is_some() {
        return true;
    }
    if runtime_shell.visible_slot_machine.is_some() {
        return true;
    }
    if runtime_shell.visible_unown_puzzle.is_some() {
        return true;
    }
    if runtime_shell.visible_unown_printer.is_some() {
        return true;
    }
    if runtime_shell.visible_mom_bank.is_some() {
        return true;
    }
    if runtime_shell.pending_day_of_week.is_some() {
        return true;
    }
    if runtime_shell.kurt_apricorn_cursor.is_some() {
        return true;
    }
    if runtime_shell.visible_buena_password.is_some() {
        return true;
    }
    if runtime_shell.visible_battle_tower_challenge_menu.is_some() {
        return true;
    }
    if runtime_shell.visible_battle_tower_room_menu.is_some() {
        return true;
    }
    if runtime_shell.buena_prize_cursor.is_some() {
        return true;
    }
    if runtime_shell.credits_screen.is_some() {
        return false;
    }
    if runtime_shell.pending_delete_save.is_some() || runtime_shell.pending_clock_reset.is_some() {
        return true;
    }
    if runtime_shell.title_menu.is_some() {
        return true;
    }
    if runtime_shell.pending_time_set.is_some() {
        return true;
    }
    if runtime_shell.pending_gender_selection.is_some() {
        return true;
    }
    let ownership = cached_runtime_snapshot(runtime_shell).map(|snapshot| {
        runtime_shell.start_menu_cursor.is_some()
            || snapshot.ui.pending_yes_no.is_some()
            || runtime_shell.pending_phone_prompt.is_some()
            || runtime_shell.pending_remember_password.is_some()
            || snapshot.ui.pending_text_wait.is_some()
            || snapshot.pending_move_learn.is_some()
            || snapshot.ui.text_window_open
            || snapshot.ui.active_pokemon_picture.is_some()
            || runtime_shell.party_menu_open
            || runtime_shell.pokedex_menu_open
            || runtime_shell.pokegear_menu_open
            || runtime_shell.trainer_card_open
            || runtime_shell.options_menu_open
            || runtime_shell.save_menu_open
            || runtime_shell.special_boundary.is_some()
            || runtime_shell.storage_cursor.is_some()
            || runtime_shell.pc_item_cursor.is_some()
            || visible_field_pack_is_open(runtime_shell)
            || snapshot.pending_shop.is_some()
            || has_visible_direction_blocking_script_work(runtime_shell, &snapshot)
            || visible_menu_has_selectable_options(&snapshot)
            || snapshot.battle.is_some()
            || runtime_shell.elevator_cursor.is_some()
    });
    fail_closed_visible_input_ownership(runtime_shell, ownership)
}

fn fail_closed_visible_input_ownership(
    runtime_shell: &mut BevyRuntimeShell,
    ownership: Result<bool>,
) -> bool {
    match ownership {
        Ok(owned) => owned,
        Err(error) => {
            record_visible_runtime_error(runtime_shell, &error);
            runtime_shell.last_error = Some(error.to_string());
            true
        }
    }
}

fn retained_text_surface_owns_gameplay_input(runtime_shell: &BevyRuntimeShell) -> bool {
    runtime_shell.field_notice.is_some()
        || runtime_shell.pc_notice.is_some()
        || !runtime_shell.battle_messages.is_empty()
        || runtime_shell
            .battle_exp_tween
            .as_ref()
            .is_some_and(|tween| tween.started)
        || runtime_shell
            .battle_level_stats
            .front()
            .is_some_and(|stats| stats.active)
}

fn has_visible_direction_blocking_script_work(
    runtime_shell: &BevyRuntimeShell,
    snapshot: &RuntimeShellSnapshot,
) -> bool {
    snapshot.script_events.pending_text_label.is_some()
        || snapshot.script_events.pending_map_load.is_some()
        || snapshot.script_events.pending_map_refresh.is_some()
        || snapshot.script_events.pending_music_fade.is_some()
        || snapshot.script_events.pending_screen_fade.is_some()
        || !snapshot.script_events.pending_delays.is_empty()
        || !snapshot.script_events.pending_earthquakes.is_empty()
        || !snapshot.script_events.pending_emotes.is_empty()
        || snapshot.script_events.pending_script_warp.is_some()
        || !snapshot.script_events.command_queue.is_empty()
        || snapshot.script_events.next_script.is_some()
        || snapshot.script_events.map_reentry_script.is_some()
        || !snapshot.script_events.deferred_scripts.is_empty()
        || snapshot.script_events.script_ended.is_some()
        || visible_auto_runtime_flag(snapshot).is_some()
        || runtime_shell.active_script_cursor.is_some()
        || runtime_shell.pokegear_phone_call.is_some()
        || runtime_shell.incoming_phone_sequence.is_some()
}

fn visible_field_shortcut_allowed(runtime_shell: &BevyRuntimeShell) -> Result<bool> {
    if runtime_shell.intro_screen.is_some()
        || runtime_shell.title_menu.is_some()
        || runtime_shell.pending_time_set.is_some()
        || runtime_shell.pending_oak_intro.is_some()
        || runtime_shell.pending_gender_selection.is_some()
        || runtime_shell.special_boundary.is_some()
        || runtime_shell.pokegear_phone_call.is_some()
        || runtime_shell.incoming_phone_sequence.is_some()
    {
        return Ok(false);
    }
    runtime_shell.shell.snapshot().map(|snapshot| {
        snapshot.battle.is_none()
            && snapshot.pending_shop.is_none()
            && !snapshot.ui.text_window_open
            && !snapshot.ui.window_open
            && snapshot.ui.menu.is_none()
            && snapshot.ui.active_pokemon_picture.is_none()
            && snapshot.ui.pending_yes_no.is_none()
            && runtime_shell.pending_phone_prompt.is_none()
            && runtime_shell.pending_remember_password.is_none()
            && snapshot.ui.pending_text_wait.is_none()
            && snapshot.pending_move_learn.is_none()
            && runtime_shell.elevator_cursor.is_none()
            && runtime_shell.start_menu_cursor.is_none()
            && !runtime_shell.party_menu_open
            && !runtime_shell.pokedex_menu_open
            && !runtime_shell.pokegear_menu_open
            && !runtime_shell.trainer_card_open
            && !runtime_shell.options_menu_open
            && !runtime_shell.save_menu_open
            && !visible_field_pack_is_open(runtime_shell)
            && runtime_shell.storage_cursor.is_none()
            && runtime_shell.pc_item_cursor.is_none()
            && !has_visible_auto_script_action(runtime_shell, &snapshot)
    })
}

fn advance_visible_pending_text_wait(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    // TextCommand_PROMPT_BUTTON plays this only after a fully printed page is
    // acknowledged. The earlier reveal-completion branch returns before this
    // function, so fast-forwarding text remains silent.
    let prompt_button = pending_text_wait_uses_prompt_button(runtime_shell);
    if prompt_button {
        queue_visible_shell_sound_effect(runtime_shell, "SFX_READ_TEXT_2")?;
    }
    record_visible_runtime_action(runtime_shell, "ui:text_wait:advance")?;
    if prompt_button {
        let advance = runtime_shell.shell.advance_pending_text_wait()?;
        runtime_shell
            .last_audio_events
            .push(format!("advanced text wait {:?}", advance.state_checksum));
    } else {
        let next_cursor = visible_active_compiled_script_cursor(runtime_shell);
        if let Some(cursor) = next_cursor {
            let advanced = runtime_shell
                .shell
                .advance_text_wait_and_run_compiled_script(
                    Some(cursor),
                    256,
                    ScriptRuntimeInputs::default(),
                    ScriptPhoneInputs::default(),
                )?;
            runtime_shell.last_audio_events.push(format!(
                "advanced text wait {:?} resumed_steps={}",
                advanced.wait.state_checksum,
                advanced.run.steps.len()
            ));
            let reached_boundary =
                integrate_visible_compiled_script_run(runtime_shell, &advanced.run.steps)?;
            arm_visible_active_script_cursor_from_run(runtime_shell, advanced.run.next_cursor);
            if reached_boundary {
                trim_event_log(&mut runtime_shell.last_audio_events);
                return Ok(());
            }
        } else {
            let advance = runtime_shell.shell.advance_pending_text_wait()?;
            runtime_shell
                .last_audio_events
                .push(format!("advanced text wait {:?}", advance.state_checksum));
        }
    }
    trim_event_log(&mut runtime_shell.last_audio_events);
    // `promptbutton` can lead directly into a source special that owns modal
    // input (notably NameRival in CopScript). Resume that opcode through the
    // shell's one-command executor so its visible UI opens before mutation.
    // Ordinary `waitbutton` keeps the established composed-run boundary.
    continue_visible_script_after_prompt(runtime_shell)?;
    Ok(())
}

fn confirm_visible_pending_yes_no(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let selected = strict_readonly_cursor_index(&runtime_shell.yes_no_cursor, "ui:yes-no", 2)
        .context("yes/no prompt is active without a valid cursor")?;
    resolve_visible_pending_yes_no(runtime_shell, selected == 0)
}

fn accept_visible_pending_yes_no(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    resolve_visible_pending_yes_no(runtime_shell, true)
}

fn decline_visible_pending_yes_no(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    resolve_visible_pending_yes_no(runtime_shell, false)
}

fn confirm_visible_phone_prompt(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let selected = strict_readonly_cursor_index(&runtime_shell.yes_no_cursor, "ui:phone-number", 2)
        .context("phone prompt is active without a valid cursor")?;
    resolve_visible_phone_prompt(runtime_shell, selected == 0)
}

fn decline_visible_phone_prompt(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    resolve_visible_phone_prompt(runtime_shell, false)
}

fn confirm_visible_remember_password_prompt(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let accepted =
        strict_readonly_cursor_index(&runtime_shell.yes_no_cursor, "script:remember-password", 2)
            .context("remember-password prompt is active without a valid cursor")?
            == 0;
    begin_closing_visible_remember_password_prompt(runtime_shell, accepted)
}

fn decline_visible_remember_password_prompt(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    begin_closing_visible_remember_password_prompt(runtime_shell, false)
}

fn begin_closing_visible_remember_password_prompt(
    runtime_shell: &mut BevyRuntimeShell,
    accepted: bool,
) -> Result<()> {
    let Some(prompt) = runtime_shell.pending_remember_password.as_mut() else {
        return Ok(());
    };
    if prompt.closing_frames.is_some() {
        return Ok(());
    }
    runtime_shell.yes_no_cursor = Some(MenuCursor {
        surface_id: "script:remember-password".to_string(),
        option_index: usize::from(!accepted),
    });
    // AskRememberPassword retains the selected VerticalMenu for 15 frames
    // before Buena_ExitMenu removes its window and returns to ScriptEvents.
    prompt.closing_frames = Some(15);
    record_visible_runtime_action(
        runtime_shell,
        format!("special:remember_password:select:{accepted}"),
    )?;
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(())
}

fn advance_visible_remember_password_prompt(runtime_shell: &mut BevyRuntimeShell) -> Result<bool> {
    let Some(prompt) = runtime_shell.pending_remember_password.as_mut() else {
        return Ok(false);
    };
    let Some(frames) = prompt.closing_frames else {
        return Ok(false);
    };
    if frames > 1 {
        prompt.closing_frames = Some(frames - 1);
        mark_runtime_presentation_dirty(runtime_shell);
        return Ok(true);
    }
    let accepted =
        strict_readonly_cursor_index(&runtime_shell.yes_no_cursor, "script:remember-password", 2)
            .context("closing remember-password prompt has no valid selection")?
            == 0;
    let used = runtime_shell
        .shell
        .ask_remember_password_special(accepted)?;
    anyhow::ensure!(
        matches!(
            used.outcome.effect,
            SpecialRoutineEffect::AskRememberPassword { remember } if remember == accepted
        ),
        "AskRememberPassword returned a different special effect"
    );
    runtime_shell.pending_remember_password = None;
    runtime_shell.yes_no_cursor = None;
    runtime_shell.last_audio_events.push(format!(
        "remember-password accepted={accepted} checksum={:?}",
        used.state_checksum
    ));
    trim_event_log(&mut runtime_shell.last_audio_events);
    mark_runtime_snapshot_dirty(runtime_shell);
    continue_visible_script_after_prompt(runtime_shell)?;
    Ok(true)
}

fn resolve_visible_phone_prompt(
    runtime_shell: &mut BevyRuntimeShell,
    accepted: bool,
) -> Result<()> {
    let Some(prompt) = runtime_shell.pending_phone_prompt.clone() else {
        record_visible_runtime_action(runtime_shell, "ui:phone_number:none_open")?;
        runtime_shell
            .last_audio_events
            .push("no pending phone prompt is open".to_string());
        set_shell_action_status(runtime_shell, "NO PHONE PROMPT");
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    };
    record_visible_runtime_action(
        runtime_shell,
        format!(
            "ui:phone_number:{}:{}:{}:{}",
            prompt.source_script, prompt.command_index, prompt.contact_id, accepted
        ),
    )?;
    runtime_shell.yes_no_cursor = Some(MenuCursor {
        surface_id: "ui:phone-number".to_string(),
        option_index: if accepted { 0 } else { 1 },
    });
    let runtime_inputs = explicit_compiled_script_runtime_inputs(
        runtime_shell,
        &prompt.source_script,
        prompt.command_index,
    )?;
    let resolved = runtime_shell
        .shell
        .resolve_phone_prompt_and_run_compiled_script(
            &prompt.source_script,
            prompt.command_index,
            runtime_inputs,
            accepted,
            256,
        )?;
    runtime_shell.last_audio_events.push(format!(
        "phone prompt contact={} accepted={} result={} resumed_steps={} checksum={:?}",
        prompt.contact_id,
        accepted,
        resolved.step.mutation.result.result_tag(),
        resolved.run.steps.len(),
        resolved.step.mutation.state_checksum
    ));
    integrate_visible_script_mutation_outcome(runtime_shell, &resolved.step.mutation)?;
    runtime_shell.pending_phone_prompt = None;
    runtime_shell.yes_no_cursor = None;
    trim_event_log(&mut runtime_shell.last_audio_events);
    if activate_visible_script_boundary_after_outcome(runtime_shell, &resolved.step.mutation)? {
        return Ok(());
    }
    let reached_boundary =
        integrate_visible_compiled_script_run(runtime_shell, &resolved.run.steps)?;
    arm_visible_active_script_cursor_from_run(runtime_shell, resolved.run.next_cursor);
    if reached_boundary {
        return Ok(());
    }
    continue_visible_script_after_prompt(runtime_shell)?;
    Ok(())
}

fn resolve_visible_pending_yes_no(
    runtime_shell: &mut BevyRuntimeShell,
    accepted: bool,
) -> Result<()> {
    if runtime_shell
        .shell
        .snapshot()?
        .bug_contest
        .pending_caught_mon
        .is_some()
    {
        let replacement = runtime_shell
            .visible_bug_contest_replacement
            .as_ref()
            .cloned()
            .context("Bug Contest replacement prompt has no visible comparison state")?;
        anyhow::ensure!(
            replacement.phase == VisibleBugContestReplacementPhase::StatsPrompt,
            "Bug Contest replacement decision arrived outside its stats prompt"
        );
        record_visible_runtime_action(
            runtime_shell,
            format!(
                "bug_contest:replace:{}",
                if accepted { "switch" } else { "keep" }
            ),
        )?;
        let resolved = runtime_shell
            .shell
            .resolve_bug_contest_caught_mon(accepted)?;
        runtime_shell.yes_no_cursor = None;
        runtime_shell.last_audio_events.push(format!(
            "Bug Contest replacement accepted={} effect={:?} checksum={:?}",
            accepted, resolved.outcome.effect, resolved.state_checksum
        ));
        set_shell_action_status(
            runtime_shell,
            if accepted {
                "BUG CONTEST SWITCHED"
            } else {
                "BUG CONTEST KEPT"
            },
        );
        if accepted {
            let candidate_name = crate::core::models::pokemon_species_display_name(
                &replacement.candidate.species.id,
            );
            let mut boundaries = visible_exported_special_text_boundaries_with_buffer(
                runtime_shell,
                "ContestCaughtMonText",
                "_ContestCaughtMonText",
                Some(&candidate_name),
            )?;
            let caught_text = boundaries
                .pop_front()
                .and_then(|boundary| boundary.details.into_iter().next())
                .context("Contest caught-mon text rendered no source page")?;
            anyhow::ensure!(
                boundaries.is_empty(),
                "Contest caught-mon text unexpectedly rendered multiple pages"
            );
            runtime_shell.field_notice = Some(caught_text);
            runtime_shell.field_notice_scene = None;
            runtime_shell.field_text_reveal = None;
            runtime_shell
                .visible_bug_contest_replacement
                .as_mut()
                .expect("checked Contest replacement")
                .phase = VisibleBugContestReplacementPhase::CaughtText;
            mark_runtime_snapshot_dirty(runtime_shell);
        } else {
            let replacement = runtime_shell
                .visible_bug_contest_replacement
                .take()
                .expect("checked Contest replacement");
            finish_visible_wild_battle_exit(
                runtime_shell,
                replacement.scripted_static_wild,
                "bug_contest_capture_kept",
            )?;
        }
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    record_visible_runtime_action(
        runtime_shell,
        format!("ui:yes_no:{}", if accepted { "yes" } else { "no" }),
    )?;
    let next_cursor = visible_active_compiled_script_cursor(runtime_shell);
    runtime_shell.yes_no_cursor = None;
    if let Some(cursor) = next_cursor {
        let resolved = runtime_shell.shell.resolve_yes_no_and_run_compiled_script(
            accepted,
            Some(cursor),
            256,
            ScriptRuntimeInputs::default(),
            ScriptPhoneInputs::default(),
        )?;
        runtime_shell.last_audio_events.push(format!(
            "yes/no accepted={} script_value={} resumed_steps={} checksum={:?}",
            resolved.resolution.accepted,
            resolved.resolution.script_value,
            resolved.run.steps.len(),
            resolved.resolution.state_checksum
        ));
        let reached_boundary =
            integrate_visible_compiled_script_run(runtime_shell, &resolved.run.steps)?;
        arm_visible_active_script_cursor_from_run(runtime_shell, resolved.run.next_cursor);
        if reached_boundary {
            trim_event_log(&mut runtime_shell.last_audio_events);
            return Ok(());
        }
    } else {
        let resolution = runtime_shell.shell.resolve_pending_yes_no(accepted)?;
        runtime_shell.last_audio_events.push(format!(
            "yes/no accepted={} script_value={} checksum={:?}",
            resolution.accepted, resolution.script_value, resolution.state_checksum
        ));
    }
    trim_event_log(&mut runtime_shell.last_audio_events);
    continue_visible_script_after_prompt(runtime_shell)?;
    Ok(())
}

fn play_pending_field_notice_sound(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    if let Some(species) = runtime_shell.pending_field_notice_cry.take() {
        queue_visible_pokemon_cry(runtime_shell, &species, "field_notice")?;
    }
    let Some(audio_id) = runtime_shell.pending_field_notice_sound.take() else {
        return Ok(());
    };
    let BevyRuntimeShell {
        shell,
        pending_audio,
        last_audio_events,
        ..
    } = runtime_shell;
    queue_visible_sound_effect(
        shell.runtime().audio(),
        pending_audio,
        last_audio_events,
        &audio_id,
    )
}

fn commit_visible_pending_block_field_move(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let Some(pending) = runtime_shell
        .shell
        .session()
        .state()
        .script_runtime
        .pending_block_field_move
        .as_ref()
    else {
        return Ok(());
    };
    let (source_script, command_index) = match pending.move_id.as_str() {
        "CUT" => ("Script_Cut", 3),
        "WHIRLPOOL" => ("Script_UsedWhirlpool", 3),
        move_id => anyhow::bail!("unsupported pending visible block field move {move_id}"),
    };
    execute_visible_deferred_field_move_callasm(runtime_shell, source_script, command_index)
}

fn commit_visible_pending_flash_field_move(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    if runtime_shell
        .shell
        .session()
        .state()
        .script_runtime
        .pending_flash_field_move
        .is_none()
    {
        return Ok(());
    }
    execute_visible_deferred_field_move_callasm(runtime_shell, "Script_UseFlash", 3)
}

fn commit_visible_pending_surf_field_move(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    if runtime_shell
        .shell
        .session()
        .state()
        .script_runtime
        .pending_surf_field_move
        .is_none()
    {
        return Ok(());
    }
    for (command_index, expected_command) in [
        (3, "callasm"),
        (4, "readmem"),
        (5, "writevar"),
        (6, "special"),
        (7, "special"),
        (8, "special"),
        (9, "applymovement"),
    ] {
        execute_visible_deferred_field_move_source_command(
            runtime_shell,
            "UsedSurfScript",
            command_index,
            expected_command,
        )?;
    }
    Ok(())
}

fn execute_visible_pending_waterfall_step(
    runtime_shell: &mut BevyRuntimeShell,
    step_index: u16,
    total_steps: u16,
) -> Result<()> {
    anyhow::ensure!(
        step_index < total_steps,
        "visible WATERFALL step {step_index} is outside total {total_steps}"
    );
    let origin_map_name = runtime_shell.shell.session().overworld().map.name.clone();
    for (command_index, expected_command) in [(0, "applymovement"), (1, "callasm")] {
        let source_script = ".loop@Script_UsedWaterfall";
        record_visible_runtime_action(
            runtime_shell,
            format!("script:step:{source_script}:{command_index}"),
        )?;
        let stepped = runtime_shell.shell.step_compiled_script_command(
            &origin_map_name,
            source_script,
            command_index,
            ScriptRuntimeInputs::default(),
            ScriptPhoneInputs::default(),
        )?;
        anyhow::ensure!(
            stepped.command == expected_command,
            "WATERFALL source loop expected {expected_command}, found {}",
            stepped.command
        );
        runtime_shell.last_audio_events.push(format!(
            "script step={source_script} command={command_index} result={} checksum={:?}",
            stepped.mutation.result.result_tag(),
            stepped.mutation.state_checksum
        ));
    }
    let expected_value = if step_index + 1 == total_steps {
        "1"
    } else {
        "0"
    };
    anyhow::ensure!(
        runtime_shell
            .shell
            .session()
            .state()
            .script_runtime
            .script_value
            .as_deref()
            == Some(expected_value),
        "WATERFALL continuation returned a source value inconsistent with step {}/{}",
        step_index + 1,
        total_steps
    );
    trim_event_log(&mut runtime_shell.last_audio_events);
    Ok(())
}

fn execute_visible_deferred_field_move_callasm(
    runtime_shell: &mut BevyRuntimeShell,
    source_script: &str,
    command_index: usize,
) -> Result<()> {
    execute_visible_deferred_field_move_source_command(
        runtime_shell,
        source_script,
        command_index,
        "callasm",
    )
}

fn execute_visible_deferred_field_move_source_command(
    runtime_shell: &mut BevyRuntimeShell,
    source_script: &str,
    command_index: usize,
    expected_command: &str,
) -> Result<()> {
    let origin_map_name = runtime_shell.shell.session().overworld().map.name.clone();
    record_visible_runtime_action(
        runtime_shell,
        format!("script:step:{source_script}:{command_index}"),
    )?;
    let stepped = runtime_shell.shell.step_compiled_script_command(
        &origin_map_name,
        source_script,
        command_index,
        ScriptRuntimeInputs::default(),
        ScriptPhoneInputs::default(),
    )?;
    anyhow::ensure!(
        stepped.command == expected_command,
        "field-move source boundary expected {expected_command}, found {}",
        stepped.command
    );
    integrate_visible_script_mutation_outcome(runtime_shell, &stepped.mutation)?;
    runtime_shell.last_audio_events.push(format!(
        "script step={source_script} command={command_index} result={} checksum={:?}",
        stepped.mutation.result.result_tag(),
        stepped.mutation.state_checksum
    ));
    trim_event_log(&mut runtime_shell.last_audio_events);
    Ok(())
}

fn begin_pending_field_notice_effect(runtime_shell: &mut BevyRuntimeShell) -> Result<bool> {
    if runtime_shell.visible_waterfall_animation.is_some() {
        return Ok(true);
    }
    if runtime_shell.pending_whirlpool_sound_wait {
        // DisappearWhirlpool writes and redraws replacement block $36 before
        // PlayWhirlpoolSound. There is no authored 32-frame overlay: the
        // routine blocks until the complete SFX_SURF program has finished.
        commit_visible_pending_block_field_move(runtime_shell)?;
        runtime_shell.field_notice_scene = None;
        runtime_shell.pending_whirlpool_sound_wait = false;
        runtime_shell.visible_wait_sfx_boundary = true;
        runtime_shell.wait_play_sfx_completion =
            Some(VisibleWaitPlaySfxCompletion::WhirlpoolFieldMove);
        return Ok(true);
    }
    if runtime_shell.pending_field_notice_effect_frames.is_none() {
        return Ok(false);
    }
    if let Some(from_tile) = runtime_shell.pending_surf_start_from {
        // UsedSurfScript switches to the surf sprite and then applies one
        // sixteen-frame `slow_step`. Execute those exact source commands only
        // after the use text closes, then interpolate from the retained land
        // tile to the newly committed destination.
        commit_visible_pending_surf_field_move(runtime_shell)?;
        runtime_shell.field_notice_scene = None;
        runtime_shell.player_walk_from = Some(from_tile);
        runtime_shell.player_walk_total_ticks = WALK_FRAME_HOLD_TICKS.saturating_mul(2);
        runtime_shell.player_walk_frame_ticks = runtime_shell.player_walk_total_ticks;
        runtime_shell.player_walk_stride = true;
        runtime_shell.player_walk_mirror_stride = false;
    } else if runtime_shell.visible_flash_animation.is_some() {
        commit_visible_pending_flash_field_move(runtime_shell)?;
    } else if runtime_shell.visible_cut_animation.is_some() {
        // The source callasm owns the block write. Commit it only after the
        // use text closes, immediately before its OAM draws over the cleared
        // tilemap.
        commit_visible_pending_block_field_move(runtime_shell)?;
        runtime_shell.field_notice_scene = None;
    } else if !runtime_shell.pending_whirlpool_sound_wait
        && runtime_shell.visible_headbutt_animation.is_none()
        && runtime_shell.visible_flash_animation.is_none()
    {
        runtime_shell.visible_earthquake = Some(VisibleEarthquake {
            intensity: 2,
            frames_remaining: 20,
            shake_frames_remaining: 20,
        });
    }
    Ok(true)
}

fn visible_field_notice_uses_prompt_arrow(runtime_shell: &BevyRuntimeShell) -> bool {
    let fruit_tree_page_has_prompt = runtime_shell
        .visible_field_item_notice
        .as_ref()
        .is_some_and(|notice| {
            matches!(
                notice.presentation,
                VisibleFieldItemPresentation::FruitTree { .. }
            ) && match notice.phase {
                VisibleFieldItemPhase::FoundText => {
                    runtime_shell.field_notice.as_deref()
                        != Some(notice.sound_trigger_text.as_str())
                }
                VisibleFieldItemPhase::PromptEachQueuedPage => true,
                _ => false,
            }
        });
    runtime_shell.pending_field_travel_delay_frames.is_none()
        && runtime_shell.visible_field_travel_animation.is_none()
        && runtime_shell.pending_surf_start_from.is_none()
        && runtime_shell.visible_waterfall_animation.is_none()
        && runtime_shell.visible_flash_animation.is_none()
        && runtime_shell.visible_cut_animation.is_none()
        && !runtime_shell.pending_whirlpool_sound_wait
        && runtime_shell.visible_headbutt_animation.is_none()
        && !runtime_shell.pending_field_battle_entry
        && (runtime_shell.field_notice_queue.is_empty() || fruit_tree_page_has_prompt)
        && !runtime_shell
            .visible_field_item_notice
            .as_ref()
            .is_some_and(|notice| match notice.phase {
                VisibleFieldItemPhase::FoundText => {
                    runtime_shell.field_notice.as_deref()
                        == Some(notice.sound_trigger_text.as_str())
                }
                VisibleFieldItemPhase::FanfarePause { .. }
                | VisibleFieldItemPhase::SpecialSoundWait => true,
                _ => false,
            })
}

fn settle_pending_field_battle_entry_after_notice(
    runtime_shell: &mut BevyRuntimeShell,
) -> Result<bool> {
    if !std::mem::take(&mut runtime_shell.pending_field_battle_entry) {
        return Ok(false);
    }
    mark_runtime_snapshot_dirty(runtime_shell);
    prepare_visible_battle_entry(runtime_shell)?;
    settle_visible_battle_after_action(runtime_shell)?;
    Ok(true)
}

fn advance_visible_heal_machine(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let (kind, party_count, frame) = runtime_shell
        .visible_heal_machine
        .as_ref()
        .map(|animation| (animation.kind, animation.party_count, animation.frame))
        .context("HealMachineAnim disappeared during its retained frame")?;
    let ball_frames = u16::from(party_count) * 30;
    if frame < ball_frames && frame % 30 == 0 {
        let BevyRuntimeShell {
            shell,
            pending_audio,
            last_audio_events,
            ..
        } = runtime_shell;
        queue_visible_sound_effect(
            shell.runtime().audio(),
            pending_audio,
            last_audio_events,
            "SFX_SECOND_PART_OF_ITEMFINDER",
        )?;
    }
    if frame == ball_frames {
        if kind == 2 {
            let BevyRuntimeShell {
                shell,
                pending_audio,
                last_audio_events,
                ..
            } = runtime_shell;
            queue_visible_sound_effect(
                shell.runtime().audio(),
                pending_audio,
                last_audio_events,
                "SFX_GAME_FREAK_LOGO_GS",
            )?;
        } else {
            queue_visible_heal_music(runtime_shell)?;
        }
    }
    let total_frames = ball_frames + 80;
    if frame >= total_frames {
        if kind == 2 {
            let BevyRuntimeShell {
                shell,
                pending_audio,
                last_audio_events,
                ..
            } = runtime_shell;
            queue_visible_sound_effect(
                shell.runtime().audio(),
                pending_audio,
                last_audio_events,
                "SFX_BOOT_PC",
            )?;
        }
        runtime_shell.visible_heal_machine = None;
        mark_runtime_snapshot_dirty(runtime_shell);
        return continue_visible_script_after_prompt(runtime_shell);
    }
    runtime_shell.visible_heal_machine.as_mut().unwrap().frame += 1;
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(())
}

fn visible_heal_machine_is_terminal(animation: &VisibleHealMachine) -> bool {
    animation.frame >= u16::from(animation.party_count) * 30 + 80
}

fn queue_visible_heal_music(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    const MUSIC_ID: &str = "MUSIC_HEAL";
    let playback = runtime_shell
        .shell
        .runtime()
        .audio()
        .require_playback_entry(AudioKind::Music, MUSIC_ID)?;
    enqueue_bevy_audio_command(
        &mut runtime_shell.pending_audio,
        BevyAudioCommand {
            battle_sound: None,
            cry_parameters: None,
            audio_id: MUSIC_ID.to_string(),
            kind: ModpackAudioKind::Music,
            mode: playback.mode,
            looped: matches!(
                playback.loop_policy,
                crate::assets::ModpackAudioLoopPolicy::Loop
            ),
        },
    );
    runtime_shell.pending_music_stop = true;
    runtime_shell.active_music = Some(MUSIC_ID.to_string());
    runtime_shell.heal_music_active = true;
    runtime_shell.faded_music = None;
    runtime_shell
        .last_audio_events
        .push("queued heal-machine music MUSIC_HEAL".to_string());
    trim_event_log(&mut runtime_shell.last_audio_events);
    Ok(())
}

fn queue_visible_magnet_train_music(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    // `InitMagnetTrain` calls PlayMusic2, whose first half invokes
    // `_PlayMusic(MUSIC_NONE)` and then blocks in DelayFrame. This is not the
    // full `_InitSound` reset used by PlayMusic(MUSIC_NONE), so channels 5-8
    // must survive. Phase zero queues the replacement after that retained
    // frame has elapsed.
    runtime_shell.pending_music_stop = true;
    clear_pending_music_commands(&mut runtime_shell.pending_audio);
    runtime_shell.active_music = None;
    runtime_shell.faded_music = None;
    Ok(())
}

fn queue_visible_magnet_train_track(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    const MUSIC_ID: &str = "MUSIC_MAGNET_TRAIN";
    let playback = runtime_shell
        .shell
        .runtime()
        .audio()
        .require_playback_entry(AudioKind::Music, MUSIC_ID)?;
    enqueue_bevy_audio_command(
        &mut runtime_shell.pending_audio,
        BevyAudioCommand {
            battle_sound: None,
            cry_parameters: None,
            audio_id: MUSIC_ID.to_string(),
            kind: ModpackAudioKind::Music,
            mode: playback.mode,
            looped: matches!(
                playback.loop_policy,
                crate::assets::ModpackAudioLoopPolicy::Loop
            ),
        },
    );
    runtime_shell.active_music = Some(MUSIC_ID.to_string());
    runtime_shell.faded_music = None;
    Ok(())
}

fn advance_visible_magnet_train(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    if runtime_shell
        .visible_magnet_train
        .as_ref()
        .is_some_and(|animation| animation.phase >= 7 && animation.arrival_sfx_played)
    {
        runtime_shell.visible_magnet_train = None;
        mark_runtime_snapshot_dirty(runtime_shell);
        return continue_visible_script_after_prompt(runtime_shell);
    }
    let phase = runtime_shell
        .visible_magnet_train
        .as_ref()
        .context("MagnetTrain disappeared during its retained frame")?
        .phase;
    if phase == 0 {
        queue_visible_magnet_train_track(runtime_shell)?;
    }
    let animation = runtime_shell
        .visible_magnet_train
        .as_mut()
        .context("MagnetTrain disappeared during its retained frame")?;
    if animation.phase != 0 {
        if !animation.player_sprite_visible {
            animation.player_sprite_visible = true;
            animation.player_sprite_frame = 0;
            animation.player_sprite_duration = 8;
        } else if animation.player_sprite_duration > 0 {
            animation.player_sprite_duration -= 1;
        } else {
            animation.player_sprite_frame = (animation.player_sprite_frame + 1) % 4;
            animation.player_sprite_duration = 8;
        }
    }
    match animation.phase {
        0 => {
            animation.wait_counter = 128;
            animation.phase = 1;
        }
        1 | 3 | 5 => {
            if animation.wait_counter > 0 {
                animation.wait_counter -= 1;
            } else {
                animation.phase += 1;
            }
        }
        2 => {
            if animation.position == animation.hold_position {
                animation.wait_counter = 128;
                animation.phase = 3;
            } else {
                animation.position -= animation.direction;
                animation.player_x += animation.direction;
            }
        }
        4 => {
            if animation.position == animation.final_position {
                animation.phase = 5;
            } else {
                animation.position -= animation.direction * 2;
                animation.player_x += animation.direction * 2;
            }
        }
        6 => animation.phase = 7,
        _ => {}
    }
    animation.offset += animation.direction * 2;
    if animation.phase < 7 {
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(());
    }
    let BevyRuntimeShell {
        shell,
        pending_audio,
        last_audio_events,
        ..
    } = runtime_shell;
    queue_visible_sound_effect(
        shell.runtime().audio(),
        pending_audio,
        last_audio_events,
        "SFX_TRAIN_ARRIVED",
    )?;
    runtime_shell
        .visible_magnet_train
        .as_mut()
        .unwrap()
        .arrival_sfx_played = true;
    mark_runtime_snapshot_dirty(runtime_shell);
    Ok(())
}

fn close_visible_unown_words(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    anyhow::ensure!(
        runtime_shell.visible_unown_words.take().is_some(),
        "Unown word display disappeared before acknowledgement"
    );
    queue_visible_shell_sound_effect(runtime_shell, "SFX_READ_TEXT_2")?;
    mark_runtime_snapshot_dirty(runtime_shell);
    continue_visible_script_after_prompt(runtime_shell)
}

fn close_visible_diploma(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    anyhow::ensure!(
        runtime_shell.visible_diploma.take().is_some(),
        "Diploma disappeared before acknowledgement"
    );
    mark_runtime_snapshot_dirty(runtime_shell);
    continue_visible_script_after_prompt(runtime_shell)
}

fn continue_visible_script_after_prompt(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    const MAX_CONTINUE_STEPS: usize = 2048;
    for _ in 0..MAX_CONTINUE_STEPS {
        if !runtime_shell.battle_messages.is_empty() { return Ok(()); }
        // PokemonCenterPC owns its synchronous special until the user exits.
        if runtime_shell.pc_hub_session_open { return Ok(()); }
        // RingTwice_StartCall and HangUp are synchronous ASM calls. Only
        // their presentation timer may release the script continuation.
        if runtime_shell.incoming_phone_sequence.is_some() {
            return Ok(());
        }
        if close_visible_noninteractive_runtime_surface(runtime_shell)? {
            continue;
        }
        let snapshot = runtime_shell.shell.presentation_snapshot()?;
        if snapshot.script_events.pending_music_fade.is_some() {
            // `musicfadeout` starts the audio fade and immediately returns in
            // ScriptEvents; it is not a script delay. Consuming the request
            // without advancing stranded radio broadcasts before their next
            // writetext forever.
            take_visible_pending_music_fade(runtime_shell)?;
            continue;
        }
        // TextLabel is not a generic auto request: PrintText owns the LCD
        // until every page has rendered. It is consumed by the typewriter's
        // full-stream completion path. Timers, fades, and map requests remain
        // automatic here.
        if snapshot.script_events.pending_text_label.is_none()
            && snapshot.script_events.pending_music_fade.is_none()
            && advance_visible_next_pending_script_request(runtime_shell, &snapshot)?
        {
            return Ok(());
        }
        if !snapshot.script_events.audio_events.is_empty() {
            drain_visible_audio_events(runtime_shell)?;
            continue;
        }
        if has_visible_pending_non_audio_script_events(&snapshot) {
            drain_visible_non_audio_script_events(runtime_shell)?;
            continue;
        }
        // End/EndCallback and map-control flags can be the final products of
        // a compiled command, after its cursor has already become None. ASM
        // consumes that terminal control work before returning to joypad
        // polling. Leaving it behind makes the first overworld direction or
        // A press service script history instead of moving/interacting.
        if snapshot.script_events.script_ended.is_some() {
            take_visible_script_end_state(runtime_shell)?;
            continue;
        }
        if let Some(flag) = visible_auto_runtime_flag(&snapshot) {
            consume_visible_runtime_flag_kind(runtime_shell, flag)?;
            continue;
        }
        if runtime_shell.active_script_cursor.is_none() {
            return Ok(());
        }
        // `writetext` leaves the text window open while the script immediately
        // advances into `waitbutton`/`promptbutton`. Likewise, acknowledging
        // that wait resumes directly into `closetext`. Treating the open
        // window itself as a boundary before those commands run strands the
        // script between its text and wait opcodes (notably MeetMomScript).
        let text_window_blocks =
            snapshot.ui.text_window_open && runtime_shell.active_script_cursor.is_none();
        if runtime_shell.visible_mom_bank.is_some()
            || runtime_shell.field_notice.is_some()
            || runtime_shell.pc_notice.is_some()
            || runtime_shell.visible_wait_sfx_boundary
            || snapshot.ui.pending_yes_no.is_some()
            || runtime_shell.pending_day_of_week.is_some()
            || runtime_shell.pending_phone_prompt.is_some()
            || runtime_shell.pending_remember_password.is_some()
            || snapshot.ui.pending_text_wait.is_some()
            || snapshot.script_events.pending_text_label.is_some()
            || snapshot.script_events.pending_script_warp.is_some()
            || snapshot.script_events.pending_map_load.is_some()
            || snapshot.script_events.pending_map_refresh.is_some()
            || snapshot.script_events.pending_music_fade.is_some()
            || snapshot.script_events.pending_screen_fade.is_some()
            || !snapshot.script_events.pending_delays.is_empty()
            || !snapshot.script_events.pending_earthquakes.is_empty()
            || !snapshot.script_events.pending_emotes.is_empty()
            || snapshot.pending_shop.is_some()
            || text_window_blocks
            || snapshot.ui.window_open
            || snapshot.ui.active_pokemon_picture.is_some()
            || runtime_shell.elevator_cursor.is_some()
            || visible_menu_has_selectable_options(&snapshot)
            || snapshot.battle.is_some()
            || runtime_shell.start_menu_cursor.is_some()
            || runtime_shell.party_menu_open
            || runtime_shell.pokedex_menu_open
            || runtime_shell.pokegear_menu_open
            || runtime_shell.options_menu_open
            || runtime_shell.save_menu_open
            || runtime_shell.special_boundary.is_some()
            || runtime_shell.visible_wait_sfx_boundary
            || runtime_shell.visible_heal_machine.is_some()
            || runtime_shell.visible_magnet_train.is_some()
            || runtime_shell.kurt_apricorn_cursor.is_some()
            || runtime_shell.visible_buena_password.is_some()
            || runtime_shell.visible_battle_tower_challenge_menu.is_some()
            || runtime_shell.visible_battle_tower_room_menu.is_some()
            || runtime_shell.buena_prize_cursor.is_some()
            || runtime_shell.intro_screen.is_some()
            || runtime_shell.credits_screen.is_some()
            || visible_field_pack_is_open(runtime_shell)
            || runtime_shell.storage_cursor.is_some()
            || runtime_shell.pc_item_cursor.is_some()
            || runtime_shell.pc_confirmation.is_some()
            || runtime_shell.decoration_menu.is_some()
            || runtime_shell.player_pc_action_cursor.is_some()
            || runtime_shell.mailbox_cursor.is_some()
            || runtime_shell.mailbox_action_cursor.is_some()
        {
            return Ok(());
        }
        execute_visible_active_script_step(runtime_shell)?;
    }
    anyhow::bail!("visible script continuation exceeded {MAX_CONTINUE_STEPS} steps")
}

fn advance_visible_wait_sfx_boundary(
    runtime_shell: &mut BevyRuntimeShell,
    presentation_snapshot: &RuntimeShellSnapshot,
    require_rendered_text: bool,
) -> Result<bool> {
    if !runtime_shell.visible_wait_sfx_boundary {
        return Ok(false);
    }
    // opentext may precede waitsfx and the first writetext. An empty
    // window has no printer page to acknowledge.
    if presentation_snapshot.ui.text_window_open
        && visible_field_dialog_pages(presentation_snapshot, runtime_shell).is_some()
    {
        if !visible_field_dialogue_is_fully_revealed(runtime_shell, presentation_snapshot) {
            return Ok(true);
        }
        // The frame-loop SFX poll is not a player button. PlaceString's
        // paragraph/CONT waits still belong to A/B, even when audio ends.
        if require_rendered_text
            && !visible_field_dialogue_is_entirely_consumed(runtime_shell, presentation_snapshot)
        {
            return Ok(true);
        }
        if advance_visible_completed_field_text_page(runtime_shell, presentation_snapshot)? {
            return Ok(true);
        }
        if require_rendered_text
            && visible_field_dialogue_is_entirely_consumed(runtime_shell, presentation_snapshot)
        {
            let Some(reveal) = runtime_shell.field_text_reveal.as_ref() else {
                return Ok(true);
            };
            let completed_identity = (reveal.text.clone(), reveal.page_index);
            if runtime_shell.rendered_field_text_identity.as_ref() != Some(&completed_identity) {
                return Ok(true);
            }
        }
    }
    drain_visible_audio_events(runtime_shell)?;
    if !visible_wait_sfx_finished(runtime_shell) {
        return Ok(true);
    }
    if let Some(audio_id) = runtime_shell.pending_wait_play_sfx.pop_front() {
        queue_visible_shell_sound_effect(runtime_shell, &audio_id)?;
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(true);
    }
    if let Some(completion) = runtime_shell.wait_play_sfx_completion.take() {
        runtime_shell.visible_wait_sfx_boundary = false;
        match completion {
            VisibleWaitPlaySfxCompletion::FieldNotice(notice) => {
                continue_visible_script_after_prompt(runtime_shell)?;
                // `WaitPlaySFX` returns before the following `writetext`. Set the
                // typed Itemfinder page after continuation has drained any stale
                // noninteractive Pack surface so that cleanup cannot erase it.
                runtime_shell.field_notice = Some(notice);
            }
            VisibleWaitPlaySfxCompletion::FieldItemPocketText => {
                let notice = runtime_shell
                    .visible_field_item_notice
                    .as_mut()
                    .context("field-item specialsound lost its presentation state")?;
                anyhow::ensure!(
                    notice.phase == VisibleFieldItemPhase::SpecialSoundWait,
                    "field-item specialsound completed in phase {:?}",
                    notice.phase
                );
                runtime_shell.field_notice = Some(notice.pocket_text.clone());
                notice.phase = VisibleFieldItemPhase::PocketText;
            }
            VisibleWaitPlaySfxCompletion::VerboseItemPrompt => {
                let notice = runtime_shell
                    .visible_field_item_notice
                    .as_mut()
                    .context("verbose-item specialsound lost its presentation state")?;
                anyhow::ensure!(
                    notice.phase == VisibleFieldItemPhase::SpecialSoundWait,
                    "verbose-item specialsound completed in phase {:?}",
                    notice.phase
                );
                notice.phase = VisibleFieldItemPhase::AwaitingPrompt;
            }
            VisibleWaitPlaySfxCompletion::SpecialBoundary(boundary) => {
                set_shell_action_status(runtime_shell, boundary.label.clone());
                runtime_shell.special_boundary = Some(boundary);
            }
            VisibleWaitPlaySfxCompletion::FlashFieldMove => {
                runtime_shell.pending_field_notice_effect_frames = Some(16);
                begin_pending_field_notice_effect(runtime_shell)?;
            }
            VisibleWaitPlaySfxCompletion::WhirlpoolFieldMove => {}
        }
        mark_runtime_snapshot_dirty(runtime_shell);
        return Ok(true);
    }
    if runtime_shell
        .shell
        .snapshot()?
        .script_events
        .pending_text_label
        .is_some()
    {
        runtime_shell
            .shell
            .take_pending_script_request(RuntimePendingScriptRequestKind::TextLabel)?;
    }
    if runtime_shell
        .shell
        .snapshot()?
        .script_events
        .waiting_for_sound_effect
    {
        runtime_shell
            .shell
            .consume_script_runtime_flag(RuntimeScriptRuntimeFlag::WaitingForSoundEffect)?;
    }
    runtime_shell.visible_wait_sfx_boundary = false;
    continue_visible_script_after_prompt(runtime_shell)?;
    Ok(true)
}

fn advance_visible_special_text_pause(runtime_shell: &mut BevyRuntimeShell) -> Result<bool> {
    let Some(frames) = runtime_shell.visible_special_text_pause_frames.as_mut() else {
        return Ok(false);
    };
    *frames = frames.saturating_sub(1);
    if *frames == 0 {
        close_visible_special_boundary(runtime_shell)?;
    } else {
        mark_runtime_snapshot_dirty(runtime_shell);
    }
    Ok(true)
}

fn advance_visible_next_pending_script_request(
    runtime_shell: &mut BevyRuntimeShell,
    snapshot: &RuntimeShellSnapshot,
) -> Result<bool> {
    if snapshot.script_events.pending_text_label.is_some() {
        advance_visible_text_label(runtime_shell)?;
        return Ok(true);
    }
    if snapshot
        .script_events
        .pending_map_load
        .as_ref()
        .is_some_and(|load| load.command == "newloadmap")
    {
        take_visible_pending_map_load(runtime_shell)?;
        return Ok(true);
    }
    if snapshot.script_events.pending_script_warp.is_some() {
        execute_visible_pending_script_warp(runtime_shell)?;
        return Ok(true);
    }
    if snapshot.script_events.pending_map_load.is_some() {
        take_visible_pending_map_load(runtime_shell)?;
        return Ok(true);
    }
    if snapshot.script_events.pending_map_refresh.is_some() {
        take_visible_pending_map_refresh(runtime_shell)?;
        return Ok(true);
    }
    if snapshot.script_events.pending_music_fade.is_some() {
        take_visible_pending_music_fade(runtime_shell)?;
        return Ok(true);
    }
    if snapshot.script_events.pending_screen_fade.is_some() {
        take_visible_pending_screen_fade(runtime_shell)?;
        return Ok(true);
    }
    if !snapshot.script_events.pending_delays.is_empty() {
        drain_visible_delays(runtime_shell)?;
        return Ok(true);
    }
    if !snapshot.script_events.pending_earthquakes.is_empty() {
        drain_visible_earthquakes(runtime_shell)?;
        return Ok(true);
    }
    if !snapshot.script_events.pending_emotes.is_empty() {
        drain_visible_emotes(runtime_shell)?;
        return Ok(true);
    }
    Ok(false)
}

fn toggle_visible_start_menu(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    // Neither BillsPC nor PCItemsJoypad maps START to the party or bag.
    if runtime_shell.storage_cursor.is_some() || runtime_shell.pc_item_cursor.is_some() {
        return Ok(());
    }
    if runtime_shell.special_boundary.is_some() {
        return Ok(());
    }
    if runtime_shell.start_menu_cursor.is_some() {
        record_visible_runtime_action(runtime_shell, "start_menu:close")?;
        close_visible_start_menu(runtime_shell);
        return Ok(());
    }
    let snapshot = runtime_shell.shell.presentation_snapshot()?;
    let blockers = visible_start_menu_blockers(runtime_shell, &snapshot);
    if !blockers.is_empty() {
        record_visible_runtime_action(
            runtime_shell,
            format!("start_menu:blocked:{}", blockers.join(",")),
        )?;
        runtime_shell
            .last_audio_events
            .push(format!("Start menu blocked by {}", blockers.join(", ")));
        set_shell_action_status(runtime_shell, "START MENU BLOCKED");
        trim_event_log(&mut runtime_shell.last_audio_events);
        return Ok(());
    }
    record_visible_runtime_action(runtime_shell, "start_menu:open")?;
    close_visible_field_pack_without_log(runtime_shell);
    close_visible_party_detail_state(runtime_shell);
    runtime_shell.pokedex_menu_open = false;
    runtime_shell.pokedex_detail_open = false;
    runtime_shell.pokedex_detail_page = 0;
    runtime_shell.pokedex_scripted_entry = false;
    runtime_shell.pokegear_menu_open = false;
    runtime_shell.pokegear_map_radio_delay = None;
    runtime_shell.pokegear_phone_status = None;
    runtime_shell.options_menu_open = false;
    runtime_shell.save_menu_open = false;
    runtime_shell.save_flow = None;
    runtime_shell.special_boundary = None;
    runtime_shell.special_boundary_queue.clear();
    runtime_shell.visible_special_text_pause_frames = None;
    runtime_shell.visible_internal_special_delay_frames = None;
    runtime_shell.pending_photo_studio_commit = None;
    runtime_shell.pending_special_cry = None;
    runtime_shell.pending_special_sound = None;
    runtime_shell.field_pack_pocket = None;
    runtime_shell.field_pack_action_cursor = None;
    runtime_shell.field_pack_target_mode = None;
    runtime_shell.start_menu_cursor = Some(MenuCursor {
        surface_id: START_MENU_SURFACE_ID.to_string(),
        option_index: 0,
    });
    runtime_shell
        .last_audio_events
        .push("opened start menu".to_string());
    set_shell_action_status(runtime_shell, "START MENU");
    trim_event_log(&mut runtime_shell.last_audio_events);
    Ok(())
}

fn visible_start_menu_blockers(
    runtime_shell: &BevyRuntimeShell,
    snapshot: &RuntimeShellSnapshot,
) -> Vec<&'static str> {
    let mut blockers = Vec::new();
    if snapshot.battle.is_some() {
        blockers.push("battle");
    }
    if snapshot.pending_shop.is_some() {
        blockers.push("shop");
    }
    if snapshot.ui.text_window_open {
        blockers.push("text_window");
    }
    if snapshot.ui.window_open {
        blockers.push("window");
    }
    if snapshot.ui.menu.is_some() {
        blockers.push("menu");
    }
    if snapshot.ui.active_pokemon_picture.is_some() {
        blockers.push("pokemon_picture");
    }
    if snapshot.ui.pending_yes_no.is_some() {
        blockers.push("yes_no");
    }
    if snapshot.ui.pending_text_wait.is_some() {
        blockers.push("text_wait");
    }
    if snapshot.pending_move_learn.is_some() {
        blockers.push("move_learn");
    }
    if runtime_shell.party_menu_open {
        blockers.push("party");
    }
    if runtime_shell.pokedex_menu_open {
        blockers.push("pokedex");
    }
    if runtime_shell.pokegear_menu_open {
        blockers.push("pokegear");
    }
    if runtime_shell.options_menu_open {
        blockers.push("options");
    }
    if runtime_shell.trainer_card_open {
        blockers.push("trainer_card");
    }
    if runtime_shell.save_menu_open {
        blockers.push("save");
    }
    if runtime_shell.special_boundary.is_some() {
        blockers.push("special_boundary");
    }
    if runtime_shell.kurt_apricorn_cursor.is_some() {
        blockers.push("kurt_apricorn");
    }
    if runtime_shell.buena_prize_cursor.is_some() {
        blockers.push("buena_prize");
    }
    if runtime_shell.visible_buena_password.is_some() {
        blockers.push("buena_password");
    }
    if runtime_shell.visible_battle_tower_challenge_menu.is_some() {
        blockers.push("battle_tower_challenge_menu");
    }
    if runtime_shell.visible_battle_tower_room_menu.is_some() {
        blockers.push("battle_tower_room_menu");
    }
    if runtime_shell.pc_hub_cursor.is_some() {
        blockers.push("pc_hub");
    }
    if runtime_shell.bill_pc_action_cursor.is_some() {
        blockers.push("bill_pc");
    }
    if runtime_shell.bill_pc_box_cursor.is_some() {
        blockers.push("bill_pc_box");
    }
    if runtime_shell.intro_screen.is_some() {
        blockers.push("intro");
    }
    if runtime_shell.credits_screen.is_some() {
        blockers.push("credits");
    }
    if runtime_shell.storage_cursor.is_some() {
        blockers.push("storage");
    }
    if runtime_shell.pc_item_cursor.is_some() {
        blockers.push("pc_item");
    }
    if visible_field_pack_is_open(runtime_shell) {
        blockers.push("pack");
    }
    if has_visible_auto_script_action(runtime_shell, snapshot) {
        blockers.push("auto_script");
    }
    blockers
}

fn visible_quick_save_blockers(
    runtime_shell: &BevyRuntimeShell,
    snapshot: &RuntimeShellSnapshot,
    allow_active_script_cursor: bool,
    allow_save_menu: bool,
    allow_bill_pc: bool,
) -> Vec<&'static str> {
    let mut blockers = visible_start_menu_blockers(runtime_shell, snapshot)
        .into_iter()
        .filter(|blocker| {
            (*blocker != "save" || !allow_save_menu)
                && *blocker != "auto_script"
                && (!allow_bill_pc
                    || !matches!(
                        *blocker,
                        "window" | "menu" | "bill_pc" | "storage" | "party" | "pc_hub"
                    ))
        })
        .collect::<Vec<_>>();
    if has_visible_save_blocking_script_work(snapshot) {
        blockers.push("auto_script");
    }
    if runtime_shell.intro_screen.is_some() {
        blockers.push("intro");
    }
    if runtime_shell.title_menu.is_some() {
        blockers.push("title");
    }
    if runtime_shell.start_menu_cursor.is_some() {
        blockers.push("start_menu");
    }
    if runtime_shell.party_summary_open {
        blockers.push("party_summary");
    }
    if runtime_shell.pokedex_detail_open {
        blockers.push("pokedex_detail");
    }
    if runtime_shell.pending_phone_prompt.is_some() {
        blockers.push("phone_prompt");
    }
    if runtime_shell.pending_remember_password.is_some() {
        blockers.push("remember_password");
    }
    if runtime_shell.pending_day_of_week.is_some() {
        blockers.push("day_of_week");
    }
    if runtime_shell.visible_mom_bank.is_some() {
        blockers.push("mom_bank");
    }
    if runtime_shell.pending_trainer_sight.is_some() {
        blockers.push("trainer_sight");
    }
    if runtime_shell.pending_name_input.is_some() {
        blockers.push("name_input");
    }
    if runtime_shell.pending_mail_input.is_some() {
        blockers.push("mail_input");
    }
    if runtime_shell.pending_mail_read.is_some() {
        blockers.push("mail_read");
    }
    if runtime_shell.pending_name_choice.is_some() {
        blockers.push("name_choice");
    }
    if runtime_shell.active_script_cursor.is_some() && !allow_active_script_cursor {
        blockers.push("script");
    }
    blockers
}

fn visible_quick_load_blockers(
    runtime_shell: &BevyRuntimeShell,
    snapshot: &RuntimeShellSnapshot,
) -> Vec<&'static str> {
    visible_quick_save_blockers(runtime_shell, snapshot, false, false, false)
}

fn has_visible_save_blocking_script_work(snapshot: &RuntimeShellSnapshot) -> bool {
    snapshot.script_events.pending_text_label.is_some()
        || snapshot.script_events.pending_map_load.is_some()
        || snapshot.script_events.pending_map_refresh.is_some()
        || snapshot.script_events.pending_music_fade.is_some()
        || snapshot.script_events.pending_screen_fade.is_some()
        || !snapshot.script_events.pending_delays.is_empty()
        || !snapshot.script_events.pending_earthquakes.is_empty()
        || !snapshot.script_events.pending_emotes.is_empty()
        || snapshot.script_events.pending_script_warp.is_some()
        || !snapshot.script_events.command_queue.is_empty()
        || snapshot.script_events.next_script.is_some()
        || snapshot.script_events.map_reentry_script.is_some()
        || !snapshot.script_events.deferred_scripts.is_empty()
        || snapshot.script_events.script_ended.is_some()
        || !snapshot.script_events.audio_events.is_empty()
        || has_visible_pending_non_audio_script_events(snapshot)
        || visible_auto_runtime_flag(snapshot).is_some()
}

fn close_visible_start_menu(runtime_shell: &mut BevyRuntimeShell) {
    runtime_shell.start_menu_cursor = None;
    runtime_shell
        .last_audio_events
        .push("closed start menu".to_string());
    trim_event_log(&mut runtime_shell.last_audio_events);
}

fn select_visible_start_menu_option(runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
    let selected = selected_visible_start_menu_option(runtime_shell)?;
    let selected_label = start_menu_option_label(selected).to_string();
    record_visible_runtime_action(
        runtime_shell,
        format!(
            "start_menu:{}",
            selected_label.replace(' ', "_").to_ascii_lowercase()
        ),
    )?;
    match selected {
        StartMenuOption::Pokemon => {
            open_visible_party_menu(runtime_shell)?;
        }
        StartMenuOption::Pack => {
            open_visible_field_pack(runtime_shell)?;
        }
        StartMenuOption::Save => {
            open_visible_save_menu(runtime_shell)?;
        }
        StartMenuOption::QuitContest => {
            close_visible_start_menu(runtime_shell);
            start_visible_script_entry(runtime_shell, "BugCatchingContestReturnToGateScript")?;
        }
        StartMenuOption::Pokedex => {
            open_visible_pokedex_menu(runtime_shell)?;
        }
        StartMenuOption::Pokegear => {
            open_visible_pokegear_menu(runtime_shell)?;
        }
        StartMenuOption::TrainerCard => {
            open_visible_trainer_card(runtime_shell)?;
        }
        StartMenuOption::Personalization => {
            crystal_customization_open();
        }
        StartMenuOption::Options => {
            open_visible_options_menu(runtime_shell)?;
        }
        StartMenuOption::Exit => {
            close_visible_start_menu(runtime_shell);
            continue_visible_script_after_prompt(runtime_shell)?;
        }
    }
    runtime_shell.start_menu_cursor = None;
    if runtime_shell.last_action_status.as_deref() == Some("START MENU") {
        set_shell_action_status(runtime_shell, format!("OPENED {selected_label}"));
    }
    trim_event_log(&mut runtime_shell.last_audio_events);
    Ok(())
}
