// Only the initial location, party and Pokédex unlock are fixtures. All PC
// service transitions use the production keyboard/MCP controller boundary.
#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn clinic_oak_pc_asks_before_rating_and_closes_both_branches_in_every_lobby() {
    let path=PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root=AssetRoot::new(path.parent().unwrap());
    let runtime=CrystalRuntime::from_loaded_compiled_pack(&root,
        crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap()).unwrap();
    for map in clinic_lobbies(&runtime) {for decision in ["no","cancel","yes"] {
        let mut c=clinic_pc_fixture(&root,&runtime,&map,None);
        c.shell.shell.session_mut().state_mut().flags.set_engine_flag("ENGINE_POKEDEX",true).unwrap();
        c.shell.shell.record_pokedex_seen("CHIKORITA").unwrap();
        c.shell.shell.record_pokedex_caught("TOTODILE").unwrap();
        let before=c.shell.shell.session().state().clone();
        if map=="PewterPokecenter1F" && decision=="no" {
            if let Some(dir)=std::env::var_os("CLINIC_ITEM_REVIEW_DIR") {
                let dir=PathBuf::from(dir);std::fs::create_dir_all(&dir).unwrap();
                c.save(dir.join("initial-oak-pc-fixture.crystalsave")).unwrap();
            }
        }
        for key in [GameButton::Up,GameButton::A] {clinic_item_key(&mut c,key);}
        clinic_pc_until(&mut c,|s|s.pc_hub_cursor.is_some()&&s.special_boundary.is_none());
        for key in [GameButton::Down,GameButton::Down,GameButton::A] {clinic_item_key(&mut c,key);}
        assert_eq!(c.shell.special_boundary.as_ref().unwrap().label,"PokecenterOaksPCText");
        assert!(c.shell.pending_special_sound.is_none(),"no rating before consent");
        clinic_pc_until(&mut c,|s|matches!(s.pc_confirmation,Some(VisiblePcConfirmation::OakRating)));
        let question=c.shell.pc_notice.clone().unwrap();
        assert_eq!(question,"Want to get your\nPOKéDEX rated?");
        c.wait_frames(96).unwrap();
        assert!(visible_pc_toss_confirmation_ready(&c.shell));
        let at=c.snapshot().unwrap().overworld.tile;
        clinic_item_key(&mut c,GameButton::Start);
        assert!(matches!(c.shell.pc_confirmation,Some(VisiblePcConfirmation::OakRating)));
        assert_eq!(c.snapshot().unwrap().overworld.tile,at);
        clinic_item_key(&mut c,GameButton::Up);
        assert_eq!(c.shell.yes_no_cursor.as_ref().unwrap().option_index,0);
        if decision=="no" {clinic_item_key(&mut c,GameButton::Down);}
        for key in [GameButton::Left,GameButton::Right] {clinic_item_key(&mut c,key);}
        assert_eq!(c.shell.yes_no_cursor.as_ref().unwrap().option_index,usize::from(decision=="no"));
        if decision=="yes" {clinic_item_capture(&mut c,"oak-question-yes");}
        if decision=="no" {clinic_item_capture(&mut c,"oak-question-no");}
        clinic_item_key(&mut c,if decision=="cancel" {GameButton::B}else{GameButton::A});
        assert!(c.shell.pc_confirmation.is_none());
        if decision=="yes" {
            assert_eq!(c.shell.special_boundary.as_ref().unwrap().label,"ProfOaksPcBoot");
            assert!(c.shell.pending_special_sound.is_some(),"accepted rating queues source fanfare");
            let pages=c.shell.special_boundary.iter().chain(c.shell.special_boundary_queue.iter())
                .flat_map(|b|b.details.iter()).cloned().collect::<Vec<_>>().join("\n");
            assert!(pages.contains(&format!("{} POKéMON seen",before.pokedex.seen_species.len()))
                &&pages.contains(&format!("{} POKéMON owned",before.pokedex.caught_species.len())),
                "source rating includes actual counts: {pages}");
            clinic_pc_until(&mut c,|s|s.special_boundary.as_ref().is_some_and(|b|b.label=="OakPCShutdown"));
        } else {
            assert!(c.shell.pending_special_sound.is_none(),"declining must not run Rate or its fanfare");
        }
        assert_eq!(c.shell.special_boundary.as_ref().unwrap().label,"OakPCShutdown");
        assert_eq!(c.shell.special_boundary.as_ref().unwrap().details,["The link to PROF.\nOAK's PC closed."]);
        c.wait_frames(96).unwrap();
        if decision=="no" {clinic_item_capture(&mut c,"oak-shutdown");}
        clinic_pc_until(&mut c,|s|s.pc_hub_cursor.is_some()&&s.special_boundary.is_none());
        let after=c.shell.shell.session().state();
        assert_eq!(after.storage,before.storage,"{map}/{decision}: rating cannot change party or boxes");
        assert_eq!(after.bag,before.bag);
        assert_eq!(after.money,before.money);
        assert_eq!(after.pokedex,before.pokedex,"rating and declining both preserve recorded Pokédex entries");
        let off=visible_pc_hub_actions(&c.shell.shell.snapshot().unwrap()).len()-1;
        for _ in 0..off {clinic_item_key(&mut c,GameButton::Down);}
        clinic_item_key(&mut c,GameButton::A);
        clinic_pc_until(&mut c,|s|!s.pc_hub_session_open&&s.special_boundary.is_none());
        for _ in 0..8 {
            clinic_item_key(&mut c,GameButton::Down);
            if c.snapshot().unwrap().overworld.tile!=at {break;}
        }
        assert_eq!(c.snapshot().unwrap().overworld.tile,TilePosition::new(at.x,at.y+1),"{map}/{decision}: PC releases walking");
        clinic_item_key(&mut c,GameButton::Start);
        assert!(c.snapshot().unwrap().ui.menu.is_some(),"{map}/{decision}: production Start returns");
        assert!(c.shell.last_error.is_none(),"{map}/{decision}: {:?}",c.shell.last_error);
    }}
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn oak_pc_question_waits_for_print_and_live_yes_no_repaints() {
    // Isolate the retained prompt/printer stage. The other regression reaches
    // this stage through actual PCScript in every lobby, with no service seeding.
    let mut shell=initialized_mail_reader_shell("FLOWER_MAIL");
    open_visible_oak_rating_question(&mut shell).unwrap();
    let question=shell.pc_notice.clone().unwrap();
    let buttons:&[(&str,fn(&mut BevyRuntimeShell)->Result<()>)]=&[
        ("A",press_visible_a_button),("B",press_visible_b_button),
        ("Up",move_visible_primary_cursor_up),("Down",move_visible_primary_cursor_down),
        ("Left",move_visible_primary_cursor_left),("Right",move_visible_primary_cursor_right)];
    for &(name,press) in buttons {
        shell.field_text_reveal=Some(VisibleFieldTextReveal{text:question.clone(),page_index:0,visible_chars:1,frames_until_next_char:1});
        press(&mut shell).unwrap();
        assert!(matches!(shell.pc_confirmation,Some(VisiblePcConfirmation::OakRating)),"{name}: question still prints");
        assert_eq!(shell.yes_no_cursor.as_ref().unwrap().option_index,0);
        assert!(shell.special_boundary.is_none()&&shell.pending_special_sound.is_none());
    }
    fn glyphs(app:&mut App)->Vec<(u64,String)> {
        let world=app.world_mut();let mut query=world.query::<(&DialogGlyphMarker,&Sprite)>();
        let mut glyphs=query.iter(world).map(|(m,s)|(m.key,format!("{:?}",s.image.id()))).collect::<Vec<_>>();
        glyphs.sort();glyphs
    }
    let mut app=menu_render_test_app(shell);app.update();let initial=glyphs(&mut app);
    for _ in 0..96 {app.update();}
    assert!(visible_pc_toss_confirmation_ready(app.world().resource::<BevyRuntimeShell>()));
    let yes=glyphs(&mut app);assert_ne!(initial,yes,"printed question and source YesNoBox must repaint");
    {
        let world=app.world_mut();
        let mut q=world.query_filtered::<Entity,Or<(With<BattleCommandMarker>,With<BattleWindowFrameMarker>)>>();
        assert_eq!(q.iter(world).count(),0,"Oak question must not create markers that suppress the native room");
    }
    press_key_for_runtime_hotkey_app(&mut app,KeyCode::ArrowDown);
    assert_eq!(app.world().resource::<BevyRuntimeShell>().yes_no_cursor.as_ref().unwrap().option_index,1);
    assert_ne!(yes,glyphs(&mut app),"NO arrow must actually repaint");
    // VerticalMenu waits for its background upload before polling again.
    for _ in 0..4 {app.update();}
    let mut before_close=glyphs(&mut app).into_iter().map(|(_,image)|image).collect::<Vec<_>>();
    before_close.sort();
    press_key_for_runtime_hotkey_app(&mut app,KeyCode::KeyX);
    assert_eq!(app.world().resource::<BevyRuntimeShell>().mailbox_confirmation_response,Some(false));
    let retained=glyphs(&mut app);
    let mut closing=retained.iter().map(|(_,image)|image.clone()).collect::<Vec<_>>();closing.sort();
    assert_eq!(closing,before_close,"question, cursor and YES/NO remain drawn during the source wait");
    assert_eq!(app.world().resource::<BevyRuntimeShell>().yes_no_cursor.as_ref().unwrap().option_index,1);
    for _ in 0..16 {app.update();}
    assert_ne!(retained,glyphs(&mut app),"the source closing wait completes before repainting shutdown");
    assert_eq!(app.world().resource::<BevyRuntimeShell>().special_boundary.as_ref().unwrap().label,"OakPCShutdown");
}
