fn quest_move_beside_npc(shell: &mut BevyRuntimeShell, map: &str, script: &str) {
    let runtime = shell.shell.runtime().clone();
    let object = runtime.data().maps[map]
        .objects
        .iter()
        .find(|object| object.script == script)
        .unwrap()
        .clone();
    let tile = [(0, 1), (-1, 0), (1, 0), (0, -1)].into_iter()
        .map(|(dx, dy)| TilePosition::new(object.x as i16 + dx, object.y as i16 + dy))
        .find(|tile| tile.x >= 0 && tile.y >= 0 && runtime.start_overworld_session_at_runtime_tile(
            &shell.asset_root, map, tile.x, tile.y).is_ok())
        .expect("quest NPC has a walkable adjacent tile");
    let (state, overworld) = shell.shell.session_mut().state_and_overworld_mut();
    runtime
        .data()
        .transition_overworld_session(
            state,
            overworld,
            map,
            tile,
            crate::core::systems::map_context::SpawnMemoryUpdate::Preserve,
            &runtime.music_ids(),
        )
        .unwrap();
    reset_visible_navigation_state(shell);
    mark_runtime_snapshot_dirty(shell);
}

fn quest_talk(shell: &mut BevyRuntimeShell, script: &str) {
    let snapshot = shell.shell.snapshot().unwrap();
    let (index, object) = shell
        .shell
        .session()
        .overworld()
        .objects
        .iter()
        .enumerate()
        .find(|(_, object)| object.script == script)
        .unwrap();
    let interaction = crate::core::world::session::OverworldInteraction {
        map_name: snapshot.overworld.map_name,
        player_tile: snapshot.overworld.tile,
        facing: if object.x as i16 > snapshot.overworld.tile.x { Direction::Right }
            else if (object.x as i16) < snapshot.overworld.tile.x { Direction::Left }
            else if object.y as i16 > snapshot.overworld.tile.y { Direction::Down }
            else { Direction::Up },
        target_tile: TilePosition::new(object.x as i16, object.y as i16),
        script: script.into(),
        target: crate::core::world::session::OverworldInteractionTarget::Object {
            object_index: index as u16,
            object_identifier: object.object_identifier.clone(),
            object_type: object.object_type.clone(),
        },
    };
    dispatch_visible_overworld_interaction(shell, interaction, "quest_regression").unwrap();
}

fn quest_dialogue_is_idle(shell: &BevyRuntimeShell) -> bool {
    shell.active_script_cursor.is_none()
        && shell.map_reload_return_cursor.is_none()
        && shell.visible_walk_warp_phase.is_none()
        && shell.screen_fade.is_none()
        && shell.pending_scene_script.is_none()
        && shell.field_notice.is_none()
        && shell.pending_name_choice.is_none()
        && shell.pending_name_input.is_none()
        && shell.pending_gift_pokemon_nickname.is_none()
        && shell.visible_script_movement.is_none()
        && !shell.shell.has_pending_script_work()
        && !shell
            .shell
            .session()
            .state()
            .script_runtime
            .text_window_open
}

fn quest_settle(
    app: &mut App,
    answer: bool,
    stop: impl Fn(&BevyRuntimeShell) -> bool,
) -> Vec<String> {
    let mut labels = Vec::new();
    for _ in 0..1600 {
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            assert!(
                shell.last_error.is_none(),
                "quest error: {:?}",
                shell.last_error
            );
            if stop(&shell) {
                return labels;
            }
            if let Some(label) = shell
                .shell
                .session()
                .state()
                .script_runtime
                .active_text_label
                .clone()
            {
                if labels.last() != Some(&label) {
                    labels.push(label);
                }
            }
            // Fast-forward only the typewriter. Script delays, walks and
            // animation boundaries still run through real host updates.
            let mut text_changed = false;
            for _ in 0..100 {
                text_changed |= tick_visible_field_text_reveal(&mut shell, true).unwrap();
            }
            if text_changed { mark_runtime_presentation_dirty(&mut shell); }
            if shell
                .shell
                .session()
                .state()
                .script_runtime
                .pending_yes_no
                .is_some()
            {
                resolve_visible_pending_yes_no(&mut shell, answer).unwrap();
            }
        }
        let gift_nickname = app.world().resource::<BevyRuntimeShell>().pending_gift_pokemon_nickname.is_some();
        press_key_for_runtime_hotkey_app(app, if gift_nickname { KeyCode::KeyX } else { KeyCode::KeyZ });
    }
    let shell = app.world().resource::<BevyRuntimeShell>();
    panic!(
        "quest stalled: cursor={:?}, text={:?}, movement={:?}, boundary={:?}, error={:?}, reveal={:?}, rendered={:?}, item={:?}, sfx_wait={}, pending_yes_no={:?}",
        shell.active_script_cursor,
        shell.field_notice,
        shell.visible_script_movement,
        shell.special_boundary,
        shell.last_error,
        shell.field_text_reveal,
        shell.rendered_field_text_identity,
        shell.visible_field_item_notice,
        shell.visible_wait_sfx_boundary,
        shell.shell.session().state().script_runtime.pending_yes_no
    );
}

fn quest_item_quantity(shell: &BevyRuntimeShell, item: &str) -> u16 {
    shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&shell.shell.runtime().data().items[item])
}

#[test]
fn medicine_quest_requires_the_request_preserves_refusal_and_heals_amphy() {
    let mut shell = progression_shell_on_map_for_test("OlivineLighthouse6F");
    quest_move_beside_npc(
        &mut shell,
        "OlivineLighthouse6F",
        "OlivineLighthouseJasmine",
    );
    quest_talk(&mut shell, "OlivineLighthouseJasmine");
    let mut app = menu_render_test_app(shell);
    quest_settle(&mut app, true, quest_dialogue_is_idle);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert!(shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_JASMINE_EXPLAINED_AMPHYS_SICKNESS")
            .unwrap());
        assert_eq!(quest_item_quantity(&shell, "SECRETPOTION"), 0);
        quest_move_beside_npc(&mut shell, "CianwoodPharmacy", "CianwoodPharmacist");
        quest_talk(&mut shell, "CianwoodPharmacist");
    }
    quest_settle(&mut app, true, quest_dialogue_is_idle);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, "SECRETPOTION"), 1);
        assert!(shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_GOT_SECRETPOTION_FROM_PHARMACY")
            .unwrap());
        quest_move_beside_npc(
            &mut shell,
            "OlivineLighthouse6F",
            "OlivineLighthouseJasmine",
        );
        quest_talk(&mut shell, "OlivineLighthouseJasmine");
    }
    quest_settle(&mut app, false, quest_dialogue_is_idle);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(
            quest_item_quantity(&shell, "SECRETPOTION"),
            1,
            "refusal consumed medicine"
        );
        assert!(!shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_JASMINE_RETURNED_TO_GYM")
            .unwrap());
        quest_assert_save_round_trip(&mut shell, "medicine-refused");
        assert_eq!(quest_item_quantity(&shell, "SECRETPOTION"), 1);
        quest_move_beside_npc(&mut shell, "OlivineLighthouse6F", "OlivineLighthouseJasmine");
        quest_talk(&mut shell, "OlivineLighthouseJasmine");
    }
    let labels = quest_settle(&mut app, true, quest_dialogue_is_idle);
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    quest_assert_save_round_trip(&mut shell, "medicine-amphy-healed");
    assert_eq!(quest_item_quantity(&shell, "SECRETPOTION"), 0);
    assert!(shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set("EVENT_JASMINE_RETURNED_TO_GYM")
        .unwrap());
    assert!(!shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set("EVENT_OLIVINE_GYM_JASMINE")
        .unwrap());
    assert!(labels.iter().any(|label| label == "JasmineThankYouText"));
    assert!(!shell
        .shell
        .snapshot()
        .unwrap()
        .visible_objects
        .iter()
        .any(|object| object.object_identifier.as_deref() == Some("OLIVINELIGHTHOUSE6F_JASMINE")));
    quest_move_beside_npc(&mut shell, "CianwoodPharmacy", "CianwoodPharmacist");
    quest_talk(&mut shell, "CianwoodPharmacist");
    drop(shell);
    quest_settle(&mut app, true, |shell| shell.shop_top_cursor.is_some());
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(shell.shell.snapshot().unwrap().pending_shop.unwrap().mart_id, "MART_CIANWOOD");
    assert_eq!(quest_item_quantity(shell, "SECRETPOTION"), 0, "healing must not enable a second potion");
}

#[test]
fn suicune_burned_tower_release_finishes_and_initializes_only_the_two_roamers() {
    let mut shell = progression_shell_on_map_for_test("BurnedTowerB1F");
    arm_visible_active_script_cursor(&mut shell, "ReleaseTheBeasts", 0);
    execute_visible_active_script_step(&mut shell).unwrap();
    let mut app = menu_render_test_app(shell);
    quest_settle(&mut app, true, quest_dialogue_is_idle);
    let shell = app.world().resource::<BevyRuntimeShell>();
    let state = shell.shell.session().state();
    assert!(state
        .flags
        .is_event_flag_set("EVENT_RELEASED_THE_BEASTS")
        .unwrap());
    assert_eq!(state.roaming_pokemon[0].species.as_deref(), Some("RAIKOU"));
    assert_eq!(state.roaming_pokemon[1].species.as_deref(), Some("ENTEI"));
    assert!(
        state.roaming_pokemon[2].species.is_none(),
        "Crystal's Suicune is a scripted encounter"
    );
    assert!(!state
        .flags
        .is_event_flag_set("EVENT_SAW_SUICUNE_AT_CIANWOOD_CITY")
        .unwrap());
    let snapshot = shell.shell.snapshot().unwrap();
    assert!(!snapshot.visible_objects.iter().any(|object| object
        .object_identifier
        .as_deref()
        .is_some_and(|id| id.starts_with("BURNEDTOWERB1F_SUICUNE"))));
    assert!(snapshot
        .visible_objects
        .iter()
        .any(|object| object.object_identifier.as_deref() == Some("BURNEDTOWERB1F_EUSINE")));
}

#[test]
fn quest_bicycle_and_all_three_rods_handle_refusal_gift_and_repeat() {
    for (map, script, item) in [
        (
            "GoldenrodBikeShop",
            "GoldenrodBikeShopClerkScript",
            "BICYCLE",
        ),
        (
            "Route32Pokecenter1F",
            "Route32Pokecenter1FFishingGuruScript",
            "OLD_ROD",
        ),
        ("OlivineGoodRodHouse", "GoodRodGuru", "GOOD_ROD"),
        (
            "Route12SuperRodHouse",
            "Route12SuperRodHouseFishingGuruScript",
            "SUPER_ROD",
        ),
    ] {
        let mut shell = progression_shell_on_map_for_test(map);
        quest_move_beside_npc(&mut shell, map, script);
        quest_talk(&mut shell, script);
        let mut app = menu_render_test_app(shell);
        quest_settle(&mut app, false, quest_dialogue_is_idle);
        assert_eq!(
            quest_item_quantity(app.world().resource::<BevyRuntimeShell>(), item),
            0,
            "{item} granted despite refusal"
        );
        quest_talk(
            &mut app.world_mut().resource_mut::<BevyRuntimeShell>(),
            script,
        );
        quest_settle(&mut app, true, quest_dialogue_is_idle);
        assert_eq!(
            quest_item_quantity(app.world().resource::<BevyRuntimeShell>(), item),
            1,
            "{item} not granted"
        );
        quest_talk(
            &mut app.world_mut().resource_mut::<BevyRuntimeShell>(),
            script,
        );
        quest_settle(&mut app, true, quest_dialogue_is_idle);
        assert_eq!(
            quest_item_quantity(app.world().resource::<BevyRuntimeShell>(), item),
            1,
            "{item} repeat changed inventory"
        );
    }
}

#[test]
fn suicune_tower_sage_requires_clear_bell_and_preserves_it_after_opening_passage() {
    let mut shell = progression_shell_on_map_for_test("EcruteakTinTowerEntrance");
    shell
        .shell
        .session_mut()
        .state_mut()
        .flags
        .set_event_flag("EVENT_CLEARED_RADIO_TOWER", true)
        .unwrap();
    quest_move_beside_npc(
        &mut shell,
        "EcruteakTinTowerEntrance",
        "EcruteakTinTowerEntranceSageScript",
    );
    quest_talk(&mut shell, "EcruteakTinTowerEntranceSageScript");
    let mut app = menu_render_test_app(shell);
    let labels = quest_settle(&mut app, true, quest_dialogue_is_idle);
    assert!(labels
        .iter()
        .any(|label| label == "EcruteakTinTowerEntranceSageText_NoClearBell"));
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        let bell = shell.shell.runtime().data().items["CLEAR_BELL"].clone();
        assert!(shell
            .shell
            .session_mut()
            .state_mut()
            .bag
            .add_item(&bell, 1)
            .unwrap());
        mark_runtime_snapshot_dirty(&mut shell);
        quest_talk(&mut shell, "EcruteakTinTowerEntranceSageScript");
    }
    let labels = quest_settle(&mut app, true, quest_dialogue_is_idle);
    assert!(labels
        .iter()
        .any(|label| label == "EcruteakTinTowerEntranceSageText_HearsClearBell"));
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(quest_item_quantity(shell, "CLEAR_BELL"), 1);
    assert!(shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set("EVENT_RANG_CLEAR_BELL_2")
        .unwrap());
}

#[test]
fn suicune_tin_tower_choreography_reaches_special_battle_and_finishes_aftermath() {
    let mut shell = progression_shell_on_map_for_test("TinTower1F");
    let runtime = shell.shell.runtime().clone();
    let (state, overworld) = shell.shell.session_mut().state_and_overworld_mut();
    runtime
        .data()
        .transition_overworld_session(
            state,
            overworld,
            "TinTower1F",
            TilePosition::new(10, 15),
            crate::core::systems::map_context::SpawnMemoryUpdate::Preserve,
            &runtime.music_ids(),
        )
        .unwrap();
    reset_visible_navigation_state(&mut shell);
    mark_runtime_snapshot_dirty(&mut shell);

    arm_visible_active_script_cursor(&mut shell, "TinTower1FSuicuneBattleScript", 0);
    execute_visible_active_script_step(&mut shell).unwrap();
    let mut app = menu_render_test_app(shell);
    quest_settle(&mut app, true, |shell| {
        shell.shell.snapshot().unwrap().battle.is_some()
    });
    let shell = app.world().resource::<BevyRuntimeShell>();
    let snapshot = shell.shell.snapshot().unwrap();
    let battle = snapshot.battle.as_ref().unwrap();
    let origin = visible_static_wild_source(&snapshot, battle).unwrap();
    assert_eq!(origin.species, "SUICUNE");
    assert_eq!(origin.level, 40);
    assert_eq!(origin.battle_type, "BATTLETYPE_SUICUNE");
    assert!(
        !shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_FOUGHT_SUICUNE")
            .unwrap(),
        "quest must wait for the battle result"
    );
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        quest_finish_static_wild_final_turn(&mut shell);
    }
    let labels = quest_settle(&mut app, true, |shell| {
        quest_dialogue_is_idle(shell)
            && shell.battle_messages.is_empty()
            && shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set("EVENT_FOUGHT_SUICUNE")
                .unwrap()
    });
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert!(labels
        .iter()
        .any(|label| label == "TinTower1FEusineSuicuneText"));
    for event in [
        "EVENT_FOUGHT_SUICUNE",
        "EVENT_SAW_SUICUNE_ON_ROUTE_42",
        "EVENT_SAW_SUICUNE_ON_ROUTE_36",
        "EVENT_SAW_SUICUNE_AT_CIANWOOD_CITY",
    ] {
        assert!(
            shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(event)
                .unwrap(),
            "{event}"
        );
    }
    assert!(shell.shell.snapshot().unwrap().battle.is_none());
    quest_assert_save_round_trip(
        &mut app.world_mut().resource_mut::<BevyRuntimeShell>(),
        "suicune",
    );
}

#[test]
fn slowpoke_well_victory_rescues_the_town_heals_and_returns_to_kurt() {
    let mut shell = progression_shell_on_map_for_test("SlowpokeWellB1F");
    shell.shell.session_mut().state_mut().storage.party.pokemon[0]
        .as_mut()
        .unwrap()
        .hp = 1;
    shell
        .shell
        .session_mut()
        .state_mut()
        .sync_party_from_storage();
    // Resume the actual trainer callback at its post-victory script.
    arm_visible_active_script_cursor(&mut shell, ".Script@TrainerGruntM1", 0);
    execute_visible_active_script_step(&mut shell).unwrap();
    let mut app = menu_render_test_app(shell);
    quest_settle(&mut app, true, quest_dialogue_is_idle);
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(
        shell.shell.snapshot().unwrap().overworld.map_name,
        "KurtsHouse"
    );
    let state = shell.shell.session().state();
    for event in [
        "EVENT_CLEARED_SLOWPOKE_WELL",
        "EVENT_SLOWPOKE_WELL_SLOWPOKES",
        "EVENT_SLOWPOKE_WELL_KURT",
    ] {
        assert!(state.flags.is_event_flag_set(event).unwrap(), "{event}");
    }
    for event in [
        "EVENT_AZALEA_TOWN_SLOWPOKES",
        "EVENT_KURTS_HOUSE_KURT_1",
        "EVENT_ILEX_FOREST_APPRENTICE",
        "EVENT_ILEX_FOREST_FARFETCHD",
    ] {
        assert!(!state.flags.is_event_flag_set(event).unwrap(), "{event}");
    }
    let pokemon = state.storage.party.pokemon[0].as_ref().unwrap();
    assert_eq!(pokemon.hp, pokemon.max_hp);
}

fn quest_start_coord_script(shell: &mut BevyRuntimeShell, map: &str, script: &str) {
    let runtime = shell.shell.runtime().clone();
    let event = runtime.data().maps[map]
        .events
        .coord_events
        .iter()
        .find(|event| event.script_name == script)
        .unwrap();
    let tile = TilePosition::new(event.x as i16, event.y as i16);
    let (state, overworld) = shell.shell.session_mut().state_and_overworld_mut();
    runtime
        .data()
        .transition_overworld_session(
            state,
            overworld,
            map,
            tile,
            crate::core::systems::map_context::SpawnMemoryUpdate::Preserve,
            &runtime.music_ids(),
        )
        .unwrap();
    reset_visible_navigation_state(shell);
    mark_runtime_snapshot_dirty(shell);
    arm_visible_active_script_cursor(shell, script, 0);
    execute_visible_active_script_step(shell).unwrap();
}

#[test]
fn suicune_route_sightings_finish_and_arm_the_next_sighting() {
    for (map, script, hidden_id, next_flag) in [
        (
            "Route42",
            "Route42SuicuneScript",
            "ROUTE42_SUICUNE",
            "EVENT_SAW_SUICUNE_ON_ROUTE_36",
        ),
        (
            "Route36",
            "Route36SuicuneScript",
            "ROUTE36_SUICUNE",
            "EVENT_SAW_SUICUNE_AT_CIANWOOD_CITY",
        ),
    ] {
        let mut shell = progression_shell_on_map_for_test(map);
        shell
            .shell
            .session_mut()
            .state_mut()
            .flags
            .set_event_flag(next_flag, true)
            .unwrap();
        quest_start_coord_script(&mut shell, map, script);
        let mut app = menu_render_test_app(shell);
        quest_settle(&mut app, true, quest_dialogue_is_idle);
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(!shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set(next_flag)
            .unwrap());
        assert!(!shell
            .shell
            .snapshot()
            .unwrap()
            .visible_objects
            .iter()
            .any(|object| object.object_identifier.as_deref() == Some(hidden_id)));
    }
}

#[test]
fn suicune_cianwood_eusine_battle_resumes_departure_and_does_not_repeat() {
    let mut shell = progression_shell_on_map_for_test("CianwoodCity");
    quest_start_coord_script(&mut shell, "CianwoodCity", "CianwoodCitySuicuneAndEusine");
    let mut app = menu_render_test_app(shell);
    quest_settle(&mut app, true, |shell| {
        shell.shell.snapshot().unwrap().battle.is_some()
    });
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        let key = shell
            .shell
            .scripted_trainer_battle_keys()
            .into_iter()
            .find(|key| key.trainer_class == "MYSTICALMAN")
            .unwrap();
        let state = shell.shell.session_mut().state_mut();
        if let crate::core::state::BattleMemory::Trainer {
            enemy_pokemon,
            enemy_party,
            ..
        } = &mut state.battle
        {
            state.battle_rewarded_enemy_party_indices = (0..enemy_party.len()).collect();
            for pokemon in enemy_party {
                pokemon.hp = 0;
            }
            enemy_pokemon.hp = 0;
        } else {
            panic!("Eusine trainer battle missing");
        }
        reset_visible_battle_exit_state(&mut shell);
        shell.battle_messages.clear();
        shell.battle_message_scene = Some(Box::new(shell.shell.snapshot().unwrap()));
        complete_visible_scripted_trainer_battle(
            &mut shell,
            &key.map_name,
            &key.source_script,
            true,
            false,
        )
        .unwrap();
    }
    let labels = quest_settle(&mut app, true, quest_dialogue_is_idle);
    assert!(labels.iter().any(|label| label == "EusineAfterText"));
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert!(shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_FOUGHT_EUSINE")
            .unwrap());
        assert!(!shell
            .shell
            .snapshot()
            .unwrap()
            .visible_objects
            .iter()
            .any(|object| object.object_identifier.as_deref() == Some("CIANWOODCITY_EUSINE")));
        // The sighting cycle can return to Cianwood; Eusine must not rebattle.
        quest_start_coord_script(&mut shell, "CianwoodCity", "CianwoodCitySuicuneAndEusine");
    }
    quest_settle(&mut app, true, quest_dialogue_is_idle);
    assert!(app
        .world()
        .resource::<BevyRuntimeShell>()
        .shell
        .snapshot()
        .unwrap()
        .battle
        .is_none());
}

#[test]
fn farfetchd_chase_handles_a_wrong_approach_then_cut_and_charcoal_rewards() {
    let shell = progression_shell_on_map_for_test("IlexForest");
    let mut app = menu_render_test_app(shell);
    // Position the player beside the bird between legs. Every chase path,
    // facing branch, cry, and reward is executed by the authored script.
    for (directions, expected_position) in [
        (
            vec![
                Direction::Up,
                Direction::Left,
                Direction::Right,
                Direction::Down,
            ],
            2,
        ),
        (vec![Direction::Up, Direction::Left, Direction::Right], 3),
        (vec![Direction::Right, Direction::Down, Direction::Up], 4),
        (vec![Direction::Up], 3), // Approaching from below sends it back.
        (vec![Direction::Right, Direction::Down, Direction::Up], 4),
        (vec![Direction::Down, Direction::Left, Direction::Right], 5),
        (vec![Direction::Down], 6),
        (vec![Direction::Up, Direction::Down, Direction::Left], 7),
        (vec![Direction::Up, Direction::Right], 8),
        (vec![Direction::Down], 9),
        (vec![Direction::Up, Direction::Left], 10),
    ] {
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            let target = shell
                .shell
                .session()
                .overworld()
                .object_runtime_tile_by_id("ILEXFOREST_FARFETCHD")
                .unwrap();
            let (facing, player) = directions
                .into_iter()
                .find_map(|facing| {
                    let (dx, dy) = match facing {
                        Direction::Up => (0, 1),
                        Direction::Down => (0, -1),
                        Direction::Left => (1, 0),
                        Direction::Right => (-1, 0),
                    };
                    let player = TilePosition::new(target.x + dx, target.y + dy);
                    shell
                        .shell
                        .runtime()
                        .start_overworld_session_at_runtime_tile(
                            &shell.asset_root,
                            "IlexForest",
                            player.x,
                            player.y,
                        )
                        .is_ok()
                        .then_some((facing, player))
                })
                .expect("a walkable approach for the expected chase branch");
            let overworld = shell.shell.session_mut().overworld_mut();
            overworld.player.tile = player;
            overworld.set_player_facing(facing);
            let (index, object) = overworld
                .objects
                .iter()
                .enumerate()
                .find(|(_, object)| object.script == "IlexForestFarfetchdScript")
                .unwrap();
            let interaction = crate::core::world::session::OverworldInteraction {
                map_name: "IlexForest".into(),
                player_tile: player,
                facing,
                target_tile: target,
                script: "IlexForestFarfetchdScript".into(),
                target: crate::core::world::session::OverworldInteractionTarget::Object {
                    object_index: index as u16,
                    object_identifier: object.object_identifier.clone(),
                    object_type: object.object_type.clone(),
                },
            };
            mark_runtime_snapshot_dirty(&mut shell);
            dispatch_visible_overworld_interaction(&mut shell, interaction, "farfetchd_regression")
                .unwrap();
        }
        quest_settle(&mut app, true, quest_dialogue_is_idle);
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(
            shell
                .shell
                .session()
                .state()
                .script_runtime
                .memory
                .get("wFarfetchdPosition"),
            Some(&expected_position.to_string())
        );
        assert_eq!(
            quest_item_quantity(shell, "HM_CUT"),
            0,
            "chasing alone must not award Cut"
        );
    }
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert!(shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_HERDED_FARFETCHD")
            .unwrap());
        quest_move_beside_npc(&mut shell, "IlexForest", "IlexForestCharcoalMasterScript");
        quest_talk(&mut shell, "IlexForestCharcoalMasterScript");
    }
    quest_settle(&mut app, true, quest_dialogue_is_idle);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, "HM_CUT"), 1);
        assert!(shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_GOT_HM01_CUT")
            .unwrap());
        quest_move_beside_npc(&mut shell, "CharcoalKiln", "CharcoalKilnApprentice");
        quest_talk(&mut shell, "CharcoalKilnApprentice");
    }
    quest_settle(&mut app, true, quest_dialogue_is_idle);
    assert_eq!(
        quest_item_quantity(app.world().resource::<BevyRuntimeShell>(), "CHARCOAL"),
        1
    );
    quest_talk(
        &mut app.world_mut().resource_mut::<BevyRuntimeShell>(),
        "CharcoalKilnApprentice",
    );
    quest_settle(&mut app, true, quest_dialogue_is_idle);
    assert_eq!(
        quest_item_quantity(app.world().resource::<BevyRuntimeShell>(), "CHARCOAL"),
        1
    );
    quest_assert_save_round_trip(
        &mut app.world_mut().resource_mut::<BevyRuntimeShell>(),
        "farfetchd",
    );
}

fn quest_assert_save_round_trip(shell: &mut BevyRuntimeShell, name: &str) {
    let path = std::env::temp_dir().join(format!(
        "geothite-quest-{name}-{}.crystalsave",
        std::process::id()
    ));
    let flags = shell.shell.session().state().flags.clone();
    let bag = shell.shell.session().state().bag.clone();
    let puzzle_position = shell
        .shell
        .session()
        .state()
        .script_runtime
        .memory
        .get("wFarfetchdPosition")
        .cloned();
    shell.shell.save(&path).unwrap();
    load_visible_runtime_save(shell, &path, "quest_round_trip").unwrap();
    let state = shell.shell.session().state();
    assert_eq!(state.flags, flags);
    assert_eq!(state.bag, bag);
    assert_eq!(
        state.script_runtime.memory.get("wFarfetchdPosition"),
        puzzle_position.as_ref()
    );
    std::fs::remove_file(path).unwrap();
}

#[test]
fn every_overworld_tm_and_hm_has_an_acquisition_source_in_the_shipped_pack() {
    let shell = progression_shell_on_map_for_test("CherrygroveCity");
    let data = shell.shell.runtime().data();
    let mut machines = data
        .items
        .iter()
        .filter(|(_, item)| item.tmhm_index.is_some())
        .map(|(id, _)| id.clone())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(machines.len(), 57, "Crystal's 50 TMs and seven HMs");
    // TM09 is the original game's Time Capsule-only exception. Do not
    // invent an overworld gift to make this coverage assertion pass.
    assert!(machines.remove("TM_PSYCH_UP"));
    let mut sources = std::collections::BTreeSet::new();
    for module in data.maps.values() {
        sources.extend(
            module
                .script_item_grants
                .iter()
                .map(|grant| grant.item_id.clone()),
        );
        sources.extend(
            module
                .script_field_pickups
                .iter()
                .filter_map(|pickup| pickup.item_id.clone()),
        );
        for shop in &module.script_shop_commands {
            if let Some(items) = data.marts.0.get(&shop.mart_id) {
                sources.extend(items.iter().cloned());
            }
        }
    }
    let missing = machines.difference(&sources).collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "TM/HM definitions without a mapped gift, pickup or shop: {missing:?}"
    );
}

#[test]
fn shiny_dv_detection_matches_all_generation_two_combinations() {
    let shell = progression_shell_on_map_for_test("LakeOfRage");
    let mut pokemon = shell.shell.session().state().storage.party.pokemon[0].as_ref().unwrap().clone();
    let mut shiny_count = 0;
    for packed in 0u32..65536 {
        pokemon.dvs = Dv::from_non_hp((packed >> 12) as u8, ((packed >> 8) & 15) as u8,
            ((packed >> 4) & 15) as u8, (packed & 15) as u8);
        let expected = packed & 0x2fff == 0x2aaa;
        assert_eq!(visible_pokemon_is_shiny(&pokemon), expected, "DV word {packed:04x}");
        shiny_count += usize::from(expected);
    }
    assert_eq!(shiny_count, 8, "eight shiny combinations out of 65536 DV words");
}

#[test]
fn lake_of_rage_script_starts_a_shiny_gyarados() {
    let mut shell = progression_shell_on_map_for_test("LakeOfRage");
    arm_visible_active_script_cursor(&mut shell, "RedGyarados", 0);
    execute_visible_active_script_step(&mut shell).unwrap();
    let mut app = menu_render_test_app(shell);
    quest_settle(&mut app, true, |shell| shell.shell.snapshot().unwrap().battle.is_some());
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    let snapshot = shell.shell.snapshot().unwrap();
    let battle = snapshot.battle.as_ref().unwrap();
    let origin = visible_static_wild_source(&snapshot, battle).unwrap();
    assert_eq!(origin.species, "GYARADOS");
    assert_eq!(origin.level, 30);
    assert_eq!(origin.battle_type, "BATTLETYPE_FORCESHINY");
    assert!(visible_pokemon_is_shiny(&battle.enemy_pokemon));
    let original_dvs = battle.enemy_pokemon.dvs;
    let ball = shell.runtime.data().items["MASTER_BALL"].clone();
    shell.shell.session_mut().state_mut().bag.add_item(&ball, 1).unwrap();
    let outcome = shell.shell.throw_ball_at_active_battle("MASTER_BALL").unwrap().outcome.unwrap();
    assert!(outcome.caught);
    let captured = shell.shell.complete_active_wild_capture(&outcome, None).unwrap().stored.unwrap();
    assert_eq!(captured.pokemon.dvs, original_dvs);
    let deposited = shell.shell.deposit_party_pokemon_to_current_box(1).unwrap();
    assert_eq!(deposited.pokemon.dvs, original_dvs);
    let withdrawn = shell.shell.withdraw_current_box_pokemon_to_party(deposited.box_slot).unwrap();
    assert_eq!(withdrawn.pokemon.dvs, original_dvs);
    quest_assert_save_round_trip(&mut shell, "red-gyarados-captured");
    let restored = shell.shell.session().state().storage.party.pokemon[1].as_ref().unwrap();
    assert_eq!(restored.species.id, "GYARADOS");
    assert_eq!(restored.dvs, original_dvs);
    assert!(visible_pokemon_is_shiny(restored));
}

#[test]
fn every_species_has_loadable_normal_and_shiny_front_and_back_art() {
    let shell = progression_shell_on_map_for_test("LakeOfRage");
    let species = &shell.shell.runtime().data().pokemon;
    assert_eq!(species.len(), 251);
    let mut images = Assets::<Image>::default();
    for id in species.keys() {
        let asset = normalize_pokemon_asset_id(id);
        for side in [PokemonSpriteSide::Front, PokemonSpriteSide::Back] {
            for shiny in [false, true] {
                let palette = load_pokemon_palette(&shell.asset_root, &asset, side, shiny)
                    .unwrap_or_else(|error| panic!("{id} {side:?} shiny={shiny} palette: {error}"));
                assert_eq!(palette[0], [255, 255, 255]);
                assert_eq!(palette[3], [0, 0, 0]);
                let frame = load_pokemon_animation_frame(
                    &shell.asset_root, id, side, shiny, 0, &mut images,
                ).unwrap_or_else(|error| panic!("{id} {side:?} shiny={shiny} art: {error}"));
                assert!(frame.size.x > 0.0 && frame.size.y > 0.0);
                assert!(images.get(&frame.handle).unwrap().data
                    .chunks_exact(4).any(|pixel| pixel[3] != 0));
            }
        }
    }
}

#[test]
fn shiny_unown_forms_and_hatch_art_use_the_shipped_palettes() {
    let mut shell = progression_shell_on_map_for_test("LakeOfRage");
    let mut images = Assets::<Image>::default();
    for letter in b'a'..=b'z' {
        let id = format!("unown_{}", char::from(letter));
        for side in [PokemonSpriteSide::Front, PokemonSpriteSide::Back] {
            load_pokemon_animation_frame(&shell.asset_root, &id, side, true, 0, &mut images)
                .unwrap_or_else(|error| panic!("{id} {side:?}: {error}"));
        }
    }
    let dvs = Dv::from_non_hp(2, 10, 10, 10);
    assert_eq!(pokemon_asset_id_for_dvs("UNOWN", dvs), "unown_i");
    assert_eq!(pokemon_asset_id_for_dvs("UNOWN", Dv::from_non_hp(14, 10, 10, 10)), "unown_v");
    shell.shell.session_mut().state_mut().storage.party.pokemon[0].as_mut().unwrap().dvs = dvs;
    let species_id = shell.shell.session().state().storage.party.pokemon[0].as_ref().unwrap().species.id.clone();
    shell.visible_egg_hatch = Some(VisibleEggHatch {
        party_index: 0, species_id: species_id.clone(), phase: VisibleEggHatchPhase::Reveal, frame: 0,
    });
    let mut world = World::new();
    let mut queue = bevy::ecs::world::CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);
    let mut art = RenderedTilesetArt::default();
    spawn_visible_egg_hatch(&mut commands, &shell, &mut art, &shell.asset_root, &mut images).unwrap();
    queue.apply(&mut world);
    let key = PokemonArtKey { species_id: normalize_pokemon_asset_id(&species_id),
        side: PokemonSpriteSide::Front, shiny: true, frame: 0 };
    assert!(art.pokemon_cache.contains_key(&key), "hatch reveal must select the shiny sprite");
    if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
        let frame = &art.pokemon_cache[&key];
        let sprite = images.get(&frame.handle).unwrap();
        image::RgbaImage::from_raw(sprite.width(), sprite.height(), sprite.data.clone()).unwrap()
            .save(PathBuf::from(directory).join("shiny-hatch-sprite.png")).unwrap();
    }
    quest_assert_save_round_trip(&mut shell, "shiny");
    let restored = shell.shell.session().state().storage.party.pokemon[0].as_ref().unwrap();
    assert_eq!(restored.dvs, dvs);
    assert!(visible_pokemon_is_shiny(restored));
}

#[test]
fn shiny_stone_evolution_preserves_dvs_and_save_state() {
    let mut shell = progression_shell_on_map_for_test("CherrygroveCity");
    let dvs = Dv::from_non_hp(14, 10, 10, 10);
    shell.shell.add_party_pokemon("PIKACHU", 20, None, None, "CHRIS", 1, dvs).unwrap();
    let stone = shell.runtime.data().items["THUNDERSTONE"].clone();
    shell.shell.session_mut().state_mut().bag.add_item(&stone, 1).unwrap();
    shell.shell.use_bag_item_on_party_pokemon("THUNDERSTONE", 1).unwrap();
    let evolved = shell.shell.session().state().storage.party.pokemon[1].as_ref().unwrap();
    assert_eq!(evolved.species.id, "RAICHU");
    assert_eq!(evolved.dvs, dvs);
    assert!(visible_pokemon_is_shiny(evolved));
    quest_assert_save_round_trip(&mut shell, "shiny-evolution");
    let restored = shell.shell.session().state().storage.party.pokemon[1].as_ref().unwrap();
    assert_eq!(restored.species.id, "RAICHU");
    assert_eq!(restored.dvs, dvs);
}

fn quest_finish_static_wild_final_turn(shell: &mut BevyRuntimeShell) {
    // The choreography reaches a real battle. Set up its final turn;
    // the move, faint, reward and script continuation execute normally.
    shell.visible_battle_transition = None;
    shell.visible_battle_sliding_intro = None;
    shell.battle_entry_messages_remaining = 0;
    shell.battle_enemy_send_out_pending = false;
    shell.battle_player_send_out_pending = false;
    shell.battle_messages.clear();
    shell.battle_message_scenes.clear();
    shell.battle_text_reveal = None;
    shell.battle_hp_tween = None;
    let state = shell.shell.session_mut().state_mut();
    state.storage.party.pokemon[0].as_mut().unwrap().moves =
        vec![crate::core::models::LearnedMove {
            name: "SWIFT".into(),
            current_pp: 20,
            pp_ups: 0,
        }];
    state.sync_party_from_storage();
    let crate::core::state::BattleMemory::StaticWild {
        enemy_pokemon,
        enemy_party,
        ..
    } = &mut state.battle
    else {
        panic!("quest must be static wild");
    };
    enemy_pokemon.hp = 1;
    enemy_pokemon.moves = vec![crate::core::models::LearnedMove {
        name: "SPLASH".into(),
        current_pp: 40,
        pp_ups: 0,
    }];
    enemy_party[0] = enemy_pokemon.clone();
    state.script_runtime.active_battle_combat = None;
    mark_runtime_snapshot_dirty(shell);
    shell.battle_message_scene = Some(Box::new(shell.shell.snapshot().unwrap()));
    resolve_visible_battle_move(shell, 0).unwrap();
}

#[test]
fn lake_of_rage_aftermath_lance_and_red_scale_trade_complete() {
    let mut shell = progression_shell_on_map_for_test("LakeOfRage");
    arm_visible_active_script_cursor(&mut shell, "RedGyarados", 0);
    execute_visible_active_script_step(&mut shell).unwrap();
    let mut app = menu_render_test_app(shell);
    quest_settle(&mut app, true, |shell| shell.shell.has_active_battle());
    quest_finish_static_wild_final_turn(&mut app.world_mut().resource_mut::<BevyRuntimeShell>());
    quest_settle(&mut app, true, |shell| quest_dialogue_is_idle(shell)
        && shell.battle_messages.is_empty() && !shell.shell.has_active_battle());
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, "RED_SCALE"), 1);
        assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_LAKE_OF_RAGE_RED_GYARADOS").unwrap());
        assert!(!shell.shell.session().state().flags.is_event_flag_set("EVENT_LAKE_OF_RAGE_LANCE").unwrap());
        quest_move_beside_npc(&mut shell, "LakeOfRage", "LakeOfRageLanceScript");
        quest_talk(&mut shell, "LakeOfRageLanceScript");
    }
    quest_settle(&mut app, false, quest_dialogue_is_idle);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert!(!shell.shell.session().state().flags.is_event_flag_set("EVENT_DECIDED_TO_HELP_LANCE").unwrap());
        assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_REFUSED_TO_HELP_LANCE_AT_LAKE_OF_RAGE").unwrap());
        quest_talk(&mut shell, "LakeOfRageLanceScript");
    }
    quest_settle(&mut app, true, quest_dialogue_is_idle);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        let state = shell.shell.session().state();
        assert!(state.flags.is_event_flag_set("EVENT_DECIDED_TO_HELP_LANCE").unwrap());
        assert!(state.flags.is_event_flag_set("EVENT_LAKE_OF_RAGE_LANCE").unwrap());
        assert!(!state.flags.is_event_flag_set("EVENT_MAHOGANY_MART_LANCE_AND_DRAGONITE").unwrap());
        assert_eq!(state.scenes.map_scenes.get("MahoganyMart1F").map(String::as_str),
            Some("SCENE_MAHOGANYMART1F_LANCE_UNCOVERS_STAIRS"));
        shell.shell.session_mut().state_mut().scenes.map_scenes.insert("MrPokemonsHouse".into(), "SCENE_MRPOKEMONSHOUSE_NOOP".into());
        shell.shell.session_mut().state_mut().scenes.map_scene_indices.insert("MrPokemonsHouse".into(), 1);
        quest_move_beside_npc(&mut shell, "MrPokemonsHouse", "MrPokemonsHouse_MrPokemonScript");
        quest_talk(&mut shell, "MrPokemonsHouse_MrPokemonScript");
    }
    quest_settle(&mut app, false, quest_dialogue_is_idle);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, "RED_SCALE"), 1);
        assert_eq!(quest_item_quantity(&shell, "EXP_SHARE"), 0);
        quest_talk(&mut shell, "MrPokemonsHouse_MrPokemonScript");
    }
    quest_settle(&mut app, true, quest_dialogue_is_idle);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, "RED_SCALE"), 0);
        assert_eq!(quest_item_quantity(&shell, "EXP_SHARE"), 1);
        quest_talk(&mut shell, "MrPokemonsHouse_MrPokemonScript");
    }
    let labels = quest_settle(&mut app, true, quest_dialogue_is_idle);
    assert!(!labels.iter().any(|label| label == "MrPokemonText_GimmeTheScale"));
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    assert_eq!(quest_item_quantity(&shell, "EXP_SHARE"), 1);
    quest_assert_save_round_trip(&mut shell, "red-scale-lance");
}

#[test]
fn red_scale_trade_preserves_the_scale_when_the_item_pocket_is_full() {
    let mut shell = progression_shell_on_map_for_test("MrPokemonsHouse");
    shell.shell.session_mut().state_mut().scenes.map_scenes.insert("MrPokemonsHouse".into(), "SCENE_MRPOKEMONSHOUSE_NOOP".into());
        shell.shell.session_mut().state_mut().scenes.map_scene_indices.insert("MrPokemonsHouse".into(), 1);
    let items = shell.runtime.data().items.values().filter(|item|
        item.pocket == "ITEM" && item.script_name != "EXP_SHARE").take(20).cloned().collect::<Vec<_>>();
    let scale = shell.runtime.data().items["RED_SCALE"].clone();
    {
        let bag = &mut shell.shell.session_mut().state_mut().bag;
        bag.items.clear();
        for item in &items { assert!(bag.add_item(item, 1).unwrap()); }
        assert!(bag.add_item(&scale, 1).unwrap());
        assert_eq!(bag.items.len(), 20);
    }
    quest_move_beside_npc(&mut shell, "MrPokemonsHouse", "MrPokemonsHouse_MrPokemonScript");
    let before = shell.shell.session().state().bag.clone();
    quest_talk(&mut shell, "MrPokemonsHouse_MrPokemonScript");
    let mut app = menu_render_test_app(shell);
    quest_settle(&mut app, true, quest_dialogue_is_idle);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(shell.shell.session().state().bag, before);
        assert!(shell.shell.session_mut().state_mut().bag.remove_item(&items[0], 1).unwrap());
        quest_talk(&mut shell, "MrPokemonsHouse_MrPokemonScript");
    }
    quest_settle(&mut app, true, quest_dialogue_is_idle);
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(quest_item_quantity(shell, "RED_SCALE"), 0);
    assert_eq!(quest_item_quantity(shell, "EXP_SHARE"), 1);
}

#[test]
fn whitney_delays_badge_until_bridgets_scene_then_grants_attract_once() {
    let mut shell = progression_shell_on_map_for_test("GoldenrodGym");
    {
        // Begin at the authored post-victory state; do not award the badge
        // merely because the trainer has been defeated.
        let state = shell.shell.session_mut().state_mut();
        for flag in ["EVENT_BEAT_WHITNEY", "EVENT_MADE_WHITNEY_CRY"] {
            state.flags.set_event_flag(flag, true).unwrap();
        }
        state.scenes.map_scenes.insert("GoldenrodGym".into(), "SCENE_GOLDENRODGYM_WHITNEY_STOPS_CRYING".into());
        state.scenes.map_scene_indices.insert("GoldenrodGym".into(), 1);
    }
    quest_move_beside_npc(&mut shell, "GoldenrodGym", "GoldenrodGymWhitneyScript");
    quest_talk(&mut shell, "GoldenrodGymWhitneyScript");
    let mut app = menu_render_test_app(shell);
    let labels = quest_settle(&mut app, true, quest_dialogue_is_idle);
    assert!(labels.iter().any(|label| label == "WhitneyYouMeanieText"));
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert!(!shell.shell.session().state().badges.johto[2]);
        assert_eq!(quest_item_quantity(&shell, "TM_ATTRACT"), 0);
        quest_start_coord_script(&mut shell, "GoldenrodGym", "WhitneyCriesScript");
    }
    quest_settle(&mut app, true, quest_dialogue_is_idle);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert!(!shell.shell.session().state().flags.is_event_flag_set("EVENT_MADE_WHITNEY_CRY").unwrap());
        assert!(!shell.shell.session().state().badges.johto[2]);
        quest_move_beside_npc(&mut shell, "GoldenrodGym", "GoldenrodGymWhitneyScript");
        quest_talk(&mut shell, "GoldenrodGymWhitneyScript");
    }
    let labels = quest_settle(&mut app, true, quest_dialogue_is_idle);
    assert!(labels.iter().any(|label| label == "PlayerReceivedPlainBadgeText"));
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert!(shell.shell.session().state().badges.johto[2]);
        assert_eq!(quest_item_quantity(&shell, "TM_ATTRACT"), 1);
        assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_GOT_TM45_ATTRACT").unwrap());
        quest_talk(&mut shell, "GoldenrodGymWhitneyScript");
    }
    let labels = quest_settle(&mut app, true, quest_dialogue_is_idle);
    assert!(labels.iter().any(|label| label == "WhitneyGoodCryText"));
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    assert_eq!(quest_item_quantity(&shell, "TM_ATTRACT"), 1);
    quest_assert_save_round_trip(&mut shell, "whitney-awards");
    assert!(shell.shell.session().state().badges.johto[2]);
}

#[test]
fn immediate_johto_gym_rewards_finish_once_and_survive_save_load() {
    for (map, trainer, badge, tm) in [
        ("VioletGym", "FALKNER", 0, "TM_MUD_SLAP"),
        ("AzaleaGym", "BUGSY", 1, "TM_FURY_CUTTER"),
        ("EcruteakGym", "MORTY", 3, "TM_SHADOW_BALL"),
        ("CianwoodGym", "CHUCK", 5, "TM_DYNAMICPUNCH"),
        ("OlivineGym", "JASMINE", 4, "TM_IRON_TAIL"),
        ("MahoganyGym", "PRYCE", 6, "TM_ICY_WIND"),
    ] {
        eprintln!("checking {trainer} reward continuation");
        let mut shell = progression_shell_on_map_for_test(map);
        // These fixtures begin with the leader available and the battle won;
        // prerequisite quests and full combat are outside this reward fixture.
        for flag in ["EVENT_OLIVINE_GYM_JASMINE"] {
            shell.shell.session_mut().state_mut().flags.set_event_flag(flag, false).unwrap();
        }
        let key = shell.shell.scripted_trainer_battle_keys().into_iter()
            .find(|key| key.map_name == map && key.trainer_class == trainer).unwrap();
        shell.shell.start_scripted_trainer_battle(&key.map_name, &key.source_script, key.startbattle_command_index).unwrap();
        {
            let state = shell.shell.session_mut().state_mut();
            let crate::core::state::BattleMemory::Trainer { enemy_pokemon, enemy_party, .. } = &mut state.battle
                else { panic!("{trainer} requires trainer battle"); };
            state.battle_rewarded_enemy_party_indices = (0..enemy_party.len()).collect();
            for pokemon in enemy_party { pokemon.hp = 0; }
            enemy_pokemon.hp = 0;
        }
        shell.battle_message_scene = Some(Box::new(shell.shell.snapshot().unwrap()));
        complete_visible_scripted_trainer_battle(&mut shell, &key.map_name, &key.source_script, true, false).unwrap();
        let mut app = menu_render_test_app(shell);
        quest_settle(&mut app, true, |shell| quest_dialogue_is_idle(shell)
            && shell.battle_messages.is_empty() && !shell.shell.has_active_battle());
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            assert!(shell.shell.session().state().badges.johto[badge], "{trainer} badge");
            assert_eq!(quest_item_quantity(&shell, tm), 1, "{trainer} TM");
            quest_move_beside_npc(&mut shell, map, &key.source_script);
            quest_talk(&mut shell, &key.source_script);
        }
        quest_settle(&mut app, true, quest_dialogue_is_idle);
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, tm), 1, "{trainer} repeat must not duplicate TM");
        quest_assert_save_round_trip(&mut shell, trainer);
        assert!(shell.shell.session().state().badges.johto[badge]);
    }
}

fn quest_dragon_quiz_answer(app: &mut App, question: u8, option: usize) {
    let menu_id = format!("DragonShrineQuestion{question}_MenuHeader");
    quest_settle(app, false, |shell| {
        shell.shell.snapshot().unwrap().ui.menu.as_ref()
            .is_some_and(|menu| menu.menu_id == menu_id)
    });
    app.update(); // Publish the menu reached by the preceding input frame.
    if question == 1 {
        save_live_pc_dialog_for_test(app.world_mut(), "dragon-quiz-menu.png");
    }
    {
        let shell = app.world().resource::<BevyRuntimeShell>();
        let menu = shell.shell.snapshot().unwrap().ui.menu.unwrap();
        assert!(visible_runtime_menu_disables_b(shell, &menu).unwrap(), "quiz B flag missing: {:?}", menu);
    }
    // Use the visible cursor and ordinary input, including the authored B lock.
    let before_b = app.world().resource::<BevyRuntimeShell>().active_script_cursor.clone();
    press_key_for_runtime_hotkey_app(app, KeyCode::KeyX);
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(shell.shell.snapshot().unwrap().ui.menu.as_ref().unwrap().menu_id, menu_id);
    assert_eq!(shell.active_script_cursor, before_b, "B must not answer the quiz");
    for _ in 0..option { press_key_for_runtime_hotkey_app(app, KeyCode::ArrowDown); }
    press_key_for_runtime_hotkey_app(app, KeyCode::KeyZ);
}

#[test]
fn clair_delays_badge_until_quiz_then_gives_tm_and_dratini_once() {
    for wrong_answer in [false, true] {
        let mut shell = progression_shell_on_map_for_test("BlackthornGym1F");
        shell.shell.session_mut().state_mut().try_advance_frame().unwrap();
        shell.shell.session_mut().overworld_mut().frame = 1;
        shell.shell.session_mut().state_mut().player_name = "CHRIS".into();
        let key = shell.shell.scripted_trainer_battle_keys().into_iter()
            .find(|key| key.map_name == "BlackthornGym1F" && key.trainer_class == "CLAIR").unwrap();
        shell.shell.start_scripted_trainer_battle(&key.map_name, &key.source_script, key.startbattle_command_index).unwrap();
        {
            // Begin at the final battle result; gym traversal/combat remain separate.
            let state = shell.shell.session_mut().state_mut();
            let crate::core::state::BattleMemory::Trainer { enemy_pokemon, enemy_party, .. } = &mut state.battle
                else { panic!("Clair requires trainer battle"); };
            state.battle_rewarded_enemy_party_indices = (0..enemy_party.len()).collect();
            for pokemon in enemy_party { pokemon.hp = 0; }
            enemy_pokemon.hp = 0;
        }
        shell.battle_message_scene = Some(Box::new(shell.shell.snapshot().unwrap()));
        complete_visible_scripted_trainer_battle(&mut shell, &key.map_name, &key.source_script, true, false).unwrap();
        let mut app = menu_render_test_app(shell);
        quest_settle(&mut app, false, |shell| quest_dialogue_is_idle(shell)
            && shell.battle_messages.is_empty() && !shell.shell.has_active_battle());
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_BEAT_CLAIR").unwrap());
            assert!(!shell.shell.session().state().badges.johto[7]);
            assert_eq!(quest_item_quantity(&shell, "TM_DRAGONBREATH"), 0);
            assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_BLACKTHORN_CITY_GRAMPS_BLOCKS_DRAGONS_DEN").unwrap());
            quest_move_beside_npc(&mut shell, "BlackthornGym1F", "BlackthornGymClairScript");
            quest_talk(&mut shell, "BlackthornGymClairScript");
        }
        let labels = quest_settle(&mut app, false, quest_dialogue_is_idle);
        assert!(labels.iter().any(|label| label == "ClairText_TooMuchToExpect"));
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            assert!(!shell.shell.session().state().badges.johto[7]);
            let runtime = shell.shell.runtime().clone();
            let (state, overworld) = shell.shell.session_mut().state_and_overworld_mut();
            runtime.data().transition_overworld_session(state, overworld, "DragonShrine",
                TilePosition::new(4, 9), crate::core::systems::map_context::SpawnMemoryUpdate::Preserve,
                &runtime.music_ids()).unwrap();
            reset_visible_navigation_state(&mut shell);
            mark_runtime_snapshot_dirty(&mut shell);
            arm_visible_active_script_cursor(&mut shell, "DragonShrineTakeTestScript", 0);
            execute_visible_active_script_step(&mut shell).unwrap();
        }
        if wrong_answer { quest_dragon_quiz_answer(&mut app, 1, 1); }
        for (question, option) in [(1, 0), (2, 0), (3, 1), (4, 0), (5, 1)] {
            quest_dragon_quiz_answer(&mut app, question, option);
        }
        let labels = quest_settle(&mut app, false, quest_dialogue_is_idle);
        assert!(labels.iter().any(|label| label == "DragonShrinePlayerReceivedRisingBadgeText"));
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            assert!(shell.shell.session().state().badges.johto[7]);
            assert_eq!(shell.shell.session().state().flags.is_event_flag_set("EVENT_ANSWERED_DRAGON_MASTER_QUIZ_WRONG").unwrap(), wrong_answer);
            assert_eq!(quest_item_quantity(&shell, "TM_DRAGONBREATH"), 0);
            assert_eq!(shell.shell.session().state().scenes.map_scenes["DragonsDenB1F"], "SCENE_DRAGONSDENB1F_CLAIR_GIVES_TM");
            quest_talk(&mut shell, "DragonShrineElder1Script");
        }
        let labels = quest_settle(&mut app, false, quest_dialogue_is_idle);
        assert!(labels.iter().any(|label| label == "DragonShrineComeAgainText"));
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            assert!(!shell.shell.session().state().flags.is_event_flag_set("EVENT_GOT_DRATINI").unwrap());
            quest_start_coord_script(&mut shell, "DragonsDenB1F", "DragonsDenB1F_ClairScene");
        }
        quest_settle(&mut app, false, quest_dialogue_is_idle);
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            assert_eq!(quest_item_quantity(&shell, "TM_DRAGONBREATH"), 1);
            assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_GOT_TM24_DRAGONBREATH").unwrap());
            assert_eq!(shell.shell.session().state().scenes.map_scenes["DragonsDenB1F"], "SCENE_DRAGONSDENB1F_NOOP");
            quest_move_beside_npc(&mut shell, "DragonShrine", "DragonShrineElder1Script");
            let runtime = shell.shell.runtime().clone();
            let (state, overworld) = shell.shell.session_mut().state_and_overworld_mut();
            runtime.data().apply_map_setup_callbacks(state, overworld, "DragonShrine", "MAPSETUP_DOOR").unwrap();
            quest_talk(&mut shell, "DragonShrineElder1Script");
        }
        quest_settle(&mut app, false, quest_dialogue_is_idle);
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_GOT_DRATINI").unwrap());
            let dratini = shell.shell.session().state().storage.party.pokemon.iter().flatten()
                .find(|pokemon| pokemon.species.id == "DRATINI").unwrap();
            assert_eq!(dratini.level, 15);
            assert_eq!(dratini.moves.iter().any(|move_| move_.name == "EXTREMESPEED"), !wrong_answer);
            quest_talk(&mut shell, "DragonShrineElder1Script");
        }
        quest_settle(&mut app, false, quest_dialogue_is_idle);
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            assert_eq!(shell.shell.session().state().storage.party.pokemon.iter().flatten()
                .filter(|pokemon| pokemon.species.id == "DRATINI").count(), 1);
            quest_move_beside_npc(&mut shell, "BlackthornGym1F", "BlackthornGymClairScript");
            quest_talk(&mut shell, "BlackthornGymClairScript");
        }
        quest_settle(&mut app, false, quest_dialogue_is_idle);
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, "TM_DRAGONBREATH"), 1);
        quest_assert_save_round_trip(&mut shell, if wrong_answer { "clair-wrong" } else { "clair-perfect" });
        assert!(shell.shell.session().state().badges.johto[7]);
    }
}

#[test]
fn dragon_shrine_full_party_keeps_dratini_available_for_retry() {
    let mut shell = progression_shell_on_map_for_test("DragonShrine");
    shell.shell.session_mut().state_mut().player_name = "CHRIS".into();
    // Post-quiz visit: the automatic entrance scene has already completed.
    {
        let state = shell.shell.session_mut().state_mut();
        state.scenes.map_scenes.insert("DragonShrine".into(), "SCENE_DRAGONSHRINE_NOOP".into());
        state.scenes.map_scene_indices.insert("DragonShrine".into(), 1);
    }
    quest_move_beside_npc(&mut shell, "DragonShrine", "DragonShrineElder1Script");
    for _ in 1..6 {
        shell.shell.add_party_pokemon("PIDGEY", 5, None, None, "CHRIS", 1, Dv::default()).unwrap();
    }
    quest_move_beside_npc(&mut shell, "DragonShrine", "DragonShrineElder1Script");
    quest_talk(&mut shell, "DragonShrineElder1Script");
    let mut app = menu_render_test_app(shell);
    let labels = quest_settle(&mut app, false, quest_dialogue_is_idle);
    assert!(labels.iter().any(|label| label == "DragonShrinePartyFullText"));
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert!(!shell.shell.session().state().flags.is_event_flag_set("EVENT_GOT_DRATINI").unwrap());
        shell.shell.deposit_party_pokemon_to_current_box(5).unwrap();
        quest_talk(&mut shell, "DragonShrineElder1Script");
    }
    quest_settle(&mut app, false, quest_dialogue_is_idle);
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_GOT_DRATINI").unwrap());
    assert_eq!(shell.shell.session().state().storage.party.pokemon.iter().flatten()
        .filter(|pokemon| pokemon.species.id == "DRATINI").count(), 1);
    quest_assert_save_round_trip(&mut shell, "dratini-full-party");
}

#[test]
fn map_entry_clears_temporary_events_but_reload_and_continue_preserve_them() {
    let mut shell = progression_shell_on_map_for_test("DragonShrine");
    let runtime = shell.shell.runtime().clone();
    for (setup, cleared) in [
        ("MAPSETUP_RELOADMAP", false), ("MAPSETUP_SUBMENU", false),
        ("MAPSETUP_CONTINUE", false), ("MAPSETUP_WARP", true),
        ("MAPSETUP_DOOR", true), ("MAPSETUP_CONNECTION", true),
        ("MAPSETUP_FALL", true), ("MAPSETUP_TRAIN", true),
        ("MAPSETUP_FLY", true), ("MAPSETUP_TELEPORT", true),
        ("MAPSETUP_BADWARP", true), ("MAPSETUP_LINKRETURN", true),
    ] {
        let (state, overworld) = shell.shell.session_mut().state_and_overworld_mut();
        for index in 1..=8 {
            state.flags.set_event_flag(&format!("EVENT_TEMPORARY_UNTIL_MAP_RELOAD_{index}"), true).unwrap();
        }
        state.flags.set_event_flag("EVENT_GOT_DRATINI", true).unwrap();
        state.flags.set_event_flag("EVENT_ANSWERED_DRAGON_MASTER_QUIZ_WRONG", true).unwrap();
        runtime.data().apply_map_setup_callbacks(state, overworld, "DragonShrine", setup).unwrap();
        for index in 1..=8 {
            assert_eq!(state.flags.is_event_flag_set(&format!("EVENT_TEMPORARY_UNTIL_MAP_RELOAD_{index}")).unwrap(), !cleared, "{setup}, flag {index}");
        }
        assert!(state.flags.is_event_flag_set("EVENT_GOT_DRATINI").unwrap());
        assert!(state.flags.is_event_flag_set("EVENT_ANSWERED_DRAGON_MASTER_QUIZ_WRONG").unwrap());
    }
}

fn quest_begin_script(shell: &mut BevyRuntimeShell, script: &str) {
    arm_visible_active_script_cursor(shell, script, 0);
    execute_visible_active_script_step(shell).unwrap();
    mark_runtime_snapshot_dirty(shell);
}

fn quest_finish_current_trainer(shell: &mut BevyRuntimeShell) {
    let map = shell.shell.session().overworld().map.name.clone();
    let source = {
        let state = shell.shell.session_mut().state_mut();
        let crate::core::state::BattleMemory::Trainer { source_script, enemy_pokemon, enemy_party, .. } = &mut state.battle
            else { panic!("expected actual scripted trainer battle"); };
        state.battle_rewarded_enemy_party_indices = (0..enemy_party.len()).collect();
        for pokemon in enemy_party { pokemon.hp = 0; }
        enemy_pokemon.hp = 0;
        source_script.clone()
    };
    shell.visible_battle_transition = None;
    shell.visible_battle_sliding_intro = None;
    shell.battle_entry_messages_remaining = 0;
    shell.battle_enemy_send_out_pending = false;
    shell.battle_player_send_out_pending = false;
    shell.battle_messages.clear();
    shell.battle_message_scenes.clear();
    shell.battle_text_reveal = None;
    shell.battle_message_scene = Some(Box::new(shell.shell.snapshot().unwrap()));
    complete_visible_scripted_trainer_battle(shell, &map, &source, true, false).unwrap();
}

#[test]
fn rocket_camera_runs_two_grunts_then_switch_disables_every_camera() {
    let mut shell = progression_shell_on_map_for_test("TeamRocketBaseB1F");
    quest_start_coord_script(&mut shell, "TeamRocketBaseB1F", "SecurityCamera1a");
    let mut app = menu_render_test_app(shell);
    for trainer in ["GRUNTM_20", "GRUNTM_21"] {
        quest_settle(&mut app, false, |shell| shell.shell.has_active_battle());
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        let crate::core::state::BattleMemory::Trainer { trainer_id, .. } = &shell.shell.session().state().battle
            else { panic!("camera must summon trainer"); };
        assert_eq!(trainer_id, trainer);
        assert!(!shell.shell.session().state().flags.is_event_flag_set("EVENT_SECURITY_CAMERA_1").unwrap());
        quest_finish_current_trainer(&mut shell);
    }
    quest_settle(&mut app, false, |shell| quest_dialogue_is_idle(shell) && !shell.shell.has_active_battle() && shell.battle_messages.is_empty());
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_SECURITY_CAMERA_1").unwrap());
        quest_begin_script(&mut shell, "TeamRocketBaseB1FSecretSwitch");
    }
    quest_settle(&mut app, false, quest_dialogue_is_idle);
    for camera in ["SecurityCamera1a", "SecurityCamera1b", "SecurityCamera2a", "SecurityCamera2b", "SecurityCamera3a", "SecurityCamera3b", "SecurityCamera4", "SecurityCamera5"] {
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            quest_start_coord_script(&mut shell, "TeamRocketBaseB1F", camera);
        }
        quest_settle(&mut app, false, quest_dialogue_is_idle);
        assert!(!app.world().resource::<BevyRuntimeShell>().shell.has_active_battle(), "disabled {camera}");
    }
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    for index in 1..=5 { assert!(shell.shell.session().state().flags.is_event_flag_set(&format!("EVENT_SECURITY_CAMERA_{index}")).unwrap()); }
    quest_assert_save_round_trip(&mut shell, "rocket-cameras");
}

#[test]
fn rocket_passwords_gate_office_and_transmitter_doors_and_survive_reload() {
    let mut shell = progression_shell_on_map_for_test("TeamRocketBaseB3F");
    quest_begin_script(&mut shell, ".Script@TeamRocketBaseB3FLockedDoor");
    let mut app = menu_render_test_app(shell);
    let labels = quest_settle(&mut app, false, quest_dialogue_is_idle);
    assert!(labels.iter().any(|label| label == "TeamRocketBaseB3FLockedDoorNeedsPasswordText"));
    for (script, defeated, password) in [
        ("SlowpokeTailGrunt", "EVENT_BEAT_ROCKET_GRUNTF_5", "EVENT_LEARNED_SLOWPOKETAIL"),
        ("RaticateTailGrunt", "EVENT_BEAT_ROCKET_GRUNTM_28", "EVENT_LEARNED_RATICATE_TAIL"),
    ] {
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            shell.shell.session_mut().state_mut().flags.set_event_flag(defeated, true).unwrap();
            quest_move_beside_npc(&mut shell, "TeamRocketBaseB3F", script);
            quest_talk(&mut shell, script);
        }
        quest_settle(&mut app, false, quest_dialogue_is_idle);
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            assert!(shell.shell.session().state().flags.is_event_flag_set(password).unwrap());
            quest_begin_script(&mut shell, ".Script@TeamRocketBaseB3FLockedDoor");
        }
        quest_settle(&mut app, false, quest_dialogue_is_idle);
        assert_eq!(app.world().resource::<BevyRuntimeShell>().shell.session().state().flags.is_event_flag_set("EVENT_OPENED_DOOR_TO_GIOVANNIS_OFFICE").unwrap(), script == "RaticateTailGrunt");
    }
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        quest_move_beside_npc(&mut shell, "TeamRocketBaseB2F", "RocketElectrode1");
        quest_begin_script(&mut shell, ".Script@TeamRocketBaseB2FLockedDoor");
    }
    let labels = quest_settle(&mut app, false, quest_dialogue_is_idle);
    assert!(labels.iter().any(|label| label == "RocketBaseDoorNoPasswordText"));
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert!(!shell.shell.session().state().flags.is_event_flag_set("EVENT_OPENED_DOOR_TO_ROCKET_HIDEOUT_TRANSMITTER").unwrap());
        quest_move_beside_npc(&mut shell, "TeamRocketBaseB3F", "RocketBaseMurkrow");
        quest_talk(&mut shell, "RocketBaseMurkrow");
    }
    quest_settle(&mut app, false, quest_dialogue_is_idle);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_LEARNED_HAIL_GIOVANNI").unwrap());
        quest_move_beside_npc(&mut shell, "TeamRocketBaseB2F", "RocketElectrode1");
        quest_begin_script(&mut shell, ".Script@TeamRocketBaseB2FLockedDoor");
    }
    quest_settle(&mut app, false, quest_dialogue_is_idle);
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_OPENED_DOOR_TO_ROCKET_HIDEOUT_TRANSMITTER").unwrap());
    assert_eq!(shell.shell.session().overworld().map.metatile_at(7, 6), Some(0x07));
    quest_assert_save_round_trip(&mut shell, "rocket-passwords");
    let state = shell.shell.session().state();
    assert_eq!(state.map_block_overrides["TeamRocketBaseB3F"].get(&(5, 4)), Some(&0x07));
    assert_eq!(state.map_block_overrides["TeamRocketBaseB2F"].get(&(7, 6)), Some(&0x07));
}

#[test]
fn rocket_electrodes_clear_in_pairs_and_award_whirlpool_after_the_third() {
    let mut shell = progression_shell_on_map_for_test("TeamRocketBaseB2F");
    {
        // Stage the post-executive scene; the three wild battles and aftermath run normally.
        let state = shell.shell.session_mut().state_mut();
        state.scenes.map_scenes.insert("TeamRocketBaseB2F".into(), "SCENE_TEAMROCKETBASEB2F_ELECTRODES".into());
        state.scenes.map_scene_indices.insert("TeamRocketBaseB2F".into(), 2);
        state.flags.set_engine_flag("ENGINE_ROCKET_SIGNAL_ON_CH20", true).unwrap();
    }
    let mut app = menu_render_test_app(shell);
    for index in 1..=3 {
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            let script = format!("RocketElectrode{index}");
            quest_move_beside_npc(&mut shell, "TeamRocketBaseB2F", &script);
            quest_talk(&mut shell, &script);
        }
        quest_settle(&mut app, false, |shell| shell.shell.has_active_battle());
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            assert_eq!(shell.shell.snapshot().unwrap().battle.unwrap().enemy_pokemon.species.id, "ELECTRODE");
            quest_finish_static_wild_final_turn(&mut shell);
        }
        quest_settle(&mut app, false, |shell| quest_dialogue_is_idle(shell) && !shell.shell.has_active_battle() && shell.battle_messages.is_empty());
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(shell.shell.session().state().flags.is_event_flag_set(&format!("EVENT_TEAM_ROCKET_BASE_B2F_ELECTRODE_{index}")).unwrap());
        for partner in [index, index + 3] {
            assert!(shell.shell.session().overworld().hidden_object_identifiers.contains(
                &format!("TEAMROCKETBASEB2F_ELECTRODE{partner}")), "Electrode {partner} remains visible");
        }
        assert_eq!(quest_item_quantity(shell, "HM_WHIRLPOOL"), u16::from(index == 3));
        assert_eq!(shell.shell.session().state().flags.is_event_flag_set("EVENT_CLEARED_ROCKET_HIDEOUT").unwrap(), index == 3);
    }
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert!(!shell.shell.session().state().flags.is_engine_flag_set("ENGINE_ROCKET_SIGNAL_ON_CH20").unwrap());
        for flag in ["EVENT_GOT_HM06_WHIRLPOOL", "EVENT_ROUTE_43_GATE_ROCKETS", "EVENT_MAHOGANY_TOWN_POKEFAN_M_BLOCKS_GYM", "EVENT_TURNED_OFF_SECURITY_CAMERAS"] {
            assert!(shell.shell.session().state().flags.is_event_flag_set(flag).unwrap(), "{flag}");
        }
        assert_eq!(shell.shell.session().state().scenes.map_scenes["TeamRocketBaseB2F"], "SCENE_TEAMROCKETBASEB2F_NOOP");
        quest_begin_script(&mut shell, "TeamRocketBaseB2FTransmitterScript");
    }
    let labels = quest_settle(&mut app, false, quest_dialogue_is_idle);
    assert!(labels.iter().any(|label| label == "RocketBaseB2FDeactivateTransmitterText"));
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    quest_assert_save_round_trip(&mut shell, "rocket-generators");
    assert_eq!(quest_item_quantity(&shell, "HM_WHIRLPOOL"), 1);
}


#[test]
fn rocket_floor_traps_start_the_authored_species_and_do_not_repeat() {
    let shell = progression_shell_on_map_for_test("TeamRocketBaseB1F");
    let mut app = menu_render_test_app(shell);
    for (index, species, level) in [(1, "KOFFING", 21), (2, "VOLTORB", 23), (3, "GEODUDE", 21)] {
        let script = format!("ExplodingTrap{index}");
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            quest_start_coord_script(&mut shell, "TeamRocketBaseB1F", &script);
        }
        quest_settle(&mut app, false, |shell| shell.shell.has_active_battle());
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            let battle = shell.shell.snapshot().unwrap().battle.unwrap();
            assert_eq!(battle.battle_type, "BATTLETYPE_TRAP");
            assert_eq!(battle.enemy_pokemon.species.id, species);
            assert_eq!(battle.enemy_pokemon.level, level);
            quest_finish_static_wild_final_turn(&mut shell);
        }
        quest_settle(&mut app, false, |shell| quest_dialogue_is_idle(shell) && !shell.shell.has_active_battle() && shell.battle_messages.is_empty());
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            assert!(shell.shell.session().state().flags.is_event_flag_set(&format!("EVENT_EXPLODING_TRAP_{index}")).unwrap());
            quest_start_coord_script(&mut shell, "TeamRocketBaseB1F", &script);
        }
        quest_settle(&mut app, false, quest_dialogue_is_idle);
        assert!(!app.world().resource::<BevyRuntimeShell>().shell.has_active_battle());
    }
    quest_assert_save_round_trip(&mut app.world_mut().resource_mut::<BevyRuntimeShell>(), "rocket-traps");
}

#[test]
fn rocket_executives_retreat_and_lance_hands_off_to_the_generators() {
    let mut shell = progression_shell_on_map_for_test("TeamRocketBaseB2F");
    shell.shell.session_mut().state_mut().storage.party.pokemon[0].as_mut().unwrap().hp = 1;
    quest_start_coord_script(&mut shell, "TeamRocketBaseB2F", "LanceHealsScript1");
    let mut app = menu_render_test_app(shell);
    quest_settle(&mut app, false, quest_dialogue_is_idle);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        let state = shell.shell.session().state();
        let player = state.storage.party.pokemon[0].as_ref().unwrap();
        assert_eq!(player.hp, player.max_hp);
        assert!(state.flags.is_event_flag_set("EVENT_LANCE_HEALED_YOU_IN_TEAM_ROCKET_BASE").unwrap());
        assert_eq!(state.scenes.map_scenes["TeamRocketBaseB2F"], "SCENE_TEAMROCKETBASEB2F_ROCKET_BOSS");
        quest_start_coord_script(&mut shell, "TeamRocketBaseB3F", "RocketBaseRival");
    }
    quest_settle(&mut app, false, quest_dialogue_is_idle);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(shell.shell.session().state().scenes.map_scenes["TeamRocketBaseB3F"], "SCENE_TEAMROCKETBASEB3F_ROCKET_BOSS");
        assert!(shell.shell.session().overworld().hidden_object_identifiers.contains("TEAMROCKETBASEB3F_RIVAL"));
        quest_start_coord_script(&mut shell, "TeamRocketBaseB3F", "RocketBaseBossLeft");
    }
    quest_settle(&mut app, false, |shell| shell.shell.has_active_battle());
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        let crate::core::state::BattleMemory::Trainer { trainer_id, .. } = &shell.shell.session().state().battle else { panic!("executive battle"); };
        assert_eq!(trainer_id, "EXECUTIVEM_4");
        quest_finish_current_trainer(&mut shell);
    }
    quest_settle(&mut app, false, |shell| quest_dialogue_is_idle(shell) && !shell.shell.has_active_battle() && shell.battle_messages.is_empty());
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_BEAT_ROCKET_EXECUTIVEM_4").unwrap());
        assert!(shell.shell.session().overworld().hidden_object_identifiers.contains("TEAMROCKETBASEB3F_ROCKET1"));
        assert_eq!(shell.shell.session().state().scenes.map_scenes["TeamRocketBaseB3F"], "SCENE_TEAMROCKETBASEB3F_NOOP");
        quest_start_coord_script(&mut shell, "TeamRocketBaseB2F", "RocketBaseBossFLeft");
    }
    quest_settle(&mut app, false, |shell| shell.shell.has_active_battle());
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        let crate::core::state::BattleMemory::Trainer { trainer_id, .. } = &shell.shell.session().state().battle else { panic!("executive battle"); };
        assert_eq!(trainer_id, "EXECUTIVEF_2");
        quest_finish_current_trainer(&mut shell);
    }
    quest_settle(&mut app, false, |shell| quest_dialogue_is_idle(shell) && !shell.shell.has_active_battle() && shell.battle_messages.is_empty());
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_BEAT_ROCKET_EXECUTIVEF_2").unwrap());
    assert_eq!(shell.shell.session().state().scenes.map_scenes["TeamRocketBaseB2F"], "SCENE_TEAMROCKETBASEB2F_ELECTRODES");
    for object in ["TEAMROCKETBASEB2F_ROCKET1", "TEAMROCKETBASEB2F_ROCKET_GIRL", "TEAMROCKETBASEB2F_DRAGON", "TEAMROCKETBASEB2F_LANCE"] {
        assert!(shell.shell.session().overworld().hidden_object_identifiers.contains(object), "{object}");
    }
    assert_eq!(quest_item_quantity(&shell, "HM_WHIRLPOOL"), 0);
    assert!(!shell.shell.session().state().flags.is_event_flag_set("EVENT_CLEARED_ROCKET_HIDEOUT").unwrap());
    quest_assert_save_round_trip(&mut shell, "rocket-executives");
}

#[test]
fn radio_tower_keys_unlock_the_shutter_and_rewards_do_not_repeat() {
    let mut shell = progression_shell_on_map_for_test("RadioTower3F");
    // Enter the takeover phase; new-game initialization leaves this shutter open.
    for flag in ["EVENT_USED_THE_CARD_KEY_IN_THE_RADIO_TOWER", "EVENT_RADIO_TOWER_ROCKET_TAKEOVER", "EVENT_BEAT_ROCKET_EXECUTIVEM_3", "EVENT_RECEIVED_CARD_KEY"] {
        shell.shell.session_mut().state_mut().flags.set_event_flag(flag, false).unwrap();
    }
    assert_eq!(quest_item_quantity(&shell, "CARD_KEY"), 0);
    quest_begin_script(&mut shell, "CardKeySlotScript");
    let mut app = menu_render_test_app(shell);
    quest_settle(&mut app, false, quest_dialogue_is_idle);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert!(!shell.shell.session().state().flags.is_event_flag_set("EVENT_USED_THE_CARD_KEY_IN_THE_RADIO_TOWER").unwrap());
        quest_start_coord_script(&mut shell, "RadioTower5F", "FakeDirectorScript");
    }
    quest_settle(&mut app, false, |shell| shell.shell.has_active_battle());
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        let crate::core::state::BattleMemory::Trainer { trainer_id, .. } = &shell.shell.session().state().battle else { panic!("fake director battle"); };
        assert_eq!(trainer_id, "EXECUTIVEM_3");
        quest_finish_current_trainer(&mut shell);
    }
    quest_settle(&mut app, false, |shell| quest_dialogue_is_idle(shell) && !shell.shell.has_active_battle() && shell.battle_messages.is_empty());
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, "BASEMENT_KEY"), 1);
        assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_BEAT_ROCKET_EXECUTIVEM_3").unwrap());
        quest_move_beside_npc(&mut shell, "RadioTower5F", "Director");
        quest_talk(&mut shell, "Director");
    }
    quest_settle(&mut app, false, quest_dialogue_is_idle);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, "BASEMENT_KEY"), 1);
        quest_move_beside_npc(&mut shell, "GoldenrodUndergroundWarehouse", "GoldenrodUndergroundWarehouseDirectorScript");
        quest_talk(&mut shell, "GoldenrodUndergroundWarehouseDirectorScript");
    }
    quest_settle(&mut app, false, quest_dialogue_is_idle);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, "CARD_KEY"), 1);
        assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_RECEIVED_CARD_KEY").unwrap());
        quest_talk(&mut shell, "GoldenrodUndergroundWarehouseDirectorScript");
    }
    quest_settle(&mut app, false, quest_dialogue_is_idle);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, "CARD_KEY"), 1);
        quest_move_beside_npc(&mut shell, "RadioTower3F", "RadioTower3FGymGuideScript");
        quest_begin_script(&mut shell, "CardKeySlotScript");
    }
    let labels = quest_settle(&mut app, false, quest_dialogue_is_idle);
    assert!(labels.iter().any(|label| label == "InsertedTheCardKeyText"));
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_USED_THE_CARD_KEY_IN_THE_RADIO_TOWER").unwrap());
        assert_eq!(shell.shell.session().overworld().map.metatile_at(7, 1), Some(0x2a));
        assert_eq!(shell.shell.session().overworld().map.metatile_at(7, 2), Some(0x01));
        quest_begin_script(&mut shell, "CardKeySlotScript");
    }
    quest_settle(&mut app, false, quest_dialogue_is_idle);
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    assert_eq!(quest_item_quantity(&shell, "CARD_KEY"), 1);
    quest_assert_save_round_trip(&mut shell, "radio-tower-keys");
    assert_eq!(shell.shell.session().state().map_block_overrides["RadioTower3F"].get(&(7, 1)), Some(&0x2a));
    assert_eq!(shell.shell.session().state().map_block_overrides["RadioTower3F"].get(&(7, 2)), Some(&0x01));
}

#[test]
fn radio_tower_final_executive_grants_clear_bell_and_restores_the_towns() {
    let mut shell = progression_shell_on_map_for_test("RadioTower5F");
    {
        let state = shell.shell.session_mut().state_mut();
        for flag in ["EVENT_CLEARED_RADIO_TOWER", "EVENT_RADIO_TOWER_ROCKET_TAKEOVER", "EVENT_GOT_CLEAR_BELL", "EVENT_TEAM_ROCKET_DISBANDED"] {
            state.flags.set_event_flag(flag, false).unwrap();
        }
        for flag in ["ENGINE_ROCKETS_IN_RADIO_TOWER", "ENGINE_ROCKETS_IN_MAHOGANY"] {
            state.flags.set_engine_flag(flag, true).unwrap();
        }
    }
    quest_start_coord_script(&mut shell, "RadioTower5F", "RadioTower5FRocketBossScript");
    let mut app = menu_render_test_app(shell);
    quest_settle(&mut app, false, |shell| shell.shell.has_active_battle());
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        let crate::core::state::BattleMemory::Trainer { trainer_id, .. } = &shell.shell.session().state().battle else { panic!("final executive battle"); };
        assert_eq!(trainer_id, "EXECUTIVEM_1");
        assert_eq!(quest_item_quantity(&shell, "CLEAR_BELL"), 0);
        quest_finish_current_trainer(&mut shell);
    }
    quest_settle(&mut app, false, |shell| quest_dialogue_is_idle(shell) && !shell.shell.has_active_battle() && shell.battle_messages.is_empty());
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    assert_eq!(quest_item_quantity(&shell, "CLEAR_BELL"), 1);
    let state = shell.shell.session().state();
    for flag in ["EVENT_BEAT_ROCKET_EXECUTIVEM_1", "EVENT_CLEARED_RADIO_TOWER", "EVENT_GOT_CLEAR_BELL", "EVENT_TEAM_ROCKET_DISBANDED", "EVENT_RADIO_TOWER_ROCKET_TAKEOVER", "EVENT_GOLDENROD_CITY_ROCKET_TAKEOVER", "EVENT_BLACKTHORN_CITY_SUPER_NERD_BLOCKS_GYM"] {
        assert!(state.flags.is_event_flag_set(flag).unwrap(), "{flag}");
    }
    for flag in ["EVENT_MAHOGANY_MART_OWNERS", "EVENT_GOLDENROD_CITY_CIVILIANS", "EVENT_RADIO_TOWER_CIVILIANS_AFTER", "EVENT_BLACKTHORN_CITY_SUPER_NERD_DOES_NOT_BLOCK_GYM"] {
        assert!(!state.flags.is_event_flag_set(flag).unwrap(), "{flag}");
    }
    for flag in ["ENGINE_ROCKETS_IN_RADIO_TOWER", "ENGINE_ROCKETS_IN_MAHOGANY"] {
        assert!(!state.flags.is_engine_flag_set(flag).unwrap(), "{flag}");
    }
    assert_eq!(state.scenes.map_scenes["RadioTower5F"], "SCENE_RADIOTOWER5F_NOOP");
    assert_eq!(state.scenes.map_scenes["EcruteakTinTowerEntrance"], "SCENE_ECRUTEAKTINTOWERENTRANCE_SAGE_BLOCKS");
    assert!(shell.shell.session().overworld().hidden_object_identifiers.contains("RADIOTOWER5F_DIRECTOR"));
    quest_assert_save_round_trip(&mut shell, "radio-tower-cleared");
    assert_eq!(quest_item_quantity(&shell, "CLEAR_BELL"), 1);
}

#[test]
fn underground_switch_maze_preserves_door_history_and_resets_from_warehouse() {
    let mut shell = progression_shell_on_map_for_test("GoldenrodUndergroundWarehouse");
    quest_move_beside_npc(&mut shell, "GoldenrodUndergroundSwitchRoomEntrances", "GoldenrodUndergroundSwitchRoomEntrancesTeacherScript");
    let mut app = menu_render_test_app(shell);
    let toggle = |app: &mut App, script: &str, yes: bool| {
        quest_begin_script(&mut app.world_mut().resource_mut::<BevyRuntimeShell>(), script);
        quest_settle(app, yes, quest_dialogue_is_idle);
    };
    toggle(&mut app, "Switch1Script", false);
    assert!(!app.world().resource::<BevyRuntimeShell>().shell.session().state().flags.is_event_flag_set("EVENT_SWITCH_1").unwrap());
    // The wrong order reaches the same total but leaves the middle corridor closed.
    for index in [1, 2, 3] { toggle(&mut app, &format!("Switch{index}Script"), true); }
    {
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(shell.shell.session().state().script_runtime.memory["wUndergroundSwitchPositions"], "6");
        assert!(!shell.shell.session().state().flags.is_event_flag_set("EVENT_DOOR_5_OPEN").unwrap());
    }
    toggle(&mut app, "EmergencySwitchScript", true);
    assert!(app.world().resource::<BevyRuntimeShell>().shell.session().state().flags.is_event_flag_set("EVENT_DOOR_5_OPEN").unwrap());
    toggle(&mut app, "EmergencySwitchScript", true);
    {
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(shell.shell.session().state().script_runtime.memory["wUndergroundSwitchPositions"], "0");
        for index in 1..=11 { assert!(!shell.shell.session().state().flags.is_event_flag_set(&format!("EVENT_DOOR_{index}_OPEN")).unwrap()); }
    }
    for index in [3, 2, 1] { toggle(&mut app, &format!("Switch{index}Script"), true); }
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        let state = shell.shell.session().state();
        assert_eq!(state.script_runtime.memory["wUndergroundSwitchPositions"], "6");
        for index in 1..=11 {
            assert_eq!(state.flags.is_event_flag_set(&format!("EVENT_DOOR_{index}_OPEN")).unwrap(), [3, 5, 6, 8, 9, 11].contains(&index), "door {index}");
        }
        assert_eq!(shell.shell.session().overworld().map.metatile_at(5, 5), Some(0x2d));
        assert_eq!(shell.shell.session().overworld().map.metatile_at(9, 5), Some(0x2a));
        assert_eq!(shell.shell.session().overworld().map.metatile_at(9, 6), Some(0x2d));
        quest_assert_save_round_trip(&mut shell, "underground-switches");
        assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_DOOR_5_OPEN").unwrap());
        quest_move_beside_npc(&mut shell, "GoldenrodUndergroundWarehouse", "GoldenrodUndergroundWarehouseDirectorScript");
        assert_eq!(shell.shell.session().state().script_runtime.memory["wUndergroundSwitchPositions"], "0");
        quest_move_beside_npc(&mut shell, "GoldenrodUndergroundSwitchRoomEntrances", "GoldenrodUndergroundSwitchRoomEntrancesTeacherScript");
        assert_eq!(shell.shell.session().overworld().map.metatile_at(5, 5), Some(0x3e));
        assert_eq!(shell.shell.session().overworld().map.metatile_at(9, 5), Some(0x3f));
        assert_eq!(shell.shell.session().overworld().map.metatile_at(9, 6), Some(0x3d));
    }
}


#[test]
fn map_setup_rebuilds_transient_tiles_before_applying_persistent_door_flags() {
    let mut shell = progression_shell_on_map_for_test("RadioTower3F");
    let runtime = shell.shell.runtime().clone();
    let base = runtime.data().overworld_map("RadioTower3F").unwrap().metatile_at(7, 1).unwrap();
    assert_ne!(base, 0x2a);
    for setup in ["MAPSETUP_WARP", "MAPSETUP_RELOADMAP", "MAPSETUP_CONTINUE", "MAPSETUP_SUBMENU", "MAPSETUP_CONNECTION"] {
        let (state, overworld) = shell.shell.session_mut().state_and_overworld_mut();
        state.flags.set_event_flag("EVENT_USED_THE_CARD_KEY_IN_THE_RADIO_TOWER", true).unwrap();
        runtime.data().apply_map_setup_callbacks(state, overworld, "RadioTower3F", setup).unwrap();
        assert_eq!(overworld.map.metatile_at(7, 1), Some(0x2a), "{setup}: persistent open flag");
        state.flags.set_event_flag("EVENT_USED_THE_CARD_KEY_IN_THE_RADIO_TOWER", false).unwrap();
        runtime.data().apply_map_setup_callbacks(state, overworld, "RadioTower3F", setup).unwrap();
        assert_eq!(overworld.map.metatile_at(7, 1), Some(base), "{setup}: stale changeblock must not survive reload");
        assert!(!state.map_block_overrides.contains_key("RadioTower3F"));
    }
}

#[test]
fn slowpoke_tail_offer_handles_both_answers_without_taking_money_or_granting_an_item() {
    let mut shell = progression_shell_on_map_for_test("Route32");
    shell.shell.session_mut().state_mut().money = 999_999;
    quest_start_coord_script(&mut shell, "Route32", "Route32WannaBuyASlowpokeTailScript");
    let mut app = menu_render_test_app(shell);
    let labels = quest_settle(&mut app, false, quest_dialogue_is_idle);
    assert!(labels.iter().any(|label| label == "Text_RefusedToBuySlowpokeTail"));
    for yes in [true, false] {
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            quest_move_beside_npc(&mut shell, "Route32", "SlowpokeTailSalesmanScript");
            quest_talk(&mut shell, "SlowpokeTailSalesmanScript");
        }
        let labels = quest_settle(&mut app, yes, quest_dialogue_is_idle);
        assert!(labels.iter().any(|label| label == if yes { "Text_ThoughtKidsWereLoaded" } else { "Text_RefusedToBuySlowpokeTail" }));
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(shell.shell.session().state().money, 999_999);
        assert_eq!(quest_item_quantity(shell, "SLOWPOKETAIL"), 0);
        assert_eq!(shell.shell.session().state().scenes.map_scenes["Route32"], "SCENE_ROUTE32_NOOP");
    }
    quest_assert_save_round_trip(&mut app.world_mut().resource_mut::<BevyRuntimeShell>(), "slowpoke-tail-offer");
}

#[test]
fn borrowed_bicycle_mileage_triggers_the_visible_shop_call_once() {
    let mut shell = progression_shell_on_map_for_test("GoldenrodBikeShop");
    quest_move_beside_npc(&mut shell, "GoldenrodBikeShop", "GoldenrodBikeShopClerkScript");
    quest_talk(&mut shell, "GoldenrodBikeShopClerkScript");
    let mut app = menu_render_test_app(shell);
    quest_settle(&mut app, true, quest_dialogue_is_idle);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, "BICYCLE"), 1);
        assert!(shell.shell.session().state().flags.is_engine_flag_set("ENGINE_BIKE_SHOP_CALL_ENABLED").unwrap());
        let runtime = shell.shell.runtime().clone();
        let (state, overworld) = shell.shell.session_mut().state_and_overworld_mut();
        runtime.data().transition_overworld_session(state, overworld, "NewBarkTown", TilePosition::new(13, 6), crate::core::systems::map_context::SpawnMemoryUpdate::Preserve, &runtime.music_ids()).unwrap();
        // Stage mileage immediately before the threshold, then ride through real input.
        state.step_events.bike_step_count = 1023;
        shell.shell.use_bag_bicycle_in_field("BICYCLE").unwrap();
        assert_eq!(shell.shell.session().overworld().player.mode, MovementMode::Bike);
        reset_visible_navigation_state(&mut shell);
        mark_runtime_snapshot_dirty(&mut shell);
    }
    for _ in 0..24 {
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::ArrowRight);
        if app.world().resource::<BevyRuntimeShell>().shell.session().state().step_events.bike_step_count == 1024 { break; }
    }
    {
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(shell.shell.session().state().step_events.bike_step_count, 1024);
        assert!(!shell.shell.session().state().flags.is_engine_flag_set("ENGINE_BIKE_SHOP_CALL_ENABLED").unwrap());
    }
    // The following count step dispatches the queued special call.
    for _ in 0..24 {
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::ArrowRight);
        let shell = app.world().resource::<BevyRuntimeShell>();
        if shell.shell.session().state().script_runtime.next_script.as_ref().is_some_and(|next| next.script == "Script_ReceivePhoneCall")
            || shell.active_script_cursor.is_some() { break; }
    }
    let labels = quest_settle(&mut app, false, quest_dialogue_is_idle);
    assert!(labels.iter().any(|label| label == "BikeShopPhoneCallerText"), "shop call did not reach its dialogue: {labels:?}");
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert!(shell.shell.session().state().script_runtime.special_phone_call.is_none());
        assert_eq!(quest_item_quantity(&shell, "BICYCLE"), 1);
        quest_assert_save_round_trip(&mut shell, "bike-shop-call");
        assert!(!shell.shell.session().state().flags.is_engine_flag_set("ENGINE_BIKE_SHOP_CALL_ENABLED").unwrap());
    }
    for _ in 0..12 { press_key_for_runtime_hotkey_app(&mut app, KeyCode::ArrowLeft); }
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert!(shell.shell.session().state().script_runtime.special_phone_call.is_none());
    assert_eq!(shell.shell.session().state().step_events.bike_step_count, 1024);
}

fn check_shiny_egg_walk_hatch_nickname(accept: bool) {
    let mut shell = progression_shell_on_map_for_test("NewBarkTown");
    let dvs = Dv::from_non_hp(2, 10, 10, 10);
    shell.shell.add_party_pokemon("TOGEPI", 5, None, None, "CHRIS", 1, dvs).unwrap();
    let runtime = shell.shell.runtime().clone();
    let (state, overworld) = shell.shell.session_mut().state_and_overworld_mut();
    runtime.data().transition_overworld_session(state, overworld, "NewBarkTown", TilePosition::new(13, 6),
        crate::core::systems::map_context::SpawnMemoryUpdate::Preserve, &runtime.music_ids()).unwrap();
    state.player_name = "CHRIS".to_string();
    state.player_id = 1;
    let egg = state.storage.party.pokemon[1].as_mut().unwrap();
    egg.is_egg = true;
    egg.nickname = "EGG".to_string();
    egg.hp = 0;
    egg.happiness = 1;
    // Stage the last hatch cycle, then use actual directional input.
    state.step_events.step_count = 127;
    state.sync_party_from_storage();
    reset_visible_navigation_state(&mut shell);
    mark_runtime_snapshot_dirty(&mut shell);
    if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
        shell.shell.save(PathBuf::from(directory).join("hatch-browser.crystalsave")).unwrap();
    }
    let mut app = menu_render_test_app(shell);
    for _ in 0..24 {
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::ArrowRight);
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(shell.last_error.is_none(), "hatch trigger error: {:?}", shell.last_error);
        if shell.visible_egg_hatch.is_some() { break; }
    }
    assert!(app.world().resource::<BevyRuntimeShell>().visible_egg_hatch.is_some(), "walking must begin hatch presentation");
    quest_settle(&mut app, false, |shell| shell.pending_egg_hatch_nickname.is_some());
    {
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(shell.visible_egg_hatch.is_none());
        assert_eq!(shell.pending_name_choice.as_ref().unwrap().options, ["YES", "NO"]);
        let hatched = shell.shell.session().state().storage.party.pokemon[1].as_ref().unwrap();
        assert!(!hatched.is_egg);
        assert_eq!(hatched.dvs, dvs);
        assert!(visible_pokemon_is_shiny(hatched));
    }
    if accept {
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
        assert!(app.world().resource::<BevyRuntimeShell>().pending_name_input.is_some());
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ); // A on the name grid.
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::Enter); // END.
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ); // Confirm.
    } else {
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyX);
    }
    let before = app.world().resource::<BevyRuntimeShell>().shell.session().overworld().player.tile;
    for _ in 0..12 { press_key_for_runtime_hotkey_app(&mut app, KeyCode::ArrowLeft); }
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    assert!(shell.last_error.is_none(), "{:?}", shell.last_error);
    assert!(shell.pending_egg_hatch_nickname.is_none());
    assert!(shell.pending_name_choice.is_none());
    assert!(shell.pending_name_input.is_none());
    assert_ne!(shell.shell.session().overworld().player.tile, before, "overworld movement resumes");
    quest_assert_save_round_trip(&mut shell, "visible-shiny-hatch");
    let restored = shell.shell.session().state().storage.party.pokemon[1].as_ref().unwrap();
    assert_eq!(restored.nickname, if accept { "A" } else { "TOGEPI" });
    assert_eq!(restored.dvs, dvs);
    assert!(!restored.is_egg);
}

#[test]
fn shiny_egg_walk_hatch_nickname_decline_restores_overworld_and_saves() {
    check_shiny_egg_walk_hatch_nickname(false);
}

#[test]
fn shiny_egg_walk_hatch_nickname_accept_restores_overworld_and_saves() {
    check_shiny_egg_walk_hatch_nickname(true);
}

#[test]
fn shiny_hatch_nickname_prompt_retains_the_front_sprite() {
    let mut shell = progression_shell_on_map_for_test("NewBarkTown");
    shell.shell.add_party_pokemon("TOGEPI", 5, None, None, "CHRIS", 1,
        Dv::from_non_hp(2, 10, 10, 10)).unwrap();
    shell.pending_egg_hatch_nickname = Some(PendingEggHatchNickname {
        party_index: 1, default_name: "TOGEPI".into(),
    });
    let choice = VisibleNameChoice {
        nickname_pages: visible_nickname_prompt_pages(&shell, "TOGEPI", true).unwrap(),
        options: vec!["YES".into(), "NO".into()], selected: 0,
        player_menu: None, player_phase: None, motion_step: 0,
        motion_frames_remaining: 0, pending_player_name: None,
    };
    let mut images = Assets::<Image>::default();
    let mut art = RenderedTilesetArt::default();
    let mut world = World::new();
    let mut queue = bevy::ecs::world::CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);
    spawn_visible_name_choice_screen(&mut commands, &shell, &mut art, &shell.asset_root,
        &mut images, &choice).unwrap();
    queue.apply(&mut world);
    let key = PokemonArtKey { species_id: "togepi".into(), side: PokemonSpriteSide::Front,
        shiny: true, frame: 0 };
    let frame = art.pokemon_cache.get(&key).expect("nickname prompt retains shiny hatch art");
    assert!(world.query::<&Handle<Image>>().iter(&world).any(|image| image == &frame.handle),
        "the retained frontpic must be drawn, not merely cached");
    let canvas = render_pc_audit_canvas(&mut world, &images, "shiny-hatch-nickname");
    if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
        canvas.save(PathBuf::from(directory).join("shiny-hatch-nickname.png")).unwrap();
    }
}

#[test]
fn kanto_gym_rewards_and_repeat_dialogue_survive_save_load() {
    for (map, trainer, badge, tm, repeat_text) in [
        ("PewterGym", "BROCK", 0, None, "BrockFightDoneText"),
        ("CeruleanGym", "MISTY", 1, None, "MistyFightDoneText"),
        ("VermilionGym", "LT_SURGE", 2, None, "LtSurgeFightDoneText"),
        ("CeladonGym", "ERIKA", 3, Some("TM_GIGA_DRAIN"), "ErikaAfterBattleText"),
        ("FuchsiaGym", "JANINE", 4, Some("TM_TOXIC"), "JanineText_ApplyMyself"),
        ("SaffronGym", "SABRINA", 5, None, "SabrinaFightDoneText"),
        ("SeafoamGym", "BLAINE", 6, None, "BlaineFightDoneText"),
        ("ViridianGym", "BLUE", 7, None, "LeaderBlueEpilogueText"),
    ] {
        eprintln!("checking {trainer} Kanto continuation");
        let mut shell = progression_shell_on_map_for_test(map);
        // Stage leader availability; access quests are checked separately.
        for flag in ["EVENT_TRAINERS_IN_CERULEAN_GYM", "EVENT_VIRIDIAN_GYM_BLUE"] {
            shell.shell.session_mut().state_mut().flags.set_event_flag(flag, false).unwrap();
        }
        let before_bag = shell.shell.session().state().bag.clone();
        let key = shell.shell.scripted_trainer_battle_keys().into_iter()
            .find(|key| key.map_name == map && key.trainer_class == trainer).unwrap();
        shell.shell.start_scripted_trainer_battle(&key.map_name, &key.source_script, key.startbattle_command_index).unwrap();
        {
            let state = shell.shell.session_mut().state_mut();
            let crate::core::state::BattleMemory::Trainer { enemy_pokemon, enemy_party, .. } = &mut state.battle
                else { panic!("{trainer} requires trainer battle"); };
            state.battle_rewarded_enemy_party_indices = (0..enemy_party.len()).collect();
            for pokemon in enemy_party { pokemon.hp = 0; }
            enemy_pokemon.hp = 0;
        }
        shell.battle_message_scene = Some(Box::new(shell.shell.snapshot().unwrap()));
        complete_visible_scripted_trainer_battle(&mut shell, &key.map_name, &key.source_script, true, false).unwrap();
        let mut app = menu_render_test_app(shell);
        quest_settle(&mut app, true, |shell| quest_dialogue_is_idle(shell)
            && shell.battle_messages.is_empty() && !shell.shell.has_active_battle());
        let awarded_bag = {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            assert!(shell.shell.session().state().badges.kanto[badge], "{trainer} badge");
            let defeat = if trainer == "LT_SURGE" { "EVENT_BEAT_LTSURGE".into() } else { format!("EVENT_BEAT_{trainer}") };
            assert!(shell.shell.session().state().flags.is_event_flag_set(&defeat).unwrap());
            if let Some(tm) = tm { assert_eq!(quest_item_quantity(&shell, tm), 1); }
            else { assert_eq!(shell.shell.session().state().bag, before_bag, "{trainer} does not award a TM"); }
            let bag = shell.shell.session().state().bag.clone();
            quest_move_beside_npc(&mut shell, map, &key.source_script);
            quest_talk(&mut shell, &key.source_script);
            bag
        };
        let labels = quest_settle(&mut app, true, quest_dialogue_is_idle);
        assert!(labels.iter().any(|label| label == repeat_text), "{trainer} repeat: {labels:?}");
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(shell.shell.session().state().bag, awarded_bag);
        assert!(!shell.shell.has_active_battle(), "{trainer} must not restart the battle");
        quest_assert_save_round_trip(&mut shell, trainer);
        assert!(shell.shell.session().state().badges.kanto[badge]);
    }
}

#[test]
fn oak_requires_sixteen_badges_and_mt_silver_gate_stays_open_after_reload() {
    let mut shell = progression_shell_on_map_for_test("OaksLab");
    for index in 0..8 { shell.shell.award_badge(RuntimeBadgeRegion::Johto, index).unwrap(); }
    let mut app = menu_render_test_app(shell);
    for (kanto_count, expected_text) in [(0, "OakNoKantoBadgesText"), (7, "OakYesKantoBadgesText"), (8, "OakOpenMtSilverText")] {
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            for index in 0..kanto_count { shell.shell.award_badge(RuntimeBadgeRegion::Kanto, index).unwrap(); }
            quest_move_beside_npc(&mut shell, "OaksLab", "Oak");
            quest_talk(&mut shell, "Oak");
        }
        let labels = quest_settle(&mut app, true, quest_dialogue_is_idle);
        assert!(labels.iter().any(|label| label == expected_text), "{kanto_count} Kanto badges: {labels:?}");
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(shell.shell.session().state().flags.is_event_flag_set("EVENT_OPENED_MT_SILVER").unwrap(), kanto_count == 8);
        quest_move_beside_npc(&mut shell, "VictoryRoadGate", "VictoryRoadGateLeftBlackBeltScript");
        assert_eq!(shell.shell.snapshot().unwrap().visible_objects.iter().any(|object|
            object.object_identifier.as_deref() == Some("VICTORYROADGATE_BLACK_BELT1")), kanto_count < 8);
    }
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        quest_assert_save_round_trip(&mut shell, "mt-silver-unlock");
        assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_OPENED_MT_SILVER").unwrap());
        assert!(!shell.shell.snapshot().unwrap().visible_objects.iter().any(|object|
            object.object_identifier.as_deref() == Some("VICTORYROADGATE_BLACK_BELT1")));
        quest_move_beside_npc(&mut shell, "OaksLab", "Oak");
        quest_talk(&mut shell, "Oak");
    }
    let labels = quest_settle(&mut app, true, quest_dialogue_is_idle);
    assert!(labels.iter().any(|label| label == "OakLabDexCheckText"));
    assert!(!labels.iter().any(|label| label == "OakOpenMtSilverText"));
}

#[test]
fn every_oak_rating_resolves_its_authored_far_text() {
    let mut shell = progression_shell_on_map_for_test("OaksLab");
    let ratings = shell.runtime.data().oak_ratings.clone();
    assert_eq!(ratings.len(), 19);
    for rating in ratings {
        open_visible_prof_oak_rating(&mut shell, rating.caught_count_limit, rating.caught_count_limit,
            &rating.text_label).unwrap();
        let mut pages = Vec::new();
        shell.pc_hub_session_open = true;
        while let Some(display) = shell.special_boundary.as_ref() {
            assert_eq!(display.details.len(), 1);
            assert!(display.details[0].lines().count() <= 2, "{}", rating.text_label);
            pages.push(display.details[0].clone());
            close_visible_special_boundary(&mut shell).unwrap();
        }
        assert!(pages.len() >= 4, "{}: {pages:?}", rating.text_label);
        assert_eq!(pages[0], "Current POKéDEX\ncompletion level:");
        assert!(pages[1].contains(&format!("{} POKéMON seen", rating.caught_count_limit)));
        assert!(pages[1].contains(&format!("{} POKéMON owned", rating.caught_count_limit)));
        assert!(shell.pc_hub_cursor.is_some(), "return to PC after the final page");
    }
}

#[test]
fn oak_assessment_prints_complete_pages_over_the_lab() {
    let mut shell = progression_shell_on_map_for_test("OaksLab");
    open_visible_prof_oak_rating(&mut shell, 251, 251, "OakRating19").unwrap();
    let mut app = menu_render_test_app(shell);
    let mut observed = Vec::new();
    for index in 0..16 {
        app.update();
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            for _ in 0..100 {
                tick_visible_field_text_reveal(&mut shell, true).unwrap();
            }
            mark_runtime_presentation_dirty(&mut shell);
        }
        app.update();
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(shell.last_error, None);
        let Some(boundary) = shell.special_boundary.as_ref() else { break; };
        assert_eq!(visible_field_dialog_pages(&shell.shell.snapshot().unwrap(), shell),
            Some(boundary.details.clone()));
        assert_eq!(visible_scene_dialog_entries(&shell.shell.snapshot().unwrap(), shell).unwrap().join("\n"),
            boundary.details[0]);
        // The source font uses one tile for contractions such as 's.
        let expected_glyphs = normalize_bitmap_font_text(&boundary.details[0])
            .chars().filter(|ch| *ch != '\n').count();
        observed.extend(boundary.details.clone());
        assert!(app.world_mut().query_filtered::<&Handle<Image>, With<DialogGlyphMarker>>()
            .iter(app.world()).count() >= expected_glyphs,
            "assessment page {index} glyphs must reach the renderer");
        save_live_pc_dialog_for_test(app.world_mut(), &format!("oak-assessment-{index}.png"));
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    }
    assert_eq!(observed.last().map(String::as_str), Some("dreamt about this!\nCongratulations!"));
    assert!(app.world().resource::<BevyRuntimeShell>().special_boundary.is_none());
}

#[test]
fn oak_rating_fanfare_waits_for_final_text_and_remembers_acknowledgement() {
    let mut shell = progression_shell_on_map_for_test("OaksLab");
    for rating in shell.runtime.data().oak_ratings.clone() {
        shell.pending_audio.clear();
        shell.pc_hub_session_open = true;
        shell.pc_hub_cursor = None;
        open_visible_prof_oak_rating(&mut shell, rating.caught_count_limit,
            rating.caught_count_limit, &rating.text_label).unwrap();
        while !shell.special_boundary_queue.is_empty() {
            for _ in 0..100 { tick_visible_field_text_reveal(&mut shell, true).unwrap(); }
            assert!(shell.pending_audio.is_empty(), "no fanfare on earlier pages");
            close_visible_special_boundary(&mut shell).unwrap();
        }
        tick_visible_field_text_reveal(&mut shell, true).unwrap();
        assert!(shell.pending_audio.is_empty(), "wait for the last page to print");
        for _ in 0..100 { tick_visible_field_text_reveal(&mut shell, true).unwrap(); }
        assert_eq!(shell.pending_audio.len(), 1);
        assert_eq!(shell.pending_audio[0].audio_id, rating.fanfare);
        assert!(!shell.pending_audio[0].looped);
        for _ in 0..100 { tick_visible_field_text_reveal(&mut shell, true).unwrap(); }
        assert_eq!(shell.pending_audio.len(), 1, "play each rating fanfare once");
        shell.pending_audio.clear();
        shell.transient_audio_playing = true;
        close_visible_special_boundary(&mut shell).unwrap();
        assert!(shell.special_boundary.is_some());
        assert!(shell.pc_hub_cursor.is_none());
        advance_visible_special_text_pause(&mut shell).unwrap();
        assert!(shell.special_boundary.is_some(), "retain text while sound plays");
        shell.transient_audio_playing = false;
        advance_visible_special_text_pause(&mut shell).unwrap();
        assert!(shell.special_boundary.is_none(), "no second button press required");
        assert!(shell.pc_hub_cursor.is_some());
    }
}

#[test]
fn oak_fanfares_match_pack_pcm_hash_frames_and_loop_metadata() {
    let shell = progression_shell_on_map_for_test("OaksLab");
    let fanfares = shell.runtime.data().oak_ratings.iter()
        .map(|rating| rating.fanfare.clone()).collect::<std::collections::BTreeSet<_>>();
    for id in fanfares {
        let program = shell.runtime.audio().program(crystal_audio::AudioKind::SoundEffect, &id).unwrap();
        let (decoded, expected_bytes) = match program.source.clone() {
            crystal_audio::AudioProgramSource::PcmGzip { bytes, format, byte_len, payload_hash,
                loop_start_sample, loop_end_sample } => (
                crystal_audio::pcm::decode_gzip_pcm(&bytes, format, byte_len, &payload_hash,
                    loop_start_sample, loop_end_sample).unwrap(), byte_len),
            crystal_audio::AudioProgramSource::Midi { midi_base64, format, byte_len, payload_hash,
                loop_start_sample, loop_end_sample } => (
                crystal_audio::pcm::decode_midi_pcm(&midi_base64, format, byte_len, &payload_hash,
                    loop_start_sample, loop_end_sample).unwrap(), byte_len),
            _ => panic!("{id} must have canonical pack PCM integrity metadata"),
        };
        assert_eq!(decoded.samples.len() / 2, expected_bytes / 4, "{id} frame count");
        assert_eq!(decoded.loop_range, None, "{id} fanfare must terminate");
        assert!(decoded.samples.iter().any(|sample| sample.unsigned_abs() > 32), "{id} is silent");
    }
}

#[test]
fn oak_lab_script_waits_for_assessment_before_goodbye() {
    let mut shell = progression_shell_on_map_for_test("OaksLab");
    quest_move_beside_npc(&mut shell, "OaksLab", "Oak");
    shell.shell.session_mut().state_mut().player_name = "CHRIS".into();
    shell.shell.session_mut().overworld_mut().player.facing = Direction::Up;
    if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
        shell.shell.save(PathBuf::from(directory).join("oak-browser.crystalsave")).unwrap();
    }
    quest_talk(&mut shell, "Oak");
    let mut app = menu_render_test_app(shell);
    let labels = quest_settle(&mut app, true, |shell| quest_dialogue_is_idle(shell)
        || shell.special_boundary.as_ref().is_some_and(|b| b.label == "ProfOaksPcBoot"));
    assert!(!labels.iter().any(|label| label == "OakLabGoodbyeText"));
    for _ in 0..120 { app.update(); }
    {
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(shell.special_boundary.as_ref().is_some_and(|b| b.label == "ProfOaksPcBoot"));
        assert_ne!(shell.shell.session().state().script_runtime.active_text_label.as_deref(),
            Some("OakLabGoodbyeText"), "the suspended special must own the conversation");
    }
    let labels = quest_settle(&mut app, true, |shell| quest_dialogue_is_idle(shell)
        && shell.special_boundary.is_none() && shell.special_boundary_queue.is_empty());
    assert!(labels.iter().any(|label| label == "OakLabGoodbyeText"));
}

#[test]
fn pharmacy_before_jasmines_request_opens_shop_without_giving_medicine() {
    let mut shell = progression_shell_on_map_for_test("CianwoodPharmacy");
    quest_move_beside_npc(&mut shell, "CianwoodPharmacy", "CianwoodPharmacist");
    let bag = shell.shell.session().state().bag.clone();
    quest_talk(&mut shell, "CianwoodPharmacist");
    let mut app = menu_render_test_app(shell);
    quest_settle(&mut app, true, |shell| shell.shop_top_cursor.is_some());
    {
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(shell.shell.snapshot().unwrap().pending_shop.unwrap().mart_id, "MART_CIANWOOD");
        assert_eq!(shell.shell.session().state().bag, bag);
        assert_eq!(quest_item_quantity(shell, "SECRETPOTION"), 0);
        assert!(!shell.shell.session().state().flags
            .is_event_flag_set("EVENT_GOT_SECRETPOTION_FROM_PHARMACY").unwrap());
    }
    for _ in 0..24 {
        app.update();
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            for _ in 0..100 { tick_visible_field_text_reveal(&mut shell, true).unwrap(); }
            mark_runtime_presentation_dirty(&mut shell);
        }
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyX);
        if quest_dialogue_is_idle(app.world().resource::<BevyRuntimeShell>()) { break; }
    }
    assert!(quest_dialogue_is_idle(app.world().resource::<BevyRuntimeShell>()));
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert!(shell.shell.snapshot().unwrap().pending_shop.is_none());
    assert_eq!(shell.shell.session().state().bag, bag);
}

#[test]
fn moomoo_farm_feeding_milestones_rewards_and_save_load() {
    fn visit(app: &mut App, map: &str, script: &str, answer: bool) -> Vec<String> {
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            quest_move_beside_npc(&mut shell, map, script);
            quest_talk(&mut shell, script);
        }
        quest_settle(app, answer, |shell| {
            quest_dialogue_is_idle(shell) && shell.special_boundary.is_none()
        })
    }
    let mut shell = progression_shell_on_map_for_test("Route39Barn");
    shell.shell.session_mut().state_mut().player_name = "CHRIS".into();
    if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
        quest_move_beside_npc(&mut shell, "Route39Barn", "MoomooScript");
        shell.shell.session_mut().overworld_mut().player.facing = Direction::Up;
        shell.shell.save(PathBuf::from(directory).join("moomoo-browser.crystalsave")).unwrap();
    }
    let mut app = menu_render_test_app(shell);
    let labels = visit(&mut app, "Route39Barn", "MoomooScript", true);
    assert!(labels
        .iter()
        .any(|label| label == "Route39BarnItsCryIsWeakText"));
    assert!(!labels
        .iter()
        .any(|label| label == "Route39BarnAskGiveBerryText"));
    let labels = visit(&mut app, "Route39Farmhouse", "PokefanF_SnoreFarmer", true);
    assert!(labels.iter().any(|label| label == "FarmerFText_InTrouble"));
    assert_eq!(
        quest_item_quantity(app.world().resource::<BevyRuntimeShell>(), "TM_SNORE"),
        0
    );
    visit(&mut app, "Route39Farmhouse", "PokefanM_DairyFarmer", true);
    assert!(app
        .world()
        .resource::<BevyRuntimeShell>()
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set("EVENT_TALKED_TO_FARMER_ABOUT_MOOMOO")
        .unwrap());
    let labels = visit(&mut app, "Route39Barn", "MoomooScript", true);
    assert!(labels
        .iter()
        .any(|label| label == "Route39BarnNoBerriesText"));
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        shell.shell.add_bag_item("BERRY", 7).unwrap();
    }
    let labels = visit(&mut app, "Route39Barn", "MoomooScript", false);
    assert!(labels
        .iter()
        .any(|label| label == "Route39BarnRefusedBerryText"));
    assert_eq!(
        quest_item_quantity(app.world().resource::<BevyRuntimeShell>(), "BERRY"),
        7
    );
    for fed in 1..=7 {
        let labels = visit(&mut app, "Route39Barn", "MoomooScript", true);
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, "BERRY"), 7 - fed);
        assert_eq!(
            shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set("EVENT_HEALED_MOOMOO")
                .unwrap(),
            fed == 7
        );
        let milestone = match fed {
            3 => Some("Route39BarnLittleHealthierText"),
            5 => Some("Route39BarnQuiteHealthyText"),
            7 => Some("Route39BarnTotallyHealthyText"),
            _ => None,
        };
        if let Some(label) = milestone {
            assert!(labels.iter().any(|actual| actual == label));
        }
        if fed == 3 || fed == 7 {
            let progress = shell
                .shell
                .session()
                .state()
                .script_runtime
                .memory
                .get("wMooMooBerries")
                .cloned();
            quest_assert_save_round_trip(&mut shell, "moomoo-feeding");
            assert_eq!(
                shell
                    .shell
                    .session()
                    .state()
                    .script_runtime
                    .memory
                    .get("wMooMooBerries"),
                progress.as_ref()
            );
        }
    }
    let labels = visit(&mut app, "Route39Barn", "MoomooScript", true);
    assert!(labels.iter().any(|label| label == "MoomooHappyMooText"));
    assert!(!labels
        .iter()
        .any(|label| label == "Route39BarnAskGiveBerryText"));
    for script in ["Route39BarnTwin1Script", "Route39BarnTwin2Script"] {
        let labels = visit(&mut app, "Route39Barn", script, true);
        assert!(labels
            .iter()
            .any(|label| label == "Route39BarnTwinWereFeedingMoomooText"));
    }
    for _ in 0..2 {
        let labels = visit(&mut app, "Route39Farmhouse", "PokefanF_SnoreFarmer", true);
        assert!(labels
            .iter()
            .any(|label| label == "FarmerFText_SnoreSpeech"));
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, "TM_SNORE"), 1);
        quest_assert_save_round_trip(&mut shell, "moomoo-snore");
    }
    let before_money = app
        .world()
        .resource::<BevyRuntimeShell>()
        .shell
        .session()
        .state()
        .money;
    let labels = visit(&mut app, "Route39Farmhouse", "PokefanM_DairyFarmer", false);
    assert!(labels.iter().any(|label| label == "FarmerMText_NoSale"));
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .shell
            .session()
            .state()
            .money,
        before_money
    );
    visit(&mut app, "Route39Farmhouse", "PokefanM_DairyFarmer", true);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, "MOOMOO_MILK"), 1);
        assert_eq!(shell.shell.session().state().money, before_money - 500);
        quest_assert_save_round_trip(&mut shell, "moomoo-milk");
    }
    let labels = visit(&mut app, "Route39Farmhouse", "PokefanM_DairyFarmer", true);
    assert!(labels.iter().any(|label| label == "FarmerMText_Milking"));
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(quest_item_quantity(shell, "MOOMOO_MILK"), 1);
    assert_eq!(shell.shell.session().state().money, before_money - 500);
}

#[test]
fn moomoo_milk_preserves_money_on_rejection_and_retries_after_making_room() {
    let mut shell = progression_shell_on_map_for_test("Route39Farmhouse");
    // Isolate sale rejection branches; the feeding test establishes healing.
    shell
        .shell
        .session_mut()
        .state_mut()
        .flags
        .set_event_flag("EVENT_HEALED_MOOMOO", true)
        .unwrap();
    shell.shell.session_mut().state_mut().money = 499;
    quest_move_beside_npc(&mut shell, "Route39Farmhouse", "PokefanM_DairyFarmer");
    quest_talk(&mut shell, "PokefanM_DairyFarmer");
    let mut app = menu_render_test_app(shell);
    let labels = quest_settle(&mut app, true, quest_dialogue_is_idle);
    assert!(labels.iter().any(|label| label == "FarmerMText_NoMoney"));
    let items;
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(shell.shell.session().state().money, 499);
        assert_eq!(quest_item_quantity(&shell, "MOOMOO_MILK"), 0);
        items = shell
            .runtime
            .data()
            .items
            .values()
            .filter(|item| item.pocket == "ITEM" && item.script_name != "MOOMOO_MILK")
            .take(20)
            .cloned()
            .collect::<Vec<_>>();
        let state = shell.shell.session_mut().state_mut();
        state.money = 500;
        state.bag.items.clear();
        for item in &items {
            assert!(state.bag.add_item(item, 1).unwrap());
        }
        assert_eq!(state.bag.items.len(), 20);
        quest_talk(&mut shell, "PokefanM_DairyFarmer");
    }
    let labels = quest_settle(&mut app, true, quest_dialogue_is_idle);
    assert!(labels.iter().any(|label| label == "FarmerMText_NoRoom"));
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(shell.shell.session().state().money, 500);
        assert_eq!(quest_item_quantity(&shell, "MOOMOO_MILK"), 0);
        assert!(shell
            .shell
            .session_mut()
            .state_mut()
            .bag
            .remove_item(&items[0], 1)
            .unwrap());
        quest_talk(&mut shell, "PokefanM_DairyFarmer");
    }
    quest_settle(&mut app, true, quest_dialogue_is_idle);
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    assert_eq!(shell.shell.session().state().money, 0);
    assert_eq!(quest_item_quantity(&shell, "MOOMOO_MILK"), 1);
    quest_assert_save_round_trip(&mut shell, "moomoo-milk-retry");
}

#[test]
fn moomoo_slow_cry_accepts_the_authored_species_value() {
    let mut shell = progression_shell_on_map_for_test("Route39Barn");
    shell
        .shell
        .step_compiled_script_command(
            "Route39Barn",
            "MoomooScript",
            1,
            Default::default(),
            Default::default(),
        )
        .unwrap();
    shell
        .shell
        .step_compiled_script_command(
            "Route39Barn",
            "MoomooScript",
            4,
            Default::default(),
            Default::default(),
        )
        .unwrap();
    let step = shell
        .shell
        .step_compiled_script_command(
            "Route39Barn",
            "MoomooScript",
            5,
            Default::default(),
            Default::default(),
        )
        .unwrap();
    assert!(
        matches!(step.mutation.result, RuntimeMutationResult::SpecialRoutineApplied(
        crate::core::systems::special_routines::SpecialRoutineOutcome {
            effect: SpecialRoutineEffect::PlaySlowCry { ref species, .. }, ..
        }) if species == "MILTANK")
    );
}


#[test]
fn moomoo_price_text_resolves_pack_decimal_constant_before_rendering() {
    let mut shell = progression_shell_on_map_for_test("Route39Farmhouse");
    if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
        shell.shell.session_mut().state_mut().flags.set_event_flag("EVENT_HEALED_MOOMOO", true).unwrap();
        shell.shell.session_mut().state_mut().player_name = "CHRIS".into();
        quest_move_beside_npc(&mut shell, "Route39Farmhouse", "PokefanM_DairyFarmer");
        shell.shell.save(PathBuf::from(directory).join("milk-price-browser.crystalsave")).unwrap();
    }
    let text = shell.shell.text_snapshot("FarmerMText_BuyMilk").unwrap();
    let body = text.body.expect("authored milk offer text");
    let pages = render_visible_script_text_pages(
        &body, &Default::default(), "CHRIS", "RIVAL", 0,
    );
    assert!(pages.iter().any(|page| page.contains("fer just ¥500.")), "{pages:?}");
    assert!(pages.iter().all(|page| !page.contains("{d:")), "{pages:?}");
}


#[test]
fn all_map_dialogue_decimal_constants_resolve_from_the_pack() {
    let mut shell = progression_shell_on_map_for_test("Route39Farmhouse");
    let targets = shell.runtime.data().maps.iter().flat_map(|(map, module)| {
        module.script_text_bodies.iter().filter(|(_, body)| {
            body.commands.iter().any(|command| command.args.iter().any(|arg| arg.contains("{d:")))
        }).map(|(label, _)| (map.clone(), label.clone())).collect::<Vec<_>>()
    }).collect::<Vec<_>>();
    assert!(!targets.is_empty());
    let mut failures = Vec::new();
    for (map, label) in &targets {
        if let crate::core::state::OverworldMemory::Active { map_name, .. } =
            &mut shell.shell.session_mut().state_mut().overworld {
            *map_name = map.clone();
        }
        let text = match shell.shell.text_snapshot(label) {
            Ok(text) => text,
            Err(error) => { failures.push(format!("{map}:{label}: {error:#}")); continue; }
        };
        let body = text.body.unwrap();
        assert!(body.commands.iter().all(|command| command.args.iter().all(|arg| !arg.contains("{d:"))), "{map}:{label}");
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    eprintln!("Resolved {} map dialogue bodies with decimal constants", targets.len());
}

#[test]
fn moomoo_slow_cry_validates_base_pcm_and_keeps_distinct_output_and_cache() {
    let mut shell = progression_shell_on_map_for_test("Route39Barn");
    shell.pending_audio.clear();
    for index in [1, 4, 5] {
        shell.shell.step_compiled_script_command("Route39Barn", "MoomooScript", index,
            Default::default(), Default::default()).unwrap();
    }
    assert!(shell.shell.session().state().script_runtime.waiting_for_sound_effect);
    queue_visible_slow_cry(&mut shell, "MILTANK").unwrap();
    assert_eq!(shell.pending_audio.len(), 1, "the ordinary cry must not also be queued");
    assert!(shell.visible_wait_sfx_boundary);
    assert!(!visible_wait_sfx_finished(&mut shell), "queued cry must hold the script");
    let slow = shell.pending_audio[0].clone();
    assert_eq!(slow.audio_id, "CRY_MON_MILTANK");
    let source = shell.runtime.audio().require_cry(&slow.audio_id).unwrap().source.clone();
    let source_copy = source.clone();
    let AudioProgramSource::Midi { midi_base64, format, byte_len, payload_hash,
        loop_start_sample, loop_end_sample } = source else { panic!("browser species cry must retain its MIDI program") };
    assert_eq!((loop_start_sample, loop_end_sample), (None, None));
    let base = crystal_audio::pcm::decode_midi_pcm(&midi_base64, format.clone(), byte_len,
        &payload_hash, None, None).unwrap();
    let request = crystal_audio::pcm::ModifiedCryRequest { format, byte_len,
        payload_hash: payload_hash.clone(), parameters: slow.cry_parameters.unwrap() };
    // Source Miltank parameters are -461/416; PlaySlowCry applies -320/+96.
    assert_eq!((request.parameters.pitch, request.parameters.length), (64755, 512));
    let modified = crystal_audio::pcm::decode_modified_cry(&midi_base64, &request).unwrap();
    assert_eq!(decoded_audio_program_source(&slow, source_copy).unwrap().bytes, modified.bytes);
    assert_eq!(modified.bytes.len() / 4, 19198);
    assert_eq!(bevy_audio_fnv1a32(&modified.bytes), 0xdb796201);

    assert_eq!(modified.loop_range, None);
    assert!(modified.bytes.len() > base.bytes.len(), "source cry length must increase duration");
    assert_ne!(modified.bytes, base.bytes);
    assert!(modified.samples.iter().any(|sample| sample.unsigned_abs() > 32));
    let mut invalid = request.clone();
    invalid.payload_hash = "00000000".into();
    assert!(crystal_audio::pcm::decode_modified_cry(&midi_base64, &invalid).is_err());
    let mut normal = slow.clone();
    normal.cry_parameters = None;
    let ordinary = decoded_native_midi_audio(&normal, &midi_base64, request.format.clone(),
        byte_len, &payload_hash, None, None).unwrap();
    assert_eq!(ordinary.bytes, base.bytes);
    shell.pending_audio.clear();
    shell.transient_audio_playing = true;
    assert!(!visible_wait_sfx_finished(&mut shell), "playing cry must hold the script");
    shell.transient_audio_playing = false;
    assert!(visible_wait_sfx_finished(&mut shell), "finished cry releases the audio wait");
    assert_ne!(BevyAudioCacheKey::from_command(&slow), BevyAudioCacheKey::from_command(&normal));
    eprintln!("MILTANK base frames={} hash={payload_hash}; slow frames={} hash={:08x} pitch={} length={}",
        base.bytes.len()/4, modified.bytes.len()/4, bevy_audio_fnv1a32(&modified.bytes),
        request.parameters.pitch, request.parameters.length);
}

#[test]
fn radio_card_quiz_refusal_each_wrong_answer_reward_and_reload() {
    const SCRIPT: &str = "RadioTower1FRadioCardWomanScript";
    const ANSWERS: [bool; 5] = [true, true, false, true, false];
    fn at_question(shell: &BevyRuntimeShell, index: usize) -> bool {
        let scripts = &shell.shell.session().state().script_runtime;
        scripts.pending_yes_no.is_some() && scripts.active_text_label.as_deref()
            == Some(format!("RadioTower1FRadioCardWomanQuestion{index}Text").as_str())
    }
    let mut shell = progression_shell_on_map_for_test("RadioTower1F");
    shell.shell.session_mut().state_mut().flags.set_engine_flag("ENGINE_POKEGEAR", true).unwrap();
    assert!(!shell.shell.session().state().flags.is_engine_flag_set("ENGINE_RADIO_CARD").unwrap());
    open_visible_pokegear_menu(&mut shell).unwrap();
    cycle_visible_pokegear_page(&mut shell, 4).unwrap();
    assert_ne!(shell.pokegear_page, PokegearPage::Radio);
    close_visible_pokegear_menu(&mut shell).unwrap();
    quest_move_beside_npc(&mut shell, "RadioTower1F", SCRIPT);
    quest_talk(&mut shell, SCRIPT);
    let mut app = menu_render_test_app(shell);
    let labels = quest_settle(&mut app, false, quest_dialogue_is_idle);
    assert!(labels.iter().any(|label| label == "RadioTower1FRadioCardWomanNotTakingQuizText"));
    for wrong in 0..5 {
        quest_talk(&mut app.world_mut().resource_mut::<BevyRuntimeShell>(), SCRIPT);
        quest_settle(&mut app, true, |shell| at_question(shell, 1));
        for (index, answer) in ANSWERS.iter().enumerate().take(wrong) {
            quest_settle(&mut app, *answer, |shell| at_question(shell, index + 2));
        }
        let labels = quest_settle(&mut app, !ANSWERS[wrong], quest_dialogue_is_idle);
        assert!(labels.iter().any(|label| label == "RadioTower1FRadioCardWomanWrongAnswerText"), "wrong question {}", wrong + 1);
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(!shell.shell.session().state().flags.is_engine_flag_set("ENGINE_RADIO_CARD").unwrap());
    }
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        quest_assert_save_round_trip(&mut shell, "radio-quiz-wrong");
        quest_move_beside_npc(&mut shell, "RadioTower1F", SCRIPT);
        quest_talk(&mut shell, SCRIPT);
    }
    quest_settle(&mut app, true, |shell| at_question(shell, 1));
    for (index, answer) in ANSWERS.iter().enumerate().take(4) {
        quest_settle(&mut app, *answer, |shell| at_question(shell, index + 2));
    }
    let labels = quest_settle(&mut app, ANSWERS[4], quest_dialogue_is_idle);
    for expected in ["RadioTower1FRadioCardWomanYouWinText", "RadioTower1FPokegearIsARadioText", "RadioTower1FRadioCardWomanTuneInText"] {
        assert!(labels.iter().any(|label| label == expected), "missing {expected}");
    }
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert!(shell.shell.session().state().flags.is_engine_flag_set("ENGINE_RADIO_CARD").unwrap());
        quest_assert_save_round_trip(&mut shell, "radio-quiz-card");
        assert!(shell.shell.session().state().flags.is_engine_flag_set("ENGINE_RADIO_CARD").unwrap());
        quest_move_beside_npc(&mut shell, "RadioTower1F", SCRIPT);
        quest_talk(&mut shell, SCRIPT);
    }
    let labels = quest_settle(&mut app, true, quest_dialogue_is_idle);
    assert!(labels.iter().any(|label| label == "RadioTower1FRadioCardWomanTuneInText"));
    assert!(!labels.iter().any(|label| label == "RadioTower1FRadioCardWomanOfferQuizText" || label == "RadioTower1FRadioCardWomanYouWinText"));
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    open_visible_pokegear_menu(&mut shell).unwrap();
    cycle_visible_pokegear_page(&mut shell, 4).unwrap();
    assert!(shell.pokegear_menu_open);
    assert_eq!(shell.pokegear_page, PokegearPage::Radio);
    close_visible_pokegear_menu(&mut shell).unwrap();
    assert!(quest_dialogue_is_idle(&shell));
}

#[test]
fn squirtbottle_floria_prerequisites_sudowoodo_and_rock_smash_persist() {
    fn visit(app: &mut App, map: &str, script: &str, answer: bool, expected: &str) {
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            quest_move_beside_npc(&mut shell, map, script);
            quest_talk(&mut shell, script);
        }
        let labels = quest_settle(app, answer, |shell| quest_dialogue_is_idle(shell)
            && shell.special_boundary.is_none());
        assert!(labels.iter().any(|label| label == expected), "{script}: {labels:?}");
    }
    fn event(app: &App, name: &str) -> bool {
        app.world().resource::<BevyRuntimeShell>().shell.session().state().flags
            .is_event_flag_set(name).unwrap()
    }
    fn quantity(app: &App, name: &str) -> u16 {
        quest_item_quantity(app.world().resource::<BevyRuntimeShell>(), name)
    }
    let mut shell = progression_shell_on_map_for_test("Route36");
    quest_move_beside_npc(&mut shell, "Route36", "SudowoodoScript");
    quest_talk(&mut shell, "SudowoodoScript");
    let mut app = menu_render_test_app(shell);
    quest_settle(&mut app, true, quest_dialogue_is_idle);
    assert!(!app.world().resource::<BevyRuntimeShell>().shell.has_active_battle());
    assert!(!event(&app, "EVENT_FOUGHT_SUDOWOODO"));
    visit(&mut app, "Route36", "Route36RockSmashGuyScript", true, "RockSmashGuyText1");
    assert_eq!(quantity(&app, "TM_ROCK_SMASH"), 0);
    visit(&mut app, "GoldenrodFlowerShop", "FlowerShopTeacherScript", true,
        "GoldenrodFlowerShopTeacherMySisterWentToSeeWigglyTreeRoute36Text");
    assert_eq!(quantity(&app, "SQUIRTBOTTLE"), 0);
    visit(&mut app, "Route36", "Route36FloriaScript", true, "FloriaText1");
    assert!(event(&app, "EVENT_MET_FLORIA"));
    assert!(!event(&app, "EVENT_FLORIA_AT_FLOWER_SHOP"));
    visit(&mut app, "GoldenrodFlowerShop", "FlowerShopTeacherScript", true,
        "GoldenrodFlowerShopTeacherLalalaHavePlentyOfWaterText");
    visit(&mut app, "GoldenrodFlowerShop", "FlowerShopFloriaScript", true,
        "GoldenrodFlowerShopFloriaWonderIfSisWillLendWaterBottleText");
    assert!(event(&app, "EVENT_TALKED_TO_FLORIA_AT_FLOWER_SHOP"));
    assert!(event(&app, "EVENT_FLORIA_AT_FLOWER_SHOP"));
    visit(&mut app, "GoldenrodFlowerShop", "FlowerShopTeacherScript", true,
        "GoldenrodFlowerShopTeacherAskWantToBorrowWaterBottleText");
    assert_eq!(quantity(&app, "SQUIRTBOTTLE"), 0);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        quest_assert_save_round_trip(&mut shell, "floria-before-badge");
        // Whitney's battle and delayed badge handoff have separate regressions.
        shell.shell.award_badge(RuntimeBadgeRegion::Johto, 2).unwrap();
    }
    visit(&mut app, "GoldenrodFlowerShop", "FlowerShopTeacherScript", true,
        "GoldenrodFlowerShopTeacherHeresTheSquirtbottleText");
    assert_eq!(quantity(&app, "SQUIRTBOTTLE"), 1);
    assert!(event(&app, "EVENT_GOT_SQUIRTBOTTLE"));
    assert!(event(&app, "EVENT_FLORIA_AT_SUDOWOODO"));
    assert!(!event(&app, "EVENT_FLORIA_AT_FLOWER_SHOP"));
    visit(&mut app, "GoldenrodFlowerShop", "FlowerShopTeacherScript", true,
        "GoldenrodFlowerShopTeacherDontDoAnythingDangerousText");
    visit(&mut app, "Route36", "SudowoodoScript", false, "UseSquirtbottleText");
    assert!(!event(&app, "EVENT_FOUGHT_SUDOWOODO"));
    assert_eq!(quantity(&app, "SQUIRTBOTTLE"), 1);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        quest_assert_save_round_trip(&mut shell, "squirtbottle-refusal");
        quest_move_beside_npc(&mut shell, "Route36", "SudowoodoScript");
        quest_talk(&mut shell, "SudowoodoScript");
    }
    let labels = quest_settle(&mut app, true, |shell| shell.shell.has_active_battle());
    assert!(labels.iter().any(|label| label == "UsedSquirtbottleText"));
    assert!(labels.iter().any(|label| label == "SudowoodoAttackedText"));
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        let crate::core::state::BattleMemory::StaticWild { enemy_pokemon, .. } =
            &shell.shell.session().state().battle else { panic!("expected Sudowoodo battle"); };
        assert_eq!(enemy_pokemon.species.id, "SUDOWOODO");
        assert_eq!(enemy_pokemon.level, 20);
        quest_finish_static_wild_final_turn(&mut shell);
    }
    quest_settle(&mut app, true, |shell| quest_dialogue_is_idle(shell)
        && shell.battle_messages.is_empty() && !shell.shell.has_active_battle());
    assert!(event(&app, "EVENT_FOUGHT_SUDOWOODO"));
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        quest_assert_save_round_trip(&mut shell, "sudowoodo-cleared");
        assert!(!shell.shell.snapshot().unwrap().visible_objects.iter().any(|object|
            object.object_identifier.as_deref() == Some("ROUTE36_WEIRD_TREE")));
    }
    // Walk onto the tree's former occupied tile using the visible input controller.
    let tree_tile = {
        let shell = app.world().resource::<BevyRuntimeShell>();
        let tree = shell.runtime.data().maps["Route36"].objects.iter()
            .find(|object| object.script == "SudowoodoScript").unwrap();
        TilePosition::new(tree.x as i16, tree.y as i16)
    };
    for _ in 0..24 {
        if app.world().resource::<BevyRuntimeShell>().shell.snapshot().unwrap().overworld.tile == tree_tile {
            break;
        }
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::ArrowUp);
    }
    assert_eq!(app.world().resource::<BevyRuntimeShell>().shell.snapshot().unwrap().overworld.tile,
        tree_tile, "cleared tree must no longer block walking after load");
    visit(&mut app, "Route36", "Route36LassScript", true, "Route36LassText_ClearedSudowoodo");
    visit(&mut app, "Route36", "Route36RockSmashGuyScript", true, "RockSmashGuyText2");
    assert_eq!(quantity(&app, "TM_ROCK_SMASH"), 1);
    assert!(event(&app, "EVENT_GOT_TM08_ROCK_SMASH"));
    visit(&mut app, "Route36", "Route36RockSmashGuyScript", true, "RockSmashGuyText3");
    assert_eq!(quantity(&app, "TM_ROCK_SMASH"), 1);
    visit(&mut app, "GoldenrodFlowerShop", "FlowerShopFloriaScript", true,
        "GoldenrodFlowerShopFloriaItReallyWasAMonText");
    visit(&mut app, "GoldenrodFlowerShop", "FlowerShopTeacherScript", true,
        "GoldenrodFlowerShopTeacherLalalaHavePlentyOfWaterText");
    assert_eq!(quantity(&app, "SQUIRTBOTTLE"), 1);
    quest_assert_save_round_trip(&mut app.world_mut().resource_mut::<BevyRuntimeShell>(),
        "sudowoodo-rock-smash-reward");
}

#[test]
fn squirtbottle_bag_use_enters_watering_without_a_second_prompt() {
    let mut shell = progression_shell_on_map_for_test("Route36");
    shell.shell.add_bag_item("SQUIRTBOTTLE", 1).unwrap();
    quest_move_beside_npc(&mut shell, "Route36", "SudowoodoScript");
    shell.shell.session_mut().overworld_mut().player.facing = Direction::Up;
    mark_runtime_snapshot_dirty(&mut shell);
    use_visible_field_bag_item_by_id(&mut shell, "SQUIRTBOTTLE".into()).unwrap();
    let mut app = menu_render_test_app(shell);
    let labels = quest_settle(&mut app, true, |shell| shell.shell.has_active_battle());
    assert!(!labels.iter().any(|label| label == "UseSquirtbottleText"),
        "bag use must enter watering directly: {labels:?}");
    assert!(labels.iter().any(|label| label == "UsedSquirtbottleText"));
    assert!(labels.iter().any(|label| label == "SudowoodoAttackedText"));
    assert_eq!(quest_item_quantity(app.world().resource::<BevyRuntimeShell>(), "SQUIRTBOTTLE"), 1);
    quest_settle(&mut app, true, |shell| shell.battle_messages.is_empty()
        && shell.visible_battle_transition.is_none() && shell.visible_battle_sliding_intro.is_none()
        && !shell.battle_enemy_send_out_pending && !shell.battle_player_send_out_pending);
    attempt_visible_battle_run(&mut app.world_mut().resource_mut::<BevyRuntimeShell>()).unwrap();
    quest_settle(&mut app, true, |shell| quest_dialogue_is_idle(shell)
        && shell.battle_messages.is_empty() && !shell.shell.has_active_battle());
    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
    assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_FOUGHT_SUDOWOODO").unwrap());
    assert!(!shell.shell.snapshot().unwrap().visible_objects.iter().any(|object|
        object.object_identifier.as_deref() == Some("ROUTE36_WEIRD_TREE")));
    assert_eq!(quest_item_quantity(&shell, "SQUIRTBOTTLE"), 1);
    quest_assert_save_round_trip(&mut shell, "sudowoodo-bag-escape");
    assert!(!shell.shell.snapshot().unwrap().visible_objects.iter().any(|object|
        object.object_identifier.as_deref() == Some("ROUTE36_WEIRD_TREE")));
}

#[test]
fn kimono_girls_all_five_battle_continuations_unlock_surf_once_and_persist() {
    fn talk_to_reward_giver(app: &mut App) -> Vec<String> {
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            quest_move_beside_npc(&mut shell, "DanceTheater", "DanceTheaterSurfGuy");
            quest_talk(&mut shell, "DanceTheaterSurfGuy");
        }
        quest_settle(app, true, quest_dialogue_is_idle)
    }
    let shell = progression_shell_on_map_for_test("DanceTheater");
    let mut app = menu_render_test_app(shell);
    for (index, (name, flag)) in [
        ("Naoko", "EVENT_BEAT_KIMONO_GIRL_NAOKO"),
        ("Sayo", "EVENT_BEAT_KIMONO_GIRL_SAYO"),
        ("Zuki", "EVENT_BEAT_KIMONO_GIRL_ZUKI"),
        ("Kuni", "EVENT_BEAT_KIMONO_GIRL_KUNI"),
        ("Miki", "EVENT_BEAT_KIMONO_GIRL_MIKI"),
    ].into_iter().enumerate() {
        let labels = talk_to_reward_giver(&mut app);
        assert!(labels.iter().any(|label| label == "SurfGuyLadGiftText"), "before battle {index}: {labels:?}");
        assert_eq!(quest_item_quantity(app.world().resource::<BevyRuntimeShell>(), "HM_SURF"), 0);
        let script = format!("TrainerKimonoGirl{name}");
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            assert!(!shell.shell.session().state().flags.is_event_flag_set("EVENT_GOT_HM03_SURF").unwrap());
            quest_move_beside_npc(&mut shell, "DanceTheater", &script);
            quest_talk(&mut shell, &script);
        }
        quest_settle(&mut app, true, |shell| shell.shell.has_active_battle());
        quest_finish_current_trainer(&mut app.world_mut().resource_mut::<BevyRuntimeShell>());
        quest_settle(&mut app, true, |shell| quest_dialogue_is_idle(shell)
            && !shell.shell.has_active_battle() && shell.battle_messages.is_empty());
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            assert!(shell.shell.session().state().flags.is_event_flag_set(flag).unwrap());
            quest_assert_save_round_trip(&mut shell, &format!("kimono-{name}"));
            quest_move_beside_npc(&mut shell, "DanceTheater", &script);
            quest_talk(&mut shell, &script);
        }
        let labels = quest_settle(&mut app, true, quest_dialogue_is_idle);
        assert!(labels.iter().any(|label| label == &format!("KimonoGirl{name}AfterBattleText")), "{name}: {labels:?}");
        assert!(!app.world().resource::<BevyRuntimeShell>().shell.has_active_battle());
    }
    let labels = talk_to_reward_giver(&mut app);
    assert!(labels.iter().any(|label| label == "SurfGuyLikeADanceText"));
    assert!(labels.iter().any(|label| label == "SurfGuySurfExplanationText"));
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, "HM_SURF"), 1);
        assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_GOT_HM03_SURF").unwrap());
        quest_assert_save_round_trip(&mut shell, "kimono-surf-reward");
    }
    let labels = talk_to_reward_giver(&mut app);
    assert!(labels.iter().any(|label| label == "SurfGuyElegantKimonoGirlsText"));
    assert!(!labels.iter().any(|label| label == "SurfGuyLikeADanceText"));
    assert_eq!(quest_item_quantity(app.world().resource::<BevyRuntimeShell>(), "HM_SURF"), 1);
}

#[test]
fn chucks_victory_unlocks_wifes_fly_reward_after_save_and_only_once() {
    fn talk_to_wife(app: &mut App) -> Vec<String> {
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            quest_move_beside_npc(&mut shell, "CianwoodCity", "CianwoodCityChucksWife");
            quest_talk(&mut shell, "CianwoodCityChucksWife");
        }
        quest_settle(app, true, quest_dialogue_is_idle)
    }
    let shell = progression_shell_on_map_for_test("CianwoodCity");
    let mut app = menu_render_test_app(shell);
    let labels = talk_to_wife(&mut app);
    assert!(labels.iter().any(|label| label == "ChucksWifeBeatChuckText"));
    assert_eq!(quest_item_quantity(app.world().resource::<BevyRuntimeShell>(), "HM_FLY"), 0);
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert!(!shell.shell.session().state().flags.is_event_flag_set("EVENT_GOT_HM02_FLY").unwrap());
        let key = shell.shell.scripted_trainer_battle_keys().into_iter()
            .find(|key| key.map_name == "CianwoodGym" && key.trainer_class == "CHUCK").unwrap();
        quest_move_beside_npc(&mut shell, "CianwoodGym", &key.source_script);
        quest_talk(&mut shell, &key.source_script);
    }
    quest_settle(&mut app, true, |shell| shell.shell.has_active_battle());
    quest_finish_current_trainer(&mut app.world_mut().resource_mut::<BevyRuntimeShell>());
    quest_settle(&mut app, true, |shell| quest_dialogue_is_idle(shell)
        && shell.battle_messages.is_empty() && !shell.shell.has_active_battle());
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_BEAT_CHUCK").unwrap());
        assert!(shell.shell.session().state().badges.johto[5]);
        assert_eq!(quest_item_quantity(&shell, "TM_DYNAMICPUNCH"), 1);
        assert_eq!(quest_item_quantity(&shell, "HM_FLY"), 0);
        quest_assert_save_round_trip(&mut shell, "chuck-before-fly");
    }
    let labels = talk_to_wife(&mut app);
    for expected in ["ChucksWifeGiveHMText", "ChucksWifeFlySpeechText", "ChucksWifeChubbyText"] {
        assert!(labels.iter().any(|label| label == expected), "missing {expected}: {labels:?}");
    }
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, "HM_FLY"), 1);
        assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_GOT_HM02_FLY").unwrap());
        quest_assert_save_round_trip(&mut shell, "chuck-fly-reward");
    }
    let labels = talk_to_wife(&mut app);
    assert!(labels.iter().any(|label| label == "ChucksWifeChubbyText"));
    assert!(!labels.iter().any(|label| label == "ChucksWifeGiveHMText"));
    assert_eq!(quest_item_quantity(app.world().resource::<BevyRuntimeShell>(), "HM_FLY"), 1);
}

#[test]
fn strength_sailor_gift_teaching_and_repeat_preserve_the_hm_after_reload() {
    const SCRIPT: &str = "OlivineCafeStrengthSailorScript";
    let mut shell = progression_shell_on_map_for_test("OlivineCafe");
    // Crystal's Totodile cannot learn Strength; use a compatible Machop.
    // An open move slot isolates acquisition/teaching from replacement choices.
    let mut learner = crate::core::models::Pokemon::new_for_tests(
        shell.runtime.data().pokemon["MACHOP"].clone(), 20, Dv::default());
    learner.moves = vec![crate::core::models::LearnedMove {
        name: "LOW_KICK".into(), current_pp: 20, pp_ups: 0 }];
    shell.shell.session_mut().state_mut().storage.party.pokemon[0] = Some(learner);
    shell.shell.session_mut().state_mut().sync_party_from_storage();
    quest_move_beside_npc(&mut shell, "OlivineCafe", SCRIPT);
    quest_talk(&mut shell, SCRIPT);
    let mut app = menu_render_test_app(shell);
    let labels = quest_settle(&mut app, true, quest_dialogue_is_idle);
    assert!(labels.iter().any(|label| label == "OlivineCafeStrengthSailorText"));
    assert!(labels.iter().any(|label| label == "OlivineCafeStrengthSailorText_GotStrength"));
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, "HM_STRENGTH"), 1);
        assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_GOT_HM04_STRENGTH").unwrap());
        quest_assert_save_round_trip(&mut shell, "strength-sailor-gift");
        shell.field_pack_pocket = Some(FieldPackPocket::TmHm);
        shell.tmhm_cursor = Some(MenuCursor { surface_id: "bag:tmhm".into(), option_index: 0 });
        open_visible_tmhm_teach_prompt(&mut shell).unwrap();
    }
    quest_settle(&mut app, true, |shell| quest_dialogue_is_idle(shell)
        && shell.shell.session().state().storage.party.pokemon[0].as_ref().unwrap()
            .moves.iter().any(|learned| learned.name == "STRENGTH"));
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, "HM_STRENGTH"), 1, "HM teaching must not consume it");
        quest_assert_save_round_trip(&mut shell, "strength-taught");
        assert!(shell.shell.session().state().storage.party.pokemon[0].as_ref().unwrap()
            .moves.iter().any(|learned| learned.name == "STRENGTH"));
        quest_move_beside_npc(&mut shell, "OlivineCafe", SCRIPT);
        quest_talk(&mut shell, SCRIPT);
    }
    let labels = quest_settle(&mut app, true, quest_dialogue_is_idle);
    assert!(labels.iter().any(|label| label == "OlivineCafeStrengthSailorText_GotStrength"));
    assert!(!labels.iter().any(|label| label == "OlivineCafeStrengthSailorText"));
    assert_eq!(quest_item_quantity(app.world().resource::<BevyRuntimeShell>(), "HM_STRENGTH"), 1);
}

#[test]
fn sprout_tower_rival_departure_and_elders_flash_reward_persist() {
    let mut shell = progression_shell_on_map_for_test("SproutTower3F");
    quest_start_coord_script(&mut shell, "SproutTower3F", "SproutTower3FRivalScene");
    let mut app = menu_render_test_app(shell);
    let labels = quest_settle(&mut app, true, |shell| quest_dialogue_is_idle(shell)
        && shell.special_boundary.is_none());
    for expected in ["SproutTowerElderLecturesRivalText", "SproutTowerRivalOnlyCareAboutStrongText",
        "SproutTowerRivalUsedEscapeRopeText"] {
        assert!(labels.iter().any(|label| label == expected), "missing {expected}: {labels:?}");
    }
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(shell.shell.session().state().scenes.map_scenes.get("SproutTower3F").map(String::as_str),
            Some("SCENE_SPROUTTOWER3F_NOOP"));
        assert!(!shell.shell.snapshot().unwrap().visible_objects.iter().any(|object|
            object.object_identifier.as_deref() == Some("SPROUTTOWER3F_RIVAL")));
        assert_eq!(quest_item_quantity(&shell, "HM_FLASH"), 0);
        quest_assert_save_round_trip(&mut shell, "sprout-rival-departure");
        assert!(!shell.shell.snapshot().unwrap().visible_objects.iter().any(|object|
            object.object_identifier.as_deref() == Some("SPROUTTOWER3F_RIVAL")));
        quest_move_beside_npc(&mut shell, "SproutTower3F", "SageLiScript");
        quest_talk(&mut shell, "SageLiScript");
    }
    let labels = quest_settle(&mut app, true, |shell| shell.shell.has_active_battle());
    assert!(labels.iter().any(|label| label == "SageLiSeenText"));
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, "HM_FLASH"), 0);
        quest_finish_current_trainer(&mut shell);
    }
    let labels = quest_settle(&mut app, true, |shell| quest_dialogue_is_idle(shell)
        && shell.battle_messages.is_empty() && !shell.shell.has_active_battle());
    for expected in ["SageLiTakeThisFlashText", "SageLiFlashExplanationText"] {
        assert!(labels.iter().any(|label| label == expected), "missing {expected}: {labels:?}");
    }
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, "HM_FLASH"), 1);
        for flag in ["EVENT_GOT_HM05_FLASH", "EVENT_BEAT_SAGE_LI"] {
            assert!(shell.shell.session().state().flags.is_event_flag_set(flag).unwrap());
        }
        quest_assert_save_round_trip(&mut shell, "sprout-flash-reward");
        quest_move_beside_npc(&mut shell, "SproutTower3F", "SageLiScript");
        quest_talk(&mut shell, "SageLiScript");
    }
    let labels = quest_settle(&mut app, true, quest_dialogue_is_idle);
    assert!(labels.iter().any(|label| label == "SageLiAfterBattleText"));
    assert!(!labels.iter().any(|label| label == "SageLiSeenText" || label == "SageLiTakeThisFlashText"));
    assert_eq!(quest_item_quantity(app.world().resource::<BevyRuntimeShell>(), "HM_FLASH"), 1);
    assert!(!app.world().resource::<BevyRuntimeShell>().shell.has_active_battle());
}

#[test]
fn every_machine_item_ball_is_collectible_and_stays_removed_after_reload() {
    let mut shell = progression_shell_on_map_for_test("IcePath1F");
    shell.shell.session_mut().state_mut().player_name = "CHRIS".into();
    let pickups = shell.runtime.data().maps.iter().flat_map(|(map, module)| {
        module.script_field_pickups.iter().filter_map(|pickup| {
            let item = pickup.item_id.as_ref()?;
            (pickup.command == "itemball" && shell.runtime.data().items[item].tmhm_index.is_some())
                .then(|| (map.clone(), pickup.clone()))
        })
    }).collect::<Vec<_>>();
    assert!(pickups.iter().any(|(_, pickup)| pickup.item_id.as_deref() == Some("HM_WATERFALL")));
    assert!(!pickups.is_empty());
    eprintln!("checking {} authored TM/HM item-ball pickups", pickups.len());
    let mut app = menu_render_test_app(shell);
    let mut collected_flags = std::collections::BTreeSet::new();
    for (map, pickup) in pickups {
        let item = pickup.item_id.as_ref().unwrap();
        let flag = pickup.event_flag.as_ref().expect("machine item ball has a persistence flag");
        let (object_id, before) = {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            quest_move_beside_npc(&mut shell, &map, &pickup.source_script);
            let object_id = shell.runtime.data().maps[&map].objects.iter()
                .find(|object| object.script == pickup.source_script).unwrap()
                .object_identifier.clone().unwrap();
            if shell.shell.session().state().flags.is_event_flag_set(flag).unwrap() {
                assert!(collected_flags.contains(flag), "{map}/{item} was not collected by this test");
                assert!(!shell.shell.snapshot().unwrap().visible_objects.iter().any(|object|
                    object.object_identifier.as_deref() == Some(object_id.as_str())),
                    "{map}/{item} shares a collected pickup and must stay hidden");
                eprintln!("passed shared pickup removal {map}/{}: {item}", pickup.source_script);
                continue;
            }
            assert!(shell.shell.snapshot().unwrap().visible_objects.iter().any(|object|
                object.object_identifier.as_deref() == Some(object_id.as_str())), "{map}/{item} must be visible");
            let before = quest_item_quantity(&shell, item);
            quest_talk(&mut shell, &pickup.source_script);
            assert!(shell.field_notice.as_deref().is_some_and(|text| text.starts_with("CHRIS found\n")),
                "{map}/{item} must present its found-item message: {:?}", shell.field_notice);
            (object_id, before)
        };
        quest_settle(&mut app, true, |shell| quest_dialogue_is_idle(shell)
            && shell.special_boundary.is_none());
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, item), before + pickup.quantity, "{map}/{item}");
        assert!(shell.shell.session().state().flags.is_event_flag_set(flag).unwrap(), "{map}/{item}");
        assert!(!shell.shell.snapshot().unwrap().visible_objects.iter().any(|object|
            object.object_identifier.as_deref() == Some(object_id.as_str())), "{map}/{item} must disappear");
        quest_assert_save_round_trip(&mut shell, &format!("machine-ball-{map}-{item}"));
        quest_move_beside_npc(&mut shell, &map, &pickup.source_script);
        assert!(!shell.shell.snapshot().unwrap().visible_objects.iter().any(|object|
            object.object_identifier.as_deref() == Some(object_id.as_str())), "{map}/{item} must stay removed");
        assert_eq!(quest_item_quantity(&shell, item), before + pickup.quantity);
        collected_flags.insert(flag.clone());
        eprintln!("passed {map}/{}: {item}", pickup.source_script);
    }
}

fn tm_mart_key_for_test(app: &mut App, key: KeyCode) {
    // Read the current message before pressing; retain real menu input and frame updates.
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        for _ in 0..100 { tick_visible_field_text_reveal(&mut shell, true).unwrap(); }
        mark_runtime_presentation_dirty(&mut shell);
    }
    press_key_for_runtime_hotkey_app(app, key);
}

#[test]
fn tm_mart_unlocks_and_visible_transactions_preserve_money_and_inventory() {
    // Goldenrod's four inventories depend on the original gift flags, not badges.
    for (map, script, mart, headbutt, rock_smash) in [
        ("GoldenrodDeptStore5F", "GoldenrodDeptStore5FClerkScript", "MART_GOLDENROD_5F_1", false, false),
        ("GoldenrodDeptStore5F", "GoldenrodDeptStore5FClerkScript", "MART_GOLDENROD_5F_2", true, false),
        ("GoldenrodDeptStore5F", "GoldenrodDeptStore5FClerkScript", "MART_GOLDENROD_5F_3", false, true),
        ("GoldenrodDeptStore5F", "GoldenrodDeptStore5FClerkScript", "MART_GOLDENROD_5F_4", true, true),
        ("CeladonDeptStore3F", "CeladonDeptStore3FClerkScript", "MART_CELADON_3F", false, false),
    ] {
        let mut shell = progression_shell_on_map_for_test(map);
        for (flag, value) in [("EVENT_GOT_TM02_HEADBUTT", headbutt), ("EVENT_GOT_TM08_ROCK_SMASH", rock_smash)] {
            shell.shell.session_mut().state_mut().flags.set_event_flag(flag, value).unwrap();
        }
        quest_move_beside_npc(&mut shell, map, script);
        if mart == "MART_GOLDENROD_5F_1" {
            if let Ok(directory) = std::env::var("POKEGEAR_PC_RENDER_DIR") {
                std::fs::create_dir_all(&directory).unwrap();
                shell.shell.session_mut().state_mut().money = 50000;
                shell.shell.session_mut().state_mut().player_name = "TEST".into();
                let original_state = shell.shell.session().state().clone();
                let original_overworld = shell.shell.session().overworld().clone();
                let runtime = shell.runtime.clone();
                let (state, overworld) = shell.shell.session_mut().state_and_overworld_mut();
                runtime.data().transition_overworld_session(state, overworld, map, TilePosition::new(8, 3),
                    crate::core::systems::map_context::SpawnMemoryUpdate::Preserve, &runtime.music_ids()).unwrap();
                shell.shell.session_mut().overworld_mut().player.facing = Direction::Down;
                assert_eq!(shell.shell.current_overworld_interaction_checked().unwrap().unwrap().script, script,
                    "browser fixture must reach the clerk from the customer side of the counter");
                shell.shell.save(PathBuf::from(directory).join("tm-mart-browser.crystalsave")).unwrap();
                let (state, overworld) = shell.shell.session_mut().state_and_overworld_mut();
                *state = original_state;
                *overworld = original_overworld;
            }
        }
        quest_talk(&mut shell, script);
        let mut app = menu_render_test_app(shell);
        quest_settle(&mut app, true, |shell| shell.shop_top_cursor.is_some() && shell.shop_welcome_seen && shell.shop_notice.is_none() && shell.field_notice.is_none());
        let inventory = {
            let shell = app.world().resource::<BevyRuntimeShell>();
            let shop = shell.shell.snapshot().unwrap().pending_shop.unwrap();
            assert_eq!(shop.mart_id, mart);
            let mut expected = if map == "CeladonDeptStore3F" {
                vec!["TM_HIDDEN_POWER", "TM_SUNNY_DAY", "TM_PROTECT", "TM_RAIN_DANCE", "TM_SANDSTORM"]
            } else {
                vec!["TM_THUNDERPUNCH", "TM_FIRE_PUNCH", "TM_ICE_PUNCH"]
            };
            if headbutt { expected.push("TM_HEADBUTT"); }
            if rock_smash { expected.push("TM_ROCK_SMASH"); }
            assert_eq!(shop.inventory, expected);
            shop.inventory
        };
        tm_mart_key_for_test(&mut app, KeyCode::KeyZ); // BUY
        for (index, item) in inventory.iter().enumerate() {
            if index > 0 { tm_mart_key_for_test(&mut app, KeyCode::ArrowDown); }
            let price = {
                let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
                assert_eq!(shell.menu_cursor.as_ref().unwrap().option_index, index);
                assert_eq!(quest_item_quantity(&shell, item), 0);
                let price = u32::from(shell.runtime.data().items[item].price);
                assert!(price > 0);
                shell.shell.session_mut().state_mut().money = price * 2;
                mark_runtime_snapshot_dirty(&mut shell);
                price
            };
            // Quantity selection and declining Yes/No must leave both balances unchanged.
            tm_mart_key_for_test(&mut app, KeyCode::KeyZ);
            tm_mart_key_for_test(&mut app, KeyCode::KeyZ);
            {
                let shell = app.world().resource::<BevyRuntimeShell>();
                assert_eq!(shell.shop_quantity.as_ref().unwrap().confirmation, Some(true));
                assert_eq!(quest_item_quantity(shell, item), 0);
                assert_eq!(shell.shell.session().state().money, price * 2);
            }
            tm_mart_key_for_test(&mut app, KeyCode::ArrowDown);
            tm_mart_key_for_test(&mut app, KeyCode::KeyZ);
            {
                let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
                assert!(shell.shop_quantity.is_none());
                assert_eq!(quest_item_quantity(&shell, item), 0);
                assert_eq!(shell.shell.session().state().money, price * 2);
                shell.shell.session_mut().state_mut().money = price - 1;
                mark_runtime_snapshot_dirty(&mut shell);
            }
            for _ in 0..3 { tm_mart_key_for_test(&mut app, KeyCode::KeyZ); }
            {
                let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
                assert_eq!(shell.shop_notice.as_deref(), Some("You don't have\nenough money."));
                assert_eq!(quest_item_quantity(&shell, item), 0);
                assert_eq!(shell.shell.session().state().money, price - 1);
                shell.shell.session_mut().state_mut().money = price * 2;
                mark_runtime_snapshot_dirty(&mut shell);
            }
            tm_mart_key_for_test(&mut app, KeyCode::KeyZ); // dismiss refusal
            tm_mart_key_for_test(&mut app, KeyCode::KeyZ); // quantity
            tm_mart_key_for_test(&mut app, KeyCode::ArrowUp); // two
            assert_eq!(app.world().resource::<BevyRuntimeShell>().shop_quantity.as_ref().unwrap().quantity, 2);
            tm_mart_key_for_test(&mut app, KeyCode::KeyZ); // confirm prompt
            tm_mart_key_for_test(&mut app, KeyCode::KeyZ); // purchase
            {
                let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
                assert_eq!(shell.shop_notice.as_deref(), Some("Here you are.\nThank you!"));
                assert_eq!(quest_item_quantity(&shell, item), 2);
                assert_eq!(shell.shell.session().state().money, 0);
                shell.shell.add_bag_item(item, 97).unwrap(); // stage the stack boundary
                shell.shell.session_mut().state_mut().money = price;
                mark_runtime_snapshot_dirty(&mut shell);
            }
            tm_mart_key_for_test(&mut app, KeyCode::KeyZ);
            for _ in 0..3 { tm_mart_key_for_test(&mut app, KeyCode::KeyZ); }
            {
                let shell = app.world().resource::<BevyRuntimeShell>();
                assert_eq!(shell.shop_notice.as_deref(), Some("You can't carry\nany more items."));
                assert_eq!(quest_item_quantity(shell, item), 99);
                assert_eq!(shell.shell.session().state().money, price);
                assert!(shell.last_error.is_none(), "{:?}", shell.last_error);
            }
            tm_mart_key_for_test(&mut app, KeyCode::KeyZ);
            eprintln!("passed {mart}/{item}: decline, insufficient money, buy two, full stack");
        }
        // Exit both the item list and clerk before persistence checks.
        tm_mart_key_for_test(&mut app, KeyCode::KeyX);
        quest_settle(&mut app, true, |shell| shell.shop_top_cursor.is_some() && shell.shop_welcome_seen && shell.shop_notice.is_none() && shell.field_notice.is_none());
        tm_mart_key_for_test(&mut app, KeyCode::KeyX);
        quest_settle(&mut app, true, |shell| quest_dialogue_is_idle(shell)
            && shell.shell.snapshot().unwrap().pending_shop.is_none());
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        let money = shell.shell.session().state().money;
        quest_assert_save_round_trip(&mut shell, mart);
        assert_eq!(shell.shell.session().state().money, money);
        for item in inventory { assert_eq!(quest_item_quantity(&shell, &item), 99); }
    }
}

#[test]
fn sunday_happiness_tm_gifts_follow_thresholds_and_remain_claimed_after_reload() {
    const MAP: &str = "GoldenrodDeptStore5F";
    const SCRIPT: &str = "GoldenrodDeptStore5FReceptionistScript";
    const OBJECT: &str = "GOLDENRODDEPTSTORE5F_RECEPTIONIST";
    const CLAIMED: &str = "ENGINE_GOLDENROD_DEPT_STORE_TM27_RETURN";
    for happiness in [0, 49, 50, 149, 150, 255] {
        let mut shell = progression_shell_on_map_for_test(MAP);
        // A civil Sunday/Monday pair keeps all clock registers consistent for saves.
        shell.shell.session_mut().state_mut().time.update_server_datetime(
            crate::core::systems::time::GameDate { year: 2000, month: 1, day: 3 }, 12, 0, 0);
        quest_move_beside_npc(&mut shell, MAP, SCRIPT);
        assert!(!shell.shell.snapshot().unwrap().visible_objects.iter().any(|object|
            object.object_identifier.as_deref() == Some(OBJECT)), "receptionist must be absent Monday");
        {
            let state = shell.shell.session_mut().state_mut();
            state.time.update_server_datetime(
                crate::core::systems::time::GameDate { year: 2000, month: 1, day: 2 }, 12, 0, 0);
            state.storage.party.pokemon[0].as_mut().unwrap().happiness = happiness;
        }
        quest_move_beside_npc(&mut shell, MAP, SCRIPT);
        assert!(shell.shell.snapshot().unwrap().visible_objects.iter().any(|object|
            object.object_identifier.as_deref() == Some(OBJECT)), "receptionist must appear Sunday");
        assert!(!shell.shell.session().state().flags.is_engine_flag_set(CLAIMED).unwrap());
        quest_talk(&mut shell, SCRIPT);
        let mut app = menu_render_test_app(shell);
        let labels = quest_settle(&mut app, true, |shell| quest_dialogue_is_idle(shell)
            && shell.special_boundary.is_none());
        let (expected, item) = if happiness < 50 {
            ("GoldenrodDeptStore5FReceptionistItLooksEvilHowAboutThisTMText", Some("TM_FRUSTRATION"))
        } else if happiness < 150 {
            ("GoldenrodDeptStore5FReceptionistItsAdorableText", None)
        } else {
            ("GoldenrodDeptStore5FReceptionistThisMoveShouldBePerfectText", Some("TM_RETURN"))
        };
        assert!(labels.iter().any(|label| label == expected), "happiness {happiness}: {labels:?}");
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            for candidate in ["TM_RETURN", "TM_FRUSTRATION"] {
                assert_eq!(quest_item_quantity(&shell, candidate), u16::from(item == Some(candidate)));
            }
            assert_eq!(shell.shell.session().state().flags.is_engine_flag_set(CLAIMED).unwrap(), item.is_some());
            quest_assert_save_round_trip(&mut shell, &format!("sunday-tm-{happiness}"));
            // An unawarded middle-range visit must not block a later eligible visit.
            // A claimed gift must block the other gift too, even after happiness changes.
            shell.shell.session_mut().state_mut().storage.party.pokemon[0].as_mut().unwrap().happiness =
                if happiness >= 150 { 0 } else { 255 };
            quest_move_beside_npc(&mut shell, MAP, SCRIPT);
            quest_talk(&mut shell, SCRIPT);
        }
        let repeat = quest_settle(&mut app, true, |shell| quest_dialogue_is_idle(shell)
            && shell.special_boundary.is_none());
        let shell = app.world().resource::<BevyRuntimeShell>();
        if item.is_some() {
            assert!(repeat.iter().any(|label|
                label == "GoldenrodDeptStore5FReceptionistThereAreTMsPerfectForMonText"));
            for candidate in ["TM_RETURN", "TM_FRUSTRATION"] {
                assert_eq!(quest_item_quantity(shell, candidate), u16::from(item == Some(candidate)));
            }
        } else {
            assert!(repeat.iter().any(|label|
                label == "GoldenrodDeptStore5FReceptionistThisMoveShouldBePerfectText"));
            assert_eq!(quest_item_quantity(shell, "TM_RETURN"), 1);
            assert_eq!(quest_item_quantity(shell, "TM_FRUSTRATION"), 0);
        }
        assert!(shell.shell.session().state().flags.is_engine_flag_set(CLAIMED).unwrap());
        let before_return = quest_item_quantity(shell, "TM_RETURN");
        let before_frustration = quest_item_quantity(shell, "TM_FRUSTRATION");
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            let runtime = shell.runtime.clone();
            runtime.data().update_clock_from_datetime(shell.shell.session_mut().state_mut(),
                crate::core::systems::time::GameDate { year: 2000, month: 1, day: 9 },
                12, 0, 0, &mut crate::core::random::ReplayDivider::new([0, 0])).unwrap();
            assert!(!shell.shell.session().state().flags.is_engine_flag_set(CLAIMED).unwrap(),
                "the daily reset must allow a gift on the next Sunday");
            quest_move_beside_npc(&mut shell, MAP, SCRIPT);
            quest_talk(&mut shell, SCRIPT);
        }
        quest_settle(&mut app, true, |shell| quest_dialogue_is_idle(shell)
            && shell.special_boundary.is_none());
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, "TM_RETURN"), before_return + u16::from(happiness < 150));
        assert_eq!(quest_item_quantity(&shell, "TM_FRUSTRATION"), before_frustration + u16::from(happiness >= 150));
        assert!(shell.shell.session().state().flags.is_engine_flag_set(CLAIMED).unwrap());
        quest_assert_save_round_trip(&mut shell, &format!("next-sunday-tm-{happiness}"));
        eprintln!("passed Sunday TM happiness {happiness}: weekday visibility, gift, reload, repeat, next week");
    }
}

#[test]
fn sunday_happiness_tm_full_stack_can_retry_without_losing_the_gift() {
    for (happiness, item) in [(0, "TM_FRUSTRATION"), (255, "TM_RETURN")] {
        let mut shell = progression_shell_on_map_for_test("GoldenrodDeptStore5F");
        {
            let state = shell.shell.session_mut().state_mut();
            state.time.update_server_datetime(
                crate::core::systems::time::GameDate { year: 2000, month: 1, day: 2 }, 12, 0, 0);
            state.storage.party.pokemon[0].as_mut().unwrap().happiness = happiness;
        }
        shell.shell.add_bag_item(item, 99).unwrap();
        quest_move_beside_npc(&mut shell, "GoldenrodDeptStore5F", "GoldenrodDeptStore5FReceptionistScript");
        quest_talk(&mut shell, "GoldenrodDeptStore5FReceptionistScript");
        let mut app = menu_render_test_app(shell);
        quest_settle(&mut app, true, |shell| quest_dialogue_is_idle(shell)
            && shell.special_boundary.is_none());
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            assert_eq!(quest_item_quantity(&shell, item), 99);
            assert!(!shell.shell.session().state().flags
                .is_engine_flag_set("ENGINE_GOLDENROD_DEPT_STORE_TM27_RETURN").unwrap());
            quest_assert_save_round_trip(&mut shell, &format!("sunday-full-{item}"));
            shell.shell.remove_bag_item(item, 1).unwrap(); // make room for a retry
            quest_move_beside_npc(&mut shell, "GoldenrodDeptStore5F", "GoldenrodDeptStore5FReceptionistScript");
            quest_talk(&mut shell, "GoldenrodDeptStore5FReceptionistScript");
        }
        quest_settle(&mut app, true, |shell| quest_dialogue_is_idle(shell)
            && shell.special_boundary.is_none());
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        assert_eq!(quest_item_quantity(&shell, item), 99);
        assert!(shell.shell.session().state().flags
            .is_engine_flag_set("ENGINE_GOLDENROD_DEPT_STORE_TM27_RETURN").unwrap());
        quest_assert_save_round_trip(&mut shell, &format!("sunday-retry-{item}"));
    }
}

fn game_corner_tm_talk_for_test(shell: &mut BevyRuntimeShell, map: &str, script: &str) {
    if map == "CeladonGameCornerPrizeRoom" {
        let runtime = shell.runtime.clone();
        let (state, overworld) = shell.shell.session_mut().state_and_overworld_mut();
        runtime.data().transition_overworld_session(state, overworld, map, TilePosition::new(2, 2),
            crate::core::systems::map_context::SpawnMemoryUpdate::Preserve, &runtime.music_ids()).unwrap();
        reset_visible_navigation_state(shell);
        shell.shell.session_mut().overworld_mut().player.facing = Direction::Up;
        let interaction = shell.shell.current_overworld_interaction_checked().unwrap().unwrap();
        assert_eq!(interaction.script, script);
        dispatch_visible_overworld_interaction(shell, interaction, "tm_prize_regression").unwrap();
    } else {
        quest_move_beside_npc(shell, map, script);
        quest_talk(shell, script);
    }
}

#[test]
fn game_corner_tm_prizes_check_coins_refusal_capacity_and_save() {
    for (map, script, prizes) in [
        ("GoldenrodGameCorner", "GoldenrodGameCornerTMVendorScript",
            [("TM_THUNDER", 5500), ("TM_BLIZZARD", 5500), ("TM_FIRE_BLAST", 5500)]),
        ("CeladonGameCornerPrizeRoom", "CeladonGameCornerPrizeRoomTMVendor",
            [("TM_DOUBLE_TEAM", 1500), ("TM_PSYCHIC_M", 3500), ("TM_HYPER_BEAM", 7500)]),
    ] {
        for (index, (item, price)) in prizes.into_iter().enumerate() {
            let mut shell = progression_shell_on_map_for_test(map);
            // Enter the interaction after a real overworld frame, as normal travel does.
            shell.shell.tick(std::iter::empty::<GameButton>()).unwrap();
            shell.shell.session_mut().state_mut().coins = price;
            game_corner_tm_talk_for_test(&mut shell, map, script);
            let mut app = menu_render_test_app(shell);
            let labels = quest_settle(&mut app, true, quest_dialogue_is_idle);
            assert!(labels.iter().any(|label| label.contains("NoCoinCase")), "{map}: {labels:?}");
            assert_eq!(quest_item_quantity(app.world().resource::<BevyRuntimeShell>(), item), 0);
            assert_eq!(app.world().resource::<BevyRuntimeShell>().shell.session().state().coins, price);
            app.world_mut().resource_mut::<BevyRuntimeShell>().shell.add_bag_item("COIN_CASE", 1).unwrap();
            for scenario in ["poor", "decline", "full", "buy"] {
                {
                    let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
                    shell.shell.session_mut().state_mut().coins = if scenario == "poor" { price - 1 } else { price };
                    if scenario == "full" { shell.shell.add_bag_item(item, 99).unwrap(); }
                    if scenario == "buy" { shell.shell.remove_bag_item(item, 99).unwrap(); }
                    game_corner_tm_talk_for_test(&mut shell, map, script);
                }
                quest_settle(&mut app, true, |shell|
                    visible_menu_has_selectable_options(&shell.shell.snapshot().unwrap()));
                for _ in 0..index { tm_mart_key_for_test(&mut app, KeyCode::ArrowDown); }
                tm_mart_key_for_test(&mut app, KeyCode::KeyZ);
                let labels = quest_settle(&mut app, scenario != "decline", |shell| {
                    if scenario == "buy" {
                        quest_item_quantity(shell, item) == 1
                            && visible_menu_has_selectable_options(&shell.shell.snapshot().unwrap())
                    } else { quest_dialogue_is_idle(shell) }
                });
                let shell = app.world().resource::<BevyRuntimeShell>();
                assert_eq!(shell.shell.session().state().coins,
                    if scenario == "buy" { 0 } else if scenario == "poor" { price - 1 } else { price },
                    "{map}/{item}/{scenario}: {labels:?}");
                assert_eq!(quest_item_quantity(shell, item),
                    if scenario == "buy" { 1 } else if scenario == "full" { 99 } else { 0 });
                if scenario == "buy" {
                    assert!(labels.iter().any(|label| label.contains("HereYouGo")), "{labels:?}");
                    tm_mart_key_for_test(&mut app, KeyCode::KeyX);
                    quest_settle(&mut app, true, quest_dialogue_is_idle);
                }
                let coins = app.world().resource::<BevyRuntimeShell>().shell.session().state().coins;
                quest_assert_save_round_trip(&mut app.world_mut().resource_mut::<BevyRuntimeShell>(),
                    &format!("tm-prize-{map}-{item}-{scenario}"));
                assert_eq!(app.world().resource::<BevyRuntimeShell>().shell.session().state().coins, coins);
                eprintln!("passed {map}/{item}: {scenario}");
            }
        }
    }
}

#[test]
fn kurt_apricorn_orders_wait_until_next_day_and_preserve_rewards_after_reload() {
    for (apricorn, ball) in [
        ("RED_APRICORN", "LEVEL_BALL"), ("BLU_APRICORN", "LURE_BALL"),
        ("YLW_APRICORN", "MOON_BALL"), ("GRN_APRICORN", "FRIEND_BALL"),
        ("WHT_APRICORN", "FAST_BALL"), ("BLK_APRICORN", "HEAVY_BALL"),
        ("PNK_APRICORN", "LOVE_BALL"),
    ] {
        let full_quantity = 99 * crate::core::models::BALL_POCKET_CAPACITY as u16;
        let mut shell = progression_shell_on_map_for_test("KurtsHouse");
        let runtime = shell.runtime.clone();
        runtime.data().update_clock_from_datetime(shell.shell.session_mut().state_mut(),
            crate::core::systems::time::GameDate::new(2000, 1, 2), 12, 0, 0,
            &mut crate::core::random::ReplayDivider::new([0; 32])).unwrap();
        for event in ["EVENT_CLEARED_SLOWPOKE_WELL", "EVENT_KURT_GAVE_YOU_LURE_BALL"] {
            shell.shell.session_mut().state_mut().flags.set_event_flag(event, true).unwrap();
        }
        shell.shell.session_mut().state_mut().player_name = "TEST".into();
        shell.shell.add_bag_item(apricorn, 3).unwrap();
        quest_move_beside_npc(&mut shell, "KurtsHouse", "Kurt1");
        kurt_browser_fixture_for_test(&mut shell, apricorn, "order");
        quest_talk(&mut shell, "Kurt1");
        let mut app = menu_render_test_app(shell);
        quest_settle(&mut app, true, |shell| shell.kurt_apricorn_cursor.is_some());
        if apricorn == "RED_APRICORN" {
            for _ in 0..3 { app.update(); }
            let text = rendered_mart_text_for_test(&mut app);
            assert!(text.contains("Which APRICORN") && text.contains("should I use?"), "{text}");
            save_live_menu_lcd_for_test(app.world_mut(), "kurt-apricorn-list.png");
        }
        if apricorn == "RED_APRICORN" {
            tm_mart_key_for_test(&mut app, KeyCode::ArrowDown);
            tm_mart_key_for_test(&mut app, KeyCode::KeyZ);
        } else { tm_mart_key_for_test(&mut app, KeyCode::KeyX); }
        quest_settle(&mut app, true, quest_dialogue_is_idle);
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            assert_eq!(quest_item_quantity(&shell, apricorn), 3);
            assert!(!shell.shell.session().state().flags.is_engine_flag_set("ENGINE_KURT_MAKING_BALLS").unwrap());
            quest_move_beside_npc(&mut shell, "KurtsHouse", "Kurt1");
            quest_talk(&mut shell, "Kurt1");
        }
        quest_settle(&mut app, true, |shell| shell.kurt_apricorn_cursor.is_some());
        tm_mart_key_for_test(&mut app, KeyCode::KeyZ);
        assert_eq!(app.world().resource::<BevyRuntimeShell>().kurt_apricorn_quantity, Some(1));
        if apricorn == "RED_APRICORN" {
            for _ in 0..3 { app.update(); }
            let text = rendered_mart_text_for_test(&mut app);
            assert!(text.contains("How many should I") && text.contains("make?"), "{text}");
            save_live_menu_lcd_for_test(app.world_mut(), "kurt-apricorn-quantity.png");
        }
        tm_mart_key_for_test(&mut app, KeyCode::KeyX);
        assert!(app.world().resource::<BevyRuntimeShell>().kurt_apricorn_cursor.is_some());
        assert_eq!(app.world().resource::<BevyRuntimeShell>().kurt_apricorn_quantity, None);
        assert_eq!(quest_item_quantity(app.world().resource::<BevyRuntimeShell>(), apricorn), 3);
        tm_mart_key_for_test(&mut app, KeyCode::KeyZ);
        tm_mart_key_for_test(&mut app, KeyCode::ArrowUp);
        tm_mart_key_for_test(&mut app, KeyCode::ArrowUp);
        assert_eq!(app.world().resource::<BevyRuntimeShell>().kurt_apricorn_quantity, Some(3));
        tm_mart_key_for_test(&mut app, KeyCode::KeyZ);
        quest_settle(&mut app, true, quest_dialogue_is_idle);
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            assert_eq!(quest_item_quantity(&shell, apricorn), 0);
            assert!(shell.shell.session().state().flags.is_engine_flag_set("ENGINE_KURT_MAKING_BALLS").unwrap());
            quest_assert_save_round_trip(&mut shell, &format!("kurt-order-{apricorn}"));
            quest_move_beside_npc(&mut shell, "KurtsHouse", "Kurt2");
            assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_KURTS_HOUSE_KURT_1").unwrap());
            assert!(!shell.shell.session().state().flags.is_event_flag_set("EVENT_KURTS_HOUSE_KURT_2").unwrap());
            quest_talk(&mut shell, "Kurt2");
        }
        quest_settle(&mut app, true, quest_dialogue_is_idle);
        assert_eq!(quest_item_quantity(app.world().resource::<BevyRuntimeShell>(), ball), 0);
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            runtime.data().update_clock_from_datetime(shell.shell.session_mut().state_mut(),
                crate::core::systems::time::GameDate::new(2000, 1, 3), 0, 0, 0,
                &mut crate::core::random::ReplayDivider::new([0; 32])).unwrap();
            quest_move_beside_npc(&mut shell, "KurtsHouse", "Kurt1");
            assert!(!shell.shell.session().state().flags.is_event_flag_set("EVENT_KURTS_HOUSE_KURT_1").unwrap());
            assert!(shell.shell.session().state().flags.is_event_flag_set("EVENT_KURTS_HOUSE_KURT_2").unwrap());
            shell.shell.add_bag_item(ball, full_quantity).unwrap();
            quest_talk(&mut shell, "Kurt1");
        }
        quest_settle(&mut app, true, quest_dialogue_is_idle);
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            assert_eq!(quest_item_quantity(&shell, ball), full_quantity);
            assert!(shell.shell.session().state().flags.is_event_flag_set(&format!("EVENT_GAVE_KURT_{apricorn}")).unwrap());
            quest_assert_save_round_trip(&mut shell, &format!("kurt-full-{apricorn}"));
            for _ in 0..crate::core::models::BALL_POCKET_CAPACITY {
                shell.shell.remove_bag_item(ball, 99).unwrap();
            }
            quest_move_beside_npc(&mut shell, "KurtsHouse", "Kurt1");
            kurt_browser_fixture_for_test(&mut shell, apricorn, "ready");
            quest_talk(&mut shell, "Kurt1");
        }
        let labels = quest_settle(&mut app, true, quest_dialogue_is_idle);
        assert!(labels.iter().any(|label| label == "KurtsHouseKurtJustFinishedYourBallText"), "{labels:?}");
        {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            assert_eq!(quest_item_quantity(&shell, ball), 3);
            quest_assert_save_round_trip(&mut shell, &format!("kurt-reward-{apricorn}"));
            quest_move_beside_npc(&mut shell, "KurtsHouse", "Kurt1");
            quest_talk(&mut shell, "Kurt1");
        }
        quest_settle(&mut app, true, quest_dialogue_is_idle);
        assert_eq!(quest_item_quantity(app.world().resource::<BevyRuntimeShell>(), ball), 3);
        eprintln!("passed Kurt {apricorn}: quantity, same-day wait, next-day collection, save and repeat");
    }
}

fn kurt_browser_fixture_for_test(shell: &mut BevyRuntimeShell, apricorn: &str, phase: &str) {
    if apricorn != "RED_APRICORN" { return; }
    let Some(directory) = std::env::var_os("POKEGEAR_PC_RENDER_DIR") else { return; };
    let directory = PathBuf::from(directory);
    std::fs::create_dir_all(&directory).unwrap();
    shell.shell.session_mut().overworld_mut().player.facing = Direction::Up;
    assert_eq!(shell.shell.current_overworld_interaction_checked().unwrap().unwrap().script, "Kurt1");
    shell.shell.save(directory.join(format!("kurt-{phase}-browser.crystalsave"))).unwrap();
}


#[test]
fn kurt_full_apricorn_list_scrolls_and_cancels_without_consuming_items() {
    let mut shell = progression_shell_on_map_for_test("KurtsHouse");
    for event in ["EVENT_CLEARED_SLOWPOKE_WELL", "EVENT_KURT_GAVE_YOU_LURE_BALL"] {
        shell.shell.session_mut().state_mut().flags.set_event_flag(event, true).unwrap();
    }
    for item in KURT_APRICORN_ORDER {
        shell.shell.add_bag_item(item, 3).unwrap();
    }
    quest_move_beside_npc(&mut shell, "KurtsHouse", "Kurt1");
    quest_talk(&mut shell, "Kurt1");
    let mut app = menu_render_test_app(shell);
    quest_settle(&mut app, true, |shell| shell.kurt_apricorn_cursor.is_some());
    for _ in 0..3 { app.update(); }
    save_live_menu_lcd_for_test(app.world_mut(), "kurt-all-apricorns-top.png");
    for _ in 0..6 { tm_mart_key_for_test(&mut app, KeyCode::ArrowDown); }
    let selected = rendered_mart_text_for_test(&mut app);
    assert!(selected.contains("PNK APRICORN"), "{selected}");
    save_live_menu_lcd_for_test(app.world_mut(), "kurt-all-apricorns-bottom.png");
    tm_mart_key_for_test(&mut app, KeyCode::KeyZ);
    let quantity = rendered_mart_text_for_test(&mut app);
    assert!(quantity.contains("PNK APRICORN") && quantity.contains("How many"), "{quantity}");
    save_live_menu_lcd_for_test(app.world_mut(), "kurt-all-apricorns-quantity.png");
    tm_mart_key_for_test(&mut app, KeyCode::KeyX);
    tm_mart_key_for_test(&mut app, KeyCode::ArrowDown);
    let cancel = rendered_mart_text_for_test(&mut app);
    assert!(cancel.contains("CANCEL"), "{cancel}");
    tm_mart_key_for_test(&mut app, KeyCode::KeyZ);
    quest_settle(&mut app, true, quest_dialogue_is_idle);
    let shell = app.world().resource::<BevyRuntimeShell>();
    for item in KURT_APRICORN_ORDER {
        assert_eq!(quest_item_quantity(shell, item), 3, "{item}");
    }
    assert!(!shell.shell.session().state().flags.is_engine_flag_set("ENGINE_KURT_MAKING_BALLS").unwrap());
}
