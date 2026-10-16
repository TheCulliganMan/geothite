// External-pack regressions exercise production callbacks and controller input.
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
    }
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn kanto_lance_controller_wins_battle_and_continues_into_hall_of_fame() {
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
        for _ in 0..512 {
            let snapshot=controller.snapshot().unwrap();
            saw_battle|=snapshot.phase==RuntimeShellPhase::TrainerBattle;
            if snapshot.overworld.map_name=="HallOfFame" {break;}
            controller.press(GameButton::A).unwrap();
        }
        assert!(saw_battle,"{map}: native trainer battle never started");
        assert!(controller.shell.shell.session().state().flags
            .is_script_flag_set(&exit_flag.flag_id).unwrap(),"{map}: victory flag not earned");
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
    }
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
