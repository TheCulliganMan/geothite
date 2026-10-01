impl GameDataSet {
    pub fn owned_decoration_categories(
        &self,
        state: &GameState,
    ) -> Result<Vec<DecorationCategory>> {
        let mut categories = Vec::new();
        for category in &self.decorations.category_order {
            let mut owned = false;
            for decoration in self
                .decorations
                .decorations
                .iter()
                .filter(|decoration| decoration.category == *category)
            {
                if state
                    .flags
                    .is_event_flag_set(&decoration.event_flag)
                    .with_context(|| {
                        format!("check decoration ownership flag {}", decoration.event_flag)
                    })?
                {
                    owned = true;
                    break;
                }
            }
            if owned {
                categories.push(*category);
            }
        }
        Ok(categories)
    }

    pub fn owned_decorations(
        &self,
        state: &GameState,
        category: DecorationCategory,
    ) -> Result<Vec<&DecorationDefinition>> {
        let mut owned = Vec::new();
        for decoration in self
            .decorations
            .decorations
            .iter()
            .filter(|decoration| decoration.category == category)
        {
            if state
                .flags
                .is_event_flag_set(&decoration.event_flag)
                .with_context(|| {
                    format!("check decoration ownership flag {}", decoration.event_flag)
                })?
            {
                owned.push(decoration);
            }
        }
        Ok(owned)
    }

    pub fn set_up_decoration(
        &self,
        state: &mut GameState,
        decoration_id: &str,
        side: Option<DecorationSide>,
    ) -> Result<DecorationActionOutcome> {
        let decoration = self
            .decorations
            .decorations
            .iter()
            .find(|decoration| decoration.id == decoration_id)
            .with_context(|| format!("unknown decoration {decoration_id}"))?;
        anyhow::ensure!(
            state
                .flags
                .is_event_flag_set(&decoration.event_flag)
                .with_context(|| format!(
                    "check decoration ownership flag {}",
                    decoration.event_flag
                ))?,
            "decoration {decoration_id} is not owned"
        );

        let memory = match decoration.category {
            DecorationCategory::Bed => "wDecoBed",
            DecorationCategory::Carpet => "wDecoCarpet",
            DecorationCategory::Plant => "wDecoPlant",
            DecorationCategory::Poster => "wDecoPoster",
            DecorationCategory::GameConsole => "wDecoConsole",
            DecorationCategory::BigDoll => "wDecoBigDoll",
            DecorationCategory::Ornament => match side.context(
                "ornament decoration setup requires an exact left or right side selection",
            )? {
                DecorationSide::Right => "wDecoRightOrnament",
                DecorationSide::Left => "wDecoLeftOrnament",
            },
        };
        anyhow::ensure!(
            decoration.category == DecorationCategory::Ornament || side.is_none(),
            "only ornament decorations accept a side selection"
        );
        let previous = state
            .script_runtime
            .memory
            .get(memory)
            .map(String::as_str)
            .unwrap_or("0");
        if previous == decoration_id {
            return Ok(DecorationActionOutcome::AlreadySetUp {
                decoration: decoration_id.to_string(),
            });
        }

        let previous = previous.to_string();
        if decoration.category == DecorationCategory::Ornament {
            let other_memory = match side.expect("ornament side was validated") {
                DecorationSide::Right => "wDecoLeftOrnament",
                DecorationSide::Left => "wDecoRightOrnament",
            };
            if state
                .script_runtime
                .memory
                .get(other_memory)
                .is_some_and(|other| other == decoration_id)
            {
                state
                    .script_runtime
                    .memory
                    .insert(other_memory.to_string(), "0".to_string());
            }
        }
        state
            .script_runtime
            .memory
            .insert(memory.to_string(), decoration_id.to_string());
        if previous == "0" {
            Ok(DecorationActionOutcome::SetUp {
                decoration: decoration_id.to_string(),
            })
        } else {
            Ok(DecorationActionOutcome::Replaced {
                decoration: decoration_id.to_string(),
                previous,
            })
        }
    }

    pub fn put_away_decoration(
        &self,
        state: &mut GameState,
        category: DecorationCategory,
        side: Option<DecorationSide>,
    ) -> Result<DecorationActionOutcome> {
        let memory = match category {
            DecorationCategory::Bed => "wDecoBed",
            DecorationCategory::Carpet => "wDecoCarpet",
            DecorationCategory::Plant => "wDecoPlant",
            DecorationCategory::Poster => "wDecoPoster",
            DecorationCategory::GameConsole => "wDecoConsole",
            DecorationCategory::BigDoll => "wDecoBigDoll",
            DecorationCategory::Ornament => match side.context(
                "ornament decoration removal requires an exact left or right side selection",
            )? {
                DecorationSide::Right => "wDecoRightOrnament",
                DecorationSide::Left => "wDecoLeftOrnament",
            },
        };
        anyhow::ensure!(
            category == DecorationCategory::Ornament || side.is_none(),
            "only ornament decorations accept a side selection"
        );
        let previous = state
            .script_runtime
            .memory
            .get(memory)
            .map(String::as_str)
            .unwrap_or("0");
        if previous == "0" {
            return Ok(DecorationActionOutcome::NothingToPutAway);
        }
        let previous = previous.to_string();
        anyhow::ensure!(
            self.decorations
                .decorations
                .iter()
                .any(|decoration| decoration.id == previous && decoration.category == category),
            "{memory} contains invalid {category:?} decoration {previous}"
        );
        state
            .script_runtime
            .memory
            .insert(memory.to_string(), "0".to_string());
        Ok(DecorationActionOutcome::PutAway {
            decoration: previous,
        })
    }

    pub fn require_current_map(&self, current_map: &str, requested_map: &str) -> Result<()> {
        if current_map != requested_map {
            anyhow::bail!(
                "script command map mismatch: session is on {current_map}, request was for {requested_map}"
            );
        }
        Ok(())
    }

    pub fn require_no_active_battle(&self, state: &GameState, context: &str) -> Result<()> {
        state.require_no_active_battle().map_err(|error| {
            anyhow::anyhow!("cannot use {context} during an active battle: {error:?}")
        })
    }

    pub fn validate_save_currency(&self, state: &GameState) -> Result<()> {
        validate_save_currency_for_runtime_pack(state, &self.currency_constants).map_err(|error| {
            anyhow::anyhow!(
                "validate Crystal runtime save against compiled pack currency constants: {error:?}"
            )
        })
    }

    pub fn process_overworld_step<S>(
        &self,
        state: &mut GameState,
        map_name: &str,
        movement_mode: MovementMode,
        rng: &mut CrystalRandom<S>,
    ) -> Result<StepEventResult>
    where
        S: DividerSource,
        S::Error: std::fmt::Display,
    {
        let caught_location = self
            .pokegear_landmarks
            .map_to_landmark
            .get(map_name)
            .and_then(|constant| {
                self.pokegear_landmarks
                    .landmarks
                    .iter()
                    .find(|landmark| landmark.constant == *constant)
            })
            .map(|landmark| landmark.id);
        let metadata = self.runtime_map_metadata_for_name(map_name)?;
        let mut next = state.clone();
        let egg_before = next.day_care.egg.clone();
        let result = core_process_overworld_step(
            &mut next,
            &self.step_event_rules,
            &self.growth_rates,
            caught_location,
            crystal_core::systems::step_events::OverworldStepContext {
                movement_mode,
                map_phone_service: metadata.phone_service,
            },
            rng,
        )
        .context("process Day Care and party step events from compiled rules")?;
        if next.day_care.egg != egg_before {
            self.normalize_day_care_egg_species(&mut next)?;
        }
        *state = next;
        Ok(result)
    }

    /// Implements `CountStep`'s leading `CheckSpecialPhoneCall`. An eligible
    /// special call starts before Repel, poison, happiness, Egg, Day Care, and
    /// encounter counters, so the caller must skip the rest of the step when
    /// this returns a dispatch.
    pub fn check_special_phone_call_after_step(
        &self,
        state: &mut GameState,
        map_name: &str,
    ) -> Result<Option<IncomingPhoneCall>> {
        let Some(call_id) = state.script_runtime.special_phone_call.clone() else {
            return Ok(None);
        };
        let rule = self.special_phone_calls.get(&call_id).with_context(|| {
            format!("active special phone call {call_id} is missing from the compiled pack")
        })?;
        let metadata = self.runtime_map_metadata_for_name(map_name)?;
        let eligible = match rule.condition.as_str() {
            "SpecialCallOnlyWhenOutside" => {
                matches!(metadata.environment.as_str(), "TOWN" | "ROUTE")
            }
            "SpecialCallWhereverYouAre" => true,
            other => {
                anyhow::bail!("special phone call {call_id} has unsupported condition {other}")
            }
        };
        if !eligible {
            return Ok(None);
        }
        let contact = self
            .phone_contacts
            .0
            .get(&rule.contact_id)
            .with_context(|| {
                format!(
                    "special phone call {call_id} references missing contact {}",
                    rule.contact_id
                )
            })?;
        if contact.contact_id != rule.contact_id {
            anyhow::bail!(
                "special phone call {call_id} contact key {} does not match record {}",
                rule.contact_id,
                contact.contact_id
            );
        }
        let dispatch = IncomingPhoneCall {
            kind: IncomingPhoneCallKind::Special {
                call_id: call_id.clone(),
            },
            contact_id: rule.contact_id.clone(),
            caller_script: rule.caller_script.clone(),
            receive_script: "Script_ReceivePhoneCall".to_string(),
            delay_frames: 30,
        };
        self.queue_incoming_phone_call(state, map_name, &dispatch);
        Ok(Some(dispatch))
    }

    /// Implements the ordinary overworld `CheckPhoneCall` scheduler. The
    /// receive timer is consumed and restarted before the 50% roll, service
    /// gate, and caller filters, exactly like the ASM.
    pub fn check_ordinary_phone_call<S>(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        rng: &mut CrystalRandom<&mut S>,
    ) -> Result<Option<IncomingPhoneCall>>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        if state.link_session.link_mode != 0 {
            return Ok(None);
        }
        if sample_collision(&session.map, &session.tileset, session.player.tile).is_some_and(
            |sample| {
                matches!(
                    sample.permission,
                    permissions::DOOR
                        | permissions::DOOR_79
                        | permissions::STAIRCASE
                        | permissions::CAVE
                )
            },
        ) {
            return Ok(None);
        }
        if !check_receive_call_timer(state) {
            return Ok(None);
        }

        let chance_roll = rng
            .random(true)
            .map_err(|error| anyhow::anyhow!("CheckPhoneCall chance divider source: {error}"))?
            .value;
        if chance_roll & 0x80 != 0 {
            return Ok(None);
        }

        let metadata = self.runtime_map_metadata_for_name(&session.map.name)?;
        if metadata.phone_service >> 4 != 0 {
            return Ok(None);
        }
        let current_time_mask = match state.time.time_of_day {
            TimeOfDay::Morning => 0x1,
            TimeOfDay::Day => 0x2,
            TimeOfDay::Night => 0x4,
        };
        let ordered_contacts = state
            .script_runtime
            .phone_number_order
            .iter()
            .filter_map(Option::as_ref)
            .collect::<BTreeSet<_>>();
        if ordered_contacts.len() != state.script_runtime.phone_numbers.len()
            || state
                .script_runtime
                .phone_number_order
                .iter()
                .filter_map(Option::as_ref)
                .any(|contact_id| !state.script_runtime.phone_numbers.contains(contact_id))
        {
            anyhow::bail!(
                "saved canonical phone-list order does not match registered phone numbers"
            );
        }
        let mut available = Vec::new();
        for contact_id in state
            .script_runtime
            .phone_number_order
            .iter()
            .filter_map(Option::as_ref)
        {
            let contact = self.phone_contacts.0.get(contact_id).with_context(|| {
                format!("registered phone contact {contact_id} is missing from the compiled pack")
            })?;
            if contact.caller_time_mask & current_time_mask == 0
                || contact.map_constant.as_deref() == Some(metadata.constant.as_str())
            {
                continue;
            }
            let caller_script = contact.caller_script.as_deref().with_context(|| {
                format!("available phone contact {contact_id} has no caller script")
            })?;
            available.push((contact_id.as_str(), caller_script));
        }
        if available.is_empty() {
            return Ok(None);
        }

        rng.random(false)
            .map_err(|error| anyhow::anyhow!("ChooseRandomCaller divider source: {error}"))?;
        let caller_add = rng.state().add;
        let sample = caller_add.rotate_left(4) & 0x1f;
        let selected = usize::from(sample) % available.len();
        let (contact_id, caller_script) = available[selected];
        let dispatch = IncomingPhoneCall {
            kind: IncomingPhoneCallKind::Ordinary,
            contact_id: contact_id.to_string(),
            caller_script: caller_script.to_string(),
            receive_script: "Script_ReceivePhoneCall".to_string(),
            delay_frames: 0,
        };
        self.queue_incoming_phone_call(state, &session.map.name, &dispatch);
        Ok(Some(dispatch))
    }

    fn queue_incoming_phone_call(
        &self,
        state: &mut GameState,
        map_name: &str,
        dispatch: &IncomingPhoneCall,
    ) {
        state
            .script_runtime
            .variables
            .insert("VAR_CALLERID".to_string(), dispatch.contact_id.clone());
        state.script_runtime.memory.insert(
            "wCallerContact + PHONE_CONTACT_SCRIPT2_BANK".to_string(),
            dispatch.caller_script.clone(),
        );
        if dispatch.delay_frames > 0 {
            state
                .script_runtime
                .pending_delays
                .push(ScriptRuntimeDelay {
                    command: "pause".to_string(),
                    parameter: dispatch.delay_frames,
                    frames: dispatch.delay_frames * 2,
                    release_all_objects: false,
                    source_script: ".script@CheckSpecialPhoneCall".to_string(),
                    command_index: 0,
                });
        }
        // Enter the exported wrapper at its first opcode. Its authored
        // reanchormap, RingTwice_StartCall, and dynamic memcall now execute
        // through the ordinary compiled interpreter in source order.
        state.script_runtime.next_script = Some(ScriptLocation {
            origin_map_name: map_name.to_string(),
            script: dispatch.receive_script.clone(),
        });
        state.script_runtime.script_ended = None;
        restart_receive_call_delay(state, true);
    }

    fn start_pokegear_phone_call(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        command: RuntimePokegearPhoneCallCommand,
    ) -> Result<RuntimePokegearPhoneCallOutcome> {
        validate_modpack_payload_token(&command.contact_id, "Pokegear phone contact")?;
        let contact = self
            .phone_contacts
            .0
            .get(&command.contact_id)
            .with_context(|| {
                format!(
                    "Pokegear phone call references missing contact {}",
                    command.contact_id
                )
            })?;
        let metadata = self.runtime_map_metadata_for_name(&session.map.name)?;
        if state.link_session.link_mode != 0 || metadata.phone_service >> 4 != 0 {
            return Ok(RuntimePokegearPhoneCallOutcome {
                contact_id: command.contact_id,
                callback_script: "LoadOutOfAreaScript".to_string(),
                callee_script: None,
            });
        }

        // MakePhoneCallFromPokegear writes wCurCaller before checking whether
        // this contact accepts calls at the current time.
        state
            .script_runtime
            .memory
            .insert("wCurCaller".to_string(), command.contact_id.clone());
        let current_time_mask = match state.time.time_of_day {
            TimeOfDay::Morning => 0x1,
            TimeOfDay::Day => 0x2,
            TimeOfDay::Night => 0x4,
        };
        if contact.callee_time_mask & current_time_mask == 0 {
            return Ok(RuntimePokegearPhoneCallOutcome {
                contact_id: command.contact_id,
                callback_script: "LoadOutOfAreaScript".to_string(),
                callee_script: None,
            });
        }

        let callee_script = if contact.map_constant.as_deref() == Some(metadata.constant.as_str()) {
            "PhoneScript_JustTalkToThem".to_string()
        } else {
            contact.callee_script.clone().with_context(|| {
                format!(
                    "Pokegear phone contact {} has no callee script",
                    command.contact_id
                )
            })?
        };
        state
            .script_runtime
            .memory
            .insert("wPhoneScriptBank".to_string(), callee_script.clone());
        Ok(RuntimePokegearPhoneCallOutcome {
            contact_id: command.contact_id,
            callback_script: "LoadPhoneScriptBank".to_string(),
            callee_script: Some(callee_script),
        })
    }

    fn check_trainer_sight_after_step(
        &self,
        state: &GameState,
        session: &OverworldSession,
    ) -> Result<Option<OverworldInteraction>> {
        let module = self
            .maps
            .get(&session.map.name)
            .with_context(|| format!("missing map module for {}", session.map.name))?;
        let mut eligible_scripts = BTreeSet::new();
        for (source_script, request) in &module.trainer_scripts {
            let defeated = !request.event_flag.is_empty()
                && state
                    .flags
                    .is_event_flag_set(&request.event_flag)
                    .map_err(|error| {
                        anyhow::anyhow!(
                            "check trainer event flag {} on {}: {error}",
                            request.event_flag,
                            session.map.name
                        )
                    })?;
            if !defeated {
                eligible_scripts.insert(source_script.as_str());
            }
        }
        session
            .check_trainer_sight_checked_with_filter(|object| {
                eligible_scripts.contains(object.script.as_str())
            })
            .map_err(|error| {
                anyhow::anyhow!("check trainer sight on {}: {error}", session.map.name)
            })
    }

    /// Applies Crystal's `BGEventJumptable` flag gates to a background-event
    /// interaction selected by the coordinate/facing layer. Conditional and
    /// hidden-item events that fail their flag test are not interactions at
    /// all: they do not play the acknowledgement sound or start a script.
    pub fn resolve_overworld_interaction(
        &self,
        state: &GameState,
        interaction: &OverworldInteraction,
    ) -> Result<Option<OverworldInteraction>> {
        let OverworldInteractionTarget::Background { event_type } = &interaction.target else {
            return Ok(Some(interaction.clone()));
        };
        let module = self.map_module(&interaction.map_name)?;
        let mut matching_events = module.events.bg_events.iter().filter(|event| {
            event.event_type == *event_type
                && event.script == interaction.script
                && background_event_tile_position_checked(event) == Some(interaction.target_tile)
        });
        let event = matching_events.next().with_context(|| {
            format!(
                "background interaction {}:{} at ({}, {}) is not declared by the compiled map",
                interaction.map_name,
                interaction.script,
                interaction.target_tile.x,
                interaction.target_tile.y
            )
        })?;
        if matching_events.next().is_some() {
            anyhow::bail!(
                "background interaction {}:{} at ({}, {}) is declared more than once",
                interaction.map_name,
                interaction.script,
                interaction.target_tile.x,
                interaction.target_tile.y
            );
        }

        let eligible = match event.event_type.as_str() {
            "BGEVENT_READ" | "BGEVENT_UP" | "BGEVENT_DOWN" | "BGEVENT_RIGHT" | "BGEVENT_LEFT" => {
                true
            }
            "BGEVENT_IFSET" | "BGEVENT_IFNOTSET" => {
                let [event_flag, _] = conditional_background_event_payload(
                    &module.scripts,
                    &event.script,
                    &event.event_type,
                )?;
                let is_set = state.flags.is_event_flag_set(event_flag).with_context(|| {
                    format!(
                        "check {} background event {} flag {}",
                        event.event_type, event.script, event_flag
                    )
                })?;
                if event.event_type == "BGEVENT_IFSET" {
                    is_set
                } else {
                    !is_set
                }
            }
            "BGEVENT_ITEM" => {
                let mut pickups = module.script_field_pickups.iter().filter(|pickup| {
                    pickup.command == "hiddenitem" && pickup.source_script == event.script
                });
                let pickup = pickups.next().with_context(|| {
                    format!(
                        "BGEVENT_ITEM background event {} has no hiddenitem payload",
                        event.script
                    )
                })?;
                if pickups.next().is_some() {
                    anyhow::bail!(
                        "BGEVENT_ITEM background event {} has multiple hiddenitem payloads",
                        event.script
                    );
                }
                let event_flag = pickup.event_flag.as_deref().with_context(|| {
                    format!(
                        "BGEVENT_ITEM background event {} has no hiddenitem event flag",
                        event.script
                    )
                })?;
                !state.flags.is_event_flag_set(event_flag).with_context(|| {
                    format!(
                        "check BGEVENT_ITEM background event {} flag {}",
                        event.script, event_flag
                    )
                })?
            }
            // This entry exists solely to copy hidden-item metadata for
            // Itemfinder. The jumptable deliberately returns no interaction.
            "BGEVENT_COPY" => false,
            other => anyhow::bail!(
                "background event {} on {} has unsupported type {}",
                event.script,
                interaction.map_name,
                other
            ),
        };

        Ok(eligible.then(|| interaction.clone()))
    }

    /// Resolves the actual script pointer selected by `TryBGEvent`. The
    /// `conditional_event` source line is data consumed by the background
    /// event engine, not a script opcode to enqueue. Dispatching its wrapper
    /// would both fall through to the target and leave a second queued copy.
    pub fn resolve_overworld_interaction_dispatch(
        &self,
        state: &GameState,
        interaction: &OverworldInteraction,
    ) -> Result<Option<OverworldInteraction>> {
        let Some(mut dispatch) = self.resolve_overworld_interaction(state, interaction)? else {
            return Ok(None);
        };
        if !matches!(
            &interaction.target,
            OverworldInteractionTarget::Background { event_type }
                if matches!(event_type.as_str(), "BGEVENT_IFSET" | "BGEVENT_IFNOTSET")
        ) {
            return Ok(Some(dispatch));
        }

        let module = self.map_module(&interaction.map_name)?;
        let [_, target] = conditional_background_event_payload(
            &module.scripts,
            &interaction.script,
            "conditional",
        )?;
        dispatch.script =
            resolve_background_event_script_target(&module.scripts, &interaction.script, target)?;
        Ok(Some(dispatch))
    }

    fn queue_strength_boulder_landing_script(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
    ) -> Result<bool> {
        for object_id in session
            .standing_strength_boulders_on_pits_checked()
            .map_err(|error| anyhow::anyhow!("scan standing Strength boulders: {error}"))?
        {
            let object_tile = session
                .object_runtime_tile_by_id(&object_id)
                .map_err(|error| {
                    anyhow::anyhow!(
                        "resolve standing Strength boulder {object_id} runtime tile: {error}"
                    )
                })?;
            let landing_warp = session
                .map_events
                .warps
                .iter()
                .enumerate()
                .find_map(|(index, warp)| {
                    (warp_tile_position_checked(warp) == Some(object_tile))
                        .then_some((index + 1) as u16)
                });
            let Some(landing_warp) = landing_warp else {
                continue;
            };
            let Some(entry) = state
                .script_runtime
                .stone_table_entries
                .iter()
                .find(|entry| entry.warp == landing_warp && entry.object_event == object_id)
                .cloned()
            else {
                continue;
            };
            if state.script_runtime.next_script.is_some() {
                anyhow::bail!(
                    "cannot queue Strength boulder landing script {} while another script is pending",
                    entry.script
                );
            }
            state.script_runtime.next_script = Some(ScriptLocation {
                origin_map_name: session.map.name.clone(),
                script: entry.script,
            });
            return Ok(true);
        }
        Ok(false)
    }

    fn queue_whirlpool_forced_movement_script(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
    ) -> Result<bool> {
        if Self::game_state_blocks_overworld_input(state) {
            return Ok(false);
        }
        let on_whirlpool = sample_collision(&session.map, &session.tileset, session.player.tile)
            .is_some_and(|sample| {
                matches!(
                    sample.permission,
                    permissions::WHIRLPOOL | permissions::WHIRLPOOL_2C
                )
            });
        if !on_whirlpool {
            return Ok(false);
        }
        let global = self
            .global_scripts
            .as_ref()
            .context("Whirlpool CheckTile requires compiled global scripts")?;
        anyhow::ensure!(
            global.scripts.contains_key("Script_ForcedMovement"),
            "Whirlpool CheckTile requires exported global root Script_ForcedMovement"
        );
        state.script_runtime.next_script = Some(ScriptLocation {
            origin_map_name: session.map.name.clone(),
            script: "Script_ForcedMovement".to_string(),
        });
        state.script_runtime.script_ended = None;
        Ok(true)
    }

    /// Crystal creates a breeding egg from the mother's pre-evolution, not
    /// from the currently evolved daycare species.  The core step hook does
    /// not own the compiled evolution/learnset catalogs, so normalize the
    /// concrete egg here at the pack boundary.
    fn normalize_day_care_egg_species(&self, state: &mut GameState) -> Result<()> {
        let Some(existing) = state.day_care.egg.clone() else {
            return Ok(());
        };
        if !existing.is_egg {
            anyhow::bail!("Day Care wEggMon lost its egg identity");
        }
        let mut species_id = existing.species.id.clone();
        // `DayCare_InitBreeding` calls GetPreEvolution exactly twice.
        for _ in 0..2 {
            let Some(previous) = self.evolutions.0.iter().find_map(|(source, entries)| {
                entries
                    .iter()
                    .any(|entry| entry.species == species_id)
                    .then_some(source.as_str())
            }) else {
                break;
            };
            species_id = previous.to_string();
        }
        let species = self.pokemon.get(&species_id).with_context(|| {
            format!("Day Care egg species {species_id} is absent from the compiled catalog")
        })?;
        let mut egg = create_pokemon_from_known_dvs(
            species,
            existing.level,
            existing.dvs,
            &self.learnsets,
            &self.moves,
            &self.growth_rates,
        )
        .with_context(|| format!("build Day Care egg species {species_id}"))?;
        // FillMoves runs before InitEggMoves. Crystal then scans the selected
        // donor's four moves in slot order. A move is heritable when it is an
        // explicit Egg Move, a level-up move known by both parents, or a
        // compatible TM/HM. LoadEggMove shifts the oldest slot when full.
        let egg_move_values = self
            .egg_moves
            .get(&species_id)
            .with_context(|| format!("missing Day Care egg-move table for {species_id}"))?
            .as_array()
            .with_context(|| format!("Day Care egg-move table for {species_id} is not an array"))?;
        let explicit_egg_moves = egg_move_values
            .iter()
            .map(|value| {
                value.as_str().with_context(|| {
                    format!("Day Care egg-move entry for {species_id} is not a move id")
                })
            })
            .collect::<Result<BTreeSet<_>>>()?;
        let level_up_moves = level_up_moves_for_species(&self.learnsets, &species_id)
            .with_context(|| format!("load Day Care level-up moves for {species_id}"))?;
        let man = state
            .day_care
            .man
            .pokemon
            .as_ref()
            .context("Day Care egg normalization requires the man resident")?;
        let lady = state
            .day_care
            .lady
            .pokemon
            .as_ref()
            .context("Day Care egg normalization requires the lady resident")?;
        let is_female = |pokemon: &Pokemon| match pokemon.species.gender_ratio {
            254 => true,
            0 | 255 => false,
            ratio => pokemon.dvs.attack.saturating_mul(17) < ratio,
        };
        // `GetBreedmonMovePointer` is distinct from `GetHeritableMoves`: it
        // points at Ditto when present, otherwise at the mother.
        let breedmon_move_pointer = if man.species.id == "DITTO" {
            &man.moves
        } else if lady.species.id == "DITTO" || !is_female(man) {
            &lady.moves
        } else {
            &man.moves
        };
        for candidate in &existing.moves {
            let move_id = candidate.name.as_str();
            if egg.moves.iter().any(|learned| learned.name == move_id) {
                continue;
            }
            let shared_level_up_move = breedmon_move_pointer
                .iter()
                .any(|learned| learned.name == move_id)
                && level_up_moves
                    .iter()
                    .any(|LearnsetEntry(_, learned)| learned == move_id);
            let tmhm_compatible = species
                .tmhm_learnset
                .iter()
                .any(|learned| learned == move_id);
            if !explicit_egg_moves.contains(move_id) && !shared_level_up_move && !tmhm_compatible {
                continue;
            }
            let move_data = self.moves.get(move_id).with_context(|| {
                format!("Day Care inherited move {move_id} is absent from the move catalog")
            })?;
            if egg.moves.len() == 4 {
                egg.moves.remove(0);
            }
            egg.moves.push(LearnedMove {
                name: move_id.to_string(),
                current_pp: move_data.pp,
                pp_ups: 0,
            });
        }
        egg.nickname = existing.nickname.clone();
        egg.item = existing.item.clone();
        egg.status = existing.status.clone();
        egg.is_egg = true;
        egg.pokerus = existing.pokerus;
        egg.caught_data = existing.caught_data.clone();
        egg.mail = existing.mail.clone();
        egg.original_trainer_name = existing.original_trainer_name.clone();
        egg.original_trainer_id = existing.original_trainer_id;
        egg.happiness = existing.happiness;
        state.day_care.egg = Some(egg);
        Ok(())
    }

    pub fn update_clock_from_datetime<S>(
        &self,
        state: &mut GameState,
        date: GameDate,
        hour: u8,
        minute: u8,
        second: u8,
        divider: &mut S,
    ) -> Result<()>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        let mut next = state.clone();
        let previous_day = next.time.current_day;
        if self.server_clock {
            next.time.update_server_datetime(date, hour, minute, second);
        } else {
            next.time.update_from_datetime(date, hour, minute, second);
        }
        let elapsed_days = crystal_days_since(previous_day, next.time.current_day);
        apply_elapsed_daily_time_events(
            &mut next,
            elapsed_days,
            divider,
            "sample Kenji break countdown during daily reset",
        )?;
        *state = next;
        Ok(())
    }

    pub fn set_manual_clock_time<S>(
        &self,
        state: &mut GameState,
        now_date: GameDate,
        now_hour: u8,
        now_minute: u8,
        now_second: u8,
        target: ClockTime,
        divider: &mut S,
    ) -> Result<()>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        if self.server_clock {
            return self.update_clock_from_datetime(
                state, now_date, now_hour, now_minute, now_second, divider,
            );
        }
        let mut next = state.clone();
        let previous_day = next.time.current_day;
        next.time
            .set_manual_time(now_date, now_hour, now_minute, now_second, target);
        let elapsed_days = crystal_days_since(previous_day, next.time.current_day);
        apply_elapsed_daily_time_events(
            &mut next,
            elapsed_days,
            divider,
            "sample Kenji break countdown during manual daily reset",
        )?;
        *state = next;
        Ok(())
    }

    pub fn runtime_spawn_point(&self, spawn_identifier: u16) -> Result<&RuntimeSpawnPoint> {
        anyhow::ensure!(
            spawn_identifier
                < crystal_core::systems::special_routines::CRYSTAL_NUM_SPAWN_POINTS,
            "spawn point {spawn_identifier} is outside Crystal's SpawnPoints table"
        );
        self.runtime_spawn_points
            .get(&spawn_identifier.to_string())
            .with_context(|| format!("compiled game pack missing spawn point {spawn_identifier}"))
    }

    fn home_spawn_identifier(&self) -> Result<u16> {
        let value = self
            .story_event_script_constants
            .global
            .get("SPAWN_HOME")
            .context("compiled pack missing source constant SPAWN_HOME")?;
        let identifier = u16::try_from(*value)
            .with_context(|| format!("compiled source constant SPAWN_HOME={value} is not a u16"))?;
        self.runtime_spawn_point(identifier).with_context(|| {
            format!("compiled source constant SPAWN_HOME={identifier} has no runtime spawn point")
        })?;
        Ok(identifier)
    }

    pub fn runtime_spawn_points(&self) -> &BTreeMap<String, RuntimeSpawnPoint> {
        &self.runtime_spawn_points
    }

    pub fn runtime_spawn_point_for_map_constant(
        &self,
        map_constant: &str,
    ) -> Result<&RuntimeSpawnPoint> {
        let mut matches = self
            .runtime_spawn_points
            .values()
            .filter(|spawn| spawn.map_constant == map_constant);
        let spawn = matches.next().with_context(|| {
            format!("compiled game pack missing spawn point for {map_constant}")
        })?;
        if let Some(other) = matches.next() {
            anyhow::bail!(
                "compiled game pack has multiple spawn points for {map_constant}: {} and {}",
                spawn.identifier,
                other.identifier
            );
        }
        Ok(spawn)
    }

    fn optional_runtime_spawn_identifier_for_map_constant(
        &self,
        map_constant: &str,
    ) -> Result<Option<u16>> {
        let mut matches = self
            .runtime_spawn_points
            .values()
            .filter(|spawn| spawn.map_constant == map_constant);
        let Some(spawn) = matches.next() else {
            return Ok(None);
        };
        if let Some(other) = matches.next() {
            anyhow::bail!(
                "compiled game pack has multiple spawn points for {map_constant}: {} and {}",
                spawn.identifier,
                other.identifier
            );
        }
        Ok(Some(spawn.identifier))
    }

    pub fn map_module(&self, map_name: &str) -> Result<&MapModule> {
        self.maps
            .get(map_name)
            .with_context(|| format!("compiled game pack missing map module {map_name}"))
    }

    fn global_script_module_for(&self, source_script: &str) -> Option<&GlobalScriptModule> {
        let module = self.global_scripts.as_ref()?;
        // Standard scripts are exported under the `StandardScripts` catalog,
        // rather than as top-level global labels.  They still use the global
        // command tables at runtime (for example `PictureBookshelfScript`
        // starts with `farjumptext`), so route them through this module too.
        (module.scripts.contains_key(source_script)
            || self.compiled_standard_script_body(source_script).is_ok())
        .then_some(module)
    }

    pub fn map_declares_script(&self, map_name: &str, script_label: &str) -> Result<bool> {
        Ok(self
            .map_module(map_name)?
            .scripts
            .contains_key(script_label)
            || self
                .global_scripts
                .as_ref()
                .is_some_and(|module| module.scripts.contains_key(script_label)))
    }

    pub fn map_script_labels(&self, map_name: &str) -> Result<BTreeSet<String>> {
        let mut labels = self
            .map_module(map_name)?
            .scripts
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        if let Some(module) = &self.global_scripts {
            labels.extend(module.scripts.keys().cloned());
        }
        Ok(labels)
    }

    fn script_target_origin_map(
        &self,
        current_map: &str,
        caller_map: &str,
        target_script: &str,
    ) -> Result<String> {
        if self.global_script_module_for(target_script).is_some() {
            return Ok(caller_map.to_string());
        }
        if self
            .map_module(current_map)?
            .scripts
            .contains_key(target_script)
        {
            return Ok(current_map.to_string());
        }
        if self
            .map_module(caller_map)?
            .scripts
            .contains_key(target_script)
        {
            return Ok(caller_map.to_string());
        }
        let owners = self
            .maps
            .iter()
            .filter_map(|(map_name, module)| {
                module
                    .scripts
                    .contains_key(target_script)
                    .then_some(map_name.as_str())
            })
            .collect::<Vec<_>>();
        match owners.as_slice() {
            [owner] => Ok((*owner).to_string()),
            // Focused/custom packs may expose a typed control command while
            // deliberately omitting the target body. Preserve its caller
            // origin; pack verification remains responsible for reporting a
            // missing compiled target.
            [] => Ok(caller_map.to_string()),
            _ => anyhow::bail!(
                "script jump target {target_script} is ambiguous across map modules: {}",
                owners.join(", ")
            ),
        }
    }

    pub fn map_tileset_name(&self, map_name: &str) -> Result<&str> {
        Ok(&self.map_module(map_name)?.attributes.tileset_name)
    }

    pub fn map_fishing_group(&self, map_name: &str) -> Result<Option<&str>> {
        Ok(self
            .map_module(map_name)?
            .attributes
            .fishing_group
            .as_deref())
    }

    pub fn map_scene_table(&self, map_name: &str) -> Result<&MapSceneTable> {
        Ok(&self.map_module(map_name)?.scenes)
    }

    pub fn script_text_labels_for_map(&self, map_name: &str) -> Result<BTreeSet<String>> {
        let mut labels = self
            .map_module(map_name)?
            .script_text_bodies
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        if let Some(module) = &self.global_scripts {
            labels.extend(module.script_text_bodies.keys().cloned());
        }
        // Standard-script farjumptext targets live in the canonical ASM text
        // catalog when they are defined outside the StdScripts source file.
        labels.extend(self.asm_text.keys().cloned());
        Ok(labels)
    }

    pub fn script_text_body_for_map(&self, map_name: &str, label: &str) -> Result<&ScriptTextBody> {
        if let Some(body) = self.map_module(map_name)?.script_text_bodies.get(label) {
            return Ok(body);
        }
        if let Some(body) = self
            .global_scripts
            .as_ref()
            .and_then(|module| module.script_text_bodies.get(label))
        {
            return Ok(body);
        }
        anyhow::bail!("script text body {label} is missing on map {map_name}")
    }

    pub fn saved_dig_warp_destination(
        &self,
        state: &GameState,
        context: &str,
    ) -> Result<SavedDigWarpDestination> {
        let Some(map_name) = state.dig_warp_map_name.as_deref() else {
            return core_saved_dig_warp_destination(state, context, &[])
                .map_err(|error| anyhow::anyhow!("{error}"));
        };
        let module = self.map_module(map_name)?;
        core_saved_dig_warp_destination(state, context, &module.events.warps)
            .map_err(|error| anyhow::anyhow!("{error}"))
    }

    pub fn scripted_wild_battle(
        &self,
        map_name: &str,
        source_script: &str,
        startbattle_command_index: usize,
    ) -> Result<&ScriptedWildBattle> {
        self.map_module(map_name)?
            .scripted_wild_battles
            .iter()
            .find(|battle| {
                battle.source_script == source_script
                    && battle.startbattle_command_index == startbattle_command_index
            })
            .with_context(|| {
                format!(
                    "map {map_name} has no scripted wild battle at {source_script}:{startbattle_command_index}"
                )
            })
    }

    pub fn scripted_trainer_battle(
        &self,
        map_name: &str,
        source_script: &str,
        startbattle_command_index: usize,
    ) -> Result<&ScriptedTrainerBattle> {
        self.map_module(map_name)?
            .scripted_trainer_battles
            .iter()
            .find(|battle| {
                battle.source_script == source_script
                    && battle.startbattle_command_index == startbattle_command_index
            })
            .with_context(|| {
                format!(
                    "map {map_name} has no scripted trainer battle at {source_script}:{startbattle_command_index}"
                )
            })
    }

    pub fn scripted_wild_battle_request(
        &self,
        map_name: &str,
        source_script: &str,
        startbattle_command_index: usize,
    ) -> Result<StaticWildBattleRequest> {
        Ok(self
            .scripted_wild_battle(map_name, source_script, startbattle_command_index)?
            .request
            .clone())
    }

    pub fn scripted_trainer_battle_request(
        &self,
        map_name: &str,
        source_script: &str,
        startbattle_command_index: usize,
    ) -> Result<TrainerBattleRequest> {
        if let Some(battle) = self
            .map_module(map_name)?
            .scripted_trainer_battles
            .iter()
            .find(|battle| {
                battle.source_script == source_script
                    && battle.startbattle_command_index == startbattle_command_index
            })
        {
            return Ok(battle.request.clone());
        }
        if let Some(request) =
            self.trainer_table_battle_request(map_name, source_script, startbattle_command_index)?
        {
            return Ok(request.clone());
        }
        anyhow::bail!(
            "map {map_name} has no scripted or trainer-table battle at {source_script}:{startbattle_command_index}"
        )
    }

    pub fn require_scripted_wild_battle_setup(
        &self,
        state: &GameState,
        map_name: &str,
        source_script: &str,
        startbattle_command_index: usize,
    ) -> Result<()> {
        let request =
            self.scripted_wild_battle_request(map_name, source_script, startbattle_command_index)?;
        let level = request.level.to_string();
        for (symbol, expected) in [
            ("wBattleScriptFlags", "128"),
            ("wTempWildMonSpecies", request.species.as_str()),
            ("wCurPartyLevel", level.as_str()),
        ] {
            let actual = state.script_runtime.memory.get(symbol).map(String::as_str);
            anyhow::ensure!(
                actual == Some(expected),
                "compiled startbattle {source_script}:{startbattle_command_index} on {map_name} requires {symbol}={expected}, found {}",
                actual.unwrap_or("<unset>")
            );
        }
        Ok(())
    }

    pub fn require_scripted_trainer_battle_setup(
        &self,
        state: &GameState,
        map_name: &str,
        source_script: &str,
        startbattle_command_index: usize,
    ) -> Result<()> {
        let request = self.scripted_trainer_battle_request(
            map_name,
            source_script,
            startbattle_command_index,
        )?;
        let win_text_pointer = if request.win_text.is_empty() {
            "0"
        } else {
            request.win_text.as_str()
        };
        let loss_text_pointer = if request.loss_text.is_empty() {
            "0"
        } else {
            request.loss_text.as_str()
        };
        for (symbol, expected) in [
            ("wBattleScriptFlags", "129"),
            ("wOtherTrainerClass", request.trainer_class.as_str()),
            ("wOtherTrainerID", request.trainer_id.as_str()),
            ("wWinTextPointer", win_text_pointer),
            ("wLossTextPointer", loss_text_pointer),
        ] {
            let actual = state.script_runtime.memory.get(symbol).map(String::as_str);
            anyhow::ensure!(
                actual == Some(expected),
                "compiled startbattle {source_script}:{startbattle_command_index} on {map_name} requires {symbol}={expected}, found {}",
                actual.unwrap_or("<unset>")
            );
        }
        Ok(())
    }

    fn trainer_table_battle_request(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<Option<&TrainerBattleRequest>> {
        let module = self.map_module(map_name)?;
        let Some(request) = module.trainer_scripts.get(source_script) else {
            return Ok(None);
        };
        let is_trainer_table = module
            .scripts
            .get(source_script)
            .and_then(Value::as_array)
            .and_then(|commands| commands.get(command_index))
            .and_then(|command| command.get("command"))
            .and_then(Value::as_str)
            == Some("trainer");
        Ok(is_trainer_table.then_some(request))
    }

    pub fn start_scripted_wild_battle<S>(
        &self,
        state: &mut GameState,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        startbattle_command_index: usize,
        divider: &mut S,
    ) -> Result<StaticWildBattleStart>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        self.require_current_map(current_map, map_name)?;
        anyhow::ensure!(
            state.pending_static_wild_terminal.is_none(),
            "cannot start a scripted wild battle before the pending static-wild terminal resumes"
        );
        let command =
            RuntimeScriptCommandRef::new(map_name, source_script, startbattle_command_index);
        let dynamic_rock = self.is_exact_rock_smash_dynamic_start_command(&command)?;
        let dynamic_tree = self.is_exact_headbutt_dynamic_start_command(&command)?;
        let dynamic_sweet_scent = self.is_exact_sweet_scent_dynamic_start_command(&command)?;
        let mut request = if dynamic_rock || dynamic_tree || dynamic_sweet_scent {
            anyhow::ensure!(
                state
                    .script_runtime
                    .memory
                    .get("wBattleScriptFlags")
                    .map(String::as_str)
                    == Some("0"),
                "dynamic field encounter startbattle requires randomwildmon to set wBattleScriptFlags=0"
            );
            let species = state
                .script_runtime
                .memory
                .get("wTempWildMonSpecies")
                .cloned()
                .context("dynamic field encounter startbattle is missing wTempWildMonSpecies")?;
            let level: u8 = state
                .script_runtime
                .memory
                .get("wCurPartyLevel")
                .context("dynamic field encounter startbattle is missing wCurPartyLevel")?
                .parse()
                .context("dynamic field encounter wCurPartyLevel is not an exact u8")?;
            anyhow::ensure!(
                species != "0" && level != 0,
                "dynamic field encounter startbattle cannot start from a no-encounter result"
            );
            StaticWildBattleRequest {
                battle_type: if dynamic_tree {
                    "BATTLETYPE_TREE"
                } else if dynamic_sweet_scent
                    && state
                        .script_runtime
                        .memory
                        .get("wBattleType")
                        .map(String::as_str)
                        == Some("BATTLETYPE_ROAMING")
                {
                    "BATTLETYPE_ROAMING"
                } else {
                    "BATTLETYPE_NORMAL"
                }
                .to_string(),
                battle_music: String::new(),
                species,
                level,
                source_script: source_script.to_string(),
            }
        } else {
            self.scripted_wild_battle_request(map_name, source_script, startbattle_command_index)?
        };
        request.battle_music = if request.battle_type == "BATTLETYPE_ROAMING" {
            anyhow::ensure!(
                self.audio
                    .iter()
                    .any(|asset| asset.id == "MUSIC_SUICUNE_BATTLE"),
                "compiled audio is missing roaming battle music MUSIC_SUICUNE_BATTLE"
            );
            "MUSIC_SUICUNE_BATTLE".to_string()
        } else {
            self.wild_battle_music_for_map_time(map_name, state.time.time_of_day)?
        };
        let catch_tutorial = request.battle_type == "BATTLETYPE_TUTORIAL";
        let start = if dynamic_sweet_scent && request.battle_type == "BATTLETYPE_ROAMING" {
            let metadata = self.runtime_map_metadata_for_name(map_name)?;
            let current_map = (
                u8::try_from(metadata.group_id).context("roaming battle map group exceeds byte")?,
                u8::try_from(metadata.map_id).context("roaming battle map number exceeds byte")?,
            );
            let (slot, roaming) = state
                .roaming_pokemon
                .iter()
                .enumerate()
                .find(|(_, roaming)| roaming.species.as_deref() == Some(request.species.as_str()))
                .context("staged roaming species does not identify an active roaming slot")?;
            anyhow::ensure!(
                (roaming.map_group, roaming.map_number) == current_map,
                "staged roaming species is on map {}/{}, not active map {}/{}",
                roaming.map_group,
                roaming.map_number,
                current_map.0,
                current_map.1,
            );
            let slot = u8::try_from(slot).context("roaming slot exceeds byte")?;
            let roaming = roaming.clone();
            let species = self
                .pokemon
                .get(&request.species)
                .with_context(|| format!("unknown roaming species {}", request.species))?;
            let mut rng = CrystalRandom::new(state.random_state, divider);
            let materialized = materialize_staged_roaming_wild_battle_with_rng(
                slot,
                &request.species,
                request.level,
                &roaming,
                species,
                &self.learnsets,
                &self.moves,
                &self.growth_rates,
                &mut rng,
            )
            .map_err(|error| anyhow::anyhow!("materialize staged roaming battle: {error}"))?;
            state.roaming_pokemon[usize::from(slot)] = materialized.roaming_after;
            StaticWildBattleStart {
                battle_type: request.battle_type,
                battle_music: request.battle_music,
                roaming_slot: Some(slot),
                species: request.species,
                level: request.level,
                source_script: request.source_script,
                enemy_party: vec![materialized.enemy_pokemon.clone()],
                enemy_pokemon: materialized.enemy_pokemon,
                random_state_after: rng.state(),
            }
        } else {
            self.static_wild_battle_start(request, state.random_state, divider)
                .with_context(|| {
                    format!(
                        "start scripted wild battle at {map_name}/{source_script}:{startbattle_command_index}"
                    )
                })?
        };
        let resume_command_index = startbattle_command_index
            .checked_add(1)
            .context("scripted wild startbattle command index overflow")?;
        let origin = StaticWildBattleOrigin {
            map_name: map_name.to_string(),
            source_script: source_script.to_string(),
            startbattle_command_index,
            resume_command_index,
        };
        activate_static_wild_battle_start(state, &start, &origin, &self.items).map_err(|error| {
            anyhow::anyhow!(
                "activate scripted wild battle at {map_name}/{source_script}:{startbattle_command_index}: {error:?}"
            )
        })?;
        crate::nuzlocke::register_wild_encounter(
            self.nuzlocke_rules,
            state,
            map_name,
            &start.battle_type,
        );
        state.battle_active_party_index = first_available_battle_party_index(state);
        state.battle_active_enemy_party_index = Some(0);
        state.battle_rewarded_enemy_party_indices.clear();
        state.battle_evolvable_party_indices.clear();
        state.battle_escape_attempts = 0;
        state.battle_pay_day_money = 0;
        set_script_battle_result_accumulator(state);
        state.random_state = start.random_state_after;
        if catch_tutorial {
            // CatchTutorial uses wMomsName as the NAME_LENGTH backup for the
            // player's name and does not restore the original "MOM" bytes.
            state
                .script_runtime
                .variables
                .insert("_moms_name".to_string(), state.player_name.clone());
        }
        Ok(start)
    }

    pub fn start_scripted_wild_battle_in_session<S>(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        map_name: &str,
        source_script: &str,
        startbattle_command_index: usize,
        divider: &mut S,
    ) -> Result<StaticWildBattleStart>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        self.start_scripted_wild_battle(
            state,
            &session.map.name,
            map_name,
            source_script,
            startbattle_command_index,
            divider,
        )
    }

    pub fn start_scripted_trainer_battle(
        &self,
        state: &mut GameState,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        startbattle_command_index: usize,
    ) -> Result<TrainerBattleStartStatus> {
        self.require_current_map(current_map, map_name)?;
        anyhow::ensure!(
            state.pending_static_wild_terminal.is_none(),
            "cannot start a trainer battle before the pending static-wild terminal resumes"
        );
        let request = self.scripted_trainer_battle_request(
            map_name,
            source_script,
            startbattle_command_index,
        )?;
        let is_trainer_table = self
            .trainer_table_battle_request(map_name, source_script, startbattle_command_index)?
            .is_some();
        let start = self.trainer_battle_start(state, request).with_context(|| {
            format!(
                "start scripted trainer battle at {map_name}/{source_script}:{startbattle_command_index}"
            )
        })?;
        let gym_battle_happiness = match &start {
            TrainerBattleStartStatus::Started(started)
                if is_gym_leader_class(&started.trainer_class) =>
            {
                Some(self.happiness_change("HAPPINESS_GYMBATTLE")?)
            }
            TrainerBattleStartStatus::Started(_)
            | TrainerBattleStartStatus::AlreadyDefeated { .. } => None,
        };
        if is_trainer_table {
            set_running_trainer_battle_script(state, false);
        }
        if let Some(changes) = gym_battle_happiness {
            for pokemon in state.storage.party.pokemon.iter_mut().flatten() {
                if pokemon.hp > 0 {
                    core_apply_happiness_change(pokemon, changes);
                }
            }
            state.sync_party_from_storage();
        }
        activate_trainer_battle_start_status(state, &start, &self.items)
            .context("activate scripted trainer battle")?;
        Ok(start)
    }

    pub fn start_scripted_trainer_battle_in_session(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        map_name: &str,
        source_script: &str,
        startbattle_command_index: usize,
    ) -> Result<TrainerBattleStartStatus> {
        self.start_scripted_trainer_battle(
            state,
            &session.map.name,
            map_name,
            source_script,
            startbattle_command_index,
        )
    }

    pub fn resolve_map_trainer_interaction(
        &self,
        state: &mut GameState,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        trainer_command_index: usize,
        defer_battle_start: bool,
    ) -> Result<RuntimeMapTrainerInteractionOutcome> {
        self.require_current_map(current_map, map_name)?;
        let request = self
            .trainer_table_battle_request(map_name, source_script, trainer_command_index)?
            .cloned()
            .with_context(|| {
                format!(
                    "map trainer interaction {map_name}/{source_script}:{trainer_command_index} is not an exact trainer-table command"
                )
            })?;

        // TalkToTrainer writes zero before checking the trainer flag. This is
        // what lets the callback's leading endifjustbattled distinguish an
        // ordinary repeat conversation from the post-battle dispatch.
        set_running_trainer_battle_script(state, false);
        let defeated = !request.event_flag.is_empty()
            && state
                .flags
                .is_event_flag_set(&request.event_flag)
                .with_context(|| {
                    format!(
                        "check map trainer interaction flag {} for {source_script}",
                        request.event_flag
                    )
                })?;
        if defeated {
            anyhow::ensure!(
                !request.callback.is_empty(),
                "defeated map trainer {source_script} has no scripttalkafter callback"
            );
            state.script_runtime.next_script = Some(ScriptLocation {
                origin_map_name: map_name.to_string(),
                script: request.callback.clone(),
            });
            state.script_runtime.script_ended = None;
            return Ok(RuntimeMapTrainerInteractionOutcome::AlreadyDefeated {
                callback: request.callback,
            });
        }

        for (symbol, value) in [
            ("wBattleScriptFlags", "129"),
            ("wOtherTrainerClass", request.trainer_class.as_str()),
            ("wOtherTrainerID", request.trainer_id.as_str()),
            (
                "wWinTextPointer",
                if request.win_text.is_empty() {
                    "0"
                } else {
                    request.win_text.as_str()
                },
            ),
            (
                "wLossTextPointer",
                if request.loss_text.is_empty() {
                    "0"
                } else {
                    request.loss_text.as_str()
                },
            ),
            (
                "wSeenTextPointer",
                if request.seen_text.is_empty() {
                    "0"
                } else {
                    request.seen_text.as_str()
                },
            ),
            (
                "wScriptAfterPointer",
                if request.callback.is_empty() {
                    "0"
                } else {
                    request.callback.as_str()
                },
            ),
        ] {
            state
                .script_runtime
                .memory
                .insert(symbol.to_string(), value.to_string());
        }
        if defer_battle_start {
            return Ok(RuntimeMapTrainerInteractionOutcome::ReadyForSeenText);
        }

        self.start_scripted_trainer_battle(
            state,
            current_map,
            map_name,
            source_script,
            trainer_command_index,
        )
        .map(RuntimeMapTrainerInteractionOutcome::BattleStarted)
    }

    pub fn scripted_wild_battle_terminal(
        &self,
        state: &GameState,
        origin: &RuntimeStaticWildBattleOrigin,
    ) -> Result<RuntimeScriptedWildBattleTerminal> {
        anyhow::ensure!(
            self.saved_static_wild_battle_origin_exists(
                &origin.map_name,
                &origin.source_script,
                origin.startbattle_command_index,
                origin.resume_command_index,
                &origin.battle_type,
                &origin.species,
                origin.level,
            ),
            "scripted wild battle origin is not present in the compiled pack: {} / {}:{} -> {} {} {} level {}",
            origin.map_name,
            origin.source_script,
            origin.startbattle_command_index,
            origin.resume_command_index,
            origin.battle_type,
            origin.species,
            origin.level,
        );
        match &state.battle {
            BattleMemory::Inactive => {
                let pending = state.pending_static_wild_terminal.as_ref().context(
                    "inactive scripted wild completion has no persisted terminal origin",
                )?;
                anyhow::ensure!(
                    pending.origin_map_name == origin.map_name
                        && pending.source_script == origin.source_script
                        && pending.startbattle_command_index == origin.startbattle_command_index
                        && pending.resume_command_index == origin.resume_command_index
                        && pending.battle_type == origin.battle_type
                        && pending.species == origin.species
                        && pending.level == origin.level,
                    "persisted scripted wild terminal does not match the recorded completion origin"
                );
                let result_code = pending.battle_result & 0x3f;
                anyhow::ensure!(
                    result_code != 1,
                    "lost scripted wild battle must resolve through whiteout without resuming its source cursor"
                );
                anyhow::ensure!(
                    result_code == 0 || result_code == 2,
                    "scripted wild terminal has unsupported base battle result {result_code}"
                );
                Ok(RuntimeScriptedWildBattleTerminal {
                    battle_result: pending.battle_result,
                    win_cleanup_applied: pending.win_cleanup_applied,
                })
            }
            BattleMemory::StaticWild { .. } => anyhow::bail!(
                "scripted wild completion cannot run before ExitBattle persists its terminal result"
            ),
            _ => anyhow::bail!(
                "scripted wild completion cannot run while a different battle is active"
            ),
        }
    }

    pub fn complete_scripted_wild_battle<S>(
        &self,
        state: &mut GameState,
        _overworld: &mut OverworldSession,
        current_map: &str,
        origin: &RuntimeStaticWildBattleOrigin,
        terminal: RuntimeScriptedWildBattleTerminal,
        divider: &mut S,
    ) -> Result<()>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        self.require_current_map(current_map, &origin.map_name)?;
        let resolved_terminal = self.scripted_wild_battle_terminal(state, origin)?;
        anyhow::ensure!(
            resolved_terminal == terminal,
            "scripted wild terminal disposition {terminal:?} contradicts authoritative state {resolved_terminal:?}"
        );
        let result_code = terminal.battle_result & 0x3f;
        let requires_win_cleanup = result_code == 0 && !terminal.win_cleanup_applied;
        let pending_pay_day_payout = if requires_win_cleanup {
            state
                .pending_static_wild_terminal
                .as_ref()
                .context("pending scripted wild cleanup lost its terminal record")?
                .pay_day_payout
        } else {
            0
        };
        if requires_win_cleanup {
            // ExitBattle orders CheckPayDay before EvolveAfterBattle and
            // GivePokerus. A faint-victory reward command already performed
            // the evolution and marks this terminal cleaned; capture and the
            // remaining direct-WIN completions cannot create a new level-up
            // evolution candidate at this later cursor boundary.
            self.claim_active_battle_pay_day_money(state, pending_pay_day_payout)?;
            state
                .spread_pokerus_after_battle(divider)
                .map_err(|error| anyhow::anyhow!("post-battle Pokerus divider failed: {error}"))?;
        }
        set_script_battle_result_accumulator(state);
        state.pending_static_wild_terminal = None;
        Ok(())
    }

    pub fn complete_scripted_wild_battle_in_session<S>(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        origin: &RuntimeStaticWildBattleOrigin,
        terminal: RuntimeScriptedWildBattleTerminal,
        divider: &mut S,
    ) -> Result<()>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        let current_map = overworld.map.name.clone();
        self.complete_scripted_wild_battle(
            state,
            overworld,
            &current_map,
            origin,
            terminal,
            divider,
        )
    }

    pub fn scripted_trainer_battle_completion(
        &self,
        map_name: &str,
        source_script: &str,
        startbattle_command_index: usize,
        won: bool,
        can_lose: bool,
    ) -> Result<TrainerBattleCompletion> {
        let request = self.scripted_trainer_battle_request(
            map_name,
            source_script,
            startbattle_command_index,
        )?;
        Ok(TrainerBattleCompletion {
            trainer_id: request.trainer_id,
            trainer_class: request.trainer_class,
            event_flag: request.event_flag,
            won,
            can_lose,
        })
    }

    pub fn complete_scripted_trainer_battle<S>(
        &self,
        state: &mut GameState,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        startbattle_command_index: usize,
        won: bool,
        can_lose: bool,
        divider: &mut S,
    ) -> Result<TrainerBattleCompletionOutcome>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        self.require_current_map(current_map, map_name)?;
        let mut staged_state = state.clone();
        let battle_end = self.active_battle_end_context_on_map(&staged_state, Some(current_map))?;
        let completion = self.scripted_trainer_battle_completion(
            map_name,
            source_script,
            startbattle_command_index,
            won,
            can_lose,
        )?;
        let is_trainer_table = self
            .trainer_table_battle_request(map_name, source_script, startbattle_command_index)?
            .is_some();
        let mut outcome = core_complete_trainer_battle(
            &mut staged_state,
            &self.currency_constants,
            &completion,
            divider,
        )
        .with_context(|| {
                format!(
                    "complete scripted trainer battle at {map_name}/{source_script}:{startbattle_command_index}"
                )
            })?;
        if matches!(staged_state.battle, BattleMemory::Inactive)
            && let Some((battle_type, roaming_slot, enemy, battle_map)) = battle_end
        {
            self.finish_battle_roaming_update_exact(
                &mut staged_state,
                &battle_type,
                roaming_slot,
                &enemy,
                &battle_map,
                divider,
            )?;
        }
        if outcome.continued_after_battle {
            outcome.money_after = staged_state.money;
            set_script_battle_result_accumulator(&mut staged_state);
            if is_trainer_table {
                // StartBattleWithMapTrainerScript writes -1 immediately before
                // scripttalkafter dispatches the trainer table's callback.
                set_running_trainer_battle_script(&mut staged_state, true);
            }
        }
        *state = staged_state;
        Ok(outcome)
    }

    pub fn complete_scripted_trainer_battle_in_session<S>(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        map_name: &str,
        source_script: &str,
        startbattle_command_index: usize,
        won: bool,
        can_lose: bool,
        divider: &mut S,
    ) -> Result<TrainerBattleCompletionOutcome>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        self.complete_scripted_trainer_battle(
            state,
            &session.map.name,
            map_name,
            source_script,
            startbattle_command_index,
            won,
            can_lose,
            divider,
        )
    }

    pub fn gift_pokemon_script(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<&GiftPokemonScript> {
        self.map_module(map_name)?
            .gift_pokemon_scripts
            .iter()
            .find(|gift| gift.source_script == source_script && gift.command_index == command_index)
            .with_context(|| {
                format!(
                    "map {map_name} has no gift Pokemon script at {source_script}:{command_index}"
                )
            })
    }

    pub fn gift_pokemon_request(
        &self,
        state: &GameState,
        map_name: &str,
        source_script: &str,
        command_index: usize,
        original_trainer_name: impl Into<String>,
        original_trainer_id: u16,
        dvs: Dv,
        nickname_accepted: bool,
        nickname: Option<String>,
    ) -> Result<GiftPokemonRequest> {
        let gift = self.gift_pokemon_script(map_name, source_script, command_index)?;
        let custom_identity = match (gift.nickname_label.as_deref(), gift.ot_label.as_deref()) {
            (Some(nickname_label), Some(ot_label)) => Some((
                self.resolve_gift_name_label(map_name, source_script, nickname_label)?,
                self.resolve_gift_ot_label(map_name, source_script, ot_label)?,
            )),
            (None, None) => None,
            _ => anyhow::bail!(
                "gift Pokemon at {map_name}/{source_script}:{command_index} must declare both custom identity labels or neither"
            ),
        };
        if custom_identity.is_some() && (nickname_accepted || nickname.is_some()) {
            anyhow::bail!(
                "gift Pokemon at {map_name}/{source_script}:{command_index} uses authored custom identity and cannot open the player nickname prompt"
            );
        }
        if custom_identity.is_none() {
            match (nickname_accepted, nickname.as_ref()) {
                (false, None) => {}
                (true, Some(_)) if !gift.egg => {}
                (true, _) => anyhow::bail!(
                    "gift Pokemon at {map_name}/{source_script}:{command_index} cannot accept the supplied nickname"
                ),
                (false, Some(_)) => anyhow::bail!(
                    "gift Pokemon at {map_name}/{source_script}:{command_index} refused nickname prompt but supplied nickname"
                ),
            }
        }
        let (nickname, original_trainer_name, original_trainer_id) =
            if let Some((nickname, (ot_name, _))) = custom_identity.as_ref() {
                let ot_id = self
                    .story_event_script_constants
                    .global
                    .get("RANDY_OT_ID")
                    .copied()
                    .context("custom givepoke requires exported source constant RANDY_OT_ID")
                    .and_then(|value| {
                        u16::try_from(value).context("RANDY_OT_ID is outside u16 range")
                    })?;
                (Some(nickname.clone()), ot_name.clone(), ot_id)
            } else {
                (nickname, original_trainer_name.into(), original_trainer_id)
            };
        if nickname.as_deref().is_some_and(str::is_empty) {
            anyhow::bail!(
                "gift Pokemon at {map_name}/{source_script}:{command_index} nickname must be nonempty when accepted"
            );
        }
        let caught_data = if let Some((_, (_, gift_gender))) = custom_identity {
            Some(CaughtData {
                level: 0,
                time_of_day: None,
                original_trainer_gender: gift_gender,
                location: self.source_u8_constant("LANDMARK_GIFT", "custom givepoke")?,
            })
        } else {
            let caught_map = if map_name == "Pokecenter2F" {
                state
                    .backup_warp_map_name
                    .as_deref()
                    .with_context(|| "givepoke on Pokecenter2F requires the source backup map")?
            } else {
                map_name
            };
            let location = u8::try_from(self.pokegear_landmark_for_map(caught_map)?.id)
                .with_context(|| format!("caught landmark for {caught_map} is outside u8 range"))?;
            Some(CaughtData {
                level: if gift.egg {
                    self.source_u8_constant("CAUGHT_EGG_LEVEL", "giveegg")?
                } else {
                    gift.level
                },
                time_of_day: Some(state.time.time_of_day),
                original_trainer_gender: state.player_gender,
                location,
            })
        };
        Ok(GiftPokemonRequest {
            species_id: gift.species_id.clone(),
            level: gift.level,
            held_item_id: gift.held_item_id.clone(),
            nickname,
            original_trainer_name: original_trainer_name.into(),
            original_trainer_id,
            caught_data,
            source_script: gift.source_script.clone(),
            command_index: gift.command_index,
            egg: gift.egg,
            dvs,
        })
    }

    fn resolve_gift_name_label(
        &self,
        map_name: &str,
        source_script: &str,
        label: &str,
    ) -> Result<String> {
        let (name, trailing) = self.resolve_gift_name_label_data(map_name, source_script, label)?;
        anyhow::ensure!(
            trailing.is_empty(),
            "custom gift nickname label {label} has bytes after its terminator"
        );
        Ok(name)
    }

    fn resolve_gift_ot_label(
        &self,
        map_name: &str,
        source_script: &str,
        label: &str,
    ) -> Result<(String, u8)> {
        let (name, trailing) = self.resolve_gift_name_label_data(map_name, source_script, label)?;
        anyhow::ensure!(
            trailing.len() == 1,
            "custom gift OT label {label} must contain exactly one caught-gender byte after its terminator"
        );
        let gender = u8::try_from(parse_script_i32(&trailing[0])?)
            .with_context(|| format!("custom gift OT label {label} gender is outside u8 range"))?;
        anyhow::ensure!(
            gender <= 1,
            "custom gift OT label {label} gender {gender} is outside Crystal range 0..1"
        );
        Ok((name, gender))
    }

    fn resolve_gift_name_label_data(
        &self,
        map_name: &str,
        source_script: &str,
        label: &str,
    ) -> Result<(String, Vec<String>)> {
        let scripts = &self.map_module(map_name)?.scripts;
        let resolved_label = if label.starts_with('.') {
            let parent = script_label_parent(source_script);
            let local = if label.contains('@') {
                anyhow::ensure!(
                    script_label_parent(label) == parent,
                    "custom gift name label {label} crosses ASM parent scope from {source_script}"
                );
                label.to_string()
            } else {
                format!("{label}@{parent}")
            };
            scripts
                .contains_key(&local)
                .then_some(local)
                .with_context(|| {
                    format!("custom gift name label {label} is missing from map {map_name}")
                })?
        } else if scripts.contains_key(label) {
            label.to_string()
        } else {
            anyhow::bail!("custom gift name label {label} is missing from map {map_name}");
        };
        let entries = scripts
            .get(&resolved_label)
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("custom gift name label {resolved_label} is not ASM data"))?;
        let mut name = String::new();
        let mut terminated = false;
        let mut trailing = Vec::new();
        for entry in entries {
            let command = entry
                .get("command")
                .and_then(serde_json::Value::as_str)
                .with_context(|| {
                    format!("custom gift name label {resolved_label} has a malformed command")
                })?;
            if command != "db" {
                anyhow::bail!(
                    "custom gift name label {resolved_label} contains non-db command {command}"
                );
            }
            let args = entry
                .get("args")
                .and_then(serde_json::Value::as_array)
                .with_context(|| {
                    format!("custom gift name label {resolved_label} db has no args")
                })?;
            for arg in args {
                let token = arg.as_str().with_context(|| {
                    format!("custom gift name label {resolved_label} db argument is not a string")
                })?;
                if terminated {
                    trailing.push(token.to_string());
                    continue;
                }
                let literal: String = serde_json::from_str(token).with_context(|| {
                    format!("custom gift name label {resolved_label} requires a quoted ASM string, found {token:?}")
                })?;
                if let Some((before, after)) = literal.split_once('@') {
                    if !after.is_empty() {
                        anyhow::bail!(
                            "custom gift name label {resolved_label} has bytes after its terminator"
                        );
                    }
                    name.push_str(before);
                    if name.is_empty() {
                        anyhow::bail!("custom gift name label {resolved_label} resolves empty");
                    }
                    terminated = true;
                    continue;
                }
                name.push_str(&literal);
            }
        }
        anyhow::ensure!(
            terminated,
            "custom gift name label {resolved_label} has no @ terminator"
        );
        Ok((name, trailing))
    }

    fn source_u8_constant(&self, constant: &str, context: &str) -> Result<u8> {
        let value = self
            .story_event_script_constants
            .global
            .get(constant)
            .copied()
            .with_context(|| format!("{context} requires exported source constant {constant}"))?;
        u8::try_from(value).with_context(|| format!("{constant} is outside u8 range"))
    }

    pub fn grant_gift_pokemon_to_state(
        &self,
        state: &mut GameState,
        request: GiftPokemonRequest,
    ) -> Result<GiftPokemonOutcome> {
        core_grant_gift_pokemon_to_state(
            state,
            &self.pokemon,
            &self.learnsets,
            &self.moves,
            &self.growth_rates,
            &self.items,
            request,
        )
        .map_err(|error| anyhow::anyhow!("grant gift Pokemon: {error:?}"))
    }

    pub fn grant_scripted_gift_pokemon(
        &self,
        state: &mut GameState,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        command_index: usize,
        original_trainer_name: impl Into<String>,
        original_trainer_id: u16,
        dvs: Dv,
        nickname_accepted: bool,
        nickname: Option<String>,
    ) -> Result<GiftPokemonOutcome> {
        self.require_current_map(current_map, map_name)?;
        let request = self.gift_pokemon_request(
            state,
            map_name,
            source_script,
            command_index,
            original_trainer_name,
            original_trainer_id,
            dvs,
            nickname_accepted,
            nickname,
        )?;
        self.grant_gift_pokemon_to_state(state, request)
            .map_err(|error| {
                anyhow::anyhow!(
                    "grant gift Pokemon at {map_name}/{source_script}:{command_index}: {error:?}"
                )
            })
    }

    pub fn grant_scripted_gift_pokemon_in_session(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
        original_trainer_name: impl Into<String>,
        original_trainer_id: u16,
        dvs: Dv,
        nickname_accepted: bool,
        nickname: Option<String>,
    ) -> Result<GiftPokemonOutcome> {
        self.grant_scripted_gift_pokemon(
            state,
            &session.map.name,
            map_name,
            source_script,
            command_index,
            original_trainer_name,
            original_trainer_id,
            dvs,
            nickname_accepted,
            nickname,
        )
    }

    pub fn grant_scripted_gift_pokemon_with_divider_in_session<S>(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
        original_trainer_name: impl Into<String>,
        original_trainer_id: u16,
        nickname_accepted: bool,
        nickname: Option<String>,
        divider: &mut S,
    ) -> Result<GiftPokemonOutcome>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        let dvs = generate_scripted_gift_dvs(state, divider)
            .map_err(|error| anyhow::anyhow!("generate scripted gift Pokemon DVs: {error}"))?;
        self.require_current_map(&session.map.name, map_name)?;
        let mut request = self.gift_pokemon_request(
            state,
            map_name,
            source_script,
            command_index,
            original_trainer_name,
            original_trainer_id,
            dvs,
            nickname_accepted,
            nickname,
        )?;
        let custom_ot = self
            .gift_pokemon_script(map_name, source_script, command_index)?
            .ot_label
            .is_some();
        let boxed_custom_ot = custom_ot
            && !state.storage.party.has_space()
            && state
                .storage
                .has_capture_space_in_box(state.current_pc_box)
                .map_err(|error| anyhow::anyhow!(error))?;
        if boxed_custom_ot {
            request.original_trainer_id = generate_scripted_box_gift_ot_id(state, divider)
                .map_err(|error| {
                    anyhow::anyhow!("generate boxed custom gift Pokemon OT ID: {error}")
                })?;
        }
        self.grant_gift_pokemon_to_state(state, request)
            .map_err(|error| {
                anyhow::anyhow!(
                    "grant gift Pokemon at {map_name}/{source_script}:{command_index}: {error:?}"
                )
            })
    }

    pub fn script_item_grant(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<&ScriptItemGrant> {
        if let Some(module) = self.global_script_module_for(source_script) {
            return find_script_entry(
                &module.script_item_grants,
                map_name,
                "global script item grant",
                source_script,
                command_index,
                |grant| (&grant.source_script, grant.command_index),
            );
        }
        find_script_entry(
            &self.map_module(map_name)?.script_item_grants,
            map_name,
            "script item grant",
            source_script,
            command_index,
            |grant| (&grant.source_script, grant.command_index),
        )
    }

    pub fn script_item_check(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<&ScriptItemAccess> {
        if let Some(module) = self.global_script_module_for(source_script) {
            return find_script_entry(
                &module.script_item_checks,
                map_name,
                "global script item check",
                source_script,
                command_index,
                |access| (&access.source_script, access.command_index),
            );
        }
        find_script_entry(
            &self.map_module(map_name)?.script_item_checks,
            map_name,
            "script item check",
            source_script,
            command_index,
            |access| (&access.source_script, access.command_index),
        )
    }

    pub fn script_item_take(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<&ScriptItemAccess> {
        if let Some(module) = self.global_script_module_for(source_script) {
            return find_script_entry(
                &module.script_item_takes,
                map_name,
                "global script item take",
                source_script,
                command_index,
                |access| (&access.source_script, access.command_index),
            );
        }
        find_script_entry(
            &self.map_module(map_name)?.script_item_takes,
            map_name,
            "script item take",
            source_script,
            command_index,
            |access| (&access.source_script, access.command_index),
        )
    }

    pub fn grant_script_item(
        &self,
        state: &mut GameState,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptItemGrantOutcome> {
        self.require_current_map(current_map, map_name)?;
        let variable_grant = self.script_runtime_command(map_name, source_script, command_index)
            .ok().filter(|command| command.command == "verbosegiveitemvar");
        let mut grant = if let Some(command) = variable_grant {
            let item_id = command.args.first().context("verbosegiveitemvar requires an item")?;
            let variable = command.args.get(1).context("verbosegiveitemvar requires a quantity variable")?;
            let quantity = state.script_runtime.variables.get(variable)
                .with_context(|| format!("verbosegiveitemvar quantity {variable} is unset"))?
                .parse::<u16>().with_context(|| format!("verbosegiveitemvar quantity {variable} is not a u16"))?;
            ScriptItemGrant {
                command: "verbosegiveitem".into(), item_id: item_id.clone(), quantity,
                source_script: source_script.into(), command_index, verbose: true,
            }
        } else {
            self.script_item_grant(map_name, source_script, command_index)?.clone()
        };
        if grant.item_id == crystal_core::systems::script_items::SCRIPT_ITEM_FROM_MEMORY_ID {
            grant.item_id = state
                .script_runtime
                .memory
                .get("wNamedObjectIndex")
                .cloned()
                .context("giveitem ITEM_FROM_MEM requires wNamedObjectIndex")?;
        }
        core_grant_script_item(state, &self.items, grant)
            .map_err(|error| anyhow::anyhow!("grant script item: {error:?}"))
    }

    pub fn grant_script_item_in_session(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptItemGrantOutcome> {
        self.grant_script_item(
            state,
            &session.map.name,
            map_name,
            source_script,
            command_index,
        )
    }

    pub fn check_script_item(
        &self,
        state: &GameState,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptItemCheckOutcome> {
        self.require_current_map(current_map, map_name)?;
        let access = self
            .script_item_check(map_name, source_script, command_index)?
            .clone();
        core_check_script_item(state, &self.items, access)
            .map_err(|error| anyhow::anyhow!("check script item: {error:?}"))
    }

    pub fn check_script_item_in_session(
        &self,
        state: &GameState,
        session: &OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptItemCheckOutcome> {
        self.check_script_item(
            state,
            &session.map.name,
            map_name,
            source_script,
            command_index,
        )
    }

    pub fn take_script_item(
        &self,
        state: &mut GameState,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptItemTakeOutcome> {
        self.require_current_map(current_map, map_name)?;
        let access = self
            .script_item_take(map_name, source_script, command_index)?
            .clone();
        core_take_script_item(state, &self.items, access)
            .map_err(|error| anyhow::anyhow!("take script item: {error:?}"))
    }

    pub fn take_script_item_in_session(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptItemTakeOutcome> {
        self.take_script_item(
            state,
            &session.map.name,
            map_name,
            source_script,
            command_index,
        )
    }

    pub fn script_field_pickup(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<&ScriptFieldPickup> {
        find_script_entry(
            &self.map_module(map_name)?.script_field_pickups,
            map_name,
            "script field pickup",
            source_script,
            command_index,
            |pickup| (&pickup.source_script, pickup.command_index),
        )
    }

    pub fn pickup_script_field_item(
        &self,
        state: &mut GameState,
        session: &mut OverworldSession,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<FieldItemPickupOutcome> {
        self.require_current_map(current_map, map_name)?;
        let pickup = self
            .script_field_pickup(map_name, source_script, command_index)?
            .clone();
        let outcome = core_pickup_script_field_item(state, &self.items, &self.fruit_trees, pickup)
            .map_err(|error| anyhow::anyhow!("pickup script field item: {error:?}"))?;
        session.sync_event_flag_memory(&state.flags);
        if let FieldItemPickupOutcome::Collected { event_flag, .. } = &outcome {
            // Item-ball collection is an explicit object removal boundary,
            // unlike ordinary setevent swaps between alternate loaded NPCs.
            session.hide_loaded_objects_with_event_flag(event_flag);
        }
        Ok(outcome)
    }

    pub fn pickup_script_field_item_in_session(
        &self,
        state: &mut GameState,
        session: &mut OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<FieldItemPickupOutcome> {
        let current_map = session.map.name.clone();
        self.pickup_script_field_item(
            state,
            session,
            &current_map,
            map_name,
            source_script,
            command_index,
        )
    }

    pub fn find_itemfinder_hidden_item(
        &self,
        state: &GameState,
        map_name: &str,
        player_tile: TilePosition,
    ) -> Result<Option<crystal_core::systems::field_items::ItemfinderHiddenItem>> {
        let module = self.map_module(map_name)?;
        self.validate_runtime_map_tile("itemfinder player", map_name, player_tile)?;
        core_find_itemfinder_hidden_item(
            state,
            map_name,
            &module.events.bg_events,
            &module.script_field_pickups,
            player_tile,
        )
        .map_err(|error| anyhow::anyhow!("find itemfinder hidden item on {map_name}: {error:?}"))
    }

    pub fn script_economy_command(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<&ScriptEconomyCommand> {
        if let Some(module) = self.global_script_module_for(source_script) {
            return find_script_entry(
                &module.script_economy_commands,
                map_name,
                "global phone script economy command",
                source_script,
                command_index,
                |command| (&command.source_script, command.command_index),
            );
        }
        find_script_entry(
            &self.map_module(map_name)?.script_economy_commands,
            map_name,
            "script economy command",
            source_script,
            command_index,
            |command| (&command.source_script, command.command_index),
        )
    }

    pub fn apply_script_economy_command(
        &self,
        state: &mut GameState,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptEconomyOutcome> {
        self.require_current_map(current_map, map_name)?;
        let command = self
            .script_economy_command(map_name, source_script, command_index)?
            .clone();
        core_apply_script_economy_command(state, command, &self.currency_constants)
            .map_err(|error| anyhow::anyhow!("apply script economy command: {error:?}"))
    }

    pub fn apply_script_economy_command_in_session(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptEconomyOutcome> {
        self.apply_script_economy_command(
            state,
            &session.map.name,
            map_name,
            source_script,
            command_index,
        )
    }

    pub fn script_phone_command(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<&ScriptPhoneCommand> {
        if let Some(module) = self.global_script_module_for(source_script) {
            return find_script_entry(
                &module.script_phone_commands,
                map_name,
                "global phone script phone command",
                source_script,
                command_index,
                |command| (&command.source_script, command.command_index),
            );
        }
        find_script_entry(
            &self.map_module(map_name)?.script_phone_commands,
            map_name,
            "script phone command",
            source_script,
            command_index,
            |command| (&command.source_script, command.command_index),
        )
    }

    pub fn initialize_permanent_phone_numbers(&self, state: &mut GameState) -> Result<Vec<String>> {
        core_initialize_permanent_phone_numbers(
            state,
            &self.phone_contacts,
            &self.permanent_phone_numbers,
        )
        .map_err(|error| anyhow::anyhow!("initialize permanent phone numbers: {error:?}"))
    }

    pub fn script_swarm_command(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<&ScriptSwarmCommand> {
        if let Some(module) = self.global_script_module_for(source_script) {
            return find_script_entry(
                &module.script_swarm_commands,
                map_name,
                "global phone script swarm command",
                source_script,
                command_index,
                |command| (&command.source_script, command.command_index),
            );
        }
        find_script_entry(
            &self.map_module(map_name)?.script_swarm_commands,
            map_name,
            "script swarm command",
            source_script,
            command_index,
            |command| (&command.source_script, command.command_index),
        )
    }

    fn runtime_map_group_table(&self) -> BTreeMap<String, (u16, u16)> {
        self.runtime_map_metadata
            .values()
            .map(|metadata| {
                (
                    metadata.constant.clone(),
                    (metadata.group_id, metadata.map_id),
                )
            })
            .collect()
    }

    pub fn apply_script_swarm_command(
        &self,
        state: &mut GameState,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptSwarmOutcome> {
        self.require_current_map(current_map, map_name)?;
        let command = self
            .script_swarm_command(map_name, source_script, command_index)?
            .clone();
        core_apply_script_swarm_command(state, command, &self.runtime_map_group_table())
            .map_err(|error| anyhow::anyhow!("apply script swarm command: {error:?}"))
    }

    pub fn apply_script_swarm_command_in_session(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptSwarmOutcome> {
        self.apply_script_swarm_command(
            state,
            &session.map.name,
            map_name,
            source_script,
            command_index,
        )
    }

    pub fn apply_script_phone_command(
        &self,
        state: &mut GameState,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        command_index: usize,
        inputs: ScriptPhoneInputs,
    ) -> Result<ScriptPhoneOutcome> {
        self.require_current_map(current_map, map_name)?;
        let command = self
            .script_phone_command(map_name, source_script, command_index)?
            .clone();
        core_apply_script_phone_command(
            state,
            command,
            &self.phone_contacts,
            &self.permanent_phone_numbers,
            inputs,
        )
        .map_err(|error| anyhow::anyhow!("apply script phone command: {error:?}"))
    }

    pub fn apply_script_phone_command_in_session(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
        inputs: ScriptPhoneInputs,
    ) -> Result<ScriptPhoneOutcome> {
        self.apply_script_phone_command(
            state,
            &session.map.name,
            map_name,
            source_script,
            command_index,
            inputs,
        )
    }

    pub fn script_flag_command(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<&ScriptFlagCommand> {
        if let Some(module) = self.global_script_module_for(source_script) {
            return find_script_entry(
                &module.script_flag_commands,
                map_name,
                "global phone script flag command",
                source_script,
                command_index,
                |command| (&command.source_script, command.command_index),
            );
        }
        find_script_entry(
            &self.map_module(map_name)?.script_flag_commands,
            map_name,
            "script flag command",
            source_script,
            command_index,
            |command| (&command.source_script, command.command_index),
        )
    }

    pub fn apply_script_flag_mutation(
        &self,
        state: &mut GameState,
        session: &mut OverworldSession,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptFlagMutationOutcome> {
        self.require_current_map(current_map, map_name)?;
        let command = self
            .script_flag_command(map_name, source_script, command_index)?
            .clone();
        let outcome = core_apply_script_flag_mutation(state, command)
            .map_err(|error| anyhow::anyhow!("apply script flag mutation: {error:?}"))?;
        session.sync_event_flag_memory(&state.flags);
        Ok(outcome)
    }

    pub fn apply_script_flag_mutation_in_session(
        &self,
        state: &mut GameState,
        session: &mut OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptFlagMutationOutcome> {
        let current_map = session.map.name.clone();
        self.apply_script_flag_mutation(
            state,
            session,
            &current_map,
            map_name,
            source_script,
            command_index,
        )
    }

    pub fn check_script_flag(
        &self,
        state: &mut GameState,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptFlagCheckOutcome> {
        self.require_current_map(current_map, map_name)?;
        let command = self
            .script_flag_command(map_name, source_script, command_index)?
            .clone();
        let outcome = core_check_script_flag(state, command)
            .map_err(|error| anyhow::anyhow!("check script flag: {error:?}"))?;
        let value = u8::from(outcome.set).to_string();
        state.script_runtime.script_value = Some(value.clone());
        // Specials and compiled conditionals read the same script variable.
        state.script_runtime.variables.insert("_value".to_string(), value);
        Ok(outcome)
    }

    pub fn check_script_flag_in_session(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptFlagCheckOutcome> {
        self.check_script_flag(
            state,
            &session.map.name,
            map_name,
            source_script,
            command_index,
        )
    }

    pub fn script_scene_command(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<&ScriptSceneCommand> {
        if let Some(module) = self.global_script_module_for(source_script) {
            return find_script_entry(
                &module.script_scene_commands,
                map_name,
                "global phone script scene command",
                source_script,
                command_index,
                |command| (&command.source_script, command.command_index),
            );
        }
        find_script_entry(
            &self.map_module(map_name)?.script_scene_commands,
            map_name,
            "script scene command",
            source_script,
            command_index,
            |command| (&command.source_script, command.command_index),
        )
    }

    pub fn apply_script_scene_command(
        &self,
        state: &mut GameState,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptSceneOutcome> {
        self.require_current_map(current_map, map_name)?;
        let command = self
            .script_scene_command(map_name, source_script, command_index)?
            .clone();
        let source_scene_table = self.map_scene_table(map_name)?;
        if !source_scene_table.scenes.is_empty() {
            state
                .scenes
                .enter_map(map_name, source_scene_table)
                .map_err(|error| {
                    anyhow::anyhow!("enter scene context for {map_name}: {error:?}")
                })?;
        }
        let (target_map_name, scene_table) = if let Some(target_map_id) = command.map_id.as_deref()
        {
            let target_map_name = self.map_name_for_constant(target_map_id).with_context(|| {
                format!("script scene command references missing map id {target_map_id}")
            })?;
            let target_scene_table = self.map_scene_table(&target_map_name)?;
            (Some(target_map_name), target_scene_table)
        } else {
            (None, source_scene_table)
        };
        let outcome = core_apply_script_scene_command(
            state,
            map_name,
            target_map_name.as_deref(),
            scene_table,
            command,
        )
        .map_err(|error| anyhow::anyhow!("apply script scene command: {error:?}"))?;
        if matches!(outcome.command.as_str(), "checkscene" | "checkmapscene") {
            state.script_runtime.script_value = Some(outcome.scene_index.to_string());
        }
        Ok(outcome)
    }

    pub fn apply_script_scene_command_in_session(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptSceneOutcome> {
        self.apply_script_scene_command(
            state,
            &session.map.name,
            map_name,
            source_script,
            command_index,
        )
    }

    pub fn script_block_change(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<&ScriptBlockChange> {
        if let Some(module) = self.global_script_module_for(source_script) {
            return find_script_entry(
                &module.script_block_changes,
                map_name,
                "global script block change",
                source_script,
                command_index,
                |change| (&change.source_script, change.command_index),
            );
        }
        find_script_entry(
            &self.map_module(map_name)?.script_block_changes,
            map_name,
            "script block change",
            source_script,
            command_index,
            |change| (&change.source_script, change.command_index),
        )
    }

    pub fn apply_script_block_change(
        &self,
        state: &mut GameState,
        session: &mut OverworldSession,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptBlockChangeOutcome> {
        self.require_current_map(current_map, map_name)?;
        let change = self
            .script_block_change(map_name, source_script, command_index)?
            .clone();
        let outcome = core_apply_script_block_change(&mut session.map, change)
            .map_err(|error| anyhow::anyhow!("apply script block change: {error:?}"))?;
        state
            .map_block_overrides
            .entry(outcome.map_name.clone())
            .or_default()
            .insert((outcome.metatile_x, outcome.metatile_y), outcome.block_id);
        Ok(outcome)
    }

    pub fn apply_script_block_change_in_session(
        &self,
        state: &mut GameState,
        session: &mut OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptBlockChangeOutcome> {
        let current_map = session.map.name.clone();
        self.apply_script_block_change(
            state,
            session,
            &current_map,
            map_name,
            source_script,
            command_index,
        )
    }

    pub fn script_audio_command(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<&ScriptAudioCommand> {
        if let Some(module) = self.global_script_module_for(source_script) {
            return find_script_entry(
                &module.script_audio_commands,
                map_name,
                "global phone script audio command",
                source_script,
                command_index,
                |command| (&command.source_script, command.command_index),
            );
        }
        find_script_entry(
            &self.map_module(map_name)?.script_audio_commands,
            map_name,
            "script audio command",
            source_script,
            command_index,
            |command| (&command.source_script, command.command_index),
        )
    }

    pub fn apply_script_audio_command(
        &self,
        state: &mut GameState,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        command_index: usize,
        music_ids: &BTreeSet<String>,
        sound_effect_ids: &BTreeSet<String>,
        cry_ids: &BTreeSet<String>,
    ) -> Result<ScriptAudioCue> {
        self.require_current_map(current_map, map_name)?;
        let command = self
            .script_audio_command(map_name, source_script, command_index)?
            .clone();
        let cry_by_species = self.cry_by_species();
        core_apply_script_audio_command(
            state,
            command,
            music_ids,
            sound_effect_ids,
            cry_ids,
            &self.pokemon,
            &cry_by_species,
        )
        .map_err(|error| anyhow::anyhow!("apply script audio command: {error:?}"))
    }

    pub fn apply_script_audio_command_in_session(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
        music_ids: &BTreeSet<String>,
        sound_effect_ids: &BTreeSet<String>,
        cry_ids: &BTreeSet<String>,
    ) -> Result<ScriptAudioCue> {
        self.apply_script_audio_command(
            state,
            &session.map.name,
            map_name,
            source_script,
            command_index,
            music_ids,
            sound_effect_ids,
            cry_ids,
        )
    }

    pub fn script_map_command(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<&ScriptMapCommand> {
        if let Some(module) = self.global_script_module_for(source_script) {
            return find_script_entry(
                &module.script_map_commands,
                map_name,
                "global phone script map command",
                source_script,
                command_index,
                |command| (&command.source_script, command.command_index),
            );
        }
        find_script_entry(
            &self.map_module(map_name)?.script_map_commands,
            map_name,
            "script map command",
            source_script,
            command_index,
            |command| (&command.source_script, command.command_index),
        )
    }

    pub fn apply_script_map_command(
        &self,
        state: &mut GameState,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptMapAction> {
        self.require_current_map(current_map, map_name)?;
        let command = self
            .script_map_command(map_name, source_script, command_index)?
            .clone();
        if command.command == "warpcheck" {
            anyhow::bail!("apply script warpcheck requires a live overworld session");
        }
        if command.command == "reloadmapafterbattle" {
            anyhow::bail!(
                "apply script reloadmapafterbattle requires a live overworld session and divider"
            );
        }
        core_apply_script_map_command(state, command, &self.map_ids())
            .map_err(|error| anyhow::anyhow!("apply script map command: {error:?}"))
    }

    pub fn apply_script_map_command_in_session(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptMapAction> {
        if self
            .script_map_command(map_name, source_script, command_index)?
            .command
            == "reloadmapafterbattle"
        {
            anyhow::bail!("apply script reloadmapafterbattle requires a recorded divider boundary");
        }
        let mut divider = ReplayDivider::new([]);
        self.apply_script_map_command_with_divider_in_session(
            state,
            session,
            map_name,
            source_script,
            command_index,
            &mut divider,
        )
    }

    pub fn apply_script_map_command_with_divider_in_session<S>(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
        divider: &mut S,
    ) -> Result<ScriptMapAction>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        self.require_current_map(&session.map.name, map_name)?;
        let command = self
            .script_map_command(map_name, source_script, command_index)?
            .clone();
        let mut staged_state = state.clone();
        if command.command == "reloadmapafterbattle" {
            let raw_flags = staged_state
                .script_runtime
                .memory
                .get("wBattleScriptFlags")
                .map(String::as_str)
                .unwrap_or("0");
            let battle_script_flags = raw_flags.parse::<u8>().with_context(|| {
                format!("saved wBattleScriptFlags value '{raw_flags}' is not a byte")
            })?;
            staged_state
                .script_runtime
                .memory
                .insert("wBattleScriptFlags".to_string(), "0".to_string());
            if staged_state.battle_result & 0x3f == 1 {
                let action = ScriptMapAction::BattleWhiteout {
                    source_script: command.source_script,
                    command_index: command.command_index,
                };
                *state = staged_state;
                return Ok(action);
            }
            if battle_script_flags & 0x80 != 0 {
                let has_phone_service =
                    self.runtime_map_metadata_for_name(map_name)?.phone_service >> 4 == 0;
                if staged_state.script_runtime.map_reentry_script.is_none() && has_phone_service {
                    if let Some(selection) = core_select_mom_purchase(
                        &mut staged_state,
                        &self.battle_reward_rules,
                        divider,
                    )
                    .map_err(|error| anyhow::anyhow!("select Mom postbattle purchase: {error}"))?
                    {
                        let delivered = match selection.rule.kind {
                            MomPurchaseKind::Item => {
                                let item =
                                    self.items.get(&selection.rule.target).with_context(|| {
                                        format!(
                                            "Mom purchase references missing item {}",
                                            selection.rule.target
                                        )
                                    })?;
                                staged_state.bag.add_pc_item(item, 1).map_err(|error| {
                                    anyhow::anyhow!("deliver Mom PC item: {error}")
                                })?
                            }
                            MomPurchaseKind::Doll => {
                                let flag = selection.rule.decoration_flag.as_deref().with_context(
                                    || {
                                        format!(
                                            "Mom doll {} has no decoration flag",
                                            selection.rule.target
                                        )
                                    },
                                )?;
                                staged_state
                                    .flags
                                    .set_event_flag(flag, true)
                                    .with_context(|| format!("deliver Mom doll flag {flag}"))?;
                                true
                            }
                        };
                        if delivered {
                            let result_script = if selection.rule.decoration_flag.is_some() {
                                ".DollScript@Mom_GetScriptPointer"
                            } else {
                                ".ItemScript@Mom_GetScriptPointer"
                            };
                            staged_state.script_runtime.map_reentry_script = Some(ScriptLocation {
                                origin_map_name: map_name.to_string(),
                                script: result_script.to_string(),
                            });
                            staged_state.pending_mom_purchase =
                                Some(crystal_core::state::PendingMomPurchase {
                                    progression: selection.progression,
                                    selected_index: selection.selected_index,
                                    cost: selection.rule.cost,
                                    target: selection.rule.target,
                                    decoration_flag: selection.rule.decoration_flag,
                                });
                        }
                    }
                }
            } else if staged_state.battle_result & 0x80 != 0
                && staged_state.script_runtime.map_reentry_script.is_none()
            {
                staged_state.script_runtime.map_reentry_script = Some(ScriptLocation {
                    origin_map_name: map_name.to_string(),
                    script: "Script_SpecialBillCall".to_string(),
                });
            }
        }
        let action = core_apply_script_map_command(&mut staged_state, command, &self.map_ids())
            .map_err(|error| anyhow::anyhow!("apply script map command: {error:?}"))?;
        if let ScriptMapAction::WarpCheck {
            source_script,
            command_index,
        } = &action
        {
            if let Some(trigger) = session
                .check_warp_checked()
                .map_err(|error| anyhow::anyhow!("check script warp: {error:?}"))?
            {
                let transition =
                    self.resolve_warp_transition_with_state(&mut staged_state, &trigger)?;
                staged_state.script_runtime.pending_script_warp = Some(ScriptWarpRequest {
                    target_map: transition.destination.map_name,
                    tile: transition.destination.tile,
                    facing: None,
                    source_script: source_script.clone(),
                    command_index: *command_index,
                });
            }
        }
        *state = staged_state;
        Ok(action)
    }

    pub fn complete_pending_script_warp(
        &self,
        state: &mut GameState,
        session: &mut OverworldSession,
    ) -> Result<ScriptWarpRequest> {
        let request = state
            .script_runtime
            .pending_script_warp
            .clone()
            .with_context(|| "cannot execute script warp without a pending script warp")?;
        apply_script_warp_arrival_facing(&mut session.player, &request);
        complete_pending_script_warp(state, &request)
            .map_err(|error| anyhow::anyhow!("complete pending script warp: {error:?}"))
    }

    pub fn transition_pending_script_warp(
        &self,
        state: &mut GameState,
        session: &mut OverworldSession,
        music_ids: &BTreeSet<String>,
    ) -> Result<ScriptWarpRequest> {
        let request = state
            .script_runtime
            .pending_script_warp
            .clone()
            .with_context(|| "cannot execute script warp without a pending script warp")?;
        let frame = session.frame;
        let mode = MovementMode::Normal;
        *session = self.overworld_session_for_traversal(
            &request.target_map,
            request.tile,
            frame,
            mode.traversal_state(),
        )?;
        session.player.mode = mode;
        begin_map_object_setup(session, state);
        clear_transient_map_object_context(state, session);
        reset_map_bike_flags(state)?;
        // EnterMap arms wWildEncounterCooldown before running map setup.
        // CheckWildEncounterCooldown permits the fifth completed step after
        // decrementing 1 -> 0, so this state must be persisted outside the
        // transient OverworldSession.
        state.wild_encounter_cooldown = 5;
        self.complete_pending_script_warp(state, session)?;
        apply_state_block_overrides(session, state)?;
        let mode = self.map_entry_movement_mode(state, session, mode)?;
        session.player.mode = mode;
        self.sync_current_map_music(state, &request.target_map, mode, music_ids)?;
        self.sync_current_map_scene(state, &request.target_map)?;
        self.apply_map_setup_callbacks(state, session, &request.target_map, "MAPSETUP_WARP")?;
        finish_map_object_setup(session, state)?;
        let callback_mode = self.map_entry_movement_mode(state, session, session.player.mode)?;
        if callback_mode != session.player.mode {
            session.player.mode = callback_mode;
            self.sync_current_map_music(state, &request.target_map, callback_mode, music_ids)?;
        }
        self.commit_overworld_snapshot(state, session, SpawnMemoryUpdate::Preserve);
        Ok(request)
    }

    pub fn script_text_command(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<&ScriptTextCommand> {
        if let Some(module) = self.global_script_module_for(source_script) {
            return find_script_entry(
                &module.script_text_commands,
                map_name,
                "global phone script text command",
                source_script,
                command_index,
                |command| (&command.source_script, command.command_index),
            );
        }
        find_script_entry(
            &self.map_module(map_name)?.script_text_commands,
            map_name,
            "script text command",
            source_script,
            command_index,
            |command| (&command.source_script, command.command_index),
        )
    }

    pub fn apply_script_text_command(
        &self,
        state: &mut GameState,
        _current_map: &str,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptTextAction> {
        let command = self
            .script_text_command(map_name, source_script, command_index)?
            .clone();
        let text_labels = self.script_text_labels_for_map(map_name)?;
        core_apply_script_text_command(state, command, &text_labels)
            .map_err(|error| anyhow::anyhow!("apply script text command: {error:?}"))
    }

    pub fn apply_script_text_command_in_session(
        &self,
        state: &mut GameState,
        session: &mut OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptTextAction> {
        let command = self
            .script_text_command(map_name, source_script, command_index)?
            .clone();
        let text_labels = self.script_text_labels_for_map(map_name)?;
        let action = core_resolve_script_text_command(command.clone(), &text_labels)
            .map_err(|error| anyhow::anyhow!("resolve script text command: {error:?}"))?;
        if let ScriptTextAction::Write {
            face_player: true,
            source_script,
            command_index,
            ..
        } = &action
        {
            self.require_current_map(&session.map.name, map_name)?;
            let face_player = ScriptObjectCommand {
                command: "faceplayer".to_string(),
                object_id: None,
                target_object_id: None,
                x: None,
                y: None,
                direction: None,
                movement: None,
                emote: None,
                duration: None,
                source_script: source_script.clone(),
                command_index: *command_index,
            };
            core_apply_script_object_mutation(state, session, &face_player).map_err(|error| {
                anyhow::anyhow!("apply jumptextfaceplayer object facing: {error:?}")
            })?;
        }
        core_apply_script_text_command(state, command, &text_labels)
            .map_err(|error| anyhow::anyhow!("apply script text command: {error:?}"))?;
        Ok(action)
    }

    pub fn script_variable_command(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<&ScriptVariableCommand> {
        if let Some(module) = self.global_script_module_for(source_script) {
            return find_script_entry(
                &module.script_variable_commands,
                map_name,
                "global phone script variable command",
                source_script,
                command_index,
                |command| (&command.source_script, command.command_index),
            );
        }
        find_script_entry(
            &self.map_module(map_name)?.script_variable_commands,
            map_name,
            "script variable command",
            source_script,
            command_index,
            |command| (&command.source_script, command.command_index),
        )
    }

    pub fn apply_script_variable_command(
        &self,
        state: &mut GameState,
        _current_map: &str,
        map_name: &str,
        source_script: &str,
        command_index: usize,
        time_of_day: Option<TimeOfDay>,
    ) -> Result<ScriptVariableOutcome> {
        let command = self
            .script_variable_command(map_name, source_script, command_index)?
            .clone();
        core_apply_script_variable_command(state, command, time_of_day)
            .map_err(|error| anyhow::anyhow!("apply script variable command: {error:?}"))
    }

    pub fn apply_script_variable_command_now(
        &self,
        state: &mut GameState,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptVariableOutcome> {
        let time_of_day = state.time.time_of_day;
        self.apply_script_variable_command(
            state,
            current_map,
            map_name,
            source_script,
            command_index,
            Some(time_of_day),
        )
    }

    pub fn apply_script_variable_command_now_in_session(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptVariableOutcome> {
        let command = self.script_variable_command(map_name, source_script, command_index)?;
        if command.command == "loadvar" && command.target.as_deref() == Some("VAR_MOVEMENT") {
            anyhow::bail!(
                "loadvar VAR_MOVEMENT requires a mutable overworld session so wPlayerState stays authoritative"
            );
        }
        if command.command == "readvar" {
            let derived = match command.target.as_deref() {
                Some("VAR_FACING") => {
                    Some(direction_script_token(session.player.facing).to_string())
                }
                Some("VAR_WEEKDAY") => Some(state.time.day_of_week.to_string()),
                Some("VAR_HOUR") => Some(state.time.registers.hours.to_string()),
                Some("VAR_XCOORD") => Some(session.player.tile.x.to_string()),
                Some("VAR_YCOORD") => Some(session.player.tile.y.to_string()),
                Some("VAR_BLUECARDBALANCE") => Some(state.blue_card_balance.to_string()),
                Some("VAR_DEXCAUGHT") => Some(state.pokedex.caught_species.len().to_string()),
                Some("VAR_MAPGROUP") => Some(
                    self.runtime_map_metadata_for_name(&session.map.name)?
                        .group_id
                        .to_string(),
                ),
                Some("VAR_ENVIRONMENT") => {
                    Some(self.map_environment(&session.map.name)?.to_string())
                }
                Some("VAR_BOXSPACE") => {
                    let occupied = state
                        .storage
                        .pc_boxes
                        .get(state.current_pc_box)
                        .map_or(0, PcBox::filled_slots);
                    Some((MAX_BOX_MONS - occupied).to_string())
                }
                Some("VAR_CONTESTMINUTES") => {
                    Some(state.bug_contest.timer_minutes_remaining.to_string())
                }
                Some("VAR_SPECIALPHONECALL") => Some(
                    match state.script_runtime.special_phone_call.as_ref() {
                        Some(call_id) => self
                            .special_phone_calls
                            .get(call_id)
                            .with_context(|| {
                                format!(
                                    "active special phone call {call_id} is missing from the compiled pack"
                                )
                            })?
                            .value,
                        None => 0,
                    }
                    .to_string(),
                ),
                Some("VAR_KENJI_BREAK") => Some(state.kenji_break_timer.to_string()),
                Some("VAR_MOVEMENT") => {
                    Some(movement_mode_script_byte(session.player.mode).to_string())
                }
                Some("VAR_PARTYCOUNT") => Some(
                    state
                        .storage
                        .party
                        .pokemon
                        .iter()
                        .filter(|pokemon| pokemon.is_some())
                        .count()
                        .to_string(),
                ),
                Some("VAR_BADGES") => Some(
                    state
                        .badges
                        .johto
                        .iter()
                        .chain(state.badges.kanto.iter())
                        .filter(|badge| **badge)
                        .count()
                        .to_string(),
                ),
                Some("VAR_UNOWNCOUNT") => Some(state.pokedex.unown_count().to_string()),
                _ => None,
            };
            if let Some(value) = derived {
                state
                    .script_runtime
                    .variables
                    .insert(command.target.clone().unwrap_or_default(), value);
            }
        } else if command.command == "readmem" {
            let derived = match command.target.as_deref() {
                Some("wParkBallsRemaining") => {
                    Some(state.bug_contest.park_balls_remaining.to_string())
                }
                _ => None,
            };
            if let Some(value) = derived {
                state
                    .script_runtime
                    .memory
                    .insert(command.target.clone().unwrap_or_default(), value);
            }
        }
        self.apply_script_variable_command_now(
            state,
            &session.map.name,
            map_name,
            source_script,
            command_index,
        )
    }

    pub fn apply_script_variable_command_now_in_mut_session(
        &self,
        state: &mut GameState,
        session: &mut OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptVariableOutcome> {
        let command = self
            .script_variable_command(map_name, source_script, command_index)?
            .clone();
        if command.command != "loadvar" || command.target.as_deref() != Some("VAR_MOVEMENT") {
            return self.apply_script_variable_command_now_in_session(
                state,
                session,
                map_name,
                source_script,
                command_index,
            );
        }

        let raw_value = command.value_tokens.join(" ");
        let mode = movement_mode_from_script_value(&raw_value)?;
        let outcome = self.apply_script_variable_command_now(
            state,
            &session.map.name,
            map_name,
            source_script,
            command_index,
        )?;
        session.player.mode = mode;
        self.commit_overworld_snapshot(state, session, SpawnMemoryUpdate::Preserve);
        Ok(outcome)
    }

    pub fn script_control_command(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<&ScriptControlCommand> {
        if let Some(module) = self.global_script_module_for(source_script) {
            return find_script_entry(
                &module.script_control_commands,
                map_name,
                "global phone script control command",
                source_script,
                command_index,
                |command| (&command.source_script, command.command_index),
            );
        }
        find_script_entry(
            &self.map_module(map_name)?.script_control_commands,
            map_name,
            "script control command",
            source_script,
            command_index,
            |command| (&command.source_script, command.command_index),
        )
    }

    pub fn apply_script_control_command(
        &self,
        state: &mut GameState,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptControlAction> {
        let command = self
            .script_control_command(map_name, source_script, command_index)?
            .clone();
        // Control opcodes mutate the retained script cursor/stack, not map
        // objects. Source scripts routinely execute sjump/if/end after a
        // warp has already installed the destination map (Battle Tower's
        // prize handoff is one canonical case).
        let constants = self.script_numeric_constants();
        let action = core_apply_script_control_command(state, map_name, command, &constants)
            .map_err(|error| anyhow::anyhow!("apply script control command: {error:?}"))?;
        if let ScriptControlAction::Jump {
            target_script,
            deferred,
            ..
        } = &action
        {
            let target_origin =
                self.script_target_origin_map(current_map, map_name, target_script)?;
            let location = if *deferred {
                state
                    .script_runtime
                    .deferred_scripts
                    .iter_mut()
                    .rev()
                    .find(|location| location.script == *target_script)
            } else {
                state.script_runtime.next_script.as_mut()
            };
            let location = location.with_context(|| {
                format!("script jump to {target_script} did not retain its target location")
            })?;
            location.origin_map_name = target_origin;
        }
        Ok(action)
    }

    pub fn apply_script_control_command_in_session(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptControlAction> {
        self.apply_script_control_command(
            state,
            &session.map.name,
            map_name,
            source_script,
            command_index,
        )
    }

    pub fn script_object_command(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<&ScriptObjectCommand> {
        if let Some(module) = self.global_script_module_for(source_script) {
            return find_script_entry(
                &module.script_object_commands,
                map_name,
                "global script object command",
                source_script,
                command_index,
                |command| (&command.source_script, command.command_index),
            );
        }
        find_script_entry(
            &self.map_module(map_name)?.script_object_commands,
            map_name,
            "script object command",
            source_script,
            command_index,
            |command| (&command.source_script, command.command_index),
        )
    }

    pub fn apply_script_object_mutation(
        &self,
        state: &mut GameState,
        session: &mut OverworldSession,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptObjectMutationOutcome> {
        self.require_current_map(current_map, map_name)?;
        let command = self
            .script_object_command(map_name, source_script, command_index)?
            .clone();
        let mut next_state = state.clone();
        let mut next_session = session.clone();
        let outcome =
            core_apply_script_object_mutation(&mut next_state, &mut next_session, &command)
                .map_err(|error| anyhow::anyhow!("apply script object mutation: {error:?}"))?;
        sync_state_object_overrides(&mut next_state, &next_session)
            .context("sync script object overrides")?;
        *state = next_state;
        *session = next_session;
        Ok(outcome)
    }

    pub fn apply_script_object_mutation_in_session(
        &self,
        state: &mut GameState,
        session: &mut OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptObjectMutationOutcome> {
        let current_map = session.map.name.clone();
        self.apply_script_object_mutation(
            state,
            session,
            &current_map,
            map_name,
            source_script,
            command_index,
        )
    }

    pub fn apply_script_movement(
        &self,
        state: &mut GameState,
        session: &mut OverworldSession,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptMovementOutcome> {
        self.require_current_map(current_map, map_name)?;
        let command = self
            .script_object_command(map_name, source_script, command_index)?
            .clone();
        let movement_label = command.movement.as_deref().with_context(|| {
            format!(
                "script movement command at {source_script}:{command_index} has no movement label"
            )
        })?;
        let movement = self
            .script_movement(map_name, &command.source_script, movement_label)?
            .clone();
        let mut next_state = state.clone();
        let mut next_session = session.clone();
        let pending_surf = if command.source_script == "UsedSurfScript"
            && command.command_index == 9
            && command.object_id.as_deref() == Some("PLAYER")
            && command.movement.as_deref() == Some("wMovementBuffer")
        {
            let pending = next_state
                .script_runtime
                .pending_surf_field_move
                .clone()
                .context("UsedSurfScript movement has no prepared SURF field move")?;
            anyhow::ensure!(
                pending.map_name == next_session.map.name
                    && pending.from_tile == next_session.player.tile
                    && pending.mode == next_session.player.mode,
                "UsedSurfScript movement no longer matches its prepared source state"
            );
            Some(pending)
        } else {
            None
        };
        let pending_waterfall = if command.source_script == ".loop@Script_UsedWaterfall"
            && command.command_index == 0
            && command.object_id.as_deref() == Some("PLAYER")
            && command.movement.as_deref() == Some(".WaterfallStep")
        {
            let pending = next_state
                .script_runtime
                .pending_waterfall_field_move
                .clone()
                .context("Waterfall source loop has no prepared WATERFALL field move")?;
            anyhow::ensure!(
                pending.map_name == next_session.map.name
                    && pending.from_tile == next_session.player.tile
                    && pending.mode == next_session.player.mode
                    && next_session.player.facing == Direction::Up,
                "Waterfall source loop no longer matches its prepared climb state"
            );
            Some(pending)
        } else {
            None
        };
        let outcome = core_apply_script_movement(&mut next_session, &command, &movement)
            .map_err(|error| anyhow::anyhow!("apply script movement: {error:?}"))?;
        if let Some(pending) = pending_surf {
            anyhow::ensure!(
                outcome.previous_tile == pending.from_tile
                    && outcome.tile == pending.to_tile
                    && outcome.steps_applied == usize::from(pending.steps),
                "UsedSurfScript movement diverged from its prepared SURF outcome"
            );
            next_state.script_runtime.pending_surf_field_move = None;
        }
        if let Some(mut pending) = pending_waterfall {
            let expected_tile = TilePosition::new(
                pending.from_tile.x,
                pending
                    .from_tile
                    .y
                    .checked_sub(1)
                    .context("prepared WATERFALL step underflows runtime tile coordinates")?,
            );
            anyhow::ensure!(
                outcome.previous_tile == pending.from_tile
                    && outcome.tile == expected_tile
                    && outcome.steps_applied == 1,
                "Waterfall source step diverged from its prepared climb outcome"
            );
            pending.from_tile = outcome.tile;
            pending.steps = pending
                .steps
                .checked_sub(1)
                .context("prepared WATERFALL step counter underflow")?;
            if pending.steps == 0 {
                anyhow::ensure!(
                    pending.from_tile == pending.to_tile,
                    "Waterfall source loop ended away from its prepared destination"
                );
                next_state.script_runtime.pending_waterfall_field_move = None;
            } else {
                next_state.script_runtime.pending_waterfall_field_move = Some(pending);
            }
        }
        Self::apply_script_movement_effects_to_state(&mut next_state, &outcome)?;
        sync_state_object_overrides(&mut next_state, &next_session)
            .context("sync script movement object overrides")?;
        *state = next_state;
        *session = next_session;
        Ok(outcome)
    }

    pub fn apply_script_movement_in_session(
        &self,
        state: &mut GameState,
        session: &mut OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptMovementOutcome> {
        let current_map = session.map.name.clone();
        self.apply_script_movement(
            state,
            session,
            &current_map,
            map_name,
            source_script,
            command_index,
        )
    }

    fn apply_script_movement_effects_to_state(
        state: &mut GameState,
        outcome: &ScriptMovementOutcome,
    ) -> Result<()> {
        for effect in &outcome.effects {
            match effect.command.as_str() {
                "teleport_from" => state.script_runtime.teleport_from_queued = true,
                "teleport_to" => state.script_runtime.teleport_from_queued = false,
                "hide_emote" => {
                    state
                        .script_runtime
                        .pending_emotes
                        .retain(|emote| emote.object != outcome.object_id);
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn script_runtime_command(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<&ScriptRuntimeCommand> {
        if let Some(module) = self.global_script_module_for(source_script) {
            return find_script_entry(
                &module.script_runtime_commands,
                map_name,
                "global phone script runtime command",
                source_script,
                command_index,
                |command| (&command.source_script, command.command_index),
            );
        }
        find_script_entry(
            &self.map_module(map_name)?.script_runtime_commands,
            map_name,
            "script runtime command",
            source_script,
            command_index,
            |command| (&command.source_script, command.command_index),
        )
    }

    /// Returns whether this is the one source-derived CPU edge implemented by
    /// the typed Rock encounter boundary. The routine body is certified, so a
    /// modpack cannot gain the capability by reusing either label alone.
    pub fn is_exact_rock_mon_encounter_command(
        &self,
        command: &RuntimeScriptCommandRef,
    ) -> Result<bool> {
        if command.source_script != "RockSmashScript" || command.command_index != 8 {
            return Ok(false);
        }
        let runtime_command = self.script_runtime_command(
            &command.map_name,
            &command.source_script,
            command.command_index,
        )?;
        if runtime_command.command != "callasm"
            || runtime_command.args.as_slice() != ["RockMonEncounter"]
        {
            return Ok(false);
        }
        let module = self
            .global_script_module_for(&command.source_script)
            .context("RockSmashScript is not owned by the exported global script module")?;
        let target = resolve_script_target_label(
            &module.definitions,
            &command.source_script,
            &runtime_command.args[0],
        )
        .context("RockMonEncounter callasm target does not resolve")?;
        certify_rock_mon_encounter_callasm_target(&module.definitions, &target).map_err(
            |failure| {
                anyhow::anyhow!(
                    "RockMonEncounter exact certificate failed at {}:{} '{}': {}",
                    failure.target_script,
                    failure.command_index,
                    failure.command,
                    failure.reason,
                )
            },
        )?;
        Ok(true)
    }

    /// Returns whether this is Crystal's exact source-derived Headbutt
    /// encounter edge. The CPU routine and surrounding script are certified
    /// before the RNG-owning typed mutation is allowed.
    pub fn is_exact_tree_mon_encounter_command(
        &self,
        command: &RuntimeScriptCommandRef,
    ) -> Result<bool> {
        if command.source_script != "HeadbuttScript" || command.command_index != 4 {
            return Ok(false);
        }
        let runtime_command = self.script_runtime_command(
            &command.map_name,
            &command.source_script,
            command.command_index,
        )?;
        if runtime_command.command != "callasm"
            || runtime_command.args.as_slice() != ["TreeMonEncounter"]
        {
            return Ok(false);
        }
        let module = self
            .global_script_module_for(&command.source_script)
            .context("HeadbuttScript is not owned by the exported global script module")?;
        let target = resolve_script_target_label(
            &module.definitions,
            &command.source_script,
            &runtime_command.args[0],
        )
        .context("TreeMonEncounter callasm target does not resolve")?;
        certify_tree_mon_encounter_callasm_target(&module.definitions, &target).map_err(
            |failure| {
                anyhow::anyhow!(
                    "TreeMonEncounter exact certificate failed at {}:{} '{}': {}",
                    failure.target_script,
                    failure.command_index,
                    failure.command,
                    failure.reason,
                )
            },
        )?;
        Ok(true)
    }

    /// Returns whether this is Crystal's exact source-derived Sweet Scent
    /// encounter edge. The CPU routine and surrounding script are certified
    /// before the RNG-owning typed mutation is allowed.
    pub fn is_exact_sweet_scent_encounter_command(
        &self,
        command: &RuntimeScriptCommandRef,
    ) -> Result<bool> {
        if command.source_script != ".SweetScent@SweetScentFromMenu" || command.command_index != 5 {
            return Ok(false);
        }
        let runtime_command = self.script_runtime_command(
            &command.map_name,
            &command.source_script,
            command.command_index,
        )?;
        if runtime_command.command != "callasm"
            || runtime_command.args.as_slice() != ["SweetScentEncounter"]
        {
            return Ok(false);
        }
        let module = self
            .global_script_module_for(&command.source_script)
            .context("Sweet Scent script is not owned by the exported global script module")?;
        let target = resolve_script_target_label(
            &module.definitions,
            &command.source_script,
            &runtime_command.args[0],
        )
        .context("SweetScentEncounter callasm target does not resolve")?;
        certify_sweet_scent_encounter_callasm_target(&module.definitions, &target).map_err(
            |failure| {
                anyhow::anyhow!(
                    "SweetScentEncounter exact certificate failed at {}:{} '{}': {}",
                    failure.target_script,
                    failure.command_index,
                    failure.command,
                    failure.reason,
                )
            },
        )?;
        Ok(true)
    }

    pub fn resolve_tree_mon_encounter<S>(
        &self,
        state: &mut GameState,
        overworld: &OverworldSession,
        command: &RuntimeScriptCommandRef,
        divider: &mut S,
    ) -> Result<HeadbuttEncounterOutcome>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        self.require_current_map(&overworld.map.name, &command.map_name)?;
        anyhow::ensure!(
            self.is_exact_tree_mon_encounter_command(command)?,
            "runtime TreeMonEncounter command is not the exact HeadbuttScript:4 typed edge"
        );
        let target = Self::checked_runtime_field_move_target(
            "HEADBUTT",
            overworld.player.tile,
            overworld.player.facing,
        )?;
        self.validate_runtime_map_tile("HEADBUTT encounter", &overworld.map.name, target)?;
        let encounters = self.require_field_encounters_for_map(&overworld.map.name)?;
        let outcome = core_resolve_headbutt_encounter(
            encounters,
            target.x,
            target.y,
            state.player_id,
            state.random_state,
            divider,
        )
        .map_err(|error| anyhow::anyhow!("resolve TreeMonEncounter: {error}"))?;
        state.random_state = outcome.random_state_after;
        let (species, level, found) = outcome
            .roll
            .resolved
            .as_ref()
            .map(|resolved| {
                (
                    resolved.encounter.species.clone(),
                    resolved.encounter.level.to_string(),
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
        state.script_runtime.memory.insert(
            "wBattleType".to_string(),
            if found { "BATTLETYPE_TREE" } else { "0" }.to_string(),
        );
        state
            .script_runtime
            .memory
            .insert("wScriptVar".to_string(), u8::from(found).to_string());
        state.script_runtime.script_value = Some(u8::from(found).to_string());
        Ok(outcome)
    }

    pub fn is_exact_rock_smash_dynamic_start_command(
        &self,
        command: &RuntimeScriptCommandRef,
    ) -> Result<bool> {
        if command.source_script != "RockSmashScript" || command.command_index != 12 {
            return Ok(false);
        }
        let callasm = RuntimeScriptCommandRef::new(&command.map_name, &command.source_script, 8);
        if !self.is_exact_rock_mon_encounter_command(&callasm)? {
            return Ok(false);
        }
        let body = self
            .compiled_script_body(&command.source_script)
            .and_then(Value::as_array)
            .context("exact RockSmashScript body is missing")?;
        Ok(body.get(command.command_index).is_some_and(|entry| {
            entry.get("command").and_then(Value::as_str) == Some("startbattle")
                && entry
                    .get("args")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
        }))
    }

    pub fn is_exact_headbutt_dynamic_start_command(
        &self,
        command: &RuntimeScriptCommandRef,
    ) -> Result<bool> {
        if command.source_script != "HeadbuttScript" || command.command_index != 8 {
            return Ok(false);
        }
        let callasm = RuntimeScriptCommandRef::new(&command.map_name, &command.source_script, 4);
        if !self.is_exact_tree_mon_encounter_command(&callasm)? {
            return Ok(false);
        }
        let body = self
            .compiled_script_body(&command.source_script)
            .and_then(Value::as_array)
            .context("exact HeadbuttScript body is missing")?;
        Ok(body.get(command.command_index).is_some_and(|entry| {
            entry.get("command").and_then(Value::as_str) == Some("startbattle")
                && entry
                    .get("args")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
        }))
    }

    pub fn is_exact_sweet_scent_dynamic_start_command(
        &self,
        command: &RuntimeScriptCommandRef,
    ) -> Result<bool> {
        if command.source_script != ".SweetScent@SweetScentFromMenu" || command.command_index != 10
        {
            return Ok(false);
        }
        let callasm = RuntimeScriptCommandRef::new(&command.map_name, &command.source_script, 5);
        if !self.is_exact_sweet_scent_encounter_command(&callasm)? {
            return Ok(false);
        }
        let body = self
            .compiled_script_body(&command.source_script)
            .and_then(Value::as_array)
            .context("exact Sweet Scent script body is missing")?;
        Ok(body.get(command.command_index).is_some_and(|entry| {
            entry.get("command").and_then(Value::as_str) == Some("startbattle")
                && entry
                    .get("args")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
        }))
    }

    pub fn resolve_rock_mon_encounter<S>(
        &self,
        state: &mut GameState,
        current_map: &str,
        command: &RuntimeScriptCommandRef,
        divider: &mut S,
    ) -> Result<RockMonEncounterOutcome>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        self.require_current_map(current_map, &command.map_name)?;
        anyhow::ensure!(
            self.is_exact_rock_mon_encounter_command(command)?,
            "runtime RockMonEncounter command is not the exact RockSmashScript:8 typed edge"
        );
        let outcome = core_resolve_rock_mon_encounter(
            self.field_encounters.get(current_map),
            state.random_state,
            divider,
        )
        .map_err(|error| anyhow::anyhow!("resolve RockMonEncounter on {current_map}: {error}"))?;
        state.random_state = outcome.random_state_after;
        let (species, level) = outcome
            .resolved
            .as_ref()
            .map(|resolved| {
                (
                    resolved.encounter.species.clone(),
                    resolved.level.to_string(),
                )
            })
            .unwrap_or_else(|| ("0".to_string(), "0".to_string()));
        state
            .script_runtime
            .memory
            .insert("wTempWildMonSpecies".to_string(), species);
        state
            .script_runtime
            .memory
            .insert("wCurPartyLevel".to_string(), level);
        Ok(outcome)
    }

    pub fn apply_script_runtime_command(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        command_index: usize,
        inputs: ScriptRuntimeInputs,
    ) -> Result<(ScriptRuntimeCommand, ScriptRuntimeOutcome)> {
        self.require_current_map(current_map, map_name)?;
        let command = self
            .script_runtime_command(map_name, source_script, command_index)?
            .clone();
        if let Some(object_id) = Self::script_runtime_live_object_reference_for_command(&command)? {
            Self::require_runtime_object_reference(overworld, &object_id)?;
        }
        let trade_was_completed = command.command == "trade"
            && command
                .args
                .first()
                .is_some_and(|trade_id| state.script_runtime.completed_trades.contains(trade_id));
        let mut next_state = state.clone();
        let mut next_overworld = overworld.clone();
        if command.command == "callasm"
            && command.args.as_slice() == [".CheckContinueWaterfall"]
            && command.source_script == ".loop@Script_UsedWaterfall"
        {
            let collision = sample_collision(
                &next_overworld.map,
                &next_overworld.tileset,
                next_overworld.player.tile,
            )
            .context("sample exact wPlayerTileCollision for Waterfall continuation")?;
            next_state.script_runtime.memory.insert(
                "wPlayerTileCollision".to_string(),
                collision.permission.to_string(),
            );
        }
        let mut inputs = inputs;
        if command.command == "getcurlandmarkname" {
            inputs.current_landmark_name =
                Some(self.pokegear_landmark_for_map(map_name)?.name.clone());
        }
        if command.command == "getitemname" {
            let item_arg = command.args.get(1).with_context(|| {
                format!(
                    "getitemname command {}:{} missing item id",
                    command.source_script, command.command_index
                )
            })?;
            let item_id = if item_arg == SCRIPT_RUNTIME_USE_SCRIPT_VAR_ID {
                next_state
                    .script_runtime
                    .script_value
                    .clone()
                    .with_context(|| {
                        format!(
                            "getitemname command {}:{} requires script_value for USE_SCRIPT_VAR",
                            command.source_script, command.command_index
                        )
                    })?
            } else {
                item_arg.clone()
            };
            anyhow::ensure!(
                self.items.contains_key(&item_id),
                "getitemname references unknown item {item_id}"
            );
            next_state
                .script_runtime
                .memory
                .insert("wNamedObjectIndex".to_string(), item_id);
        }
        let pack_resolved_name =
            self.resolve_script_runtime_name_buffer_value(&next_state, map_name, &command)?;
        if command.command == "getname" {
            // `getname` spans several ROM name tables, including intentionally
            // broken/WRAM-backed entries. Preserve the exact value supplied by
            // the typed caller when the pack has no static catalog resolution.
            // Other name opcodes remain pack-owned and overwrite caller data.
            if pack_resolved_name.is_some() {
                inputs.resolved_named_buffer_value = pack_resolved_name;
            }
        } else {
            inputs.resolved_named_buffer_value = pack_resolved_name;
        }
        inputs.resolved_stone_table_entries =
            self.resolve_script_runtime_stone_table_queue(map_name, &command)?;
        inputs.resolved_decoration =
            self.resolve_script_runtime_decoration(&next_state, &command)?;
        if command.command == "specialsound" {
            let item_id = next_state
                .script_runtime
                .memory
                .get("wCurItem")
                .with_context(|| "specialsound requires wCurItem")?;
            let item = self
                .items
                .get(item_id)
                .with_context(|| format!("specialsound references unknown wCurItem {item_id}"))?;
            let audio_id = if item.pocket == ITEM_POCKET_TM_HM {
                "SFX_GET_TM"
            } else {
                "SFX_ITEM"
            };
            anyhow::ensure!(
                self.audio.iter().any(|asset| asset.id == audio_id),
                "specialsound selected missing audio asset {audio_id}"
            );
            inputs.resolved_special_sound_effect = Some(audio_id.to_string());
        }
        if command.command == "pocketisfull" {
            let item_id = next_state
                .script_runtime
                .memory
                .get("wCurItem")
                .with_context(|| "pocketisfull requires wCurItem")?;
            let item = self
                .items
                .get(item_id)
                .with_context(|| format!("pocketisfull references unknown wCurItem {item_id}"))?;
            let pocket_name = match item.pocket.as_str() {
                ITEM_POCKET_ITEM => "ITEM POCKET",
                ITEM_POCKET_KEY_ITEM => "KEY POCKET",
                ITEM_POCKET_BALL => "BALL POCKET",
                ITEM_POCKET_TM_HM => "TM POCKET",
                pocket => anyhow::bail!("pocketisfull item {item_id} uses non-ASM pocket {pocket}"),
            };
            inputs.resolved_current_pocket_name = Some(pocket_name.to_string());
            inputs.resolved_current_item_name = Some(item.name.clone());
        }
        if command.command == "warpmod" {
            let map_constant = command.args.get(1).with_context(|| {
                format!(
                    "warpmod command {}:{} missing target map argument",
                    command.source_script, command.command_index
                )
            })?;
            inputs.resolved_warpmod_map_name = Some(self.map_name_for_constant(map_constant)?);
        }
        if matches!(
            command.command.as_str(),
            "memcall" | "memjump" | "memcallasm"
        ) {
            let pointer = if command.command == "memcall" {
                anyhow::ensure!(
                    !command.args.is_empty(),
                    "memcall command {}:{} is missing its pointer operand",
                    command.source_script,
                    command.command_index
                );
                command.args.join(" ")
            } else {
                command
                    .args
                    .first()
                    .with_context(|| {
                        format!(
                            "{} command {}:{} is missing its pointer operand",
                            command.command, command.source_script, command.command_index
                        )
                    })?
                    .clone()
            };
            let target = next_state
                .script_runtime
                .memory
                .get(&pointer)
                .with_context(|| format!("{} pointer {pointer} is unset", command.command))?
                .clone();
            let definitions =
                if let Some(module) = self.global_script_module_for(&command.source_script) {
                    &module.definitions
                } else {
                    &self.map_module(map_name)?.scripts
                };
            let resolved =
                resolve_script_target_label(definitions, &command.source_script, &target)
                    .with_context(|| {
                        format!(
                            "{} pointer {pointer} target {target} cannot resolve from {}",
                            command.command, command.source_script
                        )
                    })?;
            next_state.script_runtime.memory.insert(pointer, resolved);
        }
        let selected_party_index = inputs.selected_party_index;
        let queued_command_count = next_state.script_runtime.command_queue.len();
        let mut outcome = core_apply_script_runtime_command(
            &mut next_state,
            map_name,
            command.clone(),
            inputs,
            &self.story_event_script_constants,
        )
        .map_err(|error| anyhow::anyhow!("apply script runtime command: {error:?}"))?;
        if command.command == "warpsound" {
            let collision = sample_collision(
                &next_overworld.map,
                &next_overworld.tileset,
                next_overworld.player.tile,
            )
            .context("sample exact wPlayerTileCollision for GetWarpSFX")?;
            let audio_id = warp_sound_effect_for_collision(collision.permission);
            anyhow::ensure!(
                self.audio.iter().any(|asset| asset.id == audio_id),
                "GetWarpSFX selected missing audio asset {audio_id}"
            );
            next_state
                .script_runtime
                .audio_events
                .push(ScriptAudioRuntimeEvent {
                    command: command.command.clone(),
                    kind: ScriptAudioRuntimeKind::SoundEffect,
                    audio_id: Some(audio_id.to_string()),
                    fade_frames: None,
                    source_script: command.source_script.clone(),
                    command_index: command.command_index,
                });
        }
        if command.command == "verbosegiveitemvar" {
            let grant = self.grant_script_item(&mut next_state, map_name, map_name,
                &command.source_script, command.command_index)?;
            let value = if matches!(grant, ScriptItemGrantOutcome::Granted { .. }) { "1" } else { "0" }.to_string();
            next_state.script_runtime.script_value = Some(value.clone());
            next_state.script_runtime.variables.insert("_value".into(), value.clone());
            outcome = ScriptRuntimeOutcome::ScriptValueSet {
                command: command.command.clone(), value,
                source_script: command.source_script.clone(), command_index: command.command_index,
            };
        }
        if matches!(command.command.as_str(), "callasm" | "memcallasm") {
            let queued_count_after = next_state.script_runtime.command_queue.len();
            if queued_count_after != queued_command_count + 1 {
                anyhow::bail!(
                    "{} {}:{} enqueued {} commands instead of exactly one",
                    command.command,
                    command.source_script,
                    command.command_index,
                    queued_count_after.saturating_sub(queued_command_count)
                );
            }
            let queued = next_state
                .script_runtime
                .command_queue
                .get(queued_command_count)
                .expect("validated appended callasm queue entry");
            let expected_target = if command.command == "memcallasm" {
                let pointer = command.args.first().with_context(|| {
                    format!(
                        "memcallasm command {}:{} is missing its pointer operand",
                        command.source_script, command.command_index
                    )
                })?;
                next_state
                    .script_runtime
                    .memory
                    .get(pointer)
                    .with_context(|| format!("memcallasm pointer {pointer} is unset"))?
            } else {
                command.args.first().with_context(|| {
                    format!(
                        "callasm command {}:{} is missing its routine target",
                        command.source_script, command.command_index
                    )
                })?
            };
            if queued.command != command.command
                || queued.target != *expected_target
                || queued.source_script != command.source_script
                || queued.command_index != command.command_index
            {
                anyhow::bail!(
                    "{} {}:{} enqueued mismatched routine command {:?}",
                    command.command,
                    command.source_script,
                    command.command_index,
                    queued
                );
            }
            let queued_target = queued.target.clone();
            let mut effective_command = command.clone();
            effective_command.args = vec![queued_target.clone()];
            let execution = self.execute_script_callasm_accumulator(
                &mut next_state,
                &mut next_overworld,
                map_name,
                &effective_command,
            )?;
            if execution.is_none() {
                anyhow::bail!(
                    "{} {}:{} target {} cannot execute synchronously",
                    command.command,
                    command.source_script,
                    command.command_index,
                    queued_target
                );
            }
            if let Some(execution) = execution {
                next_state
                    .script_runtime
                    .command_queue
                    .remove(queued_command_count);
                if let Some(value) = execution.script_value {
                    outcome = ScriptRuntimeOutcome::ScriptValueSet {
                        command: command.command.clone(),
                        value,
                        source_script: command.source_script.clone(),
                        command_index: command.command_index,
                    };
                }
                if let Some(effect) = execution.phone_presentation {
                    outcome = ScriptRuntimeOutcome::PhoneCallasmPresentation {
                        effect,
                        source_script: command.source_script.clone(),
                        command_index: command.command_index,
                    };
                }
            }
        }
        if command.command == "trade" && !trade_was_completed {
            self.apply_npc_trade(
                &mut next_state,
                command.args.first().map(String::as_str),
                selected_party_index,
            )?;
        }
        if matches!(command.command.as_str(), "givepokemail" | "checkpokemail") {
            let queued_count_after = next_state.script_runtime.command_queue.len();
            if queued_count_after != queued_command_count + 1 {
                anyhow::bail!(
                    "{} {}:{} enqueued {} commands instead of exactly one",
                    command.command,
                    command.source_script,
                    command.command_index,
                    queued_count_after.saturating_sub(queued_command_count)
                );
            }
            let queued = &next_state.script_runtime.command_queue[queued_command_count];
            if queued.command != command.command
                || queued.target != command.args[0]
                || queued.source_script != command.source_script
                || queued.command_index != command.command_index
            {
                anyhow::bail!(
                    "{} {}:{} enqueued mismatched synchronous command {:?}",
                    command.command,
                    command.source_script,
                    command.command_index,
                    queued
                );
            }
            next_state
                .script_runtime
                .command_queue
                .remove(queued_command_count);
        }
        if command.command == "givepokemail" {
            self.apply_compiled_mail_definition(&mut next_state, &command.args[0])?;
        } else if command.command == "checkpokemail" {
            self.apply_compiled_mail_check(
                &mut next_state,
                &command.args[0],
                selected_party_index,
            )?;
        }
        if command.command == "blackoutmod" {
            let map_constant = command.args.first().with_context(|| {
                format!(
                    "blackoutmod command {}:{} missing target map argument",
                    command.source_script, command.command_index
                )
            })?;
            self.map_name_for_constant(map_constant)?;
            next_state.last_spawn_map_constant = Some(map_constant.clone());
        }
        if command.command == "changemapblocks" {
            let blocks_label = command.args.first().with_context(|| {
                format!(
                    "changemapblocks command {}:{} missing block-data pointer",
                    command.source_script, command.command_index
                )
            })?;
            let encoded = self.map_blocks.get(blocks_label).with_context(|| {
                format!("changemapblocks references missing block data {blocks_label}")
            })?;
            let replacement = decode_base64_bytes(encoded)
                .with_context(|| format!("decode changemapblocks payload {blocks_label}"))?
                .into_iter()
                .map(u16::from)
                .collect::<Vec<_>>();
            anyhow::ensure!(
                replacement.len() == next_overworld.map.metatile_ids.len(),
                "changemapblocks payload {blocks_label} has {} blocks for {} map blocks",
                replacement.len(),
                next_overworld.map.metatile_ids.len()
            );
            next_overworld.map.metatile_ids = replacement.clone();
            let base = &self.map_module(map_name)?.blocks;
            anyhow::ensure!(
                base.len() == replacement.len(),
                "compiled base map {map_name} changed size during changemapblocks"
            );
            let mut overrides = BTreeMap::new();
            let width = usize::from(next_overworld.map.width);
            for (index, (&block_id, &base_id)) in replacement.iter().zip(base).enumerate() {
                if block_id != base_id {
                    overrides.insert(
                        (
                            u16::try_from(index % width).context("map block x overflow")?,
                            u16::try_from(index / width).context("map block y overflow")?,
                        ),
                        block_id,
                    );
                }
            }
            if overrides.is_empty() {
                next_state.map_block_overrides.remove(map_name);
            } else {
                next_state
                    .map_block_overrides
                    .insert(map_name.to_string(), overrides);
            }
        }
        if command.command == "writevar"
            && command.args.first().map(String::as_str) == Some("VAR_MOVEMENT")
        {
            let value = next_state
                .script_runtime
                .variables
                .get("VAR_MOVEMENT")
                .context("writevar VAR_MOVEMENT did not write its exact accumulator value")?;
            next_overworld.player.mode = movement_mode_from_script_value(value)?;
            self.commit_overworld_snapshot(
                &mut next_state,
                &next_overworld,
                SpawnMemoryUpdate::Preserve,
            );
        }
        if command.command == "setlasttalked" {
            next_overworld.last_talked_object_identifier = command
                .args
                .first()
                .filter(|object_id| object_id.as_str() != "-1")
                .cloned();
            sync_state_object_overrides(&mut next_state, &next_overworld)
                .context("sync setlasttalked object overrides")?;
        }
        *state = next_state;
        *overworld = next_overworld;
        Ok((command, outcome))
    }

    /// Executes a compiled map-local CPU routine used by `callasm` when that
    /// routine returns a byte through Crystal's `wScriptVar` accumulator. The
    /// routine body, rather than its label, determines the behavior: supported
    /// byte transfers, ALU flag updates, AF stack operations, and typed
    /// conditional returns run synchronously through a taken `ret`, matching
    /// the script engine's direct CPU call. Routines outside that shape return
    /// `None` and retain their normal queued engine dispatch.
    fn execute_script_callasm_accumulator(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        map_name: &str,
        command: &ScriptRuntimeCommand,
    ) -> Result<Option<ScriptCallasmExecution>> {
        let target = command.args.first().with_context(|| {
            format!(
                "callasm command {}:{} is missing its routine target",
                command.source_script, command.command_index
            )
        })?;
        let definitions =
            if let Some(module) = self.global_script_module_for(&command.source_script) {
                &module.definitions
            } else {
                &self.map_module(map_name)?.scripts
            };
        let Some(resolved_target) =
            resolve_script_target_label(definitions, &command.source_script, target)
        else {
            return Ok(None);
        };
        let Some(entries) = definitions.get(&resolved_target).and_then(Value::as_array) else {
            return Ok(None);
        };

        if self
            .global_script_module_for(&command.source_script)
            .is_some()
            && let Some(effect) = exact_phone_callasm_effect(definitions, &resolved_target)
                .map_err(|failure| {
                    anyhow::anyhow!(
                        "exact phone callasm {} failed at {}:{} '{}': {}",
                        resolved_target,
                        failure.target_script,
                        failure.command_index,
                        failure.command,
                        failure.reason,
                    )
                })?
        {
            self.apply_exact_phone_callasm_effect(state, command, effect)?;
            return Ok(Some(ScriptCallasmExecution {
                script_value: None,
                phone_presentation: match effect {
                    ExactPhoneCallasmEffect::RingTwice => Some(
                        crystal_core::systems::script_runtime::ScriptPhoneCallasmPresentation::RingTwice,
                    ),
                    ExactPhoneCallasmEffect::HangUp => Some(
                        crystal_core::systems::script_runtime::ScriptPhoneCallasmPresentation::HangUp,
                    ),
                    ExactPhoneCallasmEffect::InitReceiveDelay
                    | ExactPhoneCallasmEffect::LoadCaller(_) => None,
                },
            }));
        }

        if certify_synchronous_script_callasm_target(definitions, &resolved_target).is_err() {
            return Ok(None);
        }

        if resolved_target == "Fishing_CheckFacingUp" {
            certify_fishing_check_facing_up_callasm_target(definitions, &resolved_target).map_err(
                |failure| {
                    anyhow::anyhow!(
                        "exact Fishing_CheckFacingUp callasm failed at {}:{} '{}': {}",
                        failure.target_script,
                        failure.command_index,
                        failure.command,
                        failure.reason,
                    )
                },
            )?;
            let value = u8::from(overworld.player.facing == Direction::Up).to_string();
            state.script_runtime.script_value = Some(value.clone());
            state
                .script_runtime
                .memory
                .insert("wScriptVar".to_string(), value.clone());
            state.script_runtime.memory.insert(
                "wPlayerDirection".to_string(),
                match overworld.player.facing {
                    Direction::Up => "OW_UP",
                    Direction::Down => "OW_DOWN",
                    Direction::Left => "OW_LEFT",
                    Direction::Right => "OW_RIGHT",
                }
                .to_string(),
            );
            return Ok(Some(ScriptCallasmExecution {
                script_value: Some(value),
                phone_presentation: None,
            }));
        }

        if let Some(effect) = exact_overworld_visual_callasm_effect(definitions, &resolved_target)
            .map_err(|failure| {
            anyhow::anyhow!(
                "exact overworld visual callasm {} failed at {}:{} '{}': {}",
                resolved_target,
                failure.target_script,
                failure.command_index,
                failure.command,
                failure.reason,
            )
        })? {
            match effect {
                ExactOverworldVisualCallasmEffect::SkipUpdateMapSprites => {
                    state.script_runtime.memory.insert(
                        "wPlayerSpriteSetupFlags.PLAYERSPRITESETUP_SKIP_RELOAD_GFX_F".to_string(),
                        "1".to_string(),
                    );
                }
                ExactOverworldVisualCallasmEffect::BlindingFlash => {
                    let pending = state
                        .script_runtime
                        .pending_flash_field_move
                        .as_ref()
                        .context("BlindingFlash source callasm has no prepared FLASH field move")?;
                    anyhow::ensure!(
                        pending.move_id == "FLASH"
                            && pending.engine_flag == "STATUSFLAGS_FLASH"
                            && !pending.was_set
                            && pending.is_set,
                        "BlindingFlash source callasm found invalid prepared FLASH outcome"
                    );
                    anyhow::ensure!(
                        !state.flags.is_engine_flag_set("STATUSFLAGS_FLASH")?,
                        "BlindingFlash source callasm found FLASH already active"
                    );
                    state
                        .flags
                        .set_engine_flag("STATUSFLAGS_FLASH", true)
                        .map_err(|error| {
                            anyhow::anyhow!("set FLASH status from callasm: {error}")
                        })?;
                    state.script_runtime.pending_flash_field_move = None;
                }
                ExactOverworldVisualCallasmEffect::CutBlockRefresh => {
                    commit_pending_block_field_move(state, overworld, "CUT")?;
                }
                ExactOverworldVisualCallasmEffect::WhirlpoolBlockRefresh => {
                    commit_pending_block_field_move(state, overworld, "WHIRLPOOL")?;
                }
                _ => {}
            }
            return Ok(Some(ScriptCallasmExecution {
                script_value: None,
                phone_presentation: None,
            }));
        }

        let branching_runtime_is_mandatory =
            certify_branching_script_callasm_target(definitions, &resolved_target).is_ok()
                || resolved_target == ".CheckContinueWaterfall@Script_UsedWaterfall";
        if branching_runtime_is_mandatory {
            let execution = self
                .execute_branching_script_callasm(state, map_name, definitions, &resolved_target)?
                .with_context(|| {
                    format!(
                        "certified synchronous callasm {resolved_target} could not execute from live runtime state"
                    )
                })?;
            return Ok(Some(execution));
        }

        let mut register_a = None;
        let mut flags = CallasmCpuFlags::default();
        let mut af_stack = Vec::new();
        let mut scratch_memory = BTreeMap::<String, Option<u8>>::new();
        let mut script_value = None;
        let mut returned = false;
        for (routine_index, entry) in entries.iter().enumerate() {
            let routine_command =
                entry
                    .get("command")
                    .and_then(Value::as_str)
                    .with_context(|| {
                        format!(
                            "callasm routine {} command {} on {} has no command name",
                            resolved_target, routine_index, map_name
                        )
                    })?;
            let args = script_command_args(map_name, &resolved_target, routine_command, entry)?;
            let Some(instruction) =
                classify_accumulator_callasm_instruction(routine_command, &args)
            else {
                return Ok(None);
            };
            match instruction {
                AccumulatorCallasmInstruction::LoadA { source } => {
                    register_a = read_callasm_byte_operand(state, &scratch_memory, source)?;
                }
                AccumulatorCallasmInstruction::StoreA { destination } => {
                    scratch_memory.insert(destination.to_string(), register_a);
                    if destination == "wScriptVar" {
                        script_value = Some(register_a.with_context(|| {
                            format!(
                                "callasm routine {} writes an unresolved byte to wScriptVar at command {}",
                                resolved_target, routine_index
                            )
                        })?);
                    }
                }
                AccumulatorCallasmInstruction::Alu { operation, operand } => {
                    let Some(left) = register_a else {
                        return Ok(None);
                    };
                    let Some(right) =
                        read_callasm_alu_operand(state, &scratch_memory, register_a, operand)?
                    else {
                        return Ok(None);
                    };
                    match operation {
                        AccumulatorCallasmAluOperation::Compare => {
                            flags.zero = Some(left == right);
                            flags.carry = Some(left < right);
                        }
                        AccumulatorCallasmAluOperation::And => {
                            let value = left & right;
                            register_a = Some(value);
                            flags.zero = Some(value == 0);
                            flags.carry = Some(false);
                        }
                        AccumulatorCallasmAluOperation::Or => {
                            let value = left | right;
                            register_a = Some(value);
                            flags.zero = Some(value == 0);
                            flags.carry = Some(false);
                        }
                        AccumulatorCallasmAluOperation::Xor => {
                            let value = left ^ right;
                            register_a = Some(value);
                            flags.zero = Some(value == 0);
                            flags.carry = Some(false);
                        }
                        AccumulatorCallasmAluOperation::Subtract => {
                            let (value, borrow) = left.overflowing_sub(right);
                            register_a = Some(value);
                            flags.zero = Some(value == 0);
                            flags.carry = Some(borrow);
                        }
                        AccumulatorCallasmAluOperation::Add => {
                            let (value, carry) = left.overflowing_add(right);
                            register_a = Some(value);
                            flags.zero = Some(value == 0);
                            flags.carry = Some(carry);
                        }
                    }
                }
                AccumulatorCallasmInstruction::IncrementA
                | AccumulatorCallasmInstruction::DecrementA => {
                    let Some(value) = register_a else {
                        return Ok(None);
                    };
                    let value = match instruction {
                        AccumulatorCallasmInstruction::IncrementA => value.wrapping_add(1),
                        AccumulatorCallasmInstruction::DecrementA => value.wrapping_sub(1),
                        _ => unreachable!("matched accumulator increment/decrement"),
                    };
                    register_a = Some(value);
                    flags.zero = Some(value == 0);
                    // INC and DEC leave the Game Boy carry flag untouched.
                }
                AccumulatorCallasmInstruction::SetCarry => flags.carry = Some(true),
                AccumulatorCallasmInstruction::ComplementCarry => {
                    let Some(carry) = flags.carry else {
                        return Ok(None);
                    };
                    flags.carry = Some(!carry);
                }
                AccumulatorCallasmInstruction::PushAf => af_stack.push((register_a, flags)),
                AccumulatorCallasmInstruction::PopAf => {
                    (register_a, flags) = af_stack.pop().with_context(|| {
                        format!(
                            "callasm routine {} pops an empty AF stack at command {}",
                            resolved_target, routine_index
                        )
                    })?;
                }
                AccumulatorCallasmInstruction::Return { condition } => {
                    let return_now = if let Some(condition) = condition {
                        let Some(is_met) = flags.condition_is_met(condition) else {
                            // The selected entry flag was never established by a
                            // supported instruction. Do not coerce it and take the
                            // wrong path.
                            return Ok(None);
                        };
                        is_met
                    } else {
                        true
                    };
                    if return_now {
                        returned = true;
                        break;
                    }
                }
            }
        }
        if !returned {
            return Ok(None);
        }
        let Some(value) = script_value else {
            return Ok(None);
        };
        let value = value.to_string();
        state.script_runtime.script_value = Some(value.clone());
        state
            .script_runtime
            .memory
            .insert("wScriptVar".to_string(), value.clone());
        Ok(Some(ScriptCallasmExecution {
            script_value: Some(value),
            phone_presentation: None,
        }))
    }

    fn apply_exact_phone_callasm_effect(
        &self,
        state: &mut GameState,
        _command: &ScriptRuntimeCommand,
        effect: ExactPhoneCallasmEffect,
    ) -> Result<()> {
        match effect {
            // Phone_CallerTextbox draws directly into the tilemap; it does
            // not open a script menu. The presentation boundary owns the
            // caller box, and farwritetext opens the subsequent speech box.
            ExactPhoneCallasmEffect::RingTwice | ExactPhoneCallasmEffect::HangUp => {}
            ExactPhoneCallasmEffect::InitReceiveDelay => {
                restart_receive_call_delay(state, true);
                state
                    .script_runtime
                    .memory
                    .insert("wTimeCyclesSinceLastCall".to_string(), "0".to_string());
            }
            ExactPhoneCallasmEffect::LoadCaller(contact_id) => {
                let contact = self.phone_contacts.0.get(contact_id).with_context(|| {
                    format!("exact phone callasm references missing contact {contact_id}")
                })?;
                let caller_script = contact.caller_script.as_ref().with_context(|| {
                    format!("exact phone callasm contact {contact_id} has no caller script")
                })?;
                state
                    .script_runtime
                    .variables
                    .insert("VAR_CALLERID".to_string(), contact_id.to_string());
                state
                    .script_runtime
                    .memory
                    .insert("wCurCaller".to_string(), contact_id.to_string());
                state
                    .script_runtime
                    .memory
                    .insert("wCallerContact".to_string(), contact_id.to_string());
                state.script_runtime.memory.insert(
                    "wCallerContact + PHONE_CONTACT_SCRIPT2_BANK".to_string(),
                    caller_script.clone(),
                );
            }
        }
        Ok(())
    }

    /// Executes the branching CPU subset used by shared overworld `callasm`
    /// routines. Control flow and effects come from the exported instructions;
    /// the only host services are the CPU calls whose implementations live
    /// outside the exported bank (party-move lookup, engine-flag lookup, and
    /// nickname copying).
    fn execute_branching_script_callasm(
        &self,
        state: &mut GameState,
        map_name: &str,
        definitions: &BTreeMap<String, Value>,
        start_label: &str,
    ) -> Result<Option<ScriptCallasmExecution>> {
        let mut staged_state = state.clone();
        let mut cpu = BranchingCallasmState::default();
        let mut label = start_label.to_string();
        let mut command_index = 0usize;

        for _ in 0..1024 {
            let Some(entries) = definitions.get(&label).and_then(Value::as_array) else {
                return Ok(None);
            };
            let Some(entry) = entries.get(command_index) else {
                return Ok(None);
            };
            let command = entry
                .get("command")
                .and_then(Value::as_str)
                .with_context(|| {
                    format!(
                        "callasm routine {label} command {command_index} on {map_name} has no command name"
                    )
            })?;
            let args = script_command_args(map_name, &label, command, entry)?;
            let Some(instruction) = classify_branching_callasm_instruction(command, &args) else {
                return Ok(None);
            };
            match instruction {
                BranchingCallasmInstruction::Load {
                    destination,
                    source,
                } => {
                    let Some(value) = read_branching_callasm_value(&staged_state, &cpu, source)
                    else {
                        return Ok(None);
                    };
                    if matches!(destination, "a" | "d" | "e" | "de" | "hl") {
                        cpu.registers.insert(destination.to_string(), value);
                    } else if let Some(destination) = callasm_indirect_symbol(destination) {
                        let destination = if destination == "hl" {
                            cpu.registers.get("hl").cloned()
                        } else {
                            Some(destination.to_string())
                        };
                        let Some(destination) = destination else {
                            return Ok(None);
                        };
                        staged_state
                            .script_runtime
                            .memory
                            .insert(destination.clone(), value.clone());
                        if destination == "wScriptVar" {
                            staged_state.script_runtime.script_value = Some(value.clone());
                            cpu.script_value = Some(value);
                        }
                    } else {
                        return Ok(None);
                    }
                    command_index += 1;
                }
                BranchingCallasmInstruction::Call { service } => {
                    if !self.apply_branching_callasm_host_service(
                        &mut staged_state,
                        &mut cpu,
                        service,
                    )? {
                        return Ok(None);
                    }
                    command_index += 1;
                }
                BranchingCallasmInstruction::Jump { condition, target } => {
                    let taken = if let Some(condition) = condition {
                        let Some(taken) = cpu.flags.condition_is_met(condition) else {
                            return Ok(None);
                        };
                        taken
                    } else {
                        true
                    };
                    if taken {
                        let Some(target) = resolve_script_target_label(definitions, &label, target)
                        else {
                            return Ok(None);
                        };
                        label = target;
                        command_index = 0;
                    } else {
                        command_index += 1;
                    }
                }
                BranchingCallasmInstruction::Bit { bit } => {
                    let Some(memory) = cpu.registers.get("hl") else {
                        return Ok(None);
                    };
                    let Some(effect) = branching_callasm_bit_effect(memory, bit) else {
                        return Ok(None);
                    };
                    let engine_flag_set = staged_state
                        .flags
                        .engine_flags
                        .get(effect.engine_flag)
                        .copied()
                        .unwrap_or(false);
                    let memory_bit_set = engine_flag_set != effect.inverted;
                    cpu.flags.zero = Some(!memory_bit_set);
                    command_index += 1;
                }
                BranchingCallasmInstruction::Set { bit } => {
                    let Some(memory) = cpu.registers.get("hl") else {
                        return Ok(None);
                    };
                    let Some(effect) = branching_callasm_bit_effect(memory, bit) else {
                        return Ok(None);
                    };
                    staged_state
                        .flags
                        .set_engine_flag(effect.engine_flag, !effect.inverted)
                        .map_err(|error| {
                            anyhow::anyhow!(
                                "set callasm runtime bit {}, {}: {error}",
                                effect.memory,
                                effect.bit
                            )
                        })?;
                    command_index += 1;
                }
                BranchingCallasmInstruction::AddHlDe => {
                    let Some(base) = cpu.registers.get("hl").cloned() else {
                        return Ok(None);
                    };
                    let offset = cpu
                        .registers
                        .get("de")
                        .and_then(|value| parse_script_i32(value).ok())
                        .or_else(|| {
                            let d = cpu
                                .registers
                                .get("d")
                                .and_then(|value| parse_script_i32(value).ok())?;
                            let e = cpu
                                .registers
                                .get("e")
                                .and_then(|value| parse_script_i32(value).ok())?;
                            Some(d * 256 + e)
                        });
                    let Some(offset) = offset else {
                        return Ok(None);
                    };
                    cpu.registers
                        .insert("hl".to_string(), format!("{base}+{offset}"));
                    command_index += 1;
                }
                BranchingCallasmInstruction::XorA => {
                    cpu.registers.insert("a".to_string(), "0".to_string());
                    cpu.flags.zero = Some(true);
                    cpu.flags.carry = Some(false);
                    command_index += 1;
                }
                BranchingCallasmInstruction::Return { condition } => {
                    let return_now = if let Some(condition) = condition {
                        let Some(taken) = cpu.flags.condition_is_met(condition) else {
                            return Ok(None);
                        };
                        taken
                    } else {
                        true
                    };
                    if return_now {
                        *state = staged_state;
                        return Ok(Some(ScriptCallasmExecution {
                            script_value: cpu.script_value,
                            phone_presentation: None,
                        }));
                    }
                    command_index += 1;
                }
            }
        }
        anyhow::bail!("callasm routine {start_label} exceeded CPU execution limit")
    }

    fn apply_branching_callasm_host_service(
        &self,
        state: &mut GameState,
        cpu: &mut BranchingCallasmState,
        service: BranchingCallasmHostService,
    ) -> Result<bool> {
        match service {
            BranchingCallasmHostService::CheckPartyMove => {
                let Some(move_id) = cpu.registers.get("d") else {
                    return Ok(false);
                };
                state
                    .script_runtime
                    .memory
                    .insert("wCurPartyMon".to_string(), "0".to_string());
                let party_count = state
                    .storage
                    .party
                    .pokemon
                    .iter()
                    .take_while(|pokemon| pokemon.is_some())
                    .count();
                let selected = state
                    .storage
                    .party
                    .pokemon
                    .iter()
                    .take(party_count)
                    .enumerate()
                    .find_map(|(party_index, pokemon)| {
                        pokemon.as_ref().and_then(|pokemon| {
                            (!pokemon.is_egg
                                && pokemon.moves.iter().any(|learned| learned.name == *move_id))
                            .then_some(party_index)
                        })
                    });
                cpu.registers.insert(
                    "a".to_string(),
                    if selected.is_some() { "0" } else { "255" }.to_string(),
                );
                cpu.registers
                    .insert("e".to_string(), selected.unwrap_or(party_count).to_string());
                cpu.flags.zero = Some(true);
                cpu.flags.carry = Some(selected.is_none());
                cpu.selected_party_index = selected;
                if let Some(party_index) = selected {
                    state
                        .script_runtime
                        .memory
                        .insert("wCurPartyMon".to_string(), party_index.to_string());
                }
                Ok(true)
            }
            BranchingCallasmHostService::CheckEngineFlag => {
                let Some(flag) = cpu.registers.get("de") else {
                    return Ok(false);
                };
                let set = branching_callasm_engine_flag_is_set(state, flag);
                cpu.registers.insert("a".to_string(), "0".to_string());
                cpu.flags.zero = Some(true);
                cpu.flags.carry = Some(!set);
                Ok(true)
            }
            BranchingCallasmHostService::GetPartyNickname => {
                let party_index = state
                    .script_runtime
                    .memory
                    .get("wCurPartyMon")
                    .and_then(|value| value.parse::<usize>().ok());
                let Some(party_index) = party_index else {
                    return Ok(false);
                };
                let Some(name) = branching_callasm_party_nickname(state, party_index) else {
                    return Ok(false);
                };
                for buffer in 1..=3 {
                    state
                        .script_runtime
                        .named_buffers
                        .insert(format!("STRING_BUFFER_{buffer}"), name.clone());
                }
                cpu.selected_party_index = Some(party_index);
                Ok(true)
            }
            BranchingCallasmHostService::GetNickname => {
                if cpu.registers.get("hl").map(String::as_str) != Some("wPartyMonNicknames") {
                    return Ok(false);
                }
                let party_index = cpu
                    .registers
                    .get("a")
                    .and_then(|value| parse_script_i32(value).ok())
                    .and_then(|value| usize::try_from(value).ok());
                let Some(party_index) = party_index else {
                    return Ok(false);
                };
                let Some(name) = branching_callasm_party_nickname(state, party_index) else {
                    return Ok(false);
                };
                state
                    .script_runtime
                    .named_buffers
                    .insert("STRING_BUFFER_1".to_string(), name);
                cpu.registers
                    .insert("de".to_string(), "wStringBuffer1".to_string());
                cpu.selected_party_index = Some(party_index);
                Ok(true)
            }
            BranchingCallasmHostService::CopyName1 => {
                let Some(source) = cpu.registers.get("de") else {
                    return Ok(false);
                };
                let Some(name) = branching_callasm_named_buffer(state, source) else {
                    return Ok(false);
                };
                state
                    .script_runtime
                    .named_buffers
                    .insert("STRING_BUFFER_2".to_string(), name);
                Ok(true)
            }
            BranchingCallasmHostService::CopyName2 => {
                let (Some(source), Some(destination)) =
                    (cpu.registers.get("de"), cpu.registers.get("hl"))
                else {
                    return Ok(false);
                };
                let Some(destination) = branching_callasm_named_buffer_key(destination) else {
                    return Ok(false);
                };
                let Some(name) = branching_callasm_named_buffer(state, source) else {
                    return Ok(false);
                };
                state
                    .script_runtime
                    .named_buffers
                    .insert(destination.to_string(), name);
                Ok(true)
            }
            BranchingCallasmHostService::UpdateSprites
            | BranchingCallasmHostService::UpdatePlayerSprite => Ok(true),
            BranchingCallasmHostService::CheckWaterfallTile => {
                let Some(value) = cpu
                    .registers
                    .get("a")
                    .and_then(|value| parse_script_i32(value).ok())
                else {
                    return Ok(false);
                };
                let value = u8::try_from(value).with_context(|| {
                    format!("CheckWaterfallTile accumulator {value} is outside byte range")
                })?;
                cpu.flags.zero =
                    Some([permissions::WATERFALL, permissions::CURRENT_DOWN].contains(&value));
                Ok(true)
            }
            BranchingCallasmHostService::StubbedTrainerRankingsWaterfall => Ok(true),
        }
    }

    /// Execute Crystal's local NPC trade after the script command has passed
    /// through the core runtime. The ASM removes the selected requested
    /// species and appends a freshly built traded Pokémon with the table's
    /// nickname, DVs, held item, OT, and caught-data provenance.
    fn apply_npc_trade(
        &self,
        state: &mut GameState,
        trade_id: Option<&str>,
        selected_party_index: Option<usize>,
    ) -> Result<()> {
        let Some(trade_id) = trade_id else {
            anyhow::bail!("NPC trade command is missing its trade id");
        };
        let rule = self
            .npc_trades
            .get(trade_id)
            .with_context(|| format!("NPC trade {trade_id} is missing from the compiled pack"))?;
        let Some(party_index) = selected_party_index else {
            set_npc_trade_result(state, 0);
            return Ok(());
        };
        let requested = rule.requested_species.as_str();
        let Some(requested_mon) = state
            .storage
            .party
            .pokemon
            .get(party_index)
            .and_then(Option::as_ref)
        else {
            set_npc_trade_result(state, 0);
            return Ok(());
        };
        if requested_mon.species.id != requested {
            set_npc_trade_result(state, 1);
            return Ok(());
        }
        let offered_species = self.pokemon.get(&rule.offered_species).with_context(|| {
            format!(
                "NPC trade {trade_id} references unknown offered species {}",
                rule.offered_species
            )
        })?;
        if rule.gender_requirement.ends_with("FEMALE")
            && !(requested_mon.species.gender_ratio == 254
                || (requested_mon.species.gender_ratio != 0
                    && requested_mon.dvs.attack.saturating_mul(17)
                        < requested_mon.species.gender_ratio))
        {
            set_npc_trade_result(state, 1);
            return Ok(());
        }
        let level = requested_mon.level;
        let dvs = rule.dvs.as_slice();
        if dvs.len() != 2 {
            anyhow::bail!("NPC trade {trade_id} has invalid two-byte DVs");
        }
        let traded_dvs = Dv::from_non_hp(dvs[0] >> 4, dvs[0] & 0x0f, dvs[1] >> 4, dvs[1] & 0x0f);
        let mut traded = create_pokemon_from_known_dvs(
            offered_species,
            level,
            traded_dvs,
            &self.learnsets,
            &self.moves,
            &self.growth_rates,
        )
        .map_err(|error| anyhow::anyhow!("build NPC trade {trade_id} Pokémon: {error}"))?;
        traded.nickname = rule.nickname.clone();
        traded.item = (!rule.held_item.is_empty()).then(|| rule.held_item.clone());
        traded.original_trainer_name = rule.original_trainer_name.clone();
        traded.original_trainer_id = rule.original_trainer_id;
        traded.caught_data = Some(crystal_core::models::pokemon::CaughtData {
            level: 0,
            time_of_day: None,
            // SetGiftPartyMonCaughtData writes LANDMARK_GIFT ($7e); the
            // girl trade variant carries the CAUGHT_BY_GIRL bit separately
            // in the high bit of the packed caught-location byte.
            original_trainer_gender: u8::from(rule.dialog_set.ends_with("GIRL")),
            location: 0x7e,
        });
        let last_party_index = state
            .storage
            .party
            .pokemon
            .iter()
            .rposition(Option::is_some)
            .context("NPC trade party unexpectedly became empty")?;
        for slot in party_index..last_party_index {
            state.storage.party.pokemon[slot] = state.storage.party.pokemon[slot + 1].take();
        }
        state.storage.party.pokemon[last_party_index] = Some(traded);
        state.sync_party_from_storage();
        state
            .script_runtime
            .completed_trades
            .push(trade_id.to_string());
        set_npc_trade_result(state, 2);
        Ok(())
    }

    fn apply_compiled_mail_definition(&self, state: &mut GameState, label: &str) -> Result<()> {
        let body = self
            .compiled_script_body(label)
            .with_context(|| format!("mail definition '{label}' is missing from compiled ASM"))?;
        let entries = body
            .as_array()
            .with_context(|| format!("mail definition '{label}' is not an array"))?;
        let item_id = entries
            .first()
            .and_then(|entry| entry.get("args"))
            .and_then(serde_json::Value::as_array)
            .and_then(|args| args.first())
            .and_then(serde_json::Value::as_str)
            .with_context(|| format!("mail definition '{label}' has no item"))?
            .to_string();
        self.item(&item_id).with_context(|| {
            format!("mail definition '{label}' references unknown item '{item_id}'")
        })?;
        let message = compiled_mail_message(entries, true)?;
        let Some(index) = state
            .storage
            .party
            .pokemon
            .iter()
            .rposition(Option::is_some)
        else {
            anyhow::bail!("cannot give mail '{label}' without a party Pokémon");
        };
        let pokemon = state.storage.party.pokemon[index]
            .as_mut()
            .expect("party index selected from occupied slots");
        pokemon.item = Some(item_id.clone());
        pokemon.mail = Some(crystal_core::models::pokemon::MailData {
            message,
            author: pokemon.original_trainer_name.clone(),
            nationality: 0,
            author_id: pokemon.original_trainer_id,
            species: pokemon.species.id.clone(),
            mail_type: item_id,
        });
        state.sync_party_from_storage();
        Ok(())
    }

    fn apply_compiled_mail_check(
        &self,
        state: &mut GameState,
        label: &str,
        selected_party_index: Option<usize>,
    ) -> Result<()> {
        let body = self.compiled_script_body(label).with_context(|| {
            format!("mail check definition '{label}' is missing from compiled ASM")
        })?;
        let entries = body
            .as_array()
            .with_context(|| format!("mail check definition '{label}' is not an array"))?;
        let expected = compiled_mail_message(entries, false)?;
        let Some(index) = selected_party_index else {
            set_compiled_mail_check_result(state, 2);
            return Ok(());
        };
        let Some(pokemon) = state
            .storage
            .party
            .pokemon
            .get(index)
            .and_then(Option::as_ref)
        else {
            set_compiled_mail_check_result(state, 2);
            return Ok(());
        };
        if !pokemon
            .item
            .as_deref()
            .is_some_and(crystal_core::models::item::is_mail_item_id)
            || pokemon.mail.is_none()
        {
            set_compiled_mail_check_result(state, 3);
            return Ok(());
        }
        if pokemon
            .mail
            .as_ref()
            .is_some_and(|mail| !strip_compiled_mail_text(&mail.message).starts_with(&expected))
        {
            set_compiled_mail_check_result(state, 0);
            return Ok(());
        }
        let party_len = state.storage.party.pokemon.len();
        let other_conscious =
            state
                .storage
                .party
                .pokemon
                .iter()
                .enumerate()
                .any(|(other_index, pokemon)| {
                    other_index != index && pokemon.as_ref().is_some_and(|pokemon| pokemon.hp > 0)
                });
        if !other_conscious {
            set_compiled_mail_check_result(state, 4);
            return Ok(());
        }
        for slot in index..(party_len - 1) {
            state.storage.party.pokemon[slot] = state.storage.party.pokemon[slot + 1].take();
        }
        state.storage.party.pokemon[party_len - 1] = None;
        state.sync_party_from_storage();
        set_compiled_mail_check_result(state, 1);
        Ok(())
    }

    fn resolve_script_runtime_name_buffer_value(
        &self,
        state: &GameState,
        map_name: &str,
        command: &ScriptRuntimeCommand,
    ) -> Result<Option<String>> {
        let resolved_name = match command.command.as_str() {
            "gettrainername" => {
                let trainer_id = command.args.get(2).with_context(|| {
                    format!(
                        "gettrainername command {}:{} missing trainer id",
                        command.source_script, command.command_index
                    )
                })?;
                let trainer = self.trainers.get(trainer_id).with_context(|| {
                    format!("gettrainername references unknown trainer {trainer_id}")
                })?;
                Some(trainer.name.clone())
            }
            "getitemname" => {
                let item_arg = command.args.get(1).with_context(|| {
                    format!(
                        "getitemname command {}:{} missing item id",
                        command.source_script, command.command_index
                    )
                })?;
                let item_id = if item_arg == SCRIPT_RUNTIME_USE_SCRIPT_VAR_ID {
                    Some(state.script_runtime.script_value.as_deref().with_context(|| {
                        format!(
                            "getitemname command {}:{} requires script_value for USE_SCRIPT_VAR",
                            command.source_script, command.command_index
                        )
                    })?)
                } else {
                    Some(item_arg.as_str())
                };
                if let Some(item_id) = item_id {
                    let item = self.items.get(item_id).with_context(|| {
                        format!("getitemname references unknown item {item_id}")
                    })?;
                    Some(item.name.clone())
                } else {
                    None
                }
            }
            "getmonname" => {
                let species_arg = command.args.get(1).with_context(|| {
                    format!(
                        "getmonname command {}:{} missing species id",
                        command.source_script, command.command_index
                    )
                })?;
                let species_id =
                    if species_arg == SCRIPT_RUNTIME_USE_SCRIPT_VAR_ID {
                        state.script_runtime.script_value.as_deref().with_context(|| {
                        format!(
                            "getmonname command {}:{} requires script_value for USE_SCRIPT_VAR",
                            command.source_script, command.command_index
                        )
                    })?
                    } else {
                        species_arg.as_str()
                    };
                self.pokemon.get(species_id).with_context(|| {
                    format!("getmonname references unknown species {species_id}")
                })?;
                Some(pokemon_species_display_name(species_id))
            }
            "getstring" => {
                let label = command.args.get(1).with_context(|| {
                    format!(
                        "getstring command {}:{} missing string label",
                        command.source_script, command.command_index
                    )
                })?;
                let definitions =
                    if let Some(module) = self.global_script_module_for(&command.source_script) {
                        &module.definitions
                    } else {
                        &self.map_module(map_name)?.scripts
                    };
                let resolved =
                    resolve_script_target_label(definitions, &command.source_script, label)
                        .with_context(|| {
                            format!(
                                "getstring command {}:{} references missing string label {}",
                                command.source_script, command.command_index, label
                            )
                        })?;
                let entries = definitions
                    .get(&resolved)
                    .and_then(Value::as_array)
                    .with_context(|| format!("getstring label {resolved} is not a command body"))?;
                let text = entries
                    .iter()
                    .map(|entry| {
                        let command_name = entry
                            .get("command")
                            .and_then(Value::as_str)
                            .context("string data command has no command name")?;
                        if command_name != "db" {
                            anyhow::bail!(
                                "getstring label {resolved} contains non-db command {command_name}"
                            );
                        }
                        let args = entry
                            .get("args")
                            .and_then(Value::as_array)
                            .context("string data db command has no args")?;
                        if args.len() != 1 {
                            anyhow::bail!(
                                "getstring label {resolved} db command requires one argument"
                            );
                        }
                        let value = args[0]
                            .as_str()
                            .context("string data db argument is not a string")?;
                        Ok(strip_compiled_mail_text(value))
                    })
                    .collect::<Result<Vec<_>>>()?
                    .join("");
                Some(text)
            }
            "getlandmarkname" => {
                let landmark_id = command.args.get(1).with_context(|| {
                    format!(
                        "getlandmarkname command {}:{} missing landmark id",
                        command.source_script, command.command_index
                    )
                })?;
                let landmark = self
                    .pokegear_landmarks
                    .landmarks
                    .iter()
                    .find(|landmark| landmark.constant == *landmark_id)
                    .with_context(|| {
                        format!(
                            "getlandmarkname command {}:{} references unknown landmark {}",
                            command.source_script, command.command_index, landmark_id
                        )
                    })?;
                Some(landmark.name.clone())
            }
            "gettrainerclassname" => {
                let trainer_class = command.args.get(1).with_context(|| {
                    format!(
                        "gettrainerclassname command {}:{} missing trainer class id",
                        command.source_script, command.command_index
                    )
                })?;
                Some(
                    self.trainer_class_names
                        .get(trainer_class)
                        .with_context(|| {
                            format!(
                                "gettrainerclassname command {}:{} references unknown trainer class {}",
                                command.source_script, command.command_index, trainer_class
                            )
                        })?
                        .clone(),
                )
            }
            _ => None,
        };
        Ok(resolved_name)
    }

    fn resolve_script_runtime_decoration(
        &self,
        state: &GameState,
        command: &ScriptRuntimeCommand,
    ) -> Result<Option<ScriptRuntimeDecorationResolution>> {
        if command.command != "describedecoration" {
            return Ok(None);
        }
        let selector = command.args.first().with_context(|| {
            format!(
                "describedecoration command {}:{} has no selector",
                command.source_script, command.command_index
            )
        })?;
        let memory_value = |key: &str| {
            state
                .script_runtime
                .memory
                .get(key)
                .map(String::as_str)
                .unwrap_or("0")
        };
        let resolution = match selector.as_str() {
            "DECODESC_POSTER" => ScriptRuntimeDecorationResolution {
                target_script: match memory_value("wDecoPoster") {
                    "0" => "DecorationDesc_NullPoster",
                    "DECO_TOWN_MAP" => "DecorationDesc_TownMapPoster",
                    "DECO_PIKACHU_POSTER" => "DecorationDesc_PikachuPoster",
                    "DECO_CLEFAIRY_POSTER" => "DecorationDesc_ClefairyPoster",
                    "DECO_JIGGLYPUFF_POSTER" => "DecorationDesc_JigglypuffPoster",
                    other => anyhow::bail!("unknown equipped poster decoration {other}"),
                }
                .to_string(),
                string_buffer_3: None,
            },
            "DECODESC_LEFT_DOLL" | "DECODESC_RIGHT_DOLL" | "DECODESC_CONSOLE" => {
                let key = match selector.as_str() {
                    "DECODESC_LEFT_DOLL" => "wDecoLeftOrnament",
                    "DECODESC_RIGHT_DOLL" => "wDecoRightOrnament",
                    "DECODESC_CONSOLE" => "wDecoConsole",
                    _ => unreachable!(),
                };
                let decoration = memory_value(key);
                ScriptRuntimeDecorationResolution {
                    target_script: ".OrnamentConsoleScript@DecorationDesc_OrnamentOrConsole"
                        .to_string(),
                    string_buffer_3: Some(decoration_display_name(decoration)?.to_string()),
                }
            }
            "DECODESC_BIG_DOLL" => ScriptRuntimeDecorationResolution {
                target_script: ".BigDollScript@DecorationDesc_GiantOrnament".to_string(),
                string_buffer_3: None,
            },
            other => anyhow::bail!("unknown describedecoration selector {other}"),
        };
        Ok(Some(resolution))
    }

    fn resolve_script_runtime_stone_table_queue(
        &self,
        map_name: &str,
        command: &ScriptRuntimeCommand,
    ) -> Result<Option<Vec<ScriptRuntimeStoneTableEntry>>> {
        if command.command != "writecmdqueue" {
            return Ok(None);
        }
        let definitions =
            if let Some(module) = self.global_script_module_for(&command.source_script) {
                &module.definitions
            } else {
                &self.map_module(map_name)?.scripts
            };
        let queue_target = command.args.first().with_context(|| {
            format!(
                "writecmdqueue command {}:{} has no data pointer",
                command.source_script, command.command_index
            )
        })?;
        let queue_label =
            resolve_script_target_label(definitions, &command.source_script, queue_target)
                .with_context(|| {
                    format!(
                        "writecmdqueue command {}:{} cannot resolve queue data {}",
                        command.source_script, command.command_index, queue_target
                    )
                })?;
        let queue_body = definitions
            .get(&queue_label)
            .and_then(Value::as_array)
            .with_context(|| format!("command queue {queue_label} has no command body"))?;
        anyhow::ensure!(
            queue_body.len() == 1,
            "command queue {queue_label} must contain exactly one cmdqueue entry, found {}",
            queue_body.len()
        );
        let queue_entry = &queue_body[0];
        let queue_command = queue_entry
            .get("command")
            .and_then(Value::as_str)
            .context("command queue entry has no command")?;
        anyhow::ensure!(
            queue_command == "cmdqueue",
            "command queue {queue_label} contains {queue_command} instead of cmdqueue"
        );
        let queue_args = script_command_args(map_name, &queue_label, queue_command, queue_entry)?;
        anyhow::ensure!(
            queue_args.len() == 2 && queue_args[0] == "CMDQUEUE_STONETABLE",
            "command queue {queue_label} is not an exact CMDQUEUE_STONETABLE entry"
        );
        let table_label = resolve_script_target_label(definitions, &queue_label, queue_args[1])
            .with_context(|| {
                format!(
                    "stone command queue {queue_label} cannot resolve table {}",
                    queue_args[1]
                )
            })?;
        let table_body = definitions
            .get(&table_label)
            .and_then(Value::as_array)
            .with_context(|| format!("stone table {table_label} has no command body"))?;
        let mut entries = Vec::new();
        let mut terminated = false;
        for (command_index, entry) in table_body.iter().enumerate() {
            let entry_command =
                entry
                    .get("command")
                    .and_then(Value::as_str)
                    .with_context(|| {
                        format!("stone table {table_label} command {command_index} has no command")
                    })?;
            let args = script_command_args(map_name, &table_label, entry_command, entry)?;
            if entry_command == "db" && args == ["-1"] {
                anyhow::ensure!(
                    command_index + 1 == table_body.len(),
                    "stone table {table_label} has data after its -1 terminator"
                );
                terminated = true;
                break;
            }
            anyhow::ensure!(
                entry_command == "stonetable" && args.len() == 3,
                "stone table {table_label} command {command_index} is not an exact stonetable entry"
            );
            let warp_value = parse_script_i32_token("stonetable", args[0])?;
            let warp = u16::try_from(warp_value).with_context(|| {
                format!(
                    "stone table {table_label} command {command_index} warp {} is outside u16",
                    args[0]
                )
            })?;
            let script = resolve_script_target_label(definitions, &table_label, args[2])
                .with_context(|| {
                    format!(
                        "stone table {table_label} command {command_index} cannot resolve script {}",
                        args[2]
                    )
                })?;
            entries.push(ScriptRuntimeStoneTableEntry {
                // The core assigns the first free wCmdQueue slot atomically.
                queue_slot: 0,
                warp,
                object_event: args[1].to_string(),
                script,
                source_script: table_label.clone(),
                command_index,
            });
        }
        anyhow::ensure!(terminated, "stone table {table_label} has no -1 terminator");
        anyhow::ensure!(
            !entries.is_empty(),
            "stone table {table_label} has no entries"
        );
        Ok(Some(entries))
    }

    pub fn apply_script_runtime_command_in_session(
        &self,
        state: &mut GameState,
        overworld: &mut OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
        inputs: ScriptRuntimeInputs,
    ) -> Result<(ScriptRuntimeCommand, ScriptRuntimeOutcome)> {
        let current_map = overworld.map.name.clone();
        self.apply_script_runtime_command(
            state,
            overworld,
            &current_map,
            map_name,
            source_script,
            command_index,
            inputs,
        )
    }

    pub fn apply_random_script_runtime_command_in_session<S>(
        &self,
        state: &mut GameState,
        overworld: &OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
        divider: &mut S,
    ) -> Result<(ScriptRuntimeCommand, ScriptRuntimeOutcome)>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        self.require_current_map(&overworld.map.name, map_name)?;
        let command = self
            .script_runtime_command(map_name, source_script, command_index)?
            .clone();
        let mut next_state = state.clone();
        let outcome = core_apply_script_random_command(&mut next_state, command.clone(), divider)
            .map_err(|error| anyhow::anyhow!("apply script random command: {error}"))?;
        *state = next_state;
        Ok((command, outcome))
    }

    pub fn apply_happiness_service_with_divider<S>(
        &self,
        state: &mut GameState,
        routine: RuntimeHappinessServiceRoutine,
        party_index: usize,
        music_ids: &BTreeSet<String>,
        divider: &mut S,
    ) -> Result<SpecialRoutineOutcome>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        let pokemon = state
            .storage
            .party
            .pokemon
            .get(party_index)
            .and_then(Option::as_ref)
            .with_context(|| format!("happiness service party slot {party_index} is empty"))?;
        anyhow::ensure!(
            !pokemon.is_egg,
            "happiness service party slot {party_index} is an Egg"
        );
        let routine_name = match routine {
            RuntimeHappinessServiceRoutine::OlderHaircutBrother => "OlderHaircutBrother",
            RuntimeHappinessServiceRoutine::YoungerHaircutBrother => "YoungerHaircutBrother",
            RuntimeHappinessServiceRoutine::DaisysGrooming => "DaisysGrooming",
        };
        let mut next_state = state.clone();
        let mut rng = CrystalRandom::new(next_state.random_state, divider);
        let roll = rng
            .random(true)
            .map_err(|error| anyhow::anyhow!("apply happiness service Random: {error}"))?
            .value;
        next_state.random_state = rng.state();
        next_state
            .script_runtime
            .variables
            .insert("_party_slot".to_string(), party_index.to_string());
        next_state
            .script_runtime
            .variables
            .insert("_rng_roll".to_string(), roll.to_string());
        let outcome = self.apply_special_routine(&mut next_state, routine_name, music_ids)?;
        *state = next_state;
        Ok(outcome)
    }

    pub fn script_runtime_live_object_reference(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<Option<String>> {
        let command = self.script_runtime_command(map_name, source_script, command_index)?;
        Self::script_runtime_live_object_reference_for_command(command)
    }

    fn script_runtime_live_object_reference_for_command(
        command: &ScriptRuntimeCommand,
    ) -> Result<Option<String>> {
        if command.command != "setlasttalked" {
            return Ok(None);
        }
        let object_id = command
            .args
            .first()
            .with_context(|| "setlasttalked command missing object id")?;
        if object_id == "-1" {
            return Ok(None);
        }
        Ok(Some(object_id.clone()))
    }

    fn require_runtime_object_reference(
        overworld: &OverworldSession,
        object_id: &str,
    ) -> Result<()> {
        if object_id == "PLAYER" {
            return Ok(());
        }
        if overworld
            .objects
            .iter()
            .any(|object| object.object_identifier.as_deref() == Some(object_id))
        {
            return Ok(());
        }
        anyhow::bail!(
            "runtime command references missing exact object id {object_id} on {}",
            overworld.map.name
        );
    }

    pub fn script_shop_command(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<&ScriptShopCommand> {
        if let Some(module) = self.global_script_module_for(source_script) {
            return find_script_entry(
                &module.script_shop_commands,
                map_name,
                "global script shop command",
                source_script,
                command_index,
                |command| (&command.source_script, command.command_index),
            );
        }
        find_script_entry(
            &self.map_module(map_name)?.script_shop_commands,
            map_name,
            "script shop command",
            source_script,
            command_index,
            |command| (&command.source_script, command.command_index),
        )
    }

    pub fn open_script_shop(
        &self,
        state: &mut GameState,
        current_map: &str,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptShopOutcome> {
        self.require_current_map(current_map, map_name)?;
        let command = self
            .script_shop_command(map_name, source_script, command_index)?
            .clone();
        core_apply_script_shop_command(state, &self.marts, &self.items, command)
            .map_err(|error| anyhow::anyhow!("open script shop: {error:?}"))
    }

    pub fn open_script_shop_in_session(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Result<ScriptShopOutcome> {
        self.open_script_shop(
            state,
            &session.map.name,
            map_name,
            source_script,
            command_index,
        )
    }

    fn select_vertical_menu_option(
        &self,
        state: &mut GameState,
        command: RuntimeVerticalMenuSelectionCommand,
    ) -> Result<RuntimeVerticalMenuSelection> {
        let active_menu = state
            .script_runtime
            .active_menu
            .as_deref()
            .context("cannot select vertical menu option because no menu is active")?;
        if active_menu != command.menu_id {
            anyhow::bail!(
                "cannot select vertical menu option for '{}' because active menu is '{}'",
                command.menu_id,
                active_menu
            );
        }
        let matches: Vec<_> = self
            .maps
            .values()
            .flat_map(|module| module.script_vertical_menus.values())
            .chain(
                self.global_scripts
                    .iter()
                    .flat_map(|module| module.script_vertical_menus.values()),
            )
            .filter(|menu| {
                menu.header_label == command.menu_id.as_str()
                    && menu.source_script == command.source_script.as_str()
                    && menu.verticalmenu_command_index == command.verticalmenu_command_index
            })
            .collect();
        if matches.is_empty() {
            anyhow::bail!(
                "compiled pack does not declare vertical menu '{}' from {} command {}",
                command.menu_id,
                command.source_script,
                command.verticalmenu_command_index
            );
        }
        if matches.len() > 1 {
            anyhow::bail!(
                "compiled pack declares duplicate vertical menu '{}' from {} command {}",
                command.menu_id,
                command.source_script,
                command.verticalmenu_command_index
            );
        }
        let menu = matches[0];
        let Some(expected_option) = menu.options.get(command.option_index) else {
            anyhow::bail!(
                "vertical menu '{}' option index {} is out of range for {} options",
                command.menu_id,
                command.option_index,
                menu.options.len()
            );
        };
        if expected_option != &command.option {
            anyhow::bail!(
                "vertical menu '{}' option index {} is '{}', not '{}'",
                command.menu_id,
                command.option_index,
                expected_option,
                command.option
            );
        }
        let script_value = (command.option_index + 1).to_string();
        state.script_runtime.script_value = Some(script_value.clone());
        state
            .script_runtime
            .memory
            .insert("wScriptVar".to_string(), script_value.clone());
        Ok(RuntimeVerticalMenuSelection {
            menu_id: command.menu_id,
            source_script: command.source_script,
            verticalmenu_command_index: command.verticalmenu_command_index,
            option_index: command.option_index,
            option: command.option,
            script_value,
        })
    }

    fn open_vertical_menu(
        &self,
        state: &mut GameState,
        command: RuntimeVerticalMenuOpenCommand,
    ) -> Result<RuntimeVerticalMenuOpen> {
        let menu = if let Some(module) = self.global_script_module_for(&command.source_script) {
            module.script_vertical_menus.get(&command.menu_key)
        } else {
            self.maps
                .get(&command.map_name)
                .with_context(|| {
                    format!("compiled pack does not declare map {}", command.map_name)
                })?
                .script_vertical_menus
                .get(&command.menu_key)
        }
        .with_context(|| {
            format!(
                "compiled pack does not declare vertical menu {} on {}",
                command.menu_key, command.map_name
            )
        })?;
        if menu.source_script != command.source_script {
            anyhow::bail!(
                "vertical menu {} on {} belongs to source script {}, not {}",
                command.menu_key,
                command.map_name,
                menu.source_script,
                command.source_script
            );
        }
        if menu.loadmenu_command_index != command.loadmenu_command_index {
            anyhow::bail!(
                "vertical menu {} on {} has loadmenu command {}, not {}",
                command.menu_key,
                command.map_name,
                menu.loadmenu_command_index,
                command.loadmenu_command_index
            );
        }
        if menu.verticalmenu_command_index != command.verticalmenu_command_index {
            anyhow::bail!(
                "vertical menu {} on {} has verticalmenu command {}, not {}",
                command.menu_key,
                command.map_name,
                menu.verticalmenu_command_index,
                command.verticalmenu_command_index
            );
        }
        state.script_runtime.active_menu = Some(menu.header_label.clone());
        state.script_runtime.window_open = true;
        Ok(RuntimeVerticalMenuOpen {
            map_name: command.map_name,
            menu_key: command.menu_key,
            menu_id: menu.header_label.clone(),
            source_script: command.source_script,
            loadmenu_command_index: command.loadmenu_command_index,
            verticalmenu_command_index: command.verticalmenu_command_index,
            options: menu.options.clone(),
        })
    }

    fn select_elevator_floor(
        &self,
        state: &mut GameState,
        command: RuntimeElevatorFloorSelectionCommand,
    ) -> Result<RuntimeElevatorFloorSelection> {
        let module = self
            .maps
            .get(&command.map_name)
            .with_context(|| format!("compiled pack does not declare map {}", command.map_name))?;
        let elevator_key = format!(
            "{}:{}",
            command.source_script, command.elevator_command_index
        );
        let elevator = module
            .script_elevators
            .get(&elevator_key)
            .with_context(|| {
                format!(
                    "compiled pack does not declare elevator '{}' from {} command {} on {}",
                    command.data_label,
                    command.source_script,
                    command.elevator_command_index,
                    command.map_name
                )
            })?;
        if elevator.source_script != command.source_script {
            anyhow::bail!(
                "elevator '{}' on {} belongs to source script {}, not {}",
                command.data_label,
                command.map_name,
                elevator.source_script,
                command.source_script
            );
        }
        if elevator.elevator_command_index != command.elevator_command_index {
            anyhow::bail!(
                "elevator '{}' on {} has command {}, not {}",
                command.data_label,
                command.map_name,
                elevator.elevator_command_index,
                command.elevator_command_index
            );
        }
        if elevator.data_label != command.data_label {
            anyhow::bail!(
                "elevator from {} command {} on {} uses data label {}, not {}",
                command.source_script,
                command.elevator_command_index,
                command.map_name,
                elevator.data_label,
                command.data_label
            );
        }
        let floor = elevator.floors.get(command.floor_index).with_context(|| {
            format!(
                "elevator '{}' floor index {} is out of range for {} floors",
                command.data_label,
                command.floor_index,
                elevator.floors.len()
            )
        })?;
        if floor.floor != command.floor.as_str()
            || floor.warp != command.warp
            || floor.target_map != command.target_map.as_str()
        {
            anyhow::bail!(
                "elevator '{}' floor index {} is ({}, {}, {}), not ({}, {}, {})",
                command.data_label,
                command.floor_index,
                floor.floor,
                floor.warp,
                floor.target_map,
                command.floor,
                command.warp,
                command.target_map
            );
        }
        let selected_floor = floor.floor.clone();
        let selected_warp = floor.warp;
        let selected_target_map = floor.target_map.clone();
        let destination_warp = self
            .maps
            .get(&selected_target_map)
            .with_context(|| {
                format!(
                    "elevator '{}' target map '{}' is missing from compiled pack",
                    command.data_label, selected_target_map
                )
            })?
            .events
            .warps
            .iter()
            .find(|warp| warp.index == selected_warp)
            .with_context(|| {
                format!(
                    "elevator '{}' target map '{}' does not declare warp {}",
                    command.data_label, selected_target_map, selected_warp
                )
            })?;
        let destination_tile =
            checked_runtime_map_event_tile(destination_warp.x, destination_warp.y).with_context(
                || {
                    format!(
                        "elevator '{}' target warp {} coordinate ({}, {}) overflows runtime tile coordinates",
                        command.data_label,
                        destination_warp.index,
                        destination_warp.x,
                        destination_warp.y
                    )
                },
            )?;
        let script_value = "1".to_string();
        state.script_runtime.script_value = Some(script_value.clone());
        state
            .script_runtime
            .memory
            .insert("wScriptVar".to_string(), script_value.clone());
        state.script_runtime.pending_script_warp = Some(ScriptWarpRequest {
            target_map: selected_target_map.clone(),
            tile: destination_tile,
            facing: None,
            source_script: elevator.source_script.clone(),
            command_index: elevator.elevator_command_index,
        });
        Ok(RuntimeElevatorFloorSelection {
            map_name: command.map_name,
            data_label: command.data_label,
            source_script: command.source_script,
            elevator_command_index: command.elevator_command_index,
            floor_index: command.floor_index,
            floor: selected_floor,
            warp: selected_warp,
            target_map: selected_target_map,
            destination_tile,
            script_value,
        })
    }

}
