// Browser tools feed the same joypad resource as physical keyboard input.
// The page never receives a mutable game session or a save-editing interface.
#[derive(serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum WebMcpCommand {
    Observe {},
    Multiplayer { interaction: String },
    Press { button: String, frames: u32 },
}

struct WebMcpPending {
    id: u32,
    command: WebMcpCommand,
    key: Option<KeyCode>,
    started: Option<u64>,
    released: Option<u64>,
    canceled: bool,
}

#[derive(Default)]
struct WebMcpBridge {
    sequence: u32,
    pending: Option<WebMcpPending>,
    result: Option<(u32, String)>,
}

thread_local! {
    static WEBMCP_BRIDGE: std::cell::RefCell<WebMcpBridge> = std::cell::RefCell::new(WebMcpBridge::default());
}

fn webmcp_key(button: &str) -> Option<KeyCode> {
    Some(match button {
        "up" => KeyCode::ArrowUp,
        "down" => KeyCode::ArrowDown,
        "left" => KeyCode::ArrowLeft,
        "right" => KeyCode::ArrowRight,
        "a" => KeyCode::KeyZ,
        "b" => KeyCode::KeyX,
        "start" => KeyCode::Enter,
        "select" => KeyCode::ShiftRight,
        _ => return None,
    })
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn crystal_webmcp_request(json: &str) -> std::result::Result<u32, String> {
    if json.len() > 256 {
        return Err("Tool input is too large".into());
    }
    let command: WebMcpCommand = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let key = match &command {
        WebMcpCommand::Observe {} => None,
        WebMcpCommand::Multiplayer { interaction } => Some(match interaction.as_str() {
            "battle" => KeyCode::KeyC,
            "trade" => KeyCode::KeyV,
            "time_capsule" => KeyCode::KeyT,
            _ => return Err("Unknown multiplayer interaction".into()),
        }),
        WebMcpCommand::Press { button, frames } => {
            if !(1..=60).contains(frames) {
                return Err("frames must be between 1 and 60".into());
            }
            Some(webmcp_key(button).ok_or("Unknown Game Boy button")?)
        }
    };
    WEBMCP_BRIDGE.with_borrow_mut(|bridge| {
        if bridge.pending.is_some() {
            return Err("Another game tool is executing".into());
        }
        bridge.sequence = bridge.sequence.wrapping_add(1).max(1);
        let id = bridge.sequence;
        bridge.result = None;
        bridge.pending = Some(WebMcpPending {
            id,
            command,
            key,
            started: None,
            released: None,
            canceled: false,
        });
        Ok(id)
    })
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn crystal_webmcp_poll(id: u32) -> Option<String> {
    WEBMCP_BRIDGE.with_borrow(|bridge| {
        bridge
            .result
            .as_ref()
            .filter(|(result_id, _)| *result_id == id)
            .map(|(_, result)| result.clone())
    })
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn crystal_webmcp_cancel(id: u32) {
    WEBMCP_BRIDGE.with_borrow_mut(|bridge| {
        if let Some(pending) = bridge.pending.as_mut().filter(|pending| pending.id == id) {
            pending.canceled = true;
        }
    });
}

fn apply_webmcp_input(
    runtime: Res<BevyRuntimeShell>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    _main_thread: Option<NonSend<MultiplayerRuntime>>,
) {
    WEBMCP_BRIDGE.with_borrow_mut(|bridge| {
        let Some(pending) = bridge.pending.as_mut() else {
            return;
        };
        let Some(key) = pending.key else {
            return;
        };
        let frame = runtime.lcd_animation_frame;
        let frames = match pending.command {
            WebMcpCommand::Press { frames, .. } => frames,
            WebMcpCommand::Multiplayer { .. } => 1,
            WebMcpCommand::Observe {} => return,
        };
        if pending.canceled
            || pending
                .started
                .is_some_and(|start| frame.saturating_sub(start) >= u64::from(frames))
        {
            if pending.released.is_none() {
                if pending.started.is_some() {
                    keys.release(key);
                }
                pending.released = Some(frame);
            }
        } else if pending.started.is_none() {
            // Refuse to take ownership of a key the human is already holding.
            if keys.get_pressed().next().is_some() {
                pending.canceled = true;
                return;
            }
            pending.started = Some(frame);
            keys.press(key);
        }
    });
}

fn webmcp_observation(
    runtime: &BevyRuntimeShell,
    multiplayer: Option<&MultiplayerRuntime>,
) -> Result<serde_json::Value> {
    let snapshot = runtime.shell.snapshot()?;
    #[cfg(feature = "fullscreen-scaling")]
    let snapshot = {
        let mut snapshot = snapshot;
        expand_fullscreen_object_presentation(&mut snapshot, runtime)?;
        snapshot
    };
    let (screen, text) = if runtime.intro_screen.is_some() {
        ("intro", format_intro_dialog(runtime))
    } else if runtime.title_menu.is_some() {
        ("title", format_title_dialog(runtime))
    } else if runtime.pending_time_set.is_some() {
        ("clock", format_time_set_dialog_overlay(runtime))
    } else if runtime.pending_oak_intro.is_some() {
        ("introduction", format_oak_intro_dialog_overlay(runtime))
    } else if runtime.pending_gender_selection.is_some() {
        ("gender", format_gender_dialog(runtime))
    } else if runtime.credits_screen.is_some() {
        ("credits", format_credits_dialog(runtime))
    } else {
        (
            if snapshot.battle.is_some() {
                "battle"
            } else if runtime.pending_name_choice.is_some() || runtime.pending_name_input.is_some()
            {
                "naming"
            } else {
                "overworld"
            },
            format_dialog_overlay(&snapshot, runtime),
        )
    };
    let module = runtime
        .runtime
        .data()
        .map_module(&snapshot.overworld.map_name)?;
    let objects: Vec<_> = snapshot
        .visible_object_runtime_tiles
        .iter()
        .map(|(name, tile)| {
            let role = module
                .objects
                .iter()
                .find(|o| {
                    o.object_identifier.as_deref() == Some(name.as_str())
                        || o.label.as_deref() == Some(name.as_str())
                })
                .map(|o| o.script.as_str());
            serde_json::json!({"name": name, "x": tile.x, "y": tile.y, "curriculum_role": role})
        })
        .collect();
    let players: Vec<_> = multiplayer.map(|multiplayer| multiplayer.remote_presences.values().filter(|presence| presence.map == snapshot.overworld.map_name).map(|presence| serde_json::json!({"name": presence.display_name, "x": presence.tile_x, "y": presence.tile_y, "facing": presence.direction})).collect()).unwrap_or_default();
    let mut menus = Vec::new();
    if let Some(choice) = &runtime.pending_name_choice {
        menus.push(serde_json::json!({"kind": "name_choices", "options": choice.options, "selected": choice.selected}));
    }
    if let Some(input) = &runtime.pending_name_input {
        menus.push(serde_json::json!({"kind": "name_input", "label": input.label, "value": input.value, "max_length": input.max_length, "keyboard": visible_name_input_layout(input.case), "cursor_column": input.cursor_column, "cursor_row": input.cursor_row}));
    }
    if runtime.start_menu_cursor.is_some() {
        menus.push(
            serde_json::json!({"kind": "start", "entries": visible_start_menu_entries(runtime)?}),
        );
    }
    if runtime.party_menu_open {
        menus.push(serde_json::json!({"kind": "party", "entries": visible_party_menu_entries(&snapshot, runtime)?}));
    }
    if runtime.field_pack_pocket.is_some() {
        menus.push(serde_json::json!({"kind": "pack", "entries": visible_field_pack_entries(&snapshot, runtime)?}));
    }
    if runtime.pokedex_menu_open {
        menus.push(serde_json::json!({"kind": "pokedex", "entries": visible_pokedex_menu_entries(&snapshot, runtime)?}));
    }
    if runtime.pokegear_menu_open {
        menus.push(serde_json::json!({"kind": "pokegear", "entries": visible_pokegear_menu_entries(&snapshot, runtime)?}));
    }
    if let Some(battle) = &snapshot.battle
        && runtime.battle_messages.is_empty()
        && !visible_battle_command_animation_active(runtime)
    {
        menus.push(serde_json::json!({"kind": "battle", "entries": visible_battle_command_menu_entries(&snapshot, runtime, battle)?}));
    }
    if let Some(menu) = menus.last_mut().filter(|m| m["kind"] == "pack") {
        let surface = if runtime.field_pack_target_mode.is_some() {
            "item_target"
        } else if runtime.field_pack_action_cursor.is_some() {
            "item_actions"
        } else {
            match runtime.field_pack_pocket.as_ref() {
                Some(FieldPackPocket::Items) => "items",
                Some(FieldPackPocket::Balls) => "balls",
                Some(FieldPackPocket::KeyItems) => "key_items",
                Some(FieldPackPocket::TmHm) => "machines",
                _ => "custom",
            }
        };
        menu["surface"] = serde_json::json!(surface);
    }
    if runtime.pc_hub_session_open || runtime.bill_pc_session_open {
        let surface = if runtime.pending_pc_release.is_some() {
            "release_confirmation"
        } else if runtime.bill_pc_pokemon_action_cursor.is_some() {
            "pokemon_actions"
        } else if runtime.bill_pc_box_action_cursor.is_some() {
            "box_actions"
        } else if runtime.bill_pc_box_cursor.is_some() {
            "boxes"
        } else if runtime.storage_cursor.is_some() {
            if runtime.bill_pc_deposit_open {
                "deposit"
            } else {
                "withdraw"
            }
        } else if runtime.bill_pc_session_open {
            "storage_actions"
        } else {
            "hub"
        };
        menus.push(serde_json::json!({"kind":"pc","surface":surface,
            "entries":visible_scene_dialog_entries(&snapshot,runtime)?,
            "box_index":visible_pc_box_index(&snapshot,runtime),
            "selected_box":runtime.bill_pc_box_cursor.as_ref().map(|c|c.option_index),
            "selected_slot":runtime.storage_cursor.as_ref().map(|c|c.option_index),
            "box_counts":runtime.shell.session().state().storage.pc_boxes.iter().enumerate().map(|(i,b)|serde_json::json!({"box":i,"count":b.count,"capacity":b.pokemon.len()})).collect::<Vec<_>>() }));
    }
    if let Some(menu) = snapshot.ui.menu.as_ref().filter(|_| menus.is_empty()) {
        if visible_field_dialogue_is_entirely_consumed(runtime, &snapshot) {
            let mut rows = Vec::new();
            push_visible_runtime_menu_dialog_entries(&mut rows, runtime, menu)?;
            if !rows.is_empty() {
                menus.push(serde_json::json!({"kind":"script","entries":rows}));
            }
        }
    }
    if runtime.pending_day_of_week.is_some() {
        let mut rows = Vec::new();
        push_visible_day_of_week_entries(&mut rows, runtime);
        menus.push(serde_json::json!({"kind":"weekday","entries":rows}));
    }
    if runtime.pending_phone_prompt.is_some() {
        let mut rows = Vec::new();
        push_visible_phone_prompt_entries(&mut rows, &snapshot, runtime)?;
        menus.push(serde_json::json!({"kind":"phone_choice","entries":rows}));
    }
    // Declared task aids for Flygon's engineered curriculum, sourced from the
    // loaded pack. These are not anatomical sensory measurements or inputs.
    let mut exits: Vec<_> = module
        .events
        .warps
        .iter()
        .map(|w| serde_json::json!({"kind":"warp","target":w.target_map,"x":w.x,"y":w.y}))
        .collect();
    exits.extend(module.attributes.connections.iter().map(
        |c| serde_json::json!({"kind":"connection","target":c.target_map,"direction":c.direction}),
    ));
    let events: Vec<_> = module
        .events
        .coord_events
        .iter()
        .map(|e| serde_json::json!({"x":e.x,"y":e.y,"script":e.script_name}))
        .collect();
    if let Some(menu) = menus.last_mut().filter(|m| m["kind"] == "battle") {
        let surface = if runtime.battle_faint_prompt_cursor.is_some() {
            "faint_prompt"
        } else if runtime.battle_shift_prompt_cursor.is_some() {
            "shift_prompt"
        } else if runtime.battle_pack_target_mode.is_some() {
            "item_target"
        } else if runtime.battle_move_cursor.is_some() {
            "moves"
        } else if runtime.battle_party_action_cursor.is_some() {
            "party_actions"
        } else if runtime.battle_switch_cursor.is_some() {
            "party"
        } else if runtime.bag_cursor.is_some() {
            "items"
        } else if runtime.ball_cursor.is_some() {
            "balls"
        } else if runtime.key_item_cursor.is_some() {
            "key_items"
        } else if runtime.tmhm_cursor.is_some() {
            "machines"
        } else {
            "commands"
        };
        menu["surface"] = serde_json::json!(surface);
    }
    if snapshot.ui.pending_yes_no.is_some()
        && visible_field_dialogue_is_entirely_consumed(runtime, &snapshot)
    {
        let selected = strict_readonly_cursor_index(&runtime.yes_no_cursor, "ui:yes-no", 2)
            .context("visible yes/no cursor missing")?;
        menus.push(serde_json::json!({"kind":"yes_no","selected":selected,"entries":if selected==0 {vec![">YES","NO"]}else{vec!["YES",">NO"]}}));
    }
    if let Some(shop) = &snapshot.pending_shop {
        let mut rows = Vec::new();
        push_visible_shop_dialog_entries(&mut rows, &snapshot, runtime, shop)?;
        let mut menu = serde_json::json!({"kind":"shop","surface":if runtime.shop_top_cursor.is_some(){"top"}else if runtime.sell_cursor.is_some(){"sell"}else{"buy"},"entries":rows});
        if let Some(q) = &runtime.shop_quantity {
            menu = serde_json::json!({"kind":"shop","surface":if q.confirmation.is_some(){"confirm"}else{"quantity"},"selling":q.selling,"item":q.item_id,"quantity":q.quantity,"unit_price":q.unit_price,"total_price":visible_shop_quantity_total(q),"entries":if q.confirmation==Some(false){vec!["YES",">NO"]}else{vec![">YES","NO"]}});
        }
        menus.push(menu);
    }
    let mut hm_compatibility = serde_json::Map::new();
    let mut enemy_hm_compatibility = serde_json::Map::new();
    let mut boxed_hm_candidates = serde_json::Map::new();
    for machine in snapshot.bag.tm_hm.iter().filter(|i| i.quantity > 0) {
        let item = runtime.runtime.data().item(&machine.item_id)?;
        let Some(move_name) = item.tmhm_move.as_deref() else {
            continue;
        };
        let able = |p: &crate::core::models::pokemon::Pokemon| -> Option<bool> {
            runtime
                .runtime
                .data()
                .saved_species(&p.species.id)
                .map(|s| !p.is_egg && s.tmhm_learnset.iter().any(|m| m == move_name))
        };
        let slots: Vec<_> = snapshot
            .party
            .slots
            .iter()
            .filter_map(|slot| {
                able(&slot.pokemon).map(|a| serde_json::json!({"slot":slot.index,"able":a}))
            })
            .collect();
        hm_compatibility.insert(machine.item_id.clone(), serde_json::json!(slots));
        if let Some(b) = &snapshot.battle {
            if let Some(a) = able(&b.enemy_pokemon) {
                enemy_hm_compatibility.insert(machine.item_id.clone(), serde_json::json!(a));
            }
        }
        for (box_index, pc_box) in runtime
            .shell
            .session()
            .state()
            .storage
            .pc_boxes
            .iter()
            .enumerate()
        {
            if let Some((position, _)) = pc_box
                .pokemon
                .iter()
                .enumerate()
                .find(|(_, p)| p.as_ref().is_some_and(|p| able(p) == Some(true)))
            {
                boxed_hm_candidates.insert(
                    machine.item_id.clone(),
                    serde_json::json!({"box":box_index,"position":position}),
                );
                break;
            }
        }
    }
    let mut owned_species_counts = std::collections::BTreeMap::<String, usize>::new();
    for slot in &snapshot.party.slots {
        if !slot.pokemon.is_egg {
            *owned_species_counts
                .entry(slot.pokemon.species.id.clone())
                .or_default() += 1;
        }
    }
    for b in &runtime.shell.session().state().storage.pc_boxes {
        for p in b.pokemon.iter().flatten().filter(|p| !p.is_egg) {
            *owned_species_counts
                .entry(p.species.id.clone())
                .or_default() += 1;
        }
    }
    let mut terrain = Vec::new();
    if screen == "overworld" {
        let tileset_name = runtime
            .runtime
            .data()
            .map_tileset_name(&snapshot.overworld.map_name)?;
        let collision = runtime.runtime.data().tileset_collision(&tileset_name)?;
        let map = &runtime.shell.session().overworld().map;
        for y in snapshot.overworld.tile.y.saturating_sub(6)
            ..=snapshot.overworld.tile.y.saturating_add(6)
        {
            let mut row = Vec::new();
            for x in snapshot.overworld.tile.x.saturating_sub(6)
                ..=snapshot.overworld.tile.x.saturating_add(6)
            {
                row.push(crate::core::world::collision::sample_collision(map, &collision, TilePosition::new(x, y)).map(|sample| {
                    let attributes = crate::core::world::collision::describe_collision(sample.permission);
                    serde_json::json!({"terrain": format!("{:?}", attributes.terrain), "permission": sample.permission})
                }));
            }
            terrain.push(row);
        }
    }
    Ok(serde_json::json!({
        "frame": runtime.lcd_animation_frame,
        "status": {"screen": screen, "player_name": snapshot.trainer.player_name, "money": snapshot.trainer.money, "badges": snapshot.progression.badges, "party": snapshot.party.slots.iter().map(|slot| { let p = &slot.pokemon; serde_json::json!({"slot": slot.index, "nickname": p.nickname, "level": p.level, "hp": p.hp, "max_hp": p.max_hp, "status": p.status, "item": p.item, "moves": p.moves, "is_egg": p.is_egg}) }).collect::<Vec<_>>()},
        "reward_state": {
            "version": 1,
            "field_actions": runtime.flygon_field_actions,
            "scenes": runtime.shell.session().state().scenes.map_scenes,
            "battle_result": runtime.shell.session().state().battle_result,
            "movement_mode": format!("{:?}", snapshot.overworld.mode),
            "event_flags": snapshot.progression.active_event_flags,
            "engine_flags": snapshot.progression.active_engine_flags,
            "caught_species": snapshot.progression.pokedex_caught_species,
            "hall_of_fame": snapshot.progression.hall_of_fame.count,
            "last_talked_object": snapshot.script_events.last_talked_object,
            "key_items": snapshot.bag.key_items.iter().filter(|i| i.quantity > 0).map(|i| &i.item_id).collect::<Vec<_>>(),
            "items": snapshot.bag.items.iter().chain(snapshot.bag.balls.iter()).map(|i|serde_json::json!({"id":i.item_id,"quantity":i.quantity})).collect::<Vec<_>>(),
            "enemy_hm_compatibility":enemy_hm_compatibility,
            "boxed_hm_candidates":boxed_hm_candidates,
            "owned_species_counts":owned_species_counts,
            "machines": snapshot.bag.tm_hm.iter().filter(|i| i.quantity > 0).map(|i| &i.item_id).collect::<Vec<_>>(),
            "battle": snapshot.battle.as_ref().map(|b| serde_json::json!({
                "enemy_party": b.enemy_party.iter().map(|p| serde_json::json!({"hp":p.hp,"max_hp":p.max_hp})).collect::<Vec<_>>(),
                "active_enemy": b.active_enemy_party_index,
                "enemy_hp": b.enemy_pokemon.hp,
                "enemy_max_hp": b.enemy_pokemon.max_hp,
                "active_player": b.active_player_party_index,
                "rewarded_enemies": b.rewarded_enemy_party_indices,
                "player_turns": b.player_turns_taken,
                "enemy_turns": b.enemy_turns_taken
            }))
        },
        "observe": {"text": text, "visible_dialogue": visible_field_dialog_text(&snapshot, runtime), "menus": menus, "pokemon_switch_open": runtime.party_switch_cursor.is_some() || runtime.battle_switch_cursor.is_some(), "battle_message": runtime.battle_messages.front(), "battle": format_battle_overlay(&snapshot, runtime)},
        "map_info": {"name": snapshot.overworld.map_name, "player": {"x": snapshot.overworld.tile.x, "y": snapshot.overworld.tile.y, "facing": format!("{:?}", snapshot.overworld.facing)}, "dimensions": runtime.runtime.data().saved_map_tile_bounds(&snapshot.overworld.map_name), "objects": objects, "players": players, "curriculum_exits":exits, "curriculum_events":events, "hm_compatibility":hm_compatibility, "terrain": {"origin_x": snapshot.overworld.tile.x.saturating_sub(6), "origin_y": snapshot.overworld.tile.y.saturating_sub(6), "rows": terrain, "note": "Terrain classes describe the current map. Directional permissions, objects, movement mode and game rules still decide whether a move succeeds."}},
        "flow_state": {"animating": visible_noninteractive_field_animation_owns_input(runtime) || visible_battle_command_animation_active(runtime) || runtime.player_walk_frame_ticks > 0, "buttons": ["up", "down", "left", "right", "a", "b", "start", "select"]},
        "multiplayer": multiplayer.map(|m| serde_json::json!({"connected": m.connection.is_some() && !m.failed, "session_active": m.session.is_some(), "pending_request": m.pending_interaction.as_ref().map(|request| serde_json::json!({"player": request.from_display_name, "kind": format!("{:?}", request.kind), "accept": "a", "decline": "b"}))})),
        "recent_events": {"last_action": runtime.last_action_status, "error": runtime.last_error}
    }))
}

fn finish_webmcp_request(
    runtime: Res<BevyRuntimeShell>,
    multiplayer: Option<NonSend<MultiplayerRuntime>>,
    art: Res<RenderedTilesetArt>,
    glyphs: Query<(&Handle<Image>, &Transform, &Visibility)>,
) {
    WEBMCP_BRIDGE.with_borrow_mut(|bridge| {
        let Some(pending) = bridge.pending.as_ref() else { return; };
        let ready = match pending.command {
            WebMcpCommand::Observe {} => true,
            WebMcpCommand::Press { .. } | WebMcpCommand::Multiplayer { .. } => pending.released.is_some_and(|frame| pending.canceled || runtime.lcd_animation_frame.saturating_sub(frame) >= 12),
        };
        if !ready { return; }
        let result = if pending.canceled { serde_json::json!({"error": "Action canceled; the button was released. Already processed input is not undone."}) }
            else { match webmcp_observation(&runtime, multiplayer.as_deref()) { Ok(mut value) => {
                    let mut letters = Vec::new();
                    if let Some(font) = art.font_cache.as_ref() {
                        for (texture, transform, visibility) in &glyphs {
                            if *visibility == Visibility::Hidden { continue; }
                            if let Some((character, _)) = font.glyphs.iter().find(|(_, frame)| frame.handle == *texture) {
                                letters.push((transform.translation.y, transform.translation.x, *character));
                            }
                        }
                    }
                    letters.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.total_cmp(&b.1)));
                    let mut lines = Vec::<String>::new();
                    let mut previous_y = None;
                    for (y, _, character) in letters {
                        if previous_y != Some(y) { lines.push(String::new()); previous_y = Some(y); }
                        if let Some(line) = lines.last_mut() { line.push(character); }
                    }
                    value["observe"]["rendered_text"] = serde_json::json!(lines);
                    value
                }, Err(error) => serde_json::json!({"error": error.to_string()}) } };
        bridge.result = Some((pending.id, result.to_string()));
        bridge.pending = None;
    });
}
