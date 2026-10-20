// Review starts and parties are disposable fixtures. All service actions use
// the same production controller as keyboard and MCP input.
include!("clinic_move_services.rs");
include!("clinic_mail_services.rs");
include!("clinic_item_services.rs");
include!("clinic_oak_services.rs");
fn clinic_lobbies(runtime: &CrystalRuntime) -> Vec<String> {
    let mut maps=runtime.map_ids().into_iter()
        .filter(|map|map.ends_with("Pokecenter1F")).collect::<Vec<_>>();
    maps.sort();
    assert_eq!(maps, [
        "AzaleaPokecenter1F", "BlackthornPokecenter1F", "CeladonPokecenter1F",
        "CeruleanPokecenter1F", "CherrygrovePokecenter1F", "CianwoodPokecenter1F",
        "CinnabarPokecenter1F", "EcruteakPokecenter1F", "FuchsiaPokecenter1F",
        "GoldenrodPokecenter1F", "IndigoPlateauPokecenter1F", "LavenderPokecenter1F",
        "MahoganyPokecenter1F", "OlivinePokecenter1F", "PewterPokecenter1F",
        "Route10Pokecenter1F", "Route32Pokecenter1F", "SaffronPokecenter1F",
        "SilverCavePokecenter1F", "VermilionPokecenter1F", "VioletPokecenter1F",
        "ViridianPokecenter1F",
    ], "every authored clinic lobby must participate in the service review");
    maps
}

fn clinic_pc_until(controller:&mut VisibleShellController, ready:impl Fn(&BevyRuntimeShell)->bool) {
    for _ in 0..256 {
        if ready(&controller.shell) {return;}
        controller.press(GameButton::A).unwrap();
    }
    panic!("clinic PC did not reach its next input surface: {:?}",controller.snapshot().unwrap().ui);
}

fn clinic_pc_fixture(root:&AssetRoot,runtime:&CrystalRuntime,map:&str,save:Option<PathBuf>)->VisibleShellController {
    let module=&runtime.data().maps[map];
    assert_eq!(module.attributes.tileset_name,"pokecenter");
    // Block08's eastern half contains the complete public PC. Its south
    // approach is two gameplay tiles below the block's northern edge.
    let computers=module.blocks.iter().enumerate().filter(|(_,b)|**b==8)
        .map(|(i,_)|TilePosition::new((i%module.attributes.width as usize*2+1)as i16,
        (i/module.attributes.width as usize*2+2)as i16)).collect::<Vec<_>>();
    assert_eq!(computers.len(),1,"{map}: authenticate the actual PC drawing");
    let at=computers[0];
    let shell=initialize_bevy_runtime_shell(root.clone(),runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
        spawn_identifier:runtime.title_new_game_spawn_identifier().unwrap(),
        map_name:map.to_string(),tile_x:at.x,tile_y:at.y,
        },BevyShellConfig{smoke_player_name:Some("CHRIS".into()),quick_save_path:save,..default()}).unwrap();
    let mut controller=VisibleShellController{shell};
    complete_visible_smoke_player_name_if_needed(&mut controller.shell,Some("CHRIS")).unwrap();
    assert_eq!(controller.snapshot().unwrap().trainer.player_name,"CHRIS");
    for (species,level) in [("CYNDAQUIL",5),("TOTODILE",8)] {
        controller.shell.shell.add_party_pokemon(species,level,None,None,"CHRIS",1,
        Dv::from_non_hp(10,10,10,10)).unwrap();
    }
    controller
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn clinic_public_pcs_deposit_withdraw_and_return_walking_in_every_lobby() {
    let path=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(path.parent().unwrap());
    let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
    let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,loaded).unwrap();
    for map in clinic_lobbies(&runtime) {
        let mut controller=clinic_pc_fixture(&root,&runtime,&map,None);
        let original=controller.shell.shell.session().state().storage.party.pokemon.iter()
            .flatten().cloned().collect::<Vec<_>>();
        controller.press(GameButton::Up).unwrap();
        assert_eq!(controller.shell.shell.current_overworld_interaction_checked().unwrap()
            .map(|i|i.script),Some("PCScript".into()),"{map}: source PC must be interactable");
        if let Some(directory)=std::env::var_os("CLINIC_REVIEW_DIR") {
            let directory=PathBuf::from(directory);std::fs::create_dir_all(&directory).unwrap();
            controller.save(directory.join(format!("pc-start-{map}.crystalsave"))).unwrap();
        }
        controller.press(GameButton::A).unwrap();
        let boot=controller.snapshot().unwrap();
        assert!(boot.ui.menu.is_none()&&boot.ui.text_window_open,
            "{map}: boot dialogue must cover the suspended core menu");
        assert!(boot.ui.text.as_ref().and_then(|t|t.asm_text.as_deref())
            .is_some_and(|text|text.contains("CHRIS")&&text.contains("turned on")));
        controller.press(GameButton::Down).unwrap();
        assert_eq!(controller.snapshot().unwrap().overworld.tile,boot.overworld.tile,
            "{map}: boot dialogue owns directional input");
        #[cfg(feature="location-tester")]
        assert!(render_walk::retained_pc_review_menu(&controller.shell,"pc:hub").unwrap().is_none(),
            "PC boot text must not count as a ready menu");
        clinic_pc_until(&mut controller,|s|s.pc_hub_cursor.is_some()&&s.special_boundary.is_none()
            &&s.pc_notice.is_none()&&s.field_notice.is_none());
        #[cfg(feature="location-tester")]
        assert!(render_walk::retained_pc_review_menu(&controller.shell,"pc:hub").unwrap().is_some());
        controller.press(GameButton::A).unwrap();
        clinic_pc_until(&mut controller,|s|s.bill_pc_action_cursor.is_some()
            &&s.special_boundary.is_none()&&s.pc_notice.is_none()&&s.field_notice.is_none());
        let snapshot=controller.snapshot().unwrap();
        let menu=snapshot.ui.menu.expect("the actual Bill PC chooser must be visible");
        let entries=menu.layout.vertical_menus.iter().flat_map(|m|m.options.iter()).collect::<Vec<_>>();
        for label in ["WITHDRAW", "DEPOSIT", "CHANGE BOX", "W/O MAIL", "SEE YA"] {
            assert!(entries.iter().any(|entry|entry.contains(label)),"{map}: missing visible PC action {label}: {entries:?}");
        }
        #[cfg(feature="location-tester")]
        {
            assert_eq!(render_walk::retained_pc_review_menu(&controller.shell,"pc:bill-actions").unwrap()
                .as_ref().map(|rows|rows.iter().collect::<Vec<_>>()),Some(entries));
            assert!(render_walk::retained_pc_review_menu(&controller.shell,"pc:hub").unwrap().is_none());
        }
        controller.press(GameButton::Down).unwrap();
        controller.press(GameButton::A).unwrap();
        assert!(controller.shell.bill_pc_deposit_open&&controller.shell.storage_cursor.is_some());
        controller.press(GameButton::A).unwrap();
        assert!(controller.shell.bill_pc_pokemon_action_cursor.is_some());
        controller.press(GameButton::A).unwrap();
        clinic_pc_until(&mut controller,|s|s.pc_transfer_sequence.is_none()
            &&s.pc_notice.is_none()&&s.bill_pc_pokemon_action_cursor.is_none());
        let snapshot=controller.snapshot().unwrap();
        assert_eq!(snapshot.party.slots.len(),1,"{map}: DEPOSIT must change authoritative party storage");
        assert_eq!(snapshot.party.slots[0].pokemon.species.id,"TOTODILE");
        let stored=snapshot.storage.boxes.iter().find(|b|b.index==snapshot.storage.current_pc_box).unwrap();
        assert_eq!(stored.slots.len(),1,"{map}: DEPOSIT must create the actual box entry");
        assert_eq!(stored.slots[0].pokemon.species.id,"CYNDAQUIL");
        controller.press(GameButton::B).unwrap();
        assert_eq!(controller.shell.bill_pc_action_cursor.as_ref().map(|c|c.option_index),Some(0),
            "{map}: returning from the party list selects WITHDRAW");
        controller.press(GameButton::A).unwrap();
        assert!(!controller.shell.bill_pc_deposit_open&&controller.shell.storage_cursor.is_some());
        controller.press(GameButton::A).unwrap();
        controller.press(GameButton::A).unwrap();
        clinic_pc_until(&mut controller,|s|s.pc_transfer_sequence.is_none()
            &&s.pc_notice.is_none()&&s.bill_pc_pokemon_action_cursor.is_none());
        let snapshot=controller.snapshot().unwrap();
        assert_eq!(snapshot.party.slots.len(),2,"{map}: WITHDRAW must restore the actual party");
        assert!(snapshot.storage.boxes.iter().all(|b|b.slots.is_empty()));
        let restored=controller.shell.shell.session().state().storage.party.pokemon.iter()
            .flatten().collect::<Vec<_>>();
        assert!(original.iter().all(|p|restored.contains(&p)),"{map}: preserve both actual Pokémon through storage");
        for _ in 0..16 {
            if !controller.shell.pc_hub_session_open {break;}
            controller.press(GameButton::B).unwrap();
        }
        assert!(!controller.shell.pc_hub_session_open);
        let before=controller.snapshot().unwrap().overworld.tile;
        for _ in 0..8 {
            controller.press(GameButton::Down).unwrap();
            if controller.snapshot().unwrap().overworld.tile!=before {break;}
        }
        let after=controller.snapshot().unwrap();
        assert_eq!(after.overworld.map_name,map);
        assert_eq!(after.overworld.tile,TilePosition::new(before.x,before.y+1),"PC must release walking input");
        assert_eq!(controller.shell.last_error,None);
        println!("{map}: real PC boot/menu/deposit/withdraw/closure/walking passed");
    }
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn clinic_public_pcs_switch_save_reload_and_release_through_real_input() {
    let path=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(path.parent().unwrap());
    let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,
        crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap()).unwrap();
    let directory=std::env::temp_dir().join(format!("geothite-clinic-branches-{}-{}",
        std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir_all(&directory).unwrap();
    for map in clinic_lobbies(&runtime) {
        let save=directory.join(format!("{map}.crystalsave"));
        let mut controller=clinic_pc_fixture(&root,&runtime,&map,Some(save.clone()));
        let original=controller.shell.shell.session().state().storage.clone();
        controller.press(GameButton::Up).unwrap();
        controller.press(GameButton::A).unwrap();
        clinic_pc_until(&mut controller,|s|s.pc_hub_cursor.is_some()&&s.special_boundary.is_none()
            &&s.pc_notice.is_none()&&s.field_notice.is_none());
        controller.press(GameButton::A).unwrap();
        clinic_pc_until(&mut controller,|s|s.bill_pc_action_cursor.is_some()&&s.special_boundary.is_none()
            &&s.pc_notice.is_none()&&s.field_notice.is_none());
        // CHANGE BOX is reached from the real PC hub; no menu cursor is seeded.
        for button in [GameButton::Down,GameButton::Down,GameButton::A] {
            controller.press(button).unwrap();
        }
        let before_boxes=controller.shell.shell.session().state().clone();
        advance_visible_script_until_player_boundary(&mut controller.shell).unwrap();
        assert!(controller.shell.bill_pc_box_cursor.is_some());
        assert!(controller.shell.bill_pc_box_action_cursor.is_none(),
            "{map}: native auto-advance must wait at the box list rather than press A");
        assert_eq!(controller.shell.shell.session().state(),&before_boxes);
        controller.press(GameButton::Down).unwrap();
        controller.press(GameButton::A).unwrap();
        advance_visible_script_until_player_boundary(&mut controller.shell).unwrap();
        assert!(controller.shell.bill_pc_box_action_cursor.is_some());
        assert!(controller.shell.save_flow.is_none(),"{map}: box actions must also wait for player input");
        controller.press(GameButton::A).unwrap();
        assert!(controller.shell.save_flow.is_some(),"{map}: SWITCH must ask to save");
        let intro=controller.snapshot().unwrap();
        assert!(intro.ui.menu.is_none()&&intro.ui.text.as_ref().and_then(|t|t.asm_text.as_deref())
            .is_some_and(|text|text.contains("When you change a")),"{map}: print the source CONT page before YES/NO");
        assert!(!scene_dialog_yes_no_active(&controller.shell.shell.presentation_snapshot().unwrap(),&controller.shell));
        assert!(visible_quick_save_blockers(&controller.shell,&controller.shell.shell.presentation_snapshot().unwrap(),true,true,true)
            .contains(&"text_window"),"the live Bill save introduction still owns the printer");
        clinic_pc_until(&mut controller,|s|s.special_boundary.is_none());
        #[cfg(feature="location-tester")]
        {
            assert!(render_walk::retained_pc_review_menu(&controller.shell,"pc:save-prompt").unwrap().is_some());
            assert!(render_walk::retained_pc_review_menu(&controller.shell,"pc:saved").unwrap().is_none());
        }
        let prompt=controller.snapshot().unwrap();
        assert!(visible_scene_dialog_entries(&controller.shell.shell.presentation_snapshot().unwrap(),&controller.shell)
            .unwrap().iter().any(|row|row.contains("will be saved. OK?")),
            "{map}: native rendering must show the save prompt over the suspended core chooser");
        assert!(prompt.ui.menu.as_ref().is_some_and(|menu|menu.layout.vertical_menus.iter()
            .flat_map(|m|m.options.iter()).any(|row|row.contains("will be saved. OK?"))),
            "{map}: show the actual save prompt: {:?}",prompt.ui);
        controller.press(GameButton::Down).unwrap();
        controller.press(GameButton::A).unwrap();
        assert!(controller.shell.save_flow.is_none());
        assert_eq!(controller.snapshot().unwrap().storage.current_pc_box,0);
        assert!(!save.exists(),"{map}: declining must not write a save");
        controller.press(GameButton::A).unwrap(); // selected box actions
        controller.press(GameButton::A).unwrap(); // SWITCH
        clinic_pc_until(&mut controller,|s|s.special_boundary.is_none());
        controller.press(GameButton::A).unwrap(); // YES
        assert_eq!(controller.snapshot().unwrap().storage.current_pc_box,1);
        assert!(save.is_file(),"{map}: reported save success without writing {}: {:?}",
            save.display(),controller.shell.last_audio_events);
        assert!(controller.shell.shell.snapshot().unwrap().ui.menu.is_some(),
            "{map}: writing a clean checkpoint must not close the live source PC owner");
        #[cfg(feature="location-tester")]
        assert!(render_walk::retained_pc_review_menu(&controller.shell,"pc:saved").unwrap().is_some());
        let mut reloaded=VisibleShellController::load_save(root.clone(),runtime.clone(),save.clone(),None).unwrap();
        // Source ChangeBoxSaveGame calls SaveBox before changing wCurBox.
        // Loading retains the saved box; the newly selected box is live state
        // until a later save. Do not turn this into a post-switch save.
        assert_eq!(reloaded.snapshot().unwrap().storage.current_pc_box,0,
            "{map}: preserve the source save-before-switch ordering on reload");
        assert_eq!(reloaded.shell.shell.session().state().storage.party,original.party);
        reloaded.press(GameButton::Start).unwrap();
        let restored_menu=reloaded.snapshot().unwrap().ui.menu.expect("restored save must expose Start");
        assert!(restored_menu.layout.vertical_menus.iter().flat_map(|m|m.options.iter())
            .any(|row|row.contains("SAVE")),"{map}: loading must not retain an invisible PC input owner");
        reloaded.press(GameButton::B).unwrap();
        let restored_at=reloaded.snapshot().unwrap().overworld.tile;
        for _ in 0..8 {
            reloaded.press(GameButton::Down).unwrap();
            if reloaded.snapshot().unwrap().overworld.tile!=restored_at {break;}
        }
        assert_eq!(reloaded.snapshot().unwrap().overworld.tile,TilePosition::new(restored_at.x,restored_at.y+1));
        controller.press(GameButton::A).unwrap(); // saved acknowledgement
        assert!(controller.shell.save_flow.is_none());
        controller.press(GameButton::B).unwrap(); // boxes -> Bill actions
        assert!(controller.shell.bill_pc_action_cursor.is_some());
        // DEPOSIT's party list also exposes RELEASE. Cancel it using B and
        // verify the actual party and the submenu's source cursor survive.
        controller.press(GameButton::Up).unwrap();
        controller.press(GameButton::A).unwrap();
        assert!(controller.shell.bill_pc_deposit_open);
        controller.press(GameButton::A).unwrap();
        controller.press(GameButton::Down).unwrap();
        controller.press(GameButton::Down).unwrap();
        controller.press(GameButton::A).unwrap();
        assert!(controller.shell.pending_pc_release.is_some());
        let before=controller.shell.shell.session().state().storage.clone();
        controller.press(GameButton::B).unwrap();
        assert!(controller.shell.pending_pc_release.is_none());
        assert_eq!(controller.shell.bill_pc_pokemon_action_cursor.as_ref().map(|c|c.option_index),Some(2));
        assert_eq!(controller.shell.shell.session().state().storage,before);
        controller.press(GameButton::A).unwrap(); // RELEASE again
        assert!(controller.shell.pending_pc_release.is_some());
        controller.press(GameButton::A).unwrap(); // YES
        assert_eq!(controller.snapshot().unwrap().party.slots.len(),1);
        clinic_pc_until(&mut controller,|s|s.pc_release_sequence.is_none()&&s.pc_notice.is_none());
        let survivor=controller.shell.shell.session().state().storage.party.clone();
        assert_eq!(survivor.pokemon[0],original.party.pokemon[1],
            "{map}: accepting RELEASE removes only the selected Pokemon and compacts the party");
        let wrong_before=controller.shell.last_audio_events.iter().filter(|event|event.contains("SFX_WRONG")).count();
        for button in [GameButton::A,GameButton::Down,GameButton::Down,GameButton::A] {
            controller.press(button).unwrap();
        }
        assert!(controller.shell.pending_pc_release.is_none(),"last party member must be refused");
        assert_eq!(controller.shell.last_audio_events.iter().filter(|event|event.contains("SFX_WRONG")).count(),wrong_before+1,
            "{map}: the production refusal must run, rather than silently ignoring RELEASE");
        assert_eq!(controller.shell.shell.session().state().storage.party,survivor);
        clinic_pc_until(&mut controller,|s|s.pc_transfer_sequence.is_none()&&s.pc_notice.is_none());
        for _ in 0..16 {
            if !controller.shell.pc_hub_session_open {break;}
            controller.press(GameButton::B).unwrap();
        }
        assert!(!controller.shell.pc_hub_session_open);
        let at=controller.snapshot().unwrap().overworld.tile;
        for _ in 0..8 {
            controller.press(GameButton::Down).unwrap();
            if controller.snapshot().unwrap().overworld.tile!=at {break;}
        }
        assert_eq!(controller.snapshot().unwrap().overworld.tile,TilePosition::new(at.x,at.y+1));
        assert_eq!(controller.shell.last_error,None);
        println!("{map}: real box/save/decline/reload/release-cancel/release/last-party-refusal/closure/walking passed");
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn clinic_bill_save_rejects_live_boot_text_and_pending_map_work() {
    let path=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(path.parent().unwrap());
    let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,
        crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap()).unwrap();
    let save=std::env::temp_dir().join(format!("geothite-clinic-blocked-{}.crystalsave",std::process::id()));
    assert!(!save.exists());
    let mut controller=clinic_pc_fixture(&root,&runtime,"PewterPokecenter1F",Some(save.clone()));
    controller.press(GameButton::Up).unwrap();controller.press(GameButton::A).unwrap();
    let boot=controller.shell.shell.snapshot().unwrap();
    assert!(visible_quick_save_blockers(&controller.shell,&boot,true,true,true).contains(&"text_window"));
    assert!(quick_save_from_bill_pc(&mut controller.shell).is_err(),
        "a refused write must propagate failure rather than report a saved game");
    assert!(!save.exists());
    clinic_pc_until(&mut controller,|s|s.pc_hub_cursor.is_some()&&s.special_boundary.is_none());
    controller.press(GameButton::A).unwrap();
    clinic_pc_until(&mut controller,|s|s.bill_pc_action_cursor.is_some()&&s.special_boundary.is_none());
    let mut retained=controller.shell.shell.snapshot().unwrap();
    assert!(retained.ui.text_window_open,"the source PCScript window is still suspended");
    assert!(!visible_quick_save_blockers(&controller.shell,&retained,true,true,true).contains(&"text_window"));
    assert!(visible_quick_save_blockers(&controller.shell,&retained,false,false,false).contains(&"text_window"),
        "ordinary saves retain the existing text ownership rule");
    retained.script_events.pending_map_load=Some(crystal_core::state::ScriptMapLoadRequest {
        command:"reloadmap".into(),map_setup:None,source_script:"SaveBoundaryFixture".into(),command_index:1,
    });
    assert!(visible_quick_save_blockers(&controller.shell,&retained,true,true,true).contains(&"auto_script"));
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn clinic_box_write_error_keeps_the_original_box_and_never_reports_saved() {
    let path=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(path.parent().unwrap());
    let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,
        crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap()).unwrap();
    let directory=std::env::temp_dir().join(format!("geothite-clinic-write-error-{}",std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let obstruction=directory.join("not-a-directory");std::fs::write(&obstruction,b"preserve").unwrap();
    let save=obstruction.join("game.crystalsave");
    let mut controller=clinic_pc_fixture(&root,&runtime,"PewterPokecenter1F",Some(save.clone()));
    controller.press(GameButton::Up).unwrap();controller.press(GameButton::A).unwrap();
    clinic_pc_until(&mut controller,|s|s.pc_hub_cursor.is_some()&&s.special_boundary.is_none());
    controller.press(GameButton::A).unwrap();
    clinic_pc_until(&mut controller,|s|s.bill_pc_action_cursor.is_some()&&s.special_boundary.is_none());
    for button in [GameButton::Down,GameButton::Down,GameButton::A,GameButton::Down,
        GameButton::A,GameButton::A] {controller.press(button).unwrap();}
    clinic_pc_until(&mut controller,|s|s.special_boundary.is_none());
    controller.press(GameButton::A).unwrap();
    assert!(matches!(controller.shell.save_flow.as_ref().map(|flow|flow.stage),Some(VisibleSaveFlowStage::Error)));
    assert_eq!(controller.snapshot().unwrap().storage.current_pc_box,0);
    assert!(!save.exists());
    assert_eq!(std::fs::read(&obstruction).unwrap(),b"preserve");
    assert!(!controller.shell.last_audio_events.iter().any(|event|event.contains("SFX_SAVE")));
    #[cfg(feature="location-tester")]
    {
        assert!(render_walk::retained_pc_review_menu(&controller.shell,"pc:save-error").unwrap().is_some());
        assert!(render_walk::retained_pc_review_menu(&controller.shell,"pc:saved").unwrap().is_none());
    }
    std::fs::remove_dir_all(directory).unwrap();
}
