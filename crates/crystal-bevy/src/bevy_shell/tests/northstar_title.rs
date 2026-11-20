#[test]
fn northstar_client_clock_selection_preserves_other_pack_policy() {
    assert!(!requires_native_server_clock(true, true));
    assert!(!requires_native_server_clock(false, true));
    assert!(requires_native_server_clock(true, false));
    assert!(!requires_native_server_clock(false, false));
}

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
    assert!(
        runtime.data().uses_host_clock(),
        "Northstar uses the host clock"
    );
    assert_eq!(
        select_native_rtc_source(
            requires_native_server_clock(
                runtime.data().server_clock,
                startup_policy::is_minnesota(&runtime),
            ),
            NativeRtcSource::system_local(),
        )
        .expect("Northstar selects the local clock"),
        NativeRtcSource::SystemLocal,
    );
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
    enter_northstar_menu_after_sequence(&mut app, &mut images);
    {
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(shell.splash_screen.is_none(), "source splash completed");
        assert!(shell.intro_screen.is_none(), "source movie completed");
        assert!(visible_title_main_menu_ready(
            shell.title_menu.as_ref().expect("game menu")
        ));
        if !northstar_opening::enabled(&shell.runtime) {
            assert!(shell.pending_audio.iter().all(|command| {
                !matches!(
                    command.audio_id.as_str(),
                    "MUSIC_TITLE" | "SFX_TITLE_SCREEN_ENTRANCE"
                )
            }));
        }
    }
    // B can return to the revised title, but must never replay its sequence.
    northstar_wait(&mut app, 4096);
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyX);
    northstar_wait(&mut app, 32);
    if app
        .world()
        .resource::<BevyRuntimeShell>()
        .title_menu
        .as_ref()
        .is_some_and(|title| title.northstar_opening.is_some())
    {
        assert!(
            app.world()
                .resource::<BevyRuntimeShell>()
                .intro_screen
                .is_none()
        );
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
        northstar_wait(&mut app, 32);
    }
    {
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(shell.intro_screen.is_none() && shell.splash_screen.is_none());
        assert!(visible_title_main_menu_ready(
            shell.title_menu.as_ref().expect("retained game menu")
        ));
    }
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
        review_northstar_frame(&images, &frame, "00-main-menu");
    }
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    let mut pages = BTreeSet::new();
    let mut sprites = BTreeSet::new();
    let mut capture = 1;
    let mut picked_name = false;
    for _ in 0..128 {
        northstar_wait(&mut app, 96);
        let (name_menu, complete) = {
            let mut shell = app.world_mut().resource_mut::<BevyRuntimeShell>();
            assert_eq!(shell.last_error, None);
            assert!(
                shell.pending_time_set.is_none(),
                "Northstar must not ask players to set the clock"
            );
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

#[test]
#[ignore = "requires CRYSTAL_NORTHSTAR_TITLE_PACK pointing to any completed Minnesota pack"]
fn minnesota_direct_menu_preserves_continue() {
    let path =
        PathBuf::from(std::env::var_os("CRYSTAL_NORTHSTAR_TITLE_PACK").expect("Minnesota pack"));
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path)
        .expect("verified Minnesota pack");
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).expect("runtime");
    let spawn_identifier = runtime
        .title_new_game_spawn_identifier()
        .expect("regional spawn");
    let save_path = std::env::temp_dir().join(format!(
        "minnesota-direct-menu-{}.crystalsave",
        uuid::Uuid::new_v4()
    ));
    let mut controller =
        VisibleShellController::new_game(root.clone(), runtime.clone(), "REED", None)
            .expect("new regional game");
    let saved = controller.snapshot().expect("saved position");
    {
        let time = &mut controller.shell.shell.session_mut().state_mut().time;
        time.set_manual_time(
            GameDate::new(2000, 1, 1),
            12,
            0,
            0,
            ClockTime::new(3, 7, 30, 0),
        );
        time.rtc_status_flags = crate::core::systems::time::RTC_RESET;
    }
    controller.save(&save_path).expect("save regional game");
    let resumed_controller =
        VisibleShellController::load_save(root.clone(), runtime.clone(), save_path.clone(), None)
            .expect("terminal resumes an old regional clock");
    let clock = &resumed_controller.shell.shell.session().state().time;
    let local = NativeRtcSource::system_local().sample();
    assert_eq!(clock.current_date, local.date);
    assert_eq!(clock.registers.hours, local.hour);
    assert_eq!(clock.start_time, ClockTime::default());
    let shell = initialize_bevy_runtime_shell(
        root,
        runtime,
        BevyShellStart::Title {
            spawn_identifier,
            save_path: Some(save_path.clone()),
        },
        BevyShellConfig::default(),
    )
    .expect("direct game menu");
    let mut app = integrated_shell_test_app(shell);
    enter_northstar_menu_after_sequence(&mut app, &mut Assets::<Image>::default());
    {
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(shell.intro_screen.is_none() && shell.splash_screen.is_none());
        let title = shell.title_menu.as_ref().expect("game menu");
        assert!(visible_title_main_menu_ready(title));
        assert!(
            visible_title_menu_entries(shell, title)
                .expect("menu entries")
                .iter()
                .any(|entry| entry.contains("CONTINUE"))
        );
    }
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    northstar_wait(&mut app, 32);
    assert!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .visible_continue_screen
            .is_some()
    );
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyX);
    assert!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .title_menu
            .as_ref()
            .is_some_and(visible_title_main_menu_ready)
    );
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    northstar_wait(&mut app, 32);
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyZ);
    northstar_wait(&mut app, 32);
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert_eq!(shell.last_error, None);
    assert!(shell.title_menu.is_none());
    let resumed = shell.shell.snapshot().expect("resumed game");
    assert_eq!(resumed.trainer.player_name, "REED");
    assert_eq!(resumed.overworld.map_name, saved.overworld.map_name);
    assert_eq!(resumed.overworld.tile, saved.overworld.tile);
    assert!(shell.pending_time_set.is_none());
    let clock = &shell.shell.session().state().time;
    let sample = native_rtc_source_for_test().sample();
    assert_eq!(clock.current_date, sample.date);
    assert_eq!(clock.registers.hours, sample.hour);
    assert_eq!(clock.start_time, ClockTime::default());
    std::fs::remove_file(save_path).expect("remove test save");
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

fn enter_northstar_menu_after_sequence(app: &mut App, images: &mut Assets<Image>) {
    let shell = app.world().resource::<BevyRuntimeShell>();
    if !northstar_opening::enabled(&shell.runtime) {
        return;
    }
    let before = shell.shell.snapshot().expect("untouched opening state");
    let mut scenes = BTreeSet::new();
    let mut captured = BTreeSet::new();
    let mut heard_opening = false;
    assert!(shell.splash_screen.is_some() && shell.intro_screen.is_some());
    for _ in 0..8000 {
        app.update();
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(
            shell.last_error, None,
            "original source movie remains playable"
        );
        heard_opening |= shell.active_music.as_deref() == Some("MUSIC_CRYSTAL_OPENING");
        if let Some(intro) = &shell.intro_screen {
            if shell.splash_screen.is_none() {
                scenes.insert(intro.jumptable_index);
                if intro.scene_frame_counter >= 16 && captured.insert(intro.jumptable_index) {
                    review_northstar_presented_frame(
                        app,
                        &format!("source-scene-{:02}", intro.jumptable_index),
                    );
                }
            }
        } else {
            assert!(shell.splash_screen.is_none());
            break;
        }
    }
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert!(
        shell.intro_screen.is_none(),
        "original source movie must finish"
    );
    assert!(
        scenes.len() >= 20,
        "run the complete original movie, not a replacement montage"
    );
    assert!(heard_opening, "source movie queues its original music");
    let current = shell
        .shell
        .snapshot()
        .expect("presentation leaves gameplay alone");
    assert_eq!(current.overworld, before.overworld);
    assert_eq!(current.party, before.party);
    assert!(
        shell.intro_completion_clock.is_some(),
        "source cleanup records its own clock"
    );
    let mut art = RenderedTilesetArt::default();
    let source_title = crate::open_runtime_image(
        shell
            .asset_root
            .runtime_assets()
            .join("gfx/title/northstar.png"),
    )
    .expect("authored native title")
    .to_rgba8();
    let expected_prompt = northstar_title_region(source_title.as_raw(), (47, 133, 67, 7));
    let expected_lettering = northstar_title_region(source_title.as_raw(), (0, 0, 160, 44));
    let expected_head = northstar_title_region(source_title.as_raw(), (54, 44, 40, 34));
    let expected_chest = northstar_title_region(source_title.as_raw(), (60, 78, 48, 32));
    let mut first_hero: Option<Vec<u8>> = None;
    let mut settled_hero: Option<Vec<u8>> = None;
    let mut settled_head: Option<Vec<u8>> = None;
    let mut settled_chest: Option<Vec<u8>> = None;
    let mut planted_feet: Option<Vec<u8>> = None;
    let mut revealed = false;
    let mut ambient_wraps = 0;
    let mut previous_clock = 0;
    let mut maximum_hero_motion = 0;
    let mut maximum_head_outline_motion = 0;
    let mut maximum_chest_outline_motion = 0;
    let mut reveal_changes = 0;
    let mut completed_lettering_seen = false;
    // Observe the one-shot entrance and two complete ambient wraps through the
    // production clock. A periodic fade or a new movie must not restart it.
    let observation_frames =
        northstar_opening::REVEAL_FRAMES + 2 * northstar_opening::LOOP_FRAMES + 8;
    for index in 0..observation_frames / 4 {
        let shell = app.world().resource::<BevyRuntimeShell>();
        let title = shell.title_menu.as_ref().expect("regional title owner");
        assert!(!visible_title_main_menu_ready(title));
        let clock = title
            .northstar_opening
            .as_ref()
            .expect("Northstar presentation clock")
            .frame;
        if index == 0 {
            assert!(
                clock < northstar_opening::REVEAL_FRAMES,
                "original movie leads into the title reveal"
            );
        }
        if revealed {
            assert!(
                clock >= northstar_opening::REVEAL_FRAMES,
                "completed reveal never plays again"
            );
            ambient_wraps += usize::from(clock < previous_clock);
        }
        revealed |= clock >= northstar_opening::REVEAL_FRAMES;
        previous_clock = clock;
        assert!(shell.intro_screen.is_none() && shell.splash_screen.is_none());
        let frame = title_screen_frame_for_art(&mut art, &shell.asset_root, title, images)
            .unwrap_or_else(|| panic!("native title render: {:?}", art.title_screen_errors));
        let pixels = images
            .get(&frame.handle)
            .and_then(|image| image.data.as_ref())
            .expect("native pixels");
        assert_eq!(
            northstar_title_region(pixels, (47, 133, 67, 7)),
            expected_prompt,
            "Start prompt remains intact during the reveal and ambient motion"
        );
        let image_clock = clock / 4 * 4;
        if image_clock == northstar_opening::REVEAL_FRAMES {
            assert_eq!(
                northstar_title_region(pixels, (0, 0, 160, 44)),
                expected_lettering,
                "the reveal settles into the full authored title lettering"
            );
            completed_lettering_seen = true;
        }
        let hero = northstar_title_region(pixels, (24, 44, 112, 84));
        if let Some(first) = &first_hero {
            if settled_hero.is_none() {
                reveal_changes = reveal_changes.max(northstar_changed_pixels(&hero, first));
            }
        } else {
            first_hero = Some(hero.clone());
        }
        if revealed {
            let head = northstar_title_region(pixels, (54, 44, 40, 34));
            let chest = northstar_title_region(pixels, (60, 78, 48, 32));
            if image_clock == northstar_opening::REVEAL_FRAMES {
                assert_eq!(
                    northstar_changed_outline_pixels(&head, &expected_head),
                    0,
                    "the resting face retains its authored outline"
                );
                assert_eq!(
                    northstar_changed_outline_pixels(&chest, &expected_chest),
                    0,
                    "the resting chest retains its authored outline"
                );
            }
            if let Some(first) = &settled_head {
                maximum_head_outline_motion =
                    maximum_head_outline_motion.max(northstar_changed_outline_pixels(&head, first));
            } else {
                settled_head = Some(head);
            }
            if let Some(first) = &settled_chest {
                maximum_chest_outline_motion = maximum_chest_outline_motion
                    .max(northstar_changed_outline_pixels(&chest, first));
            } else {
                settled_chest = Some(chest);
            }
            let feet = northstar_title_region(pixels, (24, 128, 112, 4));
            if let Some(first) = &planted_feet {
                assert_eq!(&feet, first, "creature keeps its feet planted");
            } else {
                planted_feet = Some(feet);
            }
            if let Some(first) = &settled_hero {
                maximum_hero_motion =
                    maximum_hero_motion.max(northstar_changed_pixels(&hero, first));
            } else {
                settled_hero = Some(hero);
            }
        }
        let elapsed_frames = index * 4;
        let capture = if elapsed_frames <= northstar_opening::REVEAL_FRAMES {
            elapsed_frames % 12 == 0
        } else if elapsed_frames
            <= northstar_opening::REVEAL_FRAMES + northstar_opening::LOOP_FRAMES
        {
            (elapsed_frames - northstar_opening::REVEAL_FRAMES).is_multiple_of(24)
        } else {
            index % 30 == 0
        };
        let loop_seam = image_clock == northstar_opening::REVEAL_FRAMES
            || image_clock == northstar_opening::REVEAL_FRAMES + northstar_opening::LOOP_FRAMES - 4;
        if capture || loop_seam {
            review_northstar_frame(images, &frame, &format!("animated-title-{index:03}"));
        }
        assert!(images.len() <= 2, "title retains bounded image allocations");
        northstar_wait(app, 4);
    }
    assert!(
        reveal_changes > 100,
        "the native creature has a visible entrance"
    );
    assert!(
        maximum_hero_motion > 100,
        "the settled creature visibly moves"
    );
    assert!(
        maximum_head_outline_motion > 15,
        "the head changes position rather than only recoloring its water"
    );
    assert!(
        maximum_chest_outline_motion > 10,
        "the chest changes shape while the feet stay planted"
    );
    assert!(
        ambient_wraps >= 2,
        "observe two ambient cycles after the reveal"
    );
    assert!(
        completed_lettering_seen,
        "observe the completed title lettering"
    );
    println!(
        "TITLE MOTION: reveal {reveal_changes} pixels, ambient hero max {maximum_hero_motion} pixels, head outline {maximum_head_outline_motion} pixels, chest outline {maximum_chest_outline_motion} pixels, {ambient_wraps} completed ambient wraps"
    );
    let shell = app.world().resource::<BevyRuntimeShell>();
    let after = shell.shell.snapshot().expect("title leaves gameplay alone");
    assert_eq!(after.overworld, before.overworld);
    assert_eq!(after.party, before.party);
    assert_eq!(after.trainer, before.trainer);
    press_key_for_runtime_hotkey_app(app, KeyCode::Enter);
    northstar_wait(app, 64);
    assert!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .title_menu
            .as_ref()
            .is_some_and(visible_title_main_menu_ready)
    );
}

fn northstar_title_region(pixels: &[u8], bounds: (usize, usize, usize, usize)) -> Vec<u8> {
    assert_eq!(pixels.len(), 160 * 144 * 4, "native title dimensions");
    let (left, top, width, height) = bounds;
    (top..top + height)
        .flat_map(|y| {
            let start = (y * 160 + left) * 4;
            pixels[start..start + width * 4].iter().copied()
        })
        .collect()
}

fn northstar_changed_pixels(current: &[u8], previous: &[u8]) -> usize {
    assert_eq!(current.len(), previous.len());
    current
        .as_chunks::<4>()
        .0
        .iter()
        .zip(previous.as_chunks::<4>().0)
        .filter(|(current, previous)| current != previous)
        .count()
}

fn northstar_changed_outline_pixels(current: &[u8], previous: &[u8]) -> usize {
    assert_eq!(current.len(), previous.len());
    current
        .as_chunks::<4>()
        .0
        .iter()
        .zip(previous.as_chunks::<4>().0)
        .map(|(current, previous)| (current == &[0, 0, 0, 255]) != (previous == &[0, 0, 0, 255]))
        .filter(|changed| *changed)
        .count()
}

fn review_northstar_presented_frame(app: &App, name: &str) {
    let art = app.world().resource::<RenderedTilesetArt>();
    if let Some(frame) = &art.intro_presented_surface {
        review_northstar_frame(app.world().resource::<Assets<Image>>(), frame, name);
    }
}

fn northstar_opening_test_runtime() -> (AssetRoot, CrystalRuntime) {
    let path =
        PathBuf::from(std::env::var_os("CRYSTAL_NORTHSTAR_TITLE_PACK").expect("Northstar pack"));
    let root = AssetRoot::new(path.parent().expect("pack directory"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        crystal_assets::read_loaded_verified_compiled_game_pack(&path).expect("verified pack"),
    )
    .expect("runtime");
    assert!(northstar_opening::enabled(&runtime));
    (root, runtime)
}

#[test]
#[ignore = "requires CRYSTAL_NORTHSTAR_TITLE_PACK with the original opening"]
fn northstar_original_movie_skip_preserves_title_and_menu() {
    let (root, runtime) = northstar_opening_test_runtime();
    assert!(startup_policy::plays_source_movie(&runtime, false));
    assert!(!startup_policy::plays_source_movie(&runtime, true));
    for (skip_key, confirm_key) in [
        (KeyCode::KeyX, KeyCode::Enter),
        (KeyCode::KeyZ, KeyCode::KeyZ),
        (KeyCode::Enter, KeyCode::Enter),
    ] {
        let shell = initialize_bevy_runtime_shell(
            root.clone(),
            runtime.clone(),
            BevyShellStart::Title {
                spawn_identifier: runtime.title_new_game_spawn_identifier().expect("spawn"),
                save_path: None,
            },
            BevyShellConfig::default(),
        )
        .expect("opening shell");
        let mut app = integrated_shell_test_app(shell);
        for _ in 0..2048 {
            app.update();
            if app
                .world()
                .resource::<BevyRuntimeShell>()
                .splash_screen
                .is_none()
            {
                break;
            }
        }
        assert!(
            app.world()
                .resource::<BevyRuntimeShell>()
                .splash_screen
                .is_none()
        );
        assert!(
            app.world()
                .resource::<BevyRuntimeShell>()
                .intro_screen
                .is_some()
        );
        northstar_wait(&mut app, 120);
        press_key_for_runtime_hotkey_app(&mut app, skip_key);
        northstar_wait(&mut app, 16);
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(shell.intro_screen.is_none(), "source skip cleanup finishes");
        assert!(
            !shell
                .title_menu
                .as_ref()
                .is_some_and(visible_title_main_menu_ready),
            "skip must not also open the menu"
        );
        assert!(
            shell
                .title_menu
                .as_ref()
                .and_then(|title| title.northstar_opening.as_ref())
                .is_some_and(|opening| opening.frame < northstar_opening::REVEAL_FRAMES),
            "confirm while the reveal is still in progress"
        );
        press_key_for_runtime_hotkey_app(&mut app, confirm_key);
        northstar_wait(&mut app, 64);
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert!(
            shell
                .title_menu
                .as_ref()
                .is_some_and(visible_title_main_menu_ready)
        );
        assert_eq!(shell.last_error, None);
        assert!(
            shell
                .title_menu
                .as_ref()
                .and_then(|title| title.northstar_opening.as_ref())
                .is_some_and(|opening| opening.frame >= northstar_opening::REVEAL_FRAMES),
            "early confirm finishes the reveal before opening the menu"
        );
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::KeyX);
        northstar_wait(&mut app, 32);
        let shell = app.world().resource::<BevyRuntimeShell>();
        let title = shell.title_menu.as_ref().expect("returned title");
        assert!(!visible_title_main_menu_ready(title));
        assert!(shell.intro_screen.is_none() && shell.splash_screen.is_none());
        assert!(
            title
                .northstar_opening
                .as_ref()
                .is_some_and(|opening| opening.frame >= northstar_opening::REVEAL_FRAMES),
            "B returns to the completed title without replaying its entrance"
        );
        press_key_for_runtime_hotkey_app(&mut app, KeyCode::Enter);
        northstar_wait(&mut app, 64);
        assert!(
            app.world()
                .resource::<BevyRuntimeShell>()
                .title_menu
                .as_ref()
                .is_some_and(visible_title_main_menu_ready)
        );
    }
}

#[test]
#[ignore = "requires CRYSTAL_NORTHSTAR_TITLE_PACK and GEOTHITE_REDUCED_MOTION=1"]
fn northstar_reduced_motion_starts_at_still_title() {
    assert!(
        northstar_opening::prefers_reduced_motion(),
        "set GEOTHITE_REDUCED_MOTION=1"
    );
    let (root, runtime) = northstar_opening_test_runtime();
    let shell = initialize_bevy_runtime_shell(
        root,
        runtime.clone(),
        BevyShellStart::Title {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("spawn"),
            save_path: None,
        },
        BevyShellConfig::default(),
    )
    .expect("reduced-motion opening shell");
    assert!(shell.splash_screen.is_none() && shell.intro_screen.is_none());
    let mut app = integrated_shell_test_app(shell);
    app.update();
    let first = {
        let art = app.world().resource::<RenderedTilesetArt>();
        let frame = art
            .intro_presented_surface
            .as_ref()
            .expect("still title surface");
        app.world()
            .resource::<Assets<Image>>()
            .get(&frame.handle)
            .expect("native title")
            .data
            .clone()
    };
    {
        let shell = app.world().resource::<BevyRuntimeShell>();
        let mut art = northstar_opening_art::Art::load(&shell.asset_root)
            .expect("completed native title artwork");
        let mut images = Assets::<Image>::default();
        let completed = northstar_opening::Opening {
            frame: northstar_opening::REVEAL_FRAMES,
            reduced_motion: false,
        };
        let frame = art.render(&completed, &mut images);
        assert_eq!(
            images.get(&frame.handle).expect("completed title").data,
            first,
            "reduced motion starts with the completed composition"
        );
    }
    northstar_wait(&mut app, 128);
    let art = app.world().resource::<RenderedTilesetArt>();
    let frame = art
        .intro_presented_surface
        .as_ref()
        .expect("still title surface");
    assert_eq!(
        app.world()
            .resource::<Assets<Image>>()
            .get(&frame.handle)
            .expect("native title")
            .data,
        first
    );
    assert_eq!(
        app.world()
            .resource::<BevyRuntimeShell>()
            .title_menu
            .as_ref()
            .expect("title")
            .northstar_opening
            .as_ref()
            .expect("title animation")
            .frame,
        0
    );
    press_key_for_runtime_hotkey_app(&mut app, KeyCode::Enter);
    northstar_wait(&mut app, 64);
    let shell = app.world().resource::<BevyRuntimeShell>();
    assert!(
        shell
            .title_menu
            .as_ref()
            .is_some_and(visible_title_main_menu_ready)
    );
    assert_eq!(shell.last_error, None);
}
