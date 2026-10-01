impl RuntimeGameShell {
    /// Apply session commands atomically on error without cloning the game pack.
    /// The closure may use session commands but must not replace the runtime.
    /// External effects (such as files written by the closure) are not reverted.
    pub fn try_session_update<T>(
        &mut self,
        update: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        let session = self.session.clone();
        let last_frame = self.last_frame.clone();
        let linked_menu_results = self.linked_menu_results.clone();
        let sequence = self.runtime_command_sequence;
        let commands = self.runtime_commands.clone();
        let results = self.runtime_results.clone();
        let retain_journal = self.retain_runtime_journal;
        let result = update(self);
        if result.is_err() {
            self.session = session;
            self.last_frame = last_frame;
            self.linked_menu_results = linked_menu_results;
            self.runtime_command_sequence = sequence;
            self.runtime_commands = commands;
            self.runtime_results = results;
            self.retain_runtime_journal = retain_journal;
        }
        result
    }

    /// Start a playable overworld session at the pack-defined spawn.
    pub fn new_game(
        asset_root: AssetRoot,
        runtime: CrystalRuntime,
        spawn_identifier: u16,
    ) -> Result<Self> {
        let mut session = runtime
            .start_overworld_session(&asset_root, spawn_identifier)
            .with_context(|| format!("start runtime game shell at spawn {spawn_identifier}"))?;
        // A headless RuntimeGameShell begins at FinishContinue's playable
        // overworld boundary. The visible title/new-game flow clears this
        // again while its pre-overworld sequence is active.
        session.state.set_game_timer_counting(true);
        Ok(Self {
            asset_root,
            runtime,
            session,
            last_frame: None,
            linked_menu_results: Vec::new(),
            runtime_command_sequence: 0,
            runtime_commands: Vec::new(),
            runtime_results: Vec::new(),
            retain_runtime_journal: true,
        })
    }

    /// Execute NewGame's ResetWRAM against the title session's retained hRandom
    /// registers, divider source, options, and Lucky-ID SRAM fields, then apply
    /// the pack-derived world initialization to that exact core state.
    /// Reset game state when the host completes its title/new-game flow.
    pub fn reset_new_game_from_title(&mut self, spawn_identifier: u16) -> Result<()> {
        let spawn = self.runtime.data.runtime_spawn_point(spawn_identifier)?;
        let title_state = &self.session.state;
        let mut divider_after = self.session.divider.clone();
        let reset_state = GameState::reset_wram_for_new_game_with_hardware(
            title_state.options.clone(),
            title_state.random_state,
            title_state.vblank_counter,
            title_state.lucky_number_day,
            title_state.lucky_id_number,
            &mut divider_after,
        )?;
        let (state, overworld) = self
            .runtime
            .data
            .start_overworld_session_from_new_game_state(
                spawn,
                reset_state,
                &self.runtime.audio.music_ids(),
            )?;
        self.session = RuntimeOverworldSession {
            state,
            overworld,
            joypad: JoypadState::new(),
            divider: divider_after,
        };
        self.last_frame = None;
        self.linked_menu_results.clear();
        self.runtime_command_sequence = 0;
        self.runtime_commands.clear();
        self.runtime_results.clear();
        Ok(())
    }

    #[cfg(any(test, feature = "test-fixtures", feature = "location-tester"))]
    pub fn new_game_at_runtime_tile(
        asset_root: AssetRoot,
        runtime: CrystalRuntime,
        spawn_identifier: u16,
        map_name: impl AsRef<str>,
        tile_x: i16,
        tile_y: i16,
    ) -> Result<Self> {
        let map_name = map_name.as_ref();
        let last_spawn_map_constant = runtime
            .data
            .runtime_spawn_point(spawn_identifier)?
            .map_constant
            .clone();
        let mut session = runtime
            .start_overworld_session_at_runtime_tile(&asset_root, map_name, tile_x, tile_y)
            .with_context(|| {
                format!("start runtime game shell at {map_name} runtime tile ({tile_x}, {tile_y})")
            })?;
        session.state.last_spawn_map_constant = Some(last_spawn_map_constant);
        session.state.set_game_timer_counting(true);
        Ok(Self {
            asset_root,
            runtime,
            session,
            last_frame: None,
            linked_menu_results: Vec::new(),
            runtime_command_sequence: 0,
            runtime_commands: Vec::new(),
            runtime_results: Vec::new(),
            retain_runtime_journal: true,
        })
    }

    pub fn resume_from_save(
        asset_root: AssetRoot,
        runtime: CrystalRuntime,
        save_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let state = runtime.load_save(save_path)?;
        let mut session = runtime
            .resume_overworld_session(&asset_root, state)
            .context("resume runtime game shell from save")?;
        // FinishContinueFunction sets GAME_TIMER_COUNTING_F after loading;
        // the WRAM control byte itself is not SRAM-backed.
        session.state.set_game_timer_counting(true);
        session.state.set_game_logic_paused(false);
        Ok(Self {
            asset_root,
            runtime,
            session,
            last_frame: None,
            linked_menu_results: Vec::new(),
            runtime_command_sequence: 0,
            runtime_commands: Vec::new(),
            runtime_results: Vec::new(),
            retain_runtime_journal: true,
        })
    }

    /// Disable retained command/result serialization for a real-time host.
    /// Gameplay state and the per-frame checksum remain authoritative; only
    /// the optional replay journal is omitted.
    pub fn set_runtime_journal_enabled(&mut self, enabled: bool) {
        self.retain_runtime_journal = enabled;
    }

    pub fn tick(
        &mut self,
        buttons: impl IntoIterator<Item = GameButton>,
    ) -> Result<&RuntimeOverworldFrame> {
        self.advance_game_timer_vblank()?;
        self.tick_after_vblank(buttons)
    }

    /// Apply input after the host has already advanced VBlank for this tick.
    pub fn tick_after_vblank(
        &mut self,
        buttons: impl IntoIterator<Item = GameButton>,
    ) -> Result<&RuntimeOverworldFrame> {
        let buttons = buttons.into_iter().collect::<Vec<_>>();
        if !self.retain_runtime_journal {
            let frame = self
                .session
                .apply_overworld_input_live(&self.runtime, buttons)?;
            self.last_frame = Some(frame);
            return self
                .last_frame
                .as_ref()
                .context("runtime shell did not store the live frame it just produced");
        }
        let recorded = self.session.stage_overworld_input(
            &self.runtime,
            buttons,
            self.retain_runtime_journal,
        )?;
        let mutation = self
            .apply_recorded_runtime_mutation(recorded)
            .context("advance runtime game shell")?;
        let RuntimeMutationResult::OverworldInputApplied(frame) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-overworld-input result");
        };
        self.session.joypad = JoypadState::from_previous_mask(frame.input_mask);
        self.last_frame = Some(RuntimeOverworldFrame::from_input_frame(
            frame,
            mutation.state_checksum,
        ));
        self.last_frame
            .as_ref()
            .context("runtime shell did not store the frame it just produced")
    }

    /// Advance one authoritative gameplay frame using an injected RTC sample.
    /// This is the deterministic entry point for replay/oracle adapters; the
    /// ordinary `tick` path remains available for hosts that update the clock
    /// separately.
    pub fn tick_with_rtc(
        &mut self,
        buttons: impl IntoIterator<Item = GameButton>,
        rtc: RuntimeRtcSample,
    ) -> Result<&RuntimeOverworldFrame> {
        self.advance_game_timer_vblank()?;
        self.update_clock_from_datetime(rtc.date, rtc.hour, rtc.minute, rtc.second)?;
        self.tick_after_vblank(buttons)
    }

    pub fn tick_with_rtc_after_vblank(
        &mut self,
        buttons: impl IntoIterator<Item = GameButton>,
        rtc: RuntimeRtcSample,
    ) -> Result<&RuntimeOverworldFrame> {
        self.update_clock_from_datetime(rtc.date, rtc.hour, rtc.minute, rtc.second)?;
        self.tick_after_vblank(buttons)
    }

    pub fn state_checksum_frame(&self, player_id: PlayerId) -> Result<StateChecksumFrame> {
        self.runtime
            .validate_save_state_for_runtime_pack(self.session.state())
            .context("validate runtime game shell state before checksum")?;
        self.session.state_checksum_frame(player_id)
    }

    pub fn link_session_descriptor(
        &self,
        session_id: impl Into<String>,
        player_id: PlayerId,
        display_name: impl Into<String>,
    ) -> Result<RuntimeLinkSessionDescriptor> {
        let session = LinkSessionIdentity::new(
            session_id,
            self.runtime.modpack().clone(),
            self.runtime.pack_identity().content_hash.clone(),
        )
        .context("build runtime link session identity")?;
        let local_player = PlayerIdentity::new(player_id, display_name)
            .context("build runtime link player identity")?;
        let hello = LinkHello::from_session(session.clone(), local_player.clone())
            .context("build runtime link hello")?;
        let checksum = self.state_checksum_frame(player_id)?;
        let save_checkpoint = self.runtime.session_save_checkpoint_for_state(
            session.clone(),
            self.session.state(),
            player_id,
        )?;
        Ok(RuntimeLinkSessionDescriptor {
            session,
            local_player,
            hello,
            checksum,
            save_checkpoint,
        })
    }

    pub fn validate_link_session_descriptor(
        &self,
        descriptor: &RuntimeLinkSessionDescriptor,
    ) -> Result<()> {
        validate_link_session_identity(&descriptor.session, descriptor.hello.session())
            .context("runtime link hello session does not match descriptor session")?;
        if descriptor.hello.player() != &descriptor.local_player {
            anyhow::bail!("runtime link hello player does not match descriptor local player");
        }
        if descriptor.checksum.player_id() != descriptor.local_player.id() {
            anyhow::bail!(
                "runtime link checksum player {} does not match local player {}",
                descriptor.checksum.player_id(),
                descriptor.local_player.id()
            );
        }
        if descriptor.save_checkpoint.session() != &descriptor.session {
            anyhow::bail!("runtime link save checkpoint session does not match descriptor session");
        }
        descriptor
            .save_checkpoint
            .validate()
            .context("runtime link save checkpoint is invalid")?;
        let checkpoint = descriptor.save_checkpoint.checkpoint();
        if checkpoint.summary().state_frame() != descriptor.checksum.frame()
            || checkpoint.checksum().frame() != descriptor.checksum.frame()
            || checkpoint.summary().state_hash() != descriptor.checksum.hash()
            || checkpoint.checksum().hash() != descriptor.checksum.hash()
        {
            anyhow::bail!(
                "runtime link save checkpoint frame/hash does not match descriptor checksum: summary {} {:#010x}, checkpoint {} {:#010x}, descriptor {} {:#010x}",
                checkpoint.summary().state_frame(),
                checkpoint.summary().state_hash(),
                checkpoint.checksum().frame(),
                checkpoint.checksum().hash(),
                descriptor.checksum.frame(),
                descriptor.checksum.hash()
            );
        }
        Ok(())
    }

    pub fn link_endpoint<T: crystal_net::LinkTransport>(
        &self,
        transport: T,
        descriptor: &RuntimeLinkSessionDescriptor,
    ) -> Result<crystal_net::LinkEndpoint<T>> {
        self.validate_link_session_descriptor(descriptor)
            .context("validate runtime link descriptor before endpoint creation")?;
        crystal_net::LinkEndpoint::new(transport, descriptor.hello.clone())
            .context("build runtime link endpoint")
    }

    pub fn send_link_save_checkpoint<T: crystal_net::LinkTransport>(
        &self,
        endpoint: &mut crystal_net::LinkEndpoint<T>,
        descriptor: &RuntimeLinkSessionDescriptor,
    ) -> Result<()> {
        self.validate_link_session_descriptor(descriptor)
            .context("validate runtime link descriptor before save checkpoint send")?;
        endpoint
            .send(LinkMessage::SessionSaveCheckpoint(
                descriptor.save_checkpoint.clone(),
            ))
            .context("send runtime link save checkpoint")
    }

    pub fn send_link_bootstrap<T: crystal_net::LinkTransport>(
        &self,
        endpoint: &mut crystal_net::LinkEndpoint<T>,
        descriptor: &RuntimeLinkSessionDescriptor,
    ) -> Result<()> {
        self.validate_link_session_descriptor(descriptor)
            .context("validate runtime link descriptor before bootstrap send")?;
        endpoint.send_hello().context("send runtime link hello")?;
        self.send_link_save_checkpoint(endpoint, descriptor)
    }

    pub fn require_link_checkpoints<T: crystal_net::LinkTransport>(
        &self,
        endpoint: &crystal_net::LinkEndpoint<T>,
        players: impl IntoIterator<Item = PlayerId>,
    ) -> Result<()> {
        endpoint
            .require_checkpoints_for_players(players)
            .context("require runtime link peer save checkpoints")
    }

    pub fn input_journal_from_lockstep_frames(
        &self,
        descriptor: &RuntimeLinkSessionDescriptor,
        players: impl IntoIterator<Item = PlayerId>,
        terminal_checksum: StateChecksumFrame,
        frames: Vec<LockstepFrame>,
    ) -> Result<RuntimeInputJournal> {
        self.validate_link_session_descriptor(descriptor)
            .context("validate runtime link descriptor before journal build")?;
        let journal = DeterministicInputJournal::new(
            descriptor.session.clone(),
            players,
            descriptor.checksum.clone(),
            terminal_checksum.clone(),
            frames,
        )
        .context("build deterministic runtime input journal")?;
        Ok(RuntimeInputJournal {
            journal,
            terminal_checksum,
        })
    }

    pub fn local_input_journal(
        &self,
        descriptor: &RuntimeLinkSessionDescriptor,
        terminal_checksum: StateChecksumFrame,
        inputs: impl IntoIterator<Item = (u64, u8)>,
    ) -> Result<RuntimeInputJournal> {
        let player_id = descriptor.local_player.id();
        let mut frames = Vec::new();
        for (frame, joypad_mask) in inputs {
            frames.push(
                LockstepFrame::new(frame, BTreeMap::from([(player_id, joypad_mask)]))
                    .context("build local runtime lockstep input frame")?,
            );
        }
        self.input_journal_from_lockstep_frames(descriptor, [player_id], terminal_checksum, frames)
    }

    pub fn record_local_input_journal(
        &mut self,
        descriptor: &RuntimeLinkSessionDescriptor,
        inputs: impl IntoIterator<Item = Vec<GameButton>>,
    ) -> Result<RuntimeInputJournal> {
        let player_id = descriptor.local_player.id();
        let mut next_frame = descriptor.checksum.frame();
        let mut frames = Vec::new();
        for buttons in inputs {
            let applied = self.tick(buttons)?;
            frames.push(
                LockstepFrame::new(
                    next_frame,
                    BTreeMap::from([(player_id, applied.input_mask)]),
                )
                .context("record runtime lockstep input frame")?,
            );
            next_frame = next_frame.checked_add(1).with_context(|| {
                format!("runtime input journal frame cursor overflowed at frame {next_frame}")
            })?;
        }
        let terminal_checksum = self.state_checksum_frame(player_id)?;
        self.input_journal_from_lockstep_frames(descriptor, [player_id], terminal_checksum, frames)
    }

    pub fn apply_deterministic_replay_bundle(
        &mut self,
        descriptor: &RuntimeLinkSessionDescriptor,
        bundle: &DeterministicReplayBundle,
    ) -> Result<RuntimeInputJournal> {
        validate_deterministic_replay_runtime_authority(bundle, descriptor.local_player.id())?;
        let journal = bundle.input_journal().journal();
        self.validate_local_input_journal_start(descriptor, &journal)?;
        let player_id = descriptor.local_player.id();
        let previous = self.clone();
        let original_divider = self.session.divider.clone();
        let replay = (|| {
            // No command in a deterministic bundle may sample the host DIV.
            // Trace-bearing commands construct their own ReplayDivider; an
            // accidental legacy read therefore fails closed here.
            self.session.divider = RuntimeDividerSource::replay([]);
            for (command, expected_result) in bundle
                .runtime_commands()
                .iter()
                .zip(bundle.runtime_results())
            {
                let request = command.command();
                let result_index = self.runtime_results.len();
                self.apply_runtime_command_frame(request).with_context(|| {
                    format!(
                        "apply deterministic runtime command sequence {}",
                        request.sequence()
                    )
                })?;
                let actual_result = self.runtime_results.get(result_index).with_context(|| {
                    format!(
                        "runtime command sequence {} did not retain its generated result",
                        request.sequence()
                    )
                })?;
                if actual_result != expected_result.result() {
                    anyhow::bail!(
                        "generated result for runtime command sequence {} does not match the deterministic bundle",
                        request.sequence()
                    );
                }
            }
            let terminal_checksum = self.state_checksum_frame(player_id)?;
            if &terminal_checksum != bundle.terminal_checksum() {
                anyhow::bail!(
                    "deterministic runtime replay terminal checksum does not match the bundle"
                );
            }
            self.session.divider = original_divider;
            Ok(RuntimeInputJournal {
                journal: journal.clone(),
                terminal_checksum,
            })
        })();
        if replay.is_err() {
            *self = previous;
        }
        replay
    }

    pub fn validate_local_input_journal_start(
        &self,
        descriptor: &RuntimeLinkSessionDescriptor,
        journal: &DeterministicInputJournal,
    ) -> Result<()> {
        self.validate_link_session_descriptor(descriptor)
            .context("validate runtime link descriptor before journal start validation")?;
        journal
            .validate()
            .context("validate deterministic runtime input journal")?;
        validate_link_session_identity(&descriptor.session, journal.session())
            .context("validate runtime input journal session")?;
        if journal.start_checksum() != &descriptor.checksum {
            anyhow::bail!("runtime input journal start checksum does not match descriptor");
        }
        let current_checksum = self.state_checksum_frame(descriptor.local_player.id())?;
        if current_checksum != descriptor.checksum {
            anyhow::bail!(
                "runtime input journal start checksum frame/hash {} {:#010x} does not match current state {} {:#010x}",
                descriptor.checksum.frame(),
                descriptor.checksum.hash(),
                current_checksum.frame(),
                current_checksum.hash()
            );
        }
        Ok(())
    }

    pub fn input_journal_message(&self, journal: RuntimeInputJournal) -> Result<LinkMessage> {
        Ok(LinkMessage::InputJournal(
            DeterministicInputJournalFrame::new(journal.journal)
                .context("build runtime input journal frame")?,
        ))
    }

    pub fn save_resume_replay_bundle(
        &self,
        descriptor: &RuntimeLinkSessionDescriptor,
        journal: RuntimeInputJournal,
        runtime_commands: Vec<SessionRuntimeCommandFrame>,
        runtime_results: Vec<SessionRuntimeCommandResultFrame>,
        menu_results: Vec<MenuChoiceResultFrame>,
    ) -> Result<SaveResumeReplayBundle> {
        self.validate_link_session_descriptor(descriptor)
            .context("validate runtime link descriptor before save-resume replay")?;
        let journal_frame = DeterministicInputJournalFrame::new(journal.journal)
            .context("build runtime save-resume input journal frame")?;
        let replay = DeterministicReplayBundle::new(
            journal_frame,
            runtime_commands,
            runtime_results,
            menu_results,
            journal.terminal_checksum,
        )
        .context("build runtime deterministic replay bundle")?;
        validate_deterministic_replay_runtime_authority(&replay, descriptor.local_player.id())
            .context("validate runtime replay command authority before send")?;
        SaveResumeReplayBundle::new(descriptor.save_checkpoint.clone(), replay)
            .context("build runtime save-resume replay bundle")
    }

    pub fn save_resume_replay_message(
        &self,
        descriptor: &RuntimeLinkSessionDescriptor,
        journal: RuntimeInputJournal,
        runtime_commands: Vec<SessionRuntimeCommandFrame>,
        runtime_results: Vec<SessionRuntimeCommandResultFrame>,
        menu_results: Vec<MenuChoiceResultFrame>,
    ) -> Result<LinkMessage> {
        Ok(LinkMessage::SaveResumeReplay(
            self.save_resume_replay_bundle(
                descriptor,
                journal,
                runtime_commands,
                runtime_results,
                menu_results,
            )?,
        ))
    }

    pub fn linked_menu_results(&self) -> &[MenuChoiceResultFrame] {
        &self.linked_menu_results
    }

    pub fn drain_linked_menu_results(&mut self) -> Vec<MenuChoiceResultFrame> {
        std::mem::take(&mut self.linked_menu_results)
    }

    pub fn retained_runtime_commands(&self) -> &[RuntimeCommandFrame] {
        &self.runtime_commands
    }

    pub fn retained_runtime_results(&self) -> &[RuntimeCommandResultFrame] {
        &self.runtime_results
    }

    pub fn clear_retained_runtime_commands(&mut self) {
        self.runtime_command_sequence = 0;
        self.runtime_commands.clear();
        self.runtime_results.clear();
    }

    fn require_valid_script_modal_state(&self, action: &str) -> Result<()> {
        self.session
            .state
            .script_runtime
            .validate()
            .map_err(anyhow::Error::msg)
            .with_context(|| format!("cannot {action} from invalid script modal state"))
    }

    pub fn runtime_command_frame(
        &self,
        player_id: PlayerId,
        sequence: u64,
        command: RuntimeMutationCommand,
    ) -> Result<RuntimeCommandFrame> {
        self.session
            .runtime_command_frame(player_id, sequence, command)
    }

    pub fn require_runtime_command_expected_state(
        &self,
        request: &RuntimeCommandFrame,
    ) -> Result<()> {
        self.session.require_runtime_command_expected_state(request)
    }

    pub fn runtime_mutation_result_frame(
        &self,
        request: RuntimeCommandFrame,
        outcome: &RuntimeMutationOutcome,
    ) -> Result<RuntimeCommandResultFrame> {
        self.session.runtime_mutation_result_frame(request, outcome)
    }

    pub fn apply_runtime_mutation_command(
        &mut self,
        command: RuntimeMutationCommand,
    ) -> Result<RuntimeMutationOutcome> {
        self.require_valid_script_modal_state("apply runtime mutation command")?;
        if !self.retain_runtime_journal {
            if let RuntimeMutationCommand::AdvanceGameTimerVBlanks(command) = command.clone() {
                let outcome = self
                    .runtime
                    .data
                    .advance_game_timer_vblanks_fast(
                        &mut self.session.state,
                        &mut self.session.overworld,
                        command.vblanks,
                        &command.normal_divider_trace,
                        &self.runtime.audio.music_ids(),
                        &self.runtime.audio.sound_effect_ids(),
                        &self.runtime.audio.cry_ids(),
                    )
                    .context("advance runtime game timer VBlank")?;
                self.record_runtime_mutation_outcome(&outcome);
                return Ok(outcome);
            }
        }
        let sequence = self
            .runtime_command_sequence
            .checked_add(1)
            .context("runtime command sequence overflow")?;
        let request = self
            .runtime_command_frame(RUNTIME_LOCAL_PLAYER_ID, sequence, command.clone())
            .context("build retained runtime command frame")?;
        // Local input uses the single-execution recorded path. Remote input
        // replays an explicit divider trace and must roll back state if result
        // framing/checksum validation fails after the asset-layer transaction.
        let transactional = !matches!(command, RuntimeMutationCommand::AdvanceGameTimerVBlanks(_));
        let previous_session = transactional.then(|| self.session.clone());
        let outcome = match self
            .session
            .apply_runtime_mutation_command(&self.runtime, command)
        {
            Ok(outcome) => outcome,
            Err(error) => {
                if let Some(previous_session) = previous_session {
                    self.session = previous_session;
                }
                return Err(error);
            }
        };
        let result = match self
            .runtime_mutation_result_frame(request.clone(), &outcome)
            .context("build retained runtime command result frame")
        {
            Ok(result) => result,
            Err(error) => {
                if let Some(previous_session) = previous_session {
                    self.session = previous_session;
                }
                return Err(error);
            }
        };
        if let Err(error) = self.require_valid_script_modal_state("finish runtime mutation command")
        {
            if let Some(previous_session) = previous_session {
                self.session = previous_session;
            }
            return Err(error);
        }
        self.runtime_command_sequence = sequence;
        self.runtime_commands.push(request);
        self.runtime_results.push(result);
        self.record_runtime_mutation_outcome(&outcome);
        Ok(outcome)
    }

    fn apply_recorded_runtime_mutation(
        &mut self,
        recorded: RecordedRuntimeMutation,
    ) -> Result<RuntimeMutationOutcome> {
        self.require_valid_script_modal_state("apply recorded runtime mutation")?;
        if !self.retain_runtime_journal {
            let outcome = self.session.commit_recorded_mutation(recorded);
            self.record_runtime_mutation_outcome(&outcome);
            return Ok(outcome);
        }
        let sequence = self
            .runtime_command_sequence
            .checked_add(1)
            .context("runtime command sequence overflow")?;
        // The command is formed only after the single staged execution has
        // captured its exact DIV reads, but its expected checksum is still
        // computed against the untouched pre-mutation session.
        let request = self
            .runtime_command_frame(RUNTIME_LOCAL_PLAYER_ID, sequence, recorded.command.clone())
            .context("build recorded runtime command frame")?;
        let previous_session = self.session.clone();
        let outcome = self.session.commit_recorded_mutation(recorded);
        let result = match self
            .runtime_mutation_result_frame(request.clone(), &outcome)
            .context("build recorded runtime command result frame")
        {
            Ok(result) => result,
            Err(error) => {
                self.session = previous_session;
                return Err(error);
            }
        };
        if let Err(error) =
            self.require_valid_script_modal_state("finish recorded runtime mutation")
        {
            self.session = previous_session;
            return Err(error);
        }
        self.runtime_command_sequence = sequence;
        self.runtime_commands.push(request);
        self.runtime_results.push(result);
        self.record_runtime_mutation_outcome(&outcome);
        Ok(outcome)
    }

    fn apply_special_routine_runtime_mutation(
        &mut self,
        routine: &str,
    ) -> Result<RuntimeMutationOutcome> {
        if runtime_special_routine_requires_divider_trace(routine) {
            let recorded = self
                .session
                .stage_random_special_routine(&self.runtime, routine)?;
            return self.apply_recorded_runtime_mutation(recorded);
        }
        self.apply_runtime_mutation_command(RuntimeMutationCommand::ApplySpecialRoutine {
            routine: routine.to_string(),
        })
    }

    pub fn apply_compiled_script_command(
        &mut self,
        origin_map_name: &str,
        source_script: &str,
        command_index: usize,
        inputs: ScriptRuntimeInputs,
        phone_inputs: ScriptPhoneInputs,
    ) -> Result<RuntimeMutationOutcome> {
        let map_name = origin_map_name.to_string();
        let command = self
            .runtime
            .compiled_script_command_name(source_script, command_index)?;
        let command_ref = RuntimeScriptCommandRef::new(&map_name, source_script, command_index);
        let is_gift_pokemon_command =
            self.runtime
                .has_gift_pokemon_command_at(&map_name, source_script, command_index);
        if !is_gift_pokemon_command {
            reject_unexpected_gift_pokemon_inputs(source_script, command_index, &command, &inputs)?;
        }
        if is_gift_pokemon_command {
            let recorded = self.session.stage_scripted_gift_pokemon(
                &self.runtime,
                command_ref,
                inputs.gift_original_trainer_name.clone().with_context(|| {
                    format!(
                        "compiled gift Pokemon command {}:{} requires gift_original_trainer_name input",
                        source_script, command_index
                    )
                })?,
                inputs.gift_original_trainer_id.with_context(|| {
                    format!(
                        "compiled gift Pokemon command {}:{} requires gift_original_trainer_id input",
                        source_script, command_index
                    )
                })?,
                inputs.gift_nickname_accepted.with_context(|| {
                    format!(
                        "compiled gift Pokemon command {}:{} requires gift_nickname_accepted input",
                        source_script, command_index
                    )
                })?,
                inputs.gift_nickname.clone(),
            )?;
            return self.apply_recorded_runtime_mutation(recorded);
        }
        if command == "random" {
            let recorded = self
                .session
                .stage_random_script_runtime(&self.runtime, command_ref)?;
            return self.apply_recorded_runtime_mutation(recorded);
        }
        if self
            .runtime
            .data()
            .is_exact_rock_mon_encounter_command(&command_ref)?
        {
            let recorded = self
                .session
                .stage_rock_mon_encounter(&self.runtime, command_ref)?;
            return self.apply_recorded_runtime_mutation(recorded);
        }
        if self
            .runtime
            .data()
            .is_exact_tree_mon_encounter_command(&command_ref)?
        {
            let recorded = self
                .session
                .stage_tree_mon_encounter(&self.runtime, command_ref)?;
            return self.apply_recorded_runtime_mutation(recorded);
        }
        if self
            .runtime
            .data()
            .is_exact_sweet_scent_encounter_command(&command_ref)?
        {
            let recorded = self
                .session
                .stage_sweet_scent_encounter(&self.runtime, command_ref)?;
            return self.apply_recorded_runtime_mutation(recorded);
        }
        let is_fixed_scripted_wild_battle_start = self
            .runtime
            .has_scripted_wild_battle_start_command_at(&map_name, source_script, command_index);
        let is_scripted_wild_battle_start = is_fixed_scripted_wild_battle_start
            || self
                .runtime
                .data()
                .is_exact_rock_smash_dynamic_start_command(&command_ref)?
            || self
                .runtime
                .data()
                .is_exact_headbutt_dynamic_start_command(&command_ref)?
            || self
                .runtime
                .data()
                .is_exact_sweet_scent_dynamic_start_command(&command_ref)?;
        if is_scripted_wild_battle_start {
            if is_fixed_scripted_wild_battle_start {
                self.runtime.data().require_scripted_wild_battle_setup(
                    self.session.state(),
                    &map_name,
                    source_script,
                    command_index,
                )?;
            }
            let recorded = self
                .session
                .stage_scripted_wild_battle_start(&self.runtime, command_ref)?;
            return self.apply_recorded_runtime_mutation(recorded);
        }
        let mutation = if self.runtime.has_scripted_trainer_battle_start_command_at(
            &map_name,
            source_script,
            command_index,
        ) {
            if command == "trainer" {
                RuntimeMutationCommand::ResolveMapTrainerInteraction(
                    crate::assets::RuntimeMapTrainerInteractionCommand {
                        command: command_ref,
                        defer_battle_start: false,
                    },
                )
            } else {
                self.runtime.data().require_scripted_trainer_battle_setup(
                    self.session.state(),
                    &map_name,
                    source_script,
                    command_index,
                )?;
                RuntimeMutationCommand::StartScriptedTrainerBattle(command_ref)
            }
        } else if self.runtime.has_script_item_grant_command_at(
            &map_name,
            source_script,
            command_index,
        ) {
            RuntimeMutationCommand::GrantScriptItem(command_ref)
        } else if self.runtime.has_script_item_check_command_at(
            &map_name,
            source_script,
            command_index,
        ) {
            RuntimeMutationCommand::CheckScriptItem(command_ref)
        } else if self.runtime.has_script_item_take_command_at(
            &map_name,
            source_script,
            command_index,
        ) {
            RuntimeMutationCommand::TakeScriptItem(command_ref)
        } else if self.runtime.has_script_field_pickup_command_at(
            &map_name,
            source_script,
            command_index,
        ) {
            RuntimeMutationCommand::PickupScriptFieldItem(command_ref)
        } else if self.runtime.has_script_economy_command_at(
            &map_name,
            source_script,
            command_index,
        ) {
            RuntimeMutationCommand::ApplyScriptEconomy(command_ref)
        } else if self
            .runtime
            .has_script_flag_command_at(&map_name, source_script, command_index)
        {
            match command.as_str() {
                "checkevent" | "checkflag" => RuntimeMutationCommand::CheckScriptFlag(command_ref),
                _ => RuntimeMutationCommand::ApplyScriptFlagMutation(command_ref),
            }
        } else if self
            .runtime
            .has_script_scene_command_at(&map_name, source_script, command_index)
        {
            RuntimeMutationCommand::ApplyScriptScene(command_ref)
        } else if self.runtime.has_script_block_change_command_at(
            &map_name,
            source_script,
            command_index,
        ) {
            RuntimeMutationCommand::ApplyScriptBlockChange(command_ref)
        } else if self
            .runtime
            .has_script_audio_command_at(&map_name, source_script, command_index)
        {
            RuntimeMutationCommand::ApplyScriptAudio(command_ref)
        } else if self
            .runtime
            .has_script_map_command_at(&map_name, source_script, command_index)
        {
            if command == "reloadmapafterbattle" {
                let recorded = self
                    .session
                    .stage_random_script_map(&self.runtime, command_ref)?;
                return self.apply_recorded_runtime_mutation(recorded);
            }
            RuntimeMutationCommand::ApplyScriptMap(command_ref)
        } else if self
            .runtime
            .has_script_text_command_at(&map_name, source_script, command_index)
        {
            RuntimeMutationCommand::ApplyScriptText(command_ref)
        } else if self.runtime.has_script_variable_command_at(
            &map_name,
            source_script,
            command_index,
        ) {
            RuntimeMutationCommand::ApplyScriptVariableNow(command_ref)
        } else if self
            .runtime
            .has_script_swarm_command_at(&map_name, source_script, command_index)
        {
            RuntimeMutationCommand::ApplyScriptSwarm(command_ref)
        } else if self
            .runtime
            .has_script_phone_command_at(&map_name, source_script, command_index)
        {
            RuntimeMutationCommand::ApplyScriptPhone {
                command: command_ref,
                inputs: phone_inputs,
            }
        } else if self.runtime.has_script_control_command_at(
            &map_name,
            source_script,
            command_index,
        ) {
            RuntimeMutationCommand::ApplyScriptControl(command_ref)
        } else if self.runtime.has_script_movement_command_at(
            &map_name,
            source_script,
            command_index,
        ) {
            RuntimeMutationCommand::ApplyScriptMovement(command_ref)
        } else if self
            .runtime
            .has_script_object_command_at(&map_name, source_script, command_index)
        {
            RuntimeMutationCommand::ApplyScriptObjectMutation(command_ref)
        } else if self.runtime.has_script_runtime_command_at(
            &map_name,
            source_script,
            command_index,
        ) {
            if command == "special" {
                let routine = self
                    .runtime
                    .script_runtime_command_at(&map_name, source_script, command_index)
                    .filter(|key| key.command == "special")
                    .and_then(|key| key.args.first().cloned())
                    .with_context(|| {
                        format!(
                            "compiled script command {}:{} special is missing routine id",
                            source_script, command_index
                        )
                    })?;
                if runtime_special_routine_requires_divider_trace(&routine) {
                    return self.apply_special_routine_runtime_mutation(&routine);
                }
                RuntimeMutationCommand::ApplySpecialRoutine { routine }
            } else {
                RuntimeMutationCommand::ApplyScriptRuntime {
                    command: command_ref,
                    inputs,
                }
            }
        } else if self
            .runtime
            .has_script_shop_command_at(&map_name, source_script, command_index)
        {
            RuntimeMutationCommand::OpenScriptShop(command_ref)
        } else {
            anyhow::bail!(
                "compiled script command {}:{} '{}' has no Rust runtime mutation",
                source_script,
                command_index,
                command
            );
        };
        self.apply_runtime_mutation_command(mutation)
            .with_context(|| {
                format!("apply compiled script command {source_script}:{command_index} '{command}'")
            })
    }

    pub fn step_compiled_script_command(
        &mut self,
        origin_map_name: &str,
        source_script: &str,
        command_index: usize,
        inputs: ScriptRuntimeInputs,
        phone_inputs: ScriptPhoneInputs,
    ) -> Result<RuntimeCompiledScriptStep> {
        if !self.runtime.data().maps.contains_key(origin_map_name) {
            anyhow::bail!(
                "compiled script cursor origin map {origin_map_name} is missing from the pack"
            );
        }
        let command = self
            .runtime
            .compiled_script_command_name(source_script, command_index)?;
        let command_count = self.runtime.compiled_script_commands(source_script)?.len();
        let mutation = self.apply_compiled_script_command(
            origin_map_name,
            source_script,
            command_index,
            inputs,
            phone_inputs,
        )?;
        let mut next_script = self
            .session
            .state()
            .script_runtime
            .next_script
            .as_ref()
            .filter(|next| next.script != source_script)
            .cloned();
        let mut ended = self.session.state().script_runtime.script_ended.is_some();
        if let RuntimeMutationResult::ScriptControlApplied(action) = &mutation.result {
            match action {
                ScriptControlAction::Jump {
                    target_script,
                    deferred,
                    ..
                } => {
                    if !deferred {
                        // The asset layer resolves the target's owning map
                        // against both the suspended source and the live map.
                        // This differs after a script-side warp (Battle
                        // Tower's prize handoff is canonical), so retain the
                        // authoritative ScriptLocation instead of rebuilding
                        // it from the stale source cursor.
                        next_script = self.session.state().script_runtime.next_script.clone();
                        anyhow::ensure!(
                            next_script
                                .as_ref()
                                .is_some_and(|next| next.script == *target_script),
                            "script jump to {target_script} did not retain its resolved target location"
                        );
                    }
                    ended = false;
                }
                ScriptControlAction::End { .. } => {
                    next_script = None;
                    ended = true;
                }
                ScriptControlAction::Continue { .. } => {}
            }
        }
        let next_cursor = if let Some(next_script) = &next_script {
            Some(RuntimeCompiledScriptCursor {
                origin_map_name: next_script.origin_map_name.clone(),
                source_script: next_script.script.clone(),
                command_index: 0,
            })
        } else if ended || command_index + 1 >= command_count {
            None
        } else {
            Some(RuntimeCompiledScriptCursor {
                origin_map_name: origin_map_name.to_string(),
                source_script: source_script.to_string(),
                command_index: command_index + 1,
            })
        };
        let boundary = compiled_script_boundary(self.session.state())
            .or_else(|| {
                matches!(
                    &mutation.result,
                    RuntimeMutationResult::ScriptItemGranted(
                        crate::core::systems::script_items::ScriptItemGrantOutcome::Granted {
                            verbose: true,
                            ..
                        } | crate::core::systems::script_items::ScriptItemGrantOutcome::BagFull {
                            verbose: true,
                            ..
                        }
                    )
                )
                .then_some(RuntimeCompiledScriptBoundary::VerboseItemGrant)
            })
            .or_else(|| {
                matches!(
                    mutation.result,
                    RuntimeMutationResult::ScriptMovementApplied(_)
                )
                .then_some(RuntimeCompiledScriptBoundary::ScriptMovement)
            })
            .or_else(|| {
                if let RuntimeMutationResult::ScriptRuntimeApplied(
                    _,
                    crate::core::systems::script_runtime::ScriptRuntimeOutcome::PhoneCallasmPresentation {
                        effect,
                        ..
                    },
                ) = &mutation.result
                {
                    Some(RuntimeCompiledScriptBoundary::PhoneCallasm(*effect))
                } else {
                    None
                }
            });

        Ok(RuntimeCompiledScriptStep {
            origin_map_name: origin_map_name.to_string(),
            source_script: source_script.to_string(),
            command_index,
            command,
            mutation,
            next_cursor,
            boundary,
            next_script: next_script.map(|next| next.script),
            ended,
        })
    }

    pub fn compiled_script_runtime_inputs(
        &self,
        origin_map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptRuntimeInputs> {
        self.runtime
            .compiled_script_command_name(source_script, command_index)?;
        let map_name = origin_map_name;
        let (
            gift_original_trainer_name,
            gift_original_trainer_id,
            gift_nickname_accepted,
            gift_nickname,
        ) = if self
            .runtime
            .has_gift_pokemon_command_at(&map_name, source_script, command_index)
        {
            let state = self.session.state();
            if state.player_name.is_empty() {
                anyhow::bail!(
                    "compiled gift Pokemon command {}:{} requires player identity before gift inputs can be generated",
                    source_script,
                    command_index
                );
            }
            (
                Some(state.player_name.clone()),
                Some(state.player_id),
                Some(false),
                None,
            )
        } else {
            (None, None, None, None)
        };
        Ok(ScriptRuntimeInputs {
            gift_original_trainer_name,
            gift_original_trainer_id,
            gift_nickname_accepted,
            gift_nickname,
            ..ScriptRuntimeInputs::default()
        })
    }

    pub fn run_compiled_script_until_boundary(
        &mut self,
        start: RuntimeCompiledScriptCursor,
        max_steps: usize,
        inputs: ScriptRuntimeInputs,
        phone_inputs: ScriptPhoneInputs,
    ) -> Result<RuntimeCompiledScriptRun> {
        if max_steps == 0 {
            anyhow::bail!("compiled script runner requires max_steps greater than zero");
        }
        let mut cursor = Some(start);
        let mut steps = Vec::new();
        while let Some(current) = cursor.take() {
            let follows_pending_next_script = !steps.is_empty()
                && self
                    .session
                    .state()
                    .script_runtime
                    .next_script
                    .as_ref()
                    .is_some_and(|next| {
                        next.origin_map_name == current.origin_map_name
                            && next.script == current.source_script
                    });
            if follows_pending_next_script {
                self.apply_runtime_mutation_command(RuntimeMutationCommand::TakeNextScript)
                    .with_context(|| {
                        format!(
                            "consume followed script transition {}:{}",
                            current.origin_map_name, current.source_script
                        )
                    })?;
            }
            if steps.len() >= max_steps {
                anyhow::bail!("compiled script runner exceeded max_steps {max_steps}");
            }
            let commands = self.runtime.compiled_script_commands(&current.source_script)?;
            let command = commands.get(current.command_index);
            // Registration is a synchronous choice, including when reached
            // through a jump or a returned common script. Leave the opcode
            // unexecuted until the frontend supplies the player's response.
            if phone_inputs.accepted.is_none()
                && command.and_then(|command| command.get("command"))
                    .and_then(serde_json::Value::as_str) == Some("askforphonenumber")
            {
                return Ok(RuntimeCompiledScriptRun {
                    steps,
                    next_cursor: Some(current),
                    boundary: Some(RuntimeCompiledScriptBoundary::PhoneNumberPrompt),
                    ended: false,
                });
            }
            let day_of_week_prompt = command.is_some_and(|command| {
                    command.get("command").and_then(serde_json::Value::as_str) == Some("special")
                        && command
                            .get("args")
                            .and_then(serde_json::Value::as_array)
                            .and_then(|args| args.first())
                            .and_then(serde_json::Value::as_str)
                            == Some("SetDayOfWeek")
                });
            if day_of_week_prompt {
                return Ok(RuntimeCompiledScriptRun {
                    steps,
                    next_cursor: Some(current),
                    boundary: Some(RuntimeCompiledScriptBoundary::DayOfWeekPrompt),
                    ended: false,
                });
            }
            let step_inputs = if steps.is_empty() && inputs != ScriptRuntimeInputs::default() {
                inputs.clone()
            } else {
                self.compiled_script_runtime_inputs(
                    &current.origin_map_name,
                    &current.source_script,
                    current.command_index,
                )?
            };
            let step = self
                .step_compiled_script_command(
                    &current.origin_map_name,
                    &current.source_script,
                    current.command_index,
                    step_inputs,
                    phone_inputs.clone(),
                )
                .with_context(|| {
                    format!(
                        "run compiled script {}:{}:{}",
                        current.origin_map_name, current.source_script, current.command_index
                    )
                })?;
            let boundary = step.boundary.clone();
            let ended = step.ended;
            cursor = step.next_cursor.clone();
            if std::env::var_os("CRYSTAL_SCRIPT_TRACE").is_some() {
                eprintln!(
                    "script_trace map={} script={} index={} command={} boundary={:?} next={:?} ended={}",
                    current.origin_map_name,
                    current.source_script,
                    current.command_index,
                    step.command,
                    boundary,
                    cursor,
                    ended
                );
            }
            steps.push(step);
            if ended && !self.session.state().script_runtime.call_stack.is_empty() {
                // ScriptEvents returns from an `scall`/`farscall` frame when
                // the called command stream ends. A composed run must do the
                // same instead of exposing the callee's end as the caller's
                // terminal boundary.
                self.take_script_end_state()
                    .context("consume called script end before returning")?;
                let returned = self
                    .pop_script_call_stack()
                    .context("resume compiled script call frame")?;
                cursor = Some(RuntimeCompiledScriptCursor {
                    origin_map_name: returned.frame.origin_map_name,
                    source_script: returned.frame.source_script,
                    command_index: returned.frame.next_command_index,
                });
                continue;
            }
            // `writetext`/`farwritetext` are synchronous in the ASM: the
            // interpreter does not execute the following command until the
            // text printer finishes. Expose that presentation boundary here;
            // the visible shell consumes it automatically after the complete
            // line has rendered, without requiring player input.
            if compiled_script_boundary_stops_run(
                &steps.last().expect("pushed script step").command,
                &boundary,
            ) || ended
            {
                return Ok(RuntimeCompiledScriptRun {
                    steps,
                    next_cursor: cursor,
                    boundary,
                    ended,
                });
            }
        }
        Ok(RuntimeCompiledScriptRun {
            steps,
            next_cursor: None,
            boundary: None,
            ended: false,
        })
    }

    pub fn run_next_queued_script_until_boundary(
        &mut self,
        max_steps: usize,
        inputs: ScriptRuntimeInputs,
        phone_inputs: ScriptPhoneInputs,
    ) -> Result<RuntimeQueuedCompiledScriptRun> {
        let queued = self.execute_next_queued_script_command()?;
        let run = self.run_compiled_script_until_boundary(
            RuntimeCompiledScriptCursor {
                origin_map_name: queued.queued.origin_map_name.clone(),
                source_script: queued.queued.target.clone(),
                command_index: 0,
            },
            max_steps,
            inputs,
            phone_inputs,
        )?;
        Ok(RuntimeQueuedCompiledScriptRun { queued, run })
    }

    pub fn run_pending_next_script_until_boundary(
        &mut self,
        max_steps: usize,
        inputs: ScriptRuntimeInputs,
        phone_inputs: ScriptPhoneInputs,
    ) -> Result<RuntimePendingCompiledScriptRun> {
        let next_script = self.take_next_script()?;
        let run = self.run_compiled_script_until_boundary(
            RuntimeCompiledScriptCursor {
                origin_map_name: next_script.origin_map_name.clone(),
                source_script: next_script.script.clone(),
                command_index: 0,
            },
            max_steps,
            inputs,
            phone_inputs,
        )?;
        Ok(RuntimePendingCompiledScriptRun { next_script, run })
    }

    pub fn run_next_deferred_script_until_boundary(
        &mut self,
        max_steps: usize,
        inputs: ScriptRuntimeInputs,
        phone_inputs: ScriptPhoneInputs,
    ) -> Result<RuntimeDeferredCompiledScriptRun> {
        let deferred_script = self.pop_deferred_script()?;
        let run = self.run_compiled_script_until_boundary(
            RuntimeCompiledScriptCursor {
                origin_map_name: deferred_script.origin_map_name.clone(),
                source_script: deferred_script.script.clone(),
                command_index: 0,
            },
            max_steps,
            inputs,
            phone_inputs,
        )?;
        Ok(RuntimeDeferredCompiledScriptRun {
            deferred_script,
            run,
        })
    }

    pub fn advance_text_wait_and_run_compiled_script(
        &mut self,
        next_cursor: Option<RuntimeCompiledScriptCursor>,
        max_steps: usize,
        inputs: ScriptRuntimeInputs,
        phone_inputs: ScriptPhoneInputs,
    ) -> Result<RuntimeTextWaitCompiledScriptRun> {
        let wait = self.advance_pending_text_wait()?;
        let run = if let Some(cursor) = next_cursor {
            self.run_compiled_script_until_boundary(cursor, max_steps, inputs, phone_inputs)?
        } else {
            empty_compiled_script_run()
        };
        Ok(RuntimeTextWaitCompiledScriptRun { wait, run })
    }

    pub fn resolve_yes_no_and_run_compiled_script(
        &mut self,
        accepted: bool,
        next_cursor: Option<RuntimeCompiledScriptCursor>,
        max_steps: usize,
        inputs: ScriptRuntimeInputs,
        phone_inputs: ScriptPhoneInputs,
    ) -> Result<RuntimeYesNoCompiledScriptRun> {
        let resolution = self.resolve_pending_yes_no(accepted)?;
        let run = if let Some(cursor) = next_cursor {
            self.run_compiled_script_until_boundary(cursor, max_steps, inputs, phone_inputs)?
        } else {
            empty_compiled_script_run()
        };
        Ok(RuntimeYesNoCompiledScriptRun { resolution, run })
    }

    pub fn resolve_phone_prompt_and_run_compiled_script(
        &mut self,
        source_script: &str,
        command_index: usize,
        inputs: ScriptRuntimeInputs,
        accepted: bool,
        max_steps: usize,
    ) -> Result<RuntimePhonePromptCompiledScriptRun> {
        let origin_map_name = self.session.overworld.map.name.clone();
        let step = self.step_compiled_script_command(
            &origin_map_name,
            source_script,
            command_index,
            inputs,
            ScriptPhoneInputs {
                accepted: Some(accepted),
            },
        )?;
        let RuntimeMutationResult::ScriptPhoneApplied(_) = &step.mutation.result else {
            anyhow::bail!(
                "compiled script command {source_script}:{command_index} did not resolve a phone prompt"
            );
        };
        let run = if let Some(cursor) = step.next_cursor.clone() {
            self.run_compiled_script_until_boundary(
                cursor,
                max_steps,
                ScriptRuntimeInputs::default(),
                ScriptPhoneInputs::default(),
            )?
        } else {
            empty_compiled_script_run()
        };
        Ok(RuntimePhonePromptCompiledScriptRun { step, run })
    }

    pub fn select_vertical_menu_option_and_run_compiled_script(
        &mut self,
        menu_id: impl Into<String>,
        source_script: impl Into<String>,
        verticalmenu_command_index: usize,
        option_index: usize,
        option: impl Into<String>,
        next_cursor: Option<RuntimeCompiledScriptCursor>,
        max_steps: usize,
        inputs: ScriptRuntimeInputs,
        phone_inputs: ScriptPhoneInputs,
    ) -> Result<RuntimeMenuSelectionCompiledScriptRun> {
        let selection = self.select_vertical_menu_option(
            menu_id,
            source_script,
            verticalmenu_command_index,
            option_index,
            option,
        )?;
        let run = if let Some(cursor) = next_cursor {
            self.run_compiled_script_until_boundary(cursor, max_steps, inputs, phone_inputs)?
        } else {
            empty_compiled_script_run()
        };
        Ok(RuntimeMenuSelectionCompiledScriptRun { selection, run })
    }

    pub fn select_elevator_floor_and_run_compiled_script(
        &mut self,
        map_name: impl Into<String>,
        data_label: impl Into<String>,
        source_script: impl Into<String>,
        elevator_command_index: usize,
        floor_index: usize,
        floor: impl Into<String>,
        warp: u16,
        target_map: impl Into<String>,
        next_cursor: Option<RuntimeCompiledScriptCursor>,
        max_steps: usize,
        inputs: ScriptRuntimeInputs,
        phone_inputs: ScriptPhoneInputs,
    ) -> Result<RuntimeElevatorFloorCompiledScriptRun> {
        let selection = self.select_elevator_floor(
            map_name,
            data_label,
            source_script,
            elevator_command_index,
            floor_index,
            floor,
            warp,
            target_map,
        )?;
        let run = if let Some(cursor) = next_cursor {
            self.run_compiled_script_until_boundary(cursor, max_steps, inputs, phone_inputs)?
        } else {
            empty_compiled_script_run()
        };
        Ok(RuntimeElevatorFloorCompiledScriptRun { selection, run })
    }

    pub fn transition_script_warp_and_run_compiled_script(
        &mut self,
        next_cursor: Option<RuntimeCompiledScriptCursor>,
        max_steps: usize,
        inputs: ScriptRuntimeInputs,
        phone_inputs: ScriptPhoneInputs,
    ) -> Result<RuntimeScriptWarpCompiledScriptRun> {
        let warp = self.execute_pending_script_warp()?;
        let run = if let Some(cursor) = next_cursor {
            self.run_compiled_script_until_boundary(cursor, max_steps, inputs, phone_inputs)?
        } else {
            empty_compiled_script_run()
        };
        Ok(RuntimeScriptWarpCompiledScriptRun { warp, run })
    }

    pub fn complete_scripted_wild_battle_and_run_compiled_script(
        &mut self,
        origin: RuntimeStaticWildBattleOrigin,
        max_steps: usize,
        inputs: ScriptRuntimeInputs,
        phone_inputs: ScriptPhoneInputs,
    ) -> Result<RuntimeScriptedWildBattleCompiledScriptRun> {
        let cursor = RuntimeCompiledScriptCursor {
            origin_map_name: origin.map_name.clone(),
            source_script: origin.source_script.clone(),
            command_index: origin.resume_command_index,
        };
        let completion = self.complete_scripted_wild_battle(origin)?;
        let run =
            self.run_compiled_script_until_boundary(cursor, max_steps, inputs, phone_inputs)?;
        Ok(RuntimeScriptedWildBattleCompiledScriptRun { completion, run })
    }

    pub fn complete_scripted_trainer_battle_and_run_compiled_script(
        &mut self,
        map_name: &str,
        source_script: &str,
        startbattle_command_index: usize,
        won: bool,
        can_lose: bool,
        max_steps: usize,
        inputs: ScriptRuntimeInputs,
        phone_inputs: ScriptPhoneInputs,
    ) -> Result<RuntimeScriptedTrainerBattleCompiledScriptRun> {
        let trainer_callback =
            self.snapshot()?
                .battle
                .as_ref()
                .and_then(|battle| match &battle.kind {
                    RuntimeBattleKind::Trainer { callback, .. } if !callback.is_empty() => {
                        Some(callback.clone())
                    }
                    _ => None,
                });
        let completion = self.complete_scripted_trainer_battle(
            map_name,
            source_script,
            startbattle_command_index,
            won,
            can_lose,
        )?;
        let run = if completion.continued_after_battle {
            let (resume_script, command_index) = if let Some(callback) = trainer_callback {
                (callback, 0)
            } else {
                (
                    source_script.to_string(),
                    startbattle_command_index
                        .checked_add(1)
                        .context("scripted trainer startbattle command index overflow")?,
                )
            };
            self.run_compiled_script_until_boundary(
                RuntimeCompiledScriptCursor {
                    origin_map_name: map_name.to_string(),
                    source_script: resume_script,
                    command_index,
                },
                max_steps,
                inputs,
                phone_inputs,
            )?
        } else {
            empty_compiled_script_run()
        };
        Ok(RuntimeScriptedTrainerBattleCompiledScriptRun { completion, run })
    }

    pub fn grant_compiled_gift_pokemon_command(
        &mut self,
        source_script: &str,
        command_index: usize,
        original_trainer_name: impl Into<String>,
        original_trainer_id: u16,
        nickname_accepted: bool,
        nickname: Option<String>,
    ) -> Result<RuntimeGiftPokemonGrant> {
        let map_name = self.runtime.script_owner_map(source_script)?;
        if !self
            .runtime
            .has_gift_pokemon_command_at(&map_name, source_script, command_index)
        {
            let command = self
                .runtime
                .compiled_script_command_name(source_script, command_index)?;
            anyhow::bail!(
                "compiled script command {}:{} '{}' is not a gift Pokemon command",
                source_script,
                command_index,
                command
            );
        }
        self.grant_scripted_gift_pokemon(
            &map_name,
            source_script,
            command_index,
            original_trainer_name,
            original_trainer_id,
            nickname_accepted,
            nickname,
        )
    }

    pub fn grant_compiled_gift_pokemon_command_and_run_compiled_script(
        &mut self,
        source_script: &str,
        command_index: usize,
        original_trainer_name: impl Into<String>,
        original_trainer_id: u16,
        nickname_accepted: bool,
        nickname: Option<String>,
        next_cursor: Option<RuntimeCompiledScriptCursor>,
        max_steps: usize,
        inputs: ScriptRuntimeInputs,
        phone_inputs: ScriptPhoneInputs,
    ) -> Result<RuntimeGiftPokemonCompiledScriptRun> {
        let grant = self.grant_compiled_gift_pokemon_command(
            source_script,
            command_index,
            original_trainer_name,
            original_trainer_id,
            nickname_accepted,
            nickname,
        )?;
        let run = if let Some(cursor) = next_cursor {
            self.run_compiled_script_until_boundary(cursor, max_steps, inputs, phone_inputs)?
        } else {
            empty_compiled_script_run()
        };
        Ok(RuntimeGiftPokemonCompiledScriptRun { grant, run })
    }

    pub fn apply_runtime_command_frame(
        &mut self,
        request: &RuntimeCommandFrame,
    ) -> Result<RuntimeMutationOutcome> {
        if request.player_id() != RUNTIME_LOCAL_PLAYER_ID {
            anyhow::bail!(
                "runtime command player {} does not match local player {}",
                request.player_id(),
                RUNTIME_LOCAL_PLAYER_ID
            );
        }
        let expected_sequence = self
            .runtime_command_sequence
            .checked_add(1)
            .context("runtime command sequence overflow")?;
        if request.sequence() != expected_sequence {
            anyhow::bail!(
                "runtime command sequence {} does not match next retained sequence {}",
                request.sequence(),
                expected_sequence
            );
        }
        self.require_valid_script_modal_state("apply runtime command frame")?;
        let previous_session = self.session.clone();
        let outcome = match self
            .session
            .apply_runtime_command_frame(&self.runtime, request)
        {
            Ok(outcome) => outcome,
            Err(error) => {
                self.session = previous_session;
                return Err(error);
            }
        };
        let result = match self
            .runtime_mutation_result_frame(request.clone(), &outcome)
            .context("build retained runtime command frame result")
        {
            Ok(result) => result,
            Err(error) => {
                self.session = previous_session;
                return Err(error);
            }
        };
        if let Err(error) = self.require_valid_script_modal_state("finish runtime command frame") {
            self.session = previous_session;
            return Err(error);
        }
        self.runtime_command_sequence = request.sequence();
        self.runtime_commands.push(request.clone());
        self.runtime_results.push(result);
        self.record_runtime_mutation_outcome(&outcome);
        Ok(outcome)
    }

    pub fn close_active_menu(&mut self) -> Result<RuntimeMenuClose> {
        self.require_valid_script_modal_state("close active menu")?;
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::CloseActiveMenu)?;
        let RuntimeMutationResult::ActiveMenuClosed(menu) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-menu-close result");
        };
        Ok(RuntimeMenuClose {
            menu,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn close_runtime_window(&mut self) -> Result<RuntimeWindowClose> {
        self.require_valid_script_modal_state("close runtime window")?;
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::CloseRuntimeWindow)?;
        let RuntimeMutationResult::RuntimeWindowClosed = mutation.result else {
            anyhow::bail!("runtime mutation returned non-runtime-window-close result");
        };
        Ok(RuntimeWindowClose {
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn close_text_window(&mut self) -> Result<RuntimeTextWindowClose> {
        self.require_valid_script_modal_state("close text window")?;
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::CloseTextWindow)?;
        let RuntimeMutationResult::TextWindowClosed = mutation.result else {
            anyhow::bail!("runtime mutation returned non-text-window-close result");
        };
        Ok(RuntimeTextWindowClose {
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn advance_pending_text_wait(&mut self) -> Result<RuntimeTextWaitAdvance> {
        self.require_valid_script_modal_state("advance pending text wait")?;
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::TakePendingScriptRequest(RuntimePendingScriptRequestCommand {
                kind: RuntimePendingScriptRequestKind::TextWait,
            }),
        )?;
        let RuntimeMutationResult::PendingScriptRequestTaken(
            RuntimePendingScriptRequest::TextWait(wait),
        ) = mutation.result
        else {
            anyhow::bail!("runtime mutation returned non-text-wait result");
        };
        Ok(RuntimeTextWaitAdvance {
            wait,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn resolve_pending_yes_no(&mut self, accepted: bool) -> Result<RuntimeYesNoResolution> {
        self.require_valid_script_modal_state("resolve pending yes/no")?;
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::ResolvePendingYesNo(
                RuntimePendingYesNoResolutionCommand { accepted },
            ))?;
        let RuntimeMutationResult::PendingYesNoResolved(resolution) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-yes-no-resolution result");
        };
        Ok(RuntimeYesNoResolution {
            prompt: resolution.prompt,
            accepted: resolution.accepted,
            script_value: resolution.script_value,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn open_vertical_menu(
        &mut self,
        map_name: impl Into<String>,
        menu_key: impl Into<String>,
        source_script: impl Into<String>,
        loadmenu_command_index: usize,
        verticalmenu_command_index: usize,
    ) -> Result<RuntimeVerticalMenuOpen> {
        self.require_valid_script_modal_state("open vertical menu")?;
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::OpenVerticalMenu(RuntimeVerticalMenuOpenCommand {
                map_name: map_name.into(),
                menu_key: menu_key.into(),
                source_script: source_script.into(),
                loadmenu_command_index,
                verticalmenu_command_index,
            }),
        )?;
        let RuntimeMutationResult::VerticalMenuOpened(opened) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-vertical-menu-open result");
        };
        Ok(RuntimeVerticalMenuOpen {
            map_name: opened.map_name,
            menu_key: opened.menu_key,
            menu_id: opened.menu_id,
            source_script: opened.source_script,
            loadmenu_command_index: opened.loadmenu_command_index,
            verticalmenu_command_index: opened.verticalmenu_command_index,
            options: opened.options,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn select_vertical_menu_option(
        &mut self,
        menu_id: impl Into<String>,
        source_script: impl Into<String>,
        verticalmenu_command_index: usize,
        option_index: usize,
        option: impl Into<String>,
    ) -> Result<RuntimeVerticalMenuOptionSelection> {
        self.require_valid_script_modal_state("select vertical menu option")?;
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::SelectVerticalMenuOption(RuntimeVerticalMenuSelectionCommand {
                menu_id: menu_id.into(),
                source_script: source_script.into(),
                verticalmenu_command_index,
                option_index,
                option: option.into(),
            }),
        )?;
        let RuntimeMutationResult::VerticalMenuOptionSelected(selection) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-vertical-menu-selection result");
        };
        Ok(RuntimeVerticalMenuOptionSelection {
            menu_id: selection.menu_id,
            source_script: selection.source_script,
            verticalmenu_command_index: selection.verticalmenu_command_index,
            option_index: selection.option_index,
            option: selection.option,
            script_value: selection.script_value,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn select_linked_vertical_menu_option(
        &mut self,
        descriptor: &RuntimeLinkSessionDescriptor,
        menu_id: impl Into<String>,
        source_script: impl Into<String>,
        verticalmenu_command_index: usize,
        option_index: usize,
        option: impl Into<String>,
    ) -> Result<RuntimeLinkedMenuChoice> {
        let menu_id = menu_id.into();
        let frame = self.linked_menu_choice_frame(
            descriptor,
            menu_id.clone(),
            verticalmenu_command_index,
            option_index,
        )?;
        let selection = self.select_vertical_menu_option(
            menu_id,
            source_script,
            verticalmenu_command_index,
            option_index,
            option,
        )?;
        Ok(RuntimeLinkedMenuChoice { frame, selection })
    }

    pub fn linked_menu_choice_frame(
        &self,
        descriptor: &RuntimeLinkSessionDescriptor,
        menu_id: impl Into<String>,
        verticalmenu_command_index: usize,
        option_index: usize,
    ) -> Result<MenuChoiceFrame> {
        MenuChoiceFrame::new(
            descriptor.local_player.id(),
            crystal_core::timing::Frame(self.session.state().frame_counter),
            menu_id.into(),
            option_index,
            verticalmenu_command_index,
        )
        .context("build runtime linked menu choice frame")
    }

    pub fn send_linked_vertical_menu_option<T: crystal_net::LinkTransport>(
        &mut self,
        endpoint: &mut crystal_net::LinkEndpoint<T>,
        descriptor: &RuntimeLinkSessionDescriptor,
        menu_id: impl Into<String>,
        source_script: impl Into<String>,
        verticalmenu_command_index: usize,
        option_index: usize,
        option: impl Into<String>,
    ) -> Result<RuntimeLinkedMenuChoice> {
        let menu_id = menu_id.into();
        let frame = self.linked_menu_choice_frame(
            descriptor,
            menu_id.clone(),
            verticalmenu_command_index,
            option_index,
        )?;
        endpoint
            .send(LinkMessage::MenuChoice(frame.clone()))
            .context("send runtime linked menu choice")?;
        let selection = self.select_vertical_menu_option(
            menu_id,
            source_script,
            verticalmenu_command_index,
            option_index,
            option,
        )?;
        Ok(RuntimeLinkedMenuChoice { frame, selection })
    }

    pub fn linked_menu_choice_result_frame(
        &self,
        descriptor: &RuntimeLinkSessionDescriptor,
        choice: &RuntimeLinkedMenuChoice,
    ) -> Result<MenuChoiceResultFrame> {
        MenuChoiceResultFrame::new(
            choice.frame.clone(),
            StateChecksumFrame::new(
                descriptor.local_player.id(),
                crystal_core::timing::Frame(choice.selection.state_checksum.frame()),
                choice.selection.state_checksum.hash(),
            ),
            choice.selection.script_value.clone(),
        )
        .context("build runtime linked menu choice result frame")
    }

    pub fn record_linked_menu_choice_result(
        &mut self,
        descriptor: &RuntimeLinkSessionDescriptor,
        choice: &RuntimeLinkedMenuChoice,
    ) -> Result<MenuChoiceResultFrame> {
        let result = self.linked_menu_choice_result_frame(descriptor, choice)?;
        self.linked_menu_results.push(result.clone());
        Ok(result)
    }

    pub fn send_linked_menu_choice_result<T: crystal_net::LinkTransport>(
        &mut self,
        endpoint: &mut crystal_net::LinkEndpoint<T>,
        descriptor: &RuntimeLinkSessionDescriptor,
        choice: &RuntimeLinkedMenuChoice,
    ) -> Result<MenuChoiceResultFrame> {
        let result = self.linked_menu_choice_result_frame(descriptor, choice)?;
        endpoint
            .send(LinkMessage::MenuChoiceResult(result.clone()))
            .context("send runtime linked menu choice result")?;
        self.linked_menu_results.push(result.clone());
        Ok(result)
    }

    pub fn select_elevator_floor(
        &mut self,
        map_name: impl Into<String>,
        data_label: impl Into<String>,
        source_script: impl Into<String>,
        elevator_command_index: usize,
        floor_index: usize,
        floor: impl Into<String>,
        warp: u16,
        target_map: impl Into<String>,
    ) -> Result<RuntimeElevatorFloorSelection> {
        self.require_valid_script_modal_state("select elevator floor")?;
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::SelectElevatorFloor(RuntimeElevatorFloorSelectionCommand {
                map_name: map_name.into(),
                data_label: data_label.into(),
                source_script: source_script.into(),
                elevator_command_index,
                floor_index,
                floor: floor.into(),
                warp,
                target_map: target_map.into(),
            }),
        )?;
        let RuntimeMutationResult::ElevatorFloorSelected(selection) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-elevator-floor-selection result");
        };
        Ok(RuntimeElevatorFloorSelection {
            map_name: selection.map_name,
            data_label: selection.data_label,
            source_script: selection.source_script,
            elevator_command_index: selection.elevator_command_index,
            floor_index: selection.floor_index,
            floor: selection.floor,
            warp: selection.warp,
            target_map: selection.target_map,
            destination_tile: selection.destination_tile,
            script_value: selection.script_value,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn close_active_pokemon_picture(&mut self) -> Result<RuntimePokemonPictureClose> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::CloseActivePokemonPicture)?;
        let RuntimeMutationResult::ActivePokemonPictureClosed(species_id) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-pokemon-picture-close result");
        };
        Ok(RuntimePokemonPictureClose {
            species_id,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn close_script_shop(&mut self) -> Result<RuntimeShopClose> {
        self.require_valid_script_modal_state("close script shop")?;
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::CloseScriptShop)?;
        let RuntimeMutationResult::ScriptShopClosed(shop) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-shop-close result");
        };
        Ok(RuntimeShopClose {
            shop,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn drain_script_event_queue(
        &mut self,
        queue: RuntimeScriptEventQueue,
    ) -> Result<RuntimeScriptEventDrainResult> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::DrainScriptEventQueue(RuntimeScriptEventDrainCommand { queue }),
        )?;
        let RuntimeMutationResult::ScriptEventQueueDrained(drained) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-event-drain result");
        };
        Ok(drained)
    }

    pub fn drain_audio_events(&mut self) -> Result<RuntimeAudioEventDrain> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::DrainScriptEventQueue(RuntimeScriptEventDrainCommand {
                queue: RuntimeScriptEventQueue::Audio,
            }),
        )?;
        let RuntimeMutationResult::ScriptEventQueueDrained(RuntimeScriptEventDrainResult::Audio(
            events,
        )) = mutation.result
        else {
            anyhow::bail!("runtime mutation returned non-audio-event-drain result");
        };
        Ok(RuntimeAudioEventDrain {
            events,
            state_checksum: mutation.state_checksum,
        })
    }

    pub fn drain_resolved_audio_events(&mut self) -> Result<RuntimeResolvedAudioEventDrain> {
        let drain = self.drain_audio_events()?;
        let events = self
            .runtime
            .audio()
            .resolve_audio_events(drain.events)
            .context("resolve drained runtime audio events")?;
        Ok(RuntimeResolvedAudioEventDrain {
            events,
            state_checksum: drain.state_checksum,
        })
    }

    pub fn drain_script_runtime_queue(
        &mut self,
        queue: RuntimeScriptRuntimeQueue,
    ) -> Result<RuntimeScriptRuntimeQueueDrainResult> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::DrainScriptRuntimeQueue(
                RuntimeScriptRuntimeQueueDrainCommand { queue },
            ))?;
        let RuntimeMutationResult::ScriptRuntimeQueueDrained(drained) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-runtime-queue-drain result");
        };
        Ok(drained)
    }

    pub fn take_pending_script_request(
        &mut self,
        kind: RuntimePendingScriptRequestKind,
    ) -> Result<RuntimePendingScriptRequest> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::TakePendingScriptRequest(
                RuntimePendingScriptRequestCommand { kind },
            ))?;
        let RuntimeMutationResult::PendingScriptRequestTaken(request) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-pending-script-request result");
        };
        Ok(request)
    }

    pub fn consume_script_runtime_flag(
        &mut self,
        flag: RuntimeScriptRuntimeFlag,
    ) -> Result<RuntimeScriptRuntimeFlagValue> {
        let mutation =
            self.apply_runtime_mutation_command(RuntimeMutationCommand::ConsumeScriptRuntimeFlag(
                RuntimeScriptRuntimeFlagCommand { flag },
            ))?;
        let RuntimeMutationResult::ScriptRuntimeFlagConsumed(value) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-runtime-flag result");
        };
        Ok(value)
    }

    pub fn take_script_runtime_memory_value(
        &mut self,
        value: RuntimeScriptRuntimeMemoryValue,
    ) -> Result<RuntimeScriptRuntimeMemoryValueTaken> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::TakeScriptRuntimeMemoryValue(
                RuntimeScriptRuntimeMemoryValueCommand { value },
            ),
        )?;
        let RuntimeMutationResult::ScriptRuntimeMemoryValueTaken(value) = mutation.result else {
            anyhow::bail!("runtime mutation returned non-script-runtime-memory-value result");
        };
        Ok(value)
    }

    pub fn remove_script_runtime_memory_entry(
        &mut self,
        entry: RuntimeScriptRuntimeMemoryEntry,
        key: impl Into<String>,
    ) -> Result<RuntimeScriptRuntimeMemoryEntryRemoved> {
        let mutation = self.apply_runtime_mutation_command(
            RuntimeMutationCommand::RemoveScriptRuntimeMemoryEntry(
                RuntimeScriptRuntimeMemoryEntryCommand {
                    entry,
                    key: key.into(),
                },
            ),
        )?;
        let RuntimeMutationResult::ScriptRuntimeMemoryEntryRemoved(removed) = mutation.result
        else {
            anyhow::bail!("runtime mutation returned non-script-runtime-memory-entry result");
        };
        Ok(removed)
    }

    pub fn special_routine_ids(&self) -> BTreeSet<String> {
        self.runtime.special_routine_ids()
    }

    pub fn item_ids(&self) -> BTreeSet<String> {
        self.runtime.item_ids()
    }

    pub fn move_ids(&self) -> BTreeSet<String> {
        self.runtime.move_ids()
    }

    pub fn species_ids(&self) -> BTreeSet<String> {
        self.runtime.species_ids()
    }

    pub fn map_ids(&self) -> BTreeSet<String> {
        self.runtime.map_ids()
    }

    pub fn trainer_ids(&self) -> BTreeSet<String> {
        self.runtime.trainer_ids()
    }

    pub fn text_ids(&self) -> BTreeSet<String> {
        self.runtime.text_ids()
    }

    pub fn menu_ids(&self) -> BTreeSet<String> {
        self.runtime.menu_ids()
    }

    pub fn phone_contact_ids(&self) -> BTreeSet<String> {
        self.runtime.phone_contact_ids()
    }

    pub fn special_phone_call_ids(&self) -> BTreeSet<String> {
        self.runtime.special_phone_call_ids()
    }

    pub fn npc_trade_ids(&self) -> BTreeSet<String> {
        self.runtime.npc_trade_ids()
    }

    pub fn sprite_ids(&self) -> BTreeSet<String> {
        self.runtime.sprite_ids()
    }

    pub fn map_constants(&self) -> BTreeSet<String> {
        self.runtime.map_constants()
    }

    pub fn event_flag_ids(&self) -> BTreeSet<String> {
        self.runtime.event_flag_ids()
    }

    pub fn engine_flag_ids(&self) -> BTreeSet<String> {
        self.runtime.engine_flag_ids()
    }

    pub fn spawn_identifiers(&self) -> BTreeSet<u16> {
        self.runtime.spawn_identifiers()
    }

    pub fn tileset_ids(&self) -> BTreeSet<String> {
        self.runtime.tileset_ids()
    }

    pub fn tileset_keys(&self) -> BTreeSet<RuntimeTilesetKey> {
        self.runtime.tileset_keys()
    }

    pub fn pc_string_keys(&self) -> BTreeSet<RuntimePcStringKey> {
        self.runtime.pc_string_keys()
    }

    pub fn menu_icon_keys(&self) -> BTreeSet<RuntimeMenuIconKey> {
        self.runtime.menu_icon_keys()
    }

    pub fn pokedex_entry_keys(&self) -> BTreeSet<RuntimePokedexEntryKey> {
        self.runtime.pokedex_entry_keys()
    }

    pub fn landmark_ids(&self) -> BTreeSet<String> {
        self.runtime.landmark_ids()
    }

    pub fn pokegear_landmark_keys(&self) -> BTreeSet<RuntimePokegearLandmarkKey> {
        self.runtime.pokegear_landmark_keys()
    }

    pub fn pokegear_map_landmark_keys(&self) -> BTreeSet<RuntimePokegearMapLandmarkKey> {
        self.runtime.pokegear_map_landmark_keys()
    }

    pub fn fishing_rod_ids(&self) -> BTreeSet<String> {
        self.runtime.fishing_rod_ids()
    }

    pub fn map_group_ids(&self) -> BTreeSet<String> {
        self.runtime.map_group_ids()
    }

    pub fn encounter_group_ids(&self) -> BTreeSet<String> {
        self.runtime.encounter_group_ids()
    }

    pub fn mart_ids(&self) -> BTreeSet<String> {
        self.runtime.mart_ids()
    }

    pub fn mart_keys(&self) -> BTreeSet<RuntimeMartKey> {
        self.runtime.mart_keys()
    }

    pub fn fruit_tree_ids(&self) -> BTreeSet<String> {
        self.runtime.fruit_tree_ids()
    }

    pub fn fruit_tree_keys(&self) -> BTreeSet<RuntimeFruitTreeKey> {
        self.runtime.fruit_tree_keys()
    }

    pub fn field_move_rule_ids(&self) -> BTreeSet<String> {
        self.runtime.field_move_rule_ids()
    }

    pub fn field_move_rule_keys(&self) -> BTreeSet<RuntimeFieldMoveRuleKey> {
        self.runtime.field_move_rule_keys()
    }

    pub fn fly_destination_ids(&self) -> BTreeSet<String> {
        self.runtime.fly_destination_ids()
    }

    pub fn fly_destination_keys(&self) -> BTreeSet<RuntimeFlyDestinationKey> {
        self.runtime.fly_destination_keys()
    }

    pub fn field_move_move_ids(&self) -> BTreeSet<String> {
        self.runtime.field_move_move_ids()
    }

    pub fn field_move_item_ids(&self) -> BTreeSet<String> {
        self.runtime.field_move_item_ids()
    }

    pub fn flee_mon_bucket_ids(&self) -> BTreeSet<String> {
        self.runtime.flee_mon_bucket_ids()
    }

    pub fn buena_password_category_ids(&self) -> BTreeSet<String> {
        self.runtime.buena_password_category_ids()
    }

    pub fn roaming_species_ids(&self) -> BTreeSet<String> {
        self.runtime.roaming_species_ids()
    }

    pub fn buena_prize_item_ids(&self) -> BTreeSet<String> {
        self.runtime.buena_prize_item_ids()
    }

    pub fn kurt_apricorn_item_ids(&self) -> BTreeSet<String> {
        self.runtime.kurt_apricorn_item_ids()
    }

    pub fn dratini_move_set_ids(&self) -> BTreeSet<u8> {
        self.runtime.dratini_move_set_ids()
    }

    pub fn special_feature_ids(&self) -> BTreeSet<String> {
        self.runtime.special_feature_ids()
    }

    pub fn oak_rating_text_ids(&self) -> BTreeSet<String> {
        self.runtime.oak_rating_text_ids()
    }

    pub fn odd_egg_species_ids(&self) -> BTreeSet<String> {
        self.runtime.odd_egg_species_ids()
    }

    pub fn magikarp_length_thresholds(&self) -> BTreeSet<u16> {
        self.runtime.magikarp_length_thresholds()
    }

    pub fn happiness_change_ids(&self) -> BTreeSet<u8> {
        self.runtime.happiness_change_ids()
    }

    pub fn happiness_service_ids(&self) -> BTreeSet<String> {
        self.runtime.happiness_service_ids()
    }

    pub fn pokemon_status_ids(&self) -> BTreeSet<String> {
        self.runtime.pokemon_status_ids()
    }

    pub fn fishing_daily_flag_bits(&self) -> BTreeSet<u32> {
        self.runtime.fishing_daily_flag_bits()
    }

    pub fn fishing_swarm_flags(&self) -> BTreeSet<u8> {
        self.runtime.fishing_swarm_flags()
    }

    pub fn pending_special_battle_type_ids(&self) -> BTreeSet<String> {
        self.runtime.pending_special_battle_type_ids()
    }

    pub fn scripted_trainer_battle_keys(&self) -> BTreeSet<RuntimeScriptedTrainerBattleKey> {
        self.runtime.scripted_trainer_battle_keys()
    }

    pub fn wild_encounter_origin_keys(&self) -> BTreeSet<RuntimeWildEncounterOriginKey> {
        self.runtime.wild_encounter_origin_keys()
    }

    pub fn script_label_ids(&self) -> BTreeSet<String> {
        self.runtime.script_label_ids()
    }

    pub fn script_command_keys(&self) -> BTreeSet<RuntimeScriptCommandKey> {
        self.runtime.script_command_keys()
    }

    pub fn script_command_payload_keys(&self) -> BTreeSet<RuntimeScriptCommandPayloadKey> {
        self.runtime.script_command_payload_keys()
    }

    pub fn script_return_keys(&self) -> BTreeSet<RuntimeScriptReturnKey> {
        self.runtime.script_return_keys()
    }

    pub fn script_vertical_menu_keys(&self) -> BTreeSet<RuntimeScriptVerticalMenuKey> {
        self.runtime.script_vertical_menu_keys()
    }

    pub fn script_text_body_keys(&self) -> BTreeSet<RuntimeScriptTextBodyKey> {
        self.runtime.script_text_body_keys()
    }

    pub fn script_menu_definition_keys(&self) -> BTreeSet<RuntimeScriptMenuDefinitionKey> {
        self.runtime.script_menu_definition_keys()
    }

    pub fn script_elevator_keys(&self) -> BTreeSet<RuntimeScriptElevatorKey> {
        self.runtime.script_elevator_keys()
    }

    pub fn gift_pokemon_keys(&self) -> BTreeSet<RuntimeGiftPokemonKey> {
        self.runtime.gift_pokemon_keys()
    }

    pub fn script_object_command_keys(&self) -> BTreeSet<RuntimeScriptObjectCommandKey> {
        self.runtime.script_object_command_keys()
    }

    pub fn script_movement_keys(&self) -> BTreeSet<RuntimeScriptMovementKey> {
        self.runtime.script_movement_keys()
    }

    pub fn map_script_section_command_keys(&self) -> BTreeSet<RuntimeMapScriptSectionCommandKey> {
        self.runtime.map_script_section_command_keys()
    }

    pub fn map_event_section_command_keys(&self) -> BTreeSet<RuntimeMapEventSectionCommandKey> {
        self.runtime.map_event_section_command_keys()
    }

    pub fn script_map_command_keys(&self) -> BTreeSet<RuntimeScriptMapCommandKey> {
        self.runtime.script_map_command_keys()
    }

    pub fn script_variable_command_keys(&self) -> BTreeSet<RuntimeScriptVariableCommandKey> {
        self.runtime.script_variable_command_keys()
    }

    pub fn script_control_command_keys(&self) -> BTreeSet<RuntimeScriptControlCommandKey> {
        self.runtime.script_control_command_keys()
    }

    pub fn script_swarm_command_keys(&self) -> BTreeSet<RuntimeScriptSwarmCommandKey> {
        self.runtime.script_swarm_command_keys()
    }

    pub fn script_field_pickup_keys(&self) -> BTreeSet<RuntimeScriptFieldPickupKey> {
        self.runtime.script_field_pickup_keys()
    }

    pub fn script_shop_command_keys(&self) -> BTreeSet<RuntimeScriptShopCommandKey> {
        self.runtime.script_shop_command_keys()
    }

    pub fn script_phone_command_keys(&self) -> BTreeSet<RuntimeScriptPhoneCommandKey> {
        self.runtime.script_phone_command_keys()
    }

    pub fn script_runtime_command_keys(&self) -> BTreeSet<RuntimeScriptRuntimeCommandKey> {
        self.runtime.script_runtime_command_keys()
    }

    pub fn script_runtime_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Option<RuntimeScriptRuntimeCommandKey> {
        self.runtime
            .script_runtime_command_at(map_name, source_script, command_index)
    }

    pub fn script_item_grant_keys(&self) -> BTreeSet<RuntimeScriptItemGrantKey> {
        self.runtime.script_item_grant_keys()
    }

    pub fn script_item_access_keys(&self) -> BTreeSet<RuntimeScriptItemAccessKey> {
        self.runtime.script_item_access_keys()
    }

    pub fn script_economy_command_keys(&self) -> BTreeSet<RuntimeScriptEconomyCommandKey> {
        self.runtime.script_economy_command_keys()
    }

    pub fn script_flag_command_keys(&self) -> BTreeSet<RuntimeScriptFlagCommandKey> {
        self.runtime.script_flag_command_keys()
    }

    pub fn script_scene_command_keys(&self) -> BTreeSet<RuntimeScriptSceneCommandKey> {
        self.runtime.script_scene_command_keys()
    }

    pub fn script_block_change_keys(&self) -> BTreeSet<RuntimeScriptBlockChangeKey> {
        self.runtime.script_block_change_keys()
    }

    pub fn script_audio_command_keys(&self) -> BTreeSet<RuntimeScriptAudioCommandKey> {
        self.runtime.script_audio_command_keys()
    }

    pub fn script_text_command_keys(&self) -> BTreeSet<RuntimeScriptTextCommandKey> {
        self.runtime.script_text_command_keys()
    }

    pub fn warp_keys(&self) -> BTreeSet<RuntimeWarpKey> {
        self.runtime.warp_keys()
    }

    pub fn map_object_keys(&self) -> BTreeSet<RuntimeMapObjectKey> {
        self.runtime.map_object_keys()
    }

    pub fn map_scene_keys(&self) -> BTreeSet<RuntimeMapSceneKey> {
        self.runtime.map_scene_keys()
    }

    pub fn map_metadata_keys(&self) -> BTreeSet<RuntimeMapMetadataKey> {
        self.runtime.map_metadata_keys()
    }

    pub fn currency_constant_ids(&self) -> BTreeSet<String> {
        self.runtime.currency_constant_ids()
    }

    pub fn capture_ball_rule_ids(&self) -> BTreeSet<String> {
        self.runtime.capture_ball_rule_ids()
    }

    pub fn guaranteed_capture_ball_ids(&self) -> BTreeSet<String> {
        self.runtime.guaranteed_capture_ball_ids()
    }

    pub fn capture_status_bonus_ids(&self) -> BTreeSet<String> {
        self.runtime.capture_status_bonus_ids()
    }

    pub fn fast_ball_species_ids(&self) -> BTreeSet<String> {
        self.runtime.fast_ball_species_ids()
    }

    pub fn heavy_ball_species_ids(&self) -> BTreeSet<String> {
        self.runtime.heavy_ball_species_ids()
    }

    pub fn move_priority_effect_ids(&self) -> BTreeSet<String> {
        self.runtime.move_priority_effect_ids()
    }

    pub fn move_priority_move_ids(&self) -> BTreeSet<String> {
        self.runtime.move_priority_move_ids()
    }

    pub fn capture_ball_rule_keys(&self) -> BTreeSet<RuntimeCaptureBallRuleKey> {
        self.runtime.capture_ball_rule_keys()
    }

    pub fn heavy_ball_modifier_keys(&self) -> BTreeSet<RuntimeHeavyBallModifierKey> {
        self.runtime.heavy_ball_modifier_keys()
    }

    pub fn capture_status_bonus_keys(&self) -> BTreeSet<RuntimeCaptureStatusBonusKey> {
        self.runtime.capture_status_bonus_keys()
    }

    pub fn capture_wobble_probability_keys(&self) -> BTreeSet<RuntimeCaptureWobbleProbabilityKey> {
        self.runtime.capture_wobble_probability_keys()
    }

    pub fn item_battle_use_keys(&self) -> BTreeSet<RuntimeItemBattleUseKey> {
        self.runtime.item_battle_use_keys()
    }

    pub fn item_effect_plan_keys(&self) -> BTreeSet<RuntimeItemEffectPlanKey> {
        self.runtime.item_effect_plan_keys()
    }

    pub fn item_field_use_keys(&self) -> BTreeSet<RuntimeItemFieldUseKey> {
        self.runtime.item_field_use_keys()
    }

    pub fn move_battle_data_keys(&self) -> BTreeSet<RuntimeMoveBattleDataKey> {
        self.runtime.move_battle_data_keys()
    }

    pub fn species_battle_data_keys(&self) -> BTreeSet<RuntimeSpeciesBattleDataKey> {
        self.runtime.species_battle_data_keys()
    }

    pub fn species_learnset_keys(&self) -> BTreeSet<RuntimeSpeciesLearnsetKey> {
        self.runtime.species_learnset_keys()
    }

    pub fn species_evolution_keys(&self) -> BTreeSet<RuntimeSpeciesEvolutionKey> {
        self.runtime.species_evolution_keys()
    }

    pub fn trainer_battle_data_keys(&self) -> BTreeSet<RuntimeTrainerBattleDataKey> {
        self.runtime.trainer_battle_data_keys()
    }

    pub fn trainer_party_pokemon_keys(&self) -> BTreeSet<RuntimeTrainerPartyPokemonKey> {
        self.runtime.trainer_party_pokemon_keys()
    }

    pub fn move_priority_effect_keys(&self) -> BTreeSet<RuntimeMovePriorityEffectKey> {
        self.runtime.move_priority_effect_keys()
    }

    pub fn move_priority_move_keys(&self) -> BTreeSet<RuntimeMovePriorityMoveKey> {
        self.runtime.move_priority_move_keys()
    }

    pub fn battle_stat_multiplier_keys(&self) -> BTreeSet<RuntimeBattleStatMultiplierKey> {
        self.runtime.battle_stat_multiplier_keys()
    }

    pub fn battle_reward_rule_keys(&self) -> BTreeSet<RuntimeBattleRewardRuleKey> {
        self.runtime.battle_reward_rule_keys()
    }

    pub fn battle_escape_rule_keys(&self) -> BTreeSet<RuntimeBattleEscapeRuleKey> {
        self.runtime.battle_escape_rule_keys()
    }

    pub fn physical_type_ids(&self) -> BTreeSet<String> {
        self.runtime.physical_type_ids()
    }

    pub fn special_type_ids(&self) -> BTreeSet<String> {
        self.runtime.special_type_ids()
    }

    pub fn weather_ids(&self) -> BTreeSet<String> {
        self.runtime.weather_ids()
    }

    pub fn type_effectiveness_keys(&self) -> BTreeSet<RuntimeTypeEffectivenessKey> {
        self.runtime.type_effectiveness_keys()
    }

    pub fn foresight_type_effectiveness_keys(&self) -> BTreeSet<RuntimeTypeEffectivenessKey> {
        self.runtime.foresight_type_effectiveness_keys()
    }

    pub fn weather_type_modifier_keys(&self) -> BTreeSet<RuntimeWeatherTypeModifierKey> {
        self.runtime.weather_type_modifier_keys()
    }

    pub fn weather_move_effect_modifier_keys(
        &self,
    ) -> BTreeSet<RuntimeWeatherMoveEffectModifierKey> {
        self.runtime.weather_move_effect_modifier_keys()
    }

    pub fn music_ids(&self) -> BTreeSet<String> {
        self.runtime.music_ids()
    }

    pub fn sound_effect_ids(&self) -> BTreeSet<String> {
        self.runtime.sound_effect_ids()
    }

    pub fn cry_ids(&self) -> BTreeSet<String> {
        self.runtime.cry_ids()
    }

    pub fn pokemon_cry_keys(&self) -> BTreeSet<RuntimePokemonCryKey> {
        self.runtime.pokemon_cry_keys()
    }

    pub fn audio_asset_keys(&self) -> BTreeSet<RuntimeAudioAssetKey> {
        self.runtime.audio_asset_keys()
    }

    pub fn has_special_routine(&self, routine: &str) -> bool {
        self.runtime.has_special_routine(routine)
    }

    pub fn has_item(&self, item_id: &str) -> bool {
        self.runtime.has_item(item_id)
    }

    pub fn has_move(&self, move_id: &str) -> bool {
        self.runtime.has_move(move_id)
    }

    pub fn has_species(&self, species_id: &str) -> bool {
        self.runtime.has_species(species_id)
    }

    pub fn has_map(&self, map_name: &str) -> bool {
        self.runtime.has_map(map_name)
    }

    pub fn has_trainer(&self, trainer_id: &str) -> bool {
        self.runtime.has_trainer(trainer_id)
    }

    pub fn has_text(&self, text_label: &str) -> bool {
        self.runtime.has_text(text_label)
    }

    pub fn has_menu(&self, menu: &str) -> bool {
        self.runtime.has_menu(menu)
    }

    pub fn has_phone_contact(&self, contact_id: &str) -> bool {
        self.runtime.has_phone_contact(contact_id)
    }

    pub fn has_special_phone_call(&self, call_id: &str) -> bool {
        self.runtime.has_special_phone_call(call_id)
    }

    pub fn has_npc_trade(&self, trade_id: &str) -> bool {
        self.runtime.has_npc_trade(trade_id)
    }

    pub fn has_sprite(&self, sprite_id: &str) -> bool {
        self.runtime.has_sprite(sprite_id)
    }

    pub fn has_map_constant(&self, map_constant: &str) -> bool {
        self.runtime.has_map_constant(map_constant)
    }

    pub fn has_event_flag(&self, flag: &str) -> bool {
        self.runtime.has_event_flag(flag)
    }

    pub fn has_engine_flag(&self, flag: &str) -> bool {
        self.runtime.has_engine_flag(flag)
    }

    pub fn has_spawn_identifier(&self, spawn_identifier: u16) -> bool {
        self.runtime.has_spawn_identifier(spawn_identifier)
    }

    pub fn has_tileset(&self, tileset_id: &str) -> bool {
        self.runtime.has_tileset(tileset_id)
    }

    pub fn has_tileset_row(&self, key: &RuntimeTilesetKey) -> bool {
        self.runtime.has_tileset_row(key)
    }

    pub fn has_pc_string(&self, key: &RuntimePcStringKey) -> bool {
        self.runtime.has_pc_string(key)
    }

    pub fn has_menu_icon(&self, key: &RuntimeMenuIconKey) -> bool {
        self.runtime.has_menu_icon(key)
    }

    pub fn has_pokedex_entry(&self, key: &RuntimePokedexEntryKey) -> bool {
        self.runtime.has_pokedex_entry(key)
    }

    pub fn has_landmark(&self, landmark_id: &str) -> bool {
        self.runtime.has_landmark(landmark_id)
    }

    pub fn has_pokegear_landmark(&self, key: &RuntimePokegearLandmarkKey) -> bool {
        self.runtime.has_pokegear_landmark(key)
    }

    pub fn has_pokegear_map_landmark(&self, key: &RuntimePokegearMapLandmarkKey) -> bool {
        self.runtime.has_pokegear_map_landmark(key)
    }

    pub fn has_fishing_rod(&self, rod: &str) -> bool {
        self.runtime.has_fishing_rod(rod)
    }

    pub fn has_map_group(&self, group_id: &str) -> bool {
        self.runtime.has_map_group(group_id)
    }

    pub fn has_encounter_group(&self, group_id: &str) -> bool {
        self.runtime.has_encounter_group(group_id)
    }

    pub fn has_mart(&self, mart_id: &str) -> bool {
        self.runtime.has_mart(mart_id)
    }

    pub fn has_mart_row(&self, key: &RuntimeMartKey) -> bool {
        self.runtime.has_mart_row(key)
    }

    pub fn has_fruit_tree(&self, fruit_tree_id: &str) -> bool {
        self.runtime.has_fruit_tree(fruit_tree_id)
    }

    pub fn has_fruit_tree_row(&self, key: &RuntimeFruitTreeKey) -> bool {
        self.runtime.has_fruit_tree_row(key)
    }

    pub fn has_field_move_rule(&self, rule_id: &str) -> bool {
        self.runtime.has_field_move_rule(rule_id)
    }

    pub fn has_field_move_rule_row(&self, key: &RuntimeFieldMoveRuleKey) -> bool {
        self.runtime.has_field_move_rule_row(key)
    }

    pub fn has_fly_destination(&self, flypoint_flag: &str) -> bool {
        self.runtime.has_fly_destination(flypoint_flag)
    }

    pub fn has_fly_destination_row(&self, key: &RuntimeFlyDestinationKey) -> bool {
        self.runtime.has_fly_destination_row(key)
    }

    pub fn has_field_move_move(&self, move_id: &str) -> bool {
        self.runtime.has_field_move_move(move_id)
    }

    pub fn has_field_move_item(&self, item_id: &str) -> bool {
        self.runtime.has_field_move_item(item_id)
    }

    pub fn has_flee_mon_bucket(&self, bucket_id: &str) -> bool {
        self.runtime.has_flee_mon_bucket(bucket_id)
    }

    pub fn has_buena_password_category(&self, category_id: &str) -> bool {
        self.runtime.has_buena_password_category(category_id)
    }

    pub fn has_roaming_species(&self, species_id: &str) -> bool {
        self.runtime.has_roaming_species(species_id)
    }

    pub fn has_buena_prize_item(&self, item_id: &str) -> bool {
        self.runtime.has_buena_prize_item(item_id)
    }

    pub fn has_kurt_apricorn_item(&self, item_id: &str) -> bool {
        self.runtime.has_kurt_apricorn_item(item_id)
    }

    pub fn has_dratini_move_set(&self, answer: u8) -> bool {
        self.runtime.has_dratini_move_set(answer)
    }

    pub fn has_special_feature(&self, feature_id: &str) -> bool {
        self.runtime.has_special_feature(feature_id)
    }

    pub fn has_oak_rating_text(&self, text_id: &str) -> bool {
        self.runtime.has_oak_rating_text(text_id)
    }

    pub fn has_odd_egg_species(&self, species_id: &str) -> bool {
        self.runtime.has_odd_egg_species(species_id)
    }

    pub fn has_magikarp_length_threshold(&self, threshold: u16) -> bool {
        self.runtime.has_magikarp_length_threshold(threshold)
    }

    pub fn has_happiness_change(&self, change_id: u8) -> bool {
        self.runtime.has_happiness_change(change_id)
    }

    pub fn has_happiness_service(&self, service_id: &str) -> bool {
        self.runtime.has_happiness_service(service_id)
    }

    pub fn has_pokemon_status(&self, status: &str) -> bool {
        self.runtime.has_pokemon_status(status)
    }

    pub fn has_fishing_daily_flag_bit(&self, bit: u32) -> bool {
        self.runtime.has_fishing_daily_flag_bit(bit)
    }

    pub fn has_fishing_swarm_flag(&self, swarm_flag: u8) -> bool {
        self.runtime.has_fishing_swarm_flag(swarm_flag)
    }

    pub fn has_pending_special_battle_type(&self, battle_type: &str) -> bool {
        self.runtime.has_pending_special_battle_type(battle_type)
    }

    pub fn has_wild_encounter_origin(&self, key: &RuntimeWildEncounterOriginKey) -> bool {
        self.runtime.has_wild_encounter_origin(key)
    }

    pub fn has_script_label(&self, script_label: &str) -> bool {
        self.runtime.has_script_label(script_label)
    }

    pub fn has_script_command(&self, key: &RuntimeScriptCommandKey) -> bool {
        self.runtime.has_script_command(key)
    }

    pub fn has_script_command_payload(&self, key: &RuntimeScriptCommandPayloadKey) -> bool {
        self.runtime.has_script_command_payload(key)
    }

    pub fn has_script_return(&self, key: &RuntimeScriptReturnKey) -> bool {
        self.runtime.has_script_return(key)
    }

    pub fn has_script_vertical_menu(&self, key: &RuntimeScriptVerticalMenuKey) -> bool {
        self.runtime.has_script_vertical_menu(key)
    }

    pub fn has_script_text_body(&self, key: &RuntimeScriptTextBodyKey) -> bool {
        self.runtime.has_script_text_body(key)
    }

    pub fn has_script_menu_definition(&self, key: &RuntimeScriptMenuDefinitionKey) -> bool {
        self.runtime.has_script_menu_definition(key)
    }

    pub fn has_script_elevator(&self, key: &RuntimeScriptElevatorKey) -> bool {
        self.runtime.has_script_elevator(key)
    }

    pub fn has_script_elevator_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.runtime
            .has_script_elevator_command_at(map_name, source_script, command_index)
    }

    pub fn has_gift_pokemon(&self, key: &RuntimeGiftPokemonKey) -> bool {
        self.runtime.has_gift_pokemon(key)
    }

    pub fn has_gift_pokemon_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.runtime
            .has_gift_pokemon_command_at(map_name, source_script, command_index)
    }

    pub fn has_script_phone_prompt_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.runtime
            .has_script_phone_prompt_command_at(map_name, source_script, command_index)
    }

    pub fn has_script_object_command(&self, key: &RuntimeScriptObjectCommandKey) -> bool {
        self.runtime.has_script_object_command(key)
    }

    pub fn has_script_movement(&self, key: &RuntimeScriptMovementKey) -> bool {
        self.runtime.has_script_movement(key)
    }

    pub fn has_map_script_section_command(&self, key: &RuntimeMapScriptSectionCommandKey) -> bool {
        self.runtime.has_map_script_section_command(key)
    }

    pub fn has_map_event_section_command(&self, key: &RuntimeMapEventSectionCommandKey) -> bool {
        self.runtime.has_map_event_section_command(key)
    }

    pub fn has_script_map_command(&self, key: &RuntimeScriptMapCommandKey) -> bool {
        self.runtime.has_script_map_command(key)
    }

    pub fn has_script_variable_command(&self, key: &RuntimeScriptVariableCommandKey) -> bool {
        self.runtime.has_script_variable_command(key)
    }

    pub fn has_script_control_command(&self, key: &RuntimeScriptControlCommandKey) -> bool {
        self.runtime.has_script_control_command(key)
    }

    pub fn has_script_swarm_command(&self, key: &RuntimeScriptSwarmCommandKey) -> bool {
        self.runtime.has_script_swarm_command(key)
    }

    pub fn has_script_field_pickup(&self, key: &RuntimeScriptFieldPickupKey) -> bool {
        self.runtime.has_script_field_pickup(key)
    }

    pub fn has_script_shop_command(&self, key: &RuntimeScriptShopCommandKey) -> bool {
        self.runtime.has_script_shop_command(key)
    }

    pub fn has_script_phone_command(&self, key: &RuntimeScriptPhoneCommandKey) -> bool {
        self.runtime.has_script_phone_command(key)
    }

    pub fn has_script_runtime_command(&self, key: &RuntimeScriptRuntimeCommandKey) -> bool {
        self.runtime.has_script_runtime_command(key)
    }

    pub fn has_script_item_grant(&self, key: &RuntimeScriptItemGrantKey) -> bool {
        self.runtime.has_script_item_grant(key)
    }

    pub fn has_script_item_access(&self, key: &RuntimeScriptItemAccessKey) -> bool {
        self.runtime.has_script_item_access(key)
    }

    pub fn has_script_economy_command(&self, key: &RuntimeScriptEconomyCommandKey) -> bool {
        self.runtime.has_script_economy_command(key)
    }

    pub fn has_script_flag_command(&self, key: &RuntimeScriptFlagCommandKey) -> bool {
        self.runtime.has_script_flag_command(key)
    }

    pub fn has_script_scene_command(&self, key: &RuntimeScriptSceneCommandKey) -> bool {
        self.runtime.has_script_scene_command(key)
    }

    pub fn has_script_block_change(&self, key: &RuntimeScriptBlockChangeKey) -> bool {
        self.runtime.has_script_block_change(key)
    }

    pub fn has_script_audio_command(&self, key: &RuntimeScriptAudioCommandKey) -> bool {
        self.runtime.has_script_audio_command(key)
    }

    pub fn has_script_text_command(&self, key: &RuntimeScriptTextCommandKey) -> bool {
        self.runtime.has_script_text_command(key)
    }

    pub fn has_warp(&self, key: &RuntimeWarpKey) -> bool {
        self.runtime.has_warp(key)
    }

    pub fn has_map_object(&self, key: &RuntimeMapObjectKey) -> bool {
        self.runtime.has_map_object(key)
    }

    pub fn has_map_scene(&self, key: &RuntimeMapSceneKey) -> bool {
        self.runtime.has_map_scene(key)
    }

    pub fn has_map_metadata(&self, key: &RuntimeMapMetadataKey) -> bool {
        self.runtime.has_map_metadata(key)
    }

    pub fn has_currency_constant(&self, id: &str) -> bool {
        self.runtime.has_currency_constant(id)
    }

    pub fn has_capture_ball_rule(&self, id: &str) -> bool {
        self.runtime.has_capture_ball_rule(id)
    }

    pub fn has_guaranteed_capture_ball(&self, id: &str) -> bool {
        self.runtime.has_guaranteed_capture_ball(id)
    }

    pub fn has_capture_status_bonus(&self, status: &str) -> bool {
        self.runtime.has_capture_status_bonus(status)
    }

    pub fn has_fast_ball_species(&self, species_id: &str) -> bool {
        self.runtime.has_fast_ball_species(species_id)
    }

    pub fn has_heavy_ball_species(&self, species_id: &str) -> bool {
        self.runtime.has_heavy_ball_species(species_id)
    }

    pub fn has_move_priority_effect(&self, effect_id: &str) -> bool {
        self.runtime.has_move_priority_effect(effect_id)
    }

    pub fn has_move_priority_move(&self, move_id: &str) -> bool {
        self.runtime.has_move_priority_move(move_id)
    }

    pub fn has_capture_ball_rule_key(&self, key: &RuntimeCaptureBallRuleKey) -> bool {
        self.runtime.has_capture_ball_rule_key(key)
    }

    pub fn has_heavy_ball_modifier(&self, key: &RuntimeHeavyBallModifierKey) -> bool {
        self.runtime.has_heavy_ball_modifier(key)
    }

    pub fn has_capture_status_bonus_key(&self, key: &RuntimeCaptureStatusBonusKey) -> bool {
        self.runtime.has_capture_status_bonus_key(key)
    }

    pub fn has_capture_wobble_probability(&self, key: &RuntimeCaptureWobbleProbabilityKey) -> bool {
        self.runtime.has_capture_wobble_probability(key)
    }

    pub fn has_item_battle_use(&self, key: &RuntimeItemBattleUseKey) -> bool {
        self.runtime.has_item_battle_use(key)
    }

    pub fn has_item_effect_plan(&self, key: &RuntimeItemEffectPlanKey) -> bool {
        self.runtime.has_item_effect_plan(key)
    }

    pub fn has_item_field_use(&self, key: &RuntimeItemFieldUseKey) -> bool {
        self.runtime.has_item_field_use(key)
    }

    pub fn has_move_battle_data(&self, key: &RuntimeMoveBattleDataKey) -> bool {
        self.runtime.has_move_battle_data(key)
    }

    pub fn has_species_battle_data(&self, key: &RuntimeSpeciesBattleDataKey) -> bool {
        self.runtime.has_species_battle_data(key)
    }

    pub fn has_trainer_battle_data(&self, key: &RuntimeTrainerBattleDataKey) -> bool {
        self.runtime.has_trainer_battle_data(key)
    }

    pub fn has_trainer_party_pokemon(&self, key: &RuntimeTrainerPartyPokemonKey) -> bool {
        self.runtime.has_trainer_party_pokemon(key)
    }

    pub fn has_move_priority_effect_key(&self, key: &RuntimeMovePriorityEffectKey) -> bool {
        self.runtime.has_move_priority_effect_key(key)
    }

    pub fn has_move_priority_move_key(&self, key: &RuntimeMovePriorityMoveKey) -> bool {
        self.runtime.has_move_priority_move_key(key)
    }

    pub fn has_battle_stat_multiplier(&self, key: &RuntimeBattleStatMultiplierKey) -> bool {
        self.runtime.has_battle_stat_multiplier(key)
    }

    pub fn has_battle_reward_rule(&self, key: &RuntimeBattleRewardRuleKey) -> bool {
        self.runtime.has_battle_reward_rule(key)
    }

    pub fn has_battle_escape_rule(&self, key: &RuntimeBattleEscapeRuleKey) -> bool {
        self.runtime.has_battle_escape_rule(key)
    }

    pub fn has_physical_type(&self, type_id: &str) -> bool {
        self.runtime.has_physical_type(type_id)
    }

    pub fn has_special_type(&self, type_id: &str) -> bool {
        self.runtime.has_special_type(type_id)
    }

    pub fn has_weather(&self, weather_id: &str) -> bool {
        self.runtime.has_weather(weather_id)
    }

    pub fn has_type_effectiveness(&self, key: &RuntimeTypeEffectivenessKey) -> bool {
        self.runtime.has_type_effectiveness(key)
    }

    pub fn has_foresight_type_effectiveness(&self, key: &RuntimeTypeEffectivenessKey) -> bool {
        self.runtime.has_foresight_type_effectiveness(key)
    }

    pub fn has_weather_type_modifier(&self, key: &RuntimeWeatherTypeModifierKey) -> bool {
        self.runtime.has_weather_type_modifier(key)
    }

    pub fn has_weather_move_effect_modifier(
        &self,
        key: &RuntimeWeatherMoveEffectModifierKey,
    ) -> bool {
        self.runtime.has_weather_move_effect_modifier(key)
    }

    pub fn has_audio_asset(&self, key: &RuntimeAudioAssetKey) -> bool {
        self.runtime.has_audio_asset(key)
    }

    pub fn has_pokemon_cry(&self, key: &RuntimePokemonCryKey) -> bool {
        self.runtime.has_pokemon_cry(key)
    }

    pub fn has_music(&self, music_id: &str) -> bool {
        self.runtime.has_music(music_id)
    }

    pub fn has_sound_effect(&self, sound_effect_id: &str) -> bool {
        self.runtime.has_sound_effect(sound_effect_id)
    }

    pub fn has_cry(&self, cry_id: &str) -> bool {
        self.runtime.has_cry(cry_id)
    }

    pub fn require_special_routine(&self, routine: &str) -> Result<()> {
        self.runtime.require_special_routine(routine)
    }

    pub fn require_item(&self, item_id: &str) -> Result<()> {
        self.runtime.require_item(item_id)
    }

    pub fn require_move(&self, move_id: &str) -> Result<()> {
        self.runtime.require_move(move_id)
    }

    pub fn require_species(&self, species_id: &str) -> Result<()> {
        self.runtime.require_species(species_id)
    }

    pub fn require_map(&self, map_name: &str) -> Result<()> {
        self.runtime.require_map(map_name)
    }

    pub fn require_trainer(&self, trainer_id: &str) -> Result<()> {
        self.runtime.require_trainer(trainer_id)
    }

    pub fn require_text(&self, text_label: &str) -> Result<()> {
        self.runtime.require_text(text_label)
    }

    pub fn require_menu(&self, menu: &str) -> Result<()> {
        self.runtime.require_menu(menu)
    }

    pub fn require_phone_contact(&self, contact_id: &str) -> Result<()> {
        self.runtime.require_phone_contact(contact_id)
    }

    pub fn require_special_phone_call(&self, call_id: &str) -> Result<()> {
        self.runtime.require_special_phone_call(call_id)
    }

    pub fn require_npc_trade(&self, trade_id: &str) -> Result<()> {
        self.runtime.require_npc_trade(trade_id)
    }

    pub fn require_sprite(&self, sprite_id: &str) -> Result<()> {
        self.runtime.require_sprite(sprite_id)
    }

    pub fn require_map_constant(&self, map_constant: &str) -> Result<()> {
        self.runtime.require_map_constant(map_constant)
    }

    pub fn require_event_flag(&self, flag: &str) -> Result<()> {
        self.runtime.require_event_flag(flag)
    }

    pub fn require_engine_flag(&self, flag: &str) -> Result<()> {
        self.runtime.require_engine_flag(flag)
    }

    pub fn require_spawn_identifier(&self, spawn_identifier: u16) -> Result<()> {
        self.runtime.require_spawn_identifier(spawn_identifier)
    }

    pub fn require_tileset(&self, tileset_id: &str) -> Result<()> {
        self.runtime.require_tileset(tileset_id)
    }

    pub fn require_tileset_row(&self, key: &RuntimeTilesetKey) -> Result<()> {
        self.runtime.require_tileset_row(key)
    }

    pub fn require_pc_string(&self, key: &RuntimePcStringKey) -> Result<()> {
        self.runtime.require_pc_string(key)
    }

    pub fn require_menu_icon(&self, key: &RuntimeMenuIconKey) -> Result<()> {
        self.runtime.require_menu_icon(key)
    }

    pub fn require_pokedex_entry(&self, key: &RuntimePokedexEntryKey) -> Result<()> {
        self.runtime.require_pokedex_entry(key)
    }

    pub fn require_landmark(&self, landmark_id: &str) -> Result<()> {
        self.runtime.require_landmark(landmark_id)
    }

    pub fn require_pokegear_landmark(&self, key: &RuntimePokegearLandmarkKey) -> Result<()> {
        self.runtime.require_pokegear_landmark(key)
    }

    pub fn require_pokegear_map_landmark(&self, key: &RuntimePokegearMapLandmarkKey) -> Result<()> {
        self.runtime.require_pokegear_map_landmark(key)
    }

    pub fn require_fishing_rod(&self, rod: &str) -> Result<()> {
        self.runtime.require_fishing_rod(rod)
    }

    pub fn require_map_group(&self, group_id: &str) -> Result<()> {
        self.runtime.require_map_group(group_id)
    }

    pub fn require_encounter_group(&self, group_id: &str) -> Result<()> {
        self.runtime.require_encounter_group(group_id)
    }

    pub fn require_mart(&self, mart_id: &str) -> Result<()> {
        self.runtime.require_mart(mart_id)
    }

    pub fn require_mart_row(&self, key: &RuntimeMartKey) -> Result<()> {
        self.runtime.require_mart_row(key)
    }

    pub fn require_fruit_tree(&self, fruit_tree_id: &str) -> Result<()> {
        self.runtime.require_fruit_tree(fruit_tree_id)
    }

    pub fn require_fruit_tree_row(&self, key: &RuntimeFruitTreeKey) -> Result<()> {
        self.runtime.require_fruit_tree_row(key)
    }

    pub fn require_field_move_rule(&self, rule_id: &str) -> Result<()> {
        self.runtime.require_field_move_rule(rule_id)
    }

    pub fn require_field_move_rule_row(&self, key: &RuntimeFieldMoveRuleKey) -> Result<()> {
        self.runtime.require_field_move_rule_row(key)
    }

    pub fn require_fly_destination(&self, flypoint_flag: &str) -> Result<()> {
        self.runtime.require_fly_destination(flypoint_flag)
    }

    pub fn require_fly_destination_row(&self, key: &RuntimeFlyDestinationKey) -> Result<()> {
        self.runtime.require_fly_destination_row(key)
    }

    pub fn require_field_move_move(&self, move_id: &str) -> Result<()> {
        self.runtime.require_field_move_move(move_id)
    }

    pub fn require_field_move_item(&self, item_id: &str) -> Result<()> {
        self.runtime.require_field_move_item(item_id)
    }

    pub fn require_flee_mon_bucket(&self, bucket_id: &str) -> Result<()> {
        self.runtime.require_flee_mon_bucket(bucket_id)
    }

    pub fn require_buena_password_category(&self, category_id: &str) -> Result<()> {
        self.runtime.require_buena_password_category(category_id)
    }

    pub fn require_roaming_species(&self, species_id: &str) -> Result<()> {
        self.runtime.require_roaming_species(species_id)
    }

    pub fn require_buena_prize_item(&self, item_id: &str) -> Result<()> {
        self.runtime.require_buena_prize_item(item_id)
    }

    pub fn require_kurt_apricorn_item(&self, item_id: &str) -> Result<()> {
        self.runtime.require_kurt_apricorn_item(item_id)
    }

    pub fn require_dratini_move_set(&self, answer: u8) -> Result<()> {
        self.runtime.require_dratini_move_set(answer)
    }

    pub fn require_special_feature(&self, feature_id: &str) -> Result<()> {
        self.runtime.require_special_feature(feature_id)
    }

    pub fn require_oak_rating_text(&self, text_id: &str) -> Result<()> {
        self.runtime.require_oak_rating_text(text_id)
    }

    pub fn require_odd_egg_species(&self, species_id: &str) -> Result<()> {
        self.runtime.require_odd_egg_species(species_id)
    }

    pub fn require_magikarp_length_threshold(&self, threshold: u16) -> Result<()> {
        self.runtime.require_magikarp_length_threshold(threshold)
    }

    pub fn require_happiness_change(&self, change_id: u8) -> Result<()> {
        self.runtime.require_happiness_change(change_id)
    }

    pub fn require_happiness_service(&self, service_id: &str) -> Result<()> {
        self.runtime.require_happiness_service(service_id)
    }

    pub fn require_pokemon_status(&self, status: &str) -> Result<()> {
        self.runtime.require_pokemon_status(status)
    }

    pub fn require_fishing_daily_flag_bit(&self, bit: u32) -> Result<()> {
        self.runtime.require_fishing_daily_flag_bit(bit)
    }

    pub fn require_fishing_swarm_flag(&self, swarm_flag: u8) -> Result<()> {
        self.runtime.require_fishing_swarm_flag(swarm_flag)
    }

    pub fn require_pending_special_battle_type(&self, battle_type: &str) -> Result<()> {
        self.runtime
            .require_pending_special_battle_type(battle_type)
    }

    pub fn require_wild_encounter_origin(&self, key: &RuntimeWildEncounterOriginKey) -> Result<()> {
        self.runtime.require_wild_encounter_origin(key)
    }

    pub fn require_script_label(&self, script_label: &str) -> Result<()> {
        self.runtime.require_script_label(script_label)
    }

    pub fn require_script_command(&self, key: &RuntimeScriptCommandKey) -> Result<()> {
        self.runtime.require_script_command(key)
    }

    pub fn require_script_command_payload(
        &self,
        key: &RuntimeScriptCommandPayloadKey,
    ) -> Result<()> {
        self.runtime.require_script_command_payload(key)
    }

    pub fn require_script_return(&self, key: &RuntimeScriptReturnKey) -> Result<()> {
        self.runtime.require_script_return(key)
    }

    pub fn require_script_vertical_menu(&self, key: &RuntimeScriptVerticalMenuKey) -> Result<()> {
        self.runtime.require_script_vertical_menu(key)
    }

    pub fn require_script_text_body(&self, key: &RuntimeScriptTextBodyKey) -> Result<()> {
        self.runtime.require_script_text_body(key)
    }

    pub fn require_script_menu_definition(
        &self,
        key: &RuntimeScriptMenuDefinitionKey,
    ) -> Result<()> {
        self.runtime.require_script_menu_definition(key)
    }

    pub fn require_script_elevator(&self, key: &RuntimeScriptElevatorKey) -> Result<()> {
        self.runtime.require_script_elevator(key)
    }

    pub fn require_gift_pokemon(&self, key: &RuntimeGiftPokemonKey) -> Result<()> {
        self.runtime.require_gift_pokemon(key)
    }

    pub fn require_script_object_command(&self, key: &RuntimeScriptObjectCommandKey) -> Result<()> {
        self.runtime.require_script_object_command(key)
    }

    pub fn require_script_movement(&self, key: &RuntimeScriptMovementKey) -> Result<()> {
        self.runtime.require_script_movement(key)
    }

    pub fn require_map_script_section_command(
        &self,
        key: &RuntimeMapScriptSectionCommandKey,
    ) -> Result<()> {
        self.runtime.require_map_script_section_command(key)
    }

    pub fn require_map_event_section_command(
        &self,
        key: &RuntimeMapEventSectionCommandKey,
    ) -> Result<()> {
        self.runtime.require_map_event_section_command(key)
    }

    pub fn require_script_map_command(&self, key: &RuntimeScriptMapCommandKey) -> Result<()> {
        self.runtime.require_script_map_command(key)
    }

    pub fn require_script_variable_command(
        &self,
        key: &RuntimeScriptVariableCommandKey,
    ) -> Result<()> {
        self.runtime.require_script_variable_command(key)
    }

    pub fn require_script_control_command(
        &self,
        key: &RuntimeScriptControlCommandKey,
    ) -> Result<()> {
        self.runtime.require_script_control_command(key)
    }

    pub fn require_script_swarm_command(&self, key: &RuntimeScriptSwarmCommandKey) -> Result<()> {
        self.runtime.require_script_swarm_command(key)
    }

    pub fn require_script_field_pickup(&self, key: &RuntimeScriptFieldPickupKey) -> Result<()> {
        self.runtime.require_script_field_pickup(key)
    }

    pub fn require_script_shop_command(&self, key: &RuntimeScriptShopCommandKey) -> Result<()> {
        self.runtime.require_script_shop_command(key)
    }

    pub fn require_script_phone_command(&self, key: &RuntimeScriptPhoneCommandKey) -> Result<()> {
        self.runtime.require_script_phone_command(key)
    }

    pub fn require_script_runtime_command(
        &self,
        key: &RuntimeScriptRuntimeCommandKey,
    ) -> Result<()> {
        self.runtime.require_script_runtime_command(key)
    }

    pub fn require_script_item_grant(&self, key: &RuntimeScriptItemGrantKey) -> Result<()> {
        self.runtime.require_script_item_grant(key)
    }

    pub fn require_script_item_access(&self, key: &RuntimeScriptItemAccessKey) -> Result<()> {
        self.runtime.require_script_item_access(key)
    }

    pub fn require_script_economy_command(
        &self,
        key: &RuntimeScriptEconomyCommandKey,
    ) -> Result<()> {
        self.runtime.require_script_economy_command(key)
    }

    pub fn require_script_flag_command(&self, key: &RuntimeScriptFlagCommandKey) -> Result<()> {
        self.runtime.require_script_flag_command(key)
    }

    pub fn require_script_scene_command(&self, key: &RuntimeScriptSceneCommandKey) -> Result<()> {
        self.runtime.require_script_scene_command(key)
    }

    pub fn require_script_block_change(&self, key: &RuntimeScriptBlockChangeKey) -> Result<()> {
        self.runtime.require_script_block_change(key)
    }

    pub fn require_script_audio_command(&self, key: &RuntimeScriptAudioCommandKey) -> Result<()> {
        self.runtime.require_script_audio_command(key)
    }

    pub fn require_script_text_command(&self, key: &RuntimeScriptTextCommandKey) -> Result<()> {
        self.runtime.require_script_text_command(key)
    }

    pub fn require_warp(&self, key: &RuntimeWarpKey) -> Result<()> {
        self.runtime.require_warp(key)
    }

    pub fn require_map_object(&self, key: &RuntimeMapObjectKey) -> Result<()> {
        self.runtime.require_map_object(key)
    }

    pub fn require_map_scene(&self, key: &RuntimeMapSceneKey) -> Result<()> {
        self.runtime.require_map_scene(key)
    }

    pub fn require_map_metadata(&self, key: &RuntimeMapMetadataKey) -> Result<()> {
        self.runtime.require_map_metadata(key)
    }

    pub fn require_currency_constant(&self, id: &str) -> Result<()> {
        self.runtime.require_currency_constant(id)
    }

    pub fn require_capture_ball_rule(&self, id: &str) -> Result<()> {
        self.runtime.require_capture_ball_rule(id)
    }

    pub fn require_guaranteed_capture_ball(&self, id: &str) -> Result<()> {
        self.runtime.require_guaranteed_capture_ball(id)
    }

    pub fn require_capture_status_bonus(&self, status: &str) -> Result<()> {
        self.runtime.require_capture_status_bonus(status)
    }

    pub fn require_fast_ball_species(&self, species_id: &str) -> Result<()> {
        self.runtime.require_fast_ball_species(species_id)
    }

    pub fn require_heavy_ball_species(&self, species_id: &str) -> Result<()> {
        self.runtime.require_heavy_ball_species(species_id)
    }

    pub fn require_move_priority_effect(&self, effect_id: &str) -> Result<()> {
        self.runtime.require_move_priority_effect(effect_id)
    }

    pub fn require_move_priority_move(&self, move_id: &str) -> Result<()> {
        self.runtime.require_move_priority_move(move_id)
    }

    pub fn require_capture_ball_rule_key(&self, key: &RuntimeCaptureBallRuleKey) -> Result<()> {
        self.runtime.require_capture_ball_rule_key(key)
    }

    pub fn require_heavy_ball_modifier(&self, key: &RuntimeHeavyBallModifierKey) -> Result<()> {
        self.runtime.require_heavy_ball_modifier(key)
    }

    pub fn require_capture_status_bonus_key(
        &self,
        key: &RuntimeCaptureStatusBonusKey,
    ) -> Result<()> {
        self.runtime.require_capture_status_bonus_key(key)
    }

    pub fn require_capture_wobble_probability(
        &self,
        key: &RuntimeCaptureWobbleProbabilityKey,
    ) -> Result<()> {
        self.runtime.require_capture_wobble_probability(key)
    }

    pub fn require_item_battle_use(&self, key: &RuntimeItemBattleUseKey) -> Result<()> {
        self.runtime.require_item_battle_use(key)
    }

    pub fn require_item_effect_plan(&self, key: &RuntimeItemEffectPlanKey) -> Result<()> {
        self.runtime.require_item_effect_plan(key)
    }

    pub fn require_item_field_use(&self, key: &RuntimeItemFieldUseKey) -> Result<()> {
        self.runtime.require_item_field_use(key)
    }

    pub fn require_move_battle_data(&self, key: &RuntimeMoveBattleDataKey) -> Result<()> {
        self.runtime.require_move_battle_data(key)
    }

    pub fn require_species_battle_data(&self, key: &RuntimeSpeciesBattleDataKey) -> Result<()> {
        self.runtime.require_species_battle_data(key)
    }

    pub fn require_trainer_battle_data(&self, key: &RuntimeTrainerBattleDataKey) -> Result<()> {
        self.runtime.require_trainer_battle_data(key)
    }

    pub fn require_trainer_party_pokemon(&self, key: &RuntimeTrainerPartyPokemonKey) -> Result<()> {
        self.runtime.require_trainer_party_pokemon(key)
    }

    pub fn require_move_priority_effect_key(
        &self,
        key: &RuntimeMovePriorityEffectKey,
    ) -> Result<()> {
        self.runtime.require_move_priority_effect_key(key)
    }

    pub fn require_move_priority_move_key(&self, key: &RuntimeMovePriorityMoveKey) -> Result<()> {
        self.runtime.require_move_priority_move_key(key)
    }

    pub fn require_battle_stat_multiplier(
        &self,
        key: &RuntimeBattleStatMultiplierKey,
    ) -> Result<()> {
        self.runtime.require_battle_stat_multiplier(key)
    }

    pub fn require_battle_reward_rule(&self, key: &RuntimeBattleRewardRuleKey) -> Result<()> {
        self.runtime.require_battle_reward_rule(key)
    }

    pub fn require_battle_escape_rule(&self, key: &RuntimeBattleEscapeRuleKey) -> Result<()> {
        self.runtime.require_battle_escape_rule(key)
    }

    pub fn require_physical_type(&self, type_id: &str) -> Result<()> {
        self.runtime.require_physical_type(type_id)
    }

    pub fn require_special_type(&self, type_id: &str) -> Result<()> {
        self.runtime.require_special_type(type_id)
    }

    pub fn require_weather(&self, weather_id: &str) -> Result<()> {
        self.runtime.require_weather(weather_id)
    }

    pub fn require_type_effectiveness(&self, key: &RuntimeTypeEffectivenessKey) -> Result<()> {
        self.runtime.require_type_effectiveness(key)
    }

    pub fn require_foresight_type_effectiveness(
        &self,
        key: &RuntimeTypeEffectivenessKey,
    ) -> Result<()> {
        self.runtime.require_foresight_type_effectiveness(key)
    }

    pub fn require_weather_type_modifier(&self, key: &RuntimeWeatherTypeModifierKey) -> Result<()> {
        self.runtime.require_weather_type_modifier(key)
    }

    pub fn require_weather_move_effect_modifier(
        &self,
        key: &RuntimeWeatherMoveEffectModifierKey,
    ) -> Result<()> {
        self.runtime.require_weather_move_effect_modifier(key)
    }

    pub fn require_audio_asset(&self, key: &RuntimeAudioAssetKey) -> Result<()> {
        self.runtime.require_audio_asset(key)
    }

    pub fn require_pokemon_cry(&self, key: &RuntimePokemonCryKey) -> Result<()> {
        self.runtime.require_pokemon_cry(key)
    }

    pub fn require_music(&self, music_id: &str) -> Result<()> {
        self.runtime.require_music(music_id)
    }

    pub fn require_sound_effect(&self, sound_effect_id: &str) -> Result<()> {
        self.runtime.require_sound_effect(sound_effect_id)
    }

    pub fn require_cry(&self, cry_id: &str) -> Result<()> {
        self.runtime.require_cry(cry_id)
    }

}
