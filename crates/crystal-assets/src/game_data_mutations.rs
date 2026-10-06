impl GameDataSet {
    pub fn apply_runtime_mutation_command(
        &self,
        state: &mut GameState,
        session: &mut OverworldSession,
        command: RuntimeMutationCommand,
        music_ids: &BTreeSet<String>,
        sound_effect_ids: &BTreeSet<String>,
        cry_ids: &BTreeSet<String>,
    ) -> Result<RuntimeMutationOutcome> {
        if matches!(&command, RuntimeMutationCommand::ResolveBlackoutToLastSpawn) {
            let mut next_state = state.clone();
            let mut next_session = session.clone();
            let outcome = self.apply_runtime_mutation_command_with_checksum(
                &mut next_state,
                &mut next_session,
                command,
                music_ids,
                sound_effect_ids,
                cry_ids,
                true,
            )?;
            *state = next_state;
            *session = next_session;
            return Ok(outcome);
        }
        self.apply_runtime_mutation_command_with_checksum(
            state,
            session,
            command,
            music_ids,
            sound_effect_ids,
            cry_ids,
            true,
        )
    }

    /// Real-time VBlank timer path. The timer is journaled by deterministic
    /// hosts, while an interactive host may omit the retained frame checksum
    /// just like the adjacent per-VBlank overworld-input hot path.
    pub fn advance_game_timer_vblanks_fast(
        &self,
        state: &mut GameState,
        _session: &mut OverworldSession,
        vblanks: u32,
        normal_divider_trace: &RuntimeDividerTrace,
        _music_ids: &BTreeSet<String>,
        _sound_effect_ids: &BTreeSet<String>,
        _cry_ids: &BTreeSet<String>,
    ) -> Result<RuntimeMutationOutcome> {
        if vblanks == 0 {
            anyhow::bail!("game timer advance requires a nonzero VBlank count");
        }
        apply_normal_vblank_divider_trace(state, vblanks, normal_divider_trace)?;
        let frames_before = state.time.game_time_frames;
        let seconds_before = state.time.game_time_seconds;
        let minutes_before = state.time.game_time_minutes;
        let hours_before = state.time.game_time_hours;
        state.advance_game_timer_vblanks(u64::from(vblanks));
        let counted = state.time.game_time_frames != frames_before
            || state.time.game_time_seconds != seconds_before
            || state.time.game_time_minutes != minutes_before
            || state.time.game_time_hours != hours_before;
        Ok(RuntimeMutationOutcome {
            result: RuntimeMutationResult::GameTimerVBlanksAdvanced(runtime_game_timer_outcome(
                state, counted,
            )),
            // Interactive rendering explicitly disables the replay journal;
            // this sentinel is never used as an integrity boundary.
            state_checksum: StateChecksum::new(state.frame_counter, 0),
        })
    }

    fn apply_runtime_mutation_command_with_checksum(
        &self,
        state: &mut GameState,
        session: &mut OverworldSession,
        command: RuntimeMutationCommand,
        music_ids: &BTreeSet<String>,
        sound_effect_ids: &BTreeSet<String>,
        cry_ids: &BTreeSet<String>,
        compute_checksum: bool,
    ) -> Result<RuntimeMutationOutcome> {
        let result = match command {
            RuntimeMutationCommand::ApplyOverworldInput(command) => {
                let mut next_state = state.clone();
                let mut next_session = session.clone();
                next_session.set_time(
                    next_state.time.registers.hours,
                    next_state.time.time_of_day,
                );
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let frame = self.apply_overworld_input(
                    &mut next_state,
                    &mut next_session,
                    command.buttons,
                    music_ids,
                    &mut divider,
                )?;
                require_consumed_divider_trace("apply overworld input", &divider)?;
                *state = next_state;
                *session = next_session;
                RuntimeMutationResult::OverworldInputApplied(frame)
            }
            RuntimeMutationCommand::GrantScriptItem(command) => {
                let outcome = self.grant_script_item_in_session(
                    state,
                    session,
                    &command.map_name,
                    &command.source_script,
                    command.command_index,
                )?;
                state.script_runtime.script_value = Some(
                    matches!(&outcome, ScriptItemGrantOutcome::Granted { .. })
                        .then_some("1")
                        .unwrap_or("0")
                        .to_string(),
                );
                RuntimeMutationResult::ScriptItemGranted(outcome)
            }
            RuntimeMutationCommand::CheckScriptItem(command) => {
                let outcome = self.check_script_item_in_session(
                    state,
                    session,
                    &command.map_name,
                    &command.source_script,
                    command.command_index,
                )?;
                state.script_runtime.script_value =
                    Some(if outcome.held { "1" } else { "0" }.to_string());
                RuntimeMutationResult::ScriptItemChecked(outcome)
            }
            RuntimeMutationCommand::TakeScriptItem(command) => {
                let outcome = self.take_script_item_in_session(
                    state,
                    session,
                    &command.map_name,
                    &command.source_script,
                    command.command_index,
                )?;
                state.script_runtime.script_value =
                    Some(if outcome.removed { "1" } else { "0" }.to_string());
                RuntimeMutationResult::ScriptItemTaken(outcome)
            }
            RuntimeMutationCommand::PickupScriptFieldItem(command) => {
                RuntimeMutationResult::ScriptFieldItemPickedUp(
                    self.pickup_script_field_item_in_session(
                        state,
                        session,
                        &command.map_name,
                        &command.source_script,
                        command.command_index,
                    )?,
                )
            }
            RuntimeMutationCommand::ApplyScriptEconomy(command) => {
                RuntimeMutationResult::ScriptEconomyApplied(
                    self.apply_script_economy_command_in_session(
                        state,
                        session,
                        &command.map_name,
                        &command.source_script,
                        command.command_index,
                    )?,
                )
            }
            RuntimeMutationCommand::ApplyScriptPhone { command, inputs } => {
                RuntimeMutationResult::ScriptPhoneApplied(
                    self.apply_script_phone_command_in_session(
                        state,
                        session,
                        &command.map_name,
                        &command.source_script,
                        command.command_index,
                        inputs,
                    )?,
                )
            }
            RuntimeMutationCommand::ApplyScriptFlagMutation(command) => {
                RuntimeMutationResult::ScriptFlagMutated(
                    self.apply_script_flag_mutation_in_session(
                        state,
                        session,
                        &command.map_name,
                        &command.source_script,
                        command.command_index,
                    )?,
                )
            }
            RuntimeMutationCommand::CheckScriptFlag(command) => {
                RuntimeMutationResult::ScriptFlagChecked(self.check_script_flag_in_session(
                    state,
                    session,
                    &command.map_name,
                    &command.source_script,
                    command.command_index,
                )?)
            }
            RuntimeMutationCommand::ApplyScriptScene(command) => {
                RuntimeMutationResult::ScriptSceneApplied(
                    self.apply_script_scene_command_in_session(
                        state,
                        session,
                        &command.map_name,
                        &command.source_script,
                        command.command_index,
                    )?,
                )
            }
            RuntimeMutationCommand::ApplyScriptBlockChange(command) => {
                RuntimeMutationResult::ScriptBlockChanged(
                    self.apply_script_block_change_in_session(
                        state,
                        session,
                        &command.map_name,
                        &command.source_script,
                        command.command_index,
                    )?,
                )
            }
            RuntimeMutationCommand::ApplyScriptAudio(command) => {
                RuntimeMutationResult::ScriptAudioApplied(
                    self.apply_script_audio_command_in_session(
                        state,
                        session,
                        &command.map_name,
                        &command.source_script,
                        command.command_index,
                        music_ids,
                        sound_effect_ids,
                        cry_ids,
                    )?,
                )
            }
            RuntimeMutationCommand::ApplyScriptMap(command) => {
                RuntimeMutationResult::ScriptMapApplied(self.apply_script_map_command_in_session(
                    state,
                    session,
                    &command.map_name,
                    &command.source_script,
                    command.command_index,
                )?)
            }
            RuntimeMutationCommand::ApplyRandomScriptMap(command) => {
                let mut next_state = state.clone();
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let action = self.apply_script_map_command_with_divider_in_session(
                    &mut next_state,
                    session,
                    &command.command.map_name,
                    &command.command.source_script,
                    command.command.command_index,
                    &mut divider,
                )?;
                require_consumed_divider_trace("apply random script map command", &divider)?;
                *state = next_state;
                RuntimeMutationResult::ScriptMapApplied(action)
            }
            RuntimeMutationCommand::TransitionPendingScriptWarp => {
                RuntimeMutationResult::PendingScriptWarpTransitioned(
                    self.transition_pending_script_warp(state, session, music_ids)?,
                )
            }
            RuntimeMutationCommand::TransitionToSpawnPoint {
                spawn_identifier,
                map_setup,
            } => {
                let spawn = self.runtime_spawn_point(spawn_identifier)?;
                let mut next_state = state.clone();
                let mut next_session = session.clone();
                self.transition_overworld_session_with_mode(
                    &mut next_state,
                    &mut next_session,
                    &spawn.map_name,
                    runtime_spawn_expected_tile(spawn),
                    MovementMode::Normal,
                    &map_setup,
                    SpawnMemoryUpdate::Preserve,
                    music_ids,
                )?;
                *state = next_state;
                *session = next_session;
                RuntimeMutationResult::SpawnPointTransitioned {
                    spawn_identifier,
                    map_name: spawn.map_name.clone(),
                }
            }
            RuntimeMutationCommand::ApplyMapSetupCallbacks { map_setup } => {
                let map_name = session.map.name.clone();
                self.apply_map_setup_callbacks(state, session, &map_name, &map_setup)?;
                let callback_mode =
                    self.map_entry_movement_mode(state, session, session.player.mode)?;
                if callback_mode != session.player.mode {
                    session.player.mode = callback_mode;
                    self.sync_current_map_music(state, &map_name, callback_mode, music_ids)?;
                }
                self.commit_overworld_snapshot(state, session, SpawnMemoryUpdate::Preserve);
                RuntimeMutationResult::MapSetupCallbacksApplied(map_setup.clone())
            }
            RuntimeMutationCommand::ApplyScriptText(command) => {
                RuntimeMutationResult::ScriptTextApplied(
                    self.apply_script_text_command_in_session(
                        state,
                        session,
                        &command.map_name,
                        &command.source_script,
                        command.command_index,
                    )?,
                )
            }
            RuntimeMutationCommand::ApplyScriptVariableNow(command) => {
                RuntimeMutationResult::ScriptVariableApplied(
                    self.apply_script_variable_command_now_in_mut_session(
                        state,
                        session,
                        &command.map_name,
                        &command.source_script,
                        command.command_index,
                    )?,
                )
            }
            RuntimeMutationCommand::ApplyScriptControl(command) => {
                RuntimeMutationResult::ScriptControlApplied(
                    self.apply_script_control_command_in_session(
                        state,
                        session,
                        &command.map_name,
                        &command.source_script,
                        command.command_index,
                    )?,
                )
            }
            RuntimeMutationCommand::ApplyScriptObjectMutation(command) => {
                RuntimeMutationResult::ScriptObjectMutated(
                    self.apply_script_object_mutation_in_session(
                        state,
                        session,
                        &command.map_name,
                        &command.source_script,
                        command.command_index,
                    )?,
                )
            }
            RuntimeMutationCommand::ApplyScriptMovement(command) => {
                RuntimeMutationResult::ScriptMovementApplied(
                    self.apply_script_movement_in_session(
                        state,
                        session,
                        &command.map_name,
                        &command.source_script,
                        command.command_index,
                    )?,
                )
            }
            RuntimeMutationCommand::ApplyScriptRuntime { command, inputs } => {
                let (command, outcome) = self.apply_script_runtime_command_in_session(
                    state,
                    session,
                    &command.map_name,
                    &command.source_script,
                    command.command_index,
                    inputs,
                )?;
                RuntimeMutationResult::ScriptRuntimeApplied(command, outcome)
            }
            RuntimeMutationCommand::ApplyRandomScriptRuntime(command) => {
                let mut next_state = state.clone();
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let (script_command, outcome) = self
                    .apply_random_script_runtime_command_in_session(
                        &mut next_state,
                        session,
                        &command.command.map_name,
                        &command.command.source_script,
                        command.command.command_index,
                        &mut divider,
                    )?;
                require_consumed_divider_trace("apply script random command", &divider)?;
                *state = next_state;
                RuntimeMutationResult::ScriptRuntimeApplied(script_command, outcome)
            }
            RuntimeMutationCommand::TakeNextScript => {
                let Some(script) = state.script_runtime.next_script.take() else {
                    anyhow::bail!("cannot take next script because no next script is queued");
                };
                RuntimeMutationResult::NextScriptTaken(script)
            }
            RuntimeMutationCommand::DrainScriptEventQueue(command) => {
                let drained = match command.queue {
                    RuntimeScriptEventQueue::Audio => RuntimeScriptEventDrainResult::Audio(
                        std::mem::take(&mut state.script_runtime.audio_events),
                    ),
                    RuntimeScriptEventQueue::Graphics => RuntimeScriptEventDrainResult::Graphics(
                        std::mem::take(&mut state.script_runtime.graphics_events),
                    ),
                    RuntimeScriptEventQueue::Money => RuntimeScriptEventDrainResult::Money(
                        std::mem::take(&mut state.script_runtime.money_events),
                    ),
                    RuntimeScriptEventQueue::Map => RuntimeScriptEventDrainResult::Map(
                        std::mem::take(&mut state.script_runtime.map_events),
                    ),
                    RuntimeScriptEventQueue::Text => RuntimeScriptEventDrainResult::Text(
                        std::mem::take(&mut state.script_runtime.text_events),
                    ),
                    RuntimeScriptEventQueue::Control => RuntimeScriptEventDrainResult::Control(
                        std::mem::take(&mut state.script_runtime.control_events),
                    ),
                    RuntimeScriptEventQueue::Shop => RuntimeScriptEventDrainResult::Shop(
                        std::mem::take(&mut state.script_runtime.shop_events),
                    ),
                    RuntimeScriptEventQueue::ItemUse => RuntimeScriptEventDrainResult::ItemUse(
                        std::mem::take(&mut state.script_runtime.item_use_events),
                    ),
                };
                RuntimeMutationResult::ScriptEventQueueDrained(drained)
            }
            RuntimeMutationCommand::DrainScriptRuntimeQueue(command) => {
                let drained = match command.queue {
                    RuntimeScriptRuntimeQueue::PendingDelay => {
                        let delays = std::mem::take(&mut state.script_runtime.pending_delays);
                        if delays.iter().any(|delay| delay.release_all_objects) {
                            state.script_runtime.all_input_locked = false;
                        }
                        RuntimeScriptRuntimeQueueDrainResult::PendingDelay(delays)
                    }
                    RuntimeScriptRuntimeQueue::PendingEarthquake => {
                        RuntimeScriptRuntimeQueueDrainResult::PendingEarthquake(std::mem::take(
                            &mut state.script_runtime.pending_earthquakes,
                        ))
                    }
                    RuntimeScriptRuntimeQueue::PendingEmote => {
                        RuntimeScriptRuntimeQueueDrainResult::PendingEmote(std::mem::take(
                            &mut state.script_runtime.pending_emotes,
                        ))
                    }
                    RuntimeScriptRuntimeQueue::Command => {
                        RuntimeScriptRuntimeQueueDrainResult::Command(std::mem::take(
                            &mut state.script_runtime.command_queue,
                        ))
                    }
                    RuntimeScriptRuntimeQueue::CallStack => {
                        RuntimeScriptRuntimeQueueDrainResult::CallStack(std::mem::take(
                            &mut state.script_runtime.call_stack,
                        ))
                    }
                    RuntimeScriptRuntimeQueue::DeferredScript => {
                        RuntimeScriptRuntimeQueueDrainResult::DeferredScript(std::mem::take(
                            &mut state.script_runtime.deferred_scripts,
                        ))
                    }
                    RuntimeScriptRuntimeQueue::MapReentryScript => {
                        let script =
                            state.script_runtime.map_reentry_script.take().context(
                                "cannot take map reentry script because none is pending",
                            )?;
                        if let Some(purchase) = state.pending_mom_purchase.as_ref() {
                            let expected = if purchase.decoration_flag.is_some() {
                                ".DollScript@Mom_GetScriptPointer"
                            } else {
                                ".ItemScript@Mom_GetScriptPointer"
                            };
                            anyhow::ensure!(
                                script.script == expected,
                                "pending Mom purchase map reentry script {} does not match exact target {expected}",
                                script.script
                            );
                            core_settle_pending_mom_purchase(state).map_err(|error| {
                                anyhow::anyhow!("settle map reentry Mom purchase: {error}")
                            })?;
                        }
                        RuntimeScriptRuntimeQueueDrainResult::MapReentryScript(vec![script])
                    }
                };
                RuntimeMutationResult::ScriptRuntimeQueueDrained(drained)
            }
            RuntimeMutationCommand::PopScriptCallStack => {
                let frame = state
                    .script_runtime
                    .call_stack
                    .pop()
                    .context("cannot pop script call stack because it is empty")?;
                RuntimeMutationResult::ScriptCallStackPopped(frame)
            }
            RuntimeMutationCommand::PopDeferredScript => {
                if state.script_runtime.deferred_scripts.is_empty() {
                    anyhow::bail!("cannot pop deferred script because queue is empty");
                }
                RuntimeMutationResult::DeferredScriptPopped(
                    state.script_runtime.deferred_scripts.remove(0),
                )
            }
            RuntimeMutationCommand::TakeScriptEndState => {
                let end = state
                    .script_runtime
                    .script_ended
                    .take()
                    .context("cannot take script end state because none is set")?;
                RuntimeMutationResult::ScriptEndStateTaken(end)
            }
            RuntimeMutationCommand::TakePendingScriptRequest(command) => {
                let request =
                    match command.kind {
                        RuntimePendingScriptRequestKind::MusicFade => {
                            RuntimePendingScriptRequest::MusicFade(
                                state.script_runtime.pending_music_fade.take().context(
                                    "cannot take pending music fade because none is pending",
                                )?,
                            )
                        }
                        RuntimePendingScriptRequestKind::ScreenFade => {
                            RuntimePendingScriptRequest::ScreenFade(
                                state.script_runtime.pending_screen_fade.take().context(
                                    "cannot take pending screen fade because none is pending",
                                )?,
                            )
                        }
                        RuntimePendingScriptRequestKind::ScriptWarp => {
                            RuntimePendingScriptRequest::ScriptWarp(
                                state.script_runtime.pending_script_warp.take().context(
                                    "cannot take pending script warp because none is pending",
                                )?,
                            )
                        }
                        RuntimePendingScriptRequestKind::MapLoad => {
                            let load =
                                state.script_runtime.pending_map_load.take().context(
                                    "cannot take pending map load because none is pending",
                                )?;
                            RuntimePendingScriptRequest::MapLoad(load)
                        }
                        RuntimePendingScriptRequestKind::MapRefresh => {
                            RuntimePendingScriptRequest::MapRefresh(
                                state.script_runtime.pending_map_refresh.take().context(
                                    "cannot take pending map refresh because none is pending",
                                )?,
                            )
                        }
                        RuntimePendingScriptRequestKind::TextLabel => {
                            let text_label =
                                state.script_runtime.pending_text_label.take().context(
                                    "cannot take pending text label because none is pending",
                                )?;
                            // Completing synchronous PrintText removes only
                            // its interpreter boundary. The rendered text
                            // remains the owner of the open box until another
                            // Write replaces it or CloseText closes it.
                            state.script_runtime.active_text_label = Some(text_label.clone());
                            RuntimePendingScriptRequest::TextLabel(text_label)
                        }
                        RuntimePendingScriptRequestKind::TextWait => {
                            let wait =
                                state.script_runtime.pending_text_wait.take().context(
                                    "cannot take pending text wait because none is pending",
                                )?;
                            if script_text_wait_closes_window(&wait.command) {
                                state.script_runtime.active_text_label = None;
                                state.script_runtime.pending_text_label = None;
                                state.script_runtime.text_window_open = false;
                            }
                            RuntimePendingScriptRequest::TextWait(wait)
                        }
                        RuntimePendingScriptRequestKind::YesNo => {
                            RuntimePendingScriptRequest::YesNo(
                                state.script_runtime.pending_yes_no.take().context(
                                    "cannot take pending yes/no prompt because none is pending",
                                )?,
                            )
                        }
                        RuntimePendingScriptRequestKind::Shop => RuntimePendingScriptRequest::Shop(
                            state
                                .script_runtime
                                .pending_shop
                                .take()
                                .context("cannot take pending shop because none is pending")?,
                        ),
                    };
                RuntimeMutationResult::PendingScriptRequestTaken(request)
            }
            RuntimeMutationCommand::ResolvePendingYesNo(command) => {
                let prompt = state
                    .script_runtime
                    .pending_yes_no
                    .take()
                    .context("cannot resolve pending yes/no prompt because none is pending")?;
                let script_value = if command.accepted { "1" } else { "0" }.to_string();
                state.script_runtime.script_value = Some(script_value.clone());
                RuntimeMutationResult::PendingYesNoResolved(RuntimePendingYesNoResolution {
                    prompt,
                    accepted: command.accepted,
                    script_value,
                })
            }
            RuntimeMutationCommand::OpenVerticalMenu(command) => {
                RuntimeMutationResult::VerticalMenuOpened(self.open_vertical_menu(state, command)?)
            }
            RuntimeMutationCommand::SelectVerticalMenuOption(command) => {
                RuntimeMutationResult::VerticalMenuOptionSelected(
                    self.select_vertical_menu_option(state, command)?,
                )
            }
            RuntimeMutationCommand::SelectElevatorFloor(command) => {
                RuntimeMutationResult::ElevatorFloorSelected(
                    self.select_elevator_floor(state, command)?,
                )
            }
            RuntimeMutationCommand::ConsumeScriptRuntimeFlag(command) => {
                let consumed = match command.flag {
                    RuntimeScriptRuntimeFlag::MapMusicRestartDisabled => {
                        if !state.script_runtime.map_music_restart_disabled {
                            anyhow::bail!(
                                "cannot consume map music restart-disabled flag because it is not set"
                            );
                        }
                        state.script_runtime.map_music_restart_disabled = false;
                        RuntimeScriptRuntimeFlagValue::MapMusicRestartDisabled
                    }
                    RuntimeScriptRuntimeFlag::MapMusicRequested => {
                        if !state.script_runtime.map_music_requested {
                            anyhow::bail!(
                                "cannot consume map music requested flag because it is not set"
                            );
                        }
                        self.sync_current_map_music(
                            state,
                            &session.map.name,
                            session.player.mode,
                            music_ids,
                        )?;
                        state.script_runtime.map_music_requested = false;
                        RuntimeScriptRuntimeFlagValue::MapMusicRequested
                    }
                    RuntimeScriptRuntimeFlag::WaitingForSoundEffect => {
                        if !state.script_runtime.waiting_for_sound_effect {
                            anyhow::bail!(
                                "cannot consume waiting-for-sound-effect flag because it is not set"
                            );
                        }
                        state.script_runtime.waiting_for_sound_effect = false;
                        RuntimeScriptRuntimeFlagValue::WaitingForSoundEffect
                    }
                    RuntimeScriptRuntimeFlag::ItemNotifyQueued => {
                        if !state.script_runtime.item_notify_queued {
                            anyhow::bail!(
                                "cannot consume item-notify queued flag because it is not set"
                            );
                        }
                        state.script_runtime.item_notify_queued = false;
                        RuntimeScriptRuntimeFlagValue::ItemNotifyQueued
                    }
                    RuntimeScriptRuntimeFlag::WarpSoundQueued => {
                        if !state.script_runtime.warp_sound_queued {
                            anyhow::bail!(
                                "cannot consume warp-sound queued flag because it is not set"
                            );
                        }
                        state.script_runtime.warp_sound_queued = false;
                        RuntimeScriptRuntimeFlagValue::WarpSoundQueued
                    }
                    RuntimeScriptRuntimeFlag::TeleportFromQueued => {
                        if !state.script_runtime.teleport_from_queued {
                            anyhow::bail!(
                                "cannot consume teleport-from queued flag because it is not set"
                            );
                        }
                        state.script_runtime.teleport_from_queued = false;
                        RuntimeScriptRuntimeFlagValue::TeleportFromQueued
                    }
                    RuntimeScriptRuntimeFlag::HallOfFameRequested => {
                        if !state.script_runtime.hall_of_fame_requested {
                            anyhow::bail!(
                                "cannot consume Hall of Fame requested flag because it is not set"
                            );
                        }
                        state.script_runtime.hall_of_fame_requested = false;
                        RuntimeScriptRuntimeFlagValue::HallOfFameRequested
                    }
                    RuntimeScriptRuntimeFlag::CreditsRequested => {
                        if !state.script_runtime.credits_requested {
                            anyhow::bail!(
                                "cannot consume credits requested flag because it is not set"
                            );
                        }
                        state.script_runtime.credits_requested = false;
                        RuntimeScriptRuntimeFlagValue::CreditsRequested
                    }
                    RuntimeScriptRuntimeFlag::ResetRequested => {
                        if !state.script_runtime.reset_requested {
                            anyhow::bail!(
                                "cannot consume reset requested flag because it is not set"
                            );
                        }
                        state.script_runtime.reset_requested = false;
                        RuntimeScriptRuntimeFlagValue::ResetRequested
                    }
                    RuntimeScriptRuntimeFlag::Menu2dRequested => {
                        if !state.script_runtime.menu_2d_requested {
                            anyhow::bail!(
                                "cannot consume 2D-menu requested flag because it is not set"
                            );
                        }
                        state.script_runtime.menu_2d_requested = false;
                        RuntimeScriptRuntimeFlagValue::Menu2dRequested
                    }
                };
                RuntimeMutationResult::ScriptRuntimeFlagConsumed(consumed)
            }
            RuntimeMutationCommand::TakeScriptRuntimeMemoryValue(command) => {
                let value =
                    match command.value {
                        RuntimeScriptRuntimeMemoryValue::ScriptValue => {
                            RuntimeScriptRuntimeMemoryValueTaken::ScriptValue(
                                state
                                    .script_runtime
                                    .script_value
                                    .take()
                                    .context("cannot take script value because none is set")?,
                            )
                        }
                        RuntimeScriptRuntimeMemoryValue::LastTalkedObject => {
                            RuntimeScriptRuntimeMemoryValueTaken::LastTalkedObject(
                                state.script_runtime.last_talked_object.take().context(
                                    "cannot take last talked object because none is set",
                                )?,
                            )
                        }
                    };
                RuntimeMutationResult::ScriptRuntimeMemoryValueTaken(value)
            }
            RuntimeMutationCommand::RemoveScriptRuntimeMemoryEntry(command) => {
                let removed = match command.entry {
                    RuntimeScriptRuntimeMemoryEntry::Variable => {
                        let value = state
                            .script_runtime
                            .variables
                            .remove(&command.key)
                            .with_context(|| {
                                format!(
                                    "cannot remove script variable {} because it is not set",
                                    command.key
                                )
                            })?;
                        RuntimeScriptRuntimeMemoryEntryRemoved::Variable {
                            key: command.key,
                            value,
                        }
                    }
                    RuntimeScriptRuntimeMemoryEntry::Memory => {
                        let value = state
                            .script_runtime
                            .memory
                            .remove(&command.key)
                            .with_context(|| {
                                format!(
                                    "cannot remove script memory {} because it is not set",
                                    command.key
                                )
                            })?;
                        RuntimeScriptRuntimeMemoryEntryRemoved::Memory {
                            key: command.key,
                            value,
                        }
                    }
                    RuntimeScriptRuntimeMemoryEntry::NamedBuffer => {
                        let value = state
                            .script_runtime
                            .named_buffers
                            .remove(&command.key)
                            .with_context(|| {
                                format!(
                                    "cannot remove named buffer {} because it is not set",
                                    command.key
                                )
                            })?;
                        RuntimeScriptRuntimeMemoryEntryRemoved::NamedBuffer {
                            key: command.key,
                            value,
                        }
                    }
                    RuntimeScriptRuntimeMemoryEntry::VariableSprite => {
                        let value = state
                            .script_runtime
                            .variable_sprites
                            .remove(&command.key)
                            .with_context(|| {
                                format!(
                                    "cannot remove variable sprite {} because it is not set",
                                    command.key
                                )
                            })?;
                        RuntimeScriptRuntimeMemoryEntryRemoved::VariableSprite {
                            key: command.key,
                            value,
                        }
                    }
                    RuntimeScriptRuntimeMemoryEntry::PhoneNumber => {
                        if !state.script_runtime.phone_numbers.remove(&command.key) {
                            anyhow::bail!(
                                "cannot remove phone number {} because it is not set",
                                command.key
                            );
                        }
                        let order_index = state
                            .script_runtime
                            .phone_number_order
                            .iter()
                            .position(|contact_id| {
                                contact_id.as_deref() == Some(command.key.as_str())
                            })
                            .with_context(|| {
                                format!(
                                    "phone number {} is missing from canonical phone-list order",
                                    command.key
                                )
                            })?;
                        state.script_runtime.phone_number_order[order_index] = None;
                        RuntimeScriptRuntimeMemoryEntryRemoved::PhoneNumber { key: command.key }
                    }
                };
                RuntimeMutationResult::ScriptRuntimeMemoryEntryRemoved(removed)
            }
            RuntimeMutationCommand::OpenScriptShop(command) => {
                RuntimeMutationResult::ScriptShopOpened(self.open_script_shop_in_session(
                    state,
                    session,
                    &command.map_name,
                    &command.source_script,
                    command.command_index,
                )?)
            }
            RuntimeMutationCommand::CloseActiveMenu => {
                let Some(menu) = state.script_runtime.active_menu.take() else {
                    anyhow::bail!("cannot close active menu because no runtime menu is active");
                };
                RuntimeMutationResult::ActiveMenuClosed(menu)
            }
            RuntimeMutationCommand::CloseRuntimeWindow => {
                if !state.script_runtime.window_open {
                    anyhow::bail!("cannot close runtime window because no runtime window is open");
                }
                state.script_runtime.window_open = false;
                RuntimeMutationResult::RuntimeWindowClosed
            }
            RuntimeMutationCommand::CloseTextWindow => {
                if !state.script_runtime.text_window_open {
                    anyhow::bail!("cannot close text window because no text window is open");
                }
                state.script_runtime.text_window_open = false;
                state.script_runtime.active_text_label = None;
                state.script_runtime.pending_text_label = None;
                state.script_runtime.pending_text_wait = None;
                state.script_runtime.pending_yes_no = None;
                RuntimeMutationResult::TextWindowClosed
            }
            RuntimeMutationCommand::CloseActivePokemonPicture => {
                let Some(species) = state.script_runtime.active_pokemon_picture.take() else {
                    anyhow::bail!(
                        "cannot close active Pokemon picture because no Pokemon picture is active"
                    );
                };
                RuntimeMutationResult::ActivePokemonPictureClosed(species)
            }
            RuntimeMutationCommand::CloseScriptShop => RuntimeMutationResult::ScriptShopClosed(
                core_close_active_shop(state)
                    .map_err(|error| anyhow::anyhow!("close script shop: {error}"))?,
            ),
            RuntimeMutationCommand::BuyShopItem(command) => RuntimeMutationResult::ShopItemBought(
                self.buy_shop_item(state, &command.item_id, command.quantity)?,
            ),
            RuntimeMutationCommand::SellShopItem(command) => RuntimeMutationResult::ShopItemSold(
                self.sell_shop_item(state, &command.item_id, command.quantity)?,
            ),
            RuntimeMutationCommand::ApplySpecialRoutine { routine } => {
                if runtime_special_routine_requires_divider_trace(&routine) {
                    anyhow::bail!(
                        "special routine {routine} requires an authoritative divider trace command"
                    );
                }
                let mut next_state = state.clone();
                let outcome = self.apply_special_routine(&mut next_state, &routine, music_ids)?;
                *state = next_state;
                RuntimeMutationResult::SpecialRoutineApplied(outcome)
            }
            RuntimeMutationCommand::ApplyRandomSpecialRoutine(command) => {
                if !runtime_special_routine_requires_divider_trace(&command.routine) {
                    anyhow::bail!(
                        "special routine {} must not declare a divider trace",
                        command.routine
                    );
                }
                let mut next_state = state.clone();
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let outcome = self.apply_random_special_routine(
                    &mut next_state,
                    &command.routine,
                    music_ids,
                    &mut divider,
                )?;
                require_consumed_divider_trace(
                    &format!("apply random special routine {}", command.routine),
                    &divider,
                )?;
                *state = next_state;
                RuntimeMutationResult::SpecialRoutineApplied(outcome)
            }
            RuntimeMutationCommand::ResolveBugContestCaughtMon { keep_new } => {
                let mut next_state = state.clone();
                let outcome =
                    resolve_bug_contest_caught_mon(&mut next_state, keep_new).map_err(|error| {
                        anyhow::anyhow!("resolve Bug Contest caught Pokemon: {error}")
                    })?;
                *state = next_state;
                RuntimeMutationResult::SpecialRoutineApplied(outcome)
            }
            RuntimeMutationCommand::RegisterKeyItem(command) => {
                self.validate_saved_item_reference("registered_key_item", &command.item_id)?;
                if !matches!(state.bag.key_items.get(&command.item_id), Some(quantity) if *quantity > 0)
                {
                    anyhow::bail!(
                        "cannot register key item {} because it is not carried",
                        command.item_id
                    );
                }
                let previous_item_id = state.registered_key_item.replace(command.item_id.clone());
                RuntimeMutationResult::KeyItemRegistered(RuntimeRegisteredKeyItemOutcome {
                    previous_item_id,
                    item_id: command.item_id,
                })
            }
            RuntimeMutationCommand::ApplyGraphicsSpecial(special) => {
                RuntimeMutationResult::GraphicsSpecialApplied(self.apply_special_routine(
                    state,
                    special.routine(),
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::ApplyPartyCheckSpecial(command) => {
                let routine = command.special.routine();
                let species_id = if command.special.requires_species() {
                    Some(command.species_id.with_context(|| {
                        format!("{routine} runtime command requires species_id")
                    })?)
                } else if command.species_id.is_some() {
                    anyhow::bail!("{routine} runtime command does not accept species_id");
                } else {
                    None
                };
                let threshold =
                    if command.special.requires_threshold() {
                        Some(command.threshold.with_context(|| {
                            format!("{routine} runtime command requires threshold")
                        })?)
                    } else if command.threshold.is_some() {
                        anyhow::bail!("{routine} runtime command does not accept threshold");
                    } else {
                        None
                    };
                RuntimeMutationResult::PartyCheckSpecialApplied(
                    self.apply_special_routine_transactional(
                        state,
                        routine,
                        music_ids,
                        |next_state| {
                            if let Some(species_id) = species_id {
                                next_state
                                    .script_runtime
                                    .variables
                                    .insert("_value".to_string(), species_id);
                            }
                            if let Some(threshold) = threshold {
                                next_state
                                    .script_runtime
                                    .variables
                                    .insert("_value".to_string(), threshold.to_string());
                            }
                            Ok(())
                        },
                    )?,
                )
            }
            RuntimeMutationCommand::ApplyPhoneRandomSpecial(command) => {
                let routine = command.special.routine();
                let mut next_state = state.clone();
                next_state
                    .script_runtime
                    .variables
                    .insert("VAR_CALLERID".to_string(), command.contact_id);
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let outcome = self.apply_random_special_routine(
                    &mut next_state,
                    routine,
                    music_ids,
                    &mut divider,
                )?;
                require_consumed_divider_trace(
                    &format!("apply phone random special {routine}"),
                    &divider,
                )?;
                *state = next_state;
                RuntimeMutationResult::PhoneRandomSpecialApplied(outcome)
            }
            RuntimeMutationCommand::CheckItemInPcOrBagSpecial(command) => {
                RuntimeMutationResult::ItemInPcOrBagChecked(
                    self.apply_special_routine_transactional(
                        state,
                        "UnusedFindItemInPCOrBag",
                        music_ids,
                        |next_state| {
                            next_state.script_runtime.script_value = Some(command.item_id.clone());
                            next_state
                                .script_runtime
                                .variables
                                .insert("_value".to_string(), command.item_id);
                            Ok(())
                        },
                    )?,
                )
            }
            RuntimeMutationCommand::CheckAnotherUsablePartyMonSpecial(command) => {
                RuntimeMutationResult::AnotherUsablePartyMonChecked(
                    self.apply_special_routine_transactional(
                        state,
                        "Function11ba38",
                        music_ids,
                        |next_state| {
                            next_state.script_runtime.variables.insert(
                                "_selected_party_index".to_string(),
                                command.party_index.to_string(),
                            );
                            Ok(())
                        },
                    )?,
                )
            }
            RuntimeMutationCommand::ActivateFishingSwarmSpecial(command) => {
                RuntimeMutationResult::FishingSwarmActivated(
                    self.apply_special_routine_transactional(
                        state,
                        "ActivateFishingSwarm",
                        music_ids,
                        |next_state| {
                            next_state.script_runtime.script_value =
                                Some(command.value.to_string());
                            next_state
                                .script_runtime
                                .variables
                                .insert("_value".to_string(), command.value.to_string());
                            Ok(())
                        },
                    )?,
                )
            }
            RuntimeMutationCommand::ApplyStoryGateSpecial(special) => {
                RuntimeMutationResult::StoryGateSpecialApplied(self.apply_special_routine(
                    state,
                    special.routine(),
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::GrantScriptedGiftPokemon(command) => {
                let mut next_state = state.clone();
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let outcome = self.grant_scripted_gift_pokemon_with_divider_in_session(
                    &mut next_state,
                    session,
                    &command.command.map_name,
                    &command.command.source_script,
                    command.command.command_index,
                    command.original_trainer_name,
                    command.original_trainer_id,
                    command.nickname_accepted,
                    command.nickname,
                    &mut divider,
                )?;
                require_consumed_divider_trace("grant scripted gift Pokemon", &divider)?;
                *state = next_state;
                RuntimeMutationResult::ScriptedGiftPokemonGranted(outcome)
            }
            RuntimeMutationCommand::AddPartyPokemon(command) => {
                RuntimeMutationResult::PartyPokemonAdded(self.grant_gift_pokemon_to_state(
                    state,
                    GiftPokemonRequest {
                        species_id: command.species_id,
                        level: command.level,
                        held_item_id: command.held_item_id,
                        nickname: command.nickname,
                        original_trainer_name: command.original_trainer_name,
                        original_trainer_id: command.original_trainer_id,
                        caught_data: None,
                        source_script: "RuntimeAddPartyPokemon".to_string(),
                        command_index: 0,
                        egg: false,
                        dvs: command.dvs,
                    },
                )?)
            }
            RuntimeMutationCommand::StartScriptedWildBattle(command) => {
                let mut next_state = state.clone();
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let start = self.start_scripted_wild_battle_in_session(
                    &mut next_state,
                    session,
                    &command.command.map_name,
                    &command.command.source_script,
                    command.command.command_index,
                    &mut divider,
                )?;
                require_consumed_divider_trace("start scripted wild battle", &divider)?;
                *state = next_state;
                RuntimeMutationResult::ScriptedWildBattleStarted(start)
            }
            RuntimeMutationCommand::StartScriptedTrainerBattle(command) => {
                RuntimeMutationResult::ScriptedTrainerBattleStarted(
                    self.start_scripted_trainer_battle_in_session(
                        state,
                        session,
                        &command.map_name,
                        &command.source_script,
                        command.command_index,
                    )?,
                )
            }
            RuntimeMutationCommand::ResolveMapTrainerInteraction(command) => {
                RuntimeMutationResult::MapTrainerInteractionResolved(
                    self.resolve_map_trainer_interaction(
                        state,
                        &session.map.name,
                        &command.command.map_name,
                        &command.command.source_script,
                        command.command.command_index,
                        command.defer_battle_start,
                    )?,
                )
            }
            RuntimeMutationCommand::CompleteScriptedWildBattle(command) => {
                let mut next_state = state.clone();
                let mut next_session = session.clone();
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                self.complete_scripted_wild_battle_in_session(
                    &mut next_state,
                    &mut next_session,
                    &command.origin,
                    command.terminal,
                    &mut divider,
                )?;
                require_consumed_divider_trace("complete scripted wild battle", &divider)?;
                *state = next_state;
                *session = next_session;
                RuntimeMutationResult::ScriptedWildBattleCompleted
            }
            RuntimeMutationCommand::CompleteScriptedTrainerBattle(command) => {
                let mut next_state = state.clone();
                let next_session = session.clone();
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let outcome = self.complete_scripted_trainer_battle_in_session(
                    &mut next_state,
                    &next_session,
                    &command.command.map_name,
                    &command.command.source_script,
                    command.command.command_index,
                    command.won,
                    command.can_lose,
                    &mut divider,
                )?;
                require_consumed_divider_trace("complete scripted trainer battle", &divider)?;
                *state = next_state;
                *session = next_session;
                RuntimeMutationResult::ScriptedTrainerBattleCompleted(outcome)
            }
            RuntimeMutationCommand::UseBagItem { item_id, context } => {
                RuntimeMutationResult::BagItemUsed(self.use_bag_item(state, &item_id, context)?)
            }
            RuntimeMutationCommand::ReplacePendingMoveLearn(command) => {
                RuntimeMutationResult::PendingMoveLearnReplaced(
                    self.replace_pending_move_learn(state, command.move_slot)?,
                )
            }
            RuntimeMutationCommand::DeclinePendingMoveLearn => {
                RuntimeMutationResult::PendingMoveLearnDeclined(
                    self.decline_pending_move_learn(state)?,
                )
            }
            RuntimeMutationCommand::UseBagRepelInField(command) => {
                RuntimeMutationResult::FieldRepelUsed(
                    self.use_bag_repel_in_field(state, &command.item_id)?,
                )
            }
            RuntimeMutationCommand::UseBagBicycleInField(command) => {
                RuntimeMutationResult::FieldBicycleUsed(self.use_bag_bicycle_in_field(
                    state,
                    session,
                    &command.item_id,
                )?)
            }
            RuntimeMutationCommand::UseBagItemfinderInField(command) => {
                RuntimeMutationResult::FieldItemfinderUsed(self.use_bag_itemfinder_in_field(
                    state,
                    session,
                    &command.item_id,
                )?)
            }
            RuntimeMutationCommand::UseBagSquirtbottleInField(command) => {
                RuntimeMutationResult::FieldSquirtbottleUsed(self.use_bag_squirtbottle_in_field(
                    state,
                    session,
                    &command.item_id,
                )?)
            }
            RuntimeMutationCommand::UseBagStoryKeyInField(command) => {
                RuntimeMutationResult::FieldStoryKeyUsed(self.use_bag_story_key_in_field(
                    state,
                    session,
                    &command.item_id,
                )?)
            }
            RuntimeMutationCommand::UseBagCoinCaseInField(command) => {
                RuntimeMutationResult::FieldCoinCaseUsed(
                    self.use_bag_coin_case_in_field(state, &command.item_id)?,
                )
            }
            RuntimeMutationCommand::UseBagBlueCardInField(command) => {
                RuntimeMutationResult::FieldBlueCardUsed(
                    self.use_bag_blue_card_in_field(state, &command.item_id)?,
                )
            }
            RuntimeMutationCommand::UseBagTownMapInField(command) => {
                RuntimeMutationResult::FieldTownMapUsed(self.use_bag_town_map_in_field(
                    state,
                    session,
                    &command.item_id,
                )?)
            }
            RuntimeMutationCommand::UseBagPokegearInField(command) => {
                RuntimeMutationResult::FieldPokegearUsed(
                    self.use_bag_pokegear_in_field(state, &command.item_id)?,
                )
            }
            RuntimeMutationCommand::UseBagBoxInField(command) => {
                RuntimeMutationResult::FieldBoxUsed(
                    self.use_bag_box_in_field(state, &command.item_id)?,
                )
            }
            RuntimeMutationCommand::UseBagEscapeRopeInField(command) => {
                RuntimeMutationResult::FieldEscapeRopeUsed(self.use_bag_escape_rope_in_session(
                    state,
                    session,
                    &command.item_id,
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::UseCutFieldMove(command) => {
                RuntimeMutationResult::CutFieldMoveUsed(self.use_cut_field_move(
                    state,
                    session,
                    command.party_index,
                    command.metatile_x,
                    command.metatile_y,
                )?)
            }
            RuntimeMutationCommand::UseWhirlpoolFieldMove(command) => {
                RuntimeMutationResult::WhirlpoolFieldMoveUsed(self.use_whirlpool_field_move(
                    state,
                    session,
                    command.party_index,
                    command.metatile_x,
                    command.metatile_y,
                )?)
            }
            RuntimeMutationCommand::QueueStrengthFromMenu(command) => {
                let mut next_state = state.clone();
                let mut next_session = session.clone();
                self.queue_strength_from_menu(
                    &mut next_state,
                    &mut next_session,
                    command.party_index,
                )?;
                *state = next_state;
                *session = next_session;
                RuntimeMutationResult::StrengthFromMenuQueued(RuntimeStrengthMenuOutcome {
                    party_index: command.party_index,
                    next_script: "Script_StrengthFromMenu".to_string(),
                })
            }
            RuntimeMutationCommand::UseFlashFieldMove(command) => {
                RuntimeMutationResult::FlashFieldMoveUsed(
                    self.use_flash_field_move(state, &session.map.name, command.party_index)?,
                )
            }
            RuntimeMutationCommand::UseSurfFieldMove(command) => {
                RuntimeMutationResult::SurfFieldMoveUsed(self.use_surf_field_move(
                    state,
                    session,
                    command.party_index,
                )?)
            }
            RuntimeMutationCommand::UseWaterfallFieldMove(command) => {
                RuntimeMutationResult::WaterfallFieldMoveUsed(self.use_waterfall_field_move(
                    state,
                    session,
                    command.party_index,
                )?)
            }
            RuntimeMutationCommand::UseFlyFieldMove(command) => {
                RuntimeMutationResult::FlyFieldMoveUsed(self.use_fly_field_move_in_session(
                    state,
                    session,
                    command.party_index,
                    command.destination_spawn_identifier,
                    &command.flypoint_flag,
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::UseDigFieldMove(command) => {
                RuntimeMutationResult::DigFieldMoveUsed(self.use_dig_field_move_in_session(
                    state,
                    session,
                    command.party_index,
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::UseTeleportFieldMove(command) => {
                RuntimeMutationResult::TeleportFieldMoveUsed(
                    self.use_teleport_field_move_in_session(
                        state,
                        session,
                        command.party_index,
                        music_ids,
                    )?,
                )
            }
            RuntimeMutationCommand::CommitPendingFieldTravel => {
                RuntimeMutationResult::FieldTravelCommitted(
                    self.commit_pending_field_travel(state, session, music_ids)?,
                )
            }
            RuntimeMutationCommand::QueueHeadbuttScript(command) => {
                let mut next_state = state.clone();
                let mut next_session = session.clone();
                let next_script = if command.from_menu {
                    "HeadbuttFromMenuScript"
                } else {
                    "HeadbuttScript"
                };
                self.queue_headbutt_script(
                    &mut next_state,
                    &mut next_session,
                    command.party_index,
                    command.from_menu,
                )?;
                *state = next_state;
                *session = next_session;
                RuntimeMutationResult::HeadbuttScriptQueued(RuntimeHeadbuttScriptOutcome {
                    party_index: command.party_index,
                    next_script: next_script.to_string(),
                })
            }
            RuntimeMutationCommand::QueueRockSmashFromMenu(command) => {
                let mut next_state = state.clone();
                let mut next_session = session.clone();
                let object_identifier = self.queue_rock_smash_from_menu(
                    &mut next_state,
                    &mut next_session,
                    command.party_index,
                )?;
                *state = next_state;
                *session = next_session;
                RuntimeMutationResult::RockSmashFromMenuQueued(RuntimeRockSmashMenuOutcome {
                    party_index: command.party_index,
                    object_identifier,
                    next_script: "RockSmashFromMenuScript".to_string(),
                })
            }
            RuntimeMutationCommand::ResolveRockMonEncounter(command) => {
                let mut next_state = state.clone();
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let outcome = self.resolve_rock_mon_encounter(
                    &mut next_state,
                    &session.map.name,
                    &command.command,
                    &mut divider,
                )?;
                require_consumed_divider_trace("resolve RockMonEncounter", &divider)?;
                *state = next_state;
                RuntimeMutationResult::RockMonEncounterResolved(outcome)
            }
            RuntimeMutationCommand::ResolveTreeMonEncounter(command) => {
                let mut next_state = state.clone();
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let outcome = self.resolve_tree_mon_encounter(
                    &mut next_state,
                    session,
                    &command.command,
                    &mut divider,
                )?;
                require_consumed_divider_trace("resolve TreeMonEncounter", &divider)?;
                *state = next_state;
                RuntimeMutationResult::TreeMonEncounterResolved(outcome)
            }
            RuntimeMutationCommand::QueueSweetScentFromMenu(command) => {
                let mut next_state = state.clone();
                let mut next_session = session.clone();
                self.queue_sweet_scent_from_menu(
                    &mut next_state,
                    &mut next_session,
                    command.party_index,
                )?;
                *state = next_state;
                *session = next_session;
                RuntimeMutationResult::SweetScentFromMenuQueued(RuntimeSweetScentMenuOutcome {
                    party_index: command.party_index,
                    next_script: ".SweetScent@SweetScentFromMenu".to_string(),
                })
            }
            RuntimeMutationCommand::ResolveSweetScentEncounter(command) => {
                let mut next_state = state.clone();
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let outcome = self.resolve_sweet_scent_encounter(
                    &mut next_state,
                    session,
                    &command.command,
                    &mut divider,
                )?;
                require_consumed_divider_trace("resolve SweetScentEncounter", &divider)?;
                *state = next_state;
                RuntimeMutationResult::SweetScentEncounterResolved(outcome)
            }
            RuntimeMutationCommand::UseBagItemOnPartyPokemon(command) => {
                let (item_use, item_effect) = self.use_bag_item_on_party_pokemon_now(
                    state,
                    &command.item_id,
                    command.party_index,
                )?;
                RuntimeMutationResult::PartyPokemonItemUsed(item_use, item_effect)
            }
            RuntimeMutationCommand::UseBagItemOnWholeParty(command) => {
                let (item_use, item_effect) =
                    self.use_bag_item_on_whole_party(state, &command.item_id)?;
                RuntimeMutationResult::WholePartyItemUsed(item_use, item_effect)
            }
            RuntimeMutationCommand::UseBagItemOnPartyMove(command) => {
                let (item_use, item_effect) = self.use_bag_pp_item_on_party_pokemon(
                    state,
                    &command.item_id,
                    command.party_index,
                    command.move_slot,
                )?;
                RuntimeMutationResult::PartyMoveItemUsed(item_use, item_effect)
            }
            RuntimeMutationCommand::UseBagTmHmOnPartyPokemon(command) => {
                let (item_use, learned_move) = self.use_bag_tmhm_on_party_pokemon(
                    state,
                    &command.item_id,
                    command.party_index,
                    command.replace_slot,
                )?;
                RuntimeMutationResult::TmHmItemUsed(item_use, learned_move)
            }
            RuntimeMutationCommand::UseBagItemOnActiveBattlePokemon(command) => {
                let (item_use, battle_item) =
                    self.use_bag_item_on_active_battle_pokemon(state, &command.item_id)?;
                RuntimeMutationResult::ActiveBattlePokemonItemUsed(item_use, battle_item)
            }
            RuntimeMutationCommand::UseBagItemOnBattlePartyPokemon(command) => {
                let (item_use, battle_item) = self.use_bag_item_on_battle_party_pokemon(
                    state,
                    &command.item_id,
                    command.party_index,
                )?;
                RuntimeMutationResult::BattlePartyPokemonItemUsed(item_use, battle_item)
            }
            RuntimeMutationCommand::UseBagItemOnBattlePartyMove(command) => {
                let (item_use, battle_item) = self.use_bag_item_on_battle_party_move(
                    state,
                    &command.item_id,
                    command.party_index,
                    command.move_slot,
                )?;
                RuntimeMutationResult::BattlePartyMoveItemUsed(item_use, battle_item)
            }
            RuntimeMutationCommand::ThrowBallAtActiveBattle(command) => {
                let mut next_state = state.clone();
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let outcome = self.throw_ball_at_active_battle_with_divider(
                    &mut next_state,
                    &command.item_id,
                    &mut divider,
                )?;
                require_consumed_divider_trace("throw ball at active battle", &divider)?;
                *state = next_state;
                RuntimeMutationResult::BallThrown(outcome)
            }
            RuntimeMutationCommand::CompleteActiveWildCapture(command) => {
                let mut next_state = state.clone();
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let completion = self.complete_active_wild_capture(
                    &mut next_state,
                    &command.outcome,
                    command.nickname.as_deref(),
                    &mut divider,
                )?;
                require_consumed_divider_trace("complete active wild capture", &divider)?;
                *state = next_state;
                RuntimeMutationResult::ActiveWildCaptureCompleted(completion)
            }
            RuntimeMutationCommand::SwitchActiveBattleParty(command) => {
                let outcome = switch_active_battle_party_index(
                    state,
                    command.party_index,
                    &self.items,
                )
                    .map_err(|error| anyhow::anyhow!("switch active battle party: {error}"))?;
                RuntimeMutationResult::ActiveBattlePartySwitched(outcome)
            }
            RuntimeMutationCommand::ResolveActiveBattleTurn(command) => {
                let mut next_state = state.clone();
                if let Some(item_id) = command.player_bag_item_id.as_deref() {
                    anyhow::ensure!(
                        matches!(
                            &command.player_action,
                            BattleAction::Item { item_id: action_item_id }
                                | BattleAction::PartyItem {
                                    item_id: action_item_id,
                                    ..
                                }
                                if action_item_id == item_id
                        ),
                        "recorded player Bag item {item_id} does not match the battle action",
                    );
                    self.use_bag_item(&mut next_state, item_id, ItemUseContext::Battle)?;
                }
                consume_enemy_ai_divider_trace(&mut next_state, &command.enemy_ai_divider_trace)?;
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let outcome = if let Some(selected_move_slot) = command.enemy_ai_selected_move_slot
                {
                    let enemy_action = command.enemy_action;
                    let recorded_enemy_action = enemy_action.clone();
                    let move_random_calls = command.enemy_move_ai_random_calls;
                    let post_order_random_calls = command.enemy_post_order_ai_random_calls;
                    let replaying_wild_ai = matches!(
                        next_state.battle,
                        BattleMemory::Wild { .. } | BattleMemory::StaticWild { .. }
                    );
                    let trainer_ai_move_flags = match &next_state.battle {
                        BattleMemory::Trainer { ai_move_flags, .. } => Some(*ai_move_flags),
                        BattleMemory::Wild { .. } | BattleMemory::StaticWild { .. } => None,
                        BattleMemory::Inactive => {
                            unreachable!("an AI-selected move requires an active battle")
                        }
                    };
                    let trainer_post_context = match &next_state.battle {
                        BattleMemory::Trainer {
                            trainer_id,
                            battle_type,
                            ai_item_switch_flags,
                            ..
                        } => Some((
                            trainer_id.clone(),
                            battle_type.clone(),
                            *ai_item_switch_flags,
                        )),
                        BattleMemory::Wild { .. } | BattleMemory::StaticWild { .. } => None,
                        BattleMemory::Inactive => {
                            unreachable!("an AI-selected move requires an active battle")
                        }
                    };
                    let trainer_items_used = std::rc::Rc::new(std::cell::RefCell::new(
                        next_state
                            .script_runtime
                            .active_battle_combat
                            .as_ref()
                            .map(|combat| combat.trainer_items_used.clone())
                            .unwrap_or_default(),
                    ));
                    let move_calls_observed = std::rc::Rc::new(std::cell::Cell::new(0_u16));
                    let move_calls_for_selector = std::rc::Rc::clone(&move_calls_observed);
                    let mut select_enemy_move = move |combat: &BattleCombatState,
                                                      rng: &mut dyn BattleRandomSource|
                          -> std::result::Result<EnemyMoveSelection, BattleTurnError> {
                        struct CountingRandom<'a> {
                            inner: &'a mut dyn BattleRandomSource,
                            calls: &'a std::cell::Cell<u16>,
                            recorded_limit: u16,
                        }
                        impl BattleRandomSource for CountingRandom<'_> {
                            fn battle_random_byte(&mut self) -> u8 {
                                let call = self.calls.get();
                                self.calls.set(call.saturating_add(1));
                                if call >= self.recorded_limit {
                                    // A forged or stale call count must fail atomically instead of
                                    // letting an ASM-faithful rejection loop sample forever after
                                    // the finite replay trace is exhausted. Cycling the four slot
                                    // residues guarantees the selector terminates; the observed
                                    // count mismatch below then rejects the command.
                                    return call.wrapping_sub(self.recorded_limit) as u8 & 3;
                                }
                                self.inner.battle_random_byte()
                            }
                        }
                        let mut counting = CountingRandom {
                            inner: rng,
                            calls: &move_calls_for_selector,
                            recorded_limit: move_random_calls,
                        };
                        let actual = if let Some(ai_move_flags) = trainer_ai_move_flags {
                            self.select_trainer_enemy_move_with_scratch(
                                combat,
                                ai_move_flags,
                                &mut counting,
                            )
                            .map_err(|error| {
                                BattleTurnError::EnemyActionSelectionFailed {
                                    error: format!("recompute trainer enemy move: {error:#}"),
                                }
                            })?
                        } else {
                            EnemyMoveSelection {
                                slot: core_select_wild_enemy_move_slot(combat, &mut counting),
                                ai_damage_register: 0,
                            }
                        };
                        if actual.slot != selected_move_slot {
                            let battle_kind = if replaying_wild_ai { "wild" } else { "trainer" };
                            return Err(BattleTurnError::EnemyActionSelectionFailed {
                                error: format!(
                                    "recorded {battle_kind} enemy move slot {selected_move_slot} does not match recomputed slot {}",
                                    actual.slot
                                ),
                            });
                        }
                        Ok(actual)
                    };
                    let post_calls_observed = std::rc::Rc::new(std::cell::Cell::new(0_u16));
                    let post_calls_for_selector = std::rc::Rc::clone(&post_calls_observed);
                    let used_for_selector = std::rc::Rc::clone(&trainer_items_used);
                    let mut select_enemy_action = move |combat: &BattleCombatState,
                                                        rng: &mut dyn BattleRandomSource|
                          -> std::result::Result<
                        BattleAction,
                        BattleTurnError,
                    > {
                        if replaying_wild_ai {
                            let expected = BattleAction::Move {
                                slot: selected_move_slot,
                            };
                            if enemy_action != expected {
                                return Err(BattleTurnError::EnemyActionSelectionFailed {
                                    error: format!(
                                        "recorded wild enemy action {enemy_action:?} does not match recomputed action {expected:?}"
                                    ),
                                });
                            }
                            Ok(expected)
                        } else {
                            struct CountingRandom<'a> {
                                inner: &'a mut dyn BattleRandomSource,
                                calls: &'a std::cell::Cell<u16>,
                                recorded_limit: u16,
                            }
                            impl BattleRandomSource for CountingRandom<'_> {
                                fn battle_random_byte(&mut self) -> u8 {
                                    let call = self.calls.get();
                                    self.calls.set(call.saturating_add(1));
                                    if call >= self.recorded_limit {
                                        return call.wrapping_sub(self.recorded_limit) as u8 & 3;
                                    }
                                    self.inner.battle_random_byte()
                                }
                            }
                            let (trainer_id, battle_type, flags) = trainer_post_context
                                .as_ref()
                                .expect("trainer post-order context");
                            let mut counting = CountingRandom {
                                inner: rng,
                                calls: &post_calls_for_selector,
                                recorded_limit: post_order_random_calls,
                            };
                            let actual = self
                                .select_trainer_post_order_action(
                                    combat,
                                    trainer_id,
                                    battle_type,
                                    *flags,
                                    &mut used_for_selector.borrow_mut(),
                                    selected_move_slot,
                                    &mut counting,
                                )
                                .map_err(|error| BattleTurnError::EnemyActionSelectionFailed {
                                    error: format!(
                                        "recompute trainer post-order action: {error:#}"
                                    ),
                                })?;
                            if enemy_action != actual {
                                return Err(BattleTurnError::EnemyActionSelectionFailed {
                                    error: format!(
                                        "recorded trainer enemy action {enemy_action:?} does not match recomputed action {actual:?}"
                                    ),
                                });
                            }
                            Ok(actual)
                        }
                    };
                    let outcome = if let BattleAction::Ball { item_id } = &command.player_action {
                        self.resolve_active_battle_ball_turn_with_enemy_ai_actions_with_divider(
                            &mut next_state,
                            item_id,
                            recorded_enemy_action,
                            &mut divider,
                            Some((&mut select_enemy_move, &mut select_enemy_action)),
                        )?
                        .1
                    } else {
                        self.resolve_active_battle_turn_with_enemy_ai_actions_with_divider(
                            &mut next_state,
                            command.player_action,
                            &mut divider,
                            &mut select_enemy_move,
                            &mut select_enemy_action,
                        )?
                    };
                    anyhow::ensure!(
                        move_calls_observed.get() == move_random_calls,
                        "recorded enemy move AI call count {} does not match recomputed count {}",
                        move_random_calls,
                        move_calls_observed.get(),
                    );
                    if replaying_wild_ai {
                        anyhow::ensure!(
                            post_order_random_calls == 0,
                            "recorded wild enemy post-order AI call count {} must be zero",
                            post_order_random_calls,
                        );
                    } else {
                        anyhow::ensure!(
                            post_calls_observed.get() == post_order_random_calls,
                            "recorded trainer post-order AI call count {} does not match recomputed count {}",
                            post_order_random_calls,
                            post_calls_observed.get(),
                        );
                        if let Some(combat) =
                            next_state.script_runtime.active_battle_combat.as_mut()
                        {
                            combat
                                .trainer_items_used
                                .extend(trainer_items_used.borrow().iter().cloned());
                        }
                    }
                    outcome
                } else {
                    anyhow::ensure!(
                        command.enemy_move_ai_random_calls == 0,
                        "enemy action recorded {} move-AI random calls without an AI-selected move",
                        command.enemy_move_ai_random_calls,
                    );
                    anyhow::ensure!(
                        command.enemy_post_order_ai_random_calls == 0,
                        "enemy action recorded {} post-order AI random calls without an AI-selected move",
                        command.enemy_post_order_ai_random_calls,
                    );
                    if let BattleAction::Ball { item_id } = &command.player_action {
                        self.resolve_active_battle_ball_turn_with_enemy_ai_actions_with_divider(
                            &mut next_state,
                            item_id,
                            command.enemy_action,
                            &mut divider,
                            None,
                        )?
                        .1
                    } else {
                        self.resolve_active_battle_turn_with_divider(
                            &mut next_state,
                            command.player_action,
                            command.enemy_action,
                            &mut divider,
                        )?
                    }
                };
                require_consumed_divider_trace("resolve active battle turn", &divider)?;
                *state = next_state;
                RuntimeMutationResult::ActiveBattleTurnResolved(outcome)
            }
            RuntimeMutationCommand::ResolveActiveBattleCommand(command) => {
                let mut next_state = state.clone();
                anyhow::ensure!(
                    command.player_bag_item_id.is_none(),
                    "battle command cannot consume a player Bag item",
                );
                anyhow::ensure!(
                    command.enemy_ai_selected_move_slot.is_none()
                        && command.enemy_move_ai_random_calls == 0
                        && command.enemy_post_order_ai_random_calls == 0,
                    "battle command cannot carry turn-only enemy AI selection state",
                );
                consume_enemy_ai_divider_trace(&mut next_state, &command.enemy_ai_divider_trace)?;
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let outcome = self.resolve_active_battle_command_with_divider(
                    &mut next_state,
                    command.player_action,
                    command.enemy_action,
                    &mut divider,
                )?;
                require_consumed_divider_trace("resolve active battle command", &divider)?;
                *state = next_state;
                RuntimeMutationResult::ActiveBattleCommandResolved(outcome)
            }
            RuntimeMutationCommand::ResolveActiveBattleEnemyAction(command) => {
                let mut next_state = state.clone();
                consume_enemy_ai_divider_trace(&mut next_state, &command.enemy_ai_divider_trace)?;
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let outcome = self.resolve_active_battle_enemy_action_with_divider(
                    &mut next_state,
                    command.enemy_action,
                    &mut divider,
                )?;
                require_consumed_divider_trace("resolve active battle enemy action", &divider)?;
                *state = next_state;
                RuntimeMutationResult::ActiveBattleEnemyActionResolved(outcome)
            }
            RuntimeMutationCommand::AttemptEscapeActiveWildBattle(command) => {
                let mut next_state = state.clone();
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let outcome = self
                    .resolve_active_wild_battle_run_with_divider(&mut next_state, &mut divider)?;
                require_consumed_divider_trace("attempt active wild battle escape", &divider)?;
                *state = next_state;
                RuntimeMutationResult::ActiveWildBattleEscapeAttempted(outcome)
            }
            RuntimeMutationCommand::UseBagItemToEscapeActiveWildBattle(command) => {
                let mut next_state = state.clone();
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let outcome = self.use_bag_item_to_escape_active_wild_battle_with_divider(
                    &mut next_state,
                    &command.item_id,
                    &mut divider,
                )?;
                require_consumed_divider_trace("use battle escape item", &divider)?;
                *state = next_state;
                RuntimeMutationResult::ActiveWildBattleEscapeItemUsed(outcome)
            }
            RuntimeMutationCommand::UseBagGuardSpecInActiveBattle(command) => {
                RuntimeMutationResult::ActiveBattleGuardSpecUsed(
                    self.use_bag_guard_spec_in_active_battle(state, &command.item_id)?,
                )
            }
            RuntimeMutationCommand::AdvanceActiveTrainerBattle => {
                RuntimeMutationResult::ActiveTrainerBattleAdvanced(
                    self.advance_active_trainer_battle(state)?,
                )
            }
            RuntimeMutationCommand::ClaimActiveTrainerBattleRewardsNow => {
                RuntimeMutationResult::ActiveTrainerBattleRewardsClaimed(
                    self.claim_active_trainer_battle_rewards_now(state)?,
                )
            }
            RuntimeMutationCommand::ClaimActiveWildBattleRewardsNow(trace) => {
                let mut next_state = state.clone();
                let mut divider = ReplayDivider::new(trace.samples.iter().copied());
                let outcome =
                    self.claim_active_wild_battle_rewards_now(&mut next_state, &mut divider)?;
                require_consumed_divider_trace("claim active wild battle rewards", &divider)?;
                *state = next_state;
                RuntimeMutationResult::ActiveWildBattleRewardsClaimed(outcome)
            }
            RuntimeMutationCommand::CastFishingRod(command) => {
                let mut next_state = state.clone();
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let outcome = self.cast_fishing_rod_in_session_with_divider(
                    &mut next_state,
                    session,
                    &command.rod,
                    &mut divider,
                )?;
                require_consumed_divider_trace("cast fishing rod", &divider)?;
                *state = next_state;
                RuntimeMutationResult::FishingRodCast(outcome)
            }
            RuntimeMutationCommand::UseBagFishingRodInField(command) => {
                let mut next_state = state.clone();
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let outcome = self.use_bag_fishing_rod_in_field_with_divider(
                    &mut next_state,
                    session,
                    &command.item_id,
                    &mut divider,
                )?;
                require_consumed_divider_trace("use bag fishing rod", &divider)?;
                *state = next_state;
                RuntimeMutationResult::BagFishingRodUsed(outcome)
            }
            RuntimeMutationCommand::AdvanceGameTimerVBlanks(command) => {
                if command.vblanks == 0 {
                    anyhow::bail!("game timer advance requires a nonzero VBlank count");
                }
                apply_normal_vblank_divider_trace(
                    state,
                    command.vblanks,
                    &command.normal_divider_trace,
                )?;
                let frames_before = state.time.game_time_frames;
                let seconds_before = state.time.game_time_seconds;
                let minutes_before = state.time.game_time_minutes;
                let hours_before = state.time.game_time_hours;
                state.advance_game_timer_vblanks(u64::from(command.vblanks));
                let counted = state.time.game_time_frames != frames_before
                    || state.time.game_time_seconds != seconds_before
                    || state.time.game_time_minutes != minutes_before
                    || state.time.game_time_hours != hours_before;
                RuntimeMutationResult::GameTimerVBlanksAdvanced(runtime_game_timer_outcome(
                    state, counted,
                ))
            }
            RuntimeMutationCommand::SetGameTimerCounting(command) => {
                state.set_game_timer_counting(command.counting);
                RuntimeMutationResult::GameTimerCountingSet(runtime_game_timer_outcome(
                    state, false,
                ))
            }
            RuntimeMutationCommand::SetGameLogicPaused(command) => {
                state.set_game_logic_paused(command.paused);
                RuntimeMutationResult::GameLogicPauseSet(runtime_game_timer_outcome(state, false))
            }
            RuntimeMutationCommand::UpdateClockFromDatetime(command) => {
                let mut next_state = state.clone();
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                self.update_clock_from_datetime(
                    &mut next_state,
                    command.date,
                    command.hour,
                    command.minute,
                    command.second,
                    &mut divider,
                )?;
                require_consumed_divider_trace("update clock from datetime", &divider)?;
                *state = next_state;
                RuntimeMutationResult::ClockUpdated
            }
            RuntimeMutationCommand::SetManualClockTime(command) => {
                let mut next_state = state.clone();
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                self.set_manual_clock_time(
                    &mut next_state,
                    command.now_date,
                    command.now_hour,
                    command.now_minute,
                    command.now_second,
                    command.target,
                    &mut divider,
                )?;
                require_consumed_divider_trace("set manual clock time", &divider)?;
                *state = next_state;
                RuntimeMutationResult::ManualClockSet
            }
            RuntimeMutationCommand::ApplyScriptSwarm(command) => {
                RuntimeMutationResult::ScriptSwarmApplied(
                    self.apply_script_swarm_command_in_session(
                        state,
                        session,
                        &command.map_name,
                        &command.source_script,
                        command.command_index,
                    )?,
                )
            }
            RuntimeMutationCommand::ExecuteNextQueuedScriptCommand => {
                let queued = state
                    .script_runtime
                    .command_queue
                    .first()
                    .cloned()
                    .context("cannot execute queued script command because the queue is empty")?;
                self.require_current_map(&session.map.name, &queued.origin_map_name)?;
                let definitions =
                    if let Some(module) = self.global_script_module_for(&queued.source_script) {
                        &module.definitions
                    } else {
                        &self.map_module(&queued.origin_map_name)?.scripts
                    };
                let resolved_target =
                    resolve_script_target_label(definitions, &queued.source_script, &queued.target)
                        .with_context(|| {
                            format!(
                                "queued script target {} cannot resolve from {} on {}",
                                queued.target, queued.source_script, queued.origin_map_name
                            )
                        })?;
                state.script_runtime.command_queue.remove(0);
                state.script_runtime.next_script = Some(ScriptLocation {
                    origin_map_name: queued.origin_map_name.clone(),
                    script: resolved_target.clone(),
                });
                state
                    .script_runtime
                    .control_events
                    .push(ScriptControlRuntimeEvent {
                        kind: ScriptControlRuntimeKind::Jump,
                        target_script: Some(resolved_target),
                        source_script: queued.source_script.clone(),
                        command_index: queued.command_index,
                    });
                RuntimeMutationResult::QueuedScriptCommandExecuted(queued)
            }
            RuntimeMutationCommand::UseDayCare(command) => {
                if matches!(command.action, RuntimeDayCareAction::CollectEgg)
                    && !matches!(command.caretaker, RuntimeDayCareCaretaker::Man)
                {
                    anyhow::bail!("Day Care egg collection is only available from the man");
                }
                let routine = match command.caretaker {
                    RuntimeDayCareCaretaker::Man => "DayCareMan",
                    RuntimeDayCareCaretaker::Lady => "DayCareLady",
                };
                let input = runtime_day_care_input(&command)?;
                let mut next_state = state.clone();
                next_state.script_runtime.pending_day_care_input = Some(input);
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let outcome = self.apply_random_special_routine(
                    &mut next_state,
                    routine,
                    music_ids,
                    &mut divider,
                )?;
                require_consumed_divider_trace("use Day Care", &divider)?;
                *state = next_state;
                RuntimeMutationResult::DayCareUsed(outcome)
            }
            RuntimeMutationCommand::CheckDayCareManOutsideSpecial(divider_trace) => {
                let mut next_state = state.clone();
                let mut divider = ReplayDivider::new(divider_trace.samples.iter().copied());
                let outcome = self.apply_random_special_routine(
                    &mut next_state,
                    "DayCareManOutside",
                    music_ids,
                    &mut divider,
                )?;
                require_consumed_divider_trace("check Day Care man outside", &divider)?;
                *state = next_state;
                RuntimeMutationResult::DayCareManOutsideChecked(outcome)
            }
            RuntimeMutationCommand::CheckDayCareResidentSpecial(caretaker) => {
                let routine = match caretaker {
                    RuntimeDayCareCaretaker::Man => "DayCareMon1",
                    RuntimeDayCareCaretaker::Lady => "DayCareMon2",
                };
                RuntimeMutationResult::DayCareResidentChecked(
                    self.apply_special_routine(state, routine, music_ids)?,
                )
            }
            RuntimeMutationCommand::UseBugContest(command) => {
                let action = command.action();
                let divider_trace = command.divider_trace();
                let routine = match action {
                    RuntimeBugContestAction::GiveParkBalls => "GiveParkBalls",
                    RuntimeBugContestAction::SelectContestants => {
                        "SelectRandomBugContestContestants"
                    }
                    RuntimeBugContestAction::DropOffMons => "ContestDropOffMons",
                    RuntimeBugContestAction::ReturnMons => "ContestReturnMons",
                    RuntimeBugContestAction::CheckPartyFull => "CheckPartyFullAfterContest",
                    RuntimeBugContestAction::Judge => "BugContestJudging",
                };
                let mut next_state = state.clone();
                next_state
                    .script_runtime
                    .variables
                    .remove("_bug_contest_rank");
                let outcome = if let Some(divider_trace) = divider_trace {
                    let mut divider = ReplayDivider::new(divider_trace.samples.iter().copied());
                    let outcome = self.apply_random_special_routine(
                        &mut next_state,
                        routine,
                        music_ids,
                        &mut divider,
                    )?;
                    require_consumed_divider_trace(
                        &format!(
                            "use Bug Contest {}",
                            runtime_bug_contest_action_name(action)
                        ),
                        &divider,
                    )?;
                    outcome
                } else {
                    self.apply_special_routine(&mut next_state, routine, music_ids)?
                };
                *state = next_state;
                RuntimeMutationResult::BugContestUsed(outcome)
            }
            RuntimeMutationCommand::UseKurtApricorn(command) => {
                RuntimeMutationResult::KurtApricornUsed(self.apply_special_routine_transactional(
                    state,
                    "SelectApricornForKurt",
                    music_ids,
                    |next_state| {
                        next_state
                            .script_runtime
                            .variables
                            .insert("_kurt_apricorn_type".to_string(), command.apricorn_id);
                        next_state.script_runtime.variables.insert(
                            "_kurt_apricorn_quantity".to_string(),
                            command.quantity.to_string(),
                        );
                        Ok(())
                    },
                )?)
            }
            RuntimeMutationCommand::UseBuenaPassword(command) => {
                let mut next_state = state.clone();
                match command.guess {
                    Some(guess) => {
                        next_state
                            .script_runtime
                            .variables
                            .insert("BUENA_PASSWORD".to_string(), guess);
                    }
                    None => {
                        next_state.script_runtime.variables.remove("BUENA_PASSWORD");
                    }
                }
                let outcome = self.apply_special_routine(
                    &mut next_state,
                    "BuenasPassword",
                    music_ids,
                )?;
                *state = next_state;
                RuntimeMutationResult::BuenaPasswordUsed(outcome)
            }
            RuntimeMutationCommand::UseBuenaPrize(command) => {
                RuntimeMutationResult::BuenaPrizeUsed(self.apply_special_routine_transactional(
                    state,
                    "BuenaPrize",
                    music_ids,
                    |next_state| {
                        next_state
                            .script_runtime
                            .variables
                            .insert("_selected_prize".to_string(), command.item_id);
                        next_state.script_runtime.variables.insert(
                            "_selected_prize_quantity".to_string(),
                            command.quantity.to_string(),
                        );
                        Ok(())
                    },
                )?)
            }
            RuntimeMutationCommand::UseShuckie(command) => {
                let mut next_state = state.clone();
                let outcome = match command {
                    RuntimeShuckieCommand::Give { divider_trace } => {
                        let mut divider = ReplayDivider::new(divider_trace.samples.iter().copied());
                        let outcome = self.apply_random_special_routine(
                            &mut next_state,
                            "GiveShuckle",
                            music_ids,
                            &mut divider,
                        )?;
                        require_consumed_divider_trace("use Shuckie give", &divider)?;
                        outcome
                    }
                    RuntimeShuckieCommand::Return { party_index } => {
                        match party_index {
                            Some(party_index) => {
                                next_state
                                    .script_runtime
                                    .variables
                                    .insert("_selection_cancelled".to_string(), "0".to_string());
                                next_state.script_runtime.variables.insert(
                                    "_selected_party_index".to_string(),
                                    party_index.to_string(),
                                );
                            }
                            None => {
                                next_state
                                    .script_runtime
                                    .variables
                                    .insert("_selection_cancelled".to_string(), "1".to_string());
                                next_state
                                    .script_runtime
                                    .variables
                                    .remove("_selected_party_index");
                            }
                        }
                        self.apply_special_routine(&mut next_state, "ReturnShuckie", music_ids)?
                    }
                };
                *state = next_state;
                RuntimeMutationResult::ShuckieUsed(outcome)
            }
            RuntimeMutationCommand::GiveOddEgg(command) => {
                let mut next_state = state.clone();
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let outcome = self.apply_random_special_routine(
                    &mut next_state,
                    "GiveOddEgg",
                    music_ids,
                    &mut divider,
                )?;
                require_consumed_divider_trace("give Odd Egg", &divider)?;
                *state = next_state;
                RuntimeMutationResult::OddEggGiven(outcome)
            }
            RuntimeMutationCommand::GiveDratini(command) => {
                RuntimeMutationResult::DratiniGiven(self.apply_special_routine_transactional(
                    state,
                    "GiveDratini",
                    music_ids,
                    |next_state| {
                        next_state.script_runtime.script_value = Some(command.mode.to_string());
                        next_state
                            .script_runtime
                            .variables
                            .insert("_value".to_string(), command.mode.to_string());
                        Ok(())
                    },
                )?)
            }
            RuntimeMutationCommand::UseBillsGrandfather(command) => {
                let (party_index, species_id) = runtime_bills_grandfather_inputs(&command)?;
                RuntimeMutationResult::BillsGrandfatherUsed(
                    self.apply_special_routine_transactional(
                        state,
                        "BillsGrandfather",
                        music_ids,
                        |next_state| {
                            match party_index {
                                Some(party_index) => {
                                    next_state.script_runtime.variables.insert(
                                        "_selected_party_index".to_string(),
                                        party_index.to_string(),
                                    );
                                    next_state
                                        .script_runtime
                                        .variables
                                        .remove("_selected_species");
                                }
                                None => {
                                    next_state
                                        .script_runtime
                                        .variables
                                        .remove("_selected_party_index");
                                }
                            }
                            match species_id {
                                Some(species_id) => {
                                    next_state
                                        .script_runtime
                                        .variables
                                        .insert("_selected_species".to_string(), species_id);
                                }
                                None => {
                                    next_state
                                        .script_runtime
                                        .variables
                                        .remove("_selected_species");
                                }
                            }
                            Ok(())
                        },
                    )?,
                )
            }
            RuntimeMutationCommand::InitRoamMons => RuntimeMutationResult::RoamersInitialized(
                self.apply_special_routine(state, "InitRoamMons", music_ids)?,
            ),
            RuntimeMutationCommand::CheckMagikarpLength(command) => {
                RuntimeMutationResult::MagikarpLengthChecked(
                    self.apply_special_routine_transactional(
                        state,
                        "CheckMagikarpLength",
                        music_ids,
                        |next_state| {
                            next_state
                                .script_runtime
                                .variables
                                .insert("_selection_cancelled".to_string(), "0".to_string());
                            next_state.script_runtime.variables.insert(
                                "_selected_party_index".to_string(),
                                command.party_index.to_string(),
                            );
                            Ok(())
                        },
                    )?,
                )
            }
            RuntimeMutationCommand::ShowProfOaksPcBoot => {
                RuntimeMutationResult::ProfOaksPcBootShown(self.apply_special_routine(
                    state,
                    "ProfOaksPCBoot",
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::ShowMagikarpHouseSign => {
                RuntimeMutationResult::MagikarpHouseSignShown(self.apply_special_routine(
                    state,
                    "MagikarpHouseSign",
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::ApplyBattleTowerAction(command) => {
                RuntimeMutationResult::BattleTowerActionApplied(
                    self.apply_special_routine_transactional(
                        state,
                        "BattleTowerAction",
                        music_ids,
                        |next_state| {
                            next_state.script_runtime.script_value = Some(command.action.clone());
                            next_state
                                .script_runtime
                                .variables
                                .insert("_value".to_string(), command.action);
                            Ok(())
                        },
                    )?,
                )
            }
            RuntimeMutationCommand::UseBattleTowerRoomMenu(command) => {
                RuntimeMutationResult::BattleTowerRoomMenuUsed(
                    self.apply_special_routine_transactional(
                        state,
                        "BattleTowerRoomMenu",
                        music_ids,
                        |next_state| {
                            match command.selection {
                                Some(selection) => {
                                    next_state.script_runtime.variables.insert(
                                        "_battle_tower_room_selection".to_string(),
                                        selection.to_string(),
                                    );
                                }
                                None => {
                                    next_state
                                        .script_runtime
                                        .variables
                                        .remove("_battle_tower_room_selection");
                                }
                            }
                            if command.cancelled {
                                next_state.script_runtime.variables.insert(
                                    "_battle_tower_room_cancelled".to_string(),
                                    "1".to_string(),
                                );
                            } else {
                                next_state
                                    .script_runtime
                                    .variables
                                    .remove("_battle_tower_room_cancelled");
                            }
                            Ok(())
                        },
                    )?,
                )
            }
            RuntimeMutationCommand::StartBattleTowerBattleSpecial(command) => {
                RuntimeMutationResult::BattleTowerBattleStarted(
                    self.apply_special_routine_transactional(
                        state,
                        "BattleTowerBattle",
                        music_ids,
                        |next_state| {
                            next_state.script_runtime.variables.insert(
                                "_battle_result".to_string(),
                                command.battle_result.to_string(),
                            );
                            Ok(())
                        },
                    )?,
                )
            }
            RuntimeMutationCommand::LoadBattleTowerOpponentSpecial(command) => {
                let mut next_state = state.clone();
                next_state.script_runtime.script_value = Some(command.target_object);
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let outcome = self.apply_random_special_routine(
                    &mut next_state,
                    "LoadOpponentTrainerAndPokemonWithOTSprite",
                    music_ids,
                    &mut divider,
                )?;
                require_consumed_divider_trace("load Battle Tower opponent", &divider)?;
                *state = next_state;
                RuntimeMutationResult::BattleTowerOpponentLoaded(outcome)
            }
            RuntimeMutationCommand::ShowBattleTowerMobileErrorSpecial => {
                RuntimeMutationResult::BattleTowerMobileErrorShown(self.apply_special_routine(
                    state,
                    "BattleTowerMobileError",
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::AskRememberPasswordSpecial(command) => {
                RuntimeMutationResult::RememberPasswordAsked(
                    self.apply_special_routine_transactional(
                        state,
                        "AskRememberPassword",
                        music_ids,
                        |next_state| {
                            next_state.script_runtime.variables.insert(
                                "_yes_no_result".to_string(),
                                u8::from(command.remember).to_string(),
                            );
                            Ok(())
                        },
                    )?,
                )
            }
            RuntimeMutationCommand::OpenBattleTowerLeaderboardSpecial => {
                RuntimeMutationResult::BattleTowerLeaderboardOpened(self.apply_special_routine(
                    state,
                    "Function1700ba",
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::ApplyMobileHandshakeSpecial(command) => {
                RuntimeMutationResult::MobileHandshakeApplied(
                    self.apply_special_routine_transactional(
                        state,
                        "Function1011f1",
                        music_ids,
                        |next_state| {
                            next_state.script_runtime.variables.insert(
                                "_mobile_adapter_status".to_string(),
                                u8::from(command.accepted).to_string(),
                            );
                            next_state.script_runtime.variables.insert(
                                "_mobile_adapter_secondary_status".to_string(),
                                u8::from(command.accepted).to_string(),
                            );
                            next_state
                                .script_runtime
                                .variables
                                .entry("_mobile_login_password".to_string())
                                .or_insert_with(String::new);
                            next_state
                                .script_runtime
                                .variables
                                .entry("_mobile_battle_timer".to_string())
                                .or_insert_with(|| "0,0,0".to_string());
                            Ok(())
                        },
                    )?,
                )
            }
            RuntimeMutationCommand::EndMobileSessionSpecial => {
                RuntimeMutationResult::MobileSessionEnded(self.apply_special_routine(
                    state,
                    "Function101220",
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::SetBattleTowerMobileFlagSpecial(flag) => {
                let routine = match flag {
                    RuntimeBattleTowerMobileFlag::Enabled => "Function103780",
                    RuntimeBattleTowerMobileFlag::Disabled => "Function1037c2",
                };
                RuntimeMutationResult::BattleTowerMobileFlagSet(
                    self.apply_special_routine(state, routine, music_ids)?,
                )
            }
            RuntimeMutationCommand::SelectThreeMobileMonsSpecial(command) => {
                RuntimeMutationResult::MobileThreeMonsSelected(
                    self.apply_special_routine_transactional(
                        state,
                        "Mobile_SelectThreeMons",
                        music_ids,
                        |next_state| {
                            let selected = command
                                .party_indexes
                                .iter()
                                .map(usize::to_string)
                                .collect::<Vec<_>>()
                                .join(",");
                            next_state
                                .script_runtime
                                .variables
                                .insert("_selected_party_indexes".to_string(), selected);
                            Ok(())
                        },
                    )?,
                )
            }
            RuntimeMutationCommand::ApplyHappinessService(command) => {
                let mut next_state = state.clone();
                let mut divider = ReplayDivider::new(command.divider_trace.samples.iter().copied());
                let outcome = self.apply_happiness_service_with_divider(
                    &mut next_state,
                    command.routine,
                    command.party_index,
                    music_ids,
                    &mut divider,
                )?;
                require_consumed_divider_trace("apply happiness service", &divider)?;
                *state = next_state;
                RuntimeMutationResult::HappinessServiceApplied(outcome)
            }
            RuntimeMutationCommand::UseMysteryGift(action) => {
                let routine = match action {
                    RuntimeMysteryGiftAction::Check => "CheckMysteryGift",
                    RuntimeMysteryGiftAction::ClaimItem => "GetMysteryGiftItem",
                    RuntimeMysteryGiftAction::Unlock => "UnlockMysteryGift",
                };
                RuntimeMutationResult::MysteryGiftUsed(
                    self.apply_special_routine(state, routine, music_ids)?,
                )
            }
            RuntimeMutationCommand::WarpToSpawnPoint => {
                let outcome = self.apply_special_routine(state, "WarpToSpawnPoint", music_ids)?;
                RuntimeMutationResult::SpawnPointWarped(outcome)
            }
            RuntimeMutationCommand::HealPartySpecial => {
                RuntimeMutationResult::PartyHealedBySpecial(self.apply_special_routine(
                    state,
                    "HealParty",
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::FadeOutMusicSpecial => {
                RuntimeMutationResult::MusicFadedOutBySpecial(self.apply_special_routine(
                    state,
                    "FadeOutMusic",
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::WaitSfxSpecial => RuntimeMutationResult::SoundEffectWaitQueued(
                self.apply_special_routine(state, "WaitSFX", music_ids)?,
            ),
            RuntimeMutationCommand::PlayMapMusicSpecial => {
                RuntimeMutationResult::MapMusicPlayedBySpecial(self.apply_special_routine(
                    state,
                    "PlayMapMusic",
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::RestartMapMusicSpecial => {
                RuntimeMutationResult::MapMusicRestartedBySpecial(self.apply_special_routine(
                    state,
                    "RestartMapMusic",
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::PlayCurMonCry(command) => {
                let species_id = runtime_special_cry_species(&command)?.to_string();
                RuntimeMutationResult::CurrentMonCryPlayed(
                    self.apply_special_routine_transactional(
                        state,
                        "PlayCurMonCry",
                        music_ids,
                        |next_state| {
                            next_state
                                .script_runtime
                                .variables
                                .insert("wCurPartySpecies".to_string(), species_id);
                            Ok(())
                        },
                    )?,
                )
            }
            RuntimeMutationCommand::PlaySlowCry(command) => {
                let species_id = runtime_special_cry_species(&command)?.to_string();
                RuntimeMutationResult::SlowCryPlayed(self.apply_special_routine_transactional(
                    state,
                    "PlaySlowCry",
                    music_ids,
                    |next_state| {
                        next_state
                            .script_runtime
                            .variables
                            .insert("_value".to_string(), species_id);
                        Ok(())
                    },
                )?)
            }
            RuntimeMutationCommand::OpenPokemonCenterPcSpecial => {
                RuntimeMutationResult::PokemonCenterPcOpened(self.apply_special_routine(
                    state,
                    "PokemonCenterPC",
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::OpenPlayersHousePcSpecial => {
                RuntimeMutationResult::PlayersHousePcOpened(self.apply_special_routine(
                    state,
                    "PlayersHousePC",
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::OpenOverworldTownMapSpecial => {
                RuntimeMutationResult::OverworldTownMapOpened(self.apply_special_routine(
                    state,
                    "OverworldTownMap",
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::OpenUnownPrinterSpecial => {
                RuntimeMutationResult::UnownPrinterOpened(self.apply_special_routine(
                    state,
                    "UnownPrinter",
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::OpenMapRadioSpecial(command) => {
                let station = runtime_map_radio_station(&command)?.to_string();
                RuntimeMutationResult::MapRadioOpened(self.apply_special_routine_transactional(
                    state,
                    "MapRadio",
                    music_ids,
                    |next_state| {
                        next_state
                            .script_runtime
                            .variables
                            .insert("_value".to_string(), station);
                        Ok(())
                    },
                )?)
            }
            RuntimeMutationCommand::NameRivalSpecial(command) => {
                RuntimeMutationResult::RivalNamed(self.apply_special_routine_transactional(
                    state,
                    "NameRival",
                    music_ids,
                    |next_state| {
                        next_state
                            .script_runtime
                            .variables
                            .insert("_rival_name".to_string(), command.rival_name);
                        Ok(())
                    },
                )?)
            }
            RuntimeMutationCommand::DeletePartyMoveSpecial(command) => {
                RuntimeMutationResult::PartyMoveDeletedBySpecial(
                    self.apply_special_routine_transactional(
                        state,
                        "MoveDeletion",
                        music_ids,
                        |next_state| {
                            next_state
                                .script_runtime
                                .variables
                                .insert("_party_slot".to_string(), command.party_index.to_string());
                            next_state
                                .script_runtime
                                .variables
                                .insert("_move_slot".to_string(), command.move_index.to_string());
                            Ok(())
                        },
                    )?,
                )
            }
            RuntimeMutationCommand::CheckPokerusSpecial => RuntimeMutationResult::PokerusChecked(
                self.apply_special_routine(state, "CheckPokerus", music_ids)?,
            ),
            RuntimeMutationCommand::RatePartyNicknameSpecial(command) => {
                RuntimeMutationResult::PartyNicknameRated(
                    self.apply_special_routine_transactional(
                        state,
                        "NameRater",
                        music_ids,
                        |next_state| {
                            next_state
                                .script_runtime
                                .variables
                                .insert("_party_slot".to_string(), command.party_index.to_string());
                            next_state
                                .script_runtime
                                .variables
                                .insert("_selected_nickname".to_string(), command.nickname);
                            Ok(())
                        },
                    )?,
                )
            }
            RuntimeMutationCommand::SeePartyPokemonSpecial(command) => {
                RuntimeMutationResult::PartyPokemonSeenBySeer(
                    self.apply_special_routine_transactional(
                        state,
                        "PokeSeer",
                        music_ids,
                        |next_state| {
                            next_state
                                .script_runtime
                                .variables
                                .insert("_party_slot".to_string(), command.party_index.to_string());
                            Ok(())
                        },
                    )?,
                )
            }
            RuntimeMutationCommand::TeachPartyMoveSpecial(command) => {
                RuntimeMutationResult::PartyMoveTaughtBySpecial(
                    self.apply_special_routine_transactional(
                        state,
                        "MoveTutor",
                        music_ids,
                        |next_state| {
                            next_state
                                .script_runtime
                                .variables
                                .insert("_party_slot".to_string(), command.party_index.to_string());
                            next_state
                                .script_runtime
                                .variables
                                .insert("_move".to_string(), command.move_id);
                            Ok(())
                        },
                    )?,
                )
            }
            RuntimeMutationCommand::OpenBankOfMomSpecial => RuntimeMutationResult::BankOfMomOpened(
                self.apply_special_routine(state, "BankOfMom", music_ids)?,
            ),
            RuntimeMutationCommand::OpenGameCornerSpecial(command) => {
                let divider_trace = runtime_game_corner_divider_trace(&command);
                let routine = match command.service {
                    RuntimeGameCornerService::SlotMachine => "SlotMachine",
                    RuntimeGameCornerService::CardFlip => "CardFlip",
                };
                let mut next_state = state.clone();
                let mut divider = ReplayDivider::new(divider_trace.samples.iter().copied());
                let outcome = self.apply_random_special_routine(
                    &mut next_state,
                    routine,
                    music_ids,
                    &mut divider,
                )?;
                require_consumed_divider_trace("open Game Corner service", &divider)?;
                *state = next_state;
                RuntimeMutationResult::GameCornerOpened(outcome)
            }
            RuntimeMutationCommand::OpenDisplayLinkRecordSpecial => {
                RuntimeMutationResult::DisplayLinkRecordOpened(self.apply_special_routine(
                    state,
                    "DisplayLinkRecord",
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::OpenTrainerHouseSpecial => {
                RuntimeMutationResult::TrainerHouseOpened(self.apply_special_routine(
                    state,
                    "TrainerHouse",
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::OpenPhotoStudioSpecial(command) => {
                RuntimeMutationResult::PhotoStudioOpened(self.apply_special_routine_transactional(
                    state,
                    "PhotoStudio",
                    music_ids,
                    |next_state| {
                        next_state
                            .script_runtime
                            .variables
                            .insert("_party_slot".to_string(), command.party_index.to_string());
                        Ok(())
                    },
                )?)
            }
            RuntimeMutationCommand::UseBattleTowerChallengeMenu(command) => {
                RuntimeMutationResult::BattleTowerChallengeMenuUsed(
                    self.apply_special_routine_transactional(
                        state,
                        "Menu_ChallengeExplanationCancel",
                        music_ids,
                        |next_state| {
                            let language = u8::from(command.english).to_string();
                            next_state.script_runtime.script_value = Some(language.clone());
                            next_state
                                .script_runtime
                                .variables
                                .insert("_value".to_string(), language);
                            match command.selection {
                                Some(selection) => {
                                    next_state.script_runtime.variables.insert(
                                        "_battle_tower_challenge_choice".to_string(),
                                        selection.to_string(),
                                    );
                                }
                                None => {
                                    next_state
                                        .script_runtime
                                        .variables
                                        .remove("_battle_tower_challenge_choice");
                                }
                            }
                            Ok(())
                        },
                    )?,
                )
            }
            RuntimeMutationCommand::SetPlayerPalette(command) => {
                RuntimeMutationResult::PlayerPaletteSet(self.apply_special_routine_transactional(
                    state,
                    "SetPlayerPalette",
                    music_ids,
                    |next_state| {
                        next_state.script_runtime.script_value =
                            Some(command.raw_value.to_string());
                        next_state
                            .script_runtime
                            .variables
                            .insert("_value".to_string(), command.raw_value.to_string());
                        Ok(())
                    },
                )?)
            }
            RuntimeMutationCommand::SetDayOfWeek => RuntimeMutationResult::DayOfWeekSet(
                self.apply_special_routine(state, "SetDayOfWeek", music_ids)?,
            ),
            RuntimeMutationCommand::UpdateTime => RuntimeMutationResult::TimeUpdated(
                self.apply_special_routine(state, "UpdateTime", music_ids)?,
            ),
            RuntimeMutationCommand::SetCableClubRequest(request) => {
                let routine = match request {
                    RuntimeCableClubRequest::Trade => "SetBitsForLinkTradeRequest",
                    RuntimeCableClubRequest::Battle => "SetBitsForBattleRequest",
                    RuntimeCableClubRequest::TimeCapsule => "SetBitsForTimeCapsuleRequest",
                };
                RuntimeMutationResult::CableClubRequestSet(
                    self.apply_special_routine(state, routine, music_ids)?,
                )
            }
            RuntimeMutationCommand::WaitForLinkedFriendSpecial(command) => {
                RuntimeMutationResult::LinkedFriendWaitedFor(
                    self.apply_special_routine_transactional(
                        state,
                        "WaitForLinkedFriend",
                        music_ids,
                        |next_state| {
                            next_state.link_session.serial_connection_status =
                                command.serial_connection_status;
                            Ok(())
                        },
                    )?,
                )
            }
            RuntimeMutationCommand::CheckLinkTimeoutReceptionistSpecial(command) => {
                RuntimeMutationResult::LinkTimeoutReceptionistChecked(
                    self.apply_special_routine_transactional(
                        state,
                        "CheckLinkTimeout_Receptionist",
                        music_ids,
                        |next_state| {
                            next_state.link_session.serial_connection_status =
                                command.serial_connection_status;
                            next_state.script_runtime.variables.insert(
                                "_other_player_link_mode".to_string(),
                                command.other_player_link_mode.to_string(),
                            );
                            Ok(())
                        },
                    )?,
                )
            }
            RuntimeMutationCommand::CheckBothSelectedSameRoomSpecial(command) => {
                RuntimeMutationResult::BothSelectedSameRoomChecked(
                    self.apply_special_routine_transactional(
                        state,
                        "CheckBothSelectedSameRoom",
                        music_ids,
                        |next_state| {
                            next_state.script_runtime.variables.insert(
                                "_other_player_room".to_string(),
                                command.other_player_room.to_string(),
                            );
                            Ok(())
                        },
                    )?,
                )
            }
            RuntimeMutationCommand::CloseLinkSpecial => RuntimeMutationResult::LinkClosed(
                self.apply_special_routine(state, "CloseLink", music_ids)?,
            ),
            RuntimeMutationCommand::WaitForOtherPlayerToExitSpecial => {
                RuntimeMutationResult::OtherPlayerExitWaitedFor(self.apply_special_routine(
                    state,
                    "WaitForOtherPlayerToExit",
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::FailedLinkToPastSpecial => {
                RuntimeMutationResult::LinkToPastFailed(self.apply_special_routine(
                    state,
                    "FailedLinkToPast",
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::OpenLinkRoomSpecial(room) => {
                let routine = match room {
                    RuntimeLinkRoomSpecial::TradeCenter => "TradeCenter",
                    RuntimeLinkRoomSpecial::Colosseum => "Colosseum",
                    RuntimeLinkRoomSpecial::TimeCapsule => "EnterTimeCapsule",
                };
                RuntimeMutationResult::LinkRoomOpened(
                    self.apply_special_routine(state, routine, music_ids)?,
                )
            }
            RuntimeMutationCommand::CheckTimeCapsuleCompatibilitySpecial => {
                RuntimeMutationResult::TimeCapsuleCompatibilityChecked(self.apply_special_routine(
                    state,
                    "CheckTimeCapsuleCompatibility",
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::TryQuickSaveSpecial => RuntimeMutationResult::QuickSaveTried(
                self.apply_special_routine(state, "TryQuickSave", music_ids)?,
            ),
            RuntimeMutationCommand::AskMobileOrCableSpecial => {
                RuntimeMutationResult::MobileOrCableAsked(self.apply_special_routine(
                    state,
                    "AskMobileOrCable",
                    music_ids,
                )?)
            }
            RuntimeMutationCommand::CableClubCheckWhichChrisSpecial(command) => {
                RuntimeMutationResult::CableClubChrisChecked(
                    self.apply_special_routine_transactional(
                        state,
                        "CableClubCheckWhichChris",
                        music_ids,
                        |next_state| {
                            next_state
                                .script_runtime
                                .variables
                                .insert("_player_gender".to_string(), command.gender);
                            Ok(())
                        },
                    )?,
                )
            }
            RuntimeMutationCommand::SwitchCurrentPcBox(command) => {
                if command.box_index >= MAX_PC_BOXES {
                    anyhow::bail!(
                        "PC box index {} is outside 0..{MAX_PC_BOXES}",
                        command.box_index
                    );
                }
                while state.storage.pc_boxes.len() <= command.box_index {
                    let next = state.storage.pc_boxes.len();
                    state.storage.pc_boxes.push(PcBox::new(next));
                }
                let before = state.current_pc_box;
                state.current_pc_box = command.box_index;
                RuntimeMutationResult::CurrentPcBoxSwitched(RuntimeStorageBoxSwitchOutcome {
                    box_index_before: before,
                    box_index_after: state.current_pc_box,
                })
            }
            RuntimeMutationCommand::NamePcBox(command) => {
                if command.box_index >= MAX_PC_BOXES {
                    anyhow::bail!(
                        "PC box index {} is outside 0..{MAX_PC_BOXES}",
                        command.box_index
                    );
                }
                if command.name.is_empty()
                    || command.name.chars().count() > 8
                    || command.name.trim() != command.name
                    || command.name.chars().any(char::is_control)
                {
                    anyhow::bail!("PC box name has invalid text {:?}", command.name);
                }
                while state.storage.pc_boxes.len() <= command.box_index {
                    let next = state.storage.pc_boxes.len();
                    state.storage.pc_boxes.push(PcBox::new(next));
                }
                let previous_name = std::mem::replace(
                    &mut state.storage.pc_boxes[command.box_index].name,
                    command.name.clone(),
                );
                RuntimeMutationResult::PcBoxNamed(RuntimeStorageBoxNameOutcome {
                    box_index: command.box_index,
                    previous_name,
                    name: command.name,
                })
            }
            RuntimeMutationCommand::DepositPartyPokemonToCurrentBox(command) => {
                if state.current_pc_box >= MAX_PC_BOXES {
                    anyhow::bail!(
                        "current PC box {} is outside 0..{MAX_PC_BOXES}",
                        state.current_pc_box
                    );
                }
                while state.storage.pc_boxes.len() <= state.current_pc_box {
                    let next = state.storage.pc_boxes.len();
                    state.storage.pc_boxes.push(PcBox::new(next));
                }
                let pokemon = state
                    .storage
                    .party
                    .pokemon
                    .get(command.party_index)
                    .with_context(|| {
                        format!("party index {} is outside party", command.party_index)
                    })?
                    .as_ref()
                    .with_context(|| {
                        format!("party index {} has no Pokemon", command.party_index)
                    })?;
                let nuzlocke_dead =
                    crate::nuzlocke::allows_party_storage_without_usable_replacement(
                        self.nuzlocke_rules,
                        pokemon,
                    );
                if state.storage.party.pokemon.iter().flatten().count() <= 1 && !nuzlocke_dead {
                    anyhow::bail!("cannot deposit the last party Pokemon");
                }
                if pokemon
                    .item
                    .as_deref()
                    .is_some_and(crystal_core::models::item::is_mail_item_id)
                {
                    anyhow::bail!("cannot deposit a Pokemon holding mail");
                }
                if !nuzlocke_dead
                    && !state
                    .storage
                    .party
                    .pokemon
                    .iter()
                    .enumerate()
                    .any(|(index, candidate)| {
                        index != command.party_index
                            && candidate
                                .as_ref()
                                .is_some_and(|pokemon| !pokemon.is_egg && pokemon.hp > 0)
                    })
                {
                    anyhow::bail!("cannot deposit the last usable party Pokemon");
                }
                let mut pokemon = pokemon.clone();
                restore_deposited_pokemon_pp(&self.moves, &mut pokemon)?;
                let box_slot = state.storage.pc_boxes[state.current_pc_box]
                    .next_open_slot()
                    .with_context(|| format!("PC box {} is full", state.current_pc_box))?;
                take_party_pokemon_compact(state, command.party_index)
                    .with_context(|| format!("deposit party index {}", command.party_index))?;
                state.storage.pc_boxes[state.current_pc_box]
                    .set_slot(box_slot, Some(pokemon.clone()));
                RuntimeMutationResult::PartyPokemonDeposited(RuntimeStorageDepositOutcome {
                    party_index: command.party_index,
                    box_index: state.current_pc_box,
                    box_slot,
                    pokemon,
                })
            }
            RuntimeMutationCommand::WithdrawCurrentBoxPokemonToParty(command) => {
                let party_index = state
                    .storage
                    .party
                    .next_open_slot()
                    .context("party is full")?;
                let Some(pc_box) = state.storage.pc_boxes.get_mut(state.current_pc_box) else {
                    anyhow::bail!("current PC box {} does not exist", state.current_pc_box);
                };
                let mut pokemon = pc_box
                    .pokemon
                    .get(command.box_slot)
                    .with_context(|| format!("box slot {} is outside PC box", command.box_slot))?
                    .clone()
                    .with_context(|| {
                        format!("box slot {} has no Pokemon to withdraw", command.box_slot)
                    })?;
                // This is the same source CalcLevel routine used by Day Care.
                pokemon.level = crystal_core::systems::special_routines::day_care_level_from_experience(
                    &pokemon, &self.growth_rates,
                ).context("calculate withdrawn Pokemon level from experience")?;
                rebuild_pc_party_stats(&mut pokemon, self.nuzlocke_rules);
                pc_box.set_slot(command.box_slot, None);
                pc_box.compact();
                state.storage.party.pokemon[party_index] = Some(pokemon.clone());
                state.sync_party_from_storage();
                RuntimeMutationResult::PcPokemonWithdrawn(RuntimeStorageWithdrawOutcome {
                    box_index: state.current_pc_box,
                    box_slot: command.box_slot,
                    party_index,
                    pokemon,
                })
            }
            RuntimeMutationCommand::ReleaseCurrentBoxPokemon(command) => {
                let Some(pc_box) = state.storage.pc_boxes.get_mut(state.current_pc_box) else {
                    anyhow::bail!("current PC box {} does not exist", state.current_pc_box);
                };
                let pokemon = pc_box
                    .pokemon
                    .get(command.box_slot)
                    .with_context(|| format!("box slot {} is outside PC box", command.box_slot))?
                    .clone()
                    .with_context(|| {
                        format!("box slot {} has no Pokemon to release", command.box_slot)
                    })?;
                if pokemon.is_egg {
                    anyhow::bail!("cannot release an Egg");
                }
                pc_box.set_slot(command.box_slot, None);
                pc_box.compact();
                RuntimeMutationResult::PcPokemonReleased(RuntimeStorageReleaseOutcome {
                    box_index: state.current_pc_box,
                    box_slot: command.box_slot,
                    pokemon,
                })
            }
            RuntimeMutationCommand::ReleasePartyPokemon(command) => {
                let party_index = command.party_index;
                let party = &state.storage.party;
                let pokemon = party.pokemon.get(party_index).and_then(Option::as_ref)
                    .with_context(|| format!("party slot {party_index} has no Pokemon to release"))?;
                // BillsPCDepositFuncRelease calls CheckMail_PreventBlackout
                // before IsMonAnEgg. CheckCurPartyMonFainted tests HP only.
                anyhow::ensure!(party.filled_slots() > 1, "cannot release the last party Pokemon");
                anyhow::ensure!(party.pokemon.iter().enumerate().any(|(index, other)|
                    index != party_index && other.as_ref().is_some_and(|other| other.hp > 0)),
                    "cannot release the last usable party Pokemon");
                anyhow::ensure!(!pokemon.item.as_deref().is_some_and(crystal_core::models::item::is_mail_item_id),
                    "cannot release a Pokemon holding mail");
                anyhow::ensure!(!pokemon.is_egg, "cannot release an Egg");
                RuntimeMutationResult::PartyPokemonReleased(take_party_pokemon_compact(state, party_index)?)
            }
            RuntimeMutationCommand::MovePcPokemonWithoutMail(command) => {
                let mut staged = state.clone();
                let source = command.source.clone();
                let target = command.target.clone();
                prepare_pc_move_locations(&mut staged, &source, &target)?;

                let mut pokemon = match source {
                    RuntimePokemonStorageLocation::Party { slot } => {
                        let selected = staged.storage.party.pokemon[slot]
                            .as_ref()
                            .context("validated party source is empty")?;
                        if selected
                            .item
                            .as_deref()
                            .is_some_and(crystal_core::models::item::is_mail_item_id)
                        {
                            anyhow::bail!("cannot move a party Pokemon holding mail");
                        }
                        let nuzlocke_dead =
                            crate::nuzlocke::allows_party_storage_without_usable_replacement(
                                self.nuzlocke_rules,
                                selected,
                            );
                        if staged.storage.party.filled_slots() <= 1 && !nuzlocke_dead {
                            anyhow::bail!("cannot move the last party Pokemon");
                        }
                        if !nuzlocke_dead
                            && !staged.storage.party.pokemon.iter().enumerate().any(
                            |(index, candidate)| {
                                index != slot
                                    && candidate
                                        .as_ref()
                                        .is_some_and(|pokemon| !pokemon.is_egg && pokemon.hp > 0)
                            },
                        ) {
                            anyhow::bail!("cannot move the last usable party Pokemon");
                        }
                        take_party_pokemon_compact(&mut staged, slot)
                            .context("remove party Pokemon for PC move")?
                    }
                    RuntimePokemonStorageLocation::Box { box_index, slot } => {
                        staged.storage.pc_boxes[box_index]
                            .remove_pokemon(slot)
                            .map_err(anyhow::Error::msg)?
                    }
                };

                let adjusted_target = adjust_pc_move_target_after_removal(&command.source, target);
                match adjusted_target.clone() {
                    RuntimePokemonStorageLocation::Party { slot } => {
                        if matches!(&command.source, RuntimePokemonStorageLocation::Box { .. }) {
                            // Move's CopyFromBox calls CalcBufferMonStats, which
                            // uses the stored level rather than calling CalcLevel.
                            rebuild_pc_party_stats(&mut pokemon, self.nuzlocke_rules);
                        }
                        insert_party_pokemon(&mut staged.storage.party, slot, pokemon)?;
                        staged.sync_party_from_storage();
                    }
                    RuntimePokemonStorageLocation::Box { box_index, slot } => {
                        restore_deposited_pokemon_pp(&self.moves, &mut pokemon)?;
                        staged.storage.pc_boxes[box_index]
                            .insert_pokemon(slot, pokemon)
                            .map_err(anyhow::Error::msg)?;
                        // MoveMonWOMail_InsertMon_SaveGame restores wCurBox
                        // after saving the destination and reloads that original box.
                    }
                }
                *state = staged;
                RuntimeMutationResult::PcPokemonMoved(RuntimeStorageMoveOutcome {
                    source: command.source,
                    target: adjusted_target,
                })
            }
            RuntimeMutationCommand::DepositBagItemToPc(command) => {
                let mut staged = state.clone();
                let item = self
                    .items
                    .get(&command.item_id)
                    .with_context(|| format!("unknown item {}", command.item_id))?;
                let removed = staged
                    .bag
                    .remove_item_at(item, command.stack_index, command.quantity)
                    .map_err(|error| anyhow::anyhow!("remove bag item for PC deposit: {error}"))?;
                if !removed {
                    anyhow::bail!(
                        "bag does not contain a single {} stack with quantity {}",
                        command.item_id,
                        command.quantity
                    );
                }
                let added = staged
                    .bag
                    .add_pc_item(item, command.quantity)
                    .map_err(|error| anyhow::anyhow!("add PC item: {error}"))?;
                if !added {
                    anyhow::bail!(
                        "PC item storage rejected {} x{}",
                        command.item_id,
                        command.quantity
                    );
                }
                // PlayerDepositItemMenu calls CheckRegisteredItem after the
                // transfer. A remaining duplicate keeps key-item registration.
                if let Some(registered) = staged.registered_key_item.as_ref() {
                    let definition = self.items.get(registered)
                        .context("registered PC-deposit item has no definition")?;
                    if !staged.bag.has_item(definition) {
                        staged.registered_key_item = None;
                    }
                }
                *state = staged;
                RuntimeMutationResult::BagItemDepositedToPc(RuntimePcItemTransferOutcome {
                    item_id: command.item_id,
                    quantity: command.quantity,
                    bag_quantity_after: state.bag.quantity(item),
                    pc_quantity_after: state.bag.pc_item_quantity(item),
                })
            }
            RuntimeMutationCommand::WithdrawPcItemToBag(command) => {
                let mut staged = state.clone();
                let item = self
                    .items
                    .get(&command.item_id)
                    .with_context(|| format!("unknown item {}", command.item_id))?;
                let removed = staged
                    .bag
                    .remove_pc_item_at(item, command.stack_index, command.quantity)
                    .map_err(|error| anyhow::anyhow!("remove PC item: {error}"))?;
                if !removed {
                    anyhow::bail!(
                        "PC item storage does not contain a single {} stack with quantity {}",
                        command.item_id,
                        command.quantity
                    );
                }
                let added = staged
                    .bag
                    .add_item(item, command.quantity)
                    .map_err(|error| anyhow::anyhow!("add bag item from PC: {error}"))?;
                if !added {
                    anyhow::bail!("bag rejected {} x{}", command.item_id, command.quantity);
                }
                *state = staged;
                RuntimeMutationResult::PcItemWithdrawnToBag(RuntimePcItemTransferOutcome {
                    item_id: command.item_id,
                    quantity: command.quantity,
                    bag_quantity_after: state.bag.quantity(item),
                    pc_quantity_after: state.bag.pc_item_quantity(item),
                })
            }
            RuntimeMutationCommand::TossPcItem(command) => {
                let item = self
                    .items
                    .get(&command.item_id)
                    .with_context(|| format!("unknown PC item {}", command.item_id))?;
                if item
                    .property
                    .split('|')
                    .any(|flag| flag.trim() == "CANT_TOSS")
                {
                    anyhow::bail!("PC item {} is too important to toss", command.item_id);
                }
                let removed = state
                    .bag
                    .remove_pc_item_at(item, command.stack_index, command.quantity)
                    .map_err(|error| anyhow::anyhow!("toss PC item: {error}"))?;
                if !removed {
                    anyhow::bail!(
                        "PC does not contain {} x{}",
                        command.item_id,
                        command.quantity
                    );
                }
                RuntimeMutationResult::PcItemTossed(RuntimePcItemTransferOutcome {
                    item_id: command.item_id,
                    quantity: command.quantity,
                    bag_quantity_after: state.bag.quantity(item),
                    pc_quantity_after: state.bag.pc_item_quantity(item),
                })
            }
            RuntimeMutationCommand::SetUpDecoration(command) => {
                RuntimeMutationResult::DecorationSetUp(self.set_up_decoration(
                    state,
                    &command.decoration_id,
                    command.side,
                )?)
            }
            RuntimeMutationCommand::PutAwayDecoration(command) => {
                RuntimeMutationResult::DecorationPutAway(self.put_away_decoration(
                    state,
                    command.category,
                    command.side,
                )?)
            }
            RuntimeMutationCommand::GiveBagItemToPartyPokemon(command) => {
                let mut staged_state = state.clone();
                let item = self
                    .items
                    .get(&command.item_id)
                    .with_context(|| format!("unknown item {}", command.item_id))?;
                if crystal_core::models::item::is_mail_item_id(&command.item_id) {
                    anyhow::bail!(
                        "Mail item {} requires the compose-Mail action",
                        command.item_id
                    );
                }
                let target = staged_state
                    .storage
                    .party
                    .pokemon
                    .get(command.party_index)
                    .with_context(|| {
                        format!("party index {} is outside party", command.party_index)
                    })?
                    .as_ref()
                    .with_context(|| {
                        format!(
                            "party index {} has no Pokemon for held item",
                            command.party_index
                        )
                    })?;
                if target.mail.is_some() {
                    anyhow::bail!(
                        "party index {} must remove MAIL before replacing its held item",
                        command.party_index
                    );
                }
                let previous_item_id = target.item.clone();
                if let Some(previous_item_id) = previous_item_id.as_deref() {
                    let previous_item = self
                        .items
                        .get(previous_item_id)
                        .with_context(|| format!("unknown held item {previous_item_id}"))?;
                    let added = staged_state
                        .bag
                        .add_item(previous_item, 1)
                        .map_err(|error| {
                            anyhow::anyhow!("return replaced held item to bag: {error}")
                        })?;
                    if !added {
                        anyhow::bail!("bag rejected replaced held item {previous_item_id}");
                    }
                }
                let removed = staged_state
                    .bag
                    .remove_item(item, 1)
                    .map_err(|error| anyhow::anyhow!("remove held item from bag: {error}"))?;
                if !removed {
                    anyhow::bail!("bag does not contain held item {}", command.item_id);
                }
                let pokemon = staged_state.storage.party.pokemon[command.party_index]
                    .as_mut()
                    .context("validated party Pokemon disappeared during held-item transfer")?;
                pokemon.item = Some(command.item_id.clone());
                let bag_quantity_after = staged_state.bag.quantity(item);
                staged_state.sync_party_from_storage();
                *state = staged_state;
                RuntimeMutationResult::PartyPokemonHeldItemGiven(RuntimeHeldItemTransferOutcome {
                    party_index: command.party_index,
                    item_id: command.item_id,
                    bag_quantity_after,
                })
            }
            RuntimeMutationCommand::ComposeBagMailToParty(command) => {
                let mut staged_state = state.clone();
                let item = self
                    .items
                    .get(&command.item_id)
                    .with_context(|| format!("unknown Mail item {}", command.item_id))?;
                if !crystal_core::models::item::is_mail_item_id(&command.item_id) {
                    anyhow::bail!("item {} is not an ASM Mail item", command.item_id);
                }
                let lines = command.message.split('\n').collect::<Vec<_>>();
                if lines.len() > 2
                    || lines.iter().any(|line| line.chars().count() > 16)
                    || command
                        .message
                        .chars()
                        .any(|character| character.is_control() && character != '\n')
                {
                    anyhow::bail!("Mail message must contain at most two 16-character lines");
                }
                let target = staged_state
                    .storage
                    .party
                    .pokemon
                    .get(command.party_index)
                    .with_context(|| {
                        format!("party index {} is outside party", command.party_index)
                    })?
                    .as_ref()
                    .with_context(|| {
                        format!("party index {} has no Pokemon", command.party_index)
                    })?;
                if target.is_egg {
                    anyhow::bail!("Mail cannot be attached to an Egg");
                }
                if target.mail.is_some() {
                    anyhow::bail!("party Pokemon must remove MAIL before receiving Mail");
                }
                let previous_item_id = target.item.clone();
                let species = target.species.id.clone();
                if let Some(previous_item_id) = previous_item_id.as_deref() {
                    let previous_item = self
                        .items
                        .get(previous_item_id)
                        .with_context(|| format!("unknown held item {previous_item_id}"))?;
                    let added = staged_state
                        .bag
                        .add_item(previous_item, 1)
                        .map_err(|error| {
                            anyhow::anyhow!("return replaced held item to bag: {error}")
                        })?;
                    if !added {
                        anyhow::bail!("bag rejected replaced held item {previous_item_id}");
                    }
                }
                let removed = staged_state
                    .bag
                    .remove_item(item, 1)
                    .map_err(|error| anyhow::anyhow!("remove Mail item from bag: {error}"))?;
                if !removed {
                    anyhow::bail!("bag does not contain Mail item {}", command.item_id);
                }
                let mail = crystal_core::models::pokemon::MailData {
                    message: command.message,
                    author: staged_state.player_name.clone(),
                    nationality: 0,
                    author_id: staged_state.player_id,
                    species,
                    mail_type: command.item_id.clone(),
                };
                let pokemon = staged_state.storage.party.pokemon[command.party_index]
                    .as_mut()
                    .context("validated party Pokemon disappeared during Mail composition")?;
                pokemon.item = Some(command.item_id.clone());
                pokemon.mail = Some(mail.clone());
                let bag_quantity_after = staged_state.bag.quantity(item);
                staged_state.sync_party_from_storage();
                *state = staged_state;
                RuntimeMutationResult::PartyMailComposed(RuntimeMailTransferOutcome {
                    party_index: Some(command.party_index),
                    mailbox_index: None,
                    item_id: command.item_id,
                    mail,
                    mailbox_count_after: state.mailbox.len(),
                    bag_quantity_after,
                })
            }
            RuntimeMutationCommand::TakeHeldItemFromPartyPokemon(command) => {
                let pokemon = state
                    .storage
                    .party
                    .pokemon
                    .get_mut(command.party_index)
                    .with_context(|| {
                        format!("party index {} is outside party", command.party_index)
                    })?
                    .as_mut()
                    .with_context(|| {
                        format!(
                            "party index {} has no Pokemon for held item",
                            command.party_index
                        )
                    })?;
                if pokemon.mail.is_some() {
                    anyhow::bail!(
                        "party index {} must use the MAIL action before removing its held item",
                        command.party_index
                    );
                }
                let item_id = pokemon.item.clone().with_context(|| {
                    format!("party index {} holds no item", command.party_index)
                })?;
                let item = self
                    .items
                    .get(&item_id)
                    .with_context(|| format!("unknown held item {item_id}"))?;
                let added = state
                    .bag
                    .add_item(item, 1)
                    .map_err(|error| anyhow::anyhow!("return held item to bag: {error}"))?;
                if !added {
                    anyhow::bail!("bag rejected held item {item_id}");
                }
                pokemon.item = None;
                RuntimeMutationResult::PartyPokemonHeldItemTaken(RuntimeHeldItemTransferOutcome {
                    party_index: command.party_index,
                    item_id,
                    bag_quantity_after: state.bag.quantity(item),
                })
            }
            RuntimeMutationCommand::SendPartyMailToMailbox(command) => {
                if state.mailbox.len() >= crystal_core::state::MAILBOX_CAPACITY {
                    anyhow::bail!("mailbox is full");
                }
                let mut staged_state = state.clone();
                let pokemon = staged_state
                    .storage
                    .party
                    .pokemon
                    .get_mut(command.party_index)
                    .with_context(|| {
                        format!("party index {} is outside party", command.party_index)
                    })?
                    .as_mut()
                    .with_context(|| {
                        format!("party index {} has no Pokemon", command.party_index)
                    })?;
                let item_id = pokemon.item.take().with_context(|| {
                    format!("party index {} holds no Mail item", command.party_index)
                })?;
                let mail = pokemon.mail.take().with_context(|| {
                    format!("party index {} has no Mail message", command.party_index)
                })?;
                let mailbox_index = staged_state.mailbox.len();
                staged_state.mailbox.push(crystal_core::state::MailboxMail {
                    item_id: item_id.clone(),
                    mail: mail.clone(),
                });
                staged_state.sync_party_from_storage();
                *state = staged_state;
                RuntimeMutationResult::PartyMailSentToMailbox(RuntimeMailTransferOutcome {
                    party_index: Some(command.party_index),
                    mailbox_index: Some(mailbox_index),
                    item_id,
                    mail,
                    mailbox_count_after: state.mailbox.len(),
                    bag_quantity_after: 0,
                })
            }
            RuntimeMutationCommand::DiscardPartyMailToBag(command) => {
                let mut staged_state = state.clone();
                let pokemon = staged_state
                    .storage
                    .party
                    .pokemon
                    .get_mut(command.party_index)
                    .with_context(|| {
                        format!("party index {} is outside party", command.party_index)
                    })?
                    .as_mut()
                    .with_context(|| {
                        format!("party index {} has no Pokemon", command.party_index)
                    })?;
                let item_id = pokemon.item.clone().with_context(|| {
                    format!("party index {} holds no Mail item", command.party_index)
                })?;
                let mail = pokemon.mail.clone().with_context(|| {
                    format!("party index {} has no Mail message", command.party_index)
                })?;
                let item = self
                    .items
                    .get(&item_id)
                    .with_context(|| format!("unknown Mail item {item_id}"))?;
                let added = staged_state
                    .bag
                    .add_item(item, 1)
                    .map_err(|error| anyhow::anyhow!("return Mail item to bag: {error}"))?;
                if !added {
                    anyhow::bail!("bag rejected Mail item {item_id}");
                }
                let pokemon = staged_state.storage.party.pokemon[command.party_index]
                    .as_mut()
                    .context("validated party Pokemon disappeared during Mail transfer")?;
                pokemon.item = None;
                pokemon.mail = None;
                let bag_quantity_after = staged_state.bag.quantity(item);
                staged_state.sync_party_from_storage();
                *state = staged_state;
                RuntimeMutationResult::PartyMailDiscardedToBag(RuntimeMailTransferOutcome {
                    party_index: Some(command.party_index),
                    mailbox_index: None,
                    item_id,
                    mail,
                    mailbox_count_after: state.mailbox.len(),
                    bag_quantity_after,
                })
            }
            RuntimeMutationCommand::DeleteMailboxMail(command) => {
                if command.mailbox_index >= state.mailbox.len() {
                    anyhow::bail!("mailbox index {} is outside mailbox", command.mailbox_index);
                }
                let removed = state.mailbox.remove(command.mailbox_index);
                RuntimeMutationResult::MailboxMailDeleted(RuntimeMailTransferOutcome {
                    party_index: None,
                    mailbox_index: Some(command.mailbox_index),
                    item_id: removed.item_id,
                    mail: removed.mail,
                    mailbox_count_after: state.mailbox.len(),
                    bag_quantity_after: 0,
                })
            }
            RuntimeMutationCommand::MoveMailboxMailToBag(command) => {
                let mut staged_state = state.clone();
                let entry = staged_state
                    .mailbox
                    .get(command.mailbox_index)
                    .with_context(|| {
                        format!("mailbox index {} is outside mailbox", command.mailbox_index)
                    })?
                    .clone();
                let item = self
                    .items
                    .get(&entry.item_id)
                    .with_context(|| format!("unknown Mail item {}", entry.item_id))?;
                let added = staged_state
                    .bag
                    .add_item(item, 1)
                    .map_err(|error| anyhow::anyhow!("put Mail item in bag: {error}"))?;
                if !added {
                    anyhow::bail!("bag rejected Mail item {}", entry.item_id);
                }
                staged_state.mailbox.remove(command.mailbox_index);
                let bag_quantity_after = staged_state.bag.quantity(item);
                *state = staged_state;
                RuntimeMutationResult::MailboxMailMovedToBag(RuntimeMailTransferOutcome {
                    party_index: None,
                    mailbox_index: Some(command.mailbox_index),
                    item_id: entry.item_id,
                    mail: entry.mail,
                    mailbox_count_after: state.mailbox.len(),
                    bag_quantity_after,
                })
            }
            RuntimeMutationCommand::AttachMailboxMailToParty(command) => {
                let mut staged_state = state.clone();
                let entry = staged_state
                    .mailbox
                    .get(command.mailbox_index)
                    .with_context(|| {
                        format!("mailbox index {} is outside mailbox", command.mailbox_index)
                    })?
                    .clone();
                let pokemon = staged_state
                    .storage
                    .party
                    .pokemon
                    .get_mut(command.party_index)
                    .with_context(|| {
                        format!("party index {} is outside party", command.party_index)
                    })?
                    .as_mut()
                    .with_context(|| {
                        format!("party index {} has no Pokemon", command.party_index)
                    })?;
                if pokemon.is_egg {
                    anyhow::bail!("Mail cannot be attached to an Egg");
                }
                if pokemon.item.is_some() {
                    anyhow::bail!("party Pokemon is already holding an item");
                }
                pokemon.item = Some(entry.item_id.clone());
                pokemon.mail = Some(entry.mail.clone());
                staged_state.mailbox.remove(command.mailbox_index);
                staged_state.sync_party_from_storage();
                *state = staged_state;
                RuntimeMutationResult::MailboxMailAttachedToParty(RuntimeMailTransferOutcome {
                    party_index: Some(command.party_index),
                    mailbox_index: Some(command.mailbox_index),
                    item_id: entry.item_id,
                    mail: entry.mail,
                    mailbox_count_after: state.mailbox.len(),
                    bag_quantity_after: 0,
                })
            }
            RuntimeMutationCommand::AwardBadge(command) => {
                let badges = match command.region {
                    RuntimeBadgeRegion::Johto => &mut state.badges.johto,
                    RuntimeBadgeRegion::Kanto => &mut state.badges.kanto,
                };
                let slot = badges
                    .get_mut(command.index)
                    .with_context(|| format!("badge index {} is outside region", command.index))?;
                let already_awarded = *slot;
                *slot = true;
                let flag_index = command.index + match command.region {
                    RuntimeBadgeRegion::Johto => 0, RuntimeBadgeRegion::Kanto => 8,
                };
                state.flags.engine_flags.insert(
                    crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS[flag_index].to_string(), true);
                RuntimeMutationResult::BadgeAwarded(RuntimeBadgeAwardOutcome {
                    region: command.region,
                    index: command.index,
                    already_awarded,
                    awarded_count_after: badges.iter().filter(|awarded| **awarded).count(),
                })
            }
            RuntimeMutationCommand::RecordPokedexSeen(command) => {
                let species = self
                    .pokemon
                    .get(&command.species_id)
                    .with_context(|| format!("unknown Pokemon species {}", command.species_id))?;
                let already_seen = state.pokedex.has_seen(&command.species_id);
                let already_caught = state.pokedex.has_caught(&command.species_id);
                state.pokedex.record_seen(species);
                RuntimeMutationResult::PokedexSeenRecorded(RuntimePokedexRecordOutcome {
                    species_id: command.species_id,
                    already_seen,
                    already_caught,
                    seen_count_after: state.pokedex.seen_count(),
                    caught_count_after: state.pokedex.caught_count(),
                })
            }
            RuntimeMutationCommand::RecordPokedexCaught(command) => {
                let species = self
                    .pokemon
                    .get(&command.species_id)
                    .with_context(|| format!("unknown Pokemon species {}", command.species_id))?;
                let already_seen = state.pokedex.has_seen(&command.species_id);
                let already_caught = state.pokedex.has_caught(&command.species_id);
                state.pokedex.record_caught(species);
                RuntimeMutationResult::PokedexCaughtRecorded(RuntimePokedexRecordOutcome {
                    species_id: command.species_id,
                    already_seen,
                    already_caught,
                    seen_count_after: state.pokedex.seen_count(),
                    caught_count_after: state.pokedex.caught_count(),
                })
            }
            RuntimeMutationCommand::AddBagItem(command) => {
                let item = self
                    .items
                    .get(&command.item_id)
                    .with_context(|| format!("unknown item {}", command.item_id))?;
                if item.script_name != command.item_id {
                    anyhow::bail!(
                        "compiled item {} has script_name {}, expected exact id match",
                        command.item_id,
                        item.script_name
                    );
                }
                let quantity_before = state.bag.quantity(item);
                let added = state
                    .bag
                    .add_item(item, command.quantity)
                    .map_err(anyhow::Error::msg)?;
                let quantity_after = state.bag.quantity(item);
                RuntimeMutationResult::BagItemAdded(RuntimeBagItemMutationOutcome {
                    item_id: command.item_id,
                    quantity: command.quantity,
                    added,
                    quantity_before,
                    quantity_after,
                })
            }
            RuntimeMutationCommand::AddCurrency(command) => {
                let cap = runtime_currency_cap(&self.currency_constants, command.account)?;
                let before = match command.account {
                    RuntimeCurrencyAccount::Money => state.money,
                    RuntimeCurrencyAccount::Coins => u32::from(state.coins),
                };
                let after = before.saturating_add(command.amount).min(cap);
                match command.account {
                    RuntimeCurrencyAccount::Money => state.money = after,
                    RuntimeCurrencyAccount::Coins => {
                        state.coins = u16::try_from(after)
                            .context("MAX_COINS cannot fit saved coin storage")?;
                    }
                }
                RuntimeMutationResult::CurrencyAdded(RuntimeCurrencyMutationOutcome {
                    account: command.account,
                    amount: command.amount,
                    value_before: before,
                    value_after: after,
                    cap,
                })
            }
            RuntimeMutationCommand::TakeCurrency(command) => {
                let cap = runtime_currency_cap(&self.currency_constants, command.account)?;
                let before = match command.account {
                    RuntimeCurrencyAccount::Money => state.money,
                    RuntimeCurrencyAccount::Coins => u32::from(state.coins),
                };
                let after = before.saturating_sub(command.amount);
                match command.account {
                    RuntimeCurrencyAccount::Money => state.money = after,
                    RuntimeCurrencyAccount::Coins => {
                        state.coins = u16::try_from(after)
                            .context("coin value cannot fit saved coin storage")?;
                    }
                }
                RuntimeMutationResult::CurrencyTaken(RuntimeCurrencyMutationOutcome {
                    account: command.account,
                    amount: command.amount,
                    value_before: before,
                    value_after: after,
                    cap,
                })
            }
            RuntimeMutationCommand::RecordLinkBattleResult(command) => {
                match command.result {
                    RuntimeLinkBattleResult::Win => {
                        state.link_battle_stats.wins =
                            state.link_battle_stats.wins.saturating_add(1);
                    }
                    RuntimeLinkBattleResult::Loss => {
                        state.link_battle_stats.losses =
                            state.link_battle_stats.losses.saturating_add(1);
                    }
                    RuntimeLinkBattleResult::Draw => {
                        state.link_battle_stats.draws =
                            state.link_battle_stats.draws.saturating_add(1);
                    }
                }
                RuntimeMutationResult::LinkBattleResultRecorded(RuntimeLinkBattleRecordOutcome {
                    result: command.result,
                    wins_after: state.link_battle_stats.wins,
                    losses_after: state.link_battle_stats.losses,
                    draws_after: state.link_battle_stats.draws,
                })
            }
            RuntimeMutationCommand::SetOptions(command) => {
                let before = state.options.clone();
                state.options = command.options;
                RuntimeMutationResult::OptionsSet(RuntimeOptionsSetOutcome {
                    options_before: before,
                    options_after: state.options.clone(),
                })
            }
            RuntimeMutationCommand::SetPokegearRadioTuning(command) => {
                if command.tuning_knob > 80 || command.tuning_knob % 2 != 0 {
                    anyhow::bail!(
                        "Pokegear radio tuning knob {} is outside Crystal's even range 0..=80",
                        command.tuning_knob
                    );
                }
                let before = state.radio_tuning_knob;
                state.radio_tuning_knob = command.tuning_knob;
                RuntimeMutationResult::PokegearRadioTuningSet(RuntimePokegearRadioTuningOutcome {
                    tuning_knob_before: before,
                    tuning_knob_after: state.radio_tuning_knob,
                })
            }
            RuntimeMutationCommand::SetTrainerIdentity(command) => {
                let before_name = state.player_name.clone();
                let before_id = state.player_id;
                state.player_name = command.player_name;
                state.player_id = command.player_id;
                RuntimeMutationResult::TrainerIdentitySet(RuntimeTrainerIdentityOutcome {
                    player_name_before: before_name,
                    player_id_before: before_id,
                    player_name_after: state.player_name.clone(),
                    player_id_after: state.player_id,
                })
            }
            RuntimeMutationCommand::SetPlayerGender(command) => {
                validate_saved_player_gender(command.player_gender).map_err(anyhow::Error::msg)?;
                let before = state.player_gender;
                state.player_gender = command.player_gender;
                RuntimeMutationResult::PlayerGenderSet(RuntimePlayerGenderOutcome {
                    player_gender_before: before,
                    player_gender_after: state.player_gender,
                })
            }
            RuntimeMutationCommand::RenamePartyPokemon(command) => {
                let pokemon = state
                    .storage
                    .party
                    .pokemon
                    .get_mut(command.party_index)
                    .with_context(|| {
                        format!("party index {} is outside party", command.party_index)
                    })?
                    .as_mut()
                    .with_context(|| {
                        format!(
                            "party index {} has no Pokemon to rename",
                            command.party_index
                        )
                    })?;
                let before = pokemon.nickname.clone();
                pokemon.nickname = command.nickname;
                let species_id = pokemon.species.id.clone();
                let nickname_after = pokemon.nickname.clone();
                state.sync_party_from_storage();
                RuntimeMutationResult::PartyPokemonRenamed(RuntimePartyNicknameOutcome {
                    party_index: command.party_index,
                    species_id,
                    nickname_before: before,
                    nickname_after,
                })
            }
            RuntimeMutationCommand::RenameStoredPokemon(command) => {
                let location = command.location;
                let (species_id, nickname_before, nickname_after) = match &location {
                    crystal_core::models::CaptureStorageLocation::Party { slot } => {
                        let pokemon = state
                            .storage
                            .party
                            .pokemon
                            .get_mut(*slot)
                            .with_context(|| format!("party index {slot} is outside party"))?
                            .as_mut()
                            .with_context(|| {
                                format!("party index {slot} has no Pokemon to rename")
                            })?;
                        let before = pokemon.nickname.clone();
                        pokemon.nickname = command.nickname;
                        (pokemon.species.id.clone(), before, pokemon.nickname.clone())
                    }
                    crystal_core::models::CaptureStorageLocation::Pc { box_index, slot } => {
                        let pc_box =
                            state
                                .storage
                                .pc_boxes
                                .get_mut(*box_index)
                                .with_context(|| {
                                    format!("PC box index {box_index} is outside storage")
                                })?;
                        let mut pokemon = pc_box
                            .pokemon
                            .get(*slot)
                            .with_context(|| {
                                format!("PC box {box_index} slot {slot} is outside box")
                            })?
                            .clone()
                            .with_context(|| {
                                format!("PC box {box_index} slot {slot} has no Pokemon to rename")
                            })?;
                        let before = pokemon.nickname.clone();
                        pokemon.nickname = command.nickname;
                        let species_id = pokemon.species.id.clone();
                        let after = pokemon.nickname.clone();
                        pc_box.set_slot(*slot, Some(pokemon));
                        (species_id, before, after)
                    }
                };
                state.sync_party_from_storage();
                RuntimeMutationResult::StoredPokemonRenamed(RuntimeStoredPokemonNicknameOutcome {
                    location,
                    species_id,
                    nickname_before,
                    nickname_after,
                })
            }
            RuntimeMutationCommand::SetPartyPokemonRecoveryState(command) => {
                if let Some(status) = command.status.as_deref() {
                    self.validate_saved_pokemon_status_reference(
                        "runtime.party_recovery_setup.status",
                        status,
                    )?;
                }
                let pokemon = state
                    .storage
                    .party
                    .pokemon
                    .get_mut(command.party_index)
                    .with_context(|| {
                        format!("party index {} is outside party", command.party_index)
                    })?
                    .as_mut()
                    .with_context(|| {
                        format!(
                            "party index {} has no Pokemon to set recovery state",
                            command.party_index
                        )
                    })?;
                let hp_before = pokemon.hp;
                let status_before = pokemon.status.clone();
                let first_move = pokemon.moves.first().map(|learned| learned.name.clone());
                let first_move_pp_before = pokemon.moves.first().map(|learned| learned.current_pp);
                if self.nuzlocke_rules.permadeath && pokemon.hp == 0 && command.hp > 0 {
                    anyhow::bail!("Nuzlocke permadeath prevents restoring a fainted Pokemon");
                }
                pokemon.hp = command.hp.min(pokemon.max_hp);
                pokemon.status = command.status;
                if let (Some(learned), Some(pp)) =
                    (pokemon.moves.first_mut(), command.first_move_pp)
                {
                    learned.current_pp = pp;
                }
                let species_id = pokemon.species.id.clone();
                let hp_after = pokemon.hp;
                let status_after = pokemon.status.clone();
                let first_move_pp_after = pokemon.moves.first().map(|learned| learned.current_pp);
                state.sync_party_from_storage();
                RuntimeMutationResult::PartyPokemonRecoveryStateSet(
                    RuntimePartyRecoverySetupOutcome {
                        party_index: command.party_index,
                        species_id,
                        hp_before,
                        hp_after,
                        status_before,
                        status_after,
                        first_move,
                        first_move_pp_before,
                        first_move_pp_after,
                    },
                )
            }
            RuntimeMutationCommand::TransferPartyPokemonHp(command) => {
                if command.source_party_index == command.target_party_index {
                    anyhow::bail!("party HP transfer source and target must differ");
                }
                let party_len = state.storage.party.pokemon.len();
                if command.source_party_index >= party_len
                    || command.target_party_index >= party_len
                {
                    anyhow::bail!(
                        "party HP transfer indexes {} and {} must be inside party length {}",
                        command.source_party_index,
                        command.target_party_index,
                        party_len
                    );
                }
                let (source, target) = if command.source_party_index < command.target_party_index {
                    let (left, right) = state
                        .storage
                        .party
                        .pokemon
                        .split_at_mut(command.target_party_index);
                    (
                        left[command.source_party_index]
                            .as_mut()
                            .context("party HP transfer source slot is empty")?,
                        right[0]
                            .as_mut()
                            .context("party HP transfer target slot is empty")?,
                    )
                } else {
                    let (left, right) = state
                        .storage
                        .party
                        .pokemon
                        .split_at_mut(command.source_party_index);
                    (
                        right[0]
                            .as_mut()
                            .context("party HP transfer source slot is empty")?,
                        left[command.target_party_index]
                            .as_mut()
                            .context("party HP transfer target slot is empty")?,
                    )
                };
                let source_is_egg = source.is_egg;
                let target_is_egg = target.is_egg;
                if source_is_egg || target_is_egg {
                    anyhow::bail!("party HP transfer cannot use an Egg");
                }
                let amount = source.max_hp / 5;
                if source.hp < amount {
                    anyhow::bail!("party HP transfer source does not have enough HP");
                }
                if target.hp == 0 || target.hp >= target.max_hp {
                    anyhow::bail!("party HP transfer target cannot receive HP");
                }
                let source_hp_before = source.hp;
                let target_hp_before = target.hp;
                source.hp -= amount;
                target.hp = target.hp.saturating_add(amount).min(target.max_hp);
                let source_hp_after = source.hp;
                let target_hp_after = target.hp;
                state.sync_party_from_storage();
                RuntimeMutationResult::PartyPokemonHpTransferred(RuntimePartyHpTransferOutcome {
                    source_party_index: command.source_party_index,
                    target_party_index: command.target_party_index,
                    amount,
                    source_hp_before,
                    source_hp_after,
                    target_hp_before,
                    target_hp_after,
                })
            }
            RuntimeMutationCommand::SwapPartyPokemon(command) => {
                if command.first_party_index >= state.storage.party.pokemon.len()
                    || command.second_party_index >= state.storage.party.pokemon.len()
                {
                    anyhow::bail!(
                        "party swap indexes {} and {} must be inside party",
                        command.first_party_index,
                        command.second_party_index
                    );
                }
                if state.storage.party.pokemon[command.first_party_index].is_none()
                    || state.storage.party.pokemon[command.second_party_index].is_none()
                {
                    anyhow::bail!(
                        "party swap indexes {} and {} must both contain Pokemon",
                        command.first_party_index,
                        command.second_party_index
                    );
                }
                state
                    .storage
                    .party
                    .pokemon
                    .swap(command.first_party_index, command.second_party_index);
                state.sync_party_from_storage();
                let first_species_after = state.storage.party.pokemon[command.first_party_index]
                    .as_ref()
                    .map(|pokemon| pokemon.species.id.clone())
                    .with_context(|| {
                        format!(
                            "party swap index {} unexpectedly empty after swap",
                            command.first_party_index
                        )
                    })?;
                let second_species_after = state.storage.party.pokemon[command.second_party_index]
                    .as_ref()
                    .map(|pokemon| pokemon.species.id.clone())
                    .with_context(|| {
                        format!(
                            "party swap index {} unexpectedly empty after swap",
                            command.second_party_index
                        )
                    })?;
                RuntimeMutationResult::PartyPokemonSwapped(RuntimePartySwapOutcome {
                    first_party_index: command.first_party_index,
                    second_party_index: command.second_party_index,
                    first_species_after,
                    second_species_after,
                })
            }
            RuntimeMutationCommand::SwapPartyPokemonMoves(command) => {
                if let Some(transform) = state
                    .script_runtime
                    .active_battle_combat
                    .as_mut()
                    .filter(|combat| combat.player_party_index == command.party_index)
                    .and_then(|combat| combat.player_transform.as_mut())
                {
                    if command.first_move_index >= transform.moves.len()
                        || command.second_move_index >= transform.moves.len()
                    {
                        anyhow::bail!(
                            "move swap indexes {} and {} must be inside transformed active Pokemon moves",
                            command.first_move_index,
                            command.second_move_index
                        );
                    }
                    transform
                        .moves
                        .swap(command.first_move_index, command.second_move_index);
                    RuntimeMutationResult::PartyPokemonMovesSwapped(RuntimePartyMoveSwapOutcome {
                        party_index: command.party_index,
                        first_move_index: command.first_move_index,
                        second_move_index: command.second_move_index,
                        first_move_after: transform.moves[command.first_move_index].name.clone(),
                        second_move_after: transform.moves[command.second_move_index].name.clone(),
                    })
                } else {
                    let pokemon = state
                        .storage
                        .party
                        .pokemon
                        .get_mut(command.party_index)
                        .with_context(|| {
                            format!("party index {} is outside party", command.party_index)
                        })?
                        .as_mut()
                        .with_context(|| {
                            format!("party index {} has no Pokemon", command.party_index)
                        })?;
                    if command.first_move_index >= pokemon.moves.len()
                        || command.second_move_index >= pokemon.moves.len()
                    {
                        anyhow::bail!(
                            "move swap indexes {} and {} must be inside party Pokemon {} moves",
                            command.first_move_index,
                            command.second_move_index,
                            command.party_index
                        );
                    }
                    pokemon
                        .moves
                        .swap(command.first_move_index, command.second_move_index);
                    let moves_after = pokemon.moves.clone();
                    let first_move_after = moves_after[command.first_move_index].name.clone();
                    let second_move_after = moves_after[command.second_move_index].name.clone();
                    state.sync_party_from_storage();
                    if let Some(combat) = state.script_runtime.active_battle_combat.as_mut() {
                        if let Some(party_pokemon) =
                            combat.player_party.get_mut(command.party_index)
                        {
                            party_pokemon.moves = moves_after.clone();
                        }
                        if combat.player_party_index == command.party_index {
                            combat.player.moves = moves_after;
                        }
                    }
                    RuntimeMutationResult::PartyPokemonMovesSwapped(RuntimePartyMoveSwapOutcome {
                        party_index: command.party_index,
                        first_move_index: command.first_move_index,
                        second_move_index: command.second_move_index,
                        first_move_after,
                        second_move_after,
                    })
                }
            }
            RuntimeMutationCommand::FullHealPartyPokemon(command) => {
                let recovered = full_heal_party_slot(
                    state,
                    &self.moves,
                    command.party_index,
                    self.nuzlocke_rules,
                )?;
                RuntimeMutationResult::PartyPokemonFullHealed(recovered)
            }
            RuntimeMutationCommand::FullHealWholeParty => {
                let mut recovered = Vec::new();
                for party_index in 0..state.storage.party.pokemon.len() {
                    if state.storage.party.pokemon[party_index]
                        .as_ref()
                        .is_some_and(|pokemon| !pokemon.is_egg)
                    {
                        recovered.push(full_heal_party_slot(
                            state,
                            &self.moves,
                            party_index,
                            self.nuzlocke_rules,
                        )?);
                    }
                }
                RuntimeMutationResult::WholePartyFullHealed(recovered)
            }
            RuntimeMutationCommand::CompleteBattleLoss => {
                anyhow::ensure!(
                    state.storage.party.pokemon.iter().flatten().any(|pokemon| !pokemon.is_egg)
                        && !state.storage.party.pokemon.iter().flatten()
                            .any(|pokemon| !pokemon.is_egg && pokemon.hp > 0),
                    "battle loss requires every usable party Pokemon to be fainted"
                );
                anyhow::ensure!(
                    !matches!(state.battle, BattleMemory::Inactive),
                    "battle loss requires an active battle"
                );
                crystal_core::battle::start::deactivate_battle_after_loss(state);
                RuntimeMutationResult::BattleLossCompleted
            }
            RuntimeMutationCommand::ResolveBlackoutToLastSpawn => {
                anyhow::ensure!(
                    !state
                        .storage
                        .party
                        .pokemon
                        .iter()
                        .flatten()
                        .any(|pokemon| { !pokemon.is_egg && pokemon.hp > 0 }),
                    "blackout requires every usable party Pokemon to be fainted"
                );
                anyhow::ensure!(
                    matches!(state.battle, BattleMemory::Inactive),
                    "blackout requires battle loss cleanup to finish before whiteout recovery"
                );
                anyhow::ensure!(
                    state.battle_result & 0x3f == 1,
                    "blackout cannot consume terminal result {:#04x}",
                    state.battle_result
                );
                if let Some(pending) = state.pending_static_wild_terminal.as_ref() {
                    anyhow::ensure!(
                        pending.battle_result & 0x3f == 1,
                        "blackout cannot consume static-wild terminal result {:#04x}",
                        pending.battle_result
                    );
                }
                let heal_indexes = (0..state.storage.party.pokemon.len())
                    .filter(|party_index| {
                        state.storage.party.pokemon[*party_index]
                            .as_ref()
                            .is_some_and(|pokemon| !pokemon.is_egg)
                    })
                    .collect::<Vec<_>>();
                let mut healed = Vec::new();
                for party_index in heal_indexes {
                    healed.push(full_heal_party_slot(
                        state,
                        &self.moves,
                        party_index,
                        self.nuzlocke_rules,
                    )?);
                }
                let bug_contest_active = state
                    .flags
                    .is_engine_flag_set("ENGINE_BUG_CONTEST_TIMER")
                    .map_err(|error| anyhow::anyhow!("read Bug Contest timer flag: {error}"))?;
                let (spawn_identifier, map_name, tile) = if bug_contest_active {
                    prepare_bug_contest_results_warp(state)?;
                    let map_name = "Route36NationalParkGate".to_string();
                    let tile = raw_event_tile_to_runtime_tile_checked(0, 4)
                        .context("resolve Bug Contest blackout destination")?;
                    let mode = session.player.mode;
                    self.transition_overworld_session_with_mode(
                        state,
                        session,
                        &map_name,
                        tile,
                        mode,
                        "MAPSETUP_WARP",
                        SpawnMemoryUpdate::Preserve,
                        music_ids,
                    )?;
                    (None, map_name, tile)
                } else {
                    self.apply_special_routine(state, "WarpToSpawnPoint", music_ids)?;
                    let saved_spawn_identifier = match state.last_spawn_map_constant.as_deref() {
                        Some(map_constant) => {
                            self.optional_runtime_spawn_identifier_for_map_constant(map_constant)?
                        }
                        None => None,
                    };
                    let spawn_identifier = match saved_spawn_identifier {
                        Some(identifier) => identifier,
                        None => self.home_spawn_identifier()?,
                    };
                    let spawn = self.runtime_spawn_point(spawn_identifier)?;
                    let map_name = spawn.map_name.clone();
                    let tile = runtime_spawn_expected_tile(spawn);
                    state.money /= 2;
                    self.transition_overworld_session(
                        state,
                        session,
                        &map_name,
                        tile,
                        SpawnMemoryUpdate::Preserve,
                        music_ids,
                    )?;
                    (Some(spawn_identifier), map_name, tile)
                };
                if let Some(pending) = state.pending_static_wild_terminal.as_ref() {
                    anyhow::ensure!(
                        pending.battle_result & 0x3f == 1,
                        "blackout cannot consume static-wild terminal result {:#04x}",
                        pending.battle_result
                    );
                }
                state.pending_static_wild_terminal = None;
                // Script_Whiteout terminates the interrupted script with
                // endall semantics.  The spawn warp is retained separately,
                // but no Rock/common-script cursor or return frame may survive
                // and later execute reloadmapafterbattle/disappear/end.
                state.script_runtime.next_script = None;
                state.script_runtime.call_stack.clear();
                state.script_runtime.deferred_scripts.clear();
                state.script_runtime.map_reentry_script = None;
                state.script_runtime.command_queue.clear();
                state.script_runtime.script_ended = None;
                RuntimeMutationResult::BlackoutResolved(BlackoutRecoveryOutcome {
                    spawn_identifier,
                    map_name,
                    tile,
                    healed,
                })
            }
            RuntimeMutationCommand::InitializePermanentPhoneNumbers => {
                RuntimeMutationResult::PermanentPhoneNumbersInitialized(
                    self.initialize_permanent_phone_numbers(state)?,
                )
            }
            RuntimeMutationCommand::DeletePokegearPhoneNumber { contact_id } => {
                RuntimeMutationResult::PokegearPhoneNumberDeleted(
                    crystal_core::systems::phone::delete_pokegear_phone_number(
                        state, &self.phone_contacts, &contact_id,
                    )?,
                )
            }
            RuntimeMutationCommand::StartPokegearPhoneCall(command) => {
                RuntimeMutationResult::PokegearPhoneCallStarted(
                    self.start_pokegear_phone_call(state, session, command)?,
                )
            }
        };
        session.set_time(state.time.registers.hours, state.time.time_of_day);
        session.sync_event_flag_memory(&state.flags);
        let result_tag = result.result_tag();
        // The loaded state is validated at runtime-shell construction and
        // mutation boundaries are typed. Avoid re-walking the complete save
        // graph on every 60 Hz frame; retain the exact serialized checksum.
        let state_checksum = if compute_checksum {
            game_state_checksum_unchecked(state)
                .with_context(|| format!("checksum runtime mutation {result_tag}"))?
        } else {
            StateChecksum::new(state.frame_counter, 0)
        };
        Ok(RuntimeMutationOutcome {
            result,
            state_checksum,
        })
    }

}
