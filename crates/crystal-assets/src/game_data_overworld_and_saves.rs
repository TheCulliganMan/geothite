impl GameDataSet {
    pub fn buy_shop_item(
        &self,
        state: &mut GameState,
        item_id: &str,
        quantity: u16,
    ) -> Result<ShopResult> {
        core_buy_active_shop_item(state, &self.items, item_id, quantity)
            .map_err(|error| anyhow::anyhow!("buy shop item {item_id}: {error:?}"))
    }

    pub fn sell_shop_item(
        &self,
        state: &mut GameState,
        item_id: &str,
        quantity: u16,
    ) -> Result<ShopResult> {
        core_sell_active_shop_item(
            state,
            &self.items,
            &self.currency_constants,
            item_id,
            quantity,
        )
        .map_err(|error| anyhow::anyhow!("sell shop item {item_id}: {error:?}"))
    }

    pub fn script_movement(
        &self,
        map_name: &str,
        source_script: &str,
        movement_label: &str,
    ) -> Result<&ScriptMovement> {
        // Local ASM labels are exported as `.Local@Parent`, while movement
        // definitions are intentionally keyed to the parent script scope.
        // TypeScript resolves movement data with that parent scope as well.
        let movement_source = source_script
            .rsplit_once('@')
            .map(|(_, parent)| parent)
            .unwrap_or(source_script);
        let movements = if let Some(module) = self.global_script_module_for(movement_source) {
            module.script_movements.as_slice()
        } else {
            self.map_module(map_name)?.script_movements.as_slice()
        };
        movements
            .iter()
            .find(|movement| {
                movement.label == movement_label
                    && movement.source_script.as_deref() == Some(movement_source)
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "map {map_name} has no exact movement {movement_label} for {source_script}"
                )
            })
    }

    pub fn map_ids(&self) -> BTreeSet<String> {
        self.maps.keys().cloned().collect()
    }

    pub fn map_music(&self, map_name: &str) -> Result<Option<&str>> {
        Ok(self.map_module(map_name)?.attributes.music.as_deref())
    }

    pub fn checked_map_music(
        &self,
        map_name: &str,
        music_ids: &BTreeSet<String>,
    ) -> Result<Option<String>> {
        let Some(music) = self.map_music(map_name)? else {
            return Ok(None);
        };
        if music_ids.is_empty() {
            return Ok(Some(music.to_string()));
        }
        core_validate_saved_audio_reference(
            &format!("maps.{map_name}.attributes.music"),
            music,
            ModpackAudioKind::Music.save_name(),
            music_ids
                .contains(music)
                .then_some(ModpackAudioKind::Music.save_name()),
        )
        .map_err(|error| anyhow::anyhow!("{error}"))?;
        Ok(Some(music.to_owned()))
    }

    pub fn sync_current_map_music(
        &self,
        state: &mut GameState,
        map_name: &str,
        mode: MovementMode,
        music_ids: &BTreeSet<String>,
    ) -> Result<()> {
        let bug_contest_ranking = matches!(mode, MovementMode::Normal | MovementMode::Skate)
            && matches!(
                map_name,
                "Route35NationalParkGate" | "Route36NationalParkGate"
            )
            && state
                .flags
                .is_engine_flag_set("ENGINE_BUG_CONTEST_TIMER")
                .map_err(|error| anyhow::anyhow!("read Bug Contest timer flag: {error}"))?;
        let music = match mode {
            MovementMode::Bike => Some("MUSIC_BICYCLE".to_string()),
            MovementMode::Surf | MovementMode::SurfPika => Some("MUSIC_SURF".to_string()),
            MovementMode::Normal | MovementMode::Skate if bug_contest_ranking => {
                Some("MUSIC_BUG_CATCHING_CONTEST_RANKING".to_string())
            }
            MovementMode::Normal | MovementMode::Skate => {
                self.checked_map_music(map_name, music_ids)?
            }
        };
        if !music_ids.is_empty() {
            if let Some(music_id) = music.as_deref() {
                core_validate_saved_audio_reference(
                    "overworld movement music",
                    music_id,
                    ModpackAudioKind::Music.save_name(),
                    music_ids
                        .contains(music_id)
                        .then_some(ModpackAudioKind::Music.save_name()),
                )
                .map_err(|error| anyhow::anyhow!("{error}"))?;
            }
        }
        apply_map_music_context(state, music);
        Ok(())
    }

    fn map_entry_movement_mode(
        &self,
        state: &GameState,
        session: &OverworldSession,
        previous: MovementMode,
    ) -> Result<MovementMode> {
        if state
            .flags
            .is_engine_flag_set("ENGINE_ALWAYS_ON_BIKE")
            .context("check ENGINE_ALWAYS_ON_BIKE during map entry")?
        {
            return Ok(MovementMode::Bike);
        }
        let sample = sample_collision(&session.map, &session.tileset, session.player.tile)
            .with_context(|| {
                format!(
                    "sample map-entry tile {},{} on {}",
                    session.player.tile.x, session.player.tile.y, session.map.name
                )
            })?;
        if describe_collision(sample.permission).terrain == Terrain::Water {
            return Ok(match previous {
                MovementMode::Surf | MovementMode::SurfPika => previous,
                MovementMode::Normal | MovementMode::Bike | MovementMode::Skate => {
                    MovementMode::Surf
                }
            });
        }
        Ok(match previous {
            MovementMode::Surf | MovementMode::SurfPika => MovementMode::Normal,
            MovementMode::Bike
                if !is_bicycle_environment(self.map_environment(&session.map.name)?) =>
            {
                MovementMode::Normal
            }
            MovementMode::Normal | MovementMode::Bike | MovementMode::Skate => previous,
        })
    }

    pub fn sync_current_map_scene(&self, state: &mut GameState, map_name: &str) -> Result<()> {
        let scenes = self.map_scene_table(map_name)?;
        apply_map_scene_context(state, map_name, scenes).map_err(|error| {
            anyhow::anyhow!("apply map scene context for {map_name}: {error:?}")
        })?;
        Ok(())
    }

    /// Run the data-driven callbacks reached by one exact map setup program.
    pub fn apply_map_setup_callbacks(
        &self,
        state: &mut GameState,
        session: &mut OverworldSession,
        map_name: &str,
        map_setup: &str,
    ) -> Result<()> {
        self.require_current_map(&session.map.name, map_name)?;
        let callback_kinds = map_setup_callback_kinds(map_setup)
            .with_context(|| format!("unknown map setup callback path {map_setup}"))?;
        let module = self.map_module(map_name)?;
        // HandleNewMap clears the first byte of event flags before callbacks.
        // Battle reloads, submenu returns, and Continue skip HandleNewMap.
        if callback_kinds.contains(&MAP_CALLBACK_NEWMAP) {
            for index in 1..=8 {
                state.flags.clear_event_flag(&format!("EVENT_TEMPORARY_UNTIL_MAP_RELOAD_{index}"))
                    .map_err(|error| anyhow::anyhow!("reset temporary map event: {error:?}"))?;
            }
        }
        if callback_kinds.contains(&MAP_CALLBACK_NEWMAP) && !module.scenes.scenes.is_empty() {
            state
                .scenes
                .check_scene(map_name, &module.scenes)
                .map_err(|error| {
                    anyhow::anyhow!("check map callback scene for {map_name}: {error:?}")
                })?;
        }
        let callback_names: Vec<(String, String)> = callback_kinds
            .iter()
            .flat_map(|callback_kind| {
                module
                    .map_script_section_commands
                    .iter()
                    .filter(move |command| {
                        command.command == "callback"
                            && command.args.first().map(String::as_str) == Some(*callback_kind)
                    })
                    .filter_map(|command| {
                        command
                            .args
                            .get(1)
                            .cloned()
                            .map(|name| (command.args[0].clone(), name))
                    })
            })
            .collect();
        let callback_bodies: Vec<(String, String, Vec<Value>)> = callback_names
            .into_iter()
            .filter_map(|(kind, name)| {
                module
                    .scripts
                    .get(&name)
                    .and_then(Value::as_array)
                    .map(|body| (kind, name, body.clone()))
            })
            .collect();
        for callback_kind in callback_kinds {
            if *callback_kind == MAP_CALLBACK_TILES {
                // LoadBlockData rebuilds the map before its tile callback.
                // changeblock writes belong to the loaded map, not every future
                // visit. Persistent doors are reopened by their event callbacks.
                session.map.metatile_ids = self.overworld_map(map_name)?.metatile_ids;
                state.map_block_overrides.remove(map_name);
            }
            for (kind, callback_name, body) in &callback_bodies {
                if kind == callback_kind {
                    self.execute_map_callback_script(state, session, map_name, callback_name, body)?;
                }
            }
        }
        // Callback specials such as ToggleMaptileDecorations persist their
        // authored block writes in GameState.  Apply those writes to the live
        // map before the first visible frame, just as ordinary changeblock
        // callback commands already mutate the session directly.
        apply_state_block_overrides(session, state).context("sync map block callback overrides")?;
        sync_state_object_overrides(state, session)
            .context("sync map object callback overrides")?;
        // MAPSETUP_RELOADMAP ends in ForceMapMusic, whose TryRestartMapMusic
        // branch consumes wDontPlayMapMusicOnReload by stopping music and
        // clearing wMapMusic instead of restarting it.
        if map_setup == "MAPSETUP_RELOADMAP" && state.script_runtime.map_music_restart_disabled {
            state.script_runtime.current_music = None;
            state.script_runtime.map_music_restart_disabled = false;
        }
        Ok(())
    }

    fn execute_map_callback_script(
        &self,
        state: &mut GameState,
        session: &mut OverworldSession,
        map_name: &str,
        callback_name: &str,
        body: &[Value],
    ) -> Result<()> {
        let outer_call_stack_depth = state.script_runtime.call_stack.len();
        let outer_next_script = state.script_runtime.next_script.clone();
        let outer_script_ended = state.script_runtime.script_ended.clone();
        let result = self.execute_map_callback_script_body(
            state,
            session,
            map_name,
            callback_name,
            body,
            outer_call_stack_depth,
        );
        state
            .script_runtime
            .call_stack
            .truncate(outer_call_stack_depth);
        state.script_runtime.next_script = outer_next_script;
        state.script_runtime.script_ended = outer_script_ended;
        result
    }

    fn execute_map_callback_script_body(
        &self,
        state: &mut GameState,
        session: &mut OverworldSession,
        map_name: &str,
        callback_name: &str,
        body: &[Value],
        outer_call_stack_depth: usize,
    ) -> Result<()> {
        let mut script_name = callback_name.to_string();
        let mut command_index = 0usize;
        for _ in 0..1024 {
            let script_body: &[Value] = if script_name == callback_name {
                body
            } else if let Some(target) = self
                .compiled_script_body(&script_name)
                .and_then(Value::as_array)
            {
                target
            } else {
                anyhow::bail!(
                    "callback {callback_name} targets missing compiled script {script_name}"
                );
            };
            let Some(entry) = script_body.get(command_index) else {
                return Ok(());
            };
            let command = entry
                .get("command")
                .and_then(Value::as_str)
                .with_context(|| {
                    format!("callback {callback_name} command {command_index} has no command")
                })?;
            let source = script_name.as_str();
            match command {
                "checkevent" | "checkflag" | "setevent" | "clearevent" | "setflag"
                | "clearflag" => {
                    if command.starts_with("check") {
                        self.check_script_flag_in_session(
                            state,
                            session,
                            map_name,
                            source,
                            command_index,
                        )?;
                    } else {
                        self.apply_script_flag_mutation_in_session(
                            state,
                            session,
                            map_name,
                            source,
                            command_index,
                        )?;
                    }
                    command_index += 1;
                }
                "checkitem" => {
                    let outcome = self.check_script_item_in_session(
                        state,
                        session,
                        map_name,
                        source,
                        command_index,
                    )?;
                    state.script_runtime.script_value =
                        Some(if outcome.held { "1" } else { "0" }.to_string());
                    command_index += 1;
                }
                "checkscene" | "checkmapscene" | "setscene" => {
                    self.apply_script_scene_command_in_session(
                        state,
                        session,
                        map_name,
                        source,
                        command_index,
                    )?;
                    command_index += 1;
                }
                "setmapscene" => {
                    self.apply_script_scene_command_in_session(
                        state,
                        session,
                        map_name,
                        source,
                        command_index,
                    )?;
                    command_index += 1;
                }
                "changeblock" => {
                    self.apply_script_block_change_in_session(
                        state,
                        session,
                        map_name,
                        source,
                        command_index,
                    )?;
                    command_index += 1;
                }
                "readvar" | "setval" | "writemem" => {
                    self.apply_script_variable_command_now_in_mut_session(
                        state,
                        session,
                        map_name,
                        source,
                        command_index,
                    )?;
                    command_index += 1;
                }
                "checktime" => {
                    self.apply_script_variable_command_now_in_mut_session(
                        state,
                        session,
                        map_name,
                        source,
                        command_index,
                    )?;
                    command_index += 1;
                }
                "special" => {
                    let routine = entry
                        .get("args")
                        .and_then(Value::as_array)
                        .and_then(|args| args.first())
                        .and_then(Value::as_str)
                        .with_context(|| {
                            format!("callback {callback_name} special has no routine")
                        })?;
                    self.apply_special_routine(state, routine, &BTreeSet::new())?;
                    command_index += 1;
                }
                "cmdqueue" | "writecmdqueue" | "stonetable" | "variablesprite"
                | "specialphonecall" => {
                    self.apply_script_runtime_command_in_session(
                        state,
                        session,
                        map_name,
                        source,
                        command_index,
                        ScriptRuntimeInputs::default(),
                    )?;
                    command_index += 1;
                }
                "readmem" => {
                    self.apply_script_variable_command_now_in_mut_session(
                        state,
                        session,
                        map_name,
                        source,
                        command_index,
                    )?;
                    command_index += 1;
                }
                "appear" | "disappear" | "moveobject" | "turnobject" | "faceobject"
                | "faceplayer" | "follow" | "follownotexact" | "stopfollow" | "showemote" => {
                    let object = self
                        .script_object_command(map_name, source, command_index)?
                        .clone();
                    core_apply_script_object_mutation(state, session, &object).map_err(
                        |error| anyhow::anyhow!("apply map object callback {map_name}: {error:?}"),
                    )?;
                    command_index += 1;
                }
                "iftrue" | "iffalse" | "ifequal" | "ifnotequal" | "ifgreater" | "ifless"
                | "sjump" | "jumpstd" | "callstd" | "scall" | "sdefer" | "endcallback" | "end" => {
                    let action = self.apply_script_control_command_in_session(
                        state,
                        session,
                        map_name,
                        source,
                        command_index,
                    )?;
                    match action {
                        ScriptControlAction::Continue { .. } => command_index += 1,
                        ScriptControlAction::Jump { target_script, .. } => {
                            state.script_runtime.next_script = None;
                            script_name = target_script;
                            command_index = 0;
                        }
                        ScriptControlAction::End { .. } => {
                            if state.script_runtime.call_stack.len() > outer_call_stack_depth {
                                let frame = state
                                    .script_runtime
                                    .call_stack
                                    .pop()
                                    .context("callback-local call stack became empty")?;
                                state.script_runtime.script_ended = None;
                                state.script_runtime.next_script = None;
                                script_name = frame.source_script;
                                command_index = frame.next_command_index;
                            } else {
                                return Ok(());
                            }
                        }
                    }
                }
                _ => {
                    anyhow::bail!(
                        "callback {callback_name} reached unsupported opcode {command} at {script_name}:{command_index} on map {map_name}"
                    );
                }
            }
        }
        anyhow::bail!("map object callback {callback_name} exceeded execution limit")
    }

    pub fn overworld_session(
        &self,
        map_name: &str,
        player_tile: TilePosition,
        frame: u64,
    ) -> Result<OverworldSession> {
        self.overworld_session_for_traversal(
            map_name,
            player_tile,
            frame,
            PlayerTraversalState::Walk,
        )
    }

    pub fn overworld_session_for_traversal(
        &self,
        map_name: &str,
        player_tile: TilePosition,
        frame: u64,
        traversal_state: PlayerTraversalState,
    ) -> Result<OverworldSession> {
        let module = self.map_module(map_name)?;
        let tileset = self.tileset_collision(&module.attributes.tileset_name)?;
        validate_runtime_overworld_map_blocks(map_name, module)?;
        let map =
            OverworldMapData::from_attributes(map_name, &module.attributes, module.blocks.clone());
        let (width, height) = map.checked_tile_bounds().with_context(|| {
            format!(
                "compiled map {map_name} runtime tile bounds overflow supported coordinate range"
            )
        })?;
        if player_tile.x < 0
            || player_tile.y < 0
            || i32::from(player_tile.x) >= i32::from(width)
            || i32::from(player_tile.y) >= i32::from(height)
        {
            anyhow::bail!(
                "runtime player tile ({}, {}) is outside compiled map {map_name} runtime tile bounds {width}x{height}",
                player_tile.x,
                player_tile.y
            );
        }
        if !Self::can_occupy_runtime_tile(&map, &tileset, player_tile, traversal_state)
            && !self.runtime_tile_is_connection_source(module, player_tile)?
        {
            anyhow::bail!(
                "runtime player tile ({}, {}) is not walkable on compiled map {map_name}",
                player_tile.x,
                player_tile.y
            );
        }
        let mut session = OverworldSession::with_events_and_objects(
            map,
            module.events.clone(),
            module.objects.clone(),
            tileset,
            player_tile,
        );
        session.frame = frame;
        Ok(session)
    }

    fn can_occupy_runtime_tile(
        map: &OverworldMapData,
        tileset: &TilesetCollision,
        player_tile: TilePosition,
        traversal_state: PlayerTraversalState,
    ) -> bool {
        [
            Direction::Down,
            Direction::Up,
            Direction::Left,
            Direction::Right,
        ]
        .into_iter()
        .any(|facing| can_enter_tile(map, tileset, player_tile, facing, traversal_state))
    }

    fn runtime_tile_is_connection_source(
        &self,
        module: &MapModule,
        player_tile: TilePosition,
    ) -> Result<bool> {
        for connection in &module.attributes.connections {
            let Some(trigger_tile) = connection_trigger_tile_from_source(player_tile, connection)
            else {
                continue;
            };
            let target_module = self.map_module(&connection.target_map)?;
            if connection_destination_tile_in_bounds(
                trigger_tile,
                &connection.direction,
                connection.offset,
                &target_module.attributes,
            )? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn apply_saved_overworld_overrides(
        &self,
        session: &mut OverworldSession,
        state: &GameState,
    ) -> Result<()> {
        apply_state_block_overrides(session, state)?;
        apply_state_object_overrides(session, state)?;
        Ok(())
    }

    pub fn commit_overworld_snapshot(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        spawn_update: SpawnMemoryUpdate,
    ) {
        commit_overworld_snapshot(state, &session.snapshot(), spawn_update);
    }

    pub fn commit_overworld_snapshot_data(
        &self,
        state: &mut GameState,
        snapshot: &OverworldSnapshot,
        spawn_update: SpawnMemoryUpdate,
    ) {
        commit_overworld_snapshot(state, snapshot, spawn_update);
    }

    pub fn transition_overworld_session(
        &self,
        state: &mut GameState,
        session: &mut OverworldSession,
        destination_map: &str,
        destination_tile: TilePosition,
        spawn_update: SpawnMemoryUpdate,
        music_ids: &BTreeSet<String>,
    ) -> Result<()> {
        self.transition_overworld_session_with_mode(
            state,
            session,
            destination_map,
            destination_tile,
            MovementMode::Normal,
            "MAPSETUP_WARP",
            spawn_update,
            music_ids,
        )
    }

    fn transition_overworld_session_with_mode(
        &self,
        state: &mut GameState,
        session: &mut OverworldSession,
        destination_map: &str,
        destination_tile: TilePosition,
        mode: MovementMode,
        map_setup: &str,
        spawn_update: SpawnMemoryUpdate,
        music_ids: &BTreeSet<String>,
    ) -> Result<()> {
        let frame = session.frame;
        let connection_movement = (map_setup == "MAPSETUP_CONNECTION")
            .then_some((session.player.facing, session.last_step_direction));
        *session = self.overworld_session_for_traversal(
            destination_map,
            destination_tile,
            frame,
            mode.traversal_state(),
        )?;
        session.player.mode = mode;
        begin_map_object_setup(session, state);
        if let Some((facing, last_step_direction)) = connection_movement {
            // ASM EnterMapConnection updates wMapGroup/wMapNumber, the
            // coordinates, and wOverworldMapAnchor in place. Unlike a warp,
            // MapSetupScript_Connection never resets the player object or its
            // direction, so the boundary-crossing step remains one continuous
            // movement on the destination map.
            session.player.facing = facing;
            session.last_step_direction = last_step_direction;
        }
        clear_transient_map_object_context(state, session);
        reset_map_bike_flags(state)?;
        state.wild_encounter_cooldown = 5;
        let destination_environment = &self
            .runtime_map_metadata_for_name(destination_map)?
            .environment;
        if destination_environment.eq_ignore_ascii_case("route")
            || destination_environment.eq_ignore_ascii_case("town")
        {
            // ResetFlashIfOutOfCave clears the transient illumination only
            // when map setup reaches an outdoor route or town.  Keeping this
            // flag forever made every later PALETTE_DARK cave render as lit.
            state
                .flags
                .set_engine_flag("STATUSFLAGS_FLASH", false)
                .map_err(|error| anyhow::anyhow!("reset FLASH on outdoor map entry: {error}"))?;
        }
        apply_state_block_overrides(session, state)?;
        let mode = self.map_entry_movement_mode(state, session, mode)?;
        session.player.mode = mode;
        self.sync_current_map_music(state, destination_map, mode, music_ids)?;
        self.sync_current_map_scene(state, destination_map)?;
        self.init_map_name_sign(state, destination_map)?;
        self.apply_map_setup_callbacks(state, session, destination_map, map_setup)?;
        finish_map_object_setup(session, state)?;
        let callback_mode = self.map_entry_movement_mode(state, session, session.player.mode)?;
        if callback_mode != session.player.mode {
            session.player.mode = callback_mode;
            self.sync_current_map_music(state, destination_map, callback_mode, music_ids)?;
        }
        self.commit_overworld_snapshot(state, session, spawn_update);
        Ok(())
    }

    pub fn start_overworld_session_from_spawn(
        &self,
        spawn: &RuntimeSpawnPoint,
        music_ids: &BTreeSet<String>,
    ) -> Result<(GameState, OverworldSession)> {
        self.start_overworld_session_from_new_game_state(
            spawn,
            GameState::reset_wram_for_new_game(),
            music_ids,
        )
    }

    /// Completes the asset-derived portion of NewGame after core has executed
    /// ResetWRAM against the title session's retained hardware RNG and SRAM
    /// values.
    pub fn start_overworld_session_from_new_game_state(
        &self,
        spawn: &RuntimeSpawnPoint,
        mut state: GameState,
        music_ids: &BTreeSet<String>,
    ) -> Result<(GameState, OverworldSession)> {
        let spawn_tile = runtime_spawn_expected_tile(spawn);
        let mut overworld = self.overworld_session(&spawn.map_name, spawn_tile, 0)?;
        self.initialize_new_game_state(&mut state)?;
        begin_map_object_setup(&mut overworld, &state);
        self.commit_overworld_snapshot(
            &mut state,
            &overworld,
            SpawnMemoryUpdate::Preserve,
        );
        let map_name = overworld.map.name.clone();
        overworld.player.mode =
            self.map_entry_movement_mode(&state, &overworld, overworld.player.mode)?;
        self.sync_current_map_music(&mut state, &map_name, overworld.player.mode, music_ids)?;
        self.sync_current_map_scene(&mut state, &map_name)?;
        self.init_map_name_sign(&mut state, &map_name)?;
        self.apply_map_setup_callbacks(&mut state, &mut overworld, &map_name, "MAPSETUP_WARP")?;
        finish_map_object_setup(&mut overworld, &mut state)?;
        self.commit_overworld_snapshot(
            &mut state,
            &overworld,
            SpawnMemoryUpdate::Preserve,
        );
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        Ok((state, overworld))
    }

    #[cfg(any(test, feature = "test-fixtures"))]
    pub fn start_overworld_session_at_runtime_tile(
        &self,
        map_name: &str,
        tile: TilePosition,
        music_ids: &BTreeSet<String>,
    ) -> Result<(GameState, OverworldSession)> {
        let mut overworld = self.overworld_session(map_name, tile, 0)?;
        let mut state = GameState::reset_wram_for_new_game();
        self.initialize_new_game_state(&mut state)?;
        begin_map_object_setup(&mut overworld, &state);
        // The location tester may enter any map before ordinary story scripts
        // have first written that map's WRAM bytes. A real new game reaches
        // callbacks with cleared WRAM, so seed every exact readmem target used
        // by this map to zero in this test-fixture-only boot path. Production
        // sessions and save loading retain their strict memory behavior.
        for command in &self.map_module(map_name)?.script_variable_commands {
            if command.command == "readmem"
                && let Some(memory) = command.target.as_ref()
            {
                state
                    .script_runtime
                    .memory
                    .entry(memory.clone())
                    .or_insert_with(|| "0".to_string());
            }
        }
        self.commit_overworld_snapshot(&mut state, &overworld, SpawnMemoryUpdate::Preserve);
        let map_name = overworld.map.name.clone();
        overworld.player.mode =
            self.map_entry_movement_mode(&state, &overworld, overworld.player.mode)?;
        self.sync_current_map_music(&mut state, &map_name, overworld.player.mode, music_ids)?;
        self.sync_current_map_scene(&mut state, &map_name)?;
        self.init_map_name_sign(&mut state, &map_name)?;
        self.apply_map_setup_callbacks(&mut state, &mut overworld, &map_name, "MAPSETUP_WARP")?;
        finish_map_object_setup(&mut overworld, &mut state)?;
        self.commit_overworld_snapshot(&mut state, &overworld, SpawnMemoryUpdate::Preserve);
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        Ok((state, overworld))
    }

    fn initialize_new_game_money(&self, state: &mut GameState) -> Result<()> {
        state.money = *self
            .currency_constants
            .0
            .get("START_MONEY")
            .context("compiled currency constants missing START_MONEY")?;
        state.mom_item_trigger_balance = *self
            .currency_constants
            .0
            .get("MOM_MONEY")
            .context("compiled currency constants missing MOM_MONEY")?;
        Ok(())
    }

    fn initialize_new_game_state(&self, state: &mut GameState) -> Result<()> {
        self.initialize_new_game_money(state)?;
        state.map_name_sign.previous_landmark = self.landmark_byte("LANDMARK_NEW_BARK_TOWN")?;
        crystal_core::systems::map_name_sign::force_hide_next_map_name_sign(
            &mut state.map_name_sign,
        );
        state.roaming_pokemon = crystal_core::systems::roaming::initialize_world_roaming_slots(
            &self.roaming_pokemon,
            &state.roaming_pokemon,
        )
        .map_err(|error| anyhow::anyhow!("initialize new-game roaming state: {error}"))?;
        state.wild_encounter_cooldown = 5;
        state.bag.tm_hm = initial_tmhm_flags(&self.items);
        // InitializeNPCNames copies four fixed NAME_LENGTH strings before
        // InitializeWorld. Keep all four WRAM names authoritative even where
        // only the rival currently has a later naming screen.
        for (variable, name) in [
            ("_rival_name", "???"),
            ("_moms_name", "MOM"),
            ("_reds_name", "RED"),
            ("_greens_name", "GREEN"),
        ] {
            state
                .script_runtime
                .variables
                .insert(variable.to_string(), name.to_string());
        }
        apply_initialize_events(state, &self.initialize_events)
            .map_err(|error| anyhow::anyhow!("apply initialize events: {error}"))?;
        Ok(())
    }

    pub fn resume_overworld_session_from_state(
        &self,
        mut state: GameState,
        music_ids: &BTreeSet<String>,
    ) -> Result<(GameState, OverworldSession)> {
        let (map_name, tile, facing, mode) = state
            .overworld
            .snapshot_identity()
            .with_context(|| "cannot resume overworld session from inactive GameState")?;
        let map_name = map_name.to_string();
        let mut overworld = self.overworld_session_for_traversal(
            &map_name,
            tile,
            state.frame_counter,
            mode.traversal_state(),
        )?;
        overworld.player.facing = facing;
        overworld.player.mode = mode;
        initialize_loaded_object_roster(&mut overworld, &state);
        crystal_core::systems::map_name_sign::force_hide_next_map_name_sign(
            &mut state.map_name_sign,
        );
        self.apply_saved_overworld_overrides(&mut overworld, &state)?;
        let mode = self.map_entry_movement_mode(&state, &overworld, mode)?;
        overworld.player.mode = mode;
        self.sync_current_map_music(&mut state, &map_name, mode, music_ids)?;
        self.sync_current_map_scene(&mut state, &map_name)?;
        self.init_map_name_sign(&mut state, &map_name)?;
        self.apply_map_setup_callbacks(&mut state, &mut overworld, &map_name, "MAPSETUP_CONTINUE")?;
        self.commit_overworld_snapshot(&mut state, &overworld, SpawnMemoryUpdate::Preserve);
        overworld.set_time(state.time.registers.hours, state.time.time_of_day);
        if let Some(music) = state.script_runtime.current_music.as_deref() {
            core_validate_saved_audio_reference(
                "state.script_runtime.current_music",
                music,
                ModpackAudioKind::Music.save_name(),
                music_ids
                    .contains(music)
                    .then_some(ModpackAudioKind::Music.save_name()),
            )
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        }
        Ok((state, overworld))
    }

    pub fn apply_overworld_input<S>(
        &self,
        state: &mut GameState,
        session: &mut OverworldSession,
        buttons: impl IntoIterator<Item = GameButton>,
        music_ids: &BTreeSet<String>,
        divider: &mut S,
    ) -> Result<OverworldInputFrame>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        let input_candidate = JoypadState::compute_mask(buttons);
        let mut staged_state = state.clone();
        let mut staged_session = session.clone();
        crystal_core::systems::map_name_sign::place_map_name_sign(&mut staged_state.map_name_sign);
        let mut rng = CrystalRandom::new(staged_state.random_state, &mut *divider);
        // This transactional path intentionally stages full state/session
        // clones so every divider failure is atomic. A later VBlank/object
        // frame-kernel rewrite owns any journal-disabled idle optimization;
        // do not bypass staging while phone, encounter, or object RNG may run.
        let strength_boulder_landing =
            self.queue_strength_boulder_landing_script(&mut staged_state, &staged_session)?;
        let whirlpool_forced_movement = if strength_boulder_landing {
            false
        } else {
            self.queue_whirlpool_forced_movement_script(&mut staged_state, &staged_session)?
        };
        let has_autonomous_objects = staged_session.objects.iter().any(|object| {
            matches!(
                object.spritemovedata.as_str(),
                "SPRITEMOVEDATA_WALK_LEFT_RIGHT"
                    | "SPRITEMOVEDATA_WALK_UP_DOWN"
                    | "SPRITEMOVEDATA_WANDER"
                    | "SPRITEMOVEDATA_SWIM_WANDER"
                    | "SPRITEMOVEDATA_SPINCLOCKWISE"
                    | "SPRITEMOVEDATA_SPINCOUNTERCLOCKWISE"
                    | "SPRITEMOVEDATA_SPINRANDOM_SLOW"
                    | "SPRITEMOVEDATA_SPINRANDOM_FAST"
            )
        });
        let input_locked = strength_boulder_landing
            || whirlpool_forced_movement
            || Self::game_state_blocks_overworld_input(&staged_state);
        let bug_contest_blocks_phone = staged_state
            .flags
            .is_engine_flag_set("ENGINE_BUG_CONTEST_TIMER")
            .map_err(|error| anyhow::anyhow!("check Bug Contest timer flag: {error}"))?;
        if !input_locked && !bug_contest_blocks_phone {
            if let Some(phone_call) =
                self.check_ordinary_phone_call(&mut staged_state, &staged_session, &mut rng)?
            {
                let joypad_event = staged_state
                    .apply_joypad_mask(input_candidate)
                    .map_err(|error| anyhow::anyhow!("apply phone-call joypad mask: {error}"))?;
                let (pressed_mask, input_mask) = match joypad_event {
                    crystal_core::state::GameEvent::JoypadChanged { pressed, down } => {
                        (pressed, down)
                    }
                    event => anyhow::bail!(
                        "phone-call joypad command produced unexpected event {event:?}"
                    ),
                };
                staged_session.frame = staged_session
                    .frame
                    .checked_add(1)
                    .context("advance incoming-phone-call overworld frame")?;
                let snapshot = staged_session.snapshot();
                staged_state.random_state = rng.state();
                self.commit_overworld_snapshot_data(
                    &mut staged_state,
                    &snapshot,
                    SpawnMemoryUpdate::Preserve,
                );
                *state = staged_state;
                *session = staged_session;
                return Ok(OverworldInputFrame {
                    snapshot,
                    input_mask,
                    pressed_mask,
                    autonomous_objects_changed: false,
                    movement: None,
                    ledge_jump: None,
                    grass_rustle: None,
                    phone_call: Some(phone_call),
                    step_events: None,
                    coord_event: None,
                    trainer_sight: None,
                    interaction: None,
                    warp: None,
                    connection: None,
                    wild_encounter: None,
                    wild_battle: None,
                });
            }
        }
        let forced_tile_movement_pending =
            !input_locked && staged_session.forced_movement_direction().is_some();
        let downhill_movement_pending = !input_locked
            && matches!(
                staged_session.player.mode,
                MovementMode::Bike | MovementMode::Skate
            )
            && staged_state
                .flags
                .is_engine_flag_set("ENGINE_DOWNHILL")
                .map_err(|error| anyhow::anyhow!("check downhill bike flag: {error}"))?;
        if input_candidate == 0
            && !staged_state.bug_contest.timer_active
            && !forced_tile_movement_pending
            && !downhill_movement_pending
            && !staged_session.has_required_object_steps()
            && (input_locked || !has_autonomous_objects)
        {
            staged_state
                .apply_joypad_mask(0)
                .map_err(|error| anyhow::anyhow!("apply idle joypad mask: {error}"))?;
            staged_session.frame = staged_session
                .frame
                .checked_add(1)
                .context("advance idle overworld frame")?;
            let snapshot = staged_session.snapshot();
            staged_state.random_state = rng.state();
            self.commit_overworld_snapshot_data(
                &mut staged_state,
                &snapshot,
                SpawnMemoryUpdate::Preserve,
            );
            *state = staged_state;
            *session = staged_session;
            return Ok(OverworldInputFrame {
                snapshot,
                input_mask: 0,
                pressed_mask: 0,
                autonomous_objects_changed: false,
                movement: None,
                ledge_jump: None,
                grass_rustle: None,
                phone_call: None,
                step_events: None,
                coord_event: None,
                trainer_sight: None,
                interaction: None,
                warp: None,
                connection: None,
                wild_encounter: None,
                wild_battle: None,
            });
        }
        let joypad_event = staged_state
            .apply_joypad_mask(input_candidate)
            .map_err(|error| anyhow::anyhow!("apply joypad mask: {error}"))?;
        let (pressed_mask, input_mask) = match joypad_event {
            crystal_core::state::GameEvent::JoypadChanged { pressed, down } => (pressed, down),
            event => anyhow::bail!("joypad command produced unexpected event {event:?}"),
        };

        let mut bug_contest_timed_out = false;
        if !input_locked && staged_state.link_session.link_mode == 0
            && staged_state.bug_contest.timer_active
            && staged_state
                .flags
                .is_engine_flag_set("ENGINE_BUG_CONTEST_TIMER")
                .map_err(|error| anyhow::anyhow!("check Bug Contest timer flag: {error}"))?
        {
            let timer =
                self.apply_internal_special_routine(&mut staged_state, "CheckBugContestTimer")?;
            bug_contest_timed_out = matches!(
                timer.effect,
                SpecialRoutineEffect::BugContestTimer { active: false, .. }
            );
        }

        if bug_contest_timed_out {
            // CheckTimeEvents calls the source script. Its announcement,
            // waitbutton, and results warp must retain their authored order.
            staged_state.script_runtime.next_script = Some(ScriptLocation {
                origin_map_name: staged_session.map.name.clone(),
                script: "BugCatchingContestOverScript".to_string(),
            });
            staged_state.script_runtime.script_ended = None;
        }

        let mut movement = None;
        let mut ledge_jump = None;
        let mut grass_rustle = None;
        let mut phone_call = None;
        let mut step_events = None;
        let mut coord_event = None;
        let mut trainer_sight = None;
        let mut warp = None;
        let mut connection = None;
        let mut interaction = None;
        let mut wild_encounter = None;
        let mut wild_battle = None;

        let overworld_input_locked =
            bug_contest_timed_out || Self::game_state_blocks_overworld_input(&staged_state);
        let downhill = !overworld_input_locked
            && input_candidate == 0
            && matches!(
                staged_session.player.mode,
                MovementMode::Bike | MovementMode::Skate
            )
            && staged_state
                .flags
                .is_engine_flag_set("ENGINE_DOWNHILL")
                .map_err(|error| anyhow::anyhow!("check downhill bike flag: {error}"))?;
        let tile_forced_direction = if overworld_input_locked {
            None
        } else {
            staged_session.forced_movement_direction()
        };
        let tile_forced_permission = tile_forced_direction.and_then(|_| {
            sample_collision(
                &staged_session.map,
                &staged_session.tileset,
                staged_session.player.tile,
            )
            .map(|sample| sample.permission)
        });
        let forced_direction = tile_forced_direction.or(downhill.then_some(Direction::Down));
        let direction = if overworld_input_locked {
            None
        } else if forced_direction.is_some() {
            forced_direction
        } else if pressed_mask & B_PAD_A != 0 {
            None
        } else {
            direction_from_pad_mask(input_mask)
                .map_err(|error| anyhow::anyhow!("apply overworld input: {error:?}"))?
        };

        if overworld_input_locked {
            staged_session.frame += 1;
        } else if let Some(direction) = direction {
            let movement_mode_before = staged_session.player.mode;
            let direct_forced_step = tile_forced_permission.is_some_and(|permission| {
                !matches!(permission, permissions::ICE | permissions::ICE_2B)
            });
            let blocked_connection_edge = if direct_forced_step {
                None
            } else {
                self.blocked_connection_edge_target(&staged_session, direction)?
            };
            let mut warp_trigger = None;
            if let Some(target) = blocked_connection_edge {
                staged_session.player.facing = direction;
                staged_session.frame += 1;
                movement = Some(StepOutcome::Blocked {
                    at: target,
                    facing: direction,
                });
            } else {
                let options = StepOptions {
                    // CheckTile-owned currents, walk tiles, doors and ice
                    // continue directly. Downhill instead enters through the
                    // joypad path and must preserve CheckTurning's four-frame
                    // facing change before its forced downward step.
                    force_step_after_turn: tile_forced_direction.is_some(),
                    ..StepOptions::default()
                };
                let strength_active = staged_state
                    .flags
                    .is_engine_flag_set("ENGINE_STRENGTH_ACTIVE")
                    .map_err(|error| anyhow::anyhow!("check active Strength flag: {error}"))?;
                let can_jump_ledge = !direct_forced_step
                    && staged_session.can_jump_ledge_checked(direction, options)?;
                let _requested_boulder = (strength_active && !direct_forced_step && !can_jump_ledge)
                    .then(|| {
                        staged_session
                            .request_strength_boulder_push_checked(direction, options)
                            .with_context(|| {
                                format!("push Strength boulder on {}", staged_session.map.name)
                            })
                    })
                    .transpose()?
                    .flatten();
                if direct_forced_step {
                    movement = Some(
                        staged_session
                            .forced_tile_step_checked(direction, options)
                            .with_context(|| {
                                format!(
                                    "apply direct CheckTile movement on {}",
                                    staged_session.map.name
                                )
                            })?,
                    );
                } else if can_jump_ledge {
                    let result = staged_session
                        .ledge_jump_and_check_warp_checked(direction, options)
                        .with_context(|| {
                            format!("apply overworld ledge jump on {}", staged_session.map.name)
                        })?;
                    movement = Some(match &result.outcome {
                        LedgeJumpOutcome::Jumped {
                            from,
                            to,
                            speed_multiplier,
                            ..
                        } => StepOutcome::Moved {
                            from: *from,
                            to: *to,
                            speed_multiplier: *speed_multiplier,
                        },
                        LedgeJumpOutcome::BlockedLanding { at, facing }
                        | LedgeJumpOutcome::NotLedge { at, facing } => StepOutcome::Blocked {
                            at: *at,
                            facing: *facing,
                        },
                        LedgeJumpOutcome::BlockedByObject {
                            at,
                            facing,
                            object_identifier,
                        } => StepOutcome::BlockedByObject {
                            at: *at,
                            facing: *facing,
                            object_identifier: object_identifier.clone(),
                        },
                        LedgeJumpOutcome::RuntimeTileOverflow { from, facing } => {
                            StepOutcome::RuntimeTileOverflow {
                                from: *from,
                                facing: *facing,
                            }
                        }
                    });
                    ledge_jump = Some(result.outcome);
                    warp_trigger = result.warp;
                } else {
                    let result = staged_session
                        .step_and_check_warp_checked(direction, options)
                        .with_context(|| {
                            format!("apply overworld movement on {}", staged_session.map.name)
                        })?;
                    movement = Some(result.outcome);
                    warp_trigger = result.warp;
                }
                if ledge_jump.is_none()
                    && let Some(permission) = tile_forced_permission
                    && let Some(StepOutcome::Moved {
                        speed_multiplier, ..
                    }) = movement.as_mut()
                {
                    // CheckTile uses STEP_WALK for currents, directional walk
                    // tiles, doors, staircases and caves regardless of the
                    // player's bike/skate state. Ice bypasses CheckTile and
                    // TryStep selects STEP_ICE's four-pixel slide instead.
                    let forced_speed =
                        if matches!(permission, permissions::ICE | permissions::ICE_2B) {
                            2
                        } else {
                            1
                        };
                    *speed_multiplier = forced_speed;
                    // step_checked initially retains the origin from the
                    // actor mode. The source-selected step function owns that
                    // collision tile until its visible 8/4-frame landing.
                    staged_session.player_last_tile_occupied_until_frame = staged_session
                        .frame
                        .saturating_add(u64::from(8 / forced_speed) - 1);
                }
                if tile_forced_direction.is_none()
                    && staged_state
                        .flags
                        .is_engine_flag_set("ENGINE_DOWNHILL")
                        .map_err(|error| {
                            anyhow::anyhow!("check downhill bike speed flag: {error}")
                        })?
                    && matches!(
                        movement_mode_before,
                        MovementMode::Bike | MovementMode::Skate
                    )
                    && direction != Direction::Down
                    && let Some(StepOutcome::Moved {
                        speed_multiplier, ..
                    }) = movement.as_mut()
                {
                    // ASM TryStep falls back to STEP_WALK when a downhill
                    // bike/skate actor moves in any direction except down.
                    *speed_multiplier = 1;
                    // `OverworldSession::step_checked` initially retained
                    // the vacated tile using the actor mode's ordinary bike
                    // duration. Keep OBJECT_LAST_MAP_* collision ownership
                    // for the complete overridden eight-frame walk instead
                    // of releasing it halfway through the visible stride.
                    staged_session.player_last_tile_occupied_until_frame =
                        staged_session.frame.saturating_add(7);
                }
            }
            if staged_session.player.mode != movement_mode_before {
                self.sync_current_map_music(
                    &mut staged_state,
                    &staged_session.map.name,
                    staged_session.player.mode,
                    music_ids,
                )?;
            }
            let moved = matches!(movement, Some(StepOutcome::Moved { .. }));
            // Door/cave carpet warps at a map edge activate from the tile the
            // player is standing on once the required facing is established.
            // The attempted outbound step can be collision-blocked; TS/ASM
            // still run the warp check after the turn. Checking only the
            // destination of a successful step strands every such doorway.
            if warp_trigger.is_none() {
                warp_trigger = if moved {
                    staged_session.check_warp_tile_checked()
                } else if matches!(movement, Some(StepOutcome::Turned { .. })) {
                    // `.CheckTurning` returns before `.CheckWarp`. A
                    // directional carpet therefore needs another matching
                    // input after the player has finished turning.
                    Ok(None)
                } else {
                    staged_session.check_warp_checked()
                }
                .with_context(|| format!("check current warp on {}", staged_session.map.name))?;
            }
            if !moved {
                if let Some(trigger) = warp_trigger.take() {
                    let map_setup = player_event_warp_map_setup(trigger.permission);
                    let transition =
                        self.resolve_warp_transition_with_state(&mut staged_state, &trigger)?;
                    self.apply_dig_warp_memory_for_transition(&mut staged_state, &transition)?;
                    let destination = &transition.destination;
                    let mode = staged_session.player.mode;
                    self.transition_overworld_session_with_mode(
                        &mut staged_state,
                        &mut staged_session,
                        &destination.map_name,
                        destination.tile,
                        mode,
                        map_setup,
                        SpawnMemoryUpdate::Preserve,
                        music_ids,
                    )?;
                    warp = Some(transition);
                }
            }
            if moved {
                if let Some(StepOutcome::Moved {
                    to,
                    speed_multiplier,
                    ..
                }) = movement.as_ref()
                {
                    let grass_permission =
                        sample_collision(&staged_session.map, &staged_session.tileset, *to)
                            .map(|sample| sample.permission);
                    if grass_permission.is_some_and(spawns_shaking_grass_object) {
                        grass_rustle = Some(OverworldGrassRustle {
                            tile: *to,
                            duration_frames: grass_rustle_duration_for_speed(*speed_multiplier)?,
                        });
                    }
                }
                // PlayerEvents checks trainer sight before CheckTileEvent.
                // Within CheckTileEvent the exact order is connection, warp,
                // coord event, CountStep, then a random encounter. Any earlier
                // event returns carry and therefore leaves every later step
                // counter untouched.
                trainer_sight =
                    self.check_trainer_sight_after_step(&staged_state, &staged_session)?;
                if trainer_sight.is_none() {
                    let connection_trigger =
                        staged_session.check_connection_checked().with_context(|| {
                            format!("check connection on {}", staged_session.map.name)
                        })?;
                    if let Some(trigger) = connection_trigger {
                        let transition = self.resolve_connection_transition(&trigger)?;
                        let destination = &transition.destination;
                        let mode = staged_session.player.mode;
                        self.transition_overworld_session_with_mode(
                            &mut staged_state,
                            &mut staged_session,
                            &destination.map_name,
                            destination.tile,
                            mode,
                            "MAPSETUP_CONNECTION",
                            SpawnMemoryUpdate::Preserve,
                            music_ids,
                        )?;
                        connection = Some(transition);
                    } else if let Some(trigger) = warp_trigger {
                        let map_setup = player_event_warp_map_setup(trigger.permission);
                        let transition =
                            self.resolve_warp_transition_with_state(&mut staged_state, &trigger)?;
                        self.apply_dig_warp_memory_for_transition(&mut staged_state, &transition)?;
                        let destination = &transition.destination;
                        let mode = staged_session.player.mode;
                        self.transition_overworld_session_with_mode(
                            &mut staged_state,
                            &mut staged_session,
                            &destination.map_name,
                            destination.tile,
                            mode,
                            map_setup,
                            SpawnMemoryUpdate::Preserve,
                            music_ids,
                        )?;
                        warp = Some(transition);
                    } else {
                        coord_event = self
                            .check_coord_event_after_step_checked(&staged_state, &staged_session)?;
                        if coord_event.is_none() {
                            phone_call = self.check_special_phone_call_after_step(
                                &mut staged_state,
                                &staged_session.map.name,
                            )?;
                            if phone_call.is_none() {
                                step_events = Some(self.process_overworld_step(
                                    &mut staged_state,
                                    &staged_session.map.name,
                                    staged_session.player.mode,
                                    &mut rng,
                                )?);
                                let count_step_completed =
                                    step_events.as_ref().is_some_and(|events| {
                                        events.repel_expired.is_none()
                                            && !events.egg_hatched
                                            && events.poison_result.is_none()
                                    });
                                if count_step_completed {
                                    wild_encounter = self.check_wild_encounter_after_step(
                                        &mut staged_state,
                                        &staged_session,
                                        &mut rng,
                                    )?;
                                    wild_battle = self.start_resolved_wild_encounter_after_step(
                                        &mut staged_state,
                                        &staged_session,
                                        &wild_encounter,
                                        &mut rng,
                                    )?;
                                }
                            }
                        }
                    }
                }
            }
        } else {
            staged_session.frame += 1;
        }

        if !overworld_input_locked && direction.is_none() && pressed_mask & B_PAD_A != 0 {
            let candidate = staged_session
                .check_interaction_checked(StepOptions::default().stride_tiles)
                .with_context(|| {
                    format!("check overworld interaction on {}", staged_session.map.name)
                })?;
            interaction = if let Some(candidate) = candidate {
                match self.resolve_overworld_interaction(&staged_state, &candidate)? {
                    Some(interaction) => Some(interaction),
                    None if matches!(
                        candidate.target,
                        OverworldInteractionTarget::Background { .. }
                    ) =>
                    {
                        // TryBGEvent returning carry-clear does not consume
                        // the A press. ASM continues into the facing tile's
                        // collision handler, which is how the default Town
                        // Map remains usable beneath the conditional custom
                        // poster event in the player's room.
                        sample_collision(
                            &staged_session.map,
                            &staged_session.tileset,
                            candidate.target_tile,
                        )
                        .and_then(|sample| {
                            standard_interaction_script(sample.permission).map(|script| {
                                OverworldInteraction {
                                    map_name: candidate.map_name,
                                    player_tile: candidate.player_tile,
                                    facing: candidate.facing,
                                    target_tile: candidate.target_tile,
                                    script: script.to_string(),
                                    target: OverworldInteractionTarget::Collision {
                                        permission: sample.permission,
                                    },
                                }
                            })
                        })
                    }
                    None => None,
                }
            } else {
                None
            };
        }

        if (!overworld_input_locked || staged_session.has_required_object_steps())
            && wild_battle.is_none()
            && phone_call.is_none()
            && warp.is_none()
            && connection.is_none()
        {
            let has_autonomous_object = staged_session.objects.iter().any(|object| {
                matches!(
                    object.spritemovedata.as_str(),
                    "SPRITEMOVEDATA_WALK_LEFT_RIGHT"
                        | "SPRITEMOVEDATA_WALK_UP_DOWN"
                        | "SPRITEMOVEDATA_WANDER"
                        | "SPRITEMOVEDATA_SWIM_WANDER"
                        | "SPRITEMOVEDATA_SPINRANDOM_SLOW"
                        | "SPRITEMOVEDATA_SPINRANDOM_FAST"
                        | "SPRITEMOVEDATA_SPINCLOCKWISE"
                        | "SPRITEMOVEDATA_SPINCOUNTERCLOCKWISE"
                )
            });
            if has_autonomous_object || staged_session.has_required_object_steps() {
                let object_advance = if overworld_input_locked {
                    staged_session.advance_required_object_steps_exact(&mut rng)
                } else {
                    staged_session.advance_autonomous_objects_exact(&mut rng)
                }
                    .map_err(|error| {
                        anyhow::anyhow!("advance autonomous overworld objects: {error}")
                    })?;
                for _object_id in object_advance.started_strength_boulders {
                    staged_state
                        .script_runtime
                        .audio_events
                        .push(ScriptAudioRuntimeEvent {
                            command: "playsound".to_string(),
                            kind: ScriptAudioRuntimeKind::SoundEffect,
                            audio_id: Some("SFX_STRENGTH".to_string()),
                            fade_frames: None,
                            source_script: "MovementFunction_Strength".to_string(),
                            command_index: 0,
                        });
                }
            }
        }

        let autonomous_objects_changed = session.object_runtime_tiles
            != staged_session.object_runtime_tiles
            || session.object_facings != staged_session.object_facings;
        if warp.is_some() || connection.is_some() {
            grass_rustle = None;
        }
        let snapshot = staged_session.snapshot();
        staged_state.random_state = rng.state();
        self.commit_overworld_snapshot_data(
            &mut staged_state,
            &snapshot,
            SpawnMemoryUpdate::Preserve,
        );
        *state = staged_state;
        *session = staged_session;
        Ok(OverworldInputFrame {
            snapshot,
            input_mask,
            pressed_mask,
            autonomous_objects_changed,
            movement,
            ledge_jump,
            grass_rustle,
            phone_call,
            step_events,
            coord_event,
            trainer_sight,
            interaction,
            warp,
            connection,
            wild_encounter,
            wild_battle,
        })
    }

    fn blocked_connection_edge_target(
        &self,
        session: &OverworldSession,
        direction: Direction,
    ) -> Result<Option<TilePosition>> {
        if session.player.facing != direction {
            return Ok(None);
        }
        let Some(target) = checked_move_by_stride(
            session.player.tile,
            direction,
            StepOptions::default().stride_tiles,
        ) else {
            return Ok(None);
        };
        let mut probe = session.clone();
        probe.player.tile = target;
        let Some(trigger) = probe
            .check_connection_checked()
            .with_context(|| format!("check connection on {}", probe.map.name))?
        else {
            return Ok(None);
        };
        if self.connection_trigger_has_destination(&trigger)? {
            Ok(None)
        } else {
            Ok(Some(target))
        }
    }

    fn connection_trigger_has_destination(&self, trigger: &ConnectionTrigger) -> Result<bool> {
        let target_attributes = self
            .map_attributes
            .get(&trigger.connection.target_map)
            .with_context(|| {
                format!(
                    "connection target '{}' missing attributes (referenced by {})",
                    trigger.connection.target_map, trigger.map_name
                )
            })?;
        connection_destination_tile_in_bounds(
            trigger.tile,
            &trigger.connection.direction,
            trigger.connection.offset,
            target_attributes,
        )
    }

    fn game_state_blocks_overworld_input(state: &GameState) -> bool {
        let runtime = &state.script_runtime;
        !matches!(state.battle, BattleMemory::Inactive)
            || runtime.player_input_locked
            || runtime.all_input_locked
            || !runtime.pending_delays.is_empty()
            || !runtime.pending_earthquakes.is_empty()
            || !runtime.pending_emotes.is_empty()
            || !runtime.command_queue.is_empty()
            || runtime.active_menu.is_some()
            || runtime.active_pokemon_picture.is_some()
            || runtime.window_open
            || runtime.text_window_open
            || !runtime.audio_events.is_empty()
            || runtime.pending_music_fade.is_some()
            || runtime.waiting_for_sound_effect
            || !runtime.graphics_events.is_empty()
            || runtime.pending_screen_fade.is_some()
            || !runtime.money_events.is_empty()
            || !runtime.map_events.is_empty()
            || runtime.pending_script_warp.is_some()
            || runtime.pending_map_load.is_some()
            || runtime.pending_map_refresh.is_some()
            // Completed text events are retained for rendering/history. The
            // pending label/wait/window fields below are the authoritative
            // input owners; archival lines must not freeze the overworld.
            || runtime.pending_text_label.is_some()
            || runtime.pending_text_wait.is_some()
            || runtime.pending_yes_no.is_some()
            || !runtime.control_events.is_empty()
            || runtime.next_script.is_some()
            || runtime.map_reentry_script.is_some()
            || !runtime.deferred_scripts.is_empty()
            // `script_ended` is a retained completion/history record. The
            // executable cursor, queues, and authored wait fields above own
            // input locking; an archived callback end must not permanently
            // disable the next overworld A press.
            || !runtime.shop_events.is_empty()
            || runtime.pending_shop.is_some()
            || !runtime.item_use_events.is_empty()
    }

    pub fn cry_by_species(&self) -> BTreeMap<String, String> {
        self.pokemon_cries
            .iter()
            .map(|(species_id, cry)| (species_id.clone(), cry.cry.clone()))
            .collect()
    }

    pub fn special_routine_context<'a>(
        &'a self,
        cry_by_species: &'a BTreeMap<String, String>,
    ) -> SpecialRoutineContext<'a> {
        SpecialRoutineContext {
            move_catalog: &self.moves,
            cry_by_species,
            species_catalog: &self.pokemon,
            learnsets: &self.learnsets,
            growth_rates: &self.growth_rates,
            item_catalog: &self.items,
            runtime_spawn_points: self.runtime_spawn_points(),
            roaming_pokemon: &self.roaming_pokemon,
            buena_password_categories: &self.buena_password_categories,
            buena_prizes: &self.buena_prizes,
            kurt_apricorn_recipes: &self.kurt_apricorn_recipes,
            shuckie_gift: self.shuckie_gift.as_ref(),
            dratini_move_sets: &self.dratini_move_sets,
            bug_contest_config: self.bug_contest_config.as_ref(),
            battle_tower_rules: self.battle_tower_rules.as_ref(),
            magikarp_lengths: &self.magikarp_lengths,
            happiness_data: self.happiness_data.as_ref(),
            trainer_catalog: &self.trainers,
            phone_contacts: &self.phone_contacts,
            wild_encounters: &self.wild_encounters,
            odd_egg_definitions: &self.odd_egg_definitions,
            oak_ratings: &self.oak_ratings,
        }
    }

    pub fn apply_special_routine(
        &self,
        state: &mut GameState,
        routine: &str,
        _music_ids: &BTreeSet<String>,
    ) -> Result<SpecialRoutineOutcome> {
        self.require_special_routine(routine)?;
        let fainted_slots = (self.nuzlocke_rules.permadeath
            && matches!(routine, "HealParty" | "BattleTowerBattle"))
            .then(|| {
                state
                    .storage
                    .party
                    .pokemon
                    .iter()
                    .enumerate()
                    .filter_map(|(index, pokemon)| {
                        pokemon.as_ref().is_some_and(|pokemon| pokemon.hp == 0).then_some(index)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let cry_by_species = self.cry_by_species();
        let context = self.special_routine_context(&cry_by_species);
        let mut outcome = apply_special_routine_with_context(state, context, routine)
            .map_err(|error| anyhow::anyhow!("apply special routine {routine}: {error}"))?;
        if !fainted_slots.is_empty() {
            for party_index in &fainted_slots {
                if let Some(Some(pokemon)) = state.storage.party.pokemon.get_mut(*party_index) {
                    pokemon.hp = 0;
                }
            }
            if let SpecialRoutineEffect::HealParty { healed_slots } = &mut outcome.effect {
                healed_slots.retain(|index| !fainted_slots.contains(index));
            }
            state.sync_party_from_storage();
        }
        Ok(outcome)
    }

    pub fn apply_random_special_routine<S>(
        &self,
        state: &mut GameState,
        routine: &str,
        _music_ids: &BTreeSet<String>,
        divider: &mut S,
    ) -> Result<SpecialRoutineOutcome>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        self.require_special_routine(routine)?;
        if self.nuzlocke_rules.permadeath
            && matches!(routine, "DayCareMan" | "DayCareLady")
            && let Some(DayCareInput::Deposit { party_slot }) =
                state.script_runtime.pending_day_care_input.as_ref()
        {
            let pokemon = state
                .storage
                .party
                .pokemon
                .get(*party_slot)
                .and_then(Option::as_ref)
                .with_context(|| {
                    format!("Nuzlocke Day Care deposit party slot {party_slot} is empty")
                })?;
            crate::nuzlocke::ensure_can_restore_hp(self.nuzlocke_rules, pokemon)
                .context("Nuzlocke permadeath forbids depositing a fainted Pokemon")?;
        }
        if !runtime_special_routine_requires_divider_trace(routine) {
            anyhow::bail!("special routine {routine} does not use the exact divider boundary");
        }
        let cry_by_species = self.cry_by_species();
        let context = self.special_routine_context(&cry_by_species);
        let mut next = state.clone();
        let egg_before = next.day_care.egg.clone();
        let outcome =
            apply_random_special_routine_with_context(&mut next, context, routine, divider)
                .map_err(|error| {
                    anyhow::anyhow!("apply random special routine {routine}: {error}")
                })?;
        if next.day_care.egg != egg_before {
            self.normalize_day_care_egg_species(&mut next)?;
        }
        *state = next;
        Ok(outcome)
    }

    pub fn apply_internal_special_routine(
        &self,
        state: &mut GameState,
        routine: &str,
    ) -> Result<SpecialRoutineOutcome> {
        if !matches!(routine, "StartBugContestTimer" | "CheckBugContestTimer") {
            anyhow::bail!("unsupported internal special routine {routine}");
        }
        let cry_by_species = self.cry_by_species();
        let context = self.special_routine_context(&cry_by_species);
        apply_special_routine_with_context(state, context, routine)
            .map_err(|error| anyhow::anyhow!("apply internal special routine {routine}: {error}"))
    }

    fn apply_special_routine_transactional<F>(
        &self,
        state: &mut GameState,
        routine: &str,
        music_ids: &BTreeSet<String>,
        prepare: F,
    ) -> Result<SpecialRoutineOutcome>
    where
        F: FnOnce(&mut GameState) -> Result<()>,
    {
        let mut next_state = state.clone();
        prepare(&mut next_state)?;
        let outcome = self.apply_special_routine(&mut next_state, routine, music_ids)?;
        *state = next_state;
        Ok(outcome)
    }

    pub fn audio_ids(&self) -> BTreeSet<&str> {
        self.audio.iter().map(|asset| asset.id.as_str()).collect()
    }

    pub fn audio_manifest(
        &self,
        compiled_audio: &BTreeMap<String, Vec<u8>>,
    ) -> Result<ModpackAudioManifest> {
        ModpackAudioManifest::from_assets(&self.audio, compiled_audio)
    }

    pub fn script_text_labels(module: &MapModule) -> BTreeSet<String> {
        module.script_text_bodies.keys().cloned().collect()
    }

    pub fn script_numeric_constants(&self) -> BTreeMap<String, i32> {
        let mut constants = BTreeMap::new();
        constants.insert(
            "PARTY_LENGTH".to_string(),
            crystal_core::models::PARTY_SIZE as i32,
        );
        // Script_AskForPhoneNumber returns these three script-byte values.
        // Keep them available even in packs whose derived constant catalog
        // omitted the shared phone-registration branches.
        constants.extend([
            ("PHONE_CONTACT_GOT".to_string(), 0),
            ("PHONE_CONTACTS_FULL".to_string(), 1),
            ("PHONE_CONTACT_REFUSED".to_string(), 2),
        ]);
        constants.extend(crystal_core::systems::special_routines::KURT_APRICORN_SCRIPT_VALUES
            .iter().map(|(item_id, value)| ((*item_id).to_string(), i32::from(*value))));
        // checkmoney/checkcoins return these engine comparison bytes. Resolve
        // their branch operands even when the pack omits the shared constants.
        use crystal_core::systems::economy::AmountComparison;
        for comparison in [AmountComparison::HaveMore, AmountComparison::HaveAmount, AmountComparison::HaveLess] {
            constants.insert(
                comparison.script_label().to_string(),
                comparison.script_code().parse().expect("amount comparison byte"),
            );
        }
        if let Some(rules) = &self.battle_tower_rules {
            constants.insert(
                "BATTLETOWER_STREAK_LENGTH".to_string(),
                i32::from(rules.challenge_streak_length),
            );
            if let Ok(required_party_count) = i32::try_from(rules.required_party_count) {
                constants.insert("BATTLETOWER_PARTY_LENGTH".to_string(), required_party_count);
            }
            constants.extend(
                rules
                    .reward_item_values
                    .iter()
                    .map(|(item_id, value)| (item_id.clone(), i32::from(*value))),
            );
        }
        for (constant, value) in &self.currency_constants.0 {
            if let Ok(value) = i32::try_from(*value) {
                constants.insert(constant.clone(), value);
            }
        }
        for (constant, value) in &self.story_event_script_constants.global {
            if let Ok(value) = i32::try_from(*value) {
                constants.insert(constant.clone(), value);
            }
        }
        for constants_by_map in self.story_event_script_constants.maps.values() {
            for (constant, value) in constants_by_map {
                if let Ok(value) = i32::try_from(*value) {
                    constants.insert(constant.clone(), value);
                }
            }
        }
        constants
    }

    pub fn saved_map_id(&self, map_name: &str) -> Option<&str> {
        self.maps.get(map_name).map(|module| module.id.as_str())
    }

    pub fn validate_saved_pokedex_references(&self, pokedex: &PokedexState) -> Result<()> {
        validate_saved_pokedex_references(pokedex, |species| self.saved_species_id(species))
            .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_bag_references(&self, bag: &Bag) -> Result<()> {
        validate_saved_bag_pocket_references(
            &self.items,
            "bag.items",
            &bag.items,
            ITEM_POCKET_ITEM,
        )
        .map_err(|error| anyhow::anyhow!("{error}"))?;
        validate_saved_pc_item_references(&self.items, "bag.pc_items", &bag.pc_items)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        validate_saved_bag_pocket_references(
            &self.items,
            "bag.balls",
            &bag.balls,
            ITEM_POCKET_BALL,
        )
        .map_err(|error| anyhow::anyhow!("{error}"))?;
        validate_saved_bag_pocket_references(
            &self.items,
            "bag.key_items",
            &bag.key_items,
            ITEM_POCKET_KEY_ITEM,
        )
        .map_err(|error| anyhow::anyhow!("{error}"))?;
        for (pocket_id, inventory) in &bag.custom_pockets {
            validate_saved_bag_pocket_references(
                &self.items,
                &format!("bag.custom_pockets.{pocket_id}"),
                inventory,
                pocket_id,
            )
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        }
        validate_saved_tmhm_references(&self.items, &bag.tm_hm)
            .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_mom_purchase_references(&self, state: &GameState) -> Result<()> {
        anyhow::ensure!(
            usize::from(state.mom_item_index)
                <= self.battle_reward_rules.mom_progression_items.len(),
            "saved mom_item_index {} exceeds progression table length {}",
            state.mom_item_index,
            self.battle_reward_rules.mom_progression_items.len()
        );
        anyhow::ensure!(
            state.mom_item_trigger_balance % self.battle_reward_rules.mom_money_increment == 0,
            "saved mom_item_trigger_balance {} is not a multiple of {}",
            state.mom_item_trigger_balance,
            self.battle_reward_rules.mom_money_increment
        );
        let Some(purchase) = state.pending_mom_purchase.as_ref() else {
            anyhow::ensure!(
                !state
                    .script_runtime
                    .map_reentry_script
                    .as_ref()
                    .is_some_and(|script| matches!(
                        script.script.as_str(),
                        ".ItemScript@Mom_GetScriptPointer" | ".DollScript@Mom_GetScriptPointer"
                    )),
                "saved Mom result map reentry script requires a pending Mom purchase"
            );
            return Ok(());
        };
        anyhow::ensure!(
            state
                .script_runtime
                .pending_map_load
                .as_ref()
                .is_some_and(|load| load.command == "reloadmapafterbattle"),
            "saved pending Mom purchase requires reloadmapafterbattle map-load state"
        );
        let expected_reentry_script = if purchase.decoration_flag.is_some() {
            ".DollScript@Mom_GetScriptPointer"
        } else {
            ".ItemScript@Mom_GetScriptPointer"
        };
        anyhow::ensure!(
            state
                .script_runtime
                .map_reentry_script
                .as_ref()
                .is_some_and(|script| script.script == expected_reentry_script),
            "saved pending Mom purchase requires exact map reentry script {expected_reentry_script}"
        );
        let rule = if purchase.progression {
            anyhow::ensure!(
                purchase.selected_index == state.mom_item_index,
                "saved pending Mom progression index {} does not match mom_item_index {}",
                purchase.selected_index,
                state.mom_item_index
            );
            self.battle_reward_rules
                .mom_progression_items
                .get(usize::from(purchase.selected_index))
        } else {
            self.battle_reward_rules
                .mom_random_items
                .get(usize::from(purchase.selected_index))
        }
        .with_context(|| {
            format!(
                "saved pending Mom purchase index {} is outside its source table",
                purchase.selected_index
            )
        })?;
        anyhow::ensure!(
            purchase.cost == rule.cost
                && purchase.target == rule.target
                && purchase.decoration_flag == rule.decoration_flag,
            "saved pending Mom purchase does not match compiled source row"
        );
        match rule.kind {
            MomPurchaseKind::Item => {
                let item = self.items.get(&rule.target).with_context(|| {
                    format!(
                        "saved pending Mom item {} is missing from compiled pack",
                        rule.target
                    )
                })?;
                anyhow::ensure!(
                    state.bag.pc_item_quantity(item) > 0,
                    "saved pending Mom item {} was not delivered to PC storage",
                    rule.target
                );
            }
            MomPurchaseKind::Doll => {
                let flag = rule.decoration_flag.as_deref().with_context(|| {
                    format!(
                        "saved pending Mom doll {} lacks a decoration flag",
                        rule.target
                    )
                })?;
                anyhow::ensure!(
                    state
                        .flags
                        .is_event_flag_set(flag)
                        .with_context(|| format!("read saved pending Mom doll flag {flag}"))?,
                    "saved pending Mom doll flag {flag} is not set"
                );
            }
        }
        Ok(())
    }

    pub fn validate_saved_active_repel_item(
        &self,
        item_id: &str,
        steps_remaining: u16,
    ) -> Result<()> {
        core_validate_saved_active_repel_item(
            &self.field_moves,
            item_id,
            self.saved_item(item_id),
            steps_remaining,
        )
        .map_err(|error| anyhow::anyhow!("saved active_repel_item {item_id}: {error:?}"))
    }

    pub fn validate_saved_overworld_references(&self, overworld: &OverworldMemory) -> Result<()> {
        core_validate_saved_overworld_references(overworld, |map_name| {
            self.saved_map_tile_bounds(map_name)
        })
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_scene_references(&self, scenes: &SceneMemory) -> Result<()> {
        core_validate_saved_scene_references(
            scenes,
            |map_name| self.saved_map_id(map_name).is_some(),
            |map_name, scene_name| self.saved_scene_index(map_name, scene_name),
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_block_overrides(
        &self,
        map_name: &str,
        overrides: &BTreeMap<(u16, u16), u16>,
    ) -> Result<()> {
        core_validate_saved_block_overrides(
            map_name,
            overrides,
            |map_name| self.saved_map_block_context(map_name),
            |tileset_name| self.saved_tileset_exists(tileset_name),
            |tileset_name, block_id| self.tileset_declares_metatile(tileset_name, block_id),
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_object_overrides(
        &self,
        map_name: &str,
        memory: &OverworldObjectMapMemory,
    ) -> Result<()> {
        core_validate_saved_object_overrides(
            map_name,
            memory,
            |_| self.saved_map_tile_bounds(map_name),
            |_| self.map_module(map_name).ok().map(|module| module.objects.len()),
            |object_id| self.map_declares_event_object(map_name, object_id),
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_storage_references(&self, storage: &PokemonStorage) -> Result<()> {
        core_validate_saved_storage_references(storage, |path, pokemon| {
            self.validate_saved_pokemon_reference(path, pokemon)
                .map_err(|error| error.to_string())
        })
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_bug_contest_references(
        &self,
        bug_contest: &BugContestState,
    ) -> Result<()> {
        core_validate_saved_bug_contest_references(
            bug_contest,
            |path, pokemon| {
                self.validate_saved_pokemon_reference(path, pokemon)
                    .map_err(|error| error.to_string())
            },
            |species| self.saved_species_exists(species),
            |flag| self.saved_event_flag_exists(flag),
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_day_care_references(&self, day_care: &DayCareState) -> Result<()> {
        core_validate_saved_day_care_references(day_care, |path, pokemon| {
            self.validate_saved_pokemon_reference(path, pokemon)
                .map_err(|error| error.to_string())
        })
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_roaming_references(
        &self,
        roaming_pokemon: &[RoamingPokemonState; 3],
        roaming_map_history: &crystal_core::state::RoamingMapHistory,
    ) -> Result<()> {
        core_validate_saved_roaming_references(
            roaming_pokemon,
            roaming_map_history,
            &self.roaming_pokemon,
            |species| self.saved_species_exact_exists(species),
            |map_group, map_number| {
                self.runtime_map_group_number_exists(u16::from(map_group), u16::from(map_number))
            },
        )
        .map_err(|error| anyhow::anyhow!("{error}"))?;
        for (index, roaming) in roaming_pokemon.iter().enumerate() {
            let Some(species_id) = roaming.species.as_deref() else {
                continue;
            };
            let species = self
                .pokemon
                .get(species_id)
                .with_context(|| format!("saved roaming_pokemon[{index}].species {species_id} is missing from compiled pack pokemon"))?;
            let dvs = Dv::from_non_hp(
                roaming.dvs_be[0] >> 4,
                roaming.dvs_be[0] & 0x0f,
                roaming.dvs_be[1] >> 4,
                roaming.dvs_be[1] & 0x0f,
            );
            let max_hp = calculate_stats(
                species,
                roaming.level,
                dvs,
                crystal_core::models::pokemon::StatExperience::default(),
            )
            .max_hp;
            anyhow::ensure!(
                u16::from(roaming.hp) <= max_hp,
                "saved roaming_pokemon[{index}] hp {} exceeds {} level {} max HP {max_hp} for saved DVs {:02x}{:02x}",
                roaming.hp,
                species_id,
                roaming.level,
                roaming.dvs_be[0],
                roaming.dvs_be[1]
            );
        }
        Ok(())
    }

    pub fn validate_saved_mystery_gift_references(
        &self,
        mystery_gift: &MysteryGiftState,
    ) -> Result<()> {
        core_validate_saved_mystery_gift_references(mystery_gift, |item_id| {
            self.saved_item_exists(item_id)
        })
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_magikarp_record_references(
        &self,
        record: &MagikarpRecordState,
    ) -> Result<()> {
        validate_saved_magikarp_record_references(record, !self.magikarp_lengths.is_empty())
            .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_blue_card_references(&self, state: &GameState) -> Result<()> {
        validate_saved_blue_card_balance(state, !self.buena_prizes.is_empty())
            .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_buena_password_references(
        &self,
        password: &BuenasPasswordState,
    ) -> Result<()> {
        validate_saved_buena_password_references(password, &self.buena_password_categories)
            .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_battle_tower_references(
        &self,
        tower: &BattleTowerState,
        party: &Party,
    ) -> Result<()> {
        if tower.reward_item != BattleTowerState::default().reward_item {
            self.validate_saved_item_reference("battle_tower.reward_item", &tower.reward_item)?;
        }
        core_validate_saved_battle_tower_state(tower, party, self.battle_tower_rules.as_ref())
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        if saved_battle_tower_state_is_active(tower) {
            self.validate_saved_item_reference("battle_tower.reward_item", &tower.reward_item)?;
        }
        if let Some(trainer_id) = &tower.loaded_trainer_id {
            let canonical_tower_trainer = self.battle_tower_rules.as_ref().is_some_and(|rules| {
                rules
                    .trainers
                    .iter()
                    .any(|trainer| format!("BATTLE_TOWER_{}", trainer.index) == *trainer_id)
            });
            if !canonical_tower_trainer {
                let _ = self.validate_saved_trainer_reference(
                    "battle_tower.loaded_trainer_id",
                    trainer_id,
                )?;
            }
        }
        Ok(())
    }

    pub fn validate_saved_fishing_references(&self, fishing: &FishingMemory) -> Result<()> {
        core_validate_saved_fishing_references(
            fishing,
            |rod| self.saved_fishing_rod_exists(rod),
            |bit| self.saved_fishing_daily_flag_bit_exists(bit),
            |swarm_flag| self.saved_fishing_swarm_flag_exists(swarm_flag),
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_swarm_references(&self, swarms: &SwarmMemory) -> Result<()> {
        let map_groups = self.runtime_map_group_table();
        for (swarm_token, target) in &swarms.active {
            let Some((group_id, map_id)) = map_groups.get(&target.map_id).copied() else {
                anyhow::bail!(
                    "saved swarms.active {swarm_token} references missing runtime map {}",
                    target.map_id
                );
            };
            if target.map_group != Some(group_id) || target.map_number != Some(map_id) {
                anyhow::bail!(
                    "saved swarms.active {swarm_token} map {} has group/number {:?}/{:?}, expected {group_id}/{map_id}",
                    target.map_id,
                    target.map_group,
                    target.map_number
                );
            }
        }
        Ok(())
    }

    pub fn validate_saved_pending_special_battle_type(
        &self,
        battle_type: Option<&str>,
    ) -> Result<()> {
        validate_saved_pending_special_battle_type(
            battle_type,
            |battle_type| self.saved_pending_special_battle_type_exists(battle_type),
            |routine| self.saved_special_routine_exists(routine),
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_flag_references(&self, flags: &EventFlagMemory) -> Result<()> {
        core_validate_saved_flag_references(
            flags,
            |flag| self.saved_event_flag_exists(flag),
            |flag| self.saved_engine_flag_exists(flag),
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_event_flag_reference(&self, path: &str, flag: &str) -> Result<()> {
        core_validate_saved_event_flag_reference(path, flag, |flag| {
            self.saved_event_flag_exists(flag)
        })
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_engine_flag_reference(&self, path: &str, flag: &str) -> Result<()> {
        core_validate_saved_engine_flag_reference(path, flag, |flag| {
            self.saved_engine_flag_exists(flag)
        })
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_pokemon_reference(&self, path: &str, pokemon: &Pokemon) -> Result<()> {
        core_validate_saved_pokemon_reference(
            path,
            pokemon,
            |species| self.saved_species(species),
            |item_id| self.saved_item_exists(item_id),
            |status| self.saved_pokemon_status_exists(status),
            |move_name| self.saved_move_name_and_pp(move_name),
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_item_reference(&self, path: &str, item_id: &str) -> Result<()> {
        validate_saved_exact_catalog_reference(
            path,
            item_id,
            "items",
            "item script_name",
            self.saved_item_script_name(item_id),
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_move_reference(&self, path: &str, move_id: &str) -> Result<()> {
        validate_saved_exact_catalog_reference(
            path,
            move_id,
            "moves",
            "move id",
            self.saved_move_name_and_pp(move_id).map(|(name, _)| name),
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_sprite_reference(&self, path: &str, sprite_id: &str) -> Result<()> {
        validate_saved_catalog_reference(path, sprite_id, "sprites", |sprite_id| {
            self.saved_sprite_exists(sprite_id)
        })
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_variable_sprite_reference(
        &self,
        path: &str,
        sprite_id: &str,
    ) -> Result<()> {
        validate_saved_catalog_reference(path, sprite_id, "variable sprites", |sprite_id| {
            self.saved_variable_sprite_exists(sprite_id)
        })
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_trainer_reference<'a>(
        &'a self,
        path: &str,
        trainer_id: &str,
    ) -> Result<&'a Trainer> {
        validate_saved_exact_catalog_reference(
            path,
            trainer_id,
            "trainers",
            "trainer id",
            self.saved_trainer_id(trainer_id),
        )
        .map_err(|error| anyhow::anyhow!("{error}"))?;
        let trainer = self.saved_trainer(trainer_id).with_context(|| {
            format!("saved {path} trainer id {trainer_id} validated but is missing")
        })?;
        Ok(trainer)
    }

    pub fn validate_saved_species_reference(&self, path: &str, species: &str) -> Result<()> {
        validate_saved_exact_catalog_reference(
            path,
            species,
            "pokemon",
            "species id",
            self.saved_species_id(species),
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_audio_reference(
        &self,
        path: &str,
        audio_id: &str,
        expected_kind: ModpackAudioKind,
    ) -> Result<()> {
        if expected_kind == ModpackAudioKind::Music
            && audio_id == crystal_core::systems::script_audio::MUSIC_NONE_ID
        {
            return Ok(());
        }
        let asset = self
            .audio
            .iter()
            .find(|asset| asset.id == audio_id)
            .with_context(|| {
                format!(
                    "save field {path} references missing {} audio id '{audio_id}'",
                    expected_kind.save_name()
                )
            })?;
        core_validate_saved_audio_reference(
            path,
            audio_id,
            expected_kind.save_name(),
            Some(asset.kind.save_name()),
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_map_constant_reference(
        &self,
        path: &str,
        map_constant: &str,
    ) -> Result<()> {
        validate_saved_exact_catalog_reference(
            path,
            map_constant,
            "map constants",
            "map constant",
            self.saved_map_constant(map_constant),
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_text_reference(&self, path: &str, text_label: &str) -> Result<()> {
        validate_saved_catalog_reference(path, text_label, "text", |text_label| {
            self.saved_text_exists(text_label)
        })
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_optional_text_reference(
        &self,
        path: &str,
        text_label: &str,
    ) -> Result<()> {
        validate_saved_optional_catalog_reference(path, text_label, "text", |text_label| {
            self.saved_text_exists(text_label)
        })
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_special_routine_reference(
        &self,
        path: &str,
        routine: &str,
    ) -> Result<()> {
        validate_saved_catalog_reference(path, routine, "special routines", |routine| {
            self.saved_special_routine_exists(routine)
        })
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_spawn_reference(&self, path: &str, spawn_identifier: u16) -> Result<()> {
        validate_saved_exact_catalog_reference(
            path,
            &spawn_identifier.to_string(),
            "runtime spawn points",
            "spawn identifier",
            self.saved_spawn_identifier(spawn_identifier),
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_menu_reference(&self, path: &str, menu: &str) -> Result<()> {
        validate_saved_catalog_reference(path, menu, "menus", |menu| self.saved_menu_exists(menu))
            .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_phone_contact_reference(
        &self,
        path: &str,
        contact_id: &str,
    ) -> Result<()> {
        validate_saved_exact_catalog_reference(
            path,
            contact_id,
            "phone contacts",
            "phone contact id",
            self.saved_phone_contact_id(contact_id),
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_special_phone_call_reference(
        &self,
        path: &str,
        call_id: &str,
    ) -> Result<()> {
        validate_saved_catalog_reference(path, call_id, "special phone calls", |call_id| {
            self.saved_special_phone_call_exists(call_id)
        })
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_npc_trade_reference(&self, path: &str, trade_id: &str) -> Result<()> {
        validate_saved_catalog_reference(path, trade_id, "NPC trades", |trade_id| {
            self.saved_npc_trade_exists(trade_id)
        })
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_pokemon_status_reference(&self, path: &str, status: &str) -> Result<()> {
        validate_saved_catalog_reference(path, status, "status declarations", |status| {
            self.saved_pokemon_status_exists(status)
        })
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_script_label_reference(
        &self,
        path: &str,
        script_label: &str,
    ) -> Result<()> {
        let _ = self.saved_compiled_script_body(path, script_label)?;
        Ok(())
    }

    pub fn saved_compiled_script_body(
        &self,
        path: &str,
        script_label: &str,
    ) -> Result<&serde_json::Value> {
        let script_body = self.compiled_script_body(script_label);
        validate_saved_catalog_reference(path, script_label, "scripts", |_| script_body.is_some())
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        script_body.with_context(|| {
            format!("saved {path} script label {script_label} validated but is missing")
        })
    }

    pub fn validate_saved_script_command_reference(
        &self,
        path: &str,
        script_label: &str,
        command_index: usize,
    ) -> Result<()> {
        validate_saved_compiled_script_command_reference(
            self.saved_compiled_script_body(path, script_label)?,
            path,
            script_label,
            command_index,
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_script_command_name_reference(
        &self,
        path: &str,
        script_label: &str,
        command_index: usize,
        saved_command: &str,
    ) -> Result<()> {
        validate_saved_compiled_script_command_name_reference(
            self.saved_compiled_script_body(path, script_label)?,
            path,
            script_label,
            command_index,
            saved_command,
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn saved_script_command_is(
        &self,
        path: &str,
        script_label: &str,
        command_index: usize,
        expected_command: &str,
    ) -> Result<bool> {
        let body = self.saved_compiled_script_body(path, script_label)?;
        validate_saved_compiled_script_command_reference(body, path, script_label, command_index)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        Ok(body
            .as_array()
            .and_then(|commands| commands.get(command_index))
            .and_then(|command| command.get("command"))
            .and_then(serde_json::Value::as_str)
            == Some(expected_command))
    }

    pub fn validate_saved_script_command_payload_reference(
        &self,
        path: &str,
        script_label: &str,
        command_index: usize,
        saved_command: &str,
        saved_args: &[String],
    ) -> Result<()> {
        validate_saved_compiled_script_command_payload_reference(
            self.saved_compiled_script_body(path, script_label)?,
            path,
            script_label,
            command_index,
            saved_command,
            saved_args,
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_stone_table_entry_command(
        &self,
        path: &str,
        entry: &ScriptRuntimeStoneTableEntry,
    ) -> Result<()> {
        self.validate_saved_script_command_name_reference(
            path,
            &entry.source_script,
            entry.command_index,
            "stonetable",
        )?;
        let body = self.saved_compiled_script_body(path, &entry.source_script)?;
        let target = body
            .as_array()
            .and_then(|commands| commands.get(entry.command_index))
            .and_then(|command| command.get("args"))
            .and_then(Value::as_array)
            .filter(|args| args.len() == 3)
            .and_then(|args| args[2].as_str())
            .with_context(|| {
                format!(
                    "compiled stonetable {}:{} for saved {path} has no exact target payload",
                    entry.source_script, entry.command_index
                )
            })?;
        // Match the source body's namespace, then use the same label resolution
        // as stone-table installation. The saved target is already resolved.
        let definitions = self
            .maps
            .values()
            .find(|module| module.scripts.contains_key(&entry.source_script))
            .map(|module| &module.scripts)
            .or_else(|| {
                self.global_scripts.as_ref().and_then(|module| {
                    module
                        .scripts
                        .contains_key(&entry.source_script)
                        .then_some(&module.definitions)
                })
            })
            .with_context(|| format!("saved {path} has no source script namespace"))?;
        let resolved = resolve_script_target_label(definitions, &entry.source_script, target)
            .with_context(|| {
                format!(
                    "compiled stonetable {}:{} for saved {path} cannot resolve target {target}",
                    entry.source_script, entry.command_index
                )
            })?;
        anyhow::ensure!(
            entry.script == resolved,
            "saved {path} {}:{} script {} does not match compiled stonetable target {resolved}",
            entry.source_script,
            entry.command_index,
            entry.script
        );
        // Restore only the proven target's source spelling for the exact
        // payload check; warp, object, command and index remain authoritative.
        let (command, mut args) =
            crystal_core::state::saved_stone_table_entry_command_payload(entry);
        args[2] = target.to_string();
        self.validate_saved_script_command_payload_reference(
            path,
            &entry.source_script,
            entry.command_index,
            command,
            &args,
        )
    }

    pub fn validate_saved_script_warp_reference(
        &self,
        path: &str,
        warp: &ScriptWarpRequest,
    ) -> Result<()> {
        let raw_tile = runtime_tile_to_raw_event_tile(warp.tile).with_context(|| {
            format!(
                "saved {path} {}:{} pending script warp tile ({}, {}) is not aligned to a raw map event coordinate",
                warp.source_script,
                warp.command_index,
                warp.tile.x,
                warp.tile.y
            )
        })?;
        let mut args = vec![
            self.map_constant(&warp.target_map)?.to_string(),
            raw_tile.x.to_string(),
            raw_tile.y.to_string(),
        ];
        let command = if let Some(facing) = warp.facing {
            args.push(direction_script_token(facing).to_string());
            "warpfacing"
        } else {
            "warp"
        };
        self.validate_saved_script_command_payload_reference(
            path,
            &warp.source_script,
            warp.command_index,
            command,
            &args,
        )
    }

    pub fn validate_saved_warpcheck_pending_warp_reference(
        &self,
        state: &GameState,
        path: &str,
        warp: &ScriptWarpRequest,
    ) -> Result<()> {
        self.validate_saved_script_command_payload_reference(
            path,
            &warp.source_script,
            warp.command_index,
            "warpcheck",
            &[],
        )?;
        if warp.facing.is_some() {
            anyhow::bail!("saved {path} warpcheck transition must not override player facing");
        }
        let OverworldMemory::Active {
            map_name,
            tile,
            facing,
            mode,
        } = &state.overworld
        else {
            anyhow::bail!("saved {path} warpcheck transition requires an active overworld map");
        };
        let mut session = self
            .overworld_session_for_traversal(map_name, *tile, 0, mode.traversal_state())
            .with_context(|| format!("rebuild saved {path} warpcheck source session"))?;
        session.player.facing = *facing;
        session.player.mode = *mode;
        self.apply_saved_overworld_overrides(&mut session, state)?;
        let trigger = session
            .check_warp_checked()
            .map_err(|error| anyhow::anyhow!("validate saved {path} warpcheck: {error:?}"))?
            .with_context(|| {
                format!(
                    "saved {path} warpcheck has no live warp at {map_name} runtime tile ({}, {})",
                    tile.x, tile.y
                )
            })?;
        let destination = if trigger.warp.target_warp_id > 0 {
            self.resolve_warp_transition(&trigger)?.destination
        } else {
            let destination_map = state
                .backup_warp_map_name
                .as_deref()
                .with_context(|| format!("saved {path} dynamic warpcheck has no backup map"))?;
            let destination_warp_id = Self::required_dynamic_backup_warp_index(
                state,
                trigger.warp.index,
                &trigger.map_name,
            )?;
            self.resolve_warp_destination(destination_map, destination_warp_id, &trigger)?
        };
        if warp.target_map != destination.map_name || warp.tile != destination.tile {
            anyhow::bail!(
                "saved {path} warpcheck destination {} ({}, {}) does not match live warp destination {} ({}, {})",
                warp.target_map,
                warp.tile.x,
                warp.tile.y,
                destination.map_name,
                destination.tile.x,
                destination.tile.y
            );
        }
        Ok(())
    }

    pub fn validate_saved_script_return_reference(
        &self,
        path: &str,
        script_label: &str,
        next_command_index: usize,
    ) -> Result<()> {
        validate_saved_compiled_script_return_reference(
            self.saved_compiled_script_body(path, script_label)?,
            path,
            script_label,
            next_command_index,
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_map_reference<'a>(
        &'a self,
        path: &str,
        map_name: &str,
    ) -> Result<&'a MapModule> {
        core_validate_saved_map_reference(path, map_name, self.saved_map_id(map_name))
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        let module = self
            .maps
            .get(map_name)
            .with_context(|| format!("saved {path} map {map_name} validated but is missing"))?;
        Ok(module)
    }

    pub fn validate_saved_warp_reference(
        &self,
        path: &str,
        map_name: &str,
        warp_index: u16,
    ) -> Result<()> {
        core_validate_saved_warp_reference(
            path,
            map_name,
            warp_index,
            self.saved_map_id(map_name),
            |warp_index| self.saved_warp_exists(map_name, warp_index),
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_audio_runtime_event_command(
        &self,
        path: &str,
        event: &crystal_core::state::ScriptAudioRuntimeEvent,
    ) -> Result<()> {
        let Some(args) = saved_audio_runtime_event_command_args(path, event)
            .map_err(|error| anyhow::anyhow!("{error}"))?
        else {
            if event.command == "special" {
                return self.validate_saved_special_routine_reference(path, &event.source_script);
            }
            return self.validate_saved_script_command_name_reference(
                path,
                &event.source_script,
                event.command_index,
                &event.command,
            );
        };
        self.validate_saved_script_command_payload_reference(
            path,
            &event.source_script,
            event.command_index,
            &event.command,
            &args,
        )
    }

    pub fn validate_saved_graphics_runtime_event(
        &self,
        path: &str,
        event: &crystal_core::state::ScriptGraphicsRuntimeEvent,
    ) -> Result<()> {
        self.validate_saved_special_routine_reference(path, &event.source_script)?;
        validate_saved_graphics_runtime_event_shape(path, event)
            .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_screen_fade(
        &self,
        path: &str,
        fade: &crystal_core::state::ScriptScreenFade,
    ) -> Result<()> {
        self.validate_saved_special_routine_reference(path, &fade.source_script)?;
        validate_saved_pending_screen_fade_shape(path, fade)
            .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_money_runtime_event(
        &self,
        path: &str,
        event: &crystal_core::state::ScriptMoneyRuntimeEvent,
    ) -> Result<()> {
        self.validate_saved_special_routine_reference(path, &event.source_script)?;
        validate_saved_money_runtime_event_shape(path, event)
            .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_map_runtime_event_command(
        &self,
        path: &str,
        event: &crystal_core::state::ScriptMapRuntimeEvent,
    ) -> Result<()> {
        if self.saved_special_routine_exists(&event.source_script) {
            return self.validate_saved_special_routine_reference(path, &event.source_script);
        }
        let Some(args) = self.saved_map_runtime_event_command_args(path, event)? else {
            return self.validate_saved_script_command_name_reference(
                path,
                &event.source_script,
                event.command_index,
                &event.command,
            );
        };
        self.validate_saved_script_command_payload_reference(
            path,
            &event.source_script,
            event.command_index,
            &event.command,
            &args,
        )
    }

    fn saved_map_runtime_event_command_args(
        &self,
        path: &str,
        event: &crystal_core::state::ScriptMapRuntimeEvent,
    ) -> Result<Option<Vec<String>>> {
        match event.command.as_str() {
            "warp" => {
                let Some(mut args) = saved_map_runtime_event_command_args(path, event)
                    .map_err(|error| anyhow::anyhow!("{error}"))?
                else {
                    return Ok(None);
                };
                if let Some(target_map) = event.target_map.as_deref() {
                    args[0] = self.map_constant(target_map)?.to_string();
                }
                Ok(Some(args))
            }
            "warpfacing" => {
                let Some(mut args) = saved_map_runtime_event_command_args(path, event)
                    .map_err(|error| anyhow::anyhow!("{error}"))?
                else {
                    return Ok(None);
                };
                let target_map = event.target_map.as_deref().with_context(|| {
                    format!(
                        "saved {path} {}:{} warpfacing is missing target map",
                        event.source_script, event.command_index
                    )
                })?;
                args[0] = self.map_constant(target_map)?.to_string();
                Ok(Some(args))
            }
            _ => saved_map_runtime_event_command_args(path, event)
                .map_err(|error| anyhow::anyhow!("{error}")),
        }
    }

    pub fn validate_saved_text_runtime_event_command(
        &self,
        path: &str,
        event: &crystal_core::state::ScriptTextRuntimeEvent,
    ) -> Result<()> {
        let Some(args) = saved_text_runtime_event_command_args(path, event)
            .map_err(|error| anyhow::anyhow!("{error}"))?
        else {
            return self.validate_saved_script_command_name_reference(
                path,
                &event.source_script,
                event.command_index,
                &event.command,
            );
        };
        self.validate_saved_script_command_payload_reference(
            path,
            &event.source_script,
            event.command_index,
            &event.command,
            &args,
        )
    }

    pub fn validate_saved_pending_text_wait_command(
        &self,
        runtime: &crystal_core::state::ScriptRuntimeMemory,
        wait: &crystal_core::state::ScriptTextWait,
    ) -> Result<()> {
        let path = "script_runtime.pending_text_wait.source_script";
        let Some(args) =
            saved_pending_text_wait_command_args(path, wait, runtime.pending_text_label.as_deref())
                .map_err(|error| anyhow::anyhow!("{error}"))?
        else {
            return self.validate_saved_script_command_name_reference(
                path,
                &wait.source_script,
                wait.command_index,
                &wait.command,
            );
        };
        self.validate_saved_script_command_payload_reference(
            path,
            &wait.source_script,
            wait.command_index,
            &wait.command,
            &args,
        )
    }

    pub fn validate_saved_script_end_command(
        &self,
        end: &crystal_core::state::ScriptEndState,
    ) -> Result<()> {
        let expected_command =
            saved_script_end_command(end).map_err(|error| anyhow::anyhow!("{error}"))?;
        self.validate_saved_script_command_payload_reference(
            "script_runtime.script_ended.source_script",
            &end.source_script,
            end.command_index,
            expected_command,
            &[],
        )
    }

    pub fn validate_saved_control_runtime_event_command(
        &self,
        path: &str,
        event: &crystal_core::state::ScriptControlRuntimeEvent,
    ) -> Result<()> {
        validate_saved_control_runtime_event_shape(path, event)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        self.validate_saved_script_command_reference(
            path,
            &event.source_script,
            event.command_index,
        )
    }

    pub fn validate_saved_last_talked_object_reference(
        &self,
        state: &GameState,
        object_id: &str,
    ) -> Result<()> {
        core_validate_saved_last_talked_object_reference(
            state,
            object_id,
            |map_name| self.saved_map_exists(map_name),
            |map_name, object_id| self.map_declares_object(map_name, object_id),
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_map_object_reference(
        &self,
        map_name: &str,
        path: &str,
        object_id: &str,
    ) -> Result<()> {
        core_validate_saved_map_object_reference(path, map_name, object_id, |object_id| {
            self.map_declares_object(map_name, object_id)
        })
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_pokemon_party_references(
        &self,
        path: &str,
        party: &[Pokemon],
    ) -> Result<()> {
        core_validate_saved_pokemon_party_references(path, party, |path, pokemon| {
            self.validate_saved_pokemon_reference(path, pokemon)
                .map_err(|error| error.to_string())
        })
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_trainer_battle_origin_references(
        &self,
        trainer: &Trainer,
        battle_type: &str,
        trainer_class: &str,
        event_flag: &str,
        seen_text: &str,
        win_text: &str,
        loss_text: &str,
        callback: &str,
        source_script: &str,
    ) -> Result<()> {
        if let Some(request) = self.saved_trainer_battle_request(source_script, &trainer.trainer_id)
        {
            validate_saved_trainer_battle_request_fields(
                SavedTrainerBattleFields {
                    battle_type,
                    trainer_class,
                    event_flag,
                    seen_text,
                    win_text,
                    loss_text,
                    callback,
                },
                SavedTrainerBattleFields {
                    battle_type: &request.battle_type,
                    trainer_class: &request.trainer_class,
                    event_flag: &request.event_flag,
                    seen_text: &request.seen_text,
                    win_text: &request.win_text,
                    loss_text: &request.loss_text,
                    callback: &request.callback,
                },
                source_script,
            )
            .map_err(|error| anyhow::anyhow!("{error}"))?;
            self.validate_saved_optional_text_reference("battle.trainer.seen_text", seen_text)?;
            self.validate_saved_optional_text_reference("battle.trainer.win_text", win_text)?;
            self.validate_saved_optional_text_reference("battle.trainer.loss_text", loss_text)?;
            return Ok(());
        }

        if self.saved_special_routine_exists(source_script) {
            core_validate_saved_trainer_battle_request_field(
                "event_flag",
                event_flag,
                "",
                source_script,
            )
            .map_err(|error| anyhow::anyhow!("{error}"))?;
            core_validate_saved_trainer_battle_request_field(
                "seen_text",
                seen_text,
                "",
                source_script,
            )
            .map_err(|error| anyhow::anyhow!("{error}"))?;
            core_validate_saved_trainer_battle_request_field(
                "win_text",
                win_text,
                &trainer.win_quote,
                source_script,
            )
            .map_err(|error| anyhow::anyhow!("{error}"))?;
            core_validate_saved_trainer_battle_request_field(
                "loss_text",
                loss_text,
                &trainer.lose_quote,
                source_script,
            )
            .map_err(|error| anyhow::anyhow!("{error}"))?;
            core_validate_saved_trainer_battle_request_field(
                "callback",
                callback,
                "",
                source_script,
            )
            .map_err(|error| anyhow::anyhow!("{error}"))?;
            return Ok(());
        }

        validate_saved_trainer_battle_source_reference(source_script, |_| false)
            .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_static_wild_battle_origin_references(
        &self,
        battle_type: &str,
        species: &str,
        level: u8,
        origin_map_name: &str,
        source_script: &str,
        startbattle_command_index: usize,
        resume_command_index: usize,
    ) -> Result<()> {
        core_validate_saved_static_wild_battle_origin_reference(
            battle_type,
            species,
            level,
            origin_map_name,
            source_script,
            startbattle_command_index,
            resume_command_index,
            |map_name, source_script, start, resume, battle_type, species, level| {
                self.saved_static_wild_battle_origin_exists(
                    map_name,
                    source_script,
                    start,
                    resume,
                    battle_type,
                    species,
                    level,
                )
            },
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_wild_battle_origin_references(
        &self,
        battle_type: &str,
        map_name: &str,
        enemy_pokemon: &Pokemon,
    ) -> Result<()> {
        core_validate_saved_wild_battle_origin_reference(
            battle_type,
            map_name,
            enemy_pokemon,
            |map_name, species, level| self.saved_wild_encounter_exists(map_name, species, level),
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn validate_saved_roaming_battle_origin_references(
        &self,
        map_name: &str,
        roaming_slot: u8,
        roaming: &RoamingPokemonState,
        enemy_pokemon: &Pokemon,
    ) -> Result<()> {
        let metadata = self.runtime_map_metadata_for_name(map_name)?;
        anyhow::ensure!(
            u16::from(roaming.map_group) == metadata.group_id
                && u16::from(roaming.map_number) == metadata.map_id,
            "saved roaming battle slot {roaming_slot} location {}/{} does not match battle map {map_name} location {}/{}",
            roaming.map_group,
            roaming.map_number,
            metadata.group_id,
            metadata.map_id
        );
        let species = roaming
            .species
            .as_deref()
            .with_context(|| format!("saved roaming battle slot {roaming_slot} is inactive"))?;
        anyhow::ensure!(
            species == enemy_pokemon.species.id && roaming.level == enemy_pokemon.level,
            "saved roaming battle slot {roaming_slot} identity {species} level {} does not match enemy {} level {}",
            roaming.level,
            enemy_pokemon.species.id,
            enemy_pokemon.level
        );
        anyhow::ensure!(
            self.roaming_pokemon
                .init_write(usize::from(roaming_slot))
                .is_some_and(|write| write.species == species && write.level == roaming.level),
            "saved roaming battle slot {roaming_slot} identity {species} level {} does not match compiled init slot",
            roaming.level
        );
        Ok(())
    }

    pub fn validate_saved_trainer_enemy_party(
        &self,
        trainer: &Trainer,
        enemy_party: &[Pokemon],
        enemy_pokemon: &Pokemon,
    ) -> Result<()> {
        let expected_party = materialize_trainer_party(
            trainer,
            &self.pokemon,
            &self.learnsets,
            &self.moves,
            &self.growth_rates,
        )
        .map_err(|error| {
            anyhow::anyhow!(
                "compiled trainer {} party is invalid: {error}",
                trainer.trainer_id
            )
        })?;
        validate_saved_trainer_enemy_party_identity(
            &trainer.trainer_id,
            enemy_party,
            enemy_pokemon,
            &expected_party,
        )
        .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn saved_map_exists(&self, map_name: &str) -> bool {
        self.maps.contains_key(map_name)
    }

    pub fn saved_map_dimensions(&self, map_name: &str) -> Option<(u16, u16)> {
        self.maps
            .get(map_name)
            .map(|module| (module.attributes.width, module.attributes.height))
    }

    pub fn saved_map_tile_bounds(&self, map_name: &str) -> Option<(u16, u16)> {
        let width_multiplier = u16::try_from(METATILE_WIDTH).ok()?;
        self.saved_map_dimensions(map_name)
            .and_then(|(width, height)| {
                Some((
                    width.checked_mul(width_multiplier)?,
                    height.checked_mul(width_multiplier)?,
                ))
            })
    }

    pub fn saved_map_block_context(&self, map_name: &str) -> Option<(u16, u16, String)> {
        self.maps.get(map_name).map(|module| {
            (
                module.attributes.width,
                module.attributes.height,
                module.attributes.tileset_name.clone(),
            )
        })
    }

    pub fn saved_warp_exists(&self, map_name: &str, warp_index: u16) -> bool {
        self.maps.get(map_name).is_some_and(|module| {
            module
                .events
                .warps
                .iter()
                .any(|warp| warp.index == warp_index)
        })
    }

    pub fn saved_elevator_pending_warp_exists(&self, warp: &ScriptWarpRequest) -> bool {
        self.maps.values().any(|module| {
            module.script_elevators.values().any(|elevator| {
                elevator.source_script == warp.source_script.as_str()
                    && elevator.elevator_command_index == warp.command_index
                    && elevator.floors.iter().any(|floor| {
                        floor.target_map == warp.target_map.as_str()
                            && self.maps.get(&floor.target_map).is_some_and(|target| {
                                target.events.warps.iter().any(|target_warp| {
                                    target_warp.index == floor.warp
                                        && checked_runtime_map_event_tile(
                                            target_warp.x,
                                            target_warp.y,
                                        )
                                        .is_some_and(|tile| tile == warp.tile)
                                })
                            })
                    })
            })
        })
    }

    pub fn saved_scene_index(&self, map_name: &str, scene_name: &str) -> Option<usize> {
        self.maps.get(map_name).and_then(|module| {
            module
                .scenes
                .scenes
                .iter()
                .enumerate()
                .find(|(_, scene)| scene.scene_id == scene_name)
                .map(|(index, _)| index)
        })
    }

    pub fn compiled_script_body(&self, script_label: &str) -> Option<&serde_json::Value> {
        self.maps
            .values()
            .find_map(|module| module.scripts.get(script_label))
            .or_else(|| {
                self.global_scripts
                    .as_ref()
                    .and_then(|module| module.scripts.get(script_label))
            })
    }

    pub fn materialize_global_scripts(&mut self) -> Result<()> {
        let mut definitions = BTreeMap::new();
        let mut phone_scripts = BTreeMap::new();
        for payload in &self.phone_scripts {
            let object = payload
                .as_object()
                .context("compiled phone script payload must be an object")?;
            for (label, body) in object {
                if definitions.insert(label.clone(), body.clone()).is_some() {
                    anyhow::bail!("duplicate global phone script label {label}");
                }
                if let Some(body) = canonical_global_phone_script_body(label, body)? {
                    phone_scripts.insert(label.clone(), body);
                }
            }
        }

        let mut standard_catalogs = self.story_events.iter().filter_map(|payload| {
            payload
                .as_object()
                .and_then(|payload| payload.get("StandardScripts"))
                .and_then(Value::as_object)
        });
        let standard_catalog = standard_catalogs.next().cloned();
        if standard_catalogs.next().is_some() {
            anyhow::bail!("compiled game pack contains duplicate StandardScripts catalogs");
        }
        let mut overworld_catalogs = self.story_events.iter().filter_map(|payload| {
            payload
                .as_object()
                .and_then(|payload| payload.get("OverworldEvents"))
                .and_then(Value::as_object)
        });
        let overworld_catalog = overworld_catalogs.next().cloned();
        if overworld_catalogs.next().is_some() {
            anyhow::bail!("compiled game pack contains duplicate OverworldEvents catalogs");
        }
        let mut standard_pointer_labels = Vec::new();
        let mut standard_global_roots = Vec::new();
        let mut standard_definitions = BTreeMap::new();
        if let Some(standard_catalog) = standard_catalog {
            let pointer_table = standard_catalog
                .get("StdScripts")
                .and_then(Value::as_array)
                .context(
                    "compiled StandardScripts catalog is missing the StdScripts pointer table",
                )?;
            standard_pointer_labels.reserve(pointer_table.len());
            for (index, entry) in pointer_table.iter().enumerate() {
                let label = entry
                    .get("args")
                    .and_then(Value::as_array)
                    .and_then(|args| (args.len() == 1).then(|| args[0].as_str()).flatten())
                    .with_context(|| {
                        format!("compiled StdScripts pointer {index} has no exact script label")
                    })?;
                standard_pointer_labels.push(label.to_string());
            }
            let global_roots = standard_catalog
                .get("GlobalScriptRoots")
                .and_then(Value::as_array)
                .context(
                    "compiled StandardScripts catalog is missing the GlobalScriptRoots list",
                )?;
            standard_global_roots.reserve(global_roots.len());
            for (index, entry) in global_roots.iter().enumerate() {
                let label = entry.as_str().filter(|label| !label.is_empty()).with_context(|| {
                    format!(
                        "compiled StandardScripts global root {index} is not an exact script label"
                    )
                })?;
                if standard_global_roots
                    .iter()
                    .any(|existing| existing == label)
                {
                    anyhow::bail!("compiled StandardScripts global roots repeat {label}");
                }
                standard_global_roots.push(label.to_string());
            }
            for (label, body) in standard_catalog {
                if label == "StdScripts" || label == "GlobalScriptRoots" {
                    continue;
                }
                if definitions.insert(label.clone(), body.clone()).is_some() {
                    anyhow::bail!("duplicate global script label {label}");
                }
                standard_definitions.insert(label, body);
            }
        }

        let mut overworld_roots = Vec::new();
        if let Some(overworld_catalog) = overworld_catalog {
            let overworld_catalog = overworld_catalog.into_iter().collect::<BTreeMap<_, _>>();
            let pointer_table = overworld_catalog
                .get("PlayerEventScriptPointers")
                .and_then(Value::as_array)
                .context("compiled OverworldEvents catalog is missing PlayerEventScriptPointers")?;
            overworld_roots = exact_player_event_pointer_labels(pointer_table)?;
            for (label, body) in overworld_catalog {
                if definitions.insert(label.clone(), body).is_some() {
                    anyhow::bail!("duplicate global script label {label}");
                }
            }
        }

        let mut scripts = runtime_module_script_subset(
            &standard_definitions,
            standard_pointer_labels
                .iter()
                .chain(&standard_global_roots)
                .map(String::as_str),
            false,
        );
        for (label, body) in decoration_description_scripts() {
            if definitions.insert(label.clone(), body.clone()).is_some() {
                anyhow::bail!("duplicate decoration description script label {label}");
            }
            if scripts.insert(label.clone(), body).is_some() {
                anyhow::bail!("duplicate executable decoration description script label {label}");
            }
        }
        let interpreted_overworld_roots = overworld_roots
            .iter()
            .map(|label| player_event_execution_path(label, &definitions).map(|path| (label, path)))
            .collect::<Result<Vec<_>>>()?;
        let overworld_scripts = runtime_module_script_subset(
            &definitions,
            interpreted_overworld_roots
                .iter()
                .filter(|(_, path)| *path == PlayerEventExecutionPath::CommonInterpreter)
                .map(|(label, _)| label.as_str()),
            false,
        );
        for (label, body) in phone_scripts {
            if scripts.insert(label.clone(), body).is_some() {
                anyhow::bail!("duplicate executable global script label {label}");
            }
        }
        for (label, body) in overworld_scripts {
            if scripts.insert(label.clone(), body).is_some() {
                anyhow::bail!("duplicate executable global script label {label}");
            }
        }
        if scripts.is_empty() {
            self.global_scripts = None;
            return Ok(());
        }
        for label in &standard_pointer_labels {
            let actual = scripts.get(label).with_context(|| {
                format!("compiled StdScripts pointer {label} is absent from global scripts")
            })?;
            let expected = definitions
                .get(label)
                .expect("standard pointer was copied into definitions");
            if actual != expected {
                anyhow::bail!(
                    "compiled StdScripts pointer {label} changed while materializing global scripts"
                );
            }
        }
        for label in &standard_global_roots {
            let actual = scripts.get(label).with_context(|| {
                format!(
                    "compiled StandardScripts global root {label} is absent from global scripts"
                )
            })?;
            let expected = definitions
                .get(label)
                .expect("standard global root was copied into definitions");
            if actual != expected {
                anyhow::bail!(
                    "compiled StandardScripts global root {label} changed while materializing global scripts"
                );
            }
        }

        let map_name = "GlobalScripts";
        let map_name_by_constant = self.map_name_by_constant_from_attributes()?;
        let script_item_grants = parse_script_item_grants(map_name, &scripts)?;
        let (script_item_checks, script_item_takes) =
            parse_script_item_accesses(map_name, &scripts)?;
        let script_object_commands = parse_script_object_commands(map_name, &scripts)?;
        let script_movements = parse_script_movements(map_name, &scripts, &script_object_commands)?;
        let script_menu_definitions = parse_script_menu_definitions(map_name, &scripts)?;
        let script_vertical_menus =
            parse_script_vertical_menus(map_name, &scripts, &script_menu_definitions)?;
        self.global_scripts = Some(GlobalScriptModule {
            script_item_grants,
            script_item_checks,
            script_item_takes,
            script_economy_commands: parse_script_economy_commands(map_name, &scripts)?,
            script_flag_commands: parse_script_flag_commands(map_name, &scripts)?,
            script_scene_commands: parse_script_scene_commands(map_name, &scripts)?,
            script_audio_commands: parse_script_audio_commands(map_name, &scripts)?,
            script_block_changes: parse_script_block_changes(map_name, &scripts)?,
            script_object_commands,
            script_movements,
            script_map_commands: parse_script_map_commands(
                map_name,
                &scripts,
                &map_name_by_constant,
            )?,
            script_text_commands: parse_script_text_commands(map_name, &scripts)?,
            script_text_bodies: parse_script_text_bodies(map_name, &scripts)?,
            script_menu_definitions,
            script_vertical_menus,
            script_elevators: parse_script_elevators(map_name, &scripts, &map_name_by_constant)?,
            script_variable_commands: parse_script_variable_commands(map_name, &scripts)?,
            script_control_commands: parse_script_control_commands(map_name, &scripts)?,
            script_shop_commands: parse_script_shop_commands(map_name, &scripts)?,
            script_phone_commands: parse_script_phone_commands(map_name, &scripts)?,
            script_runtime_commands: parse_script_runtime_commands(map_name, &scripts)?,
            script_swarm_commands: parse_script_swarm_commands(map_name, &scripts)?,
            scripts,
            definitions,
        });
        Ok(())
    }

    pub fn compiled_standard_script_body(&self, script_label: &str) -> Result<&[Value]> {
        validate_compiled_standard_script_catalog(self)?;
        let catalog = compiled_standard_script_catalog(self)?;
        let pointer_table = catalog
            .get("StdScripts")
            .and_then(Value::as_array)
            .context("compiled StandardScripts catalog is missing the StdScripts pointer table")?;
        let declared = pointer_table.iter().any(|entry| {
            entry.get("command").and_then(Value::as_str) == Some("add_stdscript")
                && entry
                    .get("args")
                    .and_then(Value::as_array)
                    .is_some_and(|args| args.len() == 1 && args[0].as_str() == Some(script_label))
        });
        if !declared {
            anyhow::bail!(
                "compiled standard script {script_label} is not declared by the StdScripts pointer table"
            );
        }
        catalog
            .get(script_label)
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .with_context(|| {
                format!("compiled StdScripts pointer {script_label} has no command body")
            })
    }

    pub fn map_declares_object(&self, map_name: &str, object_id: &str) -> bool {
        self.maps.get(map_name).is_some_and(|module| {
            module
                .objects
                .iter()
                .any(|object| object.object_identifier.as_deref() == Some(object_id))
        })
    }

    pub fn map_declares_event_object(&self, map_name: &str, object_id: &str) -> bool {
        self.maps.get(map_name).is_some_and(|module| {
            module
                .objects
                .iter()
                .any(|object| object.object_identifier.as_deref() == Some(object_id))
        })
    }

    pub fn tileset_declares_metatile(&self, tileset_name: &str, block_id: u16) -> bool {
        self.tilesets
            .get(tileset_name)
            .is_some_and(|tileset| tileset_declares_metatile(tileset, block_id))
    }

    pub fn saved_tileset_exists(&self, tileset_name: &str) -> bool {
        self.tilesets.contains_key(tileset_name)
    }

}
