// Opt-in review through the production keyboard schedule and native compositor.
#[test]
#[ignore = "requires CRYSTAL_NORTHSTAR_TITLE_PACK and optional CRYSTAL_NORTHSTAR_REVIEW_DIR"]
fn northstar_title_and_tamarack_opening() {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_NORTHSTAR_TITLE_PACK").expect("revised Minnesota pack"),
    );
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path)
        .expect("verified revised pack");
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).expect("runtime");
    let spawn_identifier = runtime
        .title_new_game_spawn_identifier()
        .expect("regional spawn");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::Title {
            spawn_identifier,
            save_path: None,
        },
        BevyShellConfig::default(),
    )
    .expect("production title");
    let mut app = integrated_shell_test_app(shell);
    let mut images = Assets::<Image>::default();
    let first = {
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(
            shell.splash_screen.is_none(),
            "no copyright or studio splash"
        );
        assert!(shell.intro_screen.is_none(), "no source movie");
        let frame = load_title_screen_frame(
            &shell.asset_root,
            shell.title_menu.as_ref().expect("title"),
            &mut images,
        )
        .expect("native title");
        review_northstar_frame(&images, &frame, "00-title");
        images
            .get(&frame.handle)
            .expect("title pixels")
            .data
            .clone()
    };
    // More than the source attract-mode timeout must retain the same still.
    northstar_wait(&mut app, 4096);
    {
        let shell = app.world().resource::<BevyRuntimeShell>();
        let title = shell.title_menu.as_ref().expect("retained title");
        assert_eq!(title.source_phase(), VisibleTitlePhase::PressStart);
        let frame =
            load_title_screen_frame(&shell.asset_root, title, &mut images).expect("retained frame");
        assert_eq!(
            first,
            images.get(&frame.handle).expect("retained pixels").data
        );
    }
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::Enter);
    northstar_wait(&mut app, 64);
    assert!(visible_title_main_menu_ready(
        app.world()
            .resource::<BevyRuntimeShell>()
            .title_menu
            .as_ref()
            .expect("main menu")
    ));
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyX);
    northstar_wait(&mut app, 32);
    {
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(shell.intro_screen.is_none() && shell.splash_screen.is_none());
        assert_eq!(
            shell
                .title_menu
                .as_ref()
                .expect("returned title")
                .source_phase(),
            VisibleTitlePhase::PressStart
        );
    }
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::Enter);
    northstar_wait(&mut app, 64);
    let mut art = RenderedTilesetArt::default();
    {
        let shell = app.world().resource::<BevyRuntimeShell>();
        let frame = load_visible_title_main_menu_frame(
            shell,
            shell.title_menu.as_ref().expect("menu"),
            &mut art,
            &mut images,
        )
        .expect("menu frame");
        review_northstar_frame(&images, &frame, "01-main-menu");
    }
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    let mut pages = BTreeSet::new();
    let mut sprites = BTreeSet::new();
    let mut capture = 2;
    let mut picked_name = false;
    for _ in 0..128 {
        northstar_wait(&mut app, 96);
        let (name_menu, complete) = {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            assert_eq!(shell.last_error, None);
            // MinimalPlugins has no audio device. Acknowledge the same cry
            // completion fence after the scheduled presentation has waited.
            if shell.pending_oak_intro.as_ref().is_some_and(|intro| {
                intro.scene_phase == VisibleOakIntroPhase::Cry && intro.wooper_cry_queued
            }) {
                shell
                    .pending_audio
                    .retain(|command| matches!(command.kind, ModpackAudioKind::Music));
                shell.active_transient_kind = None;
                shell.transient_audio_playing = false;
            }
            if let Some(clock) = &shell.pending_time_set {
                let text = format_time_set_dialog(clock);
                let key = format!("clock:{:?}:{text}", clock.phase);
                if pages.insert(key) {
                    let frame = load_time_set_frame(&shell.asset_root, clock, &mut images)
                        .expect("clock frame");
                    review_northstar_frame(&images, &frame, &format!("{capture:02}-clock"));
                    capture += 1;
                }
            }
            if let Some(intro) = &shell.pending_oak_intro {
                let text = visible_oak_intro_visible_dialog(intro);
                assert!(!text.contains("OAK"), "no Oak speech: {text}");
                if let Some(sprite) = &intro.current_sprite {
                    sprites.insert(sprite.clone());
                }
                if !text.is_empty() && intro.waiting_for_input && pages.insert(text.clone()) {
                    let frame = load_oak_intro_screen_frame(
                        &shell.asset_root,
                        intro,
                        &shell.shell.snapshot().expect("trainer").trainer,
                        &mut art,
                        &mut images,
                    )
                    .expect("welcome frame");
                    review_northstar_frame(&images, &frame, &format!("{capture:02}-welcome"));
                    capture += 1;
                    println!("WELCOME {}", text.replace('\n', " / "));
                }
            }
            (
                shell.pending_name_choice.as_ref().is_some_and(|choice| {
                    choice.player_phase == Some(VisiblePlayerNameChoicePhase::Menu)
                }),
                shell.title_menu.is_none()
                    && shell.pending_gender_selection.is_none()
                    && shell.pending_time_set.is_none()
                    && shell.pending_oak_intro.is_none()
                    && shell.pending_name_choice.is_none()
                    && shell.pending_name_input.is_none(),
            )
        };
        if complete {
            break;
        }
        if name_menu && !picked_name {
            press_key_for_runtime_hotkey_app(&mut app, KeyCode::ArrowDown);
            picked_name = true;
        }
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    }
    assert!(
        pages.iter().any(|p| p.contains("TAMARACK")),
        "professor introduced"
    );
    assert!(
        pages.iter().any(|p| p.contains("WINONA")),
        "regional departure"
    );
    assert!(sprites.contains("battle-trainer:mn_prof_tamarack"));
    assert!(sprites.contains("JIGLET"));
    assert!(!sprites.contains("OAK") && !sprites.contains("WOOPER"));
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert!(shell.pending_oak_intro.is_none() && shell.pending_name_choice.is_none());
    let snapshot = shell.shell.snapshot().expect("first map");
    assert_eq!(snapshot.overworld.map_name, "MnWinonaHome2F");
    assert_eq!(snapshot.trainer.player_name, "CHRIS");
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::Enter);
    assert!(
        !visible_start_menu_entries(app.world().resource::<BevyRuntimeShell>())
            .expect("Start entries")
            .is_empty()
    );
}

fn northstar_wait(app: &mut App, frames: usize) {
    for _ in 0..frames {
        app.update();
    }
}

fn review_northstar_frame(images: &Assets<Image>, frame: &SpriteFrame, name: &str) {
    let Some(directory) = std::env::var_os("CRYSTAL_NORTHSTAR_REVIEW_DIR") else {
        return;
    };
    let directory = PathBuf::from(directory);
    std::fs::create_dir_all(&directory).expect("review directory");
    let source = images.get(&frame.handle).expect("CPU image");
    let size = source.texture_descriptor.size;
    image::RgbaImage::from_raw(
        size.width,
        size.height,
        source.data.clone().expect("pixels"),
    )
    .expect("native frame")
    .save(directory.join(format!("{name}.png")))
    .expect("review image");
}
