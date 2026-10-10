include!("tests/shell_basics.rs");
include!("tests/multiplayer_rendering.rs");
include!("tests/overworld_rendering.rs");
include!("tests/intro_title_rendering.rs");
include!("tests/title_flow.rs");
include!("tests/runtime_surfaces.rs");
include!("tests/new_game_flow.rs");
include!("tests/integrated_menus.rs");
include!("tests/audio_and_battle_ui.rs");
include!("tests/menu_and_input.rs");
include!("tests/story_progression.rs");
include!("tests/quests.rs");
include!("tests/art_text_and_determinism.rs");
include!("tests/battle_render_regressions.rs");
include!("tests/battle_sliding_intro.rs");
include!("tests/shop_rendering.rs");
include!("tests/heal_machine_rendering.rs");
include!("tests/town_map_rendering.rs");

#[test]
fn flygon_observation_reports_pack_exits_and_inventory_without_mutating_game() {
    let shell = core_modular_title_shell_for_test();
    let before = shell.shell.session().state().clone();
    let observed = super::webmcp_observation(&shell, None).unwrap();
    let snapshot = shell.shell.snapshot().unwrap();
    let module = shell.runtime.data().map_module(&snapshot.overworld.map_name).unwrap();
    assert_eq!(observed["map_info"]["curriculum_exits"].as_array().unwrap().len(),
        module.events.warps.len() + module.attributes.connections.len());
    for warp in &module.events.warps {
        assert!(observed["map_info"]["curriculum_exits"].as_array().unwrap().iter().any(|e|
            e["target"] == warp.target_map && e["x"] == warp.x && e["y"] == warp.y));
    }
    assert_eq!(observed["reward_state"]["items"].as_array().unwrap().len(),
        snapshot.bag.items.len() + snapshot.bag.balls.len());
    assert_eq!(shell.shell.session().state(), &before);
}

#[test]
fn flygon_observation_reports_real_battle_health_and_foreground_menu() {
    let mut controller = VisibleShellController {
        shell: route36_battle_shell_for_render_regression(),
    };
    for _ in 0..64 {
        let v = super::webmcp_observation(&controller.shell, None).unwrap();
        if v["observe"]["menus"].as_array().unwrap().iter().any(|m|m["surface"]=="commands") {
            let snapshot = controller.snapshot().unwrap();
            let battle = snapshot.battle.as_ref().unwrap();
            assert_eq!(v["reward_state"]["battle"]["enemy_max_hp"], battle.enemy_pokemon.max_hp);
            assert_eq!(v["reward_state"]["battle"]["active_player"], serde_json::json!(battle.active_player_party_index));
            assert!(v["observe"]["battle"].as_str().unwrap().contains("Wild"));
            return;
        }
        controller.press(GameButton::A).unwrap();
    }
    panic!("real battle did not expose its command menu");
}

#[test]
fn flygon_observation_exposes_moms_authored_weekday_choice() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
    let assets = AssetRoot::new(repo);
    let runtime = workspace_desktop_runtime(&assets);
    let mut controller = VisibleShellController::new_game(assets, runtime, "CHRIS", None).unwrap();
    for (button,count) in [(GameButton::Right,4),(GameButton::Up,3),(GameButton::Down,4)] {
        for _ in 0..count {
            let before=controller.snapshot().unwrap().overworld;
            for _ in 0..4 {
                controller.press(button).unwrap();
                let after=controller.snapshot().unwrap().overworld;
                if before.map_name!=after.map_name || before.tile!=after.tile {break;}
            }
        }
    }
    for _ in 0..96 {
        if controller.shell.pending_day_of_week.is_some() {
            let v = super::webmcp_observation(&controller.shell,None).unwrap();
            assert!(v["observe"]["menus"].as_array().unwrap().iter().any(|m|m["kind"]=="weekday"),
                "a real weekday choice must not be encoded as confirmation-only dialogue");
            return;
        }
        controller.press(GameButton::A).unwrap();
    }
    panic!("Mom's weekday choice was not reached");
}

#[test]
fn script_earthquake_shakes_then_sleeps_for_both_low_six_bit_counters() {
    let mut earthquake = super::VisibleEarthquake::from_script(84, 20, 20);
    assert_eq!(earthquake.intensity, 2);
    assert_eq!(earthquake.frames_remaining, 40);
    assert_eq!(earthquake.shake_frames_remaining, 20);

    earthquake.advance(20);
    assert_eq!(earthquake.frames_remaining, 20);
    assert_eq!(earthquake.shake_frames_remaining, 0);

    earthquake.advance(20);
    assert_eq!(earthquake.frames_remaining, 0);

    let wrapped = super::VisibleEarthquake::from_script(0, 256, 256);
    assert_eq!(wrapped.frames_remaining, 512);
    assert_eq!(wrapped.shake_frames_remaining, 256);
}

#[test]
fn overworld_screen_shake_uses_the_vertical_source_counter_sequence() {
    let mut earthquake = super::VisibleEarthquake::from_script(84, 20, 20);

    assert_eq!(
        super::visible_earthquake_camera_offset(Some(earthquake)),
        (0.0, 8.0),
        "remaining counter 19 applies -2 to SCY, which projects to +8 in Bevy's upward Y axis"
    );
    earthquake.advance(1);
    assert_eq!(
        super::visible_earthquake_camera_offset(Some(earthquake)),
        (0.0, -8.0),
        "remaining counter 18 applies +2 to SCY"
    );
    earthquake.advance(18);
    assert_eq!(earthquake.shake_frames_remaining, 1);
    assert_eq!(
        super::visible_earthquake_camera_offset(Some(earthquake)),
        (0.0, 0.0),
        "StepFunction_ScreenShake restores the baseline before deleting on its final update"
    );
    earthquake.advance(1);
    assert_eq!(earthquake.shake_frames_remaining, 0);
    assert_eq!(
        super::visible_earthquake_camera_offset(Some(earthquake)),
        (0.0, 0.0),
        "the generated step_sleep phase must not continue shaking"
    );
}

#[test]
fn poison_step_uses_four_opaque_bg_palette_frames_then_one_restored_frame() {
    let poison = [230.0 / 255.0, 173.0 / 255.0, 1.0, 1.0];
    for frames_remaining in [5, 4, 3, 2] {
        assert_eq!(
            super::visible_poison_bg_palette_rgba(frames_remaining),
            Some(poison),
            "LoadPoisonBGPals keeps every CGB background color at the poison color"
        );
    }
    assert_eq!(
        super::visible_poison_bg_palette_rgba(1),
        None,
        "the trailing DelayFrame runs after _UpdateTimePals restores the map palette"
    );
    assert_eq!(super::visible_poison_bg_palette_rgba(0), None);
}

#[test]
fn egg_hatch_wobble_uses_exact_asm_pairs_and_crack_boundaries() {
    assert_eq!(visible_egg_wobble_x(0), -2);
    assert_eq!(visible_egg_wobble_x(2), -2);
    assert_eq!(visible_egg_wobble_x(3), 2);
    assert_eq!(visible_egg_wobble_x(5), 2);
    assert_eq!(visible_egg_wobble_x(6), 0);
    assert_eq!(visible_egg_wobble_x(21), 0);
    assert_eq!(visible_egg_wobble_x(22), -2);
    assert_eq!(visible_egg_wobble_x(343), 0);
    assert_eq!(
        (0..344)
            .filter(|frame| visible_egg_crack_at(*frame))
            .collect::<Vec<_>>(),
        vec![50, 124, 222]
    );
}

include!("tests/webmcp.rs");

// Optional visual-reference files are read only by the tests that use them.
// Their absence must not prevent unrelated gameplay regressions from compiling.
fn external_oracle_fixture_bytes(name: &str) -> Vec<u8> {
    let root = std::env::var_os("GEOTHITE_ORACLE_FIXTURES")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../tools/asm-oracle/fixtures")
        });
    let path = root.join(name);
    std::fs::read(&path).unwrap_or_else(|error| panic!("Visual reference {} unavailable ({error}); set GEOTHITE_ORACLE_FIXTURES to the reference directory", path.display()))
}
fn external_oracle_fixture_text(name: &str) -> String {
    String::from_utf8(external_oracle_fixture_bytes(name)).expect("UTF-8 visual reference")
}

include!("tests/runtime_frontend_smoke.rs");
