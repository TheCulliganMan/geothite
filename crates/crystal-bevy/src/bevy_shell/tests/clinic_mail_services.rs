// Only the initial room/party/mailbox is a disposable fixture. Every service
// action, including read/confirmation/party cancellation, uses production input.
fn clinic_open_player_mailbox(controller:&mut VisibleShellController) {
    controller.press(GameButton::Up).unwrap();controller.press(GameButton::A).unwrap();
    clinic_pc_until(controller,|s|s.pc_hub_cursor.is_some()&&s.special_boundary.is_none()
        &&s.pc_notice.is_none()&&s.field_notice.is_none());
    controller.press(GameButton::Down).unwrap();controller.press(GameButton::A).unwrap();
    clinic_pc_until(controller,|s|s.player_pc_action_cursor.is_some()&&s.special_boundary.is_none()
        &&s.pc_notice.is_none()&&s.field_notice.is_none());
    for button in [GameButton::Down,GameButton::Down,GameButton::Down,GameButton::A] {
        controller.press(button).unwrap();
    }
}
fn clinic_mail_capture(controller:&mut VisibleShellController,label:&str) {
    if controller.snapshot().unwrap().overworld.map_name!="PewterPokecenter1F" {return;}
    let directory=std::env::var_os("CLINIC_MAIL_REVIEW_DIR").map(PathBuf::from);
    if let Some(directory)=&directory {std::fs::create_dir_all(directory).unwrap();}
    let snapshot=controller.snapshot().unwrap();let mut world=World::new();
    let mut queue=bevy::ecs::world::CommandQueue::default();let mut images=Assets::<Image>::default();
    let mut commands=Commands::new(&mut queue,&world);
    let mut art=RenderedTilesetArt::default();
    if controller.shell.party_menu_open {
        spawn_field_party_menu(&mut commands,&snapshot,&controller.shell,&mut art,
            &controller.shell.asset_root,&mut images).unwrap();
        spawn_pc_notice(&mut commands,&controller.shell,&mut art,&controller.shell.asset_root,&mut images).unwrap();
    } else {
        spawn_scene_dialog(&mut commands,&snapshot,&controller.shell,&mut art,
            &controller.shell.asset_root,&mut images).unwrap();
    }
    queue.apply(&mut world);
    assert_eq!(art.font_error,None,"{label}: use the actual bitmap font");
    if controller.shell.pc_notice.is_some() {
        let mut query=world.query::<(&Sprite,&Transform)>();
        assert!(query.iter(&world).any(|(sprite,t)|t.translation.z==4.5
            &&sprite.custom_size==Some(Vec2::new(20.*TILE_SIZE,6.*TILE_SIZE))),
            "{label}: source notice must be drawn even with a retained player-PC chooser");
    }
    if controller.shell.party_menu_open {
        // PrintText owns rows12..17 of the20x18 LCD. Neither its window nor
        // its two authored baselines may intrude into the party list above.
        let mut query=world.query::<(&Sprite,&Transform)>();
        for (sprite,transform) in query.iter(&world).filter(|(_,t)|t.translation.z>=4.5) {
            let size=sprite.custom_size.unwrap();
            let top=PLAYFIELD_TOP-transform.translation.y-size.y*0.5;
            assert!(top>=12.*TILE_SIZE-0.01 && top+size.y<=18.*TILE_SIZE+0.01,
                "{label}: party Mail notice must use its source PrintText rows");
        }
    }
    let canvas=render_pc_audit_canvas(&mut world,&images,label);
    if let Some(directory)=directory {canvas.save(directory.join(format!("{label}.png"))).unwrap();}
}
#[test]
#[ignore="requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn clinic_public_pcs_mailbox_reads_cancels_moves_and_attaches_real_mail_in_every_lobby() {
    let path=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(path.parent().unwrap());let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,
        crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap()).unwrap();
    for map in clinic_lobbies(&runtime) {
        let mut controller=clinic_pc_fixture(&root,&runtime,&map,None);
        let letters=[("FLOWER_MAIL","AB","HELLO FROM JOHTO"),("SURF_MAIL","CD","MEET IN KANTO")]
            .map(|(item,author,message)|crate::core::state::MailboxMail {
                item_id:item.into(),mail:crate::core::models::pokemon::MailData {
                    message:message.into(),author:author.into(),nationality:0,author_id:0x1234,
                    species:"CYNDAQUIL".into(),mail_type:item.into(),
                }});
        controller.shell.shell.session_mut().state_mut().mailbox=letters.to_vec();
        mark_runtime_snapshot_dirty(&mut controller.shell);
        let original=controller.shell.shell.session().state().storage.clone();
        let bag=controller.shell.shell.session().state().bag.clone();
        if map=="PewterPokecenter1F" && let Some(directory)=std::env::var_os("CLINIC_MAIL_REVIEW_DIR") {
            let directory=PathBuf::from(directory);std::fs::create_dir_all(&directory).unwrap();
            controller.save(directory.join("initial-mailbox-fixture.crystalsave")).unwrap();
        }
        clinic_open_player_mailbox(&mut controller);
        assert!(controller.shell.mailbox_cursor.is_some(),"{map}: ordinary PCScript -> player PC -> mailbox");
        #[cfg(feature="location-tester")]
        assert!(render_walk::retained_pc_review_menu(&controller.shell,"pc:mailbox").unwrap().is_some());
        let entries=visible_scene_dialog_entries(&controller.shell.shell.presentation_snapshot().unwrap(),&controller.shell).unwrap();
        assert_eq!(entries,[">AB"," CD"," CANCEL"]);
        clinic_mail_capture(&mut controller,"mailbox-list");
        controller.press(GameButton::Down).unwrap();controller.press(GameButton::A).unwrap();
        assert_eq!(controller.shell.mailbox_action_cursor.as_ref().map(|c|c.option_index),Some(0));
        controller.press(GameButton::A).unwrap();
        assert_eq!(controller.shell.pending_mail_read.as_ref().map(|r|&r.mail),Some(&letters[1].mail));
        assert!(retained_field_fullscreen_active(&controller.shell));
        #[cfg(feature="location-tester")]
        {
            let before=controller.shell.shell.state_checksum().unwrap();
            assert!(render_walk::pc_lcd_review_ready(&controller.shell,"pc:mail-read"));
            assert!(!render_walk::pc_lcd_review_ready(&controller.shell,"pc:move-carry"));
            assert!(render_walk::retained_pc_review_menu(&controller.shell,"pc:mailbox").unwrap().is_none());
            assert_eq!(before,controller.shell.shell.state_checksum().unwrap());
        }
        clinic_mail_capture(&mut controller,"mailbox-surf-reader");
        for button in [GameButton::Down,GameButton::Up,GameButton::Left,GameButton::Right,GameButton::Start,GameButton::Select] {
            controller.press(button).unwrap();
            assert_eq!(controller.shell.mailbox_cursor.as_ref().map(|c|c.option_index),Some(1),"reader owns {button:?}");
            assert_eq!(controller.shell.pending_mail_read.as_ref().map(|r|&r.mail),Some(&letters[1].mail));
            assert!(controller.shell.start_menu_cursor.is_none());
        }
        controller.press(GameButton::B).unwrap();assert!(controller.shell.pending_mail_read.is_none());
        assert_eq!(controller.shell.shell.session().state().mailbox,letters);
        // Decline destructive PUT IN PACK; the message and bag must survive.
        for b in [GameButton::A,GameButton::Down,GameButton::A] {controller.press(b).unwrap();}
        assert!(matches!(controller.shell.pc_confirmation,Some(VisiblePcConfirmation::PutMailInPack(1))));
        clinic_mail_capture(&mut controller,"mailbox-put-warning");
        controller.press(GameButton::Down).unwrap();controller.press(GameButton::A).unwrap();
        assert!(controller.shell.pc_confirmation.is_none());
        assert_eq!(controller.shell.shell.session().state().mailbox,letters);
        assert_eq!(controller.shell.shell.session().state().bag,bag);
        // Accept the same real action. Exactly SURF MAIL becomes a bag item.
        for b in [GameButton::A,GameButton::Down,GameButton::A,GameButton::A] {controller.press(b).unwrap();}
        assert_eq!(controller.shell.shell.session().state().mailbox,[letters[0].clone()]);
        let surf=&runtime.data().items["SURF_MAIL"];
        assert_eq!(controller.shell.shell.session().state().bag.quantity(surf),bag.quantity(surf)+1);
        assert_eq!(controller.shell.shell.session().state().storage,original);
        clinic_pc_until(&mut controller,|s|s.pc_notice.is_none());
        controller.press(GameButton::Up).unwrap();
        // Party cancellation must return to the selected message, unchanged.
        for b in [GameButton::A,GameButton::Down,GameButton::Down,GameButton::A] {controller.press(b).unwrap();}
        assert!(controller.shell.party_menu_open&&controller.shell.mailbox_cursor.is_none());
        controller.press(GameButton::B).unwrap();
        assert!(!controller.shell.party_menu_open);
        assert_eq!(controller.shell.mailbox_cursor.as_ref().map(|c|c.option_index),Some(0));
        assert_eq!(controller.shell.shell.session().state().storage,original);
        for b in [GameButton::A,GameButton::Down,GameButton::Down,GameButton::A,GameButton::A] {controller.press(b).unwrap();}
        assert!(controller.shell.party_menu_open&&controller.shell.mailbox_cursor.is_none());
        assert_eq!(controller.shell.pc_notice.as_deref(),Some("The MAIL was moved\nfrom the MAILBOX."));
        let state=controller.shell.shell.session().state();assert!(state.mailbox.is_empty());
        let attached=state.storage.party.pokemon[0].as_ref().unwrap();
        assert_eq!(attached.item.as_deref(),Some("FLOWER_MAIL"));assert_eq!(attached.mail.as_ref(),Some(&letters[0].mail));
        assert_eq!(state.storage.party.pokemon[1],original.party.pokemon[1]);
        clinic_mail_capture(&mut controller,"mailbox-attached-notice");
        clinic_pc_until(&mut controller,|s|s.pc_notice.is_none());
        assert!(!controller.shell.party_menu_open);
        assert_eq!(visible_scene_dialog_entries(&controller.shell.shell.presentation_snapshot().unwrap(),&controller.shell).unwrap(),[">CANCEL"]);
        controller.press(GameButton::B).unwrap();controller.press(GameButton::A).unwrap();
        assert!(controller.shell.pc_notice.as_deref().is_some_and(|s|s.contains("MAIL")));
        clinic_mail_capture(&mut controller,"mailbox-empty");
        clinic_pc_until(&mut controller,|s|s.pc_notice.is_none());
        for _ in 0..16 {if !controller.shell.pc_hub_session_open {break;}controller.press(GameButton::B).unwrap();}
        assert!(!controller.shell.pc_hub_session_open,"{map}: return from suspended PCScript");
        let at=controller.snapshot().unwrap().overworld.tile;
        for _ in 0..8 {
            controller.press(GameButton::Down).unwrap();
            if controller.snapshot().unwrap().overworld.tile!=at {break;}
        }
        assert_eq!(controller.snapshot().unwrap().overworld.tile,TilePosition::new(at.x,at.y+1),
            "{map}: restore real walking ownership after turning/settling");
        assert!(controller.shell.last_error.is_none(),"{map}: {:?}",controller.shell.last_error);
    }
}

#[test]
#[ignore="requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn clinic_public_pcs_mailbox_rejects_eggs_held_items_and_full_bags_without_losing_mail() {
    let path=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(path.parent().unwrap());let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,
        crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap()).unwrap();
    for map in clinic_lobbies(&runtime) {
        let mut controller=clinic_pc_fixture(&root,&runtime,&map,None);
        let letter=crate::core::state::MailboxMail {item_id:"FLOWER_MAIL".into(),
            mail:crate::core::models::pokemon::MailData {message:"KEEP THIS LETTER".into(),author:"AB".into(),
                nationality:0,author_id:0x1234,species:"CYNDAQUIL".into(),mail_type:"FLOWER_MAIL".into()}};
        let state=controller.shell.shell.session_mut().state_mut();
        state.mailbox=vec![letter.clone()];
        state.storage.party.pokemon[0].as_mut().unwrap().is_egg=true;
        state.storage.party.pokemon[1].as_mut().unwrap().item=Some("POTION".into());
        state.sync_party_from_storage();state.bag.items.clear();
        for (_,item) in runtime.data().items.iter().filter(|(id,i)|i.pocket==crate::core::models::ITEM_POCKET_ITEM
            &&!crate::core::models::item::is_mail_item_id(id)).take(crate::core::models::bag::ITEM_POCKET_CAPACITY) {
            assert!(state.bag.add_item(item,1).unwrap());
        }
        assert_eq!(state.bag.items.len(),crate::core::models::bag::ITEM_POCKET_CAPACITY);
        let storage=state.storage.clone();let bag=state.bag.clone();mark_runtime_snapshot_dirty(&mut controller.shell);
        clinic_open_player_mailbox(&mut controller);
        for b in [GameButton::A,GameButton::Down,GameButton::Down,GameButton::A,GameButton::A] {
            controller.press(b).unwrap();
        }
        assert_eq!(controller.shell.pc_notice.as_deref(),Some("An EGG can't hold\nany MAIL."));
        assert!(controller.shell.party_menu_open&&controller.shell.mailbox_cursor.is_none());
        clinic_mail_capture(&mut controller,"mailbox-egg-rejected");
        clinic_pc_until(&mut controller,|s|s.pc_notice.is_none());
        controller.press(GameButton::Down).unwrap();controller.press(GameButton::A).unwrap();
        assert_eq!(controller.shell.pc_notice.as_deref(),Some("It's already hold-\ning an item."));
        clinic_mail_capture(&mut controller,"mailbox-held-item-rejected");
        clinic_pc_until(&mut controller,|s|s.pc_notice.is_none());
        controller.press(GameButton::B).unwrap();
        assert!(!controller.shell.party_menu_open&&controller.shell.mailbox_cursor.is_some());
        for b in [GameButton::A,GameButton::Down,GameButton::A,GameButton::A] {controller.press(b).unwrap();}
        assert_eq!(controller.shell.pc_notice.as_deref(),Some("The PACK is full."));
        clinic_mail_capture(&mut controller,"mailbox-full-pack-rejected");
        let state=controller.shell.shell.session().state();
        assert_eq!(state.mailbox,[letter],"{map}: every refusal must retain the complete message");
        assert_eq!(state.storage,storage,"{map}: every refusal must preserve both Pokemon");
        assert_eq!(state.bag,bag,"{map}: no failed PUT IN PACK may consume another item");
        clinic_pc_until(&mut controller,|s|s.pc_notice.is_none());
        for _ in 0..16 {if !controller.shell.pc_hub_session_open {break;}controller.press(GameButton::B).unwrap();}
        assert!(!controller.shell.pc_hub_session_open);assert!(controller.shell.last_error.is_none());
    }
}

#[test]
fn mailbox_native_keyboard_samples_releases_between_list_submenu_and_reader() {
    fn press(app: &mut App, key: KeyCode) {
        // Hold through the source menu's WaitBGMap, clearing only host edges.
        // This is a physical press/release, not a direct service dispatch.
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(key);
        for _ in 0..8 {
            app.update();
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().clear();
        }
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(key);
        for _ in 0..8 {
            app.update();
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().clear();
        }
    }
    let initial = initialized_mail_reader_shell("FLOWER_MAIL");
    let mail = initial.shell.session().state().storage.party.pokemon[0]
        .as_ref().unwrap().mail.clone().unwrap();
    let mut controller = clinic_pc_fixture(&initial.asset_root, initial.shell.runtime(),
        "PewterPokecenter1F", None);
    controller.shell.shell.session_mut().state_mut().mailbox = vec![crate::core::state::MailboxMail {
        item_id: "FLOWER_MAIL".into(), mail: mail.clone(),
    }];
    mark_runtime_snapshot_dirty(&mut controller.shell);
    controller.press(GameButton::Up).unwrap();
    controller.press(GameButton::A).unwrap();
    clinic_pc_until(&mut controller, |s| s.pc_hub_cursor.is_some()
        && s.special_boundary.is_none() && s.pc_notice.is_none() && s.field_notice.is_none());
    controller.press(GameButton::Down).unwrap();
    controller.press(GameButton::A).unwrap();
    clinic_pc_until(&mut controller, |s| s.player_pc_action_cursor.is_some()
        && s.special_boundary.is_none() && s.pc_notice.is_none() && s.field_notice.is_none());
    for _ in 0..3 { controller.press(GameButton::Down).unwrap(); }
    let shell = controller.shell;
    let position = shell.shell.session().overworld().player.tile;
    let mut app = menu_render_test_app(shell);
    app.update();
    press(&mut app, KeyCode::KeyZ);
    assert!(app.world().resource::<BevyRuntimeShell>().mailbox_cursor.is_some());
    assert!(app.world().resource::<BevyRuntimeShell>().mailbox_action_cursor.is_none(),
        "the held parent confirmation must not also select the first letter");
    press(&mut app, KeyCode::KeyZ);
    assert!(app.world().resource::<BevyRuntimeShell>().mailbox_action_cursor.is_some());
    for _ in 0..8 { app.update(); }
    assert_eq!(app.world().resource::<BevyRuntimeShell>().pc_joypad.down, 0,
        "the native modal loop must sample a release while no button is pressed");
    press(&mut app, KeyCode::KeyZ);
    assert_eq!(app.world().resource::<BevyRuntimeShell>().pending_mail_read.as_ref().map(|r| &r.mail), Some(&mail),
        "a second physical A opens the actual selected Mail reader");
    for _ in 0..8 { app.update(); }
    press(&mut app, KeyCode::KeyX);
    assert!(app.world().resource::<BevyRuntimeShell>().pending_mail_read.is_none());
    for _ in 0..8 { app.update(); }
    assert!(app.world().resource::<BevyRuntimeShell>().mailbox_cursor.is_some(),
        "reader dismissal cannot also cancel the restored list");
    press(&mut app, KeyCode::KeyZ);
    assert!(app.world().resource::<BevyRuntimeShell>().mailbox_action_cursor.is_some());
    for _ in 0..8 { app.update(); }
    press(&mut app, KeyCode::ArrowDown);
    for _ in 0..8 { app.update(); }
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(shell.mailbox_action_cursor.as_ref().unwrap().option_index, 1);
    assert_eq!(shell.shell.session().overworld().player.tile, position);
    assert_eq!(shell.shell.session().state().mailbox[0].mail, mail);
    assert_eq!(shell.last_error, None);
}
