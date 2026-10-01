impl GameDataSet {
    pub fn item(&self, item_id: &str) -> Result<&Item> {
        self.items
            .get(item_id)
            .with_context(|| format!("compiled game pack missing item {item_id}"))
    }

    pub fn use_bag_item(
        &self,
        state: &mut GameState,
        item_id: &str,
        context: ItemUseContext,
    ) -> Result<ItemUseOutcome> {
        core_use_bag_item(
            state,
            &self.items,
            ItemUseRequest {
                item_id: item_id.to_string(),
                context,
            },
        )
        .map_err(|error| anyhow::anyhow!("use bag item {item_id}: {error:?}"))
    }

    pub fn field_repel_steps(&self, item_id: &str) -> Result<u16> {
        let item = self.item(item_id)?;
        validate_repel_item(&self.field_moves, item)
            .map_err(|error| anyhow::anyhow!("validate field repel item {item_id}: {error:?}"))
    }

    pub fn field_bicycle_item(&self, item_id: &str) -> Result<&Item> {
        let item = self.item(item_id)?;
        validate_bicycle_item(&self.field_moves, item)
            .map_err(|error| anyhow::anyhow!("validate field bicycle item {item_id}: {error:?}"))?;
        Ok(item)
    }

    pub fn field_itemfinder_item(&self, item_id: &str) -> Result<&Item> {
        let item = self.item(item_id)?;
        validate_itemfinder_item(&self.field_moves, item).map_err(|error| {
            anyhow::anyhow!("validate field itemfinder item {item_id}: {error:?}")
        })?;
        Ok(item)
    }

    pub fn field_squirtbottle_item(&self, item_id: &str) -> Result<&Item> {
        let item = self.item(item_id)?;
        validate_squirtbottle_item(&self.field_moves, item).map_err(|error| {
            anyhow::anyhow!("validate field squirtbottle item {item_id}: {error:?}")
        })?;
        Ok(item)
    }

    pub fn field_coin_case_item(&self, item_id: &str) -> Result<&Item> {
        let item = self.item(item_id)?;
        validate_coin_case_item(&self.field_moves, item).map_err(|error| {
            anyhow::anyhow!("validate field coin case item {item_id}: {error:?}")
        })?;
        Ok(item)
    }

    pub fn field_blue_card_item(&self, item_id: &str) -> Result<&Item> {
        let item = self.item(item_id)?;
        validate_blue_card_item(&self.field_moves, item).map_err(|error| {
            anyhow::anyhow!("validate field blue card item {item_id}: {error:?}")
        })?;
        Ok(item)
    }

    pub fn field_town_map_item(&self, item_id: &str) -> Result<&Item> {
        let item = self.item(item_id)?;
        validate_town_map_item(&self.field_moves, item).map_err(|error| {
            anyhow::anyhow!("validate field town map item {item_id}: {error:?}")
        })?;
        Ok(item)
    }

    pub fn field_pokegear_item(&self, item_id: &str) -> Result<&Item> {
        let item = self.item(item_id)?;
        validate_pokegear_item(&self.field_moves, item).map_err(|error| {
            anyhow::anyhow!("validate field pokegear item {item_id}: {error:?}")
        })?;
        Ok(item)
    }

    pub fn field_box_item(&self, item_id: &str) -> Result<(&Item, &FieldBoxItemRule)> {
        let rule = self
            .field_box_items
            .get(item_id)
            .with_context(|| format!("field box item {item_id} is not defined by the pack"))?;
        let item = self.item(item_id)?;
        if item.effect != rule.effect {
            anyhow::bail!(
                "field box item {item_id} effect {} does not match pack rule effect {}",
                item.effect,
                rule.effect
            );
        }
        if item.field_menu != "ITEMMENU_CURRENT" {
            anyhow::bail!(
                "field box item {item_id} has field_menu {}, expected ITEMMENU_CURRENT",
                item.field_menu
            );
        }
        Ok((item, rule))
    }

    pub fn field_escape_item(&self, item_id: &str) -> Result<&Item> {
        let item = self.item(item_id)?;
        validate_field_escape_item(&self.field_moves, item)
            .map_err(|error| anyhow::anyhow!("use field escape item {item_id}: {error:?}"))?;
        Ok(item)
    }

    fn require_field_usable_item_in_bag<'a>(
        state: &GameState,
        item_id: &str,
        item: &'a Item,
        context: &str,
    ) -> Result<&'a Item> {
        if !item.field_usable {
            anyhow::bail!("field {context} item {item_id} is not usable in the field");
        }
        if !state.bag.has_item(item) {
            anyhow::bail!("field {context} item {item_id} is not in the bag");
        }
        Ok(item)
    }

    pub fn use_bag_repel_in_field(
        &self,
        state: &mut GameState,
        item_id: &str,
    ) -> Result<FieldRepelItemUseOutcome> {
        self.require_no_active_battle(state, "field repel item")?;
        if state.repel_steps_remaining > 0 {
            anyhow::bail!("the repel used earlier is still in effect");
        }
        let steps = self.field_repel_steps(item_id)?;
        let item = self.item(item_id)?;
        Self::require_field_usable_item_in_bag(state, item_id, item, "repel")?;
        let item_use = self.use_bag_item(state, item_id, ItemUseContext::Field)?;
        let repel = apply_repel_item_use(state, item_id, steps)
            .map_err(|error| anyhow::anyhow!("apply field repel item {item_id}: {error:?}"))?;
        Ok(FieldRepelItemUseOutcome {
            item_use,
            repel_steps_before: repel.repel_steps_before,
            repel_steps_after: repel.repel_steps_after,
            active_repel_item_before: repel.active_repel_item_before,
            active_repel_item_after: repel.active_repel_item_after,
        })
    }

    pub fn use_bag_bicycle_in_field(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        item_id: &str,
    ) -> Result<FieldBicycleItemUseOutcome> {
        self.require_no_active_battle(state, "field bicycle item")?;
        let item = self.field_bicycle_item(item_id)?;
        Self::require_field_usable_item_in_bag(state, item_id, item, "bicycle")?;
        let map_name = overworld.map.name.clone();
        let environment = self.map_environment(&map_name)?;
        if !is_bicycle_environment(environment) {
            anyhow::bail!("cannot use field bicycle item {item_id} in environment {environment}");
        }
        let sample = sample_collision(&overworld.map, &overworld.tileset, overworld.player.tile)
            .with_context(|| {
                format!(
                    "field bicycle item {item_id} cannot sample current tile {},{}",
                    overworld.player.tile.x, overworld.player.tile.y
                )
            })?;
        if sample.permission & 0x0f != permissions::FLOOR {
            anyhow::bail!(
                "cannot use field bicycle item {item_id} on permission {:#04x}",
                sample.permission
            );
        }
        let mode_before = overworld.player.mode;
        let always_on_bike = state
            .flags
            .is_engine_flag_set("ENGINE_ALWAYS_ON_BIKE")
            .context("check ENGINE_ALWAYS_ON_BIKE")?;
        let mode_after = match mode_before {
            MovementMode::Normal => MovementMode::Bike,
            MovementMode::Bike if always_on_bike => {
                anyhow::bail!("cannot get off bicycle while ENGINE_ALWAYS_ON_BIKE is set");
            }
            MovementMode::Bike => MovementMode::Normal,
            MovementMode::Skate | MovementMode::Surf | MovementMode::SurfPika => {
                anyhow::bail!("cannot toggle bicycle from movement mode {mode_before:?}");
            }
        };
        let music = match mode_after {
            MovementMode::Bike => Some("MUSIC_BICYCLE".to_string()),
            MovementMode::Normal => self.map_music(&map_name)?.map(str::to_owned),
            MovementMode::Skate | MovementMode::Surf | MovementMode::SurfPika => {
                anyhow::bail!("bicycle toggle finished in invalid mode {mode_after:?}")
            }
        };
        let item_use = self.use_bag_item(state, item_id, ItemUseContext::Field)?;
        overworld.player.mode = mode_after;
        commit_overworld_snapshot(state, &overworld.snapshot(), SpawnMemoryUpdate::Preserve);
        apply_map_music_context(state, music);
        Ok(FieldBicycleItemUseOutcome {
            item_use,
            map_name,
            permission: sample.permission,
            mode_before,
            mode_after,
        })
    }

    pub fn use_bag_itemfinder_in_field(
        &self,
        state: &mut GameState,
        overworld: &OverworldSession,
        item_id: &str,
    ) -> Result<FieldItemfinderUseOutcome> {
        self.require_no_active_battle(state, "field itemfinder item")?;
        let item = self.field_itemfinder_item(item_id)?;
        Self::require_field_usable_item_in_bag(state, item_id, item, "itemfinder")?;
        let found =
            self.find_itemfinder_hidden_item(state, &overworld.map.name, overworld.player.tile)?;
        let itemfinder_sound_cues = if found.is_some() { 8 } else { 0 };
        let item_use = self.use_bag_item(state, item_id, ItemUseContext::Field)?;
        Ok(FieldItemfinderUseOutcome {
            item_use,
            player_tile: overworld.player.tile,
            found,
            itemfinder_sound_cues,
        })
    }

    pub fn use_bag_squirtbottle_in_field(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        item_id: &str,
    ) -> Result<FieldSquirtBottleUseOutcome> {
        self.require_no_active_battle(state, "field squirtbottle item")?;
        let item = self.field_squirtbottle_item(item_id)?;
        Self::require_field_usable_item_in_bag(state, item_id, item, "squirtbottle")?;
        let script_labels = self.map_script_labels(&overworld.map.name)?;
        let mut target =
            resolve_squirtbottle_target(overworld, |script| script_labels.contains(script))
                .map_err(|error| anyhow::anyhow!("{error}"))?;
        // Crystal's bag effect enters the exported watering sequence directly;
        // the object's interaction entry includes a separate Yes/No question.
        // Other authored tree scripts keep their own dispatch entry.
        if overworld.map.name == "Route36"
            && target.target_script.as_deref() == Some("SudowoodoScript")
        {
            anyhow::ensure!(script_labels.contains("WateredWeirdTreeScript"),
                "Route36 SquirtBottle use requires WateredWeirdTreeScript");
            target.target_script = Some("WateredWeirdTreeScript".to_string());
        }
        let item_use = self.use_bag_item(state, item_id, ItemUseContext::Field)?;
        if let Some(script) = target.target_script.as_ref() {
            commit_interaction_script_dispatch(
                state,
                &mut overworld.last_talked_object_identifier,
                &overworld.map.name,
                script,
                target.target_object_identifier.as_deref(),
            )
            .map_err(|error| {
                anyhow::anyhow!(
                    "dispatch field squirtbottle item {item_id} script {script}: {error:?}"
                )
            })?;
            commit_overworld_snapshot(state, &overworld.snapshot(), SpawnMemoryUpdate::Preserve);
        }
        Ok(FieldSquirtBottleUseOutcome {
            item_use,
            player_tile: overworld.player.tile,
            target_tile: target.target_tile,
            target_object_identifier: target.target_object_identifier,
            target_movement: target.target_movement,
            target_script: target.target_script,
        })
    }

    pub fn use_bag_story_key_in_field(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        item_id: &str,
    ) -> Result<FieldStoryKeyUseOutcome> {
        self.require_no_active_battle(state, "field story key item")?;
        let item = self.item(item_id)?;
        let (rule, expected_effect) = if item_id == self.field_moves.card_key.item_id {
            (&self.field_moves.card_key, "CARD_KEY")
        } else if item_id == self.field_moves.basement_key.item_id {
            (&self.field_moves.basement_key, "BASEMENT_KEY")
        } else {
            anyhow::bail!("field story key item {item_id} has no exact compiled rule");
        };
        anyhow::ensure!(
            item.effect == expected_effect,
            "field story key item {item_id} has effect {}, expected {expected_effect}",
            item.effect
        );
        Self::require_field_usable_item_in_bag(state, item_id, item, "story key")?;
        let facing_tile = checked_move_by_stride(
            overworld.player.tile,
            overworld.player.facing,
            StepOptions::default().stride_tiles,
        )
        .context("field story key facing tile overflow")?;
        if overworld.map.name != rule.map_name
            || rule
                .required_facing
                .is_some_and(|facing| overworld.player.facing != facing)
            || facing_tile != rule.target_tile
        {
            anyhow::bail!("cannot use field story key item {item_id} here");
        }
        anyhow::ensure!(
            self.map_script_labels(&rule.map_name)?
                .contains(&rule.target_script),
            "field story key item {item_id} references missing exact script {}",
            rule.target_script
        );
        let item_use = self.use_bag_item(state, item_id, ItemUseContext::Field)?;
        // QueueScript changes only the next script pointer. Unlike an object
        // interaction, these item effects do not write wLastTalked.
        state.script_runtime.next_script = Some(ScriptLocation {
            origin_map_name: rule.map_name.clone(),
            script: rule.target_script.clone(),
        });
        commit_overworld_snapshot(state, &overworld.snapshot(), SpawnMemoryUpdate::Preserve);
        Ok(FieldStoryKeyUseOutcome {
            item_use,
            map_name: rule.map_name.clone(),
            player_tile: overworld.player.tile,
            target_tile: rule.target_tile,
            target_script: rule.target_script.clone(),
        })
    }

    pub fn use_bag_coin_case_in_field(
        &self,
        state: &mut GameState,
        item_id: &str,
    ) -> Result<FieldKeyItemBalanceUseOutcome> {
        self.require_no_active_battle(state, "field coin case item")?;
        let item = self.field_coin_case_item(item_id)?;
        Self::require_field_usable_item_in_bag(state, item_id, item, "coin case")?;
        let balance = u32::from(state.coins);
        let item_use = self.use_bag_item(state, item_id, ItemUseContext::Field)?;
        Ok(FieldKeyItemBalanceUseOutcome {
            item_use,
            balance_label: "COIN".to_string(),
            balance,
        })
    }

    pub fn use_bag_blue_card_in_field(
        &self,
        state: &mut GameState,
        item_id: &str,
    ) -> Result<FieldKeyItemBalanceUseOutcome> {
        self.require_no_active_battle(state, "field blue card item")?;
        let item = self.field_blue_card_item(item_id)?;
        Self::require_field_usable_item_in_bag(state, item_id, item, "blue card")?;
        let balance = u32::from(
            blue_card_balance(state)
                .map_err(|error| anyhow::anyhow!("read field blue card balance: {error:?}"))?,
        );
        let item_use = self.use_bag_item(state, item_id, ItemUseContext::Field)?;
        Ok(FieldKeyItemBalanceUseOutcome {
            item_use,
            balance_label: "POINT".to_string(),
            balance,
        })
    }

    pub fn use_bag_town_map_in_field(
        &self,
        state: &mut GameState,
        overworld: &OverworldSession,
        item_id: &str,
    ) -> Result<FieldTownMapUseOutcome> {
        self.require_no_active_battle(state, "field town map item")?;
        let item = self.field_town_map_item(item_id)?;
        Self::require_field_usable_item_in_bag(state, item_id, item, "town map")?;
        let map_name = overworld.map.name.clone();
        let map_constant = self.map_constant(&map_name)?.to_string();
        let environment = self.map_environment(&map_name)?.to_string();
        let landmark = self.pokegear_landmark_for_map(&map_name)?.clone();
        let item_use = self.use_bag_item(state, item_id, ItemUseContext::Field)?;
        Ok(FieldTownMapUseOutcome {
            item_use,
            map_name,
            map_constant,
            environment,
            landmark,
        })
    }

    pub fn use_bag_pokegear_in_field(
        &self,
        state: &mut GameState,
        item_id: &str,
    ) -> Result<FieldPokegearUseOutcome> {
        self.require_no_active_battle(state, "field pokegear item")?;
        let item = self.field_pokegear_item(item_id)?;
        Self::require_field_usable_item_in_bag(state, item_id, item, "pokegear")?;
        let item_use = self.use_bag_item(state, item_id, ItemUseContext::Field)?;
        Ok(FieldPokegearUseOutcome { item_use })
    }

    pub fn use_bag_box_in_field(
        &self,
        state: &mut GameState,
        item_id: &str,
    ) -> Result<FieldBoxItemUseOutcome> {
        self.require_no_active_battle(state, "field box item")?;
        let (item, rule) = self.field_box_item(item_id)?;
        Self::require_field_usable_item_in_bag(state, item_id, item, "box")?;
        let decoration_flag = rule.decoration_flag.as_str();
        let already_owned = state
            .flags
            .is_event_flag_set(decoration_flag)
            .with_context(|| format!("check field box decoration flag {decoration_flag}"))?;
        state
            .flags
            .set_event_flag(decoration_flag, true)
            .with_context(|| format!("set field box decoration flag {decoration_flag}"))?;
        let item_use = self.use_bag_item(state, item_id, ItemUseContext::Field)?;
        Ok(FieldBoxItemUseOutcome {
            item_use,
            decoration_flag: decoration_flag.to_string(),
            already_owned,
        })
    }

    pub fn use_bag_escape_rope_in_field(
        &self,
        state: &mut GameState,
        overworld: &OverworldSession,
        item_id: &str,
    ) -> Result<FieldEscapeRopeUseOutcome> {
        self.require_no_active_battle(state, "field escape item")?;
        let item = self.field_escape_item(item_id)?;
        Self::require_field_usable_item_in_bag(state, item_id, item, "escape")?;
        let source_map = overworld.map.name.clone();
        let current_environment = self.map_environment(&source_map)?;
        if !is_escape_rope_environment(current_environment) {
            anyhow::bail!(
                "cannot use field escape item {item_id} in environment {current_environment}"
            );
        }
        let destination =
            self.saved_dig_warp_destination(state, &format!("field escape item {item_id}"))?;
        apply_escape_rope_chamber_effect(state, &source_map)
            .map_err(|error| anyhow::anyhow!("apply Escape Rope chamber effect: {error}"))?;
        let item_use = self.use_bag_item(state, item_id, ItemUseContext::Field)?;
        Ok(FieldEscapeRopeUseOutcome {
            item_use,
            source_map,
            destination_map: destination.map_name,
            destination_warp_index: destination.warp_index,
            destination_tile: destination.tile,
        })
    }

    pub fn use_bag_escape_rope_in_session(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        item_id: &str,
        _music_ids: &BTreeSet<String>,
    ) -> Result<FieldEscapeRopeUseOutcome> {
        let mut staged_state = state.clone();
        anyhow::ensure!(
            staged_state.script_runtime.pending_field_travel.is_none(),
            "cannot prepare Escape Rope while another field travel is pending"
        );
        let outcome = self.use_bag_escape_rope_in_field(&mut staged_state, overworld, item_id)?;
        staged_state.script_runtime.pending_field_travel = Some(PendingFieldTravel {
            move_id: item_id.to_string(),
            actor_party_index: None,
            actor_species: None,
            source_map: outcome.source_map.clone(),
            destination_map: outcome.destination_map.clone(),
            destination_tile: outcome.destination_tile,
            destination_spawn_identifier: None,
            destination_warp_index: Some(outcome.destination_warp_index),
            flypoint_flag: None,
        });
        *state = staged_state;
        Ok(outcome)
    }

    pub fn apply_cut_field_move(
        &self,
        state: &mut GameState,
        storage: &PokemonStorage,
        map: &mut OverworldMapData,
        tileset: &TilesetCollision,
        party_index: usize,
        metatile_x: u16,
        metatile_y: u16,
    ) -> Result<FieldMoveBlockOutcome> {
        let tileset_name = self.map_tileset_name(&map.name)?;
        core_apply_cut_field_move(
            &self.field_moves,
            state,
            storage,
            map,
            tileset,
            tileset_name,
            party_index,
            metatile_x,
            metatile_y,
        )
        .map_err(anyhow::Error::new)
        .context("use CUT field move")
    }

    pub fn field_block_target_metatile_in_front(
        &self,
        overworld: &OverworldSession,
    ) -> Result<(u16, u16)> {
        let target = Self::checked_runtime_field_move_target(
            "BLOCK_FIELD_MOVE",
            overworld.player.tile,
            overworld.player.facing,
        )?;
        let (width, height) = overworld.map.checked_tile_bounds().with_context(|| {
            format!(
                "map {} runtime tile bounds overflow supported coordinate range",
                overworld.map.name
            )
        })?;
        if target.x < 0
            || target.y < 0
            || i32::from(target.x) >= i32::from(width)
            || i32::from(target.y) >= i32::from(height)
        {
            anyhow::bail!(
                "field block target tile ({}, {}) is outside map {} runtime tile bounds {width}x{height}",
                target.x,
                target.y,
                overworld.map.name
            );
        }
        let metatile_x = target.x.div_euclid(METATILE_WIDTH);
        let metatile_y = target.y.div_euclid(METATILE_WIDTH);
        Ok((
            u16::try_from(metatile_x).with_context(|| {
                format!("field block target metatile x {metatile_x} cannot be represented")
            })?,
            u16::try_from(metatile_y).with_context(|| {
                format!("field block target metatile y {metatile_y} cannot be represented")
            })?,
        ))
    }

    pub fn use_cut_field_move(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        party_index: usize,
        metatile_x: u16,
        metatile_y: u16,
    ) -> Result<FieldMoveBlockOutcome> {
        self.require_no_active_battle(state, "CUT field move")?;
        anyhow::ensure!(
            state.script_runtime.pending_block_field_move.is_none(),
            "cannot prepare CUT while another block field move is pending"
        );
        let storage = state.storage.clone();
        let mut validation_state = state.clone();
        let mut validation_map = overworld.map.clone();
        let outcome = self.apply_cut_field_move(
            &mut validation_state,
            &storage,
            &mut validation_map,
            &overworld.tileset,
            party_index,
            metatile_x,
            metatile_y,
        )?;
        state
            .script_runtime
            .memory
            .insert("wCurPartyMon".to_string(), party_index.to_string());
        state.script_runtime.pending_block_field_move = Some(outcome.clone());
        Ok(outcome)
    }

    pub fn use_cut_field_move_in_front(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        party_index: usize,
    ) -> Result<FieldMoveBlockOutcome> {
        let (metatile_x, metatile_y) = self.field_block_target_metatile_in_front(overworld)?;
        self.use_cut_field_move(state, overworld, party_index, metatile_x, metatile_y)
    }

    pub fn apply_whirlpool_field_move(
        &self,
        state: &mut GameState,
        storage: &PokemonStorage,
        map: &mut OverworldMapData,
        tileset: &TilesetCollision,
        party_index: usize,
        metatile_x: u16,
        metatile_y: u16,
    ) -> Result<FieldMoveBlockOutcome> {
        let tileset_name = self.map_tileset_name(&map.name)?;
        core_apply_whirlpool_field_move(
            &self.field_moves,
            state,
            storage,
            map,
            tileset,
            tileset_name,
            party_index,
            metatile_x,
            metatile_y,
        )
        .map_err(anyhow::Error::new)
        .context("use WHIRLPOOL field move")
    }

    pub fn use_whirlpool_field_move(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        party_index: usize,
        metatile_x: u16,
        metatile_y: u16,
    ) -> Result<FieldMoveBlockOutcome> {
        self.require_no_active_battle(state, "WHIRLPOOL field move")?;
        anyhow::ensure!(
            state.script_runtime.pending_block_field_move.is_none(),
            "cannot prepare WHIRLPOOL while another block field move is pending"
        );
        let storage = state.storage.clone();
        let mut validation_state = state.clone();
        let mut validation_map = overworld.map.clone();
        let outcome = self.apply_whirlpool_field_move(
            &mut validation_state,
            &storage,
            &mut validation_map,
            &overworld.tileset,
            party_index,
            metatile_x,
            metatile_y,
        )?;
        state
            .script_runtime
            .memory
            .insert("wCurPartyMon".to_string(), party_index.to_string());
        state.script_runtime.pending_block_field_move = Some(outcome.clone());
        Ok(outcome)
    }

    pub fn use_whirlpool_field_move_in_front(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        party_index: usize,
    ) -> Result<FieldMoveBlockOutcome> {
        let (metatile_x, metatile_y) = self.field_block_target_metatile_in_front(overworld)?;
        self.use_whirlpool_field_move(state, overworld, party_index, metatile_x, metatile_y)
    }

    pub fn validate_saved_pending_field_moves(&self, state: &GameState) -> Result<()> {
        if let Some(pending) = &state.script_runtime.pending_flash_field_move {
            let rule = &self.field_moves.flash;
            anyhow::ensure!(
                pending.move_id == rule.move_id
                    && pending.engine_flag == rule.engine_flag
                    && !pending.was_set
                    && pending.is_set,
                "saved pending FLASH outcome diverges from the compiled field-move rule"
            );
            let actor = state
                .storage
                .party
                .pokemon
                .get(pending.actor_party_index)
                .and_then(Option::as_ref)
                .with_context(|| {
                    format!(
                        "saved pending FLASH actor party index {} is empty",
                        pending.actor_party_index
                    )
                })?;
            anyhow::ensure!(
                actor.species.id == pending.actor_species,
                "saved pending FLASH actor species {} does not match party species {}",
                pending.actor_species,
                actor.species.id
            );
            anyhow::ensure!(
                actor
                    .moves
                    .iter()
                    .any(|known| known.name == pending.move_id),
                "saved pending FLASH actor no longer knows the move"
            );
            anyhow::ensure!(
                !state.flags.is_engine_flag_set(&pending.engine_flag)?,
                "saved pending FLASH status bit is already active"
            );
        }
        if let Some(pending) = &state.script_runtime.pending_surf_field_move {
            anyhow::ensure!(
                pending.move_id == self.field_moves.surf.move_id
                    && pending.steps == 1
                    && matches!(pending.mode, MovementMode::Surf | MovementMode::SurfPika),
                "saved pending SURF outcome diverges from the compiled field-move rule"
            );
            let actor = state
                .storage
                .party
                .pokemon
                .get(pending.actor_party_index)
                .and_then(Option::as_ref)
                .with_context(|| {
                    format!(
                        "saved pending SURF actor party index {} is empty",
                        pending.actor_party_index
                    )
                })?;
            anyhow::ensure!(
                actor.species.id == pending.actor_species
                    && actor
                        .moves
                        .iter()
                        .any(|known| known.name == pending.move_id),
                "saved pending SURF actor no longer matches the prepared move"
            );
            let (map_name, tile, _, mode) = state
                .overworld
                .snapshot_identity()
                .context("saved pending SURF has no saved overworld")?;
            anyhow::ensure!(
                map_name == pending.map_name
                    && tile == pending.from_tile
                    && !matches!(mode, MovementMode::Surf | MovementMode::SurfPika),
                "saved pending SURF source position or movement mode has already changed"
            );
        }
        if let Some(pending) = &state.script_runtime.pending_waterfall_field_move {
            anyhow::ensure!(
                pending.move_id == self.field_moves.waterfall.move_id
                    && pending.steps > 0
                    && matches!(pending.mode, MovementMode::Surf | MovementMode::SurfPika)
                    && pending.from_tile.x == pending.to_tile.x
                    && pending.from_tile.y.checked_sub(pending.to_tile.y)
                        == i16::try_from(pending.steps).ok(),
                "saved pending WATERFALL outcome diverges from its remaining source climb"
            );
            let actor = state
                .storage
                .party
                .pokemon
                .get(pending.actor_party_index)
                .and_then(Option::as_ref)
                .with_context(|| {
                    format!(
                        "saved pending WATERFALL actor party index {} is empty",
                        pending.actor_party_index
                    )
                })?;
            anyhow::ensure!(
                actor.species.id == pending.actor_species
                    && actor
                        .moves
                        .iter()
                        .any(|known| known.name == pending.move_id),
                "saved pending WATERFALL actor no longer matches the prepared move"
            );
            let (map_name, tile, facing, mode) = state
                .overworld
                .snapshot_identity()
                .context("saved pending WATERFALL has no saved overworld")?;
            anyhow::ensure!(
                map_name == pending.map_name
                    && tile == pending.from_tile
                    && facing == Direction::Up
                    && mode == pending.mode,
                "saved pending WATERFALL source position, facing, or mode has changed"
            );
        }
        if let Some(pending) = &state.script_runtime.pending_field_travel {
            anyhow::ensure!(
                pending.actor_party_index.is_some() == pending.actor_species.is_some(),
                "saved pending field travel has incomplete actor identity"
            );
            let expected_move_id = match (
                pending.destination_spawn_identifier,
                pending.destination_warp_index,
                pending.flypoint_flag.as_ref(),
                pending.actor_party_index,
            ) {
                (Some(_), None, Some(_), Some(_)) => &self.field_moves.fly.move_id,
                (None, Some(_), None, Some(_)) => &self.field_moves.dig.move_id,
                (Some(_), None, None, Some(_)) => &self.field_moves.teleport.move_id,
                (None, Some(_), None, None) => &self.field_moves.escape_rope.item_id,
                _ => anyhow::bail!("saved pending field travel has an invalid destination shape"),
            };
            anyhow::ensure!(
                pending.move_id == *expected_move_id,
                "saved pending field travel diverges from the compiled move rule"
            );
            if let (Some(actor_party_index), Some(actor_species)) =
                (pending.actor_party_index, pending.actor_species.as_deref())
            {
                let actor = state
                    .storage
                    .party
                    .pokemon
                    .get(actor_party_index)
                    .and_then(Option::as_ref)
                    .with_context(|| {
                        format!(
                            "saved pending {} actor party index {} is empty",
                            pending.move_id, actor_party_index
                        )
                    })?;
                anyhow::ensure!(
                    actor.species.id == actor_species
                        && actor
                            .moves
                            .iter()
                            .any(|known| known.name == pending.move_id),
                    "saved pending {} actor no longer matches the prepared move",
                    pending.move_id
                );
            }
            let (map_name, _, _, _) = state
                .overworld
                .snapshot_identity()
                .context("saved pending field travel has no saved overworld")?;
            anyhow::ensure!(
                map_name == pending.source_map,
                "saved pending {} source map no longer matches the live map",
                pending.move_id
            );
            self.validate_runtime_map_tile(
                &format!("saved pending {} destination", pending.move_id),
                &pending.destination_map,
                pending.destination_tile,
            )?;
            if pending.flypoint_flag.is_some() {
                let flag = pending
                    .flypoint_flag
                    .as_deref()
                    .context("saved pending FLY has no flypoint flag")?;
                anyhow::ensure!(
                    state.flags.is_engine_flag_set(flag)?,
                    "saved pending FLY destination flag {flag} is not set"
                );
            }
            if let Some(spawn_identifier) = pending.destination_spawn_identifier {
                let spawn = self.runtime_spawn_point(spawn_identifier)?;
                anyhow::ensure!(
                    spawn.map_name == pending.destination_map
                        && runtime_spawn_expected_tile(spawn) == pending.destination_tile,
                    "saved pending {} spawn no longer matches its destination",
                    pending.move_id
                );
            }
        }
        let Some(pending) = &state.script_runtime.pending_block_field_move else {
            return Ok(());
        };
        let rule = match pending.move_id.as_str() {
            "CUT" => &self.field_moves.cut,
            "WHIRLPOOL" => &self.field_moves.whirlpool,
            move_id => anyhow::bail!("saved pending block field move has unknown move {move_id}"),
        };
        anyhow::ensure!(
            rule.move_id == pending.move_id,
            "saved pending block field move {} does not match compiled rule {}",
            pending.move_id,
            rule.move_id
        );
        let actor = state
            .storage
            .party
            .pokemon
            .get(pending.actor_party_index)
            .and_then(Option::as_ref)
            .with_context(|| {
                format!(
                    "saved pending {} actor party index {} is empty",
                    pending.move_id, pending.actor_party_index
                )
            })?;
        anyhow::ensure!(
            actor.species.id == pending.actor_species,
            "saved pending {} actor species {} does not match party species {}",
            pending.move_id,
            pending.actor_species,
            actor.species.id
        );
        anyhow::ensure!(
            actor
                .moves
                .iter()
                .any(|known| known.name == pending.move_id),
            "saved pending {} actor no longer knows the move",
            pending.move_id
        );
        let module = self
            .maps
            .get(&pending.map_name)
            .with_context(|| format!("saved pending {} map is missing", pending.map_name))?;
        let tileset_name = self.map_tileset_name(&pending.map_name)?;
        anyhow::ensure!(
            tileset_name == pending.tileset_name,
            "saved pending {} tileset {} does not match compiled map tileset {}",
            pending.move_id,
            pending.tileset_name,
            tileset_name
        );
        anyhow::ensure!(
            pending.metatile_x < module.attributes.width
                && pending.metatile_y < module.attributes.height,
            "saved pending {} metatile ({}, {}) is outside {} {}x{}",
            pending.move_id,
            pending.metatile_x,
            pending.metatile_y,
            pending.map_name,
            module.attributes.width,
            module.attributes.height
        );
        let index = usize::from(pending.metatile_y) * usize::from(module.attributes.width)
            + usize::from(pending.metatile_x);
        let current_block = state
            .map_block_overrides
            .get(&pending.map_name)
            .and_then(|overrides| overrides.get(&(pending.metatile_x, pending.metatile_y)))
            .copied()
            .or_else(|| module.blocks.get(index).copied())
            .with_context(|| {
                format!(
                    "saved pending {} block index {index} is absent from {}",
                    pending.move_id, pending.map_name
                )
            })?;
        anyhow::ensure!(
            current_block == pending.previous_block_id,
            "saved pending {} expected source block {:#04x}, found {current_block:#04x}",
            pending.move_id,
            pending.previous_block_id
        );
        let replacement = rule
            .replacements
            .get(&pending.tileset_name)
            .and_then(|blocks| blocks.get(&pending.previous_block_id))
            .with_context(|| {
                format!(
                    "saved pending {} source block {:#04x} has no compiled replacement on {}",
                    pending.move_id, pending.previous_block_id, pending.tileset_name
                )
            })?;
        anyhow::ensure!(
            replacement.replacement_block_id == pending.replacement_block_id
                && replacement.variant == pending.variant,
            "saved pending {} replacement diverges from the compiled field-move rule",
            pending.move_id
        );
        Ok(())
    }

    pub fn apply_strength_field_move(
        &self,
        state: &mut GameState,
        storage: &PokemonStorage,
        party_index: usize,
    ) -> Result<FieldMoveFlagOutcome> {
        core_apply_strength_field_move(&self.field_moves, state, storage, party_index)
            .map_err(anyhow::Error::new)
            .context("use STRENGTH field move")
    }

    pub fn queue_strength_from_menu(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        party_index: usize,
    ) -> Result<()> {
        self.require_no_active_battle(state, "STRENGTH field move")?;
        let storage = state.storage.clone();
        let mut validation_state = state.clone();
        self.apply_strength_field_move(&mut validation_state, &storage, party_index)?;
        state
            .script_runtime
            .memory
            .insert("wCurPartyMon".to_string(), party_index.to_string());
        commit_interaction_script_dispatch(
            state,
            &mut overworld.last_talked_object_identifier,
            &overworld.map.name,
            "Script_StrengthFromMenu",
            None,
        )
        .map_err(|error| anyhow::anyhow!("queue Script_StrengthFromMenu: {error:?}"))?;
        commit_overworld_snapshot(state, &overworld.snapshot(), SpawnMemoryUpdate::Preserve);
        Ok(())
    }

    pub fn apply_flash_field_move(
        &self,
        state: &mut GameState,
        storage: &PokemonStorage,
        party_index: usize,
    ) -> Result<FieldMoveFlagOutcome> {
        core_apply_flash_field_move(&self.field_moves, state, storage, party_index)
            .map_err(anyhow::Error::new)
            .context("use FLASH field move")
    }

    pub fn use_flash_field_move(
        &self,
        state: &mut GameState,
        source_map: &str,
        party_index: usize,
    ) -> Result<FieldMoveFlagOutcome> {
        self.require_no_active_battle(state, "FLASH field move")?;
        anyhow::ensure!(
            state.script_runtime.pending_flash_field_move.is_none(),
            "cannot prepare FLASH while another FLASH field move is pending"
        );
        let storage = state.storage.clone();
        let mut validation_state = state.clone();
        let outcome = self.apply_flash_field_move(&mut validation_state, &storage, party_index)?;
        anyhow::ensure!(
            !outcome.was_set && outcome.is_set,
            "cannot prepare FLASH because its source status bit is already active"
        );
        let map = self.map_module(source_map)?;
        apply_flash_map_effect(state, source_map, map.attributes.palette.as_deref())
            .map_err(anyhow::Error::new)
            .context("validate FLASH source map")?;
        state
            .script_runtime
            .memory
            .insert("wCurPartyMon".to_string(), party_index.to_string());
        state.script_runtime.pending_flash_field_move = Some(outcome.clone());
        Ok(outcome)
    }

    pub fn apply_surf_field_move(
        &self,
        state: &GameState,
        storage: &PokemonStorage,
        map: &OverworldMapData,
        tileset: &TilesetCollision,
        player: &mut PlayerMovementState,
        party_index: usize,
    ) -> Result<FieldMoveTravelOutcome> {
        core_apply_surf_field_move(
            &self.field_moves,
            state,
            storage,
            map,
            tileset,
            player,
            party_index,
        )
        .map_err(anyhow::Error::new)
        .context("use SURF field move")
    }

    pub fn use_surf_field_move(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        party_index: usize,
    ) -> Result<FieldMoveTravelOutcome> {
        let mut staged_state = state.clone();
        let mut staged_overworld = overworld.clone();
        self.require_no_active_battle(&staged_state, "SURF field move")?;
        anyhow::ensure!(
            staged_state
                .script_runtime
                .pending_surf_field_move
                .is_none(),
            "cannot prepare SURF while another SURF source transition is pending"
        );
        let target = Self::checked_runtime_field_move_target(
            "SURF",
            staged_overworld.player.tile,
            staged_overworld.player.facing,
        )?;
        if let Some((_, object)) = staged_overworld
            .visible_object_at_checked(target)
            .with_context(|| {
                format!(
                    "check SURF target occupancy on {}",
                    staged_overworld.map.name
                )
            })?
        {
            anyhow::bail!(
                "cannot use SURF field move onto occupied tile {target:?} by {:?}",
                object.object_identifier
            );
        }
        let storage = staged_state.storage.clone();
        let state_snapshot = staged_state.clone();
        let outcome = self.apply_surf_field_move(
            &state_snapshot,
            &storage,
            &staged_overworld.map,
            &staged_overworld.tileset,
            &mut staged_overworld.player,
            party_index,
        )?;
        staged_state
            .script_runtime
            .memory
            .insert("wCurPartyMon".to_string(), party_index.to_string());
        staged_state.script_runtime.memory.insert(
            "wSurfingPlayerState".to_string(),
            match outcome.mode {
                MovementMode::Surf => "4",
                MovementMode::SurfPika => "8",
                mode => anyhow::bail!("SURF validation produced non-surf movement mode {mode:?}"),
            }
            .to_string(),
        );
        staged_state.script_runtime.pending_surf_field_move = Some(outcome.clone());
        *state = staged_state;
        Ok(outcome)
    }

    pub fn apply_waterfall_field_move(
        &self,
        state: &GameState,
        storage: &PokemonStorage,
        map: &OverworldMapData,
        tileset: &TilesetCollision,
        player: &mut PlayerMovementState,
        party_index: usize,
    ) -> Result<FieldMoveTravelOutcome> {
        core_apply_waterfall_field_move(
            &self.field_moves,
            state,
            storage,
            map,
            tileset,
            player,
            party_index,
        )
        .map_err(anyhow::Error::new)
        .context("use WATERFALL field move")
    }

    pub fn use_waterfall_field_move(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        party_index: usize,
    ) -> Result<FieldMoveTravelOutcome> {
        let mut staged_state = state.clone();
        let mut staged_overworld = overworld.clone();
        self.require_no_active_battle(&staged_state, "WATERFALL field move")?;
        anyhow::ensure!(
            staged_state
                .script_runtime
                .pending_waterfall_field_move
                .is_none(),
            "cannot prepare WATERFALL while another source climb is pending"
        );
        let storage = staged_state.storage.clone();
        let state_snapshot = staged_state.clone();
        let outcome = self.apply_waterfall_field_move(
            &state_snapshot,
            &storage,
            &staged_overworld.map,
            &staged_overworld.tileset,
            &mut staged_overworld.player,
            party_index,
        )?;
        staged_state
            .script_runtime
            .memory
            .insert("wCurPartyMon".to_string(), party_index.to_string());
        staged_state.script_runtime.pending_waterfall_field_move = Some(outcome.clone());
        *state = staged_state;
        Ok(outcome)
    }

    pub fn validate_fly_field_move(
        &self,
        state: &GameState,
        source_map: &str,
        party_index: usize,
    ) -> Result<FieldMoveUseOutcome> {
        let source_environment = self.map_environment(source_map)?;
        if !is_fly_source_environment(source_environment) {
            anyhow::bail!("cannot use FLY field move in environment {source_environment}");
        }
        core_validate_fly_field_move(&self.field_moves, state, &state.storage, party_index)
            .map_err(anyhow::Error::new)
            .context("use FLY field move")
    }

    pub fn use_fly_field_move(
        &self,
        state: &GameState,
        source_map: &str,
        party_index: usize,
        destination_spawn_identifier: u16,
        flypoint_flag: &str,
    ) -> Result<FlyFieldMoveOutcome> {
        self.require_no_active_battle(state, "FLY field move")?;
        let fly_rule = self.validate_fly_field_move(state, source_map, party_index)?;
        let destination_rule = self
            .fly_destinations
            .get(flypoint_flag)
            .with_context(|| format!("FLY destination flag {flypoint_flag} is not defined"))?;
        anyhow::ensure!(
            destination_rule.flypoint_flag == flypoint_flag
                && destination_rule.destination_spawn_identifier == destination_spawn_identifier,
            "FLY request {flypoint_flag}/{destination_spawn_identifier} does not match compiled destination {}/{}",
            destination_rule.flypoint_flag,
            destination_rule.destination_spawn_identifier
        );
        if !state
            .flags
            .is_engine_flag_set(flypoint_flag)
            .with_context(|| format!("check FLY destination flag {flypoint_flag}"))?
        {
            anyhow::bail!("FLY destination flag {flypoint_flag} is not set");
        }
        let destination_spawn = self.runtime_spawn_point(destination_spawn_identifier)?;
        Ok(FlyFieldMoveOutcome {
            actor_party_index: fly_rule.actor_party_index,
            actor_species: fly_rule.actor_species,
            flypoint_flag: flypoint_flag.to_string(),
            source_map: source_map.to_string(),
            destination_spawn_identifier,
            destination_map: destination_spawn.map_name.clone(),
            destination_tile: runtime_spawn_expected_tile(destination_spawn),
        })
    }

    pub fn use_fly_field_move_in_session(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        party_index: usize,
        destination_spawn_identifier: u16,
        flypoint_flag: &str,
        _music_ids: &BTreeSet<String>,
    ) -> Result<FlyFieldMoveOutcome> {
        let mut staged_state = state.clone();
        let source_map = overworld.map.name.clone();
        anyhow::ensure!(
            staged_state.script_runtime.pending_field_travel.is_none(),
            "cannot prepare FLY while another field travel is pending"
        );
        let outcome = self.use_fly_field_move(
            &staged_state,
            &source_map,
            party_index,
            destination_spawn_identifier,
            flypoint_flag,
        )?;
        staged_state
            .script_runtime
            .memory
            .insert("wCurPartyMon".to_string(), party_index.to_string());
        staged_state.script_runtime.pending_field_travel = Some(PendingFieldTravel {
            move_id: self.field_moves.fly.move_id.clone(),
            actor_party_index: Some(outcome.actor_party_index),
            actor_species: Some(outcome.actor_species.clone()),
            source_map: outcome.source_map.clone(),
            destination_map: outcome.destination_map.clone(),
            destination_tile: outcome.destination_tile,
            destination_spawn_identifier: Some(destination_spawn_identifier),
            destination_warp_index: None,
            flypoint_flag: Some(flypoint_flag.to_string()),
        });
        *state = staged_state;
        Ok(outcome)
    }

    pub fn validate_dig_field_move(
        &self,
        state: &GameState,
        source_map: &str,
        party_index: usize,
    ) -> Result<FieldMoveUseOutcome> {
        let source_environment = self.map_environment(source_map)?;
        if !is_dig_field_move_environment(source_environment) {
            anyhow::bail!("cannot use DIG field move in environment {source_environment}");
        }
        core_validate_dig_field_move(&self.field_moves, &state.storage, party_index)
            .map_err(anyhow::Error::new)
            .context("use DIG field move")
    }

    pub fn use_dig_field_move(
        &self,
        state: &GameState,
        source_map: &str,
        party_index: usize,
    ) -> Result<DigFieldMoveOutcome> {
        self.require_no_active_battle(state, "DIG field move")?;
        let dig_rule = self.validate_dig_field_move(state, source_map, party_index)?;
        let destination = self.saved_dig_warp_destination(state, "DIG field move")?;
        Ok(DigFieldMoveOutcome {
            actor_party_index: dig_rule.actor_party_index,
            actor_species: dig_rule.actor_species,
            source_map: source_map.to_string(),
            destination_map: destination.map_name,
            destination_warp_index: destination.warp_index,
            destination_tile: destination.tile,
        })
    }

    pub fn use_dig_field_move_in_session(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        party_index: usize,
        _music_ids: &BTreeSet<String>,
    ) -> Result<DigFieldMoveOutcome> {
        let mut staged_state = state.clone();
        let source_map = overworld.map.name.clone();
        anyhow::ensure!(
            staged_state.script_runtime.pending_field_travel.is_none(),
            "cannot prepare DIG while another field travel is pending"
        );
        let outcome = self.use_dig_field_move(&staged_state, &source_map, party_index)?;
        staged_state
            .script_runtime
            .memory
            .insert("wCurPartyMon".to_string(), party_index.to_string());
        staged_state.script_runtime.pending_field_travel = Some(PendingFieldTravel {
            move_id: self.field_moves.dig.move_id.clone(),
            actor_party_index: Some(outcome.actor_party_index),
            actor_species: Some(outcome.actor_species.clone()),
            source_map: outcome.source_map.clone(),
            destination_map: outcome.destination_map.clone(),
            destination_tile: outcome.destination_tile,
            destination_spawn_identifier: None,
            destination_warp_index: Some(outcome.destination_warp_index),
            flypoint_flag: None,
        });
        *state = staged_state;
        Ok(outcome)
    }

    pub fn validate_teleport_field_move(
        &self,
        state: &GameState,
        source_map: &str,
        party_index: usize,
    ) -> Result<FieldMoveUseOutcome> {
        let source_environment = self.map_environment(source_map)?;
        if !is_teleport_source_environment(source_environment) {
            anyhow::bail!("cannot use TELEPORT field move in environment {source_environment}");
        }
        core_validate_teleport_field_move(&self.field_moves, &state.storage, party_index)
            .map_err(anyhow::Error::new)
            .context("use TELEPORT field move")
    }

    pub fn use_teleport_field_move(
        &self,
        state: &GameState,
        source_map: &str,
        party_index: usize,
    ) -> Result<TeleportFieldMoveOutcome> {
        self.require_no_active_battle(state, "TELEPORT field move")?;
        let teleport_rule = self.validate_teleport_field_move(state, source_map, party_index)?;
        let last_spawn_map_constant = state
            .last_spawn_map_constant
            .as_deref()
            .with_context(|| "TELEPORT field move has no saved spawn map")?;
        let destination_spawn_identifier = self
            .optional_runtime_spawn_identifier_for_map_constant(last_spawn_map_constant)?
            .with_context(|| {
                format!(
                    "TELEPORT saved spawn map {last_spawn_map_constant} is not in SpawnPoints"
                )
            })?;
        let destination_spawn = self.runtime_spawn_point(destination_spawn_identifier)?;
        Ok(TeleportFieldMoveOutcome {
            actor_party_index: teleport_rule.actor_party_index,
            actor_species: teleport_rule.actor_species,
            source_map: source_map.to_string(),
            destination_spawn_identifier,
            destination_map: destination_spawn.map_name.clone(),
            destination_tile: runtime_spawn_expected_tile(destination_spawn),
        })
    }

    pub fn use_teleport_field_move_in_session(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        party_index: usize,
        _music_ids: &BTreeSet<String>,
    ) -> Result<TeleportFieldMoveOutcome> {
        let mut staged_state = state.clone();
        let source_map = overworld.map.name.clone();
        anyhow::ensure!(
            staged_state.script_runtime.pending_field_travel.is_none(),
            "cannot prepare TELEPORT while another field travel is pending"
        );
        let outcome = self.use_teleport_field_move(&staged_state, &source_map, party_index)?;
        staged_state
            .script_runtime
            .memory
            .insert("wCurPartyMon".to_string(), party_index.to_string());
        staged_state.script_runtime.pending_field_travel = Some(PendingFieldTravel {
            move_id: self.field_moves.teleport.move_id.clone(),
            actor_party_index: Some(outcome.actor_party_index),
            actor_species: Some(outcome.actor_species.clone()),
            source_map: outcome.source_map.clone(),
            destination_map: outcome.destination_map.clone(),
            destination_tile: outcome.destination_tile,
            destination_spawn_identifier: Some(outcome.destination_spawn_identifier),
            destination_warp_index: None,
            flypoint_flag: None,
        });
        *state = staged_state;
        Ok(outcome)
    }

    pub fn commit_pending_field_travel(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        music_ids: &BTreeSet<String>,
    ) -> Result<PendingFieldTravel> {
        let mut staged_state = state.clone();
        let mut staged_overworld = overworld.clone();
        let pending = staged_state
            .script_runtime
            .pending_field_travel
            .clone()
            .context("no pending field travel to commit")?;
        anyhow::ensure!(
            staged_overworld.map.name == pending.source_map,
            "pending {} source map {} does not match live map {}",
            pending.move_id,
            pending.source_map,
            staged_overworld.map.name
        );
        let spawn_update = match (
            pending.destination_spawn_identifier,
            pending.destination_warp_index,
            pending.flypoint_flag.as_ref(),
        ) {
            (Some(_), None, Some(_)) | (Some(_), None, None) => SpawnMemoryUpdate::Preserve,
            (None, Some(_), None) => SpawnMemoryUpdate::Preserve,
            _ => anyhow::bail!("pending field travel has an invalid destination shape"),
        };
        self.transition_overworld_session(
            &mut staged_state,
            &mut staged_overworld,
            &pending.destination_map,
            pending.destination_tile,
            spawn_update,
            music_ids,
        )?;
        staged_state.script_runtime.pending_field_travel = None;
        *state = staged_state;
        *overworld = staged_overworld;
        Ok(pending)
    }

    pub fn validate_direct_field_move_actor(
        &self,
        state: &GameState,
        party_index: usize,
        move_id: &str,
    ) -> Result<FieldMoveUseOutcome> {
        core_validate_direct_field_move_actor(&state.storage, party_index, move_id)
            .map_err(anyhow::Error::new)
            .with_context(|| format!("use {move_id} field move"))
    }

    pub fn apply_active_battle_item_effect(
        &self,
        pokemon: &mut Pokemon,
        item_id: &str,
        consumed: bool,
    ) -> Result<BattleItemOutcome> {
        let item = self.item(item_id)?;
        if item.revive_hp_percent.is_some() {
            crate::nuzlocke::ensure_can_restore_hp(self.nuzlocke_rules, pokemon)?;
        }
        let outcome = core_apply_active_battle_item_effect(pokemon, item, consumed)
            .map_err(anyhow::Error::new)
            .with_context(|| format!("use item {item_id}"))?;
        if let Some(code) = item_happiness_change_code(item) {
            core_apply_happiness_change(pokemon, self.happiness_change(code)?);
        }
        Ok(outcome)
    }

    fn apply_player_active_battle_combat_item_effect(
        &self,
        combat: &mut BattleCombatState,
        item_id: &str,
        consumed: bool,
    ) -> Result<BattleItemOutcome> {
        let item = self.item(item_id)?;
        let outcome = apply_active_battle_item_effect_to_combat(
            combat,
            BattleSide::Player,
            item,
            consumed,
            &self.battle_stat_multipliers,
        )
        .map_err(|error| anyhow::anyhow!("{error:?}"))
        .with_context(|| format!("use item {item_id}"))?;
        if let Some(code) = item_happiness_change_code(item) {
            core_apply_happiness_change(
                &mut combat.player,
                self.happiness_change(code)?,
            );
        }
        Ok(outcome)
    }

    pub fn apply_battle_pp_item_effect(
        &self,
        pokemon: &mut Pokemon,
        item_id: &str,
        move_slot: Option<usize>,
        consumed: bool,
    ) -> Result<BattleItemOutcome> {
        let item = self.item(item_id)?;
        core_apply_battle_pp_item_effect(pokemon, item, &self.moves, move_slot, consumed)
            .map_err(anyhow::Error::new)
            .with_context(|| format!("use PP item {item_id}"))
    }

    pub fn use_bag_item_on_active_battle_pokemon(
        &self,
        state: &mut GameState,
        item_id: &str,
    ) -> Result<(ItemUseOutcome, BattleItemOutcome)> {
        let active_index =
            require_active_battle_party_index(state).map_err(|error| anyhow::anyhow!("{error}"))?;
        self.use_bag_item_on_battle_party_pokemon(state, item_id, active_index)
    }

    pub fn use_bag_item_on_battle_party_pokemon(
        &self,
        state: &mut GameState,
        item_id: &str,
        party_index: usize,
    ) -> Result<(ItemUseOutcome, BattleItemOutcome)> {
        let item = self.item(item_id)?;
        let active = state.battle_active_party_index == Some(party_index);
        anyhow::ensure!(
            item.battle_menu != "ITEMMENU_CLOSE" || active,
            "use battle item {item_id}: ITEMMENU_CLOSE requires the active battle Pokemon"
        );
        let x_accuracy = item.script_name == "X_ACCURACY";
        let mut combat = active_battle_combat_state(state)
            .with_context(|| format!("use battle item {item_id} in active battle"))?;
        let active_x_item = active
            && item.script_name != "X_ACCURACY"
            && item.battle_stat_boost_stat.is_some();
        if active && x_accuracy && combat.player_x_accuracy {
            anyhow::bail!("use battle item {item_id}: X Accuracy is already active");
        }
        if active {
            let mut combat_preview = combat.clone();
            self.apply_player_active_battle_combat_item_effect(
                &mut combat_preview,
                item_id,
                false,
            )?;
        }
        let mut preview = clone_active_battle_party_pokemon(state, party_index)
            .map_err(|error| anyhow::anyhow!("use battle item {item_id}: {error:?}"))?;
        self.apply_active_battle_item_effect(&mut preview, item_id, false)?;

        let item_use = self.use_bag_item(state, item_id, ItemUseContext::Battle)?;
        let pokemon = require_active_battle_party_pokemon_mut(state, party_index)
            .map_err(|error| anyhow::anyhow!("use battle item {item_id}: {error:?}"))?;
        // XItemEffect mutates wBattleMonStatLevels, not the selected party
        // structure. Happiness is mirrored separately by ChangeHappiness.
        let stored_stat_boosts_before = active_x_item.then(|| pokemon.stat_boosts.clone());
        let stored_item =
            self.apply_active_battle_item_effect(pokemon, item_id, item_use.consumed)?;
        if let Some(stat_boosts) = stored_stat_boosts_before {
            pokemon.stat_boosts = stat_boosts;
        }
        let battle_item = if active {
            self.apply_player_active_battle_combat_item_effect(
                &mut combat,
                item_id,
                item_use.consumed,
            )?
        } else {
            stored_item
        };
        state.sync_party_from_storage();
        let stored = state.storage.party.pokemon[party_index]
            .clone()
            .with_context(|| format!("battle party index {party_index} became empty"))?;
        let combat_party_slot = combat.player_party.get_mut(party_index).with_context(|| {
            format!("active combat party is missing battle item target index {party_index}")
        })?;
        *combat_party_slot = stored;
        if active && !item.status_heals.is_empty() {
            combat.player_toxic_turns = 0;
            combat.player_nightmare_source = None;
            combat.player_badge_before_status = true;
            crystal_core::battle::turn::recalculate_loaded_stats(
                &mut combat,
                BattleSide::Player,
                &self.battle_stat_multipliers,
            )
            .map_err(|error| {
                anyhow::anyhow!("recalculate player stats after battle item {item_id}: {error:?}")
            })?;
        }
        if active && x_accuracy {
            combat.player_x_accuracy = true;
        }
        state.script_runtime.active_battle_combat = Some(combat);
        Ok((item_use, battle_item))
    }

    pub fn use_bag_item_on_battle_party_move(
        &self,
        state: &mut GameState,
        item_id: &str,
        party_index: usize,
        move_slot: Option<usize>,
    ) -> Result<(ItemUseOutcome, BattleItemOutcome)> {
        let pp_up = self.item(item_id)?.script_name == "PP_UP";
        let active = state.battle_active_party_index == Some(party_index);
        let mut combat = active_battle_combat_state(state)
            .with_context(|| format!("use battle PP item {item_id} in active battle"))?;
        let mut preview = clone_active_battle_party_pokemon(state, party_index)
            .map_err(|error| anyhow::anyhow!("use battle item {item_id}: {error:?}"))?;
        self.apply_battle_pp_item_effect(&mut preview, item_id, move_slot, false)?;

        let item_use = self.use_bag_item(state, item_id, ItemUseContext::Battle)?;
        let pokemon = require_active_battle_party_pokemon_mut(state, party_index)
            .map_err(|error| anyhow::anyhow!("use battle item {item_id}: {error:?}"))?;
        let battle_item =
            self.apply_battle_pp_item_effect(pokemon, item_id, move_slot, item_use.consumed)?;
        state.sync_party_from_storage();
        let stored = state.storage.party.pokemon[party_index]
            .clone()
            .with_context(|| format!("battle party index {party_index} became empty"))?;
        let combat_party_slot = combat.player_party.get_mut(party_index).with_context(|| {
            format!("active combat party is missing battle PP item target index {party_index}")
        })?;
        *combat_party_slot = stored.clone();
        if active && !pp_up && combat.player_transform.is_none() {
            for (battle_move, stored_move) in combat.player.moves.iter_mut().zip(&stored.moves) {
                if battle_move.name == stored_move.name {
                    battle_move.current_pp = stored_move.current_pp;
                    battle_move.pp_ups = stored_move.pp_ups;
                }
            }
        }
        state.script_runtime.active_battle_combat = Some(combat);
        Ok((item_use, battle_item))
    }

    pub fn apply_party_wide_item_effect(
        &self,
        party: &mut Party,
        item_id: &str,
        consumed: bool,
    ) -> Result<PartyItemOutcome> {
        let item = self.item(item_id)?;
        if item.party_revive_hp_percent.is_some() && self.nuzlocke_rules.permadeath {
            anyhow::ensure!(
                !party
                    .pokemon
                    .iter()
                    .flatten()
                    .any(|pokemon| pokemon.hp == 0),
                "Nuzlocke permadeath prevents reviving fainted Pokemon"
            );
        }
        core_apply_party_wide_item_effect(party, item, &self.moves, consumed)
            .map_err(anyhow::Error::new)
            .with_context(|| format!("use whole-party item {item_id}"))
    }

    pub fn apply_party_pokemon_item_effect(
        &self,
        pokemon: &mut Pokemon,
        item_id: &str,
        level_up_happiness: BattleLevelUpHappinessContext,
        time_of_day: TimeOfDay,
        consumed: bool,
    ) -> Result<BattleItemOutcome> {
        let item = self.item(item_id)?;
        if item.revive_hp_percent.is_some() {
            crate::nuzlocke::ensure_can_restore_hp(self.nuzlocke_rules, pokemon)?;
        }
        let mut outcome = if item.rare_candy_level_gain.is_some()
            || self.evolutions.contains_item_evolution(&item.script_name)
        {
            core_apply_party_special_item_effect(
                pokemon,
                item,
                &self.pokemon,
                &self.moves,
                &self.learnsets,
                &self.growth_rates,
                &self.battle_reward_rules,
                &self.evolutions,
                level_up_happiness,
                time_of_day,
                consumed,
            )
        } else {
            core_apply_active_battle_item_effect(pokemon, item, consumed)
        }
        .map_err(anyhow::Error::new)
        .with_context(|| format!("use party item {item_id}"))?;
        if let Some(code) = item_happiness_change_code(item) {
            core_apply_happiness_change(pokemon, self.happiness_change(code)?);
        }
        if item.revive_hp_percent.is_some() && outcome.hp_before == 0 && outcome.hp_after > 0 {
            pokemon.status = None;
            pokemon.sleep_turns = 0;
            outcome.status_after = None;
        }
        Ok(outcome)
    }

    pub fn teach_tmhm_move(
        &self,
        pokemon: &mut Pokemon,
        item_id: &str,
        replace_slot: Option<usize>,
        consumed: bool,
    ) -> Result<TmHmLearnOutcome> {
        let item = self.item(item_id)?;
        let hm_moves = self
            .items
            .values()
            .filter(|candidate| !candidate.consumable)
            .filter_map(|candidate| candidate.tmhm_move.clone())
            .collect();
        let outcome = core_teach_tmhm_move(
            pokemon,
            item,
            &self.moves,
            &hm_moves,
            replace_slot,
            consumed,
        )
        .map_err(anyhow::Error::new)
        .with_context(|| format!("use TM/HM {item_id}"))?;
        if item.consumable {
            core_apply_happiness_change(
                pokemon,
                self.happiness_change("HAPPINESS_LEARNMOVE")?,
            );
        }
        Ok(outcome)
    }

    pub fn use_bag_item_on_party_pokemon(
        &self,
        state: &mut GameState,
        item_id: &str,
        party_index: usize,
        time_of_day: TimeOfDay,
    ) -> Result<(ItemUseOutcome, BattleItemOutcome)> {
        let level_up_happiness = self.level_up_happiness_context(state)?;
        let mut preview = clone_field_party_pokemon(state, party_index)
            .map_err(|error| anyhow::anyhow!("use party item {item_id}: {error:?}"))?;
        if preview.is_egg {
            anyhow::bail!("use party item {item_id}: Eggs can't use that");
        }
        let preview_effect =
            self.apply_party_pokemon_item_effect(
                &mut preview,
                item_id,
                level_up_happiness,
                time_of_day,
                false,
            )?;
        self.require_no_existing_pending_move_learn_for_item_effect(
            state,
            party_index,
            &preview_effect,
        )?;

        let item_use = self.use_bag_item(state, item_id, ItemUseContext::Field)?;
        let pokemon = require_field_party_pokemon_mut(state, party_index)
            .map_err(|error| anyhow::anyhow!("use party item {item_id}: {error:?}"))?;
        let item_effect =
            self.apply_party_pokemon_item_effect(
                pokemon,
                item_id,
                level_up_happiness,
                time_of_day,
                item_use.consumed,
            )?;
        self.queue_item_pending_move_learn(state, party_index, &item_effect)
            .with_context(|| format!("queue pending move learn for party item {item_id}"))?;
        if item_effect.evolution_target.is_some()
            && item_effect.evolution_cancel_snapshot.is_none()
        {
            let evolved_pokemon = state.storage.party.pokemon[party_index]
                .as_ref()
                .context("evolved party Pokemon disappeared")?
                .clone();
            state.pokedex.record_caught_pokemon(&evolved_pokemon);
        }
        state.sync_party_from_storage();
        Ok((item_use, item_effect))
    }

    fn require_no_existing_pending_move_learn_for_item_effect(
        &self,
        state: &GameState,
        party_index: usize,
        item_effect: &BattleItemOutcome,
    ) -> Result<()> {
        if !item_effect.pending_move_learns.is_empty() && state.pending_move_learn.is_some() {
            anyhow::bail!("pending move learn already exists for party index {party_index}");
        }
        Ok(())
    }

    pub fn replace_pending_move_learn(
        &self,
        state: &mut GameState,
        move_slot: usize,
    ) -> Result<PendingMoveLearnRuntimeResolution> {
        let pending = state
            .pending_move_learn
            .as_ref()
            .ok_or_else(|| anyhow::Error::new(BattleRewardError::MissingPendingMoveLearn))?;
        let pokemon = state
            .storage
            .party
            .pokemon
            .get(pending.party_index)
            .and_then(|pokemon| pokemon.as_ref())
            .ok_or_else(|| {
                anyhow::Error::new(BattleRewardError::PendingMoveLearnEmptyPartySlot {
                    party_index: pending.party_index,
                })
            })?;
        if let Some(learned) = pokemon.moves.get(move_slot) {
            let is_hm = self.items.values().any(|item| {
                !item.consumable && item.tmhm_move.as_deref() == Some(learned.name.as_str())
            });
            if is_hm {
                return Err(anyhow::Error::new(BattleRewardError::CannotForgetHmMove {
                    move_id: learned.name.clone(),
                }))
                .context("replace pending move learn");
            }
        }
        let resolution = core_replace_pending_move_learn(state, move_slot)
            .map_err(anyhow::Error::new)
            .context("replace pending move learn")?;
        let deferred_evolution = self.resolve_deferred_evolution_after_pending_move_learn(
            state,
            &resolution,
            "replacement",
        )?;
        promote_next_pending_move_learn(state);
        Ok(PendingMoveLearnRuntimeResolution {
            resolution,
            deferred_evolution,
        })
    }

    pub fn decline_pending_move_learn(
        &self,
        state: &mut GameState,
    ) -> Result<PendingMoveLearnRuntimeResolution> {
        let resolution = core_decline_pending_move_learn(state)
            .map_err(|error| anyhow::anyhow!("decline pending move learn: {error:?}"))?;
        let deferred_evolution = self.resolve_deferred_evolution_after_pending_move_learn(
            state,
            &resolution,
            "decline",
        )?;
        promote_next_pending_move_learn(state);
        Ok(PendingMoveLearnRuntimeResolution {
            resolution,
            deferred_evolution,
        })
    }

    fn resolve_deferred_evolution_after_pending_move_learn(
        &self,
        state: &mut GameState,
        resolution: &PendingMoveLearnResolution,
        action: &str,
    ) -> Result<Option<EvolutionReport>> {
        if !resolution.defer_level_evolution {
            return Ok(None);
        }
        if let (
            BattleMemory::Trainer { enemy_party, .. },
            Some(active_enemy_index),
        ) = (&state.battle, state.battle_active_enemy_party_index)
            && enemy_party
                .iter()
                .enumerate()
                .any(|(index, pokemon)| index != active_enemy_index && pokemon.hp > 0)
        {
            return Ok(None);
        }
        let time_of_day = state.time.time_of_day;
        self.resolve_deferred_level_evolution(state, resolution.party_index, time_of_day)
            .with_context(|| {
                format!("resolve deferred level evolution after pending move learn {action}")
            })
            .map(Some)
    }

    fn queue_item_pending_move_learn(
        &self,
        state: &mut GameState,
        party_index: usize,
        item_effect: &BattleItemOutcome,
    ) -> Result<()> {
        let Some(learned_move) = item_effect.pending_move_learns.first() else {
            return Ok(());
        };
        if state.pending_move_learn.is_some() {
            anyhow::bail!("pending move learn already exists for party index {party_index}");
        }
        let pokemon = state
            .storage
            .party
            .pokemon
            .get(party_index)
            .and_then(|slot| slot.as_ref())
            .with_context(|| format!("party index {party_index} is empty"))?;
        if pokemon.moves.len() < 4 {
            anyhow::bail!(
                "pending move learn requires full move list for party index {party_index}"
            );
        }
        if pokemon
            .moves
            .iter()
            .any(|known| known.name == learned_move.name)
        {
            return Ok(());
        }
        let species_id = pokemon.species.id.clone();
        let level = pokemon.level;
        state.pending_move_learn = Some(PendingMoveLearn {
            party_index,
            species_id,
            level,
            learned_move: learned_move.clone(),
            defer_level_evolution: item_effect.deferred_level_evolution,
        });
        Ok(())
    }

    pub fn resolve_deferred_level_evolution(
        &self,
        state: &mut GameState,
        party_index: usize,
        time_of_day: TimeOfDay,
    ) -> Result<EvolutionReport> {
        if state.pending_move_learn.is_some() {
            anyhow::bail!(
                "pending move learn already exists before resolving deferred level evolution for party index {party_index}"
            );
        }
        let context = EvolutionContext {
            species: &self.pokemon,
            moves: &self.moves,
            learnsets: &self.learnsets,
            time_of_day,
            current_item: None,
            force_evolution: false,
            link_mode: LinkMode::None,
        };
        let (report, pending_move_learn) = {
            let pokemon = require_field_party_pokemon_mut(state, party_index)
                .map_err(|error| anyhow::anyhow!("resolve deferred level evolution: {error:?}"))?;
            let report = check_and_evolve(pokemon, &self.evolutions, &context, true)
                .map_err(|error| anyhow::anyhow!("resolve deferred level evolution: {error:?}"))?;
            if report.target_species.is_none() {
                anyhow::bail!(
                    "deferred level evolution did not resolve for party index {party_index}"
                );
            }
            let pending_move_learn = if let Some(learned_move) = report.pending_move_learns.first()
            {
                if pokemon.moves.len() < 4 {
                    anyhow::bail!(
                        "pending evolution move learn requires full move list for party index {party_index}"
                    );
                }
                (!pokemon
                    .moves
                    .iter()
                    .any(|known| known.name == learned_move.name))
                .then(|| PendingMoveLearn {
                    party_index,
                    species_id: pokemon.species.id.clone(),
                    level: pokemon.level,
                    learned_move: learned_move.clone(),
                    defer_level_evolution: false,
                })
            } else {
                None
            };
            (report, pending_move_learn)
        };
        if let Some(pending_move_learn) = pending_move_learn {
            state.pending_move_learn = Some(pending_move_learn);
        }
        if report.cancel_snapshot.is_none() {
            let evolved_pokemon = state.storage.party.pokemon[party_index]
                .as_ref()
                .context("deferred evolved party Pokemon disappeared")?
                .clone();
            state.pokedex.record_caught_pokemon(&evolved_pokemon);
        }
        state.sync_party_from_storage();
        state.battle_evolvable_party_indices.remove(&party_index);
        rebase_pending_move_learns_for_party(state, party_index, true);
        sync_active_combat_player_party_from_storage(state);
        Ok(report)
    }

    pub fn use_bag_item_on_party_pokemon_now(
        &self,
        state: &mut GameState,
        item_id: &str,
        party_index: usize,
    ) -> Result<(ItemUseOutcome, BattleItemOutcome)> {
        let time_of_day = state.time.time_of_day;
        self.use_bag_item_on_party_pokemon(state, item_id, party_index, time_of_day)
    }

    pub fn use_bag_item_on_whole_party(
        &self,
        state: &mut GameState,
        item_id: &str,
    ) -> Result<(ItemUseOutcome, PartyItemOutcome)> {
        let mut preview = clone_field_party(state)
            .map_err(|error| anyhow::anyhow!("use whole-party item {item_id}: {error:?}"))?;
        self.apply_party_wide_item_effect(&mut preview, item_id, false)?;

        let item_use = self.use_bag_item(state, item_id, ItemUseContext::Field)?;
        let party = require_field_party_mut(state)
            .map_err(|error| anyhow::anyhow!("use whole-party item {item_id}: {error:?}"))?;
        let item_effect = self.apply_party_wide_item_effect(party, item_id, item_use.consumed)?;
        state.sync_party_from_storage();
        Ok((item_use, item_effect))
    }

    pub fn use_bag_pp_item_on_party_pokemon(
        &self,
        state: &mut GameState,
        item_id: &str,
        party_index: usize,
        move_slot: Option<usize>,
    ) -> Result<(ItemUseOutcome, BattleItemOutcome)> {
        let mut preview = clone_field_party_pokemon(state, party_index)
            .map_err(|error| anyhow::anyhow!("use party item {item_id}: {error:?}"))?;
        if preview.is_egg {
            anyhow::bail!("use party item {item_id}: Eggs can't use that");
        }
        self.apply_battle_pp_item_effect(&mut preview, item_id, move_slot, false)?;

        let item_use = self.use_bag_item(state, item_id, ItemUseContext::Field)?;
        let pokemon = require_field_party_pokemon_mut(state, party_index)
            .map_err(|error| anyhow::anyhow!("use party item {item_id}: {error:?}"))?;
        let item_effect =
            self.apply_battle_pp_item_effect(pokemon, item_id, move_slot, item_use.consumed)?;
        state.sync_party_from_storage();
        Ok((item_use, item_effect))
    }

    pub fn use_bag_tmhm_on_party_pokemon(
        &self,
        state: &mut GameState,
        item_id: &str,
        party_index: usize,
        replace_slot: Option<usize>,
    ) -> Result<(ItemUseOutcome, TmHmLearnOutcome)> {
        let mut preview = clone_field_party_pokemon(state, party_index)
            .map_err(|error| anyhow::anyhow!("use TM/HM {item_id}: {error:?}"))?;
        if preview.is_egg {
            anyhow::bail!("use TM/HM {item_id}: Eggs can't use that");
        }
        self.teach_tmhm_move(&mut preview, item_id, replace_slot, false)?;

        let item_use = self.use_bag_item(state, item_id, ItemUseContext::Field)?;
        let pokemon = require_field_party_pokemon_mut(state, party_index)
            .map_err(|error| anyhow::anyhow!("use TM/HM {item_id}: {error:?}"))?;
        let learned_move =
            self.teach_tmhm_move(pokemon, item_id, replace_slot, item_use.consumed)?;
        state.sync_party_from_storage();
        Ok((item_use, learned_move))
    }

    pub fn saved_item(&self, item_id: &str) -> Option<&Item> {
        self.items.get(item_id)
    }

    pub fn nuzlocke_rules(&self) -> NuzlockeRules {
        self.nuzlocke_rules
    }

    pub fn nuzlocke_encounter_was_used(&self, state: &GameState, map_name: &str) -> bool {
        crate::nuzlocke::encounter_was_used(self.nuzlocke_rules, state, map_name)
    }

    pub fn nuzlocke_run_is_over(&self, state: &GameState) -> bool {
        crate::nuzlocke::run_is_over(self.nuzlocke_rules, state)
    }

    pub fn saved_species_exists(&self, species: &str) -> bool {
        self.pokemon.contains_key(species)
    }

    pub fn saved_species(&self, species: &str) -> Option<PokemonSpecies> {
        self.pokemon.get(species).cloned()
    }

    pub fn saved_species_exact_exists(&self, species: &str) -> bool {
        self.pokemon
            .get(species)
            .is_some_and(|compiled| compiled.id == species)
    }

    pub fn saved_species_id(&self, species: &str) -> Option<String> {
        self.pokemon
            .get(species)
            .map(|compiled| compiled.id.clone())
    }

    pub fn saved_item_exists(&self, item_id: &str) -> bool {
        self.items.contains_key(item_id)
    }

    pub fn saved_item_script_name(&self, item_id: &str) -> Option<String> {
        self.items.get(item_id).map(|item| item.script_name.clone())
    }

    pub fn saved_move_name_and_pp(&self, move_name: &str) -> Option<(String, u8)> {
        self.moves
            .get(move_name)
            .map(|move_data| (move_data.name.clone(), move_data.pp))
    }

    pub fn ball_item(&self, ball_id: &str) -> Result<&Item> {
        self.items
            .get(ball_id)
            .with_context(|| format!("compiled game pack missing ball item {ball_id}"))
    }

    pub fn capture_ball_item(&self, ball_id: &str) -> Result<&Item> {
        let ball = self.ball_item(ball_id)?;
        validate_capture_ball_item(&self.capture_rules, ball).with_context(|| {
            format!("battle capture item {ball_id} is not declared by exact capture rules")
        })?;
        Ok(ball)
    }

    pub fn active_battle_capture_storage_full(&self, state: &GameState) -> Result<bool> {
        match &state.battle {
            BattleMemory::Wild { .. } | BattleMemory::StaticWild { .. } => state
                .storage
                .has_capture_space_in_box(state.current_pc_box)
                .map(|has_space| !has_space)
                .map_err(|error| anyhow::anyhow!("check current capture box: {error}")),
            BattleMemory::Trainer { .. } => Ok(false),
            BattleMemory::Inactive => {
                anyhow::bail!("cannot check capture storage without an active battle")
            }
        }
    }

    #[cfg(any(test, feature = "test-fixtures"))]
    pub fn throw_ball_from_bag(
        &self,
        bag: &mut Bag,
        ball_id: &str,
        player: &Pokemon,
        enemy: &Pokemon,
        context: CaptureAttemptContext,
        rng: &mut Random,
    ) -> Result<CaptureOutcome> {
        let ball = self.capture_ball_item(ball_id)?;
        if !ball.battle_usable {
            anyhow::bail!("battle capture item {ball_id} is not usable in battle");
        }
        core_throw_ball_from_bag(
            bag,
            ball,
            player,
            enemy,
            context,
            &self.capture_rules,
            &self.capture_wobble_probabilities,
            rng,
        )
        .map_err(|error| anyhow::anyhow!("throw ball {ball_id}: {error}"))?
        .with_context(|| format!("throw ball {ball_id} did not produce a capture outcome"))
    }

    pub fn throw_ball_at_active_battle(
        &self,
        state: &mut GameState,
        ball_id: &str,
    ) -> Result<CaptureOutcome> {
        let mut divider = RuntimeDividerSource::live();
        self.throw_ball_at_active_battle_with_divider(state, ball_id, &mut divider)
    }

    pub fn throw_ball_at_active_battle_with_divider<S>(
        &self,
        state: &mut GameState,
        ball_id: &str,
        divider: &mut S,
    ) -> Result<CaptureOutcome>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        let mut next_state = state.clone();
        let outcome =
            self.throw_ball_at_active_battle_with_divider_inner(&mut next_state, ball_id, divider)?;
        *state = next_state;
        Ok(outcome)
    }

    fn throw_ball_at_active_battle_with_divider_inner<S>(
        &self,
        state: &mut GameState,
        ball_id: &str,
        divider: &mut S,
    ) -> Result<CaptureOutcome>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        crate::nuzlocke::ensure_active_capture_allowed(self.nuzlocke_rules, state)?;
        let active_index =
            require_active_battle_party_index(state).map_err(|error| anyhow::anyhow!("{error}"))?;
        let player = state.storage.party.pokemon[active_index]
            .as_ref()
            .cloned()
            .with_context(|| format!("active battle party index {active_index} has no Pokemon"))?;
        let (enemy, context) = match &state.battle {
            BattleMemory::Wild {
                battle_type,
                enemy_pokemon,
                ..
            }
            | BattleMemory::StaticWild {
                battle_type,
                enemy_pokemon,
                ..
            } => {
                let mut context = CaptureAttemptContext::wild(ball_id);
                context.battle_type = battle_type.clone();
                (enemy_pokemon.clone(), context)
            }
            BattleMemory::Trainer {
                battle_type,
                enemy_pokemon,
                ..
            } => {
                let mut context = CaptureAttemptContext::wild(ball_id);
                context.battle_type = battle_type.clone();
                context.trainer_battle = true;
                (enemy_pokemon.clone(), context)
            }
            BattleMemory::Inactive => {
                anyhow::bail!("cannot throw a ball without an active battle");
            }
        };
        let mut rng = CrystalRandom::new(state.random_state, divider);
        let player_held_item = match player.item.as_deref() {
            Some(item_id) => Some(
                self.items
                    .get(item_id)
                    .with_context(|| format!("active Pokemon holds unknown item {item_id}"))?,
            ),
            None => None,
        };
        let outcome = if context.battle_type == "BATTLETYPE_TUTORIAL" {
            if ball_id != "POKE_BALL" {
                anyhow::bail!("catching tutorial requires POKE_BALL");
            }
            // ASM PokeBallEffect short-circuits the Dude tutorial to a
            // successful four-shake catch. Its bag is temporary WRAM, so the
            // player's carried balls are not consumed.
            CaptureOutcome {
                caught: true,
                blocked: false,
                storage_full: false,
                wobble_count: 3,
                animation_shakes: 4,
                final_catch_rate: 255,
                ball_id: Some("POKE_BALL".to_string()),
            }
        } else if context.battle_type == "BATTLETYPE_CONTEST" {
            if ball_id != "PARK_BALL" {
                anyhow::bail!("Bug-Catching Contest battles require PARK_BALL");
            }
            if state.bug_contest.park_balls_remaining == 0 {
                anyhow::bail!("no PARK_BALLs remain in the Bug-Catching Contest");
            }
            let mut outcome = core_resolve_capture_attempt_exact(
                &player,
                &enemy,
                player_held_item,
                &context,
                &self.capture_rules,
                &self.capture_wobble_probabilities,
                &mut rng,
            )
            .map_err(|error| anyhow::anyhow!("throw PARK_BALL: {error}"))?;
            state.bug_contest.park_balls_remaining -= 1;
            outcome.ball_id = Some("PARK_BALL".to_string());
            outcome
        } else if !context.trainer_battle
            && !state
                .storage
                .has_capture_space_in_box(state.current_pc_box)
                .map_err(|error| anyhow::anyhow!("check current capture box: {error}"))?
        {
            CaptureOutcome {
                caught: false,
                blocked: true,
                storage_full: true,
                wobble_count: 0,
                animation_shakes: 0,
                final_catch_rate: 0,
                ball_id: Some(ball_id.to_string()),
            }
        } else {
            let ball = self.capture_ball_item(ball_id)?;
            if !ball.battle_usable {
                anyhow::bail!("battle capture item {ball_id} is not usable in battle");
            }
            if !state
                .bag
                .consume_ball(ball)
                .map_err(|error| anyhow::anyhow!("throw ball {ball_id}: {error}"))?
            {
                anyhow::bail!("throw ball {ball_id} did not produce a capture outcome");
            }
            let mut outcome = core_resolve_capture_attempt_exact(
                &player,
                &enemy,
                player_held_item,
                &context,
                &self.capture_rules,
                &self.capture_wobble_probabilities,
                &mut rng,
            )
            .map_err(|error| anyhow::anyhow!("throw ball {ball_id}: {error}"))?;
            outcome.ball_id = Some(ball_id.to_string());
            outcome
        };
        if !outcome.storage_full {
            // BattleMenu_Pack returns to ParsePlayerAction for an actual ball
            // throw, whose non-move path clears Bide before the item effect.
            // A full Box rejects the selection before that action boundary.
            if let Some(combat) = state.script_runtime.active_battle_combat.as_mut() {
                combat.player_bide_turns = 0;
                combat.player_bide_damage = 0;
                combat.player_fury_cutter_chain = 0;
                combat.player_protect_counter = 0;
                combat.player_rage_active = false;
                combat.player_rage_counter = 0;
            }
        }
        state.random_state = rng.state();
        Ok(outcome)
    }

    pub fn complete_active_wild_capture<S>(
        &self,
        state: &mut GameState,
        outcome: &CaptureOutcome,
        nickname: Option<&str>,
        divider: &mut S,
    ) -> Result<CaptureCompletion>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        crate::nuzlocke::ensure_capture_nickname(self.nuzlocke_rules, nickname)?;
        let mut staged_state = state.clone();
        if let Some(nickname) = nickname {
            if nickname.is_empty()
                || nickname.trim() != nickname
                || nickname.chars().count() > 10
                || nickname.chars().any(char::is_control)
            {
                anyhow::bail!("captured Pokemon nickname must be exact and at most 10 characters");
            }
        }
        let caught_map_name = match &staged_state.battle {
            crystal_core::state::BattleMemory::Wild { map_name, .. } => Some(map_name.clone()),
            crystal_core::state::BattleMemory::StaticWild { .. } => match &staged_state.overworld {
                crystal_core::state::OverworldMemory::Active { map_name, .. } => {
                    Some(map_name.clone())
                }
                crystal_core::state::OverworldMemory::Inactive => None,
            },
            _ => None,
        };
        let battle_end = self.active_battle_end_context(&staged_state)?;
        let pay_day_money = self.active_battle_pay_day_payout(&staged_state);
        let transformed_replacement = if staged_state
            .script_runtime
            .active_battle_combat
            .as_ref()
            .is_some_and(|combat| combat.enemy_transform.is_some())
        {
            let enemy = match &staged_state.battle {
                crystal_core::state::BattleMemory::Wild { enemy_pokemon, .. }
                | crystal_core::state::BattleMemory::StaticWild { enemy_pokemon, .. } => {
                    enemy_pokemon
                }
                crystal_core::state::BattleMemory::Trainer { trainer_id, .. } => {
                    anyhow::bail!(
                        "cannot capture transformed enemy during trainer battle {trainer_id}"
                    )
                }
                crystal_core::state::BattleMemory::Inactive => {
                    anyhow::bail!("cannot materialize transformed capture without an active battle")
                }
            };
            Some(
                self.create_pokemon("DITTO", enemy.level, enemy.dvs)
                    .context("materialize DITTO for transformed capture")?,
            )
        } else {
            None
        };
        let mut completion =
            core_complete_active_wild_capture(&mut staged_state, outcome, transformed_replacement)
                .map_err(|error| anyhow::anyhow!("complete captured Pokemon: {error}"))?;
        if let (Some(nickname), Some(stored)) = (nickname, completion.stored.as_mut()) {
            stored.pokemon.nickname = nickname.to_string();
            match stored.location {
                crystal_core::models::CaptureStorageLocation::Party { slot } => {
                    let pokemon = staged_state.storage.party.pokemon[slot]
                        .as_mut()
                        .context("captured party destination is empty")?;
                    pokemon.nickname = nickname.to_string();
                }
                crystal_core::models::CaptureStorageLocation::Pc { box_index, slot } => {
                    let pc_box = staged_state
                        .storage
                        .pc_boxes
                        .get_mut(box_index)
                        .context("captured PC destination box is missing")?;
                    let mut pokemon = pc_box.pokemon[slot]
                        .clone()
                        .context("captured PC destination is empty")?;
                    pokemon.nickname = nickname.to_string();
                    pc_box.set_slot(slot, Some(pokemon));
                }
            }
            staged_state.sync_party_from_storage();
        }
        if let (Some(map_name), Some(stored)) = (caught_map_name, completion.stored.as_mut()) {
            if let Some(location) = self
                .saved_map_id(&map_name)
                .and_then(|map_id| map_id.parse::<u8>().ok())
            {
                if let Some(caught_data) = stored.pokemon.caught_data.as_mut() {
                    caught_data.location = location;
                }
                match stored.location {
                    crystal_core::models::CaptureStorageLocation::Party { slot } => {
                        if let Some(Some(pokemon)) =
                            staged_state.storage.party.pokemon.get_mut(slot)
                        {
                            if let Some(caught_data) = pokemon.caught_data.as_mut() {
                                caught_data.location = location;
                            }
                        }
                    }
                    crystal_core::models::CaptureStorageLocation::Pc { box_index, slot } => {
                        if let Some(Some(pokemon)) = staged_state
                            .storage
                            .pc_boxes
                            .get_mut(box_index)
                            .and_then(|pc_box| pc_box.pokemon.get_mut(slot))
                        {
                            if let Some(caught_data) = pokemon.caught_data.as_mut() {
                                caught_data.location = location;
                            }
                        }
                    }
                }
                staged_state.sync_party_from_storage();
            }
        }
        let result_code = staged_state.battle_result & 0x3f;
        anyhow::ensure!(
            result_code == 0,
            "successful capture ended with non-WIN battle result {:#04x}",
            staged_state.battle_result
        );
        // ExitBattle's base-WIN branch is CheckPayDay -> EvolveAfterBattle ->
        // GivePokerus. Capture cannot itself create a new level-evolution
        // candidate, so the middle call is a no-op here; the other two remain
        // authoritative and execute exactly once at this mutation boundary.
        self.claim_active_battle_pay_day_money(&mut staged_state, pay_day_money)?;
        staged_state
            .spread_pokerus_after_battle(divider)
            .map_err(|error| anyhow::anyhow!("post-capture Pokerus divider failed: {error}"))?;
        if completion.stored.is_some()
            && let Some((battle_type, roaming_slot, enemy, map_name)) = battle_end
        {
            self.finish_battle_roaming_update_exact(
                &mut staged_state,
                &battle_type,
                roaming_slot,
                &enemy,
                &map_name,
                divider,
            )?;
        }
        if let Some(terminal) = staged_state.pending_static_wild_terminal.as_mut() {
            terminal.win_cleanup_applied = true;
        }
        *state = staged_state;
        Ok(completion)
    }

    pub fn battle_escape_item_mode(&self, item_id: &str) -> Result<String> {
        let item = self.item(item_id)?;
        let mode = validate_battle_escape_item(item)
            .map_err(|error| anyhow::anyhow!("validate battle escape item {item_id}: {error:?}"))?;
        Ok(mode.to_string())
    }

    pub fn require_battle_escape_item_context(
        &self,
        state: &GameState,
        item_id: &str,
    ) -> Result<String> {
        let mode = self.battle_escape_item_mode(item_id)?;
        require_wild_battle_for_escape_item(state)
            .map_err(|error| anyhow::anyhow!("use battle escape item {item_id}: {error:?}"))?;
        Ok(mode)
    }

    pub fn apply_battle_escape_item_use(
        &self,
        state: &mut GameState,
        item_id: &str,
    ) -> Result<String> {
        let mut divider = RuntimeDividerSource::live();
        self.apply_battle_escape_item_use_with_divider(state, item_id, &mut divider)
    }

    pub fn apply_battle_escape_item_use_with_divider<S>(
        &self,
        state: &mut GameState,
        item_id: &str,
        divider: &mut S,
    ) -> Result<String>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        let mode = self.battle_escape_item_mode(item_id)?;
        let battle_end = self.active_battle_end_context(state)?;
        apply_battle_escape_item_use(state)
            .map_err(|error| anyhow::anyhow!("use battle escape item {item_id}: {error:?}"))?;
        if let Some((battle_type, roaming_slot, enemy, map_name)) = battle_end {
            self.finish_battle_roaming_update_exact(
                state,
                &battle_type,
                roaming_slot,
                &enemy,
                &map_name,
                divider,
            )?;
        }
        Ok(mode)
    }

    pub fn use_bag_item_to_escape_active_wild_battle(
        &self,
        state: &mut GameState,
        item_id: &str,
    ) -> Result<BattleEscapeItemUseOutcome> {
        let mut divider = RuntimeDividerSource::live();
        self.use_bag_item_to_escape_active_wild_battle_with_divider(state, item_id, &mut divider)
    }

    pub fn use_bag_item_to_escape_active_wild_battle_with_divider<S>(
        &self,
        state: &mut GameState,
        item_id: &str,
        divider: &mut S,
    ) -> Result<BattleEscapeItemUseOutcome>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        let mut staged_state = state.clone();
        let battle_escape_mode = self.require_battle_escape_item_context(&staged_state, item_id)?;
        let item_use = self.use_bag_item(&mut staged_state, item_id, ItemUseContext::Battle)?;
        self.apply_battle_escape_item_use_with_divider(&mut staged_state, item_id, divider)?;
        *state = staged_state;
        Ok(BattleEscapeItemUseOutcome {
            item_use,
            battle_escape_mode,
            escaped: true,
        })
    }

    pub fn require_battle_stat_drop_guard_item_context(
        &self,
        state: &GameState,
        item_id: &str,
    ) -> Result<()> {
        let item = self.item(item_id)?;
        validate_battle_stat_drop_guard_item(item).map_err(|error| {
            anyhow::anyhow!("validate battle stat drop guard item {item_id}: {error:?}")
        })?;
        let combat = active_battle_combat_state(state)
            .with_context(|| format!("use battle state item {item_id}"))?;
        anyhow::ensure!(
            !combat.player_mist_active,
            "use battle state item {item_id}: stat drop guard is already active"
        );
        Ok(())
    }

    pub fn use_bag_guard_spec_in_active_battle(
        &self,
        state: &mut GameState,
        item_id: &str,
    ) -> Result<BattleStateItemUseOutcome> {
        let mut staged_state = state.clone();
        self.require_battle_stat_drop_guard_item_context(&staged_state, item_id)?;
        let item_use = self.use_bag_item(&mut staged_state, item_id, ItemUseContext::Battle)?;
        let mut combat = active_battle_combat_state(&staged_state)
            .with_context(|| format!("use battle state item {item_id}"))?;
        let mist_active_before = combat.player_mist_active;
        combat.player_mist_active = true;
        staged_state.script_runtime.active_battle_combat = Some(combat);
        *state = staged_state;
        Ok(BattleStateItemUseOutcome {
            item_use,
            mist_active_before,
            mist_active_after: true,
        })
    }

    pub fn advance_active_trainer_battle(
        &self,
        state: &mut GameState,
    ) -> Result<TrainerBattleAdvanceOutcome> {
        core_advance_active_trainer_battle(state)
            .map_err(|error| anyhow::anyhow!("advance active trainer battle: {error}"))
    }

    pub fn resolve_active_battle_turn(
        &self,
        state: &mut GameState,
        player_action: BattleAction,
        enemy_action: BattleAction,
    ) -> Result<BattleTurnOutcome> {
        let mut divider = RuntimeDividerSource::live();
        self.resolve_active_battle_turn_with_divider(
            state,
            player_action,
            enemy_action,
            &mut divider,
        )
    }

    pub fn resolve_active_battle_turn_with_divider<S>(
        &self,
        state: &mut GameState,
        player_action: BattleAction,
        enemy_action: BattleAction,
        divider: &mut S,
    ) -> Result<BattleTurnOutcome>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        let mut staged_state = state.clone();
        let battle_end = self.active_battle_end_context(&staged_state)?;
        let mut rng = ExactBattleRandom::new(staged_state.random_state, divider);
        let outcome = self.resolve_active_battle_turn_with_rng(
            &mut staged_state,
            player_action,
            enemy_action,
            &mut rng,
            None,
        )?;
        if let Some(error) = rng.divider_error() {
            anyhow::bail!("resolve exact active battle turn: {error}");
        }
        staged_state.random_state = rng.state();
        drop(rng);
        if matches!(staged_state.battle, BattleMemory::Inactive)
            && let Some((battle_type, roaming_slot, _, map_name)) = battle_end
        {
            self.finish_battle_roaming_update_exact(
                &mut staged_state,
                &battle_type,
                roaming_slot,
                &outcome.state.enemy,
                &map_name,
                divider,
            )?;
        }
        *state = staged_state;
        Ok(outcome)
    }

    pub fn resolve_active_battle_turn_with_enemy_ai_actions_with_divider<S>(
        &self,
        state: &mut GameState,
        player_action: BattleAction,
        divider: &mut S,
        select_enemy_move: &mut EnemyMoveSelector<'_>,
        select_enemy_action: &mut EnemyPostOrderActionSelector<'_>,
    ) -> Result<BattleTurnOutcome>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        let mut staged_state = state.clone();
        let battle_end = self.active_battle_end_context(&staged_state)?;
        let mut rng = ExactBattleRandom::new(staged_state.random_state, divider);
        let outcome = self.resolve_active_battle_turn_with_rng(
            &mut staged_state,
            player_action,
            BattleAction::Move { slot: 0 },
            &mut rng,
            Some((select_enemy_move, select_enemy_action)),
        )?;
        if let Some(error) = rng.divider_error() {
            anyhow::bail!("resolve exact active battle turn with trainer action: {error}");
        }
        staged_state.random_state = rng.state();
        drop(rng);
        if matches!(staged_state.battle, BattleMemory::Inactive)
            && let Some((battle_type, roaming_slot, _, map_name)) = battle_end
        {
            self.finish_battle_roaming_update_exact(
                &mut staged_state,
                &battle_type,
                roaming_slot,
                &outcome.state.enemy,
                &map_name,
                divider,
            )?;
        }
        *state = staged_state;
        Ok(outcome)
    }

    pub fn resolve_active_battle_ball_turn_with_enemy_ai_actions_with_divider<S>(
        &self,
        state: &mut GameState,
        ball_id: &str,
        enemy_action: BattleAction,
        divider: &mut S,
        enemy_ai_actions: Option<(
            &mut EnemyMoveSelector<'_>,
            &mut EnemyPostOrderActionSelector<'_>,
        )>,
    ) -> Result<(CaptureOutcome, BattleTurnOutcome)>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        crate::nuzlocke::ensure_active_capture_allowed(self.nuzlocke_rules, state)?;
        let ball = self.capture_ball_item(ball_id)?;
        if !ball.battle_usable {
            anyhow::bail!("battle capture item {ball_id} is not usable in battle");
        }
        let mut staged_state = state.clone();
        let battle_end = self.active_battle_end_context(&staged_state)?;
        let active_index = require_active_battle_party_index(&staged_state)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        let active_enemy_index = require_active_battle_enemy_party_index(&staged_state)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        let player = staged_state.storage.party.pokemon[active_index]
            .as_ref()
            .cloned()
            .with_context(|| format!("active battle party index {active_index} has no Pokemon"))?;
        let player_party = Self::active_battle_player_party(&staged_state)?;
        let (enemy, enemy_party, mut context) = match &staged_state.battle {
            BattleMemory::Wild {
                battle_type,
                enemy_pokemon,
                enemy_party,
                ..
            }
            | BattleMemory::StaticWild {
                battle_type,
                enemy_pokemon,
                enemy_party,
                ..
            } => {
                let mut context = CaptureAttemptContext::wild(ball_id);
                context.battle_type = battle_type.clone();
                (enemy_pokemon.clone(), enemy_party.clone(), context)
            }
            BattleMemory::Trainer {
                battle_type,
                enemy_pokemon,
                enemy_party,
                ..
            } => {
                let mut context = CaptureAttemptContext::wild(ball_id);
                context.battle_type = battle_type.clone();
                context.trainer_battle = true;
                (enemy_pokemon.clone(), enemy_party.clone(), context)
            }
            BattleMemory::Inactive => {
                anyhow::bail!("cannot throw a ball without an active battle");
            }
        };
        Self::require_active_enemy_in_battle_party(&enemy_party, active_enemy_index)?;
        if !context.trainer_battle
            && !staged_state
                .storage
                .has_capture_space_in_box(staged_state.current_pc_box)
                .map_err(|error| anyhow::anyhow!("check current capture box: {error}"))?
        {
            anyhow::bail!("capture storage is full; Ball selection does not spend a turn");
        }

        let badge_boosts_enabled = staged_state.link_session.link_mode == 0
            && active_battle_type(&staged_state) != Some("BATTLETYPE_BATTLE_TOWER");
        let mut combat = staged_state
            .script_runtime
            .active_battle_combat
            .clone()
            .unwrap_or_else(|| {
                BattleCombatState::new(player, enemy)
                    .with_parties(player_party, enemy_party.to_vec())
                    .with_party_indices(active_index, active_enemy_index)
                    .with_obedience(staged_state.player_id, staged_state.badges.johto)
                    .with_kanto_badges(staged_state.badges.kanto)
                    .with_link_context(
                        staged_state.time.time_of_day,
                        staged_state.link_session.link_mode,
                        staged_state.link_session.serial_connection_status,
                    )
                    .with_badge_boosts_enabled(badge_boosts_enabled)
            });
        combat.link_battle = staged_state.link_session.link_mode != 0;
        combat.link_colosseum = staged_state.link_session.link_mode == LINK_MODE_COLOSSEUM;
        combat.serial_connection_status = staged_state.link_session.serial_connection_status;
        combat.obedience_badges = staged_state.badges.johto;
        combat.kanto_badges = staged_state.badges.kanto;
        combat.badge_boosts_enabled = badge_boosts_enabled;

        let player_held_item = match combat.player.item.as_deref() {
            Some(item_id) => Some(
                self.items
                    .get(item_id)
                    .with_context(|| format!("active Pokemon holds unknown item {item_id}"))?,
            ),
            None => None,
        };
        context.ball_id = ball_id.to_string();
        let tutorial = context.battle_type == "BATTLETYPE_TUTORIAL";
        let contest = context.battle_type == "BATTLETYPE_CONTEST";
        if tutorial && ball_id != "POKE_BALL" {
            anyhow::bail!("catching tutorial requires POKE_BALL");
        }
        if contest && ball_id != "PARK_BALL" {
            anyhow::bail!("Bug-Catching Contest battles require PARK_BALL");
        }
        if contest && staged_state.bug_contest.park_balls_remaining == 0 {
            anyhow::bail!("no PARK_BALLs remain in the Bug-Catching Contest");
        }

        let mut capture = None;
        let mut execute_ball = |combat: &mut BattleCombatState,
                                action_ball_id: &str,
                                rng: &mut dyn BattleRandomSource,
                                events: &mut Vec<BattleEvent>|
         -> std::result::Result<(), BattleTurnError> {
            if action_ball_id != ball_id {
                return Err(BattleTurnError::BattleItem {
                    side: BattleSide::Player,
                    item_id: action_ball_id.to_string(),
                    error: format!("expected selected ball {ball_id}"),
                });
            }
            let mut outcome = if tutorial {
                CaptureOutcome {
                    caught: true,
                    blocked: false,
                    storage_full: false,
                    wobble_count: 3,
                    animation_shakes: 4,
                    final_catch_rate: 255,
                    ball_id: None,
                }
            } else {
                if contest {
                    staged_state.bug_contest.park_balls_remaining -= 1;
                } else if !staged_state.bag.consume_ball(ball).map_err(|error| {
                    BattleTurnError::BattleItem {
                        side: BattleSide::Player,
                        item_id: ball_id.to_string(),
                        error,
                    }
                })? {
                    return Err(BattleTurnError::BattleItem {
                        side: BattleSide::Player,
                        item_id: ball_id.to_string(),
                        error: "ball is absent from the Bag".to_string(),
                    });
                }
                core_resolve_capture_attempt_with_battle_rng(
                    &combat.player,
                    &combat.enemy,
                    player_held_item,
                    &context,
                    &self.capture_rules,
                    &self.capture_wobble_probabilities,
                    rng,
                )
                .map_err(|error| BattleTurnError::BattleItem {
                    side: BattleSide::Player,
                    item_id: ball_id.to_string(),
                    error: format!("{error:?}"),
                })?
            };
            outcome.ball_id = Some(ball_id.to_string());
            capture = Some(outcome.clone());
            events.push(BattleEvent::BallThrown {
                side: BattleSide::Player,
                outcome,
            });
            Ok(())
        };
        let mut rng = ExactBattleRandom::new(staged_state.random_state, divider);
        let turn = core_resolve_battle_turn_with_ball_action_and_enemy_ai_actions(
            combat,
            BattleAction::Ball {
                item_id: ball_id.to_string(),
            },
            enemy_action,
            &self.moves,
            &self.items,
            &self.move_priorities,
            &self.battle_stat_multipliers,
            &self.type_categories,
            &self.type_effectiveness,
            &self.weather_modifiers,
            &mut rng,
            !context.trainer_battle,
            enemy_ai_actions,
            &mut execute_ball,
        )
        .map_err(|error| anyhow::anyhow!("resolve active battle Ball turn: {error:?}"))?;
        drop(execute_ball);
        if let Some(error) = rng.divider_error() {
            anyhow::bail!("resolve exact active battle Ball turn: {error}");
        }
        staged_state.random_state = rng.state();
        drop(rng);
        let capture = capture.context("Ball action did not produce a capture outcome")?;
        commit_battle_turn_outcome(&mut staged_state, active_index, &turn)
            .map_err(|error| anyhow::anyhow!("commit active battle Ball turn: {error:?}"))?;
        if matches!(staged_state.battle, BattleMemory::Inactive)
            && let Some((battle_type, roaming_slot, _, map_name)) = battle_end
        {
            self.finish_battle_roaming_update_exact(
                &mut staged_state, &battle_type, roaming_slot,
                &turn.state.enemy, &map_name, divider,
            )?;
        }
        *state = staged_state;
        Ok((capture, turn))
    }

    fn resolve_active_battle_turn_with_rng(
        &self,
        state: &mut GameState,
        player_action: BattleAction,
        enemy_action: BattleAction,
        rng: &mut dyn BattleRandomSource,
        enemy_ai_actions: Option<(
            &mut EnemyMoveSelector<'_>,
            &mut EnemyPostOrderActionSelector<'_>,
        )>,
    ) -> Result<BattleTurnOutcome> {
        let active_index =
            require_active_battle_party_index(state).map_err(|error| anyhow::anyhow!("{error}"))?;
        let player = state.storage.party.pokemon[active_index]
            .as_ref()
            .cloned()
            .with_context(|| format!("active battle party index {active_index} has no Pokemon"))?;
        let player_party = Self::active_battle_player_party(state)?;
        let active_enemy_index = require_active_battle_enemy_party_index(state)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        let (enemy, enemy_party, active_enemy_index, is_wild_battle) = match &state.battle {
            BattleMemory::Wild {
                enemy_pokemon,
                enemy_party,
                ..
            }
            | BattleMemory::StaticWild {
                enemy_pokemon,
                enemy_party,
                ..
            } => (
                enemy_pokemon.clone(),
                enemy_party.clone(),
                active_enemy_index,
                true,
            ),
            BattleMemory::Trainer {
                enemy_pokemon,
                enemy_party,
                ..
            } => (
                enemy_pokemon.clone(),
                enemy_party.clone(),
                active_enemy_index,
                false,
            ),
            BattleMemory::Inactive => {
                anyhow::bail!("cannot resolve battle turn without an active battle");
            }
        };
        Self::require_active_enemy_in_battle_party(&enemy_party, active_enemy_index)?;
        let badge_boosts_enabled = state.link_session.link_mode == 0
            && active_battle_type(state) != Some("BATTLETYPE_BATTLE_TOWER");
        let mut combat = state
            .script_runtime
            .active_battle_combat
            .clone()
            .unwrap_or_else(|| {
                BattleCombatState::new(player, enemy)
                    .with_parties(player_party, enemy_party.to_vec())
                    .with_party_indices(active_index, active_enemy_index)
                    .with_obedience(state.player_id, state.badges.johto)
                    .with_kanto_badges(state.badges.kanto)
                    .with_link_context(
                        state.time.time_of_day,
                        state.link_session.link_mode,
                        state.link_session.serial_connection_status,
                    )
                    .with_badge_boosts_enabled(badge_boosts_enabled)
            });
        combat.link_battle = state.link_session.link_mode != 0;
        combat.link_colosseum = state.link_session.link_mode == LINK_MODE_COLOSSEUM;
        combat.serial_connection_status = state.link_session.serial_connection_status;
        combat.obedience_badges = state.badges.johto;
        combat.kanto_badges = state.badges.kanto;
        combat.badge_boosts_enabled = badge_boosts_enabled;
        combat.amulet_coin_active = state.battle_amulet_coin_active;
        if matches!(player_action, BattleAction::Run)
            && active_battle_type(state).is_some_and(battle_type_blocks_escape)
        {
            combat.force_switch_blocked = true;
        }
        self.apply_player_item_happiness_before_battle_turn(&mut combat, &player_action)?;
        let input = BattleTurnInput {
            player: player_action,
            enemy: enemy_action,
        };
        let outcome = if is_wild_battle {
            if let Some((select_enemy_move, select_enemy_action)) = enemy_ai_actions {
                core_resolve_wild_battle_turn_with_enemy_ai_actions(
                    combat,
                    input.player,
                    &self.moves,
                    &self.items,
                    &self.move_priorities,
                    &self.battle_stat_multipliers,
                    &self.type_categories,
                    &self.type_effectiveness,
                    &self.weather_modifiers,
                    &self.battle_escape_rules,
                    state.battle_escape_attempts,
                    rng,
                    select_enemy_move,
                    select_enemy_action,
                )
                .map_err(|error| {
                    anyhow::anyhow!("resolve active wild battle turn with AI: {error:?}")
                })?
            } else {
                self.resolve_wild_battle_turn_with_items(
                    combat,
                    input,
                    state.battle_escape_attempts,
                    rng,
                )?
            }
        } else if let Some((select_enemy_move, select_enemy_action)) = enemy_ai_actions {
            core_resolve_battle_turn_with_enemy_ai_actions(
                combat,
                input.player,
                &self.moves,
                &self.items,
                &self.move_priorities,
                &self.battle_stat_multipliers,
                &self.type_categories,
                &self.type_effectiveness,
                &self.weather_modifiers,
                rng,
                select_enemy_move,
                select_enemy_action,
            )
            .map_err(|error| {
                anyhow::anyhow!("resolve active battle turn with trainer action: {error:?}")
            })?
        } else {
            self.resolve_battle_turn_with_items(combat, input, rng)?
        };
        commit_battle_turn_outcome(state, active_index, &outcome)
            .map_err(|error| anyhow::anyhow!("commit active battle turn: {error:?}"))?;
        if is_wild_battle {
            if let Some(escape) = outcome.events.iter().find_map(|event| match event {
                BattleEvent::RunAttempt {
                    side: BattleSide::Player,
                    outcome,
                } => Some(outcome),
                _ => None,
            }) {
                commit_wild_battle_escape_attempt(state, escape);
            }
        }
        Ok(outcome)
    }

    fn apply_player_item_happiness_before_battle_turn(
        &self,
        combat: &mut BattleCombatState,
        action: &BattleAction,
    ) -> Result<()> {
        if battle_action_locked_before_menu(combat, BattleSide::Player) {
            return Ok(());
        }
        let (party_index, item_id) = match action {
            BattleAction::Item { item_id } => (combat.player_party_index, item_id.as_str()),
            BattleAction::PartyItem {
                item_id,
                party_index,
                ..
            } => (*party_index, item_id.as_str()),
            _ => return Ok(()),
        };
        let Some(item) = self.items.get(item_id) else {
            // The core turn validator owns the canonical unknown-item error.
            return Ok(());
        };
        let Some(code) = item_happiness_change_code(item) else {
            return Ok(());
        };
        let changes = self.happiness_change(code)?;
        let pokemon = combat.player_party.get_mut(party_index).with_context(|| {
            format!("battle item happiness party index {party_index} is outside the player party")
        })?;
        core_apply_happiness_change(pokemon, changes);
        if party_index == combat.player_party_index {
            combat.player.happiness = pokemon.happiness;
        }
        Ok(())
    }

    fn active_battle_player_party(state: &GameState) -> Result<Vec<Pokemon>> {
        let mut party = Vec::new();
        let mut seen_empty = false;
        for (index, slot) in state.storage.party.pokemon.iter().enumerate() {
            match slot {
                Some(pokemon) => {
                    if seen_empty {
                        anyhow::bail!(
                            "active battle party has occupied slot {index} after an empty slot"
                        );
                    }
                    party.push(pokemon.clone());
                }
                None => seen_empty = true,
            }
        }
        Ok(party)
    }

    fn require_active_enemy_in_battle_party(party: &[Pokemon], active_index: usize) -> Result<()> {
        if active_index >= party.len() {
            anyhow::bail!(
                "active battle enemy index {active_index} is outside active enemy party length {}",
                party.len()
            );
        }
        Ok(())
    }

    pub fn resolve_active_wild_battle_run(
        &self,
        state: &mut GameState,
    ) -> Result<BattleEscapeAttempt> {
        let mut divider = RuntimeDividerSource::live();
        self.resolve_active_wild_battle_run_with_divider(state, &mut divider)
    }

    pub fn resolve_active_wild_battle_run_with_divider<S>(
        &self,
        state: &mut GameState,
        divider: &mut S,
    ) -> Result<BattleEscapeAttempt>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        let mut staged_state = state.clone();
        let battle_end = self.active_battle_end_context(&staged_state)?;
        let battle_type = active_battle_type(&staged_state)
            .ok_or_else(|| anyhow::anyhow!("cannot escape without an active battle"))?;
        if battle_type_blocks_escape(battle_type) {
            return Ok(BattleEscapeAttempt {
                escaped: false,
                chance: 0,
                roll: None,
                attempts_before: staged_state.battle_escape_attempts,
                attempts_after: staged_state.battle_escape_attempts,
            });
        }
        if battle_type_guarantees_escape(battle_type) || staged_state.link_session.link_mode != 0 {
            let outcome = BattleEscapeAttempt {
                escaped: true,
                chance: self.battle_escape_rules.rng_roll_values,
                roll: None,
                attempts_before: staged_state.battle_escape_attempts,
                attempts_after: staged_state.battle_escape_attempts,
            };
            commit_wild_battle_escape_attempt(&mut staged_state, &outcome);
            if let Some((battle_type, roaming_slot, enemy, map_name)) = battle_end {
                self.finish_battle_roaming_update_exact(
                    &mut staged_state,
                    &battle_type,
                    roaming_slot,
                    &enemy,
                    &map_name,
                    divider,
                )?;
            }
            *state = staged_state;
            return Ok(outcome);
        }
        let active_index = require_active_battle_party_slot_index(&staged_state)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        let player = staged_state.storage.party.pokemon[active_index]
            .as_ref()
            .cloned()
            .with_context(|| format!("active battle party index {active_index} has no Pokemon"))?;
        let enemy = match &staged_state.battle {
            BattleMemory::Wild { enemy_pokemon, .. }
            | BattleMemory::StaticWild { enemy_pokemon, .. } => enemy_pokemon.clone(),
            BattleMemory::Trainer { trainer_id, .. } => {
                anyhow::bail!("cannot escape from trainer battle {trainer_id}");
            }
            BattleMemory::Inactive => {
                anyhow::bail!("cannot escape without an active wild battle");
            }
        };
        let mut rng = CrystalRandom::new(staged_state.random_state, &mut *divider);
        let mut combat = staged_state
            .script_runtime
            .active_battle_combat
            .clone()
            .unwrap_or_else(|| {
                BattleCombatState::new(player, enemy.clone()).with_link_context(
                    staged_state.time.time_of_day,
                    staged_state.link_session.link_mode,
                    staged_state.link_session.serial_connection_status,
                )
            });
        combat.link_battle = staged_state.link_session.link_mode != 0;
        combat.link_colosseum = staged_state.link_session.link_mode == LINK_MODE_COLOSSEUM;
        combat.serial_connection_status = staged_state.link_session.serial_connection_status;
        if combat.force_switch_blocked
            || combat.player_escape_trap.is_some()
            || combat.player_trap.is_some()
        {
            return Ok(BattleEscapeAttempt {
                escaped: false,
                chance: 0,
                roll: None,
                attempts_before: staged_state.battle_escape_attempts,
                attempts_after: staged_state.battle_escape_attempts,
            });
        }
        let held_escape = match combat.player.item.as_deref() {
            Some(item_id) => {
                self.items
                    .get(item_id)
                    .with_context(|| format!("active Pokemon holds unknown item {item_id}"))?
                    .held_effect
                    == "HELD_ESCAPE"
            }
            None => false,
        };
        let outcome = if held_escape {
            BattleEscapeAttempt {
                escaped: true,
                chance: self.battle_escape_rules.rng_roll_values,
                roll: None,
                attempts_before: staged_state.battle_escape_attempts,
                attempts_after: staged_state.battle_escape_attempts,
            }
        } else {
            let player_speed = if combat.player.hp == 0 {
                anyhow::ensure!(
                    staged_state
                        .storage
                        .party
                        .pokemon
                        .iter()
                        .enumerate()
                        .any(|(index, pokemon)| index != active_index
                            && pokemon.as_ref().is_some_and(|pokemon| pokemon.hp > 0)),
                    "fainted-player escape requires an available replacement Pokemon"
                );
                staged_state.storage.party.pokemon[0]
                    .as_ref()
                    .context("fainted-player escape requires party slot 0")?
                    .speed
            } else {
                battle_speed(&combat, BattleSide::Player, &self.battle_stat_multipliers)
                    .map_err(|error| anyhow::anyhow!("resolve player escape speed: {error:?}"))?
            };
            let enemy_speed =
                battle_speed(&combat, BattleSide::Enemy, &self.battle_stat_multipliers)
                    .map_err(|error| anyhow::anyhow!("resolve enemy escape speed: {error:?}"))?;
            core_attempt_wild_battle_escape_exact_with_loaded_speeds(
                player_speed,
                enemy_speed,
                &self.battle_escape_rules,
                staged_state.battle_escape_attempts,
                &mut rng,
            )
            .map_err(|error| anyhow::anyhow!("resolve exact wild battle run: {error}"))?
        };
        commit_wild_battle_escape_attempt(&mut staged_state, &outcome);
        staged_state.random_state = rng.state();
        drop(rng);
        if outcome.escaped
            && let Some((battle_type, roaming_slot, enemy, map_name)) = battle_end
        {
            self.finish_battle_roaming_update_exact(
                &mut staged_state,
                &battle_type,
                roaming_slot,
                &enemy,
                &map_name,
                divider,
            )?;
        }
        *state = staged_state;
        Ok(outcome)
    }

    pub fn resolve_active_battle_command(
        &self,
        state: &mut GameState,
        player_action: BattleAction,
        enemy_action: BattleAction,
    ) -> Result<ActiveBattleCommandOutcome> {
        if matches!(player_action, BattleAction::Run) {
            if active_battle_type(state).is_some_and(battle_type_guarantees_escape)
                || state.link_session.link_mode != 0
            {
                return self
                    .resolve_active_wild_battle_run(state)
                    .map(ActiveBattleCommandOutcome::Escape);
            }
            return match &state.battle {
                BattleMemory::Wild { .. } | BattleMemory::StaticWild { .. } => {
                    if matches!(enemy_action, BattleAction::Run) {
                        self.resolve_active_wild_battle_run(state)
                            .map(ActiveBattleCommandOutcome::Escape)
                    } else {
                        self.resolve_active_battle_turn(state, BattleAction::Run, enemy_action)
                            .map(ActiveBattleCommandOutcome::Turn)
                    }
                }
                BattleMemory::Trainer { .. } => self
                    .resolve_active_battle_turn(state, BattleAction::Run, enemy_action)
                    .map(ActiveBattleCommandOutcome::Turn),
                BattleMemory::Inactive => {
                    anyhow::bail!("player run requires an active battle");
                }
            };
        }
        if matches!(enemy_action, BattleAction::Run) {
            match &state.battle {
                BattleMemory::Wild { .. }
                | BattleMemory::StaticWild { .. }
                | BattleMemory::Trainer { .. } => {}
                BattleMemory::Inactive => {
                    anyhow::bail!("enemy run requires an active battle");
                }
            }
            return self
                .resolve_active_battle_turn(state, player_action, enemy_action)
                .map(ActiveBattleCommandOutcome::Turn);
        }
        self.resolve_active_battle_turn(state, player_action, enemy_action)
            .map(ActiveBattleCommandOutcome::Turn)
    }

    pub fn resolve_active_battle_command_with_divider<S>(
        &self,
        state: &mut GameState,
        player_action: BattleAction,
        enemy_action: BattleAction,
        divider: &mut S,
    ) -> Result<ActiveBattleCommandOutcome>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        if matches!(player_action, BattleAction::Run) {
            if active_battle_type(state).is_some_and(battle_type_guarantees_escape)
                || state.link_session.link_mode != 0
            {
                return self
                    .resolve_active_wild_battle_run_with_divider(state, divider)
                    .map(ActiveBattleCommandOutcome::Escape);
            }
            return match &state.battle {
                BattleMemory::Wild { .. } | BattleMemory::StaticWild { .. } => {
                    if matches!(enemy_action, BattleAction::Run) {
                        self.resolve_active_wild_battle_run_with_divider(state, divider)
                            .map(ActiveBattleCommandOutcome::Escape)
                    } else {
                        self.resolve_active_battle_turn_with_divider(
                            state,
                            BattleAction::Run,
                            enemy_action,
                            divider,
                        )
                        .map(ActiveBattleCommandOutcome::Turn)
                    }
                }
                BattleMemory::Trainer { .. } => self
                    .resolve_active_battle_turn_with_divider(
                        state,
                        BattleAction::Run,
                        enemy_action,
                        divider,
                    )
                    .map(ActiveBattleCommandOutcome::Turn),
                BattleMemory::Inactive => {
                    anyhow::bail!("player run requires an active battle");
                }
            };
        }
        if matches!(enemy_action, BattleAction::Run) {
            match &state.battle {
                BattleMemory::Wild { .. }
                | BattleMemory::StaticWild { .. }
                | BattleMemory::Trainer { .. } => {}
                BattleMemory::Inactive => {
                    anyhow::bail!("enemy run requires an active battle");
                }
            }
        }
        self.resolve_active_battle_turn_with_divider(state, player_action, enemy_action, divider)
            .map(ActiveBattleCommandOutcome::Turn)
    }

    pub fn resolve_active_battle_enemy_action(
        &self,
        state: &mut GameState,
        enemy_action: BattleAction,
    ) -> Result<BattleTurnOutcome> {
        let mut divider = RuntimeDividerSource::live();
        self.resolve_active_battle_enemy_action_with_divider(state, enemy_action, &mut divider)
    }

    pub fn resolve_active_battle_enemy_action_with_divider<S>(
        &self,
        state: &mut GameState,
        enemy_action: BattleAction,
        divider: &mut S,
    ) -> Result<BattleTurnOutcome>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        let mut staged_state = state.clone();
        let battle_end = self.active_battle_end_context(&staged_state)?;
        let mut rng = ExactBattleRandom::new(staged_state.random_state, divider);
        let outcome = self.resolve_active_battle_enemy_action_with_rng(
            &mut staged_state,
            enemy_action,
            &mut rng,
        )?;
        if let Some(error) = rng.divider_error() {
            anyhow::bail!("resolve exact active battle enemy action: {error}");
        }
        staged_state.random_state = rng.state();
        drop(rng);
        if matches!(staged_state.battle, BattleMemory::Inactive)
            && let Some((battle_type, roaming_slot, _, map_name)) = battle_end
        {
            self.finish_battle_roaming_update_exact(
                &mut staged_state,
                &battle_type,
                roaming_slot,
                &outcome.state.enemy,
                &map_name,
                divider,
            )?;
        }
        *state = staged_state;
        Ok(outcome)
    }

    fn resolve_active_battle_enemy_action_with_rng(
        &self,
        state: &mut GameState,
        enemy_action: BattleAction,
        rng: &mut dyn BattleRandomSource,
    ) -> Result<BattleTurnOutcome> {
        let active_index =
            require_active_battle_party_index(state).map_err(|error| anyhow::anyhow!("{error}"))?;
        let player = state.storage.party.pokemon[active_index]
            .as_ref()
            .cloned()
            .with_context(|| format!("active battle party index {active_index} has no Pokemon"))?;
        let player_party = Self::active_battle_player_party(state)?;
        let active_enemy_index = require_active_battle_enemy_party_index(state)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        let force_switch_ends_battle = matches!(
            &state.battle,
            BattleMemory::Wild { .. } | BattleMemory::StaticWild { .. }
        );
        let (enemy, enemy_party, active_enemy_index) = match &state.battle {
            BattleMemory::Wild {
                enemy_pokemon,
                enemy_party,
                ..
            }
            | BattleMemory::StaticWild {
                enemy_pokemon,
                enemy_party,
                ..
            }
            | BattleMemory::Trainer {
                enemy_pokemon,
                enemy_party,
                ..
            } => (
                enemy_pokemon.clone(),
                enemy_party.clone(),
                active_enemy_index,
            ),
            BattleMemory::Inactive => {
                anyhow::bail!("cannot resolve enemy battle action without an active battle");
            }
        };
        Self::require_active_enemy_in_battle_party(&enemy_party, active_enemy_index)?;
        let badge_boosts_enabled = state.link_session.link_mode == 0
            && active_battle_type(state) != Some("BATTLETYPE_BATTLE_TOWER");
        let mut combat = state
            .script_runtime
            .active_battle_combat
            .clone()
            .unwrap_or_else(|| {
                BattleCombatState::new(player, enemy)
                    .with_parties(player_party, enemy_party.to_vec())
                    .with_party_indices(active_index, active_enemy_index)
                    .with_obedience(state.player_id, state.badges.johto)
                    .with_kanto_badges(state.badges.kanto)
                    .with_link_context(
                        state.time.time_of_day,
                        state.link_session.link_mode,
                        state.link_session.serial_connection_status,
                    )
                    .with_badge_boosts_enabled(badge_boosts_enabled)
            });
        combat.link_battle = state.link_session.link_mode != 0;
        combat.link_colosseum = state.link_session.link_mode == LINK_MODE_COLOSSEUM;
        combat.serial_connection_status = state.link_session.serial_connection_status;
        combat.obedience_badges = state.badges.johto;
        combat.kanto_badges = state.badges.kanto;
        combat.badge_boosts_enabled = badge_boosts_enabled;
        let outcome = self.resolve_battle_enemy_action_with_items(
            combat,
            enemy_action,
            force_switch_ends_battle,
            rng,
        )?;
        let pay_day_money_after_turn = self.active_battle_pay_day_money_after_turn(state, &outcome);
        commit_battle_turn_outcome(state, active_index, &outcome)
            .map_err(|error| anyhow::anyhow!("commit enemy battle action: {error:?}"))?;
        if matches!(state.battle, BattleMemory::Inactive) && state.battle_result & 0x3f == 0 {
            self.claim_active_battle_pay_day_money(state, pay_day_money_after_turn)?;
        }
        Ok(outcome)
    }

    pub fn resolve_battle_turn_with_items(
        &self,
        combat: BattleCombatState,
        input: BattleTurnInput,
        rng: &mut dyn BattleRandomSource,
    ) -> Result<BattleTurnOutcome> {
        core_resolve_battle_turn_with_items(
            combat,
            input,
            &self.moves,
            &self.items,
            &self.move_priorities,
            &self.battle_stat_multipliers,
            &self.type_categories,
            &self.type_effectiveness,
            &self.weather_modifiers,
            rng,
        )
        .map_err(|error| anyhow::anyhow!("resolve active battle turn: {error:?}"))
    }

    pub fn select_trainer_enemy_move_slot(
        &self,
        combat: &BattleCombatState,
        ai_flags: u32,
        rng: &mut dyn BattleRandomSource,
    ) -> Result<usize> {
        Ok(self
            .select_trainer_enemy_move_with_scratch(combat, ai_flags, rng)?
            .slot)
    }

    pub fn select_trainer_enemy_move_with_scratch(
        &self,
        combat: &BattleCombatState,
        ai_flags: u32,
        rng: &mut dyn BattleRandomSource,
    ) -> Result<EnemyMoveSelection> {
        if battle_action_locked_before_menu(combat, BattleSide::Enemy) {
            return Ok(EnemyMoveSelection {
                slot: core_select_wild_enemy_move_slot(combat, rng),
                ai_damage_register: 0,
            });
        }
        let enemy_moves = battle_moves(combat, BattleSide::Enemy);
        let disabled = combat
            .enemy_disable
            .as_ref()
            .filter(|disable| disable.turns_remaining > 0)
            .map(|disable| disable.move_name.as_str());
        let usable_slots = enemy_moves
            .iter()
            .take(4)
            .enumerate()
            .filter_map(|(slot, learned)| {
                (learned.current_pp > 0 && disabled != Some(learned.name.as_str())).then_some(slot)
            })
            .collect::<BTreeSet<_>>();
        if usable_slots.is_empty() {
            return Ok(EnemyMoveSelection {
                slot: 0,
                ai_damage_register: 0,
            });
        }
        let mut ai_damage_register = 0;
        let move_data_by_slot = enemy_moves
            .iter()
            .take(4)
            .map(|learned| {
                self.moves
                    .get(&learned.name)
                    .with_context(|| format!("trainer AI references unknown move {}", learned.name))
                    .map(Some)
            })
            .collect::<Result<Vec<_>>>()?;
        let mut scores = move_data_by_slot
            .iter()
            .enumerate()
            .map(|(slot, move_data)| {
                if usable_slots.contains(&slot) && move_data.is_some() {
                    20_i16
                } else {
                    80_i16
                }
            })
            .collect::<Vec<_>>();
        let player_types = combat
            .player_type_override
            .as_ref()
            .map(|types| vec![types.type1.clone(), types.type2.clone()])
            .unwrap_or_else(|| {
                vec![
                    combat.player.species.type1.clone(),
                    combat.player.species.type2.clone(),
                ]
            });
        let enemy_types = combat
            .enemy_type_override
            .as_ref()
            .map(|types| vec![types.type1.clone(), types.type2.clone()])
            .or_else(|| {
                combat.enemy_transform.as_ref().map(|transform| {
                    vec![
                        transform.species.type1.clone(),
                        transform.species.type2.clone(),
                    ]
                })
            })
            .unwrap_or_else(|| {
                vec![
                    combat.enemy.species.type1.clone(),
                    combat.enemy.species.type2.clone(),
                ]
            });
        let matchup_for = |move_type: &str, defender_types: &[String], identified: bool| {
            trainer_ai_type_matchup(
                &self.type_effectiveness,
                move_type,
                defender_types,
                identified,
            )
        };
        let known_move_data = move_data_by_slot
            .iter()
            .flatten()
            .map(|move_data| (move_data.move_type.as_str(), move_data.power))
            .collect::<Vec<_>>();

        if ai_flags & (1 << 0) != 0 {
            for (score, move_data) in scores.iter_mut().zip(&move_data_by_slot) {
                let Some(move_data) = move_data else { break };
                *score +=
                    crystal_core::battle::ai::trainer_basic_score_delta(combat, &move_data.effect);
            }
        }
        if ai_flags & (1 << 1) != 0 {
            for (score, move_data) in scores.iter_mut().zip(&move_data_by_slot) {
                let Some(move_data) = move_data else { break };
                *score += crystal_core::battle::ai::trainer_setup_score_delta(
                    &move_data.effect,
                    combat.enemy_turns_taken,
                    combat.player_turns_taken,
                    rng,
                );
            }
        }
        if ai_flags & (1 << 2) != 0 {
            for (score, move_data) in scores.iter_mut().zip(&move_data_by_slot) {
                let Some(move_data) = move_data else { break };
                *score += crystal_core::battle::ai::trainer_types_score_delta(
                    &move_data.move_type,
                    move_data.power,
                    &known_move_data,
                    matchup_for(
                        &move_data.move_type,
                        &player_types,
                        combat.player_identified,
                    ),
                );
            }
        }
        if ai_flags & (1 << 3) != 0 {
            for (score, move_data) in scores.iter_mut().zip(&move_data_by_slot) {
                let Some(move_data) = move_data else { break };
                *score += crystal_core::battle::ai::trainer_offensive_score_delta(move_data.power);
            }
        }
        if ai_flags & (1 << 4) != 0 {
            let smart_moves = move_data_by_slot
                .iter()
                .map(|move_data| {
                    move_data.map(|move_data| {
                        let (power, matchup) = if move_data.effect == "HIDDEN_POWER" {
                            let (move_type, power) =
                                crystal_core::battle::ai::trainer_smart_hidden_power(combat);
                            (
                                power,
                                matchup_for(&move_type, &player_types, combat.player_identified),
                            )
                        } else {
                            (
                                move_data.power,
                                matchup_for(
                                    &move_data.move_type,
                                    &player_types,
                                    combat.player_identified,
                                ),
                            )
                        };
                        crystal_core::battle::ai::TrainerSmartMove {
                            move_id: move_data.name.as_str(),
                            effect: move_data.effect.as_str(),
                            power,
                            accuracy: move_data.accuracy,
                            matchup,
                        }
                    })
                })
                .collect::<Vec<_>>();
            let effective_player_moves = battle_moves(combat, BattleSide::Player);
            let mut smart_player_moves = Vec::with_capacity(combat.player_used_moves.len());
            for move_id in &combat.player_used_moves {
                let move_data = self.moves.get(move_id).with_context(|| {
                    format!("Smart AI references unknown player move {move_id}")
                })?;
                smart_player_moves.push(crystal_core::battle::ai::TrainerSmartPlayerMove {
                    move_id: move_data.name.as_str(),
                    effect: move_data.effect.as_str(),
                    power: move_data.power,
                    physical: crystal_core::battle::damage::is_physical_move(&self.type_categories, move_data)
                        .map_err(|error| anyhow::anyhow!("move category: {error:?}"))?,
                    matchup_against_enemy: matchup_for(
                        &move_data.move_type,
                        &enemy_types,
                        combat.enemy_identified,
                    ),
                    matchup_against_player: matchup_for(
                        &move_data.move_type,
                        &player_types,
                        combat.player_identified,
                    ),
                    current_pp: effective_player_moves
                        .iter()
                        .find(|learned| learned.name == *move_id)
                        .map(|learned| learned.current_pp),
                });
            }
            let player_type_matchups_against_enemy = player_types
                .iter()
                .filter(|type_id| type_id.as_str() != "NONE")
                .collect::<BTreeSet<_>>()
                .into_iter()
                .map(|type_id| matchup_for(type_id, &enemy_types, combat.enemy_identified))
                .collect::<Vec<_>>();
            let enemy_move_matchups_against_enemy = move_data_by_slot
                .iter()
                .map(|move_data| {
                    move_data
                        .map(|move_data| {
                            matchup_for(&move_data.move_type, &enemy_types, combat.enemy_identified)
                        })
                        .unwrap_or(crystal_core::battle::ai::TrainerAiTypeMatchup::Neutral)
                })
                .collect::<Vec<_>>();
            let mut priority_damage_by_slot = Vec::with_capacity(move_data_by_slot.len());
            for move_data in &move_data_by_slot {
                priority_damage_by_slot.push(match move_data {
                    Some(move_data) if move_data.effect == "PRIORITY_HIT" => {
                        let damage = self.trainer_ai_damage_for_move(combat, move_data, rng)?;
                        ai_damage_register = damage;
                        Some(damage)
                    }
                    Some(_) | None => None,
                });
            }
            crystal_core::battle::ai::apply_trainer_smart_scores(
                combat,
                &mut scores,
                &smart_moves,
                &smart_player_moves,
                &player_type_matchups_against_enemy,
                &enemy_move_matchups_against_enemy,
                &priority_damage_by_slot,
                rng,
            );
        }
        if ai_flags & (1 << 5) != 0
            && crystal_core::battle::ai::trainer_opportunist_discourages(
                combat.enemy.hp,
                combat.enemy.max_hp,
                rng,
            )
        {
            for (score, move_data) in scores.iter_mut().zip(&move_data_by_slot) {
                let Some(move_data) = move_data else { break };
                if crystal_core::battle::ai::trainer_ai_stall_move(&move_data.name) {
                    *score += 1;
                }
            }
        }
        if ai_flags & (1 << 6) != 0 {
            let mut evaluations = Vec::with_capacity(move_data_by_slot.len());
            for move_data in &move_data_by_slot {
                evaluations.push(match move_data {
                    Some(move_data) => {
                        let damage = self.trainer_ai_damage_for_move(combat, move_data, rng)?;
                        ai_damage_register = damage;
                        Some(crystal_core::battle::ai::TrainerAiDamageEvaluation {
                            effect: move_data.effect.as_str(),
                            power: move_data.power,
                            damage,
                        })
                    }
                    None => None,
                });
            }
            crystal_core::battle::ai::apply_trainer_aggressive_scores(&mut scores, &evaluations);
        }
        if ai_flags & (1 << 7) != 0 {
            let move_names = move_data_by_slot
                .iter()
                .map(|move_data| move_data.map(|move_data| move_data.name.as_str()))
                .collect::<Vec<_>>();
            crystal_core::battle::ai::apply_trainer_cautious_scores(
                &mut scores,
                &move_names,
                combat.enemy_turns_taken,
                rng,
            );
        }
        if ai_flags & (1 << 8) != 0 {
            let player_type_refs = player_types.iter().map(String::as_str).collect::<Vec<_>>();
            for (score, move_data) in scores.iter_mut().zip(&move_data_by_slot) {
                let Some(move_data) = move_data else { break };
                *score += crystal_core::battle::ai::trainer_status_score_delta(
                    &move_data.effect,
                    move_data.power,
                    &player_type_refs,
                    matchup_for(
                        &move_data.move_type,
                        &player_types,
                        combat.player_identified,
                    ),
                );
            }
        }
        if ai_flags & (1 << 9) != 0 {
            for (score, move_data) in scores.iter_mut().zip(&move_data_by_slot) {
                let Some(move_data) = move_data else { break };
                if crystal_core::battle::ai::trainer_risky_should_check_ko(
                    &move_data.effect,
                    move_data.power,
                    combat.enemy.hp,
                    combat.enemy.max_hp,
                    rng,
                ) {
                    let damage = self.trainer_ai_damage_for_move(combat, move_data, rng)?;
                    ai_damage_register = damage;
                    *score += crystal_core::battle::ai::trainer_risky_ko_score_delta(
                        damage,
                        combat.player.hp,
                    );
                }
            }
        }
        let best_score = scores.iter().copied().min().unwrap_or(20);
        let best = scores
            .iter()
            .enumerate()
            .filter_map(|(slot, score)| (*score == best_score).then_some(slot))
            .collect::<BTreeSet<_>>();
        loop {
            let slot = usize::from(rng.battle_random_byte() & 3);
            if best.contains(&slot) {
                return Ok(EnemyMoveSelection {
                    slot,
                    ai_damage_register,
                });
            }
        }
    }

    fn trainer_ai_damage_for_move(
        &self,
        combat: &BattleCombatState,
        move_data: &Move,
        rng: &mut dyn BattleRandomSource,
    ) -> Result<u16> {
        let held_type_boost_percent = if let Some(item_id) = combat.enemy.item.as_deref() {
            let item = self.items.get(item_id).with_context(|| {
                format!("enemy AI damage references unknown held item {item_id}")
            })?;
            match crystal_core::battle::ai::trainer_ai_held_type_boost_percent(
                &item.held_effect,
                item.parameter,
                &move_data.move_type,
            ) {
                Some(Ok(parameter)) => parameter,
                Some(Err(parameter)) => anyhow::bail!(
                    "enemy AI held item {item_id} has invalid type-boost parameter {parameter}"
                ),
                None => 0,
            }
        } else {
            0
        };
        crystal_core::battle::ai::trainer_ai_damage(
            combat,
            move_data,
            &self.battle_stat_multipliers,
            &self.type_categories,
            &self.type_effectiveness,
            &self.weather_modifiers,
            held_type_boost_percent,
            rng,
        )
        .map_err(|error| anyhow::anyhow!("enemy AI damage calculation failed: {error:?}"))
    }

    pub fn select_trainer_post_order_action(
        &self,
        combat: &BattleCombatState,
        trainer_id: &str,
        battle_type: &str,
        ai_item_switch_flags: u32,
        trainer_items_used: &mut BTreeSet<String>,
        selected_move_slot: usize,
        rng: &mut dyn BattleRandomSource,
    ) -> Result<BattleAction> {
        let switch_flags = if battle_type == "BATTLETYPE_BATTLE_TOWER" {
            self.trainers
                .trainers
                .values()
                .find(|trainer| trainer.trainer_class == "FALKNER")
                .context("Battle Tower AI requires trainer class 1 FALKNER attributes")?
                .ai_item_switch_flags
        } else {
            ai_item_switch_flags
        };
        if battle_action_locked_before_menu(combat, BattleSide::Enemy) {
            return Ok(BattleAction::Move {
                slot: selected_move_slot,
            });
        }

        // AI_SwitchOrTryItem reads the player's CANT_RUN bit, which
        // ArenaTrap sets on the user when it traps the opposing enemy.
        let switch_blocked = combat.enemy_escape_trap.is_some() || combat.enemy_trap.is_some();
        let candidate = if switch_blocked {
            None
        } else {
            self.trainer_switch_candidate(combat)?
        };
        let Some((party_index, switch_tier)) = candidate else {
            return self.trainer_item_or_move(
                combat,
                trainer_id,
                battle_type,
                switch_flags,
                trainer_items_used,
                selected_move_slot,
                rng,
            );
        };
        let switch_mask = switch_flags & 0x07;
        if switch_mask == 0 {
            return self.trainer_item_or_move(
                combat,
                trainer_id,
                battle_type,
                switch_flags,
                trainer_items_used,
                selected_move_slot,
                rng,
            );
        }
        let roll = u32::from(rng.battle_random_byte());
        let should_switch = if switch_mask & 0x01 != 0 {
            match switch_tier {
                0x10 => roll < 128,
                0x20 => roll < 200,
                _ => roll >= 10,
            }
        } else if switch_mask & 0x02 != 0 {
            match switch_tier {
                0x10 => roll < 20,
                0x20 => roll < 30,
                _ => roll >= 200,
            }
        } else {
            match switch_tier {
                0x10 => roll < 50,
                0x20 => roll < 128,
                _ => roll >= 50,
            }
        };
        if should_switch {
            return Ok(BattleAction::TrainerSwitch {
                selected_move_slot,
                party_index,
            });
        }
        self.trainer_item_or_move(
            combat,
            trainer_id,
            battle_type,
            switch_flags,
            trainer_items_used,
            selected_move_slot,
            rng,
        )
    }

    fn trainer_item_or_move(
        &self,
        combat: &BattleCombatState,
        trainer_id: &str,
        battle_type: &str,
        flags: u32,
        used: &mut BTreeSet<String>,
        selected_move_slot: usize,
        rng: &mut dyn BattleRandomSource,
    ) -> Result<BattleAction> {
        Ok(
            match self.select_trainer_item(combat, trainer_id, battle_type, flags, used, rng)? {
                Some(item_id) => BattleAction::TrainerItem {
                    selected_move_slot,
                    item_id,
                },
                None => BattleAction::Move {
                    slot: selected_move_slot,
                },
            },
        )
    }

    fn select_trainer_item(
        &self,
        combat: &BattleCombatState,
        trainer_id: &str,
        battle_type: &str,
        flags: u32,
        used: &mut BTreeSet<String>,
        rng: &mut dyn BattleRandomSource,
    ) -> Result<Option<String>> {
        if battle_type == "BATTLETYPE_BATTLE_TOWER" {
            return Ok(None);
        }
        let trainer =
            self.trainers.trainers.get(trainer_id).with_context(|| {
                format!("trainer item AI references unknown trainer {trainer_id}")
            })?;
        let active = combat
            .enemy_party
            .get(combat.enemy_party_index)
            .context("trainer item AI active enemy index is outside its party")?;
        if active.max_hp == 0 || active.hp == 0 {
            return Ok(None);
        }
        if combat
            .enemy_party
            .iter()
            .any(|pokemon| pokemon.level > active.level)
        {
            return Ok(None);
        }
        let has_status = active
            .status
            .as_deref()
            .is_some_and(|status| !status.is_empty() && status != "NONE");
        let below_half = active.hp.saturating_mul(2) <= active.max_hp;
        let below_quarter = active.hp.saturating_mul(4) <= active.max_hp;
        let context_use = flags & (1 << 6) != 0;
        let always_use = flags & (1 << 4) != 0;
        let unknown_use = flags & (1 << 5) != 0;
        let heal_item_usable = |rng: &mut dyn BattleRandomSource| {
            if context_use {
                below_quarter || (below_half && rng.battle_random_byte() < 50)
            } else if unknown_use {
                below_quarter && rng.battle_random_byte() >= 50
            } else {
                below_quarter || (below_half && rng.battle_random_byte() < 128)
            }
        };
        const ITEM_ORDER: [&str; 13] = [
            "FULL_RESTORE",
            "MAX_POTION",
            "HYPER_POTION",
            "SUPER_POTION",
            "POTION",
            "X_ACCURACY",
            "FULL_HEAL",
            "GUARD_SPEC",
            "DIRE_HIT",
            "X_ATTACK",
            "X_DEFEND",
            "X_SPEED",
            "X_SPECIAL",
        ];
        for item_id in ITEM_ORDER {
            let Some(item_slot) = trainer.items.iter().enumerate().find_map(|(slot, owned)| {
                (owned.as_deref() == Some(item_id)
                    && !used.contains(&format!("{trainer_id}:{item_id}:{slot}")))
                .then_some(slot)
            }) else {
                continue;
            };
            let usable = match item_id {
                "FULL_RESTORE" => {
                    heal_item_usable(rng)
                        || (context_use
                            && (matches!(active.status.as_deref(), Some("FREEZE" | "SLEEP"))
                                || (active.status.as_deref() == Some("BAD_POISON")
                                    && combat.enemy_toxic_turns >= 4
                                    && rng.battle_random_byte() < 128)))
                }
                "MAX_POTION" | "HYPER_POTION" | "SUPER_POTION" | "POTION" => heal_item_usable(rng),
                "FULL_HEAL" if context_use => {
                    matches!(active.status.as_deref(), Some("FREEZE" | "SLEEP"))
                        || (active.status.as_deref() == Some("BAD_POISON")
                            && combat.enemy_toxic_turns >= 4
                            && rng.battle_random_byte() < 128)
                }
                "FULL_HEAL" if always_use => has_status,
                "FULL_HEAL" => has_status && rng.battle_random_byte() < 50,
                "X_ACCURACY" | "GUARD_SPEC" | "DIRE_HIT" | "X_ATTACK" | "X_DEFEND" | "X_SPEED"
                | "X_SPECIAL" => {
                    if combat.enemy_turns_taken == 0 {
                        always_use
                            || (rng.battle_random_byte() >= 128
                                && (context_use || rng.battle_random_byte() >= 128))
                    } else {
                        always_use && rng.battle_random_byte() < 50
                    }
                }
                _ => false,
            };
            if usable {
                used.insert(format!("{trainer_id}:{item_id}:{item_slot}"));
                return Ok(Some(item_id.to_string()));
            }
        }
        Ok(None)
    }

    fn trainer_switch_candidate(&self, combat: &BattleCombatState) -> Result<Option<(usize, u8)>> {
        let active_index = combat.enemy_party_index;
        let active = combat
            .enemy_party
            .get(active_index)
            .context("trainer switch active enemy index is outside its party")?;
        let player = combat
            .player_party
            .get(combat.player_party_index)
            .context("trainer switch active player index is outside its party")?;
        let alive = combat
            .enemy_party
            .iter()
            .enumerate()
            .filter(|(index, pokemon)| {
                *index != active_index
                    && pokemon.hp > 0
                    && !pokemon.is_egg
                    && pokemon.species.id != "EGG"
            })
            .collect::<Vec<_>>();
        if alive.is_empty() {
            return Ok(None);
        }
        let last_player_move = combat
            .player_last_move
            .as_deref()
            .and_then(|move_id| self.moves.get(move_id));
        let candidate_is_healthy = |pokemon: &Pokemon| {
            pokemon.max_hp > 0 && u32::from(pokemon.hp) * 4 >= u32::from(pokemon.max_hp)
        };
        let resists_player = |pokemon: &Pokemon| -> Result<bool> {
            if let Some(move_data) = last_player_move.filter(|move_data| move_data.power > 0) {
                return Ok(trainer_switch_type_matchup(
                    &self.type_effectiveness,
                    &move_data.move_type,
                    pokemon,
                )? <= 0);
            }
            Ok(trainer_switch_type_matchup(
                &self.type_effectiveness,
                &player.species.type1,
                pokemon,
            )? <= 0
                && trainer_switch_type_matchup(
                    &self.type_effectiveness,
                    &player.species.type2,
                    pokemon,
                )? <= 0)
        };
        let has_super_effective_move = |pokemon: &Pokemon| -> Result<bool> {
            for learned in &pokemon.moves {
                let move_data = self
                    .moves
                    .get(&learned.name)
                    .with_context(|| format!("trainer switch move {} is missing", learned.name))?;
                if move_data.power > 0
                    && trainer_switch_type_matchup(
                        &self.type_effectiveness,
                        &move_data.move_type,
                        player,
                    )? > 0
                {
                    return Ok(true);
                }
            }
            Ok(false)
        };

        if active.perish_song_turns == 1 {
            for (index, pokemon) in &alive {
                if candidate_is_healthy(pokemon)
                    && resists_player(pokemon)?
                    && has_super_effective_move(pokemon)?
                {
                    return Ok(Some((*index, 0x30)));
                }
            }
            return Ok(Some((alive[0].0, 0x30)));
        }

        let switch_score = |enemy: &Pokemon, enemy_moves: &[LearnedMove]| -> Result<i8> {
            let mut score = 10_i8;
            if combat.player_used_moves.is_empty() {
                for player_type in [&player.species.type1, &player.species.type2] {
                    if trainer_switch_type_matchup(&self.type_effectiveness, player_type, enemy)?
                        > 0
                    {
                        score -= 1;
                    }
                    if player.species.type1 == player.species.type2 {
                        break;
                    }
                }
            } else {
                let mut best = 0_i8;
                for move_id in &combat.player_used_moves {
                    let move_data = self
                        .moves
                        .get(move_id)
                        .with_context(|| format!("used player move {move_id} is missing"))?;
                    if move_data.power == 0 {
                        continue;
                    }
                    let matchup = trainer_switch_type_matchup(
                        &self.type_effectiveness,
                        &move_data.move_type,
                        enemy,
                    )?;
                    if matchup > 0 {
                        score -= 1;
                        best = 2;
                        break;
                    }
                    if matchup == 0 {
                        best = 2;
                    } else if matchup == -1 && best == 0 {
                        best = 1;
                    }
                }
                if best != 2 {
                    score += 1;
                    if best == 0 {
                        score += 1;
                    }
                }
            }
            let mut enemy_matchup_score = 0_u16;
            for learned in enemy_moves {
                let move_data = self
                    .moves
                    .get(&learned.name)
                    .with_context(|| format!("trainer switch move {} is missing", learned.name))?;
                if move_data.power == 0 {
                    continue;
                }
                match trainer_switch_type_matchup(
                    &self.type_effectiveness,
                    &move_data.move_type,
                    player,
                )? {
                    -2 => {}
                    -1 => enemy_matchup_score = enemy_matchup_score.saturating_add(1),
                    0 => enemy_matchup_score = enemy_matchup_score.saturating_add(5),
                    _ => enemy_matchup_score = 100,
                }
            }
            if enemy_matchup_score == 0 {
                score -= 2;
            } else if enemy_matchup_score < 5 {
                score -= 1;
            } else if enemy_matchup_score >= 100 {
                score += 1;
            }
            Ok(score)
        };

        if switch_score(active, battle_moves(combat, BattleSide::Enemy))? >= 11 {
            return Ok(None);
        }
        if let Some(last_move) = last_player_move.filter(|move_data| move_data.power > 0) {
            let mut neutral_immune = None;
            let mut super_immune = None;
            for (index, pokemon) in &alive {
                if trainer_switch_type_matchup(
                    &self.type_effectiveness,
                    &last_move.move_type,
                    pokemon,
                )? != -2
                {
                    continue;
                }
                let mut has_neutral = false;
                for learned in &pokemon.moves {
                    let move_data = self.moves.get(&learned.name).with_context(|| {
                        format!("trainer switch move {} is missing", learned.name)
                    })?;
                    if move_data.power == 0 {
                        continue;
                    }
                    match trainer_switch_type_matchup(
                        &self.type_effectiveness,
                        &move_data.move_type,
                        player,
                    )? {
                        matchup if matchup > 0 => {
                            super_immune.get_or_insert((*index, *pokemon));
                            break;
                        }
                        0 => has_neutral = true,
                        _ => {}
                    }
                }
                if has_neutral {
                    neutral_immune.get_or_insert((*index, *pokemon));
                }
            }
            if let Some((index, pokemon)) = super_immune.or(neutral_immune) {
                let candidate_score = switch_score(pokemon, &pokemon.moves)?;
                if combat.enemy_party.len() == 2 {
                    return Ok(Some((
                        index,
                        if candidate_score < 10 { 0x20 } else { 0x10 },
                    )));
                }
                if candidate_score < 10 {
                    return Ok(Some((index, 0x10)));
                }
                return Ok(None);
            }
        }
        for (index, pokemon) in alive {
            if candidate_is_healthy(pokemon)
                && resists_player(pokemon)?
                && has_super_effective_move(pokemon)?
                && switch_score(pokemon, &pokemon.moves)? < 10
            {
                return Ok(Some((index, 0x10)));
            }
        }
        Ok(None)
    }

    pub fn resolve_wild_battle_turn_with_items(
        &self,
        combat: BattleCombatState,
        input: BattleTurnInput,
        attempts: u8,
        rng: &mut dyn BattleRandomSource,
    ) -> Result<BattleTurnOutcome> {
        core_resolve_wild_battle_turn_with_items(
            combat,
            input,
            &self.moves,
            &self.items,
            &self.move_priorities,
            &self.battle_stat_multipliers,
            &self.type_categories,
            &self.type_effectiveness,
            &self.weather_modifiers,
            &self.battle_escape_rules,
            attempts,
            rng,
        )
        .map_err(|error| anyhow::anyhow!("resolve active wild battle turn: {error:?}"))
    }

    pub fn resolve_battle_enemy_action_with_items(
        &self,
        combat: BattleCombatState,
        enemy_action: BattleAction,
        force_switch_ends_battle: bool,
        rng: &mut dyn BattleRandomSource,
    ) -> Result<BattleTurnOutcome> {
        core_resolve_battle_enemy_action_with_items(
            combat,
            enemy_action,
            force_switch_ends_battle,
            &self.moves,
            &self.items,
            &self.battle_stat_multipliers,
            &self.type_categories,
            &self.type_effectiveness,
            &self.weather_modifiers,
            rng,
        )
        .map_err(|error| anyhow::anyhow!("resolve active enemy battle action: {error:?}"))
    }

    pub fn resolve_wild_battle_run(
        &self,
        combat: &BattleCombatState,
        attempts: u8,
        rng: &mut dyn BattleRandomSource,
    ) -> Result<BattleEscapeAttempt> {
        core_resolve_wild_battle_run(
            combat,
            &self.battle_escape_rules,
            attempts,
            &self.battle_stat_multipliers,
            rng,
        )
        .map_err(|error| anyhow::anyhow!("resolve wild battle run: {error:?}"))
    }

    pub fn claim_active_trainer_battle_rewards(
        &self,
        state: &mut GameState,
        time_of_day: TimeOfDay,
    ) -> Result<BattleRewardOutcome> {
        let mut staged_state = state.clone();
        let level_up_happiness = self.level_up_happiness_context(&staged_state)?;
        let outcome = core_claim_active_trainer_battle_rewards(
            &mut staged_state,
            &self.battle_reward_rules,
            &self.pokemon,
            &self.moves,
            &self.learnsets,
            &self.growth_rates,
            &self.evolutions,
            &self.battle_stat_multipliers,
            level_up_happiness,
            time_of_day,
        )
        .map_err(|error| anyhow::anyhow!("claim trainer battle rewards: {error:?}"))?;
        *state = staged_state;
        Ok(outcome)
    }

    pub fn claim_active_trainer_battle_rewards_now(
        &self,
        state: &mut GameState,
    ) -> Result<BattleRewardOutcome> {
        self.claim_active_trainer_battle_rewards(state, state.time.time_of_day)
    }

    pub fn claim_active_wild_battle_rewards<S>(
        &self,
        state: &mut GameState,
        time_of_day: TimeOfDay,
        divider: &mut S,
    ) -> Result<BattleRewardOutcome>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        let mut staged_state = state.clone();
        let level_up_happiness = self.level_up_happiness_context(&staged_state)?;
        let battle_end = self.active_battle_end_context(&staged_state)?;
        let pay_day_money = self.active_battle_pay_day_payout(&staged_state);
        self.claim_active_battle_pay_day_money(&mut staged_state, pay_day_money)?;
        let outcome = core_claim_active_wild_battle_rewards(
            &mut staged_state,
            &self.battle_reward_rules,
            &self.pokemon,
            &self.moves,
            &self.learnsets,
            &self.growth_rates,
            &self.evolutions,
            level_up_happiness,
            time_of_day,
            divider,
        )
        .map_err(|error| anyhow::anyhow!("claim wild battle rewards: {error:?}"))?;
        if matches!(staged_state.battle, BattleMemory::Inactive)
            && let Some((battle_type, roaming_slot, enemy, map_name)) = battle_end
        {
            self.finish_battle_roaming_update_exact(
                &mut staged_state,
                &battle_type,
                roaming_slot,
                &enemy,
                &map_name,
                divider,
            )?;
        }
        if let Some(terminal) = staged_state.pending_static_wild_terminal.as_mut() {
            terminal.win_cleanup_applied = true;
        }
        *state = staged_state;
        Ok(outcome)
    }

    pub fn level_up_happiness_context(
        &self,
        state: &GameState,
    ) -> Result<BattleLevelUpHappinessContext> {
        let map_name = match &state.overworld {
            crystal_core::state::OverworldMemory::Active { map_name, .. } => map_name,
            crystal_core::state::OverworldMemory::Inactive => match &state.battle {
                BattleMemory::Wild { map_name, .. } => map_name,
                BattleMemory::StaticWild { .. }
                | BattleMemory::Trainer { .. }
                | BattleMemory::Inactive => {
                    anyhow::bail!("level-up happiness requires an active map")
                }
            },
        };
        let current_landmark = u8::try_from(self.pokegear_landmark_for_map(map_name)?.id)
            .with_context(|| format!("battle map {map_name} landmark exceeds one byte"))?;
        Ok(BattleLevelUpHappinessContext {
            current_landmark,
            gain_level: self.happiness_change("HAPPINESS_GAINLEVEL")?,
            gain_level_at_home: self.happiness_change("HAPPINESS_GAINLEVELATHOME")?,
        })
    }

    pub fn happiness_change(&self, code: &str) -> Result<[i16; 3]> {
        self.happiness_data
            .as_ref()
            .context("item and battle rewards require exported happiness changes")?
            .changes
            .values()
            .find(|entry| entry.code == code)
            .with_context(|| format!("happiness changes missing {code}"))
            .map(|entry| [entry.low, entry.mid, entry.high])
    }

    pub fn claim_active_wild_battle_rewards_now<S>(
        &self,
        state: &mut GameState,
        divider: &mut S,
    ) -> Result<BattleRewardOutcome>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        self.claim_active_wild_battle_rewards(state, state.time.time_of_day, divider)
    }

    fn claim_active_battle_pay_day_money(&self, state: &mut GameState, amount: u32) -> Result<()> {
        if amount == 0 {
            return Ok(());
        }
        let max_money = self
            .currency_constants
            .get("MAX_MONEY")
            .context("currency constants missing MAX_MONEY")?;
        state.money = state.money.saturating_add(amount).min(max_money);
        state.battle_pay_day_money = 0;
        Ok(())
    }

    fn active_battle_pay_day_payout(&self, state: &GameState) -> u32 {
        let amount = state.battle_pay_day_money.min(0x00ff_ffff);
        if state.battle_amulet_coin_active {
            amount.saturating_mul(2).min(0x00ff_ffff)
        } else {
            amount
        }
    }

    fn active_battle_pay_day_money_after_turn(
        &self,
        state: &GameState,
        outcome: &BattleTurnOutcome,
    ) -> u32 {
        let amount = state.battle_pay_day_money.wrapping_add(
            outcome
                .events
                .iter()
                .filter_map(|event| match event {
                    BattleEvent::PayDayMoney { amount, .. } => Some(*amount),
                    _ => None,
                })
                .fold(0_u32, u32::wrapping_add),
        ) & 0x00ff_ffff;
        if state.battle_amulet_coin_active {
            amount.saturating_mul(2).min(0x00ff_ffff)
        } else {
            amount
        }
    }

}
