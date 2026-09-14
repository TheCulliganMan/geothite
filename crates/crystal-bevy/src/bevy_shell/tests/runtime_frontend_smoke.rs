use crystal_runtime::test_support::load_minimal_compiled_runtime;

#[test]
fn visible_title_new_game_does_not_eagerly_load_stale_continue_save() {
    let (root, asset_root, runtime) = load_minimal_compiled_runtime("title-stale-save");
    let stale_save = root.join("target/crystal-bevy/saves/core-modular.crystalsave");
    std::fs::create_dir_all(stale_save.parent().expect("save parent")).expect("create save dir");
    std::fs::write(&stale_save, b"not a crystal runtime save").expect("write stale save");

    let smoke = smoke_visible_shell_title(asset_root, runtime, 0, Some(stale_save), false)
        .expect("title new game must boot without loading stale Continue save");

    assert_eq!(smoke.selected, "NEW_GAME");
    assert_eq!(smoke.map, "RuntimeMap");
    assert_eq!(smoke.tile_x, 0);
    assert_eq!(smoke.tile_y, 0);
    assert_eq!(smoke.saved_frame, None);
    assert_eq!(
        smoke.title_entries,
        vec![
            " CONTINUE".to_string(),
            ">NEW GAME".to_string(),
            " OPTION".to_string()
        ]
    );
    let (reject_root, reject_asset_root, reject_runtime) =
        load_minimal_compiled_runtime("title-stale-save-reject");
    let reject_stale_save = reject_root.join("target/crystal-bevy/saves/core-modular.crystalsave");
    std::fs::create_dir_all(reject_stale_save.parent().expect("save parent"))
        .expect("create save dir");
    std::fs::write(&reject_stale_save, b"not a crystal runtime save").expect("write stale save");
    let error = smoke_visible_shell_title(
        reject_asset_root,
        reject_runtime,
        0,
        Some(reject_stale_save),
        true,
    )
    .expect_err("title Continue must reject an invalid configured save");
    assert!(
        error.to_string().contains("title Continue rejected"),
        "{error:#}"
    );
    let _ = std::fs::remove_dir_all(reject_root);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn visible_title_recovers_backup_only_save_before_offering_continue() {
    let (root, asset_root, runtime) = load_minimal_compiled_runtime("title-backup-save");
    let save_path = root.join("target/crystal-bevy/saves/core-modular.crystalsave");
    let mut shell =
        RuntimeGameShell::new_game(asset_root.clone(), runtime.clone(), 0).expect("game shell");
    shell.save(&save_path).expect("write primary save");
    shell
        .save(&save_path)
        .expect("rotate primary save to backup");
    let backup_path = PathBuf::from(format!("{}.bak", save_path.display()));
    assert!(backup_path.exists(), "second save must create a backup");
    std::fs::remove_file(&save_path).expect("remove primary save");

    let smoke = smoke_visible_shell_title(asset_root, runtime, 0, Some(save_path.clone()), true)
        .expect("title Continue must recover the validated backup save");

    assert_eq!(smoke.selected, "CONTINUE");
    assert_eq!(
        smoke.title_entries,
        vec![
            ">CONTINUE".to_string(),
            " NEW GAME".to_string(),
            " OPTION".to_string()
        ]
    );
    assert!(
        save_path.exists(),
        "reading the valid backup must restore the primary save"
    );
    assert_eq!(smoke.saved_frame, Some(0));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn visible_title_new_game_name_input_is_controlled_before_spawn() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
    let asset_root = AssetRoot::new(root.clone());
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(
        root.join("content-packs/core-modular.browser.crystalpack"),
    ).expect("load canonical title runtime");
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&asset_root, loaded)
        .expect("load canonical title runtime");
    let spawn_identifier = runtime
        .title_new_game_spawn_identifier()
        .expect("canonical new-game spawn identifier");

    let smoke = smoke_visible_shell_title_name_input(
        asset_root,
        runtime,
        spawn_identifier,
        None,
        "AB",
    )
        .expect("visible title name input smoke must type and confirm player name");

    assert_eq!(smoke.selected, "NEW_GAME");
    assert_eq!(
        smoke.title_entries,
        vec![">NEW GAME".to_string(), " OPTION".to_string()]
    );
    assert_eq!(
        smoke.initial_name_entries.first().map(String::as_str),
        Some("NAME ENTRY")
    );
    assert!(
        smoke
            .initial_name_entries
            .iter()
            .any(|entry| entry == "YOUR NAME?"),
        "{:?}",
        smoke.initial_name_entries
    );
    assert!(
        smoke
            .initial_name_entries
            .iter()
            .any(|entry| entry == "A B C D E F G H I"),
        "{:?}",
        smoke.initial_name_entries
    );
    assert!(
        smoke
            .initial_name_entries
            .iter()
            .any(|entry| entry == "lower  DEL   END "),
        "{:?}",
        smoke.initial_name_entries
    );
    assert!(
        smoke
            .typed_name_entries
            .iter()
            .any(|entry| entry == "NAME AB_"),
        "{:?}",
        smoke.typed_name_entries
    );
    assert_eq!(smoke.trainer_name, "AB");
    assert_eq!(smoke.map, "PlayersHouse2F");
    assert_eq!(smoke.tile_x, 3);
    assert_eq!(smoke.tile_y, 3);
    assert_ne!(smoke.state_hash.hash(), 0);
}

#[test]
fn visible_overworld_smoke_replays_same_inputs_deterministically() {
    let input_frames = vec![
        vec![GameButton::Right],
        vec![GameButton::Right],
        vec![GameButton::Down],
        vec![GameButton::Left],
        vec![GameButton::Up],
    ];
    let (first_root, first_asset_root, first_runtime) =
        load_minimal_compiled_runtime("visible-overworld-deterministic-a");
    let first = smoke_visible_shell_overworld(
        first_asset_root,
        first_runtime,
        BevyShellStart::NewGame {
            spawn_identifier: 0,
        },
        BevyShellConfig {
            smoke_player_name: Some("AB".to_string()),
            ..Default::default()
        },
        &input_frames,
        None,
    )
    .expect("first visible overworld smoke");
    let (second_root, second_asset_root, second_runtime) =
        load_minimal_compiled_runtime("visible-overworld-deterministic-b");
    let second = smoke_visible_shell_overworld(
        second_asset_root,
        second_runtime,
        BevyShellStart::NewGame {
            spawn_identifier: 0,
        },
        BevyShellConfig {
            smoke_player_name: Some("AB".to_string()),
            ..Default::default()
        },
        &input_frames,
        None,
    )
    .expect("second visible overworld smoke");

    assert_eq!(first, second);
    assert_eq!(first.frames, input_frames.len());
    assert_eq!(first.start_map, "RuntimeMap");
    assert_eq!(first.final_map, "RuntimeMap");
    assert_eq!(first.state_hash.frame(), input_frames.len() as u64);
    assert_ne!(first.state_hash.hash(), 0);
    let _ = std::fs::remove_dir_all(second_root);
    let _ = std::fs::remove_dir_all(first_root);
}
