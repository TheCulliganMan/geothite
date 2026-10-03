    #[test]
    fn runtime_load_rejects_saved_state_under_different_currency_pack() {
        let root = temp_repository_root("load-currency-pack-mismatch");
        let asset_root = AssetRoot::new(&root);
        let writer_runtime = CrystalRuntime::from_compiled_pack(
            &asset_root,
            CompiledGamePack::new_unchecked_for_tests(
                runtime_data_with_currency_caps(999_999, 9_999),
                report(),
            ),
            identity(),
        )
        .expect("writer runtime");
        let reader_runtime = CrystalRuntime::from_compiled_pack(
            &asset_root,
            CompiledGamePack::new_unchecked_for_tests(
                runtime_data_with_currency_caps(500, 9_999),
                report(),
            ),
            identity(),
        )
        .expect("reader runtime");
        assert_ne!(
            writer_runtime.pack_identity().content_hash,
            reader_runtime.pack_identity().content_hash,
            "currency caps are compiled pack data and must change pack identity"
        );
        let save_path = root.join("slot.crystalsave");
        let mut state = GameState::default();
        state.money = 600;

        writer_runtime
            .save_game(&save_path, state)
            .expect("write state under writer cap");
        let error = reader_runtime
            .load_save(&save_path)
            .expect_err("different compiled pack must reject loaded state");
        let error = format!("{error:#}");

        assert!(
            error.contains("read Crystal runtime save for compiled modpack identity"),
            "{error}"
        );
        assert!(
            error.contains("save pack content hash") && error.contains("does not match expected"),
            "{error}"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_save_loader_rejects_different_compiled_pack_identity() {
        let root = temp_repository_root("save-mismatch");
        let asset_root = AssetRoot::new(&root);
        let first_runtime = CrystalRuntime::from_compiled_pack(
            &asset_root,
            CompiledGamePack::new_unchecked_for_tests(minimal_runtime_data(), report()),
            identity(),
        )
        .expect("first runtime");
        let second_runtime = CrystalRuntime::from_compiled_pack(
            &asset_root,
            CompiledGamePack::new_unchecked_for_tests(minimal_runtime_data(), report()),
            SaveModpackIdentity::new(
                "core-modular",
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            )
            .expect("identity"),
        )
        .expect("second runtime");
        let save_path = root.join("slot.crystalsave");

        first_runtime
            .save_game(&save_path, GameState::default())
            .expect("write runtime save");
        let error = second_runtime
            .load_save(&save_path)
            .expect_err("runtime must reject saves from another pack")
            .to_string();

        assert!(error.contains("read Crystal runtime save for compiled modpack identity"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_bootstrap_rejects_json_pack_paths() {
        let root = temp_repository_root("json");
        std::fs::write(
            root.join("apps/web/assets/data/runtime.json"),
            br#"{"not":"a runtime pack"}"#,
        )
        .expect("write json fixture");
        let asset_root = AssetRoot::new(&root);

        let error = CrystalRuntime::load_from_compiled_pack(&asset_root, "runtime.json")
            .expect_err("runtime must require .crystalpack")
            .to_string();

        assert!(error.contains("must use .crystalpack"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_bootstrap_rejects_traversing_pack_paths() {
        let root = temp_repository_root("traversing-pack-path");
        let asset_root = AssetRoot::new(&root);

        let parent_error =
            CrystalRuntime::load_from_compiled_pack(&asset_root, "../runtime.crystalpack")
                .expect_err("runtime must not load pack paths outside assets/data")
                .to_string();
        assert!(
            parent_error.contains("must not traverse parent directories"),
            "{parent_error}"
        );

        let current_error =
            CrystalRuntime::load_from_compiled_pack(&asset_root, "./runtime.crystalpack")
                .expect_err("runtime must not load current-directory pack paths")
                .to_string();
        assert!(
            current_error.contains("must not include current-directory components"),
            "{current_error}"
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_bootstrap_rejects_compiled_pack_without_manifest_identity() {
        let root = temp_repository_root("missing-identity");
        let data_root = root.join("apps/web/assets/data");
        let pack = CompiledGamePack::new_unchecked_for_tests(
            GameDataSet::default(),
            ModpackCompileReport::default(),
        );
        let error = crystal_assets::write_compiled_game_pack_for_tests(
            data_root.join("runtime.crystalpack"),
            &pack,
        )
        .expect_err("pack writer must reject missing manifest identity");
        let error = error_debug(error);

        assert!(
            error.contains("must include at least one manifest id"),
            "{error}"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_bootstrap_rejects_malformed_manifest_identity() {
        for (manifest_ids, expected) in [
            (
                vec![" core-modular".to_string()],
                "must be exact ASCII letters, numbers, underscores, hyphens, or dots",
            ),
            (
                vec!["core+modular".to_string()],
                "must be exact ASCII letters, numbers, underscores, hyphens, or dots",
            ),
            (
                vec!["core/modular".to_string()],
                "must be exact ASCII letters, numbers, underscores, hyphens, or dots",
            ),
            (
                vec!["core\nmodular".to_string()],
                "must be exact ASCII letters, numbers, underscores, hyphens, or dots",
            ),
            (
                vec!["core-modular".to_string(), "core-modular".to_string()],
                "duplicate manifest id 'core-modular'",
            ),
        ] {
            let root = temp_repository_root("malformed-identity");
            let asset_root = AssetRoot::new(&root);
            let pack = CompiledGamePack::new_unchecked_for_tests(
                minimal_runtime_data(),
                ModpackCompileReport {
                    manifests: manifest_ids,
                    ..ModpackCompileReport::default()
                },
            );

            let error = CrystalRuntime::from_compiled_pack(&asset_root, pack, identity())
                .expect_err("runtime must reject malformed report manifest identity")
                .to_string();

            assert!(error.contains(expected), "{error}");
            let _ = std::fs::remove_dir_all(root);
        }
    }

    #[test]
    fn runtime_bootstrap_cannot_load_unverified_loaded_pack_publicly() {
        let root = temp_repository_root("loaded-unverified");
        let data_root = root.join("apps/web/assets/data");
        let report = ModpackCompileReport {
            manifests: vec!["core-modular".to_string()],
            diagnostics: vec![VerificationError {
                severity: VerificationSeverity::Error,
                code: "bad_pack".to_string(),
                subject: "runtime".to_string(),
                message: "pack failed verification".to_string(),
            }],
            ..ModpackCompileReport::default()
        };
        let pack = CompiledGamePack::new_unchecked_for_tests(minimal_runtime_data(), report);
        crystal_assets::write_compiled_game_pack_for_tests(
            data_root.join("runtime.crystalpack"),
            &pack,
        )
        .expect("write compiled runtime pack");
        let asset_root = AssetRoot::new(&root);
        let error = asset_root
            .load_loaded_verified_compiled_game_pack("runtime.crystalpack")
            .expect_err("public loaded pack access must reject unverified packs");
        let error = error_debug(error);

        assert!(error.contains("compiled game pack is not verified for runtime"));
        assert!(error.contains("bad_pack"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_bootstrap_revalidates_embedded_pack_data_before_boot() {
        let root = temp_repository_root("runtime-revalidate");
        let asset_root = AssetRoot::new(&root);
        let mut data = verified_runtime_bootstrap_data();
        data.runtime_map_metadata
            .get_mut("RUNTIME_MAP")
            .expect("runtime metadata")
            .group_name
            .clear();
        let pack = CompiledGamePack::new_unchecked_for_tests(data, report());

        let error = CrystalRuntime::from_compiled_pack(&asset_root, pack, identity())
            .expect_err("runtime boot must reject mutated embedded pack data")
            .to_string();

        assert!(error.contains("invalid_runtime_map_metadata"), "{error}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_bootstrap_rejects_mp3_audio_declarations() {
        let root = temp_repository_root("mp3");
        let mut data = minimal_runtime_data();
        data.audio = vec![ModpackAudioAsset {
            id: "MUSIC_ROUTE_29".to_string(),
            path: "content-packs/test/music/MUSIC_ROUTE_29.mp3".to_string(),
            kind: ModpackAudioKind::Music,
            source: ModpackAudioSource::Pcm,
            sfx_priority: None,
            pcm_format: None,
            pcm_frame_count: None,
            payload_hash: None,
            loop_start_sample: None,
            loop_end_sample: None,
            midi_program: None,
        }];
        let error = data.audio[0]
            .validate()
            .expect_err("runtime must reject mp3 audio declarations")
            .to_string();

        assert!(error.contains("must use a .pcm file"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_bootstrap_rejects_missing_embedded_audio_payloads() {
        let root = temp_repository_root("missing-embedded-audio");
        let asset_root = AssetRoot::new(&root);
        let data = minimal_runtime_data();
        let compiled_audio = data
            .audio
            .iter()
            .filter(|asset| asset.id != "MUSIC_ROUTE_29")
            .map(|asset| (asset.id.clone(), vec![0_u8; 4]))
            .collect();
        let pack = CompiledGamePack::new_unchecked_with_audio_for_tests(
            data,
            compiled_audio,
            report(),
        );

        let error = CrystalRuntime::from_compiled_pack(&asset_root, pack, identity())
            .expect_err("runtime must not synthesize missing embedded audio");
        let error = error_debug(error);

        assert!(
            error.contains("missing embedded audio payload 'MUSIC_ROUTE_29'")
                || error.contains("missing embedded PCM audio payload MUSIC_ROUTE_29")
                || error.contains("missing compiled audio payload MUSIC_ROUTE_29")
                || error.contains("missing embedded payload")
                || error.contains("missing payload for definitive asset 'MUSIC_ROUTE_29'"),
            "{error}"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_bootstrap_rejects_undeclared_embedded_audio_payloads() {
        let root = temp_repository_root("undeclared-embedded-audio");
        let asset_root = AssetRoot::new(&root);
        let mut data = minimal_runtime_data();
        data.audio = vec![
            ModpackAudioAsset::music(
                "MUSIC_ROUTE_29",
                "content-packs/test/music/MUSIC_ROUTE_29.pcm",
            )
            .expect("music asset"),
        ];
        let compiled_audio = [
            (
                "MUSIC_ROUTE_29".to_string(),
                vec![0_u8; 4],
            ),
            (
                "MUSIC_UNDECLARED".to_string(),
                vec![0_u8; 4],
            ),
        ]
        .into_iter()
        .collect();
        let direct_error = RuntimeAudioCatalog::from_game_data(
            &data,
            &compiled_audio,
            ModpackAudioManifest::default(),
            ModpackAudioPlaybackPlan::default(),
        )
        .expect_err("runtime audio catalog must reject undeclared embedded audio")
        .to_string();
        assert!(
            direct_error.contains(
                "runtime embedded audio payload MUSIC_UNDECLARED is not declared by compiled pack data"
            ),
            "{direct_error}"
        );
        let pack =
            CompiledGamePack::new_unchecked_with_audio_for_tests(data, compiled_audio, report());

        let error = CrystalRuntime::from_compiled_pack(&asset_root, pack, identity())
            .expect_err("runtime must reject undeclared embedded audio")
            .to_string();

        assert!(
            error.contains(
                "embedded audio payload 'MUSIC_UNDECLARED' not declared by pack data"
            ) || error.contains(
                "runtime embedded audio payload MUSIC_UNDECLARED is not declared by compiled pack data"
            ) || error.contains(
                "compiled audio payload 'MUSIC_UNDECLARED' is not declared by the definitive modpack"
            ),
            "{error}"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_bootstrap_rejects_empty_clean_game_data() {
        let root = temp_repository_root("empty-game");
        let asset_root = AssetRoot::new(&root);
        let pack = CompiledGamePack::new_unchecked_for_tests(GameDataSet::default(), report());

        let error = CrystalRuntime::from_compiled_pack(&asset_root, pack, identity())
            .expect_err("runtime must not boot a clean report with no game data")
            .to_string();

        assert!(error.contains("no Pokemon species data"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_bootstrap_allows_warning_diagnostics() {
        let root = temp_repository_root("verify-warning");
        let asset_root = AssetRoot::new(&root);
        let report = ModpackCompileReport {
            manifests: vec!["core-modular".to_string()],
            diagnostics: vec![VerificationError {
                severity: VerificationSeverity::Warning,
                code: "warning_pack".to_string(),
                subject: "runtime".to_string(),
                message: "pack has an unresolved warning".to_string(),
            }],
            ..ModpackCompileReport::default()
        };
        let pack = CompiledGamePack::new_unchecked_for_tests(minimal_runtime_data(), report);

        CrystalRuntime::from_compiled_pack(&asset_root, pack, identity())
            .expect("runtime should boot warning-only compiled packs");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_bootstrap_rejects_pack_with_verification_errors() {
        let root = temp_repository_root("verify");
        let asset_root = AssetRoot::new(&root);
        let report = ModpackCompileReport {
            manifests: vec!["core-modular".to_string()],
            diagnostics: vec![
                VerificationError {
                    severity: VerificationSeverity::Warning,
                    code: "warning_pack".to_string(),
                    subject: "runtime".to_string(),
                    message: "pack has an unresolved warning".to_string(),
                },
                VerificationError {
                    severity: VerificationSeverity::Error,
                    code: "bad_pack".to_string(),
                    subject: "runtime".to_string(),
                    message: "pack failed verification".to_string(),
                },
            ],
            ..ModpackCompileReport::default()
        };
        let pack = CompiledGamePack::new_unchecked_for_tests(minimal_runtime_data(), report);

        let error = CrystalRuntime::from_compiled_pack(&asset_root, pack, identity())
            .expect_err("runtime must reject diagnostic-bearing compiled packs");
        let error = error_debug(error);

        assert!(error.contains("compiled game pack is not verified for runtime"));
        assert!(!error.contains("warning_pack"));
        assert!(error.contains("bad_pack"));
        let _ = std::fs::remove_dir_all(root);
    }

fn runtime_stone_table_save_fixture(
    source_script: &str,
    compiled_target: &str,
    saved_target: &str,
    global: bool,
) -> (GameDataSet, GameState) {
    let mut scripts = BTreeMap::new();
    scripts.insert(
        source_script.to_string(),
        serde_json::json!([
            {"command": "stonetable", "args": ["5", "RUNTIME_BOULDER", compiled_target]},
            {"command": "db", "args": ["-1"]}
        ]),
    );
    for label in [
        ".Boulder",
        ".Boulder@QueueCallback",
        ".OtherBoulder@QueueCallback",
        ".Boulder@OtherCallback",
        "GlobalBoulder",
        "OtherGlobalBoulder",
    ] {
        scripts.insert(label.to_string(), serde_json::json!([]));
    }
    let mut data = GameDataSet::default();
    if global {
        data.global_scripts = Some(crystal_assets::GlobalScriptModule {
            definitions: scripts.clone(),
            scripts,
            ..Default::default()
        });
    } else {
        let mut map = runtime_map();
        map.scripts = scripts;
        data.maps.insert("RuntimeMap".to_string(), map);
    }
    let mut state = GameState::default();
    state
        .script_runtime
        .memory
        .insert("wCmdQueueType0".to_string(), "2".to_string());
    state
        .script_runtime
        .stone_table_entries
        .push(ScriptRuntimeStoneTableEntry {
            queue_slot: 0,
            warp: 5,
            object_event: "RUNTIME_BOULDER".to_string(),
            script: saved_target.to_string(),
            source_script: source_script.to_string(),
            command_index: 0,
        });
    (data, state)
}

#[test]
fn runtime_stone_table_save_references_accept_exact_resolved_targets() {
    for global in [false, true] {
        for (source, compiled, saved) in [
            (
                ".StoneTable@QueueCallback",
                ".Boulder",
                ".Boulder@QueueCallback",
            ),
            (
                ".StoneTable@QueueCallback",
                ".Boulder@QueueCallback",
                ".Boulder@QueueCallback",
            ),
            ("QueueCallback", ".Boulder", ".Boulder@QueueCallback"),
            (
                ".StoneTable@QueueCallback",
                "GlobalBoulder",
                "GlobalBoulder",
            ),
        ] {
            let (data, state) = runtime_stone_table_save_fixture(source, compiled, saved, global);
            state
                .validate_saved_state()
                .expect("valid stone-table state shape");
            validate_saved_script_runtime_references(&data, &state)
                .expect("runtime save validation accepts the source-resolved target");
        }
    }
    let (data, state) = runtime_stone_table_save_fixture(
        ".StoneTable@QueueCallback",
        ".Boulder",
        ".Boulder@QueueCallback",
        false,
    );
    let entry = &state.script_runtime.stone_table_entries[0];
    let (command, args) = crystal_core::state::saved_stone_table_entry_command_payload(entry);
    assert!(
        data.validate_saved_script_command_payload_reference(
            "generic.payload",
            &entry.source_script,
            0,
            command,
            &args,
        )
        .is_err(),
        "generic payload validation must retain exact argument comparison"
    );
}

#[test]
fn runtime_stone_table_save_references_reject_forged_entries() {
    let (data, state) = runtime_stone_table_save_fixture(
        ".StoneTable@QueueCallback",
        ".Boulder",
        ".Boulder@QueueCallback",
        false,
    );
    // All but the last two are existing labels, so existence alone is insufficient.
    for target in [
        ".OtherBoulder@QueueCallback",
        ".Boulder@OtherCallback",
        "GlobalBoulder",
        ".Boulder",
        ".Missing@QueueCallback",
        ".Boulder@@QueueCallback",
    ] {
        let mut changed = state.clone();
        changed.script_runtime.stone_table_entries[0].script = target.to_string();
        assert!(
            validate_saved_script_runtime_references(&data, &changed).is_err(),
            "must reject substituted target {target}"
        );
    }
    for (field, value) in [
        ("warp", "6"),
        ("object_event", "OTHER_BOULDER"),
        ("source_script", "MissingTable"),
        ("command_index", "1"),
        ("command_index", "9"),
    ] {
        let mut changed = state.clone();
        let entry = &mut changed.script_runtime.stone_table_entries[0];
        match field {
            "warp" => entry.warp = value.parse().expect("test warp"),
            "object_event" => entry.object_event = value.to_string(),
            "source_script" => entry.source_script = value.to_string(),
            "command_index" => entry.command_index = value.parse().expect("test index"),
            _ => unreachable!(),
        }
        assert!(
            validate_saved_script_runtime_references(&data, &changed).is_err(),
            "must reject changed {field}"
        );
    }
}

#[test]
fn runtime_stone_table_save_references_reject_invalid_compiled_targets() {
    for (compiled, saved) in [
        (".Boulder@OtherCallback", ".Boulder@OtherCallback"),
        (".Missing", ".Boulder@QueueCallback"),
        (".Boulder@@QueueCallback", ".Boulder@QueueCallback"),
    ] {
        let (data, state) =
            runtime_stone_table_save_fixture(".StoneTable@QueueCallback", compiled, saved, false);
        assert!(
            validate_saved_script_runtime_references(&data, &state).is_err(),
            "must reject unresolved or cross-parent compiled target {compiled}"
        );
    }
    let (data, state) = runtime_stone_table_save_fixture(
        ".StoneTable@QueueCallback",
        ".Boulder",
        ".Boulder@QueueCallback",
        false,
    );
    for command in [
        serde_json::json!({"command": "stonetable"}),
        serde_json::json!({"command": "stonetable", "args": "5, RUNTIME_BOULDER, .Boulder"}),
        serde_json::json!({"command": "stonetable", "args": ["5", "RUNTIME_BOULDER"]}),
        serde_json::json!({"command": "stonetable", "args": ["5", "RUNTIME_BOULDER", null]}),
        serde_json::json!({"command": "stonetable", "args": [5, "RUNTIME_BOULDER", ".Boulder"]}),
        serde_json::json!({"command": "stonetable", "args": ["5", "RUNTIME_BOULDER", ".Boulder", "extra"]}),
        serde_json::json!({"command": "noop", "args": ["5", "RUNTIME_BOULDER", ".Boulder"]}),
    ] {
        let mut changed = data.clone();
        changed
            .maps
            .get_mut("RuntimeMap")
            .expect("map")
            .scripts
            .get_mut(".StoneTable@QueueCallback")
            .expect("table")[0] = command;
        assert!(validate_saved_script_runtime_references(&changed, &state).is_err());
    }
    // A globally known target in another map cannot satisfy a map-local table.
    let mut changed = data;
    let target = changed
        .maps
        .get_mut("RuntimeMap")
        .expect("map")
        .scripts
        .remove(".Boulder@QueueCallback")
        .expect("target");
    let mut other_map = runtime_map();
    other_map
        .scripts
        .insert(".Boulder@QueueCallback".to_string(), target);
    changed.maps.insert("OtherMap".to_string(), other_map);
    assert!(validate_saved_script_runtime_references(&changed, &state).is_err());
}
