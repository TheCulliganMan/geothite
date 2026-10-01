impl GameDataSet {
    pub fn fishing_rod_item(&self, item_id: &str) -> Result<(&Item, &str)> {
        let item = self.item(item_id)?;
        let rod = fishing_rod_for_item_id(&self.fishing, item_id).with_context(|| {
            format!(
                "field fishing rod item {item_id} is not declared by exact fishing rod item rules"
            )
        })?;
        Ok((item, rod))
    }

    pub fn field_fishing_rod(&self, state: &GameState, item_id: &str) -> Result<String> {
        let (item, rod) = self.fishing_rod_item(item_id)?;
        if !item.field_usable {
            anyhow::bail!("field fishing rod item {item_id} is not usable in the field");
        }
        if !state.bag.has_item(item) {
            anyhow::bail!("field fishing rod item {item_id} is not in the bag");
        }
        Ok(rod.to_string())
    }

    pub fn cast_fishing_rod_in_session_with_divider<S>(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        rod: &str,
        divider: &mut S,
    ) -> Result<FishingCastOutcome>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        self.require_no_active_battle(state, "fishing rod")?;
        if matches!(
            session.player.mode,
            MovementMode::Surf | MovementMode::SurfPika
        ) {
            anyhow::bail!(crystal_core::world::fishing::FishingError::CannotFishWhileSurfing);
        }
        let target = checked_move_by_stride(
            session.player.tile,
            session.player.facing,
            StepOptions::default().stride_tiles,
        )
        .ok_or_else(|| {
            anyhow::Error::new(crystal_core::world::fishing::FishingError::FacingTileOutOfBounds)
        })?;
        let sample = sample_collision(&session.map, &session.tileset, target).ok_or_else(|| {
            anyhow::Error::new(crystal_core::world::fishing::FishingError::FacingTileOutOfBounds)
        })?;
        if describe_collision(sample.permission).terrain != Terrain::Water {
            anyhow::bail!(crystal_core::world::fishing::FishingError::FacingTileIsNotWater);
        }

        let mut next_state = state.clone();
        let map_name = session.map.name.clone();
        let group = self.map_fishing_group(&map_name)?;
        let time_of_day = next_state.time.time_of_day;
        let mut rng = CrystalRandom::new(next_state.random_state, divider);
        let rolled = core_do_fishing_exact(
            &mut next_state,
            &self.fishing,
            group,
            rod,
            time_of_day,
            &mut rng,
        )
        .map_err(|error| anyhow::anyhow!("cast fishing rod {rod} on {map_name}: {error}"))?;
        let mut fishing_session = rolled.session;
        let bite_frame = fishing_session
            .start_frame
            .saturating_add(fishing_session.cast_frames)
            .saturating_add(fishing_session.bite_delay_frames);
        let bite = fishing_bite(&mut next_state, &mut fishing_session, bite_frame);
        let wild_battle = if bite == Some(true) {
            fishing_battle_trigger(&mut next_state);
            if let Some(encounter) = fishing_session.outcome.encounter.clone() {
                Some(
                    self.start_fishing_battle_with_rng(
                        &mut next_state,
                        &map_name,
                        session.player.tile,
                        encounter,
                        time_of_day,
                        rolled.bite_roll,
                        rolled
                            .slot_roll
                            .context("fishing bite is missing its slot roll")?,
                        &mut rng,
                    )?,
                )
            } else {
                None
            }
        } else {
            None
        };
        next_state.random_state = rng.state();
        *state = next_state;
        Ok(FishingCastOutcome {
            session: fishing_session,
            bite,
            wild_battle,
        })
    }

    pub fn use_bag_fishing_rod_in_field_with_divider<S>(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        item_id: &str,
        divider: &mut S,
    ) -> Result<FishingRodItemUseOutcome>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        self.require_no_active_battle(state, "field fishing rod item")?;
        let rod = self.field_fishing_rod(state, item_id)?;
        let mut next_state = state.clone();
        let cast =
            self.cast_fishing_rod_in_session_with_divider(&mut next_state, session, &rod, divider)?;
        let cast_state_checksum =
            game_state_checksum(&next_state).context("checksum field fishing rod cast")?;
        let item_use = self.use_bag_item(&mut next_state, item_id, ItemUseContext::Field)?;
        *state = next_state;
        Ok(FishingRodItemUseOutcome {
            item_use,
            rod,
            cast,
            cast_state_checksum,
        })
    }

    pub fn saved_fishing_rod_exists(&self, rod: &str) -> bool {
        self.fishing
            .groups
            .values()
            .any(|group| group.rod_tables.contains_key(rod))
    }

    pub fn saved_fishing_daily_flag_bit_exists(&self, bit: u32) -> bool {
        self.fishing
            .swarm_rules
            .values()
            .any(|rule| u32::from(rule.daily_flag_bit) == bit)
    }

    pub fn saved_fishing_swarm_flag_exists(&self, swarm_flag: u8) -> bool {
        self.fishing
            .swarm_rules
            .values()
            .any(|rule| rule.swarm == swarm_flag)
    }

    pub fn require_special_routine(&self, routine: &str) -> Result<()> {
        if self.special_routines.contains_key(routine) {
            Ok(())
        } else {
            anyhow::bail!("compiled game pack missing exact special routine {routine}")
        }
    }

    pub fn saved_special_routine_exists(&self, routine: &str) -> bool {
        self.special_routines.contains_key(routine)
            || matches!(routine, "StartBugContestTimer" | "CheckBugContestTimer")
    }

    pub fn saved_sprite_exists(&self, sprite_id: &str) -> bool {
        self.sprite_palette_defaults.contains_key(sprite_id)
    }

    pub fn saved_variable_sprite_exists(&self, sprite_id: &str) -> bool {
        self.initialize_events
            .variable_sprites
            .contains_key(sprite_id)
    }

    pub fn saved_menu_exists(&self, menu: &str) -> bool {
        self.special_routines.contains_key(menu)
            || self
                .global_scripts
                .as_ref()
                .is_some_and(|module| module.script_menu_definitions.contains_key(menu))
            || self
                .maps
                .values()
                .any(|module| module.script_menu_definitions.contains_key(menu))
    }

    pub fn saved_special_phone_call_exists(&self, call_id: &str) -> bool {
        self.special_phone_calls.contains_key(call_id)
    }

    pub fn saved_npc_trade_exists(&self, trade_id: &str) -> bool {
        self.npc_trades.contains_key(trade_id)
    }

    pub fn map_name_for_constant(&self, map_constant: &str) -> Result<String> {
        self.maps
            .iter()
            .find(|(_, module)| module.attributes.map_constant.as_deref() == Some(map_constant))
            .map(|(map_name, _)| map_name.clone())
            .with_context(|| format!("compiled game pack missing map constant {map_constant}"))
    }

    pub fn saved_map_constant(&self, map_constant: &str) -> Option<String> {
        self.runtime_map_metadata
            .get(map_constant)
            .map(|metadata| metadata.constant.clone())
    }

    pub fn runtime_map_group_number_exists(&self, map_group: u16, map_number: u16) -> bool {
        self.runtime_map_metadata
            .values()
            .any(|metadata| metadata.group_id == map_group && metadata.map_id == map_number)
    }

    pub fn saved_pending_special_battle_type_exists(&self, battle_type: &str) -> bool {
        let module_declares_battle_type = |module: &MapModule| {
            module
                .scripted_trainer_battles
                .iter()
                .any(|battle| battle.request.battle_type == battle_type)
                || module
                    .scripted_wild_battles
                    .iter()
                    .any(|battle| battle.request.battle_type == battle_type)
                || module.script_variable_commands.iter().any(|command| {
                    command.command == "loadvar"
                        && command.target.as_deref() == Some("VAR_BATTLETYPE")
                        && command.value_tokens.first().map(String::as_str) == Some(battle_type)
                })
        };
        self.maps.values().any(module_declares_battle_type)
            || self.global_scripts.as_ref().is_some_and(|module| {
                module.script_variable_commands.iter().any(|command| {
                    command.command == "loadvar"
                        && command.target.as_deref() == Some("VAR_BATTLETYPE")
                        && command.value_tokens.first().map(String::as_str) == Some(battle_type)
                })
            })
            || saved_special_battle_type_builtin_routine(battle_type)
                .is_some_and(|routine| self.saved_special_routine_exists(routine))
    }

    pub fn saved_static_wild_battle_origin_exists(
        &self,
        map_name: &str,
        source_script: &str,
        startbattle_command_index: usize,
        resume_command_index: usize,
        battle_type: &str,
        species: &str,
        level: u8,
    ) -> bool {
        let fixed = self.maps.get(map_name).is_some_and(|module| {
            module.scripted_wild_battles.iter().any(|battle| {
                let request = &battle.request;
                let commands = module.scripts.get(source_script).and_then(Value::as_array);
                let exact_load = commands
                    .and_then(|commands| commands.get(battle.loadwildmon_command_index))
                    .is_some_and(|entry| {
                        entry.get("command").and_then(Value::as_str) == Some("loadwildmon")
                            && entry
                                .get("args")
                                .and_then(Value::as_array)
                                .is_some_and(|args| {
                                    args.len() == 2
                                        && args[0].as_str() == Some(species)
                                        && args[1]
                                            .as_str()
                                            .and_then(|value| value.parse::<u8>().ok())
                                            == Some(request.level)
                                })
                    });
                let exact_start = commands
                    .and_then(|commands| commands.get(startbattle_command_index))
                    .is_some_and(|entry| {
                        let command = entry.get("command").and_then(Value::as_str);
                        let args = entry.get("args").and_then(Value::as_array);
                        (command == Some("startbattle") && args.is_some_and(Vec::is_empty))
                            || (battle_type == "BATTLETYPE_TUTORIAL"
                                && command == Some("catchtutorial")
                                && args.is_some_and(|args| {
                                    args.len() == 1 && args[0].as_str() == Some(battle_type)
                                }))
                    });
                let exact_resume = commands
                    .and_then(|commands| commands.get(resume_command_index))
                    .and_then(|entry| entry.get("command"))
                    .and_then(Value::as_str)
                    .is_some();
                battle.source_script == source_script
                    && battle.startbattle_command_index == startbattle_command_index
                    && startbattle_command_index.checked_add(1) == Some(resume_command_index)
                    && battle.loadwildmon_command_index < startbattle_command_index
                    && exact_load
                    && exact_start
                    && exact_resume
                    && request.battle_type == battle_type
                    && request.species == species
                    && request.level == level
                    && request.source_script == source_script
            })
        });
        if fixed {
            return true;
        }
        let command =
            RuntimeScriptCommandRef::new(map_name, source_script, startbattle_command_index);
        if resume_command_index == 11
            && startbattle_command_index == 10
            && self
                .is_exact_sweet_scent_dynamic_start_command(&command)
                .unwrap_or(false)
        {
            return match battle_type {
                "BATTLETYPE_NORMAL" => {
                    self.wild_encounters
                        .get(map_name)
                        .is_some_and(|encounters| {
                            encounters
                                .grass
                                .iter()
                                .chain(encounters.water.iter())
                                .flat_map(|table| {
                                    table.morning.iter().chain(&table.day).chain(&table.night)
                                })
                                .any(|entry| entry.species == species && entry.level == level)
                        })
                }
                "BATTLETYPE_ROAMING" => self
                    .roaming_pokemon
                    .init_writes
                    .iter()
                    .any(|write| write.species == species && write.level == level),
                _ => false,
            };
        }
        let encounter_kind = if battle_type == "BATTLETYPE_NORMAL"
            && resume_command_index == 13
            && startbattle_command_index == 12
            && self
                .is_exact_rock_smash_dynamic_start_command(&command)
                .unwrap_or(false)
        {
            FieldEncounterKind::RockSmash
        } else if battle_type == "BATTLETYPE_TREE"
            && resume_command_index == 9
            && startbattle_command_index == 8
            && self
                .is_exact_headbutt_dynamic_start_command(&command)
                .unwrap_or(false)
        {
            FieldEncounterKind::Headbutt
        } else {
            return false;
        };
        self.field_encounters
            .get(map_name)
            .and_then(|encounters| encounters.table(encounter_kind))
            .is_some_and(|table| {
                table
                    .common
                    .iter()
                    .chain(&table.rare)
                    .any(|entry| entry.species == species && entry.level == level)
            })
    }

    pub fn saved_scripted_trainer_battle(
        &self,
        source_script: &str,
        trainer_id: &str,
    ) -> Option<&ScriptedTrainerBattle> {
        self.maps.values().find_map(|module| {
            module.scripted_trainer_battles.iter().find(|battle| {
                battle.source_script == source_script && battle.request.trainer_id == trainer_id
            })
        })
    }

    pub fn saved_trainer_battle_request(
        &self,
        source_script: &str,
        trainer_id: &str,
    ) -> Option<&TrainerBattleRequest> {
        self.maps.values().find_map(|module| {
            module
                .scripted_trainer_battles
                .iter()
                .find(|battle| {
                    battle.source_script == source_script && battle.request.trainer_id == trainer_id
                })
                .map(|battle| &battle.request)
                .or_else(|| {
                    module
                        .trainer_scripts
                        .get(source_script)
                        .filter(|request| request.trainer_id == trainer_id)
                })
        })
    }

    pub fn saved_spawn_identifier(&self, spawn_identifier: u16) -> Option<String> {
        self.runtime_spawn_points
            .get(&spawn_identifier.to_string())
            .map(|spawn| spawn.identifier.to_string())
    }

    pub fn saved_phone_contact_id(&self, contact_id: &str) -> Option<String> {
        self.phone_contacts
            .0
            .get(contact_id)
            .map(|contact| contact.contact_id.clone())
    }

    pub fn saved_trainer(&self, trainer_id: &str) -> Option<&Trainer> {
        self.trainers.get(trainer_id)
    }

    pub fn saved_trainer_id(&self, trainer_id: &str) -> Option<String> {
        self.trainers
            .get(trainer_id)
            .map(|trainer| trainer.trainer_id.clone())
    }

    pub fn runtime_map_metadata_for_name(&self, map_name: &str) -> Result<&RuntimeMapMetadata> {
        self.runtime_map_metadata
            .values()
            .find(|metadata| metadata.name == map_name)
            .with_context(|| {
                format!("compiled game pack missing runtime metadata for map {map_name}")
            })
    }

    pub fn map_environment(&self, map_name: &str) -> Result<&str> {
        Ok(self
            .runtime_map_metadata_for_name(map_name)?
            .environment
            .as_str())
    }

    pub fn apply_dig_warp_memory_for_transition(
        &self,
        state: &mut GameState,
        transition: &WarpTransition,
    ) -> Result<()> {
        let source_environment = self.map_environment(&transition.trigger.map_name)?;
        let destination_environment = self.map_environment(&transition.destination.map_name)?;
        apply_dig_warp_memory_for_transition(
            state,
            transition,
            source_environment,
            destination_environment,
        );
        Ok(())
    }

    pub fn map_constant(&self, map_name: &str) -> Result<&str> {
        Ok(self
            .runtime_map_metadata_for_name(map_name)?
            .constant
            .as_str())
    }

    pub fn wild_encounters_for_map(&self, map_name: &str) -> Option<&WildEncounterData> {
        self.wild_encounters.get(map_name)
    }

    fn active_swarm_wild_encounters_for_map(
        &self,
        state: &GameState,
        map_name: &str,
    ) -> Result<Option<WildEncounterData>> {
        let Some(base) = self.wild_encounters_for_map(map_name) else {
            return Ok(None);
        };
        if base.swarm_overrides.is_empty() {
            return Ok(None);
        }
        let metadata = self.runtime_map_metadata_for_name(map_name)?;
        for (swarm_token, swarm) in &base.swarm_overrides {
            let Some(target) = state.swarms.active.get(swarm_token) else {
                continue;
            };
            if target.map_group != Some(metadata.group_id)
                || target.map_number != Some(metadata.map_id)
            {
                continue;
            }
            if !state
                .flags
                .is_engine_flag_set(&swarm.engine_flag)
                .map_err(|error| {
                    anyhow::anyhow!(
                        "check {} for active {swarm_token} encounter table: {error}",
                        swarm.engine_flag
                    )
                })?
            {
                continue;
            }
            let mut selected = base.clone();
            selected.grass_rates = Some(swarm.grass_rates.clone());
            selected.grass = Some(swarm.grass.clone());
            return Ok(Some(selected));
        }
        Ok(None)
    }

    pub fn require_wild_encounters_for_map(&self, map_name: &str) -> Result<&WildEncounterData> {
        self.wild_encounters
            .get(map_name)
            .with_context(|| format!("compiled game pack missing wild encounters for {map_name}"))
    }

    pub fn require_field_encounters_for_map(&self, map_name: &str) -> Result<&FieldEncounterData> {
        self.field_encounters
            .get(map_name)
            .with_context(|| format!("compiled game pack missing field encounters for {map_name}"))
    }

    #[cfg(any(test, feature = "test-fixtures"))]
    pub fn roll_headbutt_encounter(
        &self,
        map_name: &str,
        target_tile: TilePosition,
        player_id: u16,
        rng: &mut Random,
    ) -> Result<crystal_core::world::encounters::FieldEncounterRoll> {
        self.validate_runtime_map_tile("HEADBUTT encounter", map_name, target_tile)?;
        let encounters = self.require_field_encounters_for_map(map_name)?;
        core_roll_headbutt_encounter(encounters, target_tile.x, target_tile.y, player_id, rng)
            .map_err(|error| anyhow::anyhow!("roll HEADBUTT encounter on {map_name}: {error:?}"))
    }

    fn validate_direct_headbutt_target(
        rule: &FieldMoveMoveRule,
        map: &OverworldMapData,
        tileset: &TilesetCollision,
        target: TilePosition,
    ) -> Result<()> {
        Self::validate_runtime_field_move_tile_alignment(&rule.move_id, target)?;
        let sample = sample_collision(map, tileset, target).ok_or_else(|| {
            anyhow::anyhow!(FieldMoveError::TargetTileOutOfBounds {
                move_id: rule.move_id.clone(),
                map_name: map.name.clone(),
            })
        })?;
        if !rule.target_collisions.contains(&sample.permission) {
            anyhow::bail!(FieldMoveError::UnsupportedCollision {
                move_id: rule.move_id.clone(),
                block_id: sample.metatile_id,
            });
        }
        Ok(())
    }

    fn validate_direct_rock_smash_target(
        rule: &FieldMoveMoveRule,
        overworld: &OverworldSession,
        target: TilePosition,
    ) -> Result<(Option<String>, String)> {
        Self::validate_runtime_field_move_tile_alignment(&rule.move_id, target)?;
        let Some((_, object)) = overworld.visible_object_at_checked(target)? else {
            anyhow::bail!(FieldMoveError::MissingRockSmashTarget {
                move_id: rule.move_id.clone(),
                x: target.x,
                y: target.y,
            });
        };
        if object.spritemovedata != "SPRITEMOVEDATA_SMASHABLE_ROCK" {
            anyhow::bail!(FieldMoveError::TargetNotSmashableRock {
                move_id: rule.move_id.clone(),
                movement: object.spritemovedata.clone(),
            });
        }
        Ok((object.object_identifier.clone(), object.event_flag.clone()))
    }

    fn validate_runtime_field_move_tile_alignment(move_id: &str, tile: TilePosition) -> Result<()> {
        let _ = (move_id, tile);
        Ok(())
    }

    fn checked_runtime_field_move_target(
        move_id: &str,
        tile: TilePosition,
        facing: Direction,
    ) -> Result<TilePosition> {
        Self::validate_runtime_field_move_tile_alignment(move_id, tile)?;
        checked_move_by_stride(tile, facing, StepOptions::default().stride_tiles).ok_or_else(|| {
            anyhow::anyhow!(FieldMoveError::RuntimeTileOverflow {
                move_id: move_id.to_string(),
                x: tile.x,
                y: tile.y,
            })
        })
    }

    pub fn queue_headbutt_script(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        party_index: usize,
        from_menu: bool,
    ) -> Result<()> {
        self.require_no_active_battle(state, "HEADBUTT field move")?;
        anyhow::ensure!(
            self.is_exact_tree_mon_encounter_command(&RuntimeScriptCommandRef::new(
                &overworld.map.name,
                "HeadbuttScript",
                4,
            ))?,
            "compiled pack does not expose the exact HeadbuttFromMenuScript/HeadbuttScript path"
        );
        self.validate_direct_field_move_actor(
            state,
            party_index,
            &self.field_moves.headbutt.move_id,
        )?;
        let target = Self::checked_runtime_field_move_target(
            &self.field_moves.headbutt.move_id,
            overworld.player.tile,
            overworld.player.facing,
        )?;
        Self::validate_direct_headbutt_target(
            &self.field_moves.headbutt,
            &overworld.map,
            &overworld.tileset,
            target,
        )?;
        state
            .script_runtime
            .memory
            .insert("wCurPartyMon".to_string(), party_index.to_string());
        let next_script = if from_menu {
            "HeadbuttFromMenuScript"
        } else {
            "HeadbuttScript"
        };
        commit_interaction_script_dispatch(
            state,
            &mut overworld.last_talked_object_identifier,
            &overworld.map.name,
            next_script,
            None,
        )
        .map_err(|error| anyhow::anyhow!("queue Headbutt script: {error:?}"))?;
        commit_overworld_snapshot(state, &overworld.snapshot(), SpawnMemoryUpdate::Preserve);
        Ok(())
    }

    pub fn queue_rock_smash_from_menu(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        party_index: usize,
    ) -> Result<String> {
        self.require_no_active_battle(state, "ROCK_SMASH field move")?;
        anyhow::ensure!(
            self.is_exact_rock_mon_encounter_command(&RuntimeScriptCommandRef::new(
                &overworld.map.name,
                "RockSmashScript",
                8,
            ))?,
            "compiled pack does not expose the exact RockSmashFromMenuScript/RockSmashScript path"
        );
        self.validate_direct_field_move_actor(state, party_index, "ROCK_SMASH")?;
        let target = Self::checked_runtime_field_move_target(
            "ROCK_SMASH",
            overworld.player.tile,
            overworld.player.facing,
        )?;
        let (object_identifier, _) = Self::validate_direct_rock_smash_target(
            &self.field_moves.rock_smash,
            overworld,
            target,
        )?;
        let object_identifier = object_identifier.with_context(
            || "smashable rock used from the party menu has no exact object identifier",
        )?;
        state
            .script_runtime
            .memory
            .insert("wCurPartyMon".to_string(), party_index.to_string());
        commit_interaction_script_dispatch(
            state,
            &mut overworld.last_talked_object_identifier,
            &overworld.map.name,
            "RockSmashFromMenuScript",
            Some(&object_identifier),
        )
        .map_err(|error| anyhow::anyhow!("queue RockSmashFromMenuScript: {error:?}"))?;
        commit_overworld_snapshot(state, &overworld.snapshot(), SpawnMemoryUpdate::Preserve);
        Ok(object_identifier)
    }

    pub fn queue_sweet_scent_from_menu(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        party_index: usize,
    ) -> Result<()> {
        self.require_no_active_battle(state, "SWEET_SCENT field move")?;
        anyhow::ensure!(
            self.is_exact_sweet_scent_encounter_command(&RuntimeScriptCommandRef::new(
                &overworld.map.name,
                ".SweetScent@SweetScentFromMenu",
                5,
            ))?,
            "compiled pack does not expose the exact SweetScentFromMenu script path"
        );
        self.validate_direct_field_move_actor(state, party_index, "SWEET_SCENT")?;
        state
            .script_runtime
            .memory
            .insert("wCurPartyMon".to_string(), party_index.to_string());
        commit_interaction_script_dispatch(
            state,
            &mut overworld.last_talked_object_identifier,
            &overworld.map.name,
            ".SweetScent@SweetScentFromMenu",
            None,
        )
        .map_err(|error| anyhow::anyhow!("queue SweetScentFromMenu script: {error:?}"))?;
        commit_overworld_snapshot(state, &overworld.snapshot(), SpawnMemoryUpdate::Preserve);
        Ok(())
    }

    pub fn resolve_sweet_scent_encounter<S>(
        &self,
        state: &mut GameState,
        overworld: &OverworldSession,
        command: &RuntimeScriptCommandRef,
        divider: &mut S,
    ) -> Result<SweetScentEncounterOutcome>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        self.require_current_map(&overworld.map.name, &command.map_name)?;
        anyhow::ensure!(
            self.is_exact_sweet_scent_encounter_command(command)?,
            "runtime SweetScentEncounter command is not the exact Sweet Scent typed edge"
        );
        self.validate_runtime_map_tile(
            "SWEET_SCENT encounter",
            &overworld.map.name,
            overworld.player.tile,
        )?;
        let mut rng = CrystalRandom::new(state.random_state, divider);
        let contest_mode = self.bug_contest_encounter_mode(state)?;
        let swarm_encounters =
            self.active_swarm_wild_encounters_for_map(state, &overworld.map.name)?;
        let encounters = swarm_encounters
            .as_ref()
            .or_else(|| self.wild_encounters_for_map(&overworld.map.name));
        if !contest_mode && encounters.is_none() {
            state.random_state = rng.state();
            state
                .script_runtime
                .memory
                .insert("wScriptVar".to_string(), "0".to_string());
            state
                .script_runtime
                .memory
                .insert("wBattleType".to_string(), "0".to_string());
            state.script_runtime.script_value = Some("0".to_string());
            return Ok(SweetScentEncounterOutcome {
                wild_encounter: None,
            });
        }
        let metadata = self.runtime_map_metadata_for_name(&overworld.map.name)?;
        let current_map = (
            u8::try_from(metadata.group_id).context("Sweet Scent map group exceeds byte")?,
            u8::try_from(metadata.map_id).context("Sweet Scent map number exceeds byte")?,
        );
        let land_encounters_on_any_land = can_encounter_on_any_non_ice_land(&metadata.environment);
        let contest_encounters = contest_mode
            .then(|| {
                self.bug_contest_config
                    .as_ref()
                    .context("active Bug Contest is missing required encounter configuration")
                    .map(|config| config.encounters.as_slice())
            })
            .transpose()?;
        self.preflight_exact_wild_encounter_transaction(
            state,
            overworld,
            encounters,
            contest_mode,
            land_encounters_on_any_land,
        )?;
        let wild_encounter = overworld
            .check_sweet_scent_encounter_exact(
                encounters,
                &self.encounter_slot_tables,
                &mut rng,
                EncounterCheckOptions {
                    time: state.time.time_of_day,
                    land_encounters_on_any_land,
                    ..EncounterCheckOptions::default()
                },
                ExactEncounterContext {
                    roaming_pokemon: &state.roaming_pokemon,
                    current_map,
                    bug_contest_encounters: contest_encounters,
                    unlocked_unown_sets: self.unlocked_unown_sets(state)?,
                },
            )
            .map_err(|error| anyhow::anyhow!("roll exact SWEET_SCENT encounter: {error}"))?;
        state.random_state = rng.state();
        let (species, level, found) = wild_encounter
            .as_ref()
            .and_then(|roll| roll.resolved.as_ref())
            .map(|resolved| {
                (
                    resolved.encounter.species.clone(),
                    resolved.level.to_string(),
                    true,
                )
            })
            .unwrap_or_else(|| ("0".to_string(), "0".to_string(), false));
        state
            .script_runtime
            .memory
            .insert("wTempWildMonSpecies".to_string(), species);
        state
            .script_runtime
            .memory
            .insert("wCurPartyLevel".to_string(), level);
        state
            .script_runtime
            .memory
            .insert("wScriptVar".to_string(), u8::from(found).to_string());
        state.script_runtime.memory.insert(
            "wBattleType".to_string(),
            if wild_encounter
                .as_ref()
                .is_some_and(|roll| roll.roaming_slot.is_some())
            {
                "BATTLETYPE_ROAMING"
            } else {
                "0"
            }
            .to_string(),
        );
        state.script_runtime.script_value = Some(u8::from(found).to_string());
        Ok(SweetScentEncounterOutcome { wild_encounter })
    }

    pub fn check_coord_event_after_step(
        &self,
        state: &GameState,
        session: &OverworldSession,
    ) -> Option<CoordEventTrigger> {
        self.check_coord_event_after_step_checked(state, session)
            .ok()
            .flatten()
    }

    pub fn check_coord_event_after_step_checked(
        &self,
        state: &GameState,
        session: &OverworldSession,
    ) -> Result<Option<CoordEventTrigger>> {
        let current_scene = state
            .scenes
            .map_scenes
            .get(&session.map.name)
            .map(String::as_str);
        session
            .check_coord_event_checked(current_scene)
            .with_context(|| format!("check coord event on {}", session.map.name))
    }

    pub fn check_wild_encounter_after_step<S>(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        rng: &mut CrystalRandom<&mut S>,
    ) -> Result<Option<WildEncounterRoll>>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        // Validate the live session boundary before consulting any map-indexed
        // encounter state. Otherwise malformed coordinates can be hidden by a
        // missing metadata/roamer lookup and the caller receives an unrelated
        // error after entering the encounter pipeline.
        self.validate_runtime_map_tile(
            "wild encounter check",
            &session.map.name,
            session.player.tile,
        )?;
        // RandomEncounter calls CheckWildEncounterCooldown before checking
        // terrain or consuming encounter RNG. Crystal allows the check which
        // decrements 1 to 0; values remaining above zero suppress this step.
        if state.wild_encounter_cooldown > 0 {
            state.wild_encounter_cooldown -= 1;
            if state.wild_encounter_cooldown > 0 {
                return Ok(None);
            }
        }
        if state
            .flags
            .is_engine_flag_set("STATUSFLAGS_NO_WILD_ENCOUNTERS_F")
            .map_err(|error| anyhow::anyhow!("check NO_WILD_ENCOUNTERS flag: {error}"))?
        {
            return Ok(None);
        }
        let active_repel_item = if state.repel_steps_remaining > 0 {
            state.active_repel_item.clone()
        } else {
            None
        };
        let environment = &self
            .runtime_map_metadata_for_name(&session.map.name)?
            .environment;
        let swarm_encounters =
            self.active_swarm_wild_encounters_for_map(state, &session.map.name)?;
        let base_encounters = swarm_encounters
            .as_ref()
            .or_else(|| self.wild_encounters_for_map(&session.map.name));
        let zone = base_encounters
            .and_then(|data| data.zone_at(session.player.tile.x, session.player.tile.y));
        let zoned_encounters = zone.map(|zone| {
            let mut data = base_encounters
                .expect("zone came from encounter data")
                .clone();
            data.grass_rates = Some(zone.grass_rates.clone());
            data.grass = Some(zone.grass.clone());
            data.zones.clear();
            data
        });
        let land_encounters_on_any_land =
            can_encounter_on_any_non_ice_land(environment) || zone.is_some();
        let metadata = self.runtime_map_metadata_for_name(&session.map.name)?;
        let current_map = (
            u8::try_from(metadata.group_id).context("wild encounter map group exceeds byte")?,
            u8::try_from(metadata.map_id).context("wild encounter map number exceeds byte")?,
        );
        let contest_mode = self.bug_contest_encounter_mode(state)?;
        let contest_encounters = contest_mode
            .then(|| {
                self.bug_contest_config
                    .as_ref()
                    .context("active Bug Contest is missing required encounter configuration")
                    .map(|config| config.encounters.as_slice())
            })
            .transpose()?;
        let encounters = zoned_encounters.as_ref().or(base_encounters);
        if contest_mode || encounters.is_some() {
            self.preflight_exact_wild_encounter_transaction(
                state,
                session,
                encounters,
                contest_mode,
                land_encounters_on_any_land,
            )?;
        }
        let roll = session
            .check_wild_encounter_exact(
                encounters,
                &self.encounter_slot_tables,
                &self.encounter_music_modifiers,
                rng,
                EncounterCheckOptions {
                    time: state.time.time_of_day,
                    music_token: state.script_runtime.current_music.clone(),
                    has_cleanse_tag: Self::party_has_cleanse_tag(state),
                    active_repel_item,
                    lead_party_level: leading_usable_party_level(state),
                    land_encounters_on_any_land,
                    ..EncounterCheckOptions::default()
                },
                ExactEncounterContext {
                    roaming_pokemon: &state.roaming_pokemon,
                    current_map,
                    bug_contest_encounters: contest_encounters,
                    unlocked_unown_sets: self.unlocked_unown_sets(state)?,
                },
            )
            .map_err(|error| anyhow::anyhow!("check exact wild encounter: {error}"))?;
        Ok(roll)
    }

    fn unlocked_unown_sets(&self, state: &GameState) -> Result<u8> {
        let mut mask = 0u8;
        for (bit, flag) in [
            "ENGINE_UNLOCKED_UNOWNS_A_TO_K",
            "ENGINE_UNLOCKED_UNOWNS_L_TO_R",
            "ENGINE_UNLOCKED_UNOWNS_S_TO_W",
            "ENGINE_UNLOCKED_UNOWNS_X_TO_Z",
        ]
        .into_iter()
        .enumerate()
        {
            if state
                .flags
                .is_engine_flag_set(flag)
                .map_err(|error| anyhow::anyhow!("check {flag}: {error}"))?
            {
                mask |= 1 << bit;
            }
        }
        Ok(mask)
    }

    fn unique_runtime_map_metadata_for_constant(
        &self,
        map_constant: &str,
    ) -> Result<&RuntimeMapMetadata> {
        let mut matches = self
            .runtime_map_metadata
            .values()
            .filter(|metadata| metadata.constant == map_constant);
        let metadata = matches.next().with_context(|| {
            format!("compiled game pack missing runtime map constant {map_constant}")
        })?;
        anyhow::ensure!(
            matches.next().is_none(),
            "compiled game pack contains duplicate runtime map constant {map_constant}"
        );
        Ok(metadata)
    }

    fn preflight_exact_wild_encounter_transaction(
        &self,
        state: &GameState,
        session: &OverworldSession,
        encounters: Option<&WildEncounterData>,
        contest_mode: bool,
        land_encounters_on_any_land: bool,
    ) -> Result<()> {
        let Some(surface) = session
            .current_encounter_surface_checked_with_land_encounters(land_encounters_on_any_land)
            .map_err(|error| anyhow::anyhow!("resolve exact encounter surface: {error}"))?
        else {
            return Ok(());
        };
        let metadata = self.runtime_map_metadata_for_name(&session.map.name)?;
        let current_map = (
            u8::try_from(metadata.group_id).context("wild encounter map group exceeds byte")?,
            u8::try_from(metadata.map_id).context("wild encounter map number exceeds byte")?,
        );
        let mut candidates = Vec::<(&str, u8)>::new();
        self.wild_battle_music_for_map_time(&session.map.name, state.time.time_of_day)?;
        if contest_mode {
            let config = self
                .bug_contest_config
                .as_ref()
                .context("active Bug Contest is missing required encounter configuration")?;
            let event_flags = self
                .initialize_events
                .event_flags
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>();
            let issues = bug_contest_config_issues(config, &event_flags);
            anyhow::ensure!(
                issues.is_empty(),
                "invalid exact Bug Contest configuration: {issues:?}"
            );
            for entry in &config.encounters {
                anyhow::ensure!(
                    entry.species != "UNOWN",
                    "Bug Contest source encounter table cannot contain UNOWN"
                );
                candidates.push((entry.species.as_str(), entry.min_level));
                if entry.max_level != entry.min_level {
                    candidates.push((entry.species.as_str(), entry.max_level));
                }
            }
        } else {
            let encounters = encounters.context(
                "normal exact encounter preflight requires the current map encounter table",
            )?;
            let table = table_for_surface(encounters, surface, state.time.time_of_day)
                .map_err(|error| anyhow::anyhow!("resolve exact encounter table: {error}"))?;
            for percent_roll in 1..=100 {
                anyhow::ensure!(
                    select_wild_encounter(
                        encounters,
                        &self.encounter_slot_tables,
                        surface,
                        state.time.time_of_day,
                        percent_roll,
                        0,
                    )
                    .map_err(|error| anyhow::anyhow!(
                        "preflight exact encounter slot roll {percent_roll}: {error}"
                    ))?
                    .is_some(),
                    "exact encounter slot roll {percent_roll} did not resolve"
                );
            }
            for entry in table {
                candidates.push((entry.species.as_str(), entry.level));
            }
            if surface != EncounterSurface::Water {
                for roaming in &state.roaming_pokemon {
                    if (roaming.map_group, roaming.map_number) == current_map
                        && let Some(species) = roaming.species.as_deref()
                    {
                        let species_metadata = self.pokemon.get(species).with_context(|| {
                            format!("unknown selectable roaming species {species}")
                        })?;
                        let dvs = Dv::from_non_hp(
                            roaming.dvs_be[0] >> 4,
                            roaming.dvs_be[0] & 0x0f,
                            roaming.dvs_be[1] >> 4,
                            roaming.dvs_be[1] & 0x0f,
                        );
                        let candidate = create_pokemon_from_known_dvs(
                            species_metadata,
                            roaming.level,
                            dvs,
                            &self.learnsets,
                            &self.moves,
                            &self.growth_rates,
                        )
                        .with_context(|| {
                            format!("preflight selectable roaming species {species}")
                        })?;
                        anyhow::ensure!(
                            roaming.hp == 0 || u16::from(roaming.hp) <= candidate.max_hp,
                            "selectable roaming species {species} saved HP {} exceeds max {}",
                            roaming.hp,
                            candidate.max_hp
                        );
                        if roaming.hp == 0 {
                            let maximum_hp_candidate = create_pokemon_from_known_dvs(
                                species_metadata,
                                roaming.level,
                                Dv::from_non_hp(15, 15, 15, 15),
                                &self.learnsets,
                                &self.moves,
                                &self.growth_rates,
                            )?;
                            anyhow::ensure!(
                                maximum_hp_candidate.max_hp <= u16::from(u8::MAX),
                                "fresh selectable roaming species {species} can exceed the source HP byte: {}",
                                maximum_hp_candidate.max_hp
                            );
                        }
                        for item in [
                            species_metadata.item1.as_deref(),
                            species_metadata.item2.as_deref(),
                        ]
                        .into_iter()
                        .flatten()
                        {
                            anyhow::ensure!(
                                self.items.contains_key(item),
                                "selectable roaming species {species} references missing held item {item}"
                            );
                        }
                    }
                }
            }
        }

        let mut needs_magikarp_table = false;
        for (species_id, level) in candidates {
            let species = self
                .pokemon
                .get(species_id)
                .with_context(|| format!("unknown selectable wild species {species_id}"))?;
            create_pokemon_from_known_dvs(
                species,
                level,
                Dv::from_non_hp(0, 0, 0, 0),
                &self.learnsets,
                &self.moves,
                &self.growth_rates,
            )
            .with_context(|| {
                format!("preflight selectable wild species {species_id} at level {level}")
            })?;
            for item in [species.item1.as_deref(), species.item2.as_deref()]
                .into_iter()
                .flatten()
            {
                anyhow::ensure!(
                    self.items.contains_key(item),
                    "selectable wild species {species_id} references missing held item {item}"
                );
            }
            needs_magikarp_table |= species_id == "MAGIKARP";
        }
        if needs_magikarp_table {
            let issues = magikarp_length_table_issues(&self.magikarp_lengths);
            anyhow::ensure!(
                issues.is_empty(),
                "invalid exact Magikarp length table: {issues:?}"
            );
            let lake = self.unique_runtime_map_metadata_for_constant("LAKE_OF_RAGE")?;
            u8::try_from(lake.group_id).context("Lake of Rage map group exceeds source byte")?;
            u8::try_from(lake.map_id).context("Lake of Rage map number exceeds source byte")?;
        }
        Ok(())
    }

    fn bug_contest_encounter_mode(&self, state: &GameState) -> Result<bool> {
        let engine_active = state
            .flags
            .is_engine_flag_set("ENGINE_BUG_CONTEST_TIMER")
            .map_err(|error| anyhow::anyhow!("check Bug Contest timer flag: {error}"))?;
        anyhow::ensure!(
            !state.bug_contest.timer_active || engine_active,
            "active Bug Contest timer requires ENGINE_BUG_CONTEST_TIMER"
        );
        Ok(engine_active)
    }

    fn active_battle_end_context(
        &self,
        state: &GameState,
    ) -> Result<Option<(String, Option<u8>, Pokemon, String)>> {
        self.active_battle_end_context_on_map(state, None)
    }

    fn active_battle_end_context_on_map(
        &self,
        state: &GameState,
        trainer_map: Option<&str>,
    ) -> Result<Option<(String, Option<u8>, Pokemon, String)>> {
        let context = match &state.battle {
            BattleMemory::Wild {
                battle_type,
                map_name,
                roaming_slot,
                enemy_pokemon,
                ..
            } => Some((
                battle_type.clone(),
                *roaming_slot,
                enemy_pokemon.clone(),
                map_name.clone(),
            )),
            BattleMemory::StaticWild {
                battle_type,
                origin_map_name,
                roaming_slot,
                enemy_pokemon,
                ..
            } => Some((
                battle_type.clone(),
                *roaming_slot,
                enemy_pokemon.clone(),
                origin_map_name.clone(),
            )),
            BattleMemory::Trainer {
                battle_type,
                enemy_pokemon,
                ..
            } => {
                let map_name = match &state.overworld {
                    OverworldMemory::Active { map_name, .. } => map_name.clone(),
                    OverworldMemory::Inactive => trainer_map.map(str::to_string).ok_or_else(|| {
                        anyhow::anyhow!(
                            "active trainer battle has no authoritative overworld map for BattleEnd_HandleRoamMons"
                        )
                    })?,
                };
                Some((battle_type.clone(), None, enemy_pokemon.clone(), map_name))
            }
            BattleMemory::Inactive => None,
        };
        Ok(context)
    }

    fn finish_battle_roaming_update_exact<S>(
        &self,
        state: &mut GameState,
        battle_type: &str,
        roaming_slot: Option<u8>,
        enemy: &Pokemon,
        map_name: &str,
        divider: &mut S,
    ) -> Result<()>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        let metadata = self.runtime_map_metadata_for_name(map_name)?;
        let current_map = RoamingMapLocation {
            map_group: u8::try_from(metadata.group_id)
                .context("roaming battle-end map group exceeds byte")?,
            map_number: u8::try_from(metadata.map_id)
                .context("roaming battle-end map number exceeds byte")?,
        };
        let engine_state = RoamingEngineState {
            slots: state.roaming_pokemon.clone(),
            history: state.roaming_map_history,
            random_state: state.random_state,
            link_random: state.link_session.battle_random.clone(),
        };
        let (next, _) = battle_end_handle_roam_mons(
            &self.roaming_pokemon,
            &engine_state,
            RoamingBattleEndInput {
                battle_type,
                roaming_slot,
                enemy_hp: enemy.hp,
                battle_result: state.battle_result,
                link_battle: state.link_session.link_mode != 0,
                current_map,
            },
            divider,
        )
        .map_err(|error| anyhow::anyhow!("finish exact roaming battle: {error}"))?;
        state.roaming_pokemon = next.slots;
        state.roaming_map_history = next.history;
        state.random_state = next.random_state;
        state.link_session.battle_random = next.link_random;
        Ok(())
    }

    fn party_has_cleanse_tag(state: &GameState) -> bool {
        state
            .storage
            .party
            .pokemon
            .iter()
            .flatten()
            .any(|pokemon| pokemon.item.as_deref() == Some("CLEANSE_TAG"))
    }

    pub fn start_resolved_wild_encounter_after_step<S>(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        roll: &Option<WildEncounterRoll>,
        rng: &mut CrystalRandom<&mut S>,
    ) -> Result<Option<WildBattleStart>>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        let Some(encounter) = roll.clone().filter(|roll| roll.resolved.is_some()) else {
            return Ok(None);
        };
        self.start_exact_wild_battle_with_rng(state, session, encounter, rng)
            .map(Some)
    }

    fn start_exact_wild_battle_with_rng<S>(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        encounter: WildEncounterRoll,
        rng: &mut CrystalRandom<&mut S>,
    ) -> Result<WildBattleStart>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        anyhow::ensure!(
            state.pending_static_wild_terminal.is_none(),
            "cannot start a wild battle before the pending static-wild terminal resumes"
        );
        anyhow::ensure!(
            encounter.map_name == session.map.name,
            "wild encounter map {} does not match active map {}",
            encounter.map_name,
            session.map.name
        );
        anyhow::ensure!(
            encounter.tile == session.player.tile,
            "wild encounter tile {:?} does not match active player tile {:?}",
            encounter.tile,
            session.player.tile
        );
        anyhow::ensure!(
            encounter.time == state.time.time_of_day,
            "wild encounter time {:?} does not match active time {:?}",
            encounter.time,
            state.time.time_of_day
        );
        self.validate_runtime_map_tile(
            "exact wild battle encounter roll",
            &encounter.map_name,
            encounter.tile,
        )?;
        let resolved = encounter
            .resolved
            .as_ref()
            .context("cannot start exact wild battle from unresolved roll")?;
        let species = self
            .pokemon
            .get(&resolved.encounter.species)
            .with_context(|| format!("unknown wild species {}", resolved.encounter.species))?;
        let metadata = self.runtime_map_metadata_for_name(&encounter.map_name)?;
        let zone_uses_land_surface = self
            .wild_encounters_for_map(&encounter.map_name)
            .and_then(|data| data.zone_at(encounter.tile.x, encounter.tile.y))
            .is_some();
        let land_encounters_on_any_land =
            can_encounter_on_any_non_ice_land(&metadata.environment) || zone_uses_land_surface;
        let live_surface = session
            .current_encounter_surface_checked_with_land_encounters(land_encounters_on_any_land)
            .map_err(|error| anyhow::anyhow!("resolve live wild encounter surface: {error}"))?;
        anyhow::ensure!(
            live_surface == Some(encounter.surface),
            "wild encounter surface {:?} does not match live surface {:?}",
            encounter.surface,
            live_surface
        );
        let current_map = (
            u8::try_from(metadata.group_id).context("wild battle map group exceeds byte")?,
            u8::try_from(metadata.map_id).context("wild battle map number exceeds byte")?,
        );
        let (enemy_pokemon, battle_type) = if let Some(slot) = encounter.roaming_slot {
            let roaming = state
                .roaming_pokemon
                .get(usize::from(slot))
                .with_context(|| format!("roaming encounter slot {slot} is invalid"))?
                .clone();
            anyhow::ensure!(
                (roaming.map_group, roaming.map_number) == current_map,
                "roaming encounter slot {slot} is on map {}/{}, not active map {}/{}",
                roaming.map_group,
                roaming.map_number,
                current_map.0,
                current_map.1
            );
            let materialized = materialize_roaming_wild_battle_with_rng(
                &encounter,
                slot,
                &roaming,
                species,
                &self.learnsets,
                &self.moves,
                &self.growth_rates,
                rng,
            )
            .map_err(|error| anyhow::anyhow!("materialize roaming wild battle: {error}"))?;
            state.roaming_pokemon[usize::from(slot)] = materialized.roaming_after;
            (materialized.enemy_pokemon, "BATTLETYPE_ROAMING")
        } else {
            let battle_type = if self.bug_contest_encounter_mode(state)? {
                "BATTLETYPE_CONTEST"
            } else {
                "BATTLETYPE_NORMAL"
            };
            let lake_map = if resolved.encounter.species == "MAGIKARP" {
                let lake = self.unique_runtime_map_metadata_for_constant("LAKE_OF_RAGE")?;
                (
                    u8::try_from(lake.group_id).context("Lake of Rage map group exceeds byte")?,
                    u8::try_from(lake.map_id).context("Lake of Rage map number exceeds byte")?,
                )
            } else {
                current_map
            };
            let enemy = materialize_non_roaming_wild_battle_with_rng(
                &encounter,
                battle_type,
                species,
                &self.learnsets,
                &self.moves,
                &self.growth_rates,
                self.unlocked_unown_sets(state)?,
                state.player_id,
                current_map,
                lake_map,
                &self.magikarp_lengths,
                rng,
            )
            .map_err(|error| anyhow::anyhow!("materialize wild battle: {error}"))?;
            (enemy, battle_type)
        };
        let battle_music =
            self.wild_battle_music_for_map_time(&encounter.map_name, encounter.time)?;
        let battle = WildBattleStart {
            battle_type: battle_type.to_string(),
            battle_music,
            encounter,
            enemy_party: vec![enemy_pokemon.clone()],
            enemy_pokemon,
        };
        state
            .script_runtime
            .memory
            .insert("wBattleScriptFlags".to_string(), "0".to_string());
        activate_wild_battle_start(state, &battle, &self.items)
            .context("activate exact wild battle")?;
        crate::nuzlocke::register_wild_encounter(
            self.nuzlocke_rules,
            state,
            &battle.encounter.map_name,
            &battle.battle_type,
        );
        state.battle_active_party_index = first_available_battle_party_index(state);
        state.battle_active_enemy_party_index = Some(0);
        state.battle_rewarded_enemy_party_indices.clear();
        state.battle_evolvable_party_indices.clear();
        state.battle_escape_attempts = 0;
        state.battle_pay_day_money = 0;
        Ok(battle)
    }

    pub fn saved_wild_encounter_exists(&self, map_name: &str, species: &str, level: u8) -> bool {
        self.wild_encounters
            .get(map_name)
            .is_some_and(|encounters| wild_encounter_data_has(encounters, species, level))
            || self
                .field_encounters
                .get(map_name)
                .is_some_and(|encounters| field_encounter_data_has(encounters, species, level))
            || self.map_fishing_encounter_has(map_name, species, level)
    }

    pub fn map_fishing_encounter_has(&self, map_name: &str, species: &str, level: u8) -> bool {
        let Some(group_name) = self
            .maps
            .get(map_name)
            .and_then(|module| module.attributes.fishing_group.as_deref())
        else {
            return false;
        };
        let Some(group) = self.fishing.groups.get(group_name) else {
            return false;
        };
        group
            .rod_tables
            .values()
            .flat_map(|table| table.slots.iter())
            .any(|slot| fishing_slot_has(&self.fishing.time_groups, slot, species, level))
    }

    pub fn pokegear_landmark_for_map(
        &self,
        map_name: &str,
    ) -> Result<&crystal_core::models::display_metadata::PokegearLandmark> {
        let landmark_constant = self
            .pokegear_landmarks
            .map_to_landmark
            .get(map_name)
            .with_context(|| {
                format!("town map missing exact landmark mapping for map {map_name}")
            })?;
        self.pokegear_landmarks
            .landmarks
            .iter()
            .find(|landmark| landmark.constant == *landmark_constant)
            .with_context(|| {
                format!(
                    "town map landmark mapping for map {map_name} points to missing landmark {landmark_constant}"
                )
            })
    }

    fn landmark_byte(&self, constant: &str) -> Result<u8> {
        let landmark = self
            .pokegear_landmarks
            .landmarks
            .iter()
            .find(|landmark| landmark.constant == constant)
            .with_context(|| format!("compiled landmark catalog missing {constant}"))?;
        u8::try_from(landmark.id)
            .with_context(|| format!("compiled landmark {constant} id exceeds one byte"))
    }

    fn init_map_name_sign(
        &self,
        state: &mut GameState,
        map_name: &str,
    ) -> Result<crystal_core::systems::map_name_sign::MapNameSignOutcome> {
        let landmark = self.pokegear_landmark_for_map(map_name)?;
        let landmark = u8::try_from(landmark.id)
            .with_context(|| format!("compiled landmark for {map_name} exceeds one byte"))?;
        let environment = self.map_environment(map_name)?;
        let special = self.landmark_byte("LANDMARK_SPECIAL")?;
        let suppressed = [
            "LANDMARK_RADIO_TOWER",
            "LANDMARK_LAV_RADIO_TOWER",
            "LANDMARK_UNDERGROUND_PATH",
            "LANDMARK_INDIGO_PLATEAU",
            "LANDMARK_POWER_PLANT",
        ]
        .map(|constant| self.landmark_byte(constant))
        .into_iter()
        .collect::<Result<Vec<_>>>()?;
        Ok(crystal_core::systems::map_name_sign::init_map_name_sign(
            &mut state.map_name_sign,
            landmark,
            environment.eq_ignore_ascii_case("GATE"),
            matches!(
                map_name,
                "Route35NationalParkGate" | "Route36NationalParkGate"
            ),
            special,
            &suppressed,
        ))
    }

    pub fn saved_event_flag_exists(&self, flag: &str) -> bool {
        self.initialize_events
            .event_flags
            .iter()
            .any(|known| known == flag)
            || self
                .fruit_trees
                .0
                .keys()
                .any(|tree_id| fruit_tree_collected_flag(tree_id) == flag)
            || self.saved_story_event_constant_declares_flag(flag)
            || self.bug_contest_config.as_ref().is_some_and(|config| {
                config
                    .contestant_flags
                    .iter()
                    .any(|contestant_flag| contestant_flag == flag)
            })
            || self
                .battle_reward_rules
                .mom_progression_items
                .iter()
                .filter_map(|rule| rule.decoration_flag.as_deref())
                .any(|known| known == flag)
            || self
                .decorations
                .decorations
                .iter()
                .any(|decoration| decoration.event_flag == flag)
            || self.global_scripts.as_ref().is_some_and(|module| {
                module.script_flag_commands.iter().any(|command| {
                    command.flag_id == flag && !crystal_core::state::is_engine_flag_name(flag)
                })
            })
            || self.maps.values().any(|module| {
                module.script_flag_commands.iter().any(|command| {
                    command.flag_id == flag && !crystal_core::state::is_engine_flag_name(flag)
                }) || module
                    .objects
                    .iter()
                    .any(|object| object.event_flag == flag)
                    || module
                        .scripts
                        .values()
                        .filter_map(Value::as_array)
                        .any(|body| {
                            body.iter().any(|command| {
                                command.get("command").and_then(Value::as_str)
                                    == Some("conditional_event")
                                    && command
                                        .get("args")
                                        .and_then(Value::as_array)
                                        .and_then(|args| args.first())
                                        .and_then(Value::as_str)
                                        == Some(flag)
                            })
                        })
            })
    }

    pub fn saved_engine_flag_exists(&self, flag: &str) -> bool {
        self.initialize_events
            .engine_flags
            .iter()
            .any(|known| known == flag)
            || self.field_moves.strength.engine_flag == flag
            || self.field_moves.flash.engine_flag == flag
            || self.saved_story_event_constant_declares_flag(flag)
            || self.global_scripts.as_ref().is_some_and(|module| {
                module.script_flag_commands.iter().any(|command| {
                    command.flag_id == flag && crystal_core::state::is_engine_flag_name(flag)
                })
            })
            || self.maps.values().any(|module| {
                module.script_flag_commands.iter().any(|command| {
                    command.flag_id == flag && crystal_core::state::is_engine_flag_name(flag)
                })
            })
    }

    pub fn saved_story_event_constant_declares_flag(&self, flag: &str) -> bool {
        self.story_event_script_constants.global.contains_key(flag)
            || self
                .story_event_script_constants
                .maps
                .values()
                .any(|constants| constants.contains_key(flag))
    }

    pub fn saved_text_exists(&self, text_label: &str) -> bool {
        self.asm_text.contains_key(text_label)
            || self
                .global_scripts
                .as_ref()
                .is_some_and(|module| module.script_text_bodies.contains_key(text_label))
            || self
                .maps
                .values()
                .any(|module| module.script_text_bodies.contains_key(text_label))
    }

    pub fn saved_pokemon_status_exists(&self, status: &str) -> bool {
        self.step_event_rules.poison_status == status
            || self.capture_rules.status_bonus.contains_key(status)
            || self
                .items
                .values()
                .any(|item| item.status_heals.iter().any(|healed| healed == status))
            || status == "POKERUS"
    }

    pub fn tileset_collision(&self, tileset_name: &str) -> Result<TilesetCollision> {
        let definition = self
            .tilesets
            .get(tileset_name)
            .with_context(|| format!("compiled game pack missing tileset '{tileset_name}'"))?;
        tileset_collision_from_definition(tileset_name, definition)
    }

    fn load_base_json_for_compile(asset_root: &AssetRoot) -> Result<Self> {
        let index = asset_root.load_raw_content_pack_index_for_compile()?;
        Self::load_from_content_pack_index(asset_root, &index)
    }

    fn load_from_content_pack_index(
        asset_root: &AssetRoot,
        index: &ContentPackIndex,
    ) -> Result<Self> {
        let mut data = Self::default();
        data.apply_content_pack_index(asset_root, index)?;
        Ok(data)
    }

    pub(crate) fn apply_content_pack_index(
        &mut self,
        asset_root: &AssetRoot,
        index: &ContentPackIndex,
    ) -> Result<()> {
        index.validate()?;
        for pack in index.enabled_packs_sorted() {
            if let Some(compiled_path) = &pack.compiled {
                if self != &Self::default() {
                    anyhow::bail!(
                        "compiled game pack '{}' must be applied to an empty runtime dataset",
                        pack.id
                    );
                }
                let compiled_path = resolve_content_pack_compiled_game_pack_path(
                    asset_root,
                    &pack.id,
                    compiled_path,
                )?;
                let compiled = read_verified_compiled_game_pack(&compiled_path)
                    .with_context(|| format!("load compiled game pack {}", pack.id))?;
                let runtime_id = compiled.runtime_modpack_id()?;
                if runtime_id != pack.id {
                    anyhow::bail!(
                        "compiled game pack {} declared runtime modpack id {}",
                        pack.id,
                        runtime_id
                    );
                }
                *self = compiled.data().clone();
                continue;
            }

            for category in CONTENT_PACK_CATEGORIES {
                let mut seen_entries = BTreeSet::new();
                for entry in pack.files.entries(*category) {
                    if !seen_entries.insert(entry.as_str()) {
                        anyhow::bail!(
                            "content pack {} category {} includes duplicate file entry {}",
                            pack.id,
                            category.as_str(),
                            entry
                        );
                    }
                    if *category == ContentPackCategory::Audio {
                        validate_content_pack_audio_metadata_entry(&pack.id, entry)?;
                    } else {
                        validate_content_pack_json_entry(&pack.id, *category, entry)?;
                    }
                }
                for entry in pack.files.entries(*category) {
                    let path = resolve_content_pack_data_path(asset_root, &pack.id, entry)?;
                    let payload: Value = read_json_file(&path).with_context(|| {
                        format!(
                            "load content pack {} category {} file {}",
                            pack.id,
                            category.as_str(),
                            entry
                        )
                    })?;
                    self.apply_content_pack_payload(*category, payload)
                        .with_context(|| {
                            format!(
                                "apply content pack {} category {} file {}",
                                pack.id,
                                category.as_str(),
                                entry
                            )
                        })?;
                }
            }
        }
        Ok(())
    }

    fn apply_content_pack_payload(
        &mut self,
        category: ContentPackCategory,
        payload: Value,
    ) -> Result<()> {
        match category {
            ContentPackCategory::Pokemon => {
                for (species_id, species) in parse_object_map::<PokemonSpecies>(payload)? {
                    insert_keyed_pokemon_species(&mut self.pokemon, species_id, species)?;
                }
            }
            ContentPackCategory::Moves => {
                for (move_id, move_data) in parse_object_map::<Move>(payload)? {
                    validate_manifest_move(&move_data)?;
                    insert_keyed_move_data(&mut self.moves, move_id, move_data)?;
                }
            }
            ContentPackCategory::GrowthRates => {
                for (curve_id, curve) in
                    parse_object_map::<crystal_core::systems::experience::GrowthRateCurve>(payload)?
                {
                    insert_keyed_growth_rate_curve(&mut self.growth_rates, curve_id, curve)?;
                }
            }
            ContentPackCategory::Items => {
                for (item_id, item) in parse_object_map::<Item>(payload)? {
                    validate_manifest_item(&item)?;
                    insert_keyed_item(&mut self.items, item_id, item)?;
                }
            }
            ContentPackCategory::Marts => {
                merge_mart_payload(&mut self.marts, payload)?;
            }
            ContentPackCategory::CurrencyConstants => {
                merge_currency_constants_payload(&mut self.currency_constants, payload)?;
            }
            ContentPackCategory::WildEncounters => {
                for (map_name, data) in parse_object_map::<WildEncounterData>(payload)? {
                    insert_keyed_wild_encounter_data(&mut self.wild_encounters, map_name, data)?;
                }
            }
            ContentPackCategory::FieldEncounters => {
                for (map_name, data) in parse_object_map::<FieldEncounterData>(payload)? {
                    insert_keyed_field_encounter_data(&mut self.field_encounters, map_name, data)?;
                }
            }
            ContentPackCategory::RuntimeSpawnPoints => {
                merge_runtime_spawn_points(
                    &mut self.runtime_spawn_points,
                    parse_object_map_with_description::<RuntimeSpawnPoint>(
                        payload,
                        "runtime spawn points payload",
                    )?,
                )?;
            }
            ContentPackCategory::RuntimeMapMetadata => {
                merge_runtime_map_metadata(
                    &mut self.runtime_map_metadata,
                    parse_object_map_with_description::<RuntimeMapMetadata>(
                        payload,
                        "runtime map metadata payload",
                    )?,
                )?;
            }
            ContentPackCategory::FleeMons => {
                insert_flee_mon_tables(&mut self.flee_mons, serde_json::from_value(payload)?)?;
            }
            ContentPackCategory::RoamingPokemon => {
                merge_roaming_pokemon(&mut self.roaming_pokemon, serde_json::from_value(payload)?)?;
            }
            ContentPackCategory::BuenaPasswordCategories => {
                merge_buena_password_categories(
                    &mut self.buena_password_categories,
                    serde_json::from_value(payload)?,
                )?;
            }
            ContentPackCategory::BuenaPrizes => {
                merge_buena_prizes(&mut self.buena_prizes, serde_json::from_value(payload)?)?;
            }
            ContentPackCategory::KurtApricornRecipes => {
                merge_kurt_apricorn_recipes(
                    &mut self.kurt_apricorn_recipes,
                    serde_json::from_value(payload)?,
                )?;
            }
            ContentPackCategory::ShuckieGift => {
                insert_shuckie_gift(&mut self.shuckie_gift, serde_json::from_value(payload)?)?;
            }
            ContentPackCategory::DratiniMoveSets => {
                merge_dratini_move_sets(
                    &mut self.dratini_move_sets,
                    serde_json::from_value(payload)?,
                )?;
            }
            ContentPackCategory::BugContestConfig => {
                insert_bug_contest_config(
                    &mut self.bug_contest_config,
                    serde_json::from_value(payload)?,
                )?;
            }
            ContentPackCategory::BattleTowerRules => {
                insert_battle_tower_rules(
                    &mut self.battle_tower_rules,
                    serde_json::from_value(payload)?,
                )?;
            }
            ContentPackCategory::OakRatings => {
                insert_oak_rating_table(&mut self.oak_ratings, serde_json::from_value(payload)?)?;
            }
            ContentPackCategory::OddEggDefinitions => {
                insert_odd_egg_definitions(
                    &mut self.odd_egg_definitions,
                    serde_json::from_value(payload)?,
                )?;
            }
            ContentPackCategory::MagikarpLengths => {
                let entries = serde_json::from_value::<
                    crystal_core::systems::special_routines::MagikarpLengthTable,
                >(payload)?
                .0;
                insert_magikarp_length_table(&mut self.magikarp_lengths, entries)?;
            }
            ContentPackCategory::HappinessData => {
                insert_happiness_data(&mut self.happiness_data, serde_json::from_value(payload)?)?;
            }
            ContentPackCategory::EncounterSlotTables => {
                insert_encounter_slot_tables(
                    &mut self.encounter_slot_tables,
                    serde_json::from_value(payload)?,
                )?;
            }
            ContentPackCategory::EncounterMusicModifiers => {
                insert_encounter_music_modifiers(
                    &mut self.encounter_music_modifiers,
                    serde_json::from_value(payload)?,
                )?;
            }
            ContentPackCategory::BattleStatMultipliers => {
                insert_battle_stat_multiplier_tables(
                    &mut self.battle_stat_multipliers,
                    serde_json::from_value(payload)?,
                )?;
            }
            ContentPackCategory::CaptureWobbleProbabilities => {
                insert_capture_wobble_probabilities(
                    &mut self.capture_wobble_probabilities,
                    serde_json::from_value(payload)?,
                )?;
            }
            ContentPackCategory::CaptureRules => {
                insert_capture_rules(&mut self.capture_rules, serde_json::from_value(payload)?)?;
            }
            ContentPackCategory::BattleEscapeRules => {
                insert_battle_escape_rules(
                    &mut self.battle_escape_rules,
                    serde_json::from_value(payload)?,
                )?;
            }
            ContentPackCategory::MovePriorities => {
                insert_move_priority_table(
                    &mut self.move_priorities,
                    serde_json::from_value(payload)?,
                )?;
            }
            ContentPackCategory::TypeCategories => {
                insert_type_categories(
                    &mut self.type_categories,
                    serde_json::from_value(payload)?,
                )?;
            }
            ContentPackCategory::TypeEffectiveness => {
                insert_type_effectiveness(
                    &mut self.type_effectiveness,
                    serde_json::from_value(payload)?,
                )?;
            }
            ContentPackCategory::WeatherModifiers => {
                insert_weather_modifiers(
                    &mut self.weather_modifiers,
                    serde_json::from_value(payload)?,
                )?;
            }
            ContentPackCategory::BattleRewardRules => {
                insert_battle_reward_rules(
                    &mut self.battle_reward_rules,
                    serde_json::from_value(payload)?,
                )?;
            }
            ContentPackCategory::StepEventRules => {
                insert_step_event_rules(
                    &mut self.step_event_rules,
                    serde_json::from_value(payload)?,
                )?;
            }
            ContentPackCategory::Fishing => {
                insert_fishing_catalog(&mut self.fishing, serde_json::from_value(payload)?)?;
            }
            ContentPackCategory::FruitTrees => {
                merge_fruit_tree_payload(&mut self.fruit_trees, payload)?;
            }
            ContentPackCategory::FieldMoves => {
                insert_field_move_catalog(&mut self.field_moves, serde_json::from_value(payload)?)?;
            }
            ContentPackCategory::FieldBoxItems => {
                insert_field_box_items(
                    &mut self.field_box_items,
                    parse_object_map::<FieldBoxItemRule>(payload)?,
                )?;
            }
            ContentPackCategory::Decorations => {
                anyhow::ensure!(
                    self.decorations == DecorationCatalog::default(),
                    "duplicate decorations payload"
                );
                self.decorations = serde_json::from_value(payload)?;
            }
            ContentPackCategory::RuntimeTitleScreen => {
                insert_runtime_title_screen(
                    &mut self.runtime_title_screen,
                    serde_json::from_value(payload)?,
                )?;
            }
            ContentPackCategory::FlyDestinations => {
                for (flypoint_flag, destination) in parse_object_map::<FlyDestination>(payload)? {
                    insert_fly_destination(&mut self.fly_destinations, flypoint_flag, destination)?;
                }
            }
            ContentPackCategory::MapAttributes => {
                merge_map_attributes(
                    &mut self.map_attributes,
                    parse_object_map::<MapAttributes>(payload)?,
                )?;
            }
            ContentPackCategory::MapBlocks => {
                merge_map_block_payload(&mut self.map_blocks, payload)?;
            }
            ContentPackCategory::Learnsets => {
                merge_learnsets(&mut self.learnsets, parse_learnsets(payload)?)?;
            }
            ContentPackCategory::LevelUpMoves => {
                merge_level_up_moves_payload(&mut self.level_up_moves, payload)?;
            }
            ContentPackCategory::EggMoves => {
                merge_egg_moves_payload(&mut self.egg_moves, payload)?;
            }
            ContentPackCategory::Evolutions => {
                merge_evolution_payload(&mut self.evolutions, payload)?;
            }
            ContentPackCategory::Maps => {
                for (map_id, module) in parse_object_map::<MapModule>(payload)? {
                    insert_keyed_map_module(&mut self.maps, map_id, module)?;
                }
            }
            ContentPackCategory::MapScripts => {
                merge_map_script_payload(&mut self.map_scripts, payload)?;
            }
            ContentPackCategory::MapDimensions => {
                merge_map_dimensions_payload(&mut self.map_dimensions, payload)?;
            }
            ContentPackCategory::Npcs => {
                merge_npc_payload(&mut self.npcs, payload)?;
            }
            ContentPackCategory::PokegearLandmarks => {
                merge_pokegear_landmarks_payload(&mut self.pokegear_landmarks, payload)?;
            }
            ContentPackCategory::PcStrings => {
                merge_pc_strings(&mut self.pc_strings, parse_object_map::<String>(payload)?)?;
            }
            ContentPackCategory::MenuIcons => {
                merge_menu_icons(&mut self.menu_icons, parse_object_map::<String>(payload)?)?;
            }
            ContentPackCategory::Trainers => {
                for (trainer_id, trainer) in parse_object_map::<Trainer>(payload)? {
                    insert_keyed_trainer(&mut self.trainers, trainer_id, trainer)?;
                }
            }
            ContentPackCategory::TrainerClassNames => {
                merge_trainer_class_names(
                    &mut self.trainer_class_names,
                    parse_object_map::<String>(payload)?,
                )?;
            }
            ContentPackCategory::Pokedex => {
                merge_pokedex_payload(&mut self.pokedex, payload)?;
            }
            ContentPackCategory::PokedexEntries => {
                for (species, entry) in parse_object_map::<RuntimePokedexEntry>(payload)? {
                    insert_keyed_pokedex_entry(&mut self.pokedex_entries, species, entry)?;
                }
            }
            ContentPackCategory::PokemonFrontpicAnim => {
                merge_frontpic_anim_programs(&mut self.pokemon_frontpic_anim, payload)?;
            }
            ContentPackCategory::InitializeEvents => {
                insert_initialize_events(
                    &mut self.initialize_events,
                    serde_json::from_value(payload)?,
                )?;
            }
            ContentPackCategory::StoryEventScriptConstants => {
                insert_story_event_script_constants(
                    &mut self.story_event_script_constants,
                    serde_json::from_value(payload)?,
                )?;
            }
            ContentPackCategory::StoryEvents => {
                merge_raw_story_event_payload(
                    &mut self.story_events,
                    payload,
                    "story event payload",
                    "story event payload key",
                )?;
            }
            ContentPackCategory::PhoneScripts => {
                merge_raw_script_payload(
                    &mut self.phone_scripts,
                    payload,
                    "phone script payload",
                    "phone script payload key",
                )?;
            }
            ContentPackCategory::PhoneContacts => {
                merge_phone_contact_payload(&mut self.phone_contacts, payload)?;
            }
            ContentPackCategory::PermanentPhoneNumbers => {
                merge_token_keyed_map(
                    &mut self.permanent_phone_numbers,
                    parse_token_keyed_rule_map(payload, "permanent phone number")?,
                    "permanent phone number",
                )?;
            }
            ContentPackCategory::SpecialPhoneCalls => {
                merge_token_keyed_map(
                    &mut self.special_phone_calls,
                    parse_token_keyed_rule_map(payload, "special phone call")?,
                    "special phone call",
                )?;
            }
            ContentPackCategory::NpcTrades => {
                merge_token_keyed_map(
                    &mut self.npc_trades,
                    parse_token_keyed_rule_map(payload, "NPC trade")?,
                    "NPC trade",
                )?;
            }
            ContentPackCategory::SpecialRoutines => {
                merge_special_routine_rules(
                    &mut self.special_routines,
                    parse_token_keyed_rule_map(payload, "special routine")?,
                )?;
            }
            ContentPackCategory::AsmText => {
                merge_asm_text(&mut self.asm_text, payload)?;
            }
            ContentPackCategory::MoveNames => {
                insert_exact_string_vec_table(
                    &mut self.move_names,
                    parse_string_vec_payload(payload, "move names")?,
                    "move names",
                    "move name",
                )?;
            }
            ContentPackCategory::BattleAnimations => {
                merge_token_keyed_string_vec_map(
                    &mut self.battle_animations,
                    parse_object_map_with_description::<Vec<String>>(
                        payload,
                        "battle animation payload",
                    )?,
                    "battle animation",
                    "battle animation command",
                )?;
            }
            ContentPackCategory::BattleAnimationTable => {
                insert_token_string_vec_table(
                    &mut self.battle_animation_table,
                    parse_string_vec_payload(payload, "battle animation table")?,
                    "battle animation",
                    "battle animation table entry",
                )?;
            }
            ContentPackCategory::BattleAnimBundle => {
                insert_exact_string_bundle(
                    &mut self.battle_anim_bundle,
                    serde_json::to_string(&payload).context("encode battle animation bundle")?,
                    "battle animation bundle",
                    &[
                        "objects",
                        "framesets",
                        "oam_sets",
                        "gfx_table",
                        "gfx_sources",
                    ],
                )?;
            }
            ContentPackCategory::SpriteAnimBundle => {
                insert_exact_string_bundle(
                    &mut self.sprite_anim_bundle,
                    serde_json::to_string(&payload).context("encode sprite animation bundle")?,
                    "sprite animation bundle",
                    &["oam_sets", "framesets", "objects"],
                )?;
            }
            ContentPackCategory::SpritePaletteDefaults => {
                merge_sprite_palette_defaults(&mut self.sprite_palette_defaults, payload)?;
            }
            ContentPackCategory::PokegearTownMapPaletteMap => {
                merge_token_keyed_token_vec_map(
                    &mut self.pokegear_town_map_palette_map,
                    parse_object_map_with_description::<Vec<String>>(
                        payload,
                        "Pokegear town map palette payload",
                    )?,
                    "Pokegear town map palette entry",
                    "Pokegear town map palette value",
                )?;
            }
            ContentPackCategory::PokemonCries => {
                merge_pokemon_cries(&mut self.pokemon_cries, payload)?;
            }
            ContentPackCategory::Audio => {
                for (audio_id, audio_asset) in parse_object_map::<ModpackAudioAsset>(payload)? {
                    insert_keyed_audio_asset(&mut self.audio, audio_id, audio_asset)?;
                }
            }
            ContentPackCategory::Tilesets => {
                for (tileset_id, tileset) in parse_object_map::<TilesetDefinition>(payload)? {
                    insert_keyed_tileset_definition(&mut self.tilesets, tileset_id, tileset)?;
                }
            }
            ContentPackCategory::Playability => {
                let playability: PlayabilityRules = serde_json::from_value(payload)?;
                merge_playability_rules(&mut self.playability, &playability)?;
            }
        }
        Ok(())
    }

    pub(crate) fn apply_modpack(&mut self, manifest: &ModpackManifest) -> Result<()> {
        if manifest.payload.pokemon.is_empty() {
            self.pokemon.clear();
        } else {
            for (species_id, species) in &manifest.payload.pokemon {
                insert_keyed_pokemon_species(
                    &mut self.pokemon,
                    species_id.clone(),
                    species.clone(),
                )?;
            }
        }
        if manifest.payload.moves.is_empty() {
            self.moves.clear();
        } else {
            for (move_id, move_data) in &manifest.payload.moves {
                validate_manifest_move(move_data)?;
                insert_keyed_move_data(&mut self.moves, move_id.clone(), move_data.clone())?;
            }
        }
        if manifest.payload.evolutions == EvolutionTable::default() {
            self.evolutions = EvolutionTable::default();
        } else {
            merge_evolution_table(&mut self.evolutions, &manifest.payload.evolutions)?;
        }
        if manifest.payload.marts == MartCatalog::default() {
            self.marts = MartCatalog::default();
        } else {
            merge_mart_catalog(&mut self.marts, &manifest.payload.marts)?;
        }
        if manifest.payload.currency_constants.0.is_empty() {
            self.currency_constants.0.clear();
        } else {
            merge_currency_constants(
                &mut self.currency_constants,
                &manifest.payload.currency_constants,
            )?;
        }
        if manifest.payload.battle_reward_rules == BattleRewardRules::default() {
            self.battle_reward_rules = BattleRewardRules::default();
        } else {
            insert_battle_reward_rules(
                &mut self.battle_reward_rules,
                manifest.payload.battle_reward_rules.clone(),
            )?;
        }
        if manifest.payload.step_event_rules == StepEventRules::default() {
            self.step_event_rules = StepEventRules::default();
        } else {
            insert_step_event_rules(
                &mut self.step_event_rules,
                manifest.payload.step_event_rules.clone(),
            )?;
        }
        if manifest.payload.maps.is_empty() {
            self.maps.clear();
        } else {
            for (map_id, map) in &manifest.payload.maps {
                insert_keyed_map_module(&mut self.maps, map_id.clone(), map.clone())?;
            }
        }
        let move_ids: BTreeSet<String> = self.moves.keys().cloned().collect();
        if manifest.payload.items.is_empty() {
            self.items.clear();
        } else {
            for (item_id, item) in &manifest.payload.items {
                validate_manifest_item(item)?;
                validate_manifest_item_references(item, &move_ids)?;
                insert_keyed_item(&mut self.items, item_id.clone(), item.clone())?;
            }
        }
        if manifest.payload.wild_encounters.is_empty() {
            self.wild_encounters.clear();
        } else {
            for (map_name, wild_encounter_data) in &manifest.payload.wild_encounters {
                insert_keyed_wild_encounter_data(
                    &mut self.wild_encounters,
                    map_name.clone(),
                    wild_encounter_data.clone(),
                )?;
            }
        }
        if manifest.payload.field_encounters.is_empty() {
            self.field_encounters.clear();
        } else {
            for (map_name, field_encounter_data) in &manifest.payload.field_encounters {
                insert_keyed_field_encounter_data(
                    &mut self.field_encounters,
                    map_name.clone(),
                    field_encounter_data.clone(),
                )?;
            }
        }
        if manifest.payload.fishing == FishingCatalog::default() {
            self.fishing = FishingCatalog::default();
        } else {
            insert_fishing_catalog(&mut self.fishing, manifest.payload.fishing.clone())?;
        }
        if manifest.payload.fruit_trees.0.is_empty() {
            self.fruit_trees.0.clear();
        } else {
            merge_fruit_tree_catalog(&mut self.fruit_trees, &manifest.payload.fruit_trees)?;
        }
        if manifest.payload.field_moves == FieldMoveCatalog::default() {
            self.field_moves = FieldMoveCatalog::default();
        } else {
            insert_field_move_catalog(&mut self.field_moves, manifest.payload.field_moves.clone())?;
        }
        if manifest.payload.field_box_items.is_empty() {
            self.field_box_items.clear();
        } else {
            insert_field_box_items(
                &mut self.field_box_items,
                manifest.payload.field_box_items.clone(),
            )?;
        }
        self.decorations = manifest.payload.decorations.clone();
        if manifest.payload.runtime_title_screen == RuntimeTitleScreen::default() {
            self.runtime_title_screen = RuntimeTitleScreen::default();
        } else {
            insert_runtime_title_screen(
                &mut self.runtime_title_screen,
                manifest.payload.runtime_title_screen.clone(),
            )?;
        }
        if manifest.payload.runtime_spawn_points.is_empty() {
            self.runtime_spawn_points.clear();
        } else {
            merge_runtime_spawn_points(
                &mut self.runtime_spawn_points,
                manifest
                    .payload
                    .runtime_spawn_points
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect(),
            )?;
        }
        if manifest.payload.runtime_map_metadata.is_empty() {
            self.runtime_map_metadata.clear();
        } else {
            merge_runtime_map_metadata(
                &mut self.runtime_map_metadata,
                manifest
                    .payload
                    .runtime_map_metadata
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect(),
            )?;
        }
        if manifest.payload.flee_mons == FleeMonTables::default() {
            self.flee_mons = FleeMonTables::default();
        } else {
            insert_flee_mon_tables(&mut self.flee_mons, manifest.payload.flee_mons.clone())?;
        }
        if manifest.payload.roaming_pokemon.is_empty() {
            self.roaming_pokemon = RoamingPokemonCatalog::default();
        } else {
            merge_roaming_pokemon(
                &mut self.roaming_pokemon,
                manifest.payload.roaming_pokemon.clone(),
            )?;
        }
        if manifest
            .payload
            .buena_password_categories
            .categories
            .is_empty()
            && manifest.payload.buena_password_categories.order.is_empty()
        {
            self.buena_password_categories = BuenaPasswordCategories::default();
        } else {
            merge_buena_password_categories(
                &mut self.buena_password_categories,
                manifest.payload.buena_password_categories.clone(),
            )?;
        }
        if manifest.payload.buena_prizes.is_empty() {
            self.buena_prizes.clear();
        } else {
            merge_buena_prizes(
                &mut self.buena_prizes,
                manifest.payload.buena_prizes.clone(),
            )?;
        }
        if manifest.payload.kurt_apricorn_recipes.is_empty() {
            self.kurt_apricorn_recipes.clear();
        } else {
            merge_kurt_apricorn_recipes(
                &mut self.kurt_apricorn_recipes,
                manifest.payload.kurt_apricorn_recipes.clone(),
            )?;
        }
        if let Some(shuckie_gift) = &manifest.payload.shuckie_gift {
            insert_shuckie_gift(&mut self.shuckie_gift, shuckie_gift.clone())?;
        } else {
            self.shuckie_gift = None;
        }
        if manifest.payload.dratini_move_sets.is_empty() {
            self.dratini_move_sets.clear();
        } else {
            merge_dratini_move_sets(
                &mut self.dratini_move_sets,
                manifest.payload.dratini_move_sets.clone(),
            )?;
        }
        if let Some(bug_contest_config) = &manifest.payload.bug_contest_config {
            insert_bug_contest_config(&mut self.bug_contest_config, bug_contest_config.clone())?;
        } else {
            self.bug_contest_config = None;
        }
        if let Some(battle_tower_rules) = &manifest.payload.battle_tower_rules {
            insert_battle_tower_rules(&mut self.battle_tower_rules, battle_tower_rules.clone())?;
        } else {
            self.battle_tower_rules = None;
        }
        if manifest.payload.oak_ratings.is_empty() {
            self.oak_ratings.clear();
        } else {
            insert_oak_rating_table(&mut self.oak_ratings, manifest.payload.oak_ratings.clone())?;
        }
        if manifest.payload.odd_egg_definitions.is_empty() {
            self.odd_egg_definitions.clear();
        } else {
            insert_odd_egg_definitions(
                &mut self.odd_egg_definitions,
                manifest.payload.odd_egg_definitions.clone(),
            )?;
        }
        if manifest.payload.magikarp_lengths.is_empty() {
            self.magikarp_lengths.clear();
        } else {
            insert_magikarp_length_table(
                &mut self.magikarp_lengths,
                manifest.payload.magikarp_lengths.clone(),
            )?;
        }
        if let Some(happiness_data) = &manifest.payload.happiness_data {
            insert_happiness_data(&mut self.happiness_data, happiness_data.clone())?;
        } else {
            self.happiness_data = None;
        }
        if manifest.payload.encounter_slot_tables == EncounterSlotTables::default() {
            self.encounter_slot_tables = EncounterSlotTables::default();
        } else {
            insert_encounter_slot_tables(
                &mut self.encounter_slot_tables,
                manifest.payload.encounter_slot_tables.clone(),
            )?;
        }
        if manifest.payload.encounter_music_modifiers == EncounterMusicModifiers::default() {
            self.encounter_music_modifiers = EncounterMusicModifiers::default();
        } else {
            insert_encounter_music_modifiers(
                &mut self.encounter_music_modifiers,
                manifest.payload.encounter_music_modifiers.clone(),
            )?;
        }
        if manifest.payload.battle_stat_multipliers == BattleStatMultiplierTables::default() {
            self.battle_stat_multipliers = BattleStatMultiplierTables::default();
        } else {
            insert_battle_stat_multiplier_tables(
                &mut self.battle_stat_multipliers,
                manifest.payload.battle_stat_multipliers.clone(),
            )?;
        }
        if manifest.payload.capture_wobble_probabilities.is_empty() {
            self.capture_wobble_probabilities.clear();
        } else {
            insert_capture_wobble_probabilities(
                &mut self.capture_wobble_probabilities,
                manifest.payload.capture_wobble_probabilities.clone(),
            )?;
        }
        if manifest.payload.capture_rules == CaptureRules::default() {
            self.capture_rules = CaptureRules::default();
        } else {
            insert_capture_rules(
                &mut self.capture_rules,
                manifest.payload.capture_rules.clone(),
            )?;
        }
        if manifest.payload.battle_escape_rules == BattleEscapeRules::default() {
            self.battle_escape_rules = BattleEscapeRules::default();
        } else {
            insert_battle_escape_rules(
                &mut self.battle_escape_rules,
                manifest.payload.battle_escape_rules.clone(),
            )?;
        }
        if manifest.payload.move_priorities == MovePriorityTable::default() {
            self.move_priorities = MovePriorityTable::default();
        } else {
            insert_move_priority_table(
                &mut self.move_priorities,
                manifest.payload.move_priorities.clone(),
            )?;
        }
        if manifest.payload.type_categories == TypeCategories::default() {
            self.type_categories = TypeCategories::default();
        } else {
            insert_type_categories(
                &mut self.type_categories,
                manifest.payload.type_categories.clone(),
            )?;
        }
        if manifest.payload.type_effectiveness == TypeEffectivenessTable::default() {
            self.type_effectiveness = TypeEffectivenessTable::default();
        } else {
            insert_type_effectiveness(
                &mut self.type_effectiveness,
                manifest.payload.type_effectiveness.clone(),
            )?;
        }
        if manifest.payload.weather_modifiers == WeatherModifiers::default() {
            self.weather_modifiers = WeatherModifiers::default();
        } else {
            insert_weather_modifiers(
                &mut self.weather_modifiers,
                manifest.payload.weather_modifiers.clone(),
            )?;
        }
        if manifest.payload.pc_strings.is_empty() {
            self.pc_strings.clear();
        } else {
            merge_pc_strings(
                &mut self.pc_strings,
                manifest
                    .payload
                    .pc_strings
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect(),
            )?;
        }
        if manifest.payload.menu_icons.is_empty() {
            self.menu_icons.clear();
        } else {
            merge_menu_icons(
                &mut self.menu_icons,
                manifest
                    .payload
                    .menu_icons
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect(),
            )?;
        }
        if manifest.payload.pokedex_entries.is_empty() {
            self.pokedex_entries.clear();
        } else {
            for (species, entry) in &manifest.payload.pokedex_entries {
                insert_keyed_pokedex_entry(
                    &mut self.pokedex_entries,
                    species.clone(),
                    entry.clone(),
                )?;
            }
        }
        if manifest.payload.pokemon_frontpic_anim.is_empty() {
            self.pokemon_frontpic_anim.clear();
        } else {
            merge_frontpic_anim_entries(
                &mut self.pokemon_frontpic_anim,
                manifest
                    .payload
                    .pokemon_frontpic_anim
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect(),
            )?;
        }
        if manifest.payload.initialize_events == InitializeEventsConfig::default() {
            self.initialize_events = InitializeEventsConfig::default();
        } else {
            insert_initialize_events(
                &mut self.initialize_events,
                manifest.payload.initialize_events.clone(),
            )?;
        }
        if manifest.payload.story_event_script_constants == StoryEventScriptConstants::default() {
            self.story_event_script_constants = StoryEventScriptConstants::default();
        } else {
            insert_story_event_script_constants(
                &mut self.story_event_script_constants,
                manifest.payload.story_event_script_constants.clone(),
            )?;
        }
        if manifest.payload.asm_text.is_empty() {
            self.asm_text.clear();
        } else {
            merge_asm_text_entries(
                &mut self.asm_text,
                manifest
                    .payload
                    .asm_text
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect(),
            )?;
        }
        if manifest.payload.move_names.is_empty() {
            self.move_names.clear();
        } else {
            insert_token_string_vec_table(
                &mut self.move_names,
                manifest.payload.move_names.clone(),
                "move names",
                "move name",
            )?;
        }
        if manifest.payload.battle_animations.is_empty() {
            self.battle_animations.clear();
        } else {
            merge_token_keyed_string_vec_map(
                &mut self.battle_animations,
                manifest
                    .payload
                    .battle_animations
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect(),
                "battle animation",
                "battle animation command",
            )?;
        }
        if manifest.payload.battle_animation_table.is_empty() {
            self.battle_animation_table.clear();
        } else {
            insert_token_string_vec_table(
                &mut self.battle_animation_table,
                manifest.payload.battle_animation_table.clone(),
                "battle animation",
                "battle animation table entry",
            )?;
        }
        if manifest.payload.battle_anim_bundle.is_empty() {
            self.battle_anim_bundle.clear();
        } else {
            insert_exact_string_bundle(
                &mut self.battle_anim_bundle,
                manifest.payload.battle_anim_bundle.clone(),
                "battle animation bundle",
                &[
                    "objects",
                    "framesets",
                    "oam_sets",
                    "gfx_table",
                    "gfx_sources",
                ],
            )?;
        }
        if manifest.payload.sprite_anim_bundle.is_empty() {
            self.sprite_anim_bundle.clear();
        } else {
            insert_exact_string_bundle(
                &mut self.sprite_anim_bundle,
                manifest.payload.sprite_anim_bundle.clone(),
                "sprite animation bundle",
                &["oam_sets", "framesets", "objects"],
            )?;
        }
        if manifest.payload.sprite_palette_defaults.is_empty() {
            self.sprite_palette_defaults.clear();
        } else {
            merge_sprite_palette_default_entries(
                &mut self.sprite_palette_defaults,
                manifest
                    .payload
                    .sprite_palette_defaults
                    .iter()
                    .map(|(key, value)| (key.clone(), *value))
                    .collect(),
            )?;
        }
        if manifest.payload.pokegear_town_map_palette_map.is_empty() {
            self.pokegear_town_map_palette_map.clear();
        } else {
            merge_token_keyed_token_vec_map(
                &mut self.pokegear_town_map_palette_map,
                manifest
                    .payload
                    .pokegear_town_map_palette_map
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect(),
                "Pokegear town map palette entry",
                "Pokegear town map palette value",
            )?;
        }
        if manifest.payload.pokegear_landmarks.landmarks.is_empty()
            && manifest
                .payload
                .pokegear_landmarks
                .map_to_landmark
                .is_empty()
        {
            self.pokegear_landmarks.landmarks.clear();
            self.pokegear_landmarks.map_to_landmark.clear();
        } else {
            merge_pokegear_landmarks(
                &mut self.pokegear_landmarks,
                &manifest.payload.pokegear_landmarks,
            )?;
        }
        if manifest.payload.pokemon_cries.is_empty() {
            self.pokemon_cries.clear();
        } else {
            merge_pokemon_cry_entries(
                &mut self.pokemon_cries,
                manifest
                    .payload
                    .pokemon_cries
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect(),
            )?;
        }
        if manifest.payload.trainers.trainers.is_empty() {
            self.trainers.trainers.clear();
        } else {
            for (trainer_id, trainer) in &manifest.payload.trainers.trainers {
                insert_keyed_trainer(&mut self.trainers, trainer_id.clone(), trainer.clone())?;
            }
        }
        if manifest.payload.trainer_class_names.is_empty() {
            self.trainer_class_names.clear();
        } else {
            merge_trainer_class_names(
                &mut self.trainer_class_names,
                manifest.payload.trainer_class_names.clone(),
            )?;
        }
        if manifest.payload.phone_contacts.0.is_empty() {
            self.phone_contacts.0.clear();
        } else {
            merge_phone_contact_catalog(
                &mut self.phone_contacts,
                &manifest.payload.phone_contacts,
            )?;
        }
        if manifest.payload.permanent_phone_numbers.is_empty() {
            self.permanent_phone_numbers.clear();
        } else {
            merge_token_keyed_map(
                &mut self.permanent_phone_numbers,
                manifest.payload.permanent_phone_numbers.clone(),
                "permanent phone number",
            )?;
        }
        if manifest.payload.special_phone_calls.is_empty() {
            self.special_phone_calls.clear();
        } else {
            merge_token_keyed_map(
                &mut self.special_phone_calls,
                manifest.payload.special_phone_calls.clone(),
                "special phone call",
            )?;
        }
        if manifest.payload.npc_trades.is_empty() {
            self.npc_trades.clear();
        } else {
            merge_token_keyed_map(
                &mut self.npc_trades,
                manifest.payload.npc_trades.clone(),
                "NPC trade",
            )?;
        }
        if manifest.payload.special_routines.is_empty() {
            self.special_routines.clear();
        } else {
            merge_special_routine_rules(
                &mut self.special_routines,
                manifest.payload.special_routines.clone(),
            )?;
        }
        if manifest.payload.audio.is_empty() {
            self.audio.clear();
        } else {
            for (audio_id, audio_asset) in &manifest.payload.audio {
                insert_keyed_audio_asset(&mut self.audio, audio_id.clone(), audio_asset.clone())?;
            }
        }
        if manifest.payload.tilesets.is_empty() {
            self.tilesets.clear();
        } else {
            for (tileset_id, tileset) in &manifest.payload.tilesets {
                insert_keyed_tileset_definition(
                    &mut self.tilesets,
                    tileset_id.clone(),
                    tileset.clone(),
                )?;
            }
        }
        if manifest.payload.playability == PlayabilityRules::default() {
            self.playability = PlayabilityRules::default();
        } else {
            merge_playability_rules(&mut self.playability, &manifest.payload.playability)?;
        }
        Ok(())
    }

    pub fn create_pokemon(&self, species_id: &str, level: u8, dvs: Dv) -> Result<Pokemon> {
        let species = self
            .pokemon
            .get(species_id)
            .with_context(|| format!("unknown Pokemon species '{species_id}'"))?;
        Ok(create_pokemon_from_known_dvs(
            species,
            level,
            dvs,
            &self.learnsets,
            &self.moves,
            &self.growth_rates,
        )?)
    }

    fn start_fishing_battle_with_rng<S>(
        &self,
        state: &mut GameState,
        map_name: &str,
        tile: TilePosition,
        encounter: crystal_core::world::encounters::WildEncounter,
        time: TimeOfDay,
        bite_roll: u8,
        slot_roll: u8,
        rng: &mut CrystalRandom<&mut S>,
    ) -> Result<WildBattleStart>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        anyhow::ensure!(
            state.pending_static_wild_terminal.is_none(),
            "cannot start a fishing battle before the pending static-wild terminal resumes"
        );
        self.validate_runtime_map_tile("exact fishing battle", map_name, tile)?;
        let resolved = ResolvedWildEncounter {
            level: encounter.level,
            encounter,
            slot: 0,
        };
        let roll = WildEncounterRoll {
            map_name: map_name.to_string(),
            tile,
            surface: EncounterSurface::Water,
            time,
            threshold: 0,
            encounter_roll: bite_roll,
            slot_percent_roll: Some(slot_roll),
            level_roll: None,
            roaming_slot: None,
            resolved: Some(resolved.clone()),
            repelled_by: None,
        };
        let species = self
            .pokemon
            .get(&resolved.encounter.species)
            .with_context(|| format!("unknown fishing species {}", resolved.encounter.species))?;
        let metadata = self.runtime_map_metadata_for_name(map_name)?;
        let current_map = (
            u8::try_from(metadata.group_id).context("fishing battle map group exceeds byte")?,
            u8::try_from(metadata.map_id).context("fishing battle map number exceeds byte")?,
        );
        let lake_map = if resolved.encounter.species == "MAGIKARP" {
            let lake = self.unique_runtime_map_metadata_for_constant("LAKE_OF_RAGE")?;
            (
                u8::try_from(lake.group_id).context("Lake of Rage map group exceeds byte")?,
                u8::try_from(lake.map_id).context("Lake of Rage map number exceeds byte")?,
            )
        } else {
            current_map
        };
        let enemy_pokemon = materialize_non_roaming_wild_battle_with_rng(
            &roll,
            "BATTLETYPE_FISH",
            species,
            &self.learnsets,
            &self.moves,
            &self.growth_rates,
            self.unlocked_unown_sets(state)?,
            state.player_id,
            current_map,
            lake_map,
            &self.magikarp_lengths,
            rng,
        )
        .map_err(|error| anyhow::anyhow!("materialize exact fishing battle: {error}"))?;
        let battle = WildBattleStart {
            battle_type: "BATTLETYPE_FISH".to_string(),
            battle_music: self.wild_battle_music_for_map_time(map_name, time)?,
            encounter: roll,
            enemy_party: vec![enemy_pokemon.clone()],
            enemy_pokemon,
        };
        state
            .script_runtime
            .memory
            .insert("wBattleScriptFlags".to_string(), "0".to_string());
        activate_wild_battle_start(state, &battle, &self.items)
            .context("activate exact fishing battle")?;
        crate::nuzlocke::register_wild_encounter(
            self.nuzlocke_rules,
            state,
            map_name,
            &battle.battle_type,
        );
        state.battle_active_party_index = first_available_battle_party_index(state);
        state.battle_active_enemy_party_index = Some(0);
        state.battle_rewarded_enemy_party_indices.clear();
        state.battle_evolvable_party_indices.clear();
        state.battle_escape_attempts = 0;
        state.battle_pay_day_money = 0;
        Ok(battle)
    }

    fn validate_runtime_map_tile(
        &self,
        context: &str,
        map_name: &str,
        tile: TilePosition,
    ) -> Result<()> {
        let module = self.maps.get(map_name).with_context(|| {
            format!("{context} map {map_name} is missing from compiled pack maps")
        })?;
        let map =
            OverworldMapData::from_attributes(map_name, &module.attributes, module.blocks.clone());
        let (width, height) = map.checked_tile_bounds().with_context(|| {
            format!("{context} map {map_name} runtime tile bounds overflow supported coordinates")
        })?;
        if tile.x < 0
            || tile.y < 0
            || i32::from(tile.x) >= i32::from(width)
            || i32::from(tile.y) >= i32::from(height)
        {
            anyhow::bail!(
                "{context} tile ({}, {}) is outside compiled map {map_name} runtime tile bounds {width}x{height}",
                tile.x,
                tile.y
            );
        }
        Ok(())
    }

    pub fn static_wild_battle_start<S>(
        &self,
        request: StaticWildBattleRequest,
        random_state: crystal_core::random::CrystalRandomState,
        divider: &mut S,
    ) -> Result<StaticWildBattleStart>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        if request.battle_music.is_empty() {
            anyhow::bail!("static wild battle request missing exact battle_music");
        }
        Ok(static_wild_battle_start(
            &self.pokemon,
            &self.learnsets,
            &self.moves,
            &self.growth_rates,
            request,
            random_state,
            divider,
        )?)
    }

    pub fn wild_battle_music_for_map_time(
        &self,
        map_name: &str,
        time: TimeOfDay,
    ) -> Result<String> {
        let landmark = self.pokegear_landmark_for_map(map_name)?;
        let music_id = match landmark.region.as_str() {
            "KANTO" => "MUSIC_KANTO_WILD_BATTLE",
            "JOHTO" => match time {
                TimeOfDay::Night => "MUSIC_JOHTO_WILD_BATTLE_NIGHT",
                TimeOfDay::Morning | TimeOfDay::Day => "MUSIC_JOHTO_WILD_BATTLE",
            },
            region => anyhow::bail!(
                "town map landmark for map {map_name} has unsupported region {region:?} for wild battle music"
            ),
        };
        if !self.audio.iter().any(|asset| asset.id == music_id) {
            anyhow::bail!("wild battle music {music_id} for map {map_name} is missing from pack");
        }
        Ok(music_id.to_string())
    }

    pub fn trainer_battle_start(
        &self,
        state: &crystal_core::state::GameState,
        request: TrainerBattleRequest,
    ) -> Result<TrainerBattleStartStatus> {
        Ok(trainer_battle_start(
            state,
            &self.trainers,
            &self.pokemon,
            &self.learnsets,
            &self.moves,
            &self.growth_rates,
            request,
        )?)
    }

    pub fn overworld_map(&self, map_name: &str) -> Result<OverworldMapData> {
        let attributes = self
            .map_attributes
            .get(map_name)
            .with_context(|| format!("missing map attributes for {map_name}"))?;
        let blocks_label =
            required_map_attribute_label(map_name, "blocks_label", &attributes.blocks_label)?;
        let encoded_blocks = self
            .map_blocks
            .get(blocks_label)
            .with_context(|| format!("missing map block payload {blocks_label}"))?;
        let metatile_ids = decode_base64_bytes(encoded_blocks)
            .with_context(|| format!("decode map block payload {blocks_label}"))?
            .into_iter()
            .map(u16::from)
            .collect();
        Ok(OverworldMapData::from_attributes(
            map_name,
            attributes,
            metatile_ids,
        ))
    }

    /// Builds a definitive map module from the compiled split payload tables.
    ///
    /// This is not a compatibility path: every referenced label and payload must
    /// already be present in the compiled pack and must parse as the exact
    /// runtime schema.
    pub fn assemble_map_module_from_compiled_payloads(&self, map_name: &str) -> Result<MapModule> {
        if let Some(module) = self.maps.get(map_name) {
            return Ok(module.clone());
        }
        let attributes = self
            .map_attributes
            .get(map_name)
            .with_context(|| format!("missing map attributes for {map_name}"))?
            .clone();
        let map_scripts_label = required_map_attribute_label(
            map_name,
            "map_scripts_label",
            &attributes.map_scripts_label,
        )?;
        let map_events_label = required_map_attribute_label(
            map_name,
            "map_events_label",
            &attributes.map_events_label,
        )?;
        let blocks_label =
            required_map_attribute_label(map_name, "blocks_label", &attributes.blocks_label)?;

        if !self.map_scripts.contains_key(map_scripts_label) {
            anyhow::bail!("missing map scripts label {map_scripts_label}");
        }
        let events_script = self
            .map_scripts
            .get(map_events_label)
            .with_context(|| format!("missing map events label {map_events_label}"))?;
        let objects_payload = self
            .npcs
            .get(map_name)
            .with_context(|| format!("missing NPC object payload for {map_name}"))?;
        let encoded_blocks = self
            .map_blocks
            .get(blocks_label)
            .with_context(|| format!("missing map block payload {blocks_label}"))?;

        let map_scripts = self
            .map_scripts
            .get(map_scripts_label)
            .with_context(|| format!("missing map scripts label {map_scripts_label}"))?;
        let scripts = runtime_module_script_subset(
            &self.map_scripts,
            [map_scripts_label, map_events_label],
            true,
        );
        let scenes = parse_map_scene_table(map_name, map_scripts)?;
        let map_script_section_commands =
            parse_map_script_section_commands(map_name, map_scripts_label, map_scripts)?;
        let map_event_section_commands =
            parse_map_event_section_commands(map_name, map_events_label, events_script)?;
        let events = parse_map_events(map_name, events_script)?;
        let trainer_scripts = parse_trainer_scripts(map_name, &scripts)?;
        let scripted_trainer_battles = parse_scripted_trainer_battles(map_name, &scripts)?;
        let scripted_wild_battles = parse_scripted_wild_battles(map_name, &scripts)?;
        let script_item_grants = parse_script_item_grants(map_name, &scripts)?;
        let (script_item_checks, script_item_takes) =
            parse_script_item_accesses(map_name, &scripts)?;
        let script_economy_commands = parse_script_economy_commands(map_name, &scripts)?;
        let gift_pokemon_scripts =
            parse_gift_pokemon_scripts(map_name, &scripts, &self.story_event_script_constants)?;
        let script_flag_commands = parse_script_flag_commands(map_name, &scripts)?;
        let script_scene_commands = parse_script_scene_commands(map_name, &scripts)?;
        let script_audio_commands = parse_script_audio_commands(map_name, &scripts)?;
        let script_block_changes = parse_script_block_changes(map_name, &scripts)?;
        let script_object_commands = parse_script_object_commands(map_name, &scripts)?;
        let script_movements = parse_script_movements(map_name, &scripts, &script_object_commands)?;
        let map_name_by_constant = self.map_name_by_constant_from_attributes()?;
        let script_map_commands =
            parse_script_map_commands(map_name, &scripts, &map_name_by_constant)?;
        let script_text_commands = parse_script_text_commands(map_name, &scripts)?;
        let script_text_bodies = parse_script_text_bodies(map_name, &scripts)?;
        let script_menu_definitions = parse_script_menu_definitions(map_name, &scripts)?;
        let script_vertical_menus =
            parse_script_vertical_menus(map_name, &scripts, &script_menu_definitions)?;
        let script_elevators = parse_script_elevators(map_name, &scripts, &map_name_by_constant)?;
        let script_variable_commands = parse_script_variable_commands(map_name, &scripts)?;
        let script_control_commands = parse_script_control_commands(map_name, &scripts)?;
        let objects: Vec<ObjectEvent> = serde_json::from_value(objects_payload.clone())
            .with_context(|| format!("parse NPC object payload for {map_name}"))?;
        let script_field_pickups = parse_script_field_pickups(map_name, &scripts, &objects)?;
        let script_shop_commands = parse_script_shop_commands(map_name, &scripts)?;
        let script_phone_commands = parse_script_phone_commands(map_name, &scripts)?;
        let script_runtime_commands = parse_script_runtime_commands(map_name, &scripts)?;
        let script_swarm_commands = parse_script_swarm_commands(map_name, &scripts)?;
        let blocks = decode_base64_bytes(encoded_blocks)
            .with_context(|| format!("decode map block payload {blocks_label}"))?
            .into_iter()
            .map(u16::from)
            .collect();

        Ok(MapModule {
            id: map_name.to_string(),
            attributes,
            scripts,
            trainer_scripts,
            scripted_trainer_battles,
            scripted_wild_battles,
            script_item_grants,
            script_item_checks,
            script_item_takes,
            script_economy_commands,
            gift_pokemon_scripts,
            script_flag_commands,
            script_scene_commands,
            script_audio_commands,
            script_block_changes,
            script_object_commands,
            script_movements,
            script_map_commands,
            script_text_commands,
            script_text_bodies,
            script_menu_definitions,
            script_vertical_menus,
            script_elevators,
            script_variable_commands,
            script_control_commands,
            script_field_pickups,
            script_shop_commands,
            script_phone_commands,
            script_runtime_commands,
            script_swarm_commands,
            map_script_section_commands,
            map_event_section_commands,
            scenes,
            events,
            objects,
            blocks,
        })
    }

    pub fn resolve_warp_transition(&self, trigger: &WarpTrigger) -> Result<WarpTransition> {
        if !is_exact_map_reference_token(&trigger.warp.target_map) {
            anyhow::bail!(
                "warp {} on {} has invalid target_map field {:?}",
                trigger.warp.index,
                trigger.map_name,
                trigger.warp.target_map
            );
        }
        if trigger.warp.target_map != trigger.warp.target_map_constant {
            anyhow::bail!(
                "warp {} on {} target_map {:?} does not match target_map_constant {:?}",
                trigger.warp.index,
                trigger.map_name,
                trigger.warp.target_map,
                trigger.warp.target_map_constant
            );
        }
        let destination_map = self
            .map_name_for_constant(&trigger.warp.target_map_constant)
            .with_context(|| {
                format!(
                    "unknown target map constant '{}' for warp {} on {}",
                    trigger.warp.target_map_constant, trigger.warp.index, trigger.map_name
                )
            })?;
        let destination_attributes =
            self.map_attributes.get(&destination_map).with_context(|| {
                format!(
                    "warp target '{}' missing attributes (referenced by {})",
                    destination_map, trigger.map_name
                )
            })?;
        let destination_events_label = required_map_attribute_label(
            &destination_map,
            "map_events_label",
            &destination_attributes.map_events_label,
        )
        .with_context(|| format!("resolve warp target {destination_map} map_events_label"))?;
        let destination_events_payload = self
            .map_scripts
            .get(destination_events_label)
            .with_context(|| format!("missing map events label {destination_events_label}"))?;
        let destination_events = parse_map_events(&destination_map, destination_events_payload)
            .with_context(|| format!("parse warp target events for {destination_map}"))?;
        if trigger.warp.target_warp_id < 1 {
            anyhow::bail!(
                "warp {} on {} has dynamic target warp id {}",
                trigger.warp.index,
                trigger.map_name,
                trigger.warp.target_warp_id
            );
        }
        let destination_index = trigger
            .warp
            .target_warp_id
            .checked_sub(1)
            .with_context(|| {
                format!(
                    "warp {} on {} has invalid target warp id 0",
                    trigger.warp.index, trigger.map_name
                )
            })? as usize;
        let destination_warp = destination_events
            .warps
            .get(destination_index)
            .cloned()
            .with_context(|| {
                format!(
                    "warp id {} referenced by {} exceeds available warps ({}) on {}",
                    trigger.warp.target_warp_id,
                    trigger.map_name,
                    destination_events.warps.len(),
                    destination_map
                )
            })?;

        let destination_tile =
            checked_runtime_map_event_tile(destination_warp.x, destination_warp.y).with_context(
                || {
                    format!(
                        "warp id {} on {} coordinate ({}, {}) overflows runtime tile coordinates",
                        trigger.warp.target_warp_id,
                        destination_map,
                        destination_warp.x,
                        destination_warp.y
                    )
                },
            )?;

        Ok(WarpTransition {
            trigger: trigger.clone(),
            destination: WarpDestination {
                map_name: destination_map,
                tile: destination_tile,
                warp: destination_warp,
            },
        })
    }

    pub fn resolve_warp_transition_with_state(
        &self,
        state: &mut GameState,
        trigger: &WarpTrigger,
    ) -> Result<WarpTransition> {
        const LINK_ROOM_CONSTANTS: [&str; 5] = [
            "TRADE_CENTER",
            "COLOSSEUM",
            "TIME_CAPSULE",
            "MOBILE_TRADE_ROOM",
            "MOBILE_BATTLE_ROOM",
        ];

        let source_constant = self.map_constant(&trigger.map_name)?;
        let previous_is_link_room = state
            .previous_warp_map_name
            .as_deref()
            .and_then(|map_name| self.map_constant(map_name).ok())
            .is_some_and(|constant| LINK_ROOM_CONSTANTS.contains(&constant));

        let transition = if trigger.warp.target_warp_id < 1 {
            let preserve_backup = (source_constant == "POKECENTER_2F" && previous_is_link_room)
                || source_constant.ends_with("_ELEVATOR");
            if !preserve_backup {
                state.backup_warp_map_name = state.previous_warp_map_name.clone();
                state.backup_warp_index = state.previous_warp_index;
            }
            let destination_map = state.backup_warp_map_name.clone().with_context(|| {
                format!(
                    "dynamic warp {} on {} has no saved backup map",
                    trigger.warp.index, trigger.map_name
                )
            })?;
            let destination_warp_id = Self::required_dynamic_backup_warp_index(
                state,
                trigger.warp.index,
                &trigger.map_name,
            )?;
            let destination =
                self.resolve_warp_destination(&destination_map, destination_warp_id, trigger)?;
            WarpTransition {
                trigger: trigger.clone(),
                destination,
            }
        } else {
            self.resolve_warp_transition(trigger)?
        };

        let destination_constant = self.map_constant(&transition.destination.map_name)?;
        let source_environment = self.map_environment(&trigger.map_name)?;
        let destination_environment =
            self.map_environment(&transition.destination.map_name)?;
        let destination_tileset = self.map_tileset_name(&transition.destination.map_name)?;
        if matches!(source_environment, "ROUTE" | "TOWN")
            && matches!(
                destination_environment,
                "INDOOR" | "CAVE" | "DUNGEON" | "GATE"
            )
            && matches!(destination_tileset, "pokecenter" | "pokecom_center")
        {
            state.last_spawn_map_constant = Some(source_constant.to_string());
        }
        let source_is_link_room = LINK_ROOM_CONSTANTS.contains(&source_constant);
        let destination_is_link_room = LINK_ROOM_CONSTANTS.contains(&destination_constant);
        let moving_between_link_room_and_center = (source_constant == "POKECENTER_2F"
            && destination_is_link_room)
            || (source_is_link_room && destination_constant == "POKECENTER_2F");
        let leaving_dynamic_elevator =
            trigger.warp.target_warp_id < 1 && source_constant.ends_with("_ELEVATOR");
        if !moving_between_link_room_and_center && !leaving_dynamic_elevator {
            state.backup_warp_map_name = Some(trigger.map_name.clone());
        }
        if destination_constant == "POKECENTER_2F" && !source_is_link_room {
            state.backup_warp_index = Some(trigger.warp.index);
        }
        state.previous_warp_map_name = Some(trigger.map_name.clone());
        state.previous_warp_index = Some(trigger.warp.index);
        Ok(transition)
    }

    fn required_dynamic_backup_warp_index(
        state: &GameState,
        warp_index: u16,
        map_name: &str,
    ) -> Result<u16> {
        state
            .backup_warp_index
            .filter(|warp_id| *warp_id > 0)
            .with_context(|| {
                format!("dynamic warp {warp_index} on {map_name} has no saved nonzero backup warp")
            })
    }

    fn resolve_warp_destination(
        &self,
        destination_map: &str,
        destination_warp_id: u16,
        trigger: &WarpTrigger,
    ) -> Result<WarpDestination> {
        let destination_attributes = self
            .map_attributes
            .get(destination_map)
            .with_context(|| format!("warp target '{destination_map}' missing attributes"))?;
        let events_label = required_map_attribute_label(
            destination_map,
            "map_events_label",
            &destination_attributes.map_events_label,
        )?;
        let payload = self
            .map_scripts
            .get(events_label)
            .with_context(|| format!("missing map events label {events_label}"))?;
        let events = parse_map_events(destination_map, payload)
            .with_context(|| format!("parse warp target events for {destination_map}"))?;
        let destination_warp = events
            .warps
            .get(usize::from(destination_warp_id - 1))
            .cloned()
            .with_context(|| {
                format!(
                    "backup warp id {destination_warp_id} for {destination_map} exceeds available warps ({})",
                    events.warps.len()
                )
            })?;
        let tile = checked_runtime_map_event_tile(destination_warp.x, destination_warp.y)
            .with_context(|| {
                format!(
                    "warp {} on {} resolves to overflowing destination coordinates ({}, {})",
                    trigger.warp.index, trigger.map_name, destination_warp.x, destination_warp.y
                )
            })?;
        Ok(WarpDestination {
            map_name: destination_map.to_string(),
            tile,
            warp: destination_warp,
        })
    }

    pub fn map_name_for_constant_from_metadata(&self, map_constant: &str) -> Option<String> {
        map_constants(self).get(map_constant).cloned()
    }

    pub fn resolve_connection_transition(
        &self,
        trigger: &ConnectionTrigger,
    ) -> Result<ConnectionTransition> {
        let target_map = trigger.connection.target_map.clone();
        let target_attributes = self.map_attributes.get(&target_map).with_context(|| {
            format!(
                "connection target '{}' missing attributes (referenced by {})",
                target_map, trigger.map_name
            )
        })?;
        let target_tile = connection_destination_tile(
            trigger.tile,
            &trigger.connection.direction,
            trigger.connection.offset,
            target_attributes,
        )?;

        Ok(ConnectionTransition {
            trigger: trigger.clone(),
            destination: ConnectionDestination {
                map_name: target_map,
                tile: target_tile,
            },
        })
    }

    fn map_name_by_constant_from_attributes(&self) -> Result<BTreeMap<String, String>> {
        let mut names = BTreeMap::new();
        for (map_name, attributes) in &self.map_attributes {
            let Some(map_constant) = attributes.map_constant.as_ref() else {
                continue;
            };
            if let Some(previous) = names.insert(map_constant.clone(), map_name.clone()) {
                anyhow::bail!("duplicate map constant {map_constant} on {previous} and {map_name}");
            }
        }
        Ok(names)
    }
}

