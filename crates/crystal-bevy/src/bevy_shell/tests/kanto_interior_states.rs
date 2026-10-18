// External-pack regressions exercise production callbacks and controller input.
include!("kanto_pokecom_services.rs");

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn kanto_owned_bed_variants_use_pc_input_and_publish_review_fixtures() {
    let pack = PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root = AssetRoot::new(pack.parent().unwrap());
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap();
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).unwrap();
    let beds = ["DECO_FEATHERY_BED", "DECO_PINK_BED", "DECO_POLKADOT_BED", "DECO_PIKACHU_BED"];
    for (choice, bed) in beds.iter().enumerate().skip(1) {
        let mut shell = initialize_bevy_runtime_shell(root.clone(), runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: runtime.title_new_game_spawn_identifier().unwrap(),
                map_name: "PlayersHouse2F".into(), tile_x: 2, tile_y: 2,
            }, BevyShellConfig::default()).unwrap();
        // Ownership is an initial visual fixture, not proof of Mystery Gift
        // acquisition. Setup, room reload, menu closure and walking use input.
        for flag in ["EVENT_DECO_BED_2", "EVENT_DECO_BED_3", "EVENT_DECO_BED_4"] {
            shell.shell.set_script_flag_for_smoke(flag).unwrap();
        }
        let mut controller = VisibleShellController { shell };
        settle_visible_shell_until_input(&mut controller.shell).unwrap();
        controller.press(GameButton::Up).unwrap();
        assert_eq!(controller.snapshot().unwrap().overworld.tile, TilePosition::new(2, 2));
        controller.press(GameButton::A).unwrap();
        for _ in 0..256 {
            if controller.shell.player_pc_action_cursor.is_some()
                && controller.shell.field_notice.is_none() && controller.shell.pc_notice.is_none() { break; }
            controller.press(GameButton::A).unwrap();
        }
        assert!(controller.shell.player_pc_action_cursor.is_some(), "bedroom PC must open");
        for _ in 0..4 { controller.press(GameButton::Down).unwrap(); }
        controller.press(GameButton::A).unwrap();
        assert!(matches!(controller.shell.decoration_menu.as_ref().map(|m| &m.phase),
            Some(VisibleDecorationMenuPhase::Categories { categories, .. })
                if categories.first() == Some(&DecorationCategory::Bed)));
        controller.press(GameButton::A).unwrap();
        assert!(matches!(controller.shell.decoration_menu.as_ref().map(|m| &m.phase),
            Some(VisibleDecorationMenuPhase::Decorations { decorations, .. })
                if decorations.iter().map(|d| d.id.as_str()).collect::<Vec<_>>() == beds));
        for _ in 0..choice { controller.press(GameButton::Down).unwrap(); }
        controller.press(GameButton::A).unwrap();
        for _ in 0..256 {
            if controller.shell.pc_notice.is_none() { break; }
            controller.press(GameButton::A).unwrap();
        }
        assert!(controller.shell.pc_notice.is_none(), "setup notices must finish");
        assert_eq!(controller.snapshot().unwrap().script_events.memory.get("wDecoBed").map(String::as_str), Some(*bed));
        controller.press(GameButton::B).unwrap();
        controller.press(GameButton::B).unwrap();
        settle_visible_shell_until_input(&mut controller.shell).unwrap();
        assert!(controller.shell.decoration_menu.is_none() && controller.shell.player_pc_action_cursor.is_none());
        let closed = controller.snapshot().unwrap();
        assert!(controller.shell.active_script_cursor.is_none()
            && !closed.ui.text_window_open && closed.ui.pending_text_wait.is_none(),
            "{bed}: PC reload must finish before walking: ui={:?} cursor={:?} special={:?} load={:?} warp={:?} fade={:?} action={:?}",
            closed.ui, controller.shell.active_script_cursor, controller.shell.special_boundary,
            closed.script_events.pending_map_load, closed.script_events.pending_script_warp,
            controller.shell.screen_fade, controller.shell.last_runtime_action);
        // A direction first turns the trainer; continue ordinary joypad
        // frames until each approach tile is actually reached.
        for (button, target) in [(GameButton::Down, TilePosition::new(2, 4)),
            (GameButton::Left, TilePosition::new(1, 4))] {
            for _ in 0..16 {
                if controller.snapshot().unwrap().overworld.tile == target { break; }
                controller.press(button).unwrap();
            }
            assert_eq!(controller.snapshot().unwrap().overworld.tile, target,
                "{bed}: room reload must release normal walking input");
        }
        let snapshot = controller.snapshot().unwrap();
        assert_eq!(snapshot.overworld.map_name, "PlayersHouse2F");
        assert_eq!(snapshot.overworld.tile, TilePosition::new(1, 4), "room reload must return walking input");
        assert_eq!(snapshot.script_events.memory.get("wDecoBed").map(String::as_str), Some(*bed));
        if let Some(directory) = std::env::var_os("KANTO_INTERIOR_STATE_REVIEW_DIR") {
            let directory = PathBuf::from(directory).join("owned-bed-variants");
            std::fs::create_dir_all(&directory).unwrap();
            controller.save(directory.join(format!("{bed}.crystalsave"))).unwrap();
        }
        println!("owned bed {bed}: production PC setup/reload/closure/walking passed; ownership seeded");
    }
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn kanto_elite_four_controller_wins_battles_and_opens_native_exits() {
    let pack=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(pack.parent().unwrap());
    let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap();
    let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,loaded).unwrap();
    for map in ["WillsRoom","KogasRoom","BrunosRoom","KarensRoom"] {
        let module=&runtime.data().maps[map];
        let entrance=module.events.warps.iter().filter_map(warp_tile_position_checked)
            .max_by_key(|tile|tile.y).unwrap();
        let exit_flag=module.script_flag_commands.iter().find(|c|
            c.command=="setevent" && c.flag_id.ends_with("_EXIT_OPEN")).unwrap();
        let exit_change=module.script_block_changes.iter().find(|c|
            c.source_script==exit_flag.source_script).unwrap();
        let trainer=module.objects.iter().find(|o|o.script==exit_flag.source_script).unwrap();
        let trainer_tile=crate::core::world::session::object_tile_position_checked(trainer).unwrap();
        let mut shell=initialize_bevy_runtime_shell(root.clone(),runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {spawn_identifier:runtime.title_new_game_spawn_identifier().unwrap(),
                map_name:map.into(),tile_x:entrance.x,tile_y:entrance.y},BevyShellConfig::default()).unwrap();
        // This fixture party avoids level grinding. Door/victory flags are
        // never seeded: combat, PP consumption and script continuation are real.
        shell.shell.set_trainer_identity("CHRIS",1).unwrap();
        shell.shell.add_party_pokemon("MEW",100,None,None,"CHRIS",1,
            Dv::from_non_hp(15,15,15,15)).unwrap();
        let pp=runtime.data().moves["ICE_BEAM"].pp*8/5;
        {
            let state=shell.shell.session_mut().state_mut();
            state.storage.party.pokemon[0].as_mut().unwrap().moves=vec![crate::core::models::LearnedMove {
                name:"ICE_BEAM".into(),current_pp:pp,pp_ups:3}];
            state.sync_party_from_storage();
        }
        let mut controller=VisibleShellController {shell};
        settle_visible_overworld_arrival(&mut controller.shell,"walk_warp").unwrap();
        settle_visible_shell_until_input(&mut controller.shell).unwrap();
        assert!(!controller.shell.shell.session().state().flags
            .is_script_flag_set(&exit_flag.flag_id).unwrap());
        let target=TilePosition::new(trainer_tile.x,trainer_tile.y+1);
        for _ in 0..64 {
            if controller.snapshot().unwrap().overworld.tile==target {break;}
            let path=collision_path_to_tile(controller.shell.shell.session().overworld(),target);
            let button=match path[0] {Direction::Up=>GameButton::Up,Direction::Down=>GameButton::Down,
                Direction::Left=>GameButton::Left,Direction::Right=>GameButton::Right};
            controller.press(button).unwrap();
        }
        assert_eq!(controller.snapshot().unwrap().overworld.tile,target,"{map}: trainer approach");
        // The native center corridor approaches each trainer from the south.
        assert_eq!(controller.shell.shell.session().overworld().player.facing,Direction::Up);
        let mut saw_battle=false;
        for _ in 0..512 {
            let snapshot=controller.snapshot().unwrap();
            saw_battle|=snapshot.phase==RuntimeShellPhase::TrainerBattle;
            if controller.shell.shell.session().state().flags
                .is_script_flag_set(&exit_flag.flag_id).unwrap() {break;}
            controller.press(GameButton::A).unwrap();
        }
        assert!(saw_battle,"{map}: native trainer battle never started");
        assert!(controller.shell.shell.session().state().flags
            .is_script_flag_set(&exit_flag.flag_id).unwrap(),"{map}: victory did not open exit");
        let snapshot=controller.snapshot().unwrap();
        assert!(snapshot.battle.is_none(),"{map}: battle must finish before the exit opens");
        assert!(snapshot.party.slots[0].pokemon.moves[0].current_pp<pp,"{map}: no move PP spent");
        let stride=crate::core::systems::script_blocks::CHANGE_BLOCK_COORD_STRIDE;
        let xy=(exit_change.x/stride,exit_change.y/stride);
        assert_eq!(controller.shell.shell.session().state().map_block_overrides
            .get(map).and_then(|b|b.get(&xy)),Some(&exit_change.block_id));
        let i=usize::from(xy.1)*usize::from(module.attributes.width)+usize::from(xy.0);
        assert_eq!(controller.shell.shell.session().overworld().map.metatile_ids[i],exit_change.block_id);
        if let Some(directory)=std::env::var_os("KANTO_INTERIOR_STATE_REVIEW_DIR") {
            let directory=PathBuf::from(directory);std::fs::create_dir_all(&directory).unwrap();
            controller.save(directory.join(format!("{map}-battle-exit-open.crystalsave"))).unwrap();
        }
        let exit=module.events.warps.iter().filter_map(warp_tile_position_checked)
            .min_by_key(|tile|tile.y).unwrap();
        let approach=TilePosition::new(exit.x,exit.y+1);
        for _ in 0..64 {
            if controller.snapshot().unwrap().overworld.tile==approach {break;}
            let path=collision_path_to_tile(controller.shell.shell.session().overworld(),approach);
            assert!(!path.is_empty(),"{map}: earned north exit is unreachable");
            controller.press(match path[0] {Direction::Up=>GameButton::Up,Direction::Down=>GameButton::Down,
                Direction::Left=>GameButton::Left,Direction::Right=>GameButton::Right}).unwrap();
        }
        assert_eq!(controller.snapshot().unwrap().overworld.tile,approach,"{map}: exit approach");
        if let Some(directory)=std::env::var_os("KANTO_INTERIOR_STATE_REVIEW_DIR") {
            controller.save(PathBuf::from(directory).join(format!("{map}-earned-exit-approach.crystalsave"))).unwrap();
        }
        controller.press(GameButton::Up).unwrap();
        let expected=match map {"WillsRoom"=>"KogasRoom","KogasRoom"=>"BrunosRoom",
            "BrunosRoom"=>"KarensRoom","KarensRoom"=>"LancesRoom",_=>unreachable!()};
        assert_eq!(controller.snapshot().unwrap().overworld.map_name,expected,
            "{map}: the earned door must lead through the production warp");
    }
}


#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn kanto_controller_league_loss_recovers_without_opening_the_exit() {
    let pack=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(pack.parent().unwrap());
    let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap();
    let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,loaded).unwrap();
    let map="KarensRoom";
    let module=&runtime.data().maps[map];
    let flag=module.script_flag_commands.iter().find(|command|
        command.command=="setevent" && command.flag_id.ends_with("_EXIT_OPEN")).unwrap();
    let trainer=module.objects.iter().find(|object|object.script==flag.source_script).unwrap();
    let trainer=crate::core::world::session::object_tile_position_checked(trainer).unwrap();
    let entrance=module.events.warps.iter().filter_map(warp_tile_position_checked)
        .max_by_key(|tile|tile.y).unwrap();
    let spawn=runtime.title_new_game_spawn_identifier().unwrap();
    let mut shell=initialize_bevy_runtime_shell(root,runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {spawn_identifier:spawn,
            map_name:map.into(),tile_x:entrance.x,tile_y:entrance.y},
        BevyShellConfig::default()).unwrap();
    // A real losing fixture: Splash cannot defeat Karen. No outcome, HP,
    // trainer flag, script cursor or blackout phase is supplied by the test.
    shell.shell.set_trainer_identity("CHRIS",1).unwrap();
    shell.shell.add_party_pokemon("MAGIKARP",5,None,None,"CHRIS",1,
        Dv::from_non_hp(15,15,15,15)).unwrap();
    let mut controller=VisibleShellController {shell};
    settle_visible_overworld_arrival(&mut controller.shell,"walk_warp").unwrap();
    settle_visible_shell_until_input(&mut controller.shell).unwrap();
    let target=TilePosition::new(trainer.x,trainer.y+1);
    for _ in 0..64 {
        if controller.snapshot().unwrap().overworld.tile==target {break;}
        let path=collision_path_to_tile(controller.shell.shell.session().overworld(),target);
        assert!(!path.is_empty());
        controller.press(match path[0] {Direction::Up=>GameButton::Up,Direction::Down=>GameButton::Down,
            Direction::Left=>GameButton::Left,Direction::Right=>GameButton::Right}).unwrap();
    }
    assert_eq!(controller.snapshot().unwrap().overworld.tile,target);
    controller.press(GameButton::Up).unwrap();
    let mut saw_battle=false;
    let mut saw_loss=false;
    for _ in 0..512 {
        let snapshot=controller.snapshot().unwrap();
        saw_battle|=snapshot.phase==RuntimeShellPhase::TrainerBattle;
        saw_loss|=controller.shell.visible_blackout_phase.is_some();
        assert!(!controller.shell.shell.session().state().flags
            .is_script_flag_set(&flag.flag_id).unwrap(),"losing must never open Karen's exit");
        if saw_loss && controller.shell.visible_blackout_phase.is_none()
            && snapshot.overworld.map_name!=map {break;}
        controller.press(GameButton::A).unwrap();
    }
    let recovered=controller.snapshot().unwrap();
    assert!(saw_battle && saw_loss,"the fixture must actually battle and lose");
    assert_ne!(recovered.overworld.map_name,map,"whiteout must warp to the recovery spawn");
    assert!(recovered.battle.is_none());
    assert_eq!(recovered.party.slots[0].pokemon.hp,recovered.party.slots[0].pokemon.max_hp);
    assert!(controller.shell.visible_blackout_phase.is_none());
    assert!(controller.shell.active_script_cursor.is_none());
    assert!(controller.shell.battle_messages.is_empty());
    assert!(controller.shell.screen_fade.is_none());
    assert_eq!(controller.shell.shell.session().state().battle_result & 0x3f,1);
}


#[test]
#[ignore = "requires an external verified pack with credits via CRYSTAL_RENDER_TEST_PACK"]
fn kanto_controller_continuously_earns_league_and_continues_home() {
    let pack=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(pack.parent().unwrap());
    let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap();
    let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,loaded).unwrap();
    let entrance=runtime.data().maps["WillsRoom"].events.warps.iter()
        .filter_map(warp_tile_position_checked).max_by_key(|tile|tile.y).unwrap();
    let mut shell=initialize_bevy_runtime_shell(root,runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {spawn_identifier:runtime.title_new_game_spawn_identifier().unwrap(),
            map_name:"WillsRoom".into(),tile_x:entrance.x,tile_y:entrance.y},
        BevyShellConfig::default()).unwrap();
    // Only the initial legal party is supplied. This one controller carries
    // its actual HP, status, PP and flags through every battle and walking warp.
    shell.shell.set_trainer_identity("CHRIS",1).unwrap();
    shell.shell.add_party_pokemon("MEW",100,None,None,"CHRIS",1,Dv::from_non_hp(15,15,15,15)).unwrap();
    {
        let state=shell.shell.session_mut().state_mut();
        state.storage.party.pokemon[0].as_mut().unwrap().moves=
            ["SURF","ICE_BEAM","PSYCHIC_M","RECOVER"].into_iter().map(|name|
                crate::core::models::LearnedMove {name:name.into(),
                    current_pp:runtime.data().moves[name].pp*8/5,pp_ups:3}).collect();
        state.sync_party_from_storage();
    }
    assert!(shell.shell.add_bag_item("FULL_RESTORE",5).unwrap().added);
    let mut controller=VisibleShellController {shell};
    settle_visible_overworld_arrival(&mut controller.shell,"walk_warp").unwrap();
    settle_visible_shell_until_input(&mut controller.shell).unwrap();
    for (map,next) in [("WillsRoom","KogasRoom"),("KogasRoom","BrunosRoom"),
        ("BrunosRoom","KarensRoom"),("KarensRoom","LancesRoom"),("LancesRoom","HallOfFame")] {
        assert_eq!(controller.snapshot().unwrap().overworld.map_name,map);
        league_restore_party_through_pack(&mut controller);
        let module=&runtime.data().maps[map];
        let flag=module.script_flag_commands.iter().find(|command|command.command=="setevent"
            && if map=="LancesRoom" {command.flag_id=="EVENT_BEAT_CHAMPION_LANCE"}
                else {command.flag_id.ends_with("_EXIT_OPEN")}).unwrap();
        assert!(!controller.shell.shell.session().state().flags.is_script_flag_set(&flag.flag_id).unwrap(),
            "{map}: victory must not be supplied by the fixture");
        let trainer=module.objects.iter().find(|object|object.script==flag.source_script).unwrap();
        let trainer=crate::core::world::session::object_tile_position_checked(trainer).unwrap();
        let target=TilePosition::new(trainer.x,trainer.y+1);
        for _ in 0..128 {
            let snapshot=controller.snapshot().unwrap();
            if snapshot.overworld.tile==target || snapshot.phase==RuntimeShellPhase::TrainerBattle {break;}
            if snapshot.ui.window_open || snapshot.phase==RuntimeShellPhase::Text {
                controller.press(GameButton::A).unwrap();
                continue;
            }
            let path=collision_path_to_tile(controller.shell.shell.session().overworld(),target);
            assert!(!path.is_empty(),"{map}: trainer corridor must remain reachable");
            controller.press(match path[0] {Direction::Up=>GameButton::Up,Direction::Down=>GameButton::Down,
                Direction::Left=>GameButton::Left,Direction::Right=>GameButton::Right}).unwrap();
        }
        let approached=controller.snapshot().unwrap();
        assert!(approached.overworld.tile==target || approached.phase==RuntimeShellPhase::TrainerBattle,
            "{map}: approach stopped at {:?}, phase {:?}, text {:?}",approached.overworld.tile,
            approached.phase,approached.ui.text);
        if approached.overworld.tile==target && approached.phase!=RuntimeShellPhase::TrainerBattle {
            // A connected room can arrive in the left lane: pathfinding may
            // finish by stepping right. Face the trainer through ordinary Up.
            controller.press(GameButton::Up).unwrap();
            assert_eq!(controller.shell.shell.session().overworld().player.facing,Direction::Up);
        }
        let before_pp: u16=controller.snapshot().unwrap().party.slots[0].pokemon.moves.iter()
            .map(|learned|u16::from(learned.current_pp)).sum();
        let mut saw_battle=false;
        let mut battle_history=std::collections::VecDeque::new();
        for _ in 0..1024 {
            let snapshot=controller.snapshot().unwrap();
            let observation=format!("hp={} enemy={:?} text={:?} messages={:?}",
                snapshot.party.slots[0].pokemon.hp,
                snapshot.battle.as_ref().map(|battle|(&battle.enemy_pokemon.species.id,
                    battle.enemy_pokemon.hp,&battle.enemy_last_move)),
                snapshot.ui.text,controller.shell.battle_messages);
            if battle_history.back()!=Some(&observation) {
                battle_history.push_back(observation);
                if battle_history.len()>24 {battle_history.pop_front();}
            }
            saw_battle|=snapshot.phase==RuntimeShellPhase::TrainerBattle;
            if controller.shell.shell.session().state().flags.is_script_flag_set(&flag.flag_id).unwrap()
                && snapshot.battle.is_none() {break;}
            let pokemon=&snapshot.party.slots[0].pokemon;
            // Recover spends a real turn while Destiny Bond is advertised;
            // do not attack into it or rewind a bad confusion/AI roll.
            let wait_for_bond=snapshot.battle.as_ref().is_some_and(|battle|
                battle.enemy_pokemon.species.id=="GENGAR"
                    && battle.enemy_last_move.as_deref()==Some("DESTINY_BOND"));
            let desired=if (pokemon.hp<pokemon.max_hp/2 || wait_for_bond)
                && pokemon.moves[3].current_pp>0 {3} else {match map {
                "WillsRoom"=>1,
                "LancesRoom" if snapshot.battle.as_ref()
                    .is_some_and(|battle|battle.enemy_pokemon.species.id=="GYARADOS")=>2,
                "LancesRoom" if snapshot.battle.as_ref()
                    .is_some_and(|battle|battle.enemy_pokemon.species.id=="CHARIZARD")=>0,
                "LancesRoom"=>1,
                "KogasRoom"=>2,
                "KarensRoom" if snapshot.battle.as_ref()
                    .is_some_and(|battle|matches!(battle.enemy_pokemon.species.id.as_str(),
                        "VILEPLUME"|"GENGAR"))=>2,
                "KarensRoom" if snapshot.battle.as_ref()
                    .is_some_and(|battle|battle.enemy_pokemon.species.id=="MURKROW")=>1,
                _=>0,
            }};
            let desired=if snapshot.party.slots[0].pokemon.moves[desired].current_pp>0 {desired}
                else {snapshot.party.slots[0].pokemon.moves.iter().position(|learned|learned.current_pp>0)
                    .expect("the carried party must retain at least one usable move")};
            // Read the presented cursor and move it using ordinary input;
            // never assign it or execute a move through a second dispatcher.
            let button=if let Some(cursor)=controller.shell.battle_move_cursor.as_ref()
                .filter(|_|controller.shell.battle_messages.is_empty()) {
                assert!(snapshot.party.slots[0].pokemon.moves[desired].current_pp>0,
                    "{map}: selected legal move must retain earned PP");
                if cursor.option_index==desired {GameButton::A} else {GameButton::Down}
            } else {GameButton::A};
            controller.press(button).unwrap();
        }
        assert!(saw_battle,"{map}: the actual trainer battle must run");
        assert!(controller.shell.shell.session().state().flags.is_script_flag_set(&flag.flag_id).unwrap(),
            "{map}: battle must earn its flag");
        let snapshot=controller.snapshot().unwrap();
        assert!(snapshot.battle.is_none(),"{map}: battle must settle before traversal");
        let after_pp:u16=snapshot.party.slots[0].pokemon.moves.iter()
            .map(|learned|u16::from(learned.current_pp)).sum();
        assert!(after_pp<before_pp,"{map}: battle must spend carried PP");
        assert!(snapshot.party.slots[0].pokemon.hp>0,"{map}: the carried party must survive; status={:?}; history={battle_history:#?}",
            snapshot.party.slots[0].pokemon.status);
        println!("earned {map}: hp={}/{} status={:?} remaining_pp={after_pp}",
            snapshot.party.slots[0].pokemon.hp,snapshot.party.slots[0].pokemon.max_hp,
            snapshot.party.slots[0].pokemon.status);
        if map=="LancesRoom" {
            for _ in 0..256 {
                if controller.snapshot().unwrap().overworld.map_name==next {break;}
                controller.press(GameButton::A).unwrap();
            }
        } else {
            let exit=module.events.warps.iter().filter_map(warp_tile_position_checked)
                .min_by_key(|tile|tile.y).unwrap();
            let approach=TilePosition::new(exit.x,exit.y+1);
            for _ in 0..64 {
                if controller.snapshot().unwrap().overworld.tile==approach {break;}
                let path=collision_path_to_tile(controller.shell.shell.session().overworld(),approach);
                assert!(!path.is_empty(),"{map}: the earned north doorway must be reachable");
                controller.press(match path[0] {Direction::Up=>GameButton::Up,Direction::Down=>GameButton::Down,
                    Direction::Left=>GameButton::Left,Direction::Right=>GameButton::Right}).unwrap();
            }
            assert_eq!(controller.snapshot().unwrap().overworld.tile,approach);
            controller.press(GameButton::Up).unwrap();
        }
        assert_eq!(controller.snapshot().unwrap().overworld.map_name,next,"{map}: production warp");
    }
    finish_earned_hall_of_fame(&mut controller,&runtime,"continuous-league");
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn kanto_lance_controller_wins_battle_and_continues_into_hall_of_fame() {
    run_earned_lance_hall_of_fame(false);
}

#[test]
#[ignore = "requires an external verified pack with credits_source_data via CRYSTAL_RENDER_TEST_PACK"]
fn kanto_lance_controller_completes_earned_ceremony_and_continues_home() {
    run_earned_lance_hall_of_fame(true);
}

fn run_earned_lance_hall_of_fame(complete_ceremony: bool) {
    let pack=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(pack.parent().unwrap());
    let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap();
    let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,loaded).unwrap();
    for map in ["LancesRoom"] {
        let module=&runtime.data().maps[map];
        let entrance=module.events.warps.iter().filter_map(warp_tile_position_checked)
            .max_by_key(|tile|tile.y).unwrap();
        let exit_flag=module.script_flag_commands.iter().find(|c|
            c.command=="setevent" && c.flag_id=="EVENT_BEAT_CHAMPION_LANCE").unwrap();
        let exit_change=module.script_block_changes.iter().find(|c|
            c.source_script==exit_flag.source_script).unwrap();
        let trainer=module.objects.iter().find(|o|o.script==exit_flag.source_script).unwrap();
        let trainer_tile=crate::core::world::session::object_tile_position_checked(trainer).unwrap();
        let mut shell=initialize_bevy_runtime_shell(root.clone(),runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {spawn_identifier:runtime.title_new_game_spawn_identifier().unwrap(),
                map_name:map.into(),tile_x:entrance.x,tile_y:entrance.y},BevyShellConfig::default()).unwrap();
        // This fixture party avoids level grinding. Door/victory flags are
        // never seeded: combat, PP consumption and script continuation are real.
        shell.shell.set_trainer_identity("CHRIS",1).unwrap();
        shell.shell.add_party_pokemon("MEW",100,None,None,"CHRIS",1,
            Dv::from_non_hp(15,15,15,15)).unwrap();
        let review_party_size=std::env::var("KANTO_HALL_REVIEW_PARTY_SIZE").ok()
            .map(|value|value.parse::<usize>().expect("review party size must be an integer")).unwrap_or(1);
        assert!((1..=6).contains(&review_party_size),"review party size must be1..=6");
        for species in ["CELEBI","PIKACHU","LUGIA","TYRANITAR","ESPEON"].iter().take(review_party_size-1) {
            shell.shell.add_party_pokemon(species,100,None,None,"CHRIS",1,Dv::from_non_hp(15,15,15,15)).unwrap();
        }
        let pp=runtime.data().moves["ICE_BEAM"].pp*8/5;
        {
            let state=shell.shell.session_mut().state_mut();
            state.storage.party.pokemon[0].as_mut().unwrap().moves=vec![crate::core::models::LearnedMove {
                name:"ICE_BEAM".into(),current_pp:pp,pp_ups:3}];
            state.sync_party_from_storage();
        }
        let mut controller=VisibleShellController {shell};
        settle_visible_overworld_arrival(&mut controller.shell,"walk_warp").unwrap();
        settle_visible_shell_until_input(&mut controller.shell).unwrap();
        assert!(!controller.shell.shell.session().state().flags
            .is_script_flag_set(&exit_flag.flag_id).unwrap());
        // Native animation review must load a quiescent overworld checkpoint.
        // A lower-level mid-dialogue save does not serialize the visible script
        // cursor, so it cannot prove the remainder of the earned ceremony.
        if let Some(directory)=std::env::var_os("KANTO_INTERIOR_STATE_REVIEW_DIR") {
            let directory=PathBuf::from(directory);std::fs::create_dir_all(&directory).unwrap();
            assert!(controller.shell.active_script_cursor.is_none());
            assert!(!controller.snapshot().unwrap().ui.window_open);
            let checkpoint=directory.join(format!("{map}-before-battle.crystalsave"));
            let before=controller.snapshot().unwrap();
            controller.save(&checkpoint).unwrap();
            controller=VisibleShellController {shell:initialize_bevy_runtime_shell(
                root.clone(),runtime.clone(),BevyShellStart::LoadSave {save_path:checkpoint},
                BevyShellConfig::default()).unwrap()};
            let restored=controller.snapshot().unwrap();
            assert_eq!(restored.overworld.map_name,before.overworld.map_name);
            assert_eq!(restored.overworld.tile,before.overworld.tile);
            assert!(!controller.shell.shell.session().state().flags
                .is_script_flag_set(&exit_flag.flag_id).unwrap());
        }
        let target=TilePosition::new(trainer_tile.x,trainer_tile.y+1);
        for _ in 0..64 {
            if controller.snapshot().unwrap().overworld.tile==target
                || controller.snapshot().unwrap().phase==RuntimeShellPhase::TrainerBattle
                || controller.snapshot().unwrap().ui.window_open {break;}
            let path=collision_path_to_tile(controller.shell.shell.session().overworld(),target);
            let button=match path[0] {Direction::Up=>GameButton::Up,Direction::Down=>GameButton::Down,
                Direction::Left=>GameButton::Left,Direction::Right=>GameButton::Right};
            controller.press(button).unwrap();
        }

        // The native center corridor approaches each trainer from the south.

        let mut saw_battle=false;
        #[cfg(feature="location-tester")]
        let mut saw_earned_dialogue_capture=false;
        for _ in 0..512 {
            let snapshot=controller.snapshot().unwrap();
            saw_battle|=snapshot.phase==RuntimeShellPhase::TrainerBattle;
            if snapshot.overworld.map_name=="HallOfFame" {break;}
            #[cfg(feature="location-tester")]
            {
                let wanted="MARY: Oh, no!\nIt's all over!";
                let before_observation=controller.shell.shell.state_checksum().unwrap();
                let source=controller.shell.shell.presentation_snapshot().unwrap();
                if let Some((page,chars))=render_walk::revealed_dialogue_page(&controller.shell,&source,wanted) {
                    assert!(controller.shell.field_notice.is_none(),"exercise ordinary compiled text, not a special notice");
                    assert!(source.ui.text_window_open && chars>=wanted.chars().count());
                    assert_eq!(visible_field_dialog_pages(&source,&controller.shell).unwrap()[page],wanted);
                    let mut closed=source.clone();closed.ui.text_window_open=false;
                    assert!(render_walk::revealed_dialogue_page(&controller.shell,&closed,wanted).is_none(),
                        "retained source text after closetext cannot freeze a capture");
                    assert!(render_walk::revealed_dialogue_page(&controller.shell,&source,"different page").is_none());
                    assert_eq!(before_observation,controller.shell.shell.state_checksum().unwrap(),
                        "observing a QA hold must not alter game state");
                    let state=controller.shell.shell.session().state();
                    assert!(state.flags.is_script_flag_set("EVENT_BEAT_CHAMPION_LANCE").unwrap());
                    assert!(state.flags.is_script_flag_set("EVENT_LANCES_ROOM_ENTRANCE_CLOSED").unwrap());
                    assert!(!state.flags.is_script_flag_set("EVENT_LANCES_ROOM_EXIT_OPEN").unwrap());
                    assert!(source.party.slots[0].pokemon.moves[0].current_pp<pp);
                    saw_earned_dialogue_capture=true;
                }
            }
            // With reserves, decline the presented Shift choice through normal
            // B input. Confirming it would repeatedly choose the active Mew.
            let button=if controller.shell.battle_shift_prompt_cursor.is_some()
                && controller.shell.battle_messages.is_empty() {GameButton::B}else{GameButton::A};
            controller.press(button).unwrap();
        }
        assert!(saw_battle,"{map}: native trainer battle never started");
        #[cfg(feature="location-tester")]
        assert!(saw_earned_dialogue_capture,"the ordinary authored post-victory page must be observable by QA");
        assert!(controller.shell.shell.session().state().flags
            .is_script_flag_set(&exit_flag.flag_id).unwrap(),
            "{map}: victory flag not earned; map={} ui={:?} battle={:?} shift={:?} switch={:?}",
            controller.snapshot().unwrap().overworld.map_name,controller.snapshot().unwrap().ui,
            controller.snapshot().unwrap().battle,controller.shell.battle_shift_prompt_cursor,controller.shell.battle_switch_cursor);
        let snapshot=controller.snapshot().unwrap();
        assert_eq!(snapshot.overworld.map_name,"HallOfFame","native post-victory continuation must warp into Hall of Fame");
        assert!(snapshot.battle.is_none(),"{map}: battle must finish before the exit opens");
        assert!(snapshot.party.slots[0].pokemon.moves[0].current_pp<pp,"{map}: no move PP spent");
        let stride=crate::core::systems::script_blocks::CHANGE_BLOCK_COORD_STRIDE;
        let xy=(exit_change.x/stride,exit_change.y/stride);
        assert_eq!(controller.shell.shell.session().state().map_block_overrides
            .get(map).and_then(|b|b.get(&xy)),Some(&exit_change.block_id));
        assert!(!controller.shell.shell.session().state().flags.is_script_flag_set("EVENT_LANCES_ROOM_EXIT_OPEN").unwrap(),"Lance script must not invent the other rooms exit flag");
        if let Some(directory)=std::env::var_os("KANTO_INTERIOR_STATE_REVIEW_DIR") {
            let directory=PathBuf::from(directory);std::fs::create_dir_all(&directory).unwrap();
            controller.save(directory.join(format!("{map}-battle-hall-of-fame.crystalsave"))).unwrap();
        }
        if !complete_ceremony { continue; }
        finish_earned_hall_of_fame(&mut controller, &runtime, "lance");
    }
}


// The fixture supplies initial stock only. Every later HP/status change here
// goes through Start→Pack→Use→party and must consume an item without restoring PP.
fn league_restore_party_through_pack(controller:&mut VisibleShellController) {
    let before=controller.snapshot().unwrap();
    let pokemon=&before.party.slots[0].pokemon;
    if pokemon.hp==pokemon.max_hp && pokemon.status.is_none() {return;}
    assert!(pokemon.hp>0,"a defeated party cannot be repaired with a field healing shortcut");
    let quantity=|snapshot:&RuntimeShellSnapshot|snapshot.bag.items.iter()
        .find(|item|item.item_id=="FULL_RESTORE").map_or(0,|item|item.quantity);
    let stock=quantity(&before);assert!(stock>0,"initial healing stock must last the gauntlet");
    controller.press(GameButton::Start).unwrap();
    for _ in 0..16 {
        if visible_start_menu_entries(&controller.shell).unwrap().iter().any(|entry|entry==">PACK") {break;}
        controller.press(GameButton::Down).unwrap();
    }
    assert!(visible_start_menu_entries(&controller.shell).unwrap().iter().any(|entry|entry==">PACK"));
    controller.press(GameButton::A).unwrap();
    for _ in 0..32 {
        let snapshot=controller.snapshot().unwrap();
        let entries=visible_field_pack_entries(&snapshot,&controller.shell).unwrap();
        if entries.iter().any(|entry|entry.starts_with('>') && entry.replace('_'," ").contains("FULL RESTORE")) {break;}
        controller.press(GameButton::Down).unwrap();
    }
    let snapshot=controller.snapshot().unwrap();
    assert!(visible_field_pack_entries(&snapshot,&controller.shell).unwrap().iter()
        .any(|entry|entry.starts_with('>') && entry.replace('_'," ").contains("FULL RESTORE")));
    controller.press(GameButton::A).unwrap();
    for _ in 0..8 {
        let snapshot=controller.snapshot().unwrap();
        if visible_field_pack_entries(&snapshot,&controller.shell).unwrap().iter().any(|entry|entry==">USE") {break;}
        controller.press(GameButton::Down).unwrap();
    }
    let snapshot=controller.snapshot().unwrap();
    assert!(visible_field_pack_entries(&snapshot,&controller.shell).unwrap().iter().any(|entry|entry==">USE"));
    controller.press(GameButton::A).unwrap();
    for _ in 0..16 {
        let snapshot=controller.snapshot().unwrap();
        let healed=&snapshot.party.slots[0].pokemon;
        if healed.hp==healed.max_hp && healed.status.is_none() && quantity(&snapshot)==stock-1 {break;}
        controller.press(GameButton::A).unwrap();
    }
    let after=controller.snapshot().unwrap();
    assert_eq!(after.party.slots[0].pokemon.hp,after.party.slots[0].pokemon.max_hp);
    assert!(after.party.slots[0].pokemon.status.is_none());
    assert_eq!(quantity(&after),stock-1,"ordinary item use must consume its initial stock");
    assert_eq!(after.party.slots[0].pokemon.moves,pokemon.moves,"Full Restore must preserve spent PP");
    for _ in 0..16 {
        if controller.snapshot().unwrap().phase==RuntimeShellPhase::Overworld {break;}
        controller.press(GameButton::B).unwrap();
    }
    assert_eq!(controller.snapshot().unwrap().phase,RuntimeShellPhase::Overworld);
    assert_eq!(controller.snapshot().unwrap().overworld.tile,before.overworld.tile);
    println!("used Full Restore through Pack: {} hp→{}, stock {}→{}",pokemon.hp,
        after.party.slots[0].pokemon.hp,stock,stock-1);
}

fn finish_earned_hall_of_fame(
    controller: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    review_name: &str,
) {
    let expected_party_size=controller.snapshot().unwrap().party.slots.len();
    // Continue the earned victory through the original entry scene. Unlike
    // a ceremony fixture, this never seeds a script cursor or champion flag.
    let save_path = std::env::temp_dir().join(format!(
        "geothite-earned-{review_name}-hof-{}.crystalsave", std::process::id()));
    struct CeremonySaveCleanup(PathBuf);
    impl Drop for CeremonySaveCleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
            let _ = std::fs::remove_file(format!("{}.bak",self.0.display()));
        }
    }
    let _save_cleanup = CeremonySaveCleanup(save_path.clone());
    controller.shell.quick_save_path = Some(save_path.clone());
    for _ in 0..128 {
        if controller.shell.credits_screen.is_some() { break; }
        controller.press(GameButton::A).unwrap();
    }
    assert!(controller.shell.credits_screen.as_ref()
        .is_some_and(|credits| credits.hall_of_fame.is_some()),
        "earned Lance dialogue must enter the ceremony");
    assert_eq!(controller.snapshot().unwrap().overworld.tile, TilePosition::new(4, 3),
        "native slow_step must reach the machine across the platform stairs");
    let opening_elapsed = controller.shell.credits_screen.as_ref().unwrap()
        .hall_of_fame.as_ref().unwrap().sequence.elapsed_frames();
    controller.press(GameButton::Down).unwrap();
    controller.press(GameButton::Start).unwrap();
    assert!(controller.snapshot().unwrap().ui.menu.is_none(),
        "ceremony owns Start; the overworld menu must stay closed");
    assert_eq!(controller.snapshot().unwrap().overworld.tile, TilePosition::new(4, 3),
        "ceremony owns input; directions cannot move the player");
    assert!(controller.shell.credits_screen.as_ref().unwrap().hall_of_fame.as_ref()
        .unwrap().sequence.elapsed_frames() > opening_elapsed,
        "controller input must advance the production presentation clock");
    let recorded = runtime.load_save(&save_path).unwrap();
    assert_eq!(recorded.hall_of_fame.count, 1);
    assert_eq!(recorded.hall_of_fame.entries[0].team[0].as_ref().unwrap().species, "MEW");
    assert_eq!(recorded.hall_of_fame.entries[0].team.iter().flatten().count(),expected_party_size,
        "the earned Hall record must include every actual party member");
    assert!(recorded.hall_of_fame.spawn_after_champion.is_some());
    assert!(recorded.flags.is_script_flag_set("EVENT_BEAT_ELITE_FOUR").unwrap());
    assert!(!recorded.game_timer_counting);
    let mut phases = BTreeSet::new();
    let review_dir = std::env::var_os("KANTO_INTERIOR_STATE_REVIEW_DIR").map(PathBuf::from)
        .map(|directory| if review_name == "lance" { directory } else { directory.join(review_name) });
    if let Some(directory)=review_dir.as_ref() { std::fs::create_dir_all(directory).unwrap(); }
    let mut art = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    for _ in 0..40000 {
        let credits = controller.shell.credits_screen.as_ref().expect("credits owner");
        if credits.awaiting_exit { break; }
        if let Some(ceremony) = credits.hall_of_fame.as_ref() {
            let phase = ceremony.sequence.phase();
            if phases.insert(format!("{phase:?}")) {
                if let Some(directory) = review_dir.as_ref() {
                    let frame = render_visible_hall_of_fame_screen(&controller.shell, ceremony,
                        &mut art, &mut images).unwrap();
                    let pixels = images.get(&frame.handle).unwrap();
                    let native = image::RgbaImage::from_raw(160, 144,
                        pixels.data.as_ref().unwrap().clone()).unwrap();
                    let name = format!("earned-hof-{phase:?}").replace([' ', '{', '}', ':'], "");
                    native.save(directory.join(format!("{name}.png"))).unwrap();
                }
            }
            if phase == crystal_runtime::hall_of_fame::HallOfFamePhase::OakRating {
                controller.press(GameButton::A).unwrap();
            }
        }
        controller.wait_frames(1).unwrap();
        assert!(controller.shell.last_error.is_none(), "{:?}", controller.shell.last_error);
    }
    for required in ["PokemonBack", "PokemonFront", "PokemonAnimation", "PokemonHold",
        "PlayerBack", "PlayerFront", "OakRating", "ClosingFade"] {
        assert!(phases.iter().any(|phase| phase.starts_with(required)),
            "earned ceremony skipped {required}: {phases:?}");
    }
    assert!(controller.shell.credits_screen.as_ref().unwrap().awaiting_exit,
        "the complete credits program must reach THE END");
    controller.press(GameButton::A).unwrap();
    controller.wait_frames(60).unwrap();
    assert!(controller.shell.credits_screen.is_none());
    assert!(controller.shell.title_menu.is_some() || controller.shell.intro_screen.is_some());
    // Drive the actual title input path, including its exported MainMenu
    // dispatch and twenty-frame Continue summary. Calling the load helper
    // directly would miss a stranded summary confirmation.
    advance_visible_title_to_main_menu(&mut controller.shell).unwrap();
    controller.press(GameButton::A).unwrap();
    assert!(controller.shell.visible_continue_screen.is_some(), "A must open Continue summary");
    let delay=controller.shell.visible_continue_screen.as_ref().unwrap().remaining_delay_frames;
    assert_eq!(delay,20);
    controller.press(GameButton::A).unwrap();
    assert_eq!(controller.shell.visible_continue_screen.as_ref().unwrap().remaining_delay_frames,delay,
        "A must not consume or bypass the authored summary clock");
    // The native frontend runs this same presentation tick from Time;
    // this renderer-neutral fixture supplies its twenty clock frames.
    settle_visible_continue_summary_for_test(&mut controller.shell);
    controller.press(GameButton::A).unwrap();
    assert!(controller.shell.visible_continue_screen.is_none(), "A must confirm the settled summary");
    assert!(controller.shell.title_menu.is_none());
    settle_visible_shell_until_input(&mut controller.shell).unwrap();
    assert_eq!(controller.snapshot().unwrap().overworld.map_name, "NewBarkTown");
    assert_eq!(controller.shell.shell.session().state().hall_of_fame.count, 1);
    assert!(controller.shell.shell.session().state().hall_of_fame.spawn_after_champion.is_none());
    assert!(controller.shell.shell.session().state().game_timer_counting);
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn kanto_department_store_sixth_floor_controller_applies_rooftop_stair_callback() {
    let pack = PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
        .expect("set CRYSTAL_RENDER_TEST_PACK to the completed external pack"));
    let root = AssetRoot::new(pack.parent().expect("pack directory"));
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap();
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).unwrap();
    let map = "CeladonDeptStore6F";
    let declared = &runtime.data().maps[map];
    let changes: Vec<_> = declared.script_block_changes.iter().filter(|c|
        c.source_script == "CeladonDeptStore6FHideRooftopStairsCallback").collect();
    assert_eq!(changes.len(), 1, "the native variant must have one callback edit");
    let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
    let shell = initialize_bevy_runtime_shell(root, runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile { spawn_identifier, map_name: map.into(),
            tile_x: 4, tile_y: 4 },
        BevyShellConfig::default()).unwrap();
    let mut controller = VisibleShellController { shell };
    settle_visible_shell_until_input(&mut controller.shell).unwrap();
    assert_eq!(controller.snapshot().unwrap().overworld.map_name, map);
    let stride = crate::core::systems::script_blocks::CHANGE_BLOCK_COORD_STRIDE;
    for change in changes {
        let xy = (change.x / stride, change.y / stride);
        assert_eq!(controller.shell.shell.session().state().map_block_overrides
            .get(map).and_then(|blocks| blocks.get(&xy)), Some(&change.block_id));
        let index = usize::from(xy.1) * usize::from(declared.attributes.width)
            + usize::from(xy.0);
        assert_ne!(u16::from(declared.blocks[index]), change.block_id,
            "the callback must replace the original rooftop stair drawing");
        assert_eq!(controller.shell.shell.session().overworld().map.metatile_ids[index], change.block_id,
            "the authoritative collision/layout map must contain the callback result");
    }
    // Verify the production controller has settled and released input ownership.
    controller.press(GameButton::Start).unwrap();
    assert!(controller.snapshot().unwrap().ui.menu.is_some(),
        "Start must expose the production menu after the callback");
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn kanto_celadon_vending_controller_buys_refuses_and_releases_input() {
    fn menu_ready(controller:&mut VisibleShellController)->bool {
        let snapshot=controller.snapshot().unwrap();
        snapshot.ui.menu.as_ref().is_some_and(|menu|
            menu.menu_id==".MenuHeader@CeladonDeptStore6FVendingMachine")
            &&controller.shell.field_notice.is_none()
            &&controller.shell.field_notice_queue.is_empty()
            &&controller.shell.pending_item_notification.is_none()
            &&controller.shell.visible_field_item_notice.is_none()
            &&!snapshot.script_events.item_notify_queued
    }
    let pack=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(pack.parent().unwrap());
    let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap();
    let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,loaded).unwrap();
    let full_pocket=crate::core::models::bag::MAX_ITEM_STACK
        *u16::try_from(crate::core::models::bag::ITEM_POCKET_CAPACITY).unwrap();
    for (choice,item,price) in [(0,"FRESH_WATER",200),(1,"SODA_POP",300),(2,"LEMONADE",350)] {
        for (money,stock,buys) in [(1000,0,true),(0,0,false),(1000,full_pocket,false)] {
            let mut shell=initialize_bevy_runtime_shell(root.clone(),runtime.clone(),
                BevyShellStart::NewGameAtRuntimeTile {
                    spawn_identifier:runtime.title_new_game_spawn_identifier().unwrap(),
                    map_name:"CeladonDeptStore6F".into(),tile_x:8,tile_y:2},
                BevyShellConfig::default()).unwrap();
            shell.shell.set_trainer_identity("CHRIS",1).unwrap();
            shell.shell.session_mut().state_mut().money=money;
            if stock>0 {assert!(shell.shell.add_bag_item(item,stock).unwrap().added);}
            let mut controller=VisibleShellController {shell};
            settle_visible_shell_until_input(&mut controller.shell).unwrap();
            // Face the actual background event from its customer-side tile.
            controller.press(GameButton::Up).unwrap();
            let tile=controller.snapshot().unwrap().overworld.tile;
            assert_eq!(tile,TilePosition::new(8,2));
            assert_eq!(quest_item_quantity(&controller.shell,item),u16::from(stock));
            if let Some(directory)=std::env::var_os("KANTO_INTERIOR_STATE_REVIEW_DIR") {
                let directory=PathBuf::from(directory).join("celadon-vending");
                std::fs::create_dir_all(&directory).unwrap();
                let mode=if buys {"funded"}else if stock>0 {"full-pocket"}else{"no-money"};
                controller.shell.shell.save(directory.join(format!("{item}-{mode}.crystalsave"))).unwrap();
            }
            controller.press(GameButton::A).unwrap();
            for _ in 0..32 {
                if menu_ready(&mut controller) {break;}
                controller.press(GameButton::A).unwrap();
            }
            let menu=controller.snapshot().unwrap().ui.menu.expect("vending source menu must be visible");
            assert_eq!(menu.menu_id,".MenuHeader@CeladonDeptStore6FVendingMachine",
                "preserve the exact qualified source-menu identity");
            assert!(menu.layout.vertical_menus.iter().any(|vertical|
                vertical.options.iter().any(|option|option.contains("FRESH WATER"))));
            controller.press(GameButton::Down).unwrap();
            assert_eq!(controller.snapshot().unwrap().overworld.tile,tile,
                "vending menu owns movement input");
            controller.press(GameButton::Up).unwrap();
            for _ in 0..choice {controller.press(GameButton::Down).unwrap();}
            controller.press(GameButton::A).unwrap();
            // Advance source text/item notification back to the same menu.
            // Stop there so the test cannot accidentally buy a second item.
            let mut saw_item_notice=false;
            for _ in 0..64 {
                if menu_ready(&mut controller) {break;}
                if controller.shell.field_notice.is_some() {
                    saw_item_notice=true;
                    assert_eq!(controller.shell.field_notice.as_deref(),
                        Some(format!("CHRIS put the\n{} in\nthe ITEM POCKET.",
                            item.replace('_'," ")).as_str()),
                        "the source item notice must retain the actual trainer and drink name");
                    let visible=controller.snapshot().unwrap();
                    assert!(visible.ui.menu.is_none(),"the public controller must hide a menu under the notice");
                    assert_eq!(visible.ui.text.as_ref().unwrap().label,"visible-shell:field-notice");
                    assert!(visible.ui.text.as_ref().unwrap().asm_text.as_ref().is_some_and(|text|!text.is_empty()));
                    let selected=controller.shell.shell.session().state().script_runtime.script_value.clone();
                    let menu=controller.shell.shell.session().state().script_runtime.active_menu.clone();
                    controller.press(GameButton::B).unwrap();
                    if controller.shell.field_notice.is_some() {
                        assert_eq!(controller.shell.shell.session().state().script_runtime.script_value,selected,
                            "while the notice still owns input, B cannot cancel a source menu underneath it");
                        assert_eq!(controller.shell.shell.session().state().script_runtime.active_menu,menu);
                    }
                } else {controller.press(GameButton::A).unwrap();}
            }
            assert!(menu_ready(&mut controller),"source vending loop must finish item notification before returning");
            if buys {assert!(saw_item_notice,"successful purchase must present its pocket notification");}
            assert_eq!(quest_item_quantity(&controller.shell,item),u16::from(stock)+u16::from(buys));
            assert_eq!(controller.shell.shell.session().state().money,
                if buys {money-price}else{money},"{item}: exactly one charge or no charge on refusal");
            controller.press(GameButton::B).unwrap();
            settle_visible_shell_until_input(&mut controller.shell).unwrap();
            let closed=controller.snapshot().unwrap();
            assert!(closed.ui.menu.is_none()&&!closed.ui.window_open,
                "cancel must close the source text/menu and release input: ui={:?} cursor={:?} notice={:?} item={:?}",
                closed.ui,controller.shell.active_script_cursor,controller.shell.field_notice,
                controller.shell.pending_item_notification);
            controller.press(GameButton::Start).unwrap();
            assert!(controller.snapshot().unwrap().ui.menu.is_some(),
                "production Start must work after leaving the vending machine");
            println!("vending {item}: initial_money={money} initial_stock={stock} purchased={buys}; cancel/Start passed");
        }
    }
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn kanto_elite_four_controller_entry_scenes_lock_native_doors() {
    let pack = PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
        .expect("set CRYSTAL_RENDER_TEST_PACK to the completed external pack"));
    let root = AssetRoot::new(pack.parent().expect("pack directory"));
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap();
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).unwrap();
    for map in ["WillsRoom", "KogasRoom", "BrunosRoom", "KarensRoom", "LancesRoom"] {
        let module = &runtime.data().maps[map];
        let entrance = module.events.warps.iter().filter_map(warp_tile_position_checked)
            .max_by_key(|tile| tile.y).expect("native southern entrance warp");
        let lock_script = format!("{map}DoorLocksBehindYouScript");
        let flag = module.script_flag_commands.iter().find(|c|
            c.source_script == lock_script && c.command == "setevent"
                && c.flag_id.ends_with("_ENTRANCE_CLOSED"))
            .expect("authored entrance scene must set its native lock flag");
        let change = module.script_block_changes.iter().find(|c|c.source_script == lock_script)
            .expect("authored entrance scene must replace its native door block");
        let shell = initialize_bevy_runtime_shell(root.clone(), runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: runtime.title_new_game_spawn_identifier().unwrap(),
                map_name: map.into(), tile_x: entrance.x, tile_y: entrance.y,
            }, BevyShellConfig::default()).unwrap();
        let mut controller = VisibleShellController { shell };
        assert!(!controller.shell.shell.session().state().flags
            .is_script_flag_set(&flag.flag_id).unwrap(), "{map}: fixture already had a locked entrance");
        // Runtime-tile previews deliberately skip arrival scenes. Invoke the
        // same arrival transaction used by ordinary production walking warps.
        settle_visible_overworld_arrival(&mut controller.shell, "walk_warp").unwrap();
        settle_visible_shell_until_input(&mut controller.shell).unwrap();
        // No fixture flags, synthetic block edits, or second dispatcher.
        for _ in 0..8 {
            if controller.shell.shell.session().state().flags
                .is_script_flag_set(&flag.flag_id).unwrap() { break; }
            controller.press(GameButton::Up).unwrap();
        }
        assert!(controller.shell.shell.session().state().flags
            .is_script_flag_set(&flag.flag_id).unwrap(), "{map}: entry did not lock the door");
        let stride = crate::core::systems::script_blocks::CHANGE_BLOCK_COORD_STRIDE;
        let xy = (change.x / stride, change.y / stride);
        assert_eq!(controller.shell.shell.session().state().map_block_overrides
            .get(map).and_then(|blocks|blocks.get(&xy)), Some(&change.block_id), "{map}");
        let index = usize::from(xy.1) * usize::from(module.attributes.width) + usize::from(xy.0);
        assert_eq!(controller.shell.shell.session().overworld().map.metatile_ids[index],
            change.block_id, "{map}: authoritative doorway did not change");
        assert_eq!(controller.snapshot().unwrap().overworld.map_name, map);
        if let Some(directory) = std::env::var_os("KANTO_INTERIOR_STATE_REVIEW_DIR") {
            let directory = PathBuf::from(directory);
            std::fs::create_dir_all(&directory).unwrap();
            controller.save(directory.join(format!("{map}-entry-locked.crystalsave"))).unwrap();
        }
        // A faded render barrier must remain a closed gameplay doorway.
        // First prove input ownership has returned to the production controller.
        controller.press(GameButton::Start).unwrap();
        assert!(controller.snapshot().unwrap().ui.menu.is_some(), "{map}: entry retained input");
        controller.press(GameButton::B).unwrap();
        let overworld = controller.shell.shell.session().overworld();
        let blocked_y = (0..2).map(|offset|i16::try_from(change.y).unwrap()+offset)
            .find(|&y|crate::core::world::collision::sample_collision(
                &overworld.map, &overworld.tileset, TilePosition::new(entrance.x,y))
                .is_some_and(|sample|sample.metatile_id==change.block_id
                    && !crate::core::world::collision::is_permission_passable(
                        sample.permission, Direction::Down,
                        crate::core::world::collision::PlayerTraversalState::Walk)))
            .expect("native replacement must contain a closed doorway row");
        let adjacent = TilePosition::new(entrance.x, blocked_y - 1);
        for _ in 0..8 {
            if controller.snapshot().unwrap().overworld.tile == adjacent { break; }
            controller.press(GameButton::Down).unwrap();
        }
        assert_eq!(controller.snapshot().unwrap().overworld.tile, adjacent,
            "{map}: could not reach the closed entrance approach");
        controller.press(GameButton::Down).unwrap();
        controller.press(GameButton::Down).unwrap();
        assert_eq!(controller.snapshot().unwrap().overworld.tile, adjacent,
            "{map}: the closed door must reject movement even when its presentation fades");
    }
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn kanto_pokecenter_controller_heals_real_hp_and_pp_through_nurse_input() {
    let pack=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(pack.parent().unwrap());
    let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap();
    let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,loaded).unwrap();
    for map in ["PewterPokecenter1F","IndigoPlateauPokecenter1F"] {
        let module=&runtime.data().maps[map];
        let nurse=module.objects.iter().find(|o|o.script.ends_with("NurseScript")).unwrap();
        let tile=crate::core::world::session::object_tile_position_checked(nurse).unwrap();
        let shell=initialize_bevy_runtime_shell(root.clone(),runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {spawn_identifier:runtime.title_new_game_spawn_identifier().unwrap(),
                map_name:map.into(),tile_x:tile.x,tile_y:tile.y+2},BevyShellConfig::default()).unwrap();
        let mut controller=VisibleShellController{shell};
        controller.shell.shell.add_party_pokemon("CYNDAQUIL",5,None,None,"CHRIS",1,
            Dv::from_non_hp(10,10,10,10)).unwrap();
        let (max_hp,max_pp)={
            let state=controller.shell.shell.session_mut().state_mut();
            let pokemon=state.storage.party.pokemon[0].as_mut().unwrap();
            pokemon.hp=1;pokemon.moves[0].current_pp=1;
            let result=(pokemon.max_hp,runtime.data().moves[&pokemon.moves[0].name].pp);
            state.sync_party_from_storage();result
        };
        controller.press(GameButton::Up).unwrap();
        assert_eq!(controller.shell.shell.session().overworld().player.facing,Direction::Up);
        let overworld=controller.shell.shell.session().overworld();
        assert!(overworld.check_interaction_checked(1).unwrap().is_some(),
            "{map}: no nurse interaction from {:?}, nurse at {:?}",overworld.player.tile,tile);
        let mut saw_prompt=false;
        for _ in 0..128 {
            let snapshot=controller.snapshot().unwrap();saw_prompt|=snapshot.ui.text_window_open;
            let pokemon=&controller.shell.shell.session().state().storage.party.pokemon[0];
            if pokemon.as_ref().is_some_and(|p|p.hp==max_hp&&p.moves[0].current_pp==max_pp) {break;}
            controller.press(GameButton::A).unwrap();
        }
        assert!(saw_prompt,"{map}: nurse dialogue was not visible; ui={:?}, tile={:?}",controller.snapshot().unwrap().ui,controller.shell.shell.session().overworld().player.tile);
        let pokemon=controller.shell.shell.session().state().storage.party.pokemon[0].as_ref().unwrap();
        assert_eq!(pokemon.hp,max_hp,"{map}: native HealParty did not restore HP; animation={:?}, cursor={:?}",controller.shell.visible_heal_machine.as_ref().map(|a|(a.kind,a.frame)),controller.shell.active_script_cursor.as_ref().map(|c|(&c.source_script,c.next_command_index)));
        assert_eq!(pokemon.moves[0].current_pp,max_pp,"{map}: native HealParty did not restore move PP");
        for _ in 0..32 {
            if !controller.snapshot().unwrap().ui.text_window_open {break;}
            controller.press(GameButton::A).unwrap();
        }
        controller.press(GameButton::Start).unwrap();
        assert!(controller.snapshot().unwrap().ui.menu.is_some(),"{map}: nurse retained input after healing");
    }
}

#[cfg(all(feature="voxel-view",feature="location-tester"))]
#[test]
#[ignore="requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn kanto_native_nurse_input_publishes_live_healing_art_and_hides_only_consumed_overlays() {
    use bevy::ecs::system::RunSystemOnce;
    use crystal_render_api::{VisualActorId,VisualEffectId,VisualWorldFrame};
    let pack=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap());
    let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap();
    let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,loaded).unwrap();
    let mut shell=initialize_bevy_runtime_shell(root,runtime.clone(),BevyShellStart::NewGameAtRuntimeTile {
        spawn_identifier:runtime.title_new_game_spawn_identifier().unwrap(),map_name:"PewterPokecenter1F".into(),
        tile_x:3,tile_y:3},BevyShellConfig{render_test_hour:Some(12),..default()}).unwrap();
    for _ in 0..6 {shell.shell.add_party_pokemon("CYNDAQUIL",5,None,None,"CHRIS",1,Dv::from_non_hp(10,10,10,10)).unwrap();}
    let mut app=integrated_shell_test_app(shell);
    app.init_resource::<VisualWorldFrame>().add_systems(Update,publish_visual_world_frame.after(render_playfield));
    app.update();app.update();
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ArrowUp);app.update();
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();app.update();
    let mut saw=false;
    for frame in 0..1024 {
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        if frame%16==0 {app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyZ);}
        app.update();
        let published=app.world().resource::<VisualWorldFrame>();
        if !published.actors.iter().any(|a|matches!(a.id,VisualActorId::Effect(VisualEffectId::HealMachineBall(5)))) {continue;}
        assert_eq!(published.actors.iter().filter(|a|matches!(a.id,VisualActorId::Effect(VisualEffectId::HealMachineBall(_)))).count(),6);
        assert!(crystal_voxel_view::native_clinic_healing_supported(published));
        let ball=published.actors.iter().find(|a|a.id==VisualActorId::Effect(VisualEffectId::HealMachineBall(0))).unwrap();
        let world=app.world();let art=world.resource::<RenderedTilesetArt>();
        assert!(art.heal_machine_ball_cache.as_ref().unwrap().iter().any(|f|f.handle==ball.texture));
        assert_eq!(published.actors.iter().filter(|a|matches!(a.id,VisualActorId::Effect(VisualEffectId::HealMachineLamp(_)))).count(),2);
        app.insert_resource(crystal_voxel_view::VoxelViewStatus{active:true,..default()});
        app.insert_resource(crystal_voxel_view::NativeHealingEffectsStatus{consumed:true});
        app.world_mut().run_system_once(sync_native_healing_sprite_visibility).unwrap();
        let world=app.world_mut();let mut query=world.query_filtered::<&Visibility,With<HealMachineWorldSprite>>();
        assert!(query.iter(world).all(|v|*v==Visibility::Hidden));
        world.resource_mut::<crystal_voxel_view::VoxelViewStatus>().active=false;
        world.run_system_once(sync_native_healing_sprite_visibility).unwrap();
        let mut query=world.query_filtered::<&Visibility,With<HealMachineWorldSprite>>();
        assert!(query.iter(world).all(|v|*v==Visibility::Inherited));
        saw=true;break;
    }
    assert!(saw,"real nurse input never published the sixth healing ball; error={:?}",app.world().resource::<BevyRuntimeShell>().last_error);
    let mut palette_handles=std::collections::HashSet::new();
    for _ in 0..320 {
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();app.update();
        if let Some(ball)=app.world().resource::<VisualWorldFrame>().actors.iter()
            .find(|a|a.id==VisualActorId::Effect(VisualEffectId::HealMachineBall(0))) {palette_handles.insert(ball.texture.id());}
        if app.world().resource::<BevyRuntimeShell>().visible_heal_machine.is_none() {break;}
    }
    assert!(app.world().resource::<BevyRuntimeShell>().visible_heal_machine.is_none());
    assert_eq!(palette_handles.len(),4,"all authored healing palette phases must publish");
    app.update();app.update();
    assert!(!app.world().resource::<VisualWorldFrame>().actors.iter()
        .any(|a|matches!(a.id,VisualActorId::Effect(VisualEffectId::HealMachineBall(_)|VisualEffectId::HealMachineLamp(_)))));
    let world=app.world_mut();let mut sprites=world.query_filtered::<Entity,With<HealMachineWorldSprite>>();
    assert_eq!(sprites.iter(world).count(),0,"source effects must retire after the authored sequence");

}

#[test]
fn lighthouse_stair_arrivals_ignore_blocked_steps_and_allow_reentry() {
    let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK").map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../content-packs/text-tui.crystalpack"));
    if !path.is_file() {
        assert!(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").is_none(), "missing explicit test pack");
        return;
    }
    let root = AssetRoot::new(path.parent().unwrap());
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).unwrap();
    for (source, destination) in [("OlivineLighthouse5F", "OlivineLighthouse6F"),
        ("OlivineLighthouse6F", "OlivineLighthouse5F")] {
        let shell = initialize_bevy_runtime_shell(root.clone(), runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: runtime.title_new_game_spawn_identifier().unwrap(),
                map_name: source.into(), tile_x: 9, tile_y: 14,
            }, BevyShellConfig::default()).unwrap();
        let mut controller = VisibleShellController { shell };
        for _ in 0..4 {
            controller.press(GameButton::Down).unwrap();
            if controller.snapshot().unwrap().overworld.map_name == destination { break; }
        }
        let landing = controller.snapshot().unwrap().overworld;
        assert_eq!(landing.map_name, destination, "stair entry from {source}");
        assert_eq!(landing.tile, TilePosition::new(9, 15));
        for _ in 0..8 {
            controller.press(GameButton::Down).unwrap();
            let blocked = controller.snapshot().unwrap().overworld;
            assert_eq!(blocked.map_name, destination, "blocked arrival step must not reactivate stair from {source}");
            assert_eq!(blocked.tile, landing.tile);
        }
        for _ in 0..4 {
            controller.press(GameButton::Up).unwrap();
            if controller.snapshot().unwrap().overworld.tile == TilePosition::new(9, 14) { break; }
        }
        let away = controller.snapshot().unwrap().overworld;
        assert_eq!(away.map_name, destination);
        assert_eq!(away.tile, TilePosition::new(9, 14), "arrival must allow walking away");
        for _ in 0..4 {
            controller.press(GameButton::Down).unwrap();
            if controller.snapshot().unwrap().overworld.map_name == source { break; }
        }
        let returned = controller.snapshot().unwrap().overworld;
        assert_eq!(returned.map_name, source, "walking back onto the stair must still warp");
        assert_eq!(returned.tile, landing.tile);
    }
}

#[cfg(feature = "voxel-view")]
#[test]
fn warp_fade_waits_for_destination_terrain_and_actor_preparation() {
    use crystal_voxel_view::{VoxelViewSettings, VoxelViewStatus};
    let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK").map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../content-packs/text-tui.crystalpack"));
    if !path.is_file() {
        assert!(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").is_none());
        return;
    }
    let root = AssetRoot::new(path.parent().unwrap());
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).unwrap();
    let mut shell = initialize_bevy_runtime_shell(root, runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().unwrap(),
            map_name: "OlivineLighthouse6F".into(), tile_x: 9, tile_y: 15,
        }, BevyShellConfig::default()).unwrap();
    let arm = |shell: &mut BevyRuntimeShell| {
        shell.visible_walk_warp_phase = Some(VisibleWalkWarpPhase::FadeIn);
        shell.screen_fade = Some(VisibleScreenFade::new(ScriptFadeColor::White,
            ScriptFadeDirection::In, 8));
    };
    arm(&mut shell);
    let mut time = Time::<()>::default();
    time.advance_by(std::time::Duration::from_secs_f32(GAME_TICK_SECONDS));
    let mut app = App::new();
    app.insert_resource(time).insert_resource(shell)
        .insert_resource(VoxelViewSettings {enabled:true,..Default::default()})
        .insert_resource(VoxelViewStatus {active:true,
            terrain_map:Some("OlivineLighthouse5F".into()),..Default::default()})
        .add_systems(Update,tick_visible_screen_fade);
    // Catch the first update before a destination build has reported pending.
    for _ in 0..16 { app.update(); }
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(shell.screen_fade.unwrap().elapsed_frames,0);
    assert_eq!(shell.screen_fade.unwrap().alpha,255);
    assert_eq!(shell.visible_walk_warp_phase,Some(VisibleWalkWarpPhase::FadeIn));
    {
        let mut status = app.world_mut().resource_mut::<VoxelViewStatus>();
        status.terrain_map=Some("OlivineLighthouse6F".into());
        status.terrain_pending=true; // Terrain installed, bodies still preparing.
    }
    for _ in 0..16 { app.update(); }
    assert_eq!(app.world().resource::<BevyRuntimeShell>().screen_fade.unwrap().elapsed_frames,0);
    app.world_mut().resource_mut::<VoxelViewStatus>().terrain_pending=false;
    app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(200));
    app.update();
    assert_eq!(app.world().resource::<BevyRuntimeShell>().screen_fade.unwrap().elapsed_frames,0,
        "publication stall belongs to the loading hold, not the fade");
    app.world_mut().resource_mut::<Time>().advance_by(
        std::time::Duration::from_secs_f32(GAME_TICK_SECONDS));
    app.update();
    assert_eq!(app.world().resource::<BevyRuntimeShell>().screen_fade.unwrap().elapsed_frames,1);
    for _ in 0..7 { app.update(); }
    assert_eq!(app.world().resource::<BevyRuntimeShell>().visible_walk_warp_phase,None);
    assert!(app.world().resource::<BevyRuntimeShell>().screen_fade.is_none());
    // Classic presentation and the renderer-neutral controller have no wait.
    arm(&mut app.world_mut().resource_mut::<BevyRuntimeShell>());
    app.world_mut().resource_mut::<VoxelViewSettings>().enabled=false;
    app.world_mut().resource_mut::<VoxelViewStatus>().terrain_pending=true;
    app.update();
    assert_eq!(app.world().resource::<BevyRuntimeShell>().screen_fade.unwrap().elapsed_frames,1);
    arm(&mut app.world_mut().resource_mut::<BevyRuntimeShell>());
    app.world_mut().resource_mut::<VoxelViewSettings>().enabled=true;
    app.world_mut().remove_resource::<VoxelViewStatus>();
    app.update();
    assert_eq!(app.world().resource::<BevyRuntimeShell>().screen_fade.unwrap().elapsed_frames,1);
    // A failed renderer reports inactive output, not a perpetual loading wait.
    arm(&mut app.world_mut().resource_mut::<BevyRuntimeShell>());
    app.insert_resource(VoxelViewStatus {terrain_map:Some("OlivineLighthouse5F".into()),
        inactive_reason:Some("terrain sync failed".into()),..Default::default()});
    app.update();
    assert_eq!(app.world().resource::<BevyRuntimeShell>().screen_fade.unwrap().elapsed_frames,1);
}

#[test]
fn radio_tower_all_twelve_stairs_keep_authored_step_off_and_allow_reentry() {
    let path=std::env::var_os("CRYSTAL_RENDER_TEST_PACK").map(PathBuf::from)
        .unwrap_or_else(||PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
    if !path.is_file(){assert!(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").is_none());return;}
    let root=AssetRoot::new(path.parent().unwrap());
    let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
    let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,loaded).unwrap();
    for (source,x,destination,arrival_x) in [
        ("RadioTower1F",15,"RadioTower2F",15),
        ("RadioTower2F",0,"RadioTower3F",0),("RadioTower2F",15,"RadioTower1F",15),
        ("RadioTower3F",0,"RadioTower2F",0),("RadioTower3F",7,"RadioTower4F",9),("RadioTower3F",17,"RadioTower4F",17),
        ("RadioTower4F",0,"RadioTower5F",0),("RadioTower4F",9,"RadioTower3F",7),
        ("RadioTower4F",12,"RadioTower5F",12),("RadioTower4F",17,"RadioTower3F",17),
        ("RadioTower5F",0,"RadioTower4F",0),("RadioTower5F",12,"RadioTower4F",12),
    ] {
        let mut shell=initialize_bevy_runtime_shell(root.clone(),runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile{spawn_identifier:runtime.title_new_game_spawn_identifier().unwrap(),
                map_name:source.into(),tile_x:x,tile_y:1},BevyShellConfig::default()).unwrap();
        // Authored post-clear visibility state opens the 2F stair approach.
        shell.shell.session_mut().state_mut().flags.set_event_flag("EVENT_RADIO_TOWER_BLACKBELT_BLOCKS_STAIRS",true).unwrap();
        let mut controller=VisibleShellController{shell};
        for _ in 0..4 {
            controller.press(GameButton::Up).unwrap();
            if controller.snapshot().unwrap().overworld.map_name==destination{break;}
        }
        let landing=controller.snapshot().unwrap().overworld;
        assert_eq!(landing.map_name,destination,"stair entry {source}({x},0)");
        assert_eq!(landing.tile,TilePosition::new(arrival_x,0));
        // STAIRCASE overrides the requested northward input with Crystal's
        // authored Down step. It must step off once on the destination map.
        controller.press(GameButton::Up).unwrap();
        let away=controller.snapshot().unwrap().overworld;
        assert_eq!(away.map_name,destination,"arrival step must stay in {destination}");
        assert_eq!(away.tile,TilePosition::new(arrival_x,1));
        for _ in 0..4 {
            controller.press(GameButton::Up).unwrap();
            if controller.snapshot().unwrap().overworld.map_name==source{break;}
        }
        let returned=controller.snapshot().unwrap().overworld;
        assert_eq!(returned.map_name,source,"reentry must return from {destination}");
        assert_eq!(returned.tile,TilePosition::new(x,0));
    }
}

// A legal fixture party replaces grinding; every reception choice, save prompt,
// escort, opponent selection and battle transition still uses controller input.
#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn battle_tower_controller_enters_from_reception_with_legal_party() {
    let pack=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(pack.parent().unwrap());
    let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap();
    let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,loaded).unwrap();
    let retained=std::env::var_os("KANTO_INTERIOR_STATE_REVIEW_DIR").map(PathBuf::from);
    let directory=retained.clone().unwrap_or_else(||std::env::temp_dir().join(format!(
        "geothite-tower-reception-{}-{}",std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())));
    std::fs::create_dir_all(&directory).unwrap();
    let quick=directory.join("BattleTower-reception-quicksave.crystalsave");
    let mut shell=initialize_bevy_runtime_shell(root,runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile{spawn_identifier:runtime.title_new_game_spawn_identifier().unwrap(),
            map_name:"BattleTower1F".into(),tile_x:7,tile_y:7},
        BevyShellConfig{quick_save_path:Some(quick.clone()),..Default::default()}).unwrap();
    shell.shell.set_trainer_identity("CHRIS",1).unwrap();
    for species in ["CYNDAQUIL","CHIKORITA","TOTODILE"] {
        shell.shell.add_party_pokemon(species,10,None,None,"CHRIS",1,
            Dv::from_non_hp(7,7,7,7)).unwrap();
    }
    let mut controller=VisibleShellController{shell};
    controller.press(GameButton::Up).unwrap();
    controller.save(&quick).unwrap();
    if retained.is_some() {
        controller.save(directory.join("BattleTower-reception-before-challenge.crystalsave")).unwrap();
    }
    let mut source_menus=std::collections::BTreeSet::new();
    let mut maps=std::collections::BTreeSet::new();
    let mut entered=false;
    let mut saw_challenge_choice=false;
    let mut saw_level_choice=false;
    let mut trace=std::collections::VecDeque::new();
    for _ in 0..512 {
        let snapshot=controller.snapshot().unwrap();
        let note=format!("map={} cursor={:?} choice={:?} value={:?} text={:?} wait={:?} action={:?}",
            snapshot.overworld.map_name,controller.shell.active_script_cursor,
            controller.shell.visible_battle_tower_challenge_menu,
            snapshot.script_events.script_value,snapshot.script_events.pending_text_label,
            snapshot.script_events.pending_text_wait,controller.shell.last_runtime_action);
        if trace.back()!=Some(&note) {trace.push_back(note);if trace.len()>32 {trace.pop_front();}}
        maps.insert(snapshot.overworld.map_name.clone());
        if controller.shell.visible_battle_tower_challenge_menu.is_some() {
            saw_challenge_choice=true;
            let cursor=controller.shell.active_script_cursor.as_ref().unwrap();
            assert_eq!(cursor.source_script,"Script_Menu_ChallengeExplanationCancel");
            assert_eq!(runtime.compiled_script_commands(&cursor.source_script).unwrap()
                [cursor.next_command_index]["command"].as_str(),Some("ifequal"),
                "cancel comparison cannot run before the player chooses");
            let menu=snapshot.ui.menu.as_ref().expect("controller exposes the production challenge menu");
            assert!(menu.layout.vertical_menus.iter().any(|m|m.options.iter().any(|s|s==">CHALLENGE")));
            let before=controller.shell.shell.state_checksum().unwrap();
            assert!(!close_visible_noninteractive_runtime_surface(&mut controller.shell).unwrap());
            assert_eq!(controller.shell.shell.state_checksum().unwrap(),before);
        }
        if controller.shell.visible_battle_tower_room_menu.is_some() {
            saw_level_choice=true;
            assert!(snapshot.ui.menu.as_ref().is_some_and(|menu|menu.layout.vertical_menus.iter()
                .any(|m|m.options.iter().any(|s|s.starts_with(">L:")))));
        }
        if let Some(menu)=snapshot.ui.menu.as_ref() {source_menus.insert(menu.menu_id.clone());}
        if snapshot.battle.is_some() {entered=true;break;}
        controller.press(GameButton::A).unwrap();
    }
    assert!(entered,"source reception did not reach battle: maps={maps:?} menus={source_menus:?} trace={trace:#?} audio={:?}",
        controller.shell.last_audio_events.iter().rev().take(8).collect::<Vec<_>>());
    assert!(saw_challenge_choice && saw_level_choice,"both actual source menus must own input");
    let state=controller.shell.shell.session().state();
    assert!(state.battle_tower.loaded_trainer_id.is_some());
    assert_eq!(controller.snapshot().unwrap().overworld.map_name,"BattleTowerBattleRoom");
    assert!(maps.contains("BattleTower1F") && maps.contains("BattleTowerBattleRoom"));
    // One controller press can settle several escorted maps. Do not mistake
    // an unobserved intermediate snapshot for a skipped source transition.
    let saved=runtime.load_save(&quick).unwrap();
    assert!(saved.battle_tower.explanation_read,
        "the actual reception save must replace the initial unexplained fixture");
    if retained.is_none() {std::fs::remove_dir_all(&directory).unwrap();}
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn kanto_rocket_statue_sensors_use_walk_input_and_earn_both_grunt_victories() {
    let pack=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(pack.parent().unwrap());
    let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap();
    let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,loaded).unwrap();
    for (x,y,camera) in [(24,2,1),(24,3,1),(6,2,2),(6,3,2),
        (24,6,3),(24,7,3),(22,16,4),(8,16,5)] {
        let mut shell=initialize_bevy_runtime_shell(root.clone(),runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier:runtime.title_new_game_spawn_identifier().unwrap(),
                map_name:"TeamRocketBaseB1F".into(),tile_x:x-1,tile_y:y,
            },BevyShellConfig::default()).unwrap();
        // Only initial legal party/identity are supplied. No camera flags,
        // actor positions, coordinate scripts or battle results are injected.
        shell.shell.set_trainer_identity("CHRIS",1).unwrap();
        shell.shell.add_party_pokemon("MEW",100,None,None,"CHRIS",1,
            Dv::from_non_hp(15,15,15,15)).unwrap();
        {
            let state=shell.shell.session_mut().state_mut();
            state.storage.party.pokemon[0].as_mut().unwrap().moves=vec![
                crate::core::models::LearnedMove {name:"SURF".into(),
                    current_pp:runtime.data().moves["SURF"].pp,pp_ups:0}];
            state.sync_party_from_storage();
        }
        let mut controller=VisibleShellController {shell};
        settle_visible_shell_until_input(&mut controller.shell).unwrap();
        let flag=format!("EVENT_SECURITY_CAMERA_{camera}");
        assert!(!controller.shell.shell.session().state().flags.is_script_flag_set(&flag).unwrap());
        let target=TilePosition::new(x,y);
        for _ in 0..16 {
            if controller.snapshot().unwrap().overworld.tile==target {break;}
            controller.press(GameButton::Right).unwrap();
        }
        assert_eq!(controller.snapshot().unwrap().overworld.tile,target,
            "camera{camera} at({x},{y}) must be reached by ordinary movement");
        let initial_pp=runtime.data().moves["SURF"].pp;
        let mut trainers=Vec::new();
        for _ in 0..1024 {
            let snapshot=controller.snapshot().unwrap();
            if let crate::core::state::BattleMemory::Trainer {trainer_id,..}=
                &controller.shell.shell.session().state().battle {
                if trainers.last()!=Some(trainer_id) {trainers.push(trainer_id.clone());}
            }
            if controller.shell.shell.session().state().flags.is_script_flag_set(&flag).unwrap()
                &&snapshot.battle.is_none() {break;}
            assert!(snapshot.party.slots[0].pokemon.hp>0,
                "the supplied party must survive real sensor battles");
            controller.press(GameButton::A).unwrap();
        }
        assert_eq!(trainers,["GRUNTM_20","GRUNTM_21"],
            "each active source sensor must summon and complete both actual trainers");
        assert!(controller.shell.shell.session().state().flags.is_script_flag_set(&flag).unwrap(),
            "only both victories may earn camera{camera}'s cleared event");
        settle_visible_shell_until_input(&mut controller.shell).unwrap();
        let snapshot=controller.snapshot().unwrap();
        assert!(snapshot.battle.is_none()&&!snapshot.ui.window_open);
        assert!(snapshot.party.slots[0].pokemon.moves[0].current_pp<initial_pp,
            "the source sensor battles must spend actual PP");
        assert_eq!(snapshot.overworld.tile,target);
        // Re-enter the very same sensor after its earned flag. It must stay
        // traversable without another trainer or a stranded dialogue owner.
        for (button,destination) in [(GameButton::Left,TilePosition::new(x-1,y)),
            (GameButton::Right,target)] {
            for _ in 0..16 {
                if controller.snapshot().unwrap().overworld.tile==destination {break;}
                controller.press(button).unwrap();
            }
            assert_eq!(controller.snapshot().unwrap().overworld.tile,destination);
        }
        controller.wait_frames(32).unwrap();
        assert!(controller.snapshot().unwrap().battle.is_none());
        if let Some(directory)=std::env::var_os("KANTO_INTERIOR_STATE_REVIEW_DIR") {
            let directory=PathBuf::from(directory).join("rocket-sensors");
            std::fs::create_dir_all(&directory).unwrap();
            controller.save(directory.join(format!("camera{camera}-{x}-{y}-earned.crystalsave"))).unwrap();
        }
        controller.press(GameButton::Start).unwrap();
        assert!(controller.snapshot().unwrap().ui.menu.is_some(),
            "the earned cleared sensor must release production Start input");
        println!("camera{camera} sensor({x},{y}): ordinary entry, two real victories, PP use, cleared re-entry/Start passed");
    }
}
