// Layout fixtures isolate the four cursor owners. Actual service continuation
// is covered by the live Save, Mom, phone, and PC controller regressions.
#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn shared_yes_no_draws_source_borders_and_inset_two_row_labels_for_every_owner() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    for owner in ["phone", "day", "pc", "save"] {
        for selected in 0..2 {
            shell.pending_phone_prompt = None;
            shell.pending_day_of_week = None;
            shell.pc_confirmation = None;
            shell.save_flow = None;
            shell.yes_no_cursor = None;
            match owner {
                "phone" => {
                    shell.pending_phone_prompt = Some(PendingPhonePrompt {
                        source_script: "LayoutFixture".into(),
                        command_index: 0,
                        contact_id: "PHONE_ELM".into(),
                    });
                    shell.yes_no_cursor = Some(MenuCursor {
                        surface_id: "ui:phone-number".into(),
                        option_index: selected,
                    });
                }
                "day" => {
                    shell.pending_day_of_week = Some(PendingDayOfWeekPrompt {
                        origin_map_name: "PlayersHouse1F".into(),
                        source_script: "LayoutFixture".into(),
                        command_index: 0,
                        selected_day: 0,
                        confirming: true,
                        yes_no_index: selected,
                    })
                }
                "pc" => {
                    shell.pc_confirmation = Some(VisiblePcConfirmation::LinkTrade);
                    shell.yes_no_cursor = Some(MenuCursor {
                        surface_id: "pc:confirmation".into(),
                        option_index: selected,
                    });
                }
                _ => {
                    shell.save_flow = Some(VisibleSaveFlow {
                        stage: VisibleSaveFlowStage::Prompt,
                        origin: VisibleSaveFlowOrigin::StartMenu,
                        save_exists: false,
                        yes_no_index: selected,
                    })
                }
            }
            let snapshot = shell.shell.snapshot().unwrap();
            let before = snapshot.state_checksum.clone();
            assert!(scene_dialog_yes_no_active(&snapshot, &shell));
            assert_eq!(
                scene_dialog_yes_no_cursor_index(&snapshot, &shell).unwrap(),
                selected
            );
            let mut world = World::new();
            let mut queue = bevy::ecs::world::CommandQueue::default();
            let mut images = Assets::<Image>::default();
            let mut art = RenderedTilesetArt::default();
            spawn_visible_yes_no_prompt_box(
                &snapshot,
                &mut Commands::new(&mut queue, &world),
                &shell,
                &mut art,
                &shell.asset_root,
                &mut images,
            )
            .unwrap();
            queue.apply(&mut world);
            assert_eq!(art.font_error, None);
            let mut q = world.query_filtered::<&Transform, With<SceneDialogWindowFrameMarker>>();
            let actual = q
                .iter(&world)
                .map(|t| (t.translation.x.to_bits(), t.translation.y.to_bits()))
                .collect::<BTreeSet<_>>();
            // Independent literal ASM border coordinates, not the renderer's size constants.
            let expected = (7..=11)
                .flat_map(|y| {
                    (14..=19).filter_map(move |x| {
                        (x == 14 || x == 19 || y == 7 || y == 11).then_some((x, y))
                    })
                })
                .map(|(x, y)| {
                    let (x, y) = battle_hud_tile_origin(x as f32, y as f32);
                    (x.to_bits(), y.to_bits())
                })
                .collect::<BTreeSet<_>>();
            assert_eq!(
                actual, expected,
                "{owner}/{selected}: all18 source perimeter tiles"
            );
            let mut q = world.query_filtered::<&Transform, With<DialogGlyphMarker>>();
            let actual = q
                .iter(&world)
                .map(|t| (t.translation.x.to_bits(), t.translation.y.to_bits()))
                .collect::<BTreeSet<_>>();
            let expected = [
                (15, 8),
                (16, 8),
                (17, 8),
                (18, 8),
                (15, 10),
                (16, 10),
                (17, 10),
            ]
            .into_iter()
            .map(|(x, y)| {
                let (x, y) = battle_hud_tile_origin(x as f32, y as f32);
                (x.to_bits(), y.to_bits())
            })
            .collect::<BTreeSet<_>>();
            assert_eq!(
                actual, expected,
                "{owner}/{selected}: cursor column15, labels16, rows8/10"
            );
            let canvas = render_pc_audit_canvas(&mut world, &images, "shared YesNo source layout");
            if owner == "phone" {
                if let Some(dir) = std::env::var_os("SHARED_YES_NO_REVIEW_DIR") {
                    let dir = PathBuf::from(dir);
                    std::fs::create_dir_all(&dir).unwrap();
                    canvas
                        .save(dir.join(format!(
                            "source-box-{}.png",
                            if selected == 0 { "yes" } else { "no" }
                        )))
                        .unwrap();
                }
            }
            assert_eq!(
                shell.shell.snapshot().unwrap().state_checksum,
                before,
                "drawing cannot change gameplay"
            );
        }
    }
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn shared_yes_no_retained_cursor_matches_fresh_source_layout_without_rebuilding_frames() {
    let mut shell = initialized_mail_reader_shell("FLOWER_MAIL");
    // This isolated retained-dialog fixture owns an ordinary field text page.
    shell.field_notice = Some("May I register\nyour number?".into());
    shell.pending_phone_prompt = Some(PendingPhonePrompt {
        source_script: "LayoutFixture".into(),
        command_index: 0,
        contact_id: "PHONE_ELM".into(),
    });
    shell.yes_no_cursor = Some(MenuCursor {
        surface_id: "ui:phone-number".into(),
        option_index: 0,
    });
    let mut app = menu_render_test_app(shell);
    app.update();
    for _ in 0..96 {
        app.update();
    }
    fn frames(world: &mut World) -> BTreeSet<Entity> {
        world
            .query_filtered::<Entity, With<SceneDialogWindowFrameMarker>>()
            .iter(world)
            .collect()
    }
    let original = frames(app.world_mut());
    assert_eq!(original.len(), 66, "48 field +18 YesNo border tiles");
    {
        let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
        shell.yes_no_cursor.as_mut().unwrap().option_index = 1;
        mark_runtime_presentation_dirty(&mut shell);
    }
    app.update();
    assert_eq!(
        frames(app.world_mut()),
        original,
        "cursor repaint must retain every frame tile"
    );
    let retained = {
        let world = app.world_mut();
        let sprites = world
            .query_filtered::<(&Sprite, &Transform), With<SceneDialogMarker>>()
            .iter(world)
            .map(|(s, t)| (s.clone(), *t))
            .collect::<Vec<_>>();
        let mut lcd = World::new();
        for sprite in sprites {
            lcd.spawn(sprite);
        }
        render_pc_audit_canvas(
            &mut lcd,
            world.resource::<Assets<Image>>(),
            "retained source YesNo",
        )
    };
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(shell.last_error, None);
    let snapshot = shell.shell.snapshot().unwrap();
    let mut fresh = World::new();
    let mut queue = bevy::ecs::world::CommandQueue::default();
    let mut art = RenderedTilesetArt::default();
    let mut images = Assets::<Image>::default();
    spawn_scene_dialog(
        &mut Commands::new(&mut queue, &fresh),
        &snapshot,
        shell,
        &mut art,
        &shell.asset_root,
        &mut images,
    )
    .unwrap();
    queue.apply(&mut fresh);
    let fresh = render_pc_audit_canvas(&mut fresh, &images, "fresh source YesNo");
    assert_eq!(
        retained.as_raw(),
        fresh.as_raw(),
        "retained NO must match the full source-layout draw exactly"
    );
    if let Some(dir) = std::env::var_os("SHARED_YES_NO_REVIEW_DIR") {
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        retained.save(dir.join("retained-no.png")).unwrap();
        fresh.save(dir.join("fresh-no.png")).unwrap();
    }
}

#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn shared_yes_no_start_save_cancels_without_writing_and_releases_every_clinic() {
    let path = PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root = AssetRoot::new(path.parent().unwrap());
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap(),
    )
    .unwrap();
    for map in clinic_lobbies(&runtime) {
        for cancel in [false, true] {
            let scratch = std::env::temp_dir().join(format!(
                "geothite-source-save-cancel-{}-{map}-{cancel}.crystalsave",
                std::process::id()
            ));
            assert!(!scratch.exists());
            let mut c = clinic_pc_fixture(&root, &runtime, &map, Some(scratch.clone()));
            let before = c.shell.shell.session().state().clone();
            clinic_item_key(&mut c, GameButton::Start);
            let at = c.snapshot().unwrap().overworld.tile;
            let options = visible_start_menu_options(&c.shell, &c.shell.shell.snapshot().unwrap());
            let save = options
                .iter()
                .position(|o| *o == StartMenuOption::Save)
                .unwrap();
            for _ in 0..save {
                clinic_item_key(&mut c, GameButton::Down);
            }
            clinic_item_key(&mut c, GameButton::A);
            assert!(
                c.shell.save_menu_open
                    && c.shell
                        .save_flow
                        .as_ref()
                        .is_some_and(|f| f.stage == VisibleSaveFlowStage::Prompt)
            );
            if !cancel {
                clinic_item_key(&mut c, GameButton::Down);
                assert_eq!(c.shell.save_flow.as_ref().unwrap().yes_no_index, 1);
            }
            clinic_item_key(&mut c, if cancel { GameButton::B } else { GameButton::A });
            assert!(!c.shell.save_menu_open && c.shell.save_flow.is_none());
            assert!(
                !scratch.exists(),
                "{map}/{cancel}: declining must not write a save"
            );
            assert_eq!(c.shell.shell.session().state().storage, before.storage);
            assert_eq!(c.shell.shell.session().state().bag, before.bag);
            assert_eq!(c.shell.shell.session().state().money, before.money);
            if c.shell.start_menu_cursor.is_some() {
                clinic_item_key(&mut c, GameButton::B);
            }
            for _ in 0..8 {
                clinic_item_key(&mut c, GameButton::Down);
                if c.snapshot().unwrap().overworld.tile != at {
                    break;
                }
            }
            assert_eq!(
                c.snapshot().unwrap().overworld.tile,
                TilePosition::new(at.x, at.y + 1)
            );
            assert_eq!(c.shell.last_error, None);
        }
    }
}
