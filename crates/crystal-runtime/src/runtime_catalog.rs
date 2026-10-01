impl CrystalRuntime {
    pub fn growth_rates(&self) -> &crystal_core::systems::experience::GrowthRateCatalog {
        &self.data.growth_rates
    }

    pub fn load_from_compiled_pack(
        asset_root: &AssetRoot,
        compiled_pack_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let loaded = asset_root.load_loaded_verified_compiled_game_pack(compiled_pack_path)?;
        Self::from_loaded_compiled_pack(asset_root, loaded)
    }

    pub fn from_loaded_compiled_pack(
        asset_root: &AssetRoot,
        loaded: LoadedCompiledGamePack,
    ) -> Result<Self> {
        crystal_assets::verify_compiled_game_pack_for_runtime(loaded.pack())?;
        let modpack = loaded
            .save_modpack_identity()
            .context("compute compiled game pack save identity")?;
        let (_, _, pack) = loaded.into_parts();
        Self::from_verified_compiled_pack(asset_root, pack, modpack)
    }

    #[cfg(any(test, feature = "test-fixtures"))]
    fn from_compiled_pack(
        asset_root: &AssetRoot,
        pack: CompiledGamePack,
        modpack: SaveModpackIdentity,
    ) -> Result<Self> {
        crystal_assets::verify_compiled_game_pack_for_runtime(&pack)?;
        Self::from_verified_compiled_pack(asset_root, pack, modpack)
    }

    // Both entry points above verify the owned pack before entering here.
    // No mutation occurs between verification and consuming its parts.
    fn from_verified_compiled_pack(
        asset_root: &AssetRoot,
        pack: CompiledGamePack,
        modpack: SaveModpackIdentity,
    ) -> Result<Self> {
        modpack.validate()?;
        let expected_id = pack.runtime_modpack_id()?;
        if modpack.id() != expected_id {
            anyhow::bail!(
                "compiled game pack identity '{}' does not match report manifest id '{}'",
                modpack.id(),
                expected_id
            );
        }
        // Full verification already compared the derived and stored identity.
        // Retain that verified value instead of hashing the same pack again.
        let (
            _,
            data,
            compiled_audio,
            audio_manifest,
            audio_compression,
            runtime_files,
            _,
            pack_identity,
        ) = pack.into_parts();
        let audio_manifest = if audio_compression.is_none()
            && audio_manifest.music.is_empty()
            && audio_manifest.sound_effects.is_empty()
            && audio_manifest.cries.is_empty()
        {
            ModpackAudioManifest::from_assets(&data.audio, &compiled_audio)?
        } else {
            audio_manifest
        };
        let audio_playback = ModpackAudioPlaybackPlan::from_manifest(&audio_manifest)?;
        // Every gameplay audio payload is compiled into the pack.  The
        // repository asset root remains relevant to build-time loading, but
        // is deliberately not consulted by the runtime so a release binary
        // plus pack is sufficient to play.
        let _ = asset_root;
        let audio = RuntimeAudioCatalog::from_game_data_owned(
            &data,
            compiled_audio,
            audio_manifest,
            audio_playback,
            audio_compression.as_deref(),
        )?;
        let map_catalog = Self::base_map_catalog_snapshot(&data);
        let runtime = Self {
            modpack,
            pack_identity,
            data,
            runtime_files,
            audio,
            viewport: GameViewport::default(),
            map_catalog,
            catalog_cache: Arc::new(OnceLock::new()),
        };
        // Build once while loading the pack. Every later presentation
        // snapshot shares these immutable catalogs in O(1).
        let _ = runtime
            .catalog_cache
            .set(runtime.build_static_catalog_cache());
        Ok(runtime)
    }

    pub fn modpack(&self) -> &SaveModpackIdentity {
        &self.modpack
    }

    pub fn data(&self) -> &GameDataSet {
        &self.data
    }

    pub fn audio(&self) -> &RuntimeAudioCatalog {
        &self.audio
    }

    pub fn runtime_file(&self, relative_path: &str) -> Option<&[u8]> {
        self.runtime_files.get(relative_path).map(Vec::as_slice)
    }

    pub fn has_runtime_files(&self) -> bool {
        !self.runtime_files.is_empty()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn install_browser_runtime_files(&self) -> Result<()> {
        BROWSER_RUNTIME_FILES
            .set(self.runtime_files.clone())
            .map_err(|_| anyhow::anyhow!("browser runtime files were already installed"))
    }

    /// Materialize the embedded non-audio presentation bundle into an
    /// isolated runtime asset root.  The Bevy renderer still consumes the
    /// existing path-based loaders, so mounting the pack here lets those
    /// loaders work on a clean machine without the repository checkout.
    pub fn materialize_runtime_files(&self) -> Result<AssetRoot> {
        let mount = std::env::temp_dir().join(format!(
            "crystal-pack-assets-{}-{}",
            std::process::id(),
            self.pack_identity.content_hash
        ));
        validate_compiled_runtime_files(&self.runtime_files)?;
        let materialization_plan = self
            .runtime_files
            .iter()
            .map(|(relative, bytes)| {
                let path = if let Some(vendor_relative) = relative.strip_prefix("vendor/") {
                    mount.join("vendor").join(vendor_relative)
                } else {
                    mount.join("apps/web/assets").join(relative)
                };
                (path, bytes)
            })
            .collect::<Vec<_>>();
        let complete_marker = mount.join(".crystal-pack-assets-complete");
        if complete_marker.is_file() {
            return Ok(AssetRoot::new(mount));
        }
        for (path, bytes) in materialization_plan {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).with_context(|| {
                    format!("create embedded runtime asset mount {}", parent.display())
                })?;
            }
            std::fs::write(&path, bytes).with_context(|| {
                format!("materialize embedded runtime asset {}", path.display())
            })?;
        }
        std::fs::write(&complete_marker, self.pack_identity.content_hash.as_bytes()).with_context(
            || format!("finalize embedded runtime asset mount {}", mount.display()),
        )?;
        Ok(AssetRoot::new(mount))
    }

    pub fn title_music_id(&self) -> Result<&str> {
        self.data
            .runtime_title_screen
            .title_music
            .as_deref()
            .context("compiled pack title screen missing title music")
    }

    pub fn title_presentation_program(&self) -> &RuntimePresentationProgram {
        &self.data.runtime_title_screen.program
    }

    pub fn start_title_presentation(&self) -> Result<RuntimePresentationInterpreter> {
        RuntimePresentationInterpreter::new(self.title_presentation_program(), "title")
    }

    pub fn start_title_screen_subprogram(
        &self,
    ) -> Result<RuntimePresentationSubprogramInterpreter> {
        RuntimePresentationSubprogramInterpreter::new(
            self.title_presentation_program(),
            "start_title_screen",
            "title_screen",
        )
    }

    pub fn title_new_game_spawn_identifier(&self) -> Result<u16> {
        let value = self
            .data
            .story_event_script_constants
            .global
            .get("SPAWN_HOME")
            .context("compiled pack missing source constant SPAWN_HOME")?;
        let identifier = u16::try_from(*value)
            .with_context(|| format!("compiled source constant SPAWN_HOME={value} is not a u16"))?;
        anyhow::ensure!(
            self.data
                .runtime_spawn_points
                .contains_key(&identifier.to_string()),
            "compiled source constant SPAWN_HOME={identifier} has no runtime spawn point"
        );
        Ok(identifier)
    }

    pub fn special_routine_ids(&self) -> BTreeSet<String> {
        self.data.special_routines.keys().cloned().collect()
    }

    pub fn item_ids(&self) -> BTreeSet<String> {
        self.data.items.keys().cloned().collect()
    }

    pub fn move_ids(&self) -> BTreeSet<String> {
        self.data.moves.keys().cloned().collect()
    }

    pub fn species_ids(&self) -> BTreeSet<String> {
        self.data.pokemon.keys().cloned().collect()
    }

    pub fn map_ids(&self) -> BTreeSet<String> {
        self.data.maps.keys().cloned().collect()
    }

    pub fn trainer_ids(&self) -> BTreeSet<String> {
        self.data.trainers.trainers.keys().cloned().collect()
    }

    pub fn text_ids(&self) -> BTreeSet<String> {
        self.data
            .asm_text
            .keys()
            .cloned()
            .chain(
                self.data
                    .maps
                    .values()
                    .flat_map(|module| module.script_text_bodies.keys().cloned()),
            )
            .collect()
    }

    pub fn menu_ids(&self) -> BTreeSet<String> {
        self.data
            .special_routines
            .keys()
            .cloned()
            .chain(
                self.data
                    .maps
                    .values()
                    .flat_map(|module| module.script_menu_definitions.keys().cloned()),
            )
            .collect()
    }

    pub fn phone_contact_ids(&self) -> BTreeSet<String> {
        self.data.phone_contacts.0.keys().cloned().collect()
    }

    pub fn special_phone_call_ids(&self) -> BTreeSet<String> {
        self.data.special_phone_calls.keys().cloned().collect()
    }

    pub fn npc_trade_ids(&self) -> BTreeSet<String> {
        self.data.npc_trades.keys().cloned().collect()
    }

    pub fn sprite_ids(&self) -> BTreeSet<String> {
        self.data.sprite_palette_defaults.keys().cloned().collect()
    }

    pub fn map_constants(&self) -> BTreeSet<String> {
        self.data.runtime_map_metadata.keys().cloned().collect()
    }

    pub fn event_flag_ids(&self) -> BTreeSet<String> {
        let mut ids: BTreeSet<String> = self
            .data
            .initialize_events
            .event_flags
            .iter()
            .cloned()
            .collect();
        ids.extend(
            self.data
                .story_event_script_constants
                .global
                .keys()
                .cloned(),
        );
        ids.extend(
            self.data
                .story_event_script_constants
                .maps
                .values()
                .flat_map(|constants| constants.keys().cloned()),
        );
        if let Some(config) = &self.data.bug_contest_config {
            ids.extend(config.contestant_flags.iter().cloned());
        }
        ids.extend(
            self.data
                .decorations
                .decorations
                .iter()
                .map(|decoration| decoration.event_flag.clone()),
        );
        for module in self.data.maps.values() {
            ids.extend(
                module
                    .script_flag_commands
                    .iter()
                    .filter(|command| !crystal_core::state::is_engine_flag_name(&command.flag_id))
                    .map(|command| command.flag_id.clone()),
            );
            ids.extend(
                module
                    .scripts
                    .values()
                    .filter_map(|body| body.as_array())
                    .flat_map(|commands| {
                        commands.iter().filter_map(|command| {
                            (command.get("command").and_then(serde_json::Value::as_str)
                                == Some("conditional_event"))
                            .then(|| {
                                command
                                    .get("args")
                                    .and_then(serde_json::Value::as_array)
                                    .and_then(|args| args.first())
                                    .and_then(serde_json::Value::as_str)
                                    .map(str::to_string)
                            })
                            .flatten()
                        })
                    }),
            );
        }
        ids
    }

    pub fn engine_flag_ids(&self) -> BTreeSet<String> {
        let mut ids: BTreeSet<String> = self
            .data
            .initialize_events
            .engine_flags
            .iter()
            .cloned()
            .collect();
        ids.extend(
            self.data
                .story_event_script_constants
                .global
                .keys()
                .cloned(),
        );
        ids.extend(
            self.data
                .story_event_script_constants
                .maps
                .values()
                .flat_map(|constants| constants.keys().cloned()),
        );
        for module in self.data.maps.values() {
            ids.extend(
                module
                    .script_flag_commands
                    .iter()
                    .filter(|command| crystal_core::state::is_engine_flag_name(&command.flag_id))
                    .map(|command| command.flag_id.clone()),
            );
        }
        ids
    }

    pub fn spawn_identifiers(&self) -> BTreeSet<u16> {
        self.data
            .runtime_spawn_points
            .values()
            .map(|spawn| spawn.identifier)
            .collect()
    }

    pub fn tileset_ids(&self) -> BTreeSet<String> {
        self.data.tilesets.keys().cloned().collect()
    }

    pub fn tileset_keys(&self) -> BTreeSet<RuntimeTilesetKey> {
        self.data
            .tilesets
            .iter()
            .map(|(tileset_id, tileset)| RuntimeTilesetKey {
                tileset_id: tileset_id.clone(),
                collision: tileset.collision.clone(),
                palette_map: tileset.palette_map.clone(),
            })
            .collect()
    }

    pub fn pc_string_keys(&self) -> BTreeSet<RuntimePcStringKey> {
        self.data
            .pc_strings
            .iter()
            .map(|(string_id, text)| RuntimePcStringKey {
                string_id: string_id.clone(),
                text: text.clone(),
            })
            .collect()
    }

    pub fn menu_icon_keys(&self) -> BTreeSet<RuntimeMenuIconKey> {
        self.data
            .menu_icons
            .iter()
            .map(|(species_id, icon_id)| RuntimeMenuIconKey {
                species_id: species_id.clone(),
                icon_id: icon_id.clone(),
            })
            .collect()
    }

    pub fn pokedex_entry_keys(&self) -> BTreeSet<RuntimePokedexEntryKey> {
        self.data
            .pokedex_entries
            .iter()
            .map(|(species_id, entry)| RuntimePokedexEntryKey {
                species_id: species_id.clone(),
                species: entry.species.clone(),
                classification: entry.classification.clone(),
                height_digits: entry.height_digits,
                weight_digits: entry.weight_digits,
                pages: entry.pages.clone(),
            })
            .collect()
    }

    pub fn landmark_ids(&self) -> BTreeSet<String> {
        self.data
            .pokegear_landmarks
            .landmarks
            .iter()
            .map(|landmark| landmark.constant.clone())
            .collect()
    }

    pub fn pokegear_landmark_keys(&self) -> BTreeSet<RuntimePokegearLandmarkKey> {
        self.data
            .pokegear_landmarks
            .landmarks
            .iter()
            .map(|landmark| RuntimePokegearLandmarkKey {
                landmark_id: landmark.id,
                constant: landmark.constant.clone(),
                label: landmark.label.clone(),
                name: landmark.name.clone(),
                x: landmark.x,
                y: landmark.y,
                region: landmark.region.clone(),
            })
            .collect()
    }

    pub fn pokegear_map_landmark_keys(&self) -> BTreeSet<RuntimePokegearMapLandmarkKey> {
        self.data
            .pokegear_landmarks
            .map_to_landmark
            .iter()
            .map(
                |(map_name, landmark_constant)| RuntimePokegearMapLandmarkKey {
                    map_name: map_name.clone(),
                    landmark_constant: landmark_constant.clone(),
                },
            )
            .collect()
    }

    pub fn fishing_rod_ids(&self) -> BTreeSet<String> {
        self.data
            .fishing
            .groups
            .values()
            .flat_map(|group| group.rod_tables.keys().cloned())
            .collect()
    }

    pub fn map_group_ids(&self) -> BTreeSet<String> {
        self.data
            .runtime_map_metadata
            .values()
            .map(|metadata| metadata.group_name.clone())
            .collect()
    }

    pub fn encounter_group_ids(&self) -> BTreeSet<String> {
        self.data.fishing.groups.keys().cloned().collect()
    }

    pub fn mart_ids(&self) -> BTreeSet<String> {
        self.data.marts.0.keys().cloned().collect()
    }

    pub fn mart_keys(&self) -> BTreeSet<RuntimeMartKey> {
        self.data
            .marts
            .0
            .iter()
            .map(|(mart_id, item_ids)| RuntimeMartKey {
                mart_id: mart_id.clone(),
                item_ids: item_ids.clone(),
            })
            .collect()
    }

    pub fn fruit_tree_ids(&self) -> BTreeSet<String> {
        self.data.fruit_trees.0.keys().cloned().collect()
    }

    pub fn fruit_tree_keys(&self) -> BTreeSet<RuntimeFruitTreeKey> {
        self.data
            .fruit_trees
            .0
            .iter()
            .map(|(fruit_tree_id, item_id)| RuntimeFruitTreeKey {
                fruit_tree_id: fruit_tree_id.clone(),
                item_id: item_id.clone(),
            })
            .collect()
    }

    pub fn field_move_rule_ids(&self) -> BTreeSet<String> {
        [
            "cut",
            "whirlpool",
            "strength",
            "flash",
            "surf",
            "waterfall",
            "fly",
            "dig",
            "teleport",
            "escape_rope",
            "repel",
            "bicycle",
            "itemfinder",
            "squirtbottle",
            "coin_case",
            "blue_card",
            "town_map",
            "pokegear",
        ]
        .into_iter()
        .map(str::to_string)
        .collect()
    }

    pub fn field_move_rule_keys(&self) -> BTreeSet<RuntimeFieldMoveRuleKey> {
        field_move_rule_keys(&self.data.field_moves)
    }

    pub fn fly_destination_ids(&self) -> BTreeSet<String> {
        self.data.fly_destinations.keys().cloned().collect()
    }

    pub fn fly_destination_keys(&self) -> BTreeSet<RuntimeFlyDestinationKey> {
        fly_destination_keys(&self.data.fly_destinations)
    }

    pub fn field_move_move_ids(&self) -> BTreeSet<String> {
        field_move_move_ids(&self.data.field_moves)
    }

    pub fn field_move_item_ids(&self) -> BTreeSet<String> {
        field_move_item_ids(&self.data.field_moves)
    }

    pub fn field_box_item_ids(&self) -> BTreeSet<String> {
        self.data.field_box_items.keys().cloned().collect()
    }

    pub fn flee_mon_bucket_ids(&self) -> BTreeSet<String> {
        self.data.flee_mons.buckets.keys().cloned().collect()
    }

    pub fn buena_password_category_ids(&self) -> BTreeSet<String> {
        self.data
            .buena_password_categories
            .categories
            .keys()
            .cloned()
            .collect()
    }

    pub fn roaming_species_ids(&self) -> BTreeSet<String> {
        self.data
            .roaming_pokemon
            .init_writes
            .iter()
            .map(|write| write.species.clone())
            .collect()
    }

    pub fn buena_prize_item_ids(&self) -> BTreeSet<String> {
        self.data.buena_prizes.keys().cloned().collect()
    }

    pub fn kurt_apricorn_item_ids(&self) -> BTreeSet<String> {
        self.data.kurt_apricorn_recipes.keys().cloned().collect()
    }

    pub fn dratini_move_set_ids(&self) -> BTreeSet<u8> {
        self.data.dratini_move_sets.keys().copied().collect()
    }

    pub fn special_feature_ids(&self) -> BTreeSet<String> {
        let mut ids = BTreeSet::new();
        if self.data.shuckie_gift.is_some() {
            ids.insert("shuckie_gift".to_string());
        }
        if self.data.bug_contest_config.is_some() {
            ids.insert("bug_contest".to_string());
        }
        if self.data.battle_tower_rules.is_some() {
            ids.insert("battle_tower".to_string());
        }
        if !self.data.oak_ratings.is_empty() {
            ids.insert("oak_ratings".to_string());
        }
        if !self.data.odd_egg_definitions.is_empty() {
            ids.insert("odd_egg".to_string());
        }
        if !self.data.magikarp_lengths.is_empty() {
            ids.insert("magikarp_lengths".to_string());
        }
        if self.data.happiness_data.is_some() {
            ids.insert("happiness".to_string());
        }
        ids
    }

    pub fn oak_rating_text_ids(&self) -> BTreeSet<String> {
        self.data
            .oak_ratings
            .iter()
            .map(|rating| rating.text_label.clone())
            .collect()
    }

    pub fn odd_egg_species_ids(&self) -> BTreeSet<String> {
        self.data
            .odd_egg_definitions
            .iter()
            .map(|definition| definition.species.clone())
            .collect()
    }

    pub fn magikarp_length_thresholds(&self) -> BTreeSet<u16> {
        self.data
            .magikarp_lengths
            .iter()
            .map(|entry| entry.threshold)
            .collect()
    }

    pub fn happiness_change_ids(&self) -> BTreeSet<u8> {
        self.data
            .happiness_data
            .as_ref()
            .map(|data| data.changes.keys().copied().collect())
            .unwrap_or_default()
    }

    pub fn happiness_service_ids(&self) -> BTreeSet<String> {
        self.data
            .happiness_data
            .as_ref()
            .map(|data| data.services.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn pokemon_status_ids(&self) -> BTreeSet<String> {
        let mut ids = BTreeSet::from([
            self.data.step_event_rules.poison_status.clone(),
            "POKERUS".to_string(),
        ]);
        ids.extend(self.data.capture_rules.status_bonus.keys().cloned());
        ids.extend(
            self.data
                .items
                .values()
                .flat_map(|item| item.status_heals.iter().cloned()),
        );
        ids.retain(|status| !status.is_empty());
        ids
    }

    pub fn fishing_daily_flag_bits(&self) -> BTreeSet<u32> {
        self.data
            .fishing
            .swarm_rules
            .values()
            .map(|rule| u32::from(rule.daily_flag_bit))
            .collect()
    }

    pub fn fishing_swarm_flags(&self) -> BTreeSet<u8> {
        self.data
            .fishing
            .swarm_rules
            .values()
            .map(|rule| rule.swarm)
            .collect()
    }

    pub fn pending_special_battle_type_ids(&self) -> BTreeSet<String> {
        let mut ids = BTreeSet::new();
        for module in self.data.maps.values() {
            ids.extend(
                module
                    .scripted_trainer_battles
                    .iter()
                    .filter(|battle| !battle.request.battle_type.is_empty())
                    .map(|battle| battle.request.battle_type.clone()),
            );
            ids.extend(
                module
                    .scripted_wild_battles
                    .iter()
                    .filter(|battle| !battle.request.battle_type.is_empty())
                    .map(|battle| battle.request.battle_type.clone()),
            );
        }
        ids.extend(
            saved_special_battle_type_builtin_routines()
                .iter()
                .filter(|(_, routine)| self.data.special_routines.contains_key(*routine))
                .map(|(battle_type, _)| (*battle_type).to_string()),
        );
        ids
    }

    pub fn scripted_trainer_battle_keys(&self) -> BTreeSet<RuntimeScriptedTrainerBattleKey> {
        let mut keys = BTreeSet::new();
        for (map_name, module) in &self.data.maps {
            keys.extend(module.scripted_trainer_battles.iter().map(|battle| {
                RuntimeScriptedTrainerBattleKey {
                    map_name: map_name.clone(),
                    source_script: battle.source_script.clone(),
                    loadtrainer_command_index: battle.loadtrainer_command_index,
                    startbattle_command_index: battle.startbattle_command_index,
                    battle_type: battle.request.battle_type.clone(),
                    trainer_class: battle.request.trainer_class.clone(),
                    trainer_id: battle.request.trainer_id.clone(),
                }
            }));
            for (source_script, request) in &module.trainer_scripts {
                let Some(command_index) = module
                    .scripts
                    .get(source_script)
                    .and_then(serde_json::Value::as_array)
                    .and_then(|commands| {
                        commands.iter().position(|command| {
                            command.get("command").and_then(serde_json::Value::as_str)
                                == Some("trainer")
                        })
                    })
                else {
                    continue;
                };
                keys.insert(RuntimeScriptedTrainerBattleKey {
                    map_name: map_name.clone(),
                    source_script: source_script.clone(),
                    loadtrainer_command_index: command_index,
                    startbattle_command_index: command_index,
                    battle_type: request.battle_type.clone(),
                    trainer_class: request.trainer_class.clone(),
                    trainer_id: request.trainer_id.clone(),
                });
            }
        }
        keys
    }

    pub fn wild_encounter_origin_keys(&self) -> BTreeSet<RuntimeWildEncounterOriginKey> {
        let mut keys = BTreeSet::new();
        for (map_name, encounters) in &self.data.wild_encounters {
            collect_wild_encounter_keys(map_name, encounters, &mut keys);
        }
        for (map_name, encounters) in &self.data.field_encounters {
            collect_field_encounter_keys(map_name, encounters, &mut keys);
        }
        for (map_name, module) in &self.data.maps {
            let Some(group_name) = module.attributes.fishing_group.as_deref() else {
                continue;
            };
            let Some(group) = self.data.fishing.groups.get(group_name) else {
                continue;
            };
            collect_fishing_encounter_keys(
                map_name,
                group,
                &self.data.fishing.time_groups,
                &mut keys,
            );
        }
        keys
    }

    pub fn script_label_ids(&self) -> BTreeSet<String> {
        self.data
            .maps
            .values()
            .flat_map(|module| module.scripts.keys().cloned())
            .collect()
    }

    pub fn script_command_keys(&self) -> BTreeSet<RuntimeScriptCommandKey> {
        let mut keys = BTreeSet::new();
        for (script_label, body) in self
            .data
            .maps
            .values()
            .flat_map(|module| module.scripts.iter())
        {
            if let Some(commands) = body.as_array() {
                keys.extend(commands.iter().enumerate().map(|(command_index, _)| {
                    RuntimeScriptCommandKey {
                        script_label: script_label.clone(),
                        command_index,
                    }
                }));
            }
        }
        keys
    }

    pub fn script_command_payload_keys(&self) -> BTreeSet<RuntimeScriptCommandPayloadKey> {
        let mut keys = BTreeSet::new();
        for (script_label, body) in self
            .data
            .maps
            .values()
            .flat_map(|module| module.scripts.iter())
        {
            if let Some(commands) = body.as_array() {
                keys.extend(
                    commands
                        .iter()
                        .enumerate()
                        .filter_map(|(command_index, command)| {
                            let command_name = command.get("command")?.as_str()?;
                            let args = command
                                .get("args")
                                .and_then(|args| args.as_array())
                                .map(|args| {
                                    args.iter()
                                        .filter_map(|arg| arg.as_str().map(str::to_string))
                                        .collect::<Vec<_>>()
                                })
                                .unwrap_or_default();
                            Some(RuntimeScriptCommandPayloadKey {
                                script_label: script_label.clone(),
                                command_index,
                                command: command_name.to_string(),
                                args,
                            })
                        }),
                );
            }
        }
        keys
    }

    pub fn script_return_keys(&self) -> BTreeSet<RuntimeScriptReturnKey> {
        let mut keys = BTreeSet::new();
        for (script_label, body) in self
            .data
            .maps
            .values()
            .flat_map(|module| module.scripts.iter())
        {
            if let Some(commands) = body.as_array() {
                keys.extend((0..=commands.len()).map(|next_command_index| {
                    RuntimeScriptReturnKey {
                        script_label: script_label.clone(),
                        next_command_index,
                    }
                }));
            }
        }
        keys
    }

    pub fn script_vertical_menu_keys(&self) -> BTreeSet<RuntimeScriptVerticalMenuKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module
                    .script_vertical_menus
                    .iter()
                    .map(move |(menu_key, menu)| RuntimeScriptVerticalMenuKey {
                        map_name: map_name.clone(),
                        menu_key: menu_key.clone(),
                        source_script: menu.source_script.clone(),
                        loadmenu_command_index: menu.loadmenu_command_index,
                        verticalmenu_command_index: menu.verticalmenu_command_index,
                        header_label: menu.header_label.clone(),
                        data_label: menu.data_label.clone(),
                        options: menu.options.clone(),
                    })
            })
            .collect()
    }

    pub fn script_text_body_keys(&self) -> BTreeSet<RuntimeScriptTextBodyKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module
                    .script_text_bodies
                    .iter()
                    .map(move |(body_key, body)| RuntimeScriptTextBodyKey {
                        map_name: map_name.clone(),
                        body_key: body_key.clone(),
                        label: body.label.clone(),
                        commands: body
                            .commands
                            .iter()
                            .map(|command| RuntimeScriptTextBodyCommandKey {
                                command: command.command.clone(),
                                args: command.args.clone(),
                                command_index: command.command_index,
                            })
                            .collect(),
                    })
            })
            .collect()
    }

    pub fn script_menu_definition_keys(&self) -> BTreeSet<RuntimeScriptMenuDefinitionKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module
                    .script_menu_definitions
                    .iter()
                    .map(
                        move |(menu_key, definition)| RuntimeScriptMenuDefinitionKey {
                            map_name: map_name.clone(),
                            menu_key: menu_key.clone(),
                            label: definition.label.clone(),
                            commands: definition
                                .commands
                                .iter()
                                .map(|command| RuntimeScriptMenuCommandKey {
                                    command: command.command.clone(),
                                    args: command.args.clone(),
                                    command_index: command.command_index,
                                })
                                .collect(),
                        },
                    )
            })
            .collect()
    }

    pub fn script_elevator_keys(&self) -> BTreeSet<RuntimeScriptElevatorKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module
                    .script_elevators
                    .iter()
                    .map(move |(elevator_key, elevator)| RuntimeScriptElevatorKey {
                        map_name: map_name.clone(),
                        elevator_key: elevator_key.clone(),
                        source_script: elevator.source_script.clone(),
                        elevator_command_index: elevator.elevator_command_index,
                        data_label: elevator.data_label.clone(),
                        floors: elevator
                            .floors
                            .iter()
                            .map(|floor| RuntimeScriptElevatorFloorKey {
                                floor: floor.floor.clone(),
                                warp: floor.warp,
                                target_map: floor.target_map.clone(),
                                source_script: floor.source_script.clone(),
                                command_index: floor.command_index,
                            })
                            .collect(),
                    })
            })
            .collect()
    }

    pub fn gift_pokemon_keys(&self) -> BTreeSet<RuntimeGiftPokemonKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module
                    .gift_pokemon_scripts
                    .iter()
                    .map(move |gift| RuntimeGiftPokemonKey {
                        map_name: map_name.clone(),
                        species_id: gift.species_id.clone(),
                        level_token: gift.level_token.clone(),
                        level: gift.level,
                        held_item_id: gift.held_item_id.clone(),
                        nickname_label: gift.nickname_label.clone(),
                        ot_label: gift.ot_label.clone(),
                        source_script: gift.source_script.clone(),
                        command_index: gift.command_index,
                        egg: gift.egg,
                    })
            })
            .collect()
    }

    pub fn script_object_command_keys(&self) -> BTreeSet<RuntimeScriptObjectCommandKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module.script_object_commands.iter().map(move |command| {
                    RuntimeScriptObjectCommandKey {
                        map_name: map_name.clone(),
                        command: command.command.clone(),
                        object_id: command.object_id.clone(),
                        target_object_id: command.target_object_id.clone(),
                        x: command.x,
                        y: command.y,
                        direction: command.direction.clone(),
                        movement: command.movement.clone(),
                        emote: command.emote.clone(),
                        duration: command.duration,
                        source_script: command.source_script.clone(),
                        command_index: command.command_index,
                    }
                })
            })
            .collect()
    }

    pub fn script_movement_keys(&self) -> BTreeSet<RuntimeScriptMovementKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module
                    .script_movements
                    .iter()
                    .map(move |movement| RuntimeScriptMovementKey {
                        map_name: map_name.clone(),
                        label: movement.label.clone(),
                        source_script: movement.source_script.clone(),
                        steps: movement
                            .steps
                            .iter()
                            .map(|step| RuntimeScriptMovementStepKey {
                                command: step.command.clone(),
                                direction: step.direction.clone(),
                                duration: step.duration,
                                index: step.index,
                            })
                            .collect(),
                    })
            })
            .collect()
    }

    pub fn map_script_section_command_keys(&self) -> BTreeSet<RuntimeMapScriptSectionCommandKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module
                    .map_script_section_commands
                    .iter()
                    .map(move |command| RuntimeMapScriptSectionCommandKey {
                        map_name: map_name.clone(),
                        command: command.command.clone(),
                        args: command.args.clone(),
                        command_index: command.command_index,
                    })
            })
            .collect()
    }

    pub fn map_event_section_command_keys(&self) -> BTreeSet<RuntimeMapEventSectionCommandKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module
                    .map_event_section_commands
                    .iter()
                    .map(move |command| RuntimeMapEventSectionCommandKey {
                        map_name: map_name.clone(),
                        command: command.command.clone(),
                        args: command.args.clone(),
                        command_index: command.command_index,
                    })
            })
            .collect()
    }

    pub fn script_map_command_keys(&self) -> BTreeSet<RuntimeScriptMapCommandKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module
                    .script_map_commands
                    .iter()
                    .map(move |command| RuntimeScriptMapCommandKey {
                        map_name: map_name.clone(),
                        command: command.command.clone(),
                        target_map: command.target_map.clone(),
                        x: command.x,
                        y: command.y,
                        facing: command.facing.clone(),
                        map_setup: command.map_setup.clone(),
                        source_script: command.source_script.clone(),
                        command_index: command.command_index,
                    })
            })
            .collect()
    }

    pub fn script_variable_command_keys(&self) -> BTreeSet<RuntimeScriptVariableCommandKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module.script_variable_commands.iter().map(move |command| {
                    RuntimeScriptVariableCommandKey {
                        map_name: map_name.clone(),
                        command: command.command.clone(),
                        target: command.target.clone(),
                        value_tokens: command.value_tokens.clone(),
                        source_script: command.source_script.clone(),
                        command_index: command.command_index,
                    }
                })
            })
            .collect()
    }

    pub fn script_control_command_keys(&self) -> BTreeSet<RuntimeScriptControlCommandKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module.script_control_commands.iter().map(move |command| {
                    RuntimeScriptControlCommandKey {
                        map_name: map_name.clone(),
                        command: command.command.clone(),
                        compare_value: command.compare_value.clone(),
                        target_label: command.target_label.clone(),
                        resolved_target_script: command.resolved_target_script.clone(),
                        source_script: command.source_script.clone(),
                        command_index: command.command_index,
                    }
                })
            })
            .collect()
    }

    pub fn script_swarm_command_keys(&self) -> BTreeSet<RuntimeScriptSwarmCommandKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module.script_swarm_commands.iter().map(move |command| {
                    RuntimeScriptSwarmCommandKey {
                        map_name: map_name.clone(),
                        command: command.command.clone(),
                        swarm_token: command.swarm_token.clone(),
                        map_id: command.map_id.clone(),
                        source_script: command.source_script.clone(),
                        command_index: command.command_index,
                    }
                })
            })
            .collect()
    }

    pub fn script_field_pickup_keys(&self) -> BTreeSet<RuntimeScriptFieldPickupKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module
                    .script_field_pickups
                    .iter()
                    .map(move |pickup| RuntimeScriptFieldPickupKey {
                        map_name: map_name.clone(),
                        command: pickup.command.clone(),
                        item_id: pickup.item_id.clone(),
                        quantity: pickup.quantity,
                        event_flag: pickup.event_flag.clone(),
                        fruit_tree_id: pickup.fruit_tree_id.clone(),
                        source_script: pickup.source_script.clone(),
                        command_index: pickup.command_index,
                    })
            })
            .collect()
    }

    pub fn script_shop_command_keys(&self) -> BTreeSet<RuntimeScriptShopCommandKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module
                    .script_shop_commands
                    .iter()
                    .map(move |command| RuntimeScriptShopCommandKey {
                        map_name: map_name.clone(),
                        command: command.command.clone(),
                        mart_type: command.mart_type.clone(),
                        mart_id: command.mart_id.clone(),
                        source_script: command.source_script.clone(),
                        command_index: command.command_index,
                    })
            })
            .collect()
    }

    pub fn script_phone_command_keys(&self) -> BTreeSet<RuntimeScriptPhoneCommandKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module.script_phone_commands.iter().map(move |command| {
                    RuntimeScriptPhoneCommandKey {
                        map_name: map_name.clone(),
                        command: command.command.clone(),
                        contact_id: command.contact_id.clone(),
                        source_script: command.source_script.clone(),
                        command_index: command.command_index,
                    }
                })
            })
            .collect()
    }

    pub fn script_runtime_command_keys(&self) -> BTreeSet<RuntimeScriptRuntimeCommandKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module.script_runtime_commands.iter().map(move |command| {
                    RuntimeScriptRuntimeCommandKey {
                        map_name: map_name.clone(),
                        command: command.command.clone(),
                        args: command.args.clone(),
                        source_script: command.source_script.clone(),
                        command_index: command.command_index,
                    }
                })
            })
            .collect()
    }

    pub fn script_runtime_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> Option<RuntimeScriptRuntimeCommandKey> {
        self.data
            .script_runtime_command(map_name, source_script, command_index)
            .ok()
            .map(|command| RuntimeScriptRuntimeCommandKey {
                map_name: map_name.to_string(),
                command: command.command.clone(),
                args: command.args.clone(),
                source_script: command.source_script.clone(),
                command_index: command.command_index,
            })
    }

    pub fn script_item_grant_keys(&self) -> BTreeSet<RuntimeScriptItemGrantKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module
                    .script_item_grants
                    .iter()
                    .map(move |grant| RuntimeScriptItemGrantKey {
                        map_name: map_name.clone(),
                        command: grant.command.clone(),
                        item_id: grant.item_id.clone(),
                        quantity: grant.quantity,
                        source_script: grant.source_script.clone(),
                        command_index: grant.command_index,
                        verbose: grant.verbose,
                    })
            })
            .collect()
    }

    pub fn script_item_access_keys(&self) -> BTreeSet<RuntimeScriptItemAccessKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                let check_map_name = map_name.clone();
                let take_map_name = map_name.clone();
                let checks = module.script_item_checks.iter().map(move |access| {
                    RuntimeScriptItemAccessKey {
                        map_name: check_map_name.clone(),
                        command: access.command.clone(),
                        item_id: access.item_id.clone(),
                        source_script: access.source_script.clone(),
                        command_index: access.command_index,
                    }
                });
                let takes =
                    module
                        .script_item_takes
                        .iter()
                        .map(move |access| RuntimeScriptItemAccessKey {
                            map_name: take_map_name.clone(),
                            command: access.command.clone(),
                            item_id: access.item_id.clone(),
                            source_script: access.source_script.clone(),
                            command_index: access.command_index,
                        });
                checks.chain(takes)
            })
            .collect()
    }

    pub fn script_economy_command_keys(&self) -> BTreeSet<RuntimeScriptEconomyCommandKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module.script_economy_commands.iter().map(move |command| {
                    RuntimeScriptEconomyCommandKey {
                        map_name: map_name.clone(),
                        command: command.command.clone(),
                        account: command.account.clone(),
                        amount_tokens: command.amount_tokens.clone(),
                        source_script: command.source_script.clone(),
                        command_index: command.command_index,
                    }
                })
            })
            .collect()
    }

    pub fn script_flag_command_keys(&self) -> BTreeSet<RuntimeScriptFlagCommandKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module
                    .script_flag_commands
                    .iter()
                    .map(move |command| RuntimeScriptFlagCommandKey {
                        map_name: map_name.clone(),
                        command: command.command.clone(),
                        flag_id: command.flag_id.clone(),
                        source_script: command.source_script.clone(),
                        command_index: command.command_index,
                    })
            })
            .collect()
    }

    pub fn script_scene_command_keys(&self) -> BTreeSet<RuntimeScriptSceneCommandKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module.script_scene_commands.iter().map(move |command| {
                    RuntimeScriptSceneCommandKey {
                        map_name: map_name.clone(),
                        command: command.command.clone(),
                        map_id: command.map_id.clone(),
                        scene_id: command.scene_id.clone(),
                        source_script: command.source_script.clone(),
                        command_index: command.command_index,
                    }
                })
            })
            .collect()
    }

    pub fn script_block_change_keys(&self) -> BTreeSet<RuntimeScriptBlockChangeKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module
                    .script_block_changes
                    .iter()
                    .map(move |change| RuntimeScriptBlockChangeKey {
                        map_name: map_name.clone(),
                        x: change.x,
                        y: change.y,
                        block_id: change.block_id,
                        source_script: change.source_script.clone(),
                        command_index: change.command_index,
                    })
            })
            .collect()
    }

    pub fn script_audio_command_keys(&self) -> BTreeSet<RuntimeScriptAudioCommandKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module.script_audio_commands.iter().map(move |command| {
                    RuntimeScriptAudioCommandKey {
                        map_name: map_name.clone(),
                        command: command.command.clone(),
                        audio_id: command.audio_id.clone(),
                        fade_frames: command.fade_frames,
                        source_script: command.source_script.clone(),
                        command_index: command.command_index,
                    }
                })
            })
            .collect()
    }

    pub fn script_text_command_keys(&self) -> BTreeSet<RuntimeScriptTextCommandKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module
                    .script_text_commands
                    .iter()
                    .map(move |command| RuntimeScriptTextCommandKey {
                        map_name: map_name.clone(),
                        command: command.command.clone(),
                        text_label: command.text_label.clone(),
                        source_script: command.source_script.clone(),
                        command_index: command.command_index,
                    })
            })
            .collect()
    }

    pub fn warp_keys(&self) -> BTreeSet<RuntimeWarpKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module.events.warps.iter().map(move |warp| RuntimeWarpKey {
                    map_name: map_name.clone(),
                    warp_index: warp.index,
                })
            })
            .collect()
    }

    pub fn map_object_keys(&self) -> BTreeSet<RuntimeMapObjectKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module.objects.iter().filter_map(move |object| {
                    object
                        .object_identifier
                        .as_ref()
                        .map(|object_id| RuntimeMapObjectKey {
                            map_name: map_name.clone(),
                            object_id: object_id.clone(),
                        })
                })
            })
            .collect()
    }

    pub fn map_scene_keys(&self) -> BTreeSet<RuntimeMapSceneKey> {
        self.data
            .maps
            .iter()
            .flat_map(|(map_name, module)| {
                module
                    .scenes
                    .scenes
                    .iter()
                    .map(move |scene| RuntimeMapSceneKey {
                        map_name: map_name.clone(),
                        scene_id: scene.scene_id.clone(),
                    })
            })
            .collect()
    }

    pub fn map_metadata_keys(&self) -> BTreeSet<RuntimeMapMetadataKey> {
        self.data
            .maps
            .iter()
            .map(|(map_name, module)| {
                let metadata = module
                    .attributes
                    .map_constant
                    .as_ref()
                    .and_then(|constant| self.data.runtime_map_metadata.get(constant));
                RuntimeMapMetadataKey {
                    map_name: map_name.clone(),
                    map_id: module.id.clone(),
                    tileset_name: module.attributes.tileset_name.clone(),
                    border_block: module.attributes.border_block,
                    width: module.attributes.width,
                    height: module.attributes.height,
                    time_of_day: module.attributes.time_of_day.clone(),
                    phone_service: module.attributes.phone_service,
                    phone_flag: module.attributes.phone_flag,
                    environment: module.attributes.environment.clone(),
                    location: module.attributes.location.clone(),
                    music: module.attributes.music.clone(),
                    palette: module.attributes.palette.clone(),
                    fishing_group: module.attributes.fishing_group.clone(),
                    map_constant: module.attributes.map_constant.clone(),
                    map_group_constant: module.attributes.map_group_constant.clone(),
                    metadata_constant: metadata.map(|entry| entry.constant.clone()),
                    metadata_group_name: metadata.map(|entry| entry.group_name.clone()),
                    metadata_group_id: metadata.map(|entry| entry.group_id),
                    metadata_map_id: metadata.map(|entry| entry.map_id),
                    metadata_environment: metadata.map(|entry| entry.environment.clone()),
                }
            })
            .collect()
    }

    pub fn currency_constant_ids(&self) -> BTreeSet<String> {
        self.data.currency_constants.0.keys().cloned().collect()
    }

    pub fn capture_ball_rule_ids(&self) -> BTreeSet<String> {
        self.data.capture_rules.ball_rules.keys().cloned().collect()
    }

    pub fn guaranteed_capture_ball_ids(&self) -> BTreeSet<String> {
        self.data
            .capture_rules
            .guaranteed_capture_balls
            .iter()
            .cloned()
            .collect()
    }

    pub fn capture_status_bonus_ids(&self) -> BTreeSet<String> {
        self.data
            .capture_rules
            .status_bonus
            .keys()
            .cloned()
            .collect()
    }

    pub fn fast_ball_species_ids(&self) -> BTreeSet<String> {
        self.data
            .capture_rules
            .fast_ball_species
            .iter()
            .cloned()
            .collect()
    }

    pub fn heavy_ball_species_ids(&self) -> BTreeSet<String> {
        self.data
            .capture_rules
            .heavy_ball_modifiers
            .keys()
            .cloned()
            .collect()
    }

    pub fn move_priority_effect_ids(&self) -> BTreeSet<String> {
        self.data
            .move_priorities
            .effect_priorities
            .keys()
            .cloned()
            .collect()
    }

    pub fn move_priority_move_ids(&self) -> BTreeSet<String> {
        self.data
            .move_priorities
            .move_priorities
            .iter()
            .map(|priority| priority.r#move.clone())
            .collect()
    }

    pub fn capture_ball_rule_keys(&self) -> BTreeSet<RuntimeCaptureBallRuleKey> {
        self.data
            .capture_rules
            .ball_rules
            .iter()
            .map(|(ball_id, rule)| RuntimeCaptureBallRuleKey {
                ball_id: ball_id.clone(),
                multiplier_numerator: rule.multiplier_numerator,
                multiplier_denominator: rule.multiplier_denominator,
                battle_type: rule.battle_type.clone(),
                skip_hp_calc: rule.skip_hp_calc,
                use_heavy_ball_weight_modifier: rule.use_heavy_ball_weight_modifier,
                use_level_ball_multiplier: rule.use_level_ball_multiplier,
                require_same_species: rule.require_same_species,
                require_same_gender: rule.require_same_gender,
                require_fast_species: rule.require_fast_species,
            })
            .collect()
    }

    pub fn heavy_ball_modifier_keys(&self) -> BTreeSet<RuntimeHeavyBallModifierKey> {
        self.data
            .capture_rules
            .heavy_ball_modifiers
            .iter()
            .map(|(species_id, modifier)| RuntimeHeavyBallModifierKey {
                species_id: species_id.clone(),
                modifier: *modifier,
            })
            .collect()
    }

    pub fn capture_status_bonus_keys(&self) -> BTreeSet<RuntimeCaptureStatusBonusKey> {
        self.data
            .capture_rules
            .status_bonus
            .iter()
            .map(|(status, bonus)| RuntimeCaptureStatusBonusKey {
                status: status.clone(),
                bonus: *bonus,
            })
            .collect()
    }

    pub fn capture_wobble_probability_keys(&self) -> BTreeSet<RuntimeCaptureWobbleProbabilityKey> {
        self.data
            .capture_wobble_probabilities
            .iter()
            .map(|probability| RuntimeCaptureWobbleProbabilityKey {
                catch_rate: probability.catch_rate,
                chance: probability.chance,
            })
            .collect()
    }

    pub fn item_battle_use_keys(&self) -> BTreeSet<RuntimeItemBattleUseKey> {
        self.data
            .items
            .iter()
            .map(|(item_id, item)| RuntimeItemBattleUseKey {
                item_id: item_id.clone(),
                effect: item.effect.clone(),
                battle_menu: item.battle_menu.clone(),
                battle_usable: item.battle_usable,
                battle_stat_boost_stat: item.battle_stat_boost_stat.clone(),
                battle_stat_boost_stages: item.battle_stat_boost_stages,
                battle_escape_mode: item.battle_escape_mode.clone(),
                battle_focus_energy: item.battle_focus_energy,
                battle_stat_drop_guard: item.battle_stat_drop_guard,
            })
            .collect()
    }

    pub fn item_effect_plan_keys(&self) -> BTreeSet<RuntimeItemEffectPlanKey> {
        self.data
            .items
            .values()
            .flat_map(|item| {
                [
                    active_battle_item_effect_plan(item),
                    battle_pp_item_effect_plan(item),
                    party_wide_item_effect_plan(item),
                    party_special_item_effect_plan(item, &self.data.evolutions),
                ]
            })
            .flatten()
            .map(RuntimeItemEffectPlanKey::from_plan)
            .collect()
    }

    pub fn item_field_use_keys(&self) -> BTreeSet<RuntimeItemFieldUseKey> {
        self.data
            .items
            .iter()
            .map(|(item_id, item)| RuntimeItemFieldUseKey {
                item_id: item_id.clone(),
                effect: item.effect.clone(),
                field_menu: item.field_menu.clone(),
                field_usable: item.field_usable,
                consumable: item.consumable,
                repel_steps: item.repel_steps,
                escape_rope_mode: item.escape_rope_mode.clone(),
                tmhm_index: item.tmhm_index,
                tmhm_move: item.tmhm_move.clone(),
            })
            .collect()
    }

    pub fn move_battle_data_keys(&self) -> BTreeSet<RuntimeMoveBattleDataKey> {
        self.data
            .moves
            .iter()
            .map(|(move_id, move_data)| RuntimeMoveBattleDataKey {
                move_id: move_id.clone(),
                name: move_data.name.clone(),
                move_type: move_data.move_type.clone(),
                power: move_data.power,
                accuracy: move_data.accuracy,
                pp: move_data.pp,
                effect: move_data.effect.clone(),
                effect_chance: move_data.effect_chance,
                stat: move_data.stat.clone(),
                amount: move_data.amount,
            })
            .collect()
    }

    pub fn species_battle_data_keys(&self) -> BTreeSet<RuntimeSpeciesBattleDataKey> {
        self.data
            .pokemon
            .iter()
            .map(|(species_id, species)| RuntimeSpeciesBattleDataKey {
                species_id: species_id.clone(),
                int_id: species.int_id,
                base_hp: species.base_stats.hp,
                base_attack: species.base_stats.attack,
                base_defense: species.base_stats.defense,
                base_speed: species.base_stats.speed,
                base_special_attack: species.base_stats.special_attack,
                base_special_defense: species.base_stats.special_defense,
                type1: species.type1.clone(),
                type2: species.type2.clone(),
                catch_rate: species.catch_rate,
                base_exp: species.base_exp,
                item1: species.item1.clone(),
                item2: species.item2.clone(),
                gender_ratio: species.gender_ratio,
                step_cycles_to_hatch: species.step_cycles_to_hatch,
                growth_rate: species.growth_rate.clone(),
                egg_group1: species.egg_group1.clone(),
                egg_group2: species.egg_group2.clone(),
                tmhm_learnset: species.tmhm_learnset.clone(),
                ability: species.ability.clone(),
                weight: species.weight,
            })
            .collect()
    }

    pub fn species_learnset_keys(&self) -> BTreeSet<RuntimeSpeciesLearnsetKey> {
        self.data
            .learnsets
            .iter()
            .flat_map(|(species_id, entries)| {
                entries.iter().map(move |entry| RuntimeSpeciesLearnsetKey {
                    species_id: species_id.clone(),
                    level: entry.0,
                    move_id: entry.1.clone(),
                })
            })
            .collect()
    }

    pub fn species_evolution_keys(&self) -> BTreeSet<RuntimeSpeciesEvolutionKey> {
        self.data
            .evolutions
            .0
            .iter()
            .flat_map(|(source_species_id, entries)| {
                entries.iter().map(move |entry| RuntimeSpeciesEvolutionKey {
                    source_species_id: source_species_id.clone(),
                    method: entry.method.clone(),
                    target_species_id: entry.species.clone(),
                    level: entry.level,
                    item: entry.item.clone(),
                    held_item: entry.held_item.clone(),
                    happiness: entry.happiness.clone(),
                    stat_ratio: entry.stat_ratio.clone(),
                })
            })
            .collect()
    }

    pub fn trainer_battle_data_keys(&self) -> BTreeSet<RuntimeTrainerBattleDataKey> {
        self.data
            .trainers
            .trainers
            .values()
            .map(|trainer| RuntimeTrainerBattleDataKey {
                trainer_id: trainer.trainer_id.clone(),
                name: trainer.name.clone(),
                trainer_class: trainer.trainer_class.clone(),
                win_quote: trainer.win_quote.clone(),
                lose_quote: trainer.lose_quote.clone(),
                items: trainer.items.clone(),
                base_reward: trainer.base_reward,
                ai_move_flags: trainer.ai_move_flags,
                ai_item_switch_flags: trainer.ai_item_switch_flags,
                encounter_music: trainer.encounter_music.clone(),
                ai_layers: trainer.ai_layers.clone(),
            })
            .collect()
    }

    pub fn trainer_party_pokemon_keys(&self) -> BTreeSet<RuntimeTrainerPartyPokemonKey> {
        self.data
            .trainers
            .trainers
            .values()
            .flat_map(|trainer| {
                trainer
                    .party
                    .iter()
                    .enumerate()
                    .map(
                        move |(party_index, pokemon)| RuntimeTrainerPartyPokemonKey {
                            trainer_id: trainer.trainer_id.clone(),
                            party_index,
                            species: pokemon.species.clone(),
                            level: pokemon.level,
                            item: pokemon.item.clone(),
                            move_names: pokemon
                                .moves
                                .iter()
                                .map(|move_data| move_data.name.clone())
                                .collect(),
                            move_pp: pokemon
                                .moves
                                .iter()
                                .map(|move_data| move_data.current_pp)
                                .collect(),
                            move_pp_ups: pokemon
                                .moves
                                .iter()
                                .map(|move_data| move_data.pp_ups)
                                .collect(),
                            dv_attack: pokemon.dvs.attack,
                            dv_defense: pokemon.dvs.defense,
                            dv_speed: pokemon.dvs.speed,
                            dv_special: pokemon.dvs.special,
                            dv_hp: pokemon.dvs.hp,
                        },
                    )
            })
            .collect()
    }

    pub fn move_priority_effect_keys(&self) -> BTreeSet<RuntimeMovePriorityEffectKey> {
        self.data
            .move_priorities
            .effect_priorities
            .iter()
            .map(|(effect_id, priority)| RuntimeMovePriorityEffectKey {
                effect_id: effect_id.clone(),
                priority: *priority,
            })
            .collect()
    }

    pub fn move_priority_move_keys(&self) -> BTreeSet<RuntimeMovePriorityMoveKey> {
        self.data
            .move_priorities
            .move_priorities
            .iter()
            .map(|priority| RuntimeMovePriorityMoveKey {
                move_id: priority.r#move.clone(),
                priority: priority.priority,
            })
            .collect()
    }

    pub fn battle_stat_multiplier_keys(&self) -> BTreeSet<RuntimeBattleStatMultiplierKey> {
        let mut keys = BTreeSet::new();
        keys.extend(
            self.data
                .battle_stat_multipliers
                .stat
                .iter()
                .enumerate()
                .map(|(index, multiplier)| RuntimeBattleStatMultiplierKey {
                    table: "stat".to_string(),
                    stage: index as i8 - 6,
                    numerator: multiplier.numerator,
                    denominator: multiplier.denominator,
                }),
        );
        keys.extend(
            self.data
                .battle_stat_multipliers
                .accuracy
                .iter()
                .enumerate()
                .map(|(index, multiplier)| RuntimeBattleStatMultiplierKey {
                    table: "accuracy".to_string(),
                    stage: index as i8 - 6,
                    numerator: multiplier.numerator,
                    denominator: multiplier.denominator,
                }),
        );
        keys
    }

    pub fn battle_reward_rule_keys(&self) -> BTreeSet<RuntimeBattleRewardRuleKey> {
        [
            RuntimeBattleRewardRuleKey {
                field: "max_level".to_string(),
                value: i32::from(self.data.battle_reward_rules.max_level),
            },
            RuntimeBattleRewardRuleKey {
                field: "wild_exp_divisor".to_string(),
                value: self.data.battle_reward_rules.wild_exp_divisor,
            },
            RuntimeBattleRewardRuleKey {
                field: "trainer_exp_numerator".to_string(),
                value: self.data.battle_reward_rules.trainer_exp_numerator,
            },
            RuntimeBattleRewardRuleKey {
                field: "trainer_exp_denominator".to_string(),
                value: self.data.battle_reward_rules.trainer_exp_denominator,
            },
        ]
        .into_iter()
        .collect()
    }

    pub fn battle_escape_rule_keys(&self) -> BTreeSet<RuntimeBattleEscapeRuleKey> {
        [
            RuntimeBattleEscapeRuleKey {
                field: "player_speed_multiplier".to_string(),
                value: self.data.battle_escape_rules.player_speed_multiplier,
            },
            RuntimeBattleEscapeRuleKey {
                field: "enemy_speed_divisor".to_string(),
                value: self.data.battle_escape_rules.enemy_speed_divisor,
            },
            RuntimeBattleEscapeRuleKey {
                field: "failed_attempt_bonus".to_string(),
                value: self.data.battle_escape_rules.failed_attempt_bonus,
            },
            RuntimeBattleEscapeRuleKey {
                field: "rng_roll_values".to_string(),
                value: self.data.battle_escape_rules.rng_roll_values,
            },
        ]
        .into_iter()
        .collect()
    }

    pub fn physical_type_ids(&self) -> BTreeSet<String> {
        self.data.type_categories.physical.iter().cloned().collect()
    }

    pub fn special_type_ids(&self) -> BTreeSet<String> {
        self.data.type_categories.special.iter().cloned().collect()
    }

    pub fn weather_ids(&self) -> BTreeSet<String> {
        let mut ids: BTreeSet<String> = self
            .data
            .weather_modifiers
            .type_modifiers
            .keys()
            .cloned()
            .collect();
        ids.extend(
            self.data
                .weather_modifiers
                .move_effect_modifiers
                .keys()
                .cloned(),
        );
        ids
    }

    pub fn type_effectiveness_keys(&self) -> BTreeSet<RuntimeTypeEffectivenessKey> {
        self.data
            .type_effectiveness
            .matchups
            .iter()
            .map(|entry| RuntimeTypeEffectivenessKey {
                attacking_type: entry.attacker.clone(),
                defending_type: entry.defender.clone(),
            })
            .collect()
    }

    pub fn foresight_type_effectiveness_keys(&self) -> BTreeSet<RuntimeTypeEffectivenessKey> {
        self.data
            .type_effectiveness
            .foresight_matchups
            .iter()
            .map(|entry| RuntimeTypeEffectivenessKey {
                attacking_type: entry.attacker.clone(),
                defending_type: entry.defender.clone(),
            })
            .collect()
    }

    pub fn weather_type_modifier_keys(&self) -> BTreeSet<RuntimeWeatherTypeModifierKey> {
        self.data
            .weather_modifiers
            .type_modifiers
            .iter()
            .flat_map(|(weather, modifiers)| {
                modifiers
                    .keys()
                    .map(move |type_id| RuntimeWeatherTypeModifierKey {
                        weather: weather.clone(),
                        type_id: type_id.clone(),
                    })
            })
            .collect()
    }

    pub fn weather_move_effect_modifier_keys(
        &self,
    ) -> BTreeSet<RuntimeWeatherMoveEffectModifierKey> {
        self.data
            .weather_modifiers
            .move_effect_modifiers
            .iter()
            .flat_map(|(weather, modifiers)| {
                modifiers
                    .keys()
                    .map(move |effect_id| RuntimeWeatherMoveEffectModifierKey {
                        weather: weather.clone(),
                        effect_id: effect_id.clone(),
                    })
            })
            .collect()
    }

    pub fn music_ids(&self) -> BTreeSet<String> {
        self.audio.music_ids()
    }

    pub fn sound_effect_ids(&self) -> BTreeSet<String> {
        self.audio.sound_effect_ids()
    }

    pub fn cry_ids(&self) -> BTreeSet<String> {
        self.audio.cry_ids()
    }

    pub fn pokemon_cry_keys(&self) -> BTreeSet<RuntimePokemonCryKey> {
        self.data
            .pokemon_cries
            .iter()
            .map(|(species_id, cry)| RuntimePokemonCryKey {
                species_id: species_id.clone(),
                cry_id: cry.cry.clone(),
                pitch: cry.pitch,
                length: cry.length,
            })
            .collect()
    }

    pub fn audio_asset_keys(&self) -> BTreeSet<RuntimeAudioAssetKey> {
        self.audio.audio_asset_keys()
    }

    pub fn has_special_routine(&self, routine: &str) -> bool {
        self.data.special_routines.contains_key(routine)
    }

    pub fn has_item(&self, item_id: &str) -> bool {
        self.data.items.contains_key(item_id)
    }

    pub fn has_move(&self, move_id: &str) -> bool {
        self.data.moves.contains_key(move_id)
    }

    pub fn has_species(&self, species_id: &str) -> bool {
        self.data.pokemon.contains_key(species_id)
    }

    pub fn has_map(&self, map_name: &str) -> bool {
        self.data.maps.contains_key(map_name)
    }

    pub fn has_trainer(&self, trainer_id: &str) -> bool {
        self.data.trainers.trainers.contains_key(trainer_id)
    }

    pub fn has_text(&self, text_label: &str) -> bool {
        self.data.saved_text_exists(text_label)
    }

    pub fn has_menu(&self, menu: &str) -> bool {
        self.data.saved_menu_exists(menu)
    }

    pub fn has_phone_contact(&self, contact_id: &str) -> bool {
        self.data.phone_contacts.0.contains_key(contact_id)
    }

    pub fn has_special_phone_call(&self, call_id: &str) -> bool {
        self.data.saved_special_phone_call_exists(call_id)
    }

    pub fn has_npc_trade(&self, trade_id: &str) -> bool {
        self.data.saved_npc_trade_exists(trade_id)
    }

    pub fn has_sprite(&self, sprite_id: &str) -> bool {
        self.data.saved_sprite_exists(sprite_id)
    }

    pub fn has_map_constant(&self, map_constant: &str) -> bool {
        self.data.saved_map_constant(map_constant).is_some()
    }

    pub fn has_event_flag(&self, flag: &str) -> bool {
        self.data.saved_event_flag_exists(flag)
    }

    pub fn has_engine_flag(&self, flag: &str) -> bool {
        self.data.saved_engine_flag_exists(flag)
    }

    pub fn has_spawn_identifier(&self, spawn_identifier: u16) -> bool {
        self.data.saved_spawn_identifier(spawn_identifier).is_some()
    }

    pub fn has_tileset(&self, tileset_id: &str) -> bool {
        self.data.saved_tileset_exists(tileset_id)
    }

    pub fn has_tileset_row(&self, key: &RuntimeTilesetKey) -> bool {
        self.tileset_keys().contains(key)
    }

    pub fn has_pc_string(&self, key: &RuntimePcStringKey) -> bool {
        self.pc_string_keys().contains(key)
    }

    pub fn has_menu_icon(&self, key: &RuntimeMenuIconKey) -> bool {
        self.menu_icon_keys().contains(key)
    }

    pub fn has_pokedex_entry(&self, key: &RuntimePokedexEntryKey) -> bool {
        self.pokedex_entry_keys().contains(key)
    }

    pub fn has_landmark(&self, landmark_id: &str) -> bool {
        self.data
            .pokegear_landmarks
            .landmarks
            .iter()
            .any(|landmark| landmark.constant == landmark_id)
    }

    pub fn has_pokegear_landmark(&self, key: &RuntimePokegearLandmarkKey) -> bool {
        self.pokegear_landmark_keys().contains(key)
    }

    pub fn has_pokegear_map_landmark(&self, key: &RuntimePokegearMapLandmarkKey) -> bool {
        self.pokegear_map_landmark_keys().contains(key)
    }

    pub fn has_fishing_rod(&self, rod: &str) -> bool {
        self.data.saved_fishing_rod_exists(rod)
    }

    pub fn has_map_group(&self, group_id: &str) -> bool {
        self.data
            .runtime_map_metadata
            .values()
            .any(|metadata| metadata.group_name == group_id)
    }

    pub fn has_encounter_group(&self, group_id: &str) -> bool {
        self.data.fishing.groups.contains_key(group_id)
    }

    pub fn has_mart(&self, mart_id: &str) -> bool {
        self.data.marts.0.contains_key(mart_id)
    }

    pub fn has_mart_row(&self, key: &RuntimeMartKey) -> bool {
        self.mart_keys().contains(key)
    }

    pub fn has_fruit_tree(&self, fruit_tree_id: &str) -> bool {
        self.data.fruit_trees.0.contains_key(fruit_tree_id)
    }

    pub fn has_fruit_tree_row(&self, key: &RuntimeFruitTreeKey) -> bool {
        self.fruit_tree_keys().contains(key)
    }

    pub fn has_field_move_rule(&self, rule_id: &str) -> bool {
        self.field_move_rule_ids().contains(rule_id)
    }

    pub fn has_field_move_rule_row(&self, key: &RuntimeFieldMoveRuleKey) -> bool {
        self.field_move_rule_keys().contains(key)
    }

    pub fn has_fly_destination(&self, flypoint_flag: &str) -> bool {
        self.data.fly_destinations.contains_key(flypoint_flag)
    }

    pub fn has_fly_destination_row(&self, key: &RuntimeFlyDestinationKey) -> bool {
        self.fly_destination_keys().contains(key)
    }

    pub fn has_field_move_move(&self, move_id: &str) -> bool {
        self.field_move_move_ids().contains(move_id)
    }

    pub fn has_field_move_item(&self, item_id: &str) -> bool {
        self.field_move_item_ids().contains(item_id)
    }

    pub fn has_field_box_item(&self, item_id: &str) -> bool {
        self.data.field_box_items.contains_key(item_id)
    }

    pub fn has_flee_mon_bucket(&self, bucket_id: &str) -> bool {
        self.data.flee_mons.buckets.contains_key(bucket_id)
    }

    pub fn has_buena_password_category(&self, category_id: &str) -> bool {
        self.data
            .buena_password_categories
            .categories
            .contains_key(category_id)
    }

    pub fn has_roaming_species(&self, species_id: &str) -> bool {
        self.data
            .roaming_pokemon
            .init_writes
            .iter()
            .any(|write| write.species == species_id)
    }

    pub fn has_buena_prize_item(&self, item_id: &str) -> bool {
        self.data.buena_prizes.contains_key(item_id)
    }

    pub fn has_kurt_apricorn_item(&self, item_id: &str) -> bool {
        self.data.kurt_apricorn_recipes.contains_key(item_id)
    }

    pub fn has_dratini_move_set(&self, answer: u8) -> bool {
        self.data.dratini_move_sets.contains_key(&answer)
    }

    pub fn has_special_feature(&self, feature_id: &str) -> bool {
        self.special_feature_ids().contains(feature_id)
    }

    pub fn has_oak_rating_text(&self, text_id: &str) -> bool {
        self.data
            .oak_ratings
            .iter()
            .any(|rating| rating.text_label == text_id)
    }

    pub fn has_odd_egg_species(&self, species_id: &str) -> bool {
        self.data
            .odd_egg_definitions
            .iter()
            .any(|definition| definition.species == species_id)
    }

    pub fn has_magikarp_length_threshold(&self, threshold: u16) -> bool {
        self.data
            .magikarp_lengths
            .iter()
            .any(|entry| entry.threshold == threshold)
    }

    pub fn has_happiness_change(&self, change_id: u8) -> bool {
        self.data
            .happiness_data
            .as_ref()
            .is_some_and(|data| data.changes.contains_key(&change_id))
    }

    pub fn has_happiness_service(&self, service_id: &str) -> bool {
        self.data
            .happiness_data
            .as_ref()
            .is_some_and(|data| data.services.contains_key(service_id))
    }

    pub fn has_pokemon_status(&self, status: &str) -> bool {
        self.data.saved_pokemon_status_exists(status)
    }

    pub fn has_fishing_daily_flag_bit(&self, bit: u32) -> bool {
        self.data.saved_fishing_daily_flag_bit_exists(bit)
    }

    pub fn has_fishing_swarm_flag(&self, swarm_flag: u8) -> bool {
        self.data.saved_fishing_swarm_flag_exists(swarm_flag)
    }

    pub fn has_pending_special_battle_type(&self, battle_type: &str) -> bool {
        self.data
            .saved_pending_special_battle_type_exists(battle_type)
    }

    pub fn has_wild_encounter_origin(&self, key: &RuntimeWildEncounterOriginKey) -> bool {
        self.data
            .saved_wild_encounter_exists(&key.map_name, &key.species, key.level)
    }

    pub fn has_script_label(&self, script_label: &str) -> bool {
        self.data.compiled_script_body(script_label).is_some()
    }

    pub fn script_owner_map(&self, script_label: &str) -> Result<String> {
        self.data
            .maps
            .iter()
            .find(|(_, module)| module.scripts.contains_key(script_label))
            .map(|(map_name, _)| map_name.clone())
            .with_context(|| format!("compiled game pack missing script label {script_label}"))
    }

    pub fn compiled_script_command_name(
        &self,
        script_label: &str,
        command_index: usize,
    ) -> Result<String> {
        let body = self
            .data
            .compiled_script_body(script_label)
            .with_context(|| format!("compiled game pack missing script label {script_label}"))?;
        let command = body
            .as_array()
            .and_then(|commands| commands.get(command_index))
            .with_context(|| {
                format!("compiled script {script_label} missing command {command_index}")
            })?;
        command
            .get("command")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
            .with_context(|| {
                format!(
                    "compiled script {script_label} command {command_index} missing command name"
                )
            })
    }

    pub fn compiled_script_commands(&self, script_label: &str) -> Result<Vec<serde_json::Value>> {
        let body = self
            .data
            .compiled_script_body(script_label)
            .with_context(|| format!("compiled game pack missing script label {script_label}"))?;
        body.as_array()
            .cloned()
            .with_context(|| format!("compiled script {script_label} is not a command array"))
    }

    pub fn has_script_command(&self, key: &RuntimeScriptCommandKey) -> bool {
        self.data
            .compiled_script_body(&key.script_label)
            .and_then(|body| body.as_array())
            .is_some_and(|commands| key.command_index < commands.len())
    }

    pub fn has_script_command_payload(&self, key: &RuntimeScriptCommandPayloadKey) -> bool {
        self.data
            .validate_saved_script_command_payload_reference(
                "runtime.script_command_payload",
                &key.script_label,
                key.command_index,
                &key.command,
                &key.args,
            )
            .is_ok()
    }

    pub fn has_script_return(&self, key: &RuntimeScriptReturnKey) -> bool {
        self.data
            .validate_saved_script_return_reference(
                "runtime.script_return",
                &key.script_label,
                key.next_command_index,
            )
            .is_ok()
    }

    pub fn has_script_vertical_menu(&self, key: &RuntimeScriptVerticalMenuKey) -> bool {
        self.script_vertical_menu_keys().contains(key)
    }

    pub fn has_script_text_body(&self, key: &RuntimeScriptTextBodyKey) -> bool {
        self.script_text_body_keys().contains(key)
    }

    pub fn has_script_menu_definition(&self, key: &RuntimeScriptMenuDefinitionKey) -> bool {
        self.script_menu_definition_keys().contains(key)
    }

    pub fn has_script_elevator(&self, key: &RuntimeScriptElevatorKey) -> bool {
        self.script_elevator_keys().contains(key)
    }

    pub fn has_script_elevator_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.data.maps.get(map_name).is_some_and(|module| {
            module.script_elevators.values().any(|elevator| {
                elevator.source_script == source_script
                    && elevator.elevator_command_index == command_index
            })
        })
    }

    pub fn has_gift_pokemon(&self, key: &RuntimeGiftPokemonKey) -> bool {
        self.gift_pokemon_keys().contains(key)
    }

    pub fn has_gift_pokemon_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.gift_pokemon_keys().into_iter().any(|key| {
            key.map_name == map_name
                && key.source_script == source_script
                && key.command_index == command_index
        })
    }

    pub fn has_script_phone_prompt_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.data.maps.get(map_name).is_some_and(|module| {
            module.script_phone_commands.iter().any(|command| {
                command.command == "askforphonenumber"
                    && command.source_script == source_script
                    && command.command_index == command_index
            })
        })
    }

    pub fn has_scripted_wild_battle_start_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.data.maps.get(map_name).is_some_and(|module| {
            module.scripted_wild_battles.iter().any(|battle| {
                battle.source_script == source_script
                    && battle.startbattle_command_index == command_index
            })
        })
    }

    pub fn has_scripted_trainer_battle_start_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.scripted_trainer_battle_keys().into_iter().any(|key| {
            key.map_name == map_name
                && key.source_script == source_script
                && key.startbattle_command_index == command_index
        })
    }

    pub fn has_script_object_command(&self, key: &RuntimeScriptObjectCommandKey) -> bool {
        self.script_object_command_keys().contains(key)
    }

    pub fn has_script_movement(&self, key: &RuntimeScriptMovementKey) -> bool {
        self.script_movement_keys().contains(key)
    }

    pub fn has_map_script_section_command(&self, key: &RuntimeMapScriptSectionCommandKey) -> bool {
        self.map_script_section_command_keys().contains(key)
    }

    pub fn has_map_event_section_command(&self, key: &RuntimeMapEventSectionCommandKey) -> bool {
        self.map_event_section_command_keys().contains(key)
    }

    pub fn has_script_map_command(&self, key: &RuntimeScriptMapCommandKey) -> bool {
        self.data.maps.contains_key(&key.map_name)
            && self
                .data
                .script_map_command(&key.map_name, &key.source_script, key.command_index)
                .is_ok_and(|command| {
                    command.command == key.command
                        && command.target_map == key.target_map
                        && command.x == key.x
                        && command.y == key.y
                        && command.facing == key.facing
                        && command.map_setup == key.map_setup
                })
    }

    pub fn has_script_variable_command(&self, key: &RuntimeScriptVariableCommandKey) -> bool {
        self.data.maps.contains_key(&key.map_name)
            && self
                .data
                .script_variable_command(&key.map_name, &key.source_script, key.command_index)
                .is_ok_and(|command| {
                    command.command == key.command
                        && command.target == key.target
                        && command.value_tokens == key.value_tokens
                })
    }

    pub fn has_script_control_command(&self, key: &RuntimeScriptControlCommandKey) -> bool {
        self.data.maps.contains_key(&key.map_name)
            && self
                .data
                .script_control_command(&key.map_name, &key.source_script, key.command_index)
                .is_ok_and(|command| {
                    command.command == key.command
                        && command.compare_value == key.compare_value
                        && command.target_label == key.target_label
                        && command.resolved_target_script == key.resolved_target_script
                })
    }

    pub fn has_script_swarm_command(&self, key: &RuntimeScriptSwarmCommandKey) -> bool {
        self.data.maps.contains_key(&key.map_name)
            && self
                .data
                .script_swarm_command(&key.map_name, &key.source_script, key.command_index)
                .is_ok_and(|command| {
                    command.command == key.command
                        && command.swarm_token == key.swarm_token
                        && command.map_id == key.map_id
                })
    }

    pub fn has_script_field_pickup(&self, key: &RuntimeScriptFieldPickupKey) -> bool {
        self.script_field_pickup_keys().contains(key)
    }

    pub fn has_script_shop_command(&self, key: &RuntimeScriptShopCommandKey) -> bool {
        self.script_shop_command_keys().contains(key)
    }

    pub fn has_script_phone_command(&self, key: &RuntimeScriptPhoneCommandKey) -> bool {
        self.data.maps.contains_key(&key.map_name)
            && self
                .data
                .script_phone_command(&key.map_name, &key.source_script, key.command_index)
                .is_ok_and(|command| {
                    command.command == key.command && command.contact_id == key.contact_id
                })
    }

    pub fn has_script_runtime_command(&self, key: &RuntimeScriptRuntimeCommandKey) -> bool {
        self.data.maps.contains_key(&key.map_name)
            && self
                .data
                .script_runtime_command(&key.map_name, &key.source_script, key.command_index)
                .is_ok_and(|command| command.command == key.command && command.args == key.args)
    }

    pub fn has_script_item_grant(&self, key: &RuntimeScriptItemGrantKey) -> bool {
        self.script_item_grant_keys().contains(key)
    }

    pub fn has_script_item_access(&self, key: &RuntimeScriptItemAccessKey) -> bool {
        self.script_item_access_keys().contains(key)
    }

    pub fn has_script_economy_command(&self, key: &RuntimeScriptEconomyCommandKey) -> bool {
        self.data.maps.contains_key(&key.map_name)
            && self
                .data
                .script_economy_command(&key.map_name, &key.source_script, key.command_index)
                .is_ok_and(|command| {
                    command.command == key.command
                        && command.account == key.account
                        && command.amount_tokens == key.amount_tokens
                })
    }

    pub fn has_script_flag_command(&self, key: &RuntimeScriptFlagCommandKey) -> bool {
        self.data.maps.contains_key(&key.map_name)
            && self
                .data
                .script_flag_command(&key.map_name, &key.source_script, key.command_index)
                .is_ok_and(|command| {
                    command.command == key.command && command.flag_id == key.flag_id
                })
    }

    pub fn has_script_scene_command(&self, key: &RuntimeScriptSceneCommandKey) -> bool {
        self.data.maps.contains_key(&key.map_name)
            && self
                .data
                .script_scene_command(&key.map_name, &key.source_script, key.command_index)
                .is_ok_and(|command| {
                    command.command == key.command
                        && command.map_id == key.map_id
                        && command.scene_id == key.scene_id
                })
    }

    pub fn has_script_block_change(&self, key: &RuntimeScriptBlockChangeKey) -> bool {
        self.script_block_change_keys().contains(key)
    }

    pub fn has_script_audio_command(&self, key: &RuntimeScriptAudioCommandKey) -> bool {
        self.data.maps.contains_key(&key.map_name)
            && self
                .data
                .script_audio_command(&key.map_name, &key.source_script, key.command_index)
                .is_ok_and(|command| {
                    command.command == key.command
                        && command.audio_id == key.audio_id
                        && command.fade_frames == key.fade_frames
                })
    }

    pub fn has_script_text_command(&self, key: &RuntimeScriptTextCommandKey) -> bool {
        self.data.maps.contains_key(&key.map_name)
            && self
                .data
                .script_text_command(&key.map_name, &key.source_script, key.command_index)
                .is_ok_and(|command| {
                    command.command == key.command && command.text_label == key.text_label
                })
    }

    pub fn has_script_object_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.data
            .script_object_command(map_name, source_script, command_index)
            .is_ok()
    }

    pub fn has_script_movement_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.data
            .script_object_command(map_name, source_script, command_index)
            .is_ok_and(|command| command.movement.is_some())
    }

    pub fn has_script_map_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.data
            .script_map_command(map_name, source_script, command_index)
            .is_ok()
    }

    pub fn has_script_variable_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.data
            .script_variable_command(map_name, source_script, command_index)
            .is_ok()
    }

    pub fn has_script_control_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.data
            .script_control_command(map_name, source_script, command_index)
            .is_ok()
    }

    pub fn has_script_swarm_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.data
            .script_swarm_command(map_name, source_script, command_index)
            .is_ok()
    }

    pub fn has_script_phone_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.data
            .script_phone_command(map_name, source_script, command_index)
            .is_ok()
    }

    pub fn has_script_field_pickup_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.data.maps.get(map_name).is_some_and(|module| {
            module.script_field_pickups.iter().any(|pickup| {
                pickup.source_script == source_script && pickup.command_index == command_index
            })
        })
    }

    pub fn has_script_shop_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.data.maps.get(map_name).is_some_and(|module| {
            module.script_shop_commands.iter().any(|command| {
                command.source_script == source_script && command.command_index == command_index
            })
        })
    }

    pub fn has_script_runtime_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.data
            .script_runtime_command(map_name, source_script, command_index)
            .is_ok()
    }

    pub fn has_script_item_grant_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        // Variable-quantity grants share the canonical receipt/audio boundary.
        self.data.script_runtime_command(map_name, source_script, command_index)
            .is_ok_and(|command| command.command == "verbosegiveitemvar")
            || self.data.maps.get(map_name).is_some_and(|module| {
            module.script_item_grants.iter().any(|grant| {
                grant.source_script == source_script && grant.command_index == command_index
            })
        })
    }

    pub fn has_script_item_check_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.data.maps.get(map_name).is_some_and(|module| {
            module.script_item_checks.iter().any(|access| {
                access.command == "checkitem"
                    && access.source_script == source_script
                    && access.command_index == command_index
            })
        })
    }

    pub fn has_script_item_take_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.data.maps.get(map_name).is_some_and(|module| {
            module.script_item_takes.iter().any(|access| {
                access.command == "takeitem"
                    && access.source_script == source_script
                    && access.command_index == command_index
            })
        })
    }

    pub fn has_script_economy_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.data
            .script_economy_command(map_name, source_script, command_index)
            .is_ok()
    }

    pub fn has_script_flag_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.data
            .script_flag_command(map_name, source_script, command_index)
            .is_ok()
    }

    pub fn has_script_scene_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.data
            .script_scene_command(map_name, source_script, command_index)
            .is_ok()
    }

    pub fn has_script_block_change_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.data.maps.get(map_name).is_some_and(|module| {
            module.script_block_changes.iter().any(|change| {
                change.source_script == source_script && change.command_index == command_index
            })
        })
    }

    pub fn has_script_audio_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.data
            .script_audio_command(map_name, source_script, command_index)
            .is_ok()
    }

    pub fn has_script_text_command_at(
        &self,
        map_name: &str,
        source_script: &str,
        command_index: usize,
    ) -> bool {
        self.data
            .script_text_command(map_name, source_script, command_index)
            .is_ok()
    }

    pub fn has_warp(&self, key: &RuntimeWarpKey) -> bool {
        self.data.saved_warp_exists(&key.map_name, key.warp_index)
    }

    pub fn has_map_object(&self, key: &RuntimeMapObjectKey) -> bool {
        self.data.map_declares_object(&key.map_name, &key.object_id)
    }

    pub fn has_map_scene(&self, key: &RuntimeMapSceneKey) -> bool {
        self.data
            .saved_scene_index(&key.map_name, &key.scene_id)
            .is_some()
    }

    pub fn has_map_metadata(&self, key: &RuntimeMapMetadataKey) -> bool {
        self.map_metadata_keys().contains(key)
    }

    pub fn has_currency_constant(&self, id: &str) -> bool {
        self.data.currency_constants.0.contains_key(id)
    }

    pub fn has_capture_ball_rule(&self, id: &str) -> bool {
        self.data.capture_rules.ball_rules.contains_key(id)
    }

    pub fn has_guaranteed_capture_ball(&self, id: &str) -> bool {
        self.data
            .capture_rules
            .guaranteed_capture_balls
            .contains(id)
    }

    pub fn has_capture_status_bonus(&self, status: &str) -> bool {
        self.data.capture_rules.status_bonus.contains_key(status)
    }

    pub fn has_fast_ball_species(&self, species_id: &str) -> bool {
        self.data
            .capture_rules
            .fast_ball_species
            .contains(species_id)
    }

    pub fn has_heavy_ball_species(&self, species_id: &str) -> bool {
        self.data
            .capture_rules
            .heavy_ball_modifiers
            .contains_key(species_id)
    }

    pub fn has_move_priority_effect(&self, effect_id: &str) -> bool {
        self.data
            .move_priorities
            .effect_priorities
            .contains_key(effect_id)
    }

    pub fn has_move_priority_move(&self, move_id: &str) -> bool {
        self.data
            .move_priorities
            .move_priorities
            .iter()
            .any(|priority| priority.r#move == move_id)
    }

    pub fn has_capture_ball_rule_key(&self, key: &RuntimeCaptureBallRuleKey) -> bool {
        self.data
            .capture_rules
            .ball_rules
            .get(&key.ball_id)
            .is_some_and(|rule| {
                rule.multiplier_numerator == key.multiplier_numerator
                    && rule.multiplier_denominator == key.multiplier_denominator
                    && rule.battle_type == key.battle_type
                    && rule.skip_hp_calc == key.skip_hp_calc
                    && rule.use_heavy_ball_weight_modifier == key.use_heavy_ball_weight_modifier
                    && rule.use_level_ball_multiplier == key.use_level_ball_multiplier
                    && rule.require_same_species == key.require_same_species
                    && rule.require_same_gender == key.require_same_gender
                    && rule.require_fast_species == key.require_fast_species
            })
    }

    pub fn has_heavy_ball_modifier(&self, key: &RuntimeHeavyBallModifierKey) -> bool {
        self.data
            .capture_rules
            .heavy_ball_modifiers
            .get(&key.species_id)
            .is_some_and(|modifier| *modifier == key.modifier)
    }

    pub fn has_capture_status_bonus_key(&self, key: &RuntimeCaptureStatusBonusKey) -> bool {
        self.data
            .capture_rules
            .status_bonus
            .get(&key.status)
            .is_some_and(|bonus| *bonus == key.bonus)
    }

    pub fn has_capture_wobble_probability(&self, key: &RuntimeCaptureWobbleProbabilityKey) -> bool {
        self.data
            .capture_wobble_probabilities
            .iter()
            .any(|probability| {
                probability.catch_rate == key.catch_rate && probability.chance == key.chance
            })
    }

    pub fn has_item_battle_use(&self, key: &RuntimeItemBattleUseKey) -> bool {
        self.data.items.get(&key.item_id).is_some_and(|item| {
            item.effect == key.effect
                && item.battle_menu == key.battle_menu
                && item.battle_usable == key.battle_usable
                && item.battle_stat_boost_stat == key.battle_stat_boost_stat
                && item.battle_stat_boost_stages == key.battle_stat_boost_stages
                && item.battle_escape_mode == key.battle_escape_mode
                && item.battle_focus_energy == key.battle_focus_energy
                && item.battle_stat_drop_guard == key.battle_stat_drop_guard
        })
    }

    pub fn has_item_effect_plan(&self, key: &RuntimeItemEffectPlanKey) -> bool {
        self.item_effect_plan_keys().contains(key)
    }

    pub fn has_item_field_use(&self, key: &RuntimeItemFieldUseKey) -> bool {
        self.data.items.get(&key.item_id).is_some_and(|item| {
            item.effect == key.effect
                && item.field_menu == key.field_menu
                && item.field_usable == key.field_usable
                && item.consumable == key.consumable
                && item.repel_steps == key.repel_steps
                && item.escape_rope_mode == key.escape_rope_mode
                && item.tmhm_index == key.tmhm_index
                && item.tmhm_move == key.tmhm_move
        })
    }

    pub fn has_move_battle_data(&self, key: &RuntimeMoveBattleDataKey) -> bool {
        self.data.moves.get(&key.move_id).is_some_and(|move_data| {
            move_data.name == key.name
                && move_data.move_type == key.move_type
                && move_data.power == key.power
                && move_data.accuracy == key.accuracy
                && move_data.pp == key.pp
                && move_data.effect == key.effect
                && move_data.effect_chance == key.effect_chance
                && move_data.stat == key.stat
                && move_data.amount == key.amount
        })
    }

    pub fn has_species_battle_data(&self, key: &RuntimeSpeciesBattleDataKey) -> bool {
        self.data
            .pokemon
            .get(&key.species_id)
            .is_some_and(|species| {
                species.int_id == key.int_id
                    && species.base_stats.hp == key.base_hp
                    && species.base_stats.attack == key.base_attack
                    && species.base_stats.defense == key.base_defense
                    && species.base_stats.speed == key.base_speed
                    && species.base_stats.special_attack == key.base_special_attack
                    && species.base_stats.special_defense == key.base_special_defense
                    && species.type1 == key.type1
                    && species.type2 == key.type2
                    && species.catch_rate == key.catch_rate
                    && species.base_exp == key.base_exp
                    && species.item1 == key.item1
                    && species.item2 == key.item2
                    && species.gender_ratio == key.gender_ratio
                    && species.step_cycles_to_hatch == key.step_cycles_to_hatch
                    && species.growth_rate == key.growth_rate
                    && species.egg_group1 == key.egg_group1
                    && species.egg_group2 == key.egg_group2
                    && species.tmhm_learnset == key.tmhm_learnset
                    && species.ability == key.ability
                    && species.weight == key.weight
            })
    }

    pub fn has_trainer_battle_data(&self, key: &RuntimeTrainerBattleDataKey) -> bool {
        self.data
            .trainers
            .trainers
            .get(&key.trainer_id)
            .is_some_and(|trainer| {
                trainer.name == key.name
                    && trainer.trainer_class == key.trainer_class
                    && trainer.win_quote == key.win_quote
                    && trainer.lose_quote == key.lose_quote
                    && trainer.items == key.items
                    && trainer.base_reward == key.base_reward
                    && trainer.ai_move_flags == key.ai_move_flags
                    && trainer.ai_item_switch_flags == key.ai_item_switch_flags
                    && trainer.encounter_music == key.encounter_music
                    && trainer.ai_layers == key.ai_layers
            })
    }

    pub fn has_trainer_party_pokemon(&self, key: &RuntimeTrainerPartyPokemonKey) -> bool {
        self.data
            .trainers
            .trainers
            .get(&key.trainer_id)
            .and_then(|trainer| trainer.party.get(key.party_index))
            .is_some_and(|pokemon| {
                pokemon.species == key.species
                    && pokemon.level == key.level
                    && pokemon.item == key.item
                    && pokemon
                        .moves
                        .iter()
                        .map(|move_data| &move_data.name)
                        .eq(key.move_names.iter())
                    && pokemon
                        .moves
                        .iter()
                        .map(|move_data| move_data.current_pp)
                        .eq(key.move_pp.iter().copied())
                    && pokemon
                        .moves
                        .iter()
                        .map(|move_data| move_data.pp_ups)
                        .eq(key.move_pp_ups.iter().copied())
                    && pokemon.dvs.attack == key.dv_attack
                    && pokemon.dvs.defense == key.dv_defense
                    && pokemon.dvs.speed == key.dv_speed
                    && pokemon.dvs.special == key.dv_special
                    && pokemon.dvs.hp == key.dv_hp
            })
    }

    pub fn has_move_priority_effect_key(&self, key: &RuntimeMovePriorityEffectKey) -> bool {
        self.data
            .move_priorities
            .effect_priorities
            .get(&key.effect_id)
            .is_some_and(|priority| *priority == key.priority)
    }

    pub fn has_move_priority_move_key(&self, key: &RuntimeMovePriorityMoveKey) -> bool {
        self.data
            .move_priorities
            .move_priorities
            .iter()
            .any(|priority| priority.r#move == key.move_id && priority.priority == key.priority)
    }

    pub fn has_battle_stat_multiplier(&self, key: &RuntimeBattleStatMultiplierKey) -> bool {
        let index = key.stage + 6;
        if !(0..=12).contains(&index) {
            return false;
        }
        let table = match key.table.as_str() {
            "stat" => &self.data.battle_stat_multipliers.stat,
            "accuracy" => &self.data.battle_stat_multipliers.accuracy,
            _ => return false,
        };
        table.get(index as usize).is_some_and(|multiplier| {
            multiplier.numerator == key.numerator && multiplier.denominator == key.denominator
        })
    }

    pub fn has_battle_reward_rule(&self, key: &RuntimeBattleRewardRuleKey) -> bool {
        match key.field.as_str() {
            "max_level" => i32::from(self.data.battle_reward_rules.max_level) == key.value,
            "wild_exp_divisor" => self.data.battle_reward_rules.wild_exp_divisor == key.value,
            "trainer_exp_numerator" => {
                self.data.battle_reward_rules.trainer_exp_numerator == key.value
            }
            "trainer_exp_denominator" => {
                self.data.battle_reward_rules.trainer_exp_denominator == key.value
            }
            _ => false,
        }
    }

    pub fn has_battle_escape_rule(&self, key: &RuntimeBattleEscapeRuleKey) -> bool {
        match key.field.as_str() {
            "player_speed_multiplier" => {
                self.data.battle_escape_rules.player_speed_multiplier == key.value
            }
            "enemy_speed_divisor" => self.data.battle_escape_rules.enemy_speed_divisor == key.value,
            "failed_attempt_bonus" => {
                self.data.battle_escape_rules.failed_attempt_bonus == key.value
            }
            "rng_roll_values" => self.data.battle_escape_rules.rng_roll_values == key.value,
            _ => false,
        }
    }

    pub fn has_physical_type(&self, type_id: &str) -> bool {
        self.data
            .type_categories
            .physical
            .iter()
            .any(|known| known == type_id)
    }

    pub fn has_special_type(&self, type_id: &str) -> bool {
        self.data
            .type_categories
            .special
            .iter()
            .any(|known| known == type_id)
    }

    pub fn has_weather(&self, weather_id: &str) -> bool {
        self.data
            .weather_modifiers
            .type_modifiers
            .contains_key(weather_id)
            || self
                .data
                .weather_modifiers
                .move_effect_modifiers
                .contains_key(weather_id)
    }

    pub fn has_type_effectiveness(&self, key: &RuntimeTypeEffectivenessKey) -> bool {
        self.data.type_effectiveness.matchups.iter().any(|entry| {
            entry.attacker == key.attacking_type && entry.defender == key.defending_type
        })
    }

    pub fn has_foresight_type_effectiveness(&self, key: &RuntimeTypeEffectivenessKey) -> bool {
        self.data
            .type_effectiveness
            .foresight_matchups
            .iter()
            .any(|entry| {
                entry.attacker == key.attacking_type && entry.defender == key.defending_type
            })
    }

    pub fn has_weather_type_modifier(&self, key: &RuntimeWeatherTypeModifierKey) -> bool {
        self.data
            .weather_modifiers
            .type_modifiers
            .get(&key.weather)
            .is_some_and(|modifiers| modifiers.contains_key(&key.type_id))
    }

    pub fn has_weather_move_effect_modifier(
        &self,
        key: &RuntimeWeatherMoveEffectModifierKey,
    ) -> bool {
        self.data
            .weather_modifiers
            .move_effect_modifiers
            .get(&key.weather)
            .is_some_and(|modifiers| modifiers.contains_key(&key.effect_id))
    }

    pub fn has_audio_asset(&self, key: &RuntimeAudioAssetKey) -> bool {
        self.audio.has_audio_asset(key)
    }

    pub fn has_pokemon_cry(&self, key: &RuntimePokemonCryKey) -> bool {
        self.pokemon_cry_keys().contains(key)
    }

    pub fn has_music(&self, music_id: &str) -> bool {
        self.audio.music.contains_key(music_id)
    }

    pub fn has_sound_effect(&self, sound_effect_id: &str) -> bool {
        self.audio.sound_effects.contains_key(sound_effect_id)
    }

    pub fn has_cry(&self, cry_id: &str) -> bool {
        self.audio.cries.contains_key(cry_id)
    }

    pub fn require_special_routine(&self, routine: &str) -> Result<()> {
        self.data.require_special_routine(routine)
    }

    pub fn require_item(&self, item_id: &str) -> Result<()> {
        require_runtime_catalog_id("item", item_id, self.has_item(item_id))
    }

    pub fn require_move(&self, move_id: &str) -> Result<()> {
        require_runtime_catalog_id("move", move_id, self.has_move(move_id))
    }

    pub fn require_species(&self, species_id: &str) -> Result<()> {
        require_runtime_catalog_id("Pokemon species", species_id, self.has_species(species_id))
    }

    pub fn require_map(&self, map_name: &str) -> Result<()> {
        require_runtime_catalog_id("map", map_name, self.has_map(map_name))
    }

    pub fn require_trainer(&self, trainer_id: &str) -> Result<()> {
        require_runtime_catalog_id("trainer", trainer_id, self.has_trainer(trainer_id))
    }

    pub fn require_text(&self, text_label: &str) -> Result<()> {
        self.data
            .validate_saved_text_reference("runtime.text", text_label)
    }

    pub fn require_menu(&self, menu: &str) -> Result<()> {
        self.data
            .validate_saved_menu_reference("runtime.menu", menu)
    }

    pub fn require_phone_contact(&self, contact_id: &str) -> Result<()> {
        self.data
            .validate_saved_phone_contact_reference("runtime.phone_contact", contact_id)
    }

    pub fn require_special_phone_call(&self, call_id: &str) -> Result<()> {
        self.data
            .validate_saved_special_phone_call_reference("runtime.special_phone_call", call_id)
    }

    pub fn require_npc_trade(&self, trade_id: &str) -> Result<()> {
        self.data
            .validate_saved_npc_trade_reference("runtime.npc_trade", trade_id)
    }

    pub fn require_sprite(&self, sprite_id: &str) -> Result<()> {
        self.data
            .validate_saved_sprite_reference("runtime.sprite", sprite_id)
    }

    pub fn require_map_constant(&self, map_constant: &str) -> Result<()> {
        self.data
            .validate_saved_map_constant_reference("runtime.map_constant", map_constant)
    }

    pub fn require_event_flag(&self, flag: &str) -> Result<()> {
        self.data
            .validate_saved_event_flag_reference("runtime.event_flag", flag)
    }

    pub fn require_engine_flag(&self, flag: &str) -> Result<()> {
        self.data
            .validate_saved_engine_flag_reference("runtime.engine_flag", flag)
    }

    pub fn require_spawn_identifier(&self, spawn_identifier: u16) -> Result<()> {
        self.data
            .validate_saved_spawn_reference("runtime.spawn_identifier", spawn_identifier)
    }

    pub fn require_tileset(&self, tileset_id: &str) -> Result<()> {
        require_runtime_catalog_id("tileset", tileset_id, self.has_tileset(tileset_id))
    }

    pub fn require_tileset_row(&self, key: &RuntimeTilesetKey) -> Result<()> {
        if self.has_tileset_row(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact tileset row {}",
                key.tileset_id
            )
        }
    }

    pub fn require_pc_string(&self, key: &RuntimePcStringKey) -> Result<()> {
        if self.has_pc_string(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact PC string row {}",
                key.string_id
            )
        }
    }

    pub fn require_menu_icon(&self, key: &RuntimeMenuIconKey) -> Result<()> {
        if self.has_menu_icon(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact menu icon row {}",
                key.species_id
            )
        }
    }

    pub fn require_pokedex_entry(&self, key: &RuntimePokedexEntryKey) -> Result<()> {
        if self.has_pokedex_entry(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact Pokedex entry row {}",
                key.species_id
            )
        }
    }

    pub fn require_landmark(&self, landmark_id: &str) -> Result<()> {
        require_runtime_catalog_id(
            "Pokegear landmark",
            landmark_id,
            self.has_landmark(landmark_id),
        )
    }

    pub fn require_pokegear_landmark(&self, key: &RuntimePokegearLandmarkKey) -> Result<()> {
        if self.has_pokegear_landmark(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact Pokegear landmark row {}",
                key.constant
            )
        }
    }

    pub fn require_pokegear_map_landmark(&self, key: &RuntimePokegearMapLandmarkKey) -> Result<()> {
        if self.has_pokegear_map_landmark(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact Pokegear map landmark row {}",
                key.map_name
            )
        }
    }

    pub fn require_fishing_rod(&self, rod: &str) -> Result<()> {
        require_runtime_catalog_id("fishing rod", rod, self.has_fishing_rod(rod))
    }

    pub fn require_map_group(&self, group_id: &str) -> Result<()> {
        require_runtime_catalog_id("map group", group_id, self.has_map_group(group_id))
    }

    pub fn require_encounter_group(&self, group_id: &str) -> Result<()> {
        require_runtime_catalog_id(
            "encounter group",
            group_id,
            self.has_encounter_group(group_id),
        )
    }

    pub fn require_mart(&self, mart_id: &str) -> Result<()> {
        require_runtime_catalog_id("mart", mart_id, self.has_mart(mart_id))
    }

    pub fn require_mart_row(&self, key: &RuntimeMartKey) -> Result<()> {
        if self.has_mart_row(key) {
            Ok(())
        } else {
            anyhow::bail!("compiled game pack missing exact mart row {}", key.mart_id)
        }
    }

    pub fn require_fruit_tree(&self, fruit_tree_id: &str) -> Result<()> {
        require_runtime_catalog_id(
            "fruit tree",
            fruit_tree_id,
            self.has_fruit_tree(fruit_tree_id),
        )
    }

    pub fn require_fruit_tree_row(&self, key: &RuntimeFruitTreeKey) -> Result<()> {
        if self.has_fruit_tree_row(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact fruit tree row {}",
                key.fruit_tree_id
            )
        }
    }

    pub fn require_field_move_rule(&self, rule_id: &str) -> Result<()> {
        require_runtime_catalog_id(
            "field move rule",
            rule_id,
            self.has_field_move_rule(rule_id),
        )
    }

    pub fn require_field_move_rule_row(&self, key: &RuntimeFieldMoveRuleKey) -> Result<()> {
        if self.has_field_move_rule_row(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact field move rule row {}",
                key.rule_id
            )
        }
    }

    pub fn require_fly_destination(&self, flypoint_flag: &str) -> Result<()> {
        require_runtime_catalog_id(
            "fly destination",
            flypoint_flag,
            self.has_fly_destination(flypoint_flag),
        )
    }

    pub fn require_fly_destination_row(&self, key: &RuntimeFlyDestinationKey) -> Result<()> {
        if self.has_fly_destination_row(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact fly destination row {}",
                key.flypoint_flag
            )
        }
    }

    pub fn require_field_move_move(&self, move_id: &str) -> Result<()> {
        require_runtime_catalog_id(
            "field move move",
            move_id,
            self.has_field_move_move(move_id),
        )
    }

    pub fn require_field_move_item(&self, item_id: &str) -> Result<()> {
        require_runtime_catalog_id(
            "field move item",
            item_id,
            self.has_field_move_item(item_id),
        )
    }

    pub fn require_flee_mon_bucket(&self, bucket_id: &str) -> Result<()> {
        require_runtime_catalog_id(
            "flee mon bucket",
            bucket_id,
            self.has_flee_mon_bucket(bucket_id),
        )
    }

    pub fn require_buena_password_category(&self, category_id: &str) -> Result<()> {
        require_runtime_catalog_id(
            "Buena password category",
            category_id,
            self.has_buena_password_category(category_id),
        )
    }

    pub fn require_roaming_species(&self, species_id: &str) -> Result<()> {
        require_runtime_catalog_id(
            "roaming Pokemon species",
            species_id,
            self.has_roaming_species(species_id),
        )
    }

    pub fn require_buena_prize_item(&self, item_id: &str) -> Result<()> {
        require_runtime_catalog_id(
            "Buena prize item",
            item_id,
            self.has_buena_prize_item(item_id),
        )
    }

    pub fn require_kurt_apricorn_item(&self, item_id: &str) -> Result<()> {
        require_runtime_catalog_id(
            "Kurt apricorn item",
            item_id,
            self.has_kurt_apricorn_item(item_id),
        )
    }

    pub fn require_dratini_move_set(&self, answer: u8) -> Result<()> {
        if self.has_dratini_move_set(answer) {
            Ok(())
        } else {
            anyhow::bail!("compiled game pack missing exact Dratini move set id {answer}")
        }
    }

    pub fn require_special_feature(&self, feature_id: &str) -> Result<()> {
        require_runtime_catalog_id(
            "special feature",
            feature_id,
            self.has_special_feature(feature_id),
        )
    }

    pub fn require_oak_rating_text(&self, text_id: &str) -> Result<()> {
        require_runtime_catalog_id(
            "Oak rating text",
            text_id,
            self.has_oak_rating_text(text_id),
        )
    }

    pub fn require_odd_egg_species(&self, species_id: &str) -> Result<()> {
        require_runtime_catalog_id(
            "Odd Egg species",
            species_id,
            self.has_odd_egg_species(species_id),
        )
    }

    pub fn require_magikarp_length_threshold(&self, threshold: u16) -> Result<()> {
        if self.has_magikarp_length_threshold(threshold) {
            Ok(())
        } else {
            anyhow::bail!("compiled game pack missing exact Magikarp length threshold {threshold}")
        }
    }

    pub fn require_happiness_change(&self, change_id: u8) -> Result<()> {
        if self.has_happiness_change(change_id) {
            Ok(())
        } else {
            anyhow::bail!("compiled game pack missing exact happiness change id {change_id}")
        }
    }

    pub fn require_happiness_service(&self, service_id: &str) -> Result<()> {
        require_runtime_catalog_id(
            "happiness service",
            service_id,
            self.has_happiness_service(service_id),
        )
    }

    pub fn require_pokemon_status(&self, status: &str) -> Result<()> {
        self.data
            .validate_saved_pokemon_status_reference("runtime.pokemon_status", status)
    }

    pub fn require_fishing_daily_flag_bit(&self, bit: u32) -> Result<()> {
        if self.has_fishing_daily_flag_bit(bit) {
            Ok(())
        } else {
            anyhow::bail!("compiled game pack missing exact fishing daily flag bit {bit}")
        }
    }

    pub fn require_fishing_swarm_flag(&self, swarm_flag: u8) -> Result<()> {
        if self.has_fishing_swarm_flag(swarm_flag) {
            Ok(())
        } else {
            anyhow::bail!("compiled game pack missing exact fishing swarm flag {swarm_flag}")
        }
    }

    pub fn require_pending_special_battle_type(&self, battle_type: &str) -> Result<()> {
        self.data
            .validate_saved_pending_special_battle_type(Some(battle_type))
    }

    pub fn require_wild_encounter_origin(&self, key: &RuntimeWildEncounterOriginKey) -> Result<()> {
        if self.has_wild_encounter_origin(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact wild encounter origin {}:{}:{}",
                key.map_name,
                key.species,
                key.level
            )
        }
    }

    pub fn require_script_label(&self, script_label: &str) -> Result<()> {
        self.data
            .validate_saved_script_label_reference("runtime.script_label", script_label)
    }

    pub fn require_script_command(&self, key: &RuntimeScriptCommandKey) -> Result<()> {
        self.data.validate_saved_script_command_reference(
            "runtime.script_command",
            &key.script_label,
            key.command_index,
        )
    }

    pub fn require_script_command_payload(
        &self,
        key: &RuntimeScriptCommandPayloadKey,
    ) -> Result<()> {
        self.data.validate_saved_script_command_payload_reference(
            "runtime.script_command_payload",
            &key.script_label,
            key.command_index,
            &key.command,
            &key.args,
        )
    }

    pub fn require_script_return(&self, key: &RuntimeScriptReturnKey) -> Result<()> {
        self.data.validate_saved_script_return_reference(
            "runtime.script_return",
            &key.script_label,
            key.next_command_index,
        )
    }

    pub fn require_script_vertical_menu(&self, key: &RuntimeScriptVerticalMenuKey) -> Result<()> {
        if self.has_script_vertical_menu(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact script vertical menu row {}:{}",
                key.map_name,
                key.menu_key
            )
        }
    }

    pub fn require_script_text_body(&self, key: &RuntimeScriptTextBodyKey) -> Result<()> {
        if self.has_script_text_body(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact script text body row {}:{}",
                key.map_name,
                key.body_key
            )
        }
    }

    pub fn require_script_menu_definition(
        &self,
        key: &RuntimeScriptMenuDefinitionKey,
    ) -> Result<()> {
        if self.has_script_menu_definition(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact script menu definition row {}:{}",
                key.map_name,
                key.menu_key
            )
        }
    }

    pub fn require_script_elevator(&self, key: &RuntimeScriptElevatorKey) -> Result<()> {
        if self.has_script_elevator(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact script elevator row {}:{}",
                key.map_name,
                key.elevator_key
            )
        }
    }

    pub fn require_gift_pokemon(&self, key: &RuntimeGiftPokemonKey) -> Result<()> {
        if self.has_gift_pokemon(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact gift Pokemon row {}:{}",
                key.map_name,
                key.source_script
            )
        }
    }

    pub fn require_script_object_command(&self, key: &RuntimeScriptObjectCommandKey) -> Result<()> {
        if self.has_script_object_command(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact script object command row {}:{}",
                key.map_name,
                key.command_index
            )
        }
    }

    pub fn require_script_movement(&self, key: &RuntimeScriptMovementKey) -> Result<()> {
        if self.has_script_movement(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact script movement row {}:{}",
                key.map_name,
                key.label
            )
        }
    }

    pub fn require_map_script_section_command(
        &self,
        key: &RuntimeMapScriptSectionCommandKey,
    ) -> Result<()> {
        if self.has_map_script_section_command(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact map script section command row {}:{}",
                key.map_name,
                key.command_index
            )
        }
    }

    pub fn require_map_event_section_command(
        &self,
        key: &RuntimeMapEventSectionCommandKey,
    ) -> Result<()> {
        if self.has_map_event_section_command(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact map event section command row {}:{}",
                key.map_name,
                key.command_index
            )
        }
    }

    pub fn require_script_map_command(&self, key: &RuntimeScriptMapCommandKey) -> Result<()> {
        if self.has_script_map_command(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact script map command row {}:{}:{}",
                key.map_name,
                key.source_script,
                key.command_index
            )
        }
    }

    pub fn require_script_variable_command(
        &self,
        key: &RuntimeScriptVariableCommandKey,
    ) -> Result<()> {
        if self.has_script_variable_command(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact script variable command row {}:{}:{}",
                key.map_name,
                key.source_script,
                key.command_index
            )
        }
    }

    pub fn require_script_control_command(
        &self,
        key: &RuntimeScriptControlCommandKey,
    ) -> Result<()> {
        if self.has_script_control_command(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact script control command row {}:{}:{}",
                key.map_name,
                key.source_script,
                key.command_index
            )
        }
    }

    pub fn require_script_swarm_command(&self, key: &RuntimeScriptSwarmCommandKey) -> Result<()> {
        if self.has_script_swarm_command(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact script swarm command row {}:{}:{}",
                key.map_name,
                key.source_script,
                key.command_index
            )
        }
    }

    pub fn require_script_field_pickup(&self, key: &RuntimeScriptFieldPickupKey) -> Result<()> {
        if self.has_script_field_pickup(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact script field pickup row {}:{}:{}",
                key.map_name,
                key.source_script,
                key.command_index
            )
        }
    }

    pub fn require_script_shop_command(&self, key: &RuntimeScriptShopCommandKey) -> Result<()> {
        if self.has_script_shop_command(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact script shop command row {}:{}:{}",
                key.map_name,
                key.source_script,
                key.command_index
            )
        }
    }

    pub fn require_script_phone_command(&self, key: &RuntimeScriptPhoneCommandKey) -> Result<()> {
        if self.has_script_phone_command(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact script phone command row {}:{}:{}",
                key.map_name,
                key.source_script,
                key.command_index
            )
        }
    }

    pub fn require_script_runtime_command(
        &self,
        key: &RuntimeScriptRuntimeCommandKey,
    ) -> Result<()> {
        if self.has_script_runtime_command(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact script runtime command row {}:{}:{}",
                key.map_name,
                key.source_script,
                key.command_index
            )
        }
    }

    pub fn require_script_item_grant(&self, key: &RuntimeScriptItemGrantKey) -> Result<()> {
        if self.has_script_item_grant(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact script item grant row {}:{}:{}",
                key.map_name,
                key.source_script,
                key.command_index
            )
        }
    }

    pub fn require_script_item_access(&self, key: &RuntimeScriptItemAccessKey) -> Result<()> {
        if self.has_script_item_access(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact script item {} row {}:{}:{}",
                key.command,
                key.map_name,
                key.source_script,
                key.command_index
            )
        }
    }

    pub fn require_script_economy_command(
        &self,
        key: &RuntimeScriptEconomyCommandKey,
    ) -> Result<()> {
        if self.has_script_economy_command(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact script economy command row {}:{}:{}",
                key.map_name,
                key.source_script,
                key.command_index
            )
        }
    }

    pub fn require_script_flag_command(&self, key: &RuntimeScriptFlagCommandKey) -> Result<()> {
        if self.has_script_flag_command(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact script flag command row {}:{}:{}",
                key.map_name,
                key.source_script,
                key.command_index
            )
        }
    }

    pub fn require_script_scene_command(&self, key: &RuntimeScriptSceneCommandKey) -> Result<()> {
        if self.has_script_scene_command(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact script scene command row {}:{}:{}",
                key.map_name,
                key.source_script,
                key.command_index
            )
        }
    }

    pub fn require_script_block_change(&self, key: &RuntimeScriptBlockChangeKey) -> Result<()> {
        if self.has_script_block_change(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact script block change row {}:{}:{}",
                key.map_name,
                key.source_script,
                key.command_index
            )
        }
    }

    pub fn require_script_audio_command(&self, key: &RuntimeScriptAudioCommandKey) -> Result<()> {
        if self.has_script_audio_command(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact script audio command row {}:{}:{}",
                key.map_name,
                key.source_script,
                key.command_index
            )
        }
    }

    pub fn require_script_text_command(&self, key: &RuntimeScriptTextCommandKey) -> Result<()> {
        if self.has_script_text_command(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact script text command row {}:{}:{}",
                key.map_name,
                key.source_script,
                key.command_index
            )
        }
    }

    pub fn require_warp(&self, key: &RuntimeWarpKey) -> Result<()> {
        self.data
            .validate_saved_warp_reference("runtime.warp", &key.map_name, key.warp_index)
            .map(|_| ())
    }

    pub fn require_map_object(&self, key: &RuntimeMapObjectKey) -> Result<()> {
        self.data
            .validate_saved_map_object_reference(
                &key.map_name,
                "runtime.map_object",
                &key.object_id,
            )
            .map(|_| ())
    }

    pub fn require_map_scene(&self, key: &RuntimeMapSceneKey) -> Result<()> {
        if self.has_map_scene(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact map scene {}:{}",
                key.map_name,
                key.scene_id
            )
        }
    }

    pub fn require_map_metadata(&self, key: &RuntimeMapMetadataKey) -> Result<()> {
        if self.has_map_metadata(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact map metadata row {}",
                key.map_name
            )
        }
    }

    pub fn require_currency_constant(&self, id: &str) -> Result<()> {
        require_runtime_catalog_id("currency constant", id, self.has_currency_constant(id))
    }

    pub fn require_capture_ball_rule(&self, id: &str) -> Result<()> {
        require_runtime_catalog_id("capture ball rule", id, self.has_capture_ball_rule(id))
    }

    pub fn require_guaranteed_capture_ball(&self, id: &str) -> Result<()> {
        require_runtime_catalog_id(
            "guaranteed capture ball",
            id,
            self.has_guaranteed_capture_ball(id),
        )
    }

    pub fn require_capture_status_bonus(&self, status: &str) -> Result<()> {
        require_runtime_catalog_id(
            "capture status bonus",
            status,
            self.has_capture_status_bonus(status),
        )
    }

    pub fn require_fast_ball_species(&self, species_id: &str) -> Result<()> {
        require_runtime_catalog_id(
            "fast ball species",
            species_id,
            self.has_fast_ball_species(species_id),
        )
    }

    pub fn require_heavy_ball_species(&self, species_id: &str) -> Result<()> {
        require_runtime_catalog_id(
            "heavy ball species",
            species_id,
            self.has_heavy_ball_species(species_id),
        )
    }

    pub fn require_move_priority_effect(&self, effect_id: &str) -> Result<()> {
        require_runtime_catalog_id(
            "move priority effect",
            effect_id,
            self.has_move_priority_effect(effect_id),
        )
    }

    pub fn require_move_priority_move(&self, move_id: &str) -> Result<()> {
        require_runtime_catalog_id(
            "move priority move",
            move_id,
            self.has_move_priority_move(move_id),
        )
    }

    pub fn require_capture_ball_rule_key(&self, key: &RuntimeCaptureBallRuleKey) -> Result<()> {
        if self.has_capture_ball_rule_key(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact capture ball rule row {}",
                key.ball_id
            )
        }
    }

    pub fn require_heavy_ball_modifier(&self, key: &RuntimeHeavyBallModifierKey) -> Result<()> {
        if self.has_heavy_ball_modifier(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact heavy ball modifier row {}:{}",
                key.species_id,
                key.modifier
            )
        }
    }

    pub fn require_capture_status_bonus_key(
        &self,
        key: &RuntimeCaptureStatusBonusKey,
    ) -> Result<()> {
        if self.has_capture_status_bonus_key(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact capture status bonus row {}:{}",
                key.status,
                key.bonus
            )
        }
    }

    pub fn require_capture_wobble_probability(
        &self,
        key: &RuntimeCaptureWobbleProbabilityKey,
    ) -> Result<()> {
        if self.has_capture_wobble_probability(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact capture wobble probability row {}:{}",
                key.catch_rate,
                key.chance
            )
        }
    }

    pub fn require_item_battle_use(&self, key: &RuntimeItemBattleUseKey) -> Result<()> {
        if self.has_item_battle_use(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact item battle-use row {}",
                key.item_id
            )
        }
    }

    pub fn require_item_effect_plan(&self, key: &RuntimeItemEffectPlanKey) -> Result<()> {
        if self.has_item_effect_plan(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact item effect plan row {}:{}:{}",
                key.item_id,
                key.effect_id,
                key.behavior_id
            )
        }
    }

    pub fn require_item_field_use(&self, key: &RuntimeItemFieldUseKey) -> Result<()> {
        if self.has_item_field_use(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact item field-use row {}",
                key.item_id
            )
        }
    }

    pub fn require_move_battle_data(&self, key: &RuntimeMoveBattleDataKey) -> Result<()> {
        if self.has_move_battle_data(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact move battle data row {}",
                key.move_id
            )
        }
    }

    pub fn require_species_battle_data(&self, key: &RuntimeSpeciesBattleDataKey) -> Result<()> {
        if self.has_species_battle_data(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact Pokemon species battle data row {}",
                key.species_id
            )
        }
    }

    pub fn require_trainer_battle_data(&self, key: &RuntimeTrainerBattleDataKey) -> Result<()> {
        if self.has_trainer_battle_data(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact trainer battle data row {}",
                key.trainer_id
            )
        }
    }

    pub fn require_trainer_party_pokemon(&self, key: &RuntimeTrainerPartyPokemonKey) -> Result<()> {
        if self.has_trainer_party_pokemon(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact trainer party row {}:{}",
                key.trainer_id,
                key.party_index
            )
        }
    }

    pub fn require_move_priority_effect_key(
        &self,
        key: &RuntimeMovePriorityEffectKey,
    ) -> Result<()> {
        if self.has_move_priority_effect_key(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact move priority effect row {}:{}",
                key.effect_id,
                key.priority
            )
        }
    }

    pub fn require_move_priority_move_key(&self, key: &RuntimeMovePriorityMoveKey) -> Result<()> {
        if self.has_move_priority_move_key(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact move priority move row {}:{}",
                key.move_id,
                key.priority
            )
        }
    }

    pub fn require_battle_stat_multiplier(
        &self,
        key: &RuntimeBattleStatMultiplierKey,
    ) -> Result<()> {
        if self.has_battle_stat_multiplier(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact battle stat multiplier row {}:{}:{}/{}",
                key.table,
                key.stage,
                key.numerator,
                key.denominator
            )
        }
    }

    pub fn require_battle_reward_rule(&self, key: &RuntimeBattleRewardRuleKey) -> Result<()> {
        if self.has_battle_reward_rule(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact battle reward rule row {}:{}",
                key.field,
                key.value
            )
        }
    }

    pub fn require_battle_escape_rule(&self, key: &RuntimeBattleEscapeRuleKey) -> Result<()> {
        if self.has_battle_escape_rule(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact battle escape rule row {}:{}",
                key.field,
                key.value
            )
        }
    }

    pub fn require_physical_type(&self, type_id: &str) -> Result<()> {
        require_runtime_catalog_id("physical type", type_id, self.has_physical_type(type_id))
    }

    pub fn require_special_type(&self, type_id: &str) -> Result<()> {
        require_runtime_catalog_id("special type", type_id, self.has_special_type(type_id))
    }

    pub fn require_weather(&self, weather_id: &str) -> Result<()> {
        require_runtime_catalog_id("weather", weather_id, self.has_weather(weather_id))
    }

    pub fn require_type_effectiveness(&self, key: &RuntimeTypeEffectivenessKey) -> Result<()> {
        if self.has_type_effectiveness(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact type effectiveness matchup {}:{}",
                key.attacking_type,
                key.defending_type
            )
        }
    }

    pub fn require_foresight_type_effectiveness(
        &self,
        key: &RuntimeTypeEffectivenessKey,
    ) -> Result<()> {
        if self.has_foresight_type_effectiveness(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact foresight type effectiveness matchup {}:{}",
                key.attacking_type,
                key.defending_type
            )
        }
    }

    pub fn require_audio_asset(&self, key: &RuntimeAudioAssetKey) -> Result<()> {
        if self.has_audio_asset(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact audio asset row {}:{}",
                key.kind,
                key.audio_id
            )
        }
    }

    pub fn require_pokemon_cry(&self, key: &RuntimePokemonCryKey) -> Result<()> {
        if self.has_pokemon_cry(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact Pokemon cry row {}:{}",
                key.species_id,
                key.cry_id
            )
        }
    }

    pub fn require_weather_type_modifier(&self, key: &RuntimeWeatherTypeModifierKey) -> Result<()> {
        if self.has_weather_type_modifier(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact weather type modifier {}:{}",
                key.weather,
                key.type_id
            )
        }
    }

    pub fn require_weather_move_effect_modifier(
        &self,
        key: &RuntimeWeatherMoveEffectModifierKey,
    ) -> Result<()> {
        if self.has_weather_move_effect_modifier(key) {
            Ok(())
        } else {
            anyhow::bail!(
                "compiled game pack missing exact weather move effect modifier {}:{}",
                key.weather,
                key.effect_id
            )
        }
    }

    pub fn require_music(&self, music_id: &str) -> Result<()> {
        self.audio.require_music(music_id).map(|_| ())
    }

    pub fn require_sound_effect(&self, sound_effect_id: &str) -> Result<()> {
        self.audio.require_sound_effect(sound_effect_id).map(|_| ())
    }

    pub fn require_cry(&self, cry_id: &str) -> Result<()> {
        self.audio.require_cry(cry_id).map(|_| ())
    }

    pub fn pack_identity(&self) -> &CompiledGamePackIdentity {
        &self.pack_identity
    }

    pub fn viewport(&self) -> &GameViewport {
        &self.viewport
    }

    pub fn save_game(&self, path: impl AsRef<Path>, state: GameState) -> Result<()> {
        self.validate_save_state_for_runtime_pack(&state)?;
        write_save_game_for_modpack(path, state, &self.modpack, &self.pack_identity.content_hash)
            .context("write Crystal runtime save")
    }

    pub fn load_save(&self, path: impl AsRef<Path>) -> Result<GameState> {
        let save =
            read_save_game_for_modpack(path, &self.modpack, &self.pack_identity.content_hash)
                .context("read Crystal runtime save for compiled modpack identity")?;
        let mut state = save.into_state();
        self.validate_save_state_for_runtime_pack(&state)?;
        crystal_core::systems::script_flags::reconcile_saved_badge_flags(&mut state);
        Ok(state)
    }

    pub fn load_save_summary(&self, path: impl AsRef<Path>) -> Result<SaveGameSummary> {
        read_save_game_summary_for_modpack(path, &self.modpack, &self.pack_identity.content_hash)
            .context("read Crystal runtime save summary for compiled modpack identity")
    }

    pub fn list_save_slots(&self, directory: impl AsRef<Path>) -> Result<Vec<SaveSlotSummary>> {
        list_save_game_summaries_for_modpack(
            directory,
            &self.modpack,
            &self.pack_identity.content_hash,
        )
        .context("list Crystal runtime save slots for compiled modpack identity")
    }

    pub fn save_summary_for_state(&self, state: &GameState) -> Result<SaveGameSummary> {
        self.validate_save_state_for_runtime_pack(state)?;
        SaveGameSummary::new(
            self.modpack.clone(),
            self.pack_identity.content_hash.clone(),
            state,
        )
        .context("build Crystal runtime save summary")
    }

    pub fn save_checkpoint_for_state(
        &self,
        state: &GameState,
        player_id: PlayerId,
    ) -> Result<SaveCheckpointFrame> {
        let summary = self.save_summary_for_state(state)?;
        let checksum = StateChecksumFrame::from_game_state(player_id, state)
            .context("checksum Crystal runtime save checkpoint state")?;
        SaveCheckpointFrame::new(summary, checksum).context("build Crystal runtime save checkpoint")
    }

    pub fn session_save_checkpoint_for_state(
        &self,
        session: LinkSessionIdentity,
        state: &GameState,
        player_id: PlayerId,
    ) -> Result<SessionSaveCheckpointFrame> {
        let checkpoint = self.save_checkpoint_for_state(state, player_id)?;
        SessionSaveCheckpointFrame::new(session, checkpoint)
            .context("build session-bound Crystal runtime save checkpoint")
    }

    pub fn load_save_checkpoint(
        &self,
        path: impl AsRef<Path>,
        player_id: PlayerId,
    ) -> Result<SaveCheckpointFrame> {
        let state = self.load_save(path)?;
        self.save_checkpoint_for_state(&state, player_id)
    }

    fn active_menu_snapshot(&self, state: &GameState) -> Result<Option<RuntimeMenuSnapshot>> {
        let Some(menu_id) = state.script_runtime.active_menu.clone() else {
            return Ok(None);
        };
        if let OverworldMemory::Active { map_name, .. } = &state.overworld
            && let Some(module) = self.data.maps.get(map_name)
            && let Some(definition) = module.script_menu_definitions.get(&menu_id)
        {
            let vertical_menus = module
                .script_vertical_menus
                .values()
                .filter(|menu| menu.header_label == menu_id)
                .map(RuntimeVerticalMenuSnapshot::from_definition)
                .collect();
            return RuntimeMenuSnapshot::from_state(
                state,
                menu_id,
                RuntimeMenuSource::ScriptDefinition {
                    map_name: map_name.clone(),
                },
                Some(definition.clone()),
                vertical_menus,
            )
            .map(Some);
        }
        if self.data.special_routines.contains_key(&menu_id) {
            return RuntimeMenuSnapshot::from_state(
                state,
                menu_id,
                RuntimeMenuSource::SpecialRoutine,
                None,
                Vec::new(),
            )
            .map(Some);
        }
        match &state.overworld {
            OverworldMemory::Active { map_name, .. } => anyhow::bail!(
                "active runtime menu '{menu_id}' is not declared by current compiled map {map_name}"
            ),
            OverworldMemory::Inactive => anyhow::bail!(
                "active runtime menu '{menu_id}' requires an active overworld map or special routine"
            ),
        }
    }

    fn ui_snapshot(
        &self,
        state: &GameState,
        menu: Option<RuntimeMenuSnapshot>,
    ) -> Result<RuntimeUiSnapshot> {
        let text = self.active_text_snapshot(state)?;
        let elevators = self.elevator_snapshots(state);
        let gift_pokemon = self.gift_pokemon_snapshots(state);
        Ok(RuntimeUiSnapshot::from_state(
            state,
            menu,
            elevators,
            gift_pokemon,
            text,
        ))
    }

    fn elevator_snapshots(&self, state: &GameState) -> Vec<RuntimeElevatorSnapshot> {
        let OverworldMemory::Active { map_name, .. } = &state.overworld else {
            return Vec::new();
        };
        self.data
            .maps
            .get(map_name)
            .into_iter()
            .flat_map(|module| module.script_elevators.values())
            .map(|definition| RuntimeElevatorSnapshot::from_definition(map_name, definition))
            .collect()
    }

    fn gift_pokemon_snapshots(&self, state: &GameState) -> Vec<RuntimeGiftPokemonSnapshot> {
        let OverworldMemory::Active { map_name, .. } = &state.overworld else {
            return Vec::new();
        };
        self.data
            .maps
            .get(map_name)
            .into_iter()
            .flat_map(|module| module.gift_pokemon_scripts.iter())
            .map(|gift| RuntimeGiftPokemonSnapshot::from_script(map_name, gift))
            .collect()
    }

    fn active_text_snapshot(&self, state: &GameState) -> Result<Option<RuntimeTextSnapshot>> {
        if !state.script_runtime.text_window_open {
            return Ok(None);
        }
        let Some(label) = state.script_runtime.active_text_label.as_deref() else {
            return Ok(None);
        };
        self.text_snapshot_for_label(state, label)
            .map(Some)
            .with_context(|| format!("resolve active runtime text snapshot for '{label}'"))
    }

    fn resolve_text_decimal_constants(&self, state: &GameState, text: &str) -> Result<String> {
        if !text.contains("{d:") {
            return Ok(text.to_string());
        }
        let constants = &self.data.story_event_script_constants;
        let local = match &state.overworld {
            OverworldMemory::Active { map_name, .. } => constants.maps.get(map_name),
            OverworldMemory::Inactive => None,
        };
        let mut result = String::with_capacity(text.len());
        let mut remaining = text;
        while let Some(start) = remaining.find("{d:") {
            result.push_str(&remaining[..start]);
            let operand = &remaining[start + 3..];
            let end = operand.find('}').context("unterminated decimal text constant")?;
            let name = &operand[..end];
            let value = local.and_then(|values| values.get(name)).copied()
                .or_else(|| constants.global.get(name).copied())
                .or_else(|| self.data.currency_constants.get(name).map(i64::from))
                .or_else(|| match name {
                    "NUM_TMS" => Some(self.data.items.values().filter(|item| {
                        item.tmhm_index.is_some() && item.script_name.starts_with("TM_")
                    }).count() as i64),
                    "BUG_CONTEST_MINUTES" => self.data.bug_contest_config.as_ref().map(|config| i64::from(config.timer_minutes)),
                    "BUG_CONTEST_BALLS" => self.data.bug_contest_config.as_ref().map(|config| i64::from(config.park_balls)),
                    _ => None,
                })
                .with_context(|| format!("missing decimal text constant {name}"))?;
            result.push_str(&(value as i32).to_string());
            remaining = &operand[end + 1..];
        }
        result.push_str(remaining);
        Ok(result)
    }

    fn resolve_text_body_constants(&self, state: &GameState, body: &ScriptTextBody) -> Result<ScriptTextBody> {
        let mut body = body.clone();
        for command in &mut body.commands {
            for argument in &mut command.args {
                if argument.contains("{d:") {
                    *argument = self.resolve_text_decimal_constants(state, argument)?;
                }
            }
        }
        Ok(body)
    }

    fn text_snapshot_for_label(
        &self,
        state: &GameState,
        label: &str,
    ) -> Result<RuntimeTextSnapshot> {
        if let OverworldMemory::Active { map_name, .. } = &state.overworld
            && let Some(module) = self.data.maps.get(map_name)
            && let Some(body) = module.script_text_bodies.get(label)
        {
            return Ok(RuntimeTextSnapshot {
                label: label.to_string(),
                source: RuntimeTextSource::ScriptBody {
                    map_name: map_name.clone(),
                },
                asm_text: None,
                body: Some(self.resolve_text_body_constants(state, body)?),
                queued_text_events: state.script_runtime.text_events.len(),
            });
        }
        if let Some(module) = &self.data.global_scripts
            && let Some(body) = module.script_text_bodies.get(label)
        {
            return Ok(RuntimeTextSnapshot {
                label: label.to_string(),
                source: RuntimeTextSource::ScriptBody {
                    map_name: "GlobalScripts".to_string(),
                },
                asm_text: None,
                body: Some(self.resolve_text_body_constants(state, body)?),
                queued_text_events: state.script_runtime.text_events.len(),
            });
        }
        if let Some(text) = self.data.asm_text.get(label) {
            return Ok(RuntimeTextSnapshot {
                label: label.to_string(),
                source: RuntimeTextSource::AsmText,
                asm_text: Some(self.resolve_text_decimal_constants(state, text)?),
                body: None,
                queued_text_events: state.script_runtime.text_events.len(),
            });
        }
        if matches!(state.overworld, OverworldMemory::Inactive) {
            anyhow::bail!(
                "runtime UI script text label '{label}' requires an active overworld map"
            );
        }
        self.data
            .validate_saved_text_reference("runtime_ui.text.label", label)
            .with_context(|| format!("validate runtime UI text label '{label}'"))?;
        match &state.overworld {
            OverworldMemory::Active { map_name, .. } => anyhow::bail!(
                "runtime UI script text label '{label}' is not declared by current compiled map {map_name}"
            ),
            OverworldMemory::Inactive => anyhow::bail!(
                "runtime UI script text label '{label}' requires an active overworld map"
            ),
        }
    }

    fn bag_snapshot(&self, state: &GameState) -> Result<RuntimeBagSnapshot> {
        Ok(RuntimeBagSnapshot {
            items: RuntimeBagSnapshot::inventory(&state.bag.items),
            balls: RuntimeBagSnapshot::inventory(&state.bag.balls),
            key_items: RuntimeBagSnapshot::inventory(&state.bag.key_items),
            tm_hm: RuntimeBagSnapshot::tm_hm(&self.data.items, &state.bag.tm_hm)?,
            pc_items: RuntimeBagSnapshot::inventory(&state.bag.pc_items),
            custom_pockets: state
                .bag
                .custom_pockets
                .iter()
                .map(|(pocket_id, inventory)| {
                    (pocket_id.clone(), RuntimeBagSnapshot::inventory(inventory))
                })
                .collect(),
        })
    }

    fn item_catalog_snapshot(&self) -> Vec<RuntimeItemCatalogSnapshot> {
        self.data
            .items
            .iter()
            .map(|entry| RuntimeItemCatalogSnapshot::from_item(entry, &self.data.evolutions))
            .collect()
    }

    fn move_catalog_snapshot(&self) -> Vec<RuntimeMoveCatalogSnapshot> {
        self.data
            .moves
            .iter()
            .map(RuntimeMoveCatalogSnapshot::from_move)
            .collect()
    }

    fn pokemon_catalog_snapshot(&self) -> Vec<RuntimePokemonCatalogSnapshot> {
        let mut pokemon = self
            .data
            .pokemon
            .iter()
            .map(RuntimePokemonCatalogSnapshot::from_species)
            .collect::<Vec<_>>();
        pokemon.sort_by_key(|species| species.int_id);
        pokemon
    }

    fn trainer_catalog_snapshot(&self) -> Vec<RuntimeTrainerCatalogSnapshot> {
        self.data
            .trainers
            .trainers
            .values()
            .map(RuntimeTrainerCatalogSnapshot::from_trainer)
            .collect()
    }

    fn base_map_catalog_snapshot(data: &GameDataSet) -> Vec<Arc<RuntimeMapCatalogSnapshot>> {
        data.maps
            .iter()
            .map(|(map_name, module)| {
                let metadata = module
                    .attributes
                    .map_constant
                    .as_deref()
                    .and_then(|constant| data.runtime_map_metadata.get(constant));
                Arc::new(RuntimeMapCatalogSnapshot::from_module(
                    map_name, module, metadata,
                ))
            })
            .collect()
    }

    fn build_static_catalog_cache(&self) -> RuntimeStaticCatalogCache {
        RuntimeStaticCatalogCache {
            audio: Arc::new(self.audio_catalog_snapshot()),
            items: Arc::new(self.item_catalog_snapshot()),
            item_effect_plans: Arc::new(self.item_effect_plan_keys().into_iter().collect()),
            moves: Arc::new(self.move_catalog_snapshot()),
            pokemon: Arc::new(self.pokemon_catalog_snapshot()),
            trainers: Arc::new(self.trainer_catalog_snapshot()),
            spawn_points: Arc::new(self.data.runtime_spawn_points.values().cloned().collect()),
            tilesets: Arc::new(self.tileset_catalog_snapshot()),
            encounters: Arc::new(self.encounter_catalog_snapshot()),
            battle_rules: Arc::new(self.battle_rule_catalog_snapshot()),
            world_rules: Arc::new(self.world_rule_catalog_snapshot()),
            presentation: Arc::new(self.presentation_catalog_snapshot()),
            special: Arc::new(self.special_catalog_snapshot()),
            story: Arc::new(self.story_catalog_snapshot()),
            playability: Arc::new(self.playability_rules_snapshot()),
        }
    }

    fn static_catalog_cache(&self) -> &RuntimeStaticCatalogCache {
        self.catalog_cache
            .get_or_init(|| self.build_static_catalog_cache())
    }

    fn map_catalog_snapshot(
        &self,
        active_map: &crystal_core::world::map::OverworldMapData,
        state: &GameState,
    ) -> Vec<Arc<RuntimeMapCatalogSnapshot>> {
        self.map_catalog
            .iter()
            .map(|base| {
                let map_name = base.map_name.as_str();
                if active_map.name == map_name {
                    // The active OverworldSession carries callback/field-move
                    // block writes.  Rendering immutable pack blocks here
                    // erased those authoritative mutations, including the
                    // default Town Map in the player's upstairs bedroom.
                    let mut snapshot = (**base).clone();
                    snapshot.blocks.clone_from(&active_map.metatile_ids);
                    Arc::new(snapshot)
                } else if let Some(overrides) = state.map_block_overrides.get(map_name) {
                    // A connection can expose a neighboring map before it
                    // becomes the active session. Keep block writes from an
                    // earlier visit visible at that seam. `snapshot()` has
                    // already validated these coordinates against this map.
                    let mut snapshot = (**base).clone();
                    for ((x, y), block_id) in overrides {
                        let index = usize::from(*y) * usize::from(snapshot.attributes.width)
                            + usize::from(*x);
                        snapshot.blocks[index] = *block_id;
                    }
                    Arc::new(snapshot)
                } else {
                    Arc::clone(base)
                }
            })
            .collect()
    }

    fn tileset_catalog_snapshot(&self) -> Vec<RuntimeTilesetCatalogSnapshot> {
        self.data
            .tilesets
            .iter()
            .map(RuntimeTilesetCatalogSnapshot::from_tileset)
            .collect()
    }

    fn encounter_catalog_snapshot(&self) -> RuntimeEncounterCatalogSnapshot {
        RuntimeEncounterCatalogSnapshot::from_data(&self.data)
    }

    fn battle_rule_catalog_snapshot(&self) -> RuntimeBattleRuleCatalogSnapshot {
        RuntimeBattleRuleCatalogSnapshot::from_data(&self.data)
    }

    fn world_rule_catalog_snapshot(&self) -> RuntimeWorldRuleCatalogSnapshot {
        RuntimeWorldRuleCatalogSnapshot::from_data(&self.data)
    }

    fn presentation_catalog_snapshot(&self) -> RuntimePresentationCatalogSnapshot {
        RuntimePresentationCatalogSnapshot::from_data(&self.data)
    }

    fn special_catalog_snapshot(&self) -> RuntimeSpecialCatalogSnapshot {
        RuntimeSpecialCatalogSnapshot::from_data(&self.data)
    }

    fn story_catalog_snapshot(&self) -> RuntimeStoryCatalogSnapshot {
        RuntimeStoryCatalogSnapshot::from_data(&self.data)
    }

    fn audio_catalog_snapshot(&self) -> RuntimeAudioCatalogSnapshot {
        RuntimeAudioCatalogSnapshot::from_catalog(&self.audio)
    }

    fn playability_rules_snapshot(&self) -> crystal_assets::PlayabilityRules {
        self.data.playability.clone()
    }

    /// Validate a candidate save against this runtime before a host imports it.
    pub fn validate_save_state_for_runtime_pack(&self, state: &GameState) -> Result<()> {
        self.data.validate_save_currency(state)?;
        validate_save_references_for_runtime_pack(state, &self.data)
            .context("validate Crystal runtime save references against compiled pack")
    }

    pub fn boot_summary(&self) -> RuntimeBootSummary {
        RuntimeBootSummary {
            modpack_id: self.modpack.id().to_string(),
            modpack_hash: self.modpack.hash().to_string(),
            pack_content_hash: self.pack_identity.content_hash.clone(),
            pokemon_species: self.data.pokemon.len(),
            moves: self.data.moves.len(),
            maps: self.data.maps.len(),
            items: self.data.items.len(),
            wild_encounter_tables: self.data.wild_encounters.len(),
            music_tracks: self.audio.music_count(),
            sound_effects: self.audio.sound_effect_count(),
            cries: self.audio.cry_count(),
            viewport: self.viewport,
        }
    }

    fn start_overworld_session(
        &self,
        asset_root: &AssetRoot,
        spawn_identifier: u16,
    ) -> Result<RuntimeOverworldSession> {
        let spawn = self.data.runtime_spawn_point(spawn_identifier)?;
        RuntimeOverworldSession::new(self, asset_root, spawn)
    }

    #[cfg(any(test, feature = "test-fixtures", feature = "location-tester"))]
    pub fn start_overworld_session_at_runtime_tile(
        &self,
        asset_root: &AssetRoot,
        map_name: &str,
        tile_x: i16,
        tile_y: i16,
    ) -> Result<RuntimeOverworldSession> {
        RuntimeOverworldSession::new_at_runtime_tile(
            self,
            asset_root,
            map_name,
            TilePosition::new(tile_x, tile_y),
        )
    }

    pub fn resume_overworld_session(
        &self,
        asset_root: &AssetRoot,
        state: GameState,
    ) -> Result<RuntimeOverworldSession> {
        RuntimeOverworldSession::from_state(self, asset_root, state)
    }
}

fn require_runtime_catalog_id(kind: &str, id: &str, exists: bool) -> Result<()> {
    if exists {
        Ok(())
    } else {
        anyhow::bail!("compiled game pack missing exact {kind} id {id}")
    }
}

fn field_move_move_ids(field_moves: &FieldMoveCatalog) -> BTreeSet<String> {
    [
        field_moves.cut.move_id.as_str(),
        field_moves.whirlpool.move_id.as_str(),
        field_moves.strength.move_id.as_str(),
        field_moves.flash.move_id.as_str(),
        field_moves.surf.move_id.as_str(),
        field_moves.waterfall.move_id.as_str(),
        field_moves.fly.move_id.as_str(),
        field_moves.dig.move_id.as_str(),
        field_moves.teleport.move_id.as_str(),
        field_moves.headbutt.move_id.as_str(),
        field_moves.rock_smash.move_id.as_str(),
        field_moves.sweet_scent.move_id.as_str(),
    ]
    .into_iter()
    .filter(|move_id| !move_id.is_empty())
    .map(str::to_string)
    .collect()
}

fn field_move_item_ids(field_moves: &FieldMoveCatalog) -> BTreeSet<String> {
    [
        field_moves.escape_rope.item_id.as_str(),
        field_moves.bicycle.item_id.as_str(),
        field_moves.itemfinder.item_id.as_str(),
        field_moves.squirtbottle.item_id.as_str(),
        field_moves.coin_case.item_id.as_str(),
        field_moves.blue_card.item_id.as_str(),
        field_moves.town_map.item_id.as_str(),
        field_moves.pokegear.item_id.as_str(),
    ]
    .into_iter()
    .filter(|item_id| !item_id.is_empty())
    .map(str::to_string)
    .collect()
}

fn fly_destination_keys(
    destinations: &BTreeMap<String, crystal_assets::FlyDestination>,
) -> BTreeSet<RuntimeFlyDestinationKey> {
    destinations
        .values()
        .map(|destination| RuntimeFlyDestinationKey {
            flypoint_flag: destination.flypoint_flag.clone(),
            destination_spawn_identifier: destination.destination_spawn_identifier,
            label: destination.label.clone(),
        })
        .collect()
}

fn field_move_rule_keys(field_moves: &FieldMoveCatalog) -> BTreeSet<RuntimeFieldMoveRuleKey> {
    [
        field_move_block_rule_key("cut", &field_moves.cut),
        field_move_block_rule_key("whirlpool", &field_moves.whirlpool),
        field_move_flag_rule_key("strength", &field_moves.strength),
        field_move_flag_rule_key("flash", &field_moves.flash),
        field_move_travel_rule_key("surf", &field_moves.surf),
        field_move_travel_rule_key("waterfall", &field_moves.waterfall),
        field_move_badged_rule_key("fly", &field_moves.fly),
        field_move_move_rule_key("dig", &field_moves.dig),
        field_move_move_rule_key("teleport", &field_moves.teleport),
        field_move_move_rule_key("headbutt", &field_moves.headbutt),
        field_move_move_rule_key("rock_smash", &field_moves.rock_smash),
        field_move_move_rule_key("sweet_scent", &field_moves.sweet_scent),
        field_escape_item_rule_key("escape_rope", &field_moves.escape_rope),
        RuntimeFieldMoveRuleKey {
            rule_id: "repel".to_string(),
            rule_kind: "repel_item".to_string(),
            move_id: None,
            item_id: None,
            badge_region: None,
            badge_index: None,
            engine_flag: None,
            escape_rope_mode: None,
            target_collisions: Vec::new(),
            blocked_collisions: Vec::new(),
            replacements: BTreeMap::new(),
        },
        field_item_rule_key("bicycle", &field_moves.bicycle),
        field_item_rule_key("itemfinder", &field_moves.itemfinder),
        field_item_rule_key("squirtbottle", &field_moves.squirtbottle),
        field_item_rule_key(
            "card_key",
            &FieldItemRule {
                item_id: field_moves.card_key.item_id.clone(),
            },
        ),
        field_item_rule_key(
            "basement_key",
            &FieldItemRule {
                item_id: field_moves.basement_key.item_id.clone(),
            },
        ),
        field_item_rule_key("coin_case", &field_moves.coin_case),
        field_item_rule_key("blue_card", &field_moves.blue_card),
        field_item_rule_key("town_map", &field_moves.town_map),
        field_item_rule_key("pokegear", &field_moves.pokegear),
    ]
    .into_iter()
    .collect()
}

fn field_move_badge_parts(badge: &FieldMoveBadgeRequirement) -> (Option<String>, Option<usize>) {
    (Some(badge.region.clone()), Some(badge.index))
}

fn field_move_replacement_keys(
    replacements: &BTreeMap<String, BTreeMap<u16, FieldMoveReplacement>>,
) -> BTreeMap<String, BTreeMap<u16, RuntimeFieldMoveReplacementKey>> {
    replacements
        .iter()
        .map(|(tileset_id, replacements)| {
            (
                tileset_id.clone(),
                replacements
                    .iter()
                    .map(|(block_id, replacement)| {
                        (
                            *block_id,
                            RuntimeFieldMoveReplacementKey {
                                replacement_block_id: replacement.replacement_block_id,
                                variant: replacement.variant.clone(),
                            },
                        )
                    })
                    .collect(),
            )
        })
        .collect()
}

fn field_move_block_rule_key(rule_id: &str, rule: &FieldMoveBlockRule) -> RuntimeFieldMoveRuleKey {
    let (badge_region, badge_index) = field_move_badge_parts(&rule.badge);
    RuntimeFieldMoveRuleKey {
        rule_id: rule_id.to_string(),
        rule_kind: "block".to_string(),
        move_id: Some(rule.move_id.clone()),
        item_id: None,
        badge_region,
        badge_index,
        engine_flag: None,
        escape_rope_mode: None,
        target_collisions: rule.target_collisions.clone(),
        blocked_collisions: Vec::new(),
        replacements: field_move_replacement_keys(&rule.replacements),
    }
}

fn field_move_flag_rule_key(rule_id: &str, rule: &FieldMoveFlagRule) -> RuntimeFieldMoveRuleKey {
    let (badge_region, badge_index) = field_move_badge_parts(&rule.badge);
    RuntimeFieldMoveRuleKey {
        rule_id: rule_id.to_string(),
        rule_kind: "flag".to_string(),
        move_id: Some(rule.move_id.clone()),
        item_id: None,
        badge_region,
        badge_index,
        engine_flag: Some(rule.engine_flag.clone()),
        escape_rope_mode: None,
        target_collisions: Vec::new(),
        blocked_collisions: Vec::new(),
        replacements: BTreeMap::new(),
    }
}

fn field_move_travel_rule_key(
    rule_id: &str,
    rule: &FieldMoveTravelRule,
) -> RuntimeFieldMoveRuleKey {
    let (badge_region, badge_index) = field_move_badge_parts(&rule.badge);
    RuntimeFieldMoveRuleKey {
        rule_id: rule_id.to_string(),
        rule_kind: "travel".to_string(),
        move_id: Some(rule.move_id.clone()),
        item_id: None,
        badge_region,
        badge_index,
        engine_flag: None,
        escape_rope_mode: None,
        target_collisions: rule.target_collisions.clone(),
        blocked_collisions: rule.blocked_collisions.clone(),
        replacements: BTreeMap::new(),
    }
}

fn field_move_badged_rule_key(rule_id: &str, rule: &FieldMoveRule) -> RuntimeFieldMoveRuleKey {
    let (badge_region, badge_index) = field_move_badge_parts(&rule.badge);
    RuntimeFieldMoveRuleKey {
        rule_id: rule_id.to_string(),
        rule_kind: "badged_move".to_string(),
        move_id: Some(rule.move_id.clone()),
        item_id: None,
        badge_region,
        badge_index,
        engine_flag: None,
        escape_rope_mode: None,
        target_collisions: Vec::new(),
        blocked_collisions: Vec::new(),
        replacements: BTreeMap::new(),
    }
}

fn field_move_move_rule_key(rule_id: &str, rule: &FieldMoveMoveRule) -> RuntimeFieldMoveRuleKey {
    RuntimeFieldMoveRuleKey {
        rule_id: rule_id.to_string(),
        rule_kind: "move".to_string(),
        move_id: Some(rule.move_id.clone()),
        item_id: None,
        badge_region: None,
        badge_index: None,
        engine_flag: None,
        escape_rope_mode: None,
        target_collisions: rule.target_collisions.clone(),
        blocked_collisions: Vec::new(),
        replacements: BTreeMap::new(),
    }
}

fn field_escape_item_rule_key(
    rule_id: &str,
    rule: &FieldEscapeItemRule,
) -> RuntimeFieldMoveRuleKey {
    RuntimeFieldMoveRuleKey {
        rule_id: rule_id.to_string(),
        rule_kind: "escape_item".to_string(),
        move_id: None,
        item_id: Some(rule.item_id.clone()),
        badge_region: None,
        badge_index: None,
        engine_flag: None,
        escape_rope_mode: Some(rule.escape_rope_mode.clone()),
        target_collisions: Vec::new(),
        blocked_collisions: Vec::new(),
        replacements: BTreeMap::new(),
    }
}

fn field_item_rule_key(rule_id: &str, rule: &FieldItemRule) -> RuntimeFieldMoveRuleKey {
    RuntimeFieldMoveRuleKey {
        rule_id: rule_id.to_string(),
        rule_kind: "item".to_string(),
        move_id: None,
        item_id: Some(rule.item_id.clone()),
        badge_region: None,
        badge_index: None,
        engine_flag: None,
        escape_rope_mode: None,
        target_collisions: Vec::new(),
        blocked_collisions: Vec::new(),
        replacements: BTreeMap::new(),
    }
}

fn collect_wild_encounter_keys(
    map_name: &str,
    encounters: &WildEncounterData,
    keys: &mut BTreeSet<RuntimeWildEncounterOriginKey>,
) {
    if let Some(table) = &encounters.grass {
        for encounter in table
            .morning
            .iter()
            .chain(table.day.iter())
            .chain(table.night.iter())
        {
            keys.insert(RuntimeWildEncounterOriginKey {
                map_name: map_name.to_string(),
                species: encounter.species.clone(),
                level: encounter.level,
            });
        }
    }
    if let Some(table) = &encounters.water {
        for encounter in table
            .morning
            .iter()
            .chain(table.day.iter())
            .chain(table.night.iter())
        {
            keys.insert(RuntimeWildEncounterOriginKey {
                map_name: map_name.to_string(),
                species: encounter.species.clone(),
                level: encounter.level,
            });
        }
    }
}

fn collect_field_encounter_keys(
    map_name: &str,
    encounters: &FieldEncounterData,
    keys: &mut BTreeSet<RuntimeWildEncounterOriginKey>,
) {
    for table in encounters.tables.values() {
        for encounter in table.common.iter().chain(table.rare.iter()) {
            keys.insert(RuntimeWildEncounterOriginKey {
                map_name: map_name.to_string(),
                species: encounter.species.clone(),
                level: encounter.level,
            });
        }
    }
}

fn collect_fishing_encounter_keys(
    map_name: &str,
    group: &crystal_core::world::fishing::FishingGroup,
    time_groups: &BTreeMap<String, crystal_core::world::fishing::TimeFishEntry>,
    keys: &mut BTreeSet<RuntimeWildEncounterOriginKey>,
) {
    for slot in group
        .rod_tables
        .values()
        .flat_map(|table| table.slots.iter())
    {
        if let Some(species) = &slot.species {
            keys.insert(RuntimeWildEncounterOriginKey {
                map_name: map_name.to_string(),
                species: species.clone(),
                level: slot.level,
            });
        }
        if let Some(time_group) = slot
            .time_group
            .as_ref()
            .and_then(|time_group| time_groups.get(time_group))
        {
            keys.insert(RuntimeWildEncounterOriginKey {
                map_name: map_name.to_string(),
                species: time_group.day_species.clone(),
                level: time_group.day_level,
            });
            keys.insert(RuntimeWildEncounterOriginKey {
                map_name: map_name.to_string(),
                species: time_group.night_species.clone(),
                level: time_group.night_level,
            });
        }
    }
}

