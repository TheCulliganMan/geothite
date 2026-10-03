impl RuntimeOverworldSession {
    fn new(
        runtime: &CrystalRuntime,
        _asset_root: &AssetRoot,
        spawn: &RuntimeSpawnPoint,
    ) -> Result<Self> {
        let (state, overworld) = runtime
            .data
            .start_overworld_session_from_spawn(spawn, &runtime.audio.music_ids())?;
        Ok(Self {
            state,
            overworld,
            joypad: JoypadState::new(),
            divider: RuntimeDividerSource::live(),
        })
    }

    #[cfg(any(test, feature = "test-fixtures", feature = "location-tester"))]
    fn new_at_runtime_tile(
        runtime: &CrystalRuntime,
        _asset_root: &AssetRoot,
        map_name: &str,
        tile: TilePosition,
    ) -> Result<Self> {
        let (state, overworld) = runtime.data.start_overworld_session_at_runtime_tile(
            map_name,
            tile,
            &runtime.audio.music_ids(),
        )?;
        Ok(Self {
            state,
            overworld,
            joypad: JoypadState::new(),
            divider: RuntimeDividerSource::live(),
        })
    }

    fn from_state(
        runtime: &CrystalRuntime,
        _asset_root: &AssetRoot,
        state: GameState,
    ) -> Result<Self> {
        let (state, overworld) = runtime
            .data
            .resume_overworld_session_from_state(state, &runtime.audio.music_ids())?;
        Ok(Self {
            joypad: JoypadState::from_previous_mask(state.joypad.h_joy_down),
            state,
            overworld,
            divider: RuntimeDividerSource::live(),
        })
    }

    fn stage_overworld_input(
        &mut self,
        runtime: &CrystalRuntime,
        buttons: Vec<GameButton>,
        checksum: bool,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let mut overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let frame = runtime.data.apply_overworld_input(
            &mut state,
            &mut overworld,
            buttons.clone(),
            &runtime.music_ids(),
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::OverworldInputApplied(frame),
            state_checksum: if checksum {
                game_state_checksum(&state)?
            } else {
                StateChecksum::new(state.frame_counter, 0)
            },
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::ApplyOverworldInput(RuntimeOverworldInputCommand {
                buttons,
                divider_trace,
            }),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    /// Apply a real-time host frame without constructing a second outer
    /// transactional copy or a replay checksum. `GameDataSet` still owns the
    /// single atomic gameplay transaction; the shell no longer duplicates
    /// that already-staged state solely to serialize a disabled journal.
    fn apply_overworld_input_live(
        &mut self,
        runtime: &CrystalRuntime,
        buttons: Vec<GameButton>,
    ) -> Result<RuntimeOverworldFrame> {
        let frame = runtime.data.apply_overworld_input(
            &mut self.state,
            &mut self.overworld,
            buttons,
            &runtime.music_ids(),
            &mut self.divider,
        )?;
        self.joypad = JoypadState::from_previous_mask(frame.input_mask);
        Ok(RuntimeOverworldFrame::from_input_frame(
            frame,
            StateChecksum::new(self.state.frame_counter, 0),
        ))
    }

    fn stage_sweet_scent_encounter(
        &mut self,
        runtime: &CrystalRuntime,
        command: RuntimeScriptCommandRef,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let result = runtime.data.resolve_sweet_scent_encounter(
            &mut state,
            &overworld,
            &command,
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::SweetScentEncounterResolved(result),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::ResolveSweetScentEncounter(
                RuntimeSweetScentEncounterCommand {
                    command,
                    divider_trace,
                },
            ),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_fishing_rod_cast(
        &mut self,
        runtime: &CrystalRuntime,
        rod: &str,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let result = runtime.data.cast_fishing_rod_in_session_with_divider(
            &mut state,
            &overworld,
            rod,
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::FishingRodCast(result),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::CastFishingRod(RuntimeFishingCommand {
                rod: rod.to_string(),
                divider_trace,
            }),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_bag_fishing_rod_use(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let result = runtime.data.use_bag_fishing_rod_in_field_with_divider(
            &mut state,
            &overworld,
            item_id,
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::BagFishingRodUsed(result),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::UseBagFishingRodInField(RuntimeFishingItemCommand {
                item_id: item_id.to_string(),
                divider_trace,
            }),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_throw_ball_at_active_battle(
        &mut self,
        runtime: &CrystalRuntime,
        ball_id: &str,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let result = runtime.data.throw_ball_at_active_battle_with_divider(
            &mut state,
            ball_id,
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::BallThrown(result),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::ThrowBallAtActiveBattle(RuntimeBattleItemCommand {
                item_id: ball_id.to_string(),
                divider_trace,
            }),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_escape_active_wild_battle(
        &mut self,
        runtime: &CrystalRuntime,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let result = runtime
            .data
            .resolve_active_wild_battle_run_with_divider(&mut state, &mut recording)?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::ActiveWildBattleEscapeAttempted(result),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::AttemptEscapeActiveWildBattle(
                RuntimeBattleEscapeCommand { divider_trace },
            ),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_bag_item_escape_active_wild_battle(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let result = runtime
            .data
            .use_bag_item_to_escape_active_wild_battle_with_divider(
                &mut state,
                item_id,
                &mut recording,
            )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::ActiveWildBattleEscapeItemUsed(result),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::UseBagItemToEscapeActiveWildBattle(
                RuntimeBattleItemCommand {
                    item_id: item_id.to_string(),
                    divider_trace,
                },
            ),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_active_battle_turn(
        &mut self,
        runtime: &CrystalRuntime,
        player_action: BattleAction,
        enemy_action: BattleAction,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let result = runtime.data.resolve_active_battle_turn_with_divider(
            &mut state,
            player_action.clone(),
            enemy_action.clone(),
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::ActiveBattleTurnResolved(result),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::ResolveActiveBattleTurn(RuntimeBattleTurnCommand {
                player_action,
                player_bag_item_id: None,
                enemy_action,
                enemy_ai_divider_trace: RuntimeDividerTrace::new([]),
                enemy_ai_selected_move_slot: None,
                enemy_move_ai_random_calls: 0,
                enemy_post_order_ai_random_calls: 0,
                divider_trace,
            }),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_active_battle_turn_with_enemy_selector<F>(
        &mut self,
        runtime: &CrystalRuntime,
        player_action: BattleAction,
        player_bag_item_id: Option<&str>,
        select_enemy_action: F,
    ) -> Result<(
        BattleAction,
        Option<ItemUseOutcome>,
        RecordedRuntimeMutation,
    )>
    where
        F: FnOnce(&mut dyn crystal_core::random::BattleRandomSource) -> Result<BattleAction>,
    {
        let mut state = self.state.clone();
        let overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let player_bag_item = if let Some(item_id) = player_bag_item_id {
            anyhow::ensure!(
                matches!(
                    &player_action,
                    BattleAction::Item { item_id: action_item_id }
                        | BattleAction::PartyItem {
                            item_id: action_item_id,
                            ..
                        }
                        if action_item_id == item_id
                ),
                "player Bag item {item_id} does not match the battle action",
            );
            Some(
                runtime
                    .data
                    .use_bag_item(&mut state, item_id, ItemUseContext::Battle)?,
            )
        } else {
            None
        };

        let mut ai_recording = RecordingDivider::new(&mut divider_after);
        let mut ai_rng =
            crystal_core::random::ExactBattleRandom::new(state.random_state, &mut ai_recording);
        let enemy_action = select_enemy_action(&mut ai_rng)?;
        if let Some(error) = ai_rng.divider_error() {
            anyhow::bail!("select exact enemy battle action: {error}");
        }
        state.random_state = ai_rng.state();
        drop(ai_rng);
        let enemy_ai_divider_trace =
            RuntimeDividerTrace::new(ai_recording.samples().iter().copied());
        drop(ai_recording);

        let mut turn_recording = RecordingDivider::new(&mut divider_after);
        let result = runtime.data.resolve_active_battle_turn_with_divider(
            &mut state,
            player_action.clone(),
            enemy_action.clone(),
            &mut turn_recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(turn_recording.samples().iter().copied());
        drop(turn_recording);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::ActiveBattleTurnResolved(result),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok((
            enemy_action.clone(),
            player_bag_item,
            RecordedRuntimeMutation {
                command: RuntimeMutationCommand::ResolveActiveBattleTurn(
                    RuntimeBattleTurnCommand {
                        player_action,
                        player_bag_item_id: player_bag_item_id.map(str::to_string),
                        enemy_action,
                        enemy_ai_divider_trace,
                        enemy_ai_selected_move_slot: None,
                        enemy_move_ai_random_calls: 0,
                        enemy_post_order_ai_random_calls: 0,
                        divider_trace,
                    },
                ),
                state,
                overworld,
                outcome,
                divider_after: Some(divider_after),
            },
        ))
    }

    fn stage_active_battle_turn_with_enemy_selectors<FM, FP>(
        &mut self,
        runtime: &CrystalRuntime,
        player_action: BattleAction,
        player_bag_item_id: Option<&str>,
        select_enemy_move: FM,
        mut select_enemy_post_order_action: FP,
    ) -> Result<(
        BattleAction,
        Option<ItemUseOutcome>,
        RecordedRuntimeMutation,
    )>
    where
        FM: FnOnce(
            &crystal_core::battle::turn::BattleCombatState,
            &mut dyn crystal_core::random::BattleRandomSource,
        ) -> Result<crystal_core::battle::turn::EnemyMoveSelection>,
        FP: FnMut(
            usize,
            &crystal_core::battle::turn::BattleCombatState,
            &mut dyn crystal_core::random::BattleRandomSource,
        ) -> Result<BattleAction, crystal_core::battle::turn::BattleTurnError>,
    {
        struct CountingBattleRandom<'a> {
            inner: &'a mut dyn crystal_core::random::BattleRandomSource,
            calls: &'a mut usize,
        }

        impl crystal_core::random::BattleRandomSource for CountingBattleRandom<'_> {
            fn battle_random_byte(&mut self) -> u8 {
                *self.calls += 1;
                self.inner.battle_random_byte()
            }
        }

        let mut state = self.state.clone();
        let overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let player_bag_item = if let Some(item_id) = player_bag_item_id {
            anyhow::ensure!(
                matches!(
                    &player_action,
                    BattleAction::Item { item_id: action_item_id }
                        | BattleAction::PartyItem {
                            item_id: action_item_id,
                            ..
                        }
                        if action_item_id == item_id
                ),
                "player Bag item {item_id} does not match the battle action",
            );
            Some(
                runtime
                    .data
                    .use_bag_item(&mut state, item_id, ItemUseContext::Battle)?,
            )
        } else {
            None
        };

        let selected_move_slot = std::cell::Cell::new(None);
        let enemy_action = std::cell::RefCell::new(BattleAction::Move { slot: 0 });
        let mut select_enemy_move = Some(select_enemy_move);
        let mut enemy_move_ai_random_calls = 0usize;
        let mut enemy_post_order_ai_random_calls = 0usize;
        let mut move_selector =
            |combat: &crystal_core::battle::turn::BattleCombatState,
             rng: &mut dyn crystal_core::random::BattleRandomSource| {
                let mut counting_rng = CountingBattleRandom {
                    inner: rng,
                    calls: &mut enemy_move_ai_random_calls,
                };
                let selection = select_enemy_move
                    .take()
                    .expect("enemy move selector is invoked once")(
                    combat, &mut counting_rng
                )
                .map_err(|error| {
                    crystal_core::battle::turn::BattleTurnError::EnemyActionSelectionFailed {
                        error: format!("{error:#}"),
                    }
                })?;
                selected_move_slot.set(Some(selection.slot));
                *enemy_action.borrow_mut() = BattleAction::Move {
                    slot: selection.slot,
                };
                Ok(selection)
            };
        let mut post_order_selector =
            |combat: &crystal_core::battle::turn::BattleCombatState,
             rng: &mut dyn crystal_core::random::BattleRandomSource| {
                let mut counting_rng = CountingBattleRandom {
                    inner: rng,
                    calls: &mut enemy_post_order_ai_random_calls,
                };
                let selected = select_enemy_post_order_action(
                    selected_move_slot
                        .get()
                        .expect("enemy move selection precedes trainer action selection"),
                    combat,
                    &mut counting_rng,
                )?;
                *enemy_action.borrow_mut() = selected.clone();
                Ok(selected)
            };
        let mut turn_recording = RecordingDivider::new(&mut divider_after);
        let result = if let BattleAction::Ball { item_id } = &player_action {
            runtime
                .data
                .resolve_active_battle_ball_turn_with_enemy_ai_actions_with_divider(
                    &mut state,
                    item_id,
                    BattleAction::Move { slot: 0 },
                    &mut turn_recording,
                    Some((&mut move_selector, &mut post_order_selector)),
                )?
                .1
        } else {
            runtime
                .data
                .resolve_active_battle_turn_with_enemy_ai_actions_with_divider(
                    &mut state,
                    player_action.clone(),
                    &mut turn_recording,
                    &mut move_selector,
                    &mut post_order_selector,
                )?
        };
        drop(move_selector);
        drop(post_order_selector);
        let enemy_ai_selected_move_slot = selected_move_slot.get();
        let enemy_action = enemy_action.into_inner();
        let enemy_move_ai_random_calls = u16::try_from(enemy_move_ai_random_calls)
            .context("enemy move AI consumed more than u16::MAX random calls")?;
        let enemy_post_order_ai_random_calls = u16::try_from(enemy_post_order_ai_random_calls)
            .context("enemy post-order AI consumed more than u16::MAX random calls")?;
        let divider_trace = RuntimeDividerTrace::new(turn_recording.samples().iter().copied());
        drop(turn_recording);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::ActiveBattleTurnResolved(result),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok((
            enemy_action.clone(),
            player_bag_item,
            RecordedRuntimeMutation {
                command: RuntimeMutationCommand::ResolveActiveBattleTurn(
                    RuntimeBattleTurnCommand {
                        player_action,
                        player_bag_item_id: player_bag_item_id.map(str::to_string),
                        enemy_action,
                        enemy_ai_divider_trace: RuntimeDividerTrace::new([]),
                        enemy_ai_selected_move_slot,
                        enemy_move_ai_random_calls,
                        enemy_post_order_ai_random_calls,
                        divider_trace,
                    },
                ),
                state,
                overworld,
                outcome,
                divider_after: Some(divider_after),
            },
        ))
    }

    fn stage_active_battle_command(
        &mut self,
        runtime: &CrystalRuntime,
        player_action: BattleAction,
        enemy_action: BattleAction,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let result = runtime.data.resolve_active_battle_command_with_divider(
            &mut state,
            player_action.clone(),
            enemy_action.clone(),
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::ActiveBattleCommandResolved(result),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::ResolveActiveBattleCommand(RuntimeBattleTurnCommand {
                player_action,
                player_bag_item_id: None,
                enemy_action,
                enemy_ai_divider_trace: RuntimeDividerTrace::new([]),
                enemy_ai_selected_move_slot: None,
                enemy_move_ai_random_calls: 0,
                enemy_post_order_ai_random_calls: 0,
                divider_trace,
            }),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_active_battle_enemy_action(
        &mut self,
        runtime: &CrystalRuntime,
        enemy_action: BattleAction,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let result = runtime
            .data
            .resolve_active_battle_enemy_action_with_divider(
                &mut state,
                enemy_action.clone(),
                &mut recording,
            )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::ActiveBattleEnemyActionResolved(result),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::ResolveActiveBattleEnemyAction(
                RuntimeBattleEnemyActionCommand {
                    enemy_action,
                    enemy_ai_divider_trace: RuntimeDividerTrace::new([]),
                    divider_trace,
                },
            ),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_active_battle_enemy_action_with_selector<F>(
        &mut self,
        runtime: &CrystalRuntime,
        select_enemy_action: F,
    ) -> Result<(BattleAction, RecordedRuntimeMutation)>
    where
        F: FnOnce(
            &crystal_core::battle::turn::BattleCombatState,
            &mut dyn crystal_core::random::BattleRandomSource,
        ) -> Result<BattleAction>,
    {
        let mut state = self.state.clone();
        let overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();

        let mut ai_recording = RecordingDivider::new(&mut divider_after);
        let mut ai_rng =
            crystal_core::random::ExactBattleRandom::new(state.random_state, &mut ai_recording);
        let combat = state
            .script_runtime
            .active_battle_combat
            .as_ref()
            .context("enemy-only AI selection requires live core battle state")?;
        let enemy_action = select_enemy_action(combat, &mut ai_rng)?;
        if let Some(error) = ai_rng.divider_error() {
            anyhow::bail!("select exact enemy battle action: {error}");
        }
        state.random_state = ai_rng.state();
        drop(ai_rng);
        let enemy_ai_divider_trace =
            RuntimeDividerTrace::new(ai_recording.samples().iter().copied());
        drop(ai_recording);

        let mut turn_recording = RecordingDivider::new(&mut divider_after);
        let result = runtime
            .data
            .resolve_active_battle_enemy_action_with_divider(
                &mut state,
                enemy_action.clone(),
                &mut turn_recording,
            )?;
        let divider_trace = RuntimeDividerTrace::new(turn_recording.samples().iter().copied());
        drop(turn_recording);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::ActiveBattleEnemyActionResolved(result),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok((
            enemy_action.clone(),
            RecordedRuntimeMutation {
                command: RuntimeMutationCommand::ResolveActiveBattleEnemyAction(
                    RuntimeBattleEnemyActionCommand {
                        enemy_action,
                        enemy_ai_divider_trace,
                        divider_trace,
                    },
                ),
                state,
                overworld,
                outcome,
                divider_after: Some(divider_after),
            },
        ))
    }

    fn stage_random_special_routine(
        &mut self,
        runtime: &CrystalRuntime,
        routine: &str,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let mut overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let outcome = runtime.data.apply_random_special_routine(
            &mut state,
            routine,
            &runtime.music_ids(),
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        overworld.sync_event_flag_memory(&state.flags);
        let mutation_outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::SpecialRoutineApplied(outcome),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::ApplyRandomSpecialRoutine(
                RuntimeRandomSpecialRoutineCommand {
                    routine: routine.to_string(),
                    divider_trace,
                },
            ),
            state,
            overworld,
            outcome: mutation_outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_rock_mon_encounter(
        &mut self,
        runtime: &CrystalRuntime,
        command: RuntimeScriptCommandRef,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let mut overworld = self.overworld.clone();
        let current_map = overworld.map.name.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let resolved = runtime.data.resolve_rock_mon_encounter(
            &mut state,
            &current_map,
            &command,
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        overworld.sync_event_flag_memory(&state.flags);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::RockMonEncounterResolved(resolved),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::ResolveRockMonEncounter(
                RuntimeRockMonEncounterCommand {
                    command,
                    divider_trace,
                },
            ),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_tree_mon_encounter(
        &mut self,
        runtime: &CrystalRuntime,
        command: RuntimeScriptCommandRef,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let mut overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let resolved = runtime.data.resolve_tree_mon_encounter(
            &mut state,
            &overworld,
            &command,
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        overworld.sync_event_flag_memory(&state.flags);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::TreeMonEncounterResolved(resolved),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::ResolveTreeMonEncounter(
                RuntimeTreeMonEncounterCommand {
                    command,
                    divider_trace,
                },
            ),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_scripted_gift_pokemon(
        &mut self,
        runtime: &CrystalRuntime,
        command: RuntimeScriptCommandRef,
        original_trainer_name: String,
        original_trainer_id: u16,
        nickname_accepted: bool,
        nickname: Option<String>,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let mut overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let resolved = runtime
            .data
            .grant_scripted_gift_pokemon_with_divider_in_session(
                &mut state,
                &overworld,
                &command.map_name,
                &command.source_script,
                command.command_index,
                original_trainer_name.clone(),
                original_trainer_id,
                nickname_accepted,
                nickname.clone(),
                &mut recording,
            )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        overworld.sync_event_flag_memory(&state.flags);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::ScriptedGiftPokemonGranted(resolved),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::GrantScriptedGiftPokemon(RuntimeGiftPokemonCommand {
                command,
                original_trainer_name,
                original_trainer_id,
                nickname_accepted,
                nickname,
                divider_trace,
            }),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_random_script_runtime(
        &mut self,
        runtime: &CrystalRuntime,
        command: RuntimeScriptCommandRef,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let (script_command, resolved) = runtime
            .data
            .apply_random_script_runtime_command_in_session(
                &mut state,
                &overworld,
                &command.map_name,
                &command.source_script,
                command.command_index,
                &mut recording,
            )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::ScriptRuntimeApplied(script_command, resolved),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::ApplyRandomScriptRuntime(RuntimeRandomScriptCommand {
                command,
                divider_trace,
            }),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_random_script_map(
        &mut self,
        runtime: &CrystalRuntime,
        command: RuntimeScriptCommandRef,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let action = runtime
            .data
            .apply_script_map_command_with_divider_in_session(
                &mut state,
                &overworld,
                &command.map_name,
                &command.source_script,
                command.command_index,
                &mut recording,
            )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::ScriptMapApplied(action),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::ApplyRandomScriptMap(RuntimeRandomScriptMapCommand {
                command,
                divider_trace,
            }),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_happiness_service(
        &mut self,
        runtime: &CrystalRuntime,
        routine: RuntimeHappinessServiceRoutine,
        party_index: usize,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let resolved = runtime.data.apply_happiness_service_with_divider(
            &mut state,
            routine,
            party_index,
            &runtime.music_ids(),
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::HappinessServiceApplied(resolved),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::ApplyHappinessService(
                RuntimeHappinessServiceCommand {
                    routine,
                    party_index,
                    divider_trace,
                },
            ),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_scripted_wild_battle_start(
        &mut self,
        runtime: &CrystalRuntime,
        command: RuntimeScriptCommandRef,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let mut overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let start = runtime.data.start_scripted_wild_battle_in_session(
            &mut state,
            &overworld,
            &command.map_name,
            &command.source_script,
            command.command_index,
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        overworld.sync_event_flag_memory(&state.flags);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::ScriptedWildBattleStarted(start),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::StartScriptedWildBattle(
                RuntimeScriptedWildBattleStartCommand {
                    command,
                    divider_trace,
                },
            ),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_active_wild_capture_completion(
        &mut self,
        runtime: &CrystalRuntime,
        capture_outcome: &CaptureOutcome,
        nickname: Option<String>,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let mut overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let completion = runtime.data.complete_active_wild_capture(
            &mut state,
            capture_outcome,
            nickname.as_deref(),
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        overworld.sync_event_flag_memory(&state.flags);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::ActiveWildCaptureCompleted(completion),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::CompleteActiveWildCapture(
                RuntimeCaptureCompletionCommand {
                    outcome: capture_outcome.clone(),
                    nickname,
                    divider_trace,
                },
            ),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_phone_random_special(
        &mut self,
        runtime: &CrystalRuntime,
        special: RuntimePhoneRandomSpecial,
        contact_id: String,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let mut overworld = self.overworld.clone();
        state
            .script_runtime
            .variables
            .insert("VAR_CALLERID".to_string(), contact_id.clone());
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let outcome = runtime.data.apply_random_special_routine(
            &mut state,
            special.routine(),
            &runtime.music_ids(),
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        overworld.sync_event_flag_memory(&state.flags);
        let mutation_outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::PhoneRandomSpecialApplied(outcome),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::ApplyPhoneRandomSpecial(RuntimePhoneCallerCommand {
                special,
                contact_id,
                divider_trace,
            }),
            state,
            overworld,
            outcome: mutation_outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_random_bug_contest(
        &mut self,
        runtime: &CrystalRuntime,
        action: RuntimeBugContestAction,
    ) -> Result<RecordedRuntimeMutation> {
        let routine = match action {
            RuntimeBugContestAction::SelectContestants => "SelectRandomBugContestContestants",
            RuntimeBugContestAction::Judge => "BugContestJudging",
            _ => anyhow::bail!("Bug Contest exact RNG staging requested for {action:?}"),
        };
        let mut state = self.state.clone();
        let mut overworld = self.overworld.clone();
        state.script_runtime.variables.remove("_bug_contest_rank");
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let outcome = runtime.data.apply_random_special_routine(
            &mut state,
            routine,
            &runtime.music_ids(),
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        overworld.sync_event_flag_memory(&state.flags);
        let mutation_outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::BugContestUsed(outcome),
            state_checksum: game_state_checksum(&state)?,
        };
        let command = match action {
            RuntimeBugContestAction::SelectContestants => {
                RuntimeBugContestCommand::SelectContestants { divider_trace }
            }
            RuntimeBugContestAction::Judge => RuntimeBugContestCommand::Judge { divider_trace },
            _ => unreachable!("random Bug Contest actions were rejected above"),
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::UseBugContest(command),
            state,
            overworld,
            outcome: mutation_outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_buena_password(
        &mut self,
        runtime: &CrystalRuntime,
        guess: Option<String>,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let mut overworld = self.overworld.clone();
        match guess.as_deref() {
            Some(guess) => {
                state
                    .script_runtime
                    .variables
                    .insert("BUENA_PASSWORD".to_string(), guess.to_string());
            }
            None => {
                state.script_runtime.variables.remove("BUENA_PASSWORD");
            }
        }
        let outcome = runtime.data.apply_special_routine(
            &mut state,
            "BuenasPassword",
            &runtime.music_ids(),
        )?;
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        overworld.sync_event_flag_memory(&state.flags);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::BuenaPasswordUsed(outcome),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::UseBuenaPassword(RuntimeBuenaPasswordCommand {
                guess,
            }),
            state,
            overworld,
            outcome,
            divider_after: None,
        })
    }

    fn stage_shuckie_give(&mut self, runtime: &CrystalRuntime) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let mut overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let outcome = runtime.data.apply_random_special_routine(
            &mut state,
            "GiveShuckle",
            &runtime.music_ids(),
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        overworld.sync_event_flag_memory(&state.flags);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::ShuckieUsed(outcome),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::UseShuckie(RuntimeShuckieCommand::Give {
                divider_trace,
            }),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_odd_egg(&mut self, runtime: &CrystalRuntime) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let mut overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let outcome = runtime.data.apply_random_special_routine(
            &mut state,
            "GiveOddEgg",
            &runtime.music_ids(),
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        overworld.sync_event_flag_memory(&state.flags);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::OddEggGiven(outcome),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::GiveOddEgg(RuntimeOddEggCommand { divider_trace }),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_battle_tower_opponent(
        &mut self,
        runtime: &CrystalRuntime,
        target_object: String,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let mut overworld = self.overworld.clone();
        state.script_runtime.script_value = Some(target_object.clone());
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let outcome = runtime.data.apply_random_special_routine(
            &mut state,
            "LoadOpponentTrainerAndPokemonWithOTSprite",
            &runtime.music_ids(),
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        overworld.sync_event_flag_memory(&state.flags);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::BattleTowerOpponentLoaded(outcome),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::LoadBattleTowerOpponentSpecial(
                RuntimeBattleTowerOpponentCommand {
                    target_object,
                    divider_trace,
                },
            ),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_day_care(
        &mut self,
        runtime: &CrystalRuntime,
        caretaker: RuntimeDayCareCaretaker,
        action: RuntimeDayCareAction,
        party_index: Option<usize>,
    ) -> Result<RecordedRuntimeMutation> {
        anyhow::ensure!(
            !matches!(action, RuntimeDayCareAction::CollectEgg)
                || matches!(caretaker, RuntimeDayCareCaretaker::Man),
            "Day Care egg collection is only available from the man"
        );
        let routine = match caretaker {
            RuntimeDayCareCaretaker::Man => "DayCareMan",
            RuntimeDayCareCaretaker::Lady => "DayCareLady",
        };
        let action_name = match action {
            RuntimeDayCareAction::Open => "open",
            RuntimeDayCareAction::Deposit => "deposit",
            RuntimeDayCareAction::Withdraw => "withdraw",
            RuntimeDayCareAction::Inspect => "inspect",
            RuntimeDayCareAction::CollectEgg => "collect_egg",
        };
        anyhow::ensure!(
            matches!(action, RuntimeDayCareAction::Deposit) == party_index.is_some(),
            "Day Care {action_name} command has invalid party_index presence"
        );
        let mut state = self.state.clone();
        let mut overworld = self.overworld.clone();
        state.script_runtime.pending_day_care_input = Some(match action {
            RuntimeDayCareAction::Open => DayCareInput::Open {},
            RuntimeDayCareAction::Deposit => DayCareInput::Deposit {
                party_slot: party_index.expect("validated Day Care deposit party index"),
            },
            RuntimeDayCareAction::Withdraw => DayCareInput::Withdraw {},
            RuntimeDayCareAction::Inspect => DayCareInput::Inspect {},
            RuntimeDayCareAction::CollectEgg => DayCareInput::CollectEgg {},
        });
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let outcome = runtime.data.apply_random_special_routine(
            &mut state,
            routine,
            &runtime.music_ids(),
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        overworld.sync_event_flag_memory(&state.flags);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::DayCareUsed(outcome),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::UseDayCare(RuntimeDayCareCommand {
                caretaker,
                action,
                party_index,
                divider_trace,
            }),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_day_care_man_outside(
        &mut self,
        runtime: &CrystalRuntime,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let mut overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let result = runtime.data.apply_random_special_routine(
            &mut state,
            "DayCareManOutside",
            &runtime.music_ids(),
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        overworld.sync_event_flag_memory(&state.flags);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::DayCareManOutsideChecked(result),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::CheckDayCareManOutsideSpecial(divider_trace),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_game_corner(
        &mut self,
        runtime: &CrystalRuntime,
        service: RuntimeGameCornerService,
    ) -> Result<RecordedRuntimeMutation> {
        let routine = match service {
            RuntimeGameCornerService::SlotMachine => "SlotMachine",
            RuntimeGameCornerService::CardFlip => "CardFlip",
        };
        let mut state = self.state.clone();
        let mut overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let outcome = runtime.data.apply_random_special_routine(
            &mut state,
            routine,
            &runtime.music_ids(),
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        overworld.sync_event_flag_memory(&state.flags);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::GameCornerOpened(outcome),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::OpenGameCornerSpecial(RuntimeGameCornerCommand {
                service,
                divider_trace,
            }),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_scripted_wild_battle_completion(
        &mut self,
        runtime: &CrystalRuntime,
        origin: RuntimeStaticWildBattleOrigin,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let mut overworld = self.overworld.clone();
        let current_map = overworld.map.name.clone();
        let terminal = runtime
            .data
            .scripted_wild_battle_terminal(&state, &origin)?;
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        runtime.data.complete_scripted_wild_battle(
            &mut state,
            &mut overworld,
            &current_map,
            &origin,
            terminal,
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        overworld.sync_event_flag_memory(&state.flags);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::ScriptedWildBattleCompleted,
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::CompleteScriptedWildBattle(
                RuntimeScriptedWildBattleCompletionCommand {
                    origin,
                    terminal,
                    divider_trace,
                },
            ),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_scripted_trainer_battle_completion(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        command_index: usize,
        won: bool,
        can_lose: bool,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let mut overworld = self.overworld.clone();
        let current_map = self.overworld.map.name.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let completion = runtime.data.complete_scripted_trainer_battle(
            &mut state,
            &current_map,
            map_name,
            source_script,
            command_index,
            won,
            can_lose,
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        overworld.sync_event_flag_memory(&state.flags);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::ScriptedTrainerBattleCompleted(completion),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::CompleteScriptedTrainerBattle(
                RuntimeTrainerBattleCompletionCommand {
                    command: Self::script_command_ref(map_name, source_script, command_index),
                    won,
                    can_lose,
                    divider_trace,
                },
            ),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_wild_battle_rewards(
        &mut self,
        runtime: &CrystalRuntime,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let mut overworld = self.overworld.clone();
        let time_of_day = state.time.time_of_day;
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        let rewards = runtime.data.claim_active_wild_battle_rewards(
            &mut state,
            time_of_day,
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        overworld.sync_event_flag_memory(&state.flags);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::ActiveWildBattleRewardsClaimed(rewards),
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::ClaimActiveWildBattleRewardsNow(divider_trace),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_clock_update(
        &mut self,
        runtime: &CrystalRuntime,
        date: GameDate,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let mut overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        runtime.data.update_clock_from_datetime(
            &mut state,
            date,
            hour,
            minute,
            second,
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        overworld.sync_event_flag_memory(&state.flags);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::ClockUpdated,
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::UpdateClockFromDatetime(RuntimeClockUpdateCommand {
                date,
                hour,
                minute,
                second,
                divider_trace,
            }),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn stage_manual_clock_update(
        &mut self,
        runtime: &CrystalRuntime,
        now_date: GameDate,
        now_hour: u8,
        now_minute: u8,
        now_second: u8,
        target: ClockTime,
    ) -> Result<RecordedRuntimeMutation> {
        let mut state = self.state.clone();
        let mut overworld = self.overworld.clone();
        let mut divider_after = self.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        runtime.data.set_manual_clock_time(
            &mut state,
            now_date,
            now_hour,
            now_minute,
            now_second,
            target,
            &mut recording,
        )?;
        let divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        overworld.sync_event_flag_memory(&state.flags);
        let outcome = RuntimeMutationOutcome {
            result: RuntimeMutationResult::ManualClockSet,
            state_checksum: game_state_checksum(&state)?,
        };
        Ok(RecordedRuntimeMutation {
            command: RuntimeMutationCommand::SetManualClockTime(RuntimeManualClockCommand {
                now_date,
                now_hour,
                now_minute,
                now_second,
                target,
                divider_trace,
            }),
            state,
            overworld,
            outcome,
            divider_after: Some(divider_after),
        })
    }

    fn commit_recorded_mutation(
        &mut self,
        recorded: RecordedRuntimeMutation,
    ) -> RuntimeMutationOutcome {
        self.state = recorded.state;
        self.overworld = recorded.overworld;
        if let Some(divider_after) = recorded.divider_after {
            self.divider = divider_after;
        }
        recorded.outcome
    }

    pub fn apply_buttons(
        &mut self,
        runtime: &CrystalRuntime,
        _asset_root: &AssetRoot,
        buttons: impl IntoIterator<Item = GameButton>,
    ) -> Result<RuntimeOverworldFrame> {
        let recorded = self.stage_overworld_input(runtime, buttons.into_iter().collect(), true)?;
        let mutation = self.commit_recorded_mutation(recorded);
        let RuntimeMutationResult::OverworldInputApplied(frame) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-overworld-input result");
        };
        self.joypad = JoypadState::from_previous_mask(frame.input_mask);
        Ok(RuntimeOverworldFrame::from_input_frame(
            frame,
            mutation.state_checksum,
        ))
    }

    pub fn dispatch_interaction_script(
        &mut self,
        runtime: &CrystalRuntime,
        interaction: &OverworldInteraction,
    ) -> Result<RuntimeInteractionScriptDispatch> {
        let interaction = runtime
            .data
            .resolve_overworld_interaction_dispatch(&self.state, interaction)?
            .with_context(|| {
                format!(
                    "background interaction {}:{} is no longer eligible",
                    interaction.map_name, interaction.script
                )
            })?;
        if interaction.map_name != self.overworld.map.name {
            anyhow::bail!(
                "interaction script {} belongs to map {} but active overworld map is {}",
                interaction.script,
                interaction.map_name,
                self.overworld.map.name
            );
        }
        runtime.require_map(&interaction.map_name)?;
        if !runtime.has_script_label(&interaction.script)
            && !crate::core::world::collision::is_standard_interaction_script(&interaction.script)
        {
            runtime.require_script_label(&interaction.script)?;
        }
        let last_talked_object = match &interaction.target {
            OverworldInteractionTarget::Object {
                object_identifier, ..
            } => object_identifier.as_deref(),
            OverworldInteractionTarget::Background { .. }
            | OverworldInteractionTarget::Collision { .. } => None,
        };
        let dispatch = commit_interaction_script_dispatch(
            &mut self.state,
            &mut self.overworld.last_talked_object_identifier,
            &interaction.map_name,
            &interaction.script,
            last_talked_object,
        )
        .with_context(|| {
            format!(
                "dispatch interaction script {} on {}",
                interaction.script, interaction.map_name
            )
        })?;
        if let OverworldInteractionTarget::Object {
            object_identifier: Some(object_id),
            ..
        } = &interaction.target
        {
            let facing = match interaction.facing {
                Direction::Up => Direction::Down,
                Direction::Down => Direction::Up,
                Direction::Left => Direction::Right,
                Direction::Right => Direction::Left,
            };
            self.overworld
                .object_facings
                .insert(object_id.clone(), facing);
        }
        commit_overworld_snapshot(
            &mut self.state,
            &self.overworld.snapshot(),
            SpawnMemoryUpdate::Preserve,
        );
        self.overworld
            .set_time(self.state.time.registers.hours, self.state.time.time_of_day);
        Ok(RuntimeInteractionScriptDispatch {
            next_script: dispatch.next_script,
            last_talked_object: dispatch.last_talked_object,
            state_checksum: game_state_checksum(&self.state)?,
        })
    }

    pub fn dispatch_coord_event_script(
        &mut self,
        runtime: &CrystalRuntime,
        coord_event: &CoordEventTrigger,
    ) -> Result<RuntimeInteractionScriptDispatch> {
        if coord_event.map_name != self.overworld.map.name {
            anyhow::bail!(
                "coord event script {} belongs to map {} but active overworld map is {}",
                coord_event.script_name,
                coord_event.map_name,
                self.overworld.map.name
            );
        }
        runtime.require_map(&coord_event.map_name)?;
        runtime.require_script_label(&coord_event.script_name)?;
        let dispatch = commit_interaction_script_dispatch(
            &mut self.state,
            &mut self.overworld.last_talked_object_identifier,
            &coord_event.map_name,
            &coord_event.script_name,
            None,
        )
        .with_context(|| {
            format!(
                "dispatch coord event script {} on {} at ({}, {})",
                coord_event.script_name,
                coord_event.map_name,
                coord_event.tile.x,
                coord_event.tile.y
            )
        })?;
        commit_overworld_snapshot(
            &mut self.state,
            &self.overworld.snapshot(),
            SpawnMemoryUpdate::Preserve,
        );
        self.overworld
            .set_time(self.state.time.registers.hours, self.state.time.time_of_day);
        Ok(RuntimeInteractionScriptDispatch {
            next_script: dispatch.next_script,
            last_talked_object: dispatch.last_talked_object,
            state_checksum: game_state_checksum(&self.state)?,
        })
    }

    pub fn snapshot(&self) -> OverworldSnapshot {
        self.overworld.snapshot()
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut GameState {
        &mut self.state
    }

    #[cfg(any(test, feature = "test-fixtures"))]
    pub fn divider_for_tests(&self) -> &RuntimeDividerSource {
        &self.divider
    }

    #[cfg(any(test, feature = "test-fixtures"))]
    pub fn divider_mut_for_tests(&mut self) -> &mut RuntimeDividerSource {
        &mut self.divider
    }

    /// Low-level host access for presentation-driven world updates.
    /// Callers must synchronize authoritative state when changing gameplay data.
    pub fn overworld_mut(&mut self) -> &mut OverworldSession {
        &mut self.overworld
    }

    /// Borrow both halves together for existing host-side object synchronization.
    pub fn state_and_overworld_mut(&mut self) -> (&mut GameState, &mut OverworldSession) {
        (&mut self.state, &mut self.overworld)
    }

    pub fn overworld(&self) -> &OverworldSession {
        &self.overworld
    }

    pub fn state_checksum_frame(&self, player_id: PlayerId) -> Result<StateChecksumFrame> {
        StateChecksumFrame::from_game_state(player_id, &self.state)
            .context("checksum authoritative GameState for player")
    }

    pub fn runtime_command_frame(
        &self,
        player_id: PlayerId,
        sequence: u64,
        command: RuntimeMutationCommand,
    ) -> Result<RuntimeCommandFrame> {
        runtime_mutation_command_frame(player_id, sequence, &command, &self.state)
    }

    pub fn require_runtime_command_expected_state(
        &self,
        request: &RuntimeCommandFrame,
    ) -> Result<()> {
        decode_runtime_mutation_command_frame(request, &self.state).map(|_| ())
    }

    pub fn runtime_mutation_result_frame(
        &self,
        request: RuntimeCommandFrame,
        outcome: &RuntimeMutationOutcome,
    ) -> Result<RuntimeCommandResultFrame> {
        assets_runtime_mutation_result_frame(request, outcome, &self.state)
    }

    pub fn apply_runtime_mutation_command(
        &mut self,
        runtime: &CrystalRuntime,
        command: RuntimeMutationCommand,
    ) -> Result<RuntimeMutationOutcome> {
        runtime
            .data
            .apply_runtime_mutation_command(
                &mut self.state,
                &mut self.overworld,
                command,
                &runtime.audio.music_ids(),
                &runtime.audio.sound_effect_ids(),
                &runtime.audio.cry_ids(),
            )
            .context("apply runtime mutation command")
    }

    pub fn apply_runtime_command_frame(
        &mut self,
        runtime: &CrystalRuntime,
        request: &RuntimeCommandFrame,
    ) -> Result<RuntimeMutationOutcome> {
        let command = decode_runtime_mutation_command_frame(request, &self.state)?;
        self.apply_runtime_mutation_command(runtime, command)
    }

    fn script_command_ref(
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> RuntimeScriptCommandRef {
        RuntimeScriptCommandRef::new(map_name, source_script, command_index)
    }

    fn runtime_time_update_with_checksum(
        &self,
        state_checksum: StateChecksum,
    ) -> RuntimeTimeUpdate {
        RuntimeTimeUpdate {
            time_of_day: self.state.time.time_of_day,
            day_of_week: self.state.time.day_of_week,
            hour: self.state.time.registers.hours,
            minute: self.state.time.registers.minutes,
            state_checksum,
        }
    }

    pub fn start_scripted_wild_battle(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        startbattle_command_index: usize,
    ) -> Result<StaticWildBattleStart> {
        let recorded = self.stage_scripted_wild_battle_start(
            runtime,
            Self::script_command_ref(map_name, source_script, startbattle_command_index),
        )?;
        let mutation = self.commit_recorded_mutation(recorded);
        let RuntimeMutationResult::ScriptedWildBattleStarted(start) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-scripted-wild-battle-start result");
        };
        Ok(start)
    }

    pub fn start_scripted_trainer_battle(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        startbattle_command_index: usize,
    ) -> Result<TrainerBattleStartStatus> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::StartScriptedTrainerBattle(Self::script_command_ref(
                map_name,
                source_script,
                startbattle_command_index,
            )),
        )?;
        let RuntimeMutationResult::ScriptedTrainerBattleStarted(start) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-scripted-trainer-battle-start result");
        };
        Ok(start)
    }

    pub fn complete_scripted_wild_battle(
        &mut self,
        runtime: &CrystalRuntime,
        origin: RuntimeStaticWildBattleOrigin,
    ) -> Result<RuntimeScriptedBattleCompletion> {
        let recorded = self.stage_scripted_wild_battle_completion(runtime, origin)?;
        let mutation = self.commit_recorded_mutation(recorded);
        let RuntimeMutationResult::ScriptedWildBattleCompleted = mutation.result else {
            anyhow::bail!("runtime mutation returned non-scripted-wild-battle-completion result");
        };
        Ok(RuntimeScriptedBattleCompletion {
            continued_after_battle: true,
            trainer_prize_money: None,
            money_after: None,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn complete_scripted_trainer_battle(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        startbattle_command_index: usize,
        won: bool,
        can_lose: bool,
    ) -> Result<RuntimeScriptedBattleCompletion> {
        let recorded = self.stage_scripted_trainer_battle_completion(
            runtime,
            map_name,
            source_script,
            startbattle_command_index,
            won,
            can_lose,
        )?;
        let completion_mutation = self.commit_recorded_mutation(recorded);
        let RuntimeMutationResult::ScriptedTrainerBattleCompleted(completion_outcome) =
            completion_mutation.result
        else {
            anyhow::bail!(
                "runtime mutation returned non-scripted-trainer-battle-completion result"
            );
        };
        let continued_after_battle = completion_outcome.continued_after_battle;
        Ok(RuntimeScriptedBattleCompletion {
            continued_after_battle,
            trainer_prize_money: Some(completion_outcome.prize_money),
            money_after: Some(completion_outcome.money_after),
            state_checksum: completion_mutation.state_checksum,
        })
    }

    pub fn throw_ball_at_active_battle(
        &mut self,
        runtime: &CrystalRuntime,
        ball_id: &str,
    ) -> Result<RuntimeCaptureAttempt> {
        let recorded = self.stage_throw_ball_at_active_battle(runtime, ball_id)?;
        let mutation = self.commit_recorded_mutation(recorded);
        let RuntimeMutationResult::BallThrown(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-capture result");
        };
        Ok(RuntimeCaptureAttempt {
            outcome: Some(outcome),
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn complete_active_wild_capture(
        &mut self,
        runtime: &CrystalRuntime,
        outcome: &CaptureOutcome,
        nickname: Option<String>,
    ) -> Result<RuntimeCaptureCompletion> {
        let recorded = self.stage_active_wild_capture_completion(runtime, outcome, nickname)?;
        let mutation = self.commit_recorded_mutation(recorded);
        let RuntimeMutationResult::ActiveWildCaptureCompleted(CaptureCompletion {
            stored,
            contest_pokemon,
        }) = mutation.result
        else {
            anyhow::bail!("runtime mutation returned non-capture-completion result");
        };
        Ok(RuntimeCaptureCompletion {
            stored,
            contest_pokemon,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn resolve_bug_contest_caught_mon(
        &mut self,
        runtime: &CrystalRuntime,
        keep_new: bool,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::ResolveBugContestCaughtMon { keep_new },
        )?;
        let RuntimeMutationResult::SpecialRoutineApplied(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Bug Contest decision result");
        };
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn resolve_active_battle_turn(
        &mut self,
        runtime: &CrystalRuntime,
        player_action: BattleAction,
        enemy_action: BattleAction,
    ) -> Result<RuntimeBattleTurn> {
        let recorded = self.stage_active_battle_turn(runtime, player_action, enemy_action)?;
        let mutation = self.commit_recorded_mutation(recorded);
        let RuntimeMutationResult::ActiveBattleTurnResolved(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-battle-turn result");
        };
        Ok(RuntimeBattleTurn {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn resolve_active_battle_command(
        &mut self,
        runtime: &CrystalRuntime,
        player_action: BattleAction,
        enemy_action: BattleAction,
    ) -> Result<RuntimeBattleCommand> {
        let recorded = self.stage_active_battle_command(runtime, player_action, enemy_action)?;
        let mutation = self.commit_recorded_mutation(recorded);
        let RuntimeMutationResult::ActiveBattleCommandResolved(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-battle-command result");
        };
        Ok(RuntimeBattleCommand {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn resolve_active_battle_enemy_action(
        &mut self,
        runtime: &CrystalRuntime,
        enemy_action: BattleAction,
    ) -> Result<RuntimeBattleTurn> {
        let recorded = self.stage_active_battle_enemy_action(runtime, enemy_action)?;
        let mutation = self.commit_recorded_mutation(recorded);
        let RuntimeMutationResult::ActiveBattleEnemyActionResolved(outcome) = mutation.result
        else {
            anyhow::bail!("runtime mutation returned non-enemy-battle-action result");
        };
        Ok(RuntimeBattleTurn {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn attempt_escape_active_wild_battle(
        &mut self,
        runtime: &CrystalRuntime,
    ) -> Result<RuntimeBattleEscape> {
        let recorded = self.stage_escape_active_wild_battle(runtime)?;
        let mutation = self.commit_recorded_mutation(recorded);
        let RuntimeMutationResult::ActiveWildBattleEscapeAttempted(outcome) = mutation.result
        else {
            anyhow::bail!("runtime mutation returned non-battle-escape result");
        };
        Ok(RuntimeBattleEscape {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_item_to_escape_active_wild_battle(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
    ) -> Result<RuntimeBattleEscapeItemUse> {
        let recorded = self.stage_bag_item_escape_active_wild_battle(runtime, item_id)?;
        let mutation = self.commit_recorded_mutation(recorded);
        let RuntimeMutationResult::ActiveWildBattleEscapeItemUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-battle-escape-item result");
        };
        Ok(RuntimeBattleEscapeItemUse {
            item_use: outcome.item_use,
            battle_escape_mode: outcome.battle_escape_mode,
            escaped: outcome.escaped,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_guard_spec_in_active_battle(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
    ) -> Result<RuntimeBattleStateItemUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseBagGuardSpecInActiveBattle(RuntimeItemCommand {
                item_id: item_id.to_string(),
            }),
        )?;
        let RuntimeMutationResult::ActiveBattleGuardSpecUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-battle-state-item result");
        };
        Ok(RuntimeBattleStateItemUse {
            item_use: outcome.item_use,
            mist_active_before: outcome.mist_active_before,
            mist_active_after: outcome.mist_active_after,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn switch_active_battle_party(
        &mut self,
        runtime: &CrystalRuntime,
        party_index: usize,
    ) -> Result<RuntimeBattlePartySwitch> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::SwitchActiveBattleParty(RuntimePartySlotCommand {
                party_index,
            }),
        )?;
        let RuntimeMutationResult::ActiveBattlePartySwitched(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-active-battle-party-switch result");
        };
        Ok(RuntimeBattlePartySwitch {
            party_index: outcome.party_index,
            spikes: outcome.spikes,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn advance_active_trainer_battle(
        &mut self,
        runtime: &CrystalRuntime,
    ) -> Result<RuntimeTrainerBattleAdvance> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::AdvanceActiveTrainerBattle,
        )?;
        let RuntimeMutationResult::ActiveTrainerBattleAdvanced(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-trainer-battle-advance result");
        };
        Ok(RuntimeTrainerBattleAdvance {
            next_enemy: outcome.next_enemy,
            trainer_defeated: outcome.trainer_defeated,
            spikes: outcome.spikes,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn claim_active_trainer_battle_rewards(
        &mut self,
        runtime: &CrystalRuntime,
    ) -> Result<RuntimeBattleRewards> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::ClaimActiveTrainerBattleRewardsNow,
        )?;
        let RuntimeMutationResult::ActiveTrainerBattleRewardsClaimed(outcome) = mutation.result
        else {
            anyhow::bail!("runtime mutation returned non-trainer-rewards result");
        };
        Ok(RuntimeBattleRewards {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn claim_active_wild_battle_rewards(
        &mut self,
        runtime: &CrystalRuntime,
    ) -> Result<RuntimeBattleRewards> {
        let recorded = self.stage_wild_battle_rewards(runtime)?;
        let mutation = self.commit_recorded_mutation(recorded);
        let RuntimeMutationResult::ActiveWildBattleRewardsClaimed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-wild-rewards result");
        };
        Ok(RuntimeBattleRewards {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn grant_scripted_gift_pokemon(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        command_index: usize,
        original_trainer_name: impl Into<String>,
        original_trainer_id: u16,
        nickname_accepted: bool,
        nickname: Option<String>,
    ) -> Result<RuntimeGiftPokemonGrant> {
        let recorded = self.stage_scripted_gift_pokemon(
            runtime,
            RuntimeScriptCommandRef::new(map_name, source_script, command_index),
            original_trainer_name.into(),
            original_trainer_id,
            nickname_accepted,
            nickname,
        )?;
        let mutation = self.commit_recorded_mutation(recorded);
        let RuntimeMutationResult::ScriptedGiftPokemonGranted(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-gift-Pokemon result");
        };
        Ok(RuntimeGiftPokemonGrant {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_item(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
        context: ItemUseContext,
    ) -> Result<RuntimeItemUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseBagItem {
                item_id: item_id.to_string(),
                context,
            },
        )?;
        let RuntimeMutationResult::BagItemUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-item-use result");
        };
        Ok(RuntimeItemUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn register_key_item(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
    ) -> Result<RuntimeRegisteredKeyItem> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::RegisterKeyItem(RuntimeRegisteredKeyItemCommand {
                item_id: item_id.to_string(),
            }),
        )?;
        let RuntimeMutationResult::KeyItemRegistered(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-key-item-registration result");
        };
        Ok(RuntimeRegisteredKeyItem {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_repel_in_field(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
    ) -> Result<RuntimeRepelItemUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseBagRepelInField(RuntimeItemCommand {
                item_id: item_id.to_string(),
            }),
        )?;
        let RuntimeMutationResult::FieldRepelUsed(repel) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-repel result");
        };
        Ok(RuntimeRepelItemUse {
            item_use: repel.item_use,
            repel_steps_before: repel.repel_steps_before,
            repel_steps_after: repel.repel_steps_after,
            active_repel_item_before: repel.active_repel_item_before,
            active_repel_item_after: repel.active_repel_item_after,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_bicycle_in_field(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
    ) -> Result<RuntimeBicycleItemUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseBagBicycleInField(RuntimeItemCommand {
                item_id: item_id.to_string(),
            }),
        )?;
        let RuntimeMutationResult::FieldBicycleUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-bicycle result");
        };
        Ok(RuntimeBicycleItemUse {
            item_use: outcome.item_use,
            map_name: outcome.map_name,
            permission: outcome.permission,
            mode_before: outcome.mode_before,
            mode_after: outcome.mode_after,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_itemfinder_in_field(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
    ) -> Result<RuntimeItemfinderUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseBagItemfinderInField(RuntimeItemCommand {
                item_id: item_id.to_string(),
            }),
        )?;
        let RuntimeMutationResult::FieldItemfinderUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-itemfinder result");
        };
        Ok(RuntimeItemfinderUse {
            item_use: outcome.item_use,
            player_tile: outcome.player_tile,
            itemfinder_sound_cues: outcome.itemfinder_sound_cues,
            found: outcome.found,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_squirtbottle_in_field(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
    ) -> Result<RuntimeSquirtBottleUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseBagSquirtbottleInField(RuntimeItemCommand {
                item_id: item_id.to_string(),
            }),
        )?;
        let RuntimeMutationResult::FieldSquirtbottleUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-squirtbottle result");
        };
        Ok(RuntimeSquirtBottleUse {
            item_use: outcome.item_use,
            player_tile: outcome.player_tile,
            target_tile: outcome.target_tile,
            target_object_identifier: outcome.target_object_identifier,
            target_movement: outcome.target_movement,
            target_script: outcome.target_script,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_story_key_in_field(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
    ) -> Result<RuntimeStoryKeyUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseBagStoryKeyInField(RuntimeItemCommand {
                item_id: item_id.to_string(),
            }),
        )?;
        let RuntimeMutationResult::FieldStoryKeyUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-story-key result");
        };
        Ok(RuntimeStoryKeyUse {
            item_use: outcome.item_use,
            map_name: outcome.map_name,
            player_tile: outcome.player_tile,
            target_tile: outcome.target_tile,
            target_script: outcome.target_script,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_coin_case_in_field(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
    ) -> Result<RuntimeKeyItemBalanceUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseBagCoinCaseInField(RuntimeItemCommand {
                item_id: item_id.to_string(),
            }),
        )?;
        let RuntimeMutationResult::FieldCoinCaseUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-coin-case result");
        };
        Ok(RuntimeKeyItemBalanceUse {
            item_use: outcome.item_use,
            balance_label: outcome.balance_label,
            balance: outcome.balance,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_blue_card_in_field(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
    ) -> Result<RuntimeKeyItemBalanceUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseBagBlueCardInField(RuntimeItemCommand {
                item_id: item_id.to_string(),
            }),
        )?;
        let RuntimeMutationResult::FieldBlueCardUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-blue-card result");
        };
        Ok(RuntimeKeyItemBalanceUse {
            item_use: outcome.item_use,
            balance_label: outcome.balance_label,
            balance: outcome.balance,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_town_map_in_field(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
    ) -> Result<RuntimeTownMapUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseBagTownMapInField(RuntimeItemCommand {
                item_id: item_id.to_string(),
            }),
        )?;
        let RuntimeMutationResult::FieldTownMapUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-town-map result");
        };
        Ok(RuntimeTownMapUse {
            item_use: outcome.item_use,
            map_name: outcome.map_name,
            map_constant: outcome.map_constant,
            environment: outcome.environment,
            landmark: outcome.landmark,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_pokegear_in_field(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
    ) -> Result<RuntimePokegearUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseBagPokegearInField(RuntimeItemCommand {
                item_id: item_id.to_string(),
            }),
        )?;
        let RuntimeMutationResult::FieldPokegearUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-pokegear result");
        };
        Ok(RuntimePokegearUse {
            item_use: outcome.item_use,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_box_in_field(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
    ) -> Result<RuntimeBoxItemUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseBagBoxInField(RuntimeItemCommand {
                item_id: item_id.to_string(),
            }),
        )?;
        let RuntimeMutationResult::FieldBoxUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-box result");
        };
        Ok(RuntimeBoxItemUse {
            item_use: outcome.item_use,
            decoration_flag: outcome.decoration_flag,
            already_owned: outcome.already_owned,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_escape_rope_in_field(
        &mut self,
        runtime: &CrystalRuntime,
        _asset_root: &AssetRoot,
        item_id: &str,
    ) -> Result<RuntimeEscapeRopeUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseBagEscapeRopeInField(RuntimeItemCommand {
                item_id: item_id.to_string(),
            }),
        )?;
        let RuntimeMutationResult::FieldEscapeRopeUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-escape-rope result");
        };
        Ok(RuntimeEscapeRopeUse {
            item_use: outcome.item_use,
            source_map: outcome.source_map,
            destination_map: outcome.destination_map,
            destination_warp_index: outcome.destination_warp_index,
            destination_tile: outcome.destination_tile,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_cut_field_move(
        &mut self,
        runtime: &CrystalRuntime,
        party_index: usize,
        metatile_x: u16,
        metatile_y: u16,
    ) -> Result<RuntimeFieldMoveBlockUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseCutFieldMove(RuntimeFieldBlockMoveCommand {
                party_index,
                metatile_x,
                metatile_y,
            }),
        )?;
        let RuntimeMutationResult::CutFieldMoveUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-CUT result");
        };
        Ok(RuntimeFieldMoveBlockUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_cut_field_move_in_front(
        &mut self,
        runtime: &CrystalRuntime,
        party_index: usize,
    ) -> Result<RuntimeFieldMoveBlockUse> {
        let (metatile_x, metatile_y) = runtime
            .data()
            .field_block_target_metatile_in_front(&self.overworld)?;
        self.use_cut_field_move(runtime, party_index, metatile_x, metatile_y)
    }

    pub fn use_whirlpool_field_move(
        &mut self,
        runtime: &CrystalRuntime,
        party_index: usize,
        metatile_x: u16,
        metatile_y: u16,
    ) -> Result<RuntimeFieldMoveBlockUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseWhirlpoolFieldMove(RuntimeFieldBlockMoveCommand {
                party_index,
                metatile_x,
                metatile_y,
            }),
        )?;
        let RuntimeMutationResult::WhirlpoolFieldMoveUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-WHIRLPOOL result");
        };
        Ok(RuntimeFieldMoveBlockUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_whirlpool_field_move_in_front(
        &mut self,
        runtime: &CrystalRuntime,
        party_index: usize,
    ) -> Result<RuntimeFieldMoveBlockUse> {
        let (metatile_x, metatile_y) = runtime
            .data()
            .field_block_target_metatile_in_front(&self.overworld)?;
        self.use_whirlpool_field_move(runtime, party_index, metatile_x, metatile_y)
    }

    pub fn queue_strength_from_menu(
        &mut self,
        runtime: &CrystalRuntime,
        party_index: usize,
    ) -> Result<RuntimeInteractionScriptDispatch> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::QueueStrengthFromMenu(RuntimeFieldPartyCommand { party_index }),
        )?;
        let RuntimeMutationResult::StrengthFromMenuQueued(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-StrengthFromMenu result");
        };
        Ok(RuntimeInteractionScriptDispatch {
            next_script: outcome.next_script,
            last_talked_object: None,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_flash_field_move(
        &mut self,
        runtime: &CrystalRuntime,
        party_index: usize,
    ) -> Result<RuntimeFieldMoveFlagUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseFlashFieldMove(RuntimeFieldPartyCommand { party_index }),
        )?;
        let RuntimeMutationResult::FlashFieldMoveUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-FLASH result");
        };
        Ok(RuntimeFieldMoveFlagUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_surf_field_move(
        &mut self,
        runtime: &CrystalRuntime,
        party_index: usize,
    ) -> Result<RuntimeFieldMoveTravelUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseSurfFieldMove(RuntimeFieldPartyCommand { party_index }),
        )?;
        let RuntimeMutationResult::SurfFieldMoveUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-SURF result");
        };
        Ok(RuntimeFieldMoveTravelUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_waterfall_field_move(
        &mut self,
        runtime: &CrystalRuntime,
        party_index: usize,
    ) -> Result<RuntimeFieldMoveTravelUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseWaterfallFieldMove(RuntimeFieldPartyCommand { party_index }),
        )?;
        let RuntimeMutationResult::WaterfallFieldMoveUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-WATERFALL result");
        };
        Ok(RuntimeFieldMoveTravelUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_fly_field_move(
        &mut self,
        runtime: &CrystalRuntime,
        _asset_root: &AssetRoot,
        party_index: usize,
        destination_spawn_identifier: u16,
        flypoint_flag: &str,
    ) -> Result<RuntimeFlyFieldMoveUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseFlyFieldMove(RuntimeFlyCommand {
                party_index,
                destination_spawn_identifier,
                flypoint_flag: flypoint_flag.to_string(),
            }),
        )?;
        let RuntimeMutationResult::FlyFieldMoveUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-FLY result");
        };
        Ok(RuntimeFlyFieldMoveUse {
            actor_party_index: outcome.actor_party_index,
            actor_species: outcome.actor_species,
            flypoint_flag: outcome.flypoint_flag,
            source_map: outcome.source_map,
            destination_spawn_identifier: outcome.destination_spawn_identifier,
            destination_map: outcome.destination_map,
            destination_tile: outcome.destination_tile,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_dig_field_move(
        &mut self,
        runtime: &CrystalRuntime,
        _asset_root: &AssetRoot,
        party_index: usize,
    ) -> Result<RuntimeDigFieldMoveUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseDigFieldMove(RuntimeFieldPartyCommand { party_index }),
        )?;
        let RuntimeMutationResult::DigFieldMoveUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-DIG result");
        };
        Ok(RuntimeDigFieldMoveUse {
            actor_party_index: outcome.actor_party_index,
            actor_species: outcome.actor_species,
            source_map: outcome.source_map,
            destination_map: outcome.destination_map,
            destination_warp_index: outcome.destination_warp_index,
            destination_tile: outcome.destination_tile,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_teleport_field_move(
        &mut self,
        runtime: &CrystalRuntime,
        _asset_root: &AssetRoot,
        party_index: usize,
    ) -> Result<RuntimeTeleportFieldMoveUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseTeleportFieldMove(RuntimeFieldPartyCommand { party_index }),
        )?;
        let RuntimeMutationResult::TeleportFieldMoveUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-TELEPORT result");
        };
        Ok(RuntimeTeleportFieldMoveUse {
            actor_party_index: outcome.actor_party_index,
            actor_species: outcome.actor_species,
            source_map: outcome.source_map,
            destination_spawn_identifier: outcome.destination_spawn_identifier,
            destination_map: outcome.destination_map,
            destination_tile: outcome.destination_tile,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn commit_pending_field_travel(
        &mut self,
        runtime: &CrystalRuntime,
    ) -> Result<PendingFieldTravel> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::CommitPendingFieldTravel,
        )?;
        let RuntimeMutationResult::FieldTravelCommitted(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-field-travel commit result");
        };
        Ok(outcome)
    }

    pub fn queue_headbutt_script(
        &mut self,
        runtime: &CrystalRuntime,
        party_index: usize,
        from_menu: bool,
    ) -> Result<RuntimeInteractionScriptDispatch> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::QueueHeadbuttScript(RuntimeHeadbuttScriptCommand {
                party_index,
                from_menu,
            }),
        )?;
        let RuntimeMutationResult::HeadbuttScriptQueued(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-HeadbuttScript result");
        };
        Ok(RuntimeInteractionScriptDispatch {
            next_script: outcome.next_script,
            last_talked_object: None,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn queue_rock_smash_from_menu(
        &mut self,
        runtime: &CrystalRuntime,
        party_index: usize,
    ) -> Result<RuntimeInteractionScriptDispatch> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::QueueRockSmashFromMenu(RuntimeFieldPartyCommand {
                party_index,
            }),
        )?;
        let RuntimeMutationResult::RockSmashFromMenuQueued(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-RockSmashFromMenu result");
        };
        Ok(RuntimeInteractionScriptDispatch {
            next_script: outcome.next_script,
            last_talked_object: Some(outcome.object_identifier),
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn queue_sweet_scent_from_menu(
        &mut self,
        runtime: &CrystalRuntime,
        party_index: usize,
    ) -> Result<RuntimeInteractionScriptDispatch> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::QueueSweetScentFromMenu(RuntimeFieldPartyCommand {
                party_index,
            }),
        )?;
        let RuntimeMutationResult::SweetScentFromMenuQueued(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-SweetScentFromMenu result");
        };
        Ok(RuntimeInteractionScriptDispatch {
            next_script: outcome.next_script,
            last_talked_object: None,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_item_on_party_pokemon(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
        party_index: usize,
    ) -> Result<RuntimePartyItemUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseBagItemOnPartyPokemon(RuntimePartyItemCommand {
                item_id: item_id.to_string(),
                party_index,
            }),
        )?;
        let RuntimeMutationResult::PartyPokemonItemUsed(item_use, item_effect) = mutation.result
        else {
            anyhow::bail!("runtime mutation returned non-party-item result");
        };
        Ok(RuntimePartyItemUse {
            item_use,
            item_effect,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_item_on_whole_party(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
    ) -> Result<RuntimeWholePartyItemUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseBagItemOnWholeParty(RuntimeItemCommand {
                item_id: item_id.to_string(),
            }),
        )?;
        let RuntimeMutationResult::WholePartyItemUsed(item_use, item_effect) = mutation.result
        else {
            anyhow::bail!("runtime mutation returned non-whole-party-item result");
        };
        Ok(RuntimeWholePartyItemUse {
            item_use,
            item_effect,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_item_on_party_move(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
        party_index: usize,
        move_slot: Option<usize>,
    ) -> Result<RuntimePartyItemUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseBagItemOnPartyMove(RuntimePartyMoveItemCommand {
                item_id: item_id.to_string(),
                party_index,
                move_slot,
            }),
        )?;
        let RuntimeMutationResult::PartyMoveItemUsed(item_use, item_effect) = mutation.result
        else {
            anyhow::bail!("runtime mutation returned non-party-move-item result");
        };
        Ok(RuntimePartyItemUse {
            item_use,
            item_effect,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_tmhm_on_party_pokemon(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
        party_index: usize,
        replace_slot: Option<usize>,
    ) -> Result<RuntimeTmHmItemUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseBagTmHmOnPartyPokemon(RuntimeTmHmCommand {
                item_id: item_id.to_string(),
                party_index,
                replace_slot,
            }),
        )?;
        let RuntimeMutationResult::TmHmItemUsed(item_use, learned_move) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-TM/HM result");
        };
        Ok(RuntimeTmHmItemUse {
            item_use,
            learned_move,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_item_on_active_battle_pokemon(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
    ) -> Result<RuntimeBattleItemUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseBagItemOnActiveBattlePokemon(RuntimeItemCommand {
                item_id: item_id.to_string(),
            }),
        )?;
        let RuntimeMutationResult::ActiveBattlePokemonItemUsed(item_use, battle_item) =
            mutation.result
        else {
            anyhow::bail!("runtime mutation returned non-active-battle-item result");
        };
        Ok(RuntimeBattleItemUse {
            item_use,
            battle_item,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_item_on_battle_party_pokemon(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
        party_index: usize,
    ) -> Result<RuntimeBattleItemUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseBagItemOnBattlePartyPokemon(RuntimePartyItemCommand {
                item_id: item_id.to_string(),
                party_index,
            }),
        )?;
        let RuntimeMutationResult::BattlePartyPokemonItemUsed(item_use, battle_item) =
            mutation.result
        else {
            anyhow::bail!("runtime mutation returned non-battle-party-item result");
        };
        Ok(RuntimeBattleItemUse {
            item_use,
            battle_item,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_item_on_battle_party_move(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
        party_index: usize,
        move_slot: Option<usize>,
    ) -> Result<RuntimeBattleItemUse> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::UseBagItemOnBattlePartyMove(RuntimePartyMoveItemCommand {
                item_id: item_id.to_string(),
                party_index,
                move_slot,
            }),
        )?;
        let RuntimeMutationResult::BattlePartyMoveItemUsed(item_use, battle_item) = mutation.result
        else {
            anyhow::bail!("runtime mutation returned non-battle-party-move-item result");
        };
        Ok(RuntimeBattleItemUse {
            item_use,
            battle_item,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn update_clock_from_datetime(
        &mut self,
        runtime: &CrystalRuntime,
        date: GameDate,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<RuntimeTimeUpdate> {
        let recorded = self.stage_clock_update(runtime, date, hour, minute, second)?;
        let mutation = self.commit_recorded_mutation(recorded);
        let RuntimeMutationResult::ClockUpdated = mutation.result else {
            anyhow::bail!("runtime mutation returned non-clock-update result");
        };
        Ok(self.runtime_time_update_with_checksum(mutation.state_checksum))
    }

    pub fn set_manual_clock_time(
        &mut self,
        runtime: &CrystalRuntime,
        now_date: GameDate,
        now_hour: u8,
        now_minute: u8,
        now_second: u8,
        target: ClockTime,
    ) -> Result<RuntimeTimeUpdate> {
        let recorded = self.stage_manual_clock_update(
            runtime, now_date, now_hour, now_minute, now_second, target,
        )?;
        let mutation = self.commit_recorded_mutation(recorded);
        let RuntimeMutationResult::ManualClockSet = mutation.result else {
            anyhow::bail!("runtime mutation returned non-manual-clock result");
        };
        Ok(self.runtime_time_update_with_checksum(mutation.state_checksum))
    }

    pub fn cast_fishing_rod(
        &mut self,
        runtime: &CrystalRuntime,
        rod: &str,
    ) -> Result<RuntimeFishingCast> {
        let recorded = self.stage_fishing_rod_cast(runtime, rod)?;
        let mutation = self.commit_recorded_mutation(recorded);
        let RuntimeMutationResult::FishingRodCast(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-fishing-cast result");
        };
        Ok(RuntimeFishingCast {
            checked_water_target: outcome.checked_water_target,
            session: outcome.session,
            bite: outcome.bite,
            wild_battle: outcome.wild_battle,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_fishing_rod_in_field(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
    ) -> Result<RuntimeFishingRodItemUse> {
        let recorded = self.stage_bag_fishing_rod_use(runtime, item_id)?;
        let mutation = self.commit_recorded_mutation(recorded);
        let RuntimeMutationResult::BagFishingRodUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-fishing-rod-item result");
        };
        Ok(RuntimeFishingRodItemUse {
            item_use: outcome.item_use,
            rod: outcome.rod,
            cast: RuntimeFishingCast {
                checked_water_target: outcome.cast.checked_water_target,
                session: outcome.cast.session,
                bite: outcome.cast.bite,
                wild_battle: outcome.cast.wild_battle,
                state_checksum: outcome.cast_state_checksum,
            },
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn grant_script_item(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptItemGrant> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::GrantScriptItem(Self::script_command_ref(
                map_name,
                source_script,
                command_index,
            )),
        )?;
        let RuntimeMutationResult::ScriptItemGranted(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-item-grant result");
        };
        Ok(RuntimeScriptItemGrant {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn check_script_item(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptItemCheck> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::CheckScriptItem(Self::script_command_ref(
                map_name,
                source_script,
                command_index,
            )),
        )?;
        let RuntimeMutationResult::ScriptItemChecked(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-item-check result");
        };
        Ok(RuntimeScriptItemCheck {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn take_script_item(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptItemTake> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::TakeScriptItem(Self::script_command_ref(
                map_name,
                source_script,
                command_index,
            )),
        )?;
        let RuntimeMutationResult::ScriptItemTaken(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-item-take result");
        };
        Ok(RuntimeScriptItemTake {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    #[cfg(any(test, feature = "test-fixtures"))]
    pub fn apply_special_routine(
        &mut self,
        runtime: &CrystalRuntime,
        routine: &str,
    ) -> Result<RuntimeSpecialRoutineUse> {
        if runtime_special_routine_requires_divider_trace(routine) {
            let recorded = self.stage_random_special_routine(runtime, routine)?;
            let mutation = self.commit_recorded_mutation(recorded);
            let RuntimeMutationResult::SpecialRoutineApplied(outcome) = mutation.result else {
                anyhow::bail!("runtime mutation returned non-special-routine result");
            };
            return Ok(RuntimeSpecialRoutineUse {
                outcome,
                state_checksum: mutation.state_checksum,
            });
        }
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::ApplySpecialRoutine {
                routine: routine.to_string(),
            },
        )?;
        let RuntimeMutationResult::SpecialRoutineApplied(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-special-routine result");
        };
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn pickup_script_field_item(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeFieldPickup> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::PickupScriptFieldItem(Self::script_command_ref(
                map_name,
                source_script,
                command_index,
            )),
        )?;
        let RuntimeMutationResult::ScriptFieldItemPickedUp(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-field-pickup result");
        };
        Ok(RuntimeFieldPickup {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_script_economy_command(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptEconomy> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::ApplyScriptEconomy(Self::script_command_ref(
                map_name,
                source_script,
                command_index,
            )),
        )?;
        let RuntimeMutationResult::ScriptEconomyApplied(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-economy result");
        };
        Ok(RuntimeScriptEconomy {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn initialize_permanent_phone_numbers(
        &mut self,
        runtime: &CrystalRuntime,
    ) -> Result<RuntimePermanentPhoneNumbers> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::InitializePermanentPhoneNumbers,
        )?;
        let RuntimeMutationResult::PermanentPhoneNumbersInitialized(inserted) = mutation.result
        else {
            anyhow::bail!("runtime mutation returned non-permanent-phone-number result");
        };
        Ok(RuntimePermanentPhoneNumbers {
            inserted,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_script_phone_command(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        command_index: usize,
        inputs: ScriptPhoneInputs,
    ) -> Result<RuntimePhoneCommand> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::ApplyScriptPhone {
                command: Self::script_command_ref(map_name, source_script, command_index),
                inputs,
            },
        )?;
        let RuntimeMutationResult::ScriptPhoneApplied(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-phone result");
        };
        Ok(RuntimePhoneCommand {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_script_flag_mutation(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeFlagMutation> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::ApplyScriptFlagMutation(Self::script_command_ref(
                map_name,
                source_script,
                command_index,
            )),
        )?;
        let RuntimeMutationResult::ScriptFlagMutated(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-flag-mutation result");
        };
        Ok(RuntimeFlagMutation {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn check_script_flag(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeFlagCheck> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::CheckScriptFlag(Self::script_command_ref(
                map_name,
                source_script,
                command_index,
            )),
        )?;
        let RuntimeMutationResult::ScriptFlagChecked(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-flag-check result");
        };
        Ok(RuntimeFlagCheck {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_script_scene_command(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeSceneCommand> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::ApplyScriptScene(Self::script_command_ref(
                map_name,
                source_script,
                command_index,
            )),
        )?;
        let RuntimeMutationResult::ScriptSceneApplied(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-scene result");
        };
        Ok(RuntimeSceneCommand {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_script_block_change(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeBlockChange> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::ApplyScriptBlockChange(Self::script_command_ref(
                map_name,
                source_script,
                command_index,
            )),
        )?;
        let RuntimeMutationResult::ScriptBlockChanged(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-block-change result");
        };
        Ok(RuntimeBlockChange {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_script_audio_command(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptAudio> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::ApplyScriptAudio(Self::script_command_ref(
                map_name,
                source_script,
                command_index,
            )),
        )?;
        let RuntimeMutationResult::ScriptAudioApplied(cue) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-audio result");
        };
        Ok(RuntimeScriptAudio {
            cue,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_script_map_command(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptMapCommand> {
        let command = Self::script_command_ref(map_name, source_script, command_index);
        let mutation = if runtime
            .data()
            .script_map_command(map_name, source_script, command_index)?
            .command
            == "reloadmapafterbattle"
        {
            let recorded = self.stage_random_script_map(runtime, command)?;
            self.commit_recorded_mutation(recorded)
        } else {
            self.apply_runtime_mutation_command(
                runtime,
                RuntimeMutationCommand::ApplyScriptMap(command),
            )?
        };
        let RuntimeMutationResult::ScriptMapApplied(action) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-map result");
        };
        Ok(RuntimeScriptMapCommand {
            action,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn execute_pending_script_warp(
        &mut self,
        runtime: &CrystalRuntime,
        _asset_root: &AssetRoot,
    ) -> Result<RuntimeScriptWarp> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::TransitionPendingScriptWarp,
        )?;
        let RuntimeMutationResult::PendingScriptWarpTransitioned(request) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-warp result");
        };
        Ok(RuntimeScriptWarp {
            target_map: request.target_map,
            tile: request.tile,
            facing: request.facing,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_script_text_command(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptText> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::ApplyScriptText(Self::script_command_ref(
                map_name,
                source_script,
                command_index,
            )),
        )?;
        let RuntimeMutationResult::ScriptTextApplied(action) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-text result");
        };
        Ok(RuntimeScriptText {
            action,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_script_variable_command(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptVariable> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::ApplyScriptVariableNow(Self::script_command_ref(
                map_name,
                source_script,
                command_index,
            )),
        )?;
        let RuntimeMutationResult::ScriptVariableApplied(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-variable result");
        };
        Ok(RuntimeScriptVariable {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_script_swarm_command(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptSwarm> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::ApplyScriptSwarm(Self::script_command_ref(
                map_name,
                source_script,
                command_index,
            )),
        )?;
        let RuntimeMutationResult::ScriptSwarmApplied(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-swarm result");
        };
        Ok(RuntimeScriptSwarm {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_script_control_command(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptControl> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::ApplyScriptControl(Self::script_command_ref(
                map_name,
                source_script,
                command_index,
            )),
        )?;
        let RuntimeMutationResult::ScriptControlApplied(action) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-control result");
        };
        Ok(RuntimeScriptControl {
            action,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_script_object_mutation(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptObjectMutation> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::ApplyScriptObjectMutation(Self::script_command_ref(
                map_name,
                source_script,
                command_index,
            )),
        )?;
        let RuntimeMutationResult::ScriptObjectMutated(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-object result");
        };
        Ok(RuntimeScriptObjectMutation {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_script_movement(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptMovement> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::ApplyScriptMovement(Self::script_command_ref(
                map_name,
                source_script,
                command_index,
            )),
        )?;
        let RuntimeMutationResult::ScriptMovementApplied(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-movement result");
        };
        Ok(RuntimeScriptMovement {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_script_runtime_command(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        command_index: usize,
        inputs: ScriptRuntimeInputs,
    ) -> Result<RuntimeScriptRuntimeCommand> {
        let command = Self::script_command_ref(map_name, source_script, command_index);
        let is_random = runtime
            .data()
            .script_runtime_command(map_name, source_script, command_index)?
            .command
            == "random";
        let mutation = if is_random {
            anyhow::ensure!(
                inputs == ScriptRuntimeInputs::default(),
                "script random command must not declare generic runtime inputs"
            );
            let recorded = self.stage_random_script_runtime(runtime, command)?;
            self.commit_recorded_mutation(recorded)
        } else {
            self.apply_runtime_mutation_command(
                runtime,
                RuntimeMutationCommand::ApplyScriptRuntime { command, inputs },
            )?
        };
        let RuntimeMutationResult::ScriptRuntimeApplied(_, outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-runtime result");
        };
        Ok(RuntimeScriptRuntimeCommand {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn execute_next_queued_script_command(
        &mut self,
        runtime: &CrystalRuntime,
    ) -> Result<RuntimeQueuedScriptCommand> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::ExecuteNextQueuedScriptCommand,
        )?;
        let RuntimeMutationResult::QueuedScriptCommandExecuted(queued) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-queued-script-command result");
        };
        Ok(RuntimeQueuedScriptCommand {
            queued,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn take_next_script(&mut self, runtime: &CrystalRuntime) -> Result<RuntimeNextScript> {
        let mutation =
            self.apply_runtime_mutation_command(runtime, RuntimeMutationCommand::TakeNextScript)?;
        let RuntimeMutationResult::NextScriptTaken(location) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-next-script result");
        };
        Ok(RuntimeNextScript {
            origin_map_name: location.origin_map_name,
            script: location.script,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn pop_script_call_stack(
        &mut self,
        runtime: &CrystalRuntime,
    ) -> Result<RuntimeScriptReturnResume> {
        let mutation = self
            .apply_runtime_mutation_command(runtime, RuntimeMutationCommand::PopScriptCallStack)?;
        let RuntimeMutationResult::ScriptCallStackPopped(frame) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-call-stack-pop result");
        };
        Ok(RuntimeScriptReturnResume {
            frame,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn pop_deferred_script(
        &mut self,
        runtime: &CrystalRuntime,
    ) -> Result<RuntimeDeferredScript> {
        let mutation = self
            .apply_runtime_mutation_command(runtime, RuntimeMutationCommand::PopDeferredScript)?;
        let RuntimeMutationResult::DeferredScriptPopped(location) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-deferred-script-pop result");
        };
        Ok(RuntimeDeferredScript {
            origin_map_name: location.origin_map_name,
            script: location.script,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn take_script_end_state(&mut self, runtime: &CrystalRuntime) -> Result<RuntimeScriptEnd> {
        let mutation = self
            .apply_runtime_mutation_command(runtime, RuntimeMutationCommand::TakeScriptEndState)?;
        let RuntimeMutationResult::ScriptEndStateTaken(end) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-end-state-take result");
        };
        Ok(RuntimeScriptEnd {
            end,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn open_script_shop(
        &mut self,
        runtime: &CrystalRuntime,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptShop> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::OpenScriptShop(Self::script_command_ref(
                map_name,
                source_script,
                command_index,
            )),
        )?;
        let RuntimeMutationResult::ScriptShopOpened(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-shop result");
        };
        Ok(RuntimeScriptShop {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn buy_shop_item(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
        quantity: u16,
    ) -> Result<RuntimeShopTransaction> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::BuyShopItem(RuntimeShopTransactionCommand {
                item_id: item_id.to_string(),
                quantity,
            }),
        )?;
        let RuntimeMutationResult::ShopItemBought(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-shop-purchase result");
        };
        Ok(RuntimeShopTransaction {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn sell_shop_item(
        &mut self,
        runtime: &CrystalRuntime,
        item_id: &str,
        quantity: u16,
    ) -> Result<RuntimeShopTransaction> {
        let mutation = self.apply_runtime_mutation_command(
            runtime,
            RuntimeMutationCommand::SellShopItem(RuntimeShopTransactionCommand {
                item_id: item_id.to_string(),
                quantity,
            }),
        )?;
        let RuntimeMutationResult::ShopItemSold(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-shop-sale result");
        };
        Ok(RuntimeShopTransaction {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }
}

impl RuntimeAudioCatalog {
    fn from_game_data_owned(
        data: &GameDataSet,
        compiled_audio: BTreeMap<String, Vec<u8>>,
        manifest: ModpackAudioManifest,
        playback: ModpackAudioPlaybackPlan,
        audio_compression: Option<&str>,
    ) -> Result<Self> {
        Self::from_game_data_inner(data, compiled_audio, manifest, playback, audio_compression)
    }

    fn from_game_data_inner(
        data: &GameDataSet,
        mut compiled_audio: BTreeMap<String, Vec<u8>>,
        manifest: ModpackAudioManifest,
        playback: ModpackAudioPlaybackPlan,
        audio_compression: Option<&str>,
    ) -> Result<Self> {
        let declared_audio = data.audio_ids();
        for audio_id in compiled_audio.keys() {
            if !declared_audio.contains(audio_id.as_str()) {
                anyhow::bail!(
                    "runtime embedded audio payload {} is not declared by compiled pack data",
                    audio_id
                );
            }
        }

        if audio_compression.is_none() && manifest != data.audio_manifest(&compiled_audio)? {
            anyhow::bail!(
                "runtime audio manifest does not match embedded definitive audio payloads"
            );
        }
        playback
            .validate_for_manifest(&manifest)
            .context("validate runtime audio playback plan")?;
        let mut catalog = Self {
            manifest: manifest.clone(),
            playback,
            music: BTreeMap::new(),
            sound_effects: BTreeMap::new(),
            sound_effect_priorities: BTreeMap::new(),
            cries: BTreeMap::new(),
        };

        for asset in &data.audio {
            asset.validate()?;
            let source = match (asset.source, compiled_audio.remove(&asset.id)) {
                (ModpackAudioSource::Pcm, Some(bytes))
                    if audio_compression == Some(PACK_AUDIO_COMPRESSION_GZIP) =>
                {
                    let format = asset.pcm_format.as_ref().with_context(|| {
                        format!(
                            "compiled PCM audio asset '{}' missing validated pcm_format",
                            asset.id
                        )
                    })?;
                    let entry = manifest
                        .music
                        .get(&asset.id)
                        .or_else(|| manifest.sound_effects.get(&asset.id))
                        .or_else(|| manifest.cries.get(&asset.id))
                        .with_context(|| format!("audio manifest missing {}", asset.id))?;
                    AudioProgramSource::PcmGzip {
                        bytes,
                        format: AudioPcmFormat {
                            sample_rate_hz: format.sample_rate_hz,
                            channels: format.channels,
                            bits_per_sample: format.bits_per_sample,
                        },
                        byte_len: entry.byte_len,
                        payload_hash: entry.payload_hash.clone(),
                        loop_start_sample: asset.loop_start_sample,
                        loop_end_sample: asset.loop_end_sample,
                    }
                }
                (ModpackAudioSource::Pcm, Some(bytes)) => {
                    validate_compiled_audio_payload(asset, &bytes)?;
                    let format = asset.pcm_format.as_ref().with_context(|| {
                        format!(
                            "compiled PCM audio asset '{}' missing validated pcm_format",
                            asset.id
                        )
                    })?;
                    AudioProgramSource::Pcm {
                        bytes,
                        format: AudioPcmFormat {
                            sample_rate_hz: format.sample_rate_hz,
                            channels: format.channels,
                            bits_per_sample: format.bits_per_sample,
                        },
                        loop_start_sample: asset.loop_start_sample,
                        loop_end_sample: asset.loop_end_sample,
                    }
                }
                (ModpackAudioSource::Midi, None)
                    if audio_compression == Some(PACK_AUDIO_COMPRESSION_MIDI) =>
                {
                    let format = asset.pcm_format.as_ref().with_context(|| {
                        format!(
                            "MIDI audio asset '{}' missing validated pcm_format",
                            asset.id
                        )
                    })?;
                    let midi_base64 = asset
                        .midi_program
                        .as_ref()
                        .with_context(|| {
                            format!("MIDI audio asset '{}' missing program", asset.id)
                        })?
                        .midi_base64
                        .clone();
                    let entry = manifest
                        .music
                        .get(&asset.id)
                        .or_else(|| manifest.sound_effects.get(&asset.id))
                        .or_else(|| manifest.cries.get(&asset.id))
                        .with_context(|| format!("audio manifest missing {}", asset.id))?;
                    AudioProgramSource::Midi {
                        midi_base64,
                        format: AudioPcmFormat {
                            sample_rate_hz: format.sample_rate_hz,
                            channels: format.channels,
                            bits_per_sample: format.bits_per_sample,
                        },
                        byte_len: entry.byte_len,
                        payload_hash: entry.payload_hash.clone(),
                        loop_start_sample: asset.loop_start_sample,
                        loop_end_sample: asset.loop_end_sample,
                    }
                }
                (ModpackAudioSource::Pcm, None) => anyhow::bail!(
                    "compiled game pack missing embedded PCM audio payload {}",
                    asset.id
                ),
                (ModpackAudioSource::Midi, _) => {
                    anyhow::bail!("MIDI audio asset {} requires a MIDI browser pack", asset.id)
                }
            };
            let program = AudioProgram {
                cache_key: format!("{}:{}:{}", asset.kind.runtime_name(), asset.id, asset.path),
                source,
            };
            let previous = match asset.kind {
                ModpackAudioKind::Music => catalog.music.insert(asset.id.clone(), program),
                ModpackAudioKind::SoundEffect => {
                    catalog.sound_effect_priorities.insert(
                        asset.id.clone(),
                        asset
                            .sfx_priority
                            .expect("validated sound effect has sfx_priority"),
                    );
                    catalog.sound_effects.insert(asset.id.clone(), program)
                }
                ModpackAudioKind::Cry => catalog.cries.insert(asset.id.clone(), program),
            };
            if previous.is_some() {
                anyhow::bail!(
                    "duplicate runtime {} audio id '{}'",
                    asset.kind.runtime_name(),
                    asset.id
                );
            }
        }
        if !compiled_audio.is_empty() {
            anyhow::bail!("runtime compiled audio contains undeclared payloads");
        }
        Ok(catalog)
    }

    pub fn from_game_data(
        data: &GameDataSet,
        compiled_audio: &BTreeMap<String, Vec<u8>>,
        manifest: ModpackAudioManifest,
        playback: ModpackAudioPlaybackPlan,
    ) -> Result<Self> {
        let declared_audio = data.audio_ids();
        for audio_id in compiled_audio.keys() {
            if !declared_audio.contains(audio_id.as_str()) {
                anyhow::bail!(
                    "runtime embedded audio payload {} is not declared by compiled pack data",
                    audio_id
                );
            }
        }

        if manifest != data.audio_manifest(compiled_audio)? {
            anyhow::bail!(
                "runtime audio manifest does not match embedded definitive audio payloads"
            );
        }
        playback
            .validate_for_manifest(&manifest)
            .context("validate runtime audio playback plan")?;
        let mut catalog = Self {
            manifest,
            playback,
            music: BTreeMap::new(),
            sound_effects: BTreeMap::new(),
            sound_effect_priorities: BTreeMap::new(),
            cries: BTreeMap::new(),
        };

        for asset in &data.audio {
            asset.validate()?;
            let bytes = compiled_audio
                .get(&asset.id)
                .with_context(|| {
                    format!(
                        "compiled game pack missing embedded audio payload {}",
                        asset.id
                    )
                })?
                .clone();
            validate_compiled_audio_payload(asset, &bytes)?;
            let source = match asset.source {
                ModpackAudioSource::Pcm => {
                    let format = asset.pcm_format.as_ref().with_context(|| {
                        format!(
                            "compiled PCM audio asset '{}' missing validated pcm_format",
                            asset.id
                        )
                    })?;
                    AudioProgramSource::Pcm {
                        bytes,
                        format: AudioPcmFormat {
                            sample_rate_hz: format.sample_rate_hz,
                            channels: format.channels,
                            bits_per_sample: format.bits_per_sample,
                        },
                        loop_start_sample: asset.loop_start_sample,
                        loop_end_sample: asset.loop_end_sample,
                    }
                }
                ModpackAudioSource::Midi => {
                    anyhow::bail!("embedded MIDI audio asset '{}' is unsupported", asset.id)
                }
            };
            let program = AudioProgram {
                cache_key: format!("{}:{}:{}", asset.kind.runtime_name(), asset.id, asset.path),
                source,
            };
            let previous = match asset.kind {
                ModpackAudioKind::Music => catalog.music.insert(asset.id.clone(), program),
                ModpackAudioKind::SoundEffect => {
                    catalog.sound_effect_priorities.insert(
                        asset.id.clone(),
                        asset
                            .sfx_priority
                            .expect("validated sound effect has sfx_priority"),
                    );
                    catalog.sound_effects.insert(asset.id.clone(), program)
                }
                ModpackAudioKind::Cry => catalog.cries.insert(asset.id.clone(), program),
            };
            if previous.is_some() {
                anyhow::bail!(
                    "duplicate runtime {} audio id '{}'",
                    asset.kind.runtime_name(),
                    asset.id
                );
            }
        }

        Ok(catalog)
    }

    pub fn program(&self, kind: AudioKind, id: &str) -> Option<&AudioProgram> {
        match kind {
            AudioKind::Music => self.music.get(id),
            AudioKind::SoundEffect => self.sound_effects.get(id),
            AudioKind::Cry => self.cries.get(id),
        }
    }

    pub fn playback_entry(&self, kind: AudioKind, id: &str) -> Option<&ModpackAudioPlaybackEntry> {
        match kind {
            AudioKind::Music => self.playback.music.get(id),
            AudioKind::SoundEffect => self.playback.sound_effects.get(id),
            AudioKind::Cry => self.playback.cries.get(id),
        }
    }

    pub fn require_program(&self, kind: AudioKind, id: &str) -> Result<&AudioProgram> {
        let kind_name = match kind {
            AudioKind::Music => "music",
            AudioKind::SoundEffect => "sound_effect",
            AudioKind::Cry => "cry",
        };
        self.program(kind, id)
            .with_context(|| format!("runtime audio catalog missing {kind_name} id {id}"))
    }

    pub fn require_playback_entry(
        &self,
        kind: AudioKind,
        id: &str,
    ) -> Result<&ModpackAudioPlaybackEntry> {
        let kind_name = match kind {
            AudioKind::Music => "music",
            AudioKind::SoundEffect => "sound_effect",
            AudioKind::Cry => "cry",
        };
        self.playback_entry(kind, id)
            .with_context(|| format!("runtime audio playback plan missing {kind_name} id {id}"))
    }

    pub fn resolve_audio_event(
        &self,
        event: crystal_core::state::ScriptAudioRuntimeEvent,
    ) -> Result<RuntimeResolvedAudioPlayback> {
        let kind = match event.kind {
            crystal_core::state::ScriptAudioRuntimeKind::Music => {
                let audio_id = required_audio_event_id(&event)?;
                if audio_id == crystal_core::systems::script_audio::MUSIC_NONE_ID {
                    return Ok(RuntimeResolvedAudioPlayback {
                        event,
                        kind: RuntimeResolvedAudioPlaybackKind::StopMusic { audio_id },
                    });
                }
                self.require_music(&audio_id)?;
                RuntimeResolvedAudioPlaybackKind::Play {
                    playback: self
                        .require_playback_entry(AudioKind::Music, &audio_id)?
                        .clone(),
                    audio_id,
                }
            }
            crystal_core::state::ScriptAudioRuntimeKind::SoundEffect => {
                let audio_id = required_audio_event_id(&event)?;
                self.require_sound_effect(&audio_id)?;
                RuntimeResolvedAudioPlaybackKind::Play {
                    playback: self
                        .require_playback_entry(AudioKind::SoundEffect, &audio_id)?
                        .clone(),
                    audio_id,
                }
            }
            crystal_core::state::ScriptAudioRuntimeKind::Cry => {
                let audio_id = required_audio_event_id(&event)?;
                self.require_cry(&audio_id)?;
                RuntimeResolvedAudioPlaybackKind::Play {
                    playback: self
                        .require_playback_entry(AudioKind::Cry, &audio_id)?
                        .clone(),
                    audio_id,
                }
            }
            crystal_core::state::ScriptAudioRuntimeKind::FadeMusic => {
                let audio_id = required_audio_event_id(&event)?;
                let fade_frames = event.fade_frames.with_context(|| {
                    format!(
                        "runtime audio fade event {}:{} is missing fade_frames",
                        event.source_script, event.command_index
                    )
                })?;
                if audio_id != crystal_core::systems::script_audio::MUSIC_NONE_ID {
                    self.require_music(&audio_id)?;
                }
                RuntimeResolvedAudioPlaybackKind::FadeMusic {
                    audio_id,
                    fade_frames,
                }
            }
            crystal_core::state::ScriptAudioRuntimeKind::WaitForSoundEffect => {
                if event.audio_id.is_some() || event.fade_frames.is_some() {
                    anyhow::bail!(
                        "runtime audio wait event {}:{} must not carry audio_id or fade_frames",
                        event.source_script,
                        event.command_index
                    );
                }
                RuntimeResolvedAudioPlaybackKind::WaitForSoundEffect
            }
        };
        Ok(RuntimeResolvedAudioPlayback { event, kind })
    }

    pub fn resolve_audio_events(
        &self,
        events: impl IntoIterator<Item = crystal_core::state::ScriptAudioRuntimeEvent>,
    ) -> Result<Vec<RuntimeResolvedAudioPlayback>> {
        events
            .into_iter()
            .map(|event| self.resolve_audio_event(event))
            .collect()
    }

    pub fn require_music(&self, id: &str) -> Result<&AudioProgram> {
        self.require_program(AudioKind::Music, id)
    }

    pub fn require_sound_effect(&self, id: &str) -> Result<&AudioProgram> {
        self.require_program(AudioKind::SoundEffect, id)
    }

    pub fn require_sound_effect_priority(&self, id: &str) -> Result<u8> {
        self.sound_effect_priorities
            .get(id)
            .copied()
            .with_context(|| format!("runtime audio catalog missing SFX priority for {id}"))
    }

    pub fn require_cry(&self, id: &str) -> Result<&AudioProgram> {
        self.require_program(AudioKind::Cry, id)
    }

    pub fn audio_asset_keys(&self) -> BTreeSet<RuntimeAudioAssetKey> {
        let mut keys = BTreeSet::new();
        keys.extend(self.manifest.music.iter().filter_map(|(id, entry)| {
            self.music
                .get(id)
                .map(|program| runtime_audio_asset_key(entry, program))
        }));
        keys.extend(
            self.manifest
                .sound_effects
                .iter()
                .filter_map(|(id, entry)| {
                    self.sound_effects
                        .get(id)
                        .map(|program| runtime_audio_asset_key(entry, program))
                }),
        );
        keys.extend(self.manifest.cries.iter().filter_map(|(id, entry)| {
            self.cries
                .get(id)
                .map(|program| runtime_audio_asset_key(entry, program))
        }));
        keys
    }

    pub fn has_audio_asset(&self, key: &RuntimeAudioAssetKey) -> bool {
        self.audio_asset_keys().contains(key)
    }

    pub fn music_ids(&self) -> BTreeSet<String> {
        self.music.keys().cloned().collect()
    }

    pub fn sound_effect_ids(&self) -> BTreeSet<String> {
        self.sound_effects.keys().cloned().collect()
    }

    pub fn cry_ids(&self) -> BTreeSet<String> {
        self.cries.keys().cloned().collect()
    }
}

fn required_audio_event_id(event: &crystal_core::state::ScriptAudioRuntimeEvent) -> Result<String> {
    event.audio_id.clone().with_context(|| {
        format!(
            "runtime audio event {}:{} command {} is missing audio_id",
            event.source_script, event.command_index, event.command
        )
    })
}

fn runtime_audio_asset_key(
    entry: &ModpackAudioManifestEntry,
    program: &AudioProgram,
) -> RuntimeAudioAssetKey {
    let (pcm_sample_rate_hz, pcm_channels, pcm_bits_per_sample) = entry
        .pcm_format
        .as_ref()
        .map(|format| {
            (
                Some(format.sample_rate_hz),
                Some(format.channels),
                Some(format.bits_per_sample),
            )
        })
        .unwrap_or((None, None, None));
    let source = "pcm";
    RuntimeAudioAssetKey {
        audio_id: entry.id.clone(),
        kind: entry.kind.runtime_name().to_string(),
        source: source.to_string(),
        path: entry.path.clone(),
        byte_len: entry.byte_len,
        payload_hash: entry.payload_hash.clone(),
        pcm_sample_rate_hz,
        pcm_channels,
        pcm_bits_per_sample,
        pcm_frame_count: entry.pcm_frame_count,
        cache_key: program.cache_key.clone(),
    }
}

fn validate_save_references_for_runtime_pack(state: &GameState, data: &GameDataSet) -> Result<()> {
    data.validate_saved_bag_references(&state.bag)?;
    data.validate_saved_mom_purchase_references(state)?;
    data.validate_saved_pokedex_references(&state.pokedex)?;
    data.validate_saved_storage_references(&state.storage)?;
    data.validate_saved_bug_contest_references(&state.bug_contest)?;
    data.validate_saved_day_care_references(&state.day_care)?;
    data.validate_saved_roaming_references(&state.roaming_pokemon, &state.roaming_map_history)?;
    data.validate_saved_mystery_gift_references(&state.mystery_gift)?;
    data.validate_saved_magikarp_record_references(&state.magikarp_record)?;
    data.validate_saved_blue_card_references(state)?;
    data.validate_saved_buena_password_references(&state.buenas_password)?;
    data.validate_saved_battle_tower_references(&state.battle_tower, &state.storage.party)?;
    data.validate_saved_fishing_references(&state.fishing)?;
    data.validate_saved_swarm_references(&state.swarms)?;
    data.validate_saved_pending_special_battle_type(state.pending_special_battle_type.as_deref())?;
    data.validate_saved_pending_field_moves(state)?;
    validate_saved_script_runtime_references(data, state)?;
    data.validate_saved_overworld_references(&state.overworld)?;
    validate_saved_overworld_walkable_for_runtime_pack(&state.overworld, data)?;
    data.validate_saved_scene_references(&state.scenes)?;
    data.validate_saved_flag_references(&state.flags)?;
    if let Some(item_id) = &state.active_repel_item {
        data.validate_saved_active_repel_item(item_id, state.repel_steps_remaining)?;
    }
    if let Some(item_id) = &state.registered_key_item {
        data.validate_saved_item_reference("registered_key_item", item_id)?;
    }
    for pending in state
        .pending_move_learn
        .iter()
        .chain(state.pending_move_learn_queue.iter())
    {
        data.validate_saved_species_reference(
            "pending_move_learn.species_id",
            &pending.species_id,
        )?;
        data.validate_saved_move_reference(
            "pending_move_learn.learned_move.name",
            &pending.learned_move.name,
        )?;
        let pokemon = state
            .storage
            .party
            .pokemon
            .get(pending.party_index)
            .with_context(|| {
                format!(
                    "pending_move_learn.party_index {} is outside saved party range",
                    pending.party_index
                )
            })?
            .as_ref()
            .with_context(|| {
                format!(
                    "pending_move_learn.party_index {} references an empty saved party slot",
                    pending.party_index
                )
            })?;
        if pokemon.species.id != pending.species_id {
            anyhow::bail!(
                "pending_move_learn.species_id {} does not match saved storage.party[{}].species {}",
                pending.species_id,
                pending.party_index,
                pokemon.species.id
            );
        }
        if pokemon.level != pending.level {
            anyhow::bail!(
                "pending_move_learn.level {} does not match saved storage.party[{}].level {}",
                pending.level,
                pending.party_index,
                pokemon.level
            );
        }
        if pokemon
            .moves
            .iter()
            .any(|known| known.name == pending.learned_move.name)
        {
            anyhow::bail!(
                "pending_move_learn.learned_move.name {} is already known by saved storage.party[{}]",
                pending.learned_move.name,
                pending.party_index
            );
        }
    }
    if let Some(map_constant) = &state.last_spawn_map_constant {
        data.validate_saved_map_constant_reference("last_spawn_map_constant", map_constant)?;
    }
    if let Some(map_name) = &state.dig_warp_map_name {
        let _ = data.validate_saved_map_reference("dig_warp_map_name", map_name)?;
        if let Some(warp_index) = state.dig_warp_index {
            data.validate_saved_warp_reference("dig_warp_index", map_name, warp_index)?;
            validate_saved_dig_warp_destination(data, state)?;
        }
    }
    for (map_name, overrides) in &state.map_block_overrides {
        data.validate_saved_block_overrides(map_name, overrides)?;
    }
    for (map_name, memory) in &state.map_object_overrides {
        data.validate_saved_object_overrides(map_name, memory)?;
    }
    if let Some(terminal) = &state.pending_static_wild_terminal {
        data.validate_saved_static_wild_battle_origin_references(
            &terminal.battle_type,
            &terminal.species,
            terminal.level,
            &terminal.origin_map_name,
            &terminal.source_script,
            terminal.startbattle_command_index,
            terminal.resume_command_index,
        )?;
    }
    validate_saved_battle_references(data, state)
}

fn validate_saved_dig_warp_destination(data: &GameDataSet, state: &GameState) -> Result<()> {
    let destination = data
        .saved_dig_warp_destination(state, "saved dig_warp")
        .context("saved dig_warp destination is invalid")?;
    data.overworld_session(&destination.map_name, destination.tile, 0)
        .with_context(|| {
            format!(
                "saved dig_warp destination {} warp {} runtime tile ({}, {}) is invalid",
                destination.map_name,
                destination.warp_index,
                destination.tile.x,
                destination.tile.y
            )
        })?;
    Ok(())
}

fn validate_saved_overworld_walkable_for_runtime_pack(
    overworld: &OverworldMemory,
    data: &GameDataSet,
) -> Result<()> {
    let OverworldMemory::Active {
        map_name,
        tile,
        mode,
        ..
    } = overworld
    else {
        return Ok(());
    };
    data.overworld_session_for_traversal(map_name, *tile, 0, mode.traversal_state())
        .with_context(|| {
            format!(
                "saved overworld.active tile ({}, {}) is invalid on compiled map {map_name} for {:?}",
                tile.x, tile.y, mode
            )
        })?;
    Ok(())
}

fn validate_saved_script_runtime_references(data: &GameDataSet, state: &GameState) -> Result<()> {
    let runtime = &state.script_runtime;
    if let Some(script) = &runtime.next_script {
        if !data.maps.contains_key(&script.origin_map_name) {
            anyhow::bail!(
                "script_runtime.next_script has unknown origin map {}",
                script.origin_map_name
            );
        }
        data.validate_saved_script_label_reference(
            "script_runtime.next_script.script",
            &script.script,
        )?;
    }
    for (index, script) in runtime.deferred_scripts.iter().enumerate() {
        if !data.maps.contains_key(&script.origin_map_name) {
            anyhow::bail!(
                "script_runtime.deferred_scripts[{index}] has unknown origin map {}",
                script.origin_map_name
            );
        }
        data.validate_saved_script_label_reference(
            &format!("script_runtime.deferred_scripts[{index}].script"),
            &script.script,
        )?;
    }
    if let Some(script) = &runtime.map_reentry_script {
        if !data.maps.contains_key(&script.origin_map_name) {
            anyhow::bail!(
                "script_runtime.map_reentry_script has unknown origin map {}",
                script.origin_map_name
            );
        }
        data.validate_saved_script_label_reference(
            "script_runtime.map_reentry_script.script",
            &script.script,
        )?;
    }
    for (index, frame) in runtime.call_stack.iter().enumerate() {
        if !data.maps.contains_key(&frame.origin_map_name) {
            anyhow::bail!(
                "script_runtime.call_stack[{index}] has unknown origin map {}",
                frame.origin_map_name
            );
        }
        data.validate_saved_script_return_reference(
            &format!("script_runtime.call_stack[{index}].source_script"),
            &frame.source_script,
            frame.next_command_index,
        )?;
    }
    if let Some(end) = &runtime.script_ended {
        data.validate_saved_script_end_command(end)?;
    }
    if let Some(menu) = &runtime.active_menu {
        data.validate_saved_menu_reference("script_runtime.active_menu", menu)?;
    }
    if let Some(species) = &runtime.active_pokemon_picture {
        data.validate_saved_species_reference("script_runtime.active_pokemon_picture", species)?;
    }
    if let Some(object_id) = &runtime.last_talked_object {
        data.validate_saved_last_talked_object_reference(state, object_id)?;
    }
    for (sprite, replacement) in &runtime.variable_sprites {
        data.validate_saved_variable_sprite_reference(
            "script_runtime.variable_sprites key",
            sprite,
        )?;
        data.validate_saved_sprite_reference(
            &format!("script_runtime.variable_sprites[{sprite}]"),
            replacement,
        )?;
    }
    for contact_id in &runtime.phone_numbers {
        data.validate_saved_phone_contact_reference("script_runtime.phone_numbers", contact_id)?;
    }
    if let Some(call_id) = &runtime.special_phone_call {
        data.validate_saved_special_phone_call_reference(
            "script_runtime.special_phone_call",
            call_id,
        )?;
    }
    for (index, trade_id) in runtime.completed_trades.iter().enumerate() {
        data.validate_saved_npc_trade_reference(
            &format!("script_runtime.completed_trades[{index}]"),
            trade_id,
        )?;
    }
    for (index, entry) in runtime.stone_table_entries.iter().enumerate() {
        data.validate_saved_script_label_reference(
            &format!("script_runtime.stone_table_entries[{index}].script"),
            &entry.script,
        )?;
        data.validate_saved_stone_table_entry_command(
            &format!("script_runtime.stone_table_entries[{index}].source_script"),
            entry,
        )?;
    }
    for (index, delay) in runtime.pending_delays.iter().enumerate() {
        let (command, args) = saved_delay_command_payload(delay);
        data.validate_saved_script_command_payload_reference(
            &format!("script_runtime.pending_delays[{index}].source_script"),
            &delay.source_script,
            delay.command_index,
            command,
            &args,
        )?;
    }
    for (index, earthquake) in runtime.pending_earthquakes.iter().enumerate() {
        let (command, args) = saved_earthquake_command_payload(earthquake);
        data.validate_saved_script_command_payload_reference(
            &format!("script_runtime.pending_earthquakes[{index}].source_script"),
            &earthquake.source_script,
            earthquake.command_index,
            command,
            &args,
        )?;
    }
    for (index, emote) in runtime.pending_emotes.iter().enumerate() {
        let (command, args) = saved_emote_command_payload(emote);
        data.validate_saved_script_command_payload_reference(
            &format!("script_runtime.pending_emotes[{index}].source_script"),
            &emote.source_script,
            emote.command_index,
            command,
            &args,
        )?;
    }
    for (index, command) in runtime.command_queue.iter().enumerate() {
        if !data.maps.contains_key(&command.origin_map_name) {
            anyhow::bail!(
                "script_runtime.command_queue[{index}] has unknown origin map {}",
                command.origin_map_name
            );
        }
        data.validate_saved_script_label_reference(
            &format!("script_runtime.command_queue[{index}].target"),
            &command.target,
        )?;
        data.validate_saved_script_command_payload_reference(
            &format!("script_runtime.command_queue[{index}].source_script"),
            &command.source_script,
            command.command_index,
            &command.command,
            &saved_queued_command_args(command),
        )?;
    }
    if let Some(music) = &runtime.current_music {
        data.validate_saved_audio_reference(
            "script_runtime.current_music",
            music,
            ModpackAudioKind::Music,
        )?;
    }
    if let Some(fade) = &runtime.pending_music_fade {
        if fade.audio_id != crystal_core::systems::script_audio::MUSIC_NONE_ID {
            data.validate_saved_audio_reference(
                "script_runtime.pending_music_fade.audio_id",
                &fade.audio_id,
                ModpackAudioKind::Music,
            )?;
        }
        if fade.source_script == "FadeOutMusic" {
            // This fade is authored by a built-in special, not a map-script
            // musicfadeout opcode. Validate its exact emitted shape and catalog.
            anyhow::ensure!(
                fade.command_index == 0 && fade.audio_id == "MUSIC_NONE" && fade.fade_frames == 2,
                "saved FadeOutMusic request does not match the built-in fade"
            );
            data.validate_saved_special_routine_reference(
                "script_runtime.pending_music_fade.source_script",
                &fade.source_script,
            )?;
        } else {
            let (command, args) = saved_music_fade_command_payload(fade);
            data.validate_saved_script_command_payload_reference(
                "script_runtime.pending_music_fade.source_script",
                &fade.source_script,
                fade.command_index,
                command,
                &args,
            )?;
        }
    }
    for (index, event) in runtime.audio_events.iter().enumerate() {
        if let Some(audio_id) = &event.audio_id {
            let expected_kind = match event.kind {
                crystal_core::state::ScriptAudioRuntimeKind::Music
                | crystal_core::state::ScriptAudioRuntimeKind::FadeMusic => ModpackAudioKind::Music,
                crystal_core::state::ScriptAudioRuntimeKind::SoundEffect => {
                    ModpackAudioKind::SoundEffect
                }
                crystal_core::state::ScriptAudioRuntimeKind::Cry => ModpackAudioKind::Cry,
                crystal_core::state::ScriptAudioRuntimeKind::WaitForSoundEffect => {
                    ModpackAudioKind::SoundEffect
                }
            };
            let is_music_none = matches!(
                event.kind,
                crystal_core::state::ScriptAudioRuntimeKind::Music
                    | crystal_core::state::ScriptAudioRuntimeKind::FadeMusic
            ) && audio_id == crystal_core::systems::script_audio::MUSIC_NONE_ID;
            if event.kind != crystal_core::state::ScriptAudioRuntimeKind::WaitForSoundEffect
                && !is_music_none
            {
                data.validate_saved_audio_reference(
                    &format!("script_runtime.audio_events[{index}].audio_id"),
                    audio_id,
                    expected_kind,
                )?;
            }
        }
        if data
            .compiled_standard_script_body(&event.source_script)
            .is_err()
        {
            data.validate_saved_audio_runtime_event_command(
                &format!("script_runtime.audio_events[{index}].source_script"),
                event,
            )?;
        }
    }
    for (index, event) in runtime.graphics_events.iter().enumerate() {
        data.validate_saved_graphics_runtime_event(
            &format!("script_runtime.graphics_events[{index}].source_script"),
            event,
        )?;
    }
    if let Some(fade) = &runtime.pending_screen_fade {
        data.validate_saved_screen_fade("script_runtime.pending_screen_fade.source_script", fade)?;
    }
    for (index, event) in runtime.money_events.iter().enumerate() {
        data.validate_saved_money_runtime_event(
            &format!("script_runtime.money_events[{index}].source_script"),
            event,
        )?;
    }
    for (index, event) in runtime.map_events.iter().enumerate() {
        if let Some(map_name) = &event.target_map {
            let _ = data.validate_saved_map_reference(
                &format!("script_runtime.map_events[{index}].target_map"),
                map_name,
            )?;
        }
        validate_saved_map_runtime_event_destination(data, index, event)?;
        data.validate_saved_map_runtime_event_command(
            &format!("script_runtime.map_events[{index}].source_script"),
            event,
        )?;
    }
    if let Some(warp) = &runtime.pending_script_warp {
        let _ = data.validate_saved_map_reference(
            "script_runtime.pending_script_warp.target_map",
            &warp.target_map,
        )?;
        validate_saved_pending_script_warp_destination(data, warp)?;
        if data.saved_special_routine_exists(&warp.source_script) {
            data.validate_saved_special_routine_reference(
                "script_runtime.pending_script_warp.source_script",
                &warp.source_script,
            )?;
        } else if !data.saved_elevator_pending_warp_exists(warp) {
            let path = "script_runtime.pending_script_warp.source_script";
            if data.saved_script_command_is(
                path,
                &warp.source_script,
                warp.command_index,
                "warpcheck",
            )? {
                data.validate_saved_warpcheck_pending_warp_reference(state, path, warp)?;
            } else {
                data.validate_saved_script_warp_reference(path, warp)?;
            }
        }
    }
    if let Some(load) = &runtime.pending_map_load {
        let (command, args) = saved_map_load_command_payload(load);
        data.validate_saved_script_command_payload_reference(
            "script_runtime.pending_map_load.source_script",
            &load.source_script,
            load.command_index,
            command,
            &args,
        )?;
    }
    if let Some(refresh) = &runtime.pending_map_refresh {
        let (command, args) = saved_map_refresh_command_payload(refresh);
        data.validate_saved_script_command_payload_reference(
            "script_runtime.pending_map_refresh.source_script",
            &refresh.source_script,
            refresh.command_index,
            command,
            &args,
        )?;
    }
    if let Some(text_label) = &runtime.active_text_label {
        data.validate_saved_text_reference("script_runtime.active_text_label", text_label)?;
    }
    if let Some(text_label) = &runtime.pending_text_label {
        data.validate_saved_text_reference("script_runtime.pending_text_label", text_label)?;
    }
    for (index, event) in runtime.text_events.iter().enumerate() {
        if let Some(text_label) = &event.text_label {
            data.validate_saved_text_reference(
                &format!("script_runtime.text_events[{index}].text_label"),
                text_label,
            )?;
        }
        if data
            .compiled_standard_script_body(&event.source_script)
            .is_err()
        {
            data.validate_saved_text_runtime_event_command(
                &format!("script_runtime.text_events[{index}].source_script"),
                event,
            )?;
        }
    }
    if let Some(wait) = &runtime.pending_text_wait {
        if data
            .compiled_standard_script_body(&wait.source_script)
            .is_err()
        {
            data.validate_saved_pending_text_wait_command(runtime, wait)?;
        }
    }
    if let Some(prompt) = &runtime.pending_yes_no {
        if data
            .compiled_standard_script_body(&prompt.source_script)
            .is_err()
        {
            data.validate_saved_script_command_payload_reference(
                "script_runtime.pending_yes_no.source_script",
                &prompt.source_script,
                prompt.command_index,
                "yesorno",
                &[],
            )?;
        }
    }
    for (index, event) in runtime.control_events.iter().enumerate() {
        data.validate_saved_control_runtime_event_command(
            &format!("script_runtime.control_events[{index}].source_script"),
            event,
        )?;
        if let Some(target_script) = &event.target_script {
            data.validate_saved_script_label_reference(
                &format!("script_runtime.control_events[{index}].target_script"),
                target_script,
            )?;
        }
    }
    for (index, event) in runtime.shop_events.iter().enumerate() {
        let (command, args) = saved_shop_event_command_payload(event);
        data.validate_saved_script_command_payload_reference(
            &format!("script_runtime.shop_events[{index}].source_script"),
            &event.source_script,
            event.command_index,
            command,
            &args,
        )?;
        for (item_index, item_id) in event.inventory.iter().enumerate() {
            data.validate_saved_item_reference(
                &format!("script_runtime.shop_events[{index}].inventory[{item_index}]"),
                item_id,
            )?;
        }
    }
    if let Some(shop) = &runtime.pending_shop {
        let (command, args) = saved_shop_request_command_payload(shop);
        data.validate_saved_script_command_payload_reference(
            "script_runtime.pending_shop.source_script",
            &shop.source_script,
            shop.command_index,
            command,
            &args,
        )?;
        for (item_index, item_id) in shop.inventory.iter().enumerate() {
            data.validate_saved_item_reference(
                &format!("script_runtime.pending_shop.inventory[{item_index}]"),
                item_id,
            )?;
        }
    }
    for (index, event) in runtime.item_use_events.iter().enumerate() {
        data.validate_saved_item_reference(
            &format!("script_runtime.item_use_events[{index}].item_id"),
            &event.item_id,
        )?;
    }
    Ok(())
}

fn validate_saved_map_runtime_event_destination(
    data: &GameDataSet,
    index: usize,
    event: &ScriptMapRuntimeEvent,
) -> Result<()> {
    if event.kind != crystal_core::state::ScriptMapRuntimeKind::Warp {
        return Ok(());
    }
    let Some(target_map) = event.target_map.as_deref() else {
        return Ok(());
    };
    let Some(tile) = event.tile else {
        return Ok(());
    };
    data.overworld_session(target_map, tile, 0)
        .with_context(|| {
            format!(
                "saved script_runtime.map_events[{index}] destination {target_map} runtime tile ({}, {}) is invalid",
                tile.x, tile.y
            )
        })?;
    Ok(())
}

fn validate_saved_pending_script_warp_destination(
    data: &GameDataSet,
    warp: &ScriptWarpRequest,
) -> Result<()> {
    data.overworld_session(&warp.target_map, warp.tile, 0)
        .with_context(|| {
            format!(
                "saved script_runtime.pending_script_warp destination {} runtime tile ({}, {}) is invalid",
                warp.target_map, warp.tile.x, warp.tile.y
            )
        })?;
    Ok(())
}

fn validate_saved_battle_references(data: &GameDataSet, state: &GameState) -> Result<()> {
    match &state.battle {
        BattleMemory::Inactive => Ok(()),
        BattleMemory::Wild {
            battle_type,
            map_name,
            roaming_slot,
            enemy_pokemon,
            enemy_party,
            ..
        } => {
            let _ = data.validate_saved_map_reference("battle.wild.map_name", map_name)?;
            data.validate_saved_pokemon_reference("battle.wild.enemy_pokemon", enemy_pokemon)?;
            data.validate_saved_pokemon_party_references("battle.wild.enemy_party", enemy_party)?;
            if battle_type == "BATTLETYPE_ROAMING" {
                let slot = roaming_slot.context("saved roaming battle is missing roaming_slot")?;
                let roaming = state
                    .roaming_pokemon
                    .get(usize::from(slot))
                    .with_context(|| format!("saved roaming battle slot {slot} is invalid"))?;
                data.validate_saved_roaming_battle_origin_references(
                    map_name,
                    slot,
                    roaming,
                    enemy_pokemon,
                )
            } else {
                data.validate_saved_wild_battle_origin_references(
                    battle_type,
                    map_name,
                    enemy_pokemon,
                )
            }
        }
        BattleMemory::StaticWild {
            battle_type,
            roaming_slot,
            origin_map_name,
            species,
            level,
            source_script,
            startbattle_command_index,
            resume_command_index,
            enemy_pokemon,
            enemy_party,
            ..
        } => {
            data.validate_saved_species_reference("battle.static_wild.species", species)?;
            data.validate_saved_script_label_reference(
                "battle.static_wild.source_script",
                source_script,
            )?;
            data.validate_saved_static_wild_battle_origin_references(
                battle_type,
                species,
                *level,
                origin_map_name,
                source_script,
                *startbattle_command_index,
                *resume_command_index,
            )?;
            data.validate_saved_pokemon_reference(
                "battle.static_wild.enemy_pokemon",
                enemy_pokemon,
            )?;
            data.validate_saved_pokemon_party_references(
                "battle.static_wild.enemy_party",
                enemy_party,
            )?;
            if battle_type == "BATTLETYPE_ROAMING" {
                let slot = roaming_slot
                    .context("saved scripted roaming battle is missing roaming_slot")?;
                let roaming = state
                    .roaming_pokemon
                    .get(usize::from(slot))
                    .with_context(|| {
                        format!("saved scripted roaming battle slot {slot} is invalid")
                    })?;
                data.validate_saved_roaming_battle_origin_references(
                    origin_map_name,
                    slot,
                    roaming,
                    enemy_pokemon,
                )
            } else {
                Ok(())
            }
        }
        BattleMemory::Trainer {
            battle_type,
            trainer_id,
            trainer_class,
            trainer_name,
            event_flag,
            seen_text,
            win_text,
            loss_text,
            callback,
            source_script,
            encounter_music,
            ai_move_flags,
            ai_item_switch_flags,
            ai_layers,
            reward,
            enemy_pokemon,
            enemy_party,
            ..
        } => {
            if battle_type == "BATTLETYPE_LINK" {
                anyhow::ensure!(
                    state.link_session.link_mode == 3,
                    "saved link battle is not attached to an active Colosseum session"
                );
                data.validate_saved_audio_reference(
                    "battle.trainer.encounter_music",
                    encounter_music,
                    ModpackAudioKind::Music,
                )?;
                data.validate_saved_pokemon_reference(
                    "battle.trainer.enemy_pokemon",
                    enemy_pokemon,
                )?;
                data.validate_saved_pokemon_party_references(
                    "battle.trainer.enemy_party",
                    enemy_party,
                )?;
                anyhow::ensure!(*reward == 0, "saved link battle cannot award prize money");
                anyhow::ensure!(
                    *ai_move_flags == 0 && *ai_item_switch_flags == 0 && ai_layers.is_empty(),
                    "saved link battle cannot carry trainer AI"
                );
                return Ok(());
            }
            let canonical_battle_tower_trainer =
                data.battle_tower_rules.as_ref().is_some_and(|rules| {
                    trainer_id
                        .strip_prefix("BATTLE_TOWER_")
                        .and_then(|index| index.parse::<usize>().ok())
                        .is_some_and(|index| {
                            rules.trainers.iter().any(|trainer| trainer.index == index)
                        })
                });
            if canonical_battle_tower_trainer {
                data.validate_saved_audio_reference(
                    "battle.trainer.encounter_music",
                    encounter_music,
                    ModpackAudioKind::Music,
                )?;
                data.validate_saved_pokemon_reference(
                    "battle.trainer.enemy_pokemon",
                    enemy_pokemon,
                )?;
                data.validate_saved_pokemon_party_references(
                    "battle.trainer.enemy_party",
                    enemy_party,
                )?;
                if enemy_party.first() != Some(enemy_pokemon) {
                    anyhow::bail!(
                        "saved Battle Tower enemy party head does not match enemy_pokemon"
                    );
                }
                return Ok(());
            }
            let trainer =
                data.validate_saved_trainer_reference("battle.trainer.trainer_id", trainer_id)?;
            validate_saved_trainer_metadata(
                trainer,
                SavedTrainerMetadata {
                    trainer_class,
                    trainer_name,
                    ai_move_flags: *ai_move_flags,
                    ai_item_switch_flags: *ai_item_switch_flags,
                    ai_layers,
                    reward: *reward,
                    encounter_music,
                },
            )
            .map_err(|error| anyhow::anyhow!("{error}"))?;
            data.validate_saved_trainer_battle_origin_references(
                trainer,
                battle_type,
                trainer_class,
                event_flag,
                seen_text,
                win_text,
                loss_text,
                callback,
                source_script,
            )?;
            data.validate_saved_audio_reference(
                "battle.trainer.encounter_music",
                encounter_music,
                ModpackAudioKind::Music,
            )?;
            data.validate_saved_pokemon_reference("battle.trainer.enemy_pokemon", enemy_pokemon)?;
            data.validate_saved_pokemon_party_references(
                "battle.trainer.enemy_party",
                enemy_party,
            )?;
            data.validate_saved_trainer_enemy_party(trainer, enemy_party, enemy_pokemon)
        }
    }
}

