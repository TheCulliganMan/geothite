#[test]
fn battle_input_keeps_post_battle_map_script_suspended() {
    let mut shell = route36_battle_shell_for_render_regression();
    shell.visible_battle_transition = None;
    shell.battle_entry_messages_remaining = 0;
    shell.battle_enemy_send_out_pending = false;
    shell.battle_player_send_out_pending = false;
    shell.battle_messages.clear();
    shell.battle_message_scenes.clear();
    shell.battle_text_reveal = None;
    sync_visible_battle_action_cursor(&mut shell);
    arm_visible_active_script_cursor_with_origin(
        &mut shell, "Route36", "WateredWeirdTreeScript", 13,
    );
    assert!(shell.active_script_cursor.is_some());
    assert!(!shell.shell.session().state().flags.is_event_flag_set("EVENT_FOUGHT_SUDOWOODO").unwrap());
    execute_visible_active_script_step(&mut shell).unwrap();
    assert!(!shell.shell.session().state().flags.is_event_flag_set("EVENT_FOUGHT_SUDOWOODO").unwrap());
    press_visible_a_button(&mut shell).unwrap();
    assert!(!shell.shell.session().state().flags.is_event_flag_set("EVENT_FOUGHT_SUDOWOODO").unwrap());
    assert_eq!(shell.active_script_cursor.as_ref().unwrap().next_command_index, 13);
    assert!(shell.last_action_status.as_deref().is_some_and(|s| s.starts_with("FIGHT MOVES")), "{:?}", shell.last_action_status);
}

#[test]
fn battle_hp_corner_is_black_in_every_hp_palette() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let asset_root = AssetRoot::new(repo_root);
    let mut art = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    let tiles = battle_hp_bar_tiles(&mut art, &asset_root, &mut images)
        .expect("battle HP graphics");
    for zone in 0..=2 {
        let image = images.get(&tiles[&(0x6c, zone)].handle).unwrap();
        let opaque: Vec<_> = image.data.chunks_exact(4).filter(|pixel| pixel[3] != 0).collect();
        assert!(!opaque.is_empty(), "corner must be visible");
        assert!(opaque.iter().all(|pixel| *pixel == [0, 0, 0, 255]),
            "player HP corner must use black in palette {zone}");
    }
}

#[test]
fn battle_dialogue_uses_player_input_drains_once_and_returns_menu_control() {
    let mut runtime_shell = route36_battle_shell_for_render_regression();
    runtime_shell.visible_battle_transition = None;
    runtime_shell.battle_entry_messages_remaining = 0;
    runtime_shell.battle_enemy_send_out_pending = false;
    runtime_shell.battle_player_send_out_pending = false;
    runtime_shell.battle_messages.clear();
    runtime_shell.battle_message_scenes.clear();
    runtime_shell.battle_text_reveal = None;
    sync_visible_battle_action_cursor(&mut runtime_shell);

    let original_cursor = runtime_shell
        .battle_action_cursor
        .clone()
        .expect("active battle action cursor");
    let messages = [
        "CYNDAQUIL used\nTACKLE!".to_string(),
        "A deliberately long battle message that occupies more than two native textbox lines and must advance through every page exactly once.".to_string(),
        "It's not very\neffective...".to_string(),
    ];
    runtime_shell.battle_messages = messages.clone().into_iter().collect();
    runtime_shell.battle_message_scene = Some(Box::new(
        runtime_shell.shell.snapshot().expect("battle dialogue scene"),
    ));

    dispatch_visible_ui_direction(&mut runtime_shell, GameButton::Down);
    assert_eq!(
        runtime_shell.battle_action_cursor.as_ref(),
        Some(&original_cursor),
        "directional input during battle text must not navigate the hidden command menu"
    );

    let mut dismissed = Vec::new();
    for step in 0..64 {
        let Some(message) = runtime_shell.battle_messages.front().cloned() else {
            break;
        };
        let snapshot = runtime_shell
            .shell
            .presentation_snapshot()
            .expect("battle dialogue presentation snapshot");
        while !visible_battle_message_is_complete(&runtime_shell, &message) {
            assert!(
                advance_visible_battle_text_reveal(&mut runtime_shell, &snapshot, true),
                "battle dialogue reveal stalled at step {step}: {message:?}"
            );
        }
        let page_before = runtime_shell
            .battle_text_reveal
            .as_ref()
            .expect("completed battle text reveal")
            .page_index;
        press_visible_a_button(&mut runtime_shell).expect("player A advances battle dialogue");
        if runtime_shell.battle_messages.front() != Some(&message) {
            dismissed.push(message);
        } else {
            let page_after = runtime_shell
                .battle_text_reveal
                .as_ref()
                .expect("next battle text page reveal")
                .page_index;
            assert_eq!(
                page_after,
                page_before + 1,
                "A on complete battle text must advance a page or consume the message"
            );
        }
    }

    assert_eq!(dismissed, messages, "battle messages repeated or changed order");
    assert!(runtime_shell.battle_messages.is_empty());
    assert!(runtime_shell.battle_text_reveal.is_none());
    assert!(runtime_shell.battle_message_scene.is_none());
    sync_visible_battle_action_cursor(&mut runtime_shell);
    assert!(
        runtime_shell.battle_action_cursor.is_some(),
        "closing the last battle message must return command-menu control"
    );
    assert_eq!(runtime_shell.last_error, None);
}

#[test]
fn replacement_text_uses_send_out_mon_texts_quarter_max_hp_divisor() {
    let runtime_shell = route36_battle_shell_for_render_regression();
    let mut snapshot = runtime_shell.shell.snapshot().expect("battle snapshot");
    let battle = snapshot.battle.as_mut().expect("active battle");
    battle.enemy_pokemon.hp = 2;
    battle.enemy_pokemon.max_hp = 21;
    let nickname = snapshot.party.slots[0].pokemon.nickname.clone();

    assert_eq!(
        super::visible_player_send_out_message(&snapshot, 0, false).expect("send-out message"),
        format!("Go for it, {nickname}!"),
    );
}

#[test]
fn replacement_text_rejects_send_out_mon_texts_nonterminating_divisor() {
    let runtime_shell = route36_battle_shell_for_render_regression();
    let mut snapshot = runtime_shell.shell.snapshot().expect("battle snapshot");
    let battle = snapshot.battle.as_mut().expect("active battle");
    battle.enemy_pokemon.hp = 1;
    battle.enemy_pokemon.max_hp = 3;

    let error = super::visible_player_send_out_message(&snapshot, 0, false)
        .expect_err("the cartridge divide would never terminate");
    assert!(
        error
            .to_string()
            .contains("would not terminate with enemy max HP 3"),
        "{error:#}"
    );
}

#[test]
fn link_replacement_text_uses_go_mon_text_without_reading_enemy_hp() {
    let runtime_shell = route36_battle_shell_for_render_regression();
    let mut snapshot = runtime_shell.shell.snapshot().expect("battle snapshot");
    snapshot.link_session.link_mode = 1;
    let battle = snapshot.battle.as_mut().expect("active battle");
    battle.enemy_pokemon.hp = 1;
    battle.enemy_pokemon.max_hp = 3;
    let nickname = snapshot.party.slots[0].pokemon.nickname.clone();

    assert_eq!(
        super::visible_player_send_out_message(&snapshot, 0, false).expect("send-out message"),
        format!("Go! {nickname}!"),
    );
}

#[test]
fn initial_link_send_out_text_still_uses_enemy_hp() {
    let runtime_shell = route36_battle_shell_for_render_regression();
    let mut snapshot = runtime_shell.shell.snapshot().expect("battle snapshot");
    snapshot.link_session.link_mode = 1;
    let battle = snapshot.battle.as_mut().expect("active battle");
    battle.enemy_pokemon.hp = 2;
    battle.enemy_pokemon.max_hp = 21;
    let nickname = snapshot.party.slots[0].pokemon.nickname.clone();

    assert_eq!(
        super::visible_player_send_out_message(&snapshot, 0, true).expect("send-out message"),
        format!("Go for it, {nickname}!"),
    );
}

#[test]
fn withdrawal_text_uses_withdraw_mon_texts_quarter_max_hp_divisor() {
    let mut runtime_shell = route36_battle_shell_for_render_regression();
    runtime_shell.battle_enemy_hp_at_player_send_out = Some(20);
    let mut snapshot = runtime_shell.shell.snapshot().expect("battle snapshot");
    let battle = snapshot.battle.as_mut().expect("active battle");
    battle.enemy_pokemon.hp = 14;
    battle.enemy_pokemon.max_hp = 21;

    assert_eq!(
        super::visible_player_withdraw_message(&runtime_shell, &snapshot, "CYNDAQUIL")
            .expect("withdrawal message"),
        "CYNDAQUIL, OK! Come back!",
    );
}

#[test]
fn withdrawal_text_uses_wrapping_damage_and_the_low_quotient_byte() {
    let mut runtime_shell = route36_battle_shell_for_render_regression();
    runtime_shell.battle_enemy_hp_at_player_send_out = Some(10);
    let mut snapshot = runtime_shell.shell.snapshot().expect("battle snapshot");
    let battle = snapshot.battle.as_mut().expect("active battle");
    battle.enemy_pokemon.hp = 11;
    battle.enemy_pokemon.max_hp = 21;

    // (10 - 11) wraps to 65535; (65535 * 25) / (21 >> 2) has low byte 251.
    assert_eq!(
        super::visible_player_withdraw_message(&runtime_shell, &snapshot, "CYNDAQUIL")
            .expect("withdrawal message"),
        "CYNDAQUIL, good! Come back!",
    );
}

#[test]
fn withdrawal_text_rejects_withdraw_mon_texts_nonterminating_divisor() {
    let mut runtime_shell = route36_battle_shell_for_render_regression();
    runtime_shell.battle_enemy_hp_at_player_send_out = Some(2);
    let mut snapshot = runtime_shell.shell.snapshot().expect("battle snapshot");
    let battle = snapshot.battle.as_mut().expect("active battle");
    battle.enemy_pokemon.hp = 1;
    battle.enemy_pokemon.max_hp = 3;

    let error = super::visible_player_withdraw_message(&runtime_shell, &snapshot, "CYNDAQUIL")
        .expect_err("the cartridge divide would never terminate");
    assert!(
        error
            .to_string()
            .contains("would not terminate with enemy max HP 3"),
        "{error:#}"
    );
}

#[test]
fn battle_animation_loop_count_matches_asm_body_passes() {
    let key = ("BattleAnim_Test".to_string(), 4);
    let mut loops = std::collections::BTreeMap::new();

    assert!(advance_visible_battle_animation_loop(
        &mut loops,
        key.clone(),
        3
    ));
    assert!(advance_visible_battle_animation_loop(
        &mut loops,
        key.clone(),
        3
    ));
    assert!(!advance_visible_battle_animation_loop(
        &mut loops,
        key.clone(),
        3
    ));
    assert!(loops.is_empty());

    assert!(!advance_visible_battle_animation_loop(
        &mut loops,
        key.clone(),
        1
    ));
    assert!(advance_visible_battle_animation_loop(&mut loops, key, 0));
}

#[test]
fn reset_obp0_uses_the_asm_hardware_values() {
    assert_eq!(battle_anim_reset_obp0_value(false), 0xe0);
    assert_eq!(battle_anim_reset_obp0_value(true), 0xf0);
}

#[test]
fn faint_mon_drops_one_tile_every_two_frames_then_disappears() {
    let mut animation = VisibleMoveAnimation {
        trigger_message: "Enemy TEST\nfainted!".to_string(),
        move_id: "FAINT_MON".to_string(),
        animation_label: "BattleAnim_FaintMon".to_string(),
        player_move: false,
        started: true,
        waiting_for_hp: false,
        frame: 0,
        total_frames: 14,
        sound_events: Vec::new(),
        next_sound_event: 0,
        cry_events: Vec::new(),
        next_cry_event: 0,
        object_events: Vec::new(),
        bg_events: vec![VisibleMoveBgEvent {
            frame: 0,
            effect_id: "BATTLE_BG_EFFECT_FAINT_MON".to_string(),
            duration: 14,
            target: "BG_EFFECT_USER".to_string(),
            param: 0,
            incremented: false,
        }],
        actor_species_override: None,
        actor_shiny_override: None,
    };
    let source_pixel = TILE_SIZE / SOURCE_TILE_SIZE as f32;

    let enemy_y = [0_u16, 1, 2, 3, 4, 5, 12, 13].map(|frame| {
        animation.frame = frame;
        let (player, enemy) = visible_move_battler_offsets(Some(&animation));
        assert_eq!(player, Vec3::ZERO);
        enemy.y
    });

    assert_eq!(
        enemy_y,
        [-8.0, -8.0, -16.0, -16.0, -24.0, -24.0, -56.0, -56.0]
            .map(|pixels| pixels * source_pixel)
    );
    animation.frame = 0;
    assert_eq!(
        visible_move_battler_row_extractions(Some(&animation)).1,
        Some(VisibleBattlerRowExtraction {
            rows: 1,
            top: false,
            bg_rows_cleared: true,
            render_extracted: false,
        })
    );
    animation.frame = 12;
    assert_eq!(
        visible_move_battler_row_extractions(Some(&animation)).1,
        Some(VisibleBattlerRowExtraction {
            rows: 7,
            top: false,
            bg_rows_cleared: true,
            render_extracted: false,
        })
    );
    animation.frame = 13;
    assert_eq!(visible_move_battler_visibility(Some(&animation)), (true, true));
    animation.frame = 14;
    assert_eq!(visible_move_battler_visibility(Some(&animation)), (true, false));
}

#[test]
fn vibrate_mon_toggles_one_pixel_every_two_updates_for_32_frames() {
    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(),
        move_id: "TEST_VIBRATE".to_string(),
        animation_label: "BattleAnim_TestVibrate".to_string(),
        player_move: true,
        started: true,
        waiting_for_hp: false,
        frame: 0,
        total_frames: 40,
        sound_events: Vec::new(),
        next_sound_event: 0,
        cry_events: Vec::new(),
        next_cry_event: 0,
        object_events: Vec::new(),
        bg_events: vec![VisibleMoveBgEvent {
            frame: 0,
            effect_id: "BATTLE_BG_EFFECT_VIBRATE_MON".to_string(),
            duration: 0,
            target: "BG_EFFECT_USER".to_string(),
            param: 0,
            incremented: false,
        }],
        actor_species_override: None,
        actor_shiny_override: None,
    };
    let source_pixel = TILE_SIZE / SOURCE_TILE_SIZE as f32;

    let offsets = [0_u16, 1, 2, 3, 4, 31, 32, 33]
        .map(|frame| {
            animation.frame = frame;
            visible_move_battler_offsets(Some(&animation))
        });

    assert_eq!(offsets[0], (Vec3::ZERO, Vec3::ZERO));
    for index in 1..=6 {
        assert_eq!(offsets[index].1, Vec3::ZERO);
        assert_eq!(offsets[index].0.x.abs(), source_pixel);
        assert_eq!(offsets[index].0.y, 0.0);
    }
    assert_eq!(offsets[1].0.x, offsets[2].0.x);
    assert_eq!(offsets[3].0.x, offsets[4].0.x);
    assert_eq!(offsets[1].0.x, -offsets[3].0.x);
    assert_eq!(offsets[7], (Vec3::ZERO, Vec3::ZERO));
}

#[test]
fn wobble_mon_uses_the_radius_eight_asm_sine_until_incremented() {
    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(),
        move_id: "TEST_WOBBLE".to_string(),
        animation_label: "BattleAnim_TestWobble".to_string(),
        player_move: true,
        started: true,
        waiting_for_hp: false,
        frame: 0,
        total_frames: 16,
        sound_events: Vec::new(),
        next_sound_event: 0,
        cry_events: Vec::new(),
        next_cry_event: 0,
        object_events: Vec::new(),
        bg_events: vec![
            VisibleMoveBgEvent {
                frame: 0,
                effect_id: "BATTLE_BG_EFFECT_WOBBLE_MON".to_string(),
                duration: 0,
                target: "BG_EFFECT_USER".to_string(),
                param: 0,
                incremented: false,
            },
            VisibleMoveBgEvent {
                frame: 10,
                effect_id: "BATTLE_BG_EFFECT_WOBBLE_MON".to_string(),
                duration: 0,
                target: String::new(),
                param: 0,
                incremented: true,
            },
        ],
        actor_species_override: None,
        actor_shiny_override: None,
    };
    let source_pixel = TILE_SIZE / SOURCE_TILE_SIZE as f32;

    let player_y = [0_u16, 1, 2, 3, 4, 5, 6, 9, 10].map(|frame| {
        animation.frame = frame;
        let (player, enemy) = visible_move_battler_offsets(Some(&animation));
        assert_eq!(enemy, Vec3::ZERO);
        player.y
    });

    // The renderer's positive source-SCY displacement maps downward in Bevy.
    assert_eq!(
        player_y,
        [0.0, 0.0, -3.0, -5.0, -7.0, -8.0, -7.0, 0.0, 0.0]
            .map(|pixels| pixels * source_pixel)
    );
}

#[test]
fn wobble_player_uses_the_radius_six_asm_sine_for_32_updates() {
    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(),
        move_id: "TEST_PLAYER_WOBBLE".to_string(),
        animation_label: "BattleAnim_TestPlayerWobble".to_string(),
        player_move: false,
        started: true,
        waiting_for_hp: false,
        frame: 0,
        total_frames: 40,
        sound_events: Vec::new(),
        next_sound_event: 0,
        cry_events: Vec::new(),
        next_cry_event: 0,
        object_events: Vec::new(),
        bg_events: vec![VisibleMoveBgEvent {
            frame: 0,
            effect_id: "BATTLE_BG_EFFECT_WOBBLE_PLAYER".to_string(),
            duration: 0,
            target: "0".to_string(),
            param: 0,
            incremented: false,
        }],
        actor_species_override: None,
        actor_shiny_override: None,
    };
    let source_pixel = TILE_SIZE / SOURCE_TILE_SIZE as f32;

    let player_x = [0_u16, 1, 2, 5, 9, 17, 25, 32, 33].map(|frame| {
        animation.frame = frame;
        let (player, enemy) = visible_move_battler_offsets(Some(&animation));
        assert_eq!(enemy, Vec3::ZERO);
        player.x
    });

    assert_eq!(
        player_x,
        [0.0, 0.0, 1.0, 4.0, 6.0, 0.0, -6.0, -1.0, 0.0]
            .map(|pixels| pixels * source_pixel)
    );
}

#[test]
fn wobble_screen_uses_the_radius_six_asm_sine_for_32_updates() {
    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(),
        move_id: "TEST_SCREEN_WOBBLE".to_string(),
        animation_label: "BattleAnim_TestScreenWobble".to_string(),
        player_move: true,
        started: true,
        waiting_for_hp: false,
        frame: 0,
        total_frames: 40,
        sound_events: Vec::new(),
        next_sound_event: 0,
        cry_events: Vec::new(),
        next_cry_event: 0,
        object_events: Vec::new(),
        bg_events: vec![VisibleMoveBgEvent {
            frame: 0,
            effect_id: "BATTLE_BG_EFFECT_WOBBLE_SCREEN".to_string(),
            duration: 0,
            target: "0".to_string(),
            param: 0,
            incremented: false,
        }],
        actor_species_override: None,
        actor_shiny_override: None,
    };
    let source_pixel = TILE_SIZE / SOURCE_TILE_SIZE as f32;

    let screen_x = [0_u16, 1, 4, 8, 16, 24, 31, 32].map(|frame| {
        animation.frame = frame;
        visible_move_screen_offset(Some(&animation)).x
    });

    assert_eq!(
        screen_x,
        [0.0, 1.0, 4.0, 6.0, 0.0, -6.0, -1.0, 0.0]
            .map(|pixels| pixels * source_pixel)
    );
}

#[test]
fn surf_uses_prior_object_boundary_and_64_byte_wave_rotation() {
    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(), move_id: "SURF".to_string(),
        animation_label: "BattleAnim_Surf".to_string(), player_move: true,
        started: true, waiting_for_hp: false, frame: 0, total_frames: 184,
        sound_events: Vec::new(), next_sound_event: 0, cry_events: Vec::new(),
        next_cry_event: 0,
        object_events: vec![VisibleMoveObjectEvent {
            frame: 0,
            command: VisibleMoveObjectCommand::Spawn {
                object_id: "BATTLE_ANIM_OBJ_SURF".to_string(), x: 88, y: 104, param: 8,
            },
        }],
        bg_events: vec![VisibleMoveBgEvent {
            frame: 0, effect_id: "BATTLE_BG_EFFECT_SURF".to_string(), duration: 0,
            target: "$0".to_string(), param: 0, incremented: false,
        }], actor_species_override: None, actor_shiny_override: None,
    };
    assert!(visible_surf_line_offsets(Some(&animation)).unwrap().iter().all(|offset| *offset == 0));
    animation.frame = 1;
    let first = visible_surf_line_offsets(Some(&animation)).expect("first Surf copy");
    assert!(first[..=88].iter().all(|offset| *offset == 0));
    assert_eq!(first[89], visible_battle_anim_sine(52, 2) as i8);
    assert_eq!(first[94], visible_battle_anim_sine(62, 2) as i8);
    animation.frame = 2;
    let second = visible_surf_line_offsets(Some(&animation)).expect("second Surf copy");
    assert!(second[..=87].iter().all(|offset| *offset == 0));
    assert_eq!(second[88], visible_battle_anim_sine(52, 2) as i8);

    animation.object_events.push(VisibleMoveObjectEvent {
        frame: 10,
        command: VisibleMoveObjectCommand::Clear,
    });
    animation.frame = 10;
    assert!(visible_surf_line_offsets(Some(&animation)).is_some());
    animation.frame = 11;
    assert!(visible_surf_line_offsets(Some(&animation)).is_none());
}

#[test]
fn wave_deform_mon_grows_then_decays_per_scanline_around_increment() {
    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(),
        move_id: "TEST_WAVE_DEFORM".to_string(),
        animation_label: "BattleAnim_TestWaveDeform".to_string(),
        player_move: true,
        started: true,
        waiting_for_hp: false,
        frame: 0,
        total_frames: 96,
        sound_events: Vec::new(),
        next_sound_event: 0,
        cry_events: Vec::new(),
        next_cry_event: 0,
        object_events: Vec::new(),
        bg_events: vec![
            VisibleMoveBgEvent {
                frame: 0,
                effect_id: "BATTLE_BG_EFFECT_WAVE_DEFORM_MON".to_string(),
                duration: 0,
                target: "BG_EFFECT_USER".to_string(),
                param: 0,
                incremented: false,
            },
            VisibleMoveBgEvent {
                frame: 48,
                effect_id: "BATTLE_BG_EFFECT_WAVE_DEFORM_MON".to_string(),
                duration: 0,
                target: String::new(),
                param: 0,
                incremented: true,
            },
        ],
        actor_species_override: None,
        actor_shiny_override: None,
    };

    animation.frame = 9;
    let growing = visible_wave_deform_line_offsets(Some(&animation)).expect("growing wave");
    assert_eq!(growing[47], 0, "player range begins at source line $30");
    assert_eq!(growing[49], visible_battle_anim_sine(49 * 4, 8) as i8);

    animation.frame = 48;
    let first_decay = visible_wave_deform_line_offsets(Some(&animation)).expect("first decay");
    assert_eq!(first_decay[49], visible_battle_anim_sine(49 * 4, 31) as i8);

    animation.frame = 78;
    let last_decay = visible_wave_deform_line_offsets(Some(&animation)).expect("last decay");
    assert_eq!(last_decay[52], visible_battle_anim_sine(52 * 4, 1) as i8);

    animation.frame = 79;
    assert!(
        visible_wave_deform_line_offsets(Some(&animation))
            .expect("cleared wave buffer")
            .iter()
            .all(|offset| *offset == 0)
    );
}

#[test]
fn shared_screen_shake_counter_matches_the_asm_byte_state_machine() {
    let rollout = VisibleMoveBgEvent {
        frame: 0,
        effect_id: "BATTLE_BG_EFFECT_ROLLOUT".to_string(),
        duration: 0x60,
        target: "$1".to_string(),
        param: 0x01,
        incremented: false,
    };
    assert_eq!(
        [0_u16, 1, 2, 3, 94, 95, 96]
            .map(|age| visible_bg_shake_amount(&rollout, age)),
        [Some(1), Some(-1), Some(1), Some(-1), Some(1), Some(-1), None]
    );

    let grouped = VisibleMoveBgEvent {
        frame: 0,
        effect_id: "BATTLE_BG_EFFECT_SHAKE_SCREEN_Y".to_string(),
        duration: 0x20,
        target: "$2".to_string(),
        param: 0x20,
        incremented: false,
    };
    assert_eq!(
        [0_u16, 1, 2, 3, 4, 5, 6]
            .map(|age| visible_bg_shake_amount(&grouped, age)),
        [Some(-2), Some(-2), Some(-2), Some(2), Some(2), Some(2), Some(-2)]
    );
}

#[test]
fn psychic_teleport_and_night_shade_rotate_the_asm_sine_buffer() {
    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(), move_id: "PSYCHIC".to_string(),
        animation_label: "BattleAnim_TestWavyScreen".to_string(), player_move: true,
        started: true, waiting_for_hp: false, frame: 0, total_frames: 16,
        sound_events: Vec::new(), next_sound_event: 0, cry_events: Vec::new(),
        next_cry_event: 0, object_events: Vec::new(),
        bg_events: vec![VisibleMoveBgEvent {
            frame: 0, effect_id: "BATTLE_BG_EFFECT_PSYCHIC".to_string(), duration: 0,
            target: "$0".to_string(), param: 0, incremented: false,
        }],
        actor_species_override: None, actor_shiny_override: None,
    };
    let setup = visible_psychic_teleport_line_x_offsets(Some(&animation)).expect("setup wave");
    assert_eq!(setup[0], 0, "DeformScreen excludes hLYOverrideStart");
    assert_eq!(setup[1], visible_battle_anim_sine(6, 5) as i8);
    animation.frame = 1;
    let first = visible_psychic_teleport_line_x_offsets(Some(&animation)).expect("rotation");
    assert_eq!(first[0], setup[1]);
    animation.frame = 4;
    assert_eq!(visible_psychic_teleport_line_x_offsets(Some(&animation)), Some(first));
    animation.frame = 5;
    assert_eq!(visible_psychic_teleport_line_x_offsets(Some(&animation)).unwrap()[0], setup[2]);

    animation.bg_events[0].effect_id = "BATTLE_BG_EFFECT_TELEPORT".to_string();
    animation.frame = 2;
    assert_eq!(visible_psychic_teleport_line_x_offsets(Some(&animation)).unwrap()[0], setup[2]);

    animation.bg_events[0].effect_id = "BATTLE_BG_EFFECT_NIGHT_SHADE".to_string();
    animation.bg_events[0].param = 8;
    animation.frame = 0;
    let night = visible_night_shade_line_y_offsets(Some(&animation)).expect("Night Shade wave");
    assert_eq!(night[0], 0);
    assert_eq!(night[1], visible_battle_anim_sine(8, 2) as i8);
    assert!(visible_psychic_teleport_line_x_offsets(Some(&animation)).is_none());
    animation.bg_events.push(VisibleMoveBgEvent {
        frame: 1, effect_id: "BATTLE_BG_EFFECT_NIGHT_SHADE".to_string(), duration: 0,
        target: String::new(), param: 0, incremented: true,
    });
    animation.frame = 1;
    assert!(visible_night_shade_line_y_offsets(Some(&animation)).is_none());
}

#[test]
fn whirlpool_and_water_use_their_asm_scy_buffers_without_colour_overlays() {
    let event = |frame, effect_id: &str, duration, target: &str, param, incremented| {
        VisibleMoveBgEvent {
            frame,
            effect_id: effect_id.to_string(),
            duration,
            target: target.to_string(),
            param,
            incremented,
        }
    };
    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(), move_id: "WATER_TEST".to_string(),
        animation_label: "BattleAnim_WaterTest".to_string(), player_move: true,
        started: true, waiting_for_hp: false, frame: 0, total_frames: 32,
        sound_events: Vec::new(), next_sound_event: 0, cry_events: Vec::new(),
        next_cry_event: 0, object_events: Vec::new(),
        bg_events: vec![event(0, "BATTLE_BG_EFFECT_WHIRLPOOL", 0, "$0", 0, false)],
        actor_species_override: None, actor_shiny_override: None,
    };
    let whirlpool = visible_whirlpool_line_y_offsets(Some(&animation)).expect("Whirlpool setup");
    assert_eq!(whirlpool[0], 0);
    assert_eq!(whirlpool[1], visible_battle_anim_sine(2, 2) as i8);
    animation.frame = 1;
    assert_eq!(visible_whirlpool_line_y_offsets(Some(&animation)).unwrap()[0], whirlpool[1]);
    animation.bg_events.push(event(2, "BATTLE_BG_EFFECT_WHIRLPOOL", 0, "", 0, true));
    animation.frame = 2;
    assert!(visible_whirlpool_line_y_offsets(Some(&animation)).is_none());

    animation.bg_events = vec![
        event(0, "BATTLE_BG_EFFECT_START_WATER", 0, "BG_EFFECT_TARGET", 0, false),
        event(1, "BATTLE_BG_EFFECT_WATER", 0x1c, "$0", 0, false),
    ];
    animation.frame = 2;
    let enemy_water = visible_water_line_y_offsets(Some(&animation)).expect("enemy water");
    assert_eq!(enemy_water[28], visible_battle_anim_sine(4, 3) as i8);
    assert_eq!(enemy_water[27], visible_battle_anim_sine(8, 3) as i8);
    assert_eq!(enemy_water[29], visible_battle_anim_sine(8, 3) as i8);
    assert!(enemy_water[55..].iter().all(|offset| *offset == 0));
    animation.frame = 17;
    assert!(visible_water_line_y_offsets(Some(&animation)).unwrap().iter().all(|offset| *offset == 0));
    animation.bg_events.push(event(18, "BATTLE_BG_EFFECT_END_WATER", 0, "$0", 0, false));
    animation.frame = 18;
    assert!(visible_water_line_y_offsets(Some(&animation)).is_none());

    animation.bg_events = vec![
        event(0, "BATTLE_BG_EFFECT_START_WATER", 0, "BG_EFFECT_USER", 0, false),
        event(1, "BATTLE_BG_EFFECT_WATER", 0x30, "$0", 0, false),
    ];
    animation.frame = 2;
    let player_water = visible_water_line_y_offsets(Some(&animation)).expect("player water");
    assert_eq!(player_water[94], visible_battle_anim_sine(8, 3) as i8);
    assert!(player_water[..47].iter().all(|offset| *offset == 0));
}

#[test]
fn beta_send_out_mon2_decays_its_asm_scx_deformation_for_64_updates() {
    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(), move_id: "BETA_SEND_OUT".to_string(),
        animation_label: "BattleAnim_BetaSendOut".to_string(), player_move: true,
        started: true, waiting_for_hp: false, frame: 0, total_frames: 66,
        sound_events: Vec::new(), next_sound_event: 0, cry_events: Vec::new(),
        next_cry_event: 0, object_events: Vec::new(),
        bg_events: vec![VisibleMoveBgEvent {
            frame: 0, effect_id: "BATTLE_BG_EFFECT_BETA_SEND_OUT_MON2".to_string(),
            duration: 0, target: "BG_EFFECT_USER".to_string(), param: 0,
            incremented: false,
        }], actor_species_override: None, actor_shiny_override: None,
    };
    let setup = visible_beta_send_out_mon2_line_x_offsets(Some(&animation)).expect("setup");
    assert!(setup.iter().all(|offset| *offset == 0));
    animation.frame = 1;
    let radius_eight = visible_beta_send_out_mon2_line_x_offsets(Some(&animation)).unwrap();
    assert!(radius_eight[..=0x2f].iter().all(|offset| *offset == 0));
    assert_eq!(radius_eight[0x30], visible_battle_anim_sine(0x80, 8) as i8);
    assert_eq!(radius_eight[0x31], visible_battle_anim_sine(0x88, 8) as i8);
    animation.frame = 8;
    let radius_seven = visible_beta_send_out_mon2_line_x_offsets(Some(&animation)).unwrap();
    assert_eq!(radius_seven[0x30], visible_battle_anim_sine(0x50, 7) as i8);
    animation.frame = 64;
    assert!(visible_beta_send_out_mon2_line_x_offsets(Some(&animation)).unwrap().iter().all(|offset| *offset == 0));
    animation.frame = 65;
    assert!(visible_beta_send_out_mon2_line_x_offsets(Some(&animation)).is_none());
}

#[test]
fn beta_send_out_mon1_replays_the_two_pass_bgp_scanline_fill() {
    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(), move_id: "BETA_SEND_OUT_1".to_string(),
        animation_label: "BattleAnim_SendOutMon.Unknown".to_string(), player_move: true,
        started: true, waiting_for_hp: false, frame: 0, total_frames: 101,
        sound_events: Vec::new(), next_sound_event: 0, cry_events: Vec::new(),
        next_cry_event: 0, object_events: Vec::new(),
        bg_events: vec![
            VisibleMoveBgEvent {
                frame: 0, effect_id: "BATTLE_BG_EFFECT_BETA_SEND_OUT_MON1".to_string(),
                duration: 0, target: "BG_EFFECT_USER".to_string(), param: 0,
                incremented: false,
            },
            VisibleMoveBgEvent {
                frame: 5, effect_id: "BATTLE_BG_EFFECT_BETA_SEND_OUT_MON1".to_string(),
                duration: 0, target: String::new(), param: 0, incremented: true,
            },
            VisibleMoveBgEvent {
                frame: 101, effect_id: "BATTLE_BG_EFFECT_BETA_SEND_OUT_MON1".to_string(),
                duration: 0, target: String::new(), param: 0, incremented: true,
            },
        ],
        actor_species_override: None, actor_shiny_override: None,
    };
    let setup = visible_beta_send_out_mon1_line_bgps(Some(&animation)).expect("setup BGP");
    assert_eq!(setup[0x2e], 0xe4);
    assert!(setup[0x2f..=0x5e].iter().all(|bgp| *bgp == 0));
    animation.frame = 13;
    let first_pass = visible_beta_send_out_mon1_line_bgps(Some(&animation)).unwrap();
    assert_eq!((first_pass[0x2f], first_pass[0x30], first_pass[0x31]), (0x40, 0, 0x40));
    animation.frame = 38;
    let second_pass = visible_beta_send_out_mon1_line_bgps(Some(&animation)).unwrap();
    assert_eq!((second_pass[0x2f], second_pass[0x30], second_pass[0x31]), (0xe4, 0, 0xe4));
    assert_eq!(second_pass[0x5e], 0);
    animation.frame = 62;
    assert!(visible_beta_send_out_mon1_line_bgps(Some(&animation)).unwrap()[0x2f..=0x5e]
        .iter().all(|bgp| *bgp == 0xe4));
    animation.frame = 101;
    assert!(visible_beta_send_out_mon1_line_bgps(Some(&animation)).is_none());
    assert_eq!(visible_move_battler_offsets(Some(&animation)), (Vec3::ZERO, Vec3::ZERO));
}

#[test]
fn beta_send_out_mon1_bgp_keeps_mapped_shade_zero_opaque() {
    let mut images = Assets::<Image>::default();
    let source = Image::new(
        Extent3d { width: 4, height: 1, depth_or_array_layers: 1 },
        TextureDimension::D2,
        vec![255, 255, 255, 0, 200, 200, 200, 255, 100, 100, 100, 255, 10, 10, 10, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    let source = SpriteFrame {
        handle: images.add(source),
        size: Vec2::new(4.0, 1.0),
    };
    let mut rendered_art = RenderedTilesetArt::default();
    let white = battle_battler_bgp_frame(&mut rendered_art, &mut images, &source, 0x00)
        .expect("all-white BGP frame");
    assert_eq!(
        images.get(&white.handle).unwrap().data,
        [255, 255, 255, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255]
    );

    let stepped = battle_battler_bgp_frame(&mut rendered_art, &mut images, &source, 0x90)
        .expect("stepped BGP frame");
    assert_eq!(
        images.get(&stepped.handle).unwrap().data,
        [255, 255, 255, 0, 255, 255, 255, 255, 200, 200, 200, 255, 100, 100, 100, 255]
    );
}

#[test]
fn enter_and_return_mon_follow_the_asm_four_update_resize_entries() {
    let effect = |effect_id: &str, target: &str| VisibleMoveBgEvent {
        frame: 0,
        effect_id: effect_id.to_string(),
        duration: 0,
        target: target.to_string(),
        param: 0,
        incremented: false,
    };
    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(), move_id: "RESIZE_TEST".to_string(),
        animation_label: "BattleAnim_ResizeTest".to_string(), player_move: true,
        started: true, waiting_for_hp: false, frame: 0, total_frames: 32,
        sound_events: Vec::new(), next_sound_event: 0, cry_events: Vec::new(),
        next_cry_event: 0, object_events: Vec::new(),
        bg_events: vec![effect("BATTLE_BG_EFFECT_ENTER_MON", "BG_EFFECT_USER")],
        actor_species_override: None, actor_shiny_override: None,
    };
    for (frame, expected) in [(0, Some(2)), (3, Some(2)), (4, Some(4)), (8, Some(6)), (11, Some(6)), (12, None)] {
        animation.frame = frame;
        assert_eq!(visible_move_battler_clip_tiles(Some(&animation)).0, expected);
    }

    animation.player_move = false;
    animation.bg_events = vec![effect("BATTLE_BG_EFFECT_RETURN_MON", "BG_EFFECT_USER")];
    for (frame, expected) in [(0, Some(7)), (3, Some(7)), (4, Some(5)), (8, Some(3)), (11, Some(3)), (12, None)] {
        animation.frame = frame;
        assert_eq!(visible_move_battler_clip_tiles(Some(&animation)).1, expected);
        assert_eq!(visible_move_battler_visibility(Some(&animation)).1, frame < 12);
    }
}

#[test]
fn global_dmg_palette_effects_write_registers_on_their_asm_reload_cadence() {
    let event = |frame, effect_id: &str, battle_turn, param| VisibleMoveBgEvent {
        frame,
        effect_id: effect_id.to_string(),
        duration: 0,
        target: format!("${battle_turn:x}"),
        param,
        incremented: false,
    };
    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(), move_id: "PALETTE_TEST".to_string(),
        animation_label: "BattleAnim_PaletteTest".to_string(), player_move: true,
        started: true, waiting_for_hp: false, frame: 0, total_frames: 40,
        sound_events: Vec::new(), next_sound_event: 0, cry_events: Vec::new(),
        next_cry_event: 0, object_events: Vec::new(),
        bg_events: vec![event(0, "BATTLE_BG_EFFECT_FLASH_INVERTED", 4, 3)],
        actor_species_override: None, actor_shiny_override: None,
    };
    for (frame, expected) in [(0, 0xe4), (4, 0xe4), (5, 0x1b), (9, 0x1b), (10, 0xe4)] {
        animation.frame = frame;
        assert_eq!(visible_battle_dmg_palette_registers(Some(&animation)).bgp, expected);
    }

    animation.bg_events = vec![event(0, "BATTLE_BG_EFFECT_WHITE_HUES", 8, 0)];
    for (frame, expected) in [(0, 0xe4), (8, 0xe4), (9, 0xe0), (18, 0xd0), (27, 0xd0)] {
        animation.frame = frame;
        assert_eq!(visible_battle_dmg_palette_registers(Some(&animation)).bgp, expected);
    }

    animation.bg_events = vec![
        event(0, "BATTLE_PALETTE_OBP1", 0, 0x1b),
        event(2, "BATTLE_BG_EFFECT_ALTERNATE_HUES", 2, 0),
    ];
    animation.frame = 2;
    assert_eq!(
        visible_battle_dmg_palette_registers(Some(&animation)),
        VisibleBattleDmgPaletteRegisters { obp0_write_frame: None, bgp: 0xe4, obp0: 0xe4, obp1: 0xe4 }
    );
    animation.frame = 5;
    let registers = visible_battle_dmg_palette_registers(Some(&animation));
    assert_eq!((registers.bgp, registers.obp1), (0xf8, 0xf8));

    for (effect_id, cycled) in [
        ("BATTLE_BG_EFFECT_CYCLE_OBPALS_GRAY_AND_YELLOW", 0x90),
        ("BATTLE_BG_EFFECT_CYCLE_MID_OBPALS_GRAY_AND_YELLOW", 0xd8),
    ] {
        animation.bg_events = vec![event(0, effect_id, 2, 0)];
        for (frame, expected) in [(0, 0xe4), (2, 0xe4), (3, cycled), (6, 0xe4)] {
            animation.frame = frame;
            assert_eq!(visible_battle_dmg_palette_registers(Some(&animation)).obp0, expected);
        }
    }
}

#[test]
fn targeted_battler_palettes_use_rapid_cycle_pals_indexed_cadence() {
    let event = |frame, effect_id: &str, target: &str, param, incremented| VisibleMoveBgEvent {
        frame,
        effect_id: effect_id.to_string(),
        duration: 0,
        target: target.to_string(),
        param,
        incremented,
    };
    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(), move_id: "TARGET_PAL_TEST".to_string(),
        animation_label: "BattleAnim_TargetPalTest".to_string(), player_move: true,
        started: true, waiting_for_hp: false, frame: 0, total_frames: 32,
        sound_events: Vec::new(), next_sound_event: 0, cry_events: Vec::new(),
        next_cry_event: 0, object_events: Vec::new(),
        bg_events: vec![event(0, "BATTLE_BG_EFFECT_CYCLE_MON_LIGHT_DARK_REPEATING", "BG_EFFECT_USER", 0x20, false)],
        actor_species_override: None, actor_shiny_override: None,
    };
    for (frame, expected) in [(0, None), (3, None), (4, Some(0xf8)), (7, Some(0xfc)), (10, Some(0xf8)), (13, None), (16, Some(0x90))] {
        animation.frame = frame;
        assert_eq!(visible_move_battler_bgps(Some(&animation)).0, expected);
    }

    animation.bg_events.push(event(17, "BATTLE_BG_EFFECT_CYCLE_MON_LIGHT_DARK_REPEATING", "", 0, true));
    animation.frame = 17;
    assert_eq!(visible_move_battler_bgps(Some(&animation)), (None, None));

    animation.bg_events = vec![event(0, "BATTLE_BG_EFFECT_FADE_MON_TO_LIGHT", "BG_EFFECT_TARGET", 0x40, false)];
    for (frame, expected) in [(0, None), (5, None), (6, Some(0x90)), (11, Some(0x40)), (16, Some(0x40))] {
        animation.frame = frame;
        assert_eq!(visible_move_battler_bgps(Some(&animation)).1, expected);
    }
}

#[test]
fn lunge_background_effects_match_their_asm_jump_tables() {
    assert_eq!(
        (0_u16..13).map(visible_tackle_lunge_offset).collect::<Vec<_>>(),
        vec![
            Some(0), Some(0), Some(2), Some(4), Some(6), Some(8), Some(10), Some(8),
            Some(6), Some(4), Some(2), Some(0), None,
        ]
    );
    assert_eq!(visible_beta_pursuit_offset(6), Some(-10));
    assert_eq!(visible_beta_pursuit_offset(12), None);

    assert_eq!(
        [0_u16, 1, 2, 5, 6, 20, 39, 40, 41, 44, 45, 46]
            .map(|age| visible_vital_throw_offset(age, Some(40))),
        [
            Some(0), Some(0), Some(-2), Some(-8), Some(-10), Some(-10), Some(-10),
            Some(-10), Some(-8), Some(-2), Some(0), Some(0),
        ]
    );
    assert_eq!(visible_vital_throw_offset(47, Some(40)), None);
}

#[test]
fn bounce_down_uses_the_asm_cosine_until_incremented() {
    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(),
        move_id: "BODY_SLAM".to_string(),
        animation_label: "BattleAnim_BodySlam".to_string(),
        player_move: true,
        started: true,
        waiting_for_hp: false,
        frame: 0,
        total_frames: 40,
        sound_events: Vec::new(),
        next_sound_event: 0,
        cry_events: Vec::new(),
        next_cry_event: 0,
        object_events: Vec::new(),
        bg_events: vec![
            VisibleMoveBgEvent {
                frame: 0,
                effect_id: "BATTLE_BG_EFFECT_BOUNCE_DOWN".to_string(),
                duration: 0,
                target: "BG_EFFECT_USER".to_string(),
                param: 0,
                incremented: false,
            },
            VisibleMoveBgEvent {
                frame: 32,
                effect_id: "BATTLE_BG_EFFECT_BOUNCE_DOWN".to_string(),
                duration: 0,
                target: String::new(),
                param: 0,
                incremented: true,
            },
        ],
        actor_species_override: None,
        actor_shiny_override: None,
    };
    let effect = animation.bg_events[0].clone();

    let offsets = [0_u16, 1, 2, 5, 9, 17, 25, 31, 32].map(|frame| {
        animation.frame = frame;
        visible_bounce_down_offset(&animation, &effect)
    });
    assert_eq!(
        offsets,
        [Some(0), Some(1), Some(2), Some(6), Some(17), Some(33), Some(17), Some(3), None]
    );

    animation.frame = 17;
    let lines = visible_bounce_down_line_y_offsets(Some(&animation)).expect("bounce scanlines");
    assert_eq!(lines[0x2c], 0);
    assert_eq!(lines[0x2d] as u8, 0x90);
    assert_eq!(lines[0x4d] as u8, 0x90);
    assert_eq!(lines[0x4e], !33_u8 as i8);
    assert_eq!(lines[0x5e], !33_u8 as i8);

    animation.frame = 32;
    assert!(visible_bounce_down_line_y_offsets(Some(&animation)).is_none());
}

#[test]
fn flail_combines_the_two_asm_sines_on_target_scanlines() {
    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(),
        move_id: "FLAIL".to_string(),
        animation_label: "BattleAnim_Flail".to_string(),
        player_move: true,
        started: true,
        waiting_for_hp: false,
        frame: 0,
        total_frames: 32,
        sound_events: Vec::new(),
        next_sound_event: 0,
        cry_events: Vec::new(),
        next_cry_event: 0,
        object_events: Vec::new(),
        bg_events: vec![
            VisibleMoveBgEvent {
                frame: 0,
                effect_id: "BATTLE_BG_EFFECT_FLAIL".to_string(),
                duration: 0,
                target: "BG_EFFECT_USER".to_string(),
                param: 0,
                incremented: false,
            },
            VisibleMoveBgEvent {
                frame: 32,
                effect_id: "BATTLE_BG_EFFECT_FLAIL".to_string(),
                duration: 0,
                target: String::new(),
                param: 0,
                incremented: true,
            },
        ],
        actor_species_override: None,
        actor_shiny_override: None,
    };

    for frame in [0_u16, 1, 2, 5, 9, 17, 31] {
        animation.frame = frame;
        let offsets = visible_flail_line_x_offsets(Some(&animation)).expect("active Flail");
        let update = frame.saturating_sub(1) as u8;
        let expected = if frame == 0 {
            0
        } else {
            visible_battle_anim_sine(update.wrapping_mul(2), 6)
                + visible_battle_anim_sine(update.wrapping_mul(8), 2)
        } as i8;
        assert_eq!(offsets[0x2e], 0);
        assert_eq!(offsets[0x2f], expected);
        assert_eq!(offsets[0x5e], expected);
    }
    animation.frame = 32;
    assert!(visible_flail_line_x_offsets(Some(&animation)).is_none());
    assert_eq!(visible_move_battler_offsets(Some(&animation)), (Vec3::ZERO, Vec3::ZERO));
}

#[test]
fn dig_expands_its_vertical_displacement_in_four_step_bursts() {
    assert_eq!(
        [0_u16, 1, 2, 3, 4, 5, 20, 21, 22, 23, 24, 25, 40, 41]
            .map(visible_dig_displacement),
        [0, -3, -5, -7, -9, -9, -9, -11, -13, -15, -17, -17, -17, -19]
    );

    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(),
        move_id: "DIG".to_string(),
        animation_label: "BattleAnim_Dig".to_string(),
        player_move: true,
        started: true,
        waiting_for_hp: false,
        frame: 24,
        total_frames: 136,
        sound_events: Vec::new(),
        next_sound_event: 0,
        cry_events: Vec::new(),
        next_cry_event: 0,
        object_events: Vec::new(),
        bg_events: vec![
            VisibleMoveBgEvent {
                frame: 0,
                effect_id: "BATTLE_BG_EFFECT_DIG".to_string(),
                duration: 0,
                target: "BG_EFFECT_USER".to_string(),
                param: 1,
                incremented: false,
            },
            VisibleMoveBgEvent {
                frame: 136,
                effect_id: "BATTLE_BG_EFFECT_DIG".to_string(),
                duration: 0,
                target: String::new(),
                param: 0,
                incremented: true,
            },
        ],
        actor_species_override: None,
        actor_shiny_override: None,
    };
    let offsets = visible_dig_line_y_offsets(Some(&animation)).expect("active Dig displacement");
    assert_eq!(offsets[0x2e], 0);
    assert_eq!(offsets[0x2f] as u8, 0x90);
    assert_eq!(offsets[0x3e] as u8, 0x90);
    assert_eq!(offsets[0x3f], -17);
    assert_eq!(offsets[0x5e], -17);
    animation.frame = 136;
    assert!(visible_dig_line_y_offsets(Some(&animation)).is_none());
}

#[test]
fn double_team_expands_oscillates_contracts_and_clears_on_two_increments() {
    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(),
        move_id: "DOUBLE_TEAM".to_string(),
        animation_label: "BattleAnim_DoubleTeam".to_string(),
        player_move: true,
        started: true,
        waiting_for_hp: false,
        frame: 0,
        total_frames: 120,
        sound_events: Vec::new(),
        next_sound_event: 0,
        cry_events: Vec::new(),
        next_cry_event: 0,
        object_events: Vec::new(),
        bg_events: vec![
            VisibleMoveBgEvent {
                frame: 0,
                effect_id: "BATTLE_BG_EFFECT_DOUBLE_TEAM".to_string(),
                duration: 0,
                target: "BG_EFFECT_USER".to_string(),
                param: 0,
                incremented: false,
            },
            VisibleMoveBgEvent {
                frame: 96,
                effect_id: "BATTLE_BG_EFFECT_DOUBLE_TEAM".to_string(),
                duration: 0,
                target: String::new(),
                param: 0,
                incremented: true,
            },
            VisibleMoveBgEvent {
                frame: 120,
                effect_id: "BATTLE_BG_EFFECT_DOUBLE_TEAM".to_string(),
                duration: 0,
                target: String::new(),
                param: 0,
                incremented: true,
            },
        ],
        actor_species_override: None,
        actor_shiny_override: None,
    };

    for (frame, expected) in [
        (0_u16, 0_i8),
        (1, 0),
        (2, 1),
        (16, 15),
        (17, 15),
        (18, 16),
        (22, 18),
        (26, 16),
        (30, 14),
        (96, 15),
        (97, 14),
        (111, 0),
        (112, 0),
        (119, 0),
    ] {
        animation.frame = frame;
        let offsets = visible_double_team_line_x_offsets(Some(&animation))
            .expect("active Double Team scanlines");
        assert_eq!(offsets[0x2e], 0);
        assert_eq!(offsets[0x2f], expected, "frame {frame}");
        assert_eq!(offsets[0x30], -expected, "frame {frame}");
        assert_eq!(offsets[0x31], expected, "frame {frame}");
    }
    animation.frame = 120;
    assert!(visible_double_team_line_x_offsets(Some(&animation)).is_none());
}

#[test]
fn acid_armor_shifts_its_vertical_sine_buffer_downward_without_palette_tint() {
    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(),
        move_id: "ACID_ARMOR".to_string(),
        animation_label: "BattleAnim_AcidArmor".to_string(),
        player_move: true,
        started: true,
        waiting_for_hp: false,
        frame: 0,
        total_frames: 64,
        sound_events: Vec::new(),
        next_sound_event: 0,
        cry_events: Vec::new(),
        next_cry_event: 0,
        object_events: Vec::new(),
        bg_events: vec![
            VisibleMoveBgEvent {
                frame: 0,
                effect_id: "BATTLE_BG_EFFECT_ACID_ARMOR".to_string(),
                duration: 0,
                target: "BG_EFFECT_USER".to_string(),
                param: 8,
                incremented: false,
            },
            VisibleMoveBgEvent {
                frame: 64,
                effect_id: "BATTLE_BG_EFFECT_ACID_ARMOR".to_string(),
                duration: 0,
                target: String::new(),
                param: 0,
                incremented: true,
            },
        ],
        actor_species_override: None,
        actor_shiny_override: None,
    };

    let initial = visible_acid_armor_line_y_offsets(Some(&animation)).expect("initial melt");
    assert_eq!(initial[0x2f], 0);
    assert_eq!(initial[0x34], visible_battle_anim_sine(0x34 * 2, 8) as i8);
    assert_eq!(initial[0x5d], 0);
    assert_eq!(initial[0x5e], 0);

    animation.frame = 1;
    let shifted = visible_acid_armor_line_y_offsets(Some(&animation)).expect("shifted melt");
    assert_eq!(shifted[0x2f] as u8, 0x90);
    assert_eq!(shifted[0x30], initial[0x2f]);
    assert_eq!(shifted[0x35], initial[0x34]);
    assert!(
        !matches!(shifted[0x5e] as u8, 1..=0x8f | 0x91..=0xff),
        "tail must retain only zero or the blanking sentinel"
    );

    animation.frame = 64;
    assert!(visible_acid_armor_line_y_offsets(Some(&animation)).is_none());
}

#[test]
fn withdraw_compresses_the_visible_battler_instead_of_hiding_it_immediately() {
    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(),
        move_id: "WITHDRAW".to_string(),
        animation_label: "BattleAnim_Withdraw".to_string(),
        player_move: true,
        started: true,
        waiting_for_hp: false,
        frame: 0,
        total_frames: 113,
        sound_events: Vec::new(),
        next_sound_event: 0,
        cry_events: Vec::new(),
        next_cry_event: 0,
        object_events: Vec::new(),
        bg_events: vec![
            VisibleMoveBgEvent {
                frame: 0,
                effect_id: "BATTLE_BG_EFFECT_WITHDRAW".to_string(),
                duration: 0,
                target: "BG_EFFECT_USER".to_string(),
                param: 0x50,
                incremented: false,
            },
            VisibleMoveBgEvent {
                frame: 113,
                effect_id: "BATTLE_BG_EFFECT_WITHDRAW".to_string(),
                duration: 0,
                target: String::new(),
                param: 0,
                incremented: true,
            },
        ],
        actor_species_override: None,
        actor_shiny_override: None,
    };

    assert_eq!(visible_move_battler_visibility(Some(&animation)), (true, true));
    assert!(
        visible_withdraw_line_y_offsets(Some(&animation))
            .expect("setup buffer")
            .iter()
            .all(|offset| *offset == 0)
    );
    animation.frame = 1;
    let first = visible_withdraw_line_y_offsets(Some(&animation)).expect("first compression");
    assert_eq!(first[0x2e], 0);
    assert_eq!(first[0x2f] as u8, 0x90);
    assert_eq!(first[0x30], -2);
    animation.frame = 15;
    let compressed = visible_withdraw_line_y_offsets(Some(&animation)).expect("compressed user");
    assert!(compressed[0x2f..0x3e].iter().all(|offset| *offset as u8 == 0x90));
    assert_eq!(compressed[0x3e], -16);
    animation.frame = 112;
    assert!(visible_withdraw_line_y_offsets(Some(&animation)).is_some());
    animation.frame = 113;
    assert!(visible_withdraw_line_y_offsets(Some(&animation)).is_none());
}

#[test]
fn transform_reveals_the_loaded_target_picture_only_at_updateactorpic() {
    let actor_event = |frame, effect_id: &str| VisibleMoveBgEvent {
        frame,
        effect_id: effect_id.to_string(),
        duration: 0,
        target: "BG_EFFECT_USER".to_string(),
        param: 0,
        incremented: false,
    };
    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(), move_id: "TRANSFORM".to_string(),
        animation_label: "BattleAnim_Transform".to_string(), player_move: true,
        started: true, waiting_for_hp: false, frame: 0, total_frames: 104,
        sound_events: Vec::new(), next_sound_event: 0, cry_events: Vec::new(),
        next_cry_event: 0, object_events: Vec::new(),
        bg_events: vec![
            actor_event(0, "BATTLE_ACTOR_TRANSFORM"),
            actor_event(48, "BATTLE_ACTOR_UPDATEACTORPIC"),
        ],
        actor_species_override: None, actor_shiny_override: None,
    };
    assert_eq!(
        visible_move_battler_art_overrides(Some(&animation)),
        (VisibleBattlerArtOverride::Unchanged, VisibleBattlerArtOverride::Unchanged)
    );
    animation.frame = 47;
    assert_eq!(
        visible_move_battler_art_overrides(Some(&animation)).0,
        VisibleBattlerArtOverride::Unchanged
    );
    animation.frame = 48;
    assert_eq!(
        visible_move_battler_art_overrides(Some(&animation)),
        (VisibleBattlerArtOverride::Transform, VisibleBattlerArtOverride::Unchanged)
    );

    animation.player_move = false;
    assert_eq!(
        visible_move_battler_art_overrides(Some(&animation)),
        (VisibleBattlerArtOverride::Unchanged, VisibleBattlerArtOverride::Transform)
    );
}

#[test]
fn minimize_reveals_the_temporary_picture_only_at_updateactorpic() {
    let actor_event = |frame, effect_id: &str| VisibleMoveBgEvent {
        frame,
        effect_id: effect_id.to_string(),
        duration: 0,
        target: "BG_EFFECT_USER".to_string(),
        param: 0,
        incremented: false,
    };
    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(), move_id: "MINIMIZE".to_string(),
        animation_label: "BattleAnim_Minimize".to_string(), player_move: true,
        started: true, waiting_for_hp: false, frame: 0, total_frames: 104,
        sound_events: Vec::new(), next_sound_event: 0, cry_events: Vec::new(),
        next_cry_event: 0, object_events: Vec::new(),
        bg_events: vec![
            actor_event(0, "BATTLE_ACTOR_MINIMIZE"),
            actor_event(48, "BATTLE_ACTOR_UPDATEACTORPIC"),
        ],
        actor_species_override: None, actor_shiny_override: None,
    };
    assert_eq!(
        visible_move_battler_art_overrides(Some(&animation)),
        (VisibleBattlerArtOverride::Unchanged, VisibleBattlerArtOverride::Unchanged)
    );
    animation.frame = 47;
    assert_eq!(
        visible_move_battler_art_overrides(Some(&animation)).0,
        VisibleBattlerArtOverride::Unchanged
    );
    animation.frame = 48;
    assert_eq!(
        visible_move_battler_art_overrides(Some(&animation)),
        (VisibleBattlerArtOverride::Minimize, VisibleBattlerArtOverride::Unchanged)
    );
}

#[test]
fn battlerobj_extracts_fixed_head_or_feet_rows_instead_of_resizing_the_battler() {
    let effect = VisibleMoveBgEvent {
        frame: 0,
        effect_id: "BATTLE_BG_EFFECT_BATTLEROBJ_1ROW".to_string(),
        duration: 0,
        target: "BG_EFFECT_USER".to_string(),
        param: 0,
        incremented: false,
    };
    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(), move_id: "ROW_TEST".to_string(),
        animation_label: "BattleAnim_RowTest".to_string(), player_move: true,
        started: true, waiting_for_hp: false, frame: 0, total_frames: 16,
        sound_events: Vec::new(), next_sound_event: 0, cry_events: Vec::new(),
        next_cry_event: 0, object_events: Vec::new(), bg_events: vec![effect],
        actor_species_override: None, actor_shiny_override: None,
    };
    assert_eq!(visible_move_battler_clip_tiles(Some(&animation)), (None, None));
    assert_eq!(
        visible_move_battler_row_extractions(Some(&animation)).0,
        Some(VisibleBattlerRowExtraction { rows: 1, top: true, bg_rows_cleared: false, render_extracted: true })
    );
    animation.frame = 1;
    assert_eq!(
        visible_move_battler_row_extractions(Some(&animation)).0,
        Some(VisibleBattlerRowExtraction { rows: 1, top: true, bg_rows_cleared: true, render_extracted: true })
    );
    animation.bg_events.push(VisibleMoveBgEvent {
        frame: 5, effect_id: "BATTLE_BG_EFFECT_SHOW_MON".to_string(), duration: 0,
        target: "BG_EFFECT_USER".to_string(), param: 0, incremented: false,
    });
    animation.frame = 5;
    assert_eq!(
        visible_move_battler_row_extractions(Some(&animation)).0.unwrap().bg_rows_cleared,
        false
    );
    animation.object_events.push(VisibleMoveObjectEvent {
        frame: 6, command: VisibleMoveObjectCommand::Clear,
    });
    animation.frame = 6;
    assert_eq!(visible_move_battler_row_extractions(Some(&animation)), (None, None));

    animation.object_events.clear();
    animation.bg_events.truncate(1);
    animation.bg_events[0].effect_id = "BATTLE_BG_EFFECT_BATTLEROBJ_2ROW".to_string();
    animation.player_move = false;
    animation.frame = 1;
    assert_eq!(
        visible_move_battler_row_extractions(Some(&animation)).1,
        Some(VisibleBattlerRowExtraction { rows: 2, top: false, bg_rows_cleared: true, render_extracted: true })
    );
}

#[test]
fn extracted_battler_rows_render_as_an_independent_oam_strip() {
    let frame = SpriteFrame {
        handle: Handle::default(),
        size: Vec2::splat(48.0),
    };
    let mut app = App::new();
    app.add_systems(Update, move |mut commands: Commands| {
        spawn_visible_battler_extracted_rows(
            &mut commands,
            &frame,
            Vec2::splat(192.0),
            Vec3::new(100.0, 50.0, 3.0),
            VisibleBattlerRowExtraction {
                rows: 2,
                top: true,
                bg_rows_cleared: true,
                render_extracted: true,
            },
        );
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<
        (&Sprite, &Transform),
        With<BattleCommandMarker>,
    >();
    let rendered = query.iter(app.world()).collect::<Vec<_>>();
    assert_eq!(rendered.len(), 1);
    let (sprite, transform) = rendered[0];
    assert_eq!(sprite.rect, Some(Rect::new(0.0, 0.0, 48.0, 16.0)));
    assert_eq!(sprite.custom_size, Some(Vec2::new(192.0, 64.0)));
    assert_eq!(transform.translation, Vec3::new(100.0, 114.0, 3.02));
}

#[test]
fn remove_mon_shifts_whole_tiles_on_the_asm_jumptable_cadence() {
    assert_eq!(visible_remove_mon_state(0, false), (0, true));
    assert_eq!(visible_remove_mon_state(1, false), (8, true));
    assert_eq!(visible_remove_mon_state(4, false), (8, true));
    assert_eq!(visible_remove_mon_state(5, false), (16, true));
    assert_eq!(visible_remove_mon_state(28, false), (56, true));
    assert_eq!(visible_remove_mon_state(29, false), (64, false));
    assert_eq!(visible_remove_mon_state(32, false), (64, false));

    assert_eq!(visible_remove_mon_state(29, true), (-64, true));
    assert_eq!(visible_remove_mon_state(32, true), (-64, true));
    assert_eq!(visible_remove_mon_state(33, true), (-72, false));
    assert_eq!(visible_remove_mon_state(36, true), (-72, false));

    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(),
        move_id: "REMOVE_MON".to_string(),
        animation_label: "BattleAnim_TestRemoveMon".to_string(),
        player_move: true,
        started: true,
        waiting_for_hp: false,
        frame: 1,
        total_frames: 37,
        sound_events: Vec::new(),
        next_sound_event: 0,
        cry_events: Vec::new(),
        next_cry_event: 0,
        object_events: Vec::new(),
        bg_events: vec![VisibleMoveBgEvent {
            frame: 0,
            effect_id: "BATTLE_BG_EFFECT_REMOVE_MON".to_string(),
            duration: 0,
            target: "BG_EFFECT_USER".to_string(),
            param: 0,
            incremented: false,
        }],
        actor_species_override: None,
        actor_shiny_override: None,
    };
    assert_eq!(
        visible_remove_mon_clips(Some(&animation)),
        (
            Some(VisibleRemoveMonClip {
                source_pixels: 0,
                crop_left: true,
            }),
            None,
        )
    );
    animation.frame = 9;
    assert_eq!(
        visible_remove_mon_clips(Some(&animation)).0,
        Some(VisibleRemoveMonClip {
            source_pixels: 8,
            crop_left: true,
        })
    );
    animation.player_move = false;
    animation.frame = 5;
    assert_eq!(
        visible_remove_mon_clips(Some(&animation)).1,
        Some(VisibleRemoveMonClip {
            source_pixels: 8,
            crop_left: false,
        })
    );
}

#[test]
fn rollout_shakes_screen_vertically_instead_of_lunging_the_battler() {
    let mut animation = VisibleMoveAnimation {
        trigger_message: String::new(),
        move_id: "ROLLOUT".to_string(),
        animation_label: "BattleAnim_Rollout".to_string(),
        player_move: true,
        started: true,
        waiting_for_hp: false,
        frame: 0,
        total_frames: 0x60,
        sound_events: Vec::new(),
        next_sound_event: 0,
        cry_events: Vec::new(),
        next_cry_event: 0,
        object_events: Vec::new(),
        bg_events: vec![VisibleMoveBgEvent {
            frame: 0,
            effect_id: "BATTLE_BG_EFFECT_ROLLOUT".to_string(),
            duration: 0x60,
            target: "$1".to_string(),
            param: 0x01,
            incremented: false,
        }],
        actor_species_override: None,
        actor_shiny_override: None,
    };
    let source_pixel = TILE_SIZE / SOURCE_TILE_SIZE as f32;

    assert_eq!(visible_move_battler_offsets(Some(&animation)), (Vec3::ZERO, Vec3::ZERO));
    assert_eq!(visible_move_screen_offset(Some(&animation)).y, -source_pixel);
    animation.frame = 1;
    assert_eq!(visible_move_screen_offset(Some(&animation)), Vec3::ZERO);

    animation.frame = 0;
    assert_eq!(visible_rollout_object_y_offset(&animation, 0), -1);
    assert_eq!(visible_rollout_object_y_offset(&animation, 1), 0);
    animation.frame = 1;
    assert_eq!(visible_rollout_object_y_offset(&animation, 0), 0);
    animation.frame = 95;
    assert_eq!(visible_rollout_object_y_offset(&animation, 0), 0);
    animation.frame = 96;
    assert_eq!(visible_rollout_object_y_offset(&animation, 0), 0);
}

#[test]
fn master_ball_capture_timeline_includes_the_full_sparkle_wait() {
    let capture = VisibleCaptureAnimation {
        trigger_message: String::new(),
        ball_id: "MASTER_BALL".to_string(),
        animation_shakes: 4,
        blocked: false,
        caught: true,
        started: true,
        complete: false,
        sprites_cleared: false,
        frame: 0,
    };

    assert_eq!(capture.master_ball_special_frame(), Some(92));
    assert_eq!(capture.shake_entry_frame(), 156);
    assert_eq!(capture.change_dex_sound_frame(), 180);
    assert_eq!(capture.bounce_sound_frame(), 212);
    assert_eq!(capture.shake_setup_frame(), 316);
    assert_eq!(capture.first_shake_check_frame(), 364);
    assert_eq!(capture.total_frames(), 508);

    let mut visibility = capture.clone();
    visibility.frame = 155;
    assert!(!visibility.enemy_hidden());
    assert_eq!(visibility.enemy_clip_tiles(), None);
    visibility.frame = 156;
    assert_eq!(visibility.enemy_clip_tiles(), Some(7));
    visibility.frame = 164;
    assert!(visibility.enemy_hidden());
}

#[test]
fn battle_animation_numeric_parser_resolves_canonical_ball_constants() {
    assert_eq!(parse_visible_battle_animation_int("NO_ITEM"), Some(0x00));
    assert_eq!(parse_visible_battle_animation_int("MASTER_BALL"), Some(0x01));
    assert_eq!(parse_visible_battle_animation_int("ULTRA_BALL"), Some(0x02));
    assert_eq!(parse_visible_battle_animation_int("GREAT_BALL"), Some(0x04));
    assert_eq!(parse_visible_battle_animation_int("POKE_BALL"), None);
}

#[test]
fn b_cancels_visible_evolution_before_success_and_restores_exact_pokemon() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell
        .shell
        .add_party_pokemon(
            "DRAGONAIR",
            55,
            Some("BERRY".to_string()),
            None,
            "EVOLUTION_CANCEL_TEST",
            1,
            Dv::from_non_hp(10, 11, 12, 13),
        )
        .expect("add Dragonair");
    let dragonite = runtime_shell
        .runtime
        .data
        .pokemon
        .get("DRAGONITE")
        .expect("Dragonite in compiled pack")
        .clone();
    let original = runtime_shell.shell.session().state.storage.party.pokemon[0]
        .as_ref()
        .expect("Dragonair in party")
        .clone();
    {
        let state = runtime_shell.shell.session_mut().state_mut();
        let evolved = state.storage.party.pokemon[0]
            .as_mut()
            .expect("Dragonair in party");
        evolved.species = dragonite;
        evolved.nickname = "DRAGONITE".to_string();
        evolved.item = None;
        evolved.hp = evolved.hp.saturating_add(17);
        evolved.max_hp = evolved.max_hp.saturating_add(17);
        evolved.attack = evolved.attack.saturating_add(29);
        let evolution_move = crate::core::models::LearnedMove {
            name: "WING_ATTACK".to_string(),
            current_pp: 35,
            pp_ups: 0,
        };
        if let Some(first_move) = evolved.moves.first_mut() {
            *first_move = evolution_move;
        } else {
            evolved.moves.push(evolution_move);
        }
        state.pending_move_learn = Some(crate::core::state::PendingMoveLearn {
            party_index: 0,
            species_id: "DRAGONITE".to_string(),
            level: 55,
            learned_move: crate::core::models::LearnedMove {
                name: "WING_ATTACK".to_string(),
                current_pp: 35,
                pp_ups: 0,
            },
            defer_level_evolution: false,
        });
        state.sync_party_from_storage();
        assert!(!state.pokedex.seen_species.contains("DRAGONITE"));
        assert!(!state.pokedex.caught_species.contains("DRAGONITE"));
    }
    let evolving = "What? DRAGONAIR is evolving!".to_string();
    let evolved = "Congratulations! DRAGONAIR evolved into DRAGONITE!".to_string();
    let pending = "DRAGONAIR is\ntrying to learn\nWING ATTACK.".to_string();
    let report = EvolutionReport {
        target_species: Some("DRAGONITE".to_string()),
        events: vec![crate::core::systems::evolution::EvolutionEvent::Text(
            "EvolvingText",
        )],
        pending_move_learns: vec![crate::core::models::LearnedMove {
            name: "WING_ATTACK".to_string(),
            current_pp: 35,
            pp_ups: 0,
        }],
        cancel_snapshot: Some(Box::new(original.clone())),
    };
    runtime_shell.battle_messages = [evolving.clone(), evolved.clone(), pending.clone()]
        .into_iter()
        .collect();
    runtime_shell.battle_evolution_cries = [("DRAGONITE".to_string(), evolving.clone())]
        .into_iter()
        .collect();
    runtime_shell.battle_sounds_after_messages = [("SFX_CAUGHT_MON".to_string(), evolving.clone())]
        .into_iter()
        .collect();
    runtime_shell
        .battle_evolution_cancellations
        .push_back(VisibleEvolutionCancellation {
            party_index: 0,
            trigger_message: evolving,
            evolved_message: evolved.clone(),
            pending_move_messages: vec![pending.clone()],
            report,
            accepted: false,
        });
    finish_current_battle_message_for_regression(&mut runtime_shell);

    press_visible_b_button(&mut runtime_shell).expect("cancel evolution with B");

    assert_eq!(
        runtime_shell.shell.session().state.storage.party.pokemon[0].as_ref(),
        Some(&original)
    );
    assert!(
        runtime_shell
            .shell
            .session()
            .state
            .pending_move_learn
            .is_none()
    );
    assert!(
        runtime_shell
            .shell
            .session()
            .state
            .pending_move_learn_queue
            .is_empty()
    );
    assert_eq!(
        runtime_shell.battle_messages.front().map(String::as_str),
        Some("Huh? DRAGONAIR\nstopped evolving!")
    );
    assert!(!runtime_shell.battle_messages.contains(&evolved));
    assert!(!runtime_shell.battle_messages.contains(&pending));
    assert!(runtime_shell.battle_evolution_cancellations.is_empty());
    assert!(runtime_shell.battle_evolution_cries.is_empty());
    assert!(runtime_shell.battle_sounds_after_messages.is_empty());
    assert!(
        !runtime_shell
            .shell
            .session()
            .state
            .pokedex
            .seen_species
            .contains("DRAGONITE")
    );
    assert!(
        !runtime_shell
            .shell
            .session()
            .state
            .pokedex
            .caught_species
            .contains("DRAGONITE")
    );
}

#[test]
fn completed_visible_evolution_registers_the_target_species() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell
        .shell
        .add_party_pokemon(
            "DRAGONAIR",
            55,
            None,
            None,
            "EVOLUTION_POKEDEX_TEST",
            1,
            Dv::from_non_hp(10, 11, 12, 13),
        )
        .expect("add Dragonair");
    let dragonite = runtime_shell.runtime.data.pokemon["DRAGONITE"].clone();
    {
        let state = runtime_shell.shell.session_mut().state_mut();
        state.storage.party.pokemon[0]
            .as_mut()
            .expect("Dragonair in party")
            .species = dragonite;
        state.sync_party_from_storage();
    }

    record_visible_completed_evolution(&mut runtime_shell, 0)
        .expect("commit completed evolution Pokedex state");

    let pokedex = &runtime_shell.shell.session().state.pokedex;
    assert!(pokedex.seen_species.contains("DRAGONITE"));
    assert!(pokedex.caught_species.contains("DRAGONITE"));

    runtime_shell.shell.session_mut().state_mut().pokedex = Default::default();
    let intro = "DRAGONAIR is trying to learn WING ATTACK.".to_string();
    runtime_shell
        .battle_evolution_cancellations
        .push_back(VisibleEvolutionCancellation {
            party_index: 0,
            trigger_message: "What? DRAGONAIR is evolving!".to_string(),
            evolved_message: "Congratulations! DRAGONAIR evolved into DRAGONITE!".to_string(),
            pending_move_messages: vec![intro],
            report: EvolutionReport {
                target_species: Some("DRAGONITE".to_string()),
                events: Vec::new(),
                pending_move_learns: vec![crate::core::models::LearnedMove {
                    name: "WING_ATTACK".to_string(),
                    current_pp: 35,
                    pp_ups: 0,
                }],
                cancel_snapshot: None,
            },
            accepted: true,
        });
    complete_visible_accepted_evolution_after_special_boundary(
        &mut runtime_shell,
        "MoveForgotPoofText",
    )
    .expect("intermediate replacement boundary");
    assert!(
        !runtime_shell
            .shell
            .session()
            .state
            .pokedex
            .seen_species
            .contains("DRAGONITE")
    );
    complete_visible_accepted_evolution_after_special_boundary(
        &mut runtime_shell,
        "LearnedMoveText",
    )
    .expect("learned move completes evolution");
    assert!(
        runtime_shell
            .shell
            .session()
            .state
            .pokedex
            .caught_species
            .contains("DRAGONITE")
    );
}

#[test]
fn accepted_visible_evolution_registers_only_after_the_evolved_text() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell
        .shell
        .add_party_pokemon(
            "DRAGONAIR",
            55,
            None,
            None,
            "EVOLUTION_POKEDEX_TIMING_TEST",
            1,
            Dv::from_non_hp(10, 11, 12, 13),
        )
        .expect("add Dragonair");
    let original = runtime_shell.shell.session().state.storage.party.pokemon[0]
        .as_ref()
        .expect("Dragonair in party")
        .clone();
    let dragonite = runtime_shell.runtime.data.pokemon["DRAGONITE"].clone();
    runtime_shell.shell.session_mut().state_mut().storage.party.pokemon[0]
        .as_mut()
        .expect("Dragonair in party")
        .species = dragonite;
    let evolving = "What? DRAGONAIR is evolving!".to_string();
    let evolved = "Congratulations! DRAGONAIR evolved into DRAGONITE!".to_string();
    runtime_shell.battle_messages = [evolving.clone(), evolved.clone()].into_iter().collect();
    runtime_shell
        .battle_evolution_cancellations
        .push_back(VisibleEvolutionCancellation {
            party_index: 0,
            trigger_message: evolving,
            evolved_message: evolved,
            pending_move_messages: Vec::new(),
            report: EvolutionReport {
                target_species: Some("DRAGONITE".to_string()),
                events: Vec::new(),
                pending_move_learns: Vec::new(),
                cancel_snapshot: Some(Box::new(original)),
            },
            accepted: false,
        });

    finish_current_battle_message_for_regression(&mut runtime_shell);
    press_visible_a_button(&mut runtime_shell).expect("accept evolution");

    assert!(runtime_shell.battle_evolution_cancellations[0].accepted);
    assert!(
        !runtime_shell
            .shell
            .session()
            .state
            .pokedex
            .seen_species
            .contains("DRAGONITE")
    );

    runtime_shell
        .shell
        .session_mut()
        .state_mut()
        .sync_party_from_storage();
    finish_current_battle_message_for_regression(&mut runtime_shell);
    press_visible_a_button(&mut runtime_shell).expect("dismiss evolved text");

    assert!(runtime_shell.battle_evolution_cancellations.is_empty());
    let pokedex = &runtime_shell.shell.session().state.pokedex;
    assert!(pokedex.seen_species.contains("DRAGONITE"));
    assert!(pokedex.caught_species.contains("DRAGONITE"));
}

#[test]
fn accepted_evolution_with_a_move_registers_at_the_move_result_boundary() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell
        .shell
        .add_party_pokemon(
            "DRAGONAIR",
            55,
            None,
            None,
            "EVOLUTION_MOVE_POKEDEX_TIMING_TEST",
            1,
            Dv::from_non_hp(10, 11, 12, 13),
        )
        .expect("add Dragonair");
    let original = runtime_shell.shell.session().state.storage.party.pokemon[0]
        .as_ref()
        .expect("Dragonair in party")
        .clone();
    let dragonite = runtime_shell.runtime.data.pokemon["DRAGONITE"].clone();
    {
        let state = runtime_shell.shell.session_mut().state_mut();
        state.storage.party.pokemon[0]
            .as_mut()
            .expect("Dragonair in party")
            .species = dragonite;
        state.sync_party_from_storage();
    }
    let intro = "DRAGONAIR is trying to learn WING ATTACK.".to_string();
    let result = "DRAGONAIR did not learn WING ATTACK.".to_string();
    runtime_shell
        .battle_evolution_cancellations
        .push_back(VisibleEvolutionCancellation {
            party_index: 0,
            trigger_message: "What? DRAGONAIR is evolving!".to_string(),
            evolved_message: "Congratulations! DRAGONAIR evolved into DRAGONITE!".to_string(),
            pending_move_messages: vec![intro.clone(), result.clone()],
            report: EvolutionReport {
                target_species: Some("DRAGONITE".to_string()),
                events: Vec::new(),
                pending_move_learns: vec![crate::core::models::LearnedMove {
                    name: "WING_ATTACK".to_string(),
                    current_pp: 35,
                    pp_ups: 0,
                }],
                cancel_snapshot: Some(Box::new(original)),
            },
            accepted: true,
        });

    complete_visible_accepted_evolution_after_battle_message(
        &mut runtime_shell,
        Some(&intro),
    )
    .expect("intro is not completion");
    assert!(
        !runtime_shell
            .shell
            .session()
            .state
            .pokedex
            .seen_species
            .contains("DRAGONITE")
    );

    complete_visible_accepted_evolution_after_battle_message(
        &mut runtime_shell,
        Some(&result),
    )
    .expect("result completes evolution");

    assert!(runtime_shell.battle_evolution_cancellations.is_empty());
    let pokedex = &runtime_shell.shell.session().state.pokedex;
    assert!(pokedex.seen_species.contains("DRAGONITE"));
    assert!(pokedex.caught_species.contains("DRAGONITE"));
}

#[test]
fn pending_move_learn_prompt_uses_the_exported_asm_question() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell
        .shell
        .add_party_pokemon(
            "CYNDAQUIL",
            20,
            None,
            None,
            "PENDING_MOVE_TEXT_TEST",
            1,
            Dv::from_non_hp(10, 11, 12, 13),
        )
        .expect("add Cyndaquil");
    runtime_shell.shell.session_mut().state_mut().pending_move_learn =
        Some(crate::core::state::PendingMoveLearn {
            party_index: 0,
            species_id: "CYNDAQUIL".to_string(),
            level: 20,
            learned_move: crate::core::models::LearnedMove {
                name: "HEADBUTT".to_string(),
                current_pp: 15,
                pp_ups: 0,
            },
            defer_level_evolution: false,
        });
    runtime_shell.shell.session_mut().state_mut().sync_party_from_storage();

    let expected = visible_move_learning_text_pages(
        &runtime_shell,
        "_AskForgetMoveText",
        "CYNDAQUIL",
        "CYNDAQUIL",
        "HEADBUTT",
    )
    .expect("render exported move-learning text")
    .pop()
    .expect("final source question");
    let snapshot = runtime_shell
        .shell
        .presentation_snapshot()
        .expect("pending move presentation snapshot");
    let mut entries = Vec::new();
    push_visible_pending_move_learn_entries(&mut entries, &snapshot, &runtime_shell)
        .expect("render pending move learn prompt");

    assert_eq!(entries, vec![expected]);
    assert!(!entries.iter().any(|entry| entry.contains("A/B CONTINUE")));
}

#[test]
fn pending_move_replacement_uses_source_text_pause_and_sound_boundaries() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell
        .shell
        .add_party_pokemon(
            "CYNDAQUIL",
            20,
            None,
            None,
            "PENDING_MOVE_REPLACE_TEST",
            1,
            Dv::from_non_hp(10, 11, 12, 13),
        )
        .expect("add Cyndaquil");
    {
        let state = runtime_shell.shell.session_mut().state_mut();
        let pokemon = state.storage.party.pokemon[0]
            .as_mut()
            .expect("party Pokemon");
        pokemon.moves = ["TACKLE", "LEER", "SMOKESCREEN", "EMBER"]
            .into_iter()
            .map(|name| crate::core::models::LearnedMove {
                name: name.to_string(),
                current_pp: 20,
                pp_ups: 0,
            })
            .collect();
        state.pending_move_learn = Some(crate::core::state::PendingMoveLearn {
            party_index: 0,
            species_id: "CYNDAQUIL".to_string(),
            level: 20,
            learned_move: crate::core::models::LearnedMove {
                name: "HEADBUTT".to_string(),
                current_pp: 15,
                pp_ups: 0,
            },
            defer_level_evolution: false,
        });
        state.sync_party_from_storage();
    }
    runtime_shell.party_move_cursor = Some(MenuCursor {
        surface_id: party_move_cursor_surface_id(0),
        option_index: 0,
    });

    replace_visible_pending_move_learn(&mut runtime_shell)
        .expect("replace the selected move");

    assert_eq!(
        runtime_shell
            .special_boundary
            .as_ref()
            .map(|boundary| boundary.label.as_str()),
        Some("Text_1_2_and_Poof")
    );
    assert_eq!(runtime_shell.visible_special_text_pause_frames, Some(30));
    assert_eq!(
        runtime_shell
            .special_boundary_queue
            .front()
            .map(|boundary| boundary.label.as_str()),
        Some("MoveForgotPoofText")
    );
}

fn capture_ball_sprite_count(world: &mut World) -> usize {
    let mut commands = world.query_filtered::<&Transform, With<BattleCommandMarker>>();
    commands
        .iter(world)
        .filter(|transform| (transform.translation.z - 4.1).abs() < f32::EPSILON)
        .count()
}

fn assert_caught_capture_render_state(world: &mut World, ball_visible: bool) {
    let runtime_shell = world.resource::<BevyRuntimeShell>();
    let capture = runtime_shell
        .visible_capture_animation
        .as_ref()
        .expect("caught capture presentation must remain retained");
    assert!(capture.complete && capture.caught);
    assert!(
        capture.enemy_hidden(),
        "the still-live core enemy must stay hidden until capture commit"
    );
    assert_eq!(capture.ball_visible(), ball_visible);
    assert_eq!(
        runtime_shell.last_error, None,
        "capture presentation must render without a hidden asset error"
    );
    let _ = runtime_shell;

    let mut battlers = world.query_filtered::<Entity, With<BattleBattlerMarker>>();
    assert_eq!(
        battlers.iter(world).count(),
        1,
        "only the player battler may remain while the caught enemy is pending commit"
    );
    assert_eq!(
        capture_ball_sprite_count(world),
        ball_visible as usize,
        "the retained Poké Ball entity must follow the source ClearSprites boundary"
    );
}

#[test]
fn battle_screen_offset_moves_battlers_and_commands_but_not_fixed_canvas_and_restores() {
    let mut runtime_shell = core_modular_title_shell_for_test();
    runtime_shell
        .visible_move_animations
        .push_back(VisibleMoveAnimation {
            trigger_message: "screen shake".to_string(),
            move_id: "TEST_SHAKE".to_string(),
            animation_label: "BattleAnim_TestShake".to_string(),
            player_move: true,
            started: true,
            waiting_for_hp: false,
            frame: 0,
            total_frames: 4,
            sound_events: Vec::new(),
            next_sound_event: 0,
            cry_events: Vec::new(),
            next_cry_event: 0,
            object_events: Vec::new(),
            bg_events: vec![
                VisibleMoveBgEvent {
                    frame: 0,
                    effect_id: "BATTLE_BG_EFFECT_SHAKE_SCREEN_X".to_string(),
                    duration: 4,
                    target: "3".to_string(),
                    param: 0x12,
                    incremented: false,
                },
                VisibleMoveBgEvent {
                    frame: 0,
                    effect_id: "BATTLE_BG_EFFECT_SHAKE_SCREEN_Y".to_string(),
                    duration: 4,
                    target: "2".to_string(),
                    param: 0x12,
                    incremented: false,
                },
            ],
            actor_species_override: None,
            actor_shiny_override: None,
        });
    let expected_offset = visible_move_screen_offset(runtime_shell.visible_move_animations.front());
    assert_ne!(
        expected_offset,
        Vec3::ZERO,
        "fixture must produce a screen shake"
    );

    let battler_origin = Vec3::new(10.0, 20.0, 3.0);
    let command_origin = Vec3::new(-4.0, 7.0, 4.0);
    let canvas_origin = Vec3::new(1.0, 2.0, 2.7);
    let mut app = App::new();
    app.insert_resource(runtime_shell)
        .add_systems(Update, apply_visible_battle_screen_offset);
    let battler = app
        .world_mut()
        .spawn((
            Transform::from_translation(battler_origin),
            BattleBattlerMarker,
        ))
        .id();
    let command = app
        .world_mut()
        .spawn((
            Transform::from_translation(command_origin),
            BattleCommandMarker,
        ))
        .id();
    let canvas = app
        .world_mut()
        .spawn((
            Transform::from_translation(canvas_origin),
            BattleCommandMarker,
            FixedBattleCanvasMarker,
        ))
        .id();

    app.update();
    assert_eq!(
        app.world()
            .entity(battler)
            .get::<Transform>()
            .unwrap()
            .translation,
        battler_origin + expected_offset
    );
    assert_eq!(
        app.world()
            .entity(command)
            .get::<Transform>()
            .unwrap()
            .translation,
        command_origin + expected_offset
    );
    assert_eq!(
        app.world()
            .entity(canvas)
            .get::<Transform>()
            .unwrap()
            .translation,
        canvas_origin,
        "the full-screen battle canvas must remain anchored to the LCD"
    );

    app.world_mut()
        .resource_mut::<BevyRuntimeShell>()
        .visible_move_animations
        .clear();
    app.update();
    assert_eq!(
        app.world()
            .entity(battler)
            .get::<Transform>()
            .unwrap()
            .translation,
        battler_origin,
        "battler offset must be removed when the shake ends"
    );
    assert_eq!(
        app.world()
            .entity(command)
            .get::<Transform>()
            .unwrap()
            .translation,
        command_origin,
        "command/HUD offset must be removed when the shake ends"
    );
    assert_eq!(
        app.world()
            .entity(canvas)
            .get::<Transform>()
            .unwrap()
            .translation,
        canvas_origin
    );
}

#[test]
fn caught_capture_retains_then_clears_sprites_without_revealing_enemy_before_commit() {
    let mut runtime_shell = route36_battle_shell_for_render_regression();
    runtime_shell.visible_battle_transition = None;
    runtime_shell.battle_entry_messages_remaining = 0;
    runtime_shell.battle_enemy_send_out_pending = false;
    runtime_shell.battle_player_send_out_pending = false;
    runtime_shell.battle_messages.clear();
    runtime_shell.battle_message_scenes.clear();
    let battle_scene = runtime_shell
        .shell
        .snapshot()
        .expect("active capture battle snapshot");
    let unrelated = "An unrelated queued message.".to_string();
    let gotcha = "Gotcha! SUDOWOODO\nwas caught!".to_string();
    let pokedex = "SUDOWOODO's data\nwas newly added to\nthe POKéDEX.".to_string();
    runtime_shell.battle_messages.push_back(unrelated);
    runtime_shell.battle_messages.push_back(gotcha.clone());
    runtime_shell.battle_messages.push_back(pokedex.clone());
    runtime_shell.battle_message_scene = Some(Box::new(battle_scene));
    runtime_shell.visible_capture_animation = Some(VisibleCaptureAnimation {
        trigger_message: "Player used POKé BALL!".to_string(),
        ball_id: "POKE_BALL".to_string(),
        animation_shakes: 3,
        blocked: false,
        caught: true,
        started: false,
        complete: true,
        sprites_cleared: false,
        frame: 228 + 48 * 3,
    });
    let outcome = crate::core::battle::capture::CaptureOutcome {
        caught: true,
        blocked: false,
        storage_full: false,
        wobble_count: 3,
        animation_shakes: 3,
        final_catch_rate: u8::MAX,
        ball_id: Some("POKE_BALL".to_string()),
    };
    runtime_shell.pending_standard_capture = Some(PendingStandardCapture {
        outcome,
        scripted_static_wild: None,
        default_name: "SUDOWOODO".to_string(),
        prompt_for_nickname: true,
    });
    finish_current_battle_message_for_regression(&mut runtime_shell);

    let mut app = battle_render_regression_app(runtime_shell);
    app.update();
    assert_caught_capture_render_state(app.world_mut(), true);

    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        press_visible_a_button(&mut runtime_shell).expect("dismiss unrelated battle page");
        assert_eq!(runtime_shell.battle_messages.front(), Some(&gotcha));
        assert!(
            runtime_shell
                .visible_capture_animation
                .as_ref()
                .is_some_and(|capture| !capture.sprites_cleared),
            "an unrelated page must not clear the retained caught ball"
        );
    }
    app.update();
    assert_caught_capture_render_state(app.world_mut(), true);

    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        finish_current_battle_message_for_regression(&mut runtime_shell);
        press_visible_a_button(&mut runtime_shell).expect("dismiss Gotcha page");
        assert_eq!(runtime_shell.battle_messages.front(), Some(&pokedex));
        assert!(
            runtime_shell
                .visible_capture_animation
                .as_ref()
                .is_some_and(|capture| capture.sprites_cleared),
            "Gotcha dismissal must execute the source ClearSprites boundary"
        );
    }
    app.update();
    assert_caught_capture_render_state(app.world_mut(), false);

    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        finish_current_battle_message_for_regression(&mut runtime_shell);
        press_visible_a_button(&mut runtime_shell).expect("open scripted Pokedex entry");
        assert!(runtime_shell.pokedex_scripted_entry);
        assert!(runtime_shell.pokedex_detail_open);
        assert_eq!(
            runtime_shell
                .last_audio_events
                .iter()
                .filter(|event| event.contains("queued new_pokedex_entry cry"))
                .count(),
            1,
            "NewPokedexEntry is the capture flow's only species-cry boundary"
        );
    }
    app.update();
    assert_caught_capture_render_state(app.world_mut(), false);
    assert!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .battle_messages
            .is_empty(),
        "opening the scripted Pokedex entry must consume the final capture text"
    );

    for step in 0..8 {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        if !runtime_shell.pokedex_menu_open {
            break;
        }
        let capture_pending_before = runtime_shell.pending_standard_capture.is_some();
        press_visible_a_button(&mut runtime_shell).expect("advance scripted Pokedex entry");
        assert!(
            runtime_shell.battle_messages.is_empty(),
            "scripted Pokedex step {step} resumed the unfinished battle: capture_before={} capture_after={} name_choice={} {:?}",
            capture_pending_before,
            runtime_shell.pending_standard_capture.is_some(),
            runtime_shell.pending_name_choice.is_some(),
            runtime_shell.battle_messages,
        );
    }
    {
        let runtime_shell = app.world().resource::<BevyRuntimeShell>();
        assert!(
            !runtime_shell.pokedex_menu_open,
            "scripted Pokedex entry did not close: detail={} page={} battle_messages={:?} action={:?}",
            runtime_shell.pokedex_detail_open,
            runtime_shell.pokedex_detail_page,
            runtime_shell.battle_messages,
            runtime_shell.last_runtime_action
        );
        assert!(runtime_shell.pending_name_choice.is_some());
        assert!(runtime_shell.visible_capture_animation.is_some());
    }
    app.update();
    assert_caught_capture_render_state(app.world_mut(), false);

    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        confirm_visible_name_choice(&mut runtime_shell).expect("scroll capture nickname question");
        assert!(runtime_shell.pending_name_input.is_none());
        confirm_visible_name_choice(&mut runtime_shell).expect("choose to nickname capture");
        assert!(runtime_shell.pending_name_input.is_some());
        assert!(runtime_shell.visible_capture_animation.is_some());
    }
    app.update();
    assert_caught_capture_render_state(app.world_mut(), false);

    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        confirm_visible_player_name_input(&mut runtime_shell)
            .expect("commit captured Pokemon nickname");
        assert!(runtime_shell.pending_standard_capture.is_none());
        assert!(runtime_shell.shell.snapshot().unwrap().battle.is_none());
        assert!(
            runtime_shell
                .last_audio_events
                .iter()
                .all(|event| !event.contains("battle_capture_complete cry")),
            "capture storage/exit must not invent a second species cry after NewPokedexEntry"
        );
        assert!(
            runtime_shell.visible_capture_animation.is_none(),
            "capture presentation must clear only after authoritative capture commit"
        );
    }
    app.update();
    let world = app.world_mut();
    let mut battlers = world.query_filtered::<Entity, With<BattleBattlerMarker>>();
    assert_eq!(battlers.iter(world).count(), 0);
    assert_eq!(capture_ball_sprite_count(world), 0);
}

#[test]
fn new_contest_capture_stays_live_through_pokedex_then_skips_nickname() {
    let mut runtime_shell = route36_battle_shell_for_render_regression();
    runtime_shell.visible_battle_transition = None;
    runtime_shell.battle_entry_messages_remaining = 0;
    runtime_shell.battle_enemy_send_out_pending = false;
    runtime_shell.battle_player_send_out_pending = false;
    runtime_shell.battle_messages.clear();
    runtime_shell.battle_message_scenes.clear();
    let contest_pokemon = runtime_shell
        .runtime
        .data
        .create_pokemon("PIDGEY", 4, Dv::from_non_hp(10, 10, 10, 10))
        .expect("materialize a canonical Route36 Contest encounter");
    {
        let state = runtime_shell.shell.session_mut().state_mut();
        let crate::core::state::BattleMemory::StaticWild {
            battle_music,
            roaming_slot,
            ..
        } = state.battle.clone()
        else {
            panic!("contest capture fixture requires a static wild battle");
        };
        state.battle = crate::core::state::BattleMemory::Wild {
            battle_type: "BATTLETYPE_CONTEST".to_string(),
            battle_music,
            map_name: "Route36".to_string(),
            roaming_slot,
            enemy_pokemon: contest_pokemon.clone(),
            enemy_party: vec![contest_pokemon.clone()],
        };
        if let Some(combat) = state.script_runtime.active_battle_combat.as_mut() {
            combat.enemy = contest_pokemon.clone();
            combat.enemy_party = vec![contest_pokemon.clone()];
            combat.enemy_party_index = 0;
        }
    }
    let battle_scene = runtime_shell
        .shell
        .snapshot()
        .expect("active Contest capture battle snapshot");
    let gotcha = "Gotcha! PIDGEY\nwas caught!".to_string();
    let pokedex = "PIDGEY's data\nwas newly added to\nthe POKéDEX.".to_string();
    runtime_shell.battle_messages.push_back(gotcha.clone());
    runtime_shell.battle_messages.push_back(pokedex.clone());
    runtime_shell.battle_message_scene = Some(Box::new(battle_scene));
    runtime_shell.visible_capture_animation = Some(VisibleCaptureAnimation {
        trigger_message: "Player used POKé BALL!".to_string(),
        ball_id: "POKE_BALL".to_string(),
        animation_shakes: 3,
        blocked: false,
        caught: true,
        started: true,
        complete: true,
        sprites_cleared: false,
        frame: 228 + 48 * 3,
    });
    runtime_shell.pending_standard_capture = Some(PendingStandardCapture {
        outcome: crate::core::battle::capture::CaptureOutcome {
            caught: true,
            blocked: false,
            storage_full: false,
            wobble_count: 3,
            animation_shakes: 3,
            final_catch_rate: u8::MAX,
            ball_id: Some("POKE_BALL".to_string()),
        },
        scripted_static_wild: None,
        default_name: "PIDGEY".to_string(),
        prompt_for_nickname: false,
    });

    finish_current_battle_message_for_regression(&mut runtime_shell);
    press_visible_a_button(&mut runtime_shell).expect("dismiss Contest Gotcha page");
    assert_eq!(runtime_shell.battle_messages.front(), Some(&pokedex));
    assert!(runtime_shell.shell.snapshot().unwrap().battle.is_some());

    finish_current_battle_message_for_regression(&mut runtime_shell);
    press_visible_a_button(&mut runtime_shell).expect("open Contest Pokedex entry");
    assert!(runtime_shell.pokedex_scripted_entry);
    assert!(runtime_shell.shell.snapshot().unwrap().battle.is_some());

    for _ in 0..8 {
        if !runtime_shell.pokedex_menu_open {
            break;
        }
        press_visible_a_button(&mut runtime_shell).expect("advance Contest Pokedex entry");
    }
    assert!(!runtime_shell.pokedex_menu_open);
    assert!(runtime_shell.pending_standard_capture.is_none());
    assert!(runtime_shell.pending_name_choice.is_none());
    assert!(runtime_shell.pending_name_input.is_none());
    let snapshot = runtime_shell.shell.snapshot().expect("completed Contest capture");
    assert!(snapshot.battle.is_none());
    assert_eq!(
        snapshot
            .bug_contest
            .caught_mon
            .as_ref()
            .map(|pokemon| pokemon.species.id.as_str()),
        Some("PIDGEY")
    );
}

fn contest_replacement_shell_for_regression() -> (BevyRuntimeShell, crate::core::models::Pokemon) {
    let mut runtime_shell = route36_battle_shell_for_render_regression();
    runtime_shell.visible_battle_transition = None;
    runtime_shell.battle_entry_messages_remaining = 0;
    runtime_shell.battle_enemy_send_out_pending = false;
    runtime_shell.battle_player_send_out_pending = false;
    runtime_shell.battle_messages.clear();
    runtime_shell.battle_message_scenes.clear();
    let previous = runtime_shell
        .runtime
        .data
        .create_pokemon("LEDYBA", 5, Dv::from_non_hp(8, 8, 8, 8))
        .expect("materialize prior Contest catch");
    let candidate = runtime_shell
        .runtime
        .data
        .create_pokemon("PIDGEY", 4, Dv::from_non_hp(10, 10, 10, 10))
        .expect("materialize candidate Contest catch");
    {
        let state = runtime_shell.shell.session_mut().state_mut();
        let crate::core::state::BattleMemory::StaticWild {
            battle_music,
            roaming_slot,
            ..
        } = state.battle.clone()
        else {
            panic!("Contest replacement fixture requires a static wild battle");
        };
        state.battle = crate::core::state::BattleMemory::Wild {
            battle_type: "BATTLETYPE_CONTEST".to_string(),
            battle_music,
            map_name: "Route36".to_string(),
            roaming_slot,
            enemy_pokemon: candidate.clone(),
            enemy_party: vec![candidate.clone()],
        };
        state.bug_contest.caught_mon = Some(previous);
        if let Some(combat) = state.script_runtime.active_battle_combat.as_mut() {
            combat.enemy = candidate.clone();
            combat.enemy_party = vec![candidate.clone()];
            combat.enemy_party_index = 0;
        }
    }
    (runtime_shell, candidate)
}

fn complete_contest_replacement_for_regression(runtime_shell: &mut BevyRuntimeShell) {
    complete_visible_standard_capture(
        runtime_shell,
        crate::core::battle::capture::CaptureOutcome {
            caught: true,
            blocked: false,
            storage_full: false,
            wobble_count: 3,
            animation_shakes: 3,
            final_catch_rate: u8::MAX,
            ball_id: Some("PARK_BALL".to_string()),
        },
        None,
        None,
    )
    .expect("stage Contest replacement");
    finish_current_battle_message_for_regression(runtime_shell);
    press_visible_a_button(runtime_shell).expect("dismiss already-caught text");
}

#[test]
fn contest_replacement_no_keeps_stock_mon_and_exits_after_stats_prompt() {
    let (mut runtime_shell, candidate) = contest_replacement_shell_for_regression();
    complete_contest_replacement_for_regression(&mut runtime_shell);

    assert_eq!(
        runtime_shell
            .visible_bug_contest_replacement
            .as_ref()
            .map(|replacement| replacement.phase),
        Some(VisibleBugContestReplacementPhase::StatsPrompt)
    );
    assert_eq!(runtime_shell.yes_no_cursor.as_ref().unwrap().option_index, 0);
    let snapshot = runtime_shell.shell.snapshot().unwrap();
    assert_eq!(
        snapshot.bug_contest.caught_mon.as_ref().unwrap().species.id,
        "LEDYBA"
    );
    assert_eq!(
        snapshot
            .bug_contest
            .pending_caught_mon
            .as_ref()
            .unwrap()
            .species
            .id,
        candidate.species.id
    );

    let mut app = battle_render_regression_app(runtime_shell);
    app.update();
    assert!(
        app.world().resource::<BevyRuntimeShell>().last_error.is_none(),
        "Contest stats comparison failed to render"
    );
    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        press_visible_b_button(&mut runtime_shell).expect("keep prior Contest catch");
        assert!(runtime_shell.visible_bug_contest_replacement.is_none());
        let snapshot = runtime_shell.shell.snapshot().unwrap();
        assert_eq!(
            snapshot.bug_contest.caught_mon.as_ref().unwrap().species.id,
            "LEDYBA"
        );
        assert!(snapshot.bug_contest.pending_caught_mon.is_none());
        assert!(snapshot.battle.is_none());
    }
}

#[test]
fn contest_replacement_yes_commits_candidate_then_waits_for_caught_text() {
    let (mut runtime_shell, candidate) = contest_replacement_shell_for_regression();
    complete_contest_replacement_for_regression(&mut runtime_shell);
    press_visible_a_button(&mut runtime_shell).expect("switch to candidate Contest catch");

    assert_eq!(
        runtime_shell
            .visible_bug_contest_replacement
            .as_ref()
            .map(|replacement| replacement.phase),
        Some(VisibleBugContestReplacementPhase::CaughtText)
    );
    assert_eq!(runtime_shell.field_notice.as_deref(), Some("Caught PIDGEY!"));
    let snapshot = runtime_shell.shell.snapshot().unwrap();
    assert_eq!(
        snapshot.bug_contest.caught_mon.as_ref().unwrap().species.id,
        candidate.species.id
    );
    assert!(snapshot.bug_contest.pending_caught_mon.is_none());
    assert!(
        runtime_shell.visible_walk_warp_phase.is_none(),
        "caught text must precede the battle map reload"
    );

    let caught_text = runtime_shell.field_notice.clone().unwrap();
    runtime_shell.field_text_reveal = Some(VisibleFieldTextReveal {
        text: caught_text.clone(),
        page_index: 0,
        visible_chars: caught_text.chars().count(),
        frames_until_next_char: 0,
    });
    press_visible_a_button(&mut runtime_shell).expect("dismiss Contest caught text");
    assert!(runtime_shell.field_notice.is_none());
    assert!(runtime_shell.visible_bug_contest_replacement.is_none());
    assert!(runtime_shell.shell.snapshot().unwrap().battle.is_none());
}

#[test]
fn empty_terminal_battle_reward_starts_plain_map_reload_before_releasing_frame() {
    let mut runtime_shell = route36_overworld_shell_for_battle_render_regression();
    let terminal_scene = runtime_shell
        .shell
        .snapshot()
        .expect("terminal retained battle frame fixture");
    assert!(terminal_scene.battle.is_none());
    runtime_shell.battle_messages.clear();
    runtime_shell.battle_exp_tween = None;
    runtime_shell.pending_battle_exp_tweens.clear();
    runtime_shell.battle_level_stats.clear();
    runtime_shell.battle_message_scene = Some(Box::new(terminal_scene));
    runtime_shell.pending_plain_battle_map_reload = true;

    assert!(
        finish_visible_empty_battle_reward_presentation(&mut runtime_shell)
            .expect("finish empty terminal reward presentation")
    );
    assert!(runtime_shell.battle_message_scene.is_none());
    assert!(!runtime_shell.pending_plain_battle_map_reload);
    assert_eq!(
        runtime_shell.visible_walk_warp_phase,
        Some(VisibleWalkWarpPhase::MapReloadFadeIn)
    );
    let fade = runtime_shell
        .screen_fade
        .expect("plain battle exit must arm its white reload fade");
    assert_eq!(fade.color, ScriptFadeColor::White);
    assert_eq!(fade.direction, ScriptFadeDirection::In);
    assert_eq!(fade.alpha, 255);
}

#[test]
fn ordinary_cave_transition_wave_uses_old_offset_accumulator_and_native_scanline_angles() {
    const PREFIX_FRAMES: u16 = 3;
    const FLASH_FRAMES: u16 = 75;
    const BETWEEN_FRAMES: u16 = 2;

    let mut runtime_shell = route36_battle_shell_for_render_regression();
    runtime_shell.visible_battle_transition = Some(VisibleBattleTransition {
        frame: PREFIX_FRAMES + FLASH_FRAMES + BETWEEN_FRAMES,
        stronger_enemy: false,
        cave_environment: true,
        trainer_battle: false,
    });
    mark_runtime_snapshot_dirty(&mut runtime_shell);
    let mut app = battle_render_regression_app(runtime_shell);

    for outro in [0_u16, 1, 2, 5, 14] {
        {
            let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            runtime_shell.visible_battle_transition = Some(VisibleBattleTransition {
                frame: PREFIX_FRAMES + FLASH_FRAMES + BETWEEN_FRAMES + outro,
                stronger_enemy: false,
                cave_environment: true,
                trainer_battle: false,
            });
            mark_runtime_snapshot_dirty(&mut runtime_shell);
        }
        app.update();

        let mut counter = 0_u8;
        let mut offset = 0_u8;
        let mut amplitude = 0_u8;
        for _ in 0..=outro {
            amplitude = counter;
            let old_offset = offset;
            counter = counter.wrapping_add(old_offset);
            offset = offset.wrapping_add(1);
        }

        let world = app.world_mut();
        let mut strips = world.query_filtered::<(&Sprite, &Transform), With<BattleCommandMarker>>();
        let base_strips = strips
            .iter(world)
            .filter(|(sprite, transform)| {
                sprite.rect.is_some() && (transform.translation.z - 0.0).abs() < f32::EPSILON
            })
            .collect::<Vec<_>>();
        let mut source_rows = vec![Vec::<f32>::new(); 144];
        for (sprite, transform) in base_strips {
            assert_eq!(
                sprite.custom_size,
                Some(Vec2::new(PLAYFIELD_WIDTH, TILE_SIZE / 8.0)),
                "each native scanline must display as one 640x4 strip"
            );
            let rect = sprite.rect.expect("wave strip source rectangle");
            let source_y = rect.min.y as u8;
            assert_eq!(
                rect.max.y - rect.min.y,
                1.0,
                "wave strips must sample exactly one native scanline"
            );
            source_rows[usize::from(source_y)].push(transform.translation.x);
            let expected_y =
                PLAYFIELD_HEIGHT * 0.5 - (f32::from(source_y) + 0.5) * (TILE_SIZE / 8.0);
            assert_eq!(
                transform.translation.y, expected_y,
                "wave row {source_y} must stay centered on its native scanline"
            );
        }
        for (source_y, actual_positions) in source_rows.iter_mut().enumerate() {
            let expected_shift =
                visible_battle_anim_sine((source_y as u8).wrapping_mul(2), amplitude) as f32
                    * (TILE_SIZE / 8.0);
            let mut expected_positions = vec![expected_shift];
            if expected_shift > 0.0 {
                expected_positions.push(expected_shift - PLAYFIELD_WIDTH);
            } else if expected_shift < 0.0 {
                expected_positions.push(expected_shift + PLAYFIELD_WIDTH);
            }
            actual_positions.sort_by(f32::total_cmp);
            expected_positions.sort_by(f32::total_cmp);
            assert_eq!(
                *actual_positions, expected_positions,
                "outro {outro} row {source_y} must wrap its shifted SCX scanline"
            );
        }

        let mut priority_strips =
            world.query_filtered::<(&Sprite, &Transform), With<BattleCommandMarker>>();
        let priority_count = priority_strips
            .iter(world)
            .filter(|(sprite, transform)| {
                sprite.rect.is_some() && (transform.translation.z - 2.4).abs() < f32::EPSILON
            })
            .count();
        assert_eq!(
            priority_count,
            source_rows.iter().map(Vec::len).sum::<usize>(),
            "priority scanlines must wrap with the base layer"
        );
    }
}

#[test]
fn battle_transition_outros_render_their_first_and_final_source_mutations() {
    const PREFIX_FRAMES: u16 = 3;
    const FLASH_FRAMES: u16 = 75;

    // Strong cave zoom: the nine source WaitBGMap calls are the nine
    // visible boxes, from 4x2 through the complete 20x18 LCD.
    let mut runtime_shell = route36_battle_shell_for_render_regression();
    runtime_shell.visible_battle_transition = Some(VisibleBattleTransition {
        frame: PREFIX_FRAMES + FLASH_FRAMES + 1,
        stronger_enemy: true,
        cave_environment: true,
        trainer_battle: false,
    });
    mark_runtime_snapshot_dirty(&mut runtime_shell);
    let mut zoom_app = battle_render_regression_app(runtime_shell);
    zoom_app.update();
    for (outro, expected_size) in [
        (0_u16, Vec2::new(TILE_SIZE * 4.0, TILE_SIZE * 2.0)),
        (8_u16, Vec2::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT)),
    ] {
        if outro != 0 {
            let mut runtime_shell = zoom_app.world_mut().resource_mut::<BevyRuntimeShell>();
            runtime_shell.visible_battle_transition = Some(VisibleBattleTransition {
                frame: PREFIX_FRAMES + FLASH_FRAMES + 1 + outro,
                stronger_enemy: true,
                cave_environment: true,
                trainer_battle: false,
            });
            mark_runtime_snapshot_dirty(&mut runtime_shell);
            drop(runtime_shell);
            zoom_app.update();
        }
        let world = zoom_app.world_mut();
        let mut black = world.query_filtered::<(&Sprite, &Transform), With<BattleCommandMarker>>();
        let surfaces = black
            .iter(world)
            .filter(|(sprite, transform)| {
                sprite.color == Color::BLACK && (transform.translation.z - 2.7).abs() < f32::EPSILON
            })
            .filter_map(|(sprite, transform)| {
                sprite.custom_size.map(|size| (size, transform.translation))
            })
            .collect::<Vec<_>>();
        assert_eq!(surfaces.len(), 1, "zoom outro {outro}");
        assert_eq!(surfaces[0].0, expected_size, "zoom outro {outro}");
        assert_eq!(
            surfaces[0].1,
            Vec3::new(0.0, 0.0, 2.7),
            "zoom boxes must expand about the exact LCD centre"
        );
    }

    // Strong outdoor scatter writes exactly twelve fresh cells per source
    // call. The first and sixteenth calls must therefore expose 12 and 192
    // black tiles respectively before the terminal hold.
    let mut runtime_shell = route36_battle_shell_for_render_regression();
    runtime_shell.visible_battle_transition = Some(VisibleBattleTransition {
        frame: PREFIX_FRAMES + FLASH_FRAMES + 2,
        stronger_enemy: true,
        cave_environment: false,
        trainer_battle: false,
    });
    mark_runtime_snapshot_dirty(&mut runtime_shell);
    let mut scatter_app = battle_render_regression_app(runtime_shell);
    scatter_app.update();
    for (outro, expected_tiles) in [(0_u16, 12_usize), (15_u16, 192_usize)] {
        if outro != 0 {
            let mut runtime_shell = scatter_app.world_mut().resource_mut::<BevyRuntimeShell>();
            runtime_shell.visible_battle_transition = Some(VisibleBattleTransition {
                frame: PREFIX_FRAMES + FLASH_FRAMES + 2 + outro,
                stronger_enemy: true,
                cave_environment: false,
                trainer_battle: false,
            });
            mark_runtime_snapshot_dirty(&mut runtime_shell);
            drop(runtime_shell);
            scatter_app.update();
        }
        let world = scatter_app.world_mut();
        let mut black = world.query_filtered::<(&Sprite, &Transform), With<BattleCommandMarker>>();
        let count = black
            .iter(world)
            .filter(|(sprite, transform)| {
                sprite.color == Color::BLACK
                    && sprite.custom_size == Some(Vec2::splat(TILE_SIZE))
                    && (transform.translation.z - 2.7).abs() < f32::EPSILON
            })
            .count();
        assert_eq!(count, expected_tiles, "scatter outro {outro}");
    }

    // Ordinary outdoor spin writes one wedge every three displayed frames.
    // Its five wedge shapes repeat over four quadrants and cover all 360
    // LCD cells after the twentieth write.
    let mut runtime_shell = route36_battle_shell_for_render_regression();
    runtime_shell.visible_battle_transition = Some(VisibleBattleTransition {
        frame: PREFIX_FRAMES + FLASH_FRAMES + 2,
        stronger_enemy: false,
        cave_environment: false,
        trainer_battle: false,
    });
    mark_runtime_snapshot_dirty(&mut runtime_shell);
    let mut spin_app = battle_render_regression_app(runtime_shell);
    spin_app.update();
    for (outro, expected_tiles) in [(0_u16, 16_usize), (57_u16, 360_usize)] {
        if outro != 0 {
            let mut runtime_shell = spin_app.world_mut().resource_mut::<BevyRuntimeShell>();
            runtime_shell.visible_battle_transition = Some(VisibleBattleTransition {
                frame: PREFIX_FRAMES + FLASH_FRAMES + 2 + outro,
                stronger_enemy: false,
                cave_environment: false,
                trainer_battle: false,
            });
            mark_runtime_snapshot_dirty(&mut runtime_shell);
            drop(runtime_shell);
            spin_app.update();
        }
        let world = spin_app.world_mut();
        let mut black = world.query_filtered::<(&Sprite, &Transform), With<BattleCommandMarker>>();
        let count = black
            .iter(world)
            .filter(|(sprite, transform)| {
                sprite.color == Color::BLACK
                    && sprite.custom_size == Some(Vec2::splat(TILE_SIZE))
                    && (transform.translation.z - 2.7).abs() < f32::EPSILON
            })
            .count();
        assert_eq!(count, expected_tiles, "spin outro {outro}");
        if outro == 57 {
            let mut tiles =
                world.query_filtered::<(&Sprite, &Transform), With<BattleCommandMarker>>();
            let centers = tiles
                .iter(world)
                .filter(|(sprite, transform)| {
                    sprite.color == Color::BLACK
                        && sprite.custom_size == Some(Vec2::splat(TILE_SIZE))
                        && (transform.translation.z - 2.7).abs() < f32::EPSILON
                })
                .map(|(_, transform)| transform.translation.truncate())
                .collect::<Vec<_>>();
            assert_eq!(
                centers
                    .iter()
                    .map(|center| center.x)
                    .fold(f32::INFINITY, f32::min),
                PLAYFIELD_LEFT + TILE_SIZE * 0.5,
            );
            assert_eq!(
                centers
                    .iter()
                    .map(|center| center.x)
                    .fold(f32::NEG_INFINITY, f32::max),
                -PLAYFIELD_LEFT - TILE_SIZE * 0.5,
            );
            assert_eq!(
                centers
                    .iter()
                    .map(|center| center.y)
                    .fold(f32::INFINITY, f32::min),
                -PLAYFIELD_TOP + TILE_SIZE * 0.5,
            );
            assert_eq!(
                centers
                    .iter()
                    .map(|center| center.y)
                    .fold(f32::NEG_INFINITY, f32::max),
                PLAYFIELD_TOP - TILE_SIZE * 0.5,
            );
        }
    }
}

#[test]
fn every_battle_transition_variant_finishes_on_full_black_before_battle_canvas() {
    for (cave_environment, stronger_enemy, outro_frames, finish_frames) in [
        (true, false, 15_u16, 1_u16),
        (true, true, 9, 2),
        (false, false, 61, 4),
        (false, true, 21, 1),
    ] {
        let prefix_frames = 3_u16;
        let between_frames = if cave_environment && stronger_enemy {
            1
        } else {
            2
        };
        let total_frames = prefix_frames + 75 + between_frames + outro_frames + finish_frames;
        let mut runtime_shell = route36_battle_shell_for_render_regression();
        runtime_shell.visible_battle_transition = Some(VisibleBattleTransition {
            frame: total_frames - 1,
            stronger_enemy,
            cave_environment,
            trainer_battle: false,
        });
        let mut app = battle_render_regression_app(runtime_shell);
        app.update();

        let world = app.world_mut();
        let mut transition_surfaces =
            world.query_filtered::<(&Sprite, &Transform), With<BattleCommandMarker>>();
        let surfaces = transition_surfaces
            .iter(world)
            .map(|(sprite, transform)| (sprite.custom_size, sprite.color, transform.translation.z))
            .collect::<Vec<_>>();
        let terminal_black = surfaces
            .iter()
            .filter(|(size, color, z)| {
                *size == Some(Vec2::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT))
                    && *color == Color::BLACK
                    && (*z - 2.76).abs() < f32::EPSILON
            })
            .count();
        assert_eq!(
            terminal_black, 1,
            "variant cave={cave_environment} stronger={stronger_enemy} must hold one opaque full-LCD black finish frame"
        );
        assert_eq!(
            surfaces
                .iter()
                .map(|(_, _, z)| *z)
                .fold(f32::NEG_INFINITY, f32::max),
            2.76,
            "terminal black must be the top transition surface"
        );
        let mut fixed_canvas = world.query_filtered::<Entity, With<FixedBattleCanvasMarker>>();
        assert_eq!(fixed_canvas.iter(world).count(), 0);
        let _ = world;

        {
            let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            advance_visible_battle_transition(&mut runtime_shell);
            assert!(runtime_shell.visible_battle_transition.is_none());
        }
        app.update();
        let world = app.world_mut();
        let mut fixed_canvas =
            world.query_filtered::<(&Sprite, &Transform), With<FixedBattleCanvasMarker>>();
        let canvases = fixed_canvas.iter(world).collect::<Vec<_>>();
        assert_eq!(
            canvases.len(),
            1,
            "variant cave={cave_environment} stronger={stronger_enemy} must hand off directly to one fixed battle canvas"
        );
        assert_eq!(
            canvases[0].0.custom_size,
            Some(Vec2::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT))
        );
        assert_eq!(canvases[0].0.color, Color::WHITE);
        assert_eq!(canvases[0].1.translation, Vec3::new(0.0, 0.0, 2.7));
    }
}

#[test]
fn battle_redraw_retains_fixed_canvas_without_image_growth_even_if_overlay_rebuild_fails() {
    let mut runtime_shell = route36_battle_shell_for_render_regression();
    runtime_shell.visible_battle_transition = None;
    runtime_shell.battle_entry_messages_remaining = 0;
    runtime_shell.battle_enemy_send_out_pending = false;
    runtime_shell.battle_player_send_out_pending = false;
    runtime_shell.battle_messages.clear();
    runtime_shell.battle_text_reveal = None;
    sync_visible_battle_action_cursor(&mut runtime_shell);

    let mut app = battle_render_regression_app(runtime_shell);
    app.update();

    let first_canvas = {
        let world = app.world_mut();
        let mut canvases = world.query_filtered::<Entity, With<FixedBattleCanvasMarker>>();
        let canvases = canvases.iter(world).collect::<Vec<_>>();
        assert_eq!(
            canvases.len(),
            1,
            "battle must stage exactly one opaque canvas"
        );
        canvases[0]
    };
    let stable_image_count = app.world().resource::<Assets<Image>>().len();

    // Exercise both eight-frame party-icon phases and the sixteen-frame
    // cursor phase. These redraw the transient battle layers but must not
    // allocate replacement textures or replace the continuity canvas.
    for _ in 0..4 {
        {
            let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            runtime_shell.lcd_animation_frame = runtime_shell.lcd_animation_frame.wrapping_add(8);
            mark_runtime_snapshot_dirty(&mut runtime_shell);
        }
        app.update();
        let world = app.world_mut();
        let mut canvases = world.query_filtered::<Entity, With<FixedBattleCanvasMarker>>();
        assert_eq!(canvases.iter(world).collect::<Vec<_>>(), vec![first_canvas]);
        assert_eq!(
            world.resource::<Assets<Image>>().len(),
            stable_image_count,
            "steady battle redraws must reuse cached image assets"
        );
    }

    // Fail a stage that runs after battlers and HUD have begun rebuilding.
    // The prior full-LCD canvas must survive the early return and continue
    // hiding the retained overworld rather than exposing it for a frame.
    {
        let mut runtime_shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        runtime_shell.battle_action_cursor = Some(MenuCursor {
            surface_id: "invalid:battle-actions".to_string(),
            option_index: usize::MAX,
        });
        mark_runtime_snapshot_dirty(&mut runtime_shell);
    }
    app.update();

    let world = app.world_mut();
    assert!(
        world
            .resource::<BevyRuntimeShell>()
            .last_error
            .as_deref()
            .is_some_and(|error| error.contains("battle main-action cursor is invalid")),
        "fixture must reach the forced post-canvas render failure"
    );
    let mut canvases =
        world.query_filtered::<(Entity, &Sprite, &Transform), With<FixedBattleCanvasMarker>>();
    let canvases = canvases.iter(world).collect::<Vec<_>>();
    assert_eq!(canvases.len(), 1);
    assert_eq!(canvases[0].0, first_canvas);
    assert_eq!(
        canvases[0].1.custom_size,
        Some(Vec2::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT))
    );
    assert_eq!(canvases[0].2.translation, Vec3::new(0.0, 0.0, 2.7));
    assert_eq!(
        world.resource::<Assets<Image>>().len(),
        stable_image_count,
        "the failed redraw must not leak image assets"
    );
}

#[test]
fn battle_trainer_preserves_authored_palette_colors() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let assets = AssetRoot::new(root);
    let mut images = Assets::<Image>::default();
    let frame = load_oak_intro_frame(&assets, "battle-trainer:bug_catcher", &mut images)
        .expect("trainer frame");
    let source =
        crate::open_runtime_image(&assets.runtime_assets().join("gfx/trainers/bug_catcher.png"))
            .unwrap()
            .to_rgba8();
    let rendered = &images.get(&frame.handle).unwrap().data;
    for (pixel, actual) in source.pixels().zip(rendered.chunks_exact(4)) {
        if pixel.0[0..3] != [255, 255, 255] {
            assert_eq!(
                &pixel.0, actual,
                "colored trainer pixels must retain their palette index"
            );
        }
    }
}

#[test]
fn battle_animation_shared_graphics_use_authored_tile_offsets() {
    let assets = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let mut art = RenderedTilesetArt::default();
    let bundle: serde_json::Value = serde_json::from_slice(
        &crate::read_runtime_asset(&assets.runtime_assets().join("data/battle_anim_bundle.json"))
            .unwrap(),
    )
    .unwrap();
    let mut images = Assets::<Image>::default();
    let object = &bundle["objects"]["BATTLE_ANIM_OBJ_STRING_SHOT"];
    for frameset in [
        "BATTLE_ANIM_FRAMESET_STRING_SHOT_1",
        "BATTLE_ANIM_FRAMESET_STRING_SHOT_2",
        "BATTLE_ANIM_FRAMESET_STRING_SHOT_3",
    ] {
        let (_, frame) = battle_anim_frame_at_age(&bundle, frameset, 0)
            .unwrap()
            .unwrap();
        battle_anim_rendered_frame(
            &mut art,
            &bundle,
            &assets,
            "BATTLE_ANIM_OBJ_STRING_SHOT",
            object,
            frameset,
            0,
            frame,
            false,
            false,
            false,
            None,
            0xe4,
            0xe4,
            None,
            &mut images,
        )
        .expect("all String Shot variants address the same web graphics sheet");
    }
}

fn battle_anim_regression_bundle() -> serde_json::Value {
    let assets = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    serde_json::from_slice(
        &crate::read_runtime_asset(&assets.runtime_assets().join("data/battle_anim_bundle.json"))
            .unwrap(),
    )
    .unwrap()
}

fn battle_anim_regression_timeline(
    events: Vec<VisibleMoveObjectEvent>,
    frame: u16,
) -> VisibleMoveAnimation {
    VisibleMoveAnimation {
        trigger_message: String::new(),
        move_id: "TEST".into(),
        animation_label: "BattleAnim_Test".into(),
        player_move: true,
        started: true,
        waiting_for_hp: false,
        frame,
        total_frames: 256,
        sound_events: Vec::new(),
        next_sound_event: 0,
        cry_events: Vec::new(),
        next_cry_event: 0,
        object_events: events,
        bg_events: Vec::new(),
        actor_species_override: None,
        actor_shiny_override: None,
    }
}

fn battle_anim_regression_spawn(frame: u16, param: u8) -> VisibleMoveObjectEvent {
    VisibleMoveObjectEvent {
        frame,
        command: VisibleMoveObjectCommand::Spawn {
            object_id: "BATTLE_ANIM_OBJ_STRING_SHOT".into(),
            x: 64,
            y: 80,
            param,
        },
    }
}

#[test]
fn battle_anim_frames_include_the_duration_reload_tick() {
    let bundle = battle_anim_regression_bundle();
    let frameset = "BATTLE_ANIM_FRAMESET_STRING_SHOT_1";
    for age in 0..12 {
        assert_eq!(
            battle_anim_frame_at_age(&bundle, frameset, age)
                .unwrap()
                .map(|(i, _)| i),
            Some(usize::from(age / 3)),
            "age {age}"
        );
    }
    assert!(
        battle_anim_frame_at_age(&bundle, frameset, 12)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        visible_battle_animation_frameset_lifetime(&bundle, frameset),
        Some(12)
    );
}

#[test]
fn battle_anim_oamwait_hides_objects_for_its_full_duration() {
    let bundle = serde_json::json!({"framesets": {"BLINK": [
        {"command": "frame", "duration": 2},
        {"command": "wait", "duration": 2},
        {"command": "frame", "duration": 0},
        {"command": "restart"}
    ]}});
    for age in 3..6 {
        assert!(
            battle_anim_frame_at_age(&bundle, "BLINK", age)
                .unwrap()
                .is_none(),
            "blink age {age}"
        );
    }
    assert_eq!(
        battle_anim_frame_at_age(&bundle, "BLINK", 6)
            .unwrap()
            .unwrap()
            .0,
        2
    );
    assert_eq!(
        battle_anim_frame_at_age(&bundle, "BLINK", 7)
            .unwrap()
            .unwrap()
            .0,
        0
    );
}

#[test]
fn battle_anim_deleted_framesets_release_non_null_object_slots() {
    let bundle = battle_anim_regression_bundle();
    let mut events = vec![battle_anim_regression_spawn(0, 0); 10];
    events.push(battle_anim_regression_spawn(13, 1));
    let animation = battle_anim_regression_timeline(events, 13);
    let slots = visible_battle_objects(&bundle, &animation).map(|playback| playback.slots).unwrap();
    assert_eq!(slots.iter().flatten().count(), 6);
    // Five 8-piece String Shot objects fill OAM, delaying later callbacks.
    assert!(slots[5..].iter().all(Option::is_some));
    assert_eq!(
        slots[0].as_ref().unwrap().spawn_frame,
        13,
        "the live web must reuse the expired projectile slot"
    );
}

#[test]
fn battle_anim_object_commands_address_creation_ids_after_slot_reuse() {
    let bundle = battle_anim_regression_bundle();
    for command in [
        VisibleMoveObjectCommand::Increment { index: 3 },
        VisibleMoveObjectCommand::Set { index: 3, value: 1 },
    ] {
        let animation = battle_anim_regression_timeline(
            vec![
                battle_anim_regression_spawn(0, 0),
                battle_anim_regression_spawn(0, 1),
                VisibleMoveObjectEvent {
                    frame: 13,
                    command: VisibleMoveObjectCommand::Spawn {
                        object_id: "BATTLE_ANIM_OBJ_HIT".into(),
                        x: 64,
                        y: 80,
                        param: 0,
                    },
                },
                VisibleMoveObjectEvent { frame: 14, command },
            ],
            14,
        );
        let slots = visible_battle_objects(&bundle, &animation).map(|playback| playback.slots).unwrap();
        assert!(
            slots[0].as_ref().is_some_and(|live| live.bytes[0] == 0),
            "the callback deletes object 3 before its final OAM update"
        );
        assert_eq!(slots[1].as_ref().unwrap().bytes[0], 2, "object 2 must remain untouched");
    }
}

#[test]
fn battle_anim_null_function_deletes_when_incremented() {
    assert_eq!(
        battle_object_position_fixture("BATTLE_ANIM_FUNC_NULL", 64, 80, 0, 2, 1, 0, true),
        None
    );
}

#[test]
fn battle_anim_clearobjs_matches_the_cartridge_partial_clear() {
    let bundle = battle_anim_regression_bundle();
    let mut events = vec![battle_anim_regression_spawn(0, 1); 10];
    events.push(VisibleMoveObjectEvent {
        frame: 1,
        command: VisibleMoveObjectCommand::Clear,
    });
    let animation = battle_anim_regression_timeline(events, 1);
    let slots = visible_battle_objects(&bundle, &animation).map(|playback| playback.slots).unwrap();
    assert!(slots[..7].iter().all(Option::is_none));
    assert!(slots[7..].iter().all(Option::is_some));
}

#[test]
fn battle_anim_commands_run_before_same_tick_object_deletion() {
    let bundle = battle_anim_regression_bundle();
    let mut events = vec![battle_anim_regression_spawn(0, 0); 10];
    events.push(battle_anim_regression_spawn(12, 1));
    events.push(battle_anim_regression_spawn(13, 2));
    let animation = battle_anim_regression_timeline(events, 13);
    let slots = visible_battle_objects(&bundle, &animation).map(|playback| playback.slots).unwrap();
    assert_eq!(slots.iter().flatten().count(), 6);
    // Five 8-piece String Shot objects fill OAM, delaying later callbacks.
    assert!(slots[5..].iter().all(Option::is_some));
    assert_eq!(
        slots[0].as_ref().unwrap().spawn_frame,
        13,
        "frame 12 spawn must see the still-occupied structs"
    );
}

#[test]
fn battle_anim_projectiles_stop_or_delete_on_the_source_boundary() {
    let position = |function, param, age, state| {
        battle_object_position_fixture(function, 128, 56, param, age, state, 0, true)
    };
    assert_eq!(
        position("BATTLE_ANIM_FUNC_USER_TO_TARGET", 4, 0, 0),
        Some((132, 54))
    );
    assert_eq!(
        position("BATTLE_ANIM_FUNC_USER_TO_TARGET", 4, 1, 0),
        Some((132, 54))
    );
    assert_eq!(position("BATTLE_ANIM_FUNC_USER_TO_TARGET", 4, 2, 1), None);
    assert_eq!(
        position("BATTLE_ANIM_FUNC_USER_TO_TARGET_DISAPPEAR", 4, 0, 0),
        Some((132, 54))
    );
    assert_eq!(
        position("BATTLE_ANIM_FUNC_USER_TO_TARGET_DISAPPEAR", 4, 1, 0),
        None
    );
    assert_eq!(
        position("BATTLE_ANIM_FUNC_USER_TO_TARGET", 0x24, 0, 0),
        Some((132, 54))
    );
    assert_eq!(
        position("BATTLE_ANIM_FUNC_USER_TO_TARGET", 0, 10, 0),
        Some((128, 56))
    );
    assert_eq!(
        position("BATTLE_ANIM_FUNC_THROW_TO_TARGET_DISAPPEAR", 8, 0, 0),
        Some((130, 55))
    );
    assert!(position("BATTLE_ANIM_FUNC_THROW_TO_TARGET_DISAPPEAR", 8, 3, 0).is_some());
    assert!(position("BATTLE_ANIM_FUNC_THROW_TO_TARGET_DISAPPEAR", 8, 4, 0).is_none());
}

#[test]
fn battle_anim_enemy_mirrors_the_current_x_coordinate_and_offset() {
    let mut animation = battle_anim_regression_timeline(Vec::new(), 4);
    animation.player_move = false;
    assert_eq!(
        battle_object_screen_fixture(
            &animation,
            "BATTLE_ANIM_FUNC_MOVE_IN_CIRCLE",
            64,
            80,
            4,
            72,
            83,
            1,
            0x90
        ),
        (108, 67)
    );
    assert_eq!(
        battle_object_screen_fixture(
            &animation,
            "BATTLE_ANIM_FUNC_MOVE_IN_CIRCLE",
            64,
            80,
            4,
            72,
            83,
            0,
            0x90
        ),
        (72, 83)
    );
    animation.animation_label = "BattleAnim_Recover".into();
    assert_eq!(
        battle_object_screen_fixture(
            &animation,
            "BATTLE_ANIM_FUNC_RECOVER",
            64,
            80,
            4,
            72,
            83,
            1,
            0x90
        )
        .1,
        67
    );
}

#[test]
fn battle_anim_string_yflip_is_only_applied_on_the_enemy_side() {
    let bundle = battle_anim_regression_bundle();
    let assets = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let object = &bundle["objects"]["BATTLE_ANIM_OBJ_STRING_SHOT"];
    let frameset = "BATTLE_ANIM_FRAMESET_STRING_SHOT_1";
    let (i, frame) = battle_anim_frame_at_age(&bundle, frameset, 0)
        .unwrap()
        .unwrap();
    let mut art = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    let mut render = |enemy, extra| {
        let rendered = battle_anim_rendered_frame(
            &mut art,
            &bundle,
            &assets,
            "BATTLE_ANIM_OBJ_STRING_SHOT",
            object,
            frameset,
            i,
            frame,
            enemy,
            extra,
            false,
            None,
            0xe4,
            0xe4,
            None,
            &mut images,
        )
        .unwrap();
        (
            rendered.offset_y,
            images.get(&rendered.sprite.handle).unwrap().data.clone(),
        )
    };
    assert_eq!(render(false, false), render(false, true));
    assert_ne!(render(true, false), render(true, true));
}

#[test]
fn battle_anim_overlapping_oam_entries_keep_the_first_opaque_pixel() {
    let mut bundle = battle_anim_regression_bundle();
    let assets = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let object = bundle["objects"]["BATTLE_ANIM_OBJ_HIT"].clone();
    let frameset = object["frameset"].as_str().unwrap();
    let (i, frame) = battle_anim_frame_at_age(&bundle, frameset, 0)
        .unwrap()
        .unwrap();
    let frame = frame.clone();
    let oam = frame["oam_set"].as_str().unwrap().to_string();
    let first = bundle["oam_sets"][&oam]["entries"][0].clone();
    let mut second = first.clone();
    second["xflip"] = serde_json::json!(!first["xflip"].as_bool().unwrap());
    second["yflip"] = serde_json::json!(!first["yflip"].as_bool().unwrap());
    let render = |bundle: &serde_json::Value| {
        let mut art = RenderedTilesetArt::default();
        let mut images = Assets::<Image>::default();
        let result = battle_anim_rendered_frame(
            &mut art,
            bundle,
            &assets,
            "BATTLE_ANIM_OBJ_HIT",
            &object,
            frameset,
            i,
            &frame,
            false,
            false,
            false,
            None,
            0xe4,
            0xe4,
            None,
            &mut images,
        )
        .unwrap();
        images.get(&result.sprite.handle).unwrap().data.clone()
    };
    bundle["oam_sets"][&oam]["entries"] = serde_json::json!([first]);
    let expected = render(&bundle);
    bundle["oam_sets"][&oam]["entries"] = serde_json::json!([first, second]);
    let actual = render(&bundle);
    assert!(expected.chunks_exact(4).any(|pixel| pixel[3] != 0));
    for (expected, actual) in expected.chunks_exact(4).zip(actual.chunks_exact(4)) {
        if expected[3] != 0 {
            assert_eq!(actual, expected);
        }
    }
}

#[test]
fn battle_anim_return_ends_the_script_without_waiting_for_live_effects() {
    let shell = route36_battle_shell_for_render_regression();
    let mut snapshot = shell.shell.snapshot().unwrap();
    let label = "BattleAnim_ReturnRegression".to_string();
    std::sync::Arc::make_mut(&mut snapshot.presentation)
        .battle_animations
        .insert(
            label.clone(),
            vec![
                "anim_obj BATTLE_ANIM_OBJ_HIT, 64, 80, 0".into(),
                "anim_bgeffect BATTLE_BG_EFFECT_SHAKE_SCREEN_X, 60, 2, 0".into(),
                "anim_wait 1".into(),
                "anim_ret".into(),
            ],
        );
    let (_, frames, _, _, _, _) = visible_battle_animation_definition(&snapshot, label, 0).unwrap();
    assert_eq!(
        frames, 2,
        "the root anim_ret stops OAM and BG effects on its own tick"
    );
}

#[test]
fn battle_anim_wave_projectile_uses_source_sine_phase_and_amplitude() {
    let position = |age| {
        battle_object_position_fixture(
            "BATTLE_ANIM_FUNC_WAVE_TO_TARGET",
            128,
            56,
            0xff,
            age,
            0,
            0,
            true,
        )
    };
    assert_eq!(position(0), Some((131, 55)));
    assert_eq!(position(1), Some((132, 60)));
    assert!(position(3).is_some());
    assert!(position(4).is_none());
}

#[test]
fn battle_anim_enemy_projectile_mirrors_base_y_but_preserves_wave_offset() {
    let mut animation = battle_anim_regression_timeline(Vec::new(), 1);
    animation.player_move = false;
    assert_eq!(
        battle_object_screen_fixture(
            &animation,
            "BATTLE_ANIM_FUNC_USER_TO_TARGET",
            128,
            56,
            0,
            132,
            54,
            1,
            0x90
        ),
        (48, 88)
    );
    assert_eq!(
        battle_object_screen_fixture(
            &animation,
            "BATTLE_ANIM_FUNC_WAVE_TO_TARGET",
            128,
            56,
            1,
            132,
            60,
            1,
            0x90
        ),
        (48, 96)
    );
}

#[test]
fn battle_anim_shake_uses_both_parameter_nybbles_without_invented_motion() {
    let position = |param, age, state| {
        battle_object_position_fixture(
            "BATTLE_ANIM_FUNC_SHAKE",
            64,
            80,
            param,
            age,
            state,
            0,
            true,
        )
    };
    assert_eq!(position(0, 0, 0), Some((64, 80)));
    assert_eq!(position(0, 40, 0), Some((64, 80)));
    for (age, x) in [61, 61, 61, 67, 67, 67, 61].into_iter().enumerate() {
        assert_eq!(position(0x23, age as u16, 0), Some((x, 80)));
    }
    assert_eq!(position(0x23, 8, 1), Some((61, 80)));
    assert_eq!(position(0x23, 8, 2), None);
}

#[test]
fn battle_anim_source_register_order_and_fixed_point_motion() {
    let position = |function, param, age| {
        battle_object_position_fixture(function, 64, 80, param, age, 0, age, true)
    };
    // functions.asm reads VAR1/VAR2 before incrementing the phase.
    assert_eq!(
        position("BATTLE_ANIM_FUNC_ANCIENT_POWER", 32, 0),
        Some((64, 80))
    );
    assert_eq!(position("BATTLE_ANIM_FUNC_COTTON", 0, 1), Some((88, 80)));
    assert_eq!(
        position("BATTLE_ANIM_FUNC_SPEED_LINE", 0, 0),
        Some((65, 80))
    );
    assert_eq!(
        position("BATTLE_ANIM_FUNC_SPEED_LINE", 128, 0),
        Some((63, 80))
    );
    // FloatUp adds $ffa0 to its 8.8 Y accumulator, and advances phase by two.
    assert_eq!(position("BATTLE_ANIM_FUNC_FLOAT_UP", 0, 1), Some((64, 79)));
    assert_eq!(position("BATTLE_ANIM_FUNC_FLOAT_UP", 0, 7), Some((67, 77)));
    // Absorb tests the old X, and the zero-count Y loop wraps 256 times.
    assert_eq!(position("BATTLE_ANIM_FUNC_ABSORB", 0, 0), Some((64, 80)));
    assert_eq!(position("BATTLE_ANIM_FUNC_ABSORB", 1, 0), Some((63, 80)));
    assert_eq!(position("BATTLE_ANIM_FUNC_ABSORB", 4, 4), Some((44, 90)));
    assert_eq!(position("BATTLE_ANIM_FUNC_ABSORB", 4, 5), None);
}

#[test]
fn battle_anim_every_move_script_compiles_and_allocates_objects() {
    let shell = route36_battle_shell_for_render_regression();
    let snapshot = shell.shell.snapshot().unwrap();
    let bundle = battle_anim_regression_bundle();
    assert_eq!(snapshot.presentation.move_names.len(), 251);
    for (index, name) in snapshot.presentation.move_names.iter().enumerate() {
        let label = &snapshot.presentation.battle_animation_table[index + 1];
        for param in 0..=4 {
            let (frames, _, _, events, _) =
                compile_visible_battle_animation_timeline(&snapshot, label, param)
                    .unwrap_or_else(|| panic!("{name}: {label} parameter {param} failed"));
            let mut animation = battle_anim_regression_timeline(events, 0);
            animation.total_frames = frames;
            for player_move in [true, false] {
                animation.player_move = player_move;
                animation.frame = frames.saturating_sub(1);
                visible_battle_objects(&bundle, &animation).unwrap_or_else(|error| {
                    panic!("{name} parameter {param} player={player_move}: {error}")
                });
            }
        }
    }
}

#[test]
fn battle_anim_rapid_spin_deletes_after_displaying_terminal_offset() {
    for (age, expected) in [(11, Some((64, 32))), (12, None), (63, None)] {
        assert_eq!(
            battle_object_position_fixture(
                "BATTLE_ANIM_FUNC_RAPID_SPIN",
                64,
                80,
                0,
                age,
                0,
                age,
                true
            ),
            expected
        );
    }
}

#[test]
fn battle_anim_absorb_circle_starts_with_zero_radius() {
    assert_eq!(
        battle_object_position_fixture(
            "BATTLE_ANIM_FUNC_ABSORB_CIRCLE",
            128,
            48,
            0,
            0,
            0,
            0,
            true
        ),
        Some((128, 48))
    );
    assert_eq!(
        battle_object_position_fixture(
            "BATTLE_ANIM_FUNC_ABSORB_CIRCLE",
            128,
            48,
            0,
            1,
            0,
            1,
            true
        ),
        Some((127, 48))
    );
}

#[test]
fn battle_anim_thunder_wave_is_stationary_until_commanded_to_delete() {
    for state in 0..=3 {
        assert_eq!(
            battle_object_position_fixture(
                "BATTLE_ANIM_FUNC_THUNDER_WAVE",
                112,
                56,
                0,
                20,
                state,
                3,
                true
            ),
            if state == 3 { None } else { Some((112, 56)) }
        );
    }
    assert_eq!(
        battle_object_frameset_fixture(
            "BATTLE_ANIM_FUNC_THUNDER_WAVE",
            "BATTLE_ANIM_FRAMESET_THUNDER_WAVE_DISABLE",
            0,
            20,
            112,
            1,
            3,
            56,
            true
        ),
        ("BATTLE_ANIM_FRAMESET_THUNDER_WAVE_EXTRA", 3)
    );
}

#[test]
fn battle_anim_conversion_retains_full_radius_and_terminal_tick() {
    let position = |age| {
        battle_object_position_fixture(
            "BATTLE_ANIM_FUNC_CONVERSION",
            64,
            80,
            0,
            age,
            0,
            age,
            true,
        )
    };
    assert_eq!(position(64), Some((128, 80)));
    assert_eq!(position(127), Some((64, 80)));
    assert_eq!(position(128), None);
}

#[test]
fn battle_anim_bonemerang_initialization_falls_through_to_motion() {
    assert_eq!(
        battle_object_position_fixture(
            "BATTLE_ANIM_FUNC_BONEMERANG",
            64,
            80,
            0,
            0,
            0,
            0,
            true
        ),
        Some((97, 80))
    );
}

#[test]
fn battle_anim_speed_line_selects_parameter_frameset() {
    for param in 0..=2 {
        let expected = [
            "BATTLE_ANIM_FRAMESET_SPEED_LINE_1",
            "BATTLE_ANIM_FRAMESET_SPEED_LINE_2",
            "BATTLE_ANIM_FRAMESET_SPEED_LINE_3",
        ];
        for direction in [0, 128] {
            assert_eq!(
                battle_object_frameset_fixture(
                    "BATTLE_ANIM_FUNC_SPEED_LINE",
                    "BATTLE_ANIM_FRAMESET_SPEED_LINE_1",
                    param | direction,
                    8,
                    64,
                    0,
                    8,
                    80,
                    true
                ),
                (expected[param as usize], 8)
            );
        }
    }
}

#[test]
fn battle_anim_motion_matches_cartridge_routine_traces() {
    let corpus: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/battle_object_motion.json")).unwrap();
    assert_eq!(
        corpus["rom_sha1"],
        "f4cd194bdee0d04ca4eac29e09b8e4e9d818c133"
    );
    let samples = corpus["samples"].as_array().unwrap();
    assert_eq!(samples.len(), 156);
    assert_eq!(corpus["frames_per_sample"], 161);
    let mut differences = Vec::new();
    for sample in samples {
        assert_eq!(sample["positions"].as_array().unwrap().len(), 161);
        let function = sample["function"].as_str().unwrap();
        let param = sample["param"].as_u64().unwrap() as u8;
        for (age, expected) in sample["positions"].as_array().unwrap().iter().enumerate() {
            let expected = expected.as_array().map(|xy| {
                (
                    xy[0].as_i64().unwrap() as i32,
                    xy[1].as_i64().unwrap() as i32,
                )
            });
            let actual = battle_object_position_fixture(
                function,
                sample["x"].as_i64().unwrap() as i32,
                sample["y"].as_i64().unwrap() as i32,
                param,
                age as u16,
                0,
                age as u16,
                true,
            );
            if actual != expected {
                differences.push(format!(
                    "{function} param={param} age={age}: Rust {actual:?}, ROM {expected:?}"
                ));
                break;
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn battle_anim_dizzy_uses_old_phase_for_frameset_resets() {
    for (age, expected, frameset_age) in [
        (31, "BATTLE_ANIM_FRAMESET_IMP", 31),
        (32, "BATTLE_ANIM_FRAMESET_IMP_FLIPPED", 0),
        (63, "BATTLE_ANIM_FRAMESET_IMP_FLIPPED", 31),
        (64, "BATTLE_ANIM_FRAMESET_IMP", 0),
    ] {
        assert_eq!(
            battle_object_frameset_fixture(
                "BATTLE_ANIM_FUNC_DIZZY",
                "BATTLE_ANIM_FRAMESET_IMP",
                0,
                age,
                64,
                0,
                age,
                80,
                true
            ),
            (expected, frameset_age)
        );
    }
}

#[test]
fn battle_anim_agility_increment_deletes_and_heal_bell_preserves_y_parity() {
    assert_eq!(
        battle_object_position_fixture("BATTLE_ANIM_FUNC_AGILITY", 64, 80, 2, 20, 1, 0, true),
        None
    );
    assert_eq!(
        battle_object_position_fixture(
            "BATTLE_ANIM_FUNC_HEAL_BELL_NOTES",
            64,
            81,
            0,
            0,
            0,
            0,
            true
        ),
        Some((88, 82))
    );
}

// Fixture calls execute the same instruction runner as live playback. They
// deliberately omit OAM stepping to isolate callback register transitions.
fn battle_object_fixture(
    function: &str,
    base: &str,
    x: i32,
    y: i32,
    param: u8,
    age: u16,
    state: u8,
    state_age: u16,
    player: bool,
) -> (BattleObjectMachine, u16) {
    let mut machine = BattleObjectMachine::new(player);
    let function = battle_program::FUNCTIONS
        .iter()
        .position(|name| *name == function)
        .unwrap() as u8;
    let frameset = battle_program::FRAMESETS
        .iter()
        .position(|name| *name == base)
        .unwrap() as u8;
    machine.initialize(
        0,
        1,
        [0, 0, frameset, function, 0, 0],
        x as u8,
        y as u8,
        param,
    );
    let mut reset = 0;
    for tick in 0..=age {
        if state != 0 && tick == age.saturating_sub(state_age) {
            machine.object_mut(0)[14] = state;
        }
        if machine.object(0)[0] != 0 {
            machine.step_object(0).unwrap();
            if machine.frameset_reset {
                reset = tick;
            }
        }
    }
    (machine, age - reset)
}

fn battle_object_position_fixture(
    function: &str,
    x: i32,
    y: i32,
    param: u8,
    age: u16,
    state: u8,
    state_age: u16,
    player: bool,
) -> Option<(i32, i32)> {
    let (machine, _) = battle_object_fixture(
        function,
        "BATTLE_ANIM_FRAMESET_HIT_BIG",
        x,
        y,
        param,
        age,
        state,
        state_age,
        player,
    );
    let object = machine.object(0);
    (object[0] != 0).then_some((
        i32::from(object[7].wrapping_add(object[9])),
        i32::from(object[8].wrapping_add(object[10])),
    ))
}

fn battle_object_frameset_fixture(
    function: &str,
    base: &str,
    param: u8,
    age: u16,
    x: i32,
    state: u8,
    state_age: u16,
    y: i32,
    player: bool,
) -> (&'static str, u16) {
    let (machine, age) =
        battle_object_fixture(function, base, x, y, param, age, state, state_age, player);
    (
        battle_program::FRAMESETS[usize::from(machine.object(0)[3])],
        age,
    )
}

fn battle_object_screen_fixture(
    animation: &VisibleMoveAnimation,
    function: &str,
    x: i32,
    y: i32,
    age: u16,
    animated_x: i32,
    animated_y: i32,
    flags: i64,
    fix_y: i64,
) -> (i32, i32) {
    let (mut machine, _) = battle_object_fixture(
        function,
        "BATTLE_ANIM_FRAMESET_HIT_BIG",
        x,
        y,
        2,
        age,
        0,
        age,
        animation.player_move,
    );
    let object = machine.object_mut(0);
    object[1] = flags as u8;
    object[2] = fix_y as u8;
    object[7] = animated_x as u8;
    object[9] = 0;
    object[10] = (animated_y as u8).wrapping_sub(object[8]);
    machine
        .call(
            battle_program::INIT_BATTLE_ANIM_BUFFER,
            battle_program::W_ACTIVE_ANIM_OBJECTS,
        )
        .unwrap();
    (
        i32::from(
            machine
                .read(battle_program::W_BATTLE_ANIM_TEMP_X_COORD)
                .wrapping_add(machine.read(battle_program::W_BATTLE_ANIM_TEMP_X_OFFSET)),
        ),
        i32::from(
            machine
                .read(battle_program::W_BATTLE_ANIM_TEMP_Y_COORD)
                .wrapping_add(machine.read(battle_program::W_BATTLE_ANIM_TEMP_Y_OFFSET)),
        ),
    )
}

#[test]
fn battle_anim_installed_pack_matches_all_cartridge_oam_cases() {
    let bundle = battle_anim_regression_bundle();
    for (case, &(function, object, x, y, param, player)) in
        crate::battle_anim_machine::oracle_oam::CASES
            .iter()
            .enumerate()
    {
        let mut machine = BattleObjectMachine::new(player);
        install_battle_object_data(&mut machine, &bundle).unwrap();
        machine.write(battle_program::W_CUR_ITEM, 5);
        let mut definition = [0, 128, 0, function, 0, 0];
        if object != 255 {
            for (i, v) in definition.iter_mut().enumerate().take(5) {
                *v = machine
                    .read(battle_program::BATTLE_ANIM_OBJECTS + u16::from(object) * 6 + i as u16);
            }
        }
        machine.initialize(0, 1, definition, x, y, param);
        for frame in 0..crate::battle_anim_machine::oracle_oam::FRAMES {
            machine.begin_oam();
            if machine.object(0)[0] != 0 {
                machine.step_object(0).unwrap();
                machine.oam_update(0).unwrap();
            }
            let mut bytes = machine.object(0)[..17].to_vec();
            bytes.extend(
                [
                    battle_program::W_O_B_P0,
                    battle_program::H_L_C_D_C_POINTER,
                    battle_program::H_L_Y_OVERRIDE_START,
                    battle_program::H_L_Y_OVERRIDE_END,
                ]
                .map(|a| machine.read(a)),
            );
            bytes.extend(machine.oam());
            bytes.push(machine.read(battle_program::W_BATTLE_ANIM_O_A_M_POINTER_LO));
            let hash = bytes.into_iter().fold(14695981039346656037_u64, |h, b| {
                (h ^ u64::from(b)).wrapping_mul(1099511628211)
            });
            let offset = (case * crate::battle_anim_machine::oracle_oam::FRAMES + frame) * 8;
            assert_eq!(
                hash.to_le_bytes(),
                crate::battle_anim_machine::oracle_oam::RECORDS[offset..offset + 8],
                "case {case} frame {frame}"
            );
        }
    }
}
#[test]
fn battle_anim_incremental_playback_matches_uninterrupted_command_history() {
    let bundle = battle_anim_regression_bundle();
    let events = vec![
        VisibleMoveObjectEvent {
            frame: 0,
            command: VisibleMoveObjectCommand::Spawn {
                object_id: "BATTLE_ANIM_OBJ_THUNDER_WAVE".into(),
                x: 112,
                y: 56,
                param: 0,
            },
        },
        battle_anim_regression_spawn(2, 0),
        VisibleMoveObjectEvent {
            frame: 4,
            command: VisibleMoveObjectCommand::Set { index: 1, value: 1 },
        },
        VisibleMoveObjectEvent {
            frame: 6,
            command: VisibleMoveObjectCommand::Increment { index: 1 },
        },
        VisibleMoveObjectEvent {
            frame: 8,
            command: VisibleMoveObjectCommand::Clear,
        },
        battle_anim_regression_spawn(9, 2),
    ];
    for player in [true, false] {
        let mut animation = battle_anim_regression_timeline(events.clone(), 24);
        animation.player_move = player;
        let expected = visible_battle_objects(&bundle, &animation).unwrap();
        animation.frame = 0;
        let mut actual = new_visible_battle_objects(&bundle, &animation).unwrap();
        for frame in [0, 0, 1, 4, 6, 8, 8, 9, 17, 24] {
            animation.frame = frame;
            advance_visible_battle_objects(&mut actual, &bundle, &animation).unwrap();
        }
        for slot in 0..10 {
            assert_eq!(
                actual.machine.object(slot),
                expected.machine.object(slot),
                "side {player} slot {slot}"
            );
        }
        assert_eq!(actual.machine.oam(), expected.machine.oam());
        assert_eq!(actual.next_event, events.len());
        assert_eq!(actual.next_tick, 25);
    }
}

#[test]
fn battle_anim_cgb_oam_palette_bits_override_object_definition_and_dmg_selector() {
    let bundle = battle_anim_regression_bundle();
    let assets = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let animation = battle_anim_regression_timeline(
        vec![VisibleMoveObjectEvent {
            frame: 0,
            command: VisibleMoveObjectCommand::Spawn {
                object_id: "BATTLE_ANIM_OBJ_HIT".into(),
                x: 64,
                y: 80,
                param: 0,
            },
        }],
        0,
    );
    let playback = visible_battle_objects(&bundle, &animation).unwrap();
    let live = playback.slots[0].as_ref().unwrap();
    let object = &bundle["objects"]["BATTLE_ANIM_OBJ_HIT"];
    let frame = &bundle["framesets"][live.frameset][live.frame];
    let render = |palette: u8, obp0, obp1| {
        let mut oam = live.oam.clone();
        for entry in &mut oam.entries {
            entry[3] = (entry[3] & 0xe0) | palette | 0x10;
        }
        let mut art = RenderedTilesetArt::default();
        let mut images = Assets::<Image>::default();
        let result = battle_anim_rendered_frame(
            &mut art,
            &bundle,
            &assets,
            "BATTLE_ANIM_OBJ_HIT",
            object,
            live.frameset,
            live.frame,
            frame,
            false,
            false,
            false,
            None,
            obp0,
            obp1,
            Some(&oam),
            &mut images,
        )
        .unwrap();
        images.get(&result.sprite.handle).unwrap().data.clone()
    };
    let red = render(2, 0xe4, 0xe4);
    assert!(red.chunks_exact(4).any(|pixel| pixel[3] != 0));
    assert_eq!(red, render(2, 0, 0));
    assert_ne!(red, render(0, 0xe4, 0xe4));
    assert_eq!(render(0, 0xe4, 0), render(0, 0xe4, 0xe4));
    assert_ne!(render(0, 0, 0xe4), render(0, 0xe4, 0xe4));
}

#[test]
fn wild_battle_appeared_text_keeps_player_backpic_visible() {
    let mut runtime_shell = route36_battle_shell_for_render_regression();
    runtime_shell.visible_battle_transition = None;
    assert_eq!(runtime_shell.battle_entry_messages_remaining, 2);
    let mut app = battle_render_regression_app(runtime_shell);
    app.update();
    let world = app.world_mut();
    assert!(world.resource::<BevyRuntimeShell>().last_error.is_none());
    let mut battlers = world.query_filtered::<&Transform, With<BattleBattlerMarker>>();
    assert_eq!(battlers.iter(world).count(), 2,
        "ASM InitBattleDisplay retains the player backpic during WildMonAppearedText");
}

#[test]
fn battle_transition_handoff_keeps_input_owned_for_sliding_intro() {
    for trainer_battle in [false, true] {
        let mut shell = route36_battle_shell_for_render_regression();
        let transition = shell.visible_battle_transition.as_mut().unwrap();
        transition.trainer_battle = trainer_battle;
        transition.frame = visible_battle_transition_total_frames(transition) - 1;
        advance_visible_battle_transition(&mut shell);
        assert!(shell.visible_battle_transition.is_none());
        assert!(visible_noninteractive_battle_animation_owns_input(&shell),
            "BattleIntroSlidingPics must own input before trainer/wild narration");
    }
}

#[test]
fn battle_frontpic_padding_matches_asm_for_every_picture_size() {
    let mut art = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    for width in [40usize, 48, 56] {
        let source = Image::new(
            Extent3d {
                width: width as u32,
                height: width as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            [17u8, 33, 65, 255].repeat(width * width),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        let frame = SpriteFrame {
            handle: images.add(source),
            size: Vec2::splat(width as f32),
        };
        let padded = battle_padded_frontpic(&mut art, &mut images, &frame).unwrap();
        assert_eq!(padded.size, Vec2::splat(56.0));
        let pixels = &images.get(&padded.handle).unwrap().data;
        let left = if width == 56 { 0 } else { 8 };
        for y in 0..56 {
            for x in 0..56 {
                let inside = x >= left && x < left + width && y >= 56 - width;
                let expected = if inside { [17, 33, 65, 255] } else { [0; 4] };
                assert_eq!(&pixels[(y * 56 + x) * 4..(y * 56 + x + 1) * 4], &expected);
            }
        }
        let image_count = images.len();
        let repeated = battle_padded_frontpic(&mut art, &mut images, &frame).unwrap();
        assert_eq!(padded.handle, repeated.handle);
        assert_eq!(
            images.len(),
            image_count,
            "redrawing must reuse the padded texture"
        );
    }
}

#[test]
fn battle_pokemon_positions_use_native_front_and_back_boxes() {
    let mut world = World::new();
    let mut queue = bevy::ecs::world::CommandQueue::default();
    let mut art = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    for (side, width, species) in [
        (PokemonSpriteSide::Front, 40, "hoothoot"),
        (PokemonSpriteSide::Back, 48, "totodile"),
    ] {
        let frame = SpriteFrame {
            handle: images.add(Image::new(
                Extent3d {
                    width,
                    height: width,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                [17u8, 33, 65, 255].repeat((width * width) as usize),
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            )),
            size: Vec2::splat(width as f32),
        };
        art.pokemon_cache.insert(
            PokemonArtKey {
                species_id: normalize_pokemon_asset_id(species),
                side,
                shiny: false,
                frame: 0,
            },
            frame,
        );
        spawn_battler_marker(
            &mut Commands::new(&mut queue, &world),
            &mut art,
            &AssetRoot::new(PathBuf::from("unused")),
            &mut images,
            species,
            side,
            22,
            22,
            false,
            false,
            0,
            Vec3::ZERO,
            1.0,
            None,
            VisibleBattlerArtOverride::Pokemon,
            None,
            None,
            None,
            true,
            None,
        )
        .unwrap();
    }
    queue.apply(&mut world);
    let mut query = world.query_filtered::<(&Transform, &Sprite), With<BattleBattlerMarker>>();
    let positions: Vec<_> = query
        .iter(&world)
        .map(|(transform, sprite)| {
            let size = sprite.custom_size.unwrap() / 4.0;
            let left = (transform.translation.x - PLAYFIELD_LEFT) / 4.0 - size.x / 2.0;
            let top = (PLAYFIELD_TOP - transform.translation.y) / 4.0 - size.y / 2.0;
            (left, top, size)
        })
        .collect();
    assert!(positions.contains(&(96.0, 0.0, Vec2::splat(56.0))));
    assert!(positions.contains(&(16.0, 48.0, Vec2::splat(48.0))));
}
#[test]
fn final_wild_attack_retains_move_and_faint_presentation_before_rewards() {
    let mut shell = route36_battle_shell_for_render_regression();
    shell.visible_battle_transition = None;
    shell.visible_battle_sliding_intro = None;
    shell.battle_entry_messages_remaining = 0;
    shell.battle_enemy_send_out_pending = false;
    shell.battle_player_send_out_pending = false;
    shell.battle_messages.clear();
    shell.battle_message_scenes.clear();
    shell.battle_text_reveal = None;
    shell.battle_hp_tween = None;
    {
        let state = shell.shell.session_mut().state_mut();
        let player = state.storage.party.pokemon[0].as_mut().unwrap();
        player.moves = vec![crate::core::models::LearnedMove {
            name: "SWIFT".into(), current_pp: 20, pp_ups: 0,
        }];
        state.sync_party_from_storage();
        let crate::core::state::BattleMemory::StaticWild {
            enemy_pokemon, enemy_party, ..
        } = &mut state.battle else { panic!("static wild fixture"); };
        enemy_pokemon.hp = 1;
        // A faster opponent must not KO itself with Struggle recoil before
        // the finishing player attack that this test is meant to exercise.
        enemy_pokemon.moves = vec![crate::core::models::LearnedMove {
            name: "SPLASH".into(), current_pp: 40, pp_ups: 0,
        }];
        enemy_party[0] = enemy_pokemon.clone();
        state.script_runtime.active_battle_combat = None;
    }
    mark_runtime_snapshot_dirty(&mut shell);
    shell.battle_message_scene = Some(Box::new(shell.shell.snapshot().unwrap()));
    resolve_visible_battle_move(&mut shell, 0).unwrap();
    assert!(shell.battle_messages.iter().any(|message| message.contains("fainted!")),
        "the actual turn must defeat the final opponent: {:?}", shell.battle_messages);
    assert!(shell.visible_move_animations.iter().any(|animation| animation.move_id == "SWIFT"),
        "settling final rewards must retain the finishing attack animation: animations={:?}, events={:?}",
        shell.visible_move_animations.iter().map(|animation| &animation.move_id).collect::<Vec<_>>(),
        shell.last_audio_events);
    assert!(shell.visible_move_animations.iter().any(|animation| animation.move_id == "FAINT_MON"),
        "settling final rewards must retain the authored faint animation");
    assert_eq!(shell.battle_message_scene.as_ref().unwrap().battle.as_ref().unwrap().enemy_pokemon.hp, 1,
        "the finishing move begins with the pre-damage battler visible");
    let faint = shell.visible_move_animations.iter().find(|animation| animation.move_id == "FAINT_MON").unwrap();
    assert_eq!(faint.sound_events, vec![(0, "SFX_KINESIS".to_string()), (14, "SFX_FAINT".to_string())],
        "FaintEnemyPokemon plays KINESIS before the tile drop and FAINT afterward");
    let mut app = menu_render_test_app(shell);
    app.update();
    save_live_battle_canvas_for_test(app.world_mut(), "final-hit-before.png");
    let mut saw_faint = false;
    let mut saw_finishing_move = false;
    let mut finished_faint = false;
    let mut captured_frames = std::collections::BTreeSet::new();
    for _ in 0..2000 {
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(shell.last_error.is_none(), "{:?}", shell.last_error);
        if shell.visible_move_animations.front().is_some_and(|animation|
            animation.started && animation.move_id == "SWIFT" && animation.frame == 0)
        {
            saw_finishing_move = true;
            save_live_battle_canvas_for_test(app.world_mut(), "final-move-swift.png");
            let canvas = render_live_battle_canvas_for_test(app.world_mut(), "finishing move HUD");
            assert_eq!(*canvas.get_pixel(400, 376), image::Rgba([255, 255, 255, 255]),
                "BattleAnimClearHud erases the attacking player's HUD before SWIFT");
        }
        let shell = app.world().resource::<BevyRuntimeShell>();
        let faint_frame = shell.visible_move_animations.front()
            .filter(|animation| animation.started && animation.move_id == "FAINT_MON")
            .map(|animation| animation.frame);
        let faint_pending = shell.visible_move_animations.iter()
            .any(|animation| animation.move_id == "FAINT_MON");
        let between_animations = !shell.visible_move_animations.front()
            .is_some_and(|animation| animation.started);
        if faint_pending && between_animations {
            let world = app.world_mut();
            assert!(world.query_filtered::<&Transform, With<BattleBattlerMarker>>()
                .iter(world).any(|transform| transform.translation.x > 0.0),
                "zero HP must not remove the opponent before its pending faint animation");
        }
        if let Some(frame) = faint_frame {
            saw_faint = true;
            let shell = app.world().resource::<BevyRuntimeShell>();
            assert_eq!(shell.battle_message_scene.as_ref().unwrap().battle.as_ref().unwrap().enemy_pokemon.hp, 0,
                "the damage scene must reach zero HP before the faint drop");
            if let Some(tween) = &shell.battle_hp_tween {
                assert_eq!(tween.enemy_pixels, 0, "the HP bar must empty before the faint drop");
            }
            let canvas = render_live_battle_canvas_for_test(app.world_mut(), "faint speech box");
            assert!((384..416).any(|y| (16..624).any(|x| canvas.get_pixel(x, y).0[..3] == [0, 0, 0])),
                "the source speech-box top border must remain during the faint animation");
            if [0, 6, 12].contains(&frame) && captured_frames.insert(frame) {
                save_live_battle_canvas_for_test(app.world_mut(), &format!("final-faint-{frame:02}.png"));
            }
        } else if saw_faint && !faint_pending {
            save_live_battle_canvas_for_test(app.world_mut(), "final-faint-after.png");
            let world = app.world_mut();
            let mut battlers = world.query_filtered::<&Transform, With<BattleBattlerMarker>>();
            assert!(!battlers.iter(world).any(|transform| transform.translation.x > 0.0),
                "the defeated opponent must remain absent after the faint animation");
            let canvas = render_live_battle_canvas_for_test(world, "post-faint HUD");
            for y in 0..128 {
                for x in 32..352 {
                    assert_eq!(*canvas.get_pixel(x, y), image::Rgba([255, 255, 255, 255]),
                        "FaintEnemyPokemon clears the source HUD rectangle after the tile drop at ({x},{y})");
                }
            }
            finished_faint = true;
            break;
        }
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    }
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert!(saw_finishing_move, "finishing SWIFT must be rendered");
    assert!(saw_faint && finished_faint,
        "the live frame loop must play and complete the final faint: messages={:?}, animations={:?}, events={:?}",
        shell.battle_messages,
        shell.visible_move_animations.iter().map(|animation| (&animation.move_id, animation.started, animation.frame)).collect::<Vec<_>>(),
        shell.last_audio_events);

    for _ in 0..2000 {
        if app.world().resource::<BevyRuntimeShell>().battle_messages.is_empty() { break; }
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    }
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert!(shell.battle_messages.is_empty(), "reward narration must finish");
    assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_FOUGHT_SUDOWOODO").unwrap(),
        "acknowledging the last reward must resume the suspended map script without another A");
}

fn save_live_battle_canvas_for_test(world: &mut World, name: &str) {
    let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") else { return; };
    let canvas = render_live_battle_canvas_for_test(world, name);
    std::fs::create_dir_all(&directory).unwrap();
    canvas.save(PathBuf::from(directory).join(name)).unwrap();
}

fn render_live_battle_canvas_for_test(world: &mut World, name: &str) -> image::RgbaImage {
    let sprites = world.query_filtered::<(&Sprite, &Transform, &Handle<Image>),
        Or<(With<BattleBattlerMarker>, With<BattleHudMarker>, With<BattleCommandMarker>)>>()
        .iter(world).map(|(sprite, transform, image)| (sprite.clone(), *transform, image.clone()))
        .collect::<Vec<_>>();
    let mut lcd = World::new();
    for sprite in sprites { lcd.spawn(sprite); }
    render_pc_audit_canvas(&mut lcd, world.resource::<Assets<Image>>(), name)
}

#[test]
fn battle_browser_fixture_starts_from_overworld_interaction() {
    let asset_root = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..").canonicalize().unwrap());
    let runtime = workspace_desktop_runtime(&asset_root);
    let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
    let mut shell = initialize_bevy_runtime_shell(asset_root, runtime,
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier, map_name: "Route36".into(), tile_x: 35, tile_y: 10,
        }, BevyShellConfig { smoke_player_name: Some("TEST".into()), ..Default::default() }).unwrap();
    complete_visible_smoke_player_name_if_needed(&mut shell, Some("TEST")).unwrap();
    let player_id = shell.shell.session().state().player_id;
    shell.shell.add_party_pokemon("MEWTWO", 100, None, None, "TEST", player_id,
        Dv::from_non_hp(10, 10, 10, 10)).unwrap();
    {
        let state = shell.shell.session_mut().state_mut();
        state.storage.party.pokemon[0].as_mut().unwrap().moves = vec![
            crate::core::models::LearnedMove { name: "SWIFT".into(), current_pp: 20, pp_ups: 0 },
        ];
        state.sync_party_from_storage();
    }
    shell.shell.add_bag_item("SQUIRTBOTTLE", 1).unwrap();
    settle_visible_shell_smoke_until_idle(&mut shell).unwrap();
    shell.shell.session.overworld.player.facing = Direction::Up;
    assert_eq!(shell.shell.current_overworld_interaction_checked().unwrap().map(|i| i.script),
        Some("SudowoodoScript".into()));
    if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        shell.shell.save(PathBuf::from(directory).join("battle-browser.crystalsave")).unwrap();
    }
    let mut app = menu_render_test_app(shell);
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::ArrowUp);
    let mut saw_faint = false;
    for _ in 0..3000 {
        let shell = app.world().resource::<BevyRuntimeShell>();
        saw_faint |= shell.battle_messages.iter().any(|m| m.contains("fainted!"));
        assert!(shell.last_error.is_none(), "{:?}", shell.last_error);
        if saw_faint && shell.battle_messages.is_empty() { break; }
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    }
    for _ in 0..200 { app.update(); }
    let shell = app.world().resource::<BevyRuntimeShell>();
    let snapshot = shell.shell.snapshot().unwrap();
    assert!(saw_faint, "real encounter must reach final faint");
    assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_FOUGHT_SUDOWOODO").unwrap(),
        "real encounter continuation: cursor={:?}, events={:?}, ui={:?}, special={:?}, scene={}, exp={:?}, messages={:?}",
        shell.active_script_cursor, snapshot.script_events, snapshot.ui, shell.special_boundary,
        shell.battle_message_scene.is_some(), shell.battle_exp_tween, shell.battle_messages);
}


#[test]
fn battle_transition_uses_textured_fe_tiles_and_native_colour_registers() {
    let shell = route36_battle_shell_for_render_regression();
    let mut images = Assets::<Image>::default();
    let mut indices = [0; 64];
    for (i, index) in indices.iter_mut().enumerate() {
        *index = (i % 4) as u8;
    }
    let map_palette = [[200, 240, 160], [100, 180, 80], [40, 100, 20], [8, 32, 0]];
    let tiles = vec![
        BattleTransitionTile {
            priority_from_row: None,
            indices,
            palette: map_palette
        };
        (CLASSIC_SCROLL_TILES_X * CLASSIC_SCROLL_TILES_Y) as usize
    ];
    let mut transition = VisibleBattleTransition {
        frame: 2,
        trainer_battle: true,
        cave_environment: false,
        stronger_enemy: false,
    };
    let handle = prepare_battle_transition_texture(
        &shell.asset_root,
        transition,
        &tiles,
        Vec2::ZERO,
        false,
        false,
        None,
        &mut images,
    )
    .unwrap();
    let pixel = |images: &Assets<Image>, x: usize, y: usize| -> [u8; 4] {
        images.get(&handle).unwrap().data[(y * 160 + x) * 4..(y * 160 + x + 1) * 4]
            .try_into()
            .unwrap()
    };
    // First FE cell starts at LCD tile (8, 1). Its 2bpp art has a dark
    // border, a light upper facet, a mid-tone side, and a red lower facet.
    assert_eq!(pixel(&images, 64, 8), [57, 57, 57, 255]);
    assert_eq!(pixel(&images, 65, 9), [255, 148, 239, 255]);
    assert_eq!(pixel(&images, 65, 10), [255, 90, 123, 255]);
    assert_eq!(pixel(&images, 65, 14), [255, 41, 41, 255]);
    // Outside the ball the map remains detailed, using BG palette seven.
    assert_eq!(pixel(&images, 0, 0), [255, 148, 239, 255]);
    assert_eq!(pixel(&images, 1, 0), [255, 90, 123, 255]);
    transition.frame = 4;
    let reused = prepare_battle_transition_texture(
        &shell.asset_root,
        transition,
        &tiles,
        Vec2::ZERO,
        false,
        false,
        Some(handle.clone()),
        &mut images,
    )
    .unwrap();
    assert_eq!(reused, handle);
    assert_eq!(pixel(&images, 65, 9), [255, 90, 123, 255]);
    assert_eq!(pixel(&images, 65, 10), [255, 41, 41, 255]);
    assert_eq!(pixel(&images, 65, 14), [57, 57, 57, 255]);
    transition.trainer_battle = false;
    transition.frame = 3;
    prepare_battle_transition_texture(
        &shell.asset_root,
        transition,
        &tiles,
        Vec2::ZERO,
        false,
        false,
        Some(handle.clone()),
        &mut images,
    )
    .unwrap();
    assert_eq!(pixel(&images, 0, 0), [100, 180, 80, 255]);
    assert_eq!(pixel(&images, 1, 0), [40, 100, 20, 255]);
    assert_eq!(pixel(&images, 2, 0), [8, 32, 0, 255]);
    transition.frame = 13; // Identity BGP, halfway through the first flash.
    prepare_battle_transition_texture(
        &shell.asset_root,
        transition,
        &tiles,
        Vec2::ZERO,
        false,
        false,
        Some(handle.clone()),
        &mut images,
    )
    .unwrap();
    assert_eq!(pixel(&images, 0, 0), [200, 240, 160, 255]);
    assert_eq!(pixel(&images, 1, 0), [100, 180, 80, 255]);
    assert_eq!(
        tiles[0].indices, indices,
        "flashing must not mutate source indices"
    );
    assert_eq!(
        tiles[0].palette, map_palette,
        "flashing must not poison map palettes"
    );
    assert_eq!(images.len(), 1, "reuse the transition image across frames");
    let mut foreground = tiles.clone();
    for tile in &mut foreground {
        tile.priority_from_row = Some(4);
    }
    let priority = prepare_battle_transition_texture(
        &shell.asset_root,
        transition,
        &foreground,
        Vec2::ZERO,
        false,
        true,
        None,
        &mut images,
    )
    .unwrap();
    let pixels = &images.get(&priority).unwrap().data;
    assert_eq!(
        pixels[(1 * 160 + 1) * 4 + 3],
        0,
        "clipped upper tile stays behind actors"
    );
    assert_eq!(
        pixels[(4 * 160) * 4 + 3],
        0,
        "colour zero stays behind actors"
    );
    assert_eq!(
        pixels[(4 * 160 + 1) * 4 + 3],
        255,
        "foreground detail covers actors"
    );
}

#[test]
fn battle_transition_trainer_and_wild_colour_surfaces_render_before_intro() {
    let mut app = battle_render_regression_app(route36_battle_shell_for_render_regression());
    for (trainer, frame, name) in [
        (true, 2, "trainer-transition-textured.png"),
        (true, 4, "trainer-transition-flash.png"),
        (false, 13, "wild-transition-colour.png"),
        (false, 80, "wild-transition-ripple-start.png"),
    ] {
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            shell.visible_battle_transition = Some(VisibleBattleTransition {
                frame,
                trainer_battle: trainer,
                cave_environment: !trainer,
                stronger_enemy: false,
            });
            mark_runtime_snapshot_dirty(&mut shell);
        }
        app.update();
        assert!(
            app.world()
                .resource::<BevyRuntimeShell>()
                .last_error
                .is_none()
        );
        let rendered = app.world().resource::<RenderedViewport>();
        let image = app
            .world()
            .resource::<Assets<Image>>()
            .get(rendered.transition_texture.as_ref().unwrap())
            .unwrap();
        assert_eq!(image.width(), 160);
        assert_eq!(image.height(), 144);
        assert!(
            image
                .data
                .chunks_exact(4)
                .any(|p| p[0] != p[1] || p[1] != p[2]),
            "{name} must retain chromatic pixels"
        );
        save_live_battle_canvas_for_test(app.world_mut(), name);
    }
}
