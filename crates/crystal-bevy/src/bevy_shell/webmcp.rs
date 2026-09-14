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

// Report the same active shop surface the player sees, including modal prompts.
fn visible_shop_observation_menu(
    snapshot: &RuntimeShellSnapshot,
    runtime: &BevyRuntimeShell,
) -> Result<Option<serde_json::Value>> {
    let Some(shop) = snapshot.pending_shop.as_ref() else { return Ok(None); };
    if !runtime.shop_welcome_seen || runtime.shop_notice.is_some() {
        let mut entries = Vec::new();
        push_visible_shop_dialog_entries(&mut entries, snapshot, runtime, shop)?;
        return Ok(Some(serde_json::json!({"kind":"shop", "surface":"notice", "entries":entries, "buttons":["a","b"]})));
    }
    if let Some(q) = runtime.shop_quantity.as_ref() {
        let entries = match q.confirmation {
            Some(yes) => vec![if yes {">YES"} else {" YES"}, if yes {" NO"} else {">NO"}],
            None => Vec::new(),
        };
        return Ok(Some(serde_json::json!({"kind":"shop", "surface":if q.confirmation.is_some(){"confirm"}else{"quantity"},
            "entries":entries, "item":q.item_id, "selling":q.selling, "quantity":q.quantity,
            "unit_price":q.unit_price, "total_price":visible_shop_quantity_total(q)})));
    }
    let mut entries = Vec::new();
    push_visible_shop_dialog_entries(&mut entries, snapshot, runtime, shop)?;
    Ok(Some(serde_json::json!({"kind":"shop", "surface":if runtime.shop_top_cursor.is_some(){"top"}else if runtime.sell_cursor.is_some(){"sell"}else{"buy"}, "entries":entries})))
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
    let world = runtime.shell.session().overworld();
    let objects: Vec<_> = snapshot
        .visible_object_runtime_tiles
        .iter()
        .map(|(name, tile)| serde_json::json!({"name": name, "x": tile.x, "y": tile.y, "curriculum_role": world.objects.iter().find(|o| o.object_identifier.as_deref() == Some(name.as_str())).map(|o| &o.script)}))
        .collect();
    let players: Vec<_> = multiplayer.map(|multiplayer| multiplayer.remote_presences.values().filter(|presence| presence.map == snapshot.overworld.map_name).map(|presence| serde_json::json!({"name": presence.display_name, "x": presence.tile_x, "y": presence.tile_y, "facing": presence.direction})).collect()).unwrap_or_default();
    let mut menus = Vec::new();
    if let Some(species) = &snapshot.ui.active_pokemon_picture {
        menus.push(serde_json::json!({"kind": "pokemon_picture", "species": species, "buttons": ["a", "b"]}));
    }
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
        menus.push(serde_json::json!({"kind": "pack", "entries": visible_field_pack_entries(&snapshot, runtime)?, "selected_machine": strict_readonly_cursor_index(&runtime.tmhm_cursor, "bag:tmhm", field_pack_selectable_count(snapshot.bag.tm_hm.len())).and_then(|i| snapshot.bag.tm_hm.get(i)).map(|m| &m.item_id)}));
    }
    if runtime.pokedex_menu_open {
        menus.push(serde_json::json!({"kind": "pokedex", "entries": visible_pokedex_menu_entries(&snapshot, runtime)?}));
    }
    if runtime.pokegear_menu_open {
        menus.push(serde_json::json!({"kind": "pokegear", "entries": visible_pokegear_menu_entries(&snapshot, runtime)?}));
    }
    // Utility pages consume joypad input even though the world stays visible
    // in the runtime snapshot. Report them so clients do not score cursor
    // movement as collisions or checkpoint an open menu as a field boundary.
    if runtime.options_menu_open {
        menus.push(serde_json::json!({"kind": "options", "entries": visible_options_menu_entries(&snapshot, runtime)?}));
    }
    if runtime.trainer_card_open {
        menus.push(serde_json::json!({"kind": "trainer_card", "entries": visible_trainer_card_entries(&snapshot, runtime)}));
    }
    if runtime.save_menu_open {
        let mut entries = Vec::new();
        push_visible_save_dialog_entries(&mut entries, &snapshot, runtime)?;
        menus.push(serde_json::json!({"kind": "save", "entries": entries}));
    }
    if let Some(battle) = &snapshot.battle
        && runtime.battle_messages.is_empty()
        && !visible_battle_command_animation_active(runtime)
    {
        let surface=visible_battle_menu_surface(runtime);
        let mut menu=serde_json::json!({"kind": "battle", "surface": surface, "entries": visible_battle_command_menu_entries(&snapshot, runtime, battle)?});
        if surface == "moves" {
            menu["move_info"]=visible_battle_selected_move_info(&snapshot,runtime,battle);
        }
        menus.push(menu);
    }
    if let Some(menu)=visible_pc_observation_menu(&snapshot,runtime)? {menus.push(menu);}
    if let Some(menu)=visible_shop_observation_menu(&snapshot,runtime)? {menus.push(menu);}
    // Pack tabs are visible UI state. Keep them separate from item rows so
    // cursor indexes and battle render layouts remain unchanged.
    let battle_pack = screen == "battle"
        && (runtime.bag_cursor.is_some() || runtime.ball_cursor.is_some()
            || runtime.key_item_cursor.is_some() || runtime.tmhm_cursor.is_some())
        && runtime.battle_pack_target_mode.is_none();
    let field_pack = runtime.field_pack_pocket.is_some()
        && runtime.field_pack_action_cursor.is_none()
        && runtime.field_pack_target_mode.is_none()
        && menus.last().and_then(|m| m["entries"][0].as_str()).is_some_and(|s| s.starts_with("POCKET:"));
    if (battle_pack || field_pack) && let Some(menu) = menus.last_mut() {
        let pockets = if battle_pack { FIELD_PACK_POCKETS.to_vec() }
            else { carried_field_pack_pockets(&snapshot) };
        menu["pack_pockets"] = serde_json::json!(pockets.iter().map(field_pack_pocket_label).collect::<Vec<_>>());
        menu["pack_pocket"] = serde_json::json!(field_pack_pocket_label(&active_visible_field_pack_pocket(runtime)));
    }
    // Scripted choices must not look like plain dialogue to input consumers.
    // Use the renderer's visibility boundary and cursor; do not expose a
    // pending choice while its preceding text is still being revealed.
    let has_yes_no = menus.iter().any(|menu| {
        let labels: Vec<_> = menu["entries"].as_array().into_iter().flatten()
            .filter_map(|s| s.as_str()).map(|s| s.trim().trim_start_matches('>').trim()).collect();
        labels.contains(&"YES") && labels.contains(&"NO")
    });
    if scene_dialog_yes_no_active(&snapshot, runtime) && !has_yes_no {
        let selected = scene_dialog_yes_no_cursor_index(&snapshot, runtime)?;
        let mut menu=serde_json::json!({"kind":"yes_no", "selected":selected,
            "entries":[if selected == 0 {">YES"} else {" YES"}, if selected == 1 {">NO"} else {" NO"}]});
        if let Some(VisibleSaveFlow{origin:VisibleSaveFlowOrigin::BillsPcChangeBox{box_index},..})=runtime.save_flow.as_ref() {
            menu["purpose"]=serde_json::json!("pc_change_box");
            menu["target_box"]=serde_json::json!(box_index);
            menu["box_index"]=serde_json::json!(snapshot.storage.current_pc_box);
            menu["box_counts"]=serde_json::json!(snapshot.storage.boxes.iter().map(|b|serde_json::json!({"box":b.index,"count":b.count,"capacity":crate::core::models::MAX_BOX_MONS})).collect::<Vec<_>>());
        }
        if let Some(field_move)=runtime.pending_contextual_field_move.as_ref() {
            let name=match field_move {
                PartyFieldMove::Cut=>Some("CUT"), PartyFieldMove::Surf=>Some("SURF"),
                PartyFieldMove::Whirlpool=>Some("WHIRLPOOL"), PartyFieldMove::Waterfall=>Some("WATERFALL"),
                _=>None,
            };
            if let Some(name)=name {menu["purpose"]=serde_json::json!("field_move");menu["field_move"]=serde_json::json!(name);}
        }
        menus.push(menu);
    }
    // Authored navigation assistance, separate from visible terrain. This does
    // not assert that an exit is currently traversable or bypass prerequisites.
    let exits: Vec<_> = world.map_events.warps.iter().map(|w|
        serde_json::json!({"kind":"warp","x":w.x,"y":w.y,"target":w.target_map})
    ).chain(world.map.connections().iter().map(|c|
        serde_json::json!({"kind":"connection","direction":c.direction,"target":c.target_map})
    )).collect();
    // Read-only previews mirror the visible ABLE/NOT ABLE teaching screen.
    // Content/preview errors remain unknown instead of claiming incompatibility.
    let mut hm_compatibility = serde_json::Map::new();
    for item in snapshot.bag.tm_hm.iter().filter(|item|item.quantity>0 && item.item_id.starts_with("HM_")) {
        let slots: Vec<_> = snapshot.party.slots.iter().map(|slot| {
            let able = if slot.pokemon.is_egg {Some(false)} else {
                match runtime.shell.preview_tmhm_on_party_pokemon(&item.item_id,slot.index,None) {
                    Ok(_) => Some(true),
                    Err(error) => match error.downcast_ref::<TmHmLearnError>() {
                        Some(TmHmLearnError::MoveListFull | TmHmLearnError::AlreadyKnows { .. }) => Some(true),
                        Some(TmHmLearnError::CannotLearn { .. }) => Some(false),
                        _ => None,
                    }
                }
            };
            serde_json::json!({"slot":slot.index,"able":able})
        }).collect();
        hm_compatibility.insert(item.item_id.clone(),serde_json::json!(slots));
    }
    let mut boxed_hm_candidates=serde_json::Map::new();
    for item in snapshot.bag.tm_hm.iter().filter(|i|i.quantity>0 && i.item_id.starts_with("HM_")) {
        'boxes: for pc_box in &snapshot.storage.boxes {
            for (position,slot) in pc_box.slots.iter().enumerate().filter(|(_,s)|!s.pokemon.is_egg) {
                let mut candidate=slot.pokemon.clone();
                let compatible=match runtime.runtime.data().teach_tmhm_move(&mut candidate,&item.item_id,None,false) {
                    Ok(_) => true,
                    Err(error) => matches!(error.downcast_ref::<TmHmLearnError>(),Some(TmHmLearnError::MoveListFull | TmHmLearnError::AlreadyKnows { .. })),
                };
                if compatible {
                    boxed_hm_candidates.insert(item.item_id.clone(),serde_json::json!({"box":pc_box.index,"position":position,"species":slot.pokemon.species.id}));
                    break 'boxes;
                }
            }
        }
    }
    let mut enemy_hm_compatibility = serde_json::Map::new();
    if let Some(battle) = snapshot.battle.as_ref() {
        for item in snapshot.bag.tm_hm.iter().filter(|i|i.quantity>0 && i.item_id.starts_with("HM_")) {
            let mut candidate=battle.enemy_pokemon.clone();
            let able=match runtime.runtime.data().teach_tmhm_move(&mut candidate,&item.item_id,None,false) {
                Ok(_) => Some(true),
                Err(error) => match error.downcast_ref::<TmHmLearnError>() {
                    Some(TmHmLearnError::MoveListFull | TmHmLearnError::AlreadyKnows { .. }) => Some(true),
                    Some(TmHmLearnError::CannotLearn { .. }) => Some(false),
                    _ => None,
                }
            };
            enemy_hm_compatibility.insert(item.item_id.clone(),serde_json::json!(able));
        }
    }
    let storage=&runtime.shell.session().state().storage;
    let mut owned_species_counts=std::collections::BTreeMap::<String,usize>::new();
    for pokemon in storage.party.pokemon.iter().chain(storage.pc_boxes.iter().flat_map(|b|b.pokemon.iter())).flatten().filter(|p|!p.is_egg) {
        *owned_species_counts.entry(pokemon.species.id.clone()).or_default()+=1;
    }
    let curriculum_events: Vec<_> = world.map_events.coord_events.iter().map(|e|
        serde_json::json!({"x":e.x,"y":e.y,"scene":e.scene_id,"script":e.script_name})
    ).collect();
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
            "owned_species_counts": owned_species_counts,
            "boxed_hm_candidates": boxed_hm_candidates,
            "enemy_hm_compatibility": enemy_hm_compatibility,
            "hall_of_fame": snapshot.progression.hall_of_fame.count,
            "last_talked_object": snapshot.script_events.last_talked_object,
            "key_items": snapshot.bag.key_items.iter().filter(|i| i.quantity > 0).map(|i| &i.item_id).collect::<Vec<_>>(),
            "items": snapshot.bag.items.iter().chain(&snapshot.bag.balls).filter(|i| i.quantity > 0).map(|i| serde_json::json!({"id":i.item_id,"quantity":i.quantity})).collect::<Vec<_>>(),
            "machines": snapshot.bag.tm_hm.iter().filter(|i| i.quantity > 0).map(|i| &i.item_id).collect::<Vec<_>>(),
            "battle": snapshot.battle.as_ref().map(|b| serde_json::json!({
                "enemy_party": b.enemy_party.iter().map(|p| serde_json::json!({"hp":p.hp,"max_hp":p.max_hp})).collect::<Vec<_>>(),
                "active_enemy": b.active_enemy_party_index,
                "enemy_hp": b.enemy_pokemon.hp,
                "enemy_max_hp": b.enemy_pokemon.max_hp,
                "enemy_status": b.enemy_pokemon.status,
                "active_player": b.active_player_party_index,
                "player_last_move": b.player_last_move,
                "enemy_last_move": b.enemy_last_move,
                "enemy_wrapped": b.enemy_wrapped,
                "player_substitute_hp": b.player_substitute_hp,
                "player_mist_active": b.player_mist_active,
                "rewarded_enemies": b.rewarded_enemy_party_indices,
                "player_turns": b.player_turns_taken,
                "enemy_turns": b.enemy_turns_taken
            }))
        },
        "observe": {"text": text, "visible_dialogue": visible_field_dialog_text(&snapshot, runtime), "menus": menus, "pokemon_switch_open": runtime.party_switch_cursor.is_some() || runtime.battle_switch_cursor.is_some(), "battle_message": runtime.battle_messages.front(), "battle": format_battle_overlay(&snapshot, runtime)},
        "map_info": {"name": snapshot.overworld.map_name, "hm_compatibility": hm_compatibility, "curriculum_exits": exits, "curriculum_events": curriculum_events, "player": {"x": snapshot.overworld.tile.x, "y": snapshot.overworld.tile.y, "facing": format!("{:?}", snapshot.overworld.facing)}, "dimensions": runtime.runtime.data().saved_map_tile_bounds(&snapshot.overworld.map_name), "objects": objects, "players": players, "terrain": {"origin_x": snapshot.overworld.tile.x.saturating_sub(6), "origin_y": snapshot.overworld.tile.y.saturating_sub(6), "rows": terrain, "note": "Terrain classes describe the current map. Directional permissions, objects, movement mode and game rules still decide whether a move succeeds."}},
        "flow_state": {"animating": runtime.visible_script_movement.is_some() || runtime.incoming_phone_sequence.is_some() || visible_noninteractive_field_animation_owns_input(runtime) || visible_battle_command_animation_active(runtime) || runtime.player_walk_frame_ticks > 0, "buttons": ["up", "down", "left", "right", "a", "b", "start", "select"]},
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
