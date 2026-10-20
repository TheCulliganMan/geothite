// Set up only the initial room/party/inventory. Every PC operation below goes
// through the production controller, including source menu/input timing.
fn clinic_item_key(controller:&mut VisibleShellController,key:GameButton) {
    // Release the previous physical key and cross the source WaitBGMap fence.
    controller.wait_frames(8).unwrap();
    controller.press(key).unwrap();
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn item_pc_live_lcd_refreshes_printed_text_quantity_and_yes_no_cursor() {
    fn glyphs(app:&mut App)->Vec<(u64,String)> {
        let world=app.world_mut();let mut query=world.query::<(&DialogGlyphMarker,&Sprite)>();
        let mut glyphs=query.iter(world).map(|(m,s)|(m.key,format!("{:?}",s.image.id()))).collect::<Vec<_>>();
        glyphs.sort();glyphs
    }
    // A retained LCD phase fixture exercises real render invalidation and
    // keyboard dispatch. PCScript entry is covered independently in all22 lobbies.
    let mut shell=initialized_mail_reader_shell("FLOWER_MAIL");
    let potion=shell.shell.runtime().data().items["POTION"].clone();
    shell.shell.session_mut().state_mut().bag.pc_items.clear();
    assert!(shell.shell.session_mut().state_mut().bag.add_pc_item(&potion,3).unwrap());
    shell.pc_item_action=Some(VisiblePlayerPcAction::TossItem);
    shell.pc_item_cursor=Some(MenuCursor{surface_id:"pc:items".into(),option_index:0});
    shell.pc_item_quantity=Some(VisiblePcItemQuantity{action:VisiblePlayerPcAction::TossItem,
        item_id:"POTION".into(),stack_index:0,quantity:1,maximum:3});
    let question="Toss out how many\nPOTION(S)?";
    shell.pc_notice=Some(question.into());
    shell.field_text_reveal=Some(VisibleFieldTextReveal{text:question.into(),page_index:0,visible_chars:0,frames_until_next_char:1});
    let mut app=menu_render_test_app(shell);app.update();let initial=glyphs(&mut app);
    for _ in 0..96 {app.update();}
    assert!(visible_pc_item_quantity_input_ready(app.world().resource::<BevyRuntimeShell>()));
    let printed=glyphs(&mut app);assert_ne!(initial,printed,"printer progress must refresh retained PC glyphs");
    press_key_for_runtime_hotkey_app(&mut app,KeyCode::ArrowUp);
    assert_eq!(app.world().resource::<BevyRuntimeShell>().pc_item_quantity.as_ref().unwrap().quantity,2);
    assert_ne!(printed,glyphs(&mut app),"the number window must repaint without changing its inventory row");
    press_key_for_runtime_hotkey_app(&mut app,KeyCode::KeyZ);
    for _ in 0..96 {app.update();}
    assert!(visible_pc_toss_confirmation_ready(app.world().resource::<BevyRuntimeShell>()));
    let yes=glyphs(&mut app);
    press_key_for_runtime_hotkey_app(&mut app,KeyCode::ArrowDown);
    assert_eq!(app.world().resource::<BevyRuntimeShell>().yes_no_cursor.as_ref().unwrap().option_index,1);
    assert_ne!(yes,glyphs(&mut app),"the actual YesNoBox arrow must repaint");
    press_key_for_runtime_hotkey_app(&mut app,KeyCode::KeyX);
    assert_eq!(app.world().resource::<BevyRuntimeShell>().shell.session().state().bag.pc_item_quantity(&potion),3);
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn item_pc_toss_input_waits_until_its_authored_question_is_printed() {
    // Isolated printer/input-owner phase fixture. The separate all-lobby
    // integration tests enter these menus exclusively through real PCScript.
    let mut shell=initialized_mail_reader_shell("FLOWER_MAIL");
    let potion=shell.shell.runtime().data().items["POTION"].clone();
    shell.shell.session_mut().state_mut().bag.pc_items.clear();
    assert!(shell.shell.session_mut().state_mut().bag.add_pc_item(&potion,3).unwrap());
    let snapshot=shell.shell.snapshot().unwrap();
    let question=visible_pc_item_source_text(&shell,&snapshot,"_ItemsThrowAwayText","POTION",1).unwrap();
    let buttons:&[(&str,fn(&mut BevyRuntimeShell)->Result<()>)]=&[
        ("A",press_visible_a_button),("B",press_visible_b_button),
        ("Up",move_visible_primary_cursor_up),("Down",move_visible_primary_cursor_down),
        ("Left",move_visible_primary_cursor_left),("Right",move_visible_primary_cursor_right)];
    for &(name,press) in buttons {
        shell.pc_confirmation=Some(VisiblePcConfirmation::TossItem {item_id:"POTION".into(),stack_index:0,quantity:1});
        shell.yes_no_cursor=Some(MenuCursor{surface_id:"pc:confirmation".into(),option_index:0});
        shell.pc_notice=Some(question.clone());
        shell.field_text_reveal=Some(VisibleFieldTextReveal{text:question.clone(),page_index:0,visible_chars:1,frames_until_next_char:1});
        press(&mut shell).unwrap();
        assert!(matches!(shell.pc_confirmation,Some(VisiblePcConfirmation::TossItem{..})),"{name}: source MenuTextbox still owns input");
        assert_eq!(shell.yes_no_cursor.as_ref().unwrap().option_index,0);
        assert_eq!(shell.shell.session().state().bag.pc_item_quantity(&potion),3);
    }
    shell.field_text_reveal.as_mut().unwrap().visible_chars=question.chars().count();
    assert!(visible_pc_toss_confirmation_ready(&shell));
    move_visible_primary_cursor_up(&mut shell).unwrap();
    assert_eq!(shell.yes_no_cursor.as_ref().unwrap().option_index,0,"YES does not wrap upward");
    move_visible_primary_cursor_down(&mut shell).unwrap();
    move_visible_primary_cursor_down(&mut shell).unwrap();
    move_visible_primary_cursor_left(&mut shell).unwrap();
    move_visible_primary_cursor_right(&mut shell).unwrap();
    assert_eq!(shell.yes_no_cursor.as_ref().unwrap().option_index,1);
    press_visible_a_button(&mut shell).unwrap();
    assert_eq!(shell.mailbox_confirmation_response,Some(false));
    for _ in 0..14 {
        apply_visible_runtime_controls(&ButtonInput::default(),&mut shell,true);
        assert!(shell.pc_confirmation.is_some(),"source YesNoBox stays through the closing delay");
    }
    apply_visible_runtime_controls(&ButtonInput::default(),&mut shell,true);
    assert!(shell.pc_confirmation.is_none());
    assert_eq!(shell.shell.session().state().bag.pc_item_quantity(&potion),3,"NO preserves the complete stack");
}

#[test]
#[cfg(feature="location-tester")]
fn item_pc_monochrome_capture_requires_live_lcd_proof_and_contrast() {
    let path=std::env::temp_dir().join(format!("geothite-pc-readback-{}-{}.png",std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let mut image=image::RgbaImage::from_fn(640,576,|x,y|image::Rgba(
        [[0,0,0,255],[255,255,255,255],[255,255,128,255]][((x/8+y/8)%3)as usize]));
    image.save(&path).unwrap();
    assert!(validate_render_test_screenshot(&path).is_err(),"ordinary scene policy still requires four colors");
    assert!(validate_render_test_screenshot_with_pc_lcd(&path,true).is_ok(),"only the actual LCD-owner proof admits three colors");
    for pixel in image.pixels_mut() {*pixel=image::Rgba([255,255,128,255]);}
    image.save(&path).unwrap();
    assert!(validate_render_test_screenshot_with_pc_lcd(&path,true).is_err(),"LCD proof cannot admit a blank frame");
    std::fs::remove_file(path).unwrap();
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn clinic_item_pc_authored_notices_fit_every_source_item_and_quantity() {
    let path=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(path.parent().unwrap());let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,
        crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap()).unwrap();
    let c=clinic_pc_fixture(&root,&runtime,"PewterPokecenter1F",None);
    let snapshot=c.shell.shell.snapshot().unwrap();let before=c.shell.shell.state_checksum().unwrap();
    for item in snapshot.items.iter() {for quantity in [1,99] {
        let name=item_display_name(&snapshot,&item.item_id);
        let text=visible_pc_item_source_text(&c.shell,&snapshot,"_ItemsThrowAwayText",&name,quantity).unwrap();
        assert_eq!(text,format!("Throw away {quantity}\n{}(S)?",normalize_boot_text(&name)));
        assert!(text.lines().count()==2&&text.lines().all(|line|line.chars().count()<=18),"{}: {text:?}",item.item_id);
    }}
    for label in ["_ItemsTooImportantText","_ItemsDiscardedText","_PlayersPCNoRoomDepositText","_PlayersPCNoRoomWithdrawText"] {
        let text=visible_pc_item_source_text(&c.shell,&snapshot,label,"BRIGHTPOWDER",99).unwrap();
        assert_eq!(text.lines().count(),2);
        assert!(text.lines().all(|line|line.chars().count()<=18),"{label}: {text:?}");
    }
    assert_eq!(c.shell.shell.state_checksum().unwrap(),before,"drawing notices cannot mutate gameplay");
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn clinic_item_pc_uses_all_four_pockets_and_protected_single_item_rules() {
    let path=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(path.parent().unwrap());let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,
        crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap()).unwrap();
    for (id,pocket,right,prompt) in [("POTION",FieldPackPocket::Items,0,true),
        ("POKE_BALL",FieldPackPocket::Balls,1,true),("BICYCLE",FieldPackPocket::KeyItems,2,false),
        ("TM_MUD_SLAP",FieldPackPocket::TmHm,3,true),("HM_CUT",FieldPackPocket::TmHm,3,false)] {
        let mut c=clinic_pc_fixture(&root,&runtime,"PewterPokecenter1F",None);
        let item=runtime.data().items[id].clone();let n=if prompt {3}else{1};
        let bag=&mut c.shell.shell.session_mut().state_mut().bag;
        bag.items.clear();bag.balls.clear();bag.key_items.clear();bag.tm_hm.fill(0);bag.pc_items.clear();bag.custom_pockets.clear();
        assert!(bag.add_item(&item,n).unwrap());let initial=bag.clone();
        if id=="BICYCLE" {c.shell.shell.session_mut().state_mut().registered_key_item=Some(id.into());}
        clinic_open_player_pc(&mut c);
        for k in [GameButton::Down,GameButton::A] {clinic_item_key(&mut c,k);}
        for _ in 0..right {clinic_item_key(&mut c,GameButton::Right);}
        assert_eq!(active_visible_field_pack_pocket(&c.shell),pocket);
        clinic_item_key(&mut c,GameButton::A);
        if prompt {
            assert!(c.shell.pc_item_quantity.is_some());
            for k in [GameButton::Up,GameButton::A] {clinic_item_key(&mut c,k);}
        } else {assert!(c.shell.pc_item_quantity.is_none(),"{id}: protected items transfer exactly one");}
        clinic_pc_until(&mut c,|s|s.pc_notice.is_none()&&s.pc_item_quantity.is_none());
        assert_eq!(c.shell.shell.session().state().bag.pc_item_quantity(&item),if prompt {2}else{1});
        assert_eq!(active_visible_field_pack_pocket(&c.shell),pocket);
        if id=="BICYCLE" {assert_eq!(c.shell.shell.session().state().registered_key_item,None);}
        for k in [GameButton::B,GameButton::Up,GameButton::A,GameButton::A] {clinic_item_key(&mut c,k);}
        if prompt {for k in [GameButton::Up,GameButton::A] {clinic_item_key(&mut c,k);}}
        else {assert!(c.shell.pc_item_quantity.is_none());}
        clinic_pc_until(&mut c,|s|s.pc_notice.is_none()&&s.pc_item_quantity.is_none());
        assert_eq!(c.shell.shell.session().state().bag,initial,"{id}: the real source pocket receives the complete round trip");
        assert_eq!(c.shell.last_error,None);
        println!("item PC {id}: source pocket and protected quantity passed");
    }
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn clinic_item_pc_capacity_refusals_and_toss_confirmation_preserve_items() {
    let path=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(path.parent().unwrap());let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,
        crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap()).unwrap();
    let potion=runtime.data().items["POTION"].clone();let bicycle=runtime.data().items["BICYCLE"].clone();
    for case in ["empty-deposit","full-pc","full-pack","toss"] {
        let mut c=clinic_pc_fixture(&root,&runtime,"PewterPokecenter1F",None);
        let bag=&mut c.shell.shell.session_mut().state_mut().bag;
        bag.items.clear();bag.balls.clear();bag.key_items.clear();bag.tm_hm.fill(0);bag.pc_items.clear();bag.custom_pockets.clear();
        if case!="empty-deposit" {assert!(bag.add_item(&potion,if case=="full-pack" {
            crate::core::models::MAX_ITEM_STACK*u16::try_from(crate::core::models::ITEM_POCKET_CAPACITY).unwrap()
        }else{3}).unwrap());}
        assert!(bag.add_pc_item(&potion,if case=="full-pc" {
            crate::core::models::MAX_ITEM_STACK*u16::try_from(crate::core::models::PC_ITEM_CAPACITY).unwrap()
        }else{3}).unwrap());
        if case=="toss" {assert!(bag.add_pc_item(&bicycle,1).unwrap());}
        let initial=bag.clone();clinic_open_player_pc(&mut c);
        if matches!(case,"empty-deposit"|"full-pc") {clinic_item_key(&mut c,GameButton::Down);}
        if case=="toss" {for _ in 0..2 {clinic_item_key(&mut c,GameButton::Down);}}
        clinic_item_key(&mut c,GameButton::A);
        if case=="empty-deposit" {
            assert_eq!(c.shell.pc_notice.as_deref(),Some("No items here!"));
            assert!(!visible_field_pack_is_open(&c.shell));
        } else {
            clinic_item_key(&mut c,GameButton::A);
            assert!(c.shell.pc_item_quantity.is_some());
            clinic_item_key(&mut c,GameButton::A);
            if case=="toss" {
                assert!(matches!(c.shell.pc_confirmation,Some(VisiblePcConfirmation::TossItem{quantity:1,..})));
                clinic_item_capture(&mut c,"toss-confirmation");
                for k in [GameButton::Down,GameButton::A] {clinic_item_key(&mut c,k);}
                assert_eq!(c.shell.shell.session().state().bag,initial,"declining toss preserves all items");
                for k in [GameButton::A,GameButton::A,GameButton::A] {clinic_item_key(&mut c,k);}
                assert_eq!(c.shell.shell.session().state().bag.pc_item_quantity(&potion),2);
                clinic_pc_until(&mut c,|s|s.pc_notice.is_none());
                for k in [GameButton::Down,GameButton::A] {clinic_item_key(&mut c,k);}
                assert_eq!(c.shell.pc_notice.as_deref(),Some("That's too impor-\ntant to toss out!"));
                assert!(c.shell.pc_item_quantity.is_none()&&c.shell.pc_confirmation.is_none());
                assert_eq!(c.shell.shell.session().state().bag.pc_item_quantity(&bicycle),1);
                clinic_item_capture(&mut c,"protected-item-refusal");
                assert_eq!(c.shell.last_error,None);
                continue;
            }
            assert_eq!(c.shell.pc_notice.as_deref(),Some(if case=="full-pc" {"There's no room to\nstore items."}else{"There's no room\nfor more items."}));
        }
        assert_eq!(c.shell.shell.session().state().bag,initial,"{case}: refused transfers are atomic");
        clinic_item_capture(&mut c,case);
        assert_eq!(c.shell.last_error,None);
    }
}

fn clinic_open_player_pc(controller:&mut VisibleShellController) {
    clinic_item_key(controller,GameButton::Up);
    clinic_item_key(controller,GameButton::A);
    clinic_pc_until(controller,|s|s.pc_hub_cursor.is_some()&&s.special_boundary.is_none()
        &&s.pc_notice.is_none()&&s.field_notice.is_none());
    clinic_item_key(controller,GameButton::Down);
    clinic_item_key(controller,GameButton::A);
    clinic_pc_until(controller,|s|s.player_pc_action_cursor.is_some()&&s.special_boundary.is_none()
        &&s.pc_notice.is_none()&&s.field_notice.is_none());
}

fn clinic_item_capture(controller:&mut VisibleShellController,label:&str) {
    if controller.snapshot().unwrap().overworld.map_name!="PewterPokecenter1F" {return;}
    let Some(directory)=std::env::var_os("CLINIC_ITEM_REVIEW_DIR") else{return;};
    let directory=PathBuf::from(directory);std::fs::create_dir_all(&directory).unwrap();
    // Native drawing receives the core snapshot plus the production UI owner.
    // The public text-controller snapshot intentionally synthesizes its menu.
    let snapshot=controller.shell.shell.snapshot().unwrap();let mut world=World::new();
    let mut queue=bevy::ecs::world::CommandQueue::default();let mut images=Assets::<Image>::default();
    let mut commands=Commands::new(&mut queue,&world);let mut art=RenderedTilesetArt::default();
    if visible_field_pack_is_open(&controller.shell) {
        spawn_field_pack_screen(&mut commands,&snapshot,&controller.shell,&mut art,
            &controller.shell.asset_root,&mut images).unwrap();
    } else {
        spawn_scene_dialog(&mut commands,&snapshot,&controller.shell,&mut art,
            &controller.shell.asset_root,&mut images).unwrap();
    }
    queue.apply(&mut world);assert_eq!(art.font_error,None);
    render_pc_audit_canvas(&mut world,&images,label).save(directory.join(format!("{label}.png"))).unwrap();
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn clinic_public_item_pcs_transfer_cancel_and_release_input_in_every_lobby() {
    let path=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(path.parent().unwrap());
    let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,
        crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap()).unwrap();
    for map in clinic_lobbies(&runtime) {
        let mut controller=clinic_pc_fixture(&root,&runtime,&map,None);
        let potion=runtime.data().items["POTION"].clone();
        let ball=runtime.data().items["POKE_BALL"].clone();
        let bag=&mut controller.shell.shell.session_mut().state_mut().bag;
        bag.items.clear();bag.balls.clear();bag.key_items.clear();bag.tm_hm.fill(0);bag.pc_items.clear();bag.custom_pockets.clear();
        assert!(bag.add_item(&potion,3).unwrap());
        assert!(bag.add_item(&ball,2).unwrap());
        assert!(bag.add_pc_item(&potion,5).unwrap());
        let initial=controller.shell.shell.session().state().bag.clone();
        let storage=controller.shell.shell.session().state().storage.clone();
        let money=controller.shell.shell.session().state().money;
        let tile=controller.snapshot().unwrap().overworld.tile;
        if map=="PewterPokecenter1F"&&let Some(dir)=std::env::var_os("CLINIC_ITEM_REVIEW_DIR") {
            let dir=PathBuf::from(dir);std::fs::create_dir_all(&dir).unwrap();
            controller.save(dir.join("initial-item-pc-fixture.crystalsave")).unwrap();
        }
        clinic_open_player_pc(&mut controller);
        clinic_item_capture(&mut controller,"player-actions");
        clinic_item_key(&mut controller,GameButton::A);
        assert_eq!(controller.shell.pc_item_action,Some(VisiblePlayerPcAction::WithdrawItem));
        assert!(controller.shell.pc_item_cursor.is_some());
        #[cfg(feature="location-tester")]
        assert!(render_walk::retained_pc_review_menu(&controller.shell,"pc:items").unwrap().is_some());
        clinic_item_capture(&mut controller,"withdraw-list");
        clinic_item_key(&mut controller,GameButton::A);
        let quantity=controller.shell.pc_item_quantity.as_ref().expect("source quantity chooser");
        assert_eq!((quantity.item_id.as_str(),quantity.quantity,quantity.maximum),("POTION",1,5));
        clinic_item_key(&mut controller,GameButton::Up);
        assert_eq!(controller.shell.pc_item_quantity.as_ref().unwrap().quantity,2);
        #[cfg(feature="location-tester")]
        {
            let before=controller.shell.shell.state_checksum().unwrap();
            assert!(render_walk::pc_lcd_review_ready(&controller.shell,"pc:quantity"));
            assert!(render_walk::retained_pc_review_menu(&controller.shell,"pc:items").unwrap().is_none());
            assert_eq!(before,controller.shell.shell.state_checksum().unwrap());
        }
        clinic_item_capture(&mut controller,"withdraw-quantity");
        clinic_item_key(&mut controller,GameButton::Start);
        assert!(controller.shell.start_menu_cursor.is_none());
        assert_eq!(controller.snapshot().unwrap().overworld.tile,tile);
        clinic_item_key(&mut controller,GameButton::B);
        assert!(controller.shell.pc_item_quantity.is_none()&&controller.shell.pc_item_cursor.is_some());
        assert_eq!(controller.shell.shell.session().state().bag,initial,"{map}: cancelling quantity cannot transfer items");
        for key in [GameButton::A,GameButton::Up,GameButton::A] {clinic_item_key(&mut controller,key);}
        assert_eq!(controller.shell.shell.session().state().bag.quantity(&potion),5);
        assert_eq!(controller.shell.shell.session().state().bag.pc_item_quantity(&potion),3);
        clinic_item_capture(&mut controller,"withdraw-notice");
        clinic_pc_until(&mut controller,|s|s.pc_notice.is_none()&&s.pc_item_quantity.is_none());
        clinic_item_key(&mut controller,GameButton::B);
        assert_eq!(controller.shell.player_pc_action_cursor.as_ref().unwrap().option_index,0);
        for key in [GameButton::Down,GameButton::A] {clinic_item_key(&mut controller,key);}
        assert!(visible_field_pack_is_open(&controller.shell));
        clinic_item_capture(&mut controller,"deposit-pack");
        clinic_item_key(&mut controller,GameButton::A);
        assert_eq!(controller.shell.pc_item_quantity.as_ref().unwrap().maximum,5);
        clinic_item_key(&mut controller,GameButton::B);
        assert!(visible_field_pack_is_open(&controller.shell)&&controller.shell.pc_item_quantity.is_none());
        for key in [GameButton::A,GameButton::Up,GameButton::A] {clinic_item_key(&mut controller,key);}
        clinic_pc_until(&mut controller,|s|s.pc_notice.is_none()&&s.pc_item_quantity.is_none());
        assert_eq!(controller.shell.shell.session().state().bag,initial,"{map}: deposit restores the exact initial bag and PC contents");
        assert!(visible_field_pack_is_open(&controller.shell));
        assert_eq!(controller.shell.bag_cursor.as_ref().unwrap().option_index,0);
        clinic_item_key(&mut controller,GameButton::B);
        assert_eq!(controller.shell.player_pc_action_cursor.as_ref().unwrap().option_index,1);
        clinic_item_key(&mut controller,GameButton::B);
        assert!(controller.shell.pc_hub_cursor.is_some());
        clinic_item_key(&mut controller,GameButton::B);
        clinic_pc_until(&mut controller,|s|!s.pc_hub_session_open&&s.pc_notice.is_none()&&s.special_boundary.is_none());
        for _ in 0..8 {
            clinic_item_key(&mut controller,GameButton::Down);
            if controller.snapshot().unwrap().overworld.tile==TilePosition::new(tile.x,tile.y+1) {break;}
        }
        assert_eq!(controller.snapshot().unwrap().overworld.tile,TilePosition::new(tile.x,tile.y+1));
        clinic_item_key(&mut controller,GameButton::Start);
        assert!(controller.shell.start_menu_cursor.is_some());
        assert_eq!(controller.shell.shell.session().state().storage,storage);
        assert_eq!(controller.shell.shell.session().state().money,money);
        assert_eq!(controller.shell.last_error,None);
        println!("{map}: actual item PC withdraw/deposit, quantity cancellation, walking and Start passed");
    }
}
