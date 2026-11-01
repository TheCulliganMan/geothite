#[test]
fn text_controller_settles_audio_without_skipping_unread_text() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.field_notice = Some("Sound must not stop this page.".into());
    shell.shell.session_mut().state_mut().script_runtime.text_window_open = true;
    queue_visible_shell_sound_effect(&mut shell, "SFX_ITEM").unwrap();
    let special = shell.shell.wait_sfx_special().unwrap();
    activate_visible_special_routine_boundary(&mut shell, &special.outcome.effect).unwrap();
    settle_visible_shell_until_input(&mut shell).expect("audio fence must allow printer progress");
    assert!(!shell.visible_wait_sfx_boundary);
    assert_eq!(shell.field_notice.as_deref(), Some("Sound must not stop this page."));
    assert!(visible_field_text_reveal_is_complete_for_text(&shell, "Sound must not stop this page."));
    let mut shell = progression_shell_on_map_for_test("PlayersHouse1F");
    let key = shell.shell.runtime().script_text_command_keys().into_iter()
        .find(|key| key.text_label.as_deref() == Some("ElmsLookingForYouText")).unwrap();
    shell.shell.apply_script_text_command(&key.map_name, &key.source_script, key.command_index).unwrap();
    let special = shell.shell.wait_sfx_special().unwrap();
    activate_visible_special_routine_boundary(&mut shell, &special.outcome.effect).unwrap();
    settle_visible_shell_until_input(&mut shell).unwrap();
    assert!(shell.visible_wait_sfx_boundary, "sound completion must not swallow Mom's paragraph wait");
    assert_eq!(shell.field_text_reveal.as_ref().unwrap().page_index, 0);
    settle_visible_shell_until_input(&mut shell).unwrap();
    assert_eq!(shell.field_text_reveal.as_ref().unwrap().page_index, 0);
    let mut controller = VisibleShellController { shell };
    controller.press(GameButton::A).unwrap();
    assert_eq!(controller.shell.field_text_reveal.as_ref().unwrap().page_index, 1);
}

#[test]
fn text_controller_settles_pokedex_search_and_phone_rings() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.record_pokedex_caught("CYNDAQUIL").unwrap();
    open_visible_pokedex_menu(&mut shell).unwrap();
    shell.pokedex_controls.search_types = [2, 0];
    shell.pokedex_controls.search_cursor = Some(2);
    let mut controller = VisibleShellController { shell };
    controller.press(GameButton::A).unwrap();
    assert!(controller.shell.pokedex_controls.search_animation.is_none(), "search must finish without a graphical clock");
    assert!(controller.shell.pokedex_controls.search_results.is_some());
    controller.press(GameButton::B).unwrap();
    controller.shell.pokedex_controls.search_types = [16, 0];
    controller.shell.pokedex_controls.search_cursor = Some(2);
    controller.press(GameButton::A).unwrap();
    assert!(!pokedex_input_delay_active(&controller.shell), "empty search must release input too");
    controller.press(GameButton::B).unwrap();
    controller.press(GameButton::B).unwrap();
    controller.shell.shell.initialize_permanent_phone_numbers().unwrap();
    open_visible_pokegear_menu(&mut controller.shell).unwrap();
    controller.shell.pokegear_page = PokegearPage::Phone;
    controller.press(GameButton::A).unwrap();
    // Selecting a contact first opens CALL/DELETE/CANCEL.
    controller.press(GameButton::A).unwrap();
    assert!(controller.shell.pokegear_phone_call.as_ref().is_some_and(|call| matches!(call.phase, VisiblePokegearPhoneCallPhase::Calling)), "both sound rings must enter the authored callback");
    assert!(controller.shell.shell.snapshot().unwrap().script_events.memory.contains_key("wPhoneScriptBank"));
    for _ in 0..100 {
        if controller.shell.pokegear_phone_call.is_none() { break; }
        controller.press(GameButton::A).unwrap();
    }
    assert!(controller.shell.pokegear_phone_call.is_none(), "dialogue and hangup must finish");
    assert!(controller.shell.pokegear_menu_open);
    controller.press(GameButton::B).unwrap();
    assert!(!controller.shell.pokegear_menu_open);
}

#[test]
fn text_controller_pack_heals_and_pokegear_cards_remain_playable() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.add_bag_item("POTION", 2).unwrap();
    shell.shell.set_party_pokemon_recovery_state(0, 1, None, None).unwrap();
    let mut controller = VisibleShellController { shell };
    controller.press(GameButton::Start).unwrap();
    controller.press(GameButton::Down).unwrap(); // Party precedes Pack.
    controller.press(GameButton::A).unwrap();
    assert!(visible_field_pack_is_open(&controller.shell));
    for _ in 0..3 { controller.press(GameButton::A).unwrap(); }
    let state = controller.shell.shell.session().state();
    assert_eq!(state.bag.items.get("POTION"), Some(&1));
    assert_eq!(state.storage.party.pokemon[0].as_ref().unwrap().hp, 21);
    for _ in 0..6 { controller.press(GameButton::B).unwrap(); }
    controller.shell.shell.set_script_flag_for_smoke("ENGINE_MAP_CARD").unwrap();
    controller.shell.shell.set_script_flag_for_smoke("ENGINE_RADIO_CARD").unwrap();
    open_visible_pokegear_menu(&mut controller.shell).unwrap();
    controller.press(GameButton::Right).unwrap();
    assert_eq!(controller.shell.pokegear_page, PokegearPage::Map);
    let before = controller.snapshot().unwrap().ui.menu.unwrap();
    controller.press(GameButton::Up).unwrap();
    assert_ne!(controller.snapshot().unwrap().ui.menu.unwrap(), before);
    controller.press(GameButton::Right).unwrap();
    assert_eq!(controller.shell.pokegear_page, PokegearPage::Radio);
    assert!(!visible_pokegear_menu_entries(&controller.shell.shell.snapshot().unwrap(), &controller.shell).unwrap().is_empty());
    for _ in 0..8 { controller.press(GameButton::Up).unwrap(); }
    assert!(controller.shell.pokegear_radio_broadcast.is_some());
    assert!(visible_radio_observation_rows(&controller.shell).unwrap().iter().any(|row| !row.trim().is_empty()));
    controller.press(GameButton::B).unwrap();
    assert!(!controller.shell.pokegear_menu_open);
    controller.press(GameButton::Start).unwrap();
    assert!(visible_start_menu_entries(&controller.shell).unwrap().iter().any(|row| row.contains("SAVE")));
}

#[test]
fn text_controller_incoming_phone_completes_rings_and_hangup() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let map_name = shell.shell.snapshot().unwrap().overworld.map_name;
    let state = shell.shell.session_mut().state_mut();
    state.script_runtime.special_phone_call = Some("SPECIALCALL_ROBBED".into());
    state.script_runtime.variables.insert("VAR_CALLERID".into(), "PHONE_ELM".into());
    state.script_runtime.memory.insert("wCallerContact + PHONE_CONTACT_SCRIPT2_BANK".into(), "ElmPhoneCallerScript".into());
    shell.active_script_cursor = Some(ActiveScriptCursor {
        origin_map_name: map_name, source_script: "Script_ReceivePhoneCall".into(), next_command_index: 1,
    });
    execute_visible_active_script_step(&mut shell).unwrap();
    let mut controller = VisibleShellController { shell };
    settle_visible_shell_until_input(&mut controller.shell).unwrap();
    assert!(controller.shell.incoming_phone_sequence.is_none());
    assert_eq!(controller.shell.shell.snapshot().unwrap().ui.text.unwrap().label, "ElmPhoneDisasterText");
    for _ in 0..40 {
        controller.press(GameButton::A).unwrap();
        if controller.shell.active_script_cursor.is_none() { break; }
    }
    assert!(controller.shell.incoming_phone_sequence.is_none());
    assert!(controller.shell.shell.session().state().flags
        .is_event_flag_set("EVENT_ELM_CALLED_ABOUT_STOLEN_POKEMON").unwrap());
    controller.press(GameButton::Start).unwrap();
    assert!(visible_start_menu_entries(&controller.shell).unwrap().iter().any(|row| row.contains("SAVE")));
}

#[test]
fn text_controller_pc_item_switch_finishes_both_sound_fences() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.session_mut().state_mut().bag.pc_items.clear();
    for id in ["POTION", "ANTIDOTE"] {
        let item = shell.shell.runtime().data().items[id].clone();
        shell.shell.session_mut().state_mut().bag.add_pc_item(&item, 1).unwrap();
    }
    shell.pc_item_action = Some(VisiblePlayerPcAction::WithdrawItem);
    shell.pc_item_cursor = Some(MenuCursor { surface_id: "pc:items".into(), option_index: 0 });
    let mut controller = VisibleShellController { shell };
    controller.press(GameButton::Select).unwrap();
    controller.press(GameButton::Down).unwrap();
    controller.press(GameButton::A).unwrap();
    assert!(controller.shell.pc_item_move_sequence.is_none());
    assert_eq!(controller.shell.shell.snapshot().unwrap().bag.pc_items[0].item_id, "ANTIDOTE");
    controller.press(GameButton::B).unwrap();
    assert!(controller.shell.pc_item_cursor.is_none());
}

fn initialized_mail_reader_shell(mail_type: &str) -> BevyRuntimeShell {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let asset_root = AssetRoot::new(repo_root);
    let runtime = workspace_desktop_runtime(&asset_root);
    let spawn_identifier = runtime
        .title_new_game_spawn_identifier()
        .expect("title new-game spawn");
    let mut shell = initialize_bevy_runtime_shell(
        asset_root,
        runtime,
        BevyShellStart::NewGame { spawn_identifier },
        BevyShellConfig {
            smoke_player_name: Some("AB".to_string()),
            ..Default::default()
        },
    )
    .expect("initialize Mail reader shell");
    complete_visible_smoke_player_name_if_needed(&mut shell, Some("AB"))
        .expect("complete player name");
    settle_visible_shell_smoke_until_idle(&mut shell).expect("settle arrival scripts");
    let species = shell.shell.runtime().data().pokemon["CYNDAQUIL"].clone();
    let mut pokemon = crate::core::models::Pokemon::new_for_tests(
        species,
        10,
        crate::core::models::Dv::default(),
    );
    pokemon.item = Some(mail_type.to_string());
    pokemon.mail = Some(crate::core::models::pokemon::MailData {
        message: "HELLO FROM JOHTO".to_string(),
        author: "AB".to_string(),
        nationality: 0,
        author_id: 0x1234,
        species: "CYNDAQUIL".to_string(),
        mail_type: mail_type.to_string(),
    });
    shell.shell.session_mut().state_mut().storage.party.pokemon[0] = Some(pokemon);
    shell
        .shell
        .session_mut()
        .state_mut()
        .sync_party_from_storage();
    shell
}

fn sync_synthetic_current_map_image(runtime_shell: &mut BevyRuntimeShell) {
    let (state, overworld) = runtime_shell.shell.session_mut().state_and_overworld_mut();
    crate::core::systems::map_context::sync_state_object_overrides(
        state,
        overworld,
    )
    .expect("sync synthetic current-map object image");
}
