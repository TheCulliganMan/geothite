// Initial location/party are fixtures; service entry, choices, editing and
// continuation use the same production controller as native keyboard/MCP.
#[track_caller]
fn name_rater_until(c: &mut VisibleShellController, ready: impl Fn(&BevyRuntimeShell) -> bool) {
    for _ in 0..128 {
        if ready(&c.shell) { return; }
        assert!(c.shell.pending_name_input.is_none(), "unexpected naming surface before requested phase");
        clinic_item_key(c, GameButton::A);
    }
    panic!("Name Rater did not reach its next surface: {:?}", c.snapshot().unwrap().ui);
}

fn name_rater_type(c: &mut VisibleShellController, text: &str) {
    let letters = ["ABCDEFGHI", "JKLMNOPQR", "STUVWXYZ"];
    for ch in text.chars() {
        let (row, col) = letters.iter().enumerate()
            .find_map(|(row, line)| line.find(ch).map(|col| (row, col))).unwrap();
        for _ in 0..8 {
            if c.shell.pending_name_input.as_ref().unwrap().cursor_row == row { break; }
            let down = c.shell.pending_name_input.as_ref().unwrap().cursor_row < row;
            clinic_item_key(c, if down { GameButton::Down } else { GameButton::Up });
        }
        assert_eq!(c.shell.pending_name_input.as_ref().unwrap().cursor_row, row, "naming row input must move");
        for _ in 0..12 {
            if c.shell.pending_name_input.as_ref().unwrap().cursor_column == col { break; }
            let right = c.shell.pending_name_input.as_ref().unwrap().cursor_column < col;
            clinic_item_key(c, if right { GameButton::Right } else { GameButton::Left });
        }
        assert_eq!(c.shell.pending_name_input.as_ref().unwrap().cursor_column, col, "naming column input must move");
        clinic_item_key(c, GameButton::A);
    }
}

#[test]
fn name_rater_presents_the_selected_species_heading_without_changing_the_input_owner() {
    let shell = initialized_mail_reader_shell("FLOWER_MAIL");
    let input = PendingNameInput { label:"POKéMON'S NAME?".into(), value:String::new(), max_length:10,
        cursor_column:0, cursor_row:0, case:NameInputCase::Upper };
    let before = shell.shell.state_checksum().unwrap();
    let display = visible_name_entry_presentation_input(&shell, &input).unwrap();
    assert_eq!(display.label,"CYNDAQUIL'S\nNICKNAME?");
    assert_eq!(input.label,"POKéMON'S NAME?", "presentation must not replace the service dispatch owner");
    assert_eq!((&display.value,display.max_length),(&input.value,input.max_length));
    assert_eq!(shell.shell.state_checksum().unwrap(),before);
}

#[test]
fn name_rater_heading_keeps_both_source_borders_and_all_label_glyphs() {
    for (label, x) in [("POKéMON'S NAME?",4), ("YOUR NAME?",5), ("ABCDEFGHIJKLMNOPQR",1)] {
        let input = PendingNameInput { label:label.into(), value:String::new(), max_length:10,
            cursor_column:0, cursor_row:0, case:NameInputCase::Upper };
        let map = build_name_entry_tilemap(&input).unwrap();
        assert_eq!(map[2][0],NAME_ENTRY_BORDER_TILE);
        assert_eq!(map[2][19],NAME_ENTRY_BORDER_TILE);
        let tokens = tokenize_name_entry_string(label);
        for (i,token) in tokens.iter().enumerate() {
            assert_eq!(map[2][x+i],name_entry_token_tile(token).unwrap());
        }
    }
    let too_long = PendingNameInput { label:"ABCDEFGHIJKLMNOPQRS".into(), value:String::new(), max_length:10,
        cursor_column:0, cursor_row:0, case:NameInputCase::Upper };
    assert!(build_name_entry_tilemap(&too_long).is_err());
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn name_rater_real_services_start_empty_preserve_refusals_and_return_walking() {
    let path = PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root = AssetRoot::new(path.parent().unwrap());
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root,
        crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap()).unwrap();
    for map in ["LavenderNameRater", "GoldenrodNameRater"] {
        for case in ["rename", "blank", "same", "limit", "edit", "egg", "foreign-id", "foreign-name",
            "intro-no", "intro-b", "party-b", "better-no", "better-b"] {
            let npc = runtime.data().maps[map].objects.iter().find(|o| o.script == map).unwrap();
            let npc = crate::core::world::session::object_tile_position_checked(npc).unwrap();
            let at = TilePosition::new(npc.x, npc.y + 1);
            let shell = initialize_bevy_runtime_shell(root.clone(), runtime.clone(),
                BevyShellStart::NewGameAtRuntimeTile {
                    spawn_identifier: runtime.title_new_game_spawn_identifier().unwrap(),
                    map_name: map.into(), tile_x: at.x, tile_y: at.y,
                }, BevyShellConfig { smoke_player_name: Some("CHRIS".into()), ..default() }).unwrap();
            let mut c = VisibleShellController { shell };
            complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS")).unwrap();
            let id = c.shell.shell.session().state().player_id;
            c.shell.shell.add_party_pokemon("CYNDAQUIL", 8, None, None, "CHRIS", id,
                Dv::from_non_hp(10,10,10,10)).unwrap();
            {
                let state = c.shell.shell.session_mut().state_mut();
                let mon = state.storage.party.pokemon[0].as_mut().unwrap();
                mon.nickname = "EMBER".into();
                if case == "egg" { mon.is_egg = true; }
                if case == "foreign-id" { mon.original_trainer_id = id.wrapping_add(1); }
                if case == "foreign-name" { mon.original_trainer_name = "GOLD".into(); }
                state.sync_party_from_storage();
            }
            mark_runtime_snapshot_dirty(&mut c.shell);
            let before = c.shell.shell.session().state().clone();
            if case == "rename" && let Some(dir) = std::env::var_os("NAME_RATER_REVIEW_DIR") {
                let dir = PathBuf::from(dir); std::fs::create_dir_all(&dir).unwrap();
                c.save(dir.join(format!("initial-{map}.crystalsave"))).unwrap();
            }
            clinic_item_key(&mut c, GameButton::Up);
            assert_eq!(c.shell.shell.current_overworld_interaction_checked().unwrap()
                .map(|i| i.script), Some(map.into()), "{map}/{case}: actual source NPC");
            clinic_item_key(&mut c, GameButton::A);
            name_rater_until(&mut c, |s| s.pc_confirmation.is_some() && s.special_boundary.is_none());
            c.wait_frames(96).unwrap();
            assert!(!c.shell.party_menu_open, "intro must precede party selection");
            if case.starts_with("intro-") {
                if case == "intro-no" { clinic_item_key(&mut c, GameButton::Down); }
                clinic_item_key(&mut c, if case == "intro-b" { GameButton::B } else { GameButton::A });
            } else {
                clinic_item_key(&mut c, GameButton::A);
                name_rater_until(&mut c, |s| s.party_menu_open && s.special_boundary.is_none());
                clinic_item_key(&mut c, if case == "party-b" { GameButton::B } else { GameButton::A });
                if case == "egg" || case.starts_with("foreign-") {
                    let expected = if case == "egg" { "NameRaterEggText" } else { "NameRaterPerfectNameText" };
                    assert_eq!(c.shell.special_boundary.as_ref().map(|b| b.label.as_str()), Some(expected));
                    assert!(c.shell.pending_name_input.is_none());
                } else if case != "party-b" {
                    name_rater_until(&mut c, |s| matches!(s.pc_confirmation,
                        Some(VisiblePcConfirmation::NameRaterRename)) && s.special_boundary.is_none());
                    c.wait_frames(96).unwrap();
                    if case == "better-no" { clinic_item_key(&mut c, GameButton::Down); }
                    clinic_item_key(&mut c, if case == "better-b" { GameButton::B } else { GameButton::A });
                    if !case.starts_with("better-") {
                        name_rater_until(&mut c, |s| s.pending_name_input.is_some());
                        let entry = c.shell.pending_name_input.as_ref().unwrap();
                        assert_eq!((&*entry.label, entry.max_length), ("POKéMON'S NAME?", 10));
                        assert_eq!(entry.value, "", "{map}/{case}: original NamingScreen_InitNameEntry starts empty");
                        #[cfg(feature="location-tester")]
                        assert!(render_walk::pc_lcd_review_ready(&c.shell,"service:name-rater-entry"),
                            "read-only capture observes an actual service entry");
                        assert_eq!(c.snapshot().unwrap().overworld.tile, at);
                        let mut text = match case { "rename" => "BLAZE", "same" => "EMBER",
                            "limit" => "AAAAAAAAAA", "edit" => "A", _ => "" };
                        name_rater_type(&mut c, text);
                        if case == "edit" {
                            clinic_item_key(&mut c, GameButton::A);
                            assert_eq!(c.shell.pending_name_input.as_ref().unwrap().value, "AA");
                            clinic_item_key(&mut c, GameButton::B);
                            assert_eq!(c.shell.pending_name_input.as_ref().unwrap().value, "A");
                            clinic_item_key(&mut c, GameButton::Select);
                            assert!(matches!(c.shell.pending_name_input.as_ref().unwrap().case, NameInputCase::Lower));
                            clinic_item_key(&mut c, GameButton::A);
                            text = "Aa";
                        }
                        assert_eq!(c.shell.pending_name_input.as_ref().unwrap().value, text);
                        if case == "limit" {
                            let entry = c.shell.pending_name_input.as_ref().unwrap();
                            assert_eq!((entry.cursor_row,entry.cursor_column), (4,8), "full entry selects END");
                        } else {
                            clinic_item_key(&mut c, GameButton::Start);
                            assert!(c.shell.pending_name_input.is_some(), "Start selects END before confirmation");
                        }
                        clinic_item_key(&mut c, GameButton::A);
                        assert!(c.shell.pending_name_input.is_none());
                        let label = if matches!(case, "blank" | "same") { "NameRaterSameNameText" }
                            else { "NameRaterFinishedText" };
                        assert!(c.shell.special_boundary.iter().chain(c.shell.special_boundary_queue.iter())
                            .any(|b| b.label == label), "{map}/{case}: correct source completion branch");
                    }
                }
            }
            name_rater_until(&mut c, |s| s.active_script_cursor.is_none() && s.special_boundary.is_none()
                && s.pc_confirmation.is_none() && !s.party_menu_open && s.pending_name_input.is_none());
            let mut expected = before.storage.clone();
            if case == "rename" { expected.party.pokemon[0].as_mut().unwrap().nickname = "BLAZE".into(); }
            if case == "limit" { expected.party.pokemon[0].as_mut().unwrap().nickname = "AAAAAAAAAA".into(); }
            if case == "edit" { expected.party.pokemon[0].as_mut().unwrap().nickname = "Aa".into(); }
            let after = c.shell.shell.session().state();
            assert_eq!(after.storage, expected, "{map}/{case}: only accepted nickname changes");
            assert_eq!(after.bag, before.bag); assert_eq!(after.money, before.money);
            assert_eq!(after.pokedex, before.pokedex);
            assert!(c.shell.last_error.is_none(), "{map}/{case}: {:?}", c.shell.last_error);
            for _ in 0..8 {
                clinic_item_key(&mut c, GameButton::Down);
                if c.snapshot().unwrap().overworld.tile != at { break; }
            }
            assert_eq!(c.snapshot().unwrap().overworld.tile, TilePosition::new(at.x,at.y+1),
                "{map}/{case}: script must release walking");
            clinic_item_key(&mut c, GameButton::Start);
            assert!(c.snapshot().unwrap().ui.menu.is_some(), "{map}/{case}: production Start returns");
            if case == "rename" {
                clinic_item_key(&mut c, GameButton::B);
                for _ in 0..8 {
                    clinic_item_key(&mut c, GameButton::Up);
                    if c.snapshot().unwrap().overworld.tile == at { break; }
                }
                assert_eq!(c.snapshot().unwrap().overworld.tile, at);
                assert_eq!(c.shell.shell.current_overworld_interaction_checked().unwrap()
                    .map(|i| i.script), Some(map.into()), "revisit approaches the actual NPC again");
                clinic_item_key(&mut c, GameButton::A);
                name_rater_until(&mut c, |s| s.pc_confirmation.is_some() && s.special_boundary.is_none());
                c.wait_frames(96).unwrap();
                assert!(!c.shell.party_menu_open, "stale party-slot variables cannot skip the next visit's introduction");
                clinic_item_key(&mut c, GameButton::B);
                name_rater_until(&mut c, |s| s.active_script_cursor.is_none() && s.special_boundary.is_none()
                    && s.pc_confirmation.is_none() && !s.party_menu_open);
                assert_eq!(c.shell.shell.session().state().storage.party.pokemon[0].as_ref().unwrap().nickname, "BLAZE");
            }
            if std::env::var_os("NAME_RATER_TRACE").is_some() { eprintln!("Name Rater verified {map}/{case}"); }
        }
    }
}

#[test]
#[ignore = "requires external naming ROM references and the verified pack"]
fn name_rater_nickname_pixels_match_original_rom_gender_icons_and_editing() {
    let mut differences = Vec::new();
    for (stem, species, attack, gender) in [("cyndaquil-male", "CYNDAQUIL", 15, Some('♂')),
        ("cyndaquil-female", "CYNDAQUIL", 0, Some('♀')), ("magnemite", "MAGNEMITE", 15, None)] {
        let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
        shell.shell.add_party_pokemon(species, 8, None, None, "TEST", 0,
            Dv::from_non_hp(attack,15,15,15)).unwrap();
        shell.party_cursor = 1;
        shell.pending_name_input = Some(PendingNameInput { label:"POKéMON'S NAME?".into(), value:String::new(),
            max_length:10, cursor_column:0, cursor_row:0, case:NameInputCase::Upper });
        mark_runtime_snapshot_dirty(&mut shell);
        let mut c = VisibleShellController { shell };
        let source_trace: serde_json::Value = serde_json::from_str(&external_oracle_fixture_text(&format!("{stem}/trace.json"))).unwrap();
        let mut idle_variants = std::collections::HashSet::new();
        for (index, button) in [None, Some(GameButton::A), Some(GameButton::Right), Some(GameButton::A),
            Some(GameButton::B), Some(GameButton::Select), Some(GameButton::A), Some(GameButton::Start)].into_iter().enumerate() {
            if let Some(button) = button { clinic_item_key(&mut c, button); }
            let input = c.shell.pending_name_input.as_ref().unwrap();
            assert_eq!(input.value.len() as u64, source_trace["editing"][index]["length"].as_u64().unwrap());
            let display = visible_name_entry_presentation_input(&c.shell, input).unwrap();
            let mut mon = visible_name_entry_pokemon_art(&c.shell, input).unwrap().unwrap();
            assert_eq!(mon.gender, gender);
            let mut images = Assets::<Image>::default();
            let frames: Vec<_> = (0..4).map(|phase| {
                mon.frame = phase / 2;
                let frame = load_name_entry_frame_with_pokemon(&c.shell.asset_root, &display, Some(&mon), phase % 2 == 0, &mut images).unwrap();
                let image = images.get(&frame.handle).unwrap();
                image::RgbaImage::from_raw(160,144,image.data.clone().unwrap()).unwrap()
            }).collect();
            let references = if index == 0 { (0..32).map(|i| format!("{stem}/idle-{i:02}.png")).chain([format!("{stem}/edit-0.png")]).collect::<Vec<_>>() }
                else { vec![format!("{stem}/edit-{index}.png")] };
            for path in references {
                let reference = image::load_from_memory(&external_oracle_fixture_bytes(&path)).unwrap().to_rgba8();
                let counts: Vec<_> = frames.iter().map(|frame| frame.pixels().zip(reference.pixels()).filter(|(actual,expected)|
                    actual[3] != 255 || (0..3).any(|channel| actual[channel] >> 3 != expected[channel] >> 3)).count()).collect();
                let (phase, count) = counts.iter().enumerate().min_by_key(|(_, count)| *count).unwrap();
                differences.push((path.clone(), *count));
                if index == 0 { idle_variants.insert(phase / 2); }
                if let Ok(directory) = std::env::var("NAME_RATER_PIXEL_REVIEW_DIR") {
                    let output = PathBuf::from(directory).join(path);
                    std::fs::create_dir_all(output.parent().unwrap()).unwrap();
                    frames[phase].save(output).unwrap();
                }
            }
        }
        assert_eq!(idle_variants.len(),2,"original naming icon must exhibit both source frames");
    }
    assert!(differences.iter().all(|(_,count)| *count==0), "nickname RGB5 differences: {differences:?}");
}

#[test]
fn name_rater_idle_animation_reuses_the_lcd_without_gameplay_input() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    shell.pending_name_input = Some(PendingNameInput { label:"POKéMON'S NAME?".into(), value:String::new(),
        max_length:10, cursor_column:0, cursor_row:0, case:NameInputCase::Upper });
    let mut app = integrated_shell_test_app(shell);
    app.update();
    let before = app.world().resource::<BevyRuntimeShell>().shell.session().state().clone();
    let location = app.world().resource::<BevyRuntimeShell>().shell.snapshot().unwrap().overworld;
    let first_clock = app.world().resource::<BevyRuntimeShell>().lcd_animation_frame;
    let retained = retained_fullscreen_surface(app.world_mut());
    let mut pixels = std::collections::HashSet::new();
    for _ in 0..74 {
        // Ordinary production input-clock updates, with no physical keys.
        // GameTimer is allowed to advance; inventory, party and location are not.
        app.update();
        assert_retained_fullscreen_surface(app.world_mut(), &retained);
        pixels.insert(app.world().resource::<Assets<Image>>().get(&retained.texture).unwrap().data.clone().unwrap());
        let shell = app.world().resource::<BevyRuntimeShell>();
        let input = shell.pending_name_input.as_ref().unwrap();
        assert!(input.value.is_empty());
        let state = shell.shell.session().state();
        assert_eq!(state.storage,before.storage);
        assert_eq!(state.bag,before.bag); assert_eq!(state.money,before.money);
        assert_eq!(state.pokedex,before.pokedex);
        let now = shell.shell.snapshot().unwrap().overworld;
        assert_eq!((&now.map_name,now.tile),(&location.map_name,location.tile));
        assert!(shell.last_error.is_none());
    }
    assert!(app.world().resource::<BevyRuntimeShell>().lcd_animation_frame >= first_clock + 72);
    assert_eq!(pixels.len(),4,"both icon and cursor phases must reach the actual retained LCD while idle");
    let art = app.world().resource::<RenderedTilesetArt>();
    assert_eq!(art.name_entry_cache.len(),4,"two icon frames and two cursor phases, bounded across repeated cycles");
    assert!(art.name_entry_errors.is_empty());
}
