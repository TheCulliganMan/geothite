fn visible_start_menu_entries(runtime_shell: &BevyRuntimeShell) -> Result<Vec<String>> {
    let snapshot = runtime_shell.shell.snapshot()?;
    let options = visible_start_menu_options(runtime_shell, &snapshot);
    let selected = strict_readonly_cursor_index(
        &runtime_shell.start_menu_cursor,
        START_MENU_SURFACE_ID,
        options.len(),
    )
    .context("start menu is open without a valid cursor")?;
    Ok(options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            let marker = if index == selected { ">" } else { " " };
            format!(
                "{marker}{}",
                start_menu_option_display_label(*option, &snapshot)
            )
        })
        .collect())
}

fn visible_field_pack_entries(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
) -> Result<Vec<String>> {
    if let Some(cursor) = &runtime_shell.tmhm_decision_prompt_cursor {
        let selected = strict_readonly_cursor_index(
            &Some(cursor.clone()),
            "pack:tmhm:decision",
            2,
        )
        .context("TM/HM decision prompt has no valid cursor")?;
        let party = snapshot
            .party
            .slots
            .get(runtime_shell.party_cursor)
            .map(|slot| slot.pokemon.nickname.as_str())
            .context("TM/HM decision prompt has no selected party Pokemon")?;
        let prompt = match runtime_shell.tmhm_decision {
            Some(VisibleTmHmDecision::ForgetMove) => {
                format!("DELETE A MOVE FOR {party}?")
            }
            Some(VisibleTmHmDecision::StopLearning) => "STOP LEARNING THIS MOVE?".to_string(),
            None => anyhow::bail!("TM/HM decision prompt has no decision kind"),
        };
        return Ok(vec![
            compact_scene_label(&prompt, 30),
            format!("{}YES", if selected == 0 { ">" } else { " " }),
            format!("{}NO", if selected == 1 { ">" } else { " " }),
        ]);
    }
    if let Some(cursor) = &runtime_shell.tmhm_teach_prompt_cursor {
        let selected = strict_readonly_cursor_index(
            &Some(cursor.clone()),
            "pack:tmhm:teach-prompt",
            2,
        )
        .context("TM/HM teach prompt has no valid cursor")?;
        let active = FieldPackPocket::TmHm;
        let selected_tmhm = strict_readonly_cursor_index(
            &runtime_shell.tmhm_cursor,
            "bag:tmhm",
            field_pack_selectable_count(snapshot.bag.tm_hm.len()),
        )
        .context("TM/HM teach prompt has no selected TM/HM")?;
        let tmhm = snapshot
            .bag
            .tm_hm
            .get(selected_tmhm)
            .context("TM/HM teach prompt points at CANCEL instead of an item")?;
        let move_id = tmhm
            .move_id
            .as_deref()
            .context("selected TM/HM has no move")?;
        required_selected_field_pack_item_label(snapshot, runtime_shell, &active)?;
        let move_name = battle_move_display_name(snapshot, move_id);
        return Ok(vec![
            compact_scene_label(&format!("TEACH {move_name}?"), 30),
            format!("{}YES", if selected == 0 { ">" } else { " " }),
            format!("{}NO", if selected == 1 { ">" } else { " " }),
        ]);
    }
    if let Some(mode) = runtime_shell.field_pack_target_mode {
        return visible_field_pack_target_entries(snapshot, runtime_shell, mode);
    }
    let active = active_visible_field_pack_pocket(runtime_shell);
    if runtime_shell.field_pack_action_cursor.is_some() {
        return Ok(visible_field_pack_action_entries(snapshot, runtime_shell, &active)?
            .into_iter()
            .take(SCENE_MENU_VISIBLE_ROWS)
            .collect());
    }
    let mut entries = vec![format!(
        "POCKET: {}",
        field_pack_pocket_label(&active).to_uppercase()
    )];
    entries.extend(match active {
        FieldPackPocket::Items => required_selected_pack_entries(
            snapshot,
            &snapshot.bag.items,
            &runtime_shell.bag_cursor,
            "bag:items",
        )?,
        FieldPackPocket::Balls => required_selected_pack_entries(
            snapshot,
            &snapshot.bag.balls,
            &runtime_shell.ball_cursor,
            "bag:balls",
        )?,
        FieldPackPocket::KeyItems => required_selected_pack_entries(
            snapshot,
            &snapshot.bag.key_items,
            &runtime_shell.key_item_cursor,
            "bag:key-items",
        )?,
        FieldPackPocket::TmHm => required_selected_tmhm_pack_entries(snapshot, runtime_shell)?,
        FieldPackPocket::Custom(pocket_id) => {
            let items = snapshot
                .bag
                .custom_pockets
                .get(&pocket_id)
                .with_context(|| format!("active custom Pack pocket {pocket_id} is missing"))?;
            required_selected_pack_entries(
                snapshot,
                items,
                &runtime_shell.custom_item_cursor,
                &custom_pack_surface_id(&pocket_id),
            )?
        }
    });
    Ok(entries.into_iter().take(SCENE_MENU_VISIBLE_ROWS).collect())
}

fn visible_field_pack_action_entries(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
    pocket: &FieldPackPocket,
) -> Result<Vec<String>> {
    let actions = visible_selected_pack_item_actions(
        snapshot,
        runtime_shell,
        pocket,
        snapshot.battle.is_some(),
    )?;
    let selected = strict_readonly_cursor_index(
        &runtime_shell.field_pack_action_cursor,
        "pack:actions",
        actions.len(),
    )
    .context("field Pack action menu has no valid cursor")?;
    let label = required_selected_field_pack_item_label(snapshot, runtime_shell, pocket)?;
    let mut entries = vec![compact_scene_label(&format!("ACTION {label}"), 30)];
    entries.extend(actions.iter().enumerate().map(|(index, action)| {
        let marker = if index == selected { ">" } else { " " };
        format!("{marker}{}", visible_field_pack_action_label(*action))
    }));
    Ok(entries)
}

fn visible_field_pack_target_entries(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
    mode: FieldPackTargetMode,
) -> Result<Vec<String>> {
    anyhow::ensure!(
        runtime_shell.party_cursor < snapshot.party.slots.len(),
        "field Pack target party cursor {} is outside {} party slots",
        runtime_shell.party_cursor,
        snapshot.party.slots.len()
    );
    let selected_party = runtime_shell.party_cursor;
    let label = required_selected_field_pack_item_label(
        snapshot,
        runtime_shell,
        &active_visible_field_pack_pocket(runtime_shell),
    )?;
    let mut entries = vec![compact_scene_label(&format!("ITEM {label}"), 30)];
    if mode == FieldPackTargetMode::PartyMove {
        let slot = snapshot
            .party
            .slots
            .get(selected_party)
            .context("field Pack move target party slot is missing")?;
        entries.push(party_slot_entry(snapshot, slot, true));
        let selected_move = strict_readonly_cursor_index(
            &runtime_shell.party_move_cursor,
            &party_move_cursor_surface_id(slot.index),
            slot.pokemon.moves.len(),
        )
        .context("field Pack move target has no valid move cursor")?;
        entries.extend(windowed_move_entries(
            snapshot,
            &slot.pokemon.moves,
            selected_move,
        )?);
        return Ok(entries.into_iter().take(SCENE_MENU_VISIBLE_ROWS).collect());
    }
    if mode == FieldPackTargetMode::TmHmPokemon && runtime_shell.tmhm_forget_menu_open {
        let slot = snapshot
            .party
            .slots
            .get(selected_party)
            .context("TM/HM forget target party slot is missing")?;
        let row_count = slot.pokemon.moves.len() + 1;
        let selected_move = strict_readonly_cursor_index(
            &runtime_shell.party_move_cursor,
            &party_move_cursor_surface_id(slot.index),
            row_count,
        )
        .context("TM/HM forget menu has no valid move-or-CANCEL cursor")?;
        entries.push(compact_scene_label("CHOOSE A MOVE TO FORGET", 30));
        let visible_move_rows = SCENE_MENU_VISIBLE_ROWS.saturating_sub(entries.len());
        let visible_start = visible_window_start(selected_move, row_count, visible_move_rows);
        let visible_end = (visible_start + visible_move_rows).min(row_count);
        entries.extend((visible_start..visible_end).map(|index| {
            if index == slot.pokemon.moves.len() {
                Ok(format!("{}CANCEL", if index == selected_move { ">" } else { " " }))
            } else {
                let marker = if index == selected_move { ">" } else { " " };
                move_menu_entry(snapshot, &slot.pokemon.moves[index], marker)
            }
        }).collect::<Result<Vec<_>>>()?);
        return Ok(entries.into_iter().take(SCENE_MENU_VISIBLE_ROWS).collect());
    }
    if mode == FieldPackTargetMode::TmHmPokemon {
        let selected_tmhm = strict_readonly_cursor_index(
            &runtime_shell.tmhm_cursor,
            "bag:tmhm",
            field_pack_selectable_count(snapshot.bag.tm_hm.len()),
        )
        .context("TM/HM party target has no selected TM/HM")?;
        let item_id = snapshot
            .bag
            .tm_hm
            .get(selected_tmhm)
            .map(|tmhm| tmhm.item_id.as_str())
            .context("TM/HM party target points at CANCEL instead of an item")?;
        entries.extend(
            windowed_index_range(selected_party, snapshot.party.slots.len()).map(|index| {
                let slot = &snapshot.party.slots[index];
                let is_egg = slot.pokemon.is_egg || slot.pokemon.species.id == "EGG";
                let able = !is_egg
                    && match runtime_shell
                        .shell
                        .preview_tmhm_on_party_pokemon(item_id, slot.index, None)
                        {
                            Ok(_) => true,
                            Err(error) => matches!(
                                error.downcast_ref::<TmHmLearnError>(),
                                Some(
                                    TmHmLearnError::MoveListFull
                                        | TmHmLearnError::AlreadyKnows { .. }
                                )
                            ),
                        };
                compact_scene_label(
                    &format!(
                        "{} {}",
                        party_slot_entry(snapshot, slot, index == selected_party),
                        if able { "ABLE" } else { "NOT ABLE" }
                    ),
                    30,
                )
            }),
        );
        return Ok(entries.into_iter().take(SCENE_MENU_VISIBLE_ROWS).collect());
    }
    if mode == FieldPackTargetMode::PartyPokemon {
        let item_id = selected_field_pack_item_id_from_snapshot(
            snapshot,
            runtime_shell,
            &active_visible_field_pack_pocket(runtime_shell),
        )
        .context("field Pack party target has no selected item")?;
        let is_evolution_item = snapshot.item_effect_plans.iter().any(|plan| {
                plan.item_id == item_id && plan.behavior_id == ITEM_EFFECT_BEHAVIOR_EVOLUTION_STONE
            });
        if is_evolution_item {
            entries.extend(
                windowed_index_range(selected_party, snapshot.party.slots.len()).map(|index| {
                    let slot = &snapshot.party.slots[index];
                    let able = runtime_shell
                        .shell
                        .preview_party_item_on_pokemon(&item_id, slot.index)
                        .is_ok_and(|outcome| outcome.evolution_target.is_some());
                    compact_scene_label(
                        &format!(
                            "{} {}",
                            party_slot_entry(snapshot, slot, index == selected_party),
                            if able { "ABLE" } else { "NOT ABLE" }
                        ),
                        30,
                    )
                }),
            );
            return Ok(entries.into_iter().take(SCENE_MENU_VISIBLE_ROWS).collect());
        }
    }
    entries.extend(windowed_party_slot_entries(snapshot, selected_party));
    Ok(entries.into_iter().take(SCENE_MENU_VISIBLE_ROWS).collect())
}

fn visible_party_menu_entries(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
) -> Result<Vec<String>> {
    let row_count = normal_visible_party_menu_row_count(snapshot);
    anyhow::ensure!(
        runtime_shell.party_cursor < row_count,
        "party cursor {} is outside {row_count} Pokemon/CANCEL rows",
        runtime_shell.party_cursor
    );
    let selected_party_slot = runtime_shell.party_cursor;
    if runtime_shell.party_move_reorder_open {
        let slot = snapshot
            .party
            .slots
            .get(selected_party_slot)
            .context("move-reorder party cursor points at CANCEL")?;
        let move_count = slot.pokemon.moves.len();
        let selected = strict_readonly_cursor_index(
            &runtime_shell.party_move_cursor,
            &party_move_reorder_surface_id(slot.index),
            move_count,
        )
        .context("party move-reorder screen has no valid move cursor")?;
        if let Some(origin) = runtime_shell.party_move_reorder_origin {
            anyhow::ensure!(
                origin < move_count,
                "party move-reorder origin {origin} is outside {move_count} moves"
            );
        }
        let mut entries = vec![compact_scene_label(
            &format!(
                "{} \u{e10a}{} MOVE",
                slot.pokemon.nickname, slot.pokemon.level
            ),
            30,
        )];
        entries.extend(windowed_index_range(selected, move_count).map(|index| {
            let marker = if runtime_shell.party_move_reorder_origin == Some(index) {
                "\u{25b7}"
            } else if index == selected {
                ">"
            } else {
                " "
            };
            move_menu_entry(snapshot, &slot.pokemon.moves[index], marker)
        }).collect::<Result<Vec<_>>>()?);
        return Ok(entries.into_iter().take(SCENE_MENU_VISIBLE_ROWS).collect());
    }
    if let Some(give_take_cursor) = &runtime_shell.party_give_take_cursor {
        if give_take_cursor.surface_id == "party:mail-actions" {
            let selected = strict_readonly_cursor_index(
                &Some(give_take_cursor.clone()),
                "party:mail-actions",
                3,
            )
            .context("party Mail action menu has no valid cursor")?;
            let slot = snapshot
                .party
                .slots
                .get(selected_party_slot)
                .context("party Mail action cursor points at CANCEL")?;
            let mut entries = vec![party_slot_entry(
                snapshot,
                slot,
                true,
            )];
            entries.extend(
                ["READ", "TAKE", "QUIT"]
                    .iter()
                    .enumerate()
                    .map(|(index, label)| {
                        format!("{}{label}", if selected == index { ">" } else { " " })
                    }),
            );
            return Ok(entries);
        }
        let selected = strict_readonly_cursor_index(
            &Some(give_take_cursor.clone()),
            "party:give-take",
            2,
        )
        .context("party give/take menu has no valid cursor")?;
        let slot = snapshot
            .party
            .slots
            .get(selected_party_slot)
            .context("party give/take cursor points at CANCEL")?;
        return Ok(vec![
            party_slot_entry(snapshot, slot, true),
            format!("{}GIVE", if selected == 0 { ">" } else { " " }),
            format!("{}TAKE", if selected == 1 { ">" } else { " " }),
        ]);
    }
    if let Some(switch_cursor) = &runtime_shell.party_switch_cursor {
        let source_slot = snapshot
            .party
            .slots
            .get(selected_party_slot)
            .context("party switch source cursor points at CANCEL")?;
        let selected_target = strict_readonly_cursor_index(
            &Some(switch_cursor.clone()),
            &party_switch_cursor_surface_id(source_slot.index),
            snapshot.party.slots.len(),
        )
        .context("party switch screen has no valid target cursor")?;
        return Ok(windowed_index_range(selected_target, snapshot.party.slots.len())
            .map(|index| {
                let slot = &snapshot.party.slots[index];
                party_switch_slot_entry(
                    snapshot,
                    slot,
                    index == selected_target,
                    index == selected_party_slot,
                )
            })
            .collect());
    }
    if runtime_shell.party_summary_open {
        return visible_party_summary_entries(snapshot, runtime_shell);
    }
    if let Some(action_cursor) = &runtime_shell.party_action_cursor {
        let actions = visible_party_actions(snapshot, runtime_shell)
            .context("party action menu has no authoritative actions")?;
        let selected_action = strict_readonly_cursor_index(
            &Some(action_cursor.clone()),
            "party:actions",
            actions.len(),
        )
        .context("party action menu has no valid cursor")?;
        anyhow::ensure!(
            selected_party_slot < snapshot.party.slots.len(),
            "party action cursor points at CANCEL"
        );
        let selected_row = runtime_shell.party_cursor;
        let mut entries = windowed_index_range(selected_row, row_count)
            .map(|index| {
                if index >= snapshot.party.slots.len() {
                    party_cancel_entry(index == selected_row)
                } else {
                    let slot = &snapshot.party.slots[index];
                    party_slot_entry(snapshot, slot, index == selected_row)
                }
            })
            .collect::<Vec<_>>();
        entries.push("SUBMENU:".to_string());
        let visible_action_rows = SCENE_MENU_VISIBLE_ROWS.saturating_sub(entries.len());
        let action_window_start =
            visible_window_start(selected_action, actions.len(), visible_action_rows);
        let action_window_end = (action_window_start + visible_action_rows).min(actions.len());
        entries.extend((action_window_start..action_window_end).map(|index| {
            let action = actions[index];
            let marker = if index == selected_action { ">" } else { " " };
            party_submenu_action_entry(action, marker)
        }));
        return Ok(entries.into_iter().take(SCENE_MENU_VISIBLE_ROWS).collect());
    }
    if runtime_shell.fly_cursor.is_some() {
        return visible_fly_destination_entries(snapshot, runtime_shell);
    }
    let selected_row = runtime_shell.party_cursor;
    Ok(windowed_index_range(selected_row, row_count)
        .map(|index| {
            if index >= snapshot.party.slots.len() {
                party_cancel_entry(index == selected_row)
            } else {
                let slot = &snapshot.party.slots[index];
                party_slot_entry(snapshot, slot, index == selected_row)
            }
        })
        .collect())
}

fn visible_party_summary_entries(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
) -> Result<Vec<String>> {
    let selected = runtime_shell.party_cursor;
    let slot = snapshot
        .party
        .slots
        .get(selected)
        .context("party summary cursor does not select a Pokemon")?;
    visible_stats_screen_entries(snapshot, runtime_shell, &slot.pokemon, runtime_shell.party_summary_page)
}

fn visible_fly_destination_entries(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
) -> Result<Vec<String>> {
    let destinations = active_fly_destinations(snapshot, &runtime_shell.shell)?;
    let selected = strict_readonly_cursor_index(
        &runtime_shell.fly_cursor,
        "fly:destinations",
        destinations.len(),
    )
    .context("Fly destination menu has no valid cursor")?;
    Ok(destinations
        .iter()
        .enumerate()
        .skip(visible_window_start(
            selected,
            destinations.len(),
            SCENE_MENU_VISIBLE_ROWS,
        ))
        .take(SCENE_MENU_VISIBLE_ROWS)
        .map(|(index, destination)| {
            let marker = if index == selected { ">" } else { " " };
            let label = fly_destination_label(destination);
            compact_scene_label(&format!("{marker}{label}"), SCENE_DIALOG_TEXT_CHARS)
        })
        .collect())
}

fn visible_pokedex_menu_entries(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
) -> Result<Vec<String>> {
    anyhow::ensure!(
        !snapshot.pokemon.is_empty(),
        "compiled pack has no Pokemon species"
    );
    if runtime_shell.pokedex_controls.printer_open {
        return Ok(vec![
            "Printer Error 2".into(),
            "Check the Game Boy".into(),
            "Printer Manual.".into(),
            "Press B to Cancel".into(),
        ]);
    }
    if let Some(kanto) = runtime_shell.pokedex_controls.area_region {
        let species = selected_pokedex_catalog_species(snapshot, runtime_shell.pokedex_cursor)?;
        let mut entries = vec![
            format!(
                "{}'S NEST",
                crate::core::models::pokemon_species_display_name(&species.species_id)
            ),
            if kanto { "KANTO" } else { "JOHTO" }.into(),
        ];
        entries.extend(
            visible_pokedex_nests(runtime_shell, snapshot, kanto)?
                .into_iter()
                .map(|place| place.name),
        );
        return Ok(entries);
    }
    if let Some(cursor) = runtime_shell.pokedex_controls.unown_cursor {
        return runtime_shell
            .shell
            .session()
            .state()
            .pokedex
            .unown_letters
            .iter()
            .enumerate()
            .map(|(index, &letter)| {
                let word = POKEDEX_UNOWN_WORDS
                    .get(usize::from(letter).wrapping_sub(1))
                    .context("invalid caught Unown letter")?;
                Ok(format!(
                    "{}{} {}",
                    if cursor == index { ">" } else { " " },
                    char::from(b'A' + letter - 1),
                    word
                ))
            })
            .collect();
    }
    if let Some(entries) =
        pokedex_option_entries(runtime_shell).or_else(|| pokedex_search_entries(runtime_shell))
    {
        return Ok(entries);
    }
    if runtime_shell.pokedex_detail_open {
        return visible_pokedex_detail_entries(snapshot, runtime_shell);
    }
    anyhow::ensure!(
        runtime_shell.pokedex_cursor < snapshot.pokemon.len(),
        "Pokedex cursor {} is out of range for {} species",
        runtime_shell.pokedex_cursor,
        snapshot.pokemon.len()
    );
    let selected = runtime_shell.pokedex_cursor;
    let order = visible_pokedex_listing(snapshot, runtime_shell);
    Ok(order
        .into_iter()
        .skip(runtime_shell.pokedex_scroll)
        .take(visible_pokedex_listing_height(runtime_shell))
        .map(|index| {
            let species = &snapshot.pokemon[index];
            let marker = if index == selected { ">" } else { " " };
            pokedex_entry_row(snapshot, species, marker)
        })
        .collect())
}

fn visible_pokedex_detail_entries(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
) -> Result<Vec<String>> {
    let species = selected_pokedex_catalog_species(snapshot, runtime_shell.pokedex_cursor)?;
    let entry = snapshot
        .presentation
        .pokedex_entries
        .get(&species.species_id)
        .with_context(|| format!("compiled pack missing Pokedex entry {}", species.species_id))?;
    let page = entry
        .pages
        .get(runtime_shell.pokedex_detail_page)
        .with_context(|| {
            format!(
                "Pokedex detail page {} is outside {} pages for {}",
                runtime_shell.pokedex_detail_page,
                entry.pages.len(),
                species.species_id
            )
        })?;
    let caught = runtime_shell.pokedex_scripted_entry
        || snapshot
            .progression
            .pokedex_caught_species
            .contains(&species.species_id);
    let (height, weight) = pokedex_measurements(entry, caught);
    let mut entries = vec![
        format!(
            "No.{:03} {}",
            species.int_id,
            crate::core::models::pokemon_species_display_name(&species.species_id)
        ),
        entry.classification.clone(),
        format!("HT {height} WT {weight}"),
    ];
    if caught {
        entries.extend(wrap_scene_dialog_line(page, SCENE_DIALOG_TEXT_CHARS));
    }
    Ok(entries)
}

fn pokedex_measurements(
    entry: &crate::core::models::RuntimePokedexEntry,
    caught: bool,
) -> (String, String) {
    if !caught {
        return ("?'??\"".to_string(), "???lb".to_string());
    }
    (
        format!(
            "{}'{:02}\"",
            entry.height_digits / 100,
            entry.height_digits % 100
        ),
        format!(
            "{}.{:01}lb",
            entry.weight_digits / 10,
            entry.weight_digits % 10
        ),
    )
}

fn visible_pokegear_menu_entries(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
) -> Result<Vec<String>> {
    if let Some(delay) = runtime_shell.pokegear_map_radio_delay {
        if delay == 0 { return visible_radio_observation_rows(runtime_shell); }
        let station = runtime_shell.pokegear_radio_station.as_deref().context("furniture radio has no selected station")?;
        return Ok(vec![format!("“{}”", visible_radio_station_name(station, snapshot.progression.active_engine_flags.contains("ENGINE_ROCKETS_IN_RADIO_TOWER"))?)]);
    }
    if runtime_shell.pokegear_page == PokegearPage::Clock {
        let time = &snapshot.progression.time;
        // TextCommand_DAY prints the complete weekday; it does not append
        // the encounter time-of-day period.
        const DAY_NAMES: [&str; 7] = [
            "SUNDAY",
            "MONDAY",
            "TUESDAY",
            "WEDNESDAY",
            "THURSDAY",
            "FRIDAY",
            "SATURDAY",
        ];
        let day = DAY_NAMES[usize::from(time.day_of_week % 7)];
        let hour_24 = time.registers.hours;
        let hour_12 = match hour_24 % 12 {
            0 => 12,
            hour => hour,
        };
        let meridiem = if hour_24 < 12 { "AM" } else { "PM" };
        return Ok(vec![
            day.to_string(),
            format!("{hour_12:>2}:{:02} {meridiem}", time.registers.minutes),
        ]);
    }
    if runtime_shell.pokegear_page == PokegearPage::Radio {
        let Some(station) = runtime_shell.pokegear_radio_station.as_deref() else { return Ok(Vec::new()); };
        let broadcast = runtime_shell.pokegear_radio_broadcast.as_ref().context("radio station has no live broadcast")?;
        let heading = if let Some(rows) = &broadcast.host.name_tiles {
            visible_radio_tile_row(&rows[1])?.trim().to_string()
        } else {
            visible_radio_station_name(station, snapshot.progression.active_engine_flags.contains("ENGINE_ROCKETS_IN_RADIO_TOWER"))?.to_string()
        };
        let mut entries = vec![format!("RADIO  {heading}")];
        entries.extend(visible_radio_observation_rows(runtime_shell)?);
        return Ok(entries);
    }
    if runtime_shell.pokegear_page == PokegearPage::Phone {
        return visible_pokegear_phone_entries(snapshot, runtime_shell);
    }
    visible_pokegear_landmark_indices(snapshot, runtime_shell.pokegear_standalone_map)?;
    Ok(Vec::new())
}

fn visible_pokegear_phone_slots(snapshot: &RuntimeShellSnapshot) -> Result<Vec<Option<&str>>> {
    let saved = &snapshot.script_events.phone_number_order;
    anyhow::ensure!(
        saved.len() <= 10,
        "Pokégear phone list exceeds ten source slots"
    );
    let mut slots = vec![None; 10];
    for (index, contact) in saved.iter().enumerate() {
        slots[index] = contact.as_deref();
    }
    Ok(slots)
}

fn visible_pokegear_phone_window(runtime_shell: &BevyRuntimeShell) -> Result<usize> {
    let selected = runtime_shell.pokegear_phone_cursor;
    let scroll = runtime_shell.pokegear_phone_scroll;
    anyhow::ensure!(
        selected < 10 && scroll <= 6 && selected >= scroll && selected < scroll + 4,
        "Pokégear phone cursor {selected} is outside its four-row window at {scroll}"
    );
    Ok(scroll)
}

fn visible_pokegear_phone_prompt(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
) -> Result<String> {
    if let Some(call) = runtime_shell.pokegear_phone_call.as_ref() {
        match call.phase {
            VisiblePokegearPhoneCallPhase::Calling
            | VisiblePokegearPhoneCallPhase::FinishDelay { .. }
            | VisiblePokegearPhoneCallPhase::AwaitHangup => return Ok(String::new()),
            _ => {}
        }
    }
    let label = if let Some(VisiblePokegearPhoneCall {
        phase: VisiblePokegearPhoneCallPhase::HangingUp { frames_remaining }, ..
    }) = runtime_shell.pokegear_phone_call.as_ref() {
        match VISIBLE_POKEGEAR_HANGUP_FRAMES - frames_remaining {
            4..=27 => "_PhoneClickText",
            32..=55 | 84..=107 | 136..=159 => "_PhoneEllipseText",
            _ => return Ok(String::new()),
        }
    } else if runtime_shell.pokegear_phone_delete_question_retained
        || runtime_shell.pokegear_phone_menu.as_ref().is_some_and(|menu| menu.delete_confirmation.is_some()) {
        "_PokegearAskDeleteText"
    } else if runtime_shell
        .pokegear_phone_call
        .as_ref()
        .is_some_and(|call| matches!(call.phase, VisiblePokegearPhoneCallPhase::NoServicePrompt))
    {
        "_GearOutOfServiceText"
    } else if runtime_shell.pokegear_phone_call.as_ref().is_some_and(|call| {
        matches!(call.phase, VisiblePokegearPhoneCallPhase::Ringing { .. })
    }) {
        "_GearEllipseText"
    } else {
        "_PokegearAskWhoCallText"
    };
    let text = snapshot
        .presentation
        .asm_text
        .get(label)
        .with_context(|| format!("Pokégear phone is missing {label}"))?;
    Ok(normalize_visible_script_text_with_context(
        text,
        &snapshot.trainer.player_name,
        visible_rival_name(snapshot),
        snapshot.progression.time.day_of_week,
    ))
}

fn visible_pokegear_phone_entries(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
) -> Result<Vec<String>> {
    let slots = visible_pokegear_phone_slots(snapshot)?;
    let start = visible_pokegear_phone_window(runtime_shell)?;
    let mut entries = Vec::new();
    for row in 0..4 {
        let marker = if start + row == runtime_shell.pokegear_phone_cursor {
            ">"
        } else {
            " "
        };
        if let Some(contact_id) = slots[start + row] {
            let contact = snapshot
                .special
                .phone_contacts
                .0
                .get(contact_id)
                .with_context(|| format!("Pokégear phone contact {contact_id} is missing"))?;
            let primary = contact
                .lines
                .first()
                .with_context(|| format!("Pokégear phone contact {contact_id} has no name"))?;
            entries.push(format!("{marker}{primary}"));
            entries.push(
                contact
                    .lines
                    .get(1)
                    .map(|class| format!("    {class}"))
                    .unwrap_or_default(),
            );
        } else {
            // NonTrainerCallerNames.none is the source name for contact ID 0.
            entries.push(format!("{marker}----------"));
            entries.push(String::new());
        }
    }
    entries.extend(
        visible_pokegear_phone_prompt(snapshot, runtime_shell)?
            .lines()
            .map(str::to_string),
    );
    if let Some(menu) = runtime_shell.pokegear_phone_menu.as_ref() {
        let labels: &[&str] = if menu.delete_confirmation.is_some() {
            &["YES", "NO"]
        } else if menu.can_delete {
            &["CALL", "DELETE", "CANCEL"]
        } else {
            &["CALL", "CANCEL"]
        };
        let selected = menu.delete_confirmation.unwrap_or(menu.cursor);
        entries.extend(labels.iter().enumerate().map(|(index, label)|
            format!("{}{}", if index == selected { ">" } else { " " }, label)));
    }
    Ok(entries)
}

fn selected_visible_pokegear_phone_contact_id(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
) -> Result<Option<String>> {
    visible_pokegear_phone_window(runtime_shell)?;
    Ok(
        visible_pokegear_phone_slots(snapshot)?[runtime_shell.pokegear_phone_cursor]
            .map(str::to_string),
    )
}

fn visible_options_menu_entries(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
) -> Result<Vec<String>> {
    anyhow::ensure!(
        runtime_shell.options_cursor < OPTIONS_MENU_ITEMS.len(),
        "Options cursor {} is out of range for {} rows",
        runtime_shell.options_cursor,
        OPTIONS_MENU_ITEMS.len()
    );
    let selected = runtime_shell.options_cursor;
    Ok(windowed_index_range(selected, OPTIONS_MENU_ITEMS.len())
        .map(|index| {
            let item = OPTIONS_MENU_ITEMS[index];
            let marker = if index == selected { ">" } else { " " };
            let value = option_value_for_item(&snapshot.trainer.options, item);
            if value.is_empty() {
                format!("{marker}{}", options_menu_item_label(item))
            } else {
                format!("{marker}{}: {value}", options_menu_item_label(item))
            }
        })
        .collect())
}

fn visible_trainer_card_entries(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
) -> Vec<String> {
    if runtime_shell.trainer_card_page == VisibleTrainerCardPage::JohtoBadges {
        const JOHTO_BADGE_NAMES: [&str; 8] = [
            "ZEPHYR", "HIVE", "PLAIN", "FOG", "MINERAL", "STORM", "GLACIER", "RISING",
        ];
        let mut entries = vec!["BADGES".to_string()];
        entries.extend(
            JOHTO_BADGE_NAMES
                .iter()
                .zip(snapshot.progression.badges.johto.iter())
                .filter(|(_, owned)| **owned)
                .map(|(name, _)| (*name).to_string()),
        );
        return entries;
    }
    let mut entries = vec![
        compact_scene_label(&format!("NAME/ {}", snapshot.trainer.player_name), 30),
        format!("ID {:05}", snapshot.trainer.player_id),
        format!(
            "MONEY {}",
            format_trainer_card_money(snapshot.trainer.money)
        ),
    ];
    if snapshot
        .progression
        .active_engine_flags
        .contains(ENGINE_POKEDEX_FLAG)
    {
        entries.push(format!(
            "#DEX {:>3}",
            snapshot.progression.pokedex_owned.min(999)
        ));
    }
    entries.extend([
        compact_scene_label(&format!("TIME {:?}", snapshot.progression.time), 30),
        format!(
            "BADGES {}",
            visible_badge_count(&snapshot.progression.badges)
        ),
    ]);
    entries
}

fn visible_save_slot_preview_entries_for_path(
    runtime_shell: &BevyRuntimeShell,
    path: &std::path::Path,
) -> Vec<String> {
    match runtime_shell.shell.runtime().load_save_summary(path) {
        Ok(summary) => vec![
            compact_scene_label(
                &format!(
                    "EXISTS F{} {} {}",
                    summary.saved_frame(),
                    summary.modpack().id(),
                    summary.pack_content_hash()
                ),
                30,
            ),
            compact_scene_label(&format!("SAVE VERSION {}", summary.format_version()), 30),
            compact_scene_label(&format!("SAVE PACK {}", summary.modpack().id()), 30),
            compact_scene_label(
                &format!(
                    "SAVE HASH {} {}",
                    summary.modpack().hash(),
                    summary.pack_content_hash()
                ),
                30,
            ),
        ],
        Err(error) => vec![compact_scene_label(&format!("INVALID SAVE {error}"), 30)],
    }
}

fn visible_special_boundary_entries(runtime_shell: &BevyRuntimeShell) -> Vec<String> {
    let Some(boundary) = &runtime_shell.special_boundary else {
        return Vec::new();
    };
    visible_special_boundary_display_entries(boundary)
}

fn visible_special_boundary_display_entries(boundary: &SpecialBoundaryDisplay) -> Vec<String> {
    if boundary.label == "HallOfFamePC" {
        return boundary.details.clone();
    }
    if matches!(
        boundary.label.as_str(),
        "PokecenterPCCantUseText" | "ProfOaksPcBoot" | "BugContestJudging"
    ) {
        return boundary
            .details
            .iter()
            .flat_map(|line| wrap_scene_dialog_line(line, SCENE_DIALOG_TEXT_CHARS))
            .take(SCENE_MENU_VISIBLE_ROWS)
            .collect();
    }
    if boundary.label.starts_with("DayCare")
        || matches!(
            boundary.label.as_str(),
            "IllRaiseYourMonText"
                | "ComeBackLaterText"
                | "OnlyOneMonText"
                | "CantAcceptEggText"
                | "RemoveMailText"
                | "LastHealthyMonText"
                | "OhFineThenText"
                | "ComeAgainText"
        )
    {
        return boundary
            .details
            .iter()
            .flat_map(|detail| detail.lines())
            .flat_map(|line| wrap_scene_dialog_line(line, SCENE_DIALOG_TEXT_CHARS))
            .take(SCENE_MENU_VISIBLE_ROWS)
            .collect();
    }
    vec![format!(">{}", boundary.label)]
}

fn visible_move_battler_offsets(animation: Option<&VisibleMoveAnimation>) -> (Vec3, Vec3) {
    let Some(animation) = animation.filter(|animation| animation.started) else {
        return (Vec3::ZERO, Vec3::ZERO);
    };
    let mut player = Vec3::ZERO;
    let mut enemy = Vec3::ZERO;
    for effect in animation
        .bg_events
        .iter()
        .filter(|effect| !effect.incremented && effect.frame <= animation.frame)
    {
        if animation.bg_events.iter().any(|candidate| {
            !candidate.incremented
                && candidate.effect_id == effect.effect_id
                && candidate.frame > effect.frame
                && candidate.frame <= animation.frame
        }) {
            continue;
        }
        let user_is_player = animation.player_move;
        let target_is_player = match effect.target.as_str() {
            "BG_EFFECT_USER" => user_is_player,
            "BG_EFFECT_TARGET" => !user_is_player,
            _ => true,
        };
        let reset_frame = animation
            .bg_events
            .iter()
            .filter(|candidate| {
                candidate.incremented
                    && candidate.effect_id == effect.effect_id
                    && candidate.frame >= effect.frame
                    && candidate.frame <= animation.frame
            })
            .map(|candidate| candidate.frame)
            .max()
            .unwrap_or(effect.frame);
        let age = animation.frame.saturating_sub(reset_frame);
        let source_scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
        let (offset_x, offset_y, force_player) = match effect.effect_id.as_str() {
            "BATTLE_BG_EFFECT_TACKLE" | "BATTLE_BG_EFFECT_BODY_SLAM" => {
                let Some(distance) = visible_tackle_lunge_offset(age) else {
                    continue;
                };
                (
                    if target_is_player {
                        distance
                    } else {
                        -distance
                    },
                    0,
                    false,
                )
            }
            "BATTLE_BG_EFFECT_BETA_PURSUIT" => {
                let Some(distance) = visible_beta_pursuit_offset(age) else {
                    continue;
                };
                (
                    if target_is_player {
                        distance
                    } else {
                        -distance
                    },
                    0,
                    false,
                )
            }
            "BATTLE_BG_EFFECT_VITAL_THROW" => {
                let increment_frame = animation
                    .bg_events
                    .iter()
                    .find(|candidate| {
                        candidate.incremented
                            && candidate.effect_id == effect.effect_id
                            && candidate.frame >= effect.frame
                    })
                    .map(|candidate| candidate.frame);
                let Some(distance) = visible_vital_throw_offset(
                    age,
                    increment_frame.map(|frame| frame.saturating_sub(effect.frame)),
                ) else {
                    continue;
                };
                (
                    if target_is_player {
                        distance
                    } else {
                        -distance
                    },
                    0,
                    false,
                )
            }
            "BATTLE_BG_EFFECT_WOBBLE_MON" => {
                // The source ignores all script operands except the target.
                // Its setup update holds at zero, then advances the exact
                // BattleAnim sine phase by four with radius eight until the
                // script increments the background-effect state.
                let stopped = animation.bg_events.iter().any(|candidate| {
                    candidate.incremented
                        && candidate.effect_id == effect.effect_id
                        && candidate.frame >= effect.frame
                        && candidate.frame <= animation.frame
                });
                if age == 0 || stopped {
                    continue;
                }
                let angle = ((age - 1) as u8).wrapping_mul(4);
                (0, visible_battle_anim_sine(angle, 8), false)
            }
            "BATTLE_BG_EFFECT_WOBBLE_PLAYER" => {
                // BattleBGEffect_WobblePlayer always owns the player side and
                // ignores all four script operands. After setup it advances a
                // radius-six BattleAnim sine phase by two for $20 updates.
                if age == 0 || age > 0x20 {
                    continue;
                }
                let angle = ((age - 1) as u8).wrapping_mul(2);
                let value = visible_battle_anim_sine(angle, 6);
                (value, 0, true)
            }
            "BATTLE_BG_EFFECT_VIBRATE_MON" => {
                // BattleBGEffect_VibrateMon ignores the script duration/param.
                // Its setup frame has no displacement, then its fixed $20
                // counter toggles a one-pixel SCX offset every two updates.
                if age == 0 || age > 0x20 {
                    continue;
                }
                let value = if ((age - 1) / 2) & 1 == 0 { -1 } else { 1 };
                (value, 0, false)
            }
            "BATTLE_BG_EFFECT_BOUNCE_DOWN" => {
                continue;
            }
            "BATTLE_BG_EFFECT_REMOVE_MON" => {
                let (distance, _) = visible_remove_mon_state(age, target_is_player);
                (distance, 0, false)
            }
            "BATTLE_BG_EFFECT_BETA_SEND_OUT_MON1" => {
                continue;
            }
            "BATTLE_BG_EFFECT_BETA_SEND_OUT_MON2" => continue,
            "BATTLE_BG_EFFECT_FAINT_MON" => {
                // core.asm::MonFaintedAnimation copies the battler bitmap down
                // one eight-pixel tile, waits two frames, and repeats seven
                // times. The first copy happens before the first delay.
                const FAINT_FRAMES: u16 = 14;
                const FAINT_STEP_PIXELS: i32 = 8;
                if age >= FAINT_FRAMES {
                    continue;
                }
                let step_index = i32::from(age / 2 + 1);
                (0, step_index * FAINT_STEP_PIXELS, false)
            }
            _ => continue,
        };
        let offset = Vec3::new(
            offset_x as f32 * source_scale,
            -(offset_y as f32) * source_scale,
            0.0,
        );
        let target_is_player = force_player || target_is_player;
        if target_is_player {
            player += offset;
        } else {
            enemy += offset;
        }
    }
    (player, enemy)
}

fn visible_tackle_lunge_offset(age: u16) -> Option<i32> {
    const OFFSETS: [i32; 12] = [0, 0, 2, 4, 6, 8, 10, 8, 6, 4, 2, 0];
    OFFSETS.get(usize::from(age)).copied()
}

fn visible_beta_pursuit_offset(age: u16) -> Option<i32> {
    visible_tackle_lunge_offset(age).map(|offset| -offset)
}

fn visible_vital_throw_offset(age: u16, increment_age: Option<u16>) -> Option<i32> {
    let backwards = [0_i32, 0, -2, -4, -6, -8, -10];
    if age < backwards.len() as u16 {
        return Some(backwards[usize::from(age)]);
    }
    let increment_age = increment_age?;
    if age < increment_age {
        return Some(-10);
    }
    let return_age = age.saturating_sub(increment_age);
    const RETURN: [i32; 7] = [-10, -8, -6, -4, -2, 0, 0];
    RETURN.get(usize::from(return_age)).copied()
}

fn visible_remove_mon_state(age: u16, player: bool) -> (i32, bool) {
    // Setup occupies the first update. State 1 then shifts the battler's BG
    // rectangle by one tile, states 2 and 3 are one-update pauses, and state 4
    // either loops or ends. The player rectangle is nine tiles wide; the enemy
    // rectangle is eight.
    let shifts = if age == 0 {
        0
    } else {
        ((age - 1) / 4 + 1).min(if player { 9 } else { 8 })
    };
    let removed = shifts == if player { 9 } else { 8 };
    let distance = i32::from(shifts) * 8;
    (if player { -distance } else { distance }, !removed)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct VisibleRemoveMonClip {
    source_pixels: u8,
    crop_left: bool,
}

fn visible_remove_mon_clips(
    animation: Option<&VisibleMoveAnimation>,
) -> (Option<VisibleRemoveMonClip>, Option<VisibleRemoveMonClip>) {
    let Some(animation) = animation.filter(|animation| animation.started) else {
        return (None, None);
    };
    let mut player = None;
    let mut enemy = None;
    for effect in animation.bg_events.iter().filter(|effect| {
        !effect.incremented
            && effect.effect_id == "BATTLE_BG_EFFECT_REMOVE_MON"
            && effect.frame <= animation.frame
    }) {
        let target_player = target_player_for_effect(animation, effect);
        let age = animation.frame.saturating_sub(effect.frame);
        let shifts = if age == 0 { 0 } else { (age - 1) / 4 + 1 };
        // The 6x6 backpic starts two tiles inside its 9-column copy region;
        // the 7x7 frontpic has one blank column at the right of its 8-column
        // region. Cropping starts only after that padding has scrolled away.
        let padding = if target_player { 2 } else { 1 };
        let clip = VisibleRemoveMonClip {
            source_pixels: shifts.saturating_sub(padding).saturating_mul(8) as u8,
            crop_left: target_player,
        };
        if target_player {
            player = Some(clip);
        } else {
            enemy = Some(clip);
        }
    }
    (player, enemy)
}

fn visible_move_screen_offset(animation: Option<&VisibleMoveAnimation>) -> Vec3 {
    let Some(animation) = animation.filter(|animation| animation.started) else {
        return Vec3::ZERO;
    };
    let mut result = Vec3::ZERO;
    for effect in animation.bg_events.iter().filter(|effect| {
        !effect.incremented
            && effect.frame <= animation.frame
            && matches!(
                effect.effect_id.as_str(),
                "BATTLE_BG_EFFECT_SHAKE_SCREEN_X"
                    | "BATTLE_BG_EFFECT_SHAKE_SCREEN_Y"
                    | "BATTLE_BG_EFFECT_ROLLOUT"
                    | "BATTLE_BG_EFFECT_WOBBLE_SCREEN"
            )
    }) {
        if animation.bg_events.iter().any(|candidate| {
            !candidate.incremented
                && candidate.effect_id == effect.effect_id
                && candidate.frame > effect.frame
                && candidate.frame <= animation.frame
        }) {
            continue;
        }
        let reset_frame = animation
            .bg_events
            .iter()
            .filter(|candidate| {
                candidate.incremented
                    && candidate.effect_id == effect.effect_id
                    && candidate.frame >= effect.frame
                    && candidate.frame <= animation.frame
            })
            .map(|candidate| candidate.frame)
            .max()
            .unwrap_or(effect.frame);
        let age = animation.frame.saturating_sub(reset_frame);
        let (screen_x, screen_y) = if effect.effect_id == "BATTLE_BG_EFFECT_WOBBLE_SCREEN" {
            // BattleBGEffect_WobbleScreen starts immediately (there is no
            // setup jumptable state), ignores every script operand, and
            // writes a radius-six sine phase to hSCX for $20 updates.
            if age >= 0x20 {
                continue;
            }
            let value = visible_battle_anim_sine((age as u8).wrapping_mul(2), 6) as f32;
            (value, 0.0)
        } else if effect.effect_id == "BATTLE_BG_EFFECT_ROLLOUT" {
            let Some(value) = visible_bg_shake_amount(effect, age) else {
                continue;
            };
            let value = if value >= 0 { f32::from(value) } else { 0.0 };
            (0.0, value)
        } else {
            let Some(value) = visible_bg_shake_amount(effect, age) else {
                continue;
            };
            if effect.effect_id == "BATTLE_BG_EFFECT_SHAKE_SCREEN_X" {
                (f32::from(value), 0.0)
            } else {
                (0.0, f32::from(value))
            }
        };
        let source_scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
        result += Vec3::new(screen_x * source_scale, -screen_y * source_scale, 0.0);
    }
    result
}

fn visible_bg_shake_amount(effect: &VisibleMoveBgEvent, age: u16) -> Option<i8> {
    if age >= effect.duration {
        return None;
    }
    let mut parameter = effect.param;
    let mut displacement = parse_visible_battle_animation_int(&effect.target)? as u8;
    for _ in 0..=age {
        if parameter & 0x0f != 0 {
            parameter = parameter.wrapping_sub(1);
        } else {
            parameter = parameter.rotate_left(4) | parameter;
            displacement = 0_u8.wrapping_sub(displacement);
        }
    }
    Some(displacement as i8)
}

fn visible_rollout_object_y_offset(animation: &VisibleMoveAnimation, slot_index: usize) -> i32 {
    if slot_index != 0 {
        return 0;
    }
    let Some(effect) = animation.bg_events.iter().rev().find(|effect| {
        !effect.incremented
            && effect.effect_id == "BATTLE_BG_EFFECT_ROLLOUT"
            && effect.frame <= animation.frame
    }) else {
        return 0;
    };
    let age = animation.frame.saturating_sub(effect.frame);
    visible_bg_shake_amount(effect, age)
        .filter(|displacement| *displacement >= 0)
        .map_or(0, |displacement| -i32::from(displacement))
}

fn apply_visible_battle_screen_offset(
    runtime_shell: Res<BevyRuntimeShell>,
    mut battle_commands: Query<
        (Entity, &mut Transform),
        (
            Or<(With<BattleCommandMarker>, With<BattleBattlerMarker>)>,
            Without<FixedBattleCanvasMarker>,
        ),
    >,
    mut applied_offsets: Local<HashMap<Entity, Vec3>>,
) {
    let current = visible_move_screen_offset(runtime_shell.visible_move_animations.front());
    let mut live = HashSet::new();
    for (entity, mut transform) in &mut battle_commands {
        live.insert(entity);
        let previous = applied_offsets.get(&entity).copied().unwrap_or(Vec3::ZERO);
        transform.translation += current - previous;
        applied_offsets.insert(entity, current);
    }
    applied_offsets.retain(|entity, _| live.contains(entity));
}

fn visible_bounce_down_offset(
    animation: &VisibleMoveAnimation,
    effect: &VisibleMoveBgEvent,
) -> Option<i32> {
    if animation.bg_events.iter().any(|candidate| {
        candidate.incremented
            && candidate.effect_id == effect.effect_id
            && candidate.frame >= effect.frame
            && candidate.frame <= animation.frame
    }) {
        return None;
    }
    let age = animation.frame.saturating_sub(effect.frame);
    if age == 0 {
        return Some(0);
    }
    let parameter = 0x20_u8.wrapping_add((age.saturating_sub(1) as u8).wrapping_mul(2));
    Some(17 + visible_battle_anim_sine(parameter.wrapping_add(0x10), 16))
}

fn visible_move_battler_visibility(animation: Option<&VisibleMoveAnimation>) -> (bool, bool) {
    let Some(animation) = animation.filter(|animation| animation.started) else {
        return (true, true);
    };
    let mut player_visible = true;
    let mut enemy_visible = true;
    for effect in animation
        .bg_events
        .iter()
        .filter(|effect| !effect.incremented && effect.frame <= animation.frame)
    {
        let reset_frame = animation
            .bg_events
            .iter()
            .filter(|candidate| {
                candidate.incremented
                    && candidate.effect_id == effect.effect_id
                    && candidate.frame >= effect.frame
                    && candidate.frame <= animation.frame
            })
            .map(|candidate| candidate.frame)
            .max()
            .unwrap_or(effect.frame);
        let age = animation.frame.saturating_sub(reset_frame);
        let visible = match effect.effect_id.as_str() {
            "BATTLE_BG_EFFECT_HIDE_MON" => false,
            "BATTLE_BG_EFFECT_SHOW_MON" => true,
            "BATTLE_BG_EFFECT_REMOVE_MON" => {
                visible_remove_mon_state(age, target_player_for_effect(animation, effect)).1
            }
            "BATTLE_BG_EFFECT_FAINT_MON" => {
                // Seven tile shifts with two frames per shift.
                age < 14
            }
            "BATTLE_BG_EFFECT_ENTER_MON" => true,
            "BATTLE_BG_EFFECT_RETURN_MON" => age < 12,
            _ => continue,
        };
        let target_player = match effect.target.as_str() {
            "BG_EFFECT_USER" => animation.player_move,
            "BG_EFFECT_TARGET" => !animation.player_move,
            _ => matches!(
                effect.effect_id.as_str(),
                "BATTLE_BG_EFFECT_REMOVE_MON" | "BATTLE_BG_EFFECT_FAINT_MON"
            ),
        };
        if target_player {
            player_visible = visible;
        } else {
            enemy_visible = visible;
        }
    }
    (player_visible, enemy_visible)
}

fn target_player_for_effect(animation: &VisibleMoveAnimation, effect: &VisibleMoveBgEvent) -> bool {
    match effect.target.as_str() {
        "BG_EFFECT_USER" => animation.player_move,
        "BG_EFFECT_TARGET" => !animation.player_move,
        _ => matches!(
            effect.effect_id.as_str(),
            "BATTLE_BG_EFFECT_REMOVE_MON" | "BATTLE_BG_EFFECT_FAINT_MON"
        ),
    }
}

fn visible_move_controls_battler_visibility(
    animation: Option<&VisibleMoveAnimation>,
    player: bool,
) -> bool {
    let Some(animation) = animation.filter(|animation| animation.started) else {
        return false;
    };
    animation.bg_events.iter().any(|effect| {
        if !matches!(
            effect.effect_id.as_str(),
            "BATTLE_BG_EFFECT_HIDE_MON"
                | "BATTLE_BG_EFFECT_SHOW_MON"
                | "BATTLE_BG_EFFECT_REMOVE_MON"
                | "BATTLE_BG_EFFECT_ENTER_MON"
                | "BATTLE_BG_EFFECT_RETURN_MON"
        ) {
            return false;
        }
        let targets_player = match effect.target.as_str() {
            "BG_EFFECT_USER" => animation.player_move,
            "BG_EFFECT_TARGET" => !animation.player_move,
            _ => false,
        };
        targets_player == player
    })
}

fn visible_move_battler_clip_tiles(
    animation: Option<&VisibleMoveAnimation>,
) -> (Option<u8>, Option<u8>) {
    let Some(animation) = animation.filter(|animation| animation.started) else {
        return (None, None);
    };
    let mut player = None;
    let mut enemy = None;
    for effect in animation
        .bg_events
        .iter()
        .filter(|effect| !effect.incremented && effect.frame <= animation.frame)
    {
        if !matches!(
            effect.effect_id.as_str(),
            "BATTLE_BG_EFFECT_ENTER_MON" | "BATTLE_BG_EFFECT_RETURN_MON"
        ) {
            continue;
        }
        let age = animation.frame.saturating_sub(effect.frame);
        let step = age / 4;
        let target_player = match effect.target.as_str() {
            "BG_EFFECT_USER" => animation.player_move,
            "BG_EFFECT_TARGET" => !animation.player_move,
            _ => true,
        };
        let clip_tiles = match (effect.effect_id.as_str(), target_player, step) {
            ("BATTLE_BG_EFFECT_ENTER_MON", _, 3..) => None,
            ("BATTLE_BG_EFFECT_ENTER_MON", true, 0) => Some(2),
            ("BATTLE_BG_EFFECT_ENTER_MON", true, 1) => Some(4),
            ("BATTLE_BG_EFFECT_ENTER_MON", true, 2) => Some(6),
            ("BATTLE_BG_EFFECT_ENTER_MON", false, 0) => Some(3),
            ("BATTLE_BG_EFFECT_ENTER_MON", false, 1) => Some(5),
            ("BATTLE_BG_EFFECT_ENTER_MON", false, 2) => Some(7),
            ("BATTLE_BG_EFFECT_RETURN_MON", true, 0) => Some(6),
            ("BATTLE_BG_EFFECT_RETURN_MON", true, 1) => Some(4),
            ("BATTLE_BG_EFFECT_RETURN_MON", true, 2) => Some(2),
            ("BATTLE_BG_EFFECT_RETURN_MON", false, 0) => Some(7),
            ("BATTLE_BG_EFFECT_RETURN_MON", false, 1) => Some(5),
            ("BATTLE_BG_EFFECT_RETURN_MON", false, 2) => Some(3),
            ("BATTLE_BG_EFFECT_RETURN_MON", _, 3..) => None,
            _ => None,
        };
        if target_player {
            player = clip_tiles;
        } else {
            enemy = clip_tiles;
        }
    }
    (player, enemy)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct VisibleBattlerRowExtraction {
    rows: u8,
    top: bool,
    bg_rows_cleared: bool,
    render_extracted: bool,
}

fn visible_move_battler_row_extractions(
    animation: Option<&VisibleMoveAnimation>,
) -> (
    Option<VisibleBattlerRowExtraction>,
    Option<VisibleBattlerRowExtraction>,
) {
    let Some(animation) = animation.filter(|animation| animation.started) else {
        return (None, None);
    };
    let mut player = None;
    let mut enemy = None;
    for effect in animation
        .bg_events
        .iter()
        .filter(|effect| effect.frame <= animation.frame)
    {
        if effect.effect_id == "BATTLE_BG_EFFECT_FAINT_MON" {
            let target_player = target_player_for_effect(animation, effect);
            let max_rows = if target_player { 6 } else { 7 };
            let rows = u8::try_from(animation.frame.saturating_sub(effect.frame) / 2 + 1)
                .unwrap_or(u8::MAX)
                .min(max_rows);
            let extraction = VisibleBattlerRowExtraction {
                rows,
                top: false,
                bg_rows_cleared: true,
                render_extracted: false,
            };
            if target_player {
                player = Some(extraction);
            } else {
                enemy = Some(extraction);
            }
            continue;
        }
        if !matches!(
            effect.effect_id.as_str(),
            "BATTLE_BG_EFFECT_BATTLEROBJ_1ROW" | "BATTLE_BG_EFFECT_BATTLEROBJ_2ROW"
        ) {
            continue;
        }
        if animation.object_events.iter().any(|event| {
            event.frame >= effect.frame
                && event.frame <= animation.frame
                && matches!(&event.command, VisibleMoveObjectCommand::Clear)
        }) {
            continue;
        }
        let target_player = target_player_for_effect(animation, effect);
        let redrawn = animation.bg_events.iter().any(|candidate| {
            candidate.effect_id == "BATTLE_BG_EFFECT_SHOW_MON"
                && candidate.frame > effect.frame
                && candidate.frame <= animation.frame
                && target_player_for_effect(animation, candidate) == target_player
        });
        let extraction = VisibleBattlerRowExtraction {
            rows: if effect.effect_id.ends_with("2ROW") {
                2
            } else {
                1
            },
            top: target_player,
            bg_rows_cleared: animation.frame > effect.frame && !redrawn,
            render_extracted: true,
        };
        if target_player {
            player = Some(extraction);
        } else {
            enemy = Some(extraction);
        }
    }
    (player, enemy)
}

fn visible_move_battler_bgps(animation: Option<&VisibleMoveAnimation>) -> (Option<u8>, Option<u8>) {
    let Some(animation) = animation.filter(|animation| animation.started) else {
        return (None, None);
    };
    let mut player = None;
    let mut enemy = None;
    for effect in animation
        .bg_events
        .iter()
        .filter(|effect| !effect.incremented && effect.frame <= animation.frame)
    {
        let target_player = match effect.target.as_str() {
            "BG_EFFECT_USER" => animation.player_move,
            "BG_EFFECT_TARGET" => !animation.player_move,
            _ => false,
        };
        let age = animation.frame.saturating_sub(effect.frame);
        let stopped = animation.bg_events.iter().any(|candidate| {
            candidate.incremented
                && candidate.effect_id == effect.effect_id
                && candidate.frame >= effect.frame
                && candidate.frame <= animation.frame
        });
        let (table, repeating): (&[u8], bool) = match effect.effect_id.as_str() {
            "BATTLE_BG_EFFECT_FADE_MON_TO_LIGHT" => (&[0xe4, 0x90, 0x40], false),
            "BATTLE_BG_EFFECT_FADE_MON_TO_BLACK" => (&[0xe4, 0xf8, 0xfc], false),
            "BATTLE_BG_EFFECT_FADE_MON_TO_LIGHT_REPEATING" => (&[0xe4, 0x90, 0x40, 0x90], true),
            "BATTLE_BG_EFFECT_FADE_MON_TO_BLACK_REPEATING"
            | "BATTLE_BG_EFFECT_FADE_MONS_TO_BLACK_REPEATING" => (&[0xe4, 0xf8, 0xfc, 0xf8], true),
            "BATTLE_BG_EFFECT_CYCLE_MON_LIGHT_DARK_REPEATING" => {
                (&[0xe4, 0xf8, 0xfc, 0xf8, 0xe4, 0x90, 0x40, 0x90], true)
            }
            "BATTLE_BG_EFFECT_FLASH_MON_REPEATING" => (&[0xe4, 0xfc, 0xe4, 0x00], true),
            "BATTLE_BG_EFFECT_FADE_MON_TO_WHITE_WAIT_FADE_BACK" => (
                &[
                    0xe4, 0x90, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x40, 0x90, 0xe4,
                ],
                false,
            ),
            "BATTLE_BG_EFFECT_FADE_MON_FROM_WHITE" => (&[0x00, 0x40, 0x90, 0xe4], false),
            _ => continue,
        };
        if stopped {
            if effect.effect_id == "BATTLE_BG_EFFECT_FADE_MONS_TO_BLACK_REPEATING" {
                player = None;
                enemy = None;
            } else if target_player {
                player = None;
            } else {
                enemy = None;
            }
            continue;
        }
        let initial_delay = u16::from(effect.param & 0x0f).saturating_add(1);
        let interval = u16::from(effect.param >> 4).saturating_add(1);
        let value = if age < initial_delay {
            0xe4
        } else {
            let index = usize::from((age - initial_delay) / interval);
            if !repeating && index >= table.len() {
                table[table.len() - 1]
            } else {
                table[index % table.len()]
            }
        };
        let mapped = (value != 0xe4).then_some(value);
        if effect.effect_id == "BATTLE_BG_EFFECT_FADE_MONS_TO_BLACK_REPEATING" {
            player = mapped;
            enemy = mapped;
        } else if target_player {
            player = mapped;
        } else {
            enemy = mapped;
        }
    }
    (player, enemy)
}

fn visible_move_battler_art_overrides(
    animation: Option<&VisibleMoveAnimation>,
) -> (VisibleBattlerArtOverride, VisibleBattlerArtOverride) {
    let Some(animation) =
        animation.filter(|animation| animation.started || animation.waiting_for_hp)
    else {
        return (
            VisibleBattlerArtOverride::Unchanged,
            VisibleBattlerArtOverride::Unchanged,
        );
    };
    let mut player = VisibleBattlerArtOverride::Unchanged;
    let mut enemy = VisibleBattlerArtOverride::Unchanged;
    for event in animation.bg_events.iter().filter(|event| {
        event.frame <= animation.frame && event.effect_id.starts_with("BATTLE_ACTOR_")
    }) {
        let mut target_player = animation.player_move;
        let state = match event.effect_id.as_str() {
            // Transform and Minimize only load a picture into temporary vTiles0.
            // The actor BG remains unchanged until UpdateActorPic copies it.
            "BATTLE_ACTOR_TRANSFORM" | "BATTLE_ACTOR_MINIMIZE" => continue,
            "BATTLE_ACTOR_RAISESUB" => VisibleBattlerArtOverride::Substitute,
            "BATTLE_ACTOR_DROPSUB" => VisibleBattlerArtOverride::Pokemon,
            "BATTLE_ACTOR_UPDATEACTORPIC" => {
                let loaded_picture = animation
                    .bg_events
                    .iter()
                    .filter(|candidate| {
                        matches!(
                            candidate.effect_id.as_str(),
                            "BATTLE_ACTOR_TRANSFORM" | "BATTLE_ACTOR_MINIMIZE"
                        ) && candidate.frame < event.frame
                            && target_player_for_effect(animation, candidate) == target_player
                    })
                    .max_by_key(|candidate| candidate.frame)
                    .map(|candidate| candidate.effect_id.as_str());
                match loaded_picture {
                    Some("BATTLE_ACTOR_TRANSFORM") => VisibleBattlerArtOverride::Transform,
                    Some("BATTLE_ACTOR_MINIMIZE") => VisibleBattlerArtOverride::Minimize,
                    _ => VisibleBattlerArtOverride::Pokemon,
                }
            }
            "BATTLE_ACTOR_BEATUP" => VisibleBattlerArtOverride::Pokemon,
            "BATTLE_ACTOR_MINIMIZEOPP" => {
                target_player = !target_player;
                VisibleBattlerArtOverride::Minimize
            }
            _ => continue,
        };
        if target_player {
            player = state;
        } else {
            enemy = state;
        }
    }
    (player, enemy)
}

fn visible_move_battler_species_overrides(
    animation: Option<&VisibleMoveAnimation>,
) -> (Option<&str>, Option<&str>) {
    let Some(animation) = animation.filter(|animation| animation.started) else {
        return (None, None);
    };
    let beat_up_active = animation
        .bg_events
        .iter()
        .any(|event| event.frame <= animation.frame && event.effect_id == "BATTLE_ACTOR_BEATUP");
    if !beat_up_active {
        return (None, None);
    }
    let species = animation.actor_species_override.as_deref();
    if animation.player_move {
        (species, None)
    } else {
        (None, species)
    }
}

fn visible_move_battler_shiny_overrides(
    animation: Option<&VisibleMoveAnimation>,
) -> (Option<bool>, Option<bool>) {
    let Some(animation) = animation.filter(|animation| animation.started) else {
        return (None, None);
    };
    let beat_up_active = animation
        .bg_events
        .iter()
        .any(|event| event.frame <= animation.frame && event.effect_id == "BATTLE_ACTOR_BEATUP");
    if !beat_up_active {
        return (None, None);
    }
    if animation.player_move {
        (animation.actor_shiny_override, None)
    } else {
        (None, animation.actor_shiny_override)
    }
}

fn visible_surf_line_offsets(animation: Option<&VisibleMoveAnimation>) -> Option<[i8; 0x5f]> {
    let animation = animation.filter(|animation| animation.started)?;
    let surf_effect = animation.bg_events.iter().find(|effect| {
        !effect.incremented
            && effect.effect_id == "BATTLE_BG_EFFECT_SURF"
            && effect.frame <= animation.frame
    })?;
    let (spawn_index, spawn) = animation
        .object_events
        .iter()
        .enumerate()
        .find(|(_, event)| {
            event.frame <= animation.frame
                && matches!(
                    &event.command,
                    VisibleMoveObjectCommand::Spawn { object_id, .. }
                        if object_id == "BATTLE_ANIM_OBJ_SURF"
                )
        })?;
    let VisibleMoveObjectCommand::Spawn { x, y, param, .. } = &spawn.command else {
        return None;
    };
    let effect_age = animation.frame.saturating_sub(surf_effect.frame);
    if effect_age == 0 {
        return Some([0; 0x5f]);
    }
    // BG effects execute before animation objects. The copy therefore sees
    // the scanline boundary written by Surf on the preceding update.
    let object_frame = animation.frame.saturating_sub(1);
    let function = battle_program::FUNCTIONS.iter().position(|name| *name == "BATTLE_ANIM_FUNC_SURF")? as u8;
    let mut machine = BattleObjectMachine::new(animation.player_move);
    machine.initialize(0, 1, [0, 0, 0, function, 0, 0], *x as u8, *y as u8, *param);
    for tick in spawn.frame..=object_frame {
        for event in animation.object_events.iter().skip(spawn_index + 1).filter(|event| event.frame == tick) {
            match event.command {
                VisibleMoveObjectCommand::Increment { index: 1 } => {
                    machine.object_mut(0)[14] = machine.object(0)[14].wrapping_add(1);
                }
                VisibleMoveObjectCommand::Set { index: 1, value } => machine.object_mut(0)[14] = value,
                VisibleMoveObjectCommand::Clear => machine.clear_objects(),
                _ => {}
            }
        }
        if machine.object(0)[0] != 0 { machine.step_object(0).ok()?; }
    }
    let start = usize::from(machine.read(battle_program::H_L_Y_OVERRIDE_START)).min(0x5e);
    let rotation = usize::from(effect_age);
    let mut offsets = [0_i8; 0x5f];
    for line in start.saturating_add(1)..=0x5e {
        let wave_index = (line + rotation) & 0x3f;
        offsets[line] = visible_battle_anim_sine((wave_index as u8).wrapping_mul(2), 2) as i8;
    }
    Some(offsets)
}

fn visible_wave_deform_line_offsets(
    animation: Option<&VisibleMoveAnimation>,
) -> Option<[i8; 0x5f]> {
    let animation = animation.filter(|animation| animation.started)?;
    let effect = animation.bg_events.iter().rev().find(|effect| {
        !effect.incremented
            && effect.effect_id == "BATTLE_BG_EFFECT_WAVE_DEFORM_MON"
            && effect.frame <= animation.frame
    })?;
    let stop_frame = animation
        .bg_events
        .iter()
        .find(|candidate| {
            candidate.incremented
                && candidate.effect_id == effect.effect_id
                && candidate.frame >= effect.frame
        })
        .map(|candidate| candidate.frame);
    let age = animation.frame.saturating_sub(effect.frame);
    let amplitude = match stop_frame {
        Some(stop) if animation.frame >= stop => {
            31_u16.saturating_sub(animation.frame.saturating_sub(stop)) as u8
        }
        _ => age.saturating_sub(1).min(31) as u8,
    };
    let target_player = match effect.target.as_str() {
        "BG_EFFECT_USER" => animation.player_move,
        "BG_EFFECT_TARGET" => !animation.player_move,
        _ => return None,
    };
    let (first_line, last_line) = if target_player {
        (0x30_usize, 0x5e_usize)
    } else {
        (0x01_usize, 0x36_usize)
    };
    let mut offsets = [0_i8; 0x5f];
    for (line, offset) in offsets
        .iter_mut()
        .enumerate()
        .take(last_line + 1)
        .skip(first_line)
    {
        *offset = visible_battle_anim_sine((line as u8).wrapping_mul(4), amplitude) as i8;
    }
    Some(offsets)
}

fn visible_battle_line_x_offsets(animation: Option<&VisibleMoveAnimation>) -> Option<[i8; 0x5f]> {
    let sources = [
        visible_surf_line_offsets(animation),
        visible_wave_deform_line_offsets(animation),
        visible_psychic_teleport_line_x_offsets(animation),
        visible_beta_send_out_mon2_line_x_offsets(animation),
        visible_flail_line_x_offsets(animation),
        visible_double_team_line_x_offsets(animation),
    ];
    let mut combined = None::<[i8; 0x5f]>;
    for source in sources.into_iter().flatten() {
        let offsets = combined.get_or_insert([0; 0x5f]);
        for (offset, source_offset) in offsets.iter_mut().zip(source) {
            *offset = offset.wrapping_add(source_offset);
        }
    }
    combined
}

fn visible_beta_send_out_mon2_line_x_offsets(
    animation: Option<&VisibleMoveAnimation>,
) -> Option<[i8; 0x5f]> {
    let animation = animation.filter(|animation| animation.started)?;
    let effect = animation.bg_events.iter().rev().find(|effect| {
        !effect.incremented
            && effect.effect_id == "BATTLE_BG_EFFECT_BETA_SEND_OUT_MON2"
            && effect.frame <= animation.frame
    })?;
    let age = animation.frame.saturating_sub(effect.frame);
    if age >= 65 {
        return None;
    }
    let mut offsets = [0_i8; 0x5f];
    if age == 0 {
        return Some(offsets);
    }
    let counter = 65_u16.saturating_sub(age) as u8;
    let radius = (counter >> 3) & 0x0f;
    let target_player = target_player_for_effect(animation, effect);
    let (start, end) = if target_player {
        (0x2f_usize, 0x5e_usize)
    } else {
        (0x00_usize, 0x36_usize)
    };
    for (line, offset) in offsets.iter_mut().enumerate().take(end + 1).skip(start + 1) {
        *offset = visible_battle_anim_sine((line as u8).wrapping_mul(radius), radius) as i8;
    }
    Some(offsets)
}

fn visible_wavy_screen_offsets(
    animation: Option<&VisibleMoveAnimation>,
    effect_ids: &[&str],
) -> Option<[i8; 0x5f]> {
    let animation = animation.filter(|animation| animation.started)?;
    let effect = animation.bg_events.iter().rev().find(|effect| {
        !effect.incremented
            && effect_ids.contains(&effect.effect_id.as_str())
            && effect.frame <= animation.frame
    })?;
    if animation.bg_events.iter().any(|candidate| {
        candidate.incremented
            && candidate.effect_id == effect.effect_id
            && candidate.frame >= effect.frame
            && candidate.frame <= animation.frame
    }) {
        return None;
    }
    let age = animation.frame.saturating_sub(effect.frame);
    let rotations = match effect.effect_id.as_str() {
        "BATTLE_BG_EFFECT_PSYCHIC" if age > 0 => 1 + (age - 1) / 4,
        "BATTLE_BG_EFFECT_PSYCHIC" => 0,
        _ => age,
    } as usize;
    let (phase_step, amplitude) = match effect.effect_id.as_str() {
        "BATTLE_BG_EFFECT_NIGHT_SHADE" => (effect.param, 2),
        _ => (6, 5),
    };
    let mut base = [0_i8; 0x60];
    for (line, offset) in base.iter_mut().enumerate().skip(1) {
        *offset = visible_battle_anim_sine((line as u8).wrapping_mul(phase_step), amplitude) as i8;
    }
    let mut offsets = [0_i8; 0x5f];
    for (line, offset) in offsets.iter_mut().enumerate() {
        *offset = base[(line + rotations) % base.len()];
    }
    Some(offsets)
}

fn visible_psychic_teleport_line_x_offsets(
    animation: Option<&VisibleMoveAnimation>,
) -> Option<[i8; 0x5f]> {
    visible_wavy_screen_offsets(
        animation,
        &["BATTLE_BG_EFFECT_PSYCHIC", "BATTLE_BG_EFFECT_TELEPORT"],
    )
}

fn visible_night_shade_line_y_offsets(
    animation: Option<&VisibleMoveAnimation>,
) -> Option<[i8; 0x5f]> {
    visible_wavy_screen_offsets(animation, &["BATTLE_BG_EFFECT_NIGHT_SHADE"])
}

fn visible_whirlpool_line_y_offsets(
    animation: Option<&VisibleMoveAnimation>,
) -> Option<[i8; 0x5f]> {
    let animation = animation.filter(|animation| animation.started)?;
    let effect = animation.bg_events.iter().rev().find(|effect| {
        !effect.incremented
            && effect.effect_id == "BATTLE_BG_EFFECT_WHIRLPOOL"
            && effect.frame <= animation.frame
    })?;
    if animation.bg_events.iter().any(|candidate| {
        candidate.incremented
            && candidate.effect_id == effect.effect_id
            && candidate.frame >= effect.frame
            && candidate.frame <= animation.frame
    }) {
        return None;
    }
    let rotations = usize::from(animation.frame.saturating_sub(effect.frame));
    let mut base = [0_i8; 0x5f];
    for (line, offset) in base.iter_mut().enumerate().skip(1) {
        *offset = visible_battle_anim_sine((line as u8).wrapping_mul(2), 2) as i8;
    }
    let mut offsets = [0_i8; 0x5f];
    for (line, offset) in offsets.iter_mut().enumerate() {
        *offset = base[(line + rotations) % base.len()];
    }
    Some(offsets)
}

fn visible_water_line_y_offsets(animation: Option<&VisibleMoveAnimation>) -> Option<[i8; 0x5f]> {
    let animation = animation.filter(|animation| animation.started)?;
    let first_start = animation.bg_events.iter().find(|effect| {
        effect.effect_id == "BATTLE_BG_EFFECT_START_WATER" && effect.frame <= animation.frame
    })?;
    // The ASM backup allocation extends beyond the visible $5f scanlines;
    // Water centers can land at byte $5f and then expand back into view.
    let mut backup = [0_i8; 0x99];
    let mut range = None;
    for frame in first_start.frame..=animation.frame {
        for effect in animation
            .bg_events
            .iter()
            .filter(|effect| effect.frame == frame)
        {
            match effect.effect_id.as_str() {
                "BATTLE_BG_EFFECT_START_WATER" => {
                    backup.fill(0);
                    range = Some(if target_player_for_effect(animation, effect) {
                        (0x2f_usize, 0x5e_usize)
                    } else {
                        (0x00_usize, 0x36_usize)
                    });
                }
                "BATTLE_BG_EFFECT_END_WATER" => range = None,
                _ => {}
            }
        }
        let Some((start, end)) = range else {
            continue;
        };
        for effect in animation.bg_events.iter().filter(|effect| {
            !effect.incremented
                && effect.effect_id == "BATTLE_BG_EFFECT_WATER"
                && effect.frame <= frame
        }) {
            let age = frame.saturating_sub(effect.frame);
            if age >= 16 {
                if age == 16 {
                    backup.fill(0);
                }
                continue;
            }
            let timer = usize::from(age * 2);
            let mut phase = (age as u8).wrapping_mul(4);
            let amplitude = if age < 8 { 3 } else { 2 };
            let center = start + usize::from(effect.duration);
            let mut right = center;
            let mut left = center;
            for _ in 0..timer {
                let displacement = visible_battle_anim_sine(phase, amplitude) as i8;
                if right <= end && right < backup.len() {
                    backup[right] = displacement;
                    right += 1;
                }
                if left > start && left < backup.len() {
                    backup[left] = displacement;
                    left -= 1;
                }
                phase = phase.wrapping_add(4);
            }
        }
    }
    range.map(|_| {
        let mut visible = [0_i8; 0x5f];
        visible.copy_from_slice(&backup[..0x5f]);
        visible
    })
}

fn visible_double_team_line_x_offsets(
    animation: Option<&VisibleMoveAnimation>,
) -> Option<[i8; 0x5f]> {
    let animation = animation.filter(|animation| animation.started)?;
    let effect = animation.bg_events.iter().rev().find(|effect| {
        !effect.incremented
            && effect.effect_id == "BATTLE_BG_EFFECT_DOUBLE_TEAM"
            && effect.frame <= animation.frame
    })?;
    let increments = animation
        .bg_events
        .iter()
        .filter(|candidate| {
            candidate.incremented
                && candidate.effect_id == effect.effect_id
                && candidate.frame >= effect.frame
        })
        .map(|candidate| candidate.frame)
        .collect::<Vec<_>>();
    if increments
        .get(1)
        .is_some_and(|frame| animation.frame >= *frame)
    {
        return None;
    }
    let age = animation.frame.saturating_sub(effect.frame);
    let displacement = if let Some(first_increment) = increments
        .first()
        .filter(|frame| animation.frame >= **frame)
    {
        let contraction_age = animation.frame.saturating_sub(*first_increment);
        if contraction_age <= 16 {
            // Entering state 3 consumes the increment command and immediately
            // decrements the stored $10 displacement before writing it.
            15_u8.saturating_sub(contraction_age as u8)
        } else {
            0
        }
    } else {
        match age {
            0 => 0,
            1..=16 => age.saturating_sub(1) as u8,
            17 => 15,
            _ => {
                let phase = (age.saturating_sub(18) as u8).wrapping_mul(4);
                (16_i32 + visible_battle_anim_sine(phase, 2)) as u8
            }
        }
    } as i8;
    let target_player = match effect.target.as_str() {
        "BG_EFFECT_USER" => animation.player_move,
        "BG_EFFECT_TARGET" => !animation.player_move,
        _ => return None,
    };
    let (first_line, last_line) = if target_player {
        (0x2f_usize, 0x5e_usize)
    } else {
        (0x00_usize, 0x36_usize)
    };
    let mut offsets = [0_i8; 0x5f];
    for (index, offset) in offsets
        .iter_mut()
        .enumerate()
        .take(last_line + 1)
        .skip(first_line)
    {
        *offset = if (index - first_line) & 1 == 0 {
            displacement
        } else {
            -displacement
        };
    }
    Some(offsets)
}

fn visible_flail_line_x_offsets(animation: Option<&VisibleMoveAnimation>) -> Option<[i8; 0x5f]> {
    let animation = animation.filter(|animation| animation.started)?;
    let effect = animation.bg_events.iter().rev().find(|effect| {
        !effect.incremented
            && effect.effect_id == "BATTLE_BG_EFFECT_FLAIL"
            && effect.frame <= animation.frame
    })?;
    if animation.bg_events.iter().any(|candidate| {
        candidate.incremented
            && candidate.effect_id == effect.effect_id
            && candidate.frame >= effect.frame
            && candidate.frame <= animation.frame
    }) {
        return None;
    }
    let age = animation.frame.saturating_sub(effect.frame);
    let displacement = if age == 0 {
        0
    } else {
        let update = age.saturating_sub(1) as u8;
        visible_battle_anim_sine(update.wrapping_mul(2), 6)
            + visible_battle_anim_sine(update.wrapping_mul(8), 2)
    } as i8;
    let target_player = match effect.target.as_str() {
        "BG_EFFECT_USER" => animation.player_move,
        "BG_EFFECT_TARGET" => !animation.player_move,
        _ => return None,
    };
    let (first_line, last_line) = if target_player {
        (0x2f_usize, 0x5e_usize)
    } else {
        (0x00_usize, 0x36_usize)
    };
    let mut offsets = [0_i8; 0x5f];
    for offset in offsets.iter_mut().take(last_line + 1).skip(first_line) {
        *offset = displacement;
    }
    Some(offsets)
}

#[derive(Clone)]
struct VisibleBattleLineOffsets {
    x: [i8; 0x5f],
    y: [i8; 0x5f],
    bgp: Option<[u8; 0x5f]>,
}

fn visible_battler_line_offsets(
    base: Option<&VisibleBattleLineOffsets>,
    bgp: Option<u8>,
) -> Option<VisibleBattleLineOffsets> {
    let mut offsets = base.cloned().unwrap_or(VisibleBattleLineOffsets {
        x: [0; 0x5f],
        y: [0; 0x5f],
        bgp: None,
    });
    if let Some(bgp) = bgp {
        offsets.bgp = Some([bgp; 0x5f]);
    }
    (base.is_some() || bgp.is_some()).then_some(offsets)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VisibleBattleDmgPaletteRegisters {
    obp0_write_frame: Option<u16>,
    bgp: u8,
    obp0: u8,
    obp1: u8,
}

fn dmg_palette(shades: [u8; 4]) -> u8 {
    shades
        .into_iter()
        .enumerate()
        .fold(0, |value, (index, shade)| {
            value | ((shade & 3) << ((3 - index) * 2))
        })
}

fn battle_bg_effect_reload_interval(effect: &VisibleMoveBgEvent) -> u16 {
    let raw = effect.target.trim().replace('_', "");
    let battle_turn = if let Some(hex) = raw.strip_prefix('$') {
        u16::from_str_radix(hex, 16)
    } else if let Some(binary) = raw.strip_prefix('%') {
        u16::from_str_radix(binary, 2)
    } else {
        raw.parse::<u16>()
    }
    .unwrap_or_else(|_| {
        panic!(
            "palette background effect {} has nonnumeric BATTLE_TURN byte {}",
            effect.effect_id, effect.target
        )
    });
    battle_turn.saturating_add(1).max(1)
}

fn visible_battle_dmg_palette_registers(
    animation: Option<&VisibleMoveAnimation>,
) -> VisibleBattleDmgPaletteRegisters {
    let mut registers = VisibleBattleDmgPaletteRegisters {
        obp0_write_frame: None,
        bgp: 0xe4,
        obp0: 0xe4,
        obp1: 0xe4,
    };
    let Some(animation) = animation.filter(|animation| animation.started) else {
        return registers;
    };
    for frame in 0..=animation.frame {
        for effect in animation
            .bg_events
            .iter()
            .filter(|effect| effect.frame <= frame)
        {
            let age = frame - effect.frame;
            match effect.effect_id.as_str() {
                "BATTLE_PALETTE_BGP" if age == 0 => registers.bgp = effect.param,
                "BATTLE_PALETTE_OBP0" if age == 0 => { registers.obp0 = effect.param; registers.obp0_write_frame = Some(frame); },
                "BATTLE_PALETTE_OBP1" if age == 0 => registers.obp1 = effect.param,
                "BATTLE_BG_EFFECT_FLASH_INVERTED" | "BATTLE_BG_EFFECT_FLASH_WHITE" => {
                    let interval = battle_bg_effect_reload_interval(effect);
                    if age % interval != 0 {
                        continue;
                    }
                    let write = age / interval;
                    if write >= u16::from(effect.param) {
                        continue;
                    }
                    let remaining = effect.param - write as u8 - 1;
                    registers.bgp = if remaining & 1 == 0 {
                        0xe4
                    } else if effect.effect_id == "BATTLE_BG_EFFECT_FLASH_INVERTED" {
                        0x1b
                    } else {
                        0x00
                    };
                }
                "BATTLE_BG_EFFECT_WHITE_HUES"
                | "BATTLE_BG_EFFECT_BLACK_HUES"
                | "BATTLE_BG_EFFECT_ALTERNATE_HUES"
                | "BATTLE_BG_EFFECT_CYCLE_OBPALS_GRAY_AND_YELLOW"
                | "BATTLE_BG_EFFECT_CYCLE_MID_OBPALS_GRAY_AND_YELLOW"
                | "BATTLE_BG_EFFECT_CYCLE_BGPALS_INVERTED" => {
                    let interval = battle_bg_effect_reload_interval(effect);
                    if age % interval != 0 {
                        continue;
                    }
                    let write = usize::from(age / interval);
                    let (table, repeating): (&[u8], bool) = match effect.effect_id.as_str() {
                        "BATTLE_BG_EFFECT_WHITE_HUES" => (
                            &[
                                dmg_palette([3, 2, 1, 0]),
                                dmg_palette([3, 2, 0, 0]),
                                dmg_palette([3, 1, 0, 0]),
                            ],
                            false,
                        ),
                        "BATTLE_BG_EFFECT_BLACK_HUES" => (
                            &[
                                dmg_palette([3, 2, 1, 0]),
                                dmg_palette([3, 3, 1, 0]),
                                dmg_palette([3, 3, 2, 0]),
                            ],
                            false,
                        ),
                        "BATTLE_BG_EFFECT_ALTERNATE_HUES" => (
                            &[
                                dmg_palette([3, 2, 1, 0]),
                                dmg_palette([3, 3, 2, 0]),
                                dmg_palette([3, 3, 3, 0]),
                                dmg_palette([3, 3, 2, 0]),
                                dmg_palette([3, 2, 1, 0]),
                                dmg_palette([2, 1, 0, 0]),
                                dmg_palette([1, 0, 0, 0]),
                                dmg_palette([2, 1, 0, 0]),
                            ],
                            true,
                        ),
                        "BATTLE_BG_EFFECT_CYCLE_OBPALS_GRAY_AND_YELLOW" => (
                            &[dmg_palette([3, 2, 1, 0]), dmg_palette([2, 1, 0, 0])],
                            true,
                        ),
                        "BATTLE_BG_EFFECT_CYCLE_MID_OBPALS_GRAY_AND_YELLOW" => (
                            &[dmg_palette([3, 2, 1, 0]), dmg_palette([3, 1, 2, 0])],
                            true,
                        ),
                        _ => (
                            &[
                                dmg_palette([0, 1, 2, 3]),
                                dmg_palette([1, 2, 0, 3]),
                                dmg_palette([2, 0, 1, 3]),
                            ],
                            true,
                        ),
                    };
                    if !repeating && write >= table.len() {
                        continue;
                    }
                    let value = table[write % table.len()];
                    registers.bgp = value;
                    if effect.effect_id == "BATTLE_BG_EFFECT_ALTERNATE_HUES" {
                        registers.obp1 = value;
                    } else if matches!(
                        effect.effect_id.as_str(),
                        "BATTLE_BG_EFFECT_CYCLE_OBPALS_GRAY_AND_YELLOW"
                            | "BATTLE_BG_EFFECT_CYCLE_MID_OBPALS_GRAY_AND_YELLOW"
                    ) {
                        registers.obp0 = value;
                        registers.obp0_write_frame = Some(frame);
                    }
                }
                _ => {}
            }
        }
    }
    registers
}

fn visible_beta_send_out_mon1_line_bgps(
    animation: Option<&VisibleMoveAnimation>,
) -> Option<[u8; 0x5f]> {
    let animation = animation.filter(|animation| animation.started)?;
    let effect = animation.bg_events.iter().rev().find(|effect| {
        !effect.incremented
            && effect.effect_id == "BATTLE_BG_EFFECT_BETA_SEND_OUT_MON1"
            && effect.frame <= animation.frame
    })?;
    let target_player = target_player_for_effect(animation, effect);
    let (mut start, end) = if target_player {
        (0x2f_usize, 0x5f_usize)
    } else {
        (0x00_usize, 0x37_usize)
    };
    let mut bgps = [0xe4_u8; 0x5f];
    for value in bgps.iter_mut().take(end.min(0x5f)).skip(start) {
        *value = 0;
    }
    let mut state = 1_u8;
    let mut parameter = 0_u8;
    let data = [0x00_u8, 0x40, 0x90, 0xe4];
    for frame in effect.frame.saturating_add(1)..=animation.frame {
        let increments = animation
            .bg_events
            .iter()
            .filter(|candidate| {
                candidate.incremented
                    && candidate.effect_id == effect.effect_id
                    && candidate.frame == frame
            })
            .count();
        state = state.saturating_add(u8::try_from(increments).ok()?);
        match state {
            2 | 3 => {
                let index = usize::from(parameter >> 3);
                parameter = parameter.wrapping_add(1);
                let Some(&value) = data.get(index) else {
                    parameter = 0;
                    if state == 2 {
                        start = start.saturating_add(1);
                    }
                    state += 1;
                    continue;
                };
                for line in (start..end.min(0x5f)).step_by(2) {
                    bgps[line] = value;
                }
                if state == 3 && end > 0 && end - 1 < 0x5f {
                    bgps[end - 1] = value;
                }
            }
            5.. => return None,
            _ => {}
        }
    }
    Some(bgps)
}

fn visible_bounce_down_line_y_offsets(
    animation: Option<&VisibleMoveAnimation>,
) -> Option<[i8; 0x5f]> {
    let animation = animation.filter(|animation| animation.started)?;
    let effect = animation.bg_events.iter().rev().find(|effect| {
        !effect.incremented
            && effect.effect_id == "BATTLE_BG_EFFECT_BOUNCE_DOWN"
            && effect.frame <= animation.frame
    })?;
    let displacement = u8::try_from(visible_bounce_down_offset(animation, effect)?).ok()?;
    let target_player = match effect.target.as_str() {
        "BG_EFFECT_USER" => animation.player_move,
        "BG_EFFECT_TARGET" => !animation.player_move,
        _ => return None,
    };
    let (first_line, last_line) = if target_player {
        (0x2d_usize, 0x5e_usize)
    } else {
        (0x00_usize, 0x36_usize)
    };
    let mut offsets = [0_i8; 0x5f];
    let line_count = last_line + 1 - first_line;
    for (relative_line, offset) in offsets
        .iter_mut()
        .take(last_line + 1)
        .skip(first_line)
        .enumerate()
    {
        *offset = if relative_line < usize::from(displacement).min(line_count) {
            0x90_u8 as i8
        } else {
            (!displacement) as i8
        };
    }
    Some(offsets)
}

fn visible_dig_displacement(age: u16) -> i8 {
    let mut wait = 0_u8;
    let mut displacement = 0_i8;
    let mut amount = 2_u8;
    for _ in 1..=age {
        if wait != 0 {
            wait = wait.wrapping_sub(1);
            continue;
        }
        if amount > 47 {
            continue;
        }
        displacement = -((amount as i8).wrapping_add(1));
        if amount & 7 == 0 {
            wait = 16;
        }
        amount = amount.wrapping_add(2);
    }
    displacement
}

fn visible_dig_line_y_offsets(animation: Option<&VisibleMoveAnimation>) -> Option<[i8; 0x5f]> {
    let animation = animation.filter(|animation| animation.started)?;
    let effect = animation.bg_events.iter().rev().find(|effect| {
        !effect.incremented
            && effect.effect_id == "BATTLE_BG_EFFECT_DIG"
            && effect.frame <= animation.frame
    })?;
    if animation.bg_events.iter().any(|candidate| {
        candidate.incremented
            && candidate.effect_id == effect.effect_id
            && candidate.frame >= effect.frame
            && candidate.frame <= animation.frame
    }) {
        return None;
    }
    let target_player = match effect.target.as_str() {
        "BG_EFFECT_USER" => animation.player_move,
        "BG_EFFECT_TARGET" => !animation.player_move,
        _ => return None,
    };
    let (first_line, last_line) = if target_player {
        (0x2f_usize, 0x5e_usize)
    } else {
        (0x00_usize, 0x36_usize)
    };
    let displacement = visible_dig_displacement(animation.frame.saturating_sub(effect.frame));
    let mut offsets = [0_i8; 0x5f];
    if displacement == 0 {
        return Some(offsets);
    }
    let blank_lines = usize::from(!(displacement as u8));
    for (relative_line, offset) in offsets
        .iter_mut()
        .take(last_line + 1)
        .skip(first_line)
        .enumerate()
    {
        *offset = if relative_line < blank_lines {
            0x90_u8 as i8
        } else {
            displacement
        };
    }
    Some(offsets)
}

fn visible_acid_armor_line_y_offsets(
    animation: Option<&VisibleMoveAnimation>,
) -> Option<[i8; 0x5f]> {
    let animation = animation.filter(|animation| animation.started)?;
    let effect = animation.bg_events.iter().rev().find(|effect| {
        !effect.incremented
            && effect.effect_id == "BATTLE_BG_EFFECT_ACID_ARMOR"
            && effect.frame <= animation.frame
    })?;
    if animation.bg_events.iter().any(|candidate| {
        candidate.incremented
            && candidate.effect_id == effect.effect_id
            && candidate.frame >= effect.frame
            && candidate.frame <= animation.frame
    }) {
        return None;
    }
    let target_player = match effect.target.as_str() {
        "BG_EFFECT_USER" => animation.player_move,
        "BG_EFFECT_TARGET" => !animation.player_move,
        _ => return None,
    };
    let (first_line, last_line) = if target_player {
        (0x2f_usize, 0x5e_usize)
    } else {
        (0x00_usize, 0x36_usize)
    };
    let mut offsets = [0_i8; 0x5f];
    for (line, offset) in offsets
        .iter_mut()
        .enumerate()
        .take(last_line + 1)
        .skip(first_line + 1)
    {
        *offset = visible_battle_anim_sine((line as u8).wrapping_mul(2), effect.param) as i8;
    }
    offsets[last_line] = 0;
    offsets[last_line - 1] = 0;
    for _ in 0..animation.frame.saturating_sub(effect.frame) {
        for line in (first_line + 1..=last_line).rev() {
            offsets[line] = offsets[line - 1];
        }
        offsets[first_line] = 0x90_u8 as i8;
        if !matches!(offsets[last_line] as u8, 0 | 0x90) {
            offsets[last_line] = 0;
        }
        if !matches!(offsets[last_line - 1] as u8, 0 | 1 | 0x90) {
            offsets[last_line - 1] = 0;
        }
    }
    Some(offsets)
}

fn visible_withdraw_line_y_offsets(animation: Option<&VisibleMoveAnimation>) -> Option<[i8; 0x5f]> {
    let animation = animation.filter(|animation| animation.started)?;
    let effect = animation.bg_events.iter().rev().find(|effect| {
        !effect.incremented
            && effect.effect_id == "BATTLE_BG_EFFECT_WITHDRAW"
            && effect.frame <= animation.frame
    })?;
    if animation.bg_events.iter().any(|candidate| {
        candidate.incremented
            && candidate.effect_id == effect.effect_id
            && candidate.frame >= effect.frame
            && candidate.frame <= animation.frame
    }) {
        return None;
    }
    let target_player = match effect.target.as_str() {
        "BG_EFFECT_USER" => animation.player_move,
        "BG_EFFECT_TARGET" => !animation.player_move,
        _ => return None,
    };
    let (first_line, last_line) = if target_player {
        (0x2f_usize, 0x5e_usize)
    } else {
        (0x00_usize, 0x36_usize)
    };
    let limit = effect.param & 0x3f;
    let speed = effect.param.rotate_left(2) & 3;
    let age = animation.frame.saturating_sub(effect.frame);
    if age == 0 {
        return Some([0; 0x5f]);
    }
    let displacement = if speed == 0 {
        1
    } else {
        1_u16
            .saturating_add(age.saturating_sub(1).saturating_mul(u16::from(speed)))
            .min(u16::from(limit.saturating_sub(1))) as u8
    };
    let mut offsets = [0_i8; 0x5f];
    for line in first_line..=last_line {
        offsets[line] = if line - first_line < usize::from(displacement) {
            0x90_u8 as i8
        } else {
            (!displacement) as i8
        };
    }
    Some(offsets)
}

fn visible_battle_line_offsets(
    animation: Option<&VisibleMoveAnimation>,
) -> Option<VisibleBattleLineOffsets> {
    let x = visible_battle_line_x_offsets(animation);
    let global_bgp = visible_battle_dmg_palette_registers(animation).bgp;
    let bgp = visible_beta_send_out_mon1_line_bgps(animation)
        .or_else(|| (global_bgp != 0xe4).then_some([global_bgp; 0x5f]));
    let y_sources = [
        visible_bounce_down_line_y_offsets(animation),
        visible_dig_line_y_offsets(animation),
        visible_acid_armor_line_y_offsets(animation),
        visible_withdraw_line_y_offsets(animation),
        visible_night_shade_line_y_offsets(animation),
        visible_whirlpool_line_y_offsets(animation),
        visible_water_line_y_offsets(animation),
    ];
    let mut y = None::<[i8; 0x5f]>;
    for source in y_sources.into_iter().flatten() {
        let offsets = y.get_or_insert([0; 0x5f]);
        for (offset, source_offset) in offsets.iter_mut().zip(source) {
            *offset = offset.wrapping_add(source_offset);
        }
    }
    if x.is_none() && y.is_none() && bgp.is_none() {
        None
    } else {
        Some(VisibleBattleLineOffsets {
            x: x.unwrap_or([0; 0x5f]),
            y: y.unwrap_or([0; 0x5f]),
            bgp,
        })
    }
}

fn spawn_battle_battler_markers(
    commands: &mut Commands,
    snapshot: &RuntimeShellSnapshot,
    battle: &crate::RuntimeBattleSnapshot,
    entry_messages_remaining: usize,
    enemy_send_out_pending: bool,
    player_send_out_pending: bool,
    capture_enemy_hidden: bool,
    capture_enemy_clip_tiles: Option<u8>,
    capture_throw_active: bool,
    send_out_animation: Option<&VisibleSendOutAnimation>,
    trainer_exit_animation: Option<&VisibleTrainerExitAnimation>,
    frontpic_animation: Option<&VisibleFrontpicAnimation>,
    move_animation: Option<&VisibleMoveAnimation>,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<()> {
    let (player_move_offset, enemy_move_offset) = visible_move_battler_offsets(move_animation);
    let (move_player_visible, move_enemy_visible) = visible_move_battler_visibility(move_animation);
    let move_player_visible = move_player_visible
        && (!battle.player_semi_invulnerable
            || visible_move_controls_battler_visibility(move_animation, true));
    let move_enemy_visible = move_enemy_visible
        && (!battle.enemy_semi_invulnerable
            || visible_move_controls_battler_visibility(move_animation, false));
    let (move_player_clip_tiles, move_enemy_clip_tiles) =
        visible_move_battler_clip_tiles(move_animation);
    let (move_player_remove_clip, move_enemy_remove_clip) =
        visible_remove_mon_clips(move_animation);
    let (move_player_row_extraction, move_enemy_row_extraction) =
        visible_move_battler_row_extractions(move_animation);
    let (move_player_bgp, move_enemy_bgp) = visible_move_battler_bgps(move_animation);
    let (move_player_art, move_enemy_art) = visible_move_battler_art_overrides(move_animation);
    let (move_player_species, move_enemy_species) =
        visible_move_battler_species_overrides(move_animation);
    let (move_player_shiny, move_enemy_shiny) =
        visible_move_battler_shiny_overrides(move_animation);
    let line_offsets = visible_battle_line_offsets(move_animation);
    let player_line_offsets = visible_battler_line_offsets(line_offsets.as_ref(), move_player_bgp);
    let enemy_line_offsets = visible_battler_line_offsets(line_offsets.as_ref(), move_enemy_bgp);
    let render_hp = |side, hp| {
        let unchecked_spikes_ko = match side {
            crate::core::battle::turn::BattleSide::Player => {
                battle.player_spikes_zero_hp_unchecked
            }
            crate::core::battle::turn::BattleSide::Enemy => {
                battle.enemy_spikes_zero_hp_unchecked
            }
        };
        if hp == 0 && unchecked_spikes_ko {
            1
        } else {
            visible_faint_animation_render_hp(move_animation, side, hp)
        }
    };
    let active_player_pokemon = battle
        .active_player_party_index
        .and_then(|active_index| {
            snapshot
                .party
                .slots
                .iter()
                .find(|slot| slot.index == active_index)
        })
        .map(|slot| &slot.pokemon);
    let active_player_species = battle
        .player_transformed_species
        .as_deref()
        .or_else(|| active_player_pokemon.map(|pokemon| pokemon.species.id.as_str()));
    let transform_pending_player = move_animation.is_some_and(|animation| {
        animation.started
            && animation.animation_label == "BattleAnim_Transform"
            && animation.player_move
            && move_player_art != VisibleBattlerArtOverride::Transform
    });
    let transform_pending_enemy = move_animation.is_some_and(|animation| {
        animation.started
            && animation.animation_label == "BattleAnim_Transform"
            && !animation.player_move
            && move_enemy_art != VisibleBattlerArtOverride::Transform
    });
    let enemy_default_species = if transform_pending_enemy {
        &battle.enemy_pokemon.species.id
    } else {
        battle
            .enemy_transformed_species
            .as_deref()
            .unwrap_or(&battle.enemy_pokemon.species.id)
    };
    let enemy_render_species = if let Some(species) = move_enemy_species {
        species
    } else if move_enemy_art == VisibleBattlerArtOverride::Transform {
        active_player_species
            .context("enemy Transform animation requires a concrete active player species")?
    } else {
        enemy_default_species
    };
    let player_transform_species = if let Some(species) = move_player_species {
        Some(species)
    } else if move_player_art == VisibleBattlerArtOverride::Transform {
        Some(enemy_default_species)
    } else {
        None
    };
    let enemy_render_shiny = if move_enemy_art == VisibleBattlerArtOverride::Transform {
        active_player_pokemon.is_some_and(|pokemon| visible_pokemon_is_shiny(pokemon))
    } else {
        move_enemy_shiny.unwrap_or_else(|| visible_pokemon_is_shiny(&battle.enemy_pokemon))
    };
    let player_render_shiny = if move_player_art == VisibleBattlerArtOverride::Transform {
        visible_pokemon_is_shiny(&battle.enemy_pokemon)
    } else {
        false
    };
    let send_out_scale = |side| {
        send_out_animation
            .filter(|animation| animation.side == side)
            .map(VisibleSendOutAnimation::battler_scale)
            .unwrap_or(1.0)
    };
    let send_out_clip_tiles = |side| {
        send_out_animation
            .filter(|animation| animation.side == side)
            .and_then(VisibleSendOutAnimation::battler_clip_tiles)
    };
    let enemy_animation_frame = frontpic_animation
        .filter(|animation| animation.species_id == battle.enemy_pokemon.species.id)
        .map(|animation| animation.frame)
        .unwrap_or(0);
    if let Some(exit) = trainer_exit_animation
        && exit.side == crate::core::battle::turn::BattleSide::Player
    {
        spawn_battler_marker(
            commands,
            rendered_art,
            asset_root,
            images,
            enemy_render_species,
            PokemonSpriteSide::Front,
            render_hp(
                crate::core::battle::turn::BattleSide::Enemy,
                battle.enemy_pokemon.hp,
            ),
            battle.enemy_pokemon.max_hp,
            battle.enemy_substitute_hp > 0,
            enemy_render_shiny,
            enemy_animation_frame,
            enemy_move_offset,
            1.0,
            None,
            move_enemy_art,
            move_enemy_clip_tiles,
            move_enemy_remove_clip,
            move_enemy_row_extraction,
            true,
            enemy_line_offsets.as_ref(),
        )?;
        let player_stem = if snapshot.trainer.player_gender == PLAYER_GENDER_FEMALE {
            "kris_back"
        } else {
            "chris_back"
        };
        spawn_battle_trainer_marker(
            commands,
            rendered_art,
            asset_root,
            images,
            &format!("battle-player:{player_stem}"),
            Vec3::new(
                PLAYFIELD_LEFT + TILE_SIZE * 2.0 + exit.x_offset(),
                PLAYFIELD_TOP - TILE_SIZE * 6.0,
                3.0,
            ),
        )?;
        return Ok(());
    }
    if enemy_send_out_pending {
        let active_index = battle
            .active_player_party_index
            .context("enemy send-out presentation requires an active player party slot")?;
        let slot = snapshot
            .party
            .slots
            .iter()
            .find(|slot| slot.index == active_index)
            .with_context(|| {
                format!("active player party slot {active_index} is absent during enemy send-out")
            })?;
        spawn_battler_marker(
            commands,
            rendered_art,
            asset_root,
            images,
            player_transform_species.unwrap_or_else(|| {
                if transform_pending_player {
                    &slot.pokemon.species.id
                } else {
                    battle
                        .player_transformed_species
                        .as_deref()
                        .unwrap_or(&slot.pokemon.species.id)
                }
            }),
            PokemonSpriteSide::Back,
            render_hp(
                crate::core::battle::turn::BattleSide::Player,
                slot.pokemon.hp,
            ),
            slot.pokemon.max_hp,
            battle.player_substitute_hp > 0,
            if move_player_art == VisibleBattlerArtOverride::Transform {
                player_render_shiny
            } else {
                move_player_shiny.unwrap_or_else(|| visible_pokemon_is_shiny(&slot.pokemon))
            },
            0,
            player_move_offset,
            1.0,
            None,
            move_player_art,
            move_player_clip_tiles,
            move_player_remove_clip,
            move_player_row_extraction,
            true,
            player_line_offsets.as_ref(),
        )?;
        return Ok(());
    }
    if player_send_out_pending {
        spawn_battler_marker(
            commands,
            rendered_art,
            asset_root,
            images,
            enemy_render_species,
            PokemonSpriteSide::Front,
            render_hp(
                crate::core::battle::turn::BattleSide::Enemy,
                battle.enemy_pokemon.hp,
            ),
            battle.enemy_pokemon.max_hp,
            battle.enemy_substitute_hp > 0,
            enemy_render_shiny,
            enemy_animation_frame,
            enemy_move_offset,
            1.0,
            None,
            move_enemy_art,
            move_enemy_clip_tiles,
            move_enemy_remove_clip,
            move_enemy_row_extraction,
            true,
            enemy_line_offsets.as_ref(),
        )?;
        return Ok(());
    }
    if let RuntimeBattleKind::Trainer { trainer_class, .. } = &battle.kind {
        if entry_messages_remaining >= 3 {
            spawn_battle_trainer_marker(
                commands,
                rendered_art,
                asset_root,
                images,
                &format!(
                    "battle-trainer:{}",
                    normalize_battle_trainer_sprite_id(trainer_class)
                ),
                Vec3::new(PLAYFIELD_LEFT + TILE_SIZE * 12.0, PLAYFIELD_TOP, 3.0),
            )?;
            let player_stem = if snapshot.trainer.player_gender == PLAYER_GENDER_FEMALE {
                "kris_back"
            } else {
                "chris_back"
            };
            spawn_battle_trainer_marker(
                commands,
                rendered_art,
                asset_root,
                images,
                &format!("battle-player:{player_stem}"),
                Vec3::new(
                    PLAYFIELD_LEFT + TILE_SIZE * 2.0,
                    PLAYFIELD_TOP - TILE_SIZE * 6.0,
                    3.0,
                ),
            )?;
            return Ok(());
        }
        if entry_messages_remaining == 2 {
            // ShowBattleTextEnemySentOut runs after the enemy trainer has
            // slid away but before ShowSetEnemyMonAndSendOutAnimation. The
            // player's backpic remains; the enemy Pokémon is not visible yet.
            let player_stem = if snapshot.trainer.player_gender == PLAYER_GENDER_FEMALE {
                "kris_back"
            } else {
                "chris_back"
            };
            spawn_battle_trainer_marker(
                commands,
                rendered_art,
                asset_root,
                images,
                &format!("battle-player:{player_stem}"),
                Vec3::new(
                    PLAYFIELD_LEFT + TILE_SIZE * 2.0,
                    PLAYFIELD_TOP - TILE_SIZE * 6.0,
                    3.0,
                ),
            )?;
            if let Some(exit) = trainer_exit_animation
                && exit.side == crate::core::battle::turn::BattleSide::Enemy
            {
                spawn_battle_trainer_marker(
                    commands,
                    rendered_art,
                    asset_root,
                    images,
                    &format!(
                        "battle-trainer:{}",
                        normalize_battle_trainer_sprite_id(trainer_class)
                    ),
                    Vec3::new(
                        PLAYFIELD_LEFT + TILE_SIZE * 12.0 + exit.x_offset(),
                        PLAYFIELD_TOP,
                        3.0,
                    ),
                )?;
            }
            return Ok(());
        }
    }
    let enemy_scale = send_out_scale(crate::core::battle::turn::BattleSide::Enemy);
    if !capture_enemy_hidden
        && (move_enemy_visible || move_enemy_row_extraction.is_some())
        && enemy_scale > 0.0
    {
        spawn_battler_marker(
            commands,
            rendered_art,
            asset_root,
            images,
            enemy_render_species,
            PokemonSpriteSide::Front,
            render_hp(
                crate::core::battle::turn::BattleSide::Enemy,
                battle.enemy_pokemon.hp,
            ),
            battle.enemy_pokemon.max_hp,
            battle.enemy_substitute_hp > 0,
            enemy_render_shiny,
            enemy_animation_frame,
            enemy_move_offset,
            enemy_scale,
            None,
            move_enemy_art,
            capture_enemy_clip_tiles
                .or_else(|| send_out_clip_tiles(crate::core::battle::turn::BattleSide::Enemy))
                .or(move_enemy_clip_tiles),
            move_enemy_remove_clip,
            move_enemy_row_extraction,
            move_enemy_visible,
            enemy_line_offsets.as_ref(),
        )?;
    }
    if battle.battle_type == "BATTLETYPE_TUTORIAL" {
        if !capture_throw_active {
            spawn_battle_trainer_marker(
                commands,
                rendered_art,
                asset_root,
                images,
                "battle-player:dude",
                Vec3::new(
                    PLAYFIELD_LEFT + TILE_SIZE * 2.0,
                    PLAYFIELD_TOP - TILE_SIZE * 6.0,
                    3.0,
                ),
            )?;
        }
        return Ok(());
    }
    // InitBattleDisplay leaves the player's backpic on screen throughout
    // WildMonAppearedText as well as the subsequent SendOutMonText.
    if entry_messages_remaining > 0 {
        let player_stem = if snapshot.trainer.player_gender == PLAYER_GENDER_FEMALE {
            "kris_back"
        } else {
            "chris_back"
        };
        spawn_battle_trainer_marker(
            commands,
            rendered_art,
            asset_root,
            images,
            &format!("battle-player:{player_stem}"),
            Vec3::new(
                PLAYFIELD_LEFT + TILE_SIZE * 2.0,
                PLAYFIELD_TOP - TILE_SIZE * 6.0,
                3.0,
            ),
        )?;
        return Ok(());
    }
    let player_pokemon_visible = entry_messages_remaining == 0
        && (move_player_visible || move_player_row_extraction.is_some());
    if player_pokemon_visible {
        let player_scale = send_out_scale(crate::core::battle::turn::BattleSide::Player);
        if player_scale <= 0.0 {
            return Ok(());
        }
        let active_index = battle
            .active_player_party_index
            .context("visible player battler requires an active party slot")?;
        let slot = snapshot
            .party
            .slots
            .iter()
            .find(|slot| slot.index == active_index)
            .with_context(|| format!("active player party slot {active_index} is absent"))?;
        spawn_battler_marker(
            commands,
            rendered_art,
            asset_root,
            images,
            player_transform_species.unwrap_or_else(|| {
                if transform_pending_player {
                    &slot.pokemon.species.id
                } else {
                    battle
                        .player_transformed_species
                        .as_deref()
                        .unwrap_or(&slot.pokemon.species.id)
                }
            }),
            PokemonSpriteSide::Back,
            render_hp(
                crate::core::battle::turn::BattleSide::Player,
                slot.pokemon.hp,
            ),
            slot.pokemon.max_hp,
            battle.player_substitute_hp > 0,
            if move_player_art == VisibleBattlerArtOverride::Transform {
                player_render_shiny
            } else {
                move_player_shiny.unwrap_or_else(|| visible_pokemon_is_shiny(&slot.pokemon))
            },
            0,
            player_move_offset,
            player_scale,
            None,
            move_player_art,
            send_out_clip_tiles(crate::core::battle::turn::BattleSide::Player)
                .or(move_player_clip_tiles),
            move_player_remove_clip,
            move_player_row_extraction,
            move_player_visible,
            player_line_offsets.as_ref(),
        )?;
    }
    Ok(())
}

fn visible_faint_animation_render_hp(
    move_animation: Option<&VisibleMoveAnimation>,
    side: crate::core::battle::turn::BattleSide,
    hp: u16,
) -> u16 {
    if hp == 0
        && move_animation.is_some_and(|animation| {
            animation.started
                && animation.animation_label == "BattleAnim_FaintMon"
                && animation.player_move == (side == crate::core::battle::turn::BattleSide::Player)
        })
    {
        1
    } else {
        hp
    }
}

fn normalize_battle_trainer_sprite_id(trainer_class: &str) -> String {
    let normalized = trainer_class.trim().to_ascii_lowercase();
    if normalized == "pokemon_prof" {
        return "oak".to_string();
    }
    if normalized == "medium" {
        return normalized;
    }
    if normalized.ends_with('m') && !normalized.ends_with("_m") {
        format!("{}_m", &normalized[..normalized.len() - 1])
    } else if normalized.ends_with('f') && !normalized.ends_with("_f") {
        format!("{}_f", &normalized[..normalized.len() - 1])
    } else {
        normalized
    }
}

fn spawn_battle_trainer_marker(
    commands: &mut Commands,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
    asset_id: &str,
    top_left: Vec3,
) -> Result<()> {
    let key = IntroArtKey {
        asset_id: asset_id.to_string(),
    };
    if !rendered_art.intro_cache.contains_key(&key) {
        let frame = load_oak_intro_frame(asset_root, asset_id, images)
            .with_context(|| format!("load required battle trainer art {asset_id}"))?;
        rendered_art.intro_cache.insert(key.clone(), frame);
    }
    let frame = rendered_art
        .intro_cache
        .get(&key)
        .cloned()
        .context("cached battle trainer art disappeared")?;
    let source_scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
    let display_size = frame.size * source_scale;
    let position = Vec3::new(
        top_left.x + display_size.x * 0.5,
        top_left.y - display_size.y * 0.5,
        top_left.z,
    );
    commands.spawn((
        SpriteBundle {
            texture: frame.handle.clone(),
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(display_size),
                ..default()
            },
            transform: Transform::from_translation(position),
            ..default()
        },
        BattleBattlerMarker,
    ));
    Ok(())
}

fn spawn_battler_marker(
    commands: &mut Commands,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
    species_id: &str,
    side: PokemonSpriteSide,
    hp: u16,
    _max_hp: u16,
    substitute: bool,
    shiny: bool,
    animation_frame: u16,
    position_offset: Vec3,
    battler_scale: f32,
    overlay: Option<([u8; 3], u8)>,
    art_override: VisibleBattlerArtOverride,
    clip_tiles: Option<u8>,
    remove_clip: Option<VisibleRemoveMonClip>,
    row_extraction: Option<VisibleBattlerRowExtraction>,
    base_visible: bool,
    line_offsets: Option<&VisibleBattleLineOffsets>,
) -> Result<()> {
    if hp == 0 {
        return Ok(());
    }
    let substitute = match art_override {
        VisibleBattlerArtOverride::Substitute => true,
        VisibleBattlerArtOverride::Pokemon => false,
        _ => substitute,
    };
    let minimize = art_override == VisibleBattlerArtOverride::Minimize;
    let frame = if minimize {
        battle_minimize_frame(rendered_art, asset_root, images)?.clone()
    } else if substitute {
        battle_substitute_frames(rendered_art, asset_root, images)?[match side {
            PokemonSpriteSide::Front => 0,
            PokemonSpriteSide::Back => 1,
        }]
        .clone()
    } else {
        pokemon_animation_frame_for_art(
            rendered_art,
            asset_root,
            species_id,
            side,
            shiny,
            animation_frame,
            images,
        )
        .with_context(|| {
            format!(
                "required battle Pokemon art {} {:?} could not be rendered: {}",
                species_id,
                side,
                pokemon_art_error(rendered_art, species_id, side, shiny)
            )
        })?
    };
    let frame = if side == PokemonSpriteSide::Front && !minimize && !substitute {
        battle_padded_frontpic(rendered_art, images, &frame)?
    } else {
        frame
    };
    let source_scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
    let native_size = if minimize || substitute {
        Vec2::splat(TILE_SIZE * 2.0)
    } else {
        frame.size * source_scale
    };
    let display_size = native_size * battler_scale;
    let (anchor_x, anchor_y) = match side {
        PokemonSpriteSide::Front => (PLAYFIELD_LEFT + TILE_SIZE * 12.0, PLAYFIELD_TOP),
        PokemonSpriteSide::Back => (
            PLAYFIELD_LEFT + TILE_SIZE * 2.0,
            PLAYFIELD_TOP - TILE_SIZE * 6.0,
        ),
    };
    // True send-out scaling and square pic-resize clipping both retain the
    // full native frame's center.
    let position = Vec3::new(
        anchor_x + native_size.x * 0.5,
        anchor_y - native_size.y * 0.5,
        3.0,
    ) + position_offset;
    let mut bgp_frames = HashMap::new();
    if let Some(bgps) = line_offsets.and_then(|offsets| offsets.bgp.as_ref()) {
        for bgp in bgps {
            if !bgp_frames.contains_key(bgp) {
                bgp_frames.insert(
                    *bgp,
                    battle_battler_bgp_frame(rendered_art, images, &frame, *bgp)?,
                );
            }
        }
    }
    if base_visible {
        spawn_battle_battler_texture(
            commands,
            &frame,
            display_size,
            position,
            Color::WHITE,
            clip_tiles,
            remove_clip,
            row_extraction,
            line_offsets,
            (!bgp_frames.is_empty()).then_some(&bgp_frames),
        );
    }
    if base_visible && let Some((colour, alpha)) = overlay {
        let overlay_frame = battle_battler_overlay_frame(rendered_art, images, &frame, colour)?;
        spawn_battle_battler_texture(
            commands,
            &overlay_frame,
            display_size,
            position + Vec3::new(0.0, 0.0, 0.01),
            Color::srgba_u8(255, 255, 255, alpha),
            clip_tiles,
            remove_clip,
            row_extraction,
            line_offsets,
            None,
        );
    }
    if let Some(extraction) = row_extraction.filter(|extraction| extraction.render_extracted) {
        spawn_visible_battler_extracted_rows(
            commands,
            &frame,
            display_size,
            position - position_offset,
            extraction,
        );
    }
    Ok(())
}

// engine/gfx/load_pics.asm:PadFrontpic writes columns into a 7x7 box.
// Both smaller sizes start one tile from the left and end at the bottom.
fn battle_padded_frontpic(
    art: &mut RenderedTilesetArt,
    images: &mut Assets<Image>,
    frame: &SpriteFrame,
) -> Result<SpriteFrame> {
    let width = frame.size.x as usize;
    if frame.size != Vec2::splat(40.0) && frame.size != Vec2::splat(48.0) {
        return Ok(frame.clone());
    }
    let key = IntroArtKey {
        asset_id: format!("battle-padded-front:{:?}", frame.handle.id()),
    };
    if let Some(cached) = art.intro_cache.get(&key) {
        return Ok(cached.clone());
    }
    let source = images
        .get(&frame.handle)
        .context("battle frontpic image is unavailable")?;
    let mut data = vec![0; 56 * 56 * 4];
    for row in 0..width {
        let target = ((row + 56 - width) * 56 + 8) * 4;
        data[target..target + width * 4]
            .copy_from_slice(&source.data[row * width * 4..(row + 1) * width * 4]);
    }
    let mut padded = Image::new(
        Extent3d {
            width: 56,
            height: 56,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    padded.sampler = ImageSampler::nearest();
    let padded = SpriteFrame {
        handle: images.add(padded),
        size: Vec2::splat(56.0),
    };
    art.intro_cache.insert(key, padded.clone());
    Ok(padded)
}

fn spawn_battle_battler_texture(
    commands: &mut Commands,
    frame: &SpriteFrame,
    display_size: Vec2,
    position: Vec3,
    colour: Color,
    clip_tiles: Option<u8>,
    remove_clip: Option<VisibleRemoveMonClip>,
    row_extraction: Option<VisibleBattlerRowExtraction>,
    line_offsets: Option<&VisibleBattleLineOffsets>,
    bgp_frames: Option<&HashMap<u8, SpriteFrame>>,
) {
    let Some(line_offsets) = line_offsets else {
        let (mut rect, mut visible_size) = if let Some(clip_tiles) = clip_tiles {
            let clip_source_pixels = f32::from(clip_tiles) * SOURCE_TILE_SIZE as f32;
            let visible_width = frame.size.x.min(clip_source_pixels);
            let visible_height = frame.size.y.min(clip_source_pixels);
            let source_x = (frame.size.x - visible_width) * 0.5;
            let source_y = (frame.size.y - visible_height) * 0.5;
            (
                Some(Rect::new(
                    source_x,
                    source_y,
                    source_x + visible_width,
                    source_y + visible_height,
                )),
                Vec2::new(
                    display_size.x * visible_width / frame.size.x,
                    display_size.y * visible_height / frame.size.y,
                ),
            )
        } else {
            (None, display_size)
        };
        let mut position = position;
        if let Some(remove_clip) = remove_clip {
            let current = rect.unwrap_or(Rect::new(0.0, 0.0, frame.size.x, frame.size.y));
            let cut = f32::from(remove_clip.source_pixels).min(current.width());
            let displayed_cut = visible_size.x * cut / current.width();
            let (source_left, source_right, position_adjustment) = if remove_clip.crop_left {
                (current.min.x + cut, current.max.x, displayed_cut * 0.5)
            } else {
                (current.min.x, current.max.x - cut, -displayed_cut * 0.5)
            };
            rect = Some(Rect::new(
                source_left,
                current.min.y,
                source_right,
                current.max.y,
            ));
            visible_size.x -= displayed_cut;
            position.x += position_adjustment;
        }
        if let Some(extraction) = row_extraction.filter(|extraction| extraction.bg_rows_cleared) {
            let current = rect.unwrap_or(Rect::new(0.0, 0.0, frame.size.x, frame.size.y));
            let extracted_height =
                (f32::from(extraction.rows) * SOURCE_TILE_SIZE as f32).min(frame.size.y);
            let (source_top, source_bottom) = if extraction.top {
                (current.min.y.max(extracted_height), current.max.y)
            } else {
                (
                    current.min.y,
                    current.max.y.min(frame.size.y - extracted_height),
                )
            };
            let retained_height = (source_bottom - source_top).max(0.0);
            let displayed_height = visible_size.y * retained_height / current.height();
            let removed_height = visible_size.y - displayed_height;
            rect = Some(Rect::new(
                current.min.x,
                source_top,
                current.max.x,
                source_bottom,
            ));
            visible_size.y = displayed_height;
            position.y += if extraction.top {
                -removed_height * 0.5
            } else {
                removed_height * 0.5
            };
        }
        if visible_size.x <= 0.0 || visible_size.y <= 0.0 {
            return;
        }
        commands.spawn((
            SpriteBundle {
                texture: frame.handle.clone(),
                sprite: Sprite {
                    color: colour,
                    rect,
                    custom_size: Some(visible_size),
                    ..default()
                },
                transform: Transform::from_translation(position),
                ..default()
            },
            BattleBattlerMarker,
        ));
        return;
    };
    let scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
    let horizontal_rect = remove_clip.map(|remove_clip| {
        let cut = f32::from(remove_clip.source_pixels).min(frame.size.x);
        if remove_clip.crop_left {
            Rect::new(cut, 0.0, frame.size.x, frame.size.y)
        } else {
            Rect::new(0.0, 0.0, frame.size.x - cut, frame.size.y)
        }
    });
    let horizontal_width = horizontal_rect.map_or(frame.size.x, |rect| rect.width());
    if horizontal_width <= 0.0 {
        return;
    }
    let display_height = display_size.y / scale;
    let center_line = (PLAYFIELD_TOP - position.y) / scale;
    let top_line = center_line - display_height / 2.0;
    let first_line = top_line.floor() as i32;
    let last_line = (top_line + display_height).ceil() as i32;
    for line in first_line..last_line {
        if !(0..0x5f).contains(&line) {
            continue;
        }
        let line_frame = line_offsets
            .bgp
            .as_ref()
            .and_then(|bgps| bgp_frames?.get(&bgps[line as usize]))
            .unwrap_or(frame);
        let sampled_line = line as f32 + f32::from(line_offsets.y[line as usize]);
        let source_top_ratio = (sampled_line - top_line) / display_height;
        let source_bottom_ratio = (sampled_line + 1.0 - top_line) / display_height;
        if source_bottom_ratio <= 0.0 || source_top_ratio >= 1.0 {
            continue;
        }
        let source_top = source_top_ratio.clamp(0.0, 1.0) * frame.size.y;
        let source_bottom = source_bottom_ratio.clamp(0.0, 1.0) * frame.size.y;
        if source_bottom <= source_top {
            continue;
        }
        if row_extraction.is_some_and(|extraction| {
            if !extraction.bg_rows_cleared {
                return false;
            }
            let height = f32::from(extraction.rows) * SOURCE_TILE_SIZE as f32;
            let (extract_top, extract_bottom) = if extraction.top {
                (0.0, height)
            } else {
                (frame.size.y - height, frame.size.y)
            };
            source_top < extract_bottom && source_bottom > extract_top
        }) {
            continue;
        }
        commands.spawn((
            SpriteBundle {
                texture: line_frame.handle.clone(),
                sprite: Sprite {
                    color: colour,
                    rect: Some(Rect::new(
                        horizontal_rect.map_or(0.0, |rect| rect.min.x),
                        source_top,
                        horizontal_rect.map_or(frame.size.x, |rect| rect.max.x),
                        source_bottom,
                    )),
                    custom_size: Some(Vec2::new(
                        display_size.x * horizontal_width / frame.size.x,
                        scale,
                    )),
                    ..default()
                },
                transform: Transform::from_xyz(
                    position.x
                        + f32::from(line_offsets.x[line as usize]) * scale
                        + remove_clip.map_or(0.0, |clip| {
                            let displayed_cut = display_size.x
                                * f32::from(clip.source_pixels).min(frame.size.x)
                                / frame.size.x;
                            if clip.crop_left {
                                displayed_cut * 0.5
                            } else {
                                -displayed_cut * 0.5
                            }
                        }),
                    PLAYFIELD_TOP - (line as f32 + 0.5) * scale,
                    position.z,
                ),
                ..default()
            },
            BattleBattlerMarker,
        ));
    }
}

fn spawn_visible_battler_extracted_rows(
    commands: &mut Commands,
    frame: &SpriteFrame,
    display_size: Vec2,
    position: Vec3,
    extraction: VisibleBattlerRowExtraction,
) {
    let source_height = (f32::from(extraction.rows) * SOURCE_TILE_SIZE as f32).min(frame.size.y);
    if source_height <= 0.0 {
        return;
    }
    let display_height = display_size.y * source_height / frame.size.y;
    let (source_top, source_bottom, y) = if extraction.top {
        (
            0.0,
            source_height,
            position.y + (display_size.y - display_height) * 0.5,
        )
    } else {
        (
            frame.size.y - source_height,
            frame.size.y,
            position.y - (display_size.y - display_height) * 0.5,
        )
    };
    commands.spawn((
        SpriteBundle {
            texture: frame.handle.clone(),
            sprite: Sprite {
                color: Color::WHITE,
                rect: Some(Rect::new(0.0, source_top, frame.size.x, source_bottom)),
                custom_size: Some(Vec2::new(display_size.x, display_height)),
                ..default()
            },
            transform: Transform::from_xyz(position.x, y, position.z + 0.02),
            ..default()
        },
        BattleCommandMarker,
    ));
}

fn battle_minimize_frame<'a>(
    rendered_art: &'a mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<&'a SpriteFrame> {
    if rendered_art.battle_minimize_cache.is_none() && rendered_art.battle_minimize_error.is_none()
    {
        let loaded = (|| -> Result<SpriteFrame> {
            let path = asset_root.runtime_assets().join("gfx/battle/minimize.png");
            let source = crate::open_runtime_image(&path)
                .with_context(|| format!("decode Minimize battle PNG {}", path.display()))?
                .to_rgba8();
            let (width, height) = source.dimensions();
            if width == 0 || height == 0 {
                anyhow::bail!("Minimize battle PNG {} is empty", path.display());
            }
            let mut image = Image::new(
                Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                source.into_raw(),
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            );
            image.sampler = ImageSampler::nearest();
            Ok(SpriteFrame {
                handle: images.add(image),
                size: Vec2::new(width as f32, height as f32),
            })
        })();
        match loaded {
            Ok(frame) => rendered_art.battle_minimize_cache = Some(frame),
            Err(error) => rendered_art.battle_minimize_error = Some(error.to_string()),
        }
    }
    rendered_art
        .battle_minimize_cache
        .as_ref()
        .with_context(|| {
            rendered_art
                .battle_minimize_error
                .clone()
                .unwrap_or_else(|| "Minimize battle art is unavailable".to_string())
        })
}

fn battle_battler_overlay_frame(
    rendered_art: &mut RenderedTilesetArt,
    images: &mut Assets<Image>,
    source: &SpriteFrame,
    colour: [u8; 3],
) -> Result<SpriteFrame> {
    let key = (source.handle.id(), colour);
    if let Some(frame) = rendered_art.battle_battler_overlay_cache.get(&key) {
        return Ok(frame.clone());
    }
    let mut image = images
        .get(&source.handle)
        .with_context(|| "battle battler overlay source image is unavailable")?
        .clone();
    for pixel in image.data.chunks_exact_mut(4) {
        if pixel[3] == 0 {
            continue;
        }
        pixel[0] = colour[0];
        pixel[1] = colour[1];
        pixel[2] = colour[2];
    }
    image.sampler = ImageSampler::nearest();
    let frame = SpriteFrame {
        handle: images.add(image),
        size: source.size,
    };
    rendered_art
        .battle_battler_overlay_cache
        .insert(key, frame.clone());
    Ok(frame)
}

fn battle_battler_bgp_frame(
    rendered_art: &mut RenderedTilesetArt,
    images: &mut Assets<Image>,
    source: &SpriteFrame,
    bgp: u8,
) -> Result<SpriteFrame> {
    let key = (source.handle.id(), bgp);
    if let Some(frame) = rendered_art.battle_battler_bgp_cache.get(&key) {
        return Ok(frame.clone());
    }
    let mut image = images
        .get(&source.handle)
        .context("battle battler BGP source image is unavailable")?
        .clone();
    let mut colours = image
        .data
        .chunks_exact(4)
        .filter(|pixel| pixel[3] != 0)
        .map(|pixel| [pixel[0], pixel[1], pixel[2]])
        .collect::<Vec<_>>();
    colours.sort_unstable();
    colours.dedup();
    anyhow::ensure!(
        colours.len() <= 4,
        "battle battler BGP source has {} opaque colours instead of at most four",
        colours.len()
    );
    colours.sort_by_key(|colour| {
        std::cmp::Reverse(
            u32::from(colour[0]) * 299 + u32::from(colour[1]) * 587 + u32::from(colour[2]) * 114,
        )
    });
    let transparent_shade_zero = colours.len() < 4;
    let shade_zero = image
        .data
        .chunks_exact(4)
        .find(|pixel| pixel[3] == 0)
        .map(|pixel| [pixel[0], pixel[1], pixel[2]])
        .unwrap_or([255, 255, 255]);
    let source_colours = colours.clone();
    for pixel in image.data.chunks_exact_mut(4) {
        if pixel[3] == 0 {
            continue;
        }
        let source_colour = [pixel[0], pixel[1], pixel[2]];
        let source_index = source_colours
            .iter()
            .position(|colour| *colour == source_colour)
            .context("battle battler BGP source colour disappeared")?
            + usize::from(transparent_shade_zero);
        let mapped_index = usize::from((bgp >> (source_index * 2)) & 3);
        if transparent_shade_zero && mapped_index == 0 {
            pixel[..3].copy_from_slice(&shade_zero);
            pixel[3] = 255;
        } else {
            let colour_index = mapped_index.saturating_sub(usize::from(transparent_shade_zero));
            let mapped = source_colours[colour_index.min(source_colours.len() - 1)];
            pixel[..3].copy_from_slice(&mapped);
        }
    }
    image.sampler = ImageSampler::nearest();
    let frame = SpriteFrame {
        handle: images.add(image),
        size: source.size,
    };
    rendered_art
        .battle_battler_bgp_cache
        .insert(key, frame.clone());
    Ok(frame)
}

fn visible_pokemon_is_shiny(pokemon: &crate::core::models::pokemon::Pokemon) -> bool {
    pokemon.dvs.defense == 10
        && pokemon.dvs.speed == 10
        && pokemon.dvs.special == 10
        && matches!(pokemon.dvs.attack, 2 | 3 | 6 | 7 | 10 | 11 | 14 | 15)
}

fn battle_substitute_frames<'a>(
    rendered_art: &'a mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<&'a [SpriteFrame; 2]> {
    if rendered_art.battle_substitute_cache.is_none()
        && rendered_art.battle_substitute_error.is_none()
    {
        let loaded = (|| -> Result<[SpriteFrame; 2]> {
            let path = asset_root.runtime_assets().join("gfx/sprites/monster.png");
            let source = crate::open_runtime_image(&path)
                .with_context(|| format!("decode substitute source PNG {}", path.display()))?
                .to_rgba8();
            if source.width() != 16 || source.height() < 32 {
                anyhow::bail!(
                    "substitute source PNG {} must contain two 16x16 frames, found {}x{}",
                    path.display(),
                    source.width(),
                    source.height()
                );
            }
            let mut load_frame = |frame_index: u32| {
                let mut pixels = vec![0; 16 * 16 * 4];
                for y in 0..16u32 {
                    for x in 0..16u32 {
                        let pixel = source.get_pixel(x, frame_index * 16 + y);
                        let output = ((y * 16 + x) * 4) as usize;
                        if pixel[0] < 240 || pixel[1] < 240 || pixel[2] < 240 {
                            pixels[output] = pixel[0];
                            pixels[output + 1] = pixel[1];
                            pixels[output + 2] = pixel[2];
                            pixels[output + 3] = 255;
                        }
                    }
                }
                let mut image = Image::new(
                    Extent3d {
                        width: 16,
                        height: 16,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    pixels,
                    TextureFormat::Rgba8UnormSrgb,
                    RenderAssetUsages::default(),
                );
                image.sampler = ImageSampler::nearest();
                SpriteFrame {
                    handle: images.add(image),
                    size: Vec2::splat(16.0),
                }
            };
            Ok([load_frame(0), load_frame(1)])
        })();
        match loaded {
            Ok(frames) => rendered_art.battle_substitute_cache = Some(frames),
            Err(error) => rendered_art.battle_substitute_error = Some(error.to_string()),
        }
    }
    rendered_art
        .battle_substitute_cache
        .as_ref()
        .with_context(|| {
            rendered_art
                .battle_substitute_error
                .clone()
                .unwrap_or_else(|| "battle substitute art is unavailable".to_string())
        })
}

fn spawn_battle_hud(
    commands: &mut Commands,
    snapshot: &RuntimeShellSnapshot,
    battle: &crate::RuntimeBattleSnapshot,
    entry_messages_remaining: usize,
    enemy_send_out_pending: bool,
    player_send_out_pending: bool,
    trainer_exit_active: bool,
    hp_tween: Option<&VisibleBattleHpTween>,
    exp_tween: Option<&VisibleBattleExpTween>,
    growth_rates: &crate::core::systems::experience::GrowthRateCatalog,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<()> {
    require_bitmap_font_art(rendered_art, asset_root, images)?;
    if enemy_send_out_pending {
        spawn_battle_hud_borders(
            commands,
            rendered_art,
            asset_root,
            images,
            false,
            true,
            false,
        )?;
        let active_index = battle
            .active_player_party_index
            .context("enemy send-out HUD requires an active player party slot")?;
        let slot = snapshot
            .party
            .slots
            .iter()
            .find(|slot| slot.index == active_index)
            .with_context(|| {
                format!("active player party slot {active_index} is absent from the send-out HUD")
            })?;
        spawn_battle_hud_side(
            commands,
            rendered_art,
            asset_root,
            images,
            &slot.pokemon,
            BattleHpSide::Player,
            hp_tween.map(|tween| tween.player_pixels),
            hp_tween.map(|tween| tween.player_hp),
        )?;
        return Ok(());
    }
    if player_send_out_pending {
        spawn_battle_hud_borders(
            commands,
            rendered_art,
            asset_root,
            images,
            true,
            false,
            false,
        )?;
        spawn_battle_hud_side(
            commands,
            rendered_art,
            asset_root,
            images,
            &battle.enemy_pokemon,
            BattleHpSide::Enemy,
            hp_tween.map(|tween| tween.enemy_pixels),
            None,
        )?;
        return Ok(());
    }
    let trainer_enemy_pending =
        matches!(battle.kind, RuntimeBattleKind::Trainer { .. }) && entry_messages_remaining >= 2;
    let player_pokemon_visible =
        battle.battle_type != "BATTLETYPE_TUTORIAL" && entry_messages_remaining == 0;
    let party_hud_visible = match &battle.kind {
        RuntimeBattleKind::Trainer { .. } => entry_messages_remaining >= 3 || trainer_exit_active,
        RuntimeBattleKind::Wild { .. } | RuntimeBattleKind::StaticWild { .. } => {
            if battle.battle_type == "BATTLETYPE_TUTORIAL" {
                entry_messages_remaining > 0
            } else {
                entry_messages_remaining >= 2
            }
        }
    };
    if party_hud_visible {
        spawn_battle_hud_borders(commands, rendered_art, asset_root, images, true, true, true)?;
        spawn_battle_party_balls(commands, snapshot, battle, rendered_art, asset_root, images)?;
    } else if !trainer_enemy_pending {
        spawn_battle_hud_borders(
            commands,
            rendered_art,
            asset_root,
            images,
            true,
            player_pokemon_visible,
            false,
        )?;
    }
    if trainer_enemy_pending {
        return Ok(());
    }
    if !matches!(battle.kind, RuntimeBattleKind::Trainer { .. })
        && snapshot
            .progression
            .pokedex_caught_species
            .contains(&battle.enemy_pokemon.species.id)
    {
        spawn_battle_caught_icon(commands, rendered_art, asset_root, images);
    }
    spawn_battle_hud_side(
        commands,
        rendered_art,
        asset_root,
        images,
        &battle.enemy_pokemon,
        BattleHpSide::Enemy,
        hp_tween.map(|tween| tween.enemy_pixels),
        None,
    )?;
    if !player_pokemon_visible {
        return Ok(());
    }
    let active_index = battle
        .active_player_party_index
        .context("visible player battle HUD requires an active party slot")?;
    let slot = snapshot
        .party
        .slots
        .iter()
        .find(|slot| slot.index == active_index)
        .with_context(|| {
            format!("active player party slot {active_index} is absent from the HUD")
        })?;
    let mut display_pokemon = slot.pokemon.clone();
    if let Some(tween) = exp_tween {
        display_pokemon.level = tween.level;
    }
    spawn_battle_hud_side(
        commands,
        rendered_art,
        asset_root,
        images,
        &display_pokemon,
        BattleHpSide::Player,
        hp_tween.map(|tween| tween.player_pixels),
        hp_tween.map(|tween| tween.player_hp),
    )?;
    spawn_battle_exp_bar(
        commands,
        &display_pokemon,
        growth_rates,
        rendered_art,
        asset_root,
        images,
        exp_tween.map(|tween| tween.pixels),
    )?;
    Ok(())
}

fn spawn_battle_party_balls(
    commands: &mut Commands,
    snapshot: &RuntimeShellSnapshot,
    battle: &crate::RuntimeBattleSnapshot,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<()> {
    let frames = battle_party_ball_frames(rendered_art, asset_root, images)?;
    let tile_for = |pokemon: Option<&crate::core::models::Pokemon>| match pokemon {
        None => 3,
        Some(pokemon) if pokemon.hp == 0 => 2,
        Some(pokemon) if battle_status_token(pokemon.status.as_deref()).is_some() => 1,
        Some(_) => 0,
    };
    let mut spawn = |frame_index: usize, tile_x: f32, tile_y: f32| {
        let frame = &frames[frame_index];
        let (x, y) = battle_hud_tile_origin(tile_x, tile_y);
        commands.spawn((
            SpriteBundle {
                texture: frame.handle.clone(),
                sprite: Sprite {
                    custom_size: Some(frame.size),
                    ..default()
                },
                transform: Transform::from_xyz(x, y, 3.75),
                ..default()
            },
            BattleHudMarker,
            BattleCommandMarker,
        ));
    };
    for index in 0..6 {
        spawn(
            tile_for(snapshot.party.slots.get(index).map(|slot| &slot.pokemon)),
            11.0 + index as f32,
            10.0,
        );
    }
    if matches!(battle.kind, RuntimeBattleKind::Trainer { .. }) {
        for index in 0..6 {
            spawn(
                tile_for(battle.enemy_party.get(index)),
                8.0 - index as f32,
                2.0,
            );
        }
    }
    Ok(())
}

fn battle_party_ball_frames<'a>(
    rendered_art: &'a mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<&'a [SpriteFrame; 4]> {
    if rendered_art.battle_party_ball_cache.is_none()
        && rendered_art.battle_party_ball_error.is_none()
    {
        let loaded = (|| -> Result<[SpriteFrame; 4]> {
            let path = asset_root.runtime_assets().join("gfx/battle/balls.2bpp");
            let data = crate::read_runtime_asset(&path)
                .with_context(|| format!("read battle party-ball graphics {}", path.display()))?;
            if data.len() != 4 * 16 {
                anyhow::bail!(
                    "battle party-ball graphics {} must contain four tiles, found {} bytes",
                    path.display(),
                    data.len()
                );
            }
            let palette = load_battle_anim_palette(asset_root, "yellow")?;
            let mut load = |tile_index: usize| {
                let tile = &data[tile_index * 16..tile_index * 16 + 16];
                let mut pixels = vec![0_u8; SOURCE_TILE_SIZE * SOURCE_TILE_SIZE * 4];
                for row in 0..SOURCE_TILE_SIZE {
                    let lo = tile[row * 2];
                    let hi = tile[row * 2 + 1];
                    for col in 0..SOURCE_TILE_SIZE {
                        let bit = 1 << (7 - col);
                        let level = ((hi & bit != 0) as u8) << 1 | (lo & bit != 0) as u8;
                        if level == 0 {
                            continue;
                        }
                        let offset = (row * SOURCE_TILE_SIZE + col) * 4;
                        pixels[offset..offset + 4].copy_from_slice(&palette[usize::from(level)]);
                    }
                }
                let mut image = Image::new(
                    Extent3d {
                        width: SOURCE_TILE_SIZE as u32,
                        height: SOURCE_TILE_SIZE as u32,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    pixels,
                    TextureFormat::Rgba8UnormSrgb,
                    RenderAssetUsages::default(),
                );
                image.sampler = ImageSampler::nearest();
                SpriteFrame {
                    handle: images.add(image),
                    size: Vec2::splat(TILE_SIZE),
                }
            };
            Ok([load(0), load(1), load(2), load(3)])
        })();
        match loaded {
            Ok(frames) => rendered_art.battle_party_ball_cache = Some(frames),
            Err(error) => rendered_art.battle_party_ball_error = Some(error.to_string()),
        }
    }
    rendered_art
        .battle_party_ball_cache
        .as_ref()
        .with_context(|| {
            rendered_art
                .battle_party_ball_error
                .clone()
                .unwrap_or_else(|| "battle party-ball art is unavailable".to_string())
        })
}

fn spawn_battle_hud_borders(
    commands: &mut Commands,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
    enemy_visible: bool,
    player_visible: bool,
    player_party_icons: bool,
) -> Result<()> {
    let player_party_corner = if player_party_icons {
        Some(
            battle_exp_bar_tiles(rendered_art, asset_root, images)?
                .get(&0x5c)
                .context("battle party HUD corner tile $5c was not loaded")?
                .clone(),
        )
    } else {
        None
    };
    let mut tiles = battle_hud_border_tiles(rendered_art, asset_root, images)?.clone();
    if let Some(frame) = player_party_corner {
        tiles.insert(0x5c, frame);
    }
    let mut place = |tile_id: u8, tile_x: f32, tile_y: f32| -> Result<()> {
        let frame = tiles
            .get(&tile_id)
            .with_context(|| format!("battle HUD border tile ${tile_id:02x} was not loaded"))?;
        let (x, y) = battle_hud_tile_origin(tile_x, tile_y);
        commands.spawn((
            SpriteBundle {
                texture: frame.handle.clone(),
                sprite: Sprite {
                    custom_size: Some(frame.size),
                    ..default()
                },
                transform: Transform::from_xyz(x, y, 3.5),
                ..default()
            },
            BattleHudMarker,
            BattleCommandMarker,
        ));
        Ok(())
    };

    if enemy_visible {
        // DrawEnemyHUD: left elbow, lower-left corner, horizontal run, lower-right corner.
        place(0x6d, 1.0, 2.0)?;
        place(0x74, 1.0, 3.0)?;
        for x in 2..10 {
            place(0x76, x as f32, 3.0)?;
        }
        place(0x78, 10.0, 3.0)?;
    }

    if player_visible {
        // DrawPlayerHUD starts its border at (18,10); the HP bar contributes
        // its own distinct end tile at (18,9).
        place(0x73, 18.0, 10.0)?;
        place(if player_party_icons { 0x5c } else { 0x77 }, 18.0, 11.0)?;
        for x in 10..18 {
            place(0x76, x as f32, 11.0)?;
        }
        place(0x6f, 9.0, 11.0)?;
    }
    Ok(())
}

fn battle_hud_border_tiles<'a>(
    rendered_art: &'a mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<&'a HashMap<u8, SpriteFrame>> {
    if rendered_art.battle_hud_border_cache.is_none()
        && rendered_art.battle_hud_border_error.is_none()
    {
        let loaded = (|| -> Result<HashMap<u8, SpriteFrame>> {
            let battle_root = asset_root.runtime_assets().join("gfx/battle");
            let sources = [
                ("enemy_hp_bar_border.png", 0x6cu8, 4u32),
                ("hp_exp_bar_border.png", 0x73u8, 6u32),
            ];
            let mut result = HashMap::new();
            for (name, start_tile, tile_count) in sources {
                let path = battle_root.join(name);
                let source = crate::open_runtime_image(&path)
                    .with_context(|| format!("decode battle HUD PNG {}", path.display()))?
                    .to_rgba8();
                let (width, height) = source.dimensions();
                if width != tile_count * SOURCE_TILE_SIZE as u32
                    || height != SOURCE_TILE_SIZE as u32
                {
                    anyhow::bail!(
                        "battle HUD PNG {} must be {}x{}, found {}x{}",
                        path.display(),
                        tile_count * SOURCE_TILE_SIZE as u32,
                        SOURCE_TILE_SIZE,
                        width,
                        height
                    );
                }
                for tile_offset in 0..tile_count {
                    let mut pixels = vec![0; (SOURCE_TILE_SIZE * SOURCE_TILE_SIZE * 4) as usize];
                    for y in 0..SOURCE_TILE_SIZE as u32 {
                        for x in 0..SOURCE_TILE_SIZE as u32 {
                            let pixel =
                                source.get_pixel(tile_offset * SOURCE_TILE_SIZE as u32 + x, y);
                            let output = ((y * SOURCE_TILE_SIZE as u32 + x) * 4) as usize;
                            let transparent = pixel[0] > 240 && pixel[1] > 240 && pixel[2] > 240;
                            if !transparent {
                                pixels[output] = pixel[0];
                                pixels[output + 1] = pixel[1];
                                pixels[output + 2] = pixel[2];
                                pixels[output + 3] = 255;
                            }
                        }
                    }
                    let mut image = Image::new(
                        Extent3d {
                            width: SOURCE_TILE_SIZE as u32,
                            height: SOURCE_TILE_SIZE as u32,
                            depth_or_array_layers: 1,
                        },
                        TextureDimension::D2,
                        pixels,
                        TextureFormat::Rgba8UnormSrgb,
                        RenderAssetUsages::default(),
                    );
                    image.sampler = ImageSampler::nearest();
                    result.insert(
                        start_tile + tile_offset as u8,
                        SpriteFrame {
                            handle: images.add(image),
                            size: Vec2::splat(TILE_SIZE),
                        },
                    );
                }
            }
            Ok(result)
        })();
        match loaded {
            Ok(tiles) => rendered_art.battle_hud_border_cache = Some(tiles),
            Err(error) => rendered_art.battle_hud_border_error = Some(error.to_string()),
        }
    }
    rendered_art
        .battle_hud_border_cache
        .as_ref()
        .with_context(|| {
            rendered_art
                .battle_hud_border_error
                .clone()
                .unwrap_or_else(|| "battle HUD border art is unavailable".to_string())
        })
}

fn spawn_battle_caught_icon(
    commands: &mut Commands,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) {
    // The caught marker is literal font tile $5d. The <TRAINER> text command
    // expands to seven letters and must not select this contextual tile.
    spawn_battle_hud_bitmap_text(
        commands,
        rendered_art,
        asset_root,
        images,
        "\u{e110}",
        1.0,
        1.0,
        3.7,
    );
}

fn spawn_battle_exp_bar(
    commands: &mut Commands,
    pokemon: &crate::core::models::pokemon::Pokemon,
    growth_rates: &crate::core::systems::experience::GrowthRateCatalog,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
    fill_override: Option<u16>,
) -> Result<()> {
    const EXP_BAR_PIXELS: i32 = 64;
    let fill_pixels = if let Some(fill_pixels) = fill_override {
        i32::from(fill_pixels)
    } else if pokemon.level >= 100 {
        0
    } else {
        let level = pokemon.level.clamp(1, 99);
        let current_level_exp = crate::core::systems::experience::calculate_experience(
            growth_rates,
            &pokemon.species.growth_rate,
            level,
        )?;
        let next_level_exp = crate::core::systems::experience::calculate_experience(
            growth_rates,
            &pokemon.species.growth_rate,
            level + 1,
        )?;
        let span = (next_level_exp - current_level_exp).max(1);
        let capped_exp = pokemon.experience.clamp(current_level_exp, next_level_exp);
        let remaining = (next_level_exp - capped_exp).max(0);
        EXP_BAR_PIXELS - ((remaining * EXP_BAR_PIXELS) / span).clamp(0, EXP_BAR_PIXELS)
    };
    let tiles = battle_exp_bar_tiles(rendered_art, asset_root, images)?;
    let full_tiles = fill_pixels / SOURCE_TILE_SIZE as i32;
    let remainder = fill_pixels % SOURCE_TILE_SIZE as i32;
    for index in 0..8 {
        let tile_id = if index < full_tiles {
            0x6a
        } else if index == full_tiles && remainder > 0 {
            0x54 + remainder as u8
        } else {
            0x62
        };
        let frame = tiles
            .get(&tile_id)
            .with_context(|| format!("battle EXP tile ${tile_id:02x} was not loaded"))?;
        let tile_x = 17.0 - index as f32;
        let (x, y) = battle_hud_tile_origin(tile_x, 11.0);
        commands.spawn((
            SpriteBundle {
                texture: frame.handle.clone(),
                sprite: Sprite {
                    custom_size: Some(frame.size),
                    ..default()
                },
                transform: Transform::from_xyz(x, y, 3.62),
                ..default()
            },
            BattleHudMarker,
            BattleCommandMarker,
        ));
    }
    Ok(())
}

fn battle_exp_bar_tiles<'a>(
    rendered_art: &'a mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<&'a HashMap<u8, SpriteFrame>> {
    if rendered_art.battle_exp_bar_cache.is_none() && rendered_art.battle_exp_bar_error.is_none() {
        let loaded = (|| -> Result<HashMap<u8, SpriteFrame>> {
            let assets = asset_root.runtime_assets();
            let exp = crate::read_runtime_asset(assets.join("gfx/battle/expbar.2bpp"))
                .context("read battle EXP partial-tile graphics")?;
            let battle_font =
                crate::read_runtime_asset(assets.join("gfx/font/font_battle_extra.2bpp"))
                    .context("read battle HP/EXP template graphics")?;
            let mut result = HashMap::new();
            for (tile_id, data, source_index) in [
                (0x62_u8, battle_font.as_slice(), 2_usize),
                (0x6a_u8, battle_font.as_slice(), 10_usize),
            ]
            .into_iter()
            .chain((0..8).map(|index| (0x55 + index as u8, exp.as_slice(), index)))
            {
                let offset = source_index * 16;
                let tile = data
                    .get(offset..offset + 16)
                    .with_context(|| format!("battle EXP source tile {source_index} is missing"))?;
                let mut pixels = vec![0_u8; SOURCE_TILE_SIZE * SOURCE_TILE_SIZE * 4];
                for row in 0..SOURCE_TILE_SIZE {
                    let lo = tile[row * 2];
                    let hi = tile[row * 2 + 1];
                    for col in 0..SOURCE_TILE_SIZE {
                        let bit = 1 << (7 - col);
                        let level = ((hi & bit != 0) as u8) << 1 | (lo & bit != 0) as u8;
                        if level == 0 {
                            continue;
                        }
                        let color = match level {
                            1 => [247, 214, 123],
                            2 => [33, 140, 255],
                            _ => [0, 0, 0],
                        };
                        let target = (row * SOURCE_TILE_SIZE + col) * 4;
                        pixels[target..target + 3].copy_from_slice(&color);
                        pixels[target + 3] = 255;
                    }
                }
                let mut image = Image::new(
                    Extent3d {
                        width: SOURCE_TILE_SIZE as u32,
                        height: SOURCE_TILE_SIZE as u32,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    pixels,
                    TextureFormat::Rgba8UnormSrgb,
                    RenderAssetUsages::default(),
                );
                image.sampler = ImageSampler::nearest();
                result.insert(
                    tile_id,
                    SpriteFrame {
                        handle: images.add(image),
                        size: Vec2::splat(TILE_SIZE),
                    },
                );
            }
            Ok(result)
        })();
        match loaded {
            Ok(tiles) => rendered_art.battle_exp_bar_cache = Some(tiles),
            Err(error) => rendered_art.battle_exp_bar_error = Some(error.to_string()),
        }
    }
    rendered_art.battle_exp_bar_cache.as_ref().with_context(|| {
        rendered_art
            .battle_exp_bar_error
            .clone()
            .unwrap_or_else(|| "battle EXP bar art is unavailable".to_string())
    })
}

fn spawn_battle_hud_side(
    commands: &mut Commands,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
    pokemon: &crate::core::models::pokemon::Pokemon,
    side: BattleHpSide,
    hp_pixel_override: Option<u16>,
    hp_value_override: Option<u16>,
) -> Result<()> {
    let (
        name_tile_x,
        name_tile_y,
        level_tile_x,
        level_tile_y,
        status_tile_x,
        status_tile_y,
        hp_tile_x,
        hp_tile_y,
    ) = match side {
        BattleHpSide::Enemy => (1.0, 0.0, 6.0, 1.0, 6.0, 1.0, 2.0, 2.0),
        BattleHpSide::Player => (10.0, 7.0, 14.0, 8.0, 14.0, 8.0, 10.0, 9.0),
    };
    spawn_battle_hud_bitmap_text(
        commands,
        rendered_art,
        asset_root,
        images,
        &compact_scene_label(&pokemon.nickname, 10),
        name_tile_x,
        name_tile_y,
        3.7,
    );
    let status = battle_status_token(pokemon.status.as_deref());
    if let Some(raw_status) = pokemon.status.as_deref() {
        anyhow::ensure!(
            status.is_some()
                || ["", "OK", "NONE", "HEALTHY", "CONFUSION", "CNF"]
                    .iter()
                    .any(|token| raw_status.eq_ignore_ascii_case(token)),
            "battle HUD has unknown status {raw_status} for {}",
            pokemon.nickname
        );
    }
    let gender = crate::core::battle::turn::battle_pokemon_gender(pokemon);
    let level_or_status = status
        .map(str::to_string)
        .unwrap_or_else(|| format!("\u{e10a}{}", pokemon.level));
    // PrintPlayerHUD/DrawEnemyHUD decrement the level destination for a
    // genderless battler. Status text keeps the fixed three-tile position.
    let (level_or_status_tile_x, level_or_status_tile_y) = if status.is_some() {
        (status_tile_x, status_tile_y)
    } else if gender.is_none() {
        (level_tile_x - 1.0, level_tile_y)
    } else {
        (level_tile_x, level_tile_y)
    };
    spawn_battle_hud_bitmap_text(
        commands,
        rendered_art,
        asset_root,
        images,
        &level_or_status,
        level_or_status_tile_x,
        level_or_status_tile_y,
        3.7,
    );
    if let Some(gender) = gender {
        let (gender_tile_x, gender_tile_y) = match side {
            BattleHpSide::Enemy => (9.0, 1.0),
            BattleHpSide::Player => (17.0, 8.0),
        };
        let glyph = match gender {
            crate::core::battle::turn::BattlePokemonGender::Male => "♂",
            crate::core::battle::turn::BattlePokemonGender::Female => "♀",
        };
        spawn_battle_hud_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            glyph,
            gender_tile_x,
            gender_tile_y,
            3.7,
        );
    }
    spawn_battle_hud_hp_bar(
        commands,
        rendered_art,
        asset_root,
        images,
        hp_tile_x,
        hp_tile_y,
        pokemon.hp,
        pokemon.max_hp,
        side,
        hp_pixel_override,
    )?;
    if side == BattleHpSide::Player {
        let displayed_hp = hp_value_override.unwrap_or(pokemon.hp);
        spawn_battle_hud_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            &format!(
                "{:>3}/{:>3}",
                displayed_hp.min(999),
                pokemon.max_hp.min(999)
            ),
            hp_tile_x + 1.0,
            hp_tile_y + 1.0,
            3.7,
        );
    }
    Ok(())
}

fn spawn_battle_hud_bitmap_text(
    commands: &mut Commands,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
    text: &str,
    tile_x: f32,
    tile_y: f32,
    z: f32,
) {
    let (x, y) = battle_hud_tile_origin(tile_x, tile_y);
    for (index, frame) in bitmap_text_frames(rendered_art, asset_root, images, text)
        .into_iter()
        .enumerate()
    {
        commands.spawn((
            SpriteBundle {
                texture: frame.handle,
                sprite: Sprite {
                    custom_size: Some(frame.size),
                    ..default()
                },
                transform: Transform::from_xyz(x + index as f32 * BITMAP_FONT_ADVANCE, y, z),
                ..default()
            },
            BattleHudMarker,
            BattleCommandMarker,
        ));
    }
}

fn spawn_battle_hud_hp_bar(
    commands: &mut Commands,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
    tile_x: f32,
    tile_y: f32,
    hp: u16,
    max_hp: u16,
    side: BattleHpSide,
    hp_pixel_override: Option<u16>,
) -> Result<()> {
    let tiles = battle_hp_bar_tiles(rendered_art, asset_root, images)?;
    let fill_pixels = hp_pixel_override
        .unwrap_or_else(|| battle_hud_hp_pixels(hp, max_hp))
        .min(BATTLE_HUD_HP_BAR_LENGTH_PX);
    // HPBarAnim_PaletteUpdate passes the currently drawn pixel length to
    // SetHPPal.  The authoritative HP value is already committed while the
    // bar drains, so deriving the palette from `hp` changes color too early.
    let zone = visible_hp_zone(fill_pixels);
    for (offset, tile_id) in [0x60_u8, 0x61].into_iter().enumerate() {
        let frame = tiles
            .get(&(tile_id, zone))
            .with_context(|| format!("battle HP label tile ${tile_id:02x} was not loaded"))?;
        let (x, y) = battle_hud_tile_origin(tile_x + offset as f32, tile_y);
        commands.spawn((
            SpriteBundle {
                texture: frame.handle.clone(),
                sprite: Sprite {
                    custom_size: Some(frame.size),
                    ..default()
                },
                transform: Transform::from_xyz(x, y, 3.7),
                ..default()
            },
            BattleHudMarker,
            BattleCommandMarker,
        ));
    }
    let full_tiles = fill_pixels / SOURCE_TILE_SIZE as u16;
    let remainder = fill_pixels % SOURCE_TILE_SIZE as u16;
    for index in 0..BATTLE_HUD_HP_BAR_LENGTH_TILES as u16 {
        let tile_id = if index < full_tiles {
            0x6a
        } else if index == full_tiles && remainder > 0 {
            0x62 + remainder as u8
        } else {
            0x62
        };
        let frame = tiles.get(&(tile_id, zone)).with_context(|| {
            format!("battle HP fill tile ${tile_id:02x} for palette zone {zone} was not loaded")
        })?;
        let (x, y) = battle_hud_tile_origin(tile_x + 2.0 + index as f32, tile_y);
        commands.spawn((
            SpriteBundle {
                texture: frame.handle.clone(),
                sprite: Sprite {
                    custom_size: Some(frame.size),
                    ..default()
                },
                transform: Transform::from_xyz(x, y, 3.62),
                ..default()
            },
            BattleHudMarker,
            BattleCommandMarker,
        ));
    }
    let end_tile = if side == BattleHpSide::Player {
        0x6c
    } else {
        0x6b
    };
    let frame = tiles.get(&(end_tile, zone)).with_context(|| {
        format!("battle HP end tile ${end_tile:02x} for palette zone {zone} was not loaded")
    })?;
    let (x, y) = battle_hud_tile_origin(tile_x + 2.0 + BATTLE_HUD_HP_BAR_LENGTH_TILES, tile_y);
    commands.spawn((
        SpriteBundle {
            texture: frame.handle.clone(),
            sprite: Sprite {
                custom_size: Some(frame.size),
                ..default()
            },
            transform: Transform::from_xyz(x, y, 3.62),
            ..default()
        },
        BattleHudMarker,
        BattleCommandMarker,
    ));
    Ok(())
}

fn battle_hud_tile_origin(tile_x: f32, tile_y: f32) -> (f32, f32) {
    (
        PLAYFIELD_LEFT + tile_x * TILE_SIZE + (SOURCE_TILE_SIZE as f32 * BATTLE_HUD_SCALE * 0.5),
        PLAYFIELD_TOP - tile_y * TILE_SIZE - (SOURCE_TILE_SIZE as f32 * BATTLE_HUD_SCALE * 0.5),
    )
}

fn battle_hp_bar_tiles<'a>(
    rendered_art: &'a mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<&'a HashMap<(u8, u8), SpriteFrame>> {
    if rendered_art.battle_hp_bar_cache.is_none() && rendered_art.battle_hp_bar_error.is_none() {
        let loaded = (|| -> Result<HashMap<(u8, u8), SpriteFrame>> {
            let assets = asset_root.runtime_assets();
            let battle_font =
                crate::read_runtime_asset(assets.join("gfx/font/font_battle_extra.2bpp"))
                    .context("read battle HP tile graphics")?;
            let player_end =
                crate::read_runtime_asset(assets.join("gfx/battle/enemy_hp_bar_border.1bpp"))
                    .context("read player HP end tile graphics")?;
            let mut result = HashMap::new();
            for zone in 0..=2_u8 {
                let fill = match zone {
                    2 => [0, 189, 0],
                    1 => [255, 173, 0],
                    _ => [255, 0, 0],
                };
                let tile_ids = 0x60_u8..=0x6c;
                for tile_id in tile_ids {
                    let (tile, one_bpp) = if tile_id == 0x6c {
                        (
                            player_end
                                .get(0..8)
                                .context("player HP end tile is missing")?,
                            true,
                        )
                    } else {
                        let source_index = usize::from(tile_id - 0x60);
                        let offset = source_index * 16;
                        (
                            battle_font.get(offset..offset + 16).with_context(|| {
                                format!("battle HP tile ${tile_id:02x} is missing")
                            })?,
                            false,
                        )
                    };
                    let mut pixels = vec![0_u8; SOURCE_TILE_SIZE * SOURCE_TILE_SIZE * 4];
                    for row in 0..SOURCE_TILE_SIZE {
                        let lo = tile[if one_bpp { row } else { row * 2 }];
                        // Get1bpp duplicates each row into both bitplanes, so
                        // monochrome border pixels use black (palette index 3).
                        let hi = if one_bpp { lo } else { tile[row * 2 + 1] };
                        for col in 0..SOURCE_TILE_SIZE {
                            let bit = 1 << (7 - col);
                            let level = ((hi & bit != 0) as u8) << 1 | (lo & bit != 0) as u8;
                            if level == 0 {
                                continue;
                            }
                            let color = match level {
                                1 => [247, 214, 123],
                                2 => fill,
                                _ => [0, 0, 0],
                            };
                            let target = (row * SOURCE_TILE_SIZE + col) * 4;
                            pixels[target..target + 3].copy_from_slice(&color);
                            pixels[target + 3] = 255;
                        }
                    }
                    let mut image = Image::new(
                        Extent3d {
                            width: SOURCE_TILE_SIZE as u32,
                            height: SOURCE_TILE_SIZE as u32,
                            depth_or_array_layers: 1,
                        },
                        TextureDimension::D2,
                        pixels,
                        TextureFormat::Rgba8UnormSrgb,
                        RenderAssetUsages::default(),
                    );
                    image.sampler = ImageSampler::nearest();
                    result.insert(
                        (tile_id, zone),
                        SpriteFrame {
                            handle: images.add(image),
                            size: Vec2::splat(TILE_SIZE),
                        },
                    );
                }
            }
            Ok(result)
        })();
        match loaded {
            Ok(tiles) => rendered_art.battle_hp_bar_cache = Some(tiles),
            Err(error) => rendered_art.battle_hp_bar_error = Some(error.to_string()),
        }
    }
    rendered_art.battle_hp_bar_cache.as_ref().with_context(|| {
        rendered_art
            .battle_hp_bar_error
            .clone()
            .unwrap_or_else(|| "battle HP bar art is unavailable".to_string())
    })
}

fn battle_hud_hp_pixels(hp: u16, max_hp: u16) -> u16 {
    if max_hp == 0 || hp == 0 {
        return 0;
    }
    let clamped_hp = hp.min(max_hp);
    let mut product = u32::from(clamped_hp) * u32::from(BATTLE_HUD_HP_BAR_LENGTH_PX);
    let mut divisor = u32::from(max_hp);
    // ComputeHPBarPixels has only an eight-bit hardware divisor. When the
    // maximum HP has a high byte, the cartridge truncates both operands by
    // two bits before dividing.
    if max_hp > u16::from(u8::MAX) {
        product >>= 2;
        divisor >>= 2;
    }
    let pixels = (product / divisor) as u16;
    pixels.max(1).min(BATTLE_HUD_HP_BAR_LENGTH_PX)
}

fn advance_visible_hp_pixels(current: &mut u16, target: u16, frames_until_step: &mut u8) -> bool {
    if *current == target {
        return false;
    }
    if *frames_until_step > 0 {
        *frames_until_step -= 1;
        return false;
    }
    // HPBarAnim_BGMapUpdate holds an ordinary player/enemy redraw for two
    // LCD frames total. This countdown is consumed on subsequent updates, so
    // one deferred update produces the source two-frame pixel spacing; using
    // 2 here made every pixel last three frames.
    *frames_until_step = 1;
    if *current < target {
        *current += 1;
    } else {
        *current -= 1;
    }
    true
}

fn advance_visible_player_hp_number(tween: &mut VisibleBattleHpTween) {
    if tween.player_pixels == tween.player_target_pixels {
        tween.player_hp = tween.player_target_hp;
        return;
    }
    if tween.player_hp < tween.player_target_hp {
        while tween.player_hp < tween.player_target_hp {
            tween.player_hp += 1;
            if battle_hud_hp_pixels(tween.player_hp, tween.player_max_hp) >= tween.player_pixels {
                break;
            }
        }
    } else {
        while tween.player_hp > tween.player_target_hp {
            tween.player_hp -= 1;
            if battle_hud_hp_pixels(tween.player_hp, tween.player_max_hp) <= tween.player_pixels {
                break;
            }
        }
    }
}

fn visible_battle_hp_tween_active(tween: &VisibleBattleHpTween) -> bool {
    tween.player_pixels != tween.player_target_pixels
        || tween.enemy_pixels != tween.enemy_target_pixels
}

fn visible_hp_zone(pixels: u16) -> u8 {
    let red_threshold = ((u32::from(BATTLE_HUD_HP_BAR_LENGTH_PX) * 21) / 100) as u16;
    let yellow_threshold = BATTLE_HUD_HP_BAR_LENGTH_PX / 2;
    if pixels < red_threshold {
        0
    } else if pixels < yellow_threshold {
        1
    } else {
        2
    }
}

fn battle_hud_hp_color(fill_pixels: u16) -> Color {
    match visible_hp_zone(fill_pixels) {
        2 => Color::srgba(0.0, 0.74, 0.0, 0.96),
        1 => Color::srgba(0.98, 0.72, 0.0, 0.96),
        _ => Color::srgba(0.96, 0.0, 0.0, 0.96),
    }
}

fn battle_status_token(status: Option<&str>) -> Option<&'static str> {
    let status = status?;
    if ["POISON", "BAD_POISON", "BAD POISON", "PSN"]
        .iter()
        .any(|candidate| status.eq_ignore_ascii_case(candidate))
    {
        Some("PSN")
    } else if ["SLEEP", "SLP"]
        .iter()
        .any(|candidate| status.eq_ignore_ascii_case(candidate))
    {
        Some("SLP")
    } else if ["PARALYSIS", "PAR"]
        .iter()
        .any(|candidate| status.eq_ignore_ascii_case(candidate))
    {
        Some("PAR")
    } else if ["BURN", "BRN"]
        .iter()
        .any(|candidate| status.eq_ignore_ascii_case(candidate))
    {
        Some("BRN")
    } else if ["FREEZE", "FRZ"]
        .iter()
        .any(|candidate| status.eq_ignore_ascii_case(candidate))
    {
        Some("FRZ")
    } else {
        None
    }
}

fn party_status_token(pokemon: &crate::core::models::pokemon::Pokemon) -> &'static str {
    if pokemon.hp == 0 {
        return "FNT";
    }
    let Some(status) = pokemon.status.as_deref() else {
        return "OK";
    };
    battle_status_token(Some(status)).unwrap_or_else(|| {
        if ["CONFUSION", "CNF"]
            .iter()
            .any(|candidate| status.eq_ignore_ascii_case(candidate))
        {
            "CNF"
        } else {
            "OK"
        }
    })
}

fn spawn_visible_move_animation_overlay(commands: &mut Commands, runtime_shell: &BevyRuntimeShell) {
    if runtime_shell.visible_move_animations.front().is_none()
        && let Some(send_out) = runtime_shell.visible_send_out_animation.as_ref()
        && send_out.shiny
        && send_out.frame >= VisibleSendOutAnimation::NORMAL_FRAMES
    {
        let age = send_out.frame - VisibleSendOutAnimation::NORMAL_FRAMES;
        if age < 3 {
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgba_u8(255, 255, 255, 96),
                        custom_size: Some(Vec2::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT)),
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 0.0, 3.48),
                    ..default()
                },
                BattleCommandMarker,
            ));
        }
        return;
    }
    let Some(animation) = runtime_shell
        .visible_move_animations
        .front()
        .filter(|animation| animation.started)
    else {
        return;
    };
    // Move palette effects are applied through their indexed BGP/OBP state
    // when battler and object textures are composed. They do not create a
    // translucent fullscreen layer.
    let _ = animation;
}

fn visible_battle_anim_sine(angle: u8, amplitude: u8) -> i32 {
    const WAVE: [u16; 32] = [
        0x0000, 0x0019, 0x0032, 0x004a, 0x0062, 0x0079, 0x008e, 0x00a2, 0x00b5, 0x00c6, 0x00d5,
        0x00e2, 0x00ed, 0x00f5, 0x00fb, 0x00ff, 0x0100, 0x00ff, 0x00fb, 0x00f5, 0x00ed, 0x00e2,
        0x00d5, 0x00c6, 0x00b5, 0x00a2, 0x008e, 0x0079, 0x0062, 0x004a, 0x0032, 0x0019,
    ];
    let normalized = angle & 0x3f;
    let mut product = 0_u16;
    let mut multiplier = amplitude;
    let mut value = WAVE[usize::from(normalized & 0x1f)];
    while multiplier != 0 {
        if multiplier & 1 != 0 {
            product = product.wrapping_add(value);
        }
        multiplier >>= 1;
        value = value.wrapping_shl(1);
    }
    let magnitude = ((product >> 8) & 0xff) as u8;
    if normalized & 0x20 != 0 {
        i32::from(((!magnitude).wrapping_add(1)) as i8)
    } else {
        i32::from(magnitude as i8)
    }
}

fn spawn_visible_move_animation_objects(
    commands: &mut Commands,
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<()> {
    let synthetic_shiny;
    let animation = if let Some(animation) = runtime_shell
        .visible_move_animations
        .front()
        .filter(|animation| animation.started)
    {
        animation
    } else if let Some(send_out) =
        runtime_shell
            .visible_send_out_animation
            .as_ref()
            .filter(|animation| {
                animation.shiny && animation.frame >= VisibleSendOutAnimation::NORMAL_FRAMES
            })
    {
        let sparkle_frame = u16::from(send_out.frame - VisibleSendOutAnimation::NORMAL_FRAMES);
        synthetic_shiny = VisibleMoveAnimation {
            trigger_message: String::new(),
            move_id: "SEND_OUT_SHINY".to_string(),
            animation_label: "BattleAnim_SendOutMon.Shiny".to_string(),
            player_move: send_out.side == crate::core::battle::turn::BattleSide::Player,
            started: true,
            waiting_for_hp: false,
            frame: sparkle_frame,
            total_frames: u16::from(VisibleSendOutAnimation::SHINY_FRAMES),
            sound_events: Vec::new(),
            next_sound_event: 0,
            cry_events: Vec::new(),
            next_cry_event: 0,
            object_events: (0_u8..8)
                .map(|index| VisibleMoveObjectEvent {
                    frame: u16::from(index) * 4,
                    command: VisibleMoveObjectCommand::Spawn {
                        object_id: "BATTLE_ANIM_OBJ_SHINY".to_string(),
                        x: 48,
                        y: 96,
                        param: index * 8,
                    },
                })
                .collect(),
            bg_events: Vec::new(),
            actor_species_override: None,
            actor_shiny_override: None,
        };
        &synthetic_shiny
    } else if let Some(capture) = runtime_shell
        .visible_capture_animation
        .as_ref()
        .filter(|animation| animation.retained_objects_visible())
    {
        let mut object_events = Vec::new();
        if capture.blocked {
            object_events.push(VisibleMoveObjectEvent {
                frame: 20,
                command: VisibleMoveObjectCommand::Spawn {
                    object_id: "BATTLE_ANIM_OBJ_HIT_YFIX".to_string(),
                    x: 112,
                    y: 40,
                    param: 0,
                },
            });
        } else {
            object_events.push(VisibleMoveObjectEvent {
                frame: 52,
                command: VisibleMoveObjectCommand::Spawn {
                    object_id: "BATTLE_ANIM_OBJ_BALL_POOF".to_string(),
                    x: 136,
                    y: 64,
                    param: 0x10,
                },
            });
        }
        if !capture.blocked && capture.ball_id.eq_ignore_ascii_case("MASTER_BALL") {
            object_events.extend((0_u8..8).map(|index| {
                VisibleMoveObjectEvent {
                    frame: capture
                        .master_ball_special_frame()
                        .expect("Master Ball frame"),
                    command: VisibleMoveObjectCommand::Spawn {
                        object_id: "BATTLE_ANIM_OBJ_MASTER_BALL_SPARKLE".to_string(),
                        x: 136,
                        y: 56,
                        param: 0x30 + index,
                    },
                }
            }));
        }
        if !capture.blocked && !capture.caught {
            object_events.push(VisibleMoveObjectEvent {
                frame: capture.total_frames().saturating_sub(34),
                command: VisibleMoveObjectCommand::Spawn {
                    object_id: "BATTLE_ANIM_OBJ_BALL_POOF".to_string(),
                    x: 136,
                    y: 64,
                    param: 0x10,
                },
            });
        }
        synthetic_shiny = VisibleMoveAnimation {
            trigger_message: String::new(),
            move_id: format!("THROW_{}", capture.ball_id),
            animation_label: "BattleAnim_ThrowPokeBall".to_string(),
            player_move: true,
            started: true,
            waiting_for_hp: false,
            frame: capture.frame,
            total_frames: capture.total_frames(),
            sound_events: Vec::new(),
            next_sound_event: 0,
            cry_events: Vec::new(),
            next_cry_event: 0,
            object_events,
            bg_events: Vec::new(),
            actor_species_override: None,
            actor_shiny_override: None,
        };
        &synthetic_shiny
    } else {
        return Ok(());
    };
    let bundle = battle_anim_render_bundle(rendered_art, snapshot)?;
    let mut playback = match rendered_art.battle_object_runtime.take() {
        Some(playback) if playback.source == animation.object_events
            && playback.player == animation.player_move && playback.label == animation.animation_label
            && u32::from(animation.frame) + 1 >= playback.next_tick => playback,
        _ => new_visible_battle_objects(&bundle, animation)?,
    };
    advance_visible_battle_objects(&mut playback, &bundle, animation)?;
    let live_slots = playback.slots.clone();
    let object_obp0_write = playback.obp0_write;
    rendered_art.battle_object_runtime = Some(playback);
    let mut dmg_palettes = visible_battle_dmg_palette_registers(Some(animation));
    if let Some((tick, value)) = object_obp0_write {
        // Source order is commands, background effects, then object callbacks.
        if dmg_palettes.obp0_write_frame.is_none_or(|frame| tick >= u32::from(frame)) {
            dmg_palettes.obp0 = value;
        }
    }
    for (slot_index, live) in live_slots.iter().enumerate() {
        let Some(live) = live else { continue; };
        if live.oam.entries.is_empty() { continue; }
        let VisibleMoveObjectCommand::Spawn { object_id, .. } = &animation.object_events[live.event_index].command else { unreachable!() };
        let object = &bundle["objects"][object_id];
        let frameset_name = live.frameset;
        let frame_index = live.frame;
        let frame = bundle["framesets"][frameset_name].as_array()
            .and_then(|frames| frames.get(frame_index)).context("live battle frameset overran")?;
        let palette_override = Some(match live.bytes[5] & 7 {
            0 => "PAL_BATTLE_OB_GRAY", 1 => "PAL_BATTLE_OB_YELLOW", 2 => "PAL_BATTLE_OB_RED",
            3 => "PAL_BATTLE_OB_GREEN", 4 => "PAL_BATTLE_OB_BLUE", 5 => "PAL_BATTLE_OB_BROWN",
            other => anyhow::bail!("invalid live battle palette {other}"),
        });
        let rendered = battle_anim_rendered_frame(
            rendered_art,
            &bundle,
            asset_root,
            object_id,
            object,
            frameset_name,
            frame_index,
            frame,
            !animation.player_move,
            false,
            false,
            palette_override,
            dmg_palettes.obp0,
            dmg_palettes.obp1,
            Some(&live.oam),
            images,
        )?;
        let (source_x, source_y) = live.oam.origin;
        let scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
        let destination_x = source_x - 8 + i32::from(rendered.offset_x);
        let destination_y = source_y - 16
            + i32::from(rendered.offset_y)
            + visible_rollout_object_y_offset(animation, slot_index);
        commands.spawn((
            SpriteBundle {
                texture: rendered.sprite.handle.clone(),
                sprite: Sprite {
                    custom_size: Some(rendered.sprite.size),
                    ..default()
                },
                transform: Transform::from_xyz(
                    PLAYFIELD_LEFT
                        + (destination_x as f32 + rendered.sprite.size.x / scale / 2.0) * scale,
                    PLAYFIELD_TOP
                        - (destination_y as f32 + rendered.sprite.size.y / scale / 2.0) * scale,
                    3.45 - slot_index as f32 * 0.001,
                ),
                ..default()
            },
            BattleCommandMarker,
        ));
    }
    Ok(())
}

fn battle_anim_object_function<'a>(
    object_id: &str,
    object: &'a serde_json::Value,
) -> Result<&'a str> {
    let function = object
        .get("function")
        .with_context(|| format!("battle animation object {object_id} has no function field"))?;
    if function.is_null() {
        return Ok("BATTLE_ANIM_FUNC_NULL");
    }
    function
        .as_str()
        .with_context(|| format!("battle animation object {object_id} has a non-string function"))
}

fn battle_anim_render_bundle(
    rendered_art: &mut RenderedTilesetArt,
    snapshot: &RuntimeShellSnapshot,
) -> Result<serde_json::Value> {
    if rendered_art.battle_anim_bundle_cache.is_none()
        && rendered_art.battle_anim_bundle_error.is_none()
    {
        match serde_json::from_str::<serde_json::Value>(&snapshot.presentation.battle_anim_bundle)
            .context("decode runtime battle animation bundle")
        {
            Ok(bundle) => rendered_art.battle_anim_bundle_cache = Some(bundle),
            Err(error) => rendered_art.battle_anim_bundle_error = Some(error.to_string()),
        }
    }
    rendered_art
        .battle_anim_bundle_cache
        .clone()
        .with_context(|| {
            rendered_art
                .battle_anim_bundle_error
                .clone()
                .unwrap_or_else(|| "battle animation bundle is unavailable".to_string())
        })
}

fn battle_anim_frame_at_age<'a>(
    bundle: &'a serde_json::Value,
    frameset_name: &str,
    age: u16,
) -> Result<Option<(usize, &'a serde_json::Value)>> {
    let frames = bundle
        .get("framesets")
        .and_then(|framesets| framesets.get(frameset_name))
        .and_then(serde_json::Value::as_array)
        .with_context(|| format!("battle animation frameset {frameset_name} is missing"))?;
    let mut remaining = u32::from(age);
    let mut index = 0_usize;
    let mut control_guard = 0_usize;
    loop {
        let frame = frames
            .get(index)
            .with_context(|| format!("battle animation frameset {frameset_name} overran"))?;
        match frame.get("command").and_then(serde_json::Value::as_str) {
            Some("frame") => {
                let duration = frame
                    .get("duration")
                    .and_then(serde_json::Value::as_u64)
                    .with_context(|| {
                        format!("battle animation frame {frameset_name}[{index}] has no duration")
                    })?
                    .saturating_add(1) as u32;
                if remaining < duration {
                    return Ok(Some((index, frame)));
                }
                remaining -= duration;
                index += 1;
                control_guard = 0;
            }
            Some("wait") => {
                let duration = frame
                    .get("duration")
                    .and_then(serde_json::Value::as_u64)
                    .with_context(|| {
                        format!("battle animation wait {frameset_name}[{index}] has no duration")
                    })?
                    .saturating_add(1) as u32;
                if remaining < duration {
                    return Ok(None);
                }
                remaining -= duration;
                index += 1;
                control_guard = 0;
            }
            Some("delete") => return Ok(None),
            Some("restart") => {
                index = 0;
                control_guard = control_guard.saturating_add(1);
            }
            Some("end") => {
                index = index.saturating_sub(1);
                control_guard = control_guard.saturating_add(1);
            }
            other => anyhow::bail!("unknown {frameset_name} frame command {other:?}"),
        }
        if control_guard > frames.len().saturating_add(2) {
            anyhow::bail!("battle animation frameset {frameset_name} has a control-command cycle");
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn battle_anim_rendered_frame(
    rendered_art: &mut RenderedTilesetArt,
    bundle: &serde_json::Value,
    asset_root: &AssetRoot,
    object_id: &str,
    object: &serde_json::Value,
    frameset_name: &str,
    frame_index: usize,
    frame: &serde_json::Value,
    enemy_move: bool,
    extra_yflip: bool,
    suppress_enemy_flips: bool,
    palette_override: Option<&str>,
    obp0: u8,
    obp1: u8,
    runtime_oam: Option<&VisibleBattleObjectOam>,
    images: &mut Assets<Image>,
) -> Result<BattleAnimRenderedFrame> {
    let flags = object
        .get("flags")
        .and_then(serde_json::Value::as_i64)
        .with_context(|| format!("battle animation object {object_id} has no numeric flags"))?;
    let frame_xflip = frame
        .get("xflip")
        .and_then(serde_json::Value::as_bool)
        .with_context(|| {
            format!("battle animation frame {frameset_name}[{frame_index}] has no xflip")
        })?
        ^ (enemy_move && !suppress_enemy_flips && flags & 0x20 != 0);
    let frame_yflip = frame
        .get("yflip")
        .and_then(serde_json::Value::as_bool)
        .with_context(|| {
            format!("battle animation frame {frameset_name}[{frame_index}] has no yflip")
        })?
        ^ (enemy_move && !suppress_enemy_flips && (flags & 0x40 != 0 || extra_yflip));
    let cache_key = format!(
        "{object_id}:{frameset_name}:{frame_index}:{frame_xflip}:{frame_yflip}:{}:{obp0:02x}:{obp1:02x}",
        palette_override.unwrap_or("default"),
    );
    let cache_key = if let Some(oam) = runtime_oam {
        format!("{cache_key}:{:?}:{:?}:{:?}", oam.origin, oam.entries, oam.rows)
    } else { cache_key };
    if let Some(rendered) = rendered_art.battle_anim_object_cache.get(&cache_key) {
        return Ok(rendered.clone());
    }
    if let Some(error) = rendered_art.battle_anim_object_errors.get(&cache_key) {
        anyhow::bail!(error.clone());
    }
    let loaded = (|| -> Result<BattleAnimRenderedFrame> {
        let oam_name = frame
            .get("oam_set")
            .and_then(serde_json::Value::as_str)
            .with_context(|| {
                format!("battle animation frame {frameset_name}[{frame_index}] has no OAM set")
            })?;
        let oam = bundle
            .get("oam_sets")
            .and_then(|sets| sets.get(oam_name))
            .with_context(|| format!("battle animation OAM set {oam_name} is missing"))?;
        let entries = oam
            .get("entries")
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("battle animation OAM set {oam_name} has no entries"))?;
        let tile_offset = oam
            .get("tile_offset")
            .and_then(serde_json::Value::as_i64)
            .with_context(|| format!("battle animation OAM set {oam_name} has no tile offset"))?;
        let gfx_id = object
            .get("gfx_id")
            .and_then(serde_json::Value::as_str)
            .with_context(|| format!("battle animation object {object_id} has no gfx id"))?;
        let gfx_entry = bundle
            .get("gfx_table")
            .and_then(|table| table.get(gfx_id))
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("battle animation gfx table entry {gfx_id} is missing"))?;
        let gfx_label = gfx_entry
            .get(1)
            .and_then(serde_json::Value::as_str)
            .with_context(|| {
                format!("battle animation gfx table entry {gfx_id} has no source label")
            })?;
        let relative_path = bundle
            .get("gfx_sources")
            .and_then(|sources| sources.get(gfx_label))
            .and_then(serde_json::Value::as_str)
            .with_context(|| format!("battle animation gfx source {gfx_label} is missing"))?;
        let compressed_path = asset_root.runtime_assets().join(relative_path);
        let raw_path = if compressed_path
            .extension()
            .is_some_and(|extension| extension == "lz")
        {
            compressed_path.with_extension("")
        } else {
            compressed_path
        };
        let tile_data = crate::read_runtime_asset(&raw_path)
            .with_context(|| format!("read battle animation graphics {}", raw_path.display()))?;
        if tile_data.len() % 16 != 0 {
            anyhow::bail!(
                "battle animation graphics {} are not 2bpp tile aligned",
                raw_path.display()
            );
        }
        let declared_palette = object
            .get("palette")
            .and_then(serde_json::Value::as_str)
            .with_context(|| format!("battle animation object {object_id} has no palette"))?;
        let palette_name = match palette_override.unwrap_or(declared_palette) {
            "PAL_BATTLE_OB_GRAY" => "gray",
            "PAL_BATTLE_OB_YELLOW" => "yellow",
            "PAL_BATTLE_OB_RED" => "red",
            "PAL_BATTLE_OB_GREEN" => "green",
            "PAL_BATTLE_OB_BLUE" => "blue",
            "PAL_BATTLE_OB_BROWN" => "brown",
            other => anyhow::bail!("unknown battle animation palette {other}"),
        };
        let palette = load_battle_anim_palette(asset_root, palette_name)?;
        let mut pieces = Vec::<(i32, i32, [u8; 8 * 8 * 4])>::new();
        let mut min_x = 0_i32;
        let mut min_y = 0_i32;
        let mut max_x = 0_i32;
        let mut max_y = 0_i32;
        for (piece_index, entry) in entries.iter().enumerate() {
            if runtime_oam.is_some_and(|oam| piece_index >= oam.entries.len()) { break; }
            let runtime_piece = runtime_oam.map(|oam| oam.entries[piece_index]);
            let entry_x = entry
                .get("x")
                .and_then(serde_json::Value::as_i64)
                .with_context(|| {
                    format!("battle animation OAM set {oam_name} has an entry without x")
                })? as i32;
            let entry_y = entry
                .get("y")
                .and_then(serde_json::Value::as_i64)
                .with_context(|| {
                    format!("battle animation OAM set {oam_name} has an entry without y")
                })? as i32;
            let x = if frame_xflip { -(entry_x + 8) } else { entry_x };
            let y = if frame_yflip { -(entry_y + 8) } else { entry_y };
            let (x, y) = if let (Some(oam), Some(piece)) = (runtime_oam, runtime_piece) {
                (i32::from(piece[1]) - oam.origin.0, i32::from(piece[0]) - oam.origin.1)
            } else { (x, y) };
            let entry_xflip = entry
                .get("xflip")
                .and_then(serde_json::Value::as_bool)
                .with_context(|| {
                    format!("battle animation OAM set {oam_name} has an entry without xflip")
                })?;
            let entry_yflip = entry
                .get("yflip")
                .and_then(serde_json::Value::as_bool)
                .with_context(|| {
                    format!("battle animation OAM set {oam_name} has an entry without yflip")
                })?;
            // The canonical exporter omits (or writes null for) OAM attributes
            // whose OBP bit is clear. That authored zero-bit state is OBP0;
            // preserve it instead of rejecting the capture frame.
            let obp = match entry.get("obp") {
                None | Some(serde_json::Value::Null) => 0,
                Some(value) => value.as_u64().with_context(|| {
                    format!(
                        "battle animation OAM set {oam_name} has a non-numeric obp: {entry}"
                    )
                })?,
            };
            let obp = runtime_piece.map_or(obp, |piece| u64::from((piece[3] >> 4) & 1));
            let mut object_palette = match obp {
                0 => obp0,
                1 => obp1,
                other => anyhow::bail!(
                    "battle animation OAM set {oam_name} has invalid OBP selector {other}"
                ),
            };
            let palette = if let Some(piece) = runtime_piece {
                // CGB uses OAM palette bits, not the DMG OBP selector. The
                // source's BattleAnim_SetOBPals remaps only gray and yellow.
                let palette_id = piece[3] & 7;
                object_palette = if palette_id < 2 { obp0 } else { 0xe4 };
                let name = ["gray", "yellow", "red", "green", "blue", "brown"]
                    .get(usize::from(palette_id)).context("unsupported CGB object palette")?;
                load_battle_anim_palette(asset_root, name)?
            } else { palette };
            let tile_id = entry
                .get("tile_id")
                .and_then(serde_json::Value::as_i64)
                .with_context(|| {
                    format!("battle animation OAM set {oam_name} has an entry without tile_id")
                })?;
            // GetBattleAnimOAM adds this offset directly to the loaded GFX
            // block. Frameset changes never rebase the graphics sheet.
            let tile_index = tile_offset + tile_id;
            let tile_start = usize::try_from(tile_index)
                .ok()
                .and_then(|index| index.checked_mul(16))
                .context("battle animation tile index overflow")?;
            let tile = tile_data
                .get(tile_start..tile_start + 16)
                .with_context(|| {
                    format!("battle animation tile {tile_index} is missing from {gfx_label}")
                })?;
            let mut pixels = [0_u8; 8 * 8 * 4];
            for output_y in 0..8_usize {
                for output_x in 0..8_usize {
                    if runtime_oam.is_some_and(|oam| !oam.rows[piece_index][output_y]) { continue; }
                    let effective_xflip = runtime_piece.map_or(entry_xflip ^ frame_xflip, |piece| piece[3] & 0x20 != 0);
                    let effective_yflip = runtime_piece.map_or(entry_yflip ^ frame_yflip, |piece| piece[3] & 0x40 != 0);
                    let source_x = if effective_xflip {
                        7 - output_x
                    } else {
                        output_x
                    };
                    let source_y = if effective_yflip {
                        7 - output_y
                    } else {
                        output_y
                    };
                    let bit = 1 << (7 - source_x);
                    let colour = ((tile[source_y * 2] & bit != 0) as usize)
                        | (((tile[source_y * 2 + 1] & bit != 0) as usize) << 1);
                    if colour == 0 {
                        continue;
                    }
                    let mapped_colour = usize::from((object_palette >> (colour * 2)) & 3);
                    let target = (output_y * 8 + output_x) * 4;
                    pixels[target..target + 4].copy_from_slice(&palette[mapped_colour]);
                    // OBJ colour zero is transparent before palette lookup;
                    // an opaque source colour mapped to DMG shade zero stays visible.
                    pixels[target + 3] = 255;
                }
            }
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x + 8);
            max_y = max_y.max(y + 8);
            pieces.push((x, y, pixels));
        }
        if pieces.is_empty() {
            anyhow::bail!("battle animation OAM set {oam_name} rendered no pieces");
        }
        let width =
            usize::try_from((max_x - min_x).max(1)).context("battle animation width overflow")?;
        let height =
            usize::try_from((max_y - min_y).max(1)).context("battle animation height overflow")?;
        let mut composite = vec![0_u8; width * height * 4];
        for (x, y, pixels) in pieces {
            for tile_y in 0..8_usize {
                for tile_x in 0..8_usize {
                    let source = (tile_y * 8 + tile_x) * 4;
                    if pixels[source + 3] == 0 {
                        continue;
                    }
                    let target_x = usize::try_from(x - min_x).unwrap() + tile_x;
                    let target_y = usize::try_from(y - min_y).unwrap() + tile_y;
                    let target = (target_y * width + target_x) * 4;
                    // CGB OAM priority is first-entry-first, including
                    // overlapping pieces belonging to the same object.
                    if composite[target + 3] == 0 {
                        composite[target..target + 4].copy_from_slice(&pixels[source..source + 4]);
                    }
                }
            }
        }
        let mut image = Image::new(
            Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            composite,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        image.sampler = ImageSampler::nearest();
        let scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
        Ok(BattleAnimRenderedFrame {
            sprite: SpriteFrame {
                handle: images.add(image),
                size: Vec2::new(width as f32 * scale, height as f32 * scale),
            },
            offset_x: i16::try_from(min_x).context("battle animation X offset overflow")?,
            offset_y: i16::try_from(min_y).context("battle animation Y offset overflow")?,
        })
    })();
    match loaded {
        Ok(rendered) => {
            rendered_art
                .battle_anim_object_cache
                .insert(cache_key.clone(), rendered.clone());
            Ok(rendered)
        }
        Err(error) => {
            rendered_art
                .battle_anim_object_errors
                .insert(cache_key, error.to_string());
            Err(error)
        }
    }
}

fn load_battle_anim_palette(asset_root: &AssetRoot, requested: &str) -> Result<[[u8; 4]; 4]> {
    let path = asset_root
        .runtime_assets()
        .join("gfx/battle_anims/battle_anims.pal");
    let source = crate::read_runtime_asset_to_string(&path)
        .with_context(|| format!("read battle animation palettes {}", path.display()))?;
    let mut section = "";
    let mut colours = Vec::new();
    for line in source.lines() {
        let line = line.trim();
        if let Some(name) = line.strip_prefix(';') {
            section = name.trim();
            continue;
        }
        if section != requested || !line.starts_with("RGB") {
            continue;
        }
        let channels = line[3..]
            .split(',')
            .map(|channel| channel.trim().parse::<u8>())
            .collect::<std::result::Result<Vec<_>, _>>()
            .with_context(|| format!("parse battle animation palette {requested}"))?;
        if channels.len() != 3 {
            anyhow::bail!("battle animation palette {requested} has malformed RGB data");
        }
        colours.push([
            ((u16::from(channels[0]) * 255 + 15) / 31) as u8,
            ((u16::from(channels[1]) * 255 + 15) / 31) as u8,
            ((u16::from(channels[2]) * 255 + 15) / 31) as u8,
            if colours.is_empty() { 0 } else { 255 },
        ]);
    }
    colours.try_into().map_err(|_| {
        anyhow::anyhow!("battle animation palette {requested} must contain four colours")
    })
}

fn visible_battle_command_animation_active(runtime_shell: &BevyRuntimeShell) -> bool {
    runtime_shell.visible_battle_sliding_intro.is_some()
        || runtime_shell.visible_capture_animation.is_some()
        || runtime_shell.visible_frontpic_animation.is_some()
        || runtime_shell
            .visible_move_animations
            .front()
            .is_some_and(|animation| animation.started)
        || runtime_shell.visible_send_out_animation.is_some()
        || runtime_shell.visible_trainer_exit_animation.is_some()
        || runtime_shell
            .battle_hp_tween
            .as_ref()
            .is_some_and(visible_battle_hp_tween_active)
        || runtime_shell
            .battle_exp_tween
            .as_ref()
            .is_some_and(|tween| tween.started)
}

fn spawn_battle_command_menu(
    commands: &mut Commands,
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
    battle: &crate::RuntimeBattleSnapshot,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<()> {
    require_bitmap_font_art(rendered_art, asset_root, images)?;
    if visible_battle_command_animation_active(runtime_shell) {
        return Ok(());
    }
    if battle_window_frame_art(rendered_art, asset_root, images).is_none() {
        let frame_id = rendered_art.selected_window_frame_id.clamp(1, 8);
        anyhow::bail!(
            "{}",
            rendered_art
                .window_frame_errors
                .get(&frame_id)
                .cloned()
                .unwrap_or_else(|| "battle window frame art is unavailable".to_string())
        );
    }
    if let Some(stats) = runtime_shell
        .battle_level_stats
        .front()
        .filter(|stats| stats.active)
    {
        spawn_battle_window(
            commands,
            rendered_art,
            asset_root,
            images,
            9.0,
            0.0,
            11.0,
            12.0,
            4.0,
        );
        for (index, (label, value)) in [
            ("ATTACK", stats.attack),
            ("DEFENSE", stats.defense),
            ("SPCL.ATK", stats.special_attack),
            ("SPCL.DEF", stats.special_defense),
            ("SPEED", stats.speed),
        ]
        .iter()
        .enumerate()
        {
            let label_row = 1.0 + index as f32 * 2.0;
            let (x, y) = battle_hud_tile_origin(11.0, label_row);
            spawn_battle_command_bitmap_text(
                commands,
                rendered_art,
                asset_root,
                images,
                label,
                x,
                y,
                4.2,
            );
            let (x, y) = battle_hud_tile_origin(15.0, label_row + 1.0);
            spawn_battle_command_bitmap_text(
                commands,
                rendered_art,
                asset_root,
                images,
                &format!("{:>3}", value),
                x,
                y,
                4.2,
            );
        }
        return Ok(());
    }
    if let Some(message) = runtime_shell.battle_messages.front() {
        spawn_battle_window(
            commands,
            rendered_art,
            asset_root,
            images,
            BATTLE_TEXT_BOX_LEFT_TILE,
            BATTLE_TEXT_BOX_TOP_TILE,
            BATTLE_TEXT_BOX_WIDTH_TILES,
            BATTLE_TEXT_BOX_HEIGHT_TILES,
            3.5,
        );
        for (line_index, line) in visible_battle_message_lines(runtime_shell, message)
            .iter()
            .enumerate()
        {
            // SpeechTextbox prints at TEXTBOX_INNERY and INNERY + 2. Once a
            // third line begins, TextScroll moves the previous baseline up.
            let (x, y) = battle_hud_tile_origin(1.0, 14.0 + line_index as f32 * 2.0);
            spawn_battle_command_bitmap_text(
                commands,
                rendered_art,
                asset_root,
                images,
                line,
                x,
                y,
                3.8,
            );
        }
        if visible_battle_message_is_complete(runtime_shell, message)
            && visible_vblank_counter_bit4(runtime_shell)
        {
            let (x, y) = battle_hud_tile_origin(18.0, 16.0);
            spawn_battle_command_bitmap_text(
                commands,
                rendered_art,
                asset_root,
                images,
                "▼",
                x,
                y,
                3.85,
            );
        }
        return Ok(());
    }
    if snapshot.pending_move_learn.is_some() {
        spawn_battle_pending_move_learn_screen(
            commands,
            snapshot,
            runtime_shell,
            rendered_art,
            asset_root,
            images,
        )?;
        return Ok(());
    }
    if runtime_shell.battle_pack_target_mode == Some(BattlePackTargetMode::PartyPokemon)
        || (runtime_shell.battle_pack_target_mode == Some(BattlePackTargetMode::PartyMove)
            && runtime_shell.party_move_cursor.is_none())
    {
        spawn_battle_party_menu(
            commands,
            snapshot,
            runtime_shell,
            rendered_art,
            asset_root,
            images,
        )?;
        return Ok(());
    }
    if runtime_shell.battle_pack_target_mode == Some(BattlePackTargetMode::PartyMove)
        && runtime_shell.party_move_cursor.is_some()
    {
        spawn_battle_pack_move_target_screen(
            commands,
            snapshot,
            runtime_shell,
            rendered_art,
            asset_root,
            images,
        )?;
        return Ok(());
    }
    if runtime_shell.battle_pack_target_mode.is_none()
        && (runtime_shell.bag_cursor.is_some()
            || runtime_shell.ball_cursor.is_some()
            || runtime_shell.key_item_cursor.is_some()
            || runtime_shell.tmhm_cursor.is_some())
    {
        spawn_battle_pack_screen(
            commands,
            snapshot,
            runtime_shell,
            rendered_art,
            asset_root,
            images,
        )?;
        return Ok(());
    }
    if runtime_shell.battle_move_cursor.is_some() {
        spawn_battle_move_menu(
            commands,
            snapshot,
            runtime_shell,
            battle,
            rendered_art,
            asset_root,
            images,
        )?;
        return Ok(());
    }
    if runtime_shell.battle_party_summary_open {
        spawn_field_party_summary_screen(
            commands,
            snapshot,
            runtime_shell,
            rendered_art,
            asset_root,
            images,
        )?;
        return Ok(());
    }
    if runtime_shell.battle_switch_cursor.is_some() {
        spawn_battle_party_menu(
            commands,
            snapshot,
            runtime_shell,
            rendered_art,
            asset_root,
            images,
        )?;
        return Ok(());
    }
    let entries = visible_battle_command_menu_entries(snapshot, runtime_shell, battle)?;
    if entries.is_empty() {
        return Ok(());
    }
    if runtime_shell.battle_faint_prompt_cursor.is_some()
        || runtime_shell.battle_shift_prompt_cursor.is_some()
    {
        spawn_battle_yes_no_prompt(
            commands,
            runtime_shell,
            rendered_art,
            asset_root,
            images,
            &entries,
        );
        return Ok(());
    }
    if battle_command_entries_are_main_menu(&entries) {
        spawn_battle_main_command_menu(
            commands,
            snapshot,
            runtime_shell,
            battle,
            rendered_art,
            asset_root,
            images,
            &entries,
        )?;
        return Ok(());
    }
    spawn_battle_window(
        commands,
        rendered_art,
        asset_root,
        images,
        BATTLE_TEXT_BOX_LEFT_TILE,
        BATTLE_TEXT_BOX_TOP_TILE,
        BATTLE_TEXT_BOX_WIDTH_TILES,
        BATTLE_TEXT_BOX_HEIGHT_TILES,
        3.5,
    );
    let two_columns = scene_menu_uses_two_columns(&entries);
    for (index, entry) in entries.iter().enumerate() {
        let (tile_x, tile_y) = battle_submenu_entry_tile(index, two_columns);
        let (x, y) = battle_hud_tile_origin(tile_x, tile_y);
        let display_entry = compact_scene_label(
            &animated_battle_cursor_entry(runtime_shell, entry),
            if two_columns { 9 } else { 18 },
        );
        spawn_battle_command_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            &display_entry,
            x,
            y,
            3.8,
        );
    }
    Ok(())
}

fn spawn_battle_pending_move_learn_screen(
    commands: &mut Commands,
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<()> {
    let pending = snapshot
        .pending_move_learn
        .as_ref()
        .context("battle move-learning screen has no pending move")?;
    let pending_move = snapshot
        .moves
        .iter()
        .find(|move_data| move_data.move_id == pending.learned_move.name)
        .with_context(|| {
            format!(
                "battle move-learning metadata {} is missing",
                pending.learned_move.name
            )
        })?;
    let move_name = pending_move.name.replace('_', " ");
    let slot = snapshot
        .party
        .slots
        .iter()
        .find(|slot| slot.index == pending.party_index)
        .with_context(|| {
            format!(
                "battle move-learning party slot {} is missing",
                pending.party_index
            )
        })?;
    for learned in &slot.pokemon.moves {
        snapshot
            .moves
            .iter()
            .find(|move_data| move_data.move_id == learned.name)
            .with_context(|| format!("battle move-forget metadata {} is missing", learned.name))?;
    }

    if runtime_shell.move_learn_forget_menu_open {
        let option_count = slot.pokemon.moves.len() + 1;
        let selected = strict_readonly_cursor_index(
            &runtime_shell.party_move_cursor,
            &party_move_cursor_surface_id(pending.party_index),
            option_count,
        )
        .context("battle move-forget screen has no valid cursor")?;
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(1.0, 1.0, 1.0),
                    custom_size: Some(Vec2::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, 3.4),
                ..default()
            },
            BattleCommandMarker,
        ));
        for (line_index, line) in ["Which move should", "be forgotten?"].iter().enumerate() {
            let (x, y) = battle_hud_tile_origin(1.0, 1.0 + line_index as f32);
            spawn_battle_command_bitmap_text(
                commands,
                rendered_art,
                asset_root,
                images,
                line,
                x,
                y,
                3.8,
            );
        }
        for (index, learned) in slot.pokemon.moves.iter().enumerate().take(4) {
            let row = 3.0 + index as f32 * 2.0;
            let (x, y) = battle_hud_tile_origin(1.0, row);
            spawn_battle_command_bitmap_text(
                commands,
                rendered_art,
                asset_root,
                images,
                &format!(
                    "{}{}",
                    if selected == index {
                        battle_cursor_glyph(runtime_shell)
                    } else {
                        " "
                    },
                    battle_move_display_name(snapshot, &learned.name),
                ),
                x,
                y,
                3.8,
            );
            let (x, y) = battle_hud_tile_origin(11.0, row + 1.0);
            spawn_battle_command_bitmap_text(
                commands,
                rendered_art,
                asset_root,
                images,
                &visible_move_pp_text(snapshot, learned),
                x,
                y,
                3.8,
            );
        }
        let (x, y) = battle_hud_tile_origin(1.0, 12.0);
        spawn_battle_command_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            &format!(
                "{}CANCEL",
                if selected == slot.pokemon.moves.len() {
                    battle_cursor_glyph(runtime_shell)
                } else {
                    " "
                },
            ),
            x,
            y,
            3.8,
        );
        let (x, y) = battle_hud_tile_origin(1.0, 15.0);
        spawn_battle_command_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            &format!("Trying to learn {move_name}."),
            x,
            y,
            3.8,
        );
        return Ok(());
    }

    spawn_battle_window(
        commands,
        rendered_art,
        asset_root,
        images,
        BATTLE_TEXT_BOX_LEFT_TILE,
        BATTLE_TEXT_BOX_TOP_TILE,
        BATTLE_TEXT_BOX_WIDTH_TILES,
        BATTLE_TEXT_BOX_HEIGHT_TILES,
        3.5,
    );
    let prompt = match runtime_shell.move_learn_decision {
        Some(VisibleTmHmDecision::ForgetMove) => {
            format!("Delete a move to make room\nfor {move_name}?")
        }
        Some(VisibleTmHmDecision::StopLearning) => format!("Stop learning {move_name}?"),
        None => format!(
            "But {} can't learn more\nthan four moves.\nDelete an older move to\nmake room for {move_name}?",
            slot.pokemon.nickname
        ),
    };
    for (line_index, line) in wrap_boot_text_for_box(&prompt, 18, 4).iter().enumerate() {
        let (x, y) = battle_hud_tile_origin(1.0, 13.0 + line_index as f32);
        spawn_battle_command_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            line,
            x,
            y,
            3.8,
        );
    }
    if runtime_shell.move_learn_decision_cursor.is_some() {
        let selected = strict_readonly_cursor_index(
            &runtime_shell.move_learn_decision_cursor,
            "move-learn:decision",
            2,
        )
        .context("battle move-learning decision cursor is invalid")?;
        spawn_battle_window(
            commands,
            rendered_art,
            asset_root,
            images,
            FIELD_YES_NO_LEFT_TILE,
            FIELD_YES_NO_TOP_TILE,
            FIELD_YES_NO_WIDTH_TILES,
            FIELD_YES_NO_HEIGHT_TILES,
            4.0,
        );
        for (index, label) in ["YES", "NO"].iter().enumerate() {
            let (x, y) = battle_hud_tile_origin(
                FIELD_YES_NO_LEFT_TILE,
                FIELD_YES_NO_TOP_TILE + 1.0 + index as f32,
            );
            spawn_battle_command_bitmap_text(
                commands,
                rendered_art,
                asset_root,
                images,
                &format!(
                    "{}{label}",
                    if selected == index {
                        battle_cursor_glyph(runtime_shell)
                    } else {
                        " "
                    },
                ),
                x,
                y,
                4.2,
            );
        }
    }
    Ok(())
}

fn spawn_visible_capture_animation(
    commands: &mut Commands,
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<()> {
    let Some(animation) = runtime_shell
        .visible_capture_animation
        .as_ref()
        .filter(|animation| animation.ball_visible())
    else {
        return Ok(());
    };
    if !animation.blocked {
        let master_ball = animation.ball_id.eq_ignore_ascii_case("MASTER_BALL");
        let _drop_start = if master_ball { 164 } else { 92 };
        // BreakFree sets the retained ball directly to stage 11 before its
        // poof and ENTER_MON wait, which deinitializes it immediately.
        if !animation.caught && animation.frame >= animation.total_frames().saturating_sub(34) {
            return Ok(());
        }
    }
    let (screen_x, screen_y) = if animation.blocked {
        let Some((x, y)) =
            visible_capture_object_position(64, 92, 0x20, 0x70, animation.frame, true)
        else {
            return Ok(());
        };
        (x as f32, y as f32)
    } else if animation.frame < 36 {
        let master_ball = animation.ball_id.eq_ignore_ascii_case("MASTER_BALL");
        let (x, y) = visible_capture_object_position(
            if master_ball { 64 } else { 68 },
            92,
            if master_ball { 0x20 } else { 0x40 },
            0x88,
            animation.frame,
            false,
        )
        .context("ordinary Poké Ball throw deinitialized during its flight")?;
        (x as f32, y as f32)
    } else if animation.frame < 68 {
        // The second object is forced into Poké Ball stage 7. Stage 8 uses a
        // radius-$20 sine for 32 updates before deinitializing.
        let age = animation.frame.saturating_sub(36);
        let angle = 0_u8.wrapping_sub(age as u8);
        (136.0, 65.0 + visible_battle_anim_sine(angle, 0x20) as f32)
    } else {
        let master_ball = animation.ball_id.eq_ignore_ascii_case("MASTER_BALL");
        let (_, landed_y) = visible_capture_object_position(
            if master_ball { 64 } else { 68 },
            92,
            if master_ball { 0x20 } else { 0x40 },
            0x88,
            36,
            false,
        )
        .context("capture throw did not reach its retained landing object")?;
        let drop_start = if master_ball { 164 } else { 92 };
        let drop_age = animation.frame.saturating_sub(drop_start).min(127);
        let amplitude = 0x10_u8.saturating_sub(((drop_age / 32) as u8) * 4);
        let angle = 0_u8.wrapping_sub(drop_age as u8);
        let y_offset =
            if animation.frame >= drop_start && animation.frame < animation.shake_setup_frame() {
                visible_battle_anim_sine(angle, amplitude)
            } else {
                0
            };
        (136.0, landed_y as f32 + y_offset as f32)
    };
    let bundle = battle_anim_render_bundle(rendered_art, snapshot)?;
    let object_id = if animation.blocked {
        "BATTLE_ANIM_OBJ_POKE_BALL_BLOCKED"
    } else {
        "BATTLE_ANIM_OBJ_POKE_BALL"
    };
    let object = bundle
        .get("objects")
        .and_then(|objects| objects.get(object_id))
        .with_context(|| format!("battle animation object {object_id} is missing"))?;
    let drop_start = animation.shake_entry_frame().saturating_add(8);
    let (frameset, frameset_age) = if animation.blocked || animation.frame < 36 {
        ("BATTLE_ANIM_FRAMESET_POKE_BALL_1", animation.frame)
    } else if animation.frame < 68 {
        ("BATTLE_ANIM_FRAMESET_POKE_BALL_2", 0)
    } else if animation.frame < drop_start {
        // The retained throw object switches to the flattened OAM set while
        // RETURN_MON collapses the target; it remains visible beneath the
        // separate opening object rather than disappearing for 24 frames.
        ("BATTLE_ANIM_FRAMESET_POKE_BALL_3", 0)
    } else {
        let bounce_age = animation.frame.saturating_sub(drop_start);
        if bounce_age < 128 {
            ("BATTLE_ANIM_FRAMESET_POKE_BALL_1", bounce_age)
        } else {
            let first_check = animation.first_shake_check_frame();
            let completed_check = animation.frame.saturating_sub(first_check) / 48 + 1;
            let wobble_start = first_check.saturating_add(48 * completed_check.saturating_sub(1));
            let successful_wobble = animation.frame >= first_check
                && if animation.caught {
                    completed_check < u16::from(animation.animation_shakes)
                } else {
                    completed_check <= u16::from(animation.animation_shakes)
                };
            if successful_wobble {
                (
                    "BATTLE_ANIM_FRAMESET_POKE_BALL_5",
                    animation.frame.saturating_sub(wobble_start),
                )
            } else {
                ("BATTLE_ANIM_FRAMESET_POKE_BALL_4", 0)
            }
        }
    };
    let Some((frame_index, frame)) = battle_anim_frame_at_age(&bundle, frameset, frameset_age)?
    else {
        return Ok(());
    };
    let rendered = battle_anim_rendered_frame(
        rendered_art,
        &bundle,
        asset_root,
        object_id,
        object,
        frameset,
        frame_index,
        frame,
        false,
        false,
        true,
        Some(match animation.ball_id.as_str() {
            "MASTER_BALL" => "PAL_BATTLE_OB_GREEN",
            "ULTRA_BALL" | "FRIEND_BALL" => "PAL_BATTLE_OB_YELLOW",
            "GREAT_BALL" | "LURE_BALL" | "FAST_BALL" => "PAL_BATTLE_OB_BLUE",
            "HEAVY_BALL" | "MOON_BALL" => "PAL_BATTLE_OB_GRAY",
            "LEVEL_BALL" => "PAL_BATTLE_OB_BROWN",
            _ => "PAL_BATTLE_OB_RED",
        }),
        0xe4,
        0xe4,
        None,
        images,
    )?;
    let source_scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
    let destination_x = screen_x - 8.0 + f32::from(rendered.offset_x);
    let destination_y = screen_y - 16.0 + f32::from(rendered.offset_y);
    let x = PLAYFIELD_LEFT
        + (destination_x + rendered.sprite.size.x / source_scale / 2.0) * source_scale;
    let y = PLAYFIELD_TOP
        - (destination_y + rendered.sprite.size.y / source_scale / 2.0) * source_scale;
    commands.spawn((
        SpriteBundle {
            texture: rendered.sprite.handle.clone(),
            sprite: Sprite {
                custom_size: Some(rendered.sprite.size),
                ..default()
            },
            transform: Transform::from_xyz(x, y, 4.1),
            ..default()
        },
        BattleCommandMarker,
    ));
    Ok(())
}

fn visible_capture_object_position(
    start_x: i32,
    start_y: i32,
    amplitude: u8,
    target_x: i32,
    frame: u16,
    blocked: bool,
) -> Option<(i32, i32)> {
    let mut x = start_x;
    let mut y = start_y;
    let mut y_offset = 0;
    let mut angle = 0_u8;
    let mut stage = 0_u8;
    for _ in 0..frame {
        if stage == 0 {
            stage = 1;
            continue;
        }
        if stage == 1 {
            if x < target_x {
                x = (x + 2) & 0xff;
                y = (y - 1) & 0xff;
                y_offset = visible_battle_anim_sine(angle, amplitude);
                angle = angle.wrapping_sub(1);
                continue;
            }
            if !blocked {
                y = (y + y_offset) & 0xff;
                y_offset = 0;
                stage = 2;
                continue;
            }
            stage = 2;
        }
        if blocked && stage == 2 {
            if y >= 0x80 {
                return None;
            }
            y = (y + 4) & 0xff;
            x = (x - 2) & 0xff;
        }
    }
    Some((x, (y + y_offset) & 0xff))
}

fn spawn_visible_send_out_poof(
    commands: &mut Commands,
    runtime_shell: &BevyRuntimeShell,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<()> {
    let Some(animation) = runtime_shell.visible_send_out_animation.as_ref() else {
        return Ok(());
    };
    let frame_index = usize::from(animation.frame / 3);
    if frame_index >= 4 {
        return Ok(());
    }
    let frames = battle_send_out_poof_frames(rendered_art, asset_root, images)?;
    let frame = &frames[frame_index];
    let (screen_x, screen_y) = match animation.side {
        crate::core::battle::turn::BattleSide::Player => (40.0, 100.0),
        crate::core::battle::turn::BattleSide::Enemy => (120.0, 36.0),
    };
    let source_scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
    // The 40x40 smoke cache is already centered about the animation object;
    // only the Game Boy OAM hardware origin offsets remain.
    let x = PLAYFIELD_LEFT + (screen_x - 8.0) * source_scale;
    let y = PLAYFIELD_TOP - (screen_y - 16.0) * source_scale;
    commands.spawn((
        SpriteBundle {
            texture: frame.handle.clone(),
            sprite: Sprite {
                custom_size: Some(frame.size * source_scale),
                ..default()
            },
            transform: Transform::from_xyz(x, y, 4.15),
            ..default()
        },
        BattleCommandMarker,
    ));
    Ok(())
}

fn spawn_visible_fishing_animation(
    commands: &mut Commands,
    runtime_shell: &BevyRuntimeShell,
    facing: Direction,
    player_x: f32,
    player_y: f32,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<()> {
    let Some(animation) = runtime_shell.visible_fishing_animation else {
        return Ok(());
    };
    let rod = fishing_rod_frame(rendered_art, asset_root, facing, images)?;
    // FacingFish* OAM positions, measured from the 16x16 player's top-left.
    let (dx, dy) = match facing {
        Direction::Down => (-TILE_SIZE * 0.5, -TILE_SIZE * 1.5),
        Direction::Up => (-TILE_SIZE * 0.5, TILE_SIZE * 1.5),
        Direction::Left => (-TILE_SIZE * 1.5, -TILE_SIZE * 0.125),
        Direction::Right => (TILE_SIZE * 1.5, -TILE_SIZE * 0.125),
    };
    commands.spawn((
        SpriteBundle {
            texture: rod.handle.clone(),
            sprite: Sprite {
                custom_size: Some(rod.size),
                ..default()
            },
            transform: Transform::from_xyz(player_x + dx, player_y + dy, 2.8),
            ..default()
        },
        PlayerFacingMarker,
    ));
    if animation.phase == VisibleFishingPhase::Pause {
        let shock = emote_frame_for_art(rendered_art, asset_root, "EMOTE_SHOCK", images)
            .context("required fishing bite emote could not be rendered")?;
        commands.spawn((
            SpriteBundle {
                texture: shock.handle,
                sprite: Sprite {
                    custom_size: Some(shock.size),
                    ..default()
                },
                transform: Transform::from_xyz(player_x, player_y + TILE_SIZE * 1.35, 2.9),
                ..default()
            },
            PlayerFacingMarker,
        ));
    }
    Ok(())
}

fn fishing_rod_frame<'a>(
    rendered_art: &'a mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    facing: Direction,
    images: &mut Assets<Image>,
) -> Result<&'a SpriteFrame> {
    if rendered_art.fishing_rod_cache.is_none() && rendered_art.fishing_rod_error.is_none() {
        let loaded = (|| -> Result<[SpriteFrame; 3]> {
            let path = asset_root
                .runtime_assets()
                .join("gfx/overworld/fishing_rod.png");
            let source = crate::open_runtime_image(&path)
                .with_context(|| format!("decode fishing rod PNG {}", path.display()))?
                .to_rgba8();
            if source.width() != 8 || source.height() != 16 {
                anyhow::bail!("fishing rod sheet must contain exactly tiles $fc and $fd");
            }
            let make_tile = |tile: u32, mirror: bool, images: &mut Assets<Image>| {
                let mut pixels = vec![0_u8; 8 * 8 * 4];
                for y in 0..8_u32 {
                    for x in 0..8_u32 {
                        let source_x = if mirror { 7 - x } else { x };
                        let pixel = source.get_pixel(source_x, tile * 8 + y);
                        let target = ((y * 8 + x) * 4) as usize;
                        if pixel[0] <= 240 || pixel[1] <= 240 || pixel[2] <= 240 {
                            pixels[target..target + 4].copy_from_slice(&[32, 32, 32, 255]);
                        }
                    }
                }
                let mut image = Image::new(
                    Extent3d {
                        width: 8,
                        height: 8,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    pixels,
                    TextureFormat::Rgba8UnormSrgb,
                    RenderAssetUsages::default(),
                );
                image.sampler = ImageSampler::nearest();
                SpriteFrame {
                    handle: images.add(image),
                    size: Vec2::splat(TILE_SIZE),
                }
            };
            Ok([
                make_tile(0, false, images),
                make_tile(1, false, images),
                make_tile(1, true, images),
            ])
        })();
        match loaded {
            Ok(frame) => rendered_art.fishing_rod_cache = Some(frame),
            Err(error) => rendered_art.fishing_rod_error = Some(error.to_string()),
        }
    }
    let frames = rendered_art.fishing_rod_cache.as_ref().with_context(|| {
        rendered_art
            .fishing_rod_error
            .clone()
            .unwrap_or_else(|| "fishing rod art is unavailable".to_string())
    })?;
    Ok(match facing {
        Direction::Down | Direction::Up => &frames[0],
        Direction::Right => &frames[1],
        Direction::Left => &frames[2],
    })
}

fn fishing_player_frame(
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    female: bool,
    facing: Direction,
    palette_id: u8,
    time_of_day: &str,
    images: &mut Assets<Image>,
) -> Result<SpriteFrame> {
    let time = normalize_tileset_time_of_day(time_of_day);
    let key = format!(
        "{}:{facing:?}:{}:{time}",
        if female { "kris" } else { "chris" },
        palette_id & 7
    );
    if !rendered_art.fishing_player_cache.contains_key(&key)
        && !rendered_art.fishing_player_errors.contains_key(&key)
    {
        let loaded = (|| -> Result<SpriteFrame> {
            let assets = asset_root.runtime_assets();
            let player_name = if female { "kris" } else { "chris" };
            let normal_path = assets
                .join("gfx/sprites")
                .join(format!("{player_name}.png"));
            let fish_path = assets
                .join("gfx/overworld")
                .join(format!("{player_name}_fish.png"));
            let normal = crate::open_runtime_image(&normal_path)
                .with_context(|| format!("decode player sprite PNG {}", normal_path.display()))?
                .to_rgba8();
            let fish = crate::open_runtime_image(&fish_path)
                .with_context(|| format!("decode fishing sprite PNG {}", fish_path.display()))?
                .to_rgba8();
            if normal.width() != 16
                || normal.height() < 48
                || fish.width() != 16
                || fish.height() < 24
            {
                anyhow::bail!("fishing sprite sources must contain the three 16-pixel facings");
            }
            let (normal_frame, fish_row, mirror) = match facing {
                Direction::Down => (0_u32, 0_u32, false),
                Direction::Up => (1, 1, false),
                Direction::Left => (2, 2, false),
                Direction::Right => (2, 2, true),
            };
            let mut combined = image::RgbaImage::new(16, 16);
            for y in 0..16_u32 {
                for x in 0..16_u32 {
                    combined.put_pixel(x, y, *normal.get_pixel(x, normal_frame * 16 + y));
                }
            }
            // LoadFishingGFX replaces vTiles $02/$03, $06/$07, or $0a/$0b:
            // the lower 8-pixel pair of the direction's ordinary 16x16 body.
            for y in 0..8_u32 {
                for x in 0..16_u32 {
                    combined.put_pixel(x, y + 8, *fish.get_pixel(x, fish_row * 8 + y));
                }
            }
            let palette_bank = load_npc_sprite_palette_bank(asset_root, &time)?;
            let palette = palette_bank
                .get(usize::from(palette_id & 7))
                .with_context(|| {
                    format!(
                        "fishing player palette {} is missing from the {time} NPC palette bank",
                        palette_id & 7
                    )
                })?;
            Ok(create_sprite_frame(
                &combined, 16, 0, palette, mirror, images,
            ))
        })();
        match loaded {
            Ok(frame) => {
                rendered_art.fishing_player_cache.insert(key.clone(), frame);
            }
            Err(error) => {
                rendered_art
                    .fishing_player_errors
                    .insert(key.clone(), error.to_string());
            }
        }
    }
    rendered_art
        .fishing_player_cache
        .get(&key)
        .cloned()
        .with_context(|| {
            rendered_art
                .fishing_player_errors
                .get(&key)
                .cloned()
                .unwrap_or_else(|| "fishing player art is unavailable".to_string())
        })
}

fn battle_send_out_poof_frames<'a>(
    rendered_art: &'a mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<&'a [SpriteFrame; 4]> {
    if rendered_art.battle_send_out_poof_cache.is_none()
        && rendered_art.battle_send_out_poof_error.is_none()
    {
        let loaded = (|| -> Result<[SpriteFrame; 4]> {
            let data = crate::read_runtime_asset(
                asset_root
                    .runtime_assets()
                    .join("gfx/battle_anims/smoke.2bpp"),
            )
            .context("read battle send-out smoke graphics")?;
            let compose = |tile_offset: usize,
                           expanded: bool,
                           images: &mut Assets<Image>|
             -> Result<SpriteFrame> {
                let mut pixels = vec![0_u8; 40 * 40 * 4];
                let base_positions = [-16_i16, -8, 0, 8];
                let expanded_positions = [-20_i16, -12, 4, 12];
                let positions = if expanded {
                    expanded_positions
                } else {
                    base_positions
                };
                let tile_ids = [0_usize, 1, 1, 0, 2, 3, 3, 2, 2, 3, 3, 2, 0, 1, 1, 0];
                for row in 0..4 {
                    for col in 0..4 {
                        let tile_index = tile_offset + tile_ids[row * 4 + col];
                        let offset = tile_index * 16;
                        let tile = data.get(offset..offset + 16).with_context(|| {
                            format!("battle smoke tile {tile_index} is missing")
                        })?;
                        for source_y in 0..8_usize {
                            for source_x in 0..8_usize {
                                let flip_x = col >= 2;
                                let flip_y = row >= 2;
                                let sample_x = if flip_x { 7 - source_x } else { source_x };
                                let sample_y = if flip_y { 7 - source_y } else { source_y };
                                let sample_lo = tile[sample_y * 2];
                                let sample_hi = tile[sample_y * 2 + 1];
                                let bit = 1 << (7 - sample_x);
                                let level = ((sample_hi & bit != 0) as u8) << 1
                                    | (sample_lo & bit != 0) as u8;
                                if level == 0 {
                                    continue;
                                }
                                let target_x = i32::from(positions[col]) + source_x as i32 + 20;
                                let target_y = i32::from(positions[row]) + source_y as i32 + 20;
                                if !(0..40).contains(&target_x) || !(0..40).contains(&target_y) {
                                    continue;
                                }
                                let output = ((target_y * 40 + target_x) * 4) as usize;
                                let shade = match level {
                                    1 => 224,
                                    2 => 128,
                                    _ => 40,
                                };
                                pixels[output] = shade;
                                pixels[output + 1] = shade;
                                pixels[output + 2] = shade;
                                pixels[output + 3] = 255;
                            }
                        }
                    }
                }
                let mut image = Image::new(
                    Extent3d {
                        width: 40,
                        height: 40,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    pixels,
                    TextureFormat::Rgba8UnormSrgb,
                    RenderAssetUsages::default(),
                );
                image.sampler = ImageSampler::nearest();
                Ok(SpriteFrame {
                    handle: images.add(image),
                    size: Vec2::splat(40.0),
                })
            };
            Ok([
                compose(0, false, images)?,
                compose(4, false, images)?,
                compose(8, false, images)?,
                compose(8, true, images)?,
            ])
        })();
        match loaded {
            Ok(frames) => rendered_art.battle_send_out_poof_cache = Some(frames),
            Err(error) => rendered_art.battle_send_out_poof_error = Some(error.to_string()),
        }
    }
    rendered_art
        .battle_send_out_poof_cache
        .as_ref()
        .with_context(|| {
            rendered_art
                .battle_send_out_poof_error
                .clone()
                .unwrap_or_else(|| "battle send-out poof art is unavailable".to_string())
        })
}

fn spawn_battle_pack_screen(
    commands: &mut Commands,
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<()> {
    let (item_ids, cursor, surface_id, pocket_label, show_quantity) =
        if runtime_shell.ball_cursor.is_some() {
            (
                carried_ball_item_ids(snapshot),
                &runtime_shell.ball_cursor,
                "bag:balls",
                "BALL",
                true,
            )
        } else if runtime_shell.key_item_cursor.is_some() {
            (
                snapshot
                    .bag
                    .key_items
                    .iter()
                    .filter(|item| item.quantity > 0)
                    .map(|item| item.item_id.clone())
                    .collect(),
                &runtime_shell.key_item_cursor,
                "bag:key-items",
                "KEY",
                false,
            )
        } else if runtime_shell.tmhm_cursor.is_some() {
            (
                snapshot
                    .bag
                    .tm_hm
                    .iter()
                    .filter(|item| item.quantity > 0)
                    .map(|item| item.item_id.clone())
                    .collect(),
                &runtime_shell.tmhm_cursor,
                "bag:tmhm",
                "TM/HM",
                true,
            )
        } else {
            (
                carried_battle_non_ball_item_ids(snapshot),
                &runtime_shell.bag_cursor,
                "battle:bag-items",
                "ITEMS",
                true,
            )
        };
    let row_count = field_pack_selectable_count(item_ids.len());
    let selected = strict_readonly_cursor_index(cursor, surface_id, row_count)
        .with_context(|| format!("battle pack surface {surface_id} has no valid cursor"))?;
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                // gfx/pack/pack.pal color zero: RGB 31,31,31.
                color: Color::WHITE,
                custom_size: Some(Vec2::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 3.4),
            ..default()
        },
        BattleCommandMarker,
    ));
    for (row, text) in [
        (1.0, "< PACK >"),
        (4.0, "[BAG]"),
        (7.0, "<     >"),
        (8.0, pocket_label),
    ] {
        let (x, y) = battle_hud_tile_origin(0.0, row);
        spawn_battle_command_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            text,
            x,
            y,
            3.8,
        );
    }
    let list_start = visible_window_start(selected, row_count, 7);
    for visible_index in 0..7 {
        let index = list_start + visible_index;
        if index >= row_count {
            break;
        }
        let row = 2.0 + visible_index as f32;
        let (x, y) = battle_hud_tile_origin(7.0, row);
        if index >= item_ids.len() {
            spawn_battle_command_bitmap_text(
                commands,
                rendered_art,
                asset_root,
                images,
                &format!("{}CANCEL", if index == selected { ">" } else { " " }),
                x,
                y,
                3.8,
            );
            continue;
        }
        let item_id = &item_ids[index];
        let item = snapshot
            .items
            .iter()
            .find(|item| item.item_id == *item_id)
            .with_context(|| format!("battle PACK item {item_id} is missing"))?;
        spawn_battle_command_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            &format!(
                "{}{}",
                if index == selected { ">" } else { " " },
                compact_scene_label(&item.name.replace('_', " "), 8)
            ),
            x,
            y,
            3.8,
        );
        if show_quantity {
            let quantity = carried_item_quantity(snapshot, item_id)
                .or_else(|| {
                    snapshot
                        .bag
                        .tm_hm
                        .iter()
                        .find(|item| item.item_id == *item_id)
                        .map(|item| item.quantity)
                })
                .with_context(|| format!("battle pack item {item_id} has no carried quantity"))?;
            let (x, y) = battle_hud_tile_origin(16.0, row);
            spawn_battle_command_bitmap_text(
                commands,
                rendered_art,
                asset_root,
                images,
                &format!("×{:02}", quantity.min(99)),
                x,
                y,
                3.8,
            );
        }
    }
    let description = if selected >= item_ids.len() {
        "Close the PACK."
    } else {
        item_ids
            .get(selected)
            .and_then(|item_id| snapshot.items.iter().find(|item| item.item_id == *item_id))
            .map(|item| item.description.as_str())
            .with_context(|| format!("battle pack selection {selected} has no item description"))?
    };
    for (index, line) in wrap_boot_text_for_box(description, 18, 4)
        .iter()
        .enumerate()
    {
        let (x, y) = battle_hud_tile_origin(1.0, 13.0 + index as f32);
        spawn_battle_command_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            line,
            x,
            y,
            3.8,
        );
    }
    if let Some(action_cursor) = &runtime_shell.field_pack_action_cursor {
        let pocket = active_visible_field_pack_pocket(runtime_shell);
        let actions = visible_selected_pack_item_actions(snapshot, runtime_shell, &pocket, true)?;
        let action_selected = strict_readonly_cursor_index(
            &Some(action_cursor.clone()),
            "pack:actions",
            actions.len(),
        )
        .context("battle pack action menu has no valid cursor")?;
        let top = match actions.len() {
            4 => 3.0,
            3 => 5.0,
            2 => 7.0,
            _ => 9.0,
        };
        spawn_battle_window(
            commands,
            rendered_art,
            asset_root,
            images,
            13.0,
            top,
            7.0,
            actions.len() as f32 * 2.0 + 1.0,
            4.1,
        );
        for (index, action) in actions.iter().enumerate() {
            let (x, y) = battle_hud_tile_origin(14.0, top + 1.0 + index as f32 * 2.0);
            spawn_battle_command_bitmap_text(
                commands,
                rendered_art,
                asset_root,
                images,
                &format!(
                    "{}{}",
                    if index == action_selected { ">" } else { " " },
                    visible_field_pack_action_label(*action)
                ),
                x,
                y,
                4.3,
            );
        }
    }
    Ok(())
}

fn spawn_battle_party_menu(
    commands: &mut Commands,
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<()> {
    let item_target = runtime_shell.battle_pack_target_mode.is_some();
    let option_count = if item_target {
        snapshot.party.slots.len() + 1
    } else {
        battle_switch_option_count(snapshot)
    };
    let selected = if item_target {
        anyhow::ensure!(
            runtime_shell.party_cursor < option_count,
            "battle item-target party cursor {} is outside {option_count} rows",
            runtime_shell.party_cursor
        );
        runtime_shell.party_cursor
    } else {
        strict_readonly_cursor_index(
            &runtime_shell.battle_switch_cursor,
            "battle:switch",
            option_count,
        )
        .context("battle party menu requires a valid cursor")?
    };
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                // gfx/stats/party_menu_bg.pal color 0: RGB 31,31,31.
                color: Color::srgb(1.0, 1.0, 1.0),
                custom_size: Some(Vec2::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 3.4),
            ..default()
        },
        BattleCommandMarker,
    ));
    for (row_index, slot) in snapshot.party.slots.iter().enumerate().take(6) {
        let name_row = 1.0 + row_index as f32 * 2.0;
        let status_row = name_row + 1.0;
        let marker = if selected == row_index { ">" } else { " " };
        spawn_battle_party_icon(
            commands,
            snapshot,
            slot,
            row_index,
            selected == row_index,
            false,
            rendered_art,
            asset_root,
            images,
        )?;
        let (x, y) = battle_hud_tile_origin(0.0, name_row);
        spawn_battle_command_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            marker,
            x,
            y,
            3.8,
        );
        let (x, y) = battle_hud_tile_origin(3.0, name_row);
        spawn_battle_command_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            &compact_scene_label(&slot.pokemon.nickname, 10),
            x,
            y,
            3.8,
        );
        if slot.pokemon.is_egg {
            continue;
        }
        let (x, y) = battle_hud_tile_origin(13.0, name_row);
        spawn_battle_command_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            &format!(
                "{:>3}/{:>3}",
                slot.pokemon.hp.min(999),
                slot.pokemon.max_hp.min(999)
            ),
            x,
            y,
            3.8,
        );
        let (x, y) = battle_hud_tile_origin(5.0, status_row);
        spawn_battle_command_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            party_status_token(&slot.pokemon),
            x,
            y,
            3.8,
        );
        let (x, y) = battle_hud_tile_origin(8.0, status_row);
        spawn_battle_command_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            &format!("\u{e10a}{:>2}", slot.pokemon.level.min(100)),
            x,
            y,
            3.8,
        );
        spawn_battle_hud_hp_bar(
            commands,
            rendered_art,
            asset_root,
            images,
            11.0,
            status_row,
            slot.pokemon.hp,
            slot.pokemon.max_hp,
            BattleHpSide::Player,
            None,
        )?;
    }
    let cancel_row = 1.0 + snapshot.party.slots.len().min(6) as f32 * 2.0;
    let cancel_marker = if selected >= snapshot.party.slots.len() {
        ">"
    } else {
        " "
    };
    let (x, y) = battle_hud_tile_origin(1.0, cancel_row);
    spawn_battle_command_bitmap_text(
        commands,
        rendered_art,
        asset_root,
        images,
        &format!("{cancel_marker}CANCEL"),
        x,
        y,
        3.8,
    );
    spawn_battle_window(
        commands,
        rendered_art,
        asset_root,
        images,
        0.0,
        14.0,
        20.0,
        4.0,
        3.9,
    );
    let (x, y) = battle_hud_tile_origin(1.0, 15.0);
    spawn_battle_command_bitmap_text(
        commands,
        rendered_art,
        asset_root,
        images,
        if item_target {
            "Use on which <PKMN>?"
        } else if visible_active_battle_player_fainted(snapshot) {
            "Which <PKMN>?"
        } else {
            "Choose a <PKMN>."
        },
        x,
        y,
        4.1,
    );
    if !item_target {
        if runtime_shell.battle_party_action_cursor.is_some() {
            let action_selected = strict_readonly_cursor_index(
                &runtime_shell.battle_party_action_cursor,
                "battle:party-actions",
                3,
            )
            .context("battle party action cursor is invalid")?;
            spawn_battle_window(
                commands,
                rendered_art,
                asset_root,
                images,
                11.0,
                11.0,
                9.0,
                7.0,
                4.2,
            );
            for (index, label) in ["SWITCH", "STATS", "CANCEL"].iter().enumerate() {
                let (x, y) = battle_hud_tile_origin(12.0, 12.0 + index as f32 * 2.0);
                spawn_battle_command_bitmap_text(
                    commands,
                    rendered_art,
                    asset_root,
                    images,
                    &format!(
                        "{}{label}",
                        if index == action_selected { ">" } else { " " }
                    ),
                    x,
                    y,
                    4.4,
                );
            }
        }
    }
    Ok(())
}

fn spawn_battle_pack_move_target_screen(
    commands: &mut Commands,
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<()> {
    let slot = snapshot
        .party
        .slots
        .get(runtime_shell.party_cursor)
        .with_context(|| {
            format!(
                "battle PP-item target party cursor {} is outside {} slots",
                runtime_shell.party_cursor,
                snapshot.party.slots.len()
            )
        })?;
    let selected = strict_readonly_cursor_index(
        &runtime_shell.party_move_cursor,
        &party_move_cursor_surface_id(slot.index),
        slot.pokemon.moves.len(),
    )
    .with_context(|| {
        format!(
            "battle PP-item target move cursor is invalid for party slot {} with {} moves",
            slot.index,
            slot.pokemon.moves.len()
        )
    })?;
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(1.0, 1.0, 1.0),
                custom_size: Some(Vec2::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 3.4),
            ..default()
        },
        BattleCommandMarker,
    ));
    let (x, y) = battle_hud_tile_origin(1.0, 1.0);
    spawn_battle_command_bitmap_text(
        commands,
        rendered_art,
        asset_root,
        images,
        &format!(
            "{} \u{e10a}{:>2}",
            compact_scene_label(&slot.pokemon.nickname, 10),
            slot.pokemon.level
        ),
        x,
        y,
        3.8,
    );
    for (index, learned) in slot.pokemon.moves.iter().enumerate().take(4) {
        let row = 3.0 + index as f32 * 2.0;
        let (x, y) = battle_hud_tile_origin(1.0, row);
        spawn_battle_command_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            &format!(
                "{}{}",
                if index == selected { ">" } else { " " },
                battle_move_display_name(snapshot, &learned.name)
            ),
            x,
            y,
            3.8,
        );
        let (x, y) = battle_hud_tile_origin(10.0, row + 1.0);
        spawn_battle_command_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            &visible_move_pp_text(snapshot, learned),
            x,
            y,
            3.8,
        );
    }
    let item_ids = carried_battle_usable_item_ids(snapshot);
    let item_index = strict_readonly_cursor_index(
        &runtime_shell.bag_cursor,
        "battle:bag-items",
        item_ids.len(),
    )
    .with_context(|| {
        format!(
            "battle PP-item target bag cursor is invalid for {} usable items",
            item_ids.len()
        )
    })?;
    let item_id = item_ids
        .get(item_index)
        .context("battle PP-item target selection is missing from the usable item list")?;
    let item = snapshot
        .items
        .iter()
        .find(|item| item.item_id == *item_id)
        .with_context(|| format!("battle PP-item target item {item_id} is missing"))?;
    let raises_pp = item.pp_up_stages.is_some();
    let prompt = if raises_pp {
        ["Raise the PP of", "which move?"]
    } else {
        ["Restore the PP of", "which move?"]
    };
    for (line_index, line) in prompt.iter().enumerate() {
        let (x, y) = battle_hud_tile_origin(1.0, 13.0 + line_index as f32);
        spawn_battle_command_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            line,
            x,
            y,
            3.8,
        );
    }
    Ok(())
}

fn spawn_battle_party_icon(
    commands: &mut Commands,
    snapshot: &RuntimeShellSnapshot,
    slot: &crate::RuntimePartySlotSnapshot,
    row_index: usize,
    selected: bool,
    field_surface: bool,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<()> {
    let species_id = if slot.pokemon.is_egg {
        "EGG"
    } else {
        slot.pokemon.species.id.as_str()
    };
    let icon_id = snapshot
        .presentation
        .menu_icons
        .get(species_id)
        .with_context(|| format!("party icon mapping missing species {species_id}"))?
        .clone();
    if !rendered_art.party_icon_cache.contains_key(&icon_id)
        && !rendered_art.party_icon_errors.contains_key(&icon_id)
    {
        match load_party_icon_frame(asset_root, &icon_id, images) {
            Ok(frames) => {
                rendered_art
                    .party_icon_cache
                    .insert(icon_id.clone(), frames);
            }
            Err(error) => {
                rendered_art
                    .party_icon_errors
                    .insert(icon_id.clone(), error.to_string());
            }
        }
    }
    let frames = rendered_art
        .party_icon_cache
        .get(&icon_id)
        .with_context(|| {
            rendered_art
                .party_icon_errors
                .get(&icon_id)
                .cloned()
                .unwrap_or_else(|| format!("party icon {icon_id} is unavailable"))
        })?;
    let max_hp = slot.pokemon.max_hp.max(1);
    let hp = slot.pokemon.hp.min(max_hp);
    let (duration, bob_amplitude) = if u32::from(hp) * 2 >= u32::from(max_hp) {
        (8_u64, 2.0_f32)
    } else if u32::from(hp) * 100 >= u32::from(max_hp) * 21 {
        (72_u64, 1.0_f32)
    } else {
        (136_u64, 0.0_f32)
    };
    let frame_counter = snapshot.state_checksum.frame();
    let frame = &frames[((frame_counter / duration) & 1) as usize];
    let bob_pixels = if selected && bob_amplitude > 0.0 && (frame_counter / 16) & 1 != 0 {
        bob_amplitude
    } else {
        0.0
    };
    // ASM OAM starts the first 16x16 icon at LCD pixel (8,4), then advances
    // exactly 16 pixels per party slot.
    let (x, y) = battle_hud_tile_origin(1.5, 1.0 + row_index as f32 * 2.0);
    let mut icon = commands.spawn(SpriteBundle {
        texture: frame.handle.clone(),
        sprite: Sprite {
            custom_size: Some(frame.size),
            ..default()
        },
        transform: Transform::from_xyz(
            x,
            y + bob_pixels * (TILE_SIZE / SOURCE_TILE_SIZE as f32),
            3.85,
        ),
        ..default()
    });
    if field_surface {
        icon.insert(FieldCommandMarker);
    } else {
        icon.insert(BattleCommandMarker);
    }
    if let Some(item_id) = slot.pokemon.item.as_deref() {
        let overlay_id = if item_id.ends_with("_MAIL") {
            "mail"
        } else {
            "item"
        };
        spawn_battle_party_icon_overlay(
            commands,
            overlay_id,
            row_index,
            bob_pixels,
            field_surface,
            rendered_art,
            asset_root,
            images,
        )?;
    }
    Ok(())
}

fn spawn_battle_party_icon_overlay(
    commands: &mut Commands,
    overlay_id: &str,
    row_index: usize,
    bob_pixels: f32,
    field_surface: bool,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<()> {
    if !rendered_art
        .party_icon_overlay_cache
        .contains_key(overlay_id)
        && !rendered_art
            .party_icon_overlay_errors
            .contains_key(overlay_id)
    {
        match load_party_icon_overlay(asset_root, overlay_id, images) {
            Ok(frame) => {
                rendered_art
                    .party_icon_overlay_cache
                    .insert(overlay_id.to_string(), frame);
            }
            Err(error) => {
                rendered_art
                    .party_icon_overlay_errors
                    .insert(overlay_id.to_string(), error.to_string());
            }
        }
    }
    let frame = rendered_art
        .party_icon_overlay_cache
        .get(overlay_id)
        .with_context(|| {
            rendered_art
                .party_icon_overlay_errors
                .get(overlay_id)
                .cloned()
                .unwrap_or_else(|| format!("party icon overlay {overlay_id} is unavailable"))
        })?;
    let (x, y) = battle_hud_tile_origin(1.0, 1.5 + row_index as f32 * 2.0);
    let mut overlay = commands.spawn(SpriteBundle {
        texture: frame.handle.clone(),
        sprite: Sprite {
            custom_size: Some(frame.size),
            ..default()
        },
        transform: Transform::from_xyz(
            x,
            y + bob_pixels * (TILE_SIZE / SOURCE_TILE_SIZE as f32),
            3.9,
        ),
        ..default()
    });
    if field_surface {
        overlay.insert(FieldCommandMarker);
    } else {
        overlay.insert(BattleCommandMarker);
    }
    Ok(())
}

fn load_party_icon_overlay(
    asset_root: &AssetRoot,
    overlay_id: &str,
    images: &mut Assets<Image>,
) -> Result<SpriteFrame> {
    let assets = asset_root.runtime_assets();
    let palette_path = assets.join("gfx/stats/party_menu_ob.pal");
    let palette_content = crate::read_runtime_asset_to_string(&palette_path)
        .with_context(|| format!("read party icon palette {}", palette_path.display()))?;
    let palette = parse_palette_file(&palette_content, None)?
        .into_iter()
        .next()
        .context("party icon palette must define one four-color palette")?;
    let path = assets.join("gfx/stats").join(format!("{overlay_id}.2bpp"));
    let data = crate::read_runtime_asset(&path)
        .with_context(|| format!("read party icon overlay {}", path.display()))?;
    if data.len() != 16 {
        anyhow::bail!(
            "party icon overlay {} must contain exactly one 2bpp tile, found {} bytes",
            path.display(),
            data.len()
        );
    }
    let mut rgba = vec![0_u8; 8 * 8 * 4];
    for row in 0..8 {
        let lo = data[row * 2];
        let hi = data[row * 2 + 1];
        for col in 0..8 {
            let bit = 1 << (7 - col);
            let palette_index = (((hi & bit != 0) as usize) << 1) | (lo & bit != 0) as usize;
            let offset = (row * 8 + col) * 4;
            rgba[offset..offset + 3].copy_from_slice(&palette[palette_index]);
            rgba[offset + 3] = if palette_index == 0 { 0 } else { 255 };
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: 8,
            height: 8,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    Ok(SpriteFrame {
        handle: images.add(image),
        size: Vec2::splat(TILE_SIZE),
    })
}

fn load_party_icon_frame(
    asset_root: &AssetRoot,
    icon_id: &str,
    images: &mut Assets<Image>,
) -> Result<[SpriteFrame; 2]> {
    let assets = asset_root.runtime_assets();
    let palette_path = assets.join("gfx/stats/party_menu_ob.pal");
    let palette_content = crate::read_runtime_asset_to_string(&palette_path)
        .with_context(|| format!("read party icon palette {}", palette_path.display()))?;
    let palette = parse_palette_file(&palette_content, None)?
        .into_iter()
        .next()
        .context("party icon palette must define one four-color palette")?;
    let stem = icon_id
        .strip_prefix("ICON_")
        .unwrap_or(icon_id)
        .to_ascii_lowercase();
    let icon_path = assets.join("gfx/icons").join(format!("{stem}.2bpp"));
    let data = crate::read_runtime_asset(&icon_path)
        .with_context(|| format!("read party icon {}", icon_path.display()))?;
    if data.len() < 8 * 16 || data.len() % 16 != 0 {
        anyhow::bail!(
            "party icon {} must contain two 16x16 frames (at least 128 aligned bytes), found {}",
            icon_path.display(),
            data.len()
        );
    }
    let mut decoded = Vec::with_capacity(2);
    for frame_index in 0..2 {
        let mut rgba = vec![0_u8; 16 * 16 * 4];
        for tile_row in 0..2 {
            for tile_col in 0..2 {
                let tile_index = frame_index * 4 + tile_row * 2 + tile_col;
                let tile = &data[tile_index * 16..tile_index * 16 + 16];
                for row in 0..8 {
                    let lo = tile[row * 2];
                    let hi = tile[row * 2 + 1];
                    for col in 0..8 {
                        let bit = 1 << (7 - col);
                        let palette_index =
                            (((hi & bit != 0) as usize) << 1) | (lo & bit != 0) as usize;
                        let target_x = tile_col * 8 + col;
                        let target_y = tile_row * 8 + row;
                        let offset = (target_y * 16 + target_x) * 4;
                        rgba[offset..offset + 3].copy_from_slice(&palette[palette_index]);
                        rgba[offset + 3] = if palette_index == 0 { 0 } else { 255 };
                    }
                }
            }
        }
        let mut image = Image::new(
            Extent3d {
                width: 16,
                height: 16,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            rgba,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        image.sampler = ImageSampler::nearest();
        decoded.push(SpriteFrame {
            handle: images.add(image),
            size: Vec2::splat(TILE_SIZE * 2.0),
        });
    }
    decoded
        .try_into()
        .map_err(|_| anyhow::anyhow!("party icon decoder did not produce two frames"))
}

fn spawn_battle_yes_no_prompt(
    commands: &mut Commands,
    runtime_shell: &BevyRuntimeShell,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
    entries: &[String],
) {
    spawn_battle_window(
        commands,
        rendered_art,
        asset_root,
        images,
        BATTLE_TEXT_BOX_LEFT_TILE,
        BATTLE_TEXT_BOX_TOP_TILE,
        BATTLE_TEXT_BOX_WIDTH_TILES,
        BATTLE_TEXT_BOX_HEIGHT_TILES,
        3.5,
    );
    spawn_battle_window(
        commands,
        rendered_art,
        asset_root,
        images,
        FIELD_YES_NO_LEFT_TILE,
        FIELD_YES_NO_TOP_TILE,
        FIELD_YES_NO_WIDTH_TILES,
        FIELD_YES_NO_HEIGHT_TILES,
        3.9,
    );
    let prompt_count = entries.len().saturating_sub(2);
    let mut prompt_lines = Vec::new();
    for entry in entries.iter().take(prompt_count) {
        prompt_lines.extend(wrap_boot_text_for_box(entry, 18, 4));
    }
    for (line_index, line) in prompt_lines.into_iter().take(4).enumerate() {
        let (x, y) = battle_hud_tile_origin(1.0, 13.0 + line_index as f32);
        spawn_battle_command_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            &line,
            x,
            y,
            3.8,
        );
    }
    for (index, entry) in entries.iter().skip(prompt_count).enumerate() {
        let (x, y) = battle_hud_tile_origin(
            // `entry` already contains the cursor/blank prefix. Crystal's
            // menu header places that prefix on the window's left tile and
            // the YES/NO label one tile into the window.
            FIELD_YES_NO_LEFT_TILE,
            FIELD_YES_NO_TOP_TILE + 1.0 + index as f32,
        );
        spawn_battle_command_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            &animated_battle_cursor_entry(runtime_shell, entry),
            x,
            y,
            4.1,
        );
    }
}

fn battle_submenu_entry_tile(index: usize, two_columns: bool) -> (f32, f32) {
    if two_columns {
        let row = index / 2;
        let col = index % 2;
        (
            BATTLE_SUBMENU_ORIGIN_TILE_X + col as f32 * BATTLE_SUBMENU_COLUMN_SPACING_TILES,
            BATTLE_SUBMENU_ORIGIN_TILE_Y + row as f32 * BATTLE_SUBMENU_ROW_SPACING_TILES,
        )
    } else {
        (
            BATTLE_SUBMENU_ORIGIN_TILE_X,
            BATTLE_SUBMENU_ORIGIN_TILE_Y + index as f32 * BATTLE_SUBMENU_ROW_SPACING_TILES,
        )
    }
}

fn spawn_battle_main_command_menu(
    commands: &mut Commands,
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
    battle: &crate::RuntimeBattleSnapshot,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
    entries: &[String],
) -> Result<()> {
    let contest_menu = entries.get(2).is_some_and(|entry| {
        entry
            .trim_start_matches(|ch| ch == '>' || ch == ' ')
            .starts_with("PARKBALL×")
    });
    let (left, width) = if contest_menu {
        (2.0, 18.0)
    } else {
        (BATTLE_MAIN_MENU_LEFT_TILE, BATTLE_MAIN_MENU_WIDTH_TILES)
    };
    spawn_battle_window(
        commands,
        rendered_art,
        asset_root,
        images,
        BATTLE_TEXT_BOX_LEFT_TILE,
        BATTLE_TEXT_BOX_TOP_TILE,
        BATTLE_TEXT_BOX_WIDTH_TILES,
        BATTLE_TEXT_BOX_HEIGHT_TILES,
        3.45,
    );
    spawn_battle_window(
        commands,
        rendered_art,
        asset_root,
        images,
        left,
        BATTLE_MAIN_MENU_TOP_TILE,
        width,
        BATTLE_MAIN_MENU_HEIGHT_TILES,
        3.5,
    );
    if !contest_menu {
        let prompt_name = if battle.battle_type == "BATTLETYPE_TUTORIAL" {
            "DUDE".to_string()
        } else {
            let active = battle
                .active_player_party_index
                .context("battle command menu requires an active party slot")?;
            snapshot
                .party
                .slots
                .iter()
                .find(|slot| slot.index == active)
                .with_context(|| {
                    format!("battle command menu active party slot {active} is missing")
                })?
                .pokemon
                .nickname
                .clone()
        };
        for (line_index, line) in
            wrap_boot_text_for_box(&format!("What will {prompt_name} do?"), 7, 4)
                .iter()
                .enumerate()
        {
            let (x, y) = battle_hud_tile_origin(1.0, 13.0 + line_index as f32);
            spawn_battle_command_bitmap_text(
                commands,
                rendered_art,
                asset_root,
                images,
                line,
                x,
                y,
                3.8,
            );
        }
    }
    for (index, entry) in entries.iter().enumerate() {
        let (tile_x, tile_y) = if contest_menu {
            let row = index / 2;
            let col = index % 2;
            (3.0 + col as f32 * 12.0, 13.0 + row as f32 * 2.0)
        } else {
            battle_main_menu_entry_tile(index)
        };
        let (x, y) = battle_hud_tile_origin(tile_x, tile_y);
        spawn_battle_command_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            &compact_scene_label(
                &animated_battle_cursor_entry(runtime_shell, entry),
                if contest_menu { 12 } else { 7 },
            ),
            x,
            y,
            3.8,
        );
    }
    Ok(())
}

fn battle_command_entries_are_main_menu(entries: &[String]) -> bool {
    if entries.len() != BATTLE_MAIN_MENU_LABELS.len() {
        return false;
    }
    if entries.get(2).is_some_and(|entry| {
        entry
            .trim_start_matches(|ch| ch == '>' || ch == ' ')
            .starts_with("PARKBALL×")
    }) {
        return entries
            .iter()
            .enumerate()
            .all(|(index, entry)| match index {
                0 => entry.trim_start_matches(|ch| ch == '>' || ch == ' ').trim() == "FIGHT",
                1 => entry.trim_start_matches(|ch| ch == '>' || ch == ' ').trim() == "<PKMN>",
                2 => true,
                3 => entry.trim_start_matches(|ch| ch == '>' || ch == ' ').trim() == "RUN",
                _ => false,
            });
    }
    entries
        .iter()
        .zip(BATTLE_MAIN_MENU_LABELS)
        .all(|(entry, label)| {
            let trimmed = entry.trim_start_matches(|ch| ch == '>' || ch == ' ').trim();
            trimmed == label
        })
}

#[cfg(test)]
fn battle_main_menu_panel_center() -> (f32, f32) {
    (
        PLAYFIELD_LEFT
            + (BATTLE_MAIN_MENU_LEFT_TILE + BATTLE_MAIN_MENU_WIDTH_TILES * 0.5) * TILE_SIZE,
        PLAYFIELD_TOP
            - (BATTLE_MAIN_MENU_TOP_TILE + BATTLE_MAIN_MENU_HEIGHT_TILES * 0.5) * TILE_SIZE,
    )
}

fn battle_main_menu_entry_tile(index: usize) -> (f32, f32) {
    let row = index / 2;
    let col = index % 2;
    (
        // The menu rectangle begins at x=8. Its cursor occupies the first
        // inner tile at x=9, and the prefixed label therefore starts at x=10.
        BATTLE_MAIN_MENU_ORIGIN_TILE_X + col as f32 * BATTLE_MAIN_MENU_COLUMN_SPACING_TILES,
        BATTLE_MAIN_MENU_ORIGIN_TILE_Y + row as f32 * BATTLE_MAIN_MENU_ROW_SPACING_TILES,
    )
}

fn spawn_battle_move_menu(
    commands: &mut Commands,
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
    battle: &crate::RuntimeBattleSnapshot,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<()> {
    let (_slot, cursor_index, visible_rows, start) =
        battle_move_menu_state(snapshot, runtime_shell, battle)?;
    spawn_battle_window(
        commands,
        rendered_art,
        asset_root,
        images,
        BATTLE_MOVE_SELECTION_LEFT_TILE,
        BATTLE_MOVE_SELECTION_TOP_TILE,
        BATTLE_MOVE_SELECTION_WIDTH_TILES,
        BATTLE_MOVE_SELECTION_HEIGHT_TILES,
        3.5,
    );
    let total = battle.player_moves.len() + 1;
    for visible_index in 0..visible_rows {
        let index = start + visible_index;
        if index >= total {
            break;
        }
        let marker = if runtime_shell.battle_move_swap_origin == Some(index) {
            "▷"
        } else if index == cursor_index {
            battle_cursor_glyph(runtime_shell)
        } else {
            " "
        };
        let label = if index < battle.player_moves.len() {
            battle_move_display_name(snapshot, &battle.player_moves[index].name)
        } else {
            "CANCEL".to_string()
        };
        let (tile_x, tile_y) = battle_move_menu_entry_tile(visible_index);
        let (x, y) = battle_hud_tile_origin(tile_x, tile_y);
        spawn_battle_command_bitmap_text(
            commands,
            rendered_art,
            asset_root,
            images,
            &compact_scene_label(&format!("{marker}{label}"), 12),
            x,
            y,
            3.8,
        );
    }
    spawn_battle_move_info_window(
        commands,
        snapshot,
        battle,
        &battle.player_moves,
        cursor_index,
        rendered_art,
        asset_root,
        images,
    )?;
    Ok(())
}

fn spawn_battle_window(
    commands: &mut Commands,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
    tile_x: f32,
    tile_y: f32,
    width_tiles: f32,
    height_tiles: f32,
    z: f32,
) {
    let (center_x, center_y) = battle_window_center(tile_x, tile_y, width_tiles, height_tiles);
    if width_tiles > 2.0 && height_tiles > 2.0 {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(
                        TILE_SIZE * (width_tiles - 2.0),
                        TILE_SIZE * (height_tiles - 2.0),
                    )),
                    ..default()
                },
                transform: Transform::from_xyz(center_x, center_y, z),
                ..default()
            },
            BattleCommandMarker,
        ));
    }
    if let Some(frame) = battle_window_frame_art(rendered_art, asset_root, images) {
        let width = width_tiles.round().max(0.0) as usize;
        let height = height_tiles.round().max(0.0) as usize;
        if width >= 2 && height >= 2 {
            spawn_battle_window_frame_tiles(
                commands,
                frame,
                tile_x,
                tile_y,
                width,
                height,
                z + 0.05,
            );
            return;
        }
    }
}

fn spawn_battle_move_info_window(
    commands: &mut Commands,
    snapshot: &RuntimeShellSnapshot,
    battle: &crate::RuntimeBattleSnapshot,
    moves: &[crate::core::models::LearnedMove],
    cursor_index: usize,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<()> {
    spawn_battle_window(
        commands,
        rendered_art,
        asset_root,
        images,
        BATTLE_MOVE_INFO_LEFT_TILE,
        BATTLE_MOVE_INFO_TOP_TILE,
        BATTLE_MOVE_INFO_WIDTH_TILES,
        BATTLE_MOVE_INFO_HEIGHT_TILES,
        3.5,
    );
    let selected = moves.get(cursor_index);
    if let Some(selected) = selected {
        if battle.player_disabled_move.as_deref() == Some(selected.name.as_str()) {
            spawn_battle_move_info_text(
                commands,
                rendered_art,
                asset_root,
                images,
                "Disabled!",
                1.0,
                10.0,
            );
            return Ok(());
        }
        let move_data = snapshot
            .moves
            .iter()
            .find(|move_data| move_data.move_id == selected.name)
            .with_context(|| {
                format!(
                    "battle move info is missing move metadata for {}",
                    selected.name
                )
            })?;
        let move_type = move_data.move_type.as_str();
        let max_pp = crate::core::models::max_move_pp(move_data.pp, selected.pp_ups);
        spawn_battle_move_info_text(
            commands,
            rendered_art,
            asset_root,
            images,
            "TYPE/",
            1.0,
            9.0,
        );
        spawn_battle_move_info_text(
            commands,
            rendered_art,
            asset_root,
            images,
            &compact_scene_label(source_type_display_name(move_type)?, 10),
            2.0,
            10.0,
        );
        spawn_battle_move_info_text(commands, rendered_art, asset_root, images, "PP", 1.0, 11.0);
        spawn_battle_move_info_text(
            commands,
            rendered_art,
            asset_root,
            images,
            &format!("{:>2}/{:>2}", selected.current_pp.min(99), max_pp.min(99)),
            5.0,
            11.0,
        );
    } else {
        spawn_battle_move_info_text(
            commands,
            rendered_art,
            asset_root,
            images,
            "TYPE/",
            1.0,
            9.0,
        );
        spawn_battle_move_info_text(
            commands,
            rendered_art,
            asset_root,
            images,
            "----",
            2.0,
            10.0,
        );
        spawn_battle_move_info_text(
            commands,
            rendered_art,
            asset_root,
            images,
            "PP --/--",
            1.0,
            11.0,
        );
    }
    Ok(())
}

fn spawn_battle_move_info_text(
    commands: &mut Commands,
    rendered_art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
    text: &str,
    tile_x: f32,
    tile_y: f32,
) {
    let (x, y) = battle_hud_tile_origin(tile_x, tile_y);
    spawn_battle_command_bitmap_text(commands, rendered_art, asset_root, images, text, x, y, 3.8);
}

fn battle_window_frame_art<'a>(
    rendered_art: &'a mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Option<&'a WindowFrameArt> {
    let frame_id = rendered_art.selected_window_frame_id.clamp(1, 8);
    window_frame_art(rendered_art, asset_root, images, frame_id)
}

fn window_frame_art<'a>(
    rendered_art: &'a mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
    frame_id: u8,
) -> Option<&'a WindowFrameArt> {
    if !rendered_art.window_frame_cache.contains_key(&frame_id)
        && !rendered_art.window_frame_errors.contains_key(&frame_id)
    {
        match load_window_frame_art(asset_root, frame_id, images) {
            Ok(frame) => {
                rendered_art.window_frame_cache.insert(frame_id, frame);
            }
            Err(error) => {
                rendered_art
                    .window_frame_errors
                    .insert(frame_id, error.to_string());
            }
        }
    }
    rendered_art.window_frame_cache.get(&frame_id)
}

fn spawn_battle_window_frame_tiles(
    commands: &mut Commands,
    frame: &WindowFrameArt,
    tile_x: f32,
    tile_y: f32,
    width: usize,
    height: usize,
    z: f32,
) {
    let top_y = tile_y;
    let bottom_y = tile_y + height.saturating_sub(1) as f32;
    let left_x = tile_x;
    let right_x = tile_x + width.saturating_sub(1) as f32;
    spawn_battle_window_frame_tile(commands, &frame.top_left, left_x, top_y, z);
    spawn_battle_window_frame_tile(commands, &frame.top_right, right_x, top_y, z);
    spawn_battle_window_frame_tile(commands, &frame.bottom_left, left_x, bottom_y, z);
    spawn_battle_window_frame_tile(commands, &frame.bottom_right, right_x, bottom_y, z);

    for col in 1..width.saturating_sub(1) {
        let x = tile_x + col as f32;
        spawn_battle_window_frame_tile(commands, &frame.top_edge, x, top_y, z);
        spawn_battle_window_frame_tile(commands, &frame.top_edge, x, bottom_y, z);
    }
    for row in 1..height.saturating_sub(1) {
        let y = tile_y + row as f32;
        spawn_battle_window_frame_tile(commands, &frame.side_edge, left_x, y, z);
        spawn_battle_window_frame_tile(commands, &frame.side_edge, right_x, y, z);
    }
}

fn battle_window_frame_tile_count(width: usize, height: usize) -> usize {
    if width < 2 || height < 2 {
        return 0;
    }
    width * 2 + height.saturating_sub(2) * 2
}

fn spawn_battle_window_frame_tile(
    commands: &mut Commands,
    frame: &SpriteFrame,
    tile_x: f32,
    tile_y: f32,
    z: f32,
) {
    let (x, y) = battle_hud_tile_origin(tile_x, tile_y);
    commands.spawn((
        SpriteBundle {
            texture: frame.handle.clone(),
            sprite: Sprite {
                custom_size: Some(frame.size),
                ..default()
            },
            transform: Transform::from_xyz(x, y, z),
            ..default()
        },
        BattleCommandMarker,
        BattleWindowFrameMarker,
    ));
}

fn spawn_scene_dialog_window_frame_tiles(
    commands: &mut Commands,
    frame: &WindowFrameArt,
    tile_x: f32,
    tile_y: f32,
    width: usize,
    height: usize,
    z: f32,
) {
    let top_y = tile_y;
    let bottom_y = tile_y + height.saturating_sub(1) as f32;
    let left_x = tile_x;
    let right_x = tile_x + width.saturating_sub(1) as f32;
    spawn_scene_dialog_window_frame_tile(commands, &frame.top_left, left_x, top_y, z);
    spawn_scene_dialog_window_frame_tile(commands, &frame.top_right, right_x, top_y, z);
    spawn_scene_dialog_window_frame_tile(commands, &frame.bottom_left, left_x, bottom_y, z);
    spawn_scene_dialog_window_frame_tile(commands, &frame.bottom_right, right_x, bottom_y, z);

    for col in 1..width.saturating_sub(1) {
        let x = tile_x + col as f32;
        spawn_scene_dialog_window_frame_tile(commands, &frame.top_edge, x, top_y, z);
        spawn_scene_dialog_window_frame_tile(commands, &frame.top_edge, x, bottom_y, z);
    }
    for row in 1..height.saturating_sub(1) {
        let y = tile_y + row as f32;
        spawn_scene_dialog_window_frame_tile(commands, &frame.side_edge, left_x, y, z);
        spawn_scene_dialog_window_frame_tile(commands, &frame.side_edge, right_x, y, z);
    }
}

fn spawn_scene_dialog_window_frame_tile(
    commands: &mut Commands,
    frame: &SpriteFrame,
    tile_x: f32,
    tile_y: f32,
    z: f32,
) {
    let (x, y) = battle_hud_tile_origin(tile_x, tile_y);
    commands.spawn((
        SpriteBundle {
            texture: frame.handle.clone(),
            sprite: Sprite {
                custom_size: Some(frame.size),
                ..default()
            },
            transform: Transform::from_xyz(x, y, z),
            ..default()
        },
        SceneDialogMarker,
        SceneDialogWindowFrameMarker,
    ));
}

fn spawn_field_command_window_frame_tiles(
    commands: &mut Commands,
    frame: &WindowFrameArt,
    tile_x: f32,
    tile_y: f32,
    width: usize,
    height: usize,
    z: f32,
) {
    let top_y = tile_y;
    let bottom_y = tile_y + height.saturating_sub(1) as f32;
    let left_x = tile_x;
    let right_x = tile_x + width.saturating_sub(1) as f32;
    spawn_field_command_window_frame_tile(commands, &frame.top_left, left_x, top_y, z);
    spawn_field_command_window_frame_tile(commands, &frame.top_right, right_x, top_y, z);
    spawn_field_command_window_frame_tile(commands, &frame.bottom_left, left_x, bottom_y, z);
    spawn_field_command_window_frame_tile(commands, &frame.bottom_right, right_x, bottom_y, z);

    for col in 1..width.saturating_sub(1) {
        let x = tile_x + col as f32;
        spawn_field_command_window_frame_tile(commands, &frame.top_edge, x, top_y, z);
        spawn_field_command_window_frame_tile(commands, &frame.top_edge, x, bottom_y, z);
    }
    for row in 1..height.saturating_sub(1) {
        let y = tile_y + row as f32;
        spawn_field_command_window_frame_tile(commands, &frame.side_edge, left_x, y, z);
        spawn_field_command_window_frame_tile(commands, &frame.side_edge, right_x, y, z);
    }
}

fn spawn_field_command_window_frame_tile(
    commands: &mut Commands,
    frame: &SpriteFrame,
    tile_x: f32,
    tile_y: f32,
    z: f32,
) {
    let (x, y) = battle_hud_tile_origin(tile_x, tile_y);
    commands.spawn((
        SpriteBundle {
            texture: frame.handle.clone(),
            sprite: Sprite {
                custom_size: Some(frame.size),
                ..default()
            },
            transform: Transform::from_xyz(x, y, z),
            ..default()
        },
        FieldCommandMarker,
        FieldCommandWindowFrameMarker,
    ));
}

fn battle_move_menu_state<'a>(
    snapshot: &'a RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
    battle: &crate::RuntimeBattleSnapshot,
) -> Result<(&'a crate::RuntimePartySlotSnapshot, usize, usize, usize)> {
    let active_index = battle
        .active_player_party_index
        .context("battle move menu requires an active player party slot")?;
    let slot = snapshot
        .party
        .slots
        .iter()
        .find(|slot| slot.index == active_index)
        .with_context(|| {
            format!("active player party slot {active_index} is absent from the move menu")
        })?;
    let total = battle.player_moves.len() + 1;
    let cursor_index =
        strict_readonly_cursor_index(&runtime_shell.battle_move_cursor, "battle:moves", total)
            .with_context(|| format!("battle move cursor is invalid for {total} entries"))?;
    let visible_rows = battle_move_visible_rows(total);
    let start = if total > visible_rows {
        cursor_index
            .saturating_sub(visible_rows - 1)
            .min(total - visible_rows)
    } else {
        0
    };
    Ok((slot, cursor_index, visible_rows, start))
}

fn battle_move_menu_option_count(
    snapshot: &RuntimeShellSnapshot,
    battle: &crate::RuntimeBattleSnapshot,
) -> Result<usize> {
    let active_index = battle
        .active_player_party_index
        .context("battle move menu requires an active player party slot")?;
    snapshot
        .party
        .slots
        .iter()
        .find(|slot| slot.index == active_index)
        .with_context(|| {
            format!("active player party slot {active_index} is absent from the move menu")
        })?;
    anyhow::ensure!(
        battle.commands.player_move_slots.len() == battle.player_moves.len(),
        "battle move menu exposes {} moves but command selection exposes {} slots",
        battle.player_moves.len(),
        battle.commands.player_move_slots.len(),
    );
    Ok(battle.player_moves.len() + 1)
}

fn battle_move_menu_option_count_for_slot(slot: &crate::RuntimePartySlotSnapshot) -> usize {
    slot.pokemon.moves.len() + 1
}

fn battle_move_visible_rows(total: usize) -> usize {
    let inner_height = BATTLE_MOVE_SELECTION_HEIGHT_TILES - 2.0;
    let rows = (inner_height / BATTLE_MOVE_MENU_ROW_SPACING_TILES)
        .floor()
        .max(1.0) as usize;
    total.min(rows)
}

fn battle_window_center(
    tile_x: f32,
    tile_y: f32,
    width_tiles: f32,
    height_tiles: f32,
) -> (f32, f32) {
    (
        PLAYFIELD_LEFT + (tile_x + width_tiles * 0.5) * TILE_SIZE,
        PLAYFIELD_TOP - (tile_y + height_tiles * 0.5) * TILE_SIZE,
    )
}

/// Field and battle windows share the native 20x18 LCD coordinate space.
fn field_window_center(
    tile_x: f32,
    tile_y: f32,
    width_tiles: f32,
    height_tiles: f32,
) -> (f32, f32) {
    (
        PLAYFIELD_LEFT + (tile_x + width_tiles * 0.5) * TILE_SIZE,
        PLAYFIELD_TOP - (tile_y + height_tiles * 0.5) * TILE_SIZE,
    )
}

fn battle_move_menu_entry_tile(visible_index: usize) -> (f32, f32) {
    (
        // The rendered string owns the cursor tile; the move name itself
        // still begins at the ASM move-menu origin (6, 13).
        BATTLE_MOVE_MENU_ORIGIN_TILE_X - 1.0,
        BATTLE_MOVE_MENU_ORIGIN_TILE_Y + visible_index as f32 * BATTLE_MOVE_MENU_ROW_SPACING_TILES,
    )
}

fn battle_cursor_glyph(_runtime_shell: &BevyRuntimeShell) -> &'static str {
    "▶"
}

fn animated_battle_cursor_entry(runtime_shell: &BevyRuntimeShell, entry: &str) -> String {
    entry
        .strip_prefix('>')
        .map(|label| format!("{}{label}", battle_cursor_glyph(runtime_shell)))
        .unwrap_or_else(|| entry.to_string())
}

fn battle_move_display_name(snapshot: &RuntimeShellSnapshot, move_id: &str) -> String {
    snapshot
        .moves
        .iter()
        .find(|move_data| move_data.move_id == move_id)
        .map(|move_data| move_data.name.replace('_', " "))
        .unwrap_or_else(|| format!("INVALID MOVE {move_id}"))
}

fn visible_move_pp_text(
    snapshot: &RuntimeShellSnapshot,
    learned: &crate::core::models::LearnedMove,
) -> String {
    snapshot
        .moves
        .iter()
        .find(|entry| entry.move_id == learned.name)
        .map(|entry| {
            let max_pp = crate::core::models::max_move_pp(entry.pp, learned.pp_ups);
            format!("PP {:>2}/{:>2}", learned.current_pp, max_pp)
        })
        .unwrap_or_else(|| "PP INVALID".to_string())
}

fn item_display_name(snapshot: &RuntimeShellSnapshot, item_id: &str) -> String {
    snapshot
        .items
        .iter()
        .find(|item| item.item_id == item_id)
        .map(|item| item.name.replace('_', " "))
        .unwrap_or_else(|| format!("INVALID ITEM {item_id}"))
}

fn source_type_display_name(type_id: &str) -> Result<&str> {
    // constants/type_constants.asm indexes data/types/names.asm.
    match type_id {
        "NORMAL" | "FIGHTING" | "FLYING" | "POISON" | "GROUND" | "ROCK"
        | "BIRD" | "BUG" | "GHOST" | "STEEL" | "FIRE" | "WATER" | "GRASS"
        | "ELECTRIC" | "ICE" | "DRAGON" | "DARK" => Ok(type_id),
        "PSYCHIC_TYPE" => Ok("PSYCHIC"),
        "CURSE_TYPE" => Ok("???"),
        _ => anyhow::bail!("type {type_id:?} has no source TypeNames entry"),
    }
}

fn visible_battle_command_menu_entries(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
    battle: &crate::RuntimeBattleSnapshot,
) -> Result<Vec<String>> {
    if runtime_shell.battle_faint_prompt_cursor.is_some() {
        return visible_battle_faint_prompt_entries(runtime_shell);
    }
    if runtime_shell.battle_shift_prompt_cursor.is_some() {
        return visible_battle_shift_prompt_entries(snapshot, runtime_shell, battle);
    }
    if let Some(mode) = runtime_shell.battle_pack_target_mode {
        return visible_battle_pack_target_entries(snapshot, runtime_shell, mode);
    }
    if runtime_shell.battle_move_cursor.is_some() {
        return Ok(visible_battle_move_entries(snapshot, runtime_shell, battle));
    }
    if runtime_shell.battle_switch_cursor.is_some() {
        return Ok(visible_battle_switch_entries(
            snapshot,
            runtime_shell,
            battle,
        ));
    }
    if runtime_shell.bag_cursor.is_some() {
        return Ok(visible_battle_item_entries(snapshot, runtime_shell));
    }
    if runtime_shell.ball_cursor.is_some() {
        return Ok(visible_battle_ball_entries(snapshot, runtime_shell));
    }
    if runtime_shell.key_item_cursor.is_some() || runtime_shell.tmhm_cursor.is_some() {
        return visible_field_pack_entries(snapshot, runtime_shell);
    }
    if battle.enemy_pokemon.hp == 0 && !battle.enemy_spikes_zero_hp_unchecked {
        // KO settlement is driven by the retained faint/reward presentation.
        // Crystal never exposes the shell's claim/advance operations as a
        // selectable battle menu between those boundaries.
        return Ok(Vec::new());
    }
    let actions = visible_battle_action_ids(snapshot, battle);
    if actions.is_empty() {
        return Ok(Vec::new());
    }
    let selected = strict_readonly_cursor_index(
        &runtime_shell.battle_action_cursor,
        "battle:actions",
        actions.len(),
    );
    let selected = selected.context("battle main-action cursor is invalid")?;
    let selected_action = actions[selected];
    Ok(battle_main_menu_entries(
        snapshot,
        battle,
        battle_main_menu_index_for_action(selected_action),
    ))
}

fn visible_battle_faint_prompt_entries(runtime_shell: &BevyRuntimeShell) -> Result<Vec<String>> {
    let selected = strict_readonly_cursor_index(
        &runtime_shell.battle_faint_prompt_cursor,
        "battle:faint-prompt",
        2,
    )
    .context("battle faint prompt cursor is invalid")?;
    Ok(vec![
        "Use next <PKMN>?".to_string(),
        format!("{}YES", if selected == 0 { ">" } else { " " }),
        format!("{}NO", if selected == 1 { ">" } else { " " }),
    ])
}

fn battle_main_menu_entries(
    snapshot: &RuntimeShellSnapshot,
    battle: &crate::RuntimeBattleSnapshot,
    selected: usize,
) -> Vec<String> {
    battle_main_menu_entries_for_type(
        &battle.battle_type,
        snapshot.bug_contest.park_balls_remaining,
        selected,
    )
}

fn battle_main_menu_entries_for_type(
    battle_type: &str,
    park_balls_remaining: u8,
    selected: usize,
) -> Vec<String> {
    let labels = if battle_type == "BATTLETYPE_CONTEST" {
        vec![
            "FIGHT".to_string(),
            "<PKMN>".to_string(),
            format!("PARKBALL×{:>2}", park_balls_remaining),
            "RUN".to_string(),
        ]
    } else {
        BATTLE_MAIN_MENU_LABELS
            .iter()
            .map(|label| (*label).to_string())
            .collect()
    };
    labels
        .iter()
        .enumerate()
        .map(|(index, label)| format!("{}{label}", if selected == index { ">" } else { " " }))
        .collect()
}

fn battle_main_menu_index_for_action(action: VisibleBattleAction) -> usize {
    match action {
        VisibleBattleAction::Fight => 0,
        VisibleBattleAction::Pokemon => 1,
        VisibleBattleAction::Pack => 2,
        VisibleBattleAction::Run => 3,
    }
}

fn visible_battle_switch_entries(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
    _battle: &crate::RuntimeBattleSnapshot,
) -> Vec<String> {
    let selected = strict_readonly_cursor_index(
        &runtime_shell.battle_switch_cursor,
        "battle:switch",
        battle_switch_option_count(snapshot),
    );
    let Some(selected) = selected else {
        return vec![compact_scene_label("INVALID CURSOR battle:switch", 30)];
    };
    let mut entries = Vec::new();
    entries.extend(
        windowed_index_range(selected, battle_switch_option_count(snapshot)).map(|index| {
            let Some(slot) = snapshot.party.slots.get(index) else {
                return party_cancel_entry(index == selected);
            };
            party_slot_entry(snapshot, slot, index == selected)
        }),
    );
    entries.into_iter().take(SCENE_MENU_VISIBLE_ROWS).collect()
}

fn visible_battle_shift_prompt_entries(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
    battle: &crate::RuntimeBattleSnapshot,
) -> Result<Vec<String>> {
    next_unresolved_trainer_enemy_label(battle)
        .context("trainer shift prompt has no unresolved enemy")?;
    let selected = strict_readonly_cursor_index(
        &runtime_shell.battle_shift_prompt_cursor,
        "battle:shift-prompt",
        2,
    )
    .context("battle trainer-shift cursor is invalid")?;
    Ok(vec![
        compact_scene_label(&format!("Will {}", snapshot.trainer.player_name), 18),
        "change <PKMN>?".to_string(),
        format!("{}YES", if selected == 0 { ">" } else { " " }),
        format!("{}NO", if selected == 1 { ">" } else { " " }),
    ])
}

fn next_unresolved_trainer_enemy_label(battle: &crate::RuntimeBattleSnapshot) -> Option<&str> {
    let enemy_index = battle.active_enemy_party_index?;
    battle
        .enemy_party
        .iter()
        .enumerate()
        .find_map(|(index, pokemon)| {
            (index != enemy_index && pokemon.hp > 0).then_some(pokemon.nickname.as_str())
        })
}

fn visible_battle_item_entries(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
) -> Vec<String> {
    let item_ids = carried_battle_non_ball_item_ids(snapshot);
    let selected = strict_readonly_cursor_index(
        &runtime_shell.bag_cursor,
        "battle:bag-items",
        field_pack_selectable_count(item_ids.len()),
    );
    let Some(selected) = selected else {
        return vec![compact_scene_label("INVALID CURSOR battle:bag-items", 30)];
    };
    let mut entries = Vec::new();
    entries.extend(
        windowed_index_range(selected, field_pack_selectable_count(item_ids.len())).map(|index| {
            if index >= item_ids.len() {
                return pack_cancel_entry(if index == selected { ">" } else { " " });
            }
            let item_id = &item_ids[index];
            let marker = if index == selected { ">" } else { " " };
            let Some(quantity) = carried_item_quantity(snapshot, item_id) else {
                return compact_scene_label(&format!("{marker}{item_id} INVALID INVENTORY"), 30);
            };
            compact_scene_label(
                &format!(
                    "{marker}{} x{quantity}",
                    item_display_name(snapshot, item_id)
                ),
                30,
            )
        }),
    );
    entries.into_iter().take(SCENE_MENU_VISIBLE_ROWS).collect()
}

fn visible_battle_ball_entries(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
) -> Vec<String> {
    let ball_ids = carried_ball_item_ids(snapshot);
    let selected = strict_readonly_cursor_index(
        &runtime_shell.ball_cursor,
        "bag:balls",
        field_pack_selectable_count(ball_ids.len()),
    );
    let Some(selected) = selected else {
        return vec![compact_scene_label("INVALID CURSOR bag:balls", 30)];
    };
    let mut entries = Vec::new();
    entries.extend(
        windowed_index_range(selected, field_pack_selectable_count(ball_ids.len())).map(|index| {
            if index >= ball_ids.len() {
                return pack_cancel_entry(if index == selected { ">" } else { " " });
            }
            let item_id = &ball_ids[index];
            let marker = if index == selected { ">" } else { " " };
            let Some(quantity) = carried_item_quantity(snapshot, item_id) else {
                return compact_scene_label(&format!("{marker}{item_id} INVALID INVENTORY"), 30);
            };
            compact_scene_label(
                &format!(
                    "{marker}{} x{quantity}",
                    item_display_name(snapshot, item_id)
                ),
                30,
            )
        }),
    );
    entries.into_iter().take(SCENE_MENU_VISIBLE_ROWS).collect()
}

fn visible_battle_pack_target_entries(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
    mode: BattlePackTargetMode,
) -> Result<Vec<String>> {
    let selected_party = runtime_shell.party_cursor;
    anyhow::ensure!(selected_party < snapshot.party.slots.len(), "battle item target party cursor is invalid");
    selected_battle_pack_item_label(snapshot, runtime_shell)
        .context("battle item target requires a selected item")?;
    let mut entries = Vec::new();
    if mode == BattlePackTargetMode::PartyMove {
        let Some(slot) = snapshot.party.slots.get(selected_party) else {
            anyhow::bail!("battle item target party slot {selected_party} is absent");
        };
        entries.push(party_slot_entry(snapshot, slot, true));
        let selected_move = strict_readonly_cursor_index(
            &runtime_shell.party_move_cursor,
            &party_move_cursor_surface_id(slot.index),
            slot.pokemon.moves.len(),
        );
        let selected_move = selected_move.context("battle item target move cursor is invalid")?;
        entries.extend(windowed_move_entries(
            snapshot,
            &slot.pokemon.moves,
            selected_move,
        )?);
        return Ok(entries.into_iter().take(SCENE_MENU_VISIBLE_ROWS).collect());
    }
    entries.extend(windowed_party_slot_entries(snapshot, selected_party));
    Ok(entries.into_iter().take(SCENE_MENU_VISIBLE_ROWS).collect())
}

fn visible_battle_move_entries(
    snapshot: &RuntimeShellSnapshot,
    runtime_shell: &BevyRuntimeShell,
    battle: &crate::RuntimeBattleSnapshot,
) -> Vec<String> {
    let Some(active_index) = battle.active_player_party_index else {
        return vec![compact_scene_label("INVALID ACTIVE PARTY SLOT", 30)];
    };
    if !snapshot
        .party
        .slots
        .iter()
        .any(|slot| slot.index == active_index)
    {
        return vec![compact_scene_label(
            &format!("INVALID PARTY SLOT {active_index}"),
            30,
        )];
    }
    let total = battle.player_moves.len() + 1;
    let selected =
        strict_readonly_cursor_index(&runtime_shell.battle_move_cursor, "battle:moves", total);
    let Some(selected) = selected else {
        return vec![compact_scene_label("INVALID CURSOR battle:moves", 30)];
    };
    let mut entries = Vec::new();
    entries.extend(windowed_index_range(selected, total).map(|index| {
        let marker = if runtime_shell.battle_move_swap_origin == Some(index) {
            "▷"
        } else if index == selected {
            ">"
        } else {
            " "
        };
        if index >= battle.player_moves.len() {
            format!("{marker}CANCEL")
        } else {
            let learned = &battle.player_moves[index];
            compact_scene_label(
                &format!(
                    "{marker}{}",
                    battle_move_display_name(snapshot, &learned.name)
                ),
                12,
            )
        }
    }));
    entries.into_iter().take(SCENE_MENU_VISIBLE_ROWS).collect()
}

impl TilesetArt {
    #[cfg(test)]
    fn tile_handle(&self, metatile_id: u16, sub_x: usize, sub_y: usize) -> Option<Handle<Image>> {
        self.tile_handle_at_frame(metatile_id, sub_x, sub_y, 0, false)
    }

    fn tile_handle_at_frame(
        &self,
        metatile_id: u16,
        sub_x: usize,
        sub_y: usize,
        frame: u64,
        forest_restless: bool,
    ) -> Option<Handle<Image>> {
        let offset = usize::from(metatile_id)
            .checked_mul(METATILE_TILE_COUNT)?
            .checked_add(sub_y.checked_mul(RENDER_METATILE_WIDTH as usize)?)?
            .checked_add(sub_x)?;
        let tile_index = *self.metatile_layout.get(offset)? as usize;
        if let Some(animation) = self.animated_tiles.get(&tile_index)
            && !animation.frames.is_empty()
        {
            let frame_index = if animation.requires_forest_restless && !forest_restless {
                0
            } else if animation.cave_water_composite {
                let water_frame = (frame / 22) as usize % 4;
                let scroll_frame = if frame < 4 {
                    0
                } else {
                    (((frame - 4) / 19) + 1) as usize % 8
                };
                water_frame * 8 + scroll_frame
            } else if animation.advance_on_phase_offset {
                if frame < animation.phase_offset {
                    0
                } else {
                    (((frame - animation.phase_offset) / animation.frame_ticks.max(1)) + 1) as usize
                        % animation.frames.len()
                }
            } else {
                (frame.saturating_sub(animation.phase_offset) / animation.frame_ticks.max(1))
                    as usize
                    % animation.frames.len()
            };
            return animation.frames.get(frame_index).cloned();
        }
        self.tile_handles.get(tile_index).cloned()
    }

    fn priority_tile_handle(
        &self,
        metatile_id: u16,
        sub_x: usize,
        sub_y: usize,
    ) -> Option<Handle<Image>> {
        let offset = usize::from(metatile_id)
            .checked_mul(METATILE_TILE_COUNT)?
            .checked_add(sub_y.checked_mul(RENDER_METATILE_WIDTH as usize)?)?
            .checked_add(sub_x)?;
        let tile_index = *self.metatile_layout.get(offset)? as usize;
        self.priority_tile_handles.get(tile_index).cloned()
    }
}

fn load_bitmap_font_art(
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<BitmapFontArt> {
    let font_path = asset_root.runtime_assets().join("gfx/font/font.png");
    let space_path = asset_root.runtime_assets().join("gfx/font/space.png");
    let source = crate::open_runtime_image(&font_path)
        .with_context(|| format!("load bitmap font {}", font_path.display()))?
        .to_rgba8();
    let space = crate::open_runtime_image(&space_path)
        .with_context(|| format!("load bitmap font space {}", space_path.display()))?
        .to_rgba8();
    let source_width = source.width() as usize;
    let source_height = source.height() as usize;
    if source_width < BITMAP_FONT_TILE_SIZE || source_height < BITMAP_FONT_TILE_SIZE {
        anyhow::bail!(
            "bitmap font {} has invalid dimensions {}x{}",
            font_path.display(),
            source_width,
            source_height
        );
    }
    let tiles_per_row = source_width / BITMAP_FONT_TILE_SIZE;
    let mut glyphs = HashMap::new();
    for (ch, tile_id) in bitmap_font_char_map() {
        let handle = if ch == ' ' {
            bitmap_font_tile_handle(&space, 0, 1, images)?
        } else {
            // TypeScript keeps both ROM-indexed font tiles (0x80+) and the
            // low-ID tiles copied from the font sheet.  The latter include
            // Poké and several punctuation/control glyphs.
            let tile_index = if tile_id >= 0x80 {
                usize::from(tile_id - 0x80)
            } else {
                usize::from(tile_id)
            };
            bitmap_font_tile_handle(&source, tile_index, tiles_per_row, images)?
        };
        glyphs.insert(
            ch,
            SpriteFrame {
                handle,
                size: Vec2::splat(BITMAP_FONT_GLYPH_SIZE),
            },
        );
    }
    load_bitmap_font_extra_glyphs(
        &asset_root.runtime_assets().join("gfx/font"),
        &mut glyphs,
        images,
    )?;
    load_bitmap_font_frame_glyphs(
        &asset_root.runtime_assets().join("gfx/frames/1.png"),
        &mut glyphs,
        images,
    )?;
    Ok(BitmapFontArt { glyphs })
}

/// `BitmapFont.set_frame_tiles(1)` overwrites these six tile IDs after the
/// font extras load.  Box-drawing characters therefore come from the active
/// textbox frame, never from coincidentally numbered font-sheet tiles.
fn load_bitmap_font_frame_glyphs(
    frame_path: &std::path::Path,
    glyphs: &mut HashMap<char, SpriteFrame>,
    images: &mut Assets<Image>,
) -> Result<()> {
    let frame = crate::open_runtime_image(frame_path)
        .with_context(|| format!("load bitmap font frame {}", frame_path.display()))?
        .to_rgba8();
    if frame.width() != 24 || frame.height() != 16 {
        anyhow::bail!(
            "bitmap font frame {} must be 24x16, got {}x{}",
            frame_path.display(),
            frame.width(),
            frame.height()
        );
    }
    for (source_tile, tile_id) in [0x79_u16, 0x7a, 0x7b, 0x7c, 0x7d, 0x7e]
        .into_iter()
        .enumerate()
    {
        let handle = bitmap_font_tile_handle(&frame, source_tile, 3, images)?;
        let sprite = SpriteFrame {
            handle,
            size: Vec2::splat(BITMAP_FONT_GLYPH_SIZE),
        };
        for (ch, mapped_tile) in bitmap_font_char_map() {
            if mapped_tile == tile_id {
                glyphs.insert(ch, sprite.clone());
            }
        }
    }
    Ok(())
}

/// Load the low-ID tiles installed by `LoadFontsExtra` and
/// `LoadFontsBattleExtra`.  They are not present in `font.png`, so using its
/// tile at the same numeric index gives the wrong glyph for control tokens.
fn load_bitmap_font_extra_glyphs(
    font_root: &std::path::Path,
    glyphs: &mut HashMap<char, SpriteFrame>,
    images: &mut Assets<Image>,
) -> Result<()> {
    let install = |data: &[u8],
                   source_tile: usize,
                   tile_id: u16,
                   glyphs: &mut HashMap<char, SpriteFrame>,
                   images: &mut Assets<Image>|
     -> Result<()> {
        let handle = bitmap_font_2bpp_tile_handle(data, source_tile, images)?;
        let frame = SpriteFrame {
            handle,
            size: Vec2::splat(BITMAP_FONT_GLYPH_SIZE),
        };
        for (ch, mapped_tile) in bitmap_font_char_map() {
            if mapped_tile == tile_id {
                glyphs.insert(ch, frame.clone());
            }
        }
        Ok(())
    };

    let battle_extra_path = font_root.join("font_battle_extra.2bpp");
    let battle_extra = crate::read_runtime_asset(&battle_extra_path).with_context(|| {
        format!(
            "read bitmap battle font extras {}",
            battle_extra_path.display()
        )
    })?;
    for source_tile in 0..battle_extra.len() / 16 {
        install(
            &battle_extra,
            source_tile,
            0x60 + source_tile as u16,
            glyphs,
            images,
        )?;
    }
    let up_arrow_path = font_root.join("up_arrow.2bpp");
    let up_arrow = crate::read_runtime_asset(&up_arrow_path)
        .with_context(|| format!("read bitmap up-arrow glyph {}", up_arrow_path.display()))?;
    install(&up_arrow, 0, 0x61, glyphs, images)?;
    let phone_icon_path = font_root.join("phone_icon.2bpp");
    let phone_icon = crate::read_runtime_asset(&phone_icon_path)
        .with_context(|| format!("read bitmap phone glyph {}", phone_icon_path.display()))?;
    install(&phone_icon, 0, 0x62, glyphs, images)?;
    let font_extra_path = font_root.join("font_extra.2bpp");
    let font_extra = crate::read_runtime_asset(&font_extra_path)
        .with_context(|| format!("read bitmap font extras {}", font_extra_path.display()))?;
    // LoadFontsExtra copies source tiles 3..24 into VRAM tiles 0x63..0x78.
    for offset in 0..22 {
        install(
            &font_extra,
            offset + 3,
            0x63 + offset as u16,
            glyphs,
            images,
        )?;
    }
    // LoadFontsExtra installs the extras first, then LoadFontsBattleExtra
    // supplies the battle-specific Lv tile at 0x6e.
    install(&battle_extra, 0x6e - 0x60, 0x6e, glyphs, images)?;
    Ok(())
}
