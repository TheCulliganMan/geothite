impl RuntimeGameShell {
    #[cfg(any(test, feature = "test-fixtures"))]
    pub fn apply_special_routine(&mut self, routine: &str) -> Result<RuntimeSpecialRoutineUse> {
        self.runtime.require_special_routine(routine)?;
        let mutation = self.apply_special_routine_runtime_mutation(routine)?;
        let RuntimeMutationResult::SpecialRoutineApplied(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-special-routine result");
        };
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_day_care(
        &mut self,
        caretaker: RuntimeDayCareCaretaker,
        action: RuntimeDayCareAction,
        party_index: Option<usize>,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let recorded =
            self.session
                .stage_day_care(&self.runtime, caretaker, action, party_index)?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::DayCareUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-day-care result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Day Care use")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn check_day_care_man_outside_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let recorded = self.session.stage_day_care_man_outside(&self.runtime)?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::DayCareManOutsideChecked(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-day-care-man-outside result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Day Care man outside check")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn check_day_care_resident_special(
        &mut self,
        caretaker: RuntimeDayCareCaretaker,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::CheckDayCareResidentSpecial(caretaker),
        )?;
        let RuntimeMutationResult::DayCareResidentChecked(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-day-care-resident result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Day Care resident check")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bug_contest(
        &mut self,
        action: RuntimeBugContestAction,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = match action {
            RuntimeBugContestAction::SelectContestants | RuntimeBugContestAction::Judge => {
                let recorded = self
                    .session
                    .stage_random_bug_contest(&self.runtime, action)?;
                self.apply_recorded_runtime_mutation(recorded)?
            }
            RuntimeBugContestAction::GiveParkBalls
            | RuntimeBugContestAction::DropOffMons
            | RuntimeBugContestAction::ReturnMons
            | RuntimeBugContestAction::CheckPartyFull => {
                let command = match action {
                    RuntimeBugContestAction::GiveParkBalls => {
                        RuntimeBugContestCommand::GiveParkBalls {}
                    }
                    RuntimeBugContestAction::DropOffMons => {
                        RuntimeBugContestCommand::DropOffMons {}
                    }
                    RuntimeBugContestAction::ReturnMons => RuntimeBugContestCommand::ReturnMons {},
                    RuntimeBugContestAction::CheckPartyFull => {
                        RuntimeBugContestCommand::CheckPartyFull {}
                    }
                    RuntimeBugContestAction::SelectContestants | RuntimeBugContestAction::Judge => {
                        unreachable!("random Bug Contest actions are staged above")
                    }
                };
                self.apply_runtime_mutation_command(RuntimeMutationCommand::UseBugContest(command))?
            }
        };
        let RuntimeMutationResult::BugContestUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-bug-contest result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Bug Contest use")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_kurt_apricorn(
        &mut self,
        apricorn_id: String,
        quantity: u16,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::UseKurtApricorn(RuntimeKurtApricornCommand {
                apricorn_id,
                quantity,
            }),
        )?;
        let RuntimeMutationResult::KurtApricornUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Kurt-apricorn result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Kurt apricorn use")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_buena_password(
        &mut self,
        guess: Option<String>,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let recorded = self.session.stage_buena_password(&self.runtime, guess)?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::BuenaPasswordUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Buena-password result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Buena password use")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_buena_prize(
        &mut self,
        item_id: String,
        quantity: u16,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::UseBuenaPrize(RuntimeBuenaPrizeCommand { item_id, quantity }),
        )?;
        let RuntimeMutationResult::BuenaPrizeUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Buena-prize result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Buena prize use")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_shuckie(
        &mut self,
        action: RuntimeShuckieAction,
        party_index: Option<usize>,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = if matches!(action, RuntimeShuckieAction::Give) {
            if party_index.is_some() {
                anyhow::bail!("Shuckie give must not select a party index");
            }
            let recorded = self.session.stage_shuckie_give(&self.runtime)?;
            self.apply_recorded_runtime_mutation(recorded)?
        } else {
            self.apply_runtime_mutation_command(RuntimeMutationCommand::UseShuckie(
                RuntimeShuckieCommand::Return { party_index },
            ))?
        };
        let RuntimeMutationResult::ShuckieUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Shuckie result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Shuckie use")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn give_odd_egg(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let recorded = self.session.stage_odd_egg(&self.runtime)?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::OddEggGiven(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Odd-Egg result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Odd Egg gift")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn give_dratini(&mut self, mode: u8) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(RuntimeMutationCommand::GiveDratini(
            RuntimeGiveDratiniCommand { mode },
        ))?;
        let RuntimeMutationResult::DratiniGiven(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Dratini result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Dratini gift")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bills_grandfather(
        &mut self,
        party_index: Option<usize>,
        species_id: Option<String>,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::UseBillsGrandfather(RuntimeBillsGrandfatherCommand {
                party_index,
                species_id,
            }),
        )?;
        let RuntimeMutationResult::BillsGrandfatherUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Bills-Grandfather result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Bill's Grandfather use")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn init_roam_mons(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(RuntimeMutationCommand::InitRoamMons)?;
        let RuntimeMutationResult::RoamersInitialized(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-roamer-init result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after roamer init")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn check_magikarp_length(
        &mut self,
        party_index: usize,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::CheckMagikarpLength(
                RuntimeMagikarpLengthCommand { party_index },
            ))?;
        let RuntimeMutationResult::MagikarpLengthChecked(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Magikarp-length result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Magikarp length check")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn show_prof_oaks_pc_boot(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::ShowProfOaksPcBoot)?;
        let RuntimeMutationResult::ProfOaksPcBootShown(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Prof-Oak-PC result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Prof Oak PC boot")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn show_magikarp_house_sign(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::ShowMagikarpHouseSign)?;
        let RuntimeMutationResult::MagikarpHouseSignShown(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Magikarp-house-sign result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Magikarp house sign")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_battle_tower_action(
        &mut self,
        action: String,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::ApplyBattleTowerAction(
                RuntimeBattleTowerActionCommand { action },
            ))?;
        let RuntimeMutationResult::BattleTowerActionApplied(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Battle-Tower-action result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Battle Tower action")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_battle_tower_room_menu(
        &mut self,
        selection: Option<u8>,
        cancelled: bool,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::UseBattleTowerRoomMenu(RuntimeBattleTowerRoomMenuCommand {
                selection,
                cancelled,
            }),
        )?;
        let RuntimeMutationResult::BattleTowerRoomMenuUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Battle-Tower-room-menu result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Battle Tower room menu")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn start_battle_tower_battle_special(
        &mut self,
        battle_result: u8,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::StartBattleTowerBattleSpecial(
                RuntimeBattleTowerBattleCommand { battle_result },
            ),
        )?;
        let RuntimeMutationResult::BattleTowerBattleStarted(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Battle-Tower-battle result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Battle Tower battle")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn load_battle_tower_opponent_special(
        &mut self,
        target_object: String,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let recorded = self
            .session
            .stage_battle_tower_opponent(&self.runtime, target_object)?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::BattleTowerOpponentLoaded(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Battle-Tower-opponent result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Battle Tower opponent load")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn show_battle_tower_mobile_error_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::ShowBattleTowerMobileErrorSpecial,
        )?;
        let RuntimeMutationResult::BattleTowerMobileErrorShown(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Battle-Tower-mobile-error result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Battle Tower mobile error")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn ask_remember_password_special(
        &mut self,
        remember: bool,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::AskRememberPasswordSpecial(RuntimeRememberPasswordCommand {
                remember,
            }),
        )?;
        let RuntimeMutationResult::RememberPasswordAsked(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-remember-password result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after remember-password prompt")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn open_battle_tower_leaderboard_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::OpenBattleTowerLeaderboardSpecial,
        )?;
        let RuntimeMutationResult::BattleTowerLeaderboardOpened(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Battle-Tower-leaderboard result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Battle Tower leaderboard")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_mobile_handshake_special(
        &mut self,
        command: RuntimeMobileHandshakeCommand,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::ApplyMobileHandshakeSpecial(command),
        )?;
        let RuntimeMutationResult::MobileHandshakeApplied(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-mobile-handshake result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after mobile handshake")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn end_mobile_session_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::EndMobileSessionSpecial)?;
        let RuntimeMutationResult::MobileSessionEnded(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-mobile-session-end result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after mobile session end")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn set_battle_tower_mobile_flag_special(
        &mut self,
        flag: RuntimeBattleTowerMobileFlag,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::SetBattleTowerMobileFlagSpecial(flag),
        )?;
        let RuntimeMutationResult::BattleTowerMobileFlagSet(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Battle-Tower-mobile-flag result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Battle Tower mobile flag")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn select_three_mobile_mons_special(
        &mut self,
        party_indexes: [usize; 3],
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::SelectThreeMobileMonsSpecial(
                RuntimeMobileSelectThreeMonsCommand { party_indexes },
            ),
        )?;
        let RuntimeMutationResult::MobileThreeMonsSelected(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-mobile-three-mons result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after mobile three-mon selection")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_happiness_service(
        &mut self,
        routine: RuntimeHappinessServiceRoutine,
        party_index: usize,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let recorded = self
            .session
            .stage_happiness_service(&self.runtime, routine, party_index)?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::HappinessServiceApplied(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-happiness-service result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after happiness service")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_mystery_gift(
        &mut self,
        action: RuntimeMysteryGiftAction,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::UseMysteryGift(action))?;
        let RuntimeMutationResult::MysteryGiftUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Mystery-Gift result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Mystery Gift use")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn warp_to_spawn_point(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::WarpToSpawnPoint)?;
        let RuntimeMutationResult::SpawnPointWarped(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-spawn-warp result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after spawn warp")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn heal_party_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::HealPartySpecial)?;
        let RuntimeMutationResult::PartyHealedBySpecial(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-special-heal result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after special party heal")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn fade_out_music_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::FadeOutMusicSpecial)?;
        let RuntimeMutationResult::MusicFadedOutBySpecial(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-special-music-fade result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after special music fade")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn wait_sfx_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::WaitSfxSpecial)?;
        let RuntimeMutationResult::SoundEffectWaitQueued(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-special-sfx-wait result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after special sfx wait")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn play_map_music_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::PlayMapMusicSpecial)?;
        let RuntimeMutationResult::MapMusicPlayedBySpecial(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-special-map-music result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after special map music")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn restart_map_music_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::RestartMapMusicSpecial)?;
        let RuntimeMutationResult::MapMusicRestartedBySpecial(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-special-map-music-restart result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after special map music restart")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn play_cur_mon_cry(&mut self, species_id: String) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::PlayCurMonCry(RuntimeSpecialCryCommand { species_id }),
        )?;
        let RuntimeMutationResult::CurrentMonCryPlayed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-current-mon-cry result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after current-mon cry")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn play_slow_cry(&mut self, species_id: String) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(RuntimeMutationCommand::PlaySlowCry(
            RuntimeSpecialCryCommand { species_id },
        ))?;
        let RuntimeMutationResult::SlowCryPlayed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-slow-cry result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after slow cry")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn open_pokemon_center_pc_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self
            .apply_runtime_mutation_command(RuntimeMutationCommand::OpenPokemonCenterPcSpecial)?;
        let RuntimeMutationResult::PokemonCenterPcOpened(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Pokemon-Center-PC result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Pokemon Center PC")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn open_players_house_pc_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::OpenPlayersHousePcSpecial)?;
        let RuntimeMutationResult::PlayersHousePcOpened(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Players-House-PC result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after player's house PC")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn open_overworld_town_map_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self
            .apply_runtime_mutation_command(RuntimeMutationCommand::OpenOverworldTownMapSpecial)?;
        let RuntimeMutationResult::OverworldTownMapOpened(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-overworld-town-map result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after overworld town map")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn open_unown_printer_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::OpenUnownPrinterSpecial)?;
        let RuntimeMutationResult::UnownPrinterOpened(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Unown-Printer result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Unown Printer")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    /// Advance radio presentation against the shared game state and RNG stream.
    pub fn advance_radio_broadcast(
        &mut self,
        broadcast: &mut crystal_assets::radio_broadcast::RadioBroadcast,
        in_johto: bool,
        held_ab: bool,
    ) -> Result<()> {
        broadcast.advance_frame(
            self.runtime.data(),
            &mut self.session.state,
            &mut self.session.divider,
            in_johto,
            held_ab,
        )
    }

    pub fn open_map_radio_special(&mut self, station: String) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::OpenMapRadioSpecial(RuntimeMapRadioCommand { station }),
        )?;
        let RuntimeMutationResult::MapRadioOpened(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Map-Radio result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Map Radio")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn name_rival_special(&mut self, rival_name: String) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::NameRivalSpecial(RuntimeNameRivalCommand { rival_name }),
        )?;
        let RuntimeMutationResult::RivalNamed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-rival-name result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after rival name")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn delete_party_move_special(
        &mut self,
        party_index: usize,
        move_index: usize,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::DeletePartyMoveSpecial(RuntimeMoveDeletionCommand {
                party_index,
                move_index,
            }),
        )?;
        let RuntimeMutationResult::PartyMoveDeletedBySpecial(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-move-deletion result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after move deletion")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn check_pokerus_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::CheckPokerusSpecial)?;
        let RuntimeMutationResult::PokerusChecked(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Pokerus-check result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Pokerus check")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn rate_party_nickname_special(
        &mut self,
        party_index: usize,
        nickname: String,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::RatePartyNicknameSpecial(RuntimePartyNicknameCommand {
                party_index,
                nickname,
            }),
        )?;
        let RuntimeMutationResult::PartyNicknameRated(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-name-rater result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after name rater")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn see_party_pokemon_special(
        &mut self,
        party_index: usize,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::SeePartyPokemonSpecial(RuntimePartySlotCommand { party_index }),
        )?;
        let RuntimeMutationResult::PartyPokemonSeenBySeer(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Poke-Seer result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Poke Seer")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn teach_party_move_special(
        &mut self,
        party_index: usize,
        move_id: String,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::TeachPartyMoveSpecial(RuntimeMoveTutorCommand {
                party_index,
                move_id,
            }),
        )?;
        let RuntimeMutationResult::PartyMoveTaughtBySpecial(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-move-tutor result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after move tutor")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn open_bank_of_mom_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::OpenBankOfMomSpecial)?;
        let RuntimeMutationResult::BankOfMomOpened(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Bank-of-Mom result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Bank of Mom")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn open_game_corner_special(
        &mut self,
        service: RuntimeGameCornerService,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let recorded = self.session.stage_game_corner(&self.runtime, service)?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::GameCornerOpened(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Game-Corner result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Game Corner service")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn open_display_link_record_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self
            .apply_runtime_mutation_command(RuntimeMutationCommand::OpenDisplayLinkRecordSpecial)?;
        let RuntimeMutationResult::DisplayLinkRecordOpened(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-display-link-record result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after display link record")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn open_trainer_house_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::OpenTrainerHouseSpecial)?;
        let RuntimeMutationResult::TrainerHouseOpened(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Trainer-House result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Trainer House")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn open_photo_studio_special(
        &mut self,
        party_index: usize,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::OpenPhotoStudioSpecial(RuntimePartySlotCommand { party_index }),
        )?;
        let RuntimeMutationResult::PhotoStudioOpened(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Photo-Studio result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Photo Studio")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_battle_tower_challenge_menu(
        &mut self,
        english: bool,
        selection: Option<u8>,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::UseBattleTowerChallengeMenu(
                RuntimeBattleTowerChallengeMenuCommand { english, selection },
            ),
        )?;
        let RuntimeMutationResult::BattleTowerChallengeMenuUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Battle-Tower-challenge-menu result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Battle Tower challenge menu")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_declared_special_routine(
        &mut self,
        routine: &str,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_special_routine_runtime_mutation(routine)?;
        let RuntimeMutationResult::SpecialRoutineApplied(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-special-routine result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .with_context(|| format!("validate runtime state after special routine {routine}"))?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_graphics_special(
        &mut self,
        special: RuntimeGraphicsSpecial,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::ApplyGraphicsSpecial(special),
        )?;
        let RuntimeMutationResult::GraphicsSpecialApplied(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-graphics-special result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after graphics special")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_party_check_special(
        &mut self,
        special: RuntimePartyCheckSpecial,
        species_id: Option<String>,
        threshold: Option<u8>,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::ApplyPartyCheckSpecial(RuntimePartyCheckCommand {
                special,
                species_id,
                threshold,
            }),
        )?;
        let RuntimeMutationResult::PartyCheckSpecialApplied(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-party-check-special result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after party check special")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_phone_random_special(
        &mut self,
        special: RuntimePhoneRandomSpecial,
        contact_id: String,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let recorded =
            self.session
                .stage_phone_random_special(&self.runtime, special, contact_id)?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::PhoneRandomSpecialApplied(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-phone-random-special result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after phone random special")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn check_item_in_pc_or_bag_special(
        &mut self,
        item_id: String,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::CheckItemInPcOrBagSpecial(RuntimePcBagItemCheckCommand {
                item_id,
            }),
        )?;
        let RuntimeMutationResult::ItemInPcOrBagChecked(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-PC-bag-item-check result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after PC/bag item check")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn check_another_usable_party_mon_special(
        &mut self,
        party_index: usize,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::CheckAnotherUsablePartyMonSpecial(RuntimePartySlotCommand {
                party_index,
            }),
        )?;
        let RuntimeMutationResult::AnotherUsablePartyMonChecked(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-another-usable-party-mon result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after usable party mon check")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn activate_fishing_swarm_special(
        &mut self,
        value: u8,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::ActivateFishingSwarmSpecial(RuntimeFishingSwarmCommand {
                value,
            }),
        )?;
        let RuntimeMutationResult::FishingSwarmActivated(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-fishing-swarm result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after fishing swarm activation")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_story_gate_special(
        &mut self,
        special: RuntimeStoryGateSpecial,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::ApplyStoryGateSpecial(special),
        )?;
        let RuntimeMutationResult::StoryGateSpecialApplied(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-story-gate-special result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after story gate special")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn set_player_palette(&mut self, raw_value: u8) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::SetPlayerPalette(RuntimePlayerPaletteCommand { raw_value }),
        )?;
        let RuntimeMutationResult::PlayerPaletteSet(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-player-palette result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after player palette set")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn set_day_of_week(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(RuntimeMutationCommand::SetDayOfWeek)?;
        let RuntimeMutationResult::DayOfWeekSet(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-day-of-week result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after day-of-week set")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn update_time(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(RuntimeMutationCommand::UpdateTime)?;
        let RuntimeMutationResult::TimeUpdated(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-time-update result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after time update")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_script_runtime_command(
        &mut self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
        inputs: ScriptRuntimeInputs,
    ) -> Result<RuntimeScriptRuntimeCommand> {
        let command = RuntimeScriptCommandRef::new(map_name, source_script, command_index);
        let is_random = self
            .runtime
            .data()
            .script_runtime_command(map_name, source_script, command_index)?
            .command
            == "random";
        let mutation = if is_random {
            anyhow::ensure!(
                inputs == ScriptRuntimeInputs::default(),
                "script random command must not declare generic runtime inputs"
            );
            let recorded = self
                .session
                .stage_random_script_runtime(&self.runtime, command)?;
            self.apply_recorded_runtime_mutation(recorded)?
        } else {
            self.apply_runtime_mutation_command(RuntimeMutationCommand::ApplyScriptRuntime {
                command,
                inputs,
            })?
        };
        let RuntimeMutationResult::ScriptRuntimeApplied(_, outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-runtime result");
        };
        Ok(RuntimeScriptRuntimeCommand {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn execute_next_queued_script_command(&mut self) -> Result<RuntimeQueuedScriptCommand> {
        let mutation = self.apply_runtime_mutation_command(
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

    pub fn take_next_script(&mut self) -> Result<RuntimeNextScript> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::TakeNextScript)?;
        let RuntimeMutationResult::NextScriptTaken(location) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-next-script result");
        };
        Ok(RuntimeNextScript {
            origin_map_name: location.origin_map_name,
            script: location.script,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn pop_script_call_stack(&mut self) -> Result<RuntimeScriptReturnResume> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::PopScriptCallStack)?;
        let RuntimeMutationResult::ScriptCallStackPopped(frame) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-call-stack-pop result");
        };
        Ok(RuntimeScriptReturnResume {
            frame,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn pop_deferred_script(&mut self) -> Result<RuntimeDeferredScript> {
        self.session.pop_deferred_script(&self.runtime)
    }

    pub fn take_script_end_state(&mut self) -> Result<RuntimeScriptEnd> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::TakeScriptEndState)?;
        let RuntimeMutationResult::ScriptEndStateTaken(end) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-end-state-take result");
        };
        Ok(RuntimeScriptEnd {
            end,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_script_swarm_command(
        &mut self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptSwarm> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::ApplyScriptSwarm(
                RuntimeScriptCommandRef::new(map_name, source_script, command_index),
            ))?;
        let RuntimeMutationResult::ScriptSwarmApplied(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-swarm result");
        };
        Ok(RuntimeScriptSwarm {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn grant_script_item(
        &mut self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptItemGrant> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::GrantScriptItem(
                RuntimeScriptCommandRef::new(map_name, source_script, command_index),
            ))?;
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
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptItemCheck> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::CheckScriptItem(
                RuntimeScriptCommandRef::new(map_name, source_script, command_index),
            ))?;
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
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptItemTake> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::TakeScriptItem(
                RuntimeScriptCommandRef::new(map_name, source_script, command_index),
            ))?;
        let RuntimeMutationResult::ScriptItemTaken(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-item-take result");
        };
        Ok(RuntimeScriptItemTake {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn pickup_script_field_item(
        &mut self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeFieldPickup> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::PickupScriptFieldItem(
                RuntimeScriptCommandRef::new(map_name, source_script, command_index),
            ))?;
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
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptEconomy> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::ApplyScriptEconomy(
                RuntimeScriptCommandRef::new(map_name, source_script, command_index),
            ))?;
        let RuntimeMutationResult::ScriptEconomyApplied(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-economy result");
        };
        Ok(RuntimeScriptEconomy {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn initialize_permanent_phone_numbers(&mut self) -> Result<RuntimePermanentPhoneNumbers> {
        let mutation = self.apply_runtime_mutation_command(
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

    pub fn start_pokegear_phone_call(
        &mut self,
        contact_id: impl Into<String>,
    ) -> Result<RuntimePokegearPhoneCallOutcome> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::StartPokegearPhoneCall(RuntimePokegearPhoneCallCommand {
                contact_id: contact_id.into(),
            }),
        )?;
        let RuntimeMutationResult::PokegearPhoneCallStarted(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Pokegear-phone-call result");
        };
        Ok(outcome)
    }

    pub fn apply_script_phone_command(
        &mut self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
        inputs: ScriptPhoneInputs,
    ) -> Result<RuntimePhoneCommand> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::ApplyScriptPhone {
                command: RuntimeScriptCommandRef::new(map_name, source_script, command_index),
                inputs,
            })?;
        let RuntimeMutationResult::ScriptPhoneApplied(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-phone result");
        };
        Ok(RuntimePhoneCommand {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn grant_scripted_gift_pokemon(
        &mut self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
        original_trainer_name: impl Into<String>,
        original_trainer_id: u16,
        nickname_accepted: bool,
        nickname: Option<String>,
    ) -> Result<RuntimeGiftPokemonGrant> {
        let recorded = self.session.stage_scripted_gift_pokemon(
            &self.runtime,
            RuntimeScriptCommandRef::new(map_name, source_script, command_index),
            original_trainer_name.into(),
            original_trainer_id,
            nickname_accepted,
            nickname,
        )?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::ScriptedGiftPokemonGranted(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-scripted-gift-pokemon result");
        };
        Ok(RuntimeGiftPokemonGrant {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn add_party_pokemon(
        &mut self,
        species_id: &str,
        level: u8,
        held_item_id: Option<String>,
        nickname: Option<String>,
        original_trainer_name: impl Into<String>,
        original_trainer_id: u16,
        dvs: Dv,
    ) -> Result<RuntimeGiftPokemonGrant> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::AddPartyPokemon(RuntimePartyPokemonCommand {
                species_id: species_id.to_string(),
                level,
                held_item_id,
                nickname,
                original_trainer_name: original_trainer_name.into(),
                original_trainer_id,
                dvs,
            }),
        )?;
        let RuntimeMutationResult::PartyPokemonAdded(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-party-pokemon-add result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after party Pokemon add")?;
        Ok(RuntimeGiftPokemonGrant {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn apply_script_flag_mutation(
        &mut self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeFlagMutation> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::ApplyScriptFlagMutation(
                RuntimeScriptCommandRef::new(map_name, source_script, command_index),
            ))?;
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
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeFlagCheck> {
        self.session
            .check_script_flag(&self.runtime, map_name, source_script, command_index)
    }

    pub fn apply_script_scene_command(
        &mut self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeSceneCommand> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::ApplyScriptScene(
                RuntimeScriptCommandRef::new(map_name, source_script, command_index),
            ))?;
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
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeBlockChange> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::ApplyScriptBlockChange(
                RuntimeScriptCommandRef::new(map_name, source_script, command_index),
            ))?;
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
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptAudio> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::ApplyScriptAudio(
                RuntimeScriptCommandRef::new(map_name, source_script, command_index),
            ))?;
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
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptMapCommand> {
        let command = RuntimeScriptCommandRef::new(map_name, source_script, command_index);
        let mutation = if self
            .runtime
            .data()
            .script_map_command(map_name, source_script, command_index)?
            .command
            == "reloadmapafterbattle"
        {
            let recorded = self
                .session
                .stage_random_script_map(&self.runtime, command)?;
            self.apply_recorded_runtime_mutation(recorded)?
        } else {
            self.apply_runtime_mutation_command(RuntimeMutationCommand::ApplyScriptMap(command))?
        };
        let RuntimeMutationResult::ScriptMapApplied(action) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-map result");
        };
        Ok(RuntimeScriptMapCommand {
            action,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn execute_pending_script_warp(&mut self) -> Result<RuntimeScriptWarp> {
        let mutation = self
            .apply_runtime_mutation_command(RuntimeMutationCommand::TransitionPendingScriptWarp)?;
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

    pub fn transition_to_spawn_point(
        &mut self,
        spawn_identifier: u16,
        map_setup: &str,
    ) -> Result<StateChecksum> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::TransitionToSpawnPoint {
                spawn_identifier,
                map_setup: map_setup.to_string(),
            })?;
        let RuntimeMutationResult::SpawnPointTransitioned {
            spawn_identifier: transitioned_identifier,
            ..
        } = mutation.result
        else {
            anyhow::bail!("runtime mutation returned non-spawn-transition result");
        };
        anyhow::ensure!(
            transitioned_identifier == spawn_identifier,
            "runtime transitioned to spawn {transitioned_identifier}, expected {spawn_identifier}"
        );
        Ok(mutation.state_checksum)
    }

    pub fn apply_current_map_setup_callbacks(&mut self, map_setup: &str) -> Result<StateChecksum> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::ApplyMapSetupCallbacks {
                map_setup: map_setup.to_string(),
            })?;
        let RuntimeMutationResult::MapSetupCallbacksApplied(applied) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-map-setup-callback result");
        };
        anyhow::ensure!(
            applied == map_setup,
            "runtime applied map setup callbacks for {applied}, expected {map_setup}"
        );
        Ok(mutation.state_checksum)
    }

    pub fn apply_script_text_command(
        &mut self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptText> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::ApplyScriptText(
                RuntimeScriptCommandRef::new(map_name, source_script, command_index),
            ))?;
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
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptVariable> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::ApplyScriptVariableNow(
                RuntimeScriptCommandRef::new(map_name, source_script, command_index),
            ))?;
        let RuntimeMutationResult::ScriptVariableApplied(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-variable result");
        };
        Ok(RuntimeScriptVariable {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn set_script_runtime_variable(
        &mut self,
        key: &str,
        value: impl Into<String>,
    ) -> Result<()> {
        self.session
            .state
            .script_runtime
            .variables
            .insert(key.to_string(), value.into());
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .with_context(|| format!("validate script runtime variable {key}"))?;
        Ok(())
    }

    pub fn set_script_runtime_accumulator(&mut self, value: impl Into<String>) -> Result<()> {
        let value = value.into();
        self.session.state.script_runtime.script_value = Some(value.clone());
        self.session
            .state
            .script_runtime
            .memory
            .insert("wScriptVar".to_string(), value);
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate script runtime accumulator")?;
        Ok(())
    }

    pub fn apply_script_control_command(
        &mut self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptControl> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::ApplyScriptControl(
                RuntimeScriptCommandRef::new(map_name, source_script, command_index),
            ))?;
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
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptObjectMutation> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::ApplyScriptObjectMutation(RuntimeScriptCommandRef::new(
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
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptMovement> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::ApplyScriptMovement(
                RuntimeScriptCommandRef::new(map_name, source_script, command_index),
            ))?;
        let RuntimeMutationResult::ScriptMovementApplied(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-movement result");
        };
        Ok(RuntimeScriptMovement {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn update_clock_from_datetime(
        &mut self,
        date: GameDate,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<RuntimeTimeUpdate> {
        let recorded =
            self.session
                .stage_clock_update(&self.runtime, date, hour, minute, second)?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::ClockUpdated = mutation.result else {
            anyhow::bail!("runtime mutation returned non-clock-update result");
        };
        Ok(RuntimeTimeUpdate {
            time_of_day: self.session.state.time.time_of_day,
            day_of_week: self.session.state.time.day_of_week,
            hour: self.session.state.time.registers.hours,
            minute: self.session.state.time.registers.minutes,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn advance_game_timer_vblank(&mut self) -> Result<RuntimeGameTimerOutcome> {
        self.advance_game_timer_vblanks(1)
    }

    pub fn advance_game_timer_vblanks(&mut self, vblanks: u32) -> Result<RuntimeGameTimerOutcome> {
        self.advance_vblanks(vblanks, 0)
    }

    /// Advance one host VBlank batch. `normal_vblanks` identifies the frames
    /// whose active handler was VBlank_Normal; each consumes exactly two DIV
    /// samples and performs the handler's carry-cleared Random update.
    pub fn advance_vblanks(
        &mut self,
        vblanks: u32,
        normal_vblanks: u32,
    ) -> Result<RuntimeGameTimerOutcome> {
        if vblanks == 0 {
            anyhow::bail!("game timer advance requires a nonzero VBlank count");
        }
        if normal_vblanks > vblanks {
            anyhow::bail!(
                "VBlank_Normal count {normal_vblanks} exceeds elapsed VBlank count {vblanks}"
            );
        }
        let mut divider_after = self.session.divider.clone();
        let mut recording = RecordingDivider::new(&mut divider_after);
        for _ in 0..normal_vblanks {
            for _ in 0..2 {
                recording
                    .next_divider()
                    .map_err(|error| anyhow::anyhow!("sample VBlank_Normal DIV: {error}"))?;
            }
        }
        let normal_divider_trace = RuntimeDividerTrace::new(recording.samples().iter().copied());
        drop(recording);
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::AdvanceGameTimerVBlanks(RuntimeGameTimerAdvanceCommand {
                vblanks,
                normal_divider_trace,
            }),
        )?;
        self.session.divider = divider_after;
        let RuntimeMutationResult::GameTimerVBlanksAdvanced(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-game-timer-vblanks result");
        };
        Ok(outcome)
    }

    pub fn set_game_timer_counting(&mut self, counting: bool) -> Result<RuntimeGameTimerOutcome> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::SetGameTimerCounting(
                RuntimeGameTimerCountingCommand { counting },
            ))?;
        let RuntimeMutationResult::GameTimerCountingSet(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-game-timer-counting result");
        };
        Ok(outcome)
    }

    pub fn set_game_logic_paused(&mut self, paused: bool) -> Result<RuntimeGameTimerOutcome> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::SetGameLogicPaused(RuntimeGameLogicPauseCommand { paused }),
        )?;
        let RuntimeMutationResult::GameLogicPauseSet(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-game-logic-pause result");
        };
        Ok(outcome)
    }

    pub fn set_manual_clock_time(
        &mut self,
        now_date: GameDate,
        now_hour: u8,
        now_minute: u8,
        now_second: u8,
        target: ClockTime,
    ) -> Result<RuntimeTimeUpdate> {
        let recorded = self.session.stage_manual_clock_update(
            &self.runtime,
            now_date,
            now_hour,
            now_minute,
            now_second,
            target,
        )?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::ManualClockSet = mutation.result else {
            anyhow::bail!("runtime mutation returned non-manual-clock result");
        };
        Ok(RuntimeTimeUpdate {
            time_of_day: self.session.state.time.time_of_day,
            day_of_week: self.session.state.time.day_of_week,
            hour: self.session.state.time.registers.hours,
            minute: self.session.state.time.registers.minutes,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn open_script_shop(
        &mut self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<RuntimeScriptShop> {
        self.require_valid_script_modal_state("open script shop")?;
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::OpenScriptShop(
                RuntimeScriptCommandRef::new(map_name, source_script, command_index),
            ))?;
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
        item_id: &str,
        quantity: u16,
    ) -> Result<RuntimeShopTransaction> {
        let mutation = self.apply_runtime_mutation_command(RuntimeMutationCommand::BuyShopItem(
            RuntimeShopTransactionCommand {
                item_id: item_id.to_string(),
                quantity,
            },
        ))?;
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
        item_id: &str,
        quantity: u16,
    ) -> Result<RuntimeShopTransaction> {
        let mutation = self.apply_runtime_mutation_command(
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

    pub fn use_bag_item(
        &mut self,
        item_id: &str,
        context: ItemUseContext,
    ) -> Result<RuntimeItemUse> {
        let mutation = self.apply_runtime_mutation_command(RuntimeMutationCommand::UseBagItem {
            item_id: item_id.to_string(),
            context,
        })?;
        let RuntimeMutationResult::BagItemUsed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-item-use result");
        };
        Ok(RuntimeItemUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn register_key_item(&mut self, item_id: &str) -> Result<RuntimeRegisteredKeyItem> {
        let mutation = self.apply_runtime_mutation_command(
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

    pub fn use_bag_repel_in_field(&mut self, item_id: &str) -> Result<RuntimeRepelItemUse> {
        let mutation = self.apply_runtime_mutation_command(
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

    pub fn use_bag_bicycle_in_field(&mut self, item_id: &str) -> Result<RuntimeBicycleItemUse> {
        let mutation = self.apply_runtime_mutation_command(
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

    pub fn use_bag_itemfinder_in_field(&mut self, item_id: &str) -> Result<RuntimeItemfinderUse> {
        let mutation = self.apply_runtime_mutation_command(
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
        item_id: &str,
    ) -> Result<RuntimeSquirtBottleUse> {
        let mutation = self.apply_runtime_mutation_command(
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

    pub fn use_bag_story_key_in_field(&mut self, item_id: &str) -> Result<RuntimeStoryKeyUse> {
        let mutation = self.apply_runtime_mutation_command(
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
        item_id: &str,
    ) -> Result<RuntimeKeyItemBalanceUse> {
        let mutation = self.apply_runtime_mutation_command(
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
        item_id: &str,
    ) -> Result<RuntimeKeyItemBalanceUse> {
        let mutation = self.apply_runtime_mutation_command(
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

    pub fn use_bag_town_map_in_field(&mut self, item_id: &str) -> Result<RuntimeTownMapUse> {
        let mutation = self.apply_runtime_mutation_command(
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

    pub fn is_bag_pokegear_item(&self, item_id: &str) -> bool {
        self.runtime.data.field_moves.pokegear.item_id == item_id
    }

    pub fn is_bag_box_item(&self, item_id: &str) -> bool {
        self.runtime.data.field_box_items.contains_key(item_id)
    }

    pub fn use_bag_pokegear_in_field(&mut self, item_id: &str) -> Result<RuntimePokegearUse> {
        let mutation = self.apply_runtime_mutation_command(
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

    pub fn use_bag_box_in_field(&mut self, item_id: &str) -> Result<RuntimeBoxItemUse> {
        let mutation = self.apply_runtime_mutation_command(
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

    pub fn use_bag_escape_rope_in_field(&mut self, item_id: &str) -> Result<RuntimeEscapeRopeUse> {
        let mutation = self.apply_runtime_mutation_command(
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

    pub fn cast_fishing_rod(&mut self, rod: &str) -> Result<RuntimeFishingCast> {
        let recorded = self.session.stage_fishing_rod_cast(&self.runtime, rod)?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
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
        item_id: &str,
    ) -> Result<RuntimeFishingRodItemUse> {
        let recorded = self
            .session
            .stage_bag_fishing_rod_use(&self.runtime, item_id)?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
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

    pub fn use_cut_field_move(
        &mut self,
        party_index: usize,
        metatile_x: u16,
        metatile_y: u16,
    ) -> Result<RuntimeFieldMoveBlockUse> {
        let mutation = self.apply_runtime_mutation_command(
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
        party_index: usize,
    ) -> Result<RuntimeFieldMoveBlockUse> {
        let (metatile_x, metatile_y) = self
            .runtime
            .data()
            .field_block_target_metatile_in_front(&self.session.overworld)?;
        self.use_cut_field_move(party_index, metatile_x, metatile_y)
    }

    pub fn use_whirlpool_field_move(
        &mut self,
        party_index: usize,
        metatile_x: u16,
        metatile_y: u16,
    ) -> Result<RuntimeFieldMoveBlockUse> {
        let mutation = self.apply_runtime_mutation_command(
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
        party_index: usize,
    ) -> Result<RuntimeFieldMoveBlockUse> {
        let (metatile_x, metatile_y) = self
            .runtime
            .data()
            .field_block_target_metatile_in_front(&self.session.overworld)?;
        self.use_whirlpool_field_move(party_index, metatile_x, metatile_y)
    }

    pub fn queue_strength_from_menu(
        &mut self,
        party_index: usize,
    ) -> Result<RuntimeInteractionScriptDispatch> {
        let mutation = self.apply_runtime_mutation_command(
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

    pub fn use_flash_field_move(&mut self, party_index: usize) -> Result<RuntimeFieldMoveFlagUse> {
        let mutation = self.apply_runtime_mutation_command(
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

    pub fn use_surf_field_move(&mut self, party_index: usize) -> Result<RuntimeFieldMoveTravelUse> {
        let mutation = self.apply_runtime_mutation_command(
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
        party_index: usize,
    ) -> Result<RuntimeFieldMoveTravelUse> {
        let mutation = self.apply_runtime_mutation_command(
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
        party_index: usize,
        destination_spawn_identifier: u16,
        flypoint_flag: &str,
    ) -> Result<RuntimeFlyFieldMoveUse> {
        let mutation = self.apply_runtime_mutation_command(
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

    pub fn use_dig_field_move(&mut self, party_index: usize) -> Result<RuntimeDigFieldMoveUse> {
        let mutation = self.apply_runtime_mutation_command(
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
        party_index: usize,
    ) -> Result<RuntimeTeleportFieldMoveUse> {
        let mutation = self.apply_runtime_mutation_command(
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

    pub fn commit_pending_field_travel(&mut self) -> Result<PendingFieldTravel> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::CommitPendingFieldTravel)?;
        let RuntimeMutationResult::FieldTravelCommitted(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-field-travel commit result");
        };
        Ok(outcome)
    }

    pub fn queue_headbutt_script(
        &mut self,
        party_index: usize,
        from_menu: bool,
    ) -> Result<RuntimeInteractionScriptDispatch> {
        let mutation = self.apply_runtime_mutation_command(
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
        party_index: usize,
    ) -> Result<RuntimeInteractionScriptDispatch> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::QueueRockSmashFromMenu(
                RuntimeFieldPartyCommand { party_index },
            ))?;
        let RuntimeMutationResult::RockSmashFromMenuQueued(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-RockSmashFromMenu result");
        };
        Ok(RuntimeInteractionScriptDispatch {
            next_script: outcome.next_script,
            last_talked_object: Some(outcome.object_identifier),
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn current_encounter_surface(&self) -> Option<EncounterSurface> {
        self.current_encounter_surface_checked()
            .expect("current encounter surface requires verified map metadata and collision")
    }

    pub fn current_encounter_surface_checked(&self) -> Result<Option<EncounterSurface>> {
        let environment = &self
            .runtime
            .data()
            .runtime_map_metadata_for_name(&self.session.overworld.map.name)?
            .environment;
        let land_encounters_on_any_land =
            environment.eq_ignore_ascii_case("cave") || environment.eq_ignore_ascii_case("dungeon");
        self.session
            .overworld
            .current_encounter_surface_checked_with_land_encounters(land_encounters_on_any_land)
            .map_err(|error| anyhow::anyhow!("current encounter surface: {error}"))
    }

    pub fn queue_sweet_scent_from_menu(
        &mut self,
        party_index: usize,
    ) -> Result<RuntimeInteractionScriptDispatch> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::QueueSweetScentFromMenu(
                RuntimeFieldPartyCommand { party_index },
            ))?;
        let RuntimeMutationResult::SweetScentFromMenuQueued(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-SweetScentFromMenu result");
        };
        Ok(RuntimeInteractionScriptDispatch {
            next_script: outcome.next_script,
            last_talked_object: None,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn start_scripted_wild_battle(
        &mut self,
        map_name: &str,
        source_script: &str,
        startbattle_command_index: usize,
    ) -> Result<StaticWildBattleStart> {
        let recorded = self.session.stage_scripted_wild_battle_start(
            &self.runtime,
            RuntimeScriptCommandRef::new(map_name, source_script, startbattle_command_index),
        )?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::ScriptedWildBattleStarted(start) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-scripted-wild-battle-start result");
        };
        Ok(start)
    }

    pub fn start_scripted_trainer_battle(
        &mut self,
        map_name: &str,
        source_script: &str,
        startbattle_command_index: usize,
    ) -> Result<TrainerBattleStartStatus> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::StartScriptedTrainerBattle(RuntimeScriptCommandRef::new(
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

    pub fn start_link_battle(&mut self, start: &LinkBattleStart) -> Result<StateChecksum> {
        activate_link_battle_start(self.session.state_mut(), start, &self.runtime.data().items)
            .context("activate Colosseum link battle")?;
        self.runtime
            .validate_save_state_for_runtime_pack(self.session.state())
            .context("validate runtime state after Colosseum link battle start")?;
        self.state_checksum()
    }

    pub fn switch_link_battle_enemy_party(
        &mut self,
        party_index: usize,
    ) -> Result<RuntimeBattlePartySwitch> {
        let outcome = switch_active_battle_enemy_party_index(self.session.state_mut(), party_index)
            .context("apply peer Colosseum replacement")?;
        self.runtime
            .validate_save_state_for_runtime_pack(self.session.state())
            .context("validate runtime state after peer Colosseum replacement")?;
        Ok(RuntimeBattlePartySwitch {
            party_index: outcome.party_index,
            spikes: outcome.spikes,
            state_checksum: self.state_checksum()?,
        })
    }

    pub fn finish_link_battle(&mut self, result: RuntimeLinkBattleResult) -> Result<StateChecksum> {
        let is_link_battle = matches!(
            &self.session.state().battle,
            crate::core::state::BattleMemory::Trainer { battle_type, .. }
                if battle_type == "BATTLETYPE_LINK"
        );
        anyhow::ensure!(
            is_link_battle,
            "finish link battle requires an active link battle"
        );
        match result {
            RuntimeLinkBattleResult::Win => deactivate_battle_after_win(self.session.state_mut()),
            RuntimeLinkBattleResult::Loss => deactivate_battle_after_loss(self.session.state_mut()),
            RuntimeLinkBattleResult::Draw => deactivate_battle_after_draw(self.session.state_mut()),
        }
        self.runtime
            .validate_save_state_for_runtime_pack(self.session.state())
            .context("validate runtime state after link battle completion")?;
        self.state_checksum()
    }

    pub fn generate_link_battle_random_state(&mut self) -> Result<LinkBattleRandomState> {
        const SERIAL_PREAMBLE_BYTE: u8 = 0xfd;
        let RuntimeOverworldSession { state, divider, .. } = &mut self.session;
        let mut rng = CrystalRandom::new(state.random_state, divider);
        let mut seeds = [0_u8; LINK_BATTLE_RANDOM_SEED_COUNT];
        let mut index = 0;
        let mut carry = false;
        while index < seeds.len() {
            let output = rng
                .random(carry)
                .context("sample Colosseum link random seed")?;
            carry = output.value < SERIAL_PREAMBLE_BYTE;
            if output.value >= SERIAL_PREAMBLE_BYTE {
                continue;
            }
            seeds[index] = output.value;
            index += 1;
        }
        state.random_state = rng.state();
        Ok(LinkBattleRandomState { seeds, count: 0 })
    }

    pub fn complete_scripted_wild_battle(
        &mut self,
        origin: RuntimeStaticWildBattleOrigin,
    ) -> Result<RuntimeScriptedBattleCompletion> {
        let recorded = self
            .session
            .stage_scripted_wild_battle_completion(&self.runtime, origin)?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
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
        map_name: &str,
        source_script: &str,
        startbattle_command_index: usize,
        won: bool,
        can_lose: bool,
    ) -> Result<RuntimeScriptedBattleCompletion> {
        let recorded = self.session.stage_scripted_trainer_battle_completion(
            &self.runtime,
            map_name,
            source_script,
            startbattle_command_index,
            won,
            can_lose,
        )?;
        let completion_mutation = self.apply_recorded_runtime_mutation(recorded)?;
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

    pub fn dispatch_interaction_script(
        &mut self,
        interaction: &OverworldInteraction,
    ) -> Result<RuntimeInteractionScriptDispatch> {
        self.session
            .dispatch_interaction_script(&self.runtime, interaction)
    }

    pub fn dispatch_coord_event_script(
        &mut self,
        coord_event: &CoordEventTrigger,
    ) -> Result<RuntimeInteractionScriptDispatch> {
        self.session
            .dispatch_coord_event_script(&self.runtime, coord_event)
    }

    pub fn resolve_active_battle_command(
        &mut self,
        player_action: BattleAction,
        enemy_action: BattleAction,
    ) -> Result<RuntimeBattleCommand> {
        let recorded =
            self.session
                .stage_active_battle_command(&self.runtime, player_action, enemy_action)?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::ActiveBattleCommandResolved(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-battle-command result");
        };
        Ok(RuntimeBattleCommand {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn resolve_active_battle_turn(
        &mut self,
        player_action: BattleAction,
        enemy_action: BattleAction,
    ) -> Result<RuntimeBattleTurn> {
        let recorded =
            self.session
                .stage_active_battle_turn(&self.runtime, player_action, enemy_action)?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::ActiveBattleTurnResolved(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-battle-turn result");
        };
        Ok(RuntimeBattleTurn {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn resolve_active_battle_turn_with_enemy_selector<F>(
        &mut self,
        player_action: BattleAction,
        select_enemy_action: F,
    ) -> Result<(BattleAction, RuntimeBattleTurn)>
    where
        F: FnOnce(&mut dyn crystal_core::random::BattleRandomSource) -> Result<BattleAction>,
    {
        let (enemy_action, _, recorded) =
            self.session.stage_active_battle_turn_with_enemy_selector(
                &self.runtime,
                player_action,
                None,
                select_enemy_action,
            )?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::ActiveBattleTurnResolved(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-battle-turn result");
        };
        Ok((
            enemy_action,
            RuntimeBattleTurn {
                outcome,
                state_checksum: mutation.state_checksum,
            },
        ))
    }

    pub fn resolve_active_battle_item_turn_with_enemy_selector<F>(
        &mut self,
        item_id: &str,
        select_enemy_action: F,
    ) -> Result<RuntimeBattleItemTurn>
    where
        F: FnOnce(&mut dyn crystal_core::random::BattleRandomSource) -> Result<BattleAction>,
    {
        let player_action = BattleAction::Item {
            item_id: item_id.to_string(),
        };
        let (enemy_action, item_use, recorded) =
            self.session.stage_active_battle_turn_with_enemy_selector(
                &self.runtime,
                player_action,
                Some(item_id),
                select_enemy_action,
            )?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::ActiveBattleTurnResolved(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-battle-turn result");
        };
        let item_use = item_use.context("battle item turn did not consume its Bag item")?;
        let battle_item = player_battle_item_outcome(&outcome)?;
        Ok(RuntimeBattleItemTurn {
            item_use,
            battle_item,
            enemy_action,
            turn: RuntimeBattleTurn {
                outcome,
                state_checksum: mutation.state_checksum,
            },
        })
    }

    pub fn resolve_active_battle_party_item_turn_with_enemy_selector<F>(
        &mut self,
        item_id: &str,
        party_index: usize,
        move_slot: Option<usize>,
        select_enemy_action: F,
    ) -> Result<RuntimeBattleItemTurn>
    where
        F: FnOnce(&mut dyn crystal_core::random::BattleRandomSource) -> Result<BattleAction>,
    {
        let player_action = BattleAction::PartyItem {
            item_id: item_id.to_string(),
            party_index,
            move_slot,
        };
        let (enemy_action, item_use, recorded) =
            self.session.stage_active_battle_turn_with_enemy_selector(
                &self.runtime,
                player_action,
                Some(item_id),
                select_enemy_action,
            )?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::ActiveBattleTurnResolved(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-battle-turn result");
        };
        let item_use = item_use.context("battle party item turn did not consume its Bag item")?;
        let battle_item = player_battle_item_outcome(&outcome)?;
        Ok(RuntimeBattleItemTurn {
            item_use,
            battle_item,
            enemy_action,
            turn: RuntimeBattleTurn {
                outcome,
                state_checksum: mutation.state_checksum,
            },
        })
    }

    pub fn resolve_active_battle_turn_with_enemy_selectors<FM, FP>(
        &mut self,
        player_action: BattleAction,
        select_enemy_move: FM,
        select_enemy_post_order_action: FP,
    ) -> Result<(BattleAction, RuntimeBattleTurn)>
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
        let (enemy_action, _, recorded) =
            self.session.stage_active_battle_turn_with_enemy_selectors(
                &self.runtime,
                player_action,
                None,
                select_enemy_move,
                select_enemy_post_order_action,
            )?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::ActiveBattleTurnResolved(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-battle-turn result");
        };
        Ok((
            enemy_action,
            RuntimeBattleTurn {
                outcome,
                state_checksum: mutation.state_checksum,
            },
        ))
    }

    pub fn resolve_active_battle_item_turn_with_enemy_selectors<FM, FP>(
        &mut self,
        item_id: &str,
        select_enemy_move: FM,
        select_enemy_post_order_action: FP,
    ) -> Result<RuntimeBattleItemTurn>
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
        let player_action = BattleAction::Item {
            item_id: item_id.to_string(),
        };
        let (enemy_action, item_use, recorded) =
            self.session.stage_active_battle_turn_with_enemy_selectors(
                &self.runtime,
                player_action,
                Some(item_id),
                select_enemy_move,
                select_enemy_post_order_action,
            )?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::ActiveBattleTurnResolved(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-battle-turn result");
        };
        let item_use = item_use.context("battle item turn did not consume its Bag item")?;
        let battle_item = player_battle_item_outcome(&outcome)?;
        Ok(RuntimeBattleItemTurn {
            item_use,
            battle_item,
            enemy_action,
            turn: RuntimeBattleTurn {
                outcome,
                state_checksum: mutation.state_checksum,
            },
        })
    }

    pub fn resolve_active_battle_party_item_turn_with_enemy_selectors<FM, FP>(
        &mut self,
        item_id: &str,
        party_index: usize,
        move_slot: Option<usize>,
        select_enemy_move: FM,
        select_enemy_post_order_action: FP,
    ) -> Result<RuntimeBattleItemTurn>
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
        let player_action = BattleAction::PartyItem {
            item_id: item_id.to_string(),
            party_index,
            move_slot,
        };
        let (enemy_action, item_use, recorded) =
            self.session.stage_active_battle_turn_with_enemy_selectors(
                &self.runtime,
                player_action,
                Some(item_id),
                select_enemy_move,
                select_enemy_post_order_action,
            )?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::ActiveBattleTurnResolved(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-battle-turn result");
        };
        let item_use = item_use.context("battle party item turn did not consume its Bag item")?;
        let battle_item = player_battle_item_outcome(&outcome)?;
        Ok(RuntimeBattleItemTurn {
            item_use,
            battle_item,
            enemy_action,
            turn: RuntimeBattleTurn {
                outcome,
                state_checksum: mutation.state_checksum,
            },
        })
    }

    pub fn resolve_active_battle_enemy_action(
        &mut self,
        enemy_action: BattleAction,
    ) -> Result<RuntimeBattleTurn> {
        let recorded = self
            .session
            .stage_active_battle_enemy_action(&self.runtime, enemy_action)?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::ActiveBattleEnemyActionResolved(outcome) = mutation.result
        else {
            anyhow::bail!("runtime mutation returned non-enemy-battle-action result");
        };
        Ok(RuntimeBattleTurn {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn resolve_active_battle_enemy_action_with_selector<F>(
        &mut self,
        select_enemy_action: F,
    ) -> Result<(BattleAction, RuntimeBattleTurn)>
    where
        F: FnOnce(
            &crystal_core::battle::turn::BattleCombatState,
            &mut dyn crystal_core::random::BattleRandomSource,
        ) -> Result<BattleAction>,
    {
        let (enemy_action, recorded) = self
            .session
            .stage_active_battle_enemy_action_with_selector(&self.runtime, select_enemy_action)?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::ActiveBattleEnemyActionResolved(outcome) = mutation.result
        else {
            anyhow::bail!("runtime mutation returned non-enemy-battle-action result");
        };
        Ok((
            enemy_action,
            RuntimeBattleTurn {
                outcome,
                state_checksum: mutation.state_checksum,
            },
        ))
    }

    pub fn attempt_escape_active_wild_battle(&mut self) -> Result<RuntimeBattleEscape> {
        let recorded = self
            .session
            .stage_escape_active_wild_battle(&self.runtime)?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::ActiveWildBattleEscapeAttempted(outcome) = mutation.result
        else {
            anyhow::bail!("runtime mutation returned non-battle-escape result");
        };
        Ok(RuntimeBattleEscape {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn throw_ball_at_active_battle(&mut self, ball_id: &str) -> Result<RuntimeCaptureAttempt> {
        let recorded = self
            .session
            .stage_throw_ball_at_active_battle(&self.runtime, ball_id)?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::BallThrown(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-capture result");
        };
        Ok(RuntimeCaptureAttempt {
            outcome: Some(outcome),
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn active_battle_capture_storage_full(&self) -> Result<bool> {
        self.runtime
            .data
            .active_battle_capture_storage_full(&self.session.state)
    }

    pub fn resolve_active_battle_ball_turn_with_enemy_selectors<FM, FP>(
        &mut self,
        ball_id: &str,
        select_enemy_move: FM,
        select_enemy_post_order_action: FP,
    ) -> Result<RuntimeBattleBallTurn>
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
        let (enemy_action, _, recorded) =
            self.session.stage_active_battle_turn_with_enemy_selectors(
                &self.runtime,
                BattleAction::Ball {
                    item_id: ball_id.to_string(),
                },
                None,
                select_enemy_move,
                select_enemy_post_order_action,
            )?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::ActiveBattleTurnResolved(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Ball battle-turn result");
        };
        let capture = outcome
            .events
            .iter()
            .find_map(|event| match event {
                BattleEvent::BallThrown {
                    side: BattleSide::Player,
                    outcome,
                } => Some(outcome.clone()),
                _ => None,
            })
            .context("Ball battle turn did not record its capture attempt")?;
        Ok(RuntimeBattleBallTurn {
            capture,
            enemy_action,
            turn: RuntimeBattleTurn {
                outcome,
                state_checksum: mutation.state_checksum,
            },
        })
    }

    pub fn complete_active_wild_capture(
        &mut self,
        outcome: &CaptureOutcome,
        nickname: Option<String>,
    ) -> Result<RuntimeCaptureCompletion> {
        let recorded =
            self.session
                .stage_active_wild_capture_completion(&self.runtime, outcome, nickname)?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
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
        keep_new: bool,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
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

    pub fn use_bag_item_to_escape_active_wild_battle(
        &mut self,
        item_id: &str,
    ) -> Result<RuntimeBattleEscapeItemUse> {
        let recorded = self
            .session
            .stage_bag_item_escape_active_wild_battle(&self.runtime, item_id)?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
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
        item_id: &str,
    ) -> Result<RuntimeBattleStateItemUse> {
        let mutation = self.apply_runtime_mutation_command(
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
        party_index: usize,
    ) -> Result<RuntimeBattlePartySwitch> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::SwitchActiveBattleParty(
                RuntimePartySlotCommand { party_index },
            ))?;
        let RuntimeMutationResult::ActiveBattlePartySwitched(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-active-battle-party-switch result");
        };
        Ok(RuntimeBattlePartySwitch {
            party_index: outcome.party_index,
            spikes: outcome.spikes,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn use_bag_item_on_party_pokemon(
        &mut self,
        item_id: &str,
        party_index: usize,
    ) -> Result<RuntimePartyItemUse> {
        let mutation = self.apply_runtime_mutation_command(
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
        item_id: &str,
    ) -> Result<RuntimeWholePartyItemUse> {
        let mutation = self.apply_runtime_mutation_command(
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
        item_id: &str,
        party_index: usize,
        move_slot: Option<usize>,
    ) -> Result<RuntimePartyItemUse> {
        let mutation = self.apply_runtime_mutation_command(
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

    pub fn use_bag_item_on_active_battle_pokemon(
        &mut self,
        item_id: &str,
    ) -> Result<RuntimeBattleItemUse> {
        let mutation = self.apply_runtime_mutation_command(
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

    pub fn use_bag_tmhm_on_party_pokemon(
        &mut self,
        item_id: &str,
        party_index: usize,
        replace_slot: Option<usize>,
    ) -> Result<RuntimeTmHmItemUse> {
        let mutation = self.apply_runtime_mutation_command(
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

    pub fn preview_tmhm_on_party_pokemon(
        &self,
        item_id: &str,
        party_index: usize,
        replace_slot: Option<usize>,
    ) -> Result<TmHmLearnOutcome> {
        let mut pokemon = self
            .session
            .state
            .storage
            .party
            .pokemon
            .get(party_index)
            .and_then(|pokemon| pokemon.clone())
            .with_context(|| format!("party index {party_index} has no Pokemon"))?;
        self.runtime
            .data
            .teach_tmhm_move(&mut pokemon, item_id, replace_slot, false)
    }

    pub fn preview_party_item_on_pokemon(
        &self,
        item_id: &str,
        party_index: usize,
    ) -> Result<BattleItemOutcome> {
        let mut pokemon = self
            .session
            .state
            .storage
            .party
            .pokemon
            .get(party_index)
            .and_then(|pokemon| pokemon.clone())
            .with_context(|| format!("party index {party_index} has no Pokemon"))?;
        let level_up_happiness = self
            .runtime
            .data
            .level_up_happiness_context(&self.session.state)?;
        self.runtime.data.apply_party_pokemon_item_effect(
            &mut pokemon,
            item_id,
            level_up_happiness,
            self.session.state.time.time_of_day,
            false,
        )
    }

    pub fn use_bag_item_on_battle_party_pokemon(
        &mut self,
        item_id: &str,
        party_index: usize,
    ) -> Result<RuntimeBattleItemUse> {
        let mutation = self.apply_runtime_mutation_command(
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
        item_id: &str,
        party_index: usize,
        move_slot: Option<usize>,
    ) -> Result<RuntimeBattleItemUse> {
        let mutation = self.apply_runtime_mutation_command(
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

    pub fn advance_active_trainer_battle(&mut self) -> Result<RuntimeTrainerBattleAdvance> {
        let mutation = self
            .apply_runtime_mutation_command(RuntimeMutationCommand::AdvanceActiveTrainerBattle)?;
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

    pub fn claim_active_trainer_battle_rewards(&mut self) -> Result<RuntimeBattleRewards> {
        let mutation = self.apply_runtime_mutation_command(
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

    pub fn claim_active_wild_battle_rewards(&mut self) -> Result<RuntimeBattleRewards> {
        let recorded = self.session.stage_wild_battle_rewards(&self.runtime)?;
        let mutation = self.apply_recorded_runtime_mutation(recorded)?;
        let RuntimeMutationResult::ActiveWildBattleRewardsClaimed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-wild-rewards result");
        };
        Ok(RuntimeBattleRewards {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn switch_current_pc_box(&mut self, box_index: usize) -> Result<RuntimeStorageBoxSwitch> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::SwitchCurrentPcBox(RuntimePcBoxCommand { box_index }),
        )?;
        let RuntimeMutationResult::CurrentPcBoxSwitched(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-PC-box-switch result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after PC box switch")?;
        Ok(RuntimeStorageBoxSwitch {
            box_index_before: outcome.box_index_before,
            box_index_after: outcome.box_index_after,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn name_pc_box(&mut self, box_index: usize, name: String) -> Result<RuntimeStorageBoxName> {
        let mutation = self.apply_runtime_mutation_command(RuntimeMutationCommand::NamePcBox(
            RuntimePcBoxNameCommand { box_index, name },
        ))?;
        let RuntimeMutationResult::PcBoxNamed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-PC-box-name result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after PC box naming")?;
        Ok(RuntimeStorageBoxName {
            box_index: outcome.box_index,
            previous_name: outcome.previous_name,
            name: outcome.name,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn deposit_party_pokemon_to_current_box(
        &mut self,
        party_index: usize,
    ) -> Result<RuntimeStorageDeposit> {
        // BillsPC.PartyToBox pauses wGameLogicPaused across its save boundary.
        // The Rust storage mutation is atomic, so bracket that boundary in
        // the deterministic command journal and always restore the prior
        // control byte, including on a rejected deposit.
        let paused_before = self.session.state.game_logic_paused;
        self.set_game_logic_paused(true)?;
        let deposit_result = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::DepositPartyPokemonToCurrentBox(RuntimePcDepositCommand {
                party_index,
            }),
        );
        let restore_result = self.set_game_logic_paused(paused_before);
        let mutation = match (deposit_result, restore_result) {
            (Ok(mutation), Ok(_)) => mutation,
            (Err(error), Ok(_)) => return Err(error),
            (Ok(_), Err(error)) => {
                return Err(error).context("resume game logic after PC party deposit");
            }
            (Err(deposit_error), Err(restore_error)) => {
                return Err(deposit_error).context(format!(
                    "also failed to resume game logic after PC party deposit: {restore_error:#}"
                ));
            }
        };
        let RuntimeMutationResult::PartyPokemonDeposited(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-PC-deposit result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after party deposit")?;
        Ok(RuntimeStorageDeposit {
            party_index: outcome.party_index,
            box_index: outcome.box_index,
            box_slot: outcome.box_slot,
            pokemon: outcome.pokemon,
            state_checksum: game_state_checksum(&self.session.state)
                .context("checksum runtime state after resuming PC party deposit")?,
        })
    }

    pub fn withdraw_current_box_pokemon_to_party(
        &mut self,
        box_slot: usize,
    ) -> Result<RuntimeStorageWithdraw> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::WithdrawCurrentBoxPokemonToParty(RuntimePcWithdrawCommand {
                box_slot,
            }),
        )?;
        let RuntimeMutationResult::PcPokemonWithdrawn(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-PC-withdraw result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after PC withdraw")?;
        Ok(RuntimeStorageWithdraw {
            box_index: outcome.box_index,
            box_slot: outcome.box_slot,
            party_index: outcome.party_index,
            pokemon: outcome.pokemon,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn release_current_box_pokemon(
        &mut self,
        box_slot: usize,
    ) -> Result<RuntimeStorageRelease> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::ReleaseCurrentBoxPokemon(RuntimePcReleaseCommand { box_slot }),
        )?;
        let RuntimeMutationResult::PcPokemonReleased(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-PC-release result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after PC release")?;
        Ok(RuntimeStorageRelease {
            box_index: outcome.box_index,
            box_slot: outcome.box_slot,
            pokemon: outcome.pokemon,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn move_pokemon_without_mail(
        &mut self,
        source: crystal_assets::RuntimePokemonStorageLocation,
        target: crystal_assets::RuntimePokemonStorageLocation,
    ) -> Result<RuntimeStorageMove> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::MovePcPokemonWithoutMail(
                RuntimePcMoveCommand { source, target },
            ))?;
        let RuntimeMutationResult::PcPokemonMoved(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-PC-move result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after PC move")?;
        Ok(RuntimeStorageMove {
            source: outcome.source,
            target: outcome.target,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn deposit_bag_item_to_pc(
        &mut self,
        item_id: &str,
        stack_index: usize,
        quantity: u16,
    ) -> Result<RuntimePcItemTransfer> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::DepositBagItemToPc(RuntimePcItemCommand {
                item_id: item_id.to_string(),
                stack_index,
                quantity,
            }),
        )?;
        let RuntimeMutationResult::BagItemDepositedToPc(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-PC-item-deposit result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after PC item deposit")?;
        Ok(RuntimePcItemTransfer {
            item_id: outcome.item_id,
            quantity: outcome.quantity,
            bag_quantity_after: outcome.bag_quantity_after,
            pc_quantity_after: outcome.pc_quantity_after,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn withdraw_pc_item_to_bag(
        &mut self,
        item_id: &str,
        stack_index: usize,
        quantity: u16,
    ) -> Result<RuntimePcItemTransfer> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::WithdrawPcItemToBag(RuntimePcItemCommand {
                item_id: item_id.to_string(),
                stack_index,
                quantity,
            }),
        )?;
        let RuntimeMutationResult::PcItemWithdrawnToBag(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-PC-item-withdraw result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after PC item withdraw")?;
        Ok(RuntimePcItemTransfer {
            item_id: outcome.item_id,
            quantity: outcome.quantity,
            bag_quantity_after: outcome.bag_quantity_after,
            pc_quantity_after: outcome.pc_quantity_after,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn toss_pc_item(
        &mut self,
        item_id: &str,
        stack_index: usize,
        quantity: u16,
    ) -> Result<RuntimePcItemTransfer> {
        let mutation = self.apply_runtime_mutation_command(RuntimeMutationCommand::TossPcItem(
            RuntimePcItemCommand {
                item_id: item_id.to_string(),
                stack_index,
                quantity,
            },
        ))?;
        let RuntimeMutationResult::PcItemTossed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-PC-item-toss result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after PC item toss")?;
        Ok(RuntimePcItemTransfer {
            item_id: outcome.item_id,
            quantity: outcome.quantity,
            bag_quantity_after: outcome.bag_quantity_after,
            pc_quantity_after: outcome.pc_quantity_after,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn give_bag_item_to_party_pokemon(
        &mut self,
        item_id: &str,
        party_index: usize,
    ) -> Result<RuntimeHeldItemTransfer> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::GiveBagItemToPartyPokemon(RuntimeHeldItemCommand {
                item_id: item_id.to_string(),
                party_index,
            }),
        )?;
        let RuntimeMutationResult::PartyPokemonHeldItemGiven(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-held-item-give result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after held item give")?;
        Ok(RuntimeHeldItemTransfer {
            party_index: outcome.party_index,
            item_id: outcome.item_id,
            bag_quantity_after: outcome.bag_quantity_after,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn take_held_item_from_party_pokemon(
        &mut self,
        party_index: usize,
    ) -> Result<RuntimeHeldItemTransfer> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::TakeHeldItemFromPartyPokemon(RuntimePartySlotCommand {
                party_index,
            }),
        )?;
        let RuntimeMutationResult::PartyPokemonHeldItemTaken(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-held-item-take result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after held item take")?;
        Ok(RuntimeHeldItemTransfer {
            party_index: outcome.party_index,
            item_id: outcome.item_id,
            bag_quantity_after: outcome.bag_quantity_after,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn compose_bag_mail_to_party(
        &mut self,
        item_id: &str,
        party_index: usize,
        message: String,
    ) -> Result<RuntimeMailTransfer> {
        self.apply_mail_mutation(RuntimeMutationCommand::ComposeBagMailToParty(
            RuntimeComposeMailCommand {
                item_id: item_id.to_string(),
                party_index,
                message,
            },
        ))
    }

    pub fn send_party_mail_to_mailbox(
        &mut self,
        party_index: usize,
    ) -> Result<RuntimeMailTransfer> {
        self.apply_mail_mutation(RuntimeMutationCommand::SendPartyMailToMailbox(
            RuntimePartySlotCommand { party_index },
        ))
    }

    pub fn discard_party_mail_to_bag(&mut self, party_index: usize) -> Result<RuntimeMailTransfer> {
        self.apply_mail_mutation(RuntimeMutationCommand::DiscardPartyMailToBag(
            RuntimePartySlotCommand { party_index },
        ))
    }

    pub fn delete_mailbox_mail(&mut self, mailbox_index: usize) -> Result<RuntimeMailTransfer> {
        self.apply_mail_mutation(RuntimeMutationCommand::DeleteMailboxMail(
            RuntimeMailboxSlotCommand { mailbox_index },
        ))
    }

    pub fn move_mailbox_mail_to_bag(
        &mut self,
        mailbox_index: usize,
    ) -> Result<RuntimeMailTransfer> {
        self.apply_mail_mutation(RuntimeMutationCommand::MoveMailboxMailToBag(
            RuntimeMailboxSlotCommand { mailbox_index },
        ))
    }

    pub fn attach_mailbox_mail_to_party(
        &mut self,
        mailbox_index: usize,
        party_index: usize,
    ) -> Result<RuntimeMailTransfer> {
        self.apply_mail_mutation(RuntimeMutationCommand::AttachMailboxMailToParty(
            RuntimeMailboxPartyCommand {
                mailbox_index,
                party_index,
            },
        ))
    }

    fn apply_mail_mutation(
        &mut self,
        command: RuntimeMutationCommand,
    ) -> Result<RuntimeMailTransfer> {
        let mutation = self.apply_runtime_mutation_command(command)?;
        let outcome = match mutation.result {
            RuntimeMutationResult::PartyMailComposed(outcome)
            | RuntimeMutationResult::PartyMailSentToMailbox(outcome)
            | RuntimeMutationResult::PartyMailDiscardedToBag(outcome)
            | RuntimeMutationResult::MailboxMailDeleted(outcome) => outcome,
            RuntimeMutationResult::MailboxMailMovedToBag(outcome)
            | RuntimeMutationResult::MailboxMailAttachedToParty(outcome) => outcome,
            _ => anyhow::bail!("runtime mutation returned non-Mail-transfer result"),
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Mail transfer")?;
        Ok(RuntimeMailTransfer {
            party_index: outcome.party_index,
            mailbox_index: outcome.mailbox_index,
            item_id: outcome.item_id,
            mail: outcome.mail,
            mailbox_count_after: outcome.mailbox_count_after,
            bag_quantity_after: outcome.bag_quantity_after,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn award_badge(
        &mut self,
        region: RuntimeBadgeRegion,
        index: usize,
    ) -> Result<RuntimeBadgeAward> {
        let mutation = self.apply_runtime_mutation_command(RuntimeMutationCommand::AwardBadge(
            RuntimeBadgeCommand { region, index },
        ))?;
        let RuntimeMutationResult::BadgeAwarded(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-badge-award result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after badge award")?;
        Ok(RuntimeBadgeAward {
            region: outcome.region,
            index: outcome.index,
            already_awarded: outcome.already_awarded,
            awarded_count_after: outcome.awarded_count_after,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn record_pokedex_seen(&mut self, species_id: &str) -> Result<RuntimePokedexRecord> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::RecordPokedexSeen(RuntimePokedexCommand {
                species_id: species_id.to_string(),
            }),
        )?;
        let RuntimeMutationResult::PokedexSeenRecorded(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Pokedex-seen result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Pokedex seen record")?;
        Ok(RuntimePokedexRecord {
            species_id: outcome.species_id,
            already_seen: outcome.already_seen,
            already_caught: outcome.already_caught,
            seen_count_after: outcome.seen_count_after,
            caught_count_after: outcome.caught_count_after,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn record_pokedex_caught(&mut self, species_id: &str) -> Result<RuntimePokedexRecord> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::RecordPokedexCaught(RuntimePokedexCommand {
                species_id: species_id.to_string(),
            }),
        )?;
        let RuntimeMutationResult::PokedexCaughtRecorded(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Pokedex-caught result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Pokedex caught record")?;
        Ok(RuntimePokedexRecord {
            species_id: outcome.species_id,
            already_seen: outcome.already_seen,
            already_caught: outcome.already_caught,
            seen_count_after: outcome.seen_count_after,
            caught_count_after: outcome.caught_count_after,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn add_currency(
        &mut self,
        account: RuntimeCurrencyAccount,
        amount: u32,
    ) -> Result<RuntimeCurrencyMutation> {
        let mutation = self.apply_runtime_mutation_command(RuntimeMutationCommand::AddCurrency(
            RuntimeCurrencyDeltaCommand { account, amount },
        ))?;
        let RuntimeMutationResult::CurrencyAdded(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-currency-add result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after currency add")?;
        Ok(RuntimeCurrencyMutation {
            account: outcome.account,
            amount: outcome.amount,
            value_before: outcome.value_before,
            value_after: outcome.value_after,
            cap: outcome.cap,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn take_currency(
        &mut self,
        account: RuntimeCurrencyAccount,
        amount: u32,
    ) -> Result<RuntimeCurrencyMutation> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::TakeCurrency(RuntimeCurrencyDeltaCommand { account, amount }),
        )?;
        let RuntimeMutationResult::CurrencyTaken(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-currency-take result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after currency take")?;
        Ok(RuntimeCurrencyMutation {
            account: outcome.account,
            amount: outcome.amount,
            value_before: outcome.value_before,
            value_after: outcome.value_after,
            cap: outcome.cap,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn add_bag_item(&mut self, item_id: &str, quantity: u16) -> Result<RuntimeBagItemMutation> {
        let mutation = self.apply_runtime_mutation_command(RuntimeMutationCommand::AddBagItem(
            RuntimeBagItemDeltaCommand {
                item_id: item_id.to_string(),
                quantity,
            },
        ))?;
        let RuntimeMutationResult::BagItemAdded(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-bag-item-add result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after bag item add")?;
        Ok(RuntimeBagItemMutation {
            item_id: outcome.item_id,
            quantity: outcome.quantity,
            added: outcome.added,
            quantity_before: outcome.quantity_before,
            quantity_after: outcome.quantity_after,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn remove_bag_item(
        &mut self,
        item_id: &str,
        quantity: u16,
    ) -> Result<RuntimeBagItemMutation> {
        let item = self
            .runtime
            .data
            .items
            .get(item_id)
            .cloned()
            .with_context(|| format!("unknown bag item {item_id}"))?;
        let quantity_before = self.session.state.bag.quantity(&item);
        let removed = self
            .session
            .state
            .bag
            .remove_item(&item, quantity)
            .map_err(anyhow::Error::msg)?;
        if !removed {
            anyhow::bail!("bag does not contain a single {item_id} stack with quantity {quantity}");
        }
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after bag item remove")?;
        let item_id = item.script_name.clone();
        Ok(RuntimeBagItemMutation {
            item_id,
            quantity,
            added: false,
            quantity_before,
            quantity_after: self.session.state.bag.quantity(&item),
            state_checksum: game_state_checksum(&self.session.state)?,
        })
    }

    pub fn switch_pc_item_stacks(&mut self, source_index: usize, target_index: usize) -> Result<()> {
        self.session.state.bag.switch_pc_item_stacks(source_index, target_index).map_err(anyhow::Error::msg)?;
        self.runtime.validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after PC item-stack switch")
    }

    pub fn switch_bag_item_stacks(
        &mut self,
        pocket: &str,
        source_index: usize,
        target_index: usize,
    ) -> Result<usize> {
        let target_index = self
            .session
            .state
            .bag
            .switch_item_stacks(pocket, source_index, target_index)
            .map_err(anyhow::Error::msg)?;
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after item-stack switch")?;
        Ok(target_index)
    }

    pub fn record_link_battle_result(
        &mut self,
        result: RuntimeLinkBattleResult,
    ) -> Result<RuntimeLinkBattleRecord> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::RecordLinkBattleResult(
                RuntimeLinkBattleRecordCommand { result },
            ))?;
        let RuntimeMutationResult::LinkBattleResultRecorded(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-link-battle-record result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after link battle record")?;
        Ok(RuntimeLinkBattleRecord {
            result: outcome.result,
            wins_after: outcome.wins_after,
            losses_after: outcome.losses_after,
            draws_after: outcome.draws_after,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn set_cable_club_request(
        &mut self,
        request: RuntimeCableClubRequest,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self
            .apply_runtime_mutation_command(RuntimeMutationCommand::SetCableClubRequest(request))?;
        let RuntimeMutationResult::CableClubRequestSet(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-cable-club-request result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Cable Club request")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn wait_for_linked_friend_special(
        &mut self,
        serial_connection_status: LinkSerialConnectionStatus,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::WaitForLinkedFriendSpecial(RuntimeLinkFriendReadyCommand {
                serial_connection_status,
            }),
        )?;
        let RuntimeMutationResult::LinkedFriendWaitedFor(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-linked-friend-wait result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after linked friend wait")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn check_link_timeout_receptionist_special(
        &mut self,
        other_player_link_mode: u8,
        serial_connection_status: LinkSerialConnectionStatus,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::CheckLinkTimeoutReceptionistSpecial(
                RuntimeLinkTimeoutCommand {
                    other_player_link_mode,
                    serial_connection_status,
                },
            ),
        )?;
        let RuntimeMutationResult::LinkTimeoutReceptionistChecked(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-link-timeout-result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after link timeout receptionist check")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn check_both_selected_same_room_special(
        &mut self,
        other_player_room: u8,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::CheckBothSelectedSameRoomSpecial(
                RuntimeLinkRoomSelectionCommand { other_player_room },
            ),
        )?;
        let RuntimeMutationResult::BothSelectedSameRoomChecked(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-same-room-result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after same-room check")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn close_link_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::CloseLinkSpecial)?;
        let RuntimeMutationResult::LinkClosed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-link-close result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after link close")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn wait_for_other_player_to_exit_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::WaitForOtherPlayerToExitSpecial,
        )?;
        let RuntimeMutationResult::OtherPlayerExitWaitedFor(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-other-player-exit result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after other player exit wait")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn failed_link_to_past_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::FailedLinkToPastSpecial)?;
        let RuntimeMutationResult::LinkToPastFailed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-failed-link-to-past result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after failed link to past")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn open_link_room_special(
        &mut self,
        room: RuntimeLinkRoomSpecial,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::OpenLinkRoomSpecial(room))?;
        let RuntimeMutationResult::LinkRoomOpened(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-link-room result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after link room")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn check_time_capsule_compatibility_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::CheckTimeCapsuleCompatibilitySpecial,
        )?;
        let RuntimeMutationResult::TimeCapsuleCompatibilityChecked(outcome) = mutation.result
        else {
            anyhow::bail!("runtime mutation returned non-time-capsule-compatibility result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Time Capsule compatibility check")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn try_quick_save_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::TryQuickSaveSpecial)?;
        let RuntimeMutationResult::QuickSaveTried(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-quick-save result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after quick save special")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn ask_mobile_or_cable_special(&mut self) -> Result<RuntimeSpecialRoutineUse> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::AskMobileOrCableSpecial)?;
        let RuntimeMutationResult::MobileOrCableAsked(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-mobile-or-cable result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after mobile/cable prompt")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn cable_club_check_which_chris_special(
        &mut self,
        gender: String,
    ) -> Result<RuntimeSpecialRoutineUse> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::CableClubCheckWhichChrisSpecial(
                RuntimeCableClubGenderCommand { gender },
            ),
        )?;
        let RuntimeMutationResult::CableClubChrisChecked(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-Cable-Club-Chris result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after Cable Club Chris check")?;
        Ok(RuntimeSpecialRoutineUse {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn set_options(&mut self, options: Options) -> Result<RuntimeOptionsSet> {
        let mutation = self.apply_runtime_mutation_command(RuntimeMutationCommand::SetOptions(
            RuntimeOptionsCommand { options },
        ))?;
        let RuntimeMutationResult::OptionsSet(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-options-set result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after options set")?;
        Ok(RuntimeOptionsSet {
            options_before: outcome.options_before,
            options_after: outcome.options_after,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn set_trainer_identity(
        &mut self,
        player_name: impl Into<String>,
        player_id: u16,
    ) -> Result<RuntimeTrainerIdentitySet> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::SetTrainerIdentity(RuntimeTrainerIdentityCommand {
                player_name: player_name.into(),
                player_id,
            }),
        )?;
        let RuntimeMutationResult::TrainerIdentitySet(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-trainer-identity result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after trainer identity set")?;
        Ok(RuntimeTrainerIdentitySet {
            player_name_before: outcome.player_name_before,
            player_id_before: outcome.player_id_before,
            player_name_after: outcome.player_name_after,
            player_id_after: outcome.player_id_after,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn set_player_gender(&mut self, player_gender: u8) -> Result<RuntimePlayerGenderSet> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::SetPlayerGender(RuntimePlayerGenderCommand { player_gender }),
        )?;
        let RuntimeMutationResult::PlayerGenderSet(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-player-gender result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after player gender set")?;
        Ok(RuntimePlayerGenderSet {
            player_gender_before: outcome.player_gender_before,
            player_gender_after: outcome.player_gender_after,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn rename_party_pokemon(
        &mut self,
        party_index: usize,
        nickname: impl Into<String>,
    ) -> Result<RuntimePartyNicknameSet> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::RenamePartyPokemon(RuntimePartyNicknameCommand {
                party_index,
                nickname: nickname.into(),
            }),
        )?;
        let RuntimeMutationResult::PartyPokemonRenamed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-party-nickname result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after party nickname set")?;
        Ok(RuntimePartyNicknameSet {
            party_index: outcome.party_index,
            species_id: outcome.species_id,
            nickname_before: outcome.nickname_before,
            nickname_after: outcome.nickname_after,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn set_party_pokemon_recovery_state(
        &mut self,
        party_index: usize,
        hp: u16,
        status: Option<String>,
        first_move_pp: Option<u8>,
    ) -> Result<RuntimePartyRecoveryStateSet> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::SetPartyPokemonRecoveryState(
                RuntimePartyRecoverySetupCommand {
                    party_index,
                    hp,
                    status,
                    first_move_pp,
                },
            ),
        )?;
        let RuntimeMutationResult::PartyPokemonRecoveryStateSet(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-party-recovery-setup result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after party recovery setup")?;
        Ok(RuntimePartyRecoveryStateSet {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn full_heal_party_pokemon(&mut self, party_index: usize) -> Result<RuntimePartyRecovery> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::FullHealPartyPokemon(RuntimePartySlotCommand { party_index }),
        )?;
        let RuntimeMutationResult::PartyPokemonFullHealed(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-party-recovery result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after party Pokemon recovery")?;
        Ok(runtime_party_recovery(outcome, mutation.state_checksum))
    }

    pub fn transfer_party_pokemon_hp(
        &mut self,
        source_party_index: usize,
        target_party_index: usize,
    ) -> Result<RuntimePartyHpTransfer> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::TransferPartyPokemonHp(RuntimePartyHpTransferCommand {
                source_party_index,
                target_party_index,
            }),
        )?;
        let RuntimeMutationResult::PartyPokemonHpTransferred(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-party-HP-transfer result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after party HP transfer")?;
        Ok(RuntimePartyHpTransfer {
            outcome,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn full_heal_whole_party(&mut self) -> Result<Vec<RuntimePartyRecovery>> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::FullHealWholeParty)?;
        let RuntimeMutationResult::WholePartyFullHealed(outcomes) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-whole-party-recovery result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after whole-party recovery")?;
        Ok(outcomes
            .into_iter()
            .map(|outcome| runtime_party_recovery(outcome, mutation.state_checksum.clone()))
            .collect())
    }

    pub fn replace_pending_move_learn(
        &mut self,
        move_slot: usize,
    ) -> Result<RuntimePendingMoveLearnResolution> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::ReplacePendingMoveLearn(
                RuntimeMoveLearnReplacementCommand { move_slot },
            ))?;
        let RuntimeMutationResult::PendingMoveLearnReplaced(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-pending-move-learn-replacement result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after pending move learn replacement")?;
        Ok(RuntimePendingMoveLearnResolution {
            resolution: outcome.resolution,
            deferred_evolution: outcome.deferred_evolution,
        })
    }

    pub fn decline_pending_move_learn(&mut self) -> Result<RuntimePendingMoveLearnResolution> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::DeclinePendingMoveLearn)?;
        let RuntimeMutationResult::PendingMoveLearnDeclined(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-pending-move-learn-decline result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after pending move learn decline")?;
        Ok(RuntimePendingMoveLearnResolution {
            resolution: outcome.resolution,
            deferred_evolution: outcome.deferred_evolution,
        })
    }

    /// Commit ExitBattle's loss cleanup before presenting whiteout recovery.
    pub fn complete_battle_loss(&mut self) -> Result<()> {
        self.apply_runtime_mutation_command(RuntimeMutationCommand::CompleteBattleLoss)?;
        Ok(())
    }

    pub fn resolve_blackout_to_last_spawn(&mut self) -> Result<RuntimeBlackoutRecovery> {
        let mutation = self
            .apply_runtime_mutation_command(RuntimeMutationCommand::ResolveBlackoutToLastSpawn)?;
        let RuntimeMutationResult::BlackoutResolved(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-blackout-recovery result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after blackout recovery")?;
        let state = self.session.state.clone();
        self.session = self
            .runtime
            .resume_overworld_session(&self.asset_root, state)
            .context("resume overworld after blackout recovery")?;
        self.last_frame = None;
        Ok(runtime_blackout_recovery(outcome, mutation.state_checksum))
    }

    pub fn swap_party_pokemon(
        &mut self,
        first_party_index: usize,
        second_party_index: usize,
    ) -> Result<RuntimePartySwap> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::SwapPartyPokemon(RuntimePartySwapCommand {
                first_party_index,
                second_party_index,
            }),
        )?;
        let RuntimeMutationResult::PartyPokemonSwapped(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-party-swap result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after party swap")?;
        Ok(RuntimePartySwap {
            first_party_index: outcome.first_party_index,
            second_party_index: outcome.second_party_index,
            first_species_after: outcome.first_species_after,
            second_species_after: outcome.second_species_after,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn swap_party_pokemon_moves(
        &mut self,
        party_index: usize,
        first_move_index: usize,
        second_move_index: usize,
    ) -> Result<RuntimePartyMoveSwap> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::SwapPartyPokemonMoves(RuntimePartyMoveSwapCommand {
                party_index,
                first_move_index,
                second_move_index,
            }),
        )?;
        let RuntimeMutationResult::PartyPokemonMovesSwapped(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-party-move-swap result");
        };
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after party move swap")?;
        Ok(RuntimePartyMoveSwap {
            party_index: outcome.party_index,
            first_move_index: outcome.first_move_index,
            second_move_index: outcome.second_move_index,
            first_move_after: outcome.first_move_after,
            second_move_after: outcome.second_move_after,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn save(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let paused_before = self.session.state.game_logic_paused;
        self.set_game_logic_paused(true)?;
        let save_result = self
            .runtime
            .save_game(path, self.session.state.clone())
            .context("save runtime game shell state");
        let restore_result = self.set_game_logic_paused(paused_before);
        match (save_result, restore_result) {
            (Ok(()), Ok(_)) => Ok(()),
            (Err(error), Ok(_)) => Err(error),
            (Ok(()), Err(error)) => Err(error).context("resume game logic after save"),
            (Err(save_error), Err(restore_error)) => Err(save_error).context(format!(
                "also failed to resume game logic after save: {restore_error:#}"
            )),
        }
    }

    pub fn load(&mut self, path: impl AsRef<Path>) -> Result<()> {
        // Continue loads SRAM-backed WRAM while the title process remains
        // alive. hRandomAdd/hRandomSub, hVBlankCounter, and the hardware DIV
        // source are HRAM/hardware state and therefore must not come from the
        // save payload or restart at this boundary.
        let random_state = self.session.state.random_state;
        let vblank_counter = self.session.state.vblank_counter;
        let divider = self.session.divider.clone();
        let mut state = self.runtime.load_save(path)?;
        state.random_state = random_state;
        state.vblank_counter = vblank_counter;
        state.set_game_timer_counting(true);
        state.set_game_logic_paused(false);
        let mut session = self
            .runtime
            .resume_overworld_session(&self.asset_root, state)
            .context("load runtime game shell state")?;
        session.divider = divider;
        self.session = session;
        self.last_frame = None;
        self.linked_menu_results.clear();
        self.clear_retained_runtime_commands();
        Ok(())
    }

    pub fn snapshot(&self) -> Result<RuntimeShellSnapshot> {
        self.snapshot_with_integrity(true)
    }

    /// Build the read-only shell view used by the real-time renderer.
    ///
    /// Save validation and deterministic whole-state checksums belong at
    /// persistence, replay, and network boundaries. Running both for every
    /// LCD update cloned and walked the complete game state twice and made a
    /// presentation-only text change capable of missing multiple frames.
    pub fn presentation_snapshot(&self) -> Result<RuntimeShellSnapshot> {
        self.snapshot_with_integrity(false)
    }

    fn snapshot_with_integrity(&self, integrity: bool) -> Result<RuntimeShellSnapshot> {
        #[cfg(feature = "operation-trace")]
        let _span = tracing::info_span!("crystal_snapshot", integrity).entered();
        let (state_checksum, visual_state_hash) = if integrity {
            self.runtime
                .validate_save_state_for_runtime_pack(&self.session.state)
                .context("validate runtime game shell state before snapshot")?;
            let state_checksum = game_state_checksum(&self.session.state)
                .context("checksum runtime game shell state")?;
            let mut visual_state = self.session.state.clone();
            visual_state.frame_counter = 0;
            let visual_state_hash = game_state_checksum(&visual_state)
                .context("checksum render-relevant runtime game state")?
                .hash();
            (state_checksum, visual_state_hash)
        } else {
            // Presentation keys are maintained by BevyRuntimeShell's explicit
            // revision and render-key fields. Keep only the authoritative
            // frame number for UI animation code; no state checksum is
            // computed on this path.
            (StateChecksum::new(self.session.state.frame_counter, 0), 0)
        };
        let menu = self.runtime.active_menu_snapshot(&self.session.state)?;
        let ui = self
            .runtime
            .ui_snapshot(&self.session.state, menu.clone())?;
        let catalogs = self.runtime.static_catalog_cache();
        let visible_object_entries = self
            .session
            .overworld
            .objects
            .iter()
            .enumerate()
            .filter(|(index, _)| self.session.overworld.object_struct_is_visible(*index))
            .collect::<Vec<_>>();
        let mut visible_object_runtime_tiles = BTreeMap::new();
        let mut visible_object_facings = BTreeMap::new();
        for (index, object) in &visible_object_entries {
            let Some(object_id) = object.object_identifier.as_ref() else {
                continue;
            };
            let tile = self
                .session
                .overworld
                .object_runtime_tile_checked(*index, object)
                .with_context(|| {
                    format!("resolve loaded object {object_id} coordinates for runtime snapshot")
                })?;
            let facing = self
                .session
                .overworld
                .object_facings
                .get(object_id)
                .copied()
                .with_context(|| {
                    format!("loaded object {object_id} is missing its source-initialized facing")
                })?;
            visible_object_runtime_tiles.insert(object_id.clone(), tile);
            visible_object_facings.insert(object_id.clone(), facing);
        }
        Ok(RuntimeShellSnapshot {
            boot: self.runtime.boot_summary(),
            overworld: self.session.snapshot(),
            overworld_player_hidden: self.session.overworld.player_hidden,
            visible_objects: visible_object_entries
                .iter()
                .map(|(_, object)| (*object).clone())
                .collect(),
            visible_object_slots: visible_object_entries
                .iter()
                .map(|(index, _)| *index)
                .collect(),
            visible_object_runtime_tiles,
            visible_object_facings,
            state_checksum,
            visual_state_hash,
            phase: RuntimeShellPhase::from_state(&self.session.state),
            trainer: RuntimeTrainerSnapshot::from_state(&self.session.state),
            progression: RuntimeProgressionSnapshot::from_state(&self.session.state),
            roaming_pokemon: self.session.state.roaming_pokemon.clone(),
            map_name_sign: self.session.state.map_name_sign,
            day_care: self.session.state.day_care.clone(),
            bug_contest: self.session.state.bug_contest.clone(),
            magikarp_record: self.session.state.magikarp_record.clone(),
            buenas_password: self.session.state.buenas_password.clone(),
            mystery_gift: RuntimeMysteryGiftSnapshot {
                unlocked: self.session.state.mystery_gift_unlocked,
                stored_item: self.session.state.mystery_gift.stored_item.clone(),
                backup_item: self.session.state.mystery_gift.backup_item.clone(),
                trainer_house_flag: self.session.state.mystery_gift.trainer_house_flag,
            },
            link_session: RuntimeLinkSessionSnapshot::from_state(&self.session.state),
            battle_tower: self.session.state.battle_tower.clone(),
            mobile_link: self.session.state.mobile_link.clone(),
            audio: RuntimeShellAudioState::from_state(&self.session.state),
            audio_catalog: Arc::clone(&catalogs.audio),
            menu,
            ui,
            battle: if battle_tower_opponent_is_staged(&self.session.state) {
                None
            } else {
                RuntimeBattleSnapshot::from_state(&self.session.state)?
            },
            pending_move_learn: RuntimePendingMoveLearnSnapshot::from_state(&self.session.state),
            party: RuntimePartySnapshot::from_state(&self.session.state),
            storage: RuntimeStorageSnapshot::from_state(&self.session.state),
            mailbox: self.session.state.mailbox.clone(),
            bag: self.runtime.bag_snapshot(&self.session.state)?,
            items: Arc::clone(&catalogs.items),
            item_effect_plans: Arc::clone(&catalogs.item_effect_plans),
            moves: Arc::clone(&catalogs.moves),
            pokemon: Arc::clone(&catalogs.pokemon),
            trainers: Arc::clone(&catalogs.trainers),
            maps: self
                .runtime
                .map_catalog_snapshot(&self.session.overworld.map, &self.session.state),
            spawn_points: Arc::clone(&catalogs.spawn_points),
            tilesets: Arc::clone(&catalogs.tilesets),
            encounters: Arc::clone(&catalogs.encounters),
            battle_rules: Arc::clone(&catalogs.battle_rules),
            world_rules: Arc::clone(&catalogs.world_rules),
            presentation: Arc::clone(&catalogs.presentation),
            special: Arc::clone(&catalogs.special),
            story: Arc::clone(&catalogs.story),
            playability: Arc::clone(&catalogs.playability),
            script_events: RuntimeScriptEventsSnapshot::from_state(&self.session.state),
            pending_shop: self.session.state.script_runtime.pending_shop.clone(),
            linked_menu_results: self.linked_menu_results.clone(),
        })
    }

    pub fn text_snapshot(&self, label: &str) -> Result<RuntimeTextSnapshot> {
        self.runtime
            .text_snapshot_for_label(&self.session.state, label)
            .with_context(|| format!("resolve runtime game shell text '{label}'"))
    }

    pub fn state_checksum(&self) -> Result<StateChecksum> {
        game_state_checksum(&self.session.state).context("checksum runtime game shell state")
    }

    pub fn set_script_flag_for_smoke(&mut self, flag_id: &str) -> Result<StateChecksum> {
        if is_engine_flag_name(flag_id) {
            self.runtime.require_engine_flag(flag_id)?;
        } else {
            self.runtime.require_event_flag(flag_id)?;
        }
        self.session
            .state
            .flags
            .set_script_flag(flag_id, true)
            .with_context(|| format!("set smoke script flag {flag_id}"))?;
        self.state_checksum()
    }

    pub fn set_blue_card_balance_for_smoke(&mut self, balance: u8) -> Result<StateChecksum> {
        self.session.state.blue_card_balance = balance;
        self.runtime
            .validate_save_state_for_runtime_pack(&self.session.state)
            .context("validate runtime state after smoke Blue Card balance seed")?;
        self.state_checksum()
    }

    pub fn current_map_name(&self) -> &str {
        &self.session.overworld.map.name
    }

    pub fn script_events_snapshot(&self) -> RuntimeScriptEventsSnapshot {
        RuntimeScriptEventsSnapshot::from_state(&self.session.state)
    }

    pub fn owned_decoration_categories(&self) -> Result<Vec<DecorationCategory>> {
        self.runtime
            .data
            .owned_decoration_categories(&self.session.state)
    }

    pub fn owned_decorations(
        &self,
        category: DecorationCategory,
    ) -> Result<Vec<DecorationDefinition>> {
        Ok(self
            .runtime
            .data
            .owned_decorations(&self.session.state, category)?
            .into_iter()
            .cloned()
            .collect())
    }

    pub fn set_up_decoration(
        &mut self,
        decoration_id: &str,
        side: Option<DecorationSide>,
    ) -> Result<DecorationActionOutcome> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::SetUpDecoration(RuntimeDecorationSetupCommand {
                decoration_id: decoration_id.to_string(),
                side,
            }),
        )?;
        let RuntimeMutationResult::DecorationSetUp(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-decoration-setup result");
        };
        Ok(outcome)
    }

    pub fn put_away_decoration(
        &mut self,
        category: DecorationCategory,
        side: Option<DecorationSide>,
    ) -> Result<DecorationActionOutcome> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::PutAwayDecoration(
                RuntimeDecorationPutAwayCommand { category, side },
            ))?;
        let RuntimeMutationResult::DecorationPutAway(outcome) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-decoration-put-away result");
        };
        Ok(outcome)
    }

    /// Cheap predicate for the host frame loop.  Unlike `snapshot()` this
    /// does not clone the semantic state or calculate a checksum, so an idle
    /// overworld frame can stay on the fast path.
    pub fn has_pending_script_work(&self) -> bool {
        self.pending_script_work_reason().is_some()
    }

    /// Identify the first authoritative script condition that still owns a
    /// frame. This is intentionally allocation-free so input diagnostics can
    /// report a precise capture reason without cloning a runtime snapshot.
    pub fn pending_script_work_reason(&self) -> Option<&'static str> {
        let script = &self.session.state.script_runtime;
        Some(if script.pending_text_label.is_some() {
            "pending_text_label"
        } else if script.pending_text_wait.is_some() {
            "pending_text_wait"
        } else if script.pending_yes_no.is_some() {
            "pending_yes_no"
        } else if script.pending_shop.is_some() {
            "pending_shop"
        } else if script.active_pokemon_picture.is_some() {
            "active_pokemon_picture"
        } else if script.window_open {
            "window_open"
        } else if script.text_window_open {
            "text_window_open"
        } else if script.pending_map_load.is_some() {
            "pending_map_load"
        } else if script.pending_map_refresh.is_some() {
            "pending_map_refresh"
        } else if script.pending_music_fade.is_some() {
            "pending_music_fade"
        } else if script.pending_screen_fade.is_some() {
            "pending_screen_fade"
        } else if !script.pending_delays.is_empty() {
            "pending_delays"
        } else if !script.pending_earthquakes.is_empty() {
            "pending_earthquakes"
        } else if !script.pending_emotes.is_empty() {
            "pending_emotes"
        } else if script.pending_script_warp.is_some() {
            "pending_script_warp"
        } else if !script.command_queue.is_empty() {
            "command_queue"
        } else if script.next_script.is_some() {
            "next_script"
        } else if !script.deferred_scripts.is_empty() {
            "deferred_scripts"
        } else if script.map_reentry_script.is_some() {
            "map_reentry_script"
        } else if script.script_ended.is_some() {
            "script_ended"
        } else if !script.audio_events.is_empty() {
            "audio_events"
        } else if !script.graphics_events.is_empty() {
            "graphics_events"
        } else if !script.money_events.is_empty() {
            "money_events"
        } else if !script.map_events.is_empty() {
            "map_events"
        }
        // Text events are retained execution history, not pending work.
        else if !script.control_events.is_empty() {
            "control_events"
        } else if !script.shop_events.is_empty() {
            "shop_events"
        } else if !script.item_use_events.is_empty() {
            "item_use_events"
        } else if script.active_menu.is_some() {
            "active_menu"
        } else if script.waiting_for_sound_effect {
            "waiting_for_sound_effect"
        } else if script.player_input_locked {
            "player_input_locked"
        } else if script.all_input_locked {
            "all_input_locked"
        } else if script.script_stop_requested {
            "script_stop_requested"
        } else {
            return None;
        })
    }

    /// Avoid entering the transactional audio-queue mutation path when the
    /// game has not emitted a sound event.
    pub fn has_pending_audio_events(&self) -> bool {
        !self.session.state.script_runtime.audio_events.is_empty()
    }

    pub fn current_music_id(&self) -> Option<&str> {
        self.session.state.script_runtime.current_music.as_deref()
    }

    pub fn has_pending_music_fade(&self) -> bool {
        self.session
            .state
            .script_runtime
            .pending_music_fade
            .is_some()
    }

    pub fn has_active_bug_contest_timer(&self) -> bool {
        self.session.state.bug_contest.timer_active
    }

    pub fn audio_state(&self) -> RuntimeShellAudioState {
        RuntimeShellAudioState::from_state(&self.session.state)
    }

    pub fn has_active_battle(&self) -> bool {
        self.session.state.battle_active_party_index.is_some()
    }

    pub fn runtime(&self) -> &CrystalRuntime {
        &self.runtime
    }

    pub fn facing_tile_collision_permission(&self) -> anyhow::Result<Option<u8>> {
        let target = crystal_core::world::movement::checked_move_by_stride(
            self.session.overworld.player.tile,
            self.session.overworld.player.facing,
            crystal_core::world::movement::StepOptions::default().stride_tiles,
        )
        .with_context(|| {
            format!(
                "facing tile overflows runtime coordinates from ({}, {}) facing {:?}",
                self.session.overworld.player.tile.x,
                self.session.overworld.player.tile.y,
                self.session.overworld.player.facing
            )
        })?;
        Ok(crystal_core::world::collision::sample_collision(
            &self.session.overworld.map,
            &self.session.overworld.tileset,
            target,
        )
        .map(|sample| sample.permission))
    }

    /// Report whether the current facing prevents the contextual Surf action.
    pub fn contextual_surf_direction_is_blocked(&self) -> anyhow::Result<bool> {
        let player = &self.session.overworld.player;
        let target = crystal_core::world::movement::checked_move_by_stride(
            player.tile,
            player.facing,
            crystal_core::world::movement::StepOptions::default().stride_tiles,
        )
        .with_context(|| {
            format!(
                "Surf direction overflows runtime coordinates from ({}, {}) facing {:?}",
                player.tile.x, player.tile.y, player.facing
            )
        })?;
        let current = crystal_core::world::collision::sample_collision(
            &self.session.overworld.map,
            &self.session.overworld.tileset,
            player.tile,
        )
        .with_context(|| {
            format!(
                "Surf player tile {:?} is outside map {}",
                player.tile, self.session.overworld.map.name
            )
        })?;
        let target = crystal_core::world::collision::sample_collision(
            &self.session.overworld.map,
            &self.session.overworld.tileset,
            target,
        )
        .with_context(|| {
            format!(
                "Surf facing tile {target:?} is outside map {}",
                self.session.overworld.map.name
            )
        })?;
        Ok(
            crystal_core::world::collision::is_direction_blocked_leaving(
                current.permission,
                player.facing,
            ) || crystal_core::world::collision::is_direction_blocked(
                target.permission,
                player.facing,
            ),
        )
    }

    pub fn current_overworld_interaction(&self) -> Option<OverworldInteraction> {
        self.current_overworld_interaction_checked()
            .expect("current overworld interaction coordinates must be valid")
    }

    pub fn current_overworld_interaction_checked(&self) -> Result<Option<OverworldInteraction>> {
        let candidate = self
            .session
            .overworld
            .check_interaction_checked(
                crystal_core::world::movement::StepOptions::default().stride_tiles,
            )
            .with_context(|| {
                format!(
                    "check current overworld interaction on {}",
                    self.session.overworld.map.name
                )
            })?;
        candidate
            .as_ref()
            .map(|candidate| {
                self.runtime
                    .data
                    .resolve_overworld_interaction(&self.session.state, candidate)
            })
            .transpose()
            .map(Option::flatten)
    }

    pub fn current_scene_script(&self) -> Result<Option<RuntimeCurrentSceneScript>> {
        let map_name = self.session.snapshot().map_name;
        let Some(module) = self.runtime.data.maps.get(&map_name) else {
            anyhow::bail!("current map {map_name} missing from runtime map catalog");
        };
        if module.scenes.scenes.is_empty() {
            return Ok(None);
        }
        let scene_memory = &self.session.state.scenes;
        let scene_id = scene_memory.map_scenes.get(&map_name);
        let Some(scene_id) = scene_id else {
            return Ok(None);
        };
        let scene = module
            .scenes
            .scenes
            .iter()
            .find(|scene| scene.scene_id == *scene_id)
            .with_context(|| {
                format!("current scene {map_name}:{scene_id} missing from compiled map")
            })?;
        Ok(Some(RuntimeCurrentSceneScript {
            map_name,
            scene_id: scene.scene_id.clone(),
            script_name: scene.script_name.clone(),
        }))
    }

    pub fn session(&self) -> &RuntimeOverworldSession {
        &self.session
    }

    pub fn session_mut(&mut self) -> &mut RuntimeOverworldSession {
        &mut self.session
    }

    pub fn apply_link_trade_outcome(
        &mut self,
        outcome: &TradeOutcome,
        player_id: PlayerId,
        transfer_mode: LinkTradeTransferMode,
    ) -> Result<RuntimeLinkTradeApply> {
        anyhow::ensure!(
            !outcome.cancelled(),
            "cancelled link trade has no party outcome"
        );
        let received_party_index = self
            .session
            .state
            .storage
            .party
            .filled_slots()
            .checked_sub(1)
            .context("link trade requires a nonempty party")?;
        let sent = outcome
            .apply_to_party_for_mode(
                player_id,
                &mut self.session.state.storage.party,
                transfer_mode,
            )?
            .context("completed link trade did not return the sent Pokemon")?;
        let received_before_evolution = self.session.state.storage.party.pokemon
            [received_party_index]
            .as_ref()
            .context("received link-trade Pokemon is absent from the appended party slot")?
            .clone();
        if !received_before_evolution.is_egg {
            self.session
                .state
                .pokedex
                .record_caught_pokemon(&received_before_evolution);
        }
        let link_mode = match transfer_mode {
            LinkTradeTransferMode::TradeCenter => LinkMode::Link,
            LinkTradeTransferMode::TimeCapsule => LinkMode::TimeCapsule,
        };
        let context = EvolutionContext {
            species: &self.runtime.data.pokemon,
            moves: &self.runtime.data.moves,
            learnsets: &self.runtime.data.learnsets,
            time_of_day: self.session.state.time.time_of_day,
            current_item: None,
            force_evolution: true,
            link_mode,
        };
        let evolution = {
            let received = self.session.state.storage.party.pokemon[received_party_index]
                .as_mut()
                .context("received link-trade Pokemon is absent from the appended party slot")?;
            check_and_evolve(received, &self.runtime.data.evolutions, &context, false)
                .map_err(|error| anyhow::anyhow!("evolve received link-trade Pokemon: {error:?}"))?
        };
        if evolution.target_species.is_some() {
            let evolved_pokemon = self.session.state.storage.party.pokemon[received_party_index]
                .as_ref()
                .context("evolved link-trade Pokemon disappeared")?
                .clone();
            self.session
                .state
                .pokedex
                .record_caught_pokemon(&evolved_pokemon);
        }
        if !evolution.pending_move_learns.is_empty() {
            anyhow::ensure!(
                self.session.state.pending_move_learn.is_none()
                    && self.session.state.pending_move_learn_queue.is_empty(),
                "link-trade evolution cannot replace an existing pending move learn"
            );
            let received = self.session.state.storage.party.pokemon[received_party_index]
                .as_ref()
                .context("evolved link-trade Pokemon disappeared")?;
            let pending = evolution
                .pending_move_learns
                .iter()
                .cloned()
                .map(|learned_move| PendingMoveLearn {
                    party_index: received_party_index,
                    species_id: received.species.id.clone(),
                    level: received.level,
                    learned_move,
                    defer_level_evolution: false,
                })
                .collect::<Vec<_>>();
            self.session.state.pending_move_learn = pending.first().cloned();
            self.session
                .state
                .pending_move_learn_queue
                .extend(pending.into_iter().skip(1));
        }
        self.session.state.sync_party_from_storage();
        Ok(RuntimeLinkTradeApply {
            sent,
            received_party_index,
            evolution,
        })
    }

    pub fn last_frame(&self) -> Option<&RuntimeOverworldFrame> {
        self.last_frame.as_ref()
    }

    pub fn asset_root(&self) -> &AssetRoot {
        &self.asset_root
    }

    fn record_runtime_mutation_outcome(&mut self, outcome: &RuntimeMutationOutcome) {
        if let RuntimeMutationResult::OverworldInputApplied(frame) = &outcome.result {
            self.last_frame = Some(RuntimeOverworldFrame::from_input_frame(
                frame.clone(),
                outcome.state_checksum.clone(),
            ));
        }
    }
}

