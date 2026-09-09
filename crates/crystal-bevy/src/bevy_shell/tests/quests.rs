fn quest_move_beside_npc(shell: &mut BevyRuntimeShell, map: &str, script: &str) {
    let runtime = shell.shell.runtime().clone();
    let object = runtime.data().maps[map]
        .objects
        .iter()
        .find(|object| object.script == script)
        .unwrap()
        .clone();
    let tile = TilePosition::new(object.x as i16, object.y as i16 + 1);
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
        facing: Direction::Up,
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
        press_key_for_runtime_hotkey_app(app, KeyCode::KeyZ);
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
        quest_talk(&mut shell, "OlivineLighthouseJasmine");
    }
    let labels = quest_settle(&mut app, true, quest_dialogue_is_idle);
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(quest_item_quantity(shell, "SECRETPOTION"), 0);
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
