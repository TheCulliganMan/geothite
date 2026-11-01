#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn clinic_public_pcs_move_without_mail_saves_real_transfers_and_cancels_pickup_in_every_lobby() {
    let path=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(path.parent().unwrap());
    let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,
        crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap()).unwrap();
    let directory=std::env::temp_dir().join(format!("geothite-clinic-move-{}-{}",
        std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir_all(&directory).unwrap();
    for map in clinic_lobbies(&runtime) {
        let save=directory.join(format!("{map}.crystalsave"));
        let mut controller=clinic_pc_fixture(&root,&runtime,&map,Some(save.clone()));
        let original=controller.shell.shell.session().state().storage.clone();
        controller.press(GameButton::Up).unwrap();controller.press(GameButton::A).unwrap();
        clinic_pc_until(&mut controller,|s|s.pc_hub_cursor.is_some()&&s.special_boundary.is_none());
        controller.press(GameButton::A).unwrap();
        clinic_pc_until(&mut controller,|s|s.bill_pc_action_cursor.is_some()&&s.special_boundary.is_none());
        for button in [GameButton::Down,GameButton::Down,GameButton::Down,GameButton::A] {
            controller.press(button).unwrap();
        }
        let intro=controller.snapshot().unwrap();
        assert!(intro.ui.menu.is_none()&&intro.ui.text.as_ref().and_then(|t|t.asm_text.as_deref())
            .is_some_and(|text|text.contains("Each time you move")),"{map}: show the source CONT introduction before YES/NO");
        assert!(!scene_dialog_yes_no_active(&controller.shell.shell.presentation_snapshot().unwrap(),&controller.shell));
        clinic_pc_until(&mut controller,|s|s.special_boundary.is_none());
        let prompt=controller.snapshot().unwrap();
        assert!(prompt.ui.menu.as_ref().is_some_and(|menu|menu.layout.vertical_menus.iter()
            .flat_map(|m|m.options.iter()).any(|row|row.contains("will be saved. OK?"))));
        controller.press(GameButton::Down).unwrap();controller.press(GameButton::A).unwrap();
        assert!(!save.exists());assert!(!controller.shell.bill_pc_move_open);
        assert_eq!(controller.shell.shell.session().state().storage,original);
        controller.press(GameButton::A).unwrap(); // MOVE again; source cursor remains on that action
        clinic_pc_until(&mut controller,|s|s.special_boundary.is_none());
        controller.press(GameButton::A).unwrap();
        assert!(save.is_file());
        controller.press(GameButton::A).unwrap(); // saved -> actual MOVE list
        assert!(controller.shell.bill_pc_move_open&&!controller.shell.bill_pc_move_party_open);
        #[cfg(feature="location-tester")]
        assert!(render_walk::retained_pc_review_menu(&controller.shell,"pc:move-carry").unwrap().is_none());
        let entry_save=std::fs::read(&save).unwrap();
        controller.press(GameButton::Left).unwrap();
        assert!(controller.shell.bill_pc_move_party_open);
        controller.press(GameButton::A).unwrap();controller.press(GameButton::A).unwrap();
        assert!(controller.shell.bill_pc_move_source.is_some());
        #[cfg(feature="location-tester")]
        {
            let before=controller.shell.shell.state_checksum().unwrap();
            assert!(render_walk::retained_pc_review_menu(&controller.shell,"pc:move-carry").unwrap().is_some());
            assert!(retained_field_fullscreen_active(&controller.shell));
            assert_eq!(controller.shell.shell.state_checksum().unwrap(),before,"QA observes an actual carry owner without changing it");
        }
        controller.press(GameButton::B).unwrap();
        assert!(controller.shell.bill_pc_move_source.is_none());
        #[cfg(feature="location-tester")]
        assert!(render_walk::retained_pc_review_menu(&controller.shell,"pc:move-carry").unwrap().is_none());
        assert!(controller.shell.bill_pc_move_party_open);
        assert_eq!(controller.shell.storage_cursor.as_ref().map(|c|c.option_index),Some(0));
        assert_eq!(controller.shell.shell.session().state().storage,original);
        assert_eq!(std::fs::read(&save).unwrap(),entry_save,"{map}: cancelling pickup must not autosave");
        // Pick up party Cyndaquil and insert it into the empty first box.
        for button in [GameButton::A,GameButton::A,GameButton::Right,GameButton::A] {
            controller.press(button).unwrap();
        }
        clinic_pc_until(&mut controller,|s|s.bill_pc_move_save.is_none());
        let stored=controller.shell.shell.session().state().storage.clone();
        assert_eq!(stored.party.filled_slots(),1);
        assert_eq!(stored.party.pokemon[0],original.party.pokemon[1]);
        assert_eq!(stored.pc_boxes[0].count,1);
        assert_eq!(stored.pc_boxes[0].pokemon[0],original.party.pokemon[0]);
        assert_eq!(controller.shell.shell.runtime().load_save(&save).unwrap().storage,stored,
            "{map}: the post-insertion storage must be the actual written save");
        if map=="PewterPokecenter1F" && let Some(directory)=std::env::var_os("CLINIC_MOVE_REVIEW_DIR") {
            let directory=PathBuf::from(directory);std::fs::create_dir_all(&directory).unwrap();
            // Reuse the real per-move autosave, including its clean Continue
            // boundary. No boxed Pokemon or PC menu is seeded for rendering.
            std::fs::copy(&save,directory.join("move-after-party-box-PewterPokecenter1F.crystalsave")).unwrap();
        }
        // Box0 -> Box1 uses browsing, not CHANGE BOX's authoritative selector.
        for button in [GameButton::A,GameButton::A,GameButton::Right,GameButton::A] {
            controller.press(button).unwrap();
        }
        clinic_pc_until(&mut controller,|s|s.bill_pc_move_save.is_none());
        let moved=controller.shell.shell.session().state().storage.clone();
        assert_eq!(moved.pc_boxes[0].count,0);assert_eq!(moved.pc_boxes[1].count,1);
        assert_eq!(moved.pc_boxes[1].pokemon[0],original.party.pokemon[0]);
        assert_eq!(controller.snapshot().unwrap().storage.current_pc_box,0);
        assert_eq!(controller.shell.shell.runtime().load_save(&save).unwrap().storage,moved);
        // Box1 -> Party appends Cyndaquil behind the original Totodile.
        for button in [GameButton::A,GameButton::A,GameButton::Left,GameButton::Left,
            GameButton::Down,GameButton::A] {controller.press(button).unwrap();}
        clinic_pc_until(&mut controller,|s|s.bill_pc_move_save.is_none());
        let restored=controller.shell.shell.session().state().storage.clone();
        assert_eq!(restored.party.filled_slots(),2);
        assert_eq!(restored.party.pokemon[0],original.party.pokemon[1]);
        assert_eq!(restored.party.pokemon[1],original.party.pokemon[0]);
        assert!(restored.pc_boxes.iter().all(|b|b.count==0));
        assert_eq!(controller.shell.shell.runtime().load_save(&save).unwrap().storage,restored);
        let mut loaded=VisibleShellController::load_save(root.clone(),runtime.clone(),save.clone(),None).unwrap();
        loaded.press(GameButton::Start).unwrap();
        assert!(loaded.snapshot().unwrap().ui.menu.unwrap().layout.vertical_menus.iter()
            .flat_map(|m|m.options.iter()).any(|row|row.contains("SAVE")),"{map}: a per-move save must load without an invisible PC owner");
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
        println!("{map}: source MOVE prompt/decline/save/pickup-cancel/party-box/box-box/box-party/autosave/reload/closure/walking passed");
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn clinic_public_pcs_move_mail_gate_prints_both_source_pages_and_preserves_storage() {
    let path=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(path.parent().unwrap());
    let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,
        crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap()).unwrap();
    let directory=std::env::temp_dir().join(format!("geothite-clinic-mail-gate-{}",std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    for map in clinic_lobbies(&runtime) {for slot in [0,1] {
        let save=directory.join(format!("{map}-{slot}.crystalsave"));
        let mut controller=clinic_pc_fixture(&root,&runtime,&map,Some(save.clone()));
        // Explicit initial fixture: either real party slot holds complete Mail.
        // No storage, item, script or menu state is patched after PC input starts.
        let state=controller.shell.shell.session_mut().state_mut();
        let pokemon=state.storage.party.pokemon[slot].as_mut().unwrap();
        pokemon.item=Some("FLOWER_MAIL".into());
        pokemon.mail=Some(crate::core::models::pokemon::MailData {
            message:"HELLO FROM JOHTO".into(),author:"CHRIS".into(),nationality:0,author_id:1,
            species:pokemon.species.id.clone(),mail_type:"FLOWER_MAIL".into(),
        });
        state.sync_party_from_storage();mark_runtime_snapshot_dirty(&mut controller.shell);
        let original=controller.shell.shell.session().state().storage.clone();
        controller.press(GameButton::Up).unwrap();
        if let Some(directory)=std::env::var_os("CLINIC_MOVE_REVIEW_DIR") {
            let directory=PathBuf::from(directory);std::fs::create_dir_all(&directory).unwrap();
            controller.save(directory.join(format!("mail-start-{map}-{slot}.crystalsave"))).unwrap();
        }
        controller.press(GameButton::A).unwrap();
        clinic_pc_until(&mut controller,|s|s.pc_hub_cursor.is_some()&&s.special_boundary.is_none());
        controller.press(GameButton::A).unwrap();
        clinic_pc_until(&mut controller,|s|s.bill_pc_action_cursor.is_some()&&s.special_boundary.is_none());
        for button in [GameButton::Down,GameButton::Down,GameButton::Down,GameButton::A] {
            controller.press(button).unwrap();
        }
        assert!(controller.shell.save_flow.is_none()&&!controller.shell.bill_pc_move_open);
        let first=controller.snapshot().unwrap();
        assert!(first.ui.menu.is_none()&&first.ui.text.as_ref().and_then(|t|t.asm_text.as_deref())
            .is_some_and(|t|t.contains("There is a")&&t.contains("holding MAIL.")));
        for direction in [GameButton::Up,GameButton::Down,GameButton::Left,GameButton::Right] {
            controller.press(direction).unwrap();
            assert_eq!(controller.shell.bill_pc_action_cursor.as_ref().map(|c|c.option_index),Some(3),
                "{map}: a direction must not move the Bill cursor hidden behind the source Mail page");
        }
        assert_eq!(controller.snapshot().unwrap().overworld.tile,first.overworld.tile);
        controller.press(GameButton::A).unwrap();
        let second=controller.snapshot().unwrap();
        assert!(second.ui.menu.is_none()&&second.ui.text.as_ref().and_then(|t|t.asm_text.as_deref())
            .is_some_and(|t|t.contains("Please remove the")&&t.contains("MAIL.")),
            "{map}, Mail slot{slot}: the source's second page must remain visible: {:?}",second.ui);
        for direction in [GameButton::Up,GameButton::Down,GameButton::Left,GameButton::Right] {
            controller.press(direction).unwrap();
            assert_eq!(controller.shell.bill_pc_action_cursor.as_ref().map(|c|c.option_index),Some(3));
        }
        controller.press(GameButton::A).unwrap();
        assert!(controller.shell.special_boundary.is_none()&&controller.shell.pc_notice.is_none());
        assert_eq!(controller.shell.bill_pc_action_cursor.as_ref().map(|c|c.option_index),Some(3));
        assert!(!save.exists());
        assert_eq!(controller.shell.shell.session().state().storage,original);
        assert!(!controller.shell.last_audio_events.iter().any(|event|event.contains("SFX_SAVE")));
        println!("{map}, Mail slot{slot}: real MOVE gate/source pages/blocked movement/storage preservation passed");
    }}
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn clinic_move_insertion_write_failure_preserves_storage_and_can_retry() {
    let path=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(path.parent().unwrap());
    let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,
        crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap()).unwrap();
    let directory=std::env::temp_dir().join(format!("geothite-clinic-move-recovery-{}",std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let live=directory.join("live");std::fs::create_dir(&live).unwrap();
    let save=live.join("game.crystalsave");
    let mut controller=clinic_pc_fixture(&root,&runtime,"PewterPokecenter1F",Some(save.clone()));
    let original=controller.shell.shell.session().state().storage.clone();
    controller.press(GameButton::Up).unwrap();controller.press(GameButton::A).unwrap();
    clinic_pc_until(&mut controller,|s|s.pc_hub_cursor.is_some()&&s.special_boundary.is_none());
    controller.press(GameButton::A).unwrap();
    clinic_pc_until(&mut controller,|s|s.bill_pc_action_cursor.is_some()&&s.special_boundary.is_none());
    for button in [GameButton::Down,GameButton::Down,GameButton::Down,GameButton::A] {
        controller.press(button).unwrap();
    }
    clinic_pc_until(&mut controller,|s|s.special_boundary.is_none());
    controller.press(GameButton::A).unwrap();controller.press(GameButton::A).unwrap();
    let entry=std::fs::read(&save).unwrap();
    let save_sounds=controller.shell.last_audio_events.iter().filter(|event|event.contains("SFX_SAVE")).count();
    // Obstruct only the filesystem after the actual entry save. No live
    // Pokemon, cursor, script or input owner is changed by the fixture.
    let retained=directory.join("retained");std::fs::rename(&live,&retained).unwrap();
    std::fs::write(&live,b"keep obstruction").unwrap();
    let mut insertion_error=None;
    for button in [GameButton::Left,GameButton::A,GameButton::A,GameButton::Right,GameButton::A] {
        if let Err(error)=controller.press(button) {insertion_error=Some(error);break;}
    }
    assert_eq!(controller.shell.shell.session().state().storage,original,
        "a failed write must not commit the in-memory insertion: {insertion_error:?}");
    assert!(insertion_error.is_none(),"the PC must present a recoverable failure notice");
    controller.wait_frames(64).unwrap();
    assert_eq!(controller.shell.shell.session().state().storage,original,
        "failed insertion must roll back before another host frame can move the next Pokemon");
    assert!(controller.shell.bill_pc_move_save.is_none(),"do not retain a commit that retries itself");
    assert!(controller.shell.bill_pc_move_source.is_some(),"retain the original pickup for a deliberate retry");
    assert!(controller.shell.pc_notice.as_deref().is_some_and(|text|text.contains("SAVE FAILED")));
    assert_eq!(controller.shell.last_audio_events.iter().filter(|event|event.contains("SFX_SAVE")).count(),save_sounds,
        "the successful entry save may sound; the failed insertion must not");
    let failed=controller.snapshot().unwrap();
    assert!(failed.ui.text.as_ref().and_then(|t|t.asm_text.as_deref()).is_some_and(|text|text.contains("SAVE FAILED")),
        "the failure must reach the production visible text surface");
    // Render the actual reached error owner through the production LCD
    // compositor. This is UI evidence, not a substitute for native room QA.
    let mut world=World::new();
    let mut queue=bevy::ecs::world::CommandQueue::default();
    let mut images=Assets::<Image>::default();
    let mut commands=Commands::new(&mut queue,&world);
    spawn_field_storage_screen(&mut commands,&failed,&controller.shell,
        &mut RenderedTilesetArt::default(),&controller.shell.asset_root,&mut images).unwrap();
    queue.apply(&mut world);
    let canvas=render_pc_audit_canvas(&mut world,&images,"pc-move-save-failure");
    if let Some(directory)=std::env::var_os("CLINIC_MOVE_RECOVERY_REVIEW_DIR") {
        let directory=PathBuf::from(directory);std::fs::create_dir_all(&directory).unwrap();
        canvas.save(directory.join("insertion-save-failure.png")).unwrap();
    }
    assert_eq!(std::fs::read(retained.join("game.crystalsave")).unwrap(),entry);
    assert_eq!(std::fs::read(&live).unwrap(),b"keep obstruction");
    std::fs::remove_file(&live).unwrap();std::fs::rename(&retained,&live).unwrap();
    controller.press(GameButton::A).unwrap(); // acknowledge failure
    controller.press(GameButton::A).unwrap(); // retry the same insertion
    controller.wait_frames(64).unwrap();
    assert!(controller.shell.bill_pc_move_save.is_none());
    let stored=controller.shell.shell.session().state().storage.clone();
    assert_eq!(stored.party.filled_slots(),1);
    assert_eq!(stored.party.pokemon[0],original.party.pokemon[1]);
    assert_eq!(stored.pc_boxes[0].count,1);
    assert_eq!(stored.pc_boxes[0].pokemon[0],original.party.pokemon[0]);
    assert_eq!(controller.shell.shell.runtime().load_save(&save).unwrap().storage,stored);
    assert!(controller.shell.bill_pc_move_source.is_none());
    std::fs::remove_dir_all(directory).unwrap();
}
