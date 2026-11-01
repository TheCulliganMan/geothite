// Disposable starts exercise every restored nurse through production input.
#[test]
#[ignore = "requires CRYSTAL_CENTER_PARENT and a fresh ignored CRYSTAL_CENTER_CANDIDATE"]
fn minnesota_restored_centers_build_review_candidate() {
    let parent = PathBuf::from(std::env::var_os("CRYSTAL_CENTER_PARENT").expect("parent pack"));
    let path = PathBuf::from(std::env::var_os("CRYSTAL_CENTER_CANDIDATE").expect("candidate path"));
    assert!(!path.exists(), "candidate must be a fresh output");
    assert!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&path)
            .status()
            .expect("git check-ignore")
            .success(),
        "candidate must remain ignored"
    );
    let parent = crystal_assets::read_verified_compiled_game_pack(parent).expect("verified parent");
    let candidate =
        crystal_assets::restore_minnesota_center_nurses(&parent).expect("restored candidate");
    candidate
        .write_preserving_storage(&path)
        .expect("write candidate");
    assert_eq!(
        crystal_assets::read_verified_compiled_game_pack(path).expect("verify candidate"),
        candidate
    );
}

#[test]
#[ignore = "requires a restored candidate in CRYSTAL_RENDER_TEST_PACK"]
fn minnesota_restored_centers_heal_through_native_nurse_input() {
    let pack = PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("candidate pack"));
    let root = AssetRoot::new(pack.parent().expect("pack directory"));
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&pack).expect("candidate");
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).expect("runtime");
    let mut centers: Vec<_> = runtime
        .data()
        .maps
        .iter()
        .filter(|(name, room)| {
            name.starts_with("Mn")
                && matches!(
                    room.attributes.tileset_name.as_str(),
                    "pokecenter" | "pokecom_center"
                )
                && room
                    .objects
                    .iter()
                    .any(|object| object.sprite == "SPRITE_NURSE")
        })
        .map(|(name, _)| name.clone())
        .collect();
    centers.sort();
    assert!(
        centers.len() >= 19,
        "missing Minnesota centers: {centers:?}"
    );
    for map in centers {
        let room = &runtime.data().maps[&map];
        let nurse = room
            .objects
            .iter()
            .find(|object| object.sprite == "SPRITE_NURSE")
            .expect("nurse");
        let nurse_tile =
            crystal_core::world::session::object_tile_position_checked(nurse).expect("nurse tile");
        assert_eq!(nurse_tile, TilePosition::new(3, 1), "{map}");
        let shell = initialize_bevy_runtime_shell(
            root.clone(),
            runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: runtime.title_new_game_spawn_identifier().expect("start"),
                map_name: map.clone(),
                tile_x: 3,
                tile_y: 5,
            },
            BevyShellConfig {
                smoke_player_name: Some("CHRIS".into()),
                ..default()
            },
        )
        .expect("center fixture");
        let mut controller = VisibleShellController { shell };
        complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
            .expect("name");
        controller
            .shell
            .shell
            .add_party_pokemon(
                "BIRCHIT",
                5,
                None,
                None,
                "CHRIS",
                1,
                Dv::from_non_hp(10, 10, 10, 10),
            )
            .expect("partner");
        let (max_hp, max_pp) = {
            let state = controller.shell.shell.session_mut().state_mut();
            let pokemon = state.storage.party.pokemon[0].as_mut().expect("partner");
            pokemon.hp = 1;
            pokemon.moves[0].current_pp = 0;
            let result = (
                pokemon.max_hp,
                runtime.data().moves[&pokemon.moves[0].name].pp,
            );
            state.sync_party_from_storage();
            result
        };
        for target_y in [4, 3] {
            for _ in 0..8 {
                if controller.snapshot().expect("snapshot").overworld.tile.y == target_y {
                    break;
                }
                controller.press(GameButton::Up).expect("approach nurse");
            }
            assert_eq!(
                controller.snapshot().expect("snapshot").overworld.tile,
                TilePosition::new(3, target_y),
                "{map}: nurse approach blocked"
            );
        }
        controller.press(GameButton::Up).expect("face nurse");
        assert!(
            controller
                .shell
                .shell
                .session()
                .overworld()
                .check_interaction_checked(1)
                .expect("interaction")
                .is_some(),
            "{map}: nurse unreachable"
        );
        if let Some(directory) = std::env::var_os("CLINIC_REVIEW_DIR") {
            let directory = PathBuf::from(directory);
            std::fs::create_dir_all(&directory).expect("review directory");
            controller
                .save(directory.join(format!("{map}.crystalsave")))
                .expect("review fixture");
        }
        let mut saw_text = false;
        for _ in 0..128 {
            let snapshot = controller.snapshot().expect("snapshot");
            saw_text |= snapshot.ui.text_window_open;
            let pokemon = controller
                .shell
                .shell
                .session()
                .state()
                .storage
                .party
                .pokemon[0]
                .as_ref()
                .expect("partner");
            if pokemon.hp == max_hp && pokemon.moves[0].current_pp == max_pp {
                break;
            }
            controller.press(GameButton::A).expect("nurse input");
        }
        assert!(saw_text, "{map}: no nurse dialogue");
        let pokemon = controller
            .shell
            .shell
            .session()
            .state()
            .storage
            .party
            .pokemon[0]
            .as_ref()
            .expect("partner");
        assert_eq!(
            (pokemon.hp, pokemon.moves[0].current_pp),
            (max_hp, max_pp),
            "{map}"
        );
        for _ in 0..32 {
            if !controller.snapshot().expect("snapshot").ui.text_window_open {
                break;
            }
            controller
                .press(GameButton::A)
                .expect("finish nurse dialogue");
        }
        let save_path = std::env::temp_dir().join(format!(
            "geothite-center-review-{}-{map}.crystalsave",
            std::process::id()
        ));
        assert!(!save_path.exists(), "fresh review save required");
        controller.save(&save_path).expect("save after healing");
        let saved = runtime.load_save(&save_path).expect("reload healed save");
        assert_eq!(
            saved.last_spawn_map_constant.as_deref(),
            room.attributes.map_constant.as_deref(),
            "{map}: healed location is not saved"
        );
        runtime
            .data()
            .resume_overworld_session_from_state(saved, &runtime.music_ids())
            .expect("resume in restored room");
        std::fs::remove_file(save_path).expect("remove disposable save");
        controller.press(GameButton::Start).expect("open Start");
        assert!(
            controller.snapshot().expect("snapshot").ui.menu.is_some(),
            "{map}: input retained"
        );
        controller.press(GameButton::B).expect("close Start");
        let exit = room
            .events
            .warps
            .iter()
            .find(|warp| warp.index == 1)
            .expect("exit");
        let destination = runtime
            .data()
            .map_name_for_constant(&exit.target_map_constant)
            .expect("exit destination");
        for _ in 0..32 {
            if controller.snapshot().expect("snapshot").overworld.map_name != map {
                break;
            }
            controller.press(GameButton::Down).expect("walk out");
        }
        assert_eq!(
            controller.snapshot().expect("snapshot").overworld.map_name,
            destination,
            "{map}: exterior warp"
        );
        assert_eq!(controller.shell.last_error, None, "{map}");
        let pc_shell = initialize_bevy_runtime_shell(
            root.clone(),
            runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: runtime.title_new_game_spawn_identifier().expect("start"),
                map_name: map.clone(),
                tile_x: 9,
                tile_y: 2,
            },
            BevyShellConfig {
                smoke_player_name: Some("CHRIS".into()),
                ..default()
            },
        )
        .expect("PC fixture");
        let mut pc = VisibleShellController { shell: pc_shell };
        complete_visible_smoke_player_name_if_needed(&mut pc.shell, Some("CHRIS")).expect("name");
        pc.press(GameButton::Up).expect("face PC");
        let script = pc
            .shell
            .shell
            .current_overworld_interaction_checked()
            .expect("PC interaction")
            .expect("PC target")
            .script;
        assert!(
            script == "PCScript"
                || room
                    .scripts
                    .get(&script)
                    .and_then(|body| body.as_array())
                    .is_some_and(
                        |body| body.iter().any(|command| command["command"] == "special"
                            && command["args"][0] == "PokemonCenterPC")
                    ),
            "{map}: PC unavailable: {script}"
        );
        println!("{map}: nurse, HP/PP recovery, save/continue, exit and PC interaction passed");
    }
}

#[test]
#[ignore = "requires a restored candidate in CRYSTAL_RENDER_TEST_PACK"]
fn minnesota_restored_centers_rochester_rival_approaches_before_dialogue() {
    let pack = PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("candidate pack"));
    let root = AssetRoot::new(pack.parent().expect("pack directory"));
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&pack).expect("candidate");
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).expect("runtime");
    for x in [3, 4, 6, 9] {
        let shell = initialize_bevy_runtime_shell(
            root.clone(),
            runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: runtime.title_new_game_spawn_identifier().expect("start"),
                map_name: "MnRochesterPokecenter1F".into(),
                tile_x: x,
                tile_y: 5,
            },
            BevyShellConfig {
                smoke_player_name: Some("CHRIS".into()),
                ..default()
            },
        )
        .expect("scene fixture");
        let mut controller = VisibleShellController { shell };
        complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
            .expect("name");
        controller
            .shell
            .shell
            .session_mut()
            .state_mut()
            .flags
            .set_event_flag("EVENT_MN_DELIVERED_ROCHESTER_PARCEL", true)
            .expect("prior delivery");
        mark_runtime_snapshot_dirty(&mut controller.shell);
        let mut observed = false;
        for _ in 0..32 {
            controller.press(GameButton::Up).expect("enter scene");
            let snapshot = controller.snapshot().expect("scene snapshot");
            if snapshot.ui.text_window_open {
                let reed = snapshot
                    .visible_object_runtime_tiles
                    .get("MN_ARC_RC_REED")
                    .expect("visible Reed");
                let player = snapshot.overworld.tile;
                assert_eq!(
                    (reed.x - player.x).abs() + (reed.y - player.y).abs(),
                    1,
                    "Reed must arrive before dialogue in lane {x}"
                );
                observed = true;
                break;
            }
        }
        assert!(observed, "Rochester rival dialogue missing in lane {x}");
        assert_eq!(controller.shell.last_error, None, "lane {x}");
    }
}

#[test]
#[ignore = "requires a restored candidate and external source art"]
fn minnesota_restored_centers_render_the_native_healing_machine() {
    let pack = PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("candidate pack"));
    let root = AssetRoot::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository"),
    );
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&pack).expect("candidate");
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).expect("runtime");
    for map in ["MnRochesterPokecenter1F", "MnWinonaPokecenter1F"] {
        let mut shell = initialize_bevy_runtime_shell(
            root.clone(),
            runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: runtime.title_new_game_spawn_identifier().expect("start"),
                map_name: map.into(),
                tile_x: 3,
                tile_y: 3,
            },
            BevyShellConfig {
                render_test_hour: Some(12),
                ..default()
            },
        )
        .expect("render fixture");
        shell
            .shell
            .add_party_pokemon(
                "BIRCHIT",
                5,
                None,
                None,
                "CHRIS",
                1,
                Dv::from_non_hp(10, 10, 10, 10),
            )
            .expect("partner");
        let mut app = integrated_shell_test_app(shell);
        app.update();
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowUp);
        app.update();
        let mut saw_animation = false;
        for frame in 0..1024 {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .reset_all();
            if frame % 16 == 0 {
                app.world_mut()
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .press(KeyCode::KeyZ);
            }
            app.update();
            if let Some(animation) = app
                .world()
                .resource::<BevyRuntimeShell>()
                .visible_heal_machine
                .as_ref()
            {
                assert_eq!((animation.kind, animation.party_count), (0, 1), "{map}");
                if app
                    .world()
                    .resource::<RenderedTilesetArt>()
                    .heal_machine_ball_cache
                    .is_some()
                {
                    let world = app.world_mut();
                    let mut sprites =
                        world.query_filtered::<&Sprite, With<HealMachineWorldSprite>>();
                    if sprites.iter(world).count() >= 3 {
                        saw_animation = true;
                        break;
                    }
                }
            }
        }
        assert!(
            saw_animation,
            "{map}: actual nurse flow did not render the native ball and lamps; error={:?}",
            app.world().resource::<BevyRuntimeShell>().last_error
        );
    }
}
