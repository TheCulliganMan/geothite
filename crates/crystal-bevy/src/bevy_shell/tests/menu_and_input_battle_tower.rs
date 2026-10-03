fn seventh_battle_tower_win_shell() -> BevyRuntimeShell {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.shell.session_mut().state_mut().overworld = crate::core::state::OverworldMemory::Active {
        map_name: "BattleTowerBattleRoom".to_string(),
        tile: TilePosition::new(4, 6),
        facing: Direction::Up,
        mode: MovementMode::Normal,
    };
    *runtime_shell.shell.session_mut().overworld_mut() = runtime_shell
        .shell
        .runtime()
        .data()
        .overworld_session("BattleTowerBattleRoom", TilePosition::new(4, 6), 0)
        .expect("start Battle Tower battle-room session");
    sync_synthetic_current_map_image(&mut runtime_shell);
    runtime_shell.shell.session_mut().state_mut().battle_tower.level_group = 1;
    runtime_shell
        .shell
        .session_mut().state_mut()
        .battle_tower
        .beaten_trainers = 6;
    runtime_shell.shell.session_mut().state_mut().battle_tower.reward_item = "HP_UP".to_string();
    runtime_shell
        .shell
        .load_battle_tower_opponent_special("BATTLETOWERBATTLEROOM_YOUNGSTER".to_string())
        .expect("load seventh Battle Tower opponent");

    let battle_command_index = runtime_shell
        .shell
        .runtime()
        .compiled_script_commands("Script_BattleRoomLoop")
        .expect("compiled Battle Tower room loop")
        .iter()
        .position(|command| {
            command.get("command").and_then(serde_json::Value::as_str) == Some("special")
                && command
                    .get("args")
                    .and_then(serde_json::Value::as_array)
                    .and_then(|args| args.first())
                    .and_then(serde_json::Value::as_str)
                    == Some("BattleTowerBattle")
        })
        .expect("BattleTowerBattle command");
    runtime_shell
        .shell
        .apply_compiled_script_command(
            "BattleTowerBattleRoom",
            "Script_BattleRoomLoop",
            battle_command_index,
            ScriptRuntimeInputs::default(),
            ScriptPhoneInputs::default(),
        )
        .expect("start the source BattleTowerBattle special");
    arm_visible_active_script_cursor_with_origin(
        &mut runtime_shell,
        "BattleTowerBattleRoom",
        "Script_BattleRoomLoop",
        battle_command_index + 1,
    );
    runtime_shell.shell.session_mut().state_mut().battle_result = 0;
    runtime_shell
}

#[test]
fn seventh_battle_tower_win_warps_to_1f_and_awards_five_source_rewards() {
    let mut runtime_shell = seventh_battle_tower_win_shell();

    complete_visible_battle_tower_battle(&mut runtime_shell)
        .expect("resume room script after seventh win");
    settle_visible_shell_smoke_until_idle(&mut runtime_shell)
        .expect("complete seventh-win warp and prize script");

    let state = runtime_shell.shell.session().state();
    assert_eq!(runtime_shell.shell.current_map_name(), "BattleTower1F");
    assert_eq!(state.battle_tower.beaten_trainers, 7);
    assert_eq!(
        state.battle_tower.challenge_state,
        4,
        "prize script did not finish: error={:?} cursor={:?} phase={:?} boundary={:?} text={:?} events={:?}",
        runtime_shell.last_error,
        runtime_shell.active_script_cursor,
        runtime_shell.visible_walk_warp_phase,
        runtime_shell.special_boundary,
        state.script_runtime.pending_text_label,
        runtime_shell.last_audio_events
    );
    assert!(state.battle_tower.reward_given);
    assert_eq!(state.bag.items.get("HP_UP"), Some(&5));
}

#[test]
fn seventh_battle_tower_win_keeps_the_prize_claimable_when_the_item_pocket_is_full() {
    let mut runtime_shell = seventh_battle_tower_win_shell();
    let full_item_pocket = runtime_shell
        .shell
        .runtime()
        .data()
        .items
        .iter()
        .filter(|(item_id, item)| item.pocket == "ITEM" && item_id.as_str() != "HP_UP")
        .take(20)
        .map(|(item_id, _)| item_id.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        full_item_pocket.len(),
        20,
        "test pack needs twenty item slots"
    );
    runtime_shell.shell.session_mut().state_mut().bag.items.clear();
    for item_id in full_item_pocket {
        runtime_shell
            .shell
            .session_mut().state_mut()
            .bag
            .items
            .insert(item_id, 1);
    }

    complete_visible_battle_tower_battle(&mut runtime_shell)
        .expect("resume room script after full-Pack seventh win");
    settle_visible_shell_smoke_until_idle(&mut runtime_shell)
        .expect("complete full-Pack prize refusal");

    let state = runtime_shell.shell.session().state();
    assert_eq!(runtime_shell.shell.current_map_name(), "BattleTower1F");
    assert_eq!(state.battle_tower.beaten_trainers, 7);
    assert_eq!(state.battle_tower.challenge_state, 3);
    assert!(!state.battle_tower.reward_given);
    assert_eq!(state.bag.items.get("HP_UP"), None);
}

#[test]
fn visible_pack_menu_renders_and_confirms_cancel_row_from_normal_inputs() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let asset_root = AssetRoot::new(repo_root);
    let runtime = workspace_desktop_runtime(&asset_root);
    let spawn_identifier = runtime
        .title_new_game_spawn_identifier()
        .expect("title new-game spawn");
    let mut runtime_shell = initialize_bevy_runtime_shell(
        asset_root,
        runtime,
        BevyShellStart::NewGame { spawn_identifier },
        BevyShellConfig {
            smoke_player_name: Some("AB".to_string()),
            ..Default::default()
        },
    )
    .expect("initialize overworld shell");
    complete_visible_smoke_player_name_if_needed(&mut runtime_shell, Some("AB"))
        .expect("complete player name and run arrival callbacks");
    settle_visible_shell_smoke_until_idle(&mut runtime_shell)
        .expect("settle arrival scripts before opening the Pack");
    runtime_shell
        .shell
        .add_bag_item("POTION", 1)
        .expect("add Potion for Pack test");

    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::Start])
        .expect("Start opens the start menu through normal input dispatch");
    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::A])
        .expect("A opens Pack from the start menu through normal input dispatch");
    {
        let snapshot = runtime_shell.shell.snapshot().expect("Pack snapshot");
        let entries =
            visible_field_pack_entries(&snapshot, &runtime_shell).expect("render Pack item list");
        assert_eq!(entries.first().map(String::as_str), Some("POCKET: ITEMS"));
        assert!(
            entries.iter().any(|entry| entry == ">POTION x01"),
            "Pack should expose item quantity rows: {entries:?}"
        );
        assert!(
            entries.iter().any(|entry| entry == " CANCEL"),
            "Pack should expose the trailing CANCEL row: {entries:?}"
        );
        let items = snapshot
            .bag
            .items
            .iter()
            .filter(|item| item.quantity > 0)
            .map(|item| (item.item_id.clone(), item.quantity))
            .collect::<Vec<_>>();
        let mut images = Assets::<Image>::default();
        let frame = load_visible_field_pack_frame(
            &snapshot,
            &runtime_shell,
            &FieldPackPocket::Items,
            &items,
            0,
            0,
            snapshot
                .items
                .iter()
                .find(|item| item.item_id == "POTION")
                .map(|item| item.description.as_str())
                .expect("Potion description"),
            &mut images,
        )
        .expect("compose canonical Pack LCD frame");
        let image = images.get(&frame.handle).expect("Pack frame image");
        assert_eq!(image.texture_descriptor.size.width, 160);
        assert_eq!(image.texture_descriptor.size.height, 144);
        let colors = image
            .data
            .chunks_exact(4)
            .map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .collect::<std::collections::BTreeSet<_>>();
        assert!(
            colors.len() >= 7,
            "Pack must compose its canonical menu, icon, label, font, and textbox palettes; colors={colors:?}"
        );
    }

    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::A])
        .expect("A opens the Pack action menu for the selected item");
    {
        let snapshot = runtime_shell
            .shell
            .snapshot()
            .expect("Pack action snapshot");
        let entries =
            visible_field_pack_entries(&snapshot, &runtime_shell).expect("render Pack action menu");
        assert_eq!(
            entries,
            vec!["ACTION POTION x01", ">USE", " GIVE", " TOSS", " QUIT"],
            "Pack action menu should match the TypeScript item action options"
        );
    }
    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::Down])
        .expect("Down moves inside the Pack action menu");
    {
        let snapshot = runtime_shell
            .shell
            .snapshot()
            .expect("Pack action moved snapshot");
        assert!(
            visible_field_pack_entries(&snapshot, &runtime_shell)
                .expect("render moved Pack action menu")
                .iter()
                .any(|entry| entry == ">GIVE"),
            "Pack action cursor should move before item-list cursor movement"
        );
    }
    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::B])
        .expect("B closes the Pack action menu without closing Pack");
    {
        let snapshot = runtime_shell
            .shell
            .snapshot()
            .expect("Pack list restored snapshot");
        assert!(
            visible_field_pack_entries(&snapshot, &runtime_shell)
                .expect("render restored Pack list")
                .iter()
                .any(|entry| entry == ">POTION x01"),
            "B in the Pack action menu should return to the item list"
        );
    }

    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::A])
        .expect("A reopens the Pack action menu before tossing");
    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::Down])
        .expect("Down moves from USE to GIVE");
    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::Down])
        .expect("Down moves from GIVE to TOSS");
    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::A])
        .expect("A opens the ASM toss quantity prompt");
    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::A])
        .expect("A confirms the toss quantity");
    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::A])
        .expect("A accepts the toss confirmation");
    {
        let snapshot = runtime_shell
            .shell
            .snapshot()
            .expect("Pack tossed snapshot");
        let entries =
            visible_field_pack_entries(&snapshot, &runtime_shell).expect("render Pack after toss");
        assert!(
            entries.iter().any(|entry| entry == ">CANCEL"),
            "Tossing the only item should leave the Pack on CANCEL: {entries:?}"
        );
        assert_eq!(
            runtime_shell.last_action_status.as_deref(),
            Some("TOSSED POTION x1")
        );
    }

    for _ in 0..256 {
        let snapshot = runtime_shell
            .shell
            .presentation_snapshot()
            .expect("Pack notice presentation snapshot");
        if visible_field_dialogue_is_fully_revealed(&runtime_shell, &snapshot) {
            break;
        }
        tick_visible_field_text_reveal(&mut runtime_shell, true)
            .expect("advance the canonical threw-away notice printer");
    }
    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::A])
        .expect("A dismisses the canonical threw-away notice");
    {
        let snapshot = runtime_shell
            .shell
            .snapshot()
            .expect("Pack cancel snapshot");
        assert!(
            visible_field_pack_entries(&snapshot, &runtime_shell)
                .expect("render Pack CANCEL row")
                .iter()
                .any(|entry| entry == ">CANCEL"),
            "Pack cursor should reach the CANCEL row"
        );
    }

    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::A])
        .expect("A closes Pack from the CANCEL row through normal input dispatch");
    assert!(!visible_field_pack_is_open(&runtime_shell));
    assert_eq!(
        runtime_shell.last_action_status.as_deref(),
        Some("PACK CLOSED")
    );
}

#[test]
fn visible_mail_composer_uses_asm_grid_and_atomically_attaches_player_mail() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let asset_root = AssetRoot::new(repo_root);
    let runtime = workspace_desktop_runtime(&asset_root);
    let spawn_identifier = runtime
        .title_new_game_spawn_identifier()
        .expect("title new-game spawn");
    let mut runtime_shell = initialize_bevy_runtime_shell(
        asset_root,
        runtime,
        BevyShellStart::NewGame { spawn_identifier },
        BevyShellConfig {
            smoke_player_name: Some("AB".to_string()),
            ..Default::default()
        },
    )
    .expect("initialize overworld shell");
    complete_visible_smoke_player_name_if_needed(&mut runtime_shell, Some("AB"))
        .expect("complete player name");
    settle_visible_shell_smoke_until_idle(&mut runtime_shell).expect("settle arrival scripts");
    let species = runtime_shell.shell.runtime().data().pokemon["CYNDAQUIL"].clone();
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .storage
        .party
        .pokemon[0] = Some(crate::core::models::Pokemon::new_for_tests(
        species,
        10,
        crate::core::models::Dv::default(),
    ));
    runtime_shell.shell.session_mut().state_mut().player_id = 0x1234;
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .sync_party_from_storage();
    runtime_shell
        .shell
        .add_bag_item("FLOWER_MAIL", 1)
        .expect("add Mail to Pack");
    open_visible_field_pack_pocket(&mut runtime_shell, FieldPackPocket::Items)
        .expect("construct the Items pocket");
    runtime_shell.party_held_item_give_target = Some(0);

    give_selected_held_item(&mut runtime_shell).expect("open Mail composer");
    let input = runtime_shell
        .pending_mail_input
        .as_ref()
        .expect("Mail item must open the source composer");
    assert_eq!((input.cursor_column, input.cursor_row), (0, 0));
    assert_eq!(
        visible_mail_input_layout(NameInputCase::Upper),
        &[
            "A B C D E F G H I J",
            "K L M N O P Q R S T",
            "U V W X Y Z   , ? !",
            "1 2 3 4 5 6 7 8 9 0",
            "<PK> <MN> <PO> <KE> é ♂ ♀ ¥ … ×",
            "lower  DEL   END   ",
        ]
    );

    let mut images = Assets::<Image>::default();
    let frame = load_mail_entry_frame(&runtime_shell.asset_root, input, &mut images)
        .expect("render Mail composer LCD");
    assert_eq!(frame.size, Vec2::new(160.0, 144.0));

    select_visible_mail_grid_key(&mut runtime_shell).expect("write A");
    move_visible_mail_cursor(&mut runtime_shell, 1, 0).expect("move to B");
    select_visible_mail_grid_key(&mut runtime_shell).expect("write B");
    runtime_shell
        .pending_mail_input
        .as_mut()
        .expect("composer remains open")
        .cursor_row = MAIL_INPUT_ROWS - 1;
    runtime_shell
        .pending_mail_input
        .as_mut()
        .expect("composer remains open")
        .cursor_column = 9;
    select_visible_mail_grid_key(&mut runtime_shell).expect("finish Mail");

    let state = runtime_shell.shell.session().state();
    let pokemon = state.storage.party.pokemon[0]
        .as_ref()
        .expect("party Pokemon");
    assert_eq!(pokemon.item.as_deref(), Some("FLOWER_MAIL"));
    let mail = pokemon.mail.as_ref().expect("composed Mail metadata");
    assert_eq!(mail.message, "AB");
    assert_eq!(mail.author, "AB");
    assert_eq!(mail.nationality, 0);
    assert_eq!(mail.author_id, 0x1234);
    assert_eq!(mail.species, "CYNDAQUIL");
    assert_eq!(mail.mail_type, "FLOWER_MAIL");
    assert_eq!(
        runtime_shell
            .shell
            .snapshot()
            .expect("Mail snapshot")
            .bag
            .items
            .iter()
            .find(|entry| entry.item_id == "FLOWER_MAIL")
            .map(|entry| entry.quantity)
            .unwrap_or(0),
        0
    );
    state
        .validate_saved_state()
        .expect("composed Mail save state");
}

#[test]
fn party_mail_read_opens_item_specific_full_lcd_until_game_boy_dismissal() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.party_menu_open = true;
    runtime_shell.party_cursor = 0;
    runtime_shell.party_give_take_cursor = Some(MenuCursor {
        surface_id: "party:mail-actions".to_string(),
        option_index: 0,
    });

    confirm_visible_party_mail_action(&mut runtime_shell).expect("READ party Mail");
    assert!(runtime_shell.field_notice.is_none());
    let reader = runtime_shell
        .pending_mail_read
        .as_ref()
        .expect("READ must own a full Mail LCD");
    assert_eq!(reader.mail.mail_type, "FLOWER_MAIL");

    let mut images = Assets::<Image>::default();
    let frame = load_mail_read_frame(&runtime_shell.asset_root, reader, &mut images)
        .expect("render source Flower Mail stationery");
    assert_eq!(frame.size, Vec2::new(160.0, 144.0));
    let image = images.get(&frame.handle).expect("Mail reader image");
    assert!(
        image
            .data
            .chunks_exact(4)
            .any(|pixel| pixel[..3] != [255, 255, 255]),
        "Mail stationery must contain source border/art pixels"
    );

    close_visible_mail_read(&mut runtime_shell).expect("B closes Mail reader");
    assert!(runtime_shell.pending_mail_read.is_none());
    assert!(runtime_shell.party_give_take_cursor.is_some());
}

#[test]
fn every_asm_mail_type_builds_its_source_stationery_lcd() {
    let mut images = Assets::<Image>::default();
    for mail_type in crate::core::models::item::MAIL_ITEM_IDS {
        let runtime_shell = initialized_mail_reader_shell(mail_type);
        let reader = VisibleMailRead {
            mail: runtime_shell.shell.session().state().storage.party.pokemon[0]
                .as_ref()
                .and_then(|pokemon| pokemon.mail.clone())
                .expect("Mail fixture"),
        };
        let frame = load_mail_read_frame(&runtime_shell.asset_root, &reader, &mut images)
            .unwrap_or_else(|error| panic!("{mail_type} stationery failed: {error:#}"));
        assert_eq!(frame.size, Vec2::new(160.0, 144.0), "{mail_type}");
    }
}

#[test]
fn card_flip_lcd_composes_source_cards_instead_of_text_placeholders() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let asset_root = AssetRoot::new(repo_root);
    let sources = load_card_flip_render_sources(&asset_root).expect("Card Flip source art");
    let mut images = Assets::<Image>::default();
    let mut game = VisibleCardFlip {
        phase: VisibleCardFlipPhase::AskPlay,
        animation: VisibleCardFlipAnimation::None,
        yes_no_index: 0,
        which_card: 0,
        bet_x: 2,
        bet_y: 2,
        round: 0,
        face_card: None,
        coins: 99,
        payout: 0,
        deck: Vec::new(),
        revealed: vec![false; 24],
        message: "PLAY WITH THREE COINS?".to_string(),
    };
    let ask = render_visible_card_flip_frame(&sources, &game, &mut images)
        .expect("render initial Card Flip LCD");
    game.phase = VisibleCardFlipPhase::ChooseCard;
    game.animation = VisibleCardFlipAnimation::Cycle {
        frames_until_toggle: 4,
    };
    let choose = render_visible_card_flip_frame(&sources, &game, &mut images)
        .expect("render face-down cards");
    game.phase = VisibleCardFlipPhase::PlayAgain;
    game.face_card = Some(("PIKACHU".to_string(), 1));
    game.revealed[0] = true;
    let reveal =
        render_visible_card_flip_frame(&sources, &game, &mut images).expect("render revealed card");

    let ask_pixels = images.get(&ask.handle).expect("initial image").data.clone();
    let choose_pixels = images
        .get(&choose.handle)
        .expect("choose image")
        .data
        .clone();
    let reveal_pixels = images
        .get(&reveal.handle)
        .expect("reveal image")
        .data
        .clone();
    assert_ne!(
        ask_pixels, choose_pixels,
        "face-down cards must be BG tiles, not host text"
    );
    assert_ne!(
        choose_pixels, reveal_pixels,
        "revealing must replace the chosen card with its source face"
    );
}

#[test]
fn card_flip_group_bet_cursors_use_the_complete_asm_oam_extents() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let sources =
        load_card_flip_render_sources(&AssetRoot::new(repo_root)).expect("Card Flip source art");

    let mut poke_group = vec![0_u8; 160 * 144 * 4];
    draw_card_flip_bet_cursor(&sources, 2, 1, &mut poke_group);
    let poke_group_max_y = poke_group
        .chunks_exact(4)
        .enumerate()
        .filter(|(_, pixel)| pixel[3] != 0)
        .map(|(index, _)| index / 160)
        .max()
        .expect("Pokemon group cursor pixels");
    assert!(
        poke_group_max_y >= 111,
        "Pokemon group cursor must span the source's eleven-tile perimeter"
    );

    let mut number_group = vec![0_u8; 160 * 144 * 4];
    draw_card_flip_bet_cursor(&sources, 1, 2, &mut number_group);
    let number_group_min_x = number_group
        .chunks_exact(4)
        .enumerate()
        .filter(|(_, pixel)| pixel[3] != 0)
        .map(|(index, _)| index % 160)
        .min()
        .expect("number group cursor pixels");
    let number_group_max_x = number_group
        .chunks_exact(4)
        .enumerate()
        .filter(|(_, pixel)| pixel[3] != 0)
        .map(|(index, _)| index % 160)
        .max()
        .expect("number group cursor pixels");
    assert!(
        number_group_min_x <= 95 && number_group_max_x >= 159,
        "number group cursor must span the source's nine-tile horizontal perimeter"
    );
}

#[test]
fn visible_card_flip_commits_the_stake_and_deck_before_card_selection() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let coin_case = runtime_shell.shell.runtime().data().items["COIN_CASE"].clone();
    runtime_shell.shell.session_mut().state_mut().coins = 99;
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .bag
        .add_item(&coin_case, 1)
        .expect("add Coin Case");
    runtime_shell.visible_card_flip = Some(VisibleCardFlip {
        phase: VisibleCardFlipPhase::AskPlay,
        animation: VisibleCardFlipAnimation::None,
        yes_no_index: 0,
        which_card: 0,
        bet_x: 2,
        bet_y: 2,
        round: 0,
        face_card: None,
        coins: 99,
        payout: 0,
        deck: Vec::new(),
        revealed: vec![false; 24],
        message: "PLAY WITH THREE COINS?".to_string(),
    });
    flip_visible_card(&mut runtime_shell).expect("accept Card Flip stake");

    let game = runtime_shell
        .visible_card_flip
        .as_ref()
        .expect("Card Flip remains open");
    assert_eq!(game.phase, VisibleCardFlipPhase::ChooseCard);
    assert_eq!(game.coins, 96);
    assert_eq!(game.deck.len(), 24);
    assert!(game.revealed.iter().all(|revealed| !revealed));
    assert_eq!(runtime_shell.shell.session().state().coins, 96);
    assert_eq!(game.animation, VisibleCardFlipAnimation::WaitStake);
    assert!(runtime_shell.pending_audio.iter().any(|command| {
        command.audio_id == "SFX_TRANSACTION"
            && command.kind == crystal_assets::ModpackAudioKind::SoundEffect
    }));
    advance_visible_card_flip_animation(&mut runtime_shell)
        .expect("stake remains blocked on queued transaction sound");
    assert_eq!(
        runtime_shell.visible_card_flip.as_ref().unwrap().animation,
        VisibleCardFlipAnimation::WaitStake
    );
    runtime_shell.pending_audio.clear();
    runtime_shell.transient_audio_playing = true;
    advance_visible_card_flip_animation(&mut runtime_shell)
        .expect("stake remains blocked on playing transaction sound");
    assert_eq!(
        runtime_shell.visible_card_flip.as_ref().unwrap().animation,
        VisibleCardFlipAnimation::WaitStake
    );
    runtime_shell.transient_audio_playing = false;
    advance_visible_card_flip_animation(&mut runtime_shell)
        .expect("transaction completion starts dealing");
    assert_eq!(
        runtime_shell.visible_card_flip.as_ref().unwrap().animation,
        VisibleCardFlipAnimation::Deal { frame: 0 }
    );

    for _ in 0..40 {
        advance_visible_card_flip_animation(&mut runtime_shell).expect("deal Card Flip frame");
    }
    assert!(matches!(
        runtime_shell.visible_card_flip.as_ref().unwrap().animation,
        VisibleCardFlipAnimation::Cycle { .. }
    ));

    let selected_before_direction = runtime_shell.visible_card_flip.as_ref().unwrap().which_card;
    move_visible_card_flip_cursor(&mut runtime_shell, 1, 0)
        .expect("directional input during automatic card cycling");
    assert_eq!(
        runtime_shell
            .visible_card_flip
            .as_ref()
            .expect("Card Flip remains open")
            .which_card,
        selected_before_direction,
        "ChooseACard ignores directions and advances its border automatically"
    );

    for _ in 0..4 {
        advance_visible_card_flip_animation(&mut runtime_shell).expect("cycle Card Flip border");
    }
    assert_eq!(
        runtime_shell.visible_card_flip.as_ref().unwrap().which_card,
        selected_before_direction ^ 1,
        "ChooseACard alternates the highlighted card every four source delay frames"
    );

    flip_visible_card(&mut runtime_shell).expect("choose first card");
    assert!(matches!(
        runtime_shell.visible_card_flip.as_ref().unwrap().animation,
        VisibleCardFlipAnimation::SelectFlash { frame: 0 }
    ));
    for _ in 0..24 {
        advance_visible_card_flip_animation(&mut runtime_shell).expect("flash selected card");
    }
    assert_eq!(
        runtime_shell.visible_card_flip.as_ref().unwrap().phase,
        VisibleCardFlipPhase::PlaceBet
    );
    {
        let game = runtime_shell.visible_card_flip.as_mut().unwrap();
        let face = game.deck[game.round * 2 + game.which_card]
            .parse::<u8>()
            .expect("encoded Card Flip face");
        game.bet_x = usize::from(face & 3) + 2;
        game.bet_y = usize::from(face >> 2) + 2;
    }
    runtime_shell.pending_audio.clear();
    runtime_shell.transient_audio_playing = true;
    flip_visible_card(&mut runtime_shell).expect("begin revealing first card");
    assert_eq!(
        runtime_shell.visible_card_flip.as_ref().unwrap().animation,
        VisibleCardFlipAnimation::WaitBeforeReveal
    );
    advance_visible_card_flip_animation(&mut runtime_shell)
        .expect("prior cursor sound blocks choose-card cue");
    assert!(runtime_shell.pending_audio.is_empty());
    runtime_shell.transient_audio_playing = false;
    advance_visible_card_flip_animation(&mut runtime_shell)
        .expect("queue choose-card cue after prior sound");
    assert_eq!(
        runtime_shell.visible_card_flip.as_ref().unwrap().animation,
        VisibleCardFlipAnimation::WaitReveal
    );
    assert_eq!(runtime_shell.pending_audio.len(), 1);
    assert_eq!(runtime_shell.pending_audio[0].audio_id, "SFX_CHOOSE_A_CARD");
    advance_visible_card_flip_animation(&mut runtime_shell)
        .expect("queued choose-card cue blocks reveal");
    assert_eq!(
        runtime_shell.visible_card_flip.as_ref().unwrap().animation,
        VisibleCardFlipAnimation::WaitReveal
    );
    runtime_shell.pending_audio.clear();
    runtime_shell.transient_audio_playing = true;
    advance_visible_card_flip_animation(&mut runtime_shell)
        .expect("playing choose-card cue blocks reveal");
    assert_eq!(
        runtime_shell.visible_card_flip.as_ref().unwrap().animation,
        VisibleCardFlipAnimation::WaitReveal
    );
    runtime_shell.transient_audio_playing = false;
    advance_visible_card_flip_animation(&mut runtime_shell)
        .expect("choose-card completion reveals and tabulates");
    let game = runtime_shell
        .visible_card_flip
        .as_ref()
        .expect("reveal remains visible");
    assert_eq!(game.phase, VisibleCardFlipPhase::Result);
    assert_eq!(game.payout, 72);
    assert_eq!(game.coins, 96, "reveal precedes the source payout loop");
    assert_eq!(
        game.animation,
        VisibleCardFlipAnimation::WaitResult { payout: 72 }
    );
    assert_eq!(runtime_shell.pending_audio.len(), 1);
    assert_eq!(runtime_shell.pending_audio[0].audio_id, "SFX_2ND_PLACE");
    advance_visible_card_flip_animation(&mut runtime_shell)
        .expect("queued result cue blocks payout");
    assert_eq!(
        runtime_shell.visible_card_flip.as_ref().unwrap().animation,
        VisibleCardFlipAnimation::WaitResult { payout: 72 }
    );
    runtime_shell.pending_audio.clear();
    runtime_shell.transient_audio_playing = true;
    advance_visible_card_flip_animation(&mut runtime_shell)
        .expect("playing result cue blocks payout");
    runtime_shell.transient_audio_playing = false;
    advance_visible_card_flip_animation(&mut runtime_shell)
        .expect("result completion starts payout");
    assert_eq!(
        runtime_shell.visible_card_flip.as_ref().unwrap().animation,
        VisibleCardFlipAnimation::Payout {
            remaining: 72,
            frames_until_coin: 0,
        }
    );

    for frame in 1_u16..=145 {
        advance_visible_card_flip_animation(&mut runtime_shell).expect("advance payout frame");
        let expected_paid = ((frame + 1) / 2).min(72);
        assert_eq!(
            runtime_shell.visible_card_flip.as_ref().unwrap().coins,
            96 + expected_paid,
            "CardFlip_Payout pays one coin every two frames"
        );
    }
    let game = runtime_shell.visible_card_flip.as_ref().unwrap();
    assert_eq!(game.animation, VisibleCardFlipAnimation::AwaitResult);
    assert_eq!(game.coins, 168);
    assert_eq!(runtime_shell.shell.session().state().coins, 168);

    flip_visible_card(&mut runtime_shell).expect("acknowledge result");
    let game = runtime_shell.visible_card_flip.as_ref().unwrap();
    assert_eq!(game.phase, VisibleCardFlipPhase::PlayAgain);
    assert_eq!(game.message, "WANT TO PLAY\nAGAIN?");
    let card_flip_play_again = runtime_shell
        .shell
        .session()
        .state()
        .script_runtime
        .card_flip
        .clone()
        .expect("typed Card Flip play-again state");

    runtime_shell.visible_card_flip.as_mut().unwrap().round = 11;
    {
        let core_game = runtime_shell
            .shell
            .session_mut()
            .state_mut()
            .script_runtime
            .card_flip
            .as_mut()
            .expect("typed Card Flip state");
        core_game.num_cards_played = 11;
        core_game.discard_pile.fill(false);
        let face = usize::from(core_game.face_up_card.expect("revealed face"));
        core_game.discard_pile[face] = true;
        for index in 0..24 {
            if core_game.discard_pile.iter().filter(|flag| **flag).count() == 12 {
                break;
            }
            core_game.discard_pile[index] = true;
        }
    }
    flip_visible_card(&mut runtime_shell).expect("accept twelfth-round replay");
    let game = runtime_shell.visible_card_flip.as_ref().unwrap();
    assert_eq!(game.phase, VisibleCardFlipPhase::Shuffled);
    assert_eq!(game.message, "THE CARDS HAVE\nBEEN SHUFFLED.");
    assert_eq!(game.coins, 168, "reshuffling precedes the next stake");
    assert!(game.revealed.iter().all(|revealed| !revealed));

    flip_visible_card(&mut runtime_shell).expect("acknowledge reshuffle");
    let game = runtime_shell.visible_card_flip.as_ref().unwrap();
    assert_eq!(game.phase, VisibleCardFlipPhase::ChooseCard);
    assert_eq!(game.animation, VisibleCardFlipAnimation::WaitStake);
    assert_eq!(
        game.coins, 165,
        "the stake follows reshuffle acknowledgement"
    );

    runtime_shell.pending_audio.clear();
    runtime_shell.transient_audio_playing = true;
    {
        let game = runtime_shell.visible_card_flip.as_mut().unwrap();
        game.phase = VisibleCardFlipPhase::PlayAgain;
        game.animation = VisibleCardFlipAnimation::None;
        game.yes_no_index = 1;
    }
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .script_runtime
        .card_flip = Some(card_flip_play_again);
    flip_visible_card(&mut runtime_shell).expect("decline another game");
    assert_eq!(
        runtime_shell.visible_card_flip.as_ref().unwrap().animation,
        VisibleCardFlipAnimation::QuitWaitBefore
    );
    advance_visible_card_flip_animation(&mut runtime_shell)
        .expect("active sound blocks Card Flip quit cue");
    assert!(runtime_shell.pending_audio.is_empty());
    runtime_shell.transient_audio_playing = false;
    advance_visible_card_flip_animation(&mut runtime_shell).expect("queue Card Flip quit cue");
    assert_eq!(runtime_shell.pending_audio.len(), 1);
    assert_eq!(runtime_shell.pending_audio[0].audio_id, "SFX_QUIT_SLOTS");
    assert_eq!(
        runtime_shell.visible_card_flip.as_ref().unwrap().animation,
        VisibleCardFlipAnimation::QuitWaitAfter
    );
    advance_visible_card_flip_animation(&mut runtime_shell)
        .expect("queued Card Flip quit cue blocks close");
    assert!(runtime_shell.visible_card_flip.is_some());
    runtime_shell.pending_audio.clear();
    runtime_shell.transient_audio_playing = true;
    advance_visible_card_flip_animation(&mut runtime_shell)
        .expect("playing Card Flip quit cue blocks close");
    assert!(runtime_shell.visible_card_flip.is_some());
    runtime_shell.transient_audio_playing = false;
    advance_visible_card_flip_animation(&mut runtime_shell)
        .expect("Card Flip closes after quit cue");
    assert!(runtime_shell.visible_card_flip.is_none());
}

#[test]
fn visible_slot_machine_pays_one_coin_every_other_frame_after_result_sound() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let coin_case = runtime_shell.shell.runtime().data().items["COIN_CASE"].clone();
    runtime_shell.shell.session_mut().state_mut().coins = 99;
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .bag
        .add_item(&coin_case, 1)
        .expect("add Coin Case");
    runtime_shell.shell.session_mut().state_mut().random_state =
        crystal_core::random::CrystalRandomState::default();
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .script_runtime
        .script_value = None;
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .script_runtime
        .slot_machine = Some(SlotMachineState {
        phase: SlotMachinePhase::Betting,
        lucky: false,
        keep_seven_bias_chance: false,
        bet: 3,
        bias: None,
        offsets: [14; 3],
        next_reel: 1,
        matched_symbol: None,
        payout_remaining: 0,
    });
    *runtime_shell.shell.session_mut().divider_mut_for_tests() = crystal_core::random::RuntimeDividerSource::replay(
        std::iter::repeat_n([0_u8, 255_u8], 64).flatten(),
    );
    runtime_shell.visible_slot_machine = Some(VisibleSlotMachine {
        phase: VisibleSlotMachinePhase::Betting,
        animation: VisibleSlotMachineAnimation::None,
        yes_no_index: 0,
        bet: 3,
        coins: 99,
        payout: 0,
        offsets: [14; 3],
        spin_ticks: [0; 3],
        spinning: [false; 3],
        next_reel: 1,
        actor: None,
        secondary_actor: None,
        background_y_offset: 0,
        windows: [
            ["CHERRY".into(), "SEVEN".into(), "SQUIRTLE".into()],
            ["PIKACHU".into(), "SEVEN".into(), "STARYU".into()],
            ["PIKACHU".into(), "SEVEN".into(), "PIKACHU".into()],
        ],
        message: "BET 3".to_string(),
    });

    spin_visible_slot_machine(&mut runtime_shell).expect("start deterministic winning spin");
    let machine = runtime_shell.visible_slot_machine.as_ref().unwrap();
    assert_eq!(machine.phase, VisibleSlotMachinePhase::Spinning);
    assert_eq!(
        machine.coins, 96,
        "starting the reels commits only the stake"
    );
    assert!(matches!(
        machine.animation,
        VisibleSlotMachineAnimation::Spinning {
            start_delay: 32,
            ..
        }
    ));
    assert_eq!(runtime_shell.shell.session().state().coins, 96);

    for _ in 0..32 {
        advance_visible_slot_machine_animation(&mut runtime_shell)
            .expect("advance mandatory start delay");
    }
    for reel in 1_u8..=3 {
        let machine = runtime_shell.visible_slot_machine.as_mut().unwrap();
        machine.offsets[usize::from(reel - 1)] = 14;
        machine.spin_ticks[usize::from(reel - 1)] = 0;
        machine.windows = visible_slot_windows(machine.offsets);
        spin_visible_slot_machine(&mut runtime_shell).expect("press A to stop reel");
        for _ in 0..2000 {
            advance_visible_slot_machine_animation(&mut runtime_shell)
                .expect("advance staged reel stop");
            if runtime_shell
                .visible_slot_machine
                .as_ref()
                .unwrap()
                .next_reel
                > reel
            {
                break;
            }
        }
        assert_eq!(
            runtime_shell
                .visible_slot_machine
                .as_ref()
                .unwrap()
                .next_reel,
            reel + 1,
            "each A press stops exactly one reel"
        );
    }
    assert!(matches!(
        runtime_shell
            .visible_slot_machine
            .as_ref()
            .unwrap()
            .animation,
        VisibleSlotMachineAnimation::FlashResult {
            frames_remaining: 16
        }
    ));
    runtime_shell.pending_audio.clear();
    runtime_shell.transient_audio_playing = false;
    for _ in 0..16 {
        advance_visible_slot_machine_animation(&mut runtime_shell)
            .expect("advance exact win flash");
    }
    let machine = runtime_shell.visible_slot_machine.as_ref().unwrap();
    assert_eq!(machine.phase, VisibleSlotMachinePhase::Result);
    assert_eq!(machine.payout, 300);
    assert_eq!(runtime_shell.pending_audio.len(), 1);
    assert_eq!(runtime_shell.pending_audio[0].audio_id, "SFX_2ND_PLACE");
    assert_eq!(
        runtime_shell
            .visible_slot_machine
            .as_ref()
            .unwrap()
            .animation,
        VisibleSlotMachineAnimation::WaitResult { payout: 300 }
    );
    runtime_shell.pending_audio.clear();
    advance_visible_slot_machine_animation(&mut runtime_shell)
        .expect("result cue completion starts payout");
    assert_eq!(
        runtime_shell
            .visible_slot_machine
            .as_ref()
            .unwrap()
            .animation,
        VisibleSlotMachineAnimation::Payout {
            remaining: 300,
            frames_until_coin: 1,
            delay_counter: 0,
        }
    );

    for frame in 1_u16..=600 {
        runtime_shell.pending_audio.clear();
        advance_visible_slot_machine_animation(&mut runtime_shell)
            .expect("advance slot payout frame");
        let paid = frame / 2;
        assert_eq!(
            runtime_shell.visible_slot_machine.as_ref().unwrap().coins,
            96 + paid,
            "SlotsAction_PayoutAnim pays only on alternate frames"
        );
        let should_sound = frame % 2 == 0 && paid % 4 != 0;
        assert_eq!(
            runtime_shell
                .pending_audio
                .iter()
                .any(|command| command.audio_id == "SFX_GET_COIN_FROM_SLOTS"),
            should_sound,
            "the source suppresses every fourth overlapping payout cue"
        );
    }
    runtime_shell.pending_audio.clear();
    advance_visible_slot_machine_animation(&mut runtime_shell)
        .expect("first terminal payout delay frame");
    assert!(matches!(
        runtime_shell
            .visible_slot_machine
            .as_ref()
            .unwrap()
            .animation,
        VisibleSlotMachineAnimation::Payout { remaining: 0, .. }
    ));
    advance_visible_slot_machine_animation(&mut runtime_shell)
        .expect("finish terminal payout delay");
    assert_eq!(
        runtime_shell
            .visible_slot_machine
            .as_ref()
            .unwrap()
            .animation,
        VisibleSlotMachineAnimation::AwaitResult
    );
    assert_eq!(runtime_shell.shell.session().state().coins, 396);

    spin_visible_slot_machine(&mut runtime_shell).expect("acknowledge slot result");
    let machine = runtime_shell.visible_slot_machine.as_ref().unwrap();
    assert_eq!(machine.phase, VisibleSlotMachinePhase::PlayAgain);
    assert_eq!(machine.message, "PLAY AGAIN?");

    runtime_shell.pending_audio.clear();
    runtime_shell.transient_audio_playing = true;
    runtime_shell
        .visible_slot_machine
        .as_mut()
        .unwrap()
        .yes_no_index = 1;
    spin_visible_slot_machine(&mut runtime_shell).expect("decline another slot round");
    assert_eq!(
        runtime_shell
            .visible_slot_machine
            .as_ref()
            .unwrap()
            .animation,
        VisibleSlotMachineAnimation::QuitWaitBefore
    );
    advance_visible_slot_machine_animation(&mut runtime_shell)
        .expect("active sound blocks slot quit cue");
    assert!(runtime_shell.pending_audio.is_empty());
    runtime_shell.transient_audio_playing = false;
    advance_visible_slot_machine_animation(&mut runtime_shell).expect("queue slot quit cue");
    assert_eq!(runtime_shell.pending_audio.len(), 1);
    assert_eq!(runtime_shell.pending_audio[0].audio_id, "SFX_QUIT_SLOTS");
    assert_eq!(
        runtime_shell
            .visible_slot_machine
            .as_ref()
            .unwrap()
            .animation,
        VisibleSlotMachineAnimation::QuitWaitAfter
    );
    advance_visible_slot_machine_animation(&mut runtime_shell)
        .expect("queued slot quit cue blocks close");
    assert!(runtime_shell.visible_slot_machine.is_some());
    runtime_shell.pending_audio.clear();
    runtime_shell.transient_audio_playing = true;
    advance_visible_slot_machine_animation(&mut runtime_shell)
        .expect("playing slot quit cue blocks close");
    assert!(runtime_shell.visible_slot_machine.is_some());
    runtime_shell.transient_audio_playing = false;
    advance_visible_slot_machine_animation(&mut runtime_shell).expect("slot closes after quit cue");
    assert!(runtime_shell.visible_slot_machine.is_none());
}

#[test]
fn visible_slot_machine_runs_exact_golem_and_chansey_phase_counts() {
    fn machine(animation: VisibleSlotMachineAnimation) -> VisibleSlotMachine {
        let offsets = [14; 3];
        VisibleSlotMachine {
            phase: VisibleSlotMachinePhase::Spinning,
            animation,
            yes_no_index: 0,
            bet: 3,
            coins: 99,
            payout: 0,
            offsets,
            spin_ticks: [0; 3],
            spinning: [false, false, true],
            next_reel: 3,
            actor: None,
            secondary_actor: None,
            background_y_offset: 0,
            windows: visible_slot_windows(offsets),
            message: "SPINNING".to_string(),
        }
    }

    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.visible_slot_machine = Some(machine(VisibleSlotMachineAnimation::Golem {
        target: 0,
        remaining: 1,
        phase: VisibleSlotGolemPhase::Init,
        phase_frame: 0,
    }));
    advance_visible_slot_machine_animation(&mut runtime_shell).expect("initialize Golem fall");
    assert!(matches!(
        runtime_shell.visible_slot_machine.as_ref().unwrap().actor,
        Some(VisibleSlotActor::Golem { x: 96, .. })
    ));
    for _ in 0..16 {
        advance_visible_slot_machine_animation(&mut runtime_shell).expect("advance Golem fall");
    }
    assert!(matches!(
        runtime_shell
            .visible_slot_machine
            .as_ref()
            .unwrap()
            .animation,
        VisibleSlotMachineAnimation::Golem {
            phase: VisibleSlotGolemPhase::Fall,
            phase_frame: 31,
            ..
        }
    ));
    runtime_shell.pending_audio.clear();
    advance_visible_slot_machine_animation(&mut runtime_shell).expect("land Golem");
    assert!(
        runtime_shell
            .pending_audio
            .iter()
            .any(|command| command.audio_id == "SFX_PLACE_PUZZLE_PIECE_DOWN")
    );
    advance_visible_slot_machine_animation(&mut runtime_shell).expect("start Golem roll");
    assert_eq!(
        runtime_shell
            .visible_slot_machine
            .as_ref()
            .unwrap()
            .background_y_offset,
        -2
    );
    assert!(matches!(
        runtime_shell.visible_slot_machine.as_ref().unwrap().actor,
        Some(VisibleSlotActor::Golem { x: 98, .. })
    ));
    advance_visible_slot_machine_animation(&mut runtime_shell).expect("drop reel with Golem");
    assert_eq!(
        runtime_shell.visible_slot_machine.as_ref().unwrap().offsets[2],
        0,
        "rate 8 advances reel three once on the second roll frame"
    );
    for _ in 2..=35 {
        advance_visible_slot_machine_animation(&mut runtime_shell).expect("finish Golem roll");
    }
    advance_visible_slot_machine_animation(&mut runtime_shell).expect("restart Golem actor");
    assert_eq!(
        runtime_shell
            .visible_slot_machine
            .as_ref()
            .unwrap()
            .background_y_offset,
        0
    );
    assert!(matches!(
        runtime_shell.visible_slot_machine.as_ref().unwrap().actor,
        Some(VisibleSlotActor::Golem { x: 170, .. })
    ));
    advance_visible_slot_machine_animation(&mut runtime_shell).expect("delete final Golem");
    assert!(matches!(
        runtime_shell
            .visible_slot_machine
            .as_ref()
            .unwrap()
            .animation,
        VisibleSlotMachineAnimation::Stopping { reel: 3, .. }
    ));

    runtime_shell.visible_slot_machine = Some(machine(VisibleSlotMachineAnimation::Chansey {
        target: 1,
        remaining_eggs: 1,
        phase: VisibleSlotChanseyPhase::Walk,
        phase_frame: 0,
    }));
    runtime_shell.visible_slot_machine.as_mut().unwrap().actor = Some(VisibleSlotActor::Chansey {
        x: 96,
        frame: 0,
        frame_tick: 0,
        finishing: false,
    });
    for _ in 0..9 {
        advance_visible_slot_machine_animation(&mut runtime_shell).expect("walk Chansey");
    }
    assert!(matches!(
        runtime_shell
            .visible_slot_machine
            .as_ref()
            .unwrap()
            .animation,
        VisibleSlotMachineAnimation::Chansey {
            phase: VisibleSlotChanseyPhase::PrepareEgg,
            ..
        }
    ));
    assert!(matches!(
        runtime_shell.visible_slot_machine.as_ref().unwrap().actor,
        Some(VisibleSlotActor::Chansey { x: 105, .. })
    ));
    for _ in 0..9 {
        advance_visible_slot_machine_animation(&mut runtime_shell).expect("prepare Chansey egg");
    }
    assert!(matches!(
        runtime_shell
            .visible_slot_machine
            .as_ref()
            .unwrap()
            .secondary_actor,
        Some(VisibleSlotActor::Egg { x: 96, .. })
    ));
    for _ in 0..50 {
        advance_visible_slot_machine_animation(&mut runtime_shell).expect("arc Chansey egg");
    }
    assert!(matches!(
        runtime_shell
            .visible_slot_machine
            .as_ref()
            .unwrap()
            .animation,
        VisibleSlotMachineAnimation::Chansey {
            phase: VisibleSlotChanseyPhase::DropReel,
            ..
        }
    ));
    for _ in 0..17 {
        advance_visible_slot_machine_animation(&mut runtime_shell).expect("drop reel 17 symbols");
    }
    assert_eq!(
        runtime_shell.visible_slot_machine.as_ref().unwrap().offsets[2],
        1
    );
    assert!(matches!(
        runtime_shell
            .visible_slot_machine
            .as_ref()
            .unwrap()
            .animation,
        VisibleSlotMachineAnimation::Chansey {
            phase: VisibleSlotChanseyPhase::CheckMatch,
            ..
        }
    ));
    advance_visible_slot_machine_animation(&mut runtime_shell).expect("finish Chansey routine");
    assert!(matches!(
        runtime_shell
            .visible_slot_machine
            .as_ref()
            .unwrap()
            .animation,
        VisibleSlotMachineAnimation::Stopping { reel: 3, .. }
    ));
}

#[test]
fn slot_machine_renderer_composites_source_oam_actors_and_scy_shake() {
    let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let asset_root = AssetRoot::new(repository_root);
    let sources = load_slot_machine_render_sources(&asset_root).expect("load slot source art");
    let offsets = [14_usize; 3];
    let mut machine = VisibleSlotMachine {
        phase: VisibleSlotMachinePhase::Spinning,
        animation: VisibleSlotMachineAnimation::None,
        yes_no_index: 0,
        bet: 3,
        coins: 99,
        payout: 0,
        offsets,
        spin_ticks: [0; 3],
        spinning: [false; 3],
        next_reel: 3,
        actor: None,
        secondary_actor: None,
        background_y_offset: 0,
        windows: visible_slot_windows(offsets),
        message: "SPINNING".to_string(),
    };
    let mut images = Assets::<Image>::default();
    let plain = render_visible_slot_machine_frame(&sources, &machine, &mut images)
        .expect("render plain slots");
    let plain = images.get(&plain.handle).unwrap().data.clone();

    machine.actor = Some(VisibleSlotActor::Golem {
        x: 96,
        y_offset: 0,
        frame: 0,
        frame_tick: 0,
        flip_x: false,
        flip_y: false,
    });
    let golem = render_visible_slot_machine_frame(&sources, &machine, &mut images)
        .expect("render Golem OAM");
    let golem = images.get(&golem.handle).unwrap().data.clone();
    assert_ne!(plain, golem, "slots_3 Golem pixels must be composited");

    machine.actor = Some(VisibleSlotActor::Chansey {
        x: 105,
        frame: 2,
        frame_tick: 0,
        finishing: true,
    });
    let chansey = render_visible_slot_machine_frame(&sources, &machine, &mut images)
        .expect("render Chansey OAM");
    let chansey = images.get(&chansey.handle).unwrap().data.clone();
    assert_ne!(plain, chansey, "slots_3 Chansey pixels must be composited");
    assert_ne!(golem, chansey);

    machine.actor = None;
    machine.secondary_actor = Some(VisibleSlotActor::Egg {
        x: 112,
        y_offset: -16,
    });
    let egg =
        render_visible_slot_machine_frame(&sources, &machine, &mut images).expect("render Egg OAM");
    let egg = images.get(&egg.handle).unwrap().data.clone();
    assert_ne!(plain, egg, "slots_3 Egg pixels must be composited");

    machine.secondary_actor = None;
    machine.background_y_offset = -2;
    let shaken = render_visible_slot_machine_frame(&sources, &machine, &mut images)
        .expect("render Golem SCY shake");
    let shaken = images.get(&shaken.handle).unwrap().data.clone();
    assert_ne!(
        plain, shaken,
        "Golem roll must move the BG plane through hSCY"
    );
}

#[test]
fn visible_slot_machine_runs_source_ran_out_text_and_sixty_frame_exit() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let offsets = [14; 3];
    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .script_runtime
        .slot_machine = Some(SlotMachineState {
        phase: SlotMachinePhase::Result,
        lucky: false,
        keep_seven_bias_chance: false,
        bet: 1,
        bias: None,
        offsets: offsets.map(|offset| u8::try_from(offset).expect("slot offset fits byte")),
        next_reel: 4,
        matched_symbol: None,
        payout_remaining: 0,
    });
    runtime_shell.shell.session_mut().state_mut().coins = 0;
    runtime_shell.visible_slot_machine = Some(VisibleSlotMachine {
        phase: VisibleSlotMachinePhase::Result,
        animation: VisibleSlotMachineAnimation::AwaitResult,
        yes_no_index: 0,
        bet: 1,
        coins: 0,
        payout: 0,
        offsets,
        spin_ticks: [0; 3],
        spinning: [false; 3],
        next_reel: 4,
        actor: None,
        secondary_actor: None,
        background_y_offset: 0,
        windows: visible_slot_windows(offsets),
        message: "DARN".to_string(),
    });

    spin_visible_slot_machine(&mut runtime_shell).expect("acknowledge losing result");
    let machine = runtime_shell.visible_slot_machine.as_ref().unwrap();
    assert_eq!(machine.phase, VisibleSlotMachinePhase::RanOut);
    assert_eq!(machine.message, "DARN… RAN OUT OF\nCOINS…");
    assert_eq!(machine.animation, VisibleSlotMachineAnimation::None);

    spin_visible_slot_machine(&mut runtime_shell).expect("acknowledge ran-out text");
    assert_eq!(
        runtime_shell
            .visible_slot_machine
            .as_ref()
            .unwrap()
            .animation,
        VisibleSlotMachineAnimation::RanOutDelay {
            frames_remaining: 60
        }
    );
    for frame in 1..=59 {
        advance_visible_slot_machine_animation(&mut runtime_shell)
            .expect("advance ran-out delay frame");
        assert!(matches!(
            runtime_shell.visible_slot_machine.as_ref().unwrap().animation,
            VisibleSlotMachineAnimation::RanOutDelay { frames_remaining }
                if frames_remaining == 60 - frame
        ));
    }
    advance_visible_slot_machine_animation(&mut runtime_shell).expect("finish ran-out delay");
    assert_eq!(
        runtime_shell
            .visible_slot_machine
            .as_ref()
            .unwrap()
            .animation,
        VisibleSlotMachineAnimation::QuitWaitBefore
    );
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

#[test]
fn malformed_script_snapshot_cannot_fall_through_b_to_overworld_input() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    {
        let state = runtime_shell.shell.session_mut().state_mut();
        state.script_runtime.text_window_open = true;
        state.script_runtime.active_text_label = Some("MISSING_COMPILED_TEXT_LABEL".to_string());
        state.script_runtime.pending_text_label = Some("MISSING_COMPILED_TEXT_LABEL".to_string());
    }
    mark_runtime_snapshot_dirty(&mut runtime_shell);
    assert!(runtime_shell.shell.presentation_snapshot().is_err());

    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::KeyX);
    apply_visible_runtime_controls(&keys, &mut runtime_shell, false);

    assert!(
        runtime_shell
            .last_error
            .as_deref()
            .is_some_and(|error| error.contains("MISSING_COMPILED_TEXT_LABEL")),
        "a malformed script surface must stop B routing at the visible error boundary: {:?}",
        runtime_shell.last_error
    );

    for (button, ownership) in [
        (
            "Start",
            has_visible_shell_start_action as fn(&mut BevyRuntimeShell) -> bool,
        ),
        ("Select", has_visible_shell_select_action),
        ("direction", has_visible_shell_direction_action),
    ] {
        let owned = ownership(&mut runtime_shell);
        assert!(owned, "malformed snapshot must fail closed for {button}");
        assert!(
            runtime_shell
                .last_error
                .as_deref()
                .is_some_and(|error| error.contains("MISSING_COMPILED_TEXT_LABEL")),
            "{button} ownership must retain the snapshot error"
        );
        runtime_shell.last_error = None;
    }
    assert!(has_visible_shell_a_action(&mut runtime_shell).is_err());
    assert!(visible_field_shortcut_allowed(&runtime_shell).is_err());
}

#[test]
fn start_menu_labels_match_asm_glyph_entries() {
    assert_eq!(start_menu_option_label(StartMenuOption::Pokedex), "#DEX");
    assert_eq!(start_menu_option_label(StartMenuOption::Pokemon), "#MON");
    assert_eq!(start_menu_option_label(StartMenuOption::Pack), "PACK");
    assert_eq!(start_menu_option_label(StartMenuOption::Pokegear), "<POKE>GEAR");
    let glyphs = bitmap_font_char_map();
    let gear = normalize_bitmap_font_text(start_menu_option_label(StartMenuOption::Pokegear));
    assert_eq!(gear.chars().map(|ch| glyphs[&ch]).collect::<Vec<_>>(),
        [0x70, 0x71, 0x86, 0x84, 0x80, 0x91]);
    assert_eq!(
        start_menu_option_label(StartMenuOption::TrainerCard),
        "STATUS"
    );
    assert_eq!(start_menu_option_label(StartMenuOption::Save), "SAVE");
    assert_eq!(start_menu_option_label(StartMenuOption::Options), "OPTION");
    assert_eq!(start_menu_option_label(StartMenuOption::Exit), "EXIT");
}

#[test]
fn start_menu_field_command_renders_bitmap_glyph_sprites() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let asset_root = AssetRoot::new(repo_root);
    let runtime = CrystalRuntime::load_from_compiled_pack(
        &asset_root,
        "content-packs/core-modular.crystalpack",
    )
    .expect("load compiled pack");
    let spawn_identifier = runtime
        .title_new_game_spawn_identifier()
        .expect("title new-game spawn");
    let mut runtime_shell = initialize_bevy_runtime_shell(
        asset_root,
        runtime,
        BevyShellStart::NewGame { spawn_identifier },
        BevyShellConfig {
            smoke_player_name: Some("AB".to_string()),
            ..Default::default()
        },
    )
    .expect("initialize overworld shell");
    complete_visible_smoke_player_name_if_needed(&mut runtime_shell, Some("AB"))
        .expect("complete player name and run arrival callbacks");
    settle_visible_shell_smoke_until_idle(&mut runtime_shell)
        .expect("settle arrival scripts before opening the start menu");
    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::Start])
        .expect("open start menu");

    let mut app = App::new();
    app.insert_resource(runtime_shell)
        .insert_resource(RenderedViewport::default())
        .insert_resource(RenderedTilesetArt::default())
        .init_resource::<Assets<Image>>()
        .add_systems(Update, render_playfield);
    app.update();

    let world = app.world_mut();
    let mut field_command_entities = world.query_filtered::<Entity, With<FieldCommandMarker>>();
    assert!(
        field_command_entities.iter(world).count() > 2,
        "start menu field-command surface must include bitmap glyph sprites, not only panel sprites"
    );
    let rendered_art = world.resource::<RenderedTilesetArt>();
    assert!(
        rendered_art.font_cache.is_some(),
        "start menu render must load bitmap font art"
    );
    assert_eq!(rendered_art.font_error, None);
}

#[test]
fn arrow_key_mapping_moves_overworld_player_when_held() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let asset_root = AssetRoot::new(repo_root);
    let runtime = workspace_desktop_runtime(&asset_root);
    let spawn_identifier = runtime
        .title_new_game_spawn_identifier()
        .expect("title new-game spawn");
    let mut runtime_shell = initialize_bevy_runtime_shell(
        asset_root,
        runtime,
        BevyShellStart::NewGame { spawn_identifier },
        BevyShellConfig {
            smoke_player_name: Some("AB".to_string()),
            ..Default::default()
        },
    )
    .expect("initialize overworld shell");
    complete_visible_smoke_player_name_if_needed(&mut runtime_shell, Some("AB"))
        .expect("complete player name and run arrival callbacks");
    settle_visible_shell_smoke_until_idle(&mut runtime_shell)
        .expect("settle arrival scripts before moving");
    let routing_snapshot = runtime_shell
        .shell
        .snapshot()
        .expect("snapshot settled movement routing");
    let shell_routes_direction = has_visible_shell_direction_action(&mut runtime_shell);
    assert!(
        !shell_routes_direction,
        "settled overworld direction must route to joypad; cursor={:?} start={:?} special={:?} events={:?}",
        runtime_shell.active_script_cursor,
        runtime_shell.start_menu_cursor,
        runtime_shell.special_boundary,
        routing_snapshot.script_events
    );
    let start_tile = runtime_shell
        .shell
        .snapshot()
        .expect("snapshot before movement")
        .overworld
        .tile;

    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::ArrowRight);
    for _ in 0..6 {
        let buttons = collect_overworld_keyboard_buttons(&keys, false, false, false, false, false);
        assert_eq!(buttons, vec![GameButton::Right]);
        apply_visible_shell_smoke_frame(&mut runtime_shell, &buttons)
            .expect("ArrowRight frame advances overworld input");
    }

    let end_tile = runtime_shell
        .shell
        .snapshot()
        .expect("snapshot after movement")
        .overworld
        .tile;
    assert!(
        end_tile.x > start_tile.x,
        "held ArrowRight should move player right from {start_tile:?} to {end_tile:?}"
    );
}

#[test]
fn live_runtime_hotkeys_move_overworld_player_when_arrow_key_is_held() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let asset_root = AssetRoot::new(repo_root);
    let runtime = workspace_desktop_runtime(&asset_root);
    let spawn_identifier = runtime
        .title_new_game_spawn_identifier()
        .expect("title new-game spawn");
    let mut runtime_shell = initialize_bevy_runtime_shell(
        asset_root,
        runtime,
        BevyShellStart::NewGame { spawn_identifier },
        BevyShellConfig::default(),
    )
    .expect("initialize overworld shell");
    complete_visible_smoke_player_name_if_needed(&mut runtime_shell, Some("AB"))
        .expect("complete player name and run arrival callbacks");
    settle_visible_shell_smoke_until_idle(&mut runtime_shell)
        .expect("settle arrival scripts before moving");
    let routing_snapshot = runtime_shell
        .shell
        .snapshot()
        .expect("snapshot settled live movement routing");
    let shell_routes_direction = has_visible_shell_direction_action(&mut runtime_shell);
    assert!(
        !shell_routes_direction,
        "settled live direction must route to joypad; cursor={:?} start={:?} special={:?} events={:?}",
        runtime_shell.active_script_cursor,
        runtime_shell.start_menu_cursor,
        runtime_shell.special_boundary,
        routing_snapshot.script_events
    );
    let start_tile = runtime_shell
        .shell
        .snapshot()
        .expect("snapshot before movement")
        .overworld
        .tile;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(runtime_shell)
        .insert_resource(native_rtc_source_for_test())
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(RuntimeTickTimer::new(0.0))
        .insert_resource(HeldArrowRightTestFrames(6))
        .add_systems(
            Update,
            (
                inject_held_arrow_right_for_test,
                apply_keyboard_input,
                apply_runtime_hotkeys,
            )
                .chain(),
        );
    for _ in 0..6 {
        app.update();
    }

    let runtime_shell = app.world().resource::<BevyRuntimeShell>();
    assert!(
        runtime_shell.last_overworld_input.is_some(),
        "held ArrowRight should be recorded as overworld input"
    );
    let end_tile = runtime_shell
        .shell
        .snapshot()
        .expect("snapshot after live movement")
        .overworld
        .tile;
    assert_eq!(runtime_shell.last_error, None);
    assert!(
        end_tile.x > start_tile.x,
        "held ArrowRight through the live hotkey system should move player right from {start_tile:?} to {end_tile:?}; input={:?} status={:?} facing={:?}",
        runtime_shell.last_overworld_input,
        runtime_shell.last_action_status,
        runtime_shell
            .shell
            .snapshot()
            .expect("movement diagnostic snapshot")
            .overworld
            .facing
    );
    assert!(
        runtime_shell.player_walk_frame_ticks > 0,
        "a successful live movement step must hold the walking sprite frame"
    );
}

#[test]
fn arrow_key_dispatch_moves_visible_start_menu_cursor() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let asset_root = AssetRoot::new(repo_root);
    let runtime = CrystalRuntime::load_from_compiled_pack(
        &asset_root,
        "content-packs/core-modular.crystalpack",
    )
    .expect("load compiled pack");
    let spawn_identifier = runtime
        .title_new_game_spawn_identifier()
        .expect("title new-game spawn");
    let mut runtime_shell = initialize_bevy_runtime_shell(
        asset_root,
        runtime,
        BevyShellStart::NewGame { spawn_identifier },
        BevyShellConfig::default(),
    )
    .expect("initialize overworld shell");
    complete_visible_smoke_player_name_if_needed(&mut runtime_shell, Some("AB"))
        .expect("complete player name and run arrival callbacks");
    settle_visible_shell_smoke_until_idle(&mut runtime_shell)
        .expect("settle arrival scripts before opening the start menu");
    apply_visible_shell_smoke_frame(&mut runtime_shell, &[GameButton::Start])
        .expect("open start menu");
    assert_eq!(
        visible_start_menu_entries(&runtime_shell).expect("start menu entries"),
        vec![">PACK", " AB", " SAVE", " OPTION", " EXIT"]
    );

    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::ArrowDown);
    apply_visible_runtime_controls(&keys, &mut runtime_shell, true);

    assert_eq!(
        visible_start_menu_entries(&runtime_shell).expect("moved start menu entries"),
        vec![" PACK", ">AB", " SAVE", " OPTION", " EXIT"]
    );
}

#[test]
fn battle_move_held_direction_repeat_depends_on_credits_h_in_menu_leak() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    runtime_shell.battle_move_cursor = Some(MenuCursor {
        surface_id: "battle:moves".to_string(),
        option_index: 0,
    });

    assert_eq!(runtime_shell.h_in_menu, 0);
    assert!(!visible_ui_direction_can_repeat(&runtime_shell));

    runtime_shell.h_in_menu = 1;
    assert!(
        visible_ui_direction_can_repeat(&runtime_shell),
        "Credits' unrestored hInMenu must enable held movement in the battle move menu"
    );

    runtime_shell.battle_move_cursor = None;
    runtime_shell.h_in_menu = 0;
    assert!(
        visible_ui_direction_can_repeat(&runtime_shell),
        "ordinary menus manage their own repeat behavior"
    );
}

#[test]
fn held_overworld_direction_is_restored_after_warp_navigation_reset() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::ArrowDown);
    sync_overworld_held_directions(&keys, &mut runtime_shell, false);
    keys.clear();

    // A stair/door warp resets presentation navigation while the host still
    // reports the physical key as held. There is no second key-down edge on
    // the destination map.
    reset_visible_navigation_state_preserving_held_directions(&mut runtime_shell);
    assert!(keys.pressed(KeyCode::ArrowDown));
    assert!(!keys.just_pressed(KeyCode::ArrowDown));

    sync_overworld_held_directions(&keys, &mut runtime_shell, false);

    assert_eq!(
        runtime_shell.overworld_held_directions,
        VecDeque::from([GameButton::Down]),
        "a continuous physical hold must resume overworld walking after the warp"
    );
}

#[test]
fn visible_overworld_normal_inputs_walk_through_bedroom_warp() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let asset_root = AssetRoot::new(repo_root);
    let runtime = CrystalRuntime::load_from_compiled_pack(
        &asset_root,
        "content-packs/core-modular.crystalpack",
    )
    .expect("load compiled pack");
    let spawn_identifier = runtime
        .title_new_game_spawn_identifier()
        .expect("title new-game spawn");
    let smoke = smoke_visible_shell_overworld(
        asset_root,
        runtime,
        BevyShellStart::NewGame { spawn_identifier },
        BevyShellConfig {
            smoke_player_name: Some("AB".to_string()),
            ..Default::default()
        },
        &[
            vec![GameButton::Right],
            vec![GameButton::Right],
            vec![GameButton::Right],
            vec![GameButton::Right],
            vec![GameButton::Right],
            vec![GameButton::Right],
            vec![GameButton::Up],
            vec![GameButton::Up],
            vec![GameButton::Up],
            vec![GameButton::Up],
        ],
        None,
    )
    .expect("normal inputs walk through bedroom warp");

    assert_eq!(smoke.start_map, "PlayersHouse2F");
    assert_eq!((smoke.start_tile_x, smoke.start_tile_y), (3, 3));
    assert_eq!(smoke.final_map, "PlayersHouse1F");
    assert_eq!((smoke.final_tile_x, smoke.final_tile_y), (9, 0));
    assert_eq!(
        smoke.final_scene.as_deref(),
        Some("SCENE_PLAYERSHOUSE1F_MEET_MOM")
    );
    assert_eq!(smoke.warps, 1);
    assert_eq!(smoke.active_music.as_deref(), Some("MUSIC_NEW_BARK_TOWN"));
    assert!(smoke.pending_audio > 0);
    assert!(
        smoke
            .frame_events
            .iter()
            .any(|event| event.contains("warp=true"))
    );
}

#[test]
fn pokedex_visible_data_respects_ownership_and_formats_dimensions() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let asset_root = AssetRoot::new(repo_root);
    let runtime = CrystalRuntime::load_from_compiled_pack(
        &asset_root,
        "content-packs/core-modular.crystalpack",
    )
    .expect("load compiled pack");
    let spawn_identifier = runtime
        .title_new_game_spawn_identifier()
        .expect("title new-game spawn");
    let mut runtime_shell = initialize_bevy_runtime_shell(
        asset_root,
        runtime,
        BevyShellStart::NewGame { spawn_identifier },
        BevyShellConfig {
            smoke_player_name: Some("AB".to_string()),
            ..Default::default()
        },
    )
    .expect("initialize overworld shell");
    complete_visible_smoke_player_name_if_needed(&mut runtime_shell, Some("AB"))
        .expect("complete player name and run arrival callbacks");
    settle_visible_shell_smoke_until_idle(&mut runtime_shell)
        .expect("settle arrival scripts before opening the start menu");
    let mut snapshot = runtime_shell.shell.snapshot().expect("snapshot");
    let species = snapshot.pokemon[0].clone();
    snapshot.progression.pokedex_seen_species.clear();
    snapshot.progression.pokedex_caught_species.clear();
    let row = pokedex_entry_row(&snapshot, &species, ">");
    assert!(row.contains("-----"), "unseen species leaked: {row}");
    snapshot
        .progression
        .pokedex_seen_species
        .insert(species.species_id.clone());
    runtime_shell.pokedex_detail_open = true;
    let entry = snapshot
        .presentation
        .pokedex_entries
        .get(&species.species_id)
        .unwrap();
    let description = entry.pages[0].clone();
    let height = format!(
        "{}'{:02}\"",
        entry.height_digits / 100,
        entry.height_digits % 100
    );
    let weight = format!(
        "{}.{:01}lb",
        entry.weight_digits / 10,
        entry.weight_digits % 10
    );
    let seen_rows = visible_pokedex_detail_entries(&snapshot, &runtime_shell).unwrap();
    assert!(
        !seen_rows
            .join(" ")
            .contains(description.split_whitespace().next().unwrap())
    );
    assert!(seen_rows.iter().any(|row| row.contains("???")));
    snapshot
        .progression
        .pokedex_caught_species
        .insert(species.species_id.clone());
    let caught_rows = visible_pokedex_detail_entries(&snapshot, &runtime_shell).unwrap();
    assert!(
        caught_rows.iter().any(|row| row.contains(&height)),
        "{caught_rows:?}"
    );
    assert!(
        caught_rows.iter().any(|row| row.contains(&weight)),
        "{caught_rows:?}"
    );
    assert!(!caught_rows.join(" ").contains("CATCH"));
    let asset_root = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let caught_species = snapshot.progression.pokedex_caught_species.clone();
    for (label, detail, caught) in [
        ("list", false, true),
        ("unknown", false, false),
        ("entry", true, true),
        ("seen", true, false),
        ("options", false, true),
        ("search", false, true),
        ("results", false, true),
        ("area", true, true),
        ("printer", true, true),
        ("unown", false, true),
    ] {
        snapshot.progression.pokedex_caught_species = caught_species.clone();
        runtime_shell.pokedex_controls = VisiblePokedexControls::default();
        if matches!(label, "list" | "entry" | "seen" | "unknown") {
            runtime_shell.pokedex_controls.mode = VisiblePokedexMode::Old;
        }
        match label {
            "options" => runtime_shell.pokedex_controls.option_cursor = Some(0),
            "search" => {
                runtime_shell.pokedex_controls.search_cursor = Some(0);
                runtime_shell.pokedex_controls.search_types = [1, 0];
            }
            "results" => {
                runtime_shell.pokedex_controls.search_results =
                    Some(vec![runtime_shell.pokedex_cursor]);
                runtime_shell.pokedex_controls.search_types = [1, 0];
            }
            "area" => runtime_shell.pokedex_controls.area_region = Some(false),
            "printer" => runtime_shell.pokedex_controls.printer_open = true,
            "unown" => {
                runtime_shell.pokedex_controls.unown_cursor = Some(1);
                runtime_shell
                    .shell
                    .session_mut()
                    .state_mut()
                    .pokedex
                    .unown_letters = vec![1, 26];
            }
            _ => {}
        }
        runtime_shell.pokedex_detail_open = detail;
        snapshot.progression.pokedex_seen_species = snapshot
            .pokemon
            .iter()
            .map(|mon| mon.species_id.clone())
            .collect();
        if label == "unknown" {
            snapshot.progression.pokedex_seen_species.clear();
        }
        if !caught {
            snapshot.progression.pokedex_caught_species.clear();
        }
        snapshot.progression.pokedex_seen = snapshot.progression.pokedex_seen_species.len();
        snapshot.progression.pokedex_owned = snapshot.progression.pokedex_caught_species.len();
        let mut world = World::new();
        let mut queue = bevy::ecs::world::CommandQueue::default();
        let mut images = Assets::<Image>::default();
        let mut art = RenderedTilesetArt::default();
        spawn_field_pokedex_screen(
            &mut Commands::new(&mut queue, &world),
            &snapshot,
            &runtime_shell,
            &mut art,
            &asset_root,
            &mut images,
        )
        .expect("render Pokédex");
        queue.apply(&mut world);
        assert_eq!(art.font_error, None);
        let mut query = world.query::<(&Sprite, &Transform, &Handle<Image>)>();
        let mut sprites = query.iter(&world).collect::<Vec<_>>();
        sprites.sort_by(|a, b| a.1.translation.z.total_cmp(&b.1.translation.z));
        let mut canvas = image::RgbaImage::new(PLAYFIELD_WIDTH as u32, PLAYFIELD_HEIGHT as u32);
        for (sprite, transform, handle) in sprites {
            let source = images.get(handle).expect("sprite image");
            let size = sprite.custom_size.expect("explicit sprite size");
            if source.texture_descriptor.size.width == 56 {
                assert_eq!(
                    size,
                    Vec2::splat(7.0 * TILE_SIZE),
                    "Pokédex front picture must fill its seven-tile canvas"
                );
            }
            let left = transform.translation.x - size.x / 2.0 - PLAYFIELD_LEFT;
            let top = PLAYFIELD_TOP - transform.translation.y - size.y / 2.0;
            assert!(
                left >= -0.01 && left + size.x <= PLAYFIELD_WIDTH + 0.01,
                "{label}: sprite clips horizontally: {left} + {}",
                size.x
            );
            assert!(
                top >= -0.01 && top + size.y <= PLAYFIELD_HEIGHT + 0.01,
                "{label}: sprite clips vertically: {top} + {}",
                size.y
            );
            let raster = image::RgbaImage::from_raw(
                source.texture_descriptor.size.width,
                source.texture_descriptor.size.height,
                source.data.clone(),
            )
            .expect("RGBA sprite");
            let scaled = image::imageops::resize(
                &raster,
                size.x as u32,
                size.y as u32,
                image::imageops::FilterType::Nearest,
            );
            image::imageops::overlay(
                &mut canvas,
                &scaled,
                left.round() as i64,
                top.round() as i64,
            );
        }
        if let Ok(directory) = std::env::var("POKEDEX_RENDER_DIR") {
            std::fs::create_dir_all(&directory).unwrap();
            canvas
                .save(PathBuf::from(directory).join(format!("pokedex-{label}.png")))
                .unwrap();
        }
    }
}

#[test]
fn pokedex_measurements_preserve_inches_and_tenths_of_pounds() {
    let entry = crate::core::models::RuntimePokedexEntry {
        species: "TEST".into(),
        classification: "TEST".into(),
        height_digits: 211,
        weight_digits: 141,
        pages: vec!["Test.".into()],
    };
    assert_eq!(
        pokedex_measurements(&entry, true),
        ("2'11\"".into(), "14.1lb".into())
    );
    assert_eq!(
        pokedex_measurements(&entry, false),
        ("?'??\"".into(), "???lb".into())
    );
}

#[test]
fn standalone_town_map_ship_animation_matches_rom_poses_and_cadence() {
    let mut shell = initialized_town_map_location_shell("FastShip1F");
    activate_visible_special_routine_boundary(&mut shell,
        &SpecialRoutineEffect::OverworldTownMap { map_name: Some("FastShip1F".into()) }).unwrap();
    let records: serde_json::Value = serde_json::from_str(&external_oracle_fixture_text("standalone-town-map-ship-animation.json")).unwrap();
    for (index, row) in records.as_array().unwrap().iter().filter(|row| row["pose"] != 255).enumerate() {
        if index > 0 { advance_visible_pokegear_map_animation(&mut shell, 1); }
        assert_eq!(u64::from(shell.pokegear_map_animation_frame / 9), row["pose"].as_u64().unwrap());
        if index % 9 == 0 && index < 36 {
            capture_town_map_location_frame(&shell, &format!("standalone-ship-pose-{}", index / 9));
        }
    }
}

#[test]
fn standalone_town_map_trainer_animation_matches_rom_poses() {
    let mut shell = initialized_town_map_location_shell("Pokecenter2F");
    shell.shell.session_mut().state_mut().backup_warp_map_name = Some("CeladonDeptStore1F".into());
    mark_runtime_snapshot_dirty(&mut shell);
    activate_visible_special_routine_boundary(&mut shell,
        &SpecialRoutineEffect::OverworldTownMap { map_name: Some("Pokecenter2F".into()) }).unwrap();
    for pose in 0..4 {
        if pose > 0 { advance_visible_pokegear_map_animation(&mut shell, 9); }
        capture_town_map_location_frame(&shell, &format!("standalone-special-kanto-pose-{pose}"));
    }
    advance_visible_pokegear_map_animation(&mut shell, 9);
    assert_eq!(shell.pokegear_map_animation_frame, 0);
    close_visible_pokegear_menu(&mut shell).unwrap();
    advance_visible_pokegear_map_animation(&mut shell, 9);
    assert_eq!(shell.pokegear_map_animation_frame, 0, "closed map must not animate");
}

#[test]
fn standalone_town_map_ignores_overworld_player_palette_override() {
    let mut shell = initialized_town_map_location_shell("Pokecenter2F");
    shell.shell.session_mut().state_mut().backup_warp_map_name = Some("CeladonDeptStore1F".into());
    shell.shell.session_mut().state_mut().player_palette_id = 5;
    mark_runtime_snapshot_dirty(&mut shell);
    activate_visible_special_routine_boundary(&mut shell,
        &SpecialRoutineEffect::OverworldTownMap { map_name: Some("Pokecenter2F".into()) }).unwrap();
    capture_town_map_location_frame(&shell, "standalone-special-kanto");
}

#[test]
fn standalone_town_map_kris_poses_use_source_blue_palette() {
    let mut shell = initialized_town_map_location_shell("Pokecenter2F");
    shell.shell.session_mut().state_mut().backup_warp_map_name = Some("CeladonDeptStore1F".into());
    shell.shell.session_mut().state_mut().player_gender = PLAYER_GENDER_FEMALE;
    // Source map OAM selects PAL_OW_BLUE independently of overworld recoloring.
    shell.shell.session_mut().state_mut().player_palette_id = 5;
    mark_runtime_snapshot_dirty(&mut shell);
    activate_visible_special_routine_boundary(&mut shell,
        &SpecialRoutineEffect::OverworldTownMap { map_name: Some("Pokecenter2F".into()) }).unwrap();
    for pose in 0..4 {
        if pose > 0 { advance_visible_pokegear_map_animation(&mut shell, 9); }
        capture_town_map_location_frame(&shell, &format!("standalone-kris-pose-{pose}"));
    }
}

#[test]
fn npc_sprite_palette_requires_the_requested_source_time_group() {
    let content = format!("; morn\n{}", "RGB 31,31,31, 20,20,20, 10,10,10, 0,0,0\n".repeat(8));
    assert!(parse_npc_sprite_palette_bank(&content, "day").is_err(),
        "a missing day bank must not silently use morning");
    assert_eq!(parse_npc_sprite_palette_bank(&content, "morning").unwrap().len(), 8);
    let day = content.replace("; morn", "; day");
    assert_eq!(parse_npc_sprite_palette_bank(&day, "indoor").unwrap(),
        parse_npc_sprite_palette_bank(&day, "day").unwrap());
}

#[test]
fn pokegear_radio_a_does_not_advance_or_close_the_broadcast() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    open_visible_pokegear_menu(&mut shell).unwrap();
    shell.pokegear_page = PokegearPage::Radio;
    shell.pokegear_radio_station = Some("OAKS_POKEMON_TALK".into());
    load_visible_radio_broadcast(&mut shell).unwrap();
    for _ in 0..6 {
        press_visible_a_button(&mut shell).unwrap();
        assert!(shell.pokegear_menu_open, "PokegearRadio_Joypad does not exit on A");
        assert!(shell.pokegear_radio_broadcast.is_some(),
            "broadcast text is driven by PlayRadioShow, not A presses");
    }
}

#[test]
fn pokegear_radio_furniture_holds_input_then_exits_on_a() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    activate_visible_special_routine_boundary(&mut shell,
        &SpecialRoutineEffect::MapRadio { station: "MAPRADIO_POKEMON_CHANNEL".into() }).unwrap();
    press_visible_a_button(&mut shell).unwrap();
    assert!(shell.pokegear_radio_broadcast.is_some(), "initial hold must ignore A");
    press_visible_b_button(&mut shell).unwrap();
    assert!(shell.pokegear_menu_open, "initial hold must ignore B");
    advance_visible_map_radio_delay(&mut shell, 99);
    press_visible_b_button(&mut shell).unwrap();
    assert!(shell.pokegear_menu_open, "all 100 delay frames belong to PlayRadio");
    advance_visible_map_radio_delay(&mut shell, 1);
    let knob = shell.pokegear_radio_tuning_knob;
    move_visible_pokegear_cursor(&mut shell, -1).unwrap();
    cycle_visible_pokegear_page(&mut shell, -1).unwrap();
    assert_eq!(shell.pokegear_radio_tuning_knob, knob, "furniture radio has no tuning controls");
    assert_eq!(shell.pokegear_page, PokegearPage::Radio, "furniture radio has no cards");
    press_visible_a_button(&mut shell).unwrap();
    assert!(!shell.pokegear_menu_open, "furniture radio stops on A, not its last transcript page");
    assert_eq!(shell.pokegear_map_radio_delay, None);
    shell.pokegear_map_radio_delay = Some(100);
    assert_eq!(advance_visible_map_radio_delay(&mut shell, 150), 50,
        "batched ticks after DelayFrames belong to the broadcast");
    assert_eq!(shell.pokegear_map_radio_delay, Some(0));

}

#[test]
fn pokegear_radio_furniture_entry_renders_source_textbox_without_covering_map() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    activate_visible_special_routine_boundary(&mut shell,
        &SpecialRoutineEffect::MapRadio { station: "MAPRADIO_OAKS_POKEMON_TALK".into() }).unwrap();
    let snapshot = shell.shell.snapshot().unwrap();
    let mut world = World::new();
    let mut queue = bevy::ecs::world::CommandQueue::default();
    let mut images = Assets::<Image>::default();
    let mut art = RenderedTilesetArt::default();
    let mut commands = Commands::new(&mut queue, &world);
    spawn_field_pokegear_screen(&mut commands, &snapshot, &shell, &mut art,
        &shell.asset_root, &mut images).unwrap();
    queue.apply(&mut world);
    assert_eq!(art.font_error, None);
    let canvas = render_pc_audit_canvas(&mut world, &images, "furniture-radio");
    let reference = image::load_from_memory(&external_oracle_fixture_bytes("furniture-radio-oak-entry.png")).unwrap().to_rgba8();
    let scale = canvas.width() / 160;
    for y in 0..144 {
        for x in 0..160 {
            let actual = canvas.get_pixel(x * scale, y * scale).0;
            if y < 96 {
                assert_eq!(actual[3], 0, "radio must preserve map pixel {x},{y}");
            } else {
                assert_eq!(actual.map(|value| value >> 3),
                    reference.get_pixel(x,y).0.map(|value| value >> 3),
                    "source furniture radio pixel {x},{y}");
            }
        }
    }
    if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        canvas.save(PathBuf::from(directory).join("furniture-radio-entry.png")).unwrap();
    }
}

#[test]
fn pokegear_radio_furniture_channel_uses_source_region_and_time() {
    let shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let mut snapshot = shell.shell.snapshot().unwrap();
    use crate::core::world::encounters::TimeOfDay;
    for (map, backup, johto) in [
        ("NewBarkTown", None, true),
        ("FastShip1F", None, true),
        ("VictoryRoad", None, false),
        ("Pokecenter2F", Some("CeladonDeptStore1F"), false),
        // IsInJohto's ship exception precedes SPECIAL resolution.
        ("Pokecenter2F", Some("FastShip1F"), false),
    ] {
        snapshot.overworld.map_name = map.into();
        snapshot.progression.backup_warp_map_name = backup.map(str::to_string);
        for time in [TimeOfDay::Morning, TimeOfDay::Day, TimeOfDay::Night] {
            snapshot.progression.time.time_of_day = time;
            let expected = if !johto { "PLACES_AND_PEOPLE" }
                else if time == TimeOfDay::Morning { "POKEDEX_SHOW" }
                else { "OAKS_POKEMON_TALK" };
            assert_eq!(visible_furniture_radio_station(&snapshot, "MAPRADIO_POKEMON_CHANNEL").unwrap(), expected);
        }
    }
    assert!(visible_furniture_radio_station(&snapshot, "MISSING_RADIO").is_err());
    assert!(visible_radio_station_name("MISSING_RADIO", false).is_err());
}

#[test]
fn pokegear_radio_takeover_preserves_signal_gates_and_station_identity() {
    let shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let mut snapshot = shell.shell.snapshot().unwrap();
    snapshot.progression.active_engine_flags.insert("ENGINE_ROCKETS_IN_RADIO_TOWER".into());
    snapshot.progression.active_engine_flags.insert("ENGINE_ROCKET_SIGNAL_ON_CH20".into());
    snapshot.progression.active_engine_flags.insert("ENGINE_EXPN_CARD".into());
    snapshot.progression.time.time_of_day = crate::core::world::encounters::TimeOfDay::Day;
    for (landmark, handler, expected) in [
        ("LANDMARK_NEW_BARK_TOWN", "PlacesAndPeople", None),
        ("LANDMARK_NEW_BARK_TOWN", "RuinsOfAlphRadio", None),
        ("LANDMARK_NEW_BARK_TOWN", "EvolutionRadio", None),
        ("LANDMARK_NEW_BARK_TOWN", "PKMNTalkAndPokedexShow", Some(("OAKS_POKEMON_TALK", "MUSIC_ROCKET_OVERTURE"))),
        ("LANDMARK_NEW_BARK_TOWN", "BuenasPassword", Some(("BUENAS_PASSWORD", "MUSIC_ROCKET_OVERTURE"))),
        ("LANDMARK_RUINS_OF_ALPH", "RuinsOfAlphRadio", Some(("UNOWN_RADIO", "MUSIC_RUINS_OF_ALPH_RADIO"))),
        ("LANDMARK_MAHOGANY_TOWN", "EvolutionRadio", Some(("EVOLUTION_RADIO", "MUSIC_LAKE_OF_RAGE_ROCKET_RADIO"))),
        ("LANDMARK_CELADON_CITY", "PKMNTalkAndPokedexShow", None),
        ("LANDMARK_CELADON_CITY", "PlacesAndPeople", Some(("PLACES_AND_PEOPLE", "MUSIC_VIRIDIAN_CITY"))),
        ("LANDMARK_CELADON_CITY", "PokeFluteRadio", Some(("POKE_FLUTE_RADIO", "MUSIC_POKE_FLUTE_CHANNEL"))),
        ("LANDMARK_FAST_SHIP", "PKMNTalkAndPokedexShow", Some(("OAKS_POKEMON_TALK", "MUSIC_ROCKET_OVERTURE"))),
    ] {
        std::sync::Arc::make_mut(&mut snapshot.presentation).pokegear_landmarks.map_to_landmark
            .insert(snapshot.overworld.map_name.clone(), landmark.into());
        let actual = visible_pokegear_radio_station(&snapshot, handler).unwrap();
        assert_eq!(actual.as_ref().map(|(station, music)| (*station, music.as_str())), expected,
            "{landmark} {handler}: source channel gate must run before PlayRadioShow takeover");
    }
    // Gear reception uses the resolved ship landmark, but PlayRadioShow's
    // IsInJohto checks raw FAST_SHIP before resolving a SPECIAL map.
    snapshot.overworld.map_name = "Pokecenter2F".into();
    snapshot.progression.backup_warp_map_name = Some("FastShip1F".into());
    assert_eq!(visible_pokegear_radio_station(&snapshot, "PKMNTalkAndPokedexShow").unwrap(),
        Some(("OAKS_POKEMON_TALK", "MUSIC_POKEMON_TALK".into())));
}

#[test]
fn pokegear_radio_station_title_uses_source_loader_name() {
    for (station, name) in [
        ("OAKS_POKEMON_TALK", "OAK's <PK><MN> Talk"),
        ("POKEDEX_SHOW", "#DEX Show"), ("POKEMON_MUSIC", "#MON Music"),
        ("LUCKY_CHANNEL", "Lucky Channel"), ("UNOWN_RADIO", "?????"),
        ("EVOLUTION_RADIO", "?????"), ("POKE_FLUTE_RADIO", "# FLUTE"),
        ("PLACES_AND_PEOPLE", "Places & People"),
        ("LETS_ALL_SING", "Let's All Sing!"), ("ROCKET_RADIO", "Let's All Sing!"),
    ] {
        for rockets in [false, true] {
            assert_eq!(visible_radio_station_name(station, rockets).unwrap(), name);
        }
    }
    assert_eq!(visible_radio_station_name("BUENAS_PASSWORD", false).unwrap(), "");
    assert_eq!(visible_radio_station_name("BUENAS_PASSWORD", true).unwrap(), "BUENA'S PASSWORD");
    assert!(visible_radio_station_name("MISSING_RADIO", false).is_err());
}

#[test]
fn pokegear_radio_fern_uses_the_music_channel_weekday_song() {
    let shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let mut snapshot = shell.shell.snapshot().unwrap();
    snapshot.overworld.map_name = "CeladonDeptStore1F".into();
    snapshot.progression.active_engine_flags.insert("ENGINE_EXPN_CARD".into());
    snapshot.progression.active_engine_flags.remove("ENGINE_ROCKETS_IN_RADIO_TOWER");
    for day in 0..7 {
        snapshot.progression.time.day_of_week = day;
        let station = visible_pokegear_radio_station(&snapshot, "LetsAllSing").unwrap();
        assert_eq!(station, Some(("LETS_ALL_SING", if day & 1 == 0 {
            "MUSIC_POKEMON_MARCH".into()
        } else { "MUSIC_POKEMON_LULLABY".into() })),
            "FernMonMusic1 calls StartPokemonMusicChannel, independently of its RadioChannelSongs entry");
    }
}

#[test]
fn pokegear_radio_retuning_resets_all_audio_even_for_the_same_song() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.pokegear_menu_open = true;
    shell.pokegear_page = PokegearPage::Radio;
    shell.shell.session_mut().state_mut().flags.clear_engine_flag("ENGINE_ROCKETS_IN_RADIO_TOWER").unwrap();
    let mut snapshot = shell.shell.snapshot().unwrap();
    snapshot.progression.time.time_of_day = crate::core::world::encounters::TimeOfDay::Day;
    snapshot.progression.active_engine_flags.remove("ENGINE_ROCKETS_IN_RADIO_TOWER");
    for (knob, previous_song) in [(16, "MUSIC_POKEMON_TALK"), (16, "MUSIC_NEW_BARK_TOWN"), (18, "MUSIC_POKEMON_TALK")] {
        snapshot.progression.radio_tuning_knob = knob;
        shell.pending_audio.clear();
        shell.active_music = Some(previous_song.into());
        shell.pending_music_stop = false;
        shell.pending_full_audio_reset = false;
        shell.transient_audio_playing = true;
        shell.active_transient_kind = Some(ModpackAudioKind::Cry);
        shell.pending_audio.push(BevyAudioCommand {
            battle_sound: None,
            cry_parameters: None,
            audio_id: "SFX_READ_TEXT".into(), kind: ModpackAudioKind::SoundEffect,
            mode: ModpackAudioPlaybackMode::RawPcm, looped: false,
        });
        sync_visible_pokegear_radio(&mut shell, &snapshot).unwrap();
        if knob == 16 {
            assert!(!shell.pending_full_audio_reset, "LoadStation sets the program; PlayRadioShow starts its music");
            advance_visible_radio_broadcast(&mut shell, 1, false).unwrap();
        }
        assert!(shell.pending_full_audio_reset,
            "RadioMusicRestartDE/NoRadioMusic call MUSIC_NONE, resetting all channels");
        assert!(shell.pending_music_stop);
        if knob == 16 {
            assert_eq!(shell.pending_audio.len(), 1, "earlier queued sound must not survive the reset");
            assert_eq!(shell.pending_audio[0].audio_id, "MUSIC_POKEMON_TALK");
        } else {
            assert!(shell.pending_audio.is_empty());
            assert_eq!(shell.active_music.as_deref(), Some("MUSIC_NONE"));
        }
        let mut app = App::new();
        app.insert_resource(shell).add_systems(Update, play_pending_audio);
        app.world_mut().spawn(TransientAudioMarker);
        app.update();
        let result = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(result.last_error, None);
        assert!(!result.transient_audio_playing);
        assert!(result.active_transient_kind.is_none());
        assert_eq!(app.world_mut().query_filtered::<Entity, With<TransientAudioMarker>>().iter(app.world()).count(), 0);
        assert_eq!(app.world_mut().query_filtered::<Entity, With<MusicAudioMarker>>().iter(app.world()).count(),
            usize::from(knob == 16), "only a received station should leave a music playback entity");
        shell = app.world_mut().remove_resource::<BevyRuntimeShell>().unwrap();
    }
}

#[test]
fn pc_item_quantity_buttons_match_source_wrap_and_ten_item_steps() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    // These cases exercise SelectQuantityToToss after MenuTextbox returns.
    let question = "How many do you\nwant to withdraw?";
    shell.pc_notice = Some(question.into());
    shell.field_text_reveal = Some(VisibleFieldTextReveal {
        text: question.into(),
        page_index: 0,
        visible_chars: question.chars().count(),
        frames_until_next_char: 0,
    });
    let cases: &[(fn(&mut BevyRuntimeShell) -> Result<()>, u16, u16, u16)] = &[
        (move_visible_primary_cursor_up, 23, 23, 1),
        (move_visible_primary_cursor_up, 1, 23, 2),
        (move_visible_primary_cursor_down, 1, 23, 23),
        (move_visible_primary_cursor_down, 23, 23, 22),
        (move_visible_primary_cursor_left, 12, 23, 2),
        (move_visible_primary_cursor_left, 10, 23, 1),
        (move_visible_primary_cursor_right, 1, 23, 11),
        (move_visible_primary_cursor_right, 20, 23, 23),
        (move_visible_primary_cursor_up, 1, 1, 1),
        (move_visible_primary_cursor_down, 1, 1, 1),
        (move_visible_primary_cursor_right, 1, 1, 1),
    ];
    for action in [
        VisiblePlayerPcAction::WithdrawItem,
        VisiblePlayerPcAction::DepositItem,
        VisiblePlayerPcAction::TossItem,
    ] {
        for &(press, quantity, maximum, expected) in cases {
            shell.pc_item_quantity = Some(VisiblePcItemQuantity {
                action,
                item_id: "POTION".into(),
                stack_index: 0,
                quantity,
                maximum,
            });
            press(&mut shell).unwrap();
            assert_eq!(
                shell.pc_item_quantity.as_ref().unwrap().quantity,
                expected,
                "{action:?}: quantity {quantity}, maximum {maximum}"
            );
        }
    }
}

#[test]
fn pc_item_deposit_checks_all_source_bag_pockets_before_refusing_entry() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    for item_id in ["POKE_BALL", "BICYCLE", "TM_DYNAMICPUNCH"] {
        let definition = shell.shell.runtime().data().items[item_id].clone();
        {
            let bag = &mut shell.shell.session_mut().state_mut().bag;
            bag.items.clear();
            bag.balls.clear();
            bag.key_items.clear();
            bag.pc_items.clear();
            bag.tm_hm.fill(0);
            bag.custom_pockets.clear();
        }
        assert!(
            shell
                .shell
                .session_mut()
                .state_mut()
                .bag
                .add_item(&definition, 1)
                .unwrap()
        );
        shell.pc_notice = None;
        shell.field_pack_pocket = None;
        shell.bag_cursor = None;
        shell.pc_item_action = Some(VisiblePlayerPcAction::DepositItem);
        open_visible_pc_item_deposit_pack(&mut shell).unwrap();
        assert_eq!(
            shell.field_pack_pocket,
            Some(FieldPackPocket::Items),
            "HasNoItems must allow opening an empty ITEM pocket when carrying {item_id}"
        );
        assert!(
            shell.pc_notice.is_none(),
            "{item_id}: {:?}",
            shell.pc_notice
        );
        assert_eq!(
            shell.bag_cursor.as_ref().unwrap().option_index,
            0,
            "empty ITEM pocket retains its CANCEL row"
        );
    }
    {
        let bag = &mut shell.shell.session_mut().state_mut().bag;
        bag.items.clear();
        bag.balls.clear();
        bag.key_items.clear();
        bag.pc_items.clear();
        bag.tm_hm.fill(0);
        bag.custom_pockets.clear();
    }
    for item_id in ["POTION", "ANTIDOTE"] {
        let definition = shell.shell.runtime().data().items[item_id].clone();
        assert!(
            shell
                .shell
                .session_mut()
                .state_mut()
                .bag
                .add_item(&definition, 1)
                .unwrap()
        );
    }
    shell.bag_cursor = None;
    shell.field_pack_cursor_positions[0] = 1;
    open_visible_pc_item_deposit_pack(&mut shell).unwrap();
    assert_eq!(
        shell.bag_cursor.as_ref().unwrap().option_index,
        1,
        "DepositSellInitPackBuffers retains pocket cursor memory"
    );
}

#[test]
fn pc_item_deposit_returns_to_the_same_pack_selection_after_transfer() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let potion = shell.shell.runtime().data().items["POTION"].clone();
    {
        let bag = &mut shell.shell.session_mut().state_mut().bag;
        bag.items.clear();
        bag.balls.clear();
        bag.key_items.clear();
        bag.pc_items.clear();
        bag.tm_hm.fill(0);
        bag.custom_pockets.clear();
    }
    assert!(
        shell
            .shell
            .session_mut()
            .state_mut()
            .bag
            .add_item(&potion, 5)
            .unwrap()
    );
    shell.pc_item_action = Some(VisiblePlayerPcAction::DepositItem);
    open_visible_pc_item_deposit_pack(&mut shell).unwrap();
    begin_visible_pc_item_quantity(&mut shell).unwrap();
    commit_visible_pc_item_quantity(&mut shell).unwrap();
    assert_eq!(shell.shell.session().state().bag.quantity(&potion), 4);
    assert_eq!(
        shell.shell.session().state().bag.pc_item_quantity(&potion),
        1
    );
    assert_eq!(shell.field_pack_pocket, Some(FieldPackPocket::Items));
    assert_eq!(
        shell.pc_item_action,
        Some(VisiblePlayerPcAction::DepositItem)
    );
    assert_eq!(shell.bag_cursor.as_ref().unwrap().option_index, 0);
    dismiss_visible_pc_notice(&mut shell).unwrap();
    begin_visible_pc_item_quantity(&mut shell).unwrap();
    adjust_visible_pc_item_quantity(&mut shell, 10).unwrap();
    commit_visible_pc_item_quantity(&mut shell).unwrap();
    assert_eq!(shell.shell.session().state().bag.quantity(&potion), 0);
    assert_eq!(
        shell.shell.session().state().bag.pc_item_quantity(&potion),
        5
    );
    assert_eq!(shell.field_pack_pocket, Some(FieldPackPocket::Items));
    assert_eq!(
        shell.pc_item_action,
        Some(VisiblePlayerPcAction::DepositItem)
    );
    assert_eq!(
        shell.bag_cursor.as_ref().unwrap().option_index,
        0,
        "last stack becomes CANCEL"
    );
}

#[test]
fn pc_item_deposit_cancel_returns_to_player_pc_from_each_pocket() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let potion = shell.shell.runtime().data().items["POTION"].clone();
    assert!(
        shell
            .shell
            .session_mut()
            .state_mut()
            .bag
            .add_item(&potion, 1)
            .unwrap()
    );
    for press in [
        press_visible_a_button as fn(&mut BevyRuntimeShell) -> Result<()>,
        press_visible_b_button,
    ] {
        for pocket in [
            FieldPackPocket::Items,
            FieldPackPocket::Balls,
            FieldPackPocket::KeyItems,
            FieldPackPocket::TmHm,
        ] {
            shell.pc_item_action = Some(VisiblePlayerPcAction::DepositItem);
            shell.player_pc_action_cursor = None;
            shell.field_pack_cursor_positions = [0; 4];
            open_visible_pc_item_deposit_pack(&mut shell).unwrap();
            open_visible_field_pack_pocket(&mut shell, pocket.clone()).unwrap();
            let count = field_pack_pocket_count(&shell.shell.snapshot().unwrap(), &pocket);
            move_visible_active_field_pack_cursor(&mut shell, count as isize).unwrap();
            press(&mut shell).unwrap();
            assert_eq!(shell.field_pack_pocket, None, "{pocket:?}");
            assert_eq!(shell.pc_item_action, None, "{pocket:?}");
            assert_eq!(
                shell.player_pc_action_cursor.as_ref().unwrap().option_index,
                1
            );
            assert!(shell.pc_item_quantity.is_none());
        }
    }
}

#[test]
fn pc_item_deposit_uses_each_source_pocket_and_skips_protected_item_quantities() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    for (item_id, pocket, prompt) in [
        ("POKE_BALL", FieldPackPocket::Balls, true),
        ("TM_MUD_SLAP", FieldPackPocket::TmHm, true),
        ("BICYCLE", FieldPackPocket::KeyItems, false),
        ("HM_CUT", FieldPackPocket::TmHm, false),
    ] {
        let definition = shell.shell.runtime().data().items[item_id].clone();
        {
            let bag = &mut shell.shell.session_mut().state_mut().bag;
            bag.items.clear();
            bag.balls.clear();
            bag.key_items.clear();
            bag.pc_items.clear();
            bag.tm_hm.fill(0);
            bag.custom_pockets.clear();
            assert!(
                bag.add_item(&definition, if prompt { 3 } else { 1 })
                    .unwrap()
            );
        }
        shell.pc_notice = None;
        shell.shell.session_mut().state_mut().registered_key_item =
            (item_id == "BICYCLE").then(|| item_id.to_string());
        shell.pc_item_action = Some(VisiblePlayerPcAction::DepositItem);
        shell.player_pc_action_cursor = None;
        shell.field_pack_cursor_positions = [0; 4];
        open_visible_pc_item_deposit_pack(&mut shell).unwrap();
        open_visible_field_pack_pocket(&mut shell, pocket.clone()).unwrap();
        begin_visible_pc_item_quantity(&mut shell).unwrap();
        if prompt {
            assert_eq!(
                shell.pc_item_quantity.as_ref().unwrap().maximum,
                3,
                "{item_id}"
            );
            adjust_visible_pc_item_quantity(&mut shell, 1).unwrap();
            commit_visible_pc_item_quantity(&mut shell).unwrap();
        } else {
            assert!(
                shell.pc_item_quantity.is_none(),
                "{item_id} must transfer one without a quantity prompt"
            );
        }
        assert_eq!(
            shell
                .shell
                .session()
                .state()
                .bag
                .pc_item_quantity(&definition),
            if prompt { 2 } else { 1 },
            "{item_id}"
        );
        assert_eq!(
            shell.shell.session().state().bag.quantity(&definition),
            if prompt { 1 } else { 0 },
            "{item_id}"
        );
        assert_eq!(shell.field_pack_pocket, Some(pocket));
        if item_id == "BICYCLE" {
            assert_eq!(
                shell.shell.session().state().registered_key_item,
                None,
                "CheckRegisteredItem clears a deposited registered key item"
            );
        }
    }
}

#[test]
fn pc_item_withdrawal_checks_the_items_actual_destination_pocket() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let ball = shell.shell.runtime().data().items["POKE_BALL"].clone();
    let item_definitions = shell
        .shell
        .runtime()
        .data()
        .items
        .values()
        .filter(|item| item.pocket == "ITEM")
        .take(20)
        .cloned()
        .collect::<Vec<_>>();
    {
        let bag = &mut shell.shell.session_mut().state_mut().bag;
        bag.items.clear();
        bag.balls.clear();
        bag.pc_items.clear();
        for item in &item_definitions {
            assert!(bag.add_item(item, 99).unwrap());
        }
        assert!(bag.add_pc_item(&ball, 2).unwrap());
    }
    shell.pc_item_action = Some(VisiblePlayerPcAction::WithdrawItem);
    shell.pc_item_cursor = Some(MenuCursor {
        surface_id: "pc:items".into(),
        option_index: 0,
    });
    begin_visible_pc_item_quantity(&mut shell).unwrap();
    commit_visible_pc_item_quantity(&mut shell).unwrap();
    assert_eq!(
        shell.shell.session().state().bag.quantity(&ball),
        1,
        "a full ITEM pocket must not block withdrawal into the BALL pocket"
    );
    assert_eq!(shell.shell.session().state().bag.pc_item_quantity(&ball), 1);
}

#[test]
fn pc_item_quantity_arrows_leave_the_printed_question_unchanged() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let question = "How many do you\nwant to withdraw?";
    shell.pc_item_quantity = Some(VisiblePcItemQuantity {
        action: VisiblePlayerPcAction::WithdrawItem,
        item_id: "POTION".into(),
        stack_index: 0,
        quantity: 1,
        maximum: 23,
    });
    shell.pc_notice = Some(question.into());
    shell.field_text_reveal = Some(VisibleFieldTextReveal {
        text: question.into(),
        page_index: 0,
        visible_chars: question.chars().count(),
        frames_until_next_char: 0,
    });
    for delta in [1, 10, -1, -10] {
        adjust_visible_pc_item_quantity(&mut shell, delta).unwrap();
        assert_eq!(shell.pc_notice.as_deref(), Some(question));
        assert_eq!(
            visible_revealed_shell_notice_text(&shell, question),
            question
        );
    }
}

#[test]
fn pc_item_list_matches_source_initial_geometry() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.session_mut().state_mut().bag.pc_items.clear();
    for (id, quantity) in [
        ("POTION", 23),
        ("ANTIDOTE", 1),
        ("POKE_BALL", 12),
        ("GREAT_BALL", 2),
        ("ESCAPE_ROPE", 3),
        ("REPEL", 4),
    ] {
        let item = shell.shell.runtime().data().items[id].clone();
        assert!(shell
            .shell
            .session_mut()
            .state_mut()
            .bag
            .add_pc_item(&item, quantity)
            .unwrap());
    }
    shell.pc_item_cursor = Some(MenuCursor {
        surface_id: "pc:items".into(),
        option_index: 0,
    });
    let snapshot = shell.shell.snapshot().unwrap();
    let mut world = World::new();
    let mut queue = bevy::ecs::world::CommandQueue::default();
    let mut images = Assets::<Image>::default();
    let mut art = RenderedTilesetArt::default();
    let mut commands = Commands::new(&mut queue, &world);
    spawn_field_pc_item_screen(
        &mut commands,
        &snapshot,
        &shell,
        &mut art,
        &shell.asset_root,
        &mut images,
    )
    .unwrap();
    queue.apply(&mut world);
    assert_eq!(art.font_error, None);
    let canvas = render_pc_audit_canvas(&mut world, &images, "pc-items-initial");
    let reference = image::load_from_memory(&external_oracle_fixture_bytes("pc-items-initial.png"))
    .unwrap()
    .to_rgba8();
    let scale = canvas.width() / 160;
    if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        canvas
            .save(PathBuf::from(directory).join("pc-items-initial.png"))
            .unwrap();
    }
    // Compare source ink independently of the inherited CGB palette. This
    // checks all glyphs, frame pixels and spacing, not palette ownership.
    for y in 0..144 {
        for x in 0..160 {
            let actual = canvas.get_pixel(x * scale, y * scale).0;
            let expected = reference.get_pixel(x, y).0;
            assert_eq!(
                actual[..3].iter().all(|v| *v < 64),
                expected[..3].iter().all(|v| *v < 64),
                "source ink at {x},{y}"
            );
        }
    }
}

#[test]
fn pc_item_list_ignores_horizontal_input_and_stops_at_cancel() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.session_mut().state_mut().bag.pc_items.clear();
    for id in ["POTION", "ANTIDOTE"] {
        let item = shell.shell.runtime().data().items[id].clone();
        assert!(shell
            .shell
            .session_mut()
            .state_mut()
            .bag
            .add_pc_item(&item, 1)
            .unwrap());
    }
    shell.pc_item_action = Some(VisiblePlayerPcAction::WithdrawItem);
    shell.pc_item_cursor = Some(MenuCursor {
        surface_id: "pc:items".into(),
        option_index: 0,
    });
    shell.pc_notice = None;
    move_visible_primary_cursor_right(&mut shell).unwrap();
    assert_eq!(
        shell.pc_item_cursor.as_ref().unwrap().option_index,
        0,
        "PCItemsMenuData does not enable horizontal input"
    );
    move_visible_primary_cursor_left(&mut shell).unwrap();
    assert_eq!(shell.pc_item_cursor.as_ref().unwrap().option_index, 0);
    move_visible_pc_item_cursor(&mut shell, -1).unwrap();
    assert_eq!(shell.pc_item_cursor.as_ref().unwrap().option_index, 0);
    for expected in [1, 2, 2] {
        move_visible_pc_item_cursor(&mut shell, 1).unwrap();
        assert_eq!(
            shell.pc_item_cursor.as_ref().unwrap().option_index,
            expected
        );
    }
    press_visible_a_button(&mut shell).unwrap();
    assert!(shell.pc_item_cursor.is_none());
    assert!(shell.pc_item_quantity.is_none());
    assert_eq!(
        shell.player_pc_action_cursor.as_ref().unwrap().option_index,
        0
    );
}

#[test]
fn pc_item_list_scroll_matches_source_down_trace() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.session_mut().state_mut().bag.pc_items.clear();
    for id in [
        "POTION",
        "ANTIDOTE",
        "POKE_BALL",
        "GREAT_BALL",
        "ESCAPE_ROPE",
        "REPEL",
    ] {
        let item = shell.shell.runtime().data().items[id].clone();
        assert!(shell
            .shell
            .session_mut()
            .state_mut()
            .bag
            .add_pc_item(&item, 1)
            .unwrap());
    }
    shell.pc_item_action = Some(VisiblePlayerPcAction::WithdrawItem);
    shell.pc_item_cursor = Some(MenuCursor {
        surface_id: "pc:items".into(),
        option_index: 0,
    });
    let source: serde_json::Value = serde_json::from_str(&external_oracle_fixture_text("pc-items-scroll.json"))
    .unwrap();
    for (index, record) in source["records"].as_array().unwrap().iter().enumerate() {
        if index != 0 {
            move_visible_pc_item_cursor(&mut shell, 1).unwrap();
        }
        let scroll = record["scroll"].as_u64().unwrap() as usize;
        let row = record["cursor"].as_u64().unwrap() as usize - 1;
        assert_eq!(
            shell.pc_item_cursor.as_ref().unwrap().option_index,
            scroll + row
        );
        assert_eq!(shell.pc_item_scroll, scroll, "source down input {index}");
    }
}

#[test]
fn pc_item_list_empty_withdraw_and_toss_open_on_cancel() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.session_mut().state_mut().bag.pc_items.clear();
    for (index, action) in [
        (0, VisiblePlayerPcAction::WithdrawItem),
        (2, VisiblePlayerPcAction::TossItem),
    ] {
        shell.pc_notice = None;
        shell.pc_item_cursor = None;
        shell.pc_item_action = None;
        shell.player_pc_action_cursor = Some(MenuCursor {
            surface_id: "pc:player-actions".into(),
            option_index: index,
        });
        confirm_visible_player_pc_action(&mut shell).unwrap();
        assert_eq!(shell.pc_item_action, Some(action));
        assert_eq!(
            shell
                .pc_item_cursor
                .as_ref()
                .map(|cursor| cursor.option_index),
            Some(0)
        );
        assert!(
            shell.pc_notice.is_none(),
            "source opens its empty scrolling list"
        );
        press_visible_a_button(&mut shell).unwrap();
        assert_eq!(
            shell.player_pc_action_cursor.as_ref().unwrap().option_index,
            index
        );
        assert!(shell.pc_item_quantity.is_none());
    }
}

#[test]
fn pc_item_list_withdrawal_keeps_source_row_and_cancel() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    for (ids, selected, scroll, expected_index, expected_scroll) in [
        (vec!["POTION", "ANTIDOTE", "GREAT_BALL"], 1, 0, 1, 0),
        (vec!["POTION"], 0, 0, 0, 0),
        (
            vec![
                "POTION",
                "ANTIDOTE",
                "POKE_BALL",
                "GREAT_BALL",
                "ESCAPE_ROPE",
                "REPEL",
            ],
            5,
            2,
            5,
            2,
        ),
    ] {
        shell.shell.session_mut().state_mut().bag.pc_items.clear();
        for id in ids {
            let item = shell.shell.runtime().data().items[id].clone();
            assert!(shell
                .shell
                .session_mut()
                .state_mut()
                .bag
                .add_pc_item(&item, 1)
                .unwrap());
        }
        shell.pc_item_action = Some(VisiblePlayerPcAction::WithdrawItem);
        shell.pc_item_cursor = Some(MenuCursor {
            surface_id: "pc:items".into(),
            option_index: selected,
        });
        shell.pc_item_scroll = scroll;
        shell.player_pc_action_cursor = None;
        shell.pc_notice = None;
        begin_visible_pc_item_quantity(&mut shell).unwrap();
        commit_visible_pc_item_quantity(&mut shell).unwrap();
        dismiss_visible_pc_notice(&mut shell).unwrap();
        assert_eq!(
            shell.pc_item_action,
            Some(VisiblePlayerPcAction::WithdrawItem)
        );
        assert!(shell.player_pc_action_cursor.is_none());
        assert_eq!(
            shell.pc_item_cursor.as_ref().unwrap().option_index,
            expected_index
        );
        assert_eq!(shell.pc_item_scroll, expected_scroll);
    }
}

#[test]
fn pc_item_list_reopens_at_its_source_row_and_scroll() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.session_mut().state_mut().bag.pc_items.clear();
    for id in [
        "POTION",
        "ANTIDOTE",
        "POKE_BALL",
        "GREAT_BALL",
        "ESCAPE_ROPE",
        "REPEL",
    ] {
        let item = shell.shell.runtime().data().items[id].clone();
        assert!(shell
            .shell
            .session_mut()
            .state_mut()
            .bag
            .add_pc_item(&item, 1)
            .unwrap());
    }
    shell.pc_item_action = Some(VisiblePlayerPcAction::WithdrawItem);
    shell.pc_item_cursor = Some(MenuCursor {
        surface_id: "pc:items".into(),
        option_index: 0,
    });
    for _ in 0..5 {
        move_visible_pc_item_cursor(&mut shell, 1).unwrap();
    }
    close_visible_pc_item_list(&mut shell);
    confirm_visible_player_pc_action(&mut shell).unwrap();
    assert_eq!(shell.pc_item_cursor.as_ref().unwrap().option_index, 5);
    assert_eq!(shell.pc_item_scroll, 2);
}

#[test]
fn pc_item_list_toss_keeps_source_row_and_cancel() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    for (ids, selected, scroll, expected_index, expected_scroll) in [
        (vec!["POTION", "ANTIDOTE", "GREAT_BALL"], 1, 0, 1, 0),
        (vec!["POTION"], 0, 0, 0, 0),
        (
            vec![
                "POTION",
                "ANTIDOTE",
                "POKE_BALL",
                "GREAT_BALL",
                "ESCAPE_ROPE",
                "REPEL",
            ],
            5,
            2,
            5,
            2,
        ),
    ] {
        shell.shell.session_mut().state_mut().bag.pc_items.clear();
        for id in ids {
            let item = shell.shell.runtime().data().items[id].clone();
            assert!(shell
                .shell
                .session_mut()
                .state_mut()
                .bag
                .add_pc_item(&item, 1)
                .unwrap());
        }
        shell.pc_item_action = Some(VisiblePlayerPcAction::TossItem);
        shell.pc_item_cursor = Some(MenuCursor {
            surface_id: "pc:items".into(),
            option_index: selected,
        });
        shell.pc_item_scroll = scroll;
        shell.player_pc_action_cursor = None;
        shell.pc_notice = None;
        begin_visible_pc_item_quantity(&mut shell).unwrap();
        commit_visible_pc_item_quantity(&mut shell).unwrap();
        resolve_visible_pc_confirmation(&mut shell, true).unwrap();
        dismiss_visible_pc_notice(&mut shell).unwrap();
        assert_eq!(
            shell.pc_item_action,
            Some(VisiblePlayerPcAction::TossItem)
        );
        assert!(shell.player_pc_action_cursor.is_none());
        assert_eq!(
            shell.pc_item_cursor.as_ref().unwrap().option_index,
            expected_index
        );
        assert_eq!(shell.pc_item_scroll, expected_scroll);
    }
}

#[test]
fn pc_item_quantity_question_finishes_before_selector_input() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let potion = shell.shell.runtime().data().items["POTION"].clone();
    shell.shell.session_mut().state_mut().bag.pc_items.clear();
    assert!(shell
        .shell
        .session_mut()
        .state_mut()
        .bag
        .add_pc_item(&potion, 3)
        .unwrap());
    let question = "How many do you\nwant to withdraw?";
    let buttons: &[(&str, fn(&mut BevyRuntimeShell) -> Result<()>)] = &[
        ("A", press_visible_a_button),
        ("B", press_visible_b_button),
        ("Up", move_visible_primary_cursor_up),
        ("Down", move_visible_primary_cursor_down),
        ("Left", move_visible_primary_cursor_left),
        ("Right", move_visible_primary_cursor_right),
    ];
    for &(name, press) in buttons {
        shell.pc_item_action = Some(VisiblePlayerPcAction::WithdrawItem);
        shell.pc_item_cursor = Some(MenuCursor {
            surface_id: "pc:items".into(),
            option_index: 0,
        });
        shell.pc_item_quantity = Some(VisiblePcItemQuantity {
            action: VisiblePlayerPcAction::WithdrawItem,
            item_id: "POTION".into(),
            stack_index: 0,
            quantity: 1,
            maximum: 3,
        });
        shell.pc_notice = Some(question.into());
        shell.field_text_reveal = Some(VisibleFieldTextReveal {
            text: question.into(),
            page_index: 0,
            visible_chars: 1,
            frames_until_next_char: 1,
        });
        press(&mut shell).unwrap();
        assert_eq!(
            shell.pc_item_quantity.as_ref().map(|q| q.quantity),
            Some(1),
            "{name} reached the selector before MenuTextbox returned"
        );
        assert_eq!(
            shell.shell.session().state().bag.pc_item_quantity(&potion),
            3
        );
    }
    shell.field_text_reveal.as_mut().unwrap().visible_chars = question.chars().count();
    press_visible_a_button(&mut shell).unwrap();
    assert!(shell.pc_item_quantity.is_none());
    assert_eq!(
        shell.shell.session().state().bag.pc_item_quantity(&potion),
        2
    );
}

#[test]
fn pokegear_live_furniture_radio_prints_broadcast_after_initial_hold() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    activate_visible_special_routine_boundary(&mut shell,
        &SpecialRoutineEffect::MapRadio { station: "MAPRADIO_ROCKET".into() }).unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(shell)
        .insert_resource(native_rtc_source_for_test())
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(RuntimeTickTimer::new(0.0))
        .add_systems(Update, apply_keyboard_input);
    for _ in 0..150 { app.update(); }
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(shell.last_error, None);
    assert_eq!(shell.pokegear_map_radio_delay, Some(0));
    let snapshot = shell.shell.snapshot().unwrap();
    let text = visible_pokegear_menu_entries(&snapshot, shell).unwrap().join("\n");
    assert!(text.contains("Ahem, we are"),
        "PlayRadioShow must print source RocketRadioText1 after its 100-frame hold, got {text:?}");
    for index in 0..17 {
        if index > 0 {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            let mut complete = false;
            for _ in 0..150 {
                advance_visible_radio_broadcast(&mut shell, 1, false).unwrap();
                let state = &shell.pokegear_radio_broadcast.as_ref().unwrap().playback.state;
                if state.current_line == 84 && state.delay == 100 { complete = true; break; }
            }
            assert!(complete, "radio print {index} did not complete");
        }
        let shell = app.world().resource::<BevyRuntimeShell>();
        let snapshot = shell.shell.snapshot().unwrap();
        let mut world = World::new();
        let mut queue = bevy::ecs::world::CommandQueue::default();
        let mut images = Assets::<Image>::default();
        let mut art = RenderedTilesetArt::default();
        let mut commands = Commands::new(&mut queue, &world);
        spawn_field_pokegear_screen(&mut commands, &snapshot, shell, &mut art, &shell.asset_root, &mut images).unwrap();
        queue.apply(&mut world);
        assert_eq!(art.font_error, None);
        let canvas = render_pc_audit_canvas(&mut world, &images, "radio-broadcast");
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../../tools/asm-oracle/fixtures/radio-rocket-render/radio-print-{index}.png"));
        let reference = image::open(source).unwrap().to_rgba8();
        let scale = canvas.width() / 160;
        for y in 0..144 {
            for x in 0..160 {
                let actual = canvas.get_pixel(x * scale, y * scale).0;
                if y < 96 { assert_eq!(actual[3], 0, "radio overlay covers map {x},{y}"); }
                else { assert_eq!(actual.map(|value| value >> 3), reference.get_pixel(x, y).0.map(|value| value >> 3), "source radio print {index} pixel {x},{y}"); }
            }
        }
        if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
            std::fs::create_dir_all(&directory).unwrap();
            canvas.save(PathBuf::from(directory).join(format!("radio-print-{index}.png"))).unwrap();
        }
    }

}

#[test]
fn pc_item_select_moves_stack_with_a_instead_of_opening_quantity() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.session_mut().state_mut().bag.pc_items.clear();
    for (id, quantity) in [("POTION", 23), ("ANTIDOTE", 2), ("POKE_BALL", 12)] {
        let item = shell.shell.runtime().data().items[id].clone();
        assert!(
            shell
                .shell
                .session_mut()
                .state_mut()
                .bag
                .add_pc_item(&item, quantity)
                .unwrap()
        );
    }
    shell.pc_item_action = Some(VisiblePlayerPcAction::WithdrawItem);
    shell.pc_item_cursor = Some(MenuCursor {
        surface_id: "pc:items".into(),
        option_index: 0,
    });
    assert!(
        has_visible_shell_select_action(&mut shell),
        "PCItemsJoypad enables SELECT"
    );
    shell.pending_audio.clear();
    shell.transient_audio_playing = false;
    press_visible_select_button(&mut shell).unwrap();
    assert!(
        shell.field_notice.is_none(),
        "PC Select must not use the registered-item path"
    );
    move_visible_pc_item_cursor(&mut shell, 2).unwrap();
    press_visible_a_button(&mut shell).unwrap();
    assert!(
        shell.pc_item_quantity.is_none(),
        "A places the carried stack before entering withdrawal"
    );
    assert!(shell.pc_item_move_sequence.is_some());
    advance_visible_pc_item_move_sequence(&mut shell).unwrap();
    assert!(
        shell.pc_item_move_sequence.is_some(),
        "first switch sound must finish before the second starts"
    );
    shell.pending_audio.clear();
    shell.transient_audio_playing = false;
    advance_visible_pc_item_move_sequence(&mut shell).unwrap();
    assert!(
        shell.pc_item_move_sequence.is_none(),
        "second sound starts without another wait"
    );
    assert!(
        shell
            .pending_audio
            .iter()
            .any(|audio| audio.audio_id == "SFX_SWITCH_POKEMON")
    );
    let snapshot = shell.shell.snapshot().unwrap();
    assert_eq!(
        snapshot
            .bag
            .pc_items
            .iter()
            .map(|stack| (stack.item_id.as_str(), stack.quantity))
            .collect::<Vec<_>>(),
        [("ANTIDOTE", 2), ("POKE_BALL", 12), ("POTION", 23)]
    );
}

#[test]
fn pc_item_select_b_cancels_move_without_closing_the_list() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.session_mut().state_mut().bag.pc_items.clear();
    let item = shell.shell.runtime().data().items["POTION"].clone();
    assert!(
        shell
            .shell
            .session_mut()
            .state_mut()
            .bag
            .add_pc_item(&item, 2)
            .unwrap()
    );
    shell.pc_item_action = Some(VisiblePlayerPcAction::WithdrawItem);
    shell.pc_item_cursor = Some(MenuCursor {
        surface_id: "pc:items".into(),
        option_index: 0,
    });
    press_visible_select_button(&mut shell).unwrap();
    press_visible_b_button(&mut shell).unwrap();
    assert!(
        shell.pc_item_cursor.is_some(),
        "PCItemsJoypad .b_2 clears wSwitchItem and reopens the list"
    );
    assert_eq!(
        shell.pc_item_action,
        Some(VisiblePlayerPcAction::WithdrawItem)
    );
    press_visible_b_button(&mut shell).unwrap();
    assert!(shell.pc_item_cursor.is_none(), "the next B exits normally");
}

#[test]
fn pokegear_compiled_radio_retains_raw_landmark_names() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for filename in [
        "core-modular.crystalpack",
        "core-modular.browser.crystalpack",
    ] {
        let pack = crystal_assets::read_verified_compiled_game_pack(
            root.join("content-packs").join(filename),
        )
        .unwrap();
        let definitions = &pack.data().global_scripts.as_ref().unwrap().definitions;
        let rows = definitions["Landmarks"].as_array().unwrap();
        assert_eq!(rows.len(), 96, "{filename}");
        assert_eq!(
            definitions["NewBarkTownName"][0]["args"][0], "\"NEW BARK<BSP>TOWN@\"",
            "{filename}"
        );
        for row in rows {
            let label = row["args"][2].as_str().unwrap();
            assert!(
                definitions.contains_key(label),
                "{filename} omitted source name {label}"
            );
        }
    }
}

#[test]
fn pc_item_text_observation_accepts_cancel_in_empty_and_nonempty_lists() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.session_mut().state_mut().bag.pc_items.clear();
    shell.pc_item_action = Some(VisiblePlayerPcAction::WithdrawItem);
    for count in 0..=1 {
        if count == 1 {
            let item = shell.shell.runtime().data().items["POTION"].clone();
            assert!(
                shell
                    .shell
                    .session_mut()
                    .state_mut()
                    .bag
                    .add_pc_item(&item, 1)
                    .unwrap()
            );
        }
        shell.pc_item_cursor = Some(MenuCursor {
            surface_id: "pc:items".into(),
            option_index: count,
        });
        let snapshot = shell.shell.snapshot().unwrap();
        let mut entries = Vec::new();
        push_visible_pc_item_dialog_entries(&mut entries, &snapshot, &shell).unwrap();
        assert!(
            entries.iter().any(|line| line.contains(">CANCEL")),
            "source CANCEL row must be visible: {entries:?}"
        );
        assert!(
            !entries
                .iter()
                .any(|line| line.contains("INVALID CURSOR") || line == "EMPTY"),
            "{entries:?}"
        );
    }
}

#[test]
fn pc_item_select_is_available_over_source_pc_window() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.session_mut().state_mut().script_runtime.window_open = true;
    shell.pc_item_action = Some(VisiblePlayerPcAction::WithdrawItem);
    shell.pc_item_cursor = Some(MenuCursor { surface_id: "pc:items".into(), option_index: 0 });
    mark_runtime_snapshot_dirty(&mut shell);
    assert!(shell.shell.snapshot().unwrap().ui.window_open);
    assert!(has_visible_shell_select_action(&mut shell), "PCItemsJoypad owns SELECT while its parent script window remains open");
}

#[test]
fn pc_item_move_render_matches_source_select_place_and_cancel_frames() {
    let fixture: serde_json::Value = serde_json::from_str(&external_oracle_fixture_text("pc-items-move.json"))
    .unwrap();
    let references: [&[u8]; 8] = [
        &external_oracle_fixture_bytes("pc-items-move-0.png"),
        &external_oracle_fixture_bytes("pc-items-move-1.png"),
        &external_oracle_fixture_bytes("pc-items-move-2.png"),
        &external_oracle_fixture_bytes("pc-items-move-3.png"),
        &external_oracle_fixture_bytes("pc-items-move-4.png"),
        &external_oracle_fixture_bytes("pc-items-move-5.png"),
        &external_oracle_fixture_bytes("pc-items-move-6.png"),
        &external_oracle_fixture_bytes("pc-items-move-7.png"),
    ];
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.session_mut().state_mut().bag.pc_items.clear();
    for (id, quantity) in [
        ("POTION", 23),
        ("ANTIDOTE", 1),
        ("POKE_BALL", 12),
        ("GREAT_BALL", 2),
        ("ESCAPE_ROPE", 3),
        ("REPEL", 4),
    ] {
        let item = shell.shell.runtime().data().items[id].clone();
        assert!(
            shell
                .shell
                .session_mut()
                .state_mut()
                .bag
                .add_pc_item(&item, quantity)
                .unwrap()
        );
    }
    shell.pc_item_action = Some(VisiblePlayerPcAction::WithdrawItem);
    shell.pc_item_cursor = Some(MenuCursor {
        surface_id: "pc:items".into(),
        option_index: 0,
    });
    let mut images = Assets::<Image>::default();
    for (index, record) in fixture["records"].as_array().unwrap().iter().enumerate() {
        shell.pending_audio.clear();
        shell.transient_audio_playing = false;
        match record["button"].as_str() {
            None => {}
            Some("select") => press_visible_select_button(&mut shell).unwrap(),
            Some("down") => move_visible_pc_item_cursor(&mut shell, 1).unwrap(),
            Some("a") => press_visible_a_button(&mut shell).unwrap(),
            Some("b") => press_visible_b_button(&mut shell).unwrap(),
            other => panic!("unexpected source input {other:?}"),
        }
        for _ in 0..3 {
            if shell.pc_item_move_sequence.is_none() {
                break;
            }
            shell.pending_audio.clear();
            shell.transient_audio_playing = false;
            advance_visible_pc_item_move_sequence(&mut shell).unwrap();
        }
        assert!(shell.pc_item_move_sequence.is_none());
        assert_eq!(
            shell.pc_item_switch_origin.map_or(0, |i| i + 1),
            record["switch_item"].as_u64().unwrap() as usize
        );
        assert_eq!(
            shell.pc_item_scroll,
            record["scroll"].as_u64().unwrap() as usize
        );
        assert_eq!(
            shell.pc_item_cursor.as_ref().unwrap().option_index - shell.pc_item_scroll + 1,
            record["cursor"].as_u64().unwrap() as usize
        );
        let snapshot = shell.shell.snapshot().unwrap();
        let mut world = World::new();
        let mut art = RenderedTilesetArt::default();
        let mut queue = bevy::ecs::world::CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);
        spawn_field_pc_item_screen(
            &mut commands,
            &snapshot,
            &shell,
            &mut art,
            &shell.asset_root,
            &mut images,
        )
        .unwrap();
        queue.apply(&mut world);
        assert_eq!(art.font_error, None);
        let canvas = render_pc_audit_canvas(&mut world, &images, "pc-items-move");
        let reference = image::load_from_memory(references[index])
            .unwrap()
            .to_rgba8();
        let scale = canvas.width() / 160;
        // Compare source RGB5 as well as ink; display color expansion may
        // differ while the actual Game Boy color values must agree.
        for y in 0..144 {
            for x in 0..160 {
                let actual = canvas.get_pixel(x * scale, y * scale).0;
                let expected = reference.get_pixel(x, y).0;
                assert_eq!(actual.map(|value| value >> 3), expected.map(|value| value >> 3),
                    "frame {index}, source RGB5 {x},{y}");
                assert_eq!(
                    actual[..3].iter().all(|v| *v < 64),
                    expected[..3].iter().all(|v| *v < 64),
                    "frame {index}, source ink {x},{y}"
                );
            }
        }
    }
}

#[test]
fn pc_withdraw_quantity_matches_real_source_menu_frames() {
    let fixture: serde_json::Value = serde_json::from_str(&external_oracle_fixture_text("pc-withdraw-quantity.json"))
    .unwrap();
    let references: [&[u8]; 4] = [
        &external_oracle_fixture_bytes("pc-withdraw-quantity-0.png"),
        &external_oracle_fixture_bytes("pc-withdraw-quantity-1.png"),
        &external_oracle_fixture_bytes("pc-withdraw-quantity-2.png"),
        &external_oracle_fixture_bytes("pc-withdraw-quantity-3.png"),
    ];
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.session_mut().state_mut().bag.pc_items.clear();
    for (id, quantity) in [
        ("POTION", 23),
        ("ANTIDOTE", 1),
        ("POKE_BALL", 12),
        ("GREAT_BALL", 2),
        ("ESCAPE_ROPE", 3),
        ("REPEL", 4),
    ] {
        let item = shell.shell.runtime().data().items[id].clone();
        assert!(
            shell
                .shell
                .session_mut()
                .state_mut()
                .bag
                .add_pc_item(&item, quantity)
                .unwrap()
        );
    }
    shell.pc_item_action = Some(VisiblePlayerPcAction::WithdrawItem);
    shell.pc_item_cursor = Some(MenuCursor {
        surface_id: "pc:items".into(),
        option_index: 0,
    });
    press_visible_a_button(&mut shell).unwrap();
    let question = shell.pc_notice.clone().unwrap();
    shell.field_text_reveal = Some(VisibleFieldTextReveal {
        text: question.clone(),
        page_index: 0,
        visible_chars: question.chars().count(),
        frames_until_next_char: 0,
    });
    for (index, record) in fixture["records"].as_array().unwrap().iter().enumerate() {
        match record["button"].as_str() {
            None => {}
            Some("down") => adjust_visible_pc_item_quantity(&mut shell, -1).unwrap(),
            Some("up") => adjust_visible_pc_item_quantity(&mut shell, 1).unwrap(),
            Some("right") => adjust_visible_pc_item_quantity(&mut shell, 10).unwrap(),
            other => panic!("unexpected source quantity input {other:?}"),
        }
        assert_eq!(
            shell.pc_item_quantity.as_ref().unwrap().quantity as u64,
            record["quantity"].as_u64().unwrap()
        );
        let snapshot = shell.shell.snapshot().unwrap();
        let mut world = World::new();
        let mut queue = bevy::ecs::world::CommandQueue::default();
        let mut images = Assets::<Image>::default();
        let mut art = RenderedTilesetArt::default();
        let mut commands = Commands::new(&mut queue, &world);
        spawn_scene_dialog(
            &mut commands,
            &snapshot,
            &shell,
            &mut art,
            &shell.asset_root,
            &mut images,
        )
        .unwrap();
        queue.apply(&mut world);
        let canvas = render_pc_audit_canvas(&mut world, &images, "pc-withdraw-quantity");
        let reference = image::load_from_memory(references[index])
            .unwrap()
            .to_rgba8();
        let scale = canvas.width() / 160;
        for y in 0..144 {
            for x in 0..160 {
                assert_eq!(
                    canvas.get_pixel(x * scale, y * scale).0.map(|v| v >> 3),
                    reference.get_pixel(x, y).0.map(|v| v >> 3),
                    "frame {index}, source pixel {x},{y}"
                );
            }
        }
    }
}

#[test]
fn pc_deposit_quantity_matches_real_source_menu_frames() {
    let fixture: serde_json::Value = serde_json::from_str(&external_oracle_fixture_text("pc-deposit-quantity.json"))
    .unwrap();
    let references: [&[u8]; 4] = [
        &external_oracle_fixture_bytes("pc-deposit-quantity-0.png"),
        &external_oracle_fixture_bytes("pc-deposit-quantity-1.png"),
        &external_oracle_fixture_bytes("pc-deposit-quantity-2.png"),
        &external_oracle_fixture_bytes("pc-deposit-quantity-3.png"),
    ];
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let potion = shell.shell.runtime().data().items["POTION"].clone();
    shell.shell.session_mut().state_mut().bag.items.clear();
    shell
        .shell
        .session_mut()
        .state_mut()
        .bag
        .add_item(&potion, 23)
        .unwrap();
    shell.pc_item_action = Some(VisiblePlayerPcAction::DepositItem);
    open_visible_pc_item_deposit_pack(&mut shell).unwrap();
    press_visible_a_button(&mut shell).unwrap();
    let question = shell.pc_notice.clone().unwrap();
    shell.field_text_reveal = Some(VisibleFieldTextReveal {
        text: question.clone(),
        page_index: 0,
        visible_chars: question.chars().count(),
        frames_until_next_char: 0,
    });
    for (index, record) in fixture["records"].as_array().unwrap().iter().enumerate() {
        match record["button"].as_str() {
            None => {}
            Some("down") => adjust_visible_pc_item_quantity(&mut shell, -1).unwrap(),
            Some("up") => adjust_visible_pc_item_quantity(&mut shell, 1).unwrap(),
            Some("right") => adjust_visible_pc_item_quantity(&mut shell, 10).unwrap(),
            other => panic!("unexpected source quantity input {other:?}"),
        }
        assert_eq!(
            shell.pc_item_quantity.as_ref().unwrap().quantity as u64,
            record["quantity"].as_u64().unwrap()
        );
        let snapshot = shell.shell.snapshot().unwrap();
        let mut world = World::new();
        let mut queue = bevy::ecs::world::CommandQueue::default();
        let mut images = Assets::<Image>::default();
        let mut art = RenderedTilesetArt::default();
        let mut commands = Commands::new(&mut queue, &world);
        spawn_field_pack_screen(
            &mut commands,
            &snapshot,
            &shell,
            &mut art,
            &shell.asset_root,
            &mut images,
        )
        .unwrap();
        queue.apply(&mut world);
        let canvas = render_pc_audit_canvas(&mut world, &images, "pc-deposit-quantity");
        let reference = image::load_from_memory(references[index])
            .unwrap()
            .to_rgba8();
        let scale = canvas.width() / 160;
        for y in 0..144 {
            for x in 0..160 {
                assert_eq!(
                    canvas.get_pixel(x * scale, y * scale).0.map(|v| v >> 3),
                    reference.get_pixel(x, y).0.map(|v| v >> 3),
                    "frame {index}, source pixel {x},{y}"
                );
            }
        }
    }
}

#[test]
fn pc_deposit_pack_scroll_matches_source_in_both_directions() {
    let fixture: serde_json::Value = serde_json::from_str(&external_oracle_fixture_text("pc-deposit-scroll/pc-deposit-scroll.json"))
    .unwrap();
    let references: [&[u8]; 12] = [
        &external_oracle_fixture_bytes("pc-deposit-scroll/rom-pc-deposit-scroll-0.png"),
        &external_oracle_fixture_bytes("pc-deposit-scroll/rom-pc-deposit-scroll-1.png"),
        &external_oracle_fixture_bytes("pc-deposit-scroll/rom-pc-deposit-scroll-2.png"),
        &external_oracle_fixture_bytes("pc-deposit-scroll/rom-pc-deposit-scroll-3.png"),
        &external_oracle_fixture_bytes("pc-deposit-scroll/rom-pc-deposit-scroll-4.png"),
        &external_oracle_fixture_bytes("pc-deposit-scroll/rom-pc-deposit-scroll-5.png"),
        &external_oracle_fixture_bytes("pc-deposit-scroll/rom-pc-deposit-scroll-6.png"),
        &external_oracle_fixture_bytes("pc-deposit-scroll/rom-pc-deposit-scroll-7.png"),
        &external_oracle_fixture_bytes("pc-deposit-scroll/rom-pc-deposit-scroll-8.png"),
        &external_oracle_fixture_bytes("pc-deposit-scroll/rom-pc-deposit-scroll-9.png"),
        &external_oracle_fixture_bytes("pc-deposit-scroll/rom-pc-deposit-scroll-10.png"),
        &external_oracle_fixture_bytes("pc-deposit-scroll/rom-pc-deposit-scroll-11.png"),
    ];
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.session_mut().state_mut().bag.items.clear();
    for (id, quantity) in [
        ("POTION", 23),
        ("ANTIDOTE", 1),
        ("ESCAPE_ROPE", 3),
        ("REPEL", 4),
        ("SUPER_POTION", 5),
        ("HYPER_POTION", 6),
    ] {
        let item = shell.shell.runtime().data().items[id].clone();
        shell
            .shell
            .session_mut()
            .state_mut()
            .bag
            .add_item(&item, quantity)
            .unwrap();
    }
    shell.pc_item_action = Some(VisiblePlayerPcAction::DepositItem);
    open_visible_pc_item_deposit_pack(&mut shell).unwrap();
    for (index, record) in fixture["records"].as_array().unwrap().iter().enumerate() {
        match record["button"].as_str() {
            None => {}
            Some("down") => move_visible_active_field_pack_cursor(&mut shell, 1).unwrap(),
            Some("up") => move_visible_active_field_pack_cursor(&mut shell, -1).unwrap(),
            other => panic!("unexpected source scroll input {other:?}"),
        }
        let snapshot = shell.shell.snapshot().unwrap();
        let mut world = World::new();
        let mut queue = bevy::ecs::world::CommandQueue::default();
        let mut images = Assets::<Image>::default();
        let mut art = RenderedTilesetArt::default();
        let mut commands = Commands::new(&mut queue, &world);
        spawn_field_pack_screen(
            &mut commands,
            &snapshot,
            &shell,
            &mut art,
            &shell.asset_root,
            &mut images,
        )
        .unwrap();
        queue.apply(&mut world);
        let canvas = render_pc_audit_canvas(&mut world, &images, "pc-deposit-scroll");
        if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
            std::fs::create_dir_all(&directory).unwrap();
            canvas.save(PathBuf::from(directory).join(format!("pc-deposit-scroll-{index}.png"))).unwrap();
        }
        let reference = image::load_from_memory(references[index])
            .unwrap()
            .to_rgba8();
        let scale = canvas.width() / 160;
        for y in 0..144 {
            for x in 0..160 {
                assert_eq!(
                    canvas.get_pixel(x * scale, y * scale).0.map(|v| v >> 3),
                    reference.get_pixel(x, y).0.map(|v| v >> 3),
                    "frame {index}, source pixel {x},{y}"
                );
            }
        }
    }
    open_visible_field_pack_pocket(&mut shell, FieldPackPocket::Balls).unwrap();
    open_visible_field_pack_pocket(&mut shell, FieldPackPocket::Items).unwrap();
    assert_eq!(shell.field_pack_scroll_positions[0], 2);
    assert_eq!(shell.bag_cursor.as_ref().unwrap().option_index, 3);

}

#[test]
fn pack_reentry_after_removing_a_stack_preserves_the_source_screen_row() {
    // Six items + CANCEL, bottom row selected. Removing one item makes
    // InitScrollingMenuCursor reduce scroll from two to one before restoring Y.
    let mut cursor = Some(MenuCursor { surface_id: "bag:items".into(), option_index: 6 });
    let mut scroll = 2;
    move_visible_pack_cursor_slot(&mut cursor, "bag:items".into(), 6, 0,
        &mut scroll, &mut Vec::new()).unwrap();
    assert_eq!(scroll, 1);
    assert_eq!(cursor.unwrap().option_index, 5);
}

#[test]
fn pokegear_live_all_radio_stations_match_source_textboxes() {
    check_live_radio_source_fixtures(&["buena-day", "buena-night", "lucky", "oak", "pokedex", "places-people", "ben-sunday", "fern-monday"]);
}

#[test]
fn pokegear_station_loading_preserves_source_radio_music_state() {
    use crate::assets::radio_host::RadioMusicEffect;
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.pokegear_menu_open = true;
    shell.pokegear_page = PokegearPage::Radio;
    shell.pokegear_radio_station = Some("OAKS_POKEMON_TALK".into());
    load_visible_radio_broadcast(&mut shell).unwrap();
    for mode in [RadioMusicEffect::Stop, RadioMusicEffect::PokemonChannel,
        RadioMusicEffect::Restart("MUSIC_POKEMON_TALK")]
    {
        shell.pokegear_radio_broadcast.as_mut().unwrap().host.music_mode = Some(mode.clone());
        shell.pokegear_radio_station = Some("LUCKY_CHANNEL".into());
        load_visible_radio_broadcast(&mut shell).unwrap();
        assert_eq!(shell.pokegear_radio_broadcast.as_ref().unwrap().host.music_mode,
            Some(mode), "LoadStation_LuckyChannel does not write wPokegearRadioMusicPlaying");
        assert_eq!(shell.pokegear_radio_broadcast.as_ref().unwrap().playback.state.current_line, 3);
    }
}

#[test]
fn pokegear_phone_last_text_button_is_not_a_new_hangup_press() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.initialize_permanent_phone_numbers().unwrap();
    shell.pokegear_menu_open = true;
    shell.pokegear_page = PokegearPage::Phone;
    shell.pokegear_joypad.sample(crate::core::input::B_PAD_A, true);
    start_visible_pokegear_phone_call(&mut shell).unwrap();
    for _ in 0..2 {
        shell.pending_audio.clear();
        shell.transient_audio_playing = false;
        advance_visible_pokegear_phone_call(&mut shell, 1).unwrap();
    }
    assert_eq!(shell.pokegear_phone_call.as_ref().unwrap().phase, VisiblePokegearPhoneCallPhase::Calling);
    let mut keys = ButtonInput::default();
    for _ in 0..40 {
        advance_visible_script_until_player_boundary(&mut shell).unwrap();
        for _ in 0..256 {
            let snapshot = shell.shell.snapshot().unwrap();
            if visible_field_dialogue_is_fully_revealed(&shell, &snapshot) { break; }
            tick_visible_field_text_reveal(&mut shell, true).unwrap();
        }
        if visible_text_label_can_auto_continue(&shell).unwrap() {
            advance_visible_text_label(&mut shell).unwrap();
        }
        keys = ButtonInput::default();
        keys.press(KeyCode::KeyX);
        apply_visible_runtime_controls(&keys, &mut shell, true);
        assert_eq!(shell.last_error, None);
        keys.clear();
        advance_visible_pokegear_phone_call(&mut shell, 1).unwrap();
        if matches!(shell.pokegear_phone_call.as_ref().unwrap().phase, VisiblePokegearPhoneCallPhase::FinishDelay { .. }) { break; }
    }
    let snapshot = shell.shell.snapshot().unwrap();
    assert!(matches!(shell.pokegear_phone_call.as_ref().unwrap().phase, VisiblePokegearPhoneCallPhase::FinishDelay { .. }),
        "compiled phone conversation must finish at its source ten-frame hold: phase={:?} cursor={:?} label={:?} pending_label={:?} wait={:?} last_action={:?}",
        shell.pokegear_phone_call.as_ref().unwrap().phase, shell.active_script_cursor,
        snapshot.ui.text.as_ref().map(|text| &text.label), snapshot.script_events.pending_text_label,
        snapshot.ui.pending_text_wait, shell.last_runtime_action.as_ref().map(|record| &record.action));
    advance_visible_pokegear_phone_call(&mut shell, 10).unwrap();
    assert!(keys.pressed(KeyCode::KeyX));
    assert!(!keys.just_pressed(KeyCode::KeyX));
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pokegear_phone_call.as_ref().unwrap().phase, VisiblePokegearPhoneCallPhase::AwaitHangup,
        "B that finished the conversation is already in hJoyDown and must not auto-hang up");
}

#[test]
fn pokegear_radio_held_cancel_waits_for_the_source_program_call_to_return() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.pokegear_menu_open = true;
    shell.pokegear_page = PokegearPage::Radio;
    shell.pokegear_map_radio_delay = None;
    shell.pokegear_radio_station = Some("ROCKET_RADIO".into());
    load_visible_radio_broadcast(&mut shell).unwrap();
    // Source line 55 prints RocketRadioText7, including TextCommand_PAUSE.
    shell.pokegear_radio_broadcast.as_mut().unwrap().playback.state.current_line = 55;
    advance_visible_radio_broadcast(&mut shell, 1, false).unwrap();
    assert!(shell.pokegear_radio_broadcast.as_ref().unwrap().playback.call_suspended());
    let mut keys = ButtonInput::default();
    keys.press(KeyCode::KeyX);
    // PokegearRadio_Joypad cannot sample B inside its suspended FarCall.
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert!(shell.pokegear_menu_open, "B must not interrupt the source radio printer");
    keys.clear();
    for _ in 0..200 {
        advance_visible_radio_broadcast(&mut shell, 1, true).unwrap();
        if !shell.pokegear_radio_broadcast.as_ref().unwrap().playback.call_suspended() { break; }
    }
    assert!(!shell.pokegear_radio_broadcast.as_ref().unwrap().playback.call_suspended());
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert!(shell.pokegear_menu_open, "the returning FarCall still owns its completion frame");
    advance_visible_radio_broadcast(&mut shell, 1, true).unwrap();
    // The next portable joypad call tests hJoyLast, not a new press edge.
    assert!(keys.pressed(KeyCode::KeyX));
    assert!(!keys.just_pressed(KeyCode::KeyX));
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pokegear_exit, Some(VisiblePokegearExitPhase::Requested),
        "held B must request exit when the source joypad loop resumes");
}

#[test]
fn pokegear_clock_samples_held_buttons_before_right() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    for flag in ["ENGINE_MAP_CARD", "ENGINE_PHONE_CARD", "ENGINE_RADIO_CARD"] {
        shell.shell.session_mut().state_mut().flags.set_engine_flag(flag, true).unwrap();
    }
    mark_runtime_snapshot_dirty(&mut shell);
    for button in [KeyCode::KeyZ, KeyCode::KeyX, KeyCode::Enter, KeyCode::ShiftRight] {
        shell.pokegear_menu_open = true;
        shell.pokegear_page = PokegearPage::Clock;
        shell.pokegear_joypad = crate::core::input::JoyTextDelay::default();
        let mut keys = ButtonInput::default();
        keys.press(button);
        keys.press(KeyCode::ArrowRight);
        keys.clear(); // hJoyLast is held input, with no new press edge.
        apply_visible_runtime_controls(&keys, &mut shell, true);
        assert_eq!(shell.pokegear_exit, Some(VisiblePokegearExitPhase::Requested),
            "PokegearClock_Joypad must request exit for held {button:?} before examining Right");
        assert_eq!(shell.pokegear_page, PokegearPage::Clock);
        close_visible_pokegear_menu(&mut shell).unwrap();
    }
    shell.pokegear_menu_open = true;
    shell.pokegear_page = PokegearPage::Clock;
        shell.pokegear_joypad = crate::core::input::JoyTextDelay::default();
    let mut keys = ButtonInput::default();
    keys.press(KeyCode::ArrowRight);
    keys.clear();
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pokegear_page, PokegearPage::Map,
        "Clock hJoyLast Right does not wait for the generic menu repeat timer");
}

#[test]
fn pokegear_live_portable_buena_matches_source_day_and_night_screens() {
    check_live_radio_source_fixtures(&["portable-buena-day", "portable-buena-night"]);
}

#[test]
fn pokegear_cards_use_source_repeat_and_simultaneous_button_priority() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.initialize_permanent_phone_numbers().unwrap();
    for flag in ["ENGINE_MAP_CARD", "ENGINE_PHONE_CARD", "ENGINE_RADIO_CARD"] {
        shell.shell.session_mut().state_mut().flags.set_engine_flag(flag, true).unwrap();
    }
    mark_runtime_snapshot_dirty(&mut shell);
    open_visible_pokegear_menu(&mut shell).unwrap();
    shell.pokegear_page = PokegearPage::Map;
    let mut keys = ButtonInput::default();
    keys.press(KeyCode::ArrowUp);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    let first = shell.pokegear_cursor;
    keys.clear();
    for _ in 0..14 {
        apply_visible_runtime_controls(&keys, &mut shell, true);
        assert_eq!(shell.pokegear_cursor, first, "JoyTextDelay's initial 15-frame hold");
    }
    apply_visible_runtime_controls(&keys, &mut shell, true);
    let second = shell.pokegear_cursor;
    assert_ne!(second, first);
    for _ in 0..4 {
        apply_visible_runtime_controls(&keys, &mut shell, true);
        assert_eq!(shell.pokegear_cursor, second, "JoyTextDelay's 5-frame repeat");
    }
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_ne!(shell.pokegear_cursor, second);

    shell.pokegear_page = PokegearPage::Phone;
    shell.pokegear_joypad = crate::core::input::JoyTextDelay::default();
    let mut keys = ButtonInput::default();
    keys.press(KeyCode::KeyZ);
    keys.press(KeyCode::ArrowRight);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pokegear_page, PokegearPage::Phone, "Phone A precedes Right");
    assert!(shell.pokegear_phone_menu.is_some());
    shell.pokegear_phone_menu = Some(VisiblePokegearPhoneMenu {
        contact_id: "PHONE_BILL".into(), can_delete: true, cursor: 0, delete_confirmation: None,
    });
    let mut keys = ButtonInput::default();
    keys.press(KeyCode::ArrowDown);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    keys.clear();
    for _ in 0..30 { apply_visible_runtime_controls(&keys, &mut shell, true); }
    assert_eq!(shell.pokegear_phone_menu.as_ref().unwrap().cursor, 1,
        "contact submenu directions use hJoyPressed, not held-repeat hJoyLast");

    shell.pokegear_phone_menu = None;
    for phase in [VisiblePokegearPhoneCallPhase::NoServicePrompt, VisiblePokegearPhoneCallPhase::AwaitHangup] {
        shell.pokegear_phone_call = Some(VisiblePokegearPhoneCall {
            contact_id: "PHONE_MOM".into(), phase,
        });
        shell.pokegear_joypad = crate::core::input::JoyTextDelay::default();
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::KeyX);
        apply_visible_runtime_controls(&keys, &mut shell, true);
        keys.clear();
        if shell.pokegear_phone_call.is_some() {
            advance_visible_pokegear_phone_call(&mut shell, u32::from(VISIBLE_POKEGEAR_HANGUP_FRAMES)).unwrap();
        }
        apply_visible_runtime_controls(&keys, &mut shell, true);
        assert!(shell.pokegear_phone_call.is_none());
        assert!(shell.pokegear_menu_open, "held B after a phone prompt is not a new contact-list B");
        assert_eq!(shell.pokegear_page, PokegearPage::Phone);
    }
    shell.pokegear_page = PokegearPage::Radio;
    shell.pokegear_joypad = crate::core::input::JoyTextDelay::default();
    shell.pokegear_radio_tuning_knob = 40;
    shell.shell.session_mut().state_mut().radio_tuning_knob = 40;
    mark_runtime_snapshot_dirty(&mut shell);
    let mut keys = ButtonInput::default();
    keys.press(KeyCode::ArrowUp);
    keys.press(KeyCode::ArrowDown);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pokegear_radio_tuning_knob, 38,
        "AnimateTuningKnob tests Down before Up");
    assert_eq!(shell.last_error, None);
}

fn check_live_radio_source_fixtures(names: &[&str]) {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let baseline = shell.shell.session().state().clone();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../tools/asm-oracle/fixtures/radio-live");
    for &name in names {
        let directory = root.join(name);
        let trace: serde_json::Value = serde_json::from_slice(&std::fs::read(directory.join("trace.json")).unwrap()).unwrap();
        let initial = &trace["initial_state"];
        let portable_knob = trace["portable_knob"].as_u64().map(|value| value as u8);
        let prints = trace["prints"].as_array().unwrap();
        let last_print_frame = prints.last().unwrap()["end"].as_u64().unwrap();
        let first = &trace["loops"][0];
        *shell.shell.session_mut().state_mut() = baseline.clone();
        let caught = trace["caught_species"].as_u64().map(|id| {
            shell.shell.runtime().data().pokemon.values().find(|species| u64::from(species.int_id) == id).unwrap().id.clone()
        });
        let game = shell.shell.session_mut().state_mut();
        game.time.current_day = initial["day"].as_u64().unwrap() as u8;
        game.time.day_of_week = game.time.current_day % 7;
        game.time.registers.hours = prints[0]["hour"].as_u64().unwrap() as u8;
        game.player_gender = 0;
        if let Some(knob) = portable_knob {
            game.radio_tuning_knob = knob;
            for flag in ["ENGINE_POKEGEAR", "ENGINE_MAP_CARD", "ENGINE_RADIO_CARD", "ENGINE_PHONE_CARD"] {
                game.flags.set_engine_flag(flag, true).unwrap();
            }
        }
        game.pokedex.caught_species.clear();
        if let Some(species) = caught { game.pokedex.seen_species.insert(species.clone()); game.pokedex.caught_species.insert(species); }
        game.flags.clear_engine_flag("ENGINE_ROCKETS_IN_RADIO_TOWER").unwrap();
        game.flags.engine_flags.insert("STATUSFLAGS_HALL_OF_FAME_F".into(), initial["status_flags"].as_u64().unwrap() & 64 != 0);
        let badges = initial["kanto_badges"].as_u64().unwrap() as u8;
        game.badges.kanto = std::array::from_fn(|index| badges & (1 << index) != 0);
        game.lucky_number_countdown.remaining_days = initial["lucky_timer"][0].as_u64().unwrap() as u8;
        game.lucky_number_countdown.last_checked_day = initial["lucky_timer"][1].as_u64().unwrap() as u8;
        game.lucky_id_number = initial["lucky_number"].as_u64().unwrap() as u16;
        game.lucky_number_day = None;
        let password = first["buena_password"].as_u64().unwrap() as u8;
        game.buenas_password.category_index = usize::from(password >> 4);
        game.buenas_password.option_index = usize::from(password & 15);
        game.flags.set_engine_flag("ENGINE_BUENAS_PASSWORD", first["buena_generated"].as_bool().unwrap()).unwrap();
        // Legal DIV stimuli reproduce the ROM's captured choices. These are
        // deliberately not presented as captured CPU/VBlank timing. Force the
        // ADC boundary so an incorrect caller carry changes the returned byte.
        game.random_state = crate::core::random::CrystalRandomState { add: 0, sub: 255 };
        let mut add = 0u8;
        let mut sub = 255u8;
        let mut samples = Vec::new();
        for call in trace["random"].as_array().unwrap().iter().filter(|call| call["frame"].as_u64().unwrap() <= last_print_frame) {
            let carry = u8::from(call["carry_in"].as_bool().unwrap());
            let value = call["value"].as_u64().unwrap() as u8;
            samples.push(255u8.wrapping_sub(add));
            samples.push(sub.wrapping_sub(value).wrapping_sub(carry));
            add = 255u8.wrapping_add(carry);
            sub = value;
        }
        let expected_reads = samples.len();
        *shell.shell.session_mut().divider_mut_for_tests() = crate::core::random::RuntimeDividerSource::replay(samples);
        shell.pokegear_menu_open = true;
        shell.pokegear_page = PokegearPage::Radio;
        shell.pokegear_map_radio_delay = if portable_knob.is_some() { None } else { Some(0) };
        if let Some(knob) = portable_knob { shell.pokegear_radio_tuning_knob = knob; }
        shell.pokegear_radio_station = Some(match initial["radio_line"].as_u64().unwrap() {
            0 => "OAKS_POKEMON_TALK", 1 => "POKEDEX_SHOW", 2 => "POKEMON_MUSIC", 3 => "LUCKY_CHANNEL",
            4 => "BUENAS_PASSWORD", 5 => "PLACES_AND_PEOPLE", 6 => "LETS_ALL_SING", _ => unreachable!(),
        }.into());
        shell.active_pokegear_radio = None;
        load_visible_radio_broadcast(&mut shell).unwrap();
        for (index, record) in prints.iter().enumerate() {
            let mut complete = false;
            for _ in 0..700 {
                advance_visible_radio_broadcast(&mut shell, 1, false).unwrap_or_else(|error| panic!("{name} print {index}: {error:#}"));
                let state = &shell.pokegear_radio_broadcast.as_ref().unwrap().playback.state;
                if state.current_line == 84 && state.delay == 100 { complete = true; break; }
            }
            assert!(complete, "{name} print {index} did not complete");
            let broadcast = shell.pokegear_radio_broadcast.as_ref().unwrap();
            let actual = broadcast.playback.window.tiles.iter().flatten().map(|tile| format!("{tile:02x}")).collect::<String>();
            assert_eq!(actual, record["tiles_after"].as_str().unwrap(), "{name} print {index} source tiles");
            let snapshot = shell.shell.snapshot().unwrap();
            let mut world = World::new();
            let mut queue = bevy::ecs::world::CommandQueue::default();
            let mut images = Assets::<Image>::default();
            let mut art = RenderedTilesetArt::default();
            let mut commands = Commands::new(&mut queue, &world);
            spawn_field_pokegear_screen(&mut commands, &snapshot, &shell, &mut art, &shell.asset_root, &mut images).unwrap();
            queue.apply(&mut world);
            assert_eq!(art.font_error, None);
            let canvas = render_pc_audit_canvas(&mut world, &images, name);
            let reference = image::open(directory.join(format!("radio-print-{index}.png"))).unwrap().to_rgba8();
            let scale = canvas.width() / 160;
            if let Ok(output) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
                let output = PathBuf::from(output).join(name);
                std::fs::create_dir_all(&output).unwrap();
                canvas.save(output.join(format!("radio-print-{index}.png"))).unwrap();
            }
            for y in 0..144 {
                for x in 0..160 {
                    let actual = canvas.get_pixel(x * scale, y * scale).0;
                    if y < 96 && portable_knob.is_none() { assert_eq!(actual[3], 0, "{name} covers map {x},{y}"); }
                    else { assert_eq!(actual.map(|v| v >> 3), reference.get_pixel(x, y).0.map(|v| v >> 3), "{name} print {index} pixel {x},{y}"); }
                }
            }

            if index == 0 {
                let before = cached_runtime_snapshot(&mut shell).unwrap();
                let render_before = shell_render_key(&shell);
                advance_visible_radio_broadcast(&mut shell, 1, false).unwrap();
                let after = cached_runtime_snapshot(&mut shell).unwrap();
                assert!(Arc::ptr_eq(&before, &after), "{name}: a radio countdown must not rebuild the game snapshot");
                assert_eq!(render_before, shell_render_key(&shell), "{name}: a countdown must not redraw unchanged tiles");
            }

        }
        let crate::core::random::RuntimeDividerSource::Replay(divider) = &shell.shell.session().divider_for_tests() else { unreachable!() };
        assert_eq!(divider.consumed(), expected_reads, "{name} Random call count differs from the ROM trace");
    }
}

#[test]
fn pokegear_exit_retains_final_tuning_frame_and_waits_for_sound() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.pokegear_menu_open = true;
    shell.pokegear_page = PokegearPage::Radio;
    shell.shell.apply_runtime_mutation_command(crate::RuntimeMutationCommand::SetPokegearRadioTuning(
        crate::assets::RuntimePokegearRadioTuningCommand { tuning_knob: 40 },
    )).unwrap();
    shell.pokegear_radio_tuning_knob = 40;
    shell.pokegear_radio_station = Some("BUENAS_PASSWORD".into());
    let snapshot = shell.shell.snapshot().unwrap();
    sync_visible_pokegear_radio(&mut shell, &snapshot).unwrap();
    let mut keys = ButtonInput::default();
    keys.press(KeyCode::KeyX);
    keys.press(KeyCode::ArrowDown);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.last_error, None);
    assert_eq!(shell.shell.snapshot().unwrap().progression.radio_tuning_knob, 38,
        "source PlaySpriteAnimations runs after Radio_Joypad sets the EXIT bit");
    assert!(shell.pokegear_menu_open,
        "source exit retains the card through DelayFrame and WaitSFX");
    assert_eq!(shell.pokegear_exit, Some(VisiblePokegearExitPhase::Requested));
    shell.pokegear_return_start_menu_cursor = Some(MenuCursor {
        surface_id: START_MENU_SURFACE_ID.into(), option_index: 2,
    });
    advance_visible_pokegear_exit(&mut shell, 1).unwrap();
    assert_eq!(shell.pokegear_exit, Some(VisiblePokegearExitPhase::WaitSound));
    assert!(shell.pending_audio.iter().any(|command| command.audio_id == "SFX_READ_TEXT_2"));
    for _ in 0..30 { advance_visible_pokegear_exit(&mut shell, 1).unwrap(); }
    assert_eq!(shell.pokegear_exit, Some(VisiblePokegearExitPhase::WaitSound),
        "queued closing SFX must not be replaced by a fixed countdown");
    shell.pending_audio.retain(|command| matches!(command.kind, ModpackAudioKind::Music));
    shell.transient_audio_playing = true;
    advance_visible_pokegear_exit(&mut shell, 1).unwrap();
    assert_eq!(shell.pokegear_exit, Some(VisiblePokegearExitPhase::WaitSound));
    shell.transient_audio_playing = false;
    advance_visible_pokegear_exit(&mut shell, 1).unwrap();
    assert_eq!(shell.pokegear_exit, Some(VisiblePokegearExitPhase::ClearPalettes { frames_remaining: 4 }));
    assert!(!visible_pokegear_exit_palettes_are_clear(&shell), "the scanned frame precedes the requested CGB palette upload");
    advance_visible_pokegear_exit(&mut shell, 1).unwrap();
    assert!(visible_pokegear_exit_palettes_are_clear(&shell));
    let snapshot = shell.shell.snapshot().unwrap();
    let mut world = World::new();
    let mut queue = bevy::ecs::world::CommandQueue::default();
    let mut images = Assets::<Image>::default();
    let mut art = RenderedTilesetArt::default();
    let mut commands = Commands::new(&mut queue, &world);
    spawn_field_pokegear_screen(&mut commands, &snapshot, &shell, &mut art, &shell.asset_root, &mut images).unwrap();
    queue.apply(&mut world);
    let canvas = render_pc_audit_canvas(&mut world, &images, "pokegear-exit-clear");
    let source = image::load_from_memory(&external_oracle_fixture_bytes("pokegear-exit/buena-down/exit-frame-20.png")).unwrap().to_rgba8();
    let scale = canvas.width() / source.width();
    for y in 0..canvas.height() { for x in 0..canvas.width() {
        assert_eq!(canvas.get_pixel(x, y).0.map(|v| v >> 3), source.get_pixel(x / scale, y / scale).0.map(|v| v >> 3),
            "ClearPalettes source LCD pixel {x},{y}");
    }}
    for remaining in [2, 1] {
        advance_visible_pokegear_exit(&mut shell, 1).unwrap();
        assert_eq!(shell.pokegear_exit, Some(VisiblePokegearExitPhase::ClearPalettes { frames_remaining: remaining }));
        assert!(shell.pokegear_menu_open);
    }
    advance_visible_pokegear_exit(&mut shell, 1).unwrap();
    assert!(matches!(shell.pokegear_exit, Some(VisiblePokegearExitPhase::RestoreMusic { .. })));
    assert_eq!(shell.active_music.as_deref(), Some("MUSIC_NONE"));
    assert!(shell.pokegear_menu_open, "DelayFrame between MUSIC_NONE and the restored song retains the blank card");
    advance_visible_pokegear_exit(&mut shell, 1).unwrap();
    assert!(!shell.pokegear_menu_open);
    assert!(shell.pokegear_exit.is_none());
    assert_eq!(shell.active_music.as_deref(), shell.shell.current_music_id());
    assert_eq!(shell.start_menu_cursor.as_ref().unwrap().option_index, 2);
    assert!(shell.pokegear_exit_input_blocked, "returning from the submenu must not consume this frame's buttons again");
}

#[test]
fn pc_item_confirm_with_down_selects_the_original_stack() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.session_mut().state_mut().bag.pc_items.clear();
    for (id, quantity) in [("POTION", 23), ("ANTIDOTE", 1)] {
        let item = shell.shell.runtime().data().items[id].clone();
        assert!(shell.shell.session_mut().state_mut().bag.add_pc_item(&item, quantity).unwrap());
    }
    shell.pc_item_action = Some(VisiblePlayerPcAction::WithdrawItem);
    shell.pc_item_cursor = Some(MenuCursor { surface_id: "pc:items".into(), option_index: 0 });
    mark_runtime_snapshot_dirty(&mut shell);
    while shell.pc_menu_input_wait_frames > 1 {
        apply_visible_runtime_controls(&ButtonInput::default(), &mut shell, true);
    }
    let mut keys = ButtonInput::default();
    keys.press(KeyCode::KeyZ);
    keys.press(KeyCode::ArrowDown);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.last_error, None);
    let quantity = shell.pc_item_quantity.as_ref().expect("A opens the selected stack's quantity dialog");
    assert_eq!(quantity.item_id, "POTION", "ScrollingMenuJoyAction tests A before Down; the confirmation must not move first");
    assert_eq!(quantity.maximum, 23);
    shell.pc_item_quantity = None;
    shell.pc_notice = None;
    shell.field_text_reveal = None;
    while shell.pc_menu_input_wait_frames > 1 {
        apply_visible_runtime_controls(&ButtonInput::default(), &mut shell, true);
    }
    let mut keys = ButtonInput::default();
    keys.press(KeyCode::ShiftRight);
    keys.press(KeyCode::ArrowDown);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pc_item_switch_origin, Some(0), "SELECT + Down marks the original stack, as in the ROM trace");
    assert_eq!(shell.pc_item_cursor.as_ref().unwrap().option_index, 0);
    while shell.pc_menu_input_wait_frames > 1 {
        apply_visible_runtime_controls(&ButtonInput::default(), &mut shell, true);
    }
    let mut keys = ButtonInput::default();
    keys.press(KeyCode::KeyX);
    keys.press(KeyCode::ArrowDown);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pc_item_switch_origin, None, "B cancels the marked stack before any movement");
    assert_eq!(shell.pc_item_cursor.as_ref().unwrap().option_index, 0);
    while shell.pc_menu_input_wait_frames > 1 {
        apply_visible_runtime_controls(&ButtonInput::default(), &mut shell, true);
    }
    let mut keys = ButtonInput::default();
    keys.press(KeyCode::Enter);
    keys.press(KeyCode::ArrowDown);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pc_item_cursor.as_ref().unwrap().option_index, 0, "disabled START still wins over Down");
    toggle_visible_start_menu(&mut shell).unwrap();
    assert!(!visible_field_pack_is_open(&shell), "START is not a PC deposit shortcut");
    shell.pc_item_cursor = None;
    shell.storage_cursor = Some(MenuCursor { surface_id: storage_cursor_surface_id(0), option_index: 0 });
    toggle_visible_start_menu(&mut shell).unwrap();
    assert!(!shell.party_menu_open, "START is not a Bill's PC party shortcut");
    assert_eq!(shell.last_error, None);
}

#[test]
fn pc_item_quantity_cancel_wins_over_confirm_and_directions() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.session_mut().state_mut().bag.pc_items.clear();
    let potion = shell.shell.runtime().data().items["POTION"].clone();
    assert!(shell.shell.session_mut().state_mut().bag.add_pc_item(&potion, 23).unwrap());
    shell.pc_item_action = Some(VisiblePlayerPcAction::WithdrawItem);
    shell.pc_item_cursor = Some(MenuCursor { surface_id: "pc:items".into(), option_index: 0 });
    mark_runtime_snapshot_dirty(&mut shell);
    begin_visible_pc_item_quantity(&mut shell).unwrap();
    let question = shell.pc_notice.clone().unwrap();
    shell.field_text_reveal = Some(VisibleFieldTextReveal {
        text: question.clone(), page_index: 0, visible_chars: question.chars().count(), frames_until_next_char: 0,
    });
    let mut keys = ButtonInput::default();
    keys.press(KeyCode::KeyZ);
    keys.press(KeyCode::KeyX);
    keys.press(KeyCode::ArrowDown);
    shell.field_text_consumed_a = true;
    shell.field_text_consumed_b = true;
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert!(shell.pc_item_quantity.is_some(), "a printer-consumed edge must not dismiss the newly ready selector");
    assert_eq!(shell.shell.session().state().bag.pc_item_quantity(&potion), 23);
    shell.field_text_consumed_a = false;
    shell.field_text_consumed_b = false;
    keys.clear();
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert!(shell.pc_item_quantity.is_some(), "held text buttons are not new selector presses");
    assert_eq!(shell.shell.session().state().bag.pc_item_quantity(&potion), 23);
    apply_visible_runtime_controls(&ButtonInput::default(), &mut shell, true);
    keys = ButtonInput::default();
    keys.press(KeyCode::KeyZ);
    keys.press(KeyCode::KeyX);
    keys.press(KeyCode::ArrowDown);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.last_error, None);
    assert_eq!(shell.shell.session().state().bag.pc_item_quantity(&potion), 23,
        "BuySellToss_InterpretJoypad tests B before A and Down, so cancel must not withdraw anything");
    assert!(shell.pc_item_quantity.is_none());
    begin_visible_pc_item_quantity(&mut shell).unwrap();
    shell.field_text_reveal = Some(VisibleFieldTextReveal {
        text: question.clone(), page_index: 0, visible_chars: 1, frames_until_next_char: 0,
    });
    let mut keys = ButtonInput::default();
    keys.press(KeyCode::KeyZ);
    keys.press(KeyCode::ArrowDown);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pc_item_quantity.as_ref().unwrap().quantity, 1,
        "the question printer owns input before the quantity selector");
    assert_eq!(shell.shell.session().state().bag.pc_item_quantity(&potion), 23);
    shell.field_text_reveal.as_mut().unwrap().visible_chars = question.chars().count();
    shell.field_text_consumed_a = true;
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pc_item_quantity.as_ref().unwrap().quantity, 1,
        "finishing the question must not reuse its A press to confirm");
    shell.field_text_consumed_a = false;
    keys.clear();
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert!(shell.pc_item_quantity.is_some(), "holding the question's A must not confirm after the consumed flag clears");
    assert_eq!(shell.shell.session().state().bag.pc_item_quantity(&potion), 23);
    apply_visible_runtime_controls(&ButtonInput::default(), &mut shell, true);
    keys = ButtonInput::default();
    keys.press(KeyCode::KeyZ);
    keys.press(KeyCode::ArrowDown);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.shell.session().state().bag.pc_item_quantity(&potion), 22,
        "A confirms the displayed one item before Down could wrap it to 23");
    assert!(shell.pc_item_quantity.is_none());
    assert_eq!(shell.last_error, None);
}

#[test]
fn pc_directions_follow_the_source_owner_priority() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.session_mut().state_mut().bag.pc_items.clear();
    for (id, quantity) in [("POTION", 23), ("ANTIDOTE", 1), ("POKE_BALL", 12)] {
        let item = shell.shell.runtime().data().items[id].clone();
        assert!(shell.shell.session_mut().state_mut().bag.add_pc_item(&item, quantity).unwrap());
    }
    shell.pc_item_action = Some(VisiblePlayerPcAction::WithdrawItem);
    shell.pc_item_cursor = Some(MenuCursor { surface_id: "pc:items".into(), option_index: 1 });
    mark_runtime_snapshot_dirty(&mut shell);
    let mut keys = ButtonInput::default();
    keys.press(KeyCode::ArrowLeft);
    keys.press(KeyCode::ArrowDown);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pc_item_cursor.as_ref().unwrap().option_index, 1,
        "ScrollingMenuJoyAction tests disabled Left before Down");
    shell.pc_item_cursor.as_mut().unwrap().option_index = 0;
    shell.pc_item_quantity = Some(VisiblePcItemQuantity {
        action: VisiblePlayerPcAction::WithdrawItem, item_id: "POTION".into(),
        stack_index: 0, quantity: 10, maximum: 23,
    });
    let question = "How many do you\nwant to withdraw?";
    shell.pc_notice = Some(question.into());
    shell.field_text_reveal = Some(VisibleFieldTextReveal {
        text: question.into(), page_index: 0, visible_chars: question.chars().count(), frames_until_next_char: 0,
    });
    let mut keys = ButtonInput::default();
    keys.press(KeyCode::ArrowDown);
    keys.press(KeyCode::ArrowUp);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pc_item_quantity.as_ref().unwrap().quantity, 9,
        "BuySellToss_InterpretJoypad tests Down before Up");
    apply_visible_runtime_controls(&ButtonInput::default(), &mut shell, true);
    let mut keys = ButtonInput::default();
    keys.press(KeyCode::ArrowDown);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pc_item_quantity.as_ref().unwrap().quantity, 8);
    keys.clear();
    keys.press(KeyCode::ArrowUp);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pc_item_quantity.as_ref().unwrap().quantity, 7,
        "new Up does not outrank an already-held Down in hJoyLast");
    keys.clear();
    for _ in 0..14 {
        apply_visible_runtime_controls(&keys, &mut shell, true);
        assert_eq!(shell.pc_item_quantity.as_ref().unwrap().quantity, 7,
            "quantity JoyTextDelay initial repeat is fifteen frames");
    }
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pc_item_quantity.as_ref().unwrap().quantity, 6);
    for _ in 0..4 {
        apply_visible_runtime_controls(&keys, &mut shell, true);
        assert_eq!(shell.pc_item_quantity.as_ref().unwrap().quantity, 6);
    }
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pc_item_quantity.as_ref().unwrap().quantity, 5);
    keys.press(KeyCode::ShiftRight);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pc_item_quantity.as_ref().unwrap().quantity, 4,
        "fresh ignored SELECT still exposes held directions through JoyTextDelay");
    assert_eq!(shell.last_error, None);
}

#[test]
fn pc_quantity_releasing_one_direction_preserves_the_source_repeat_counter() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let question = "How many do you\nwant to withdraw?";
    shell.pc_item_quantity = Some(VisiblePcItemQuantity {
        action: VisiblePlayerPcAction::WithdrawItem, item_id: "POTION".into(),
        stack_index: 0, quantity: 10, maximum: 23,
    });
    shell.pc_notice = Some(question.into());
    shell.field_text_reveal = Some(VisibleFieldTextReveal {
        text: question.into(), page_index: 0, visible_chars: question.chars().count(), frames_until_next_char: 0,
    });
    let mut keys = ButtonInput::default();
    keys.press(KeyCode::ArrowDown);
    keys.press(KeyCode::ArrowUp);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pc_item_quantity.as_ref().unwrap().quantity, 9);
    keys.clear();
    for frame in 1..=15 {
        if frame == 6 { keys.release(KeyCode::ArrowDown); }
        apply_visible_runtime_controls(&keys, &mut shell, true);
        assert_eq!(shell.pc_item_quantity.as_ref().unwrap().quantity, if frame == 15 { 10 } else { 9 },
            "GetJoypad release must not restart wTextDelayFrames at source frame {frame}");
        keys.clear();
    }
    assert_eq!(shell.last_error, None);
}

#[test]
fn pokegear_delete_confirmation_buttons_precede_directions() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.initialize_permanent_phone_numbers().unwrap();
    {
        let state = shell.shell.session_mut().state_mut();
        state.flags.set_engine_flag("ENGINE_PHONE_CARD", true).unwrap();
        state.script_runtime.phone_numbers.insert("PHONE_BILL".into());
        state.script_runtime.phone_number_order.push(Some("PHONE_BILL".into()));
    }
    mark_runtime_snapshot_dirty(&mut shell);
    open_visible_pokegear_menu(&mut shell).unwrap();
    shell.pokegear_page = PokegearPage::Phone;
    for buttons in [[KeyCode::KeyZ, KeyCode::ArrowUp], [KeyCode::KeyZ, KeyCode::KeyX]] {
        shell.pokegear_phone_menu = Some(VisiblePokegearPhoneMenu {
            contact_id: "PHONE_BILL".into(), can_delete: true, cursor: 1,
            delete_confirmation: Some(if buttons[1] == KeyCode::ArrowUp { 1 } else { 0 }),
        });
        shell.pokegear_joypad = crate::core::input::JoyTextDelay::default();
        let mut keys = ButtonInput::default();
        for button in buttons { keys.press(button); }
        apply_visible_runtime_controls(&keys, &mut shell, true);
        assert!(shell.pokegear_phone_menu.is_none(), "YesNoBox must close");
        assert!(shell.shell.session().state().script_runtime.phone_numbers.contains("PHONE_BILL"),
            "VerticalMenu confirms the original NO before Up and treats A+B as cancel");
        assert_eq!(shell.last_error, None);
        keys.clear();
        apply_visible_runtime_controls(&keys, &mut shell, true);
        assert!(shell.pokegear_phone_menu.is_none(), "held A must not reopen the contact submenu");
        assert!(shell.pokegear_menu_open, "held B must not close the restored phone card");
    }
    for ignored in [KeyCode::Enter, KeyCode::ShiftRight, KeyCode::ArrowLeft, KeyCode::ArrowRight] {
        shell.pokegear_phone_menu = Some(VisiblePokegearPhoneMenu {
            contact_id: "PHONE_BILL".into(), can_delete: true, cursor: 1, delete_confirmation: Some(0),
        });
        shell.pokegear_joypad = crate::core::input::JoyTextDelay::default();
        let mut keys = ButtonInput::default();
        keys.press(ignored);
        keys.press(KeyCode::ArrowDown);
        apply_visible_runtime_controls(&keys, &mut shell, true);
        assert_eq!(shell.pokegear_phone_menu.as_ref().unwrap().delete_confirmation, Some(0),
            "disabled {ignored:?} still has priority over Down");
    }
}

#[test]
fn pokegear_delete_cancellation_retains_the_original_question() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.initialize_permanent_phone_numbers().unwrap();
    shell.shell.session_mut().state_mut().flags.set_engine_flag("ENGINE_PHONE_CARD", true).unwrap();
    mark_runtime_snapshot_dirty(&mut shell);
    open_visible_pokegear_menu(&mut shell).unwrap();
    shell.pokegear_page = PokegearPage::Phone;
    let snapshot = shell.shell.snapshot().unwrap();
    for cancel_with_b in [false, true] {
        shell.pokegear_phone_menu = Some(VisiblePokegearPhoneMenu {
            contact_id: "PHONE_BILL".into(), can_delete: true, cursor: 1,
            delete_confirmation: Some(1),
        });
        let question = visible_pokegear_phone_prompt(&snapshot, &shell).unwrap();
        assert!(question.contains("Delete this stored"));
        if cancel_with_b { press_visible_b_button(&mut shell).unwrap(); }
        else { confirm_visible_pokegear_phone_menu(&mut shell).unwrap(); }
        assert!(shell.pokegear_phone_menu.is_none());
        assert_eq!(visible_pokegear_phone_prompt(&snapshot, &shell).unwrap(), question,
            ".CancelDelete retains the LCD question after NO or B");
        move_visible_pokegear_cursor(&mut shell, 1).unwrap();
        assert_eq!(visible_pokegear_phone_prompt(&snapshot, &shell).unwrap(), question,
            "moving through contacts does not print AskWhoCallText");
    }
}

#[test]
fn pc_item_list_does_not_sample_a_press_during_redraw_waits() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.session_mut().state_mut().bag.pc_items.clear();
    for id in ["POTION", "ANTIDOTE", "POKE_BALL"] {
        let item = shell.shell.runtime().data().items[id].clone();
        assert!(shell.shell.session_mut().state_mut().bag.add_pc_item(&item, 23).unwrap());
    }
    shell.pc_item_action = Some(VisiblePlayerPcAction::WithdrawItem);
    shell.pc_item_cursor = Some(MenuCursor { surface_id: "pc:items".into(), option_index: 0 });
    mark_runtime_snapshot_dirty(&mut shell);
    let mut keys = ButtonInput::default();
    keys.press(KeyCode::ArrowDown);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pc_item_cursor.as_ref().unwrap().option_index, 1);
    // _ScrollingMenu.zero waits three frames after drawing; the following
    // MenuJoypadLoop.BGMap_OAM waits four more before GetJoypad runs again.
    keys = ButtonInput::default();
    keys.press(KeyCode::KeyZ);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert!(shell.pc_item_quantity.is_none(), "a one-frame A pulse during redraw must not select an item");
    assert_eq!(shell.pc_joypad.down, GameButton::Down.pad_bit(), "GetJoypad mirrors stay unchanged during upload");
    for _ in 0..6 { apply_visible_runtime_controls(&ButtonInput::default(), &mut shell, true); }
    assert!(shell.pc_item_quantity.is_none(), "an unsampled pulse must not be buffered into the next menu poll");
    assert_eq!(shell.pc_item_cursor.as_ref().unwrap().option_index, 1);
    // After the next idle poll, hold A through the following WaitBGMap.
    // It must be recognized at the real poll despite no new Bevy edge then.
    let mut keys = ButtonInput::default();
    keys.press(KeyCode::KeyZ);
    for _ in 0..3 {
        apply_visible_runtime_controls(&keys, &mut shell, true);
        keys.clear();
        assert!(shell.pc_item_quantity.is_none());
    }
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pc_item_quantity.as_ref().unwrap().item_id, "ANTIDOTE");

}

#[test]
fn mailbox_list_preserves_the_source_four_rows_cancel_and_direction_limits() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let mail = shell.shell.session().state().storage.party.pokemon[0].as_ref().unwrap().mail.clone().unwrap();
    shell.shell.session_mut().state_mut().mailbox = (0..6).map(|index| {
        let mut mail = mail.clone();
        mail.author = format!("MAIL{index}");
        crate::core::state::MailboxMail { item_id: "FLOWER_MAIL".into(), mail }
    }).collect();
    shell.mailbox_cursor = Some(MenuCursor { surface_id: "pc:mailbox".into(), option_index: 0 });
    mark_runtime_snapshot_dirty(&mut shell);
    move_visible_primary_cursor_up(&mut shell).unwrap();
    assert_eq!(shell.mailbox_cursor.as_ref().unwrap().option_index, 0,
        "MailboxPC scrolling menu does not wrap at its first message");
    move_visible_primary_cursor_down(&mut shell).unwrap();
    move_visible_primary_cursor_left(&mut shell).unwrap();
    move_visible_primary_cursor_right(&mut shell).unwrap();
    assert_eq!(shell.mailbox_cursor.as_ref().unwrap().option_index, 1,
        "mailbox does not enable horizontal input");
    for _ in 0..4 { move_visible_primary_cursor_down(&mut shell).unwrap(); }
    let snapshot = shell.shell.snapshot().unwrap();
    let entries = visible_scene_dialog_entries(&snapshot, &shell).unwrap();
    assert_eq!(entries, [" MAIL2", " MAIL3", " MAIL4", ">MAIL5"],
        "four-row window must scroll to keep the selected author visible");
    confirm_visible_mailbox_selection(&mut shell).unwrap();
    move_visible_primary_cursor_up(&mut shell).unwrap();
    move_visible_primary_cursor_right(&mut shell).unwrap();
    assert_eq!(shell.mailbox_action_cursor.as_ref().unwrap().option_index, 0,
        "mailbox submenu neither wraps nor accepts horizontal navigation");
    shell.mailbox_action_cursor.as_mut().unwrap().option_index = 2;
    confirm_visible_mailbox_action(&mut shell).unwrap();
    assert!(shell.party_menu_open);
    press_visible_b_button(&mut shell).unwrap();
    assert_eq!(shell.mailbox_cursor.as_ref().unwrap().option_index, 5,
        "cancelling ATTACH MAIL restores the selected letter");
    move_visible_primary_cursor_down(&mut shell).unwrap();
    assert_eq!(shell.mailbox_cursor.as_ref().unwrap().option_index, 6,
        "ScrollingMenu includes CANCEL after the final message");
    move_visible_primary_cursor_down(&mut shell).unwrap();
    assert_eq!(shell.mailbox_cursor.as_ref().unwrap().option_index, 6);
    assert_eq!(visible_scene_dialog_entries(&snapshot, &shell).unwrap().last().unwrap(), ">CANCEL");
    shell.shell.session_mut().state_mut().mailbox.clear();
    mark_runtime_snapshot_dirty(&mut shell);
    restore_visible_mailbox_position(&mut shell, 6).unwrap();
    let empty = shell.shell.snapshot().unwrap();
    assert_eq!(visible_scene_dialog_entries(&empty, &shell).unwrap(), [">CANCEL"],
        "MailboxPC.loop retains a CANCEL-only list after its last letter is removed");
    confirm_visible_mailbox_selection(&mut shell).unwrap();
    assert!(shell.mailbox_cursor.is_none());
    assert_eq!(shell.player_pc_action_cursor.as_ref().unwrap().option_index, 3);
}

#[test]
fn mailbox_attach_success_waits_on_party_screen_before_returning() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let pokemon = shell.shell.session_mut().state_mut().storage.party.pokemon[0].as_mut().unwrap();
    let mail = pokemon.mail.take().unwrap();
    pokemon.item = None;
    shell.shell.session_mut().state_mut().mailbox = vec![crate::core::state::MailboxMail {
        item_id: "FLOWER_MAIL".into(), mail,
    }];
    shell.mailbox_cursor = Some(MenuCursor { surface_id: "pc:mailbox".into(), option_index: 0 });
    mark_runtime_snapshot_dirty(&mut shell);
    confirm_visible_mailbox_selection(&mut shell).unwrap();
    shell.mailbox_action_cursor.as_mut().unwrap().option_index = 2;
    confirm_visible_mailbox_action(&mut shell).unwrap();
    attach_visible_mailbox_mail(&mut shell).unwrap();
    assert!(shell.party_menu_open,
        "MailboxPC.AttachMail PrintText waits on the party screen before CloseSubmenu");
    assert!(shell.mailbox_cursor.is_none(), "the hidden mailbox cannot own this prompt");
    assert_eq!(shell.pc_notice.as_deref(), Some("The MAIL was moved\nfrom the MAILBOX."));
    assert!(shell.shell.session().state().mailbox.is_empty());
    let text = shell.pc_notice.clone().unwrap();
    shell.field_text_reveal = Some(VisibleFieldTextReveal {
        visible_chars: text.chars().count(), text, page_index: 0, frames_until_next_char: 0,
    });
    press_visible_a_button(&mut shell).unwrap();
    assert!(!shell.party_menu_open);
    assert!(shell.pc_notice.is_none());
    assert_eq!(shell.mailbox_cursor.as_ref().unwrap().option_index, 0);
    assert_eq!(visible_scene_dialog_entries(&shell.shell.snapshot().unwrap(), &shell).unwrap(), [">CANCEL"]);
}

#[test]
fn mailbox_buttons_choose_before_directions_and_cancel_wins_in_submenu() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let mail = shell.shell.session().state().storage.party.pokemon[0].as_ref().unwrap().mail.clone().unwrap();
    shell.shell.session_mut().state_mut().mailbox = (0..2).map(|_| crate::core::state::MailboxMail {
        item_id: "FLOWER_MAIL".into(), mail: mail.clone(),
    }).collect();
    shell.mailbox_cursor = Some(MenuCursor { surface_id: "pc:mailbox".into(), option_index: 0 });
    mark_runtime_snapshot_dirty(&mut shell);
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::KeyZ);
    keys.press(KeyCode::ArrowDown);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.mailbox_cursor.as_ref().unwrap().option_index, 0,
        "ScrollingMenuJoyAction resolves A before Down");
    assert!(shell.mailbox_action_cursor.is_some());
    keys.reset_all();
    for _ in 0..8 { apply_visible_runtime_controls(&keys, &mut shell, true); }
    keys.press(KeyCode::KeyZ);
    keys.press(KeyCode::KeyX);
    keys.press(KeyCode::ArrowDown);
    for _ in 0..8 {
        apply_visible_runtime_controls(&keys, &mut shell, true);
        keys.clear();
        if shell.mailbox_action_cursor.is_none() { break; }
    }
    assert!(shell.mailbox_action_cursor.is_none(), "VerticalMenu exits on buttons before directions");
    assert!(shell.pending_mail_read.is_none(), "VerticalMenu gives B priority over A");
    assert_eq!(shell.mailbox_cursor.as_ref().unwrap().option_index, 0);
    keys.reset_all();
    for _ in 0..8 { apply_visible_runtime_controls(&keys, &mut shell, true); }
    confirm_visible_mailbox_selection(&mut shell).unwrap();
    keys.press(KeyCode::ArrowDown);
    for _ in 0..40 {
        apply_visible_runtime_controls(&keys, &mut shell, true);
        keys.clear();
    }
    assert_eq!(shell.mailbox_action_cursor.as_ref().unwrap().option_index, 1,
        "ScrollingMenu clears hInMenu before VerticalMenu, so held directions do not repeat");
}

#[test]
fn mailbox_confirmation_keeps_its_choice_and_waits_fifteen_vblanks() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let mail = shell.shell.session().state().storage.party.pokemon[0].as_ref().unwrap().mail.clone().unwrap();
    shell.shell.session_mut().state_mut().mailbox = (0..2).map(|_| crate::core::state::MailboxMail {
        item_id: "FLOWER_MAIL".into(), mail: mail.clone(),
    }).collect();
    shell.mailbox_cursor = Some(MenuCursor { surface_id: "pc:mailbox".into(), option_index: 0 });
    mark_runtime_snapshot_dirty(&mut shell);
    for (choice, direction, cancel, remaining) in [(1, KeyCode::ArrowUp, false, 2),
        (0, KeyCode::ArrowDown, true, 2), (0, KeyCode::ArrowDown, false, 1)] {
        confirm_visible_mailbox_selection(&mut shell).unwrap();
        shell.mailbox_action_cursor.as_mut().unwrap().option_index = 1;
        confirm_visible_mailbox_action(&mut shell).unwrap();
        shell.yes_no_cursor.as_mut().unwrap().option_index = choice;
        let text = shell.pc_notice.clone().unwrap();
        shell.field_text_reveal = Some(VisibleFieldTextReveal {
            visible_chars: text.chars().count(), text, page_index: 0, frames_until_next_char: 0,
        });
        let mut keys = ButtonInput::<KeyCode>::default();
        for _ in 0..8 { apply_visible_runtime_controls(&keys, &mut shell, true); }
        keys.press(if choice == 0 { KeyCode::ArrowUp } else { KeyCode::ArrowDown });
        apply_visible_runtime_controls(&keys, &mut shell, true);
        assert_eq!(shell.yes_no_cursor.as_ref().unwrap().option_index, choice,
            "YesNoMenuHeader has no wrap flag");
        keys.reset_all();
        for _ in 0..8 { apply_visible_runtime_controls(&keys, &mut shell, true); }
        keys.press(KeyCode::KeyZ);
        if cancel { keys.press(KeyCode::KeyX); }
        keys.press(direction);
        apply_visible_runtime_controls(&keys, &mut shell, true);
        assert!(shell.pc_confirmation.is_some(),
            "InterpretTwoOptionMenu delays 15 VBlanks before closing YesNoBox");
        assert_eq!(shell.yes_no_cursor.as_ref().unwrap().option_index, choice,
            "A chooses the displayed answer before a simultaneous direction");
        press_visible_b_button(&mut shell).unwrap();
        press_visible_a_button(&mut shell).unwrap();
        assert!(shell.pc_confirmation.is_some(), "button helpers cannot bypass the source closing wait");
        assert_eq!(shell.shell.session().state().mailbox.len(), 2);
        keys.clear();
        for _ in 0..14 {
            apply_visible_runtime_controls(&keys, &mut shell, true);
            assert!(shell.pc_confirmation.is_some());
            assert_eq!(shell.shell.session().state().mailbox.len(), 2);
        }
        apply_visible_runtime_controls(&keys, &mut shell, true);
        assert!(shell.pc_confirmation.is_none());
        assert_eq!(shell.shell.session().state().mailbox.len(), remaining);
    }
    assert_eq!(shell.pc_notice.as_deref(), Some("The cleared MAIL\nwas put away."));
}

#[test]
fn mailbox_reader_close_does_not_reuse_held_b_on_the_restored_list() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let mail = shell.shell.session().state().storage.party.pokemon[0].as_ref().unwrap().mail.clone().unwrap();
    shell.shell.session_mut().state_mut().mailbox = vec![crate::core::state::MailboxMail {
        item_id: "FLOWER_MAIL".into(), mail,
    }];
    shell.mailbox_cursor = Some(MenuCursor { surface_id: "pc:mailbox".into(), option_index: 0 });
    mark_runtime_snapshot_dirty(&mut shell);
    confirm_visible_mailbox_selection(&mut shell).unwrap();
    confirm_visible_mailbox_action(&mut shell).unwrap();
    assert!(shell.pending_mail_read.is_some());
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::KeyX);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert!(shell.pending_mail_read.is_none());
    for _ in 0..20 {
        keys.clear();
        apply_visible_runtime_controls(&keys, &mut shell, true);
        assert!(shell.mailbox_cursor.is_some(),
            "ReadAnyMail.GetJoypad must retain B history when MailboxPC resumes");
    }
    keys.reset_all();
    for _ in 0..8 { apply_visible_runtime_controls(&keys, &mut shell, true); }
    let text = "The cleared MAIL\nwas put away.".to_string();
    shell.pc_notice = Some(text.clone());
    shell.field_text_reveal = Some(VisibleFieldTextReveal {
        visible_chars: text.chars().count(), text, page_index: 0, frames_until_next_char: 0,
    });
    keys.press(KeyCode::KeyX);
    apply_visible_runtime_controls(&keys, &mut shell, false);
    assert!(shell.pc_notice.is_none());
    for _ in 0..20 {
        keys.clear();
        apply_visible_runtime_controls(&keys, &mut shell, true);
        assert!(shell.mailbox_cursor.is_some(), "PC notice dismissal also preserves GetJoypad history");
    }
    keys.reset_all();
    for _ in 0..8 { apply_visible_runtime_controls(&keys, &mut shell, true); }
    keys.press(KeyCode::KeyX);
    for _ in 0..8 {
        apply_visible_runtime_controls(&keys, &mut shell, true);
        keys.clear();
        if shell.mailbox_cursor.is_none() { break; }
    }
    assert!(shell.mailbox_cursor.is_none(), "a fresh B press still exits the mailbox");
}

#[test]
fn unown_puzzle_uses_start_to_quit_and_b_to_acknowledge_success() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let puzzle = VisibleUnownPuzzle {
        puzzle_id: "KABUTO".into(), layout: [[0; 6]; 6], holding_piece: None,
        cursor_x: 0, cursor_y: 0, solved: false,
    };
    shell.visible_unown_puzzle = Some(puzzle.clone());
    set_visible_script_numeric_value(&mut shell, 0);
    press_visible_b_button(&mut shell).unwrap();
    assert!(shell.visible_unown_puzzle.is_some(), "UnownPuzzleJumptable ignores B before completion");
    assert!(has_visible_shell_start_action(&mut shell), "START belongs to the puzzle");
    press_visible_start_button(&mut shell).unwrap();
    assert!(shell.visible_unown_puzzle.is_none(), "START quits the unsolved puzzle");
    shell.visible_unown_puzzle = Some(VisibleUnownPuzzle { solved: true, ..puzzle });
    set_visible_script_numeric_value(&mut shell, 1);
    press_visible_start_button(&mut shell).unwrap();
    assert!(shell.visible_unown_puzzle.is_some(), "SimpleWaitPressAorB does not accept START");
    press_visible_b_button(&mut shell).unwrap();
    assert!(shell.visible_unown_puzzle.is_none());
    assert_eq!(shell.shell.session().state().script_runtime.script_value.as_deref(), Some("1"),
        "B acknowledges completion without converting the chamber result to cancellation");
}

#[test]
fn card_flip_yes_no_choices_stop_at_the_source_menu_edges() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.visible_card_flip = Some(VisibleCardFlip {
        phase: VisibleCardFlipPhase::AskPlay, animation: VisibleCardFlipAnimation::None,
        yes_no_index: 0, which_card: 0, bet_x: 2, bet_y: 2, round: 0,
        face_card: None, coins: 99, payout: 0, deck: Vec::new(), revealed: vec![false; 24],
        message: "PLAY WITH THREE COINS?".into(),
    });
    let audio_count = shell.pending_audio.len();
    for phase in [VisibleCardFlipPhase::AskPlay, VisibleCardFlipPhase::PlayAgain] {
        shell.visible_card_flip.as_mut().unwrap().phase = phase;
        shell.visible_card_flip.as_mut().unwrap().yes_no_index = 0;
        move_visible_card_flip_cursor(&mut shell, 0, -1).unwrap();
        assert_eq!(shell.visible_card_flip.as_ref().unwrap().yes_no_index, 0,
            "YesNoBox does not wrap Up from YES to NO");
        move_visible_card_flip_cursor(&mut shell, 0, 1).unwrap();
        assert_eq!(shell.visible_card_flip.as_ref().unwrap().yes_no_index, 1);
        move_visible_card_flip_cursor(&mut shell, 0, 1).unwrap();
        assert_eq!(shell.visible_card_flip.as_ref().unwrap().yes_no_index, 1,
            "YesNoBox does not wrap Down from NO to YES");
    }
    assert_eq!(shell.pending_audio.len(), audio_count, "cursor movement is silent");
}

#[test]
fn slot_machine_yes_no_choices_stop_at_the_source_menu_edges() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.visible_slot_machine = Some(VisibleSlotMachine {
        phase: VisibleSlotMachinePhase::PlayAgain, animation: VisibleSlotMachineAnimation::None,
        yes_no_index: 0, bet: 3, coins: 99, payout: 0, offsets: [14; 3],
        spin_ticks: [0; 3], spinning: [false; 3], next_reel: 1,
        actor: None, secondary_actor: None, background_y_offset: 0,
        windows: visible_slot_windows([14; 3]), message: "PLAY AGAIN?".into(),
    });
    let audio_count = shell.pending_audio.len();
    change_visible_slot_machine_bet(&mut shell, 1).unwrap();
    assert_eq!(shell.visible_slot_machine.as_ref().unwrap().yes_no_index, 0);
    change_visible_slot_machine_bet(&mut shell, -1).unwrap();
    assert_eq!(shell.visible_slot_machine.as_ref().unwrap().yes_no_index, 1);
    change_visible_slot_machine_bet(&mut shell, -1).unwrap();
    assert_eq!(shell.visible_slot_machine.as_ref().unwrap().yes_no_index, 1);
    assert_eq!(shell.pending_audio.len(), audio_count, "cursor movement is silent");
}

#[test]
fn held_direction_survives_a_press_between_simulation_ticks() {
    let mut runtime_shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::ArrowRight);
    // The render frame saw the edge, but no simulation tick ran in it.
    keys.clear();
    sync_overworld_held_directions(&keys, &mut runtime_shell, false);
    assert_eq!(runtime_shell.overworld_held_directions, VecDeque::from([GameButton::Right]));
    keys.press(KeyCode::ArrowUp);
    sync_overworld_held_directions(&keys, &mut runtime_shell, false);
    keys.clear();
    sync_overworld_held_directions(&keys, &mut runtime_shell, false);
    assert_eq!(runtime_shell.overworld_held_directions.back(), Some(&GameButton::Up));
    keys.release(KeyCode::ArrowUp);
    sync_overworld_held_directions(&keys, &mut runtime_shell, false);
    assert_eq!(runtime_shell.overworld_held_directions, VecDeque::from([GameButton::Right]));
}

#[test]
fn dialogue_regression_pickup_receipt_scrolls_to_pocket_and_waits() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    for item in ["BERRY", "ANTIDOTE"] {
        shell.field_notice = Some(format!("CHRIS put the\n{item} in\nthe ITEM POCKET."));
        shell.field_text_reveal = None;
        let snapshot = shell.shell.presentation_snapshot().unwrap();
        assert_eq!(visible_field_dialog_pages(&snapshot, &shell).unwrap(),
            [format!("CHRIS put the\n{item} in"), format!("{item} in\nthe ITEM POCKET.")]);
        for _ in 0..128 { tick_visible_field_text_reveal(&mut shell, true).unwrap(); }
        assert!(!visible_field_dialogue_is_entirely_consumed(&shell, &snapshot));
        press_visible_a_button(&mut shell).unwrap();
        let reveal = shell.field_text_reveal.as_ref().unwrap();
        assert_eq!(reveal.page_index, 1);
        assert_eq!(reveal.visible_chars, format!("{item} in\n").chars().count());
        assert!(shell.field_notice.is_some());
        for _ in 0..128 { tick_visible_field_text_reveal(&mut shell, true).unwrap(); }
        assert_eq!(visible_scene_dialog_entries(&snapshot, &shell).unwrap(), [format!("{item} in"), "the ITEM POCKET.".into()]);
        assert!(shell.field_notice.is_some(), "completed receipt remains until acknowledged");
        press_visible_a_button(&mut shell).unwrap();
        assert!(shell.field_notice.is_none());
    }
}

#[test]
fn dialogue_regression_new_page_cannot_borrow_previous_printer_progress() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.field_text_reveal = Some(VisibleFieldTextReveal {
        text: "Your POKéMON are\nfully healed.".into(), page_index: 0,
        visible_chars: 100, frames_until_next_char: 0,
    });
    assert_eq!(visible_revealed_field_dialog_text(&shell, "We hope to see you\nagain."), "");
    shell.field_text_reveal = Some(VisibleFieldTextReveal {
        text: "CHRIS put the\nBERRY in\u{1e}BERRY in\nthe ITEM POCKET.".into(),
        page_index: 1, visible_chars: 9, frames_until_next_char: 0,
    });
    assert_eq!(visible_revealed_field_dialog_text(&shell, "BERRY in\nthe ITEM POCKET."), "BERRY in\n");
}

#[test]
fn pokegear_start_menu_confirmation_must_be_released_before_clock_exit() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.session_mut().state_mut().flags.set_engine_flag("ENGINE_POKEGEAR", true).unwrap();
    shell.shell.session_mut().state_mut().flags.set_engine_flag("ENGINE_MAP_CARD", true).unwrap();
    mark_runtime_snapshot_dirty(&mut shell);
    if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        shell.shell.save(PathBuf::from(directory).join("pokegear-browser.crystalsave"))
            .expect("save isolated browser verification fixture");
    }
    toggle_visible_start_menu(&mut shell).unwrap();
    for _ in 0..8 {
        if selected_visible_start_menu_option(&mut shell).unwrap() == StartMenuOption::Pokegear { break; }
        move_visible_start_menu_cursor(&mut shell, 1).unwrap();
    }
    assert_eq!(selected_visible_start_menu_option(&mut shell).unwrap(), StartMenuOption::Pokegear);
    let mut keys = ButtonInput::default();
    keys.press(KeyCode::KeyZ);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert!(shell.pokegear_menu_open);
    keys.clear();
    for _ in 0..120 {
        apply_visible_runtime_controls(&keys, &mut shell, true);
        assert!(shell.pokegear_exit.is_none(), "the opening touch hold must not exit the clock");
    }
    let snapshot = shell.shell.snapshot().unwrap();
    let mut world = World::new();
    let mut queue = bevy::ecs::world::CommandQueue::default();
    let mut images = Assets::<Image>::default();
    let mut art = RenderedTilesetArt::default();
    spawn_field_pokegear_screen(
        &mut Commands::new(&mut queue, &world), &snapshot, &shell,
        &mut art, &shell.asset_root, &mut images,
    ).expect("the held-open clock must render without an error banner");
    queue.apply(&mut world);
    assert!(world.query_filtered::<Entity, With<FieldCommandMarker>>().iter(&world).count() > 0);
    if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        render_pc_audit_canvas(&mut world, &images, "pokegear-held-open")
            .save(PathBuf::from(directory).join("pokegear-held-open.png")).unwrap();
    }
    // A newly pressed direction must remain usable while the opening A is held.
    keys.press(KeyCode::ArrowRight);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pokegear_page, PokegearPage::Map);
    assert!(shell.pokegear_exit.is_none());
    keys.release(KeyCode::ArrowRight);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    keys.press(KeyCode::ArrowLeft);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pokegear_page, PokegearPage::Clock);
    assert!(shell.pokegear_exit.is_none());
    keys.release(KeyCode::ArrowLeft);
    keys.release(KeyCode::KeyZ);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    keys.press(KeyCode::KeyZ);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    assert_eq!(shell.pokegear_exit, Some(VisiblePokegearExitPhase::Requested));
    settle_visible_shell_smoke_until_idle(&mut shell).unwrap();
    assert!(!shell.pokegear_menu_open);
    assert!(shell.start_menu_cursor.is_some());
    assert!(shell.last_error.is_none(), "{:?}", shell.last_error);
}

#[test]
fn pokedex_defaults_to_source_new_order_and_stops_at_last_seen() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.record_pokedex_caught("CYNDAQUIL").unwrap();
    shell.shell.record_pokedex_seen("CHIKORITA").unwrap();
    shell.shell.session_mut().state_mut().flags.set_engine_flag("ENGINE_POKEDEX", true).unwrap();
    if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        shell.shell.save(PathBuf::from(directory).join("pokedex-browser.crystalsave")).unwrap();
    }
    open_visible_pokedex_menu(&mut shell).unwrap();
    let snapshot = shell.shell.snapshot().unwrap();
    if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
        let mut world = World::new();
        let mut queue = bevy::ecs::world::CommandQueue::default();
        let mut art = RenderedTilesetArt::default();
        let mut images = Assets::<Image>::default();
        spawn_field_pokedex_screen(&mut Commands::new(&mut queue, &world), &snapshot, &shell,
            &mut art, &shell.asset_root, &mut images).unwrap();
        queue.apply(&mut world);
        std::fs::create_dir_all(&directory).unwrap();
        render_pc_audit_canvas(&mut world, &images, "pokedex-order")
            .save(PathBuf::from(directory).join("pokedex-order.png")).unwrap();
    }
    assert_eq!(snapshot.pokemon[shell.pokedex_cursor].species_id, "CHIKORITA",
        "a fresh Pokédex opens in source NEW mode");
    for _ in 0..300 { move_visible_pokedex_cursor(&mut shell, 1).unwrap(); }
    assert_eq!(snapshot.pokemon[shell.pokedex_cursor].species_id, "CYNDAQUIL",
        "the source list ends at its last seen species");
}

#[test]
fn elm_robbery_call_continues_after_ringing_into_disaster_text() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let map_name = shell.shell.snapshot().unwrap().overworld.map_name;
    let state = &mut shell.shell.session_mut().state_mut();
    state.script_runtime.special_phone_call = Some("SPECIALCALL_ROBBED".to_string());
    state
        .script_runtime
        .variables
        .insert("VAR_CALLERID".to_string(), "PHONE_ELM".to_string());
    state.script_runtime.memory.insert(
        "wCallerContact + PHONE_CONTACT_SCRIPT2_BANK".to_string(),
        "ElmPhoneCallerScript".to_string(),
    );
    shell.active_script_cursor = Some(ActiveScriptCursor {
        origin_map_name: map_name,
        source_script: "Script_ReceivePhoneCall".to_string(),
        next_command_index: 1,
    });
    execute_visible_active_script_step(&mut shell).expect("start ASM double ring");
    assert!(shell.incoming_phone_sequence.is_some());
    let mut keys = ButtonInput::default();
    keys.press(KeyCode::KeyX);
    apply_visible_runtime_controls(&keys, &mut shell, true);
    continue_visible_script_after_prompt(&mut shell).unwrap();
    assert_eq!(
        shell
            .active_script_cursor
            .as_ref()
            .unwrap()
            .next_command_index,
        2,
        "input and auto continuation must not skip the ringing presentation"
    );
    advance_visible_incoming_phone_sequence(&mut shell, 120).expect("complete ASM double ring");
    let snapshot = shell.shell.snapshot().unwrap();
    assert_eq!(
        snapshot.script_events.pending_text_label.as_deref(),
        Some("ElmPhoneDisasterText"),
        "finishing RingTwice_StartCall must immediately execute the caller script"
    );
    assert!(
        !snapshot.ui.window_open,
        "the caller textbox is not a script menu"
    );

    // A/B advances PrintText's pages, then the wrapper's waitbutton starts
    // HangUp. The caller clears the special call and marks the robbery first.
    for _ in 0..40 {
        advance_visible_script_until_player_boundary(&mut shell).unwrap();
        for _ in 0..256 {
            let snapshot = shell.shell.snapshot().unwrap();
            if visible_field_dialogue_is_fully_revealed(&shell, &snapshot) {
                break;
            }
            tick_visible_field_text_reveal(&mut shell, true).unwrap();
        }
        if visible_text_label_can_auto_continue(&shell).unwrap() {
            advance_visible_text_label(&mut shell).unwrap();
        }
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::KeyX);
        apply_visible_runtime_controls(&keys, &mut shell, true);
        assert_eq!(shell.last_error, None);
        if matches!(
            shell.incoming_phone_sequence,
            Some(VisibleIncomingPhoneSequence::HangUp { .. })
        ) {
            break;
        }
    }
    assert!(matches!(
        shell.incoming_phone_sequence,
        Some(VisibleIncomingPhoneSequence::HangUp { .. })
    ));
    let state = &shell.shell.session().state();
    assert!(state.script_runtime.special_phone_call.is_none());
    assert!(
        state
            .flags
            .is_event_flag_set("EVENT_ELM_CALLED_ABOUT_STOLEN_POKEMON")
            .unwrap()
    );
    advance_visible_incoming_phone_sequence(&mut shell, 140).expect("finish ASM hangup");
    let snapshot = shell.shell.snapshot().unwrap();
    assert!(!snapshot.ui.text_window_open);
    assert!(!snapshot.ui.window_open);
    assert!(shell.active_script_cursor.is_none());
    assert!(
        shell
            .shell
            .session()
            .state()
            .script_runtime
            .phone_call_timer
            .initialized
    );
}

#[test]
fn elm_robbery_call_runs_after_overworld_step_without_a() {
    let asset_root = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..").canonicalize().unwrap());
    let runtime = workspace_desktop_runtime(&asset_root);
    let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
    let mut shell = initialize_bevy_runtime_shell(asset_root, runtime,
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier, map_name: "NewBarkTown".to_string(), tile_x: 9, tile_y: 8,
        }, BevyShellConfig { smoke_player_name: Some("TEST".to_string()), ..Default::default() }).unwrap();
    complete_visible_smoke_player_name_if_needed(&mut shell, Some("TEST")).unwrap();
    settle_visible_shell_smoke_until_idle(&mut shell).unwrap();
    shell.shell.session_mut().state_mut().flags.set_engine_flag("ENGINE_POKEGEAR", true).unwrap();
    shell.shell.session_mut().state_mut().script_runtime.special_phone_call = Some("SPECIALCALL_ROBBED".to_string());
    if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        shell.shell.save(PathBuf::from(directory).join("elm-browser.crystalsave")).unwrap();
    }
    let mut app = menu_render_test_app(shell);
    app.update();
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::ArrowDown);
    let mut rang = false;
    let mut reached_disaster = false;
    for _ in 0..400 {
        app.update();
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(shell.last_error.is_none(), "{:?}", shell.last_error);
        rang |= shell.incoming_phone_sequence.is_some();
        reached_disaster = shell.shell.snapshot().unwrap().script_events.pending_text_label.as_deref() == Some("ElmPhoneDisasterText");
        if reached_disaster { break; }
    }
    assert!(rang && reached_disaster, "a real overworld step must ring and start Elm's disaster text without A");
}


#[test]
fn incoming_phone_renders_caller_box_and_asm_name_flash() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.incoming_phone_contact = Some("PHONE_ELM".to_string());
    let snapshot = shell.shell.snapshot().unwrap();
    let mut frames = Vec::new();
    for (remaining, name_visible) in [
        (120, false),
        (100, true),
        (80, false),
        (60, false),
        (40, true),
        (20, false),
    ] {
        shell.incoming_phone_sequence = Some(VisibleIncomingPhoneSequence::RingTwice {
            frames_remaining: remaining,
            second_ring_started: remaining <= 60,
        });
        let mut world = World::new();
        let mut queue = bevy::ecs::world::CommandQueue::default();
        let mut images = Assets::<Image>::default();
        let mut art = RenderedTilesetArt::default();
        spawn_scene_dialog(
            &mut Commands::new(&mut queue, &world),
            &snapshot,
            &shell,
            &mut art,
            &shell.asset_root,
            &mut images,
        )
        .unwrap();
        queue.apply(&mut world);
        assert!(art.font_error.is_none(), "{:?}", art.font_error);
        let glyph_count = world.query::<&DialogGlyphMarker>().iter(&world).count();
        assert_eq!(
            glyph_count > 0,
            name_visible,
            "ring frame {}",
            120 - remaining
        );
        assert!(
            world.query::<&SceneDialogMarker>().iter(&world).count() > 0,
            "Phone_CallerTextbox must remain visible even while the name is blank"
        );
        let canvas = render_pc_audit_canvas(&mut world, &images, "incoming-phone");
        if let Some(directory) = std::env::var_os("CRYSTAL_PHONE_RENDER_DIR") {
            std::fs::create_dir_all(&directory).unwrap();
            canvas
                .save(PathBuf::from(directory).join(format!("ring-{}.png", 120 - remaining)))
                .unwrap();
        }
        frames.push(canvas);
    }
    assert_eq!(frames[0], frames[2]);
    assert_eq!(frames[0], frames[3]);
    assert_eq!(frames[1], frames[4]);
    assert_ne!(frames[0], frames[1]);
}

#[test]
fn pokedex_entry_navigation_preserves_page_at_boundary_and_plays_cry_on_open() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let snapshot = shell.shell.snapshot().unwrap();
    let index = snapshot
        .pokemon
        .iter()
        .position(|p| p.species_id == "CYNDAQUIL")
        .unwrap();
    shell.shell.record_pokedex_seen("CYNDAQUIL").unwrap();
    open_visible_pokedex_menu(&mut shell).unwrap();
    shell.pokedex_cursor = index;
    inspect_visible_pokedex_selection(&mut shell).unwrap();
    assert!(
        shell
            .last_audio_events
            .iter()
            .any(|event| event.contains("queued pokedex_entry cry")),
        "Pokedex_InitDexEntryScreen plays the selected species cry"
    );
    shell.pokedex_detail_page = 1;
    // With only Cyndaquil seen, neither direction can select another entry.
    move_visible_pokedex_cursor(&mut shell, -1).unwrap();
    assert_eq!(shell.pokedex_cursor, index);
    assert_eq!(
        shell.pokedex_detail_page, 1,
        "failed navigation must not reinitialize the entry"
    );
}

#[test]
fn pokedex_page_buttons_preserve_cursor_row_like_asm() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let snapshot = shell.shell.snapshot().unwrap();
    for species in &*snapshot.pokemon {
        shell
            .shell
            .record_pokedex_seen(&species.species_id)
            .unwrap();
    }
    shell.pokedex_controls.mode = VisiblePokedexMode::Old;
    open_visible_pokedex_menu(&mut shell).unwrap();
    shell.pokedex_cursor = 3;
    shell.pokedex_scroll = 0;
    // Left on the first page does nothing, even below the first row.
    page_visible_pokedex_cursor(&mut shell, -1).unwrap();
    assert_eq!(shell.pokedex_cursor, 3);
    page_visible_pokedex_cursor(&mut shell, 1).unwrap();
    assert_eq!(shell.pokedex_cursor, 10);
    page_visible_pokedex_cursor(&mut shell, -1).unwrap();
    assert_eq!(shell.pokedex_cursor, 3);
}

#[test]
fn pokedex_select_and_start_stay_inside_dex_and_search_caught_species() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.record_pokedex_caught("CYNDAQUIL").unwrap();
    shell.shell.record_pokedex_seen("QUILAVA").unwrap();
    open_visible_pokedex_menu(&mut shell).unwrap();
    press_visible_select_button(&mut shell).unwrap();
    assert_eq!(shell.pokedex_controls.option_cursor, Some(0));
    assert!(
        shell.field_notice.is_none(),
        "Select must not run registered-item logic"
    );
    press_visible_b_button(&mut shell).unwrap();
    assert!(shell.pokedex_menu_open);
    press_visible_start_button(&mut shell).unwrap();
    assert_eq!(shell.pokedex_controls.search_cursor, Some(0));
    shell.pokedex_controls.search_types = [2, 0]; // Fire
    shell.pokedex_controls.search_cursor = Some(2);
    press_visible_pokedex_a_button(&mut shell).unwrap();
    assert!(shell.pokedex_controls.search_results.is_none());
    advance_visible_pokedex_search(&mut shell, 207);
    let snapshot = shell.shell.snapshot().unwrap();
    let names = visible_pokedex_listing(&snapshot, &shell)
        .into_iter()
        .map(|i| snapshot.pokemon[i].species_id.clone())
        .collect::<Vec<_>>();
    assert_eq!(names, ["CYNDAQUIL"]);
    press_visible_b_button(&mut shell).unwrap();
    assert_eq!(shell.pokedex_controls.search_cursor, Some(0));
    press_visible_start_button(&mut shell).unwrap();
    assert_eq!(shell.pokedex_controls.search_cursor, None);
    assert!(shell.pokedex_menu_open);
}

#[test]
fn pokedex_orders_end_at_last_seen_and_alphabetical_omits_unseen() {
    let shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let mut snapshot = shell.shell.snapshot().unwrap();
    snapshot.progression.pokedex_seen_species = ["CYNDAQUIL".to_string(), "CHIKORITA".to_string()]
        .into_iter()
        .collect();
    let names = |mode| {
        visible_pokedex_order(&snapshot, mode)
            .into_iter()
            .map(|i| snapshot.pokemon[i].species_id.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        names(VisiblePokedexMode::New),
        ["CHIKORITA", "BAYLEEF", "MEGANIUM", "CYNDAQUIL"]
    );
    assert_eq!(
        names(VisiblePokedexMode::Alphabetical),
        ["CHIKORITA", "CYNDAQUIL"]
    );
    let old = names(VisiblePokedexMode::Old);
    assert_eq!(old.first().unwrap(), "BULBASAUR");
    assert_eq!(old.last().unwrap(), "CYNDAQUIL");
}

#[test]
fn pokedex_entry_actions_have_separate_input_and_return_to_entry() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.record_pokedex_caught("CYNDAQUIL").unwrap();
    let snapshot = shell.shell.snapshot().unwrap();
    let index = snapshot
        .pokemon
        .iter()
        .position(|p| p.species_id == "CYNDAQUIL")
        .unwrap();
    open_visible_pokedex_menu(&mut shell).unwrap();
    shell.pokedex_cursor = index;
    inspect_visible_pokedex_selection(&mut shell).unwrap();
    page_visible_pokedex_cursor(&mut shell, 1).unwrap();
    assert_eq!(shell.pokedex_controls.entry_action, 1);
    press_visible_pokedex_a_button(&mut shell).unwrap();
    assert_eq!(shell.pokedex_controls.area_region, Some(false));
    press_visible_pokedex_a_button(&mut shell).unwrap();
    assert_eq!(shell.pokedex_controls.area_region, None);
    assert!(shell.pokedex_detail_open);
    page_visible_pokedex_cursor(&mut shell, 1).unwrap();
    let page = shell.pokedex_detail_page;
    press_visible_pokedex_a_button(&mut shell).unwrap();
    assert_eq!(
        shell.pokedex_detail_page, page,
        "CRY must not turn the page"
    );
    page_visible_pokedex_cursor(&mut shell, 1).unwrap();
    press_visible_pokedex_a_button(&mut shell).unwrap();
    assert!(shell.pokedex_controls.printer_open);
    press_visible_pokedex_a_button(&mut shell).unwrap();
    assert!(
        shell.pokedex_controls.printer_open,
        "only B cancels Printer Error 2"
    );
    press_visible_b_button(&mut shell).unwrap();
    assert!(!shell.pokedex_controls.printer_open);
    assert!(shell.pokedex_detail_open);
}

#[test]
fn pokedex_unown_mode_requires_upgrade_and_keeps_catch_order() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    open_visible_pokedex_menu(&mut shell).unwrap();
    press_visible_select_button(&mut shell).unwrap();
    assert_eq!(pokedex_option_entries(&shell).unwrap().len(), 3);
    shell
        .shell
        .set_script_flag_for_smoke("ENGINE_UNOWN_DEX")
        .unwrap();
    shell.shell.record_pokedex_caught("UNOWN").unwrap();
    shell.shell.session_mut().state_mut().pokedex.unown_letters = vec![26, 1, 13];
    assert_eq!(pokedex_option_entries(&shell).unwrap().len(), 4);
    shell.pokedex_controls.option_cursor = Some(3);
    press_visible_pokedex_a_button(&mut shell).unwrap();
    assert_eq!(shell.pokedex_controls.unown_cursor, Some(0));
    page_visible_pokedex_cursor(&mut shell, -1).unwrap();
    assert_eq!(shell.pokedex_controls.unown_cursor, Some(0));
    page_visible_pokedex_cursor(&mut shell, 1).unwrap();
    assert_eq!(shell.pokedex_controls.unown_cursor, Some(1));
    assert_eq!(
        shell.shell.session().state().pokedex.unown_letters,
        [26, 1, 13]
    );
    press_visible_b_button(&mut shell).unwrap();
    assert!(shell.pokedex_controls.option_cursor.is_some());
    assert!(shell.pokedex_controls.unown_cursor.is_none());
}

#[test]
fn pokedex_seen_caught_counts_and_unown_forms_survive_save_roundtrip() {
    use crate::core::models::{Dv, PokedexState, Pokemon};
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let before = shell.shell.snapshot().unwrap().progression;
    shell.shell.record_pokedex_seen("PIDGEY").unwrap();
    shell.shell.record_pokedex_seen("PIDGEY").unwrap();
    let seen = shell.shell.snapshot().unwrap().progression;
    assert!(seen.pokedex_seen_species.contains("PIDGEY"));
    assert!(!seen.pokedex_caught_species.contains("PIDGEY"));
    assert_eq!(seen.pokedex_seen, before.pokedex_seen + 1);
    assert_eq!(seen.pokedex_owned, before.pokedex_owned);
    shell.shell.record_pokedex_caught("PIDGEY").unwrap();
    shell.shell.record_pokedex_caught("PIDGEY").unwrap();
    let caught = shell.shell.snapshot().unwrap().progression;
    assert_eq!(caught.pokedex_seen, seen.pokedex_seen);
    assert_eq!(caught.pokedex_owned, before.pokedex_owned + 1);
    assert!(caught.pokedex_caught_species.contains("PIDGEY"));
    let species = shell
        .shell
        .runtime()
        .data()
        .pokemon
        .get("UNOWN")
        .unwrap()
        .clone();
    let z = Pokemon::new_for_tests(species.clone(), 5, Dv::from_non_hp(15, 15, 15, 15));
    let a = Pokemon::new_for_tests(species, 5, Dv::from_non_hp(0, 0, 0, 0));
    assert_eq!((z.dvs.unown_letter(), a.dvs.unown_letter()), (26, 1));
    let dex = &mut shell.shell.session_mut().state_mut().pokedex;
    assert!(dex.record_caught_pokemon(&z));
    assert!(!dex.record_caught_pokemon(&a));
    assert!(!dex.record_caught_pokemon(&z));
    assert_eq!(dex.unown_letters, [26, 1]);
    assert!(dex.has_seen("UNOWN"));
    assert!(dex.has_caught("UNOWN"));
    let restored: PokedexState = serde_json::from_slice(&serde_json::to_vec(dex).unwrap()).unwrap();
    assert_eq!(&restored, dex);
}

#[test]
fn pokedex_empty_search_blocks_input_until_source_delays_finish() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    open_visible_pokedex_menu(&mut shell).unwrap();
    press_visible_start_button(&mut shell).unwrap();
    shell.pokedex_controls.search_types = [16, 0]; // No caught Dark Pokemon.
    shell.pokedex_controls.search_cursor = Some(2);
    press_visible_pokedex_a_button(&mut shell).unwrap();
    advance_visible_pokedex_search(&mut shell, 206);
    press_visible_b_button(&mut shell).unwrap();
    assert!(shell.pokedex_controls.search_animation.is_some());
    assert!(shell.pokedex_controls.search_cursor.is_some());
    advance_visible_pokedex_search(&mut shell, 1);
    assert!(shell.pokedex_controls.search_not_found);
    assert_eq!(shell.pokedex_controls.not_found_frames, 128);
    press_visible_b_button(&mut shell).unwrap();
    assert!(shell.pokedex_controls.search_cursor.is_some());
    advance_visible_pokedex_search(&mut shell, 128);
    assert!(!shell.pokedex_controls.search_not_found);
    press_visible_b_button(&mut shell).unwrap();
    assert!(shell.pokedex_controls.search_cursor.is_none());
    assert!(shell.pokedex_menu_open);
}

#[test]
fn pokedex_first_open_does_not_treat_catalog_zero_as_a_previous_entry() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.record_pokedex_seen("BULBASAUR").unwrap();
    open_visible_pokedex_menu(&mut shell).unwrap();
    let snapshot = shell.shell.snapshot().unwrap();
    assert_eq!(snapshot.pokemon[shell.pokedex_cursor].species_id, "CHIKORITA");
    move_visible_pokedex_cursor(&mut shell, 999).unwrap();
    inspect_visible_pokedex_selection(&mut shell).unwrap();
    close_visible_pokedex_menu(&mut shell);
    open_visible_pokedex_menu(&mut shell).unwrap();
    assert_eq!(snapshot.pokemon[shell.pokedex_cursor].species_id, "BULBASAUR",
        "reopening remembers an entry that was actually displayed");
}

#[test]
fn pokedex_start_key_reaches_search_through_input_routing() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.shell.record_pokedex_seen("CYNDAQUIL").unwrap();
    open_visible_pokedex_menu(&mut shell).unwrap();
    let mut app = menu_render_test_app(shell);
    app.update();
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::Enter);
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert!(shell.last_error.is_none(), "{:?}", shell.last_error);
    assert_eq!(shell.pokedex_controls.search_cursor, Some(0));
}
