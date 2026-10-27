use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use crate::VisibleShellController;

#[test]
fn renderer_neutral_controller_resumes_bedroom_with_movement_and_start() {
    let pack = env::var_os("CRYSTAL_RENDER_TEST_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content-packs/text-tui.crystalpack")
        });
    if !pack.is_file() && env::var_os("CRYSTAL_RENDER_TEST_PACK").is_none() {
        return;
    }
    let root = AssetRoot::new(pack.parent().expect("pack parent"));
    let loaded = read_loaded_verified_compiled_game_pack(&pack).expect("verified external pack");
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).expect("runtime");
    let mut controller =
        VisibleShellController::new_game(root.clone(), runtime.clone(), "CHRIS", None)
            .expect("new game");
    controller
        .wait_frames(12)
        .expect("settle bedroom presentation");
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let directory = env::temp_dir().join(format!(
        "geothite-bedroom-resume-{}-{unique}",
        std::process::id(),
    ));
    fs::create_dir(&directory).expect("create save fixture directory");
    let save = directory.join("checkpoint.crystalsave");
    controller.save(&save).expect("save bedroom");
    let result = VisibleShellController::load_save(root, runtime, save, None);
    fs::remove_dir_all(directory).expect("remove save fixture directory");
    let mut resumed = result.expect("resume bedroom");
    let before = resumed.snapshot().expect("resumed snapshot");
    assert_eq!(before.overworld.map_name, "PlayersHouse2F");
    assert_eq!((before.overworld.tile.x, before.overworld.tile.y), (3, 3));
    assert!(
        before.ui.menu.is_none(),
        "completed callback retained input ownership"
    );
    for _ in 0..4 {
        resumed
            .press_buttons([GameButton::B, GameButton::Right])
            .expect("walk after Continue");
        if resumed
            .snapshot()
            .expect("movement snapshot")
            .overworld
            .tile
            != before.overworld.tile
        {
            break;
        }
    }
    assert_eq!(
        resumed.snapshot().expect("moved snapshot").overworld.tile.x,
        4
    );
    resumed
        .press(GameButton::Start)
        .expect("open Start after Continue");
    let menu = resumed.snapshot().expect("Start snapshot");
    assert!(menu.ui.menu.is_some(), "production Start menu did not open");
}

fn assert_presentation_fields_match_integrity_snapshot(controller: &VisibleShellController) {
    let shell = &controller.shell.shell;
    let full = shell.snapshot().expect("validated integrity snapshot");
    assert_eq!(
        full.state_checksum,
        shell.state_checksum().expect("authoritative checksum")
    );
    let mut presentation = shell
        .presentation_snapshot()
        .expect("presentation snapshot");
    assert_eq!(
        presentation.state_checksum.frame(),
        full.state_checksum.frame()
    );
    assert_eq!(presentation.state_checksum.hash(), 0);
    assert_eq!(presentation.visual_state_hash, 0);
    presentation.state_checksum = full.state_checksum.clone();
    presentation.visual_state_hash = full.visual_state_hash;
    assert_eq!(presentation, full);
}

#[test]
fn renderer_neutral_controller_preserves_snapshot_fields_and_integrity_boundaries() {
    let pack = env::var_os("CRYSTAL_RENDER_TEST_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content-packs/text-tui.crystalpack")
        });
    if !pack.is_file() && env::var_os("CRYSTAL_RENDER_TEST_PACK").is_none() {
        return;
    }
    let root = AssetRoot::new(pack.parent().expect("pack parent"));
    let loaded = read_loaded_verified_compiled_game_pack(&pack).expect("verified external pack");
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).expect("runtime");
    let mut controller =
        VisibleShellController::new_game(root, runtime, "CHRIS", None).expect("new game");
    controller.set_runtime_journal_enabled(false);
    let initial = controller.snapshot().expect("initial snapshot");
    assert_presentation_fields_match_integrity_snapshot(&controller);

    for _ in 0..24 {
        controller.press(GameButton::Right).expect("walk right");
        assert_presentation_fields_match_integrity_snapshot(&controller);
    }
    let moved = controller.snapshot().expect("walking snapshot");
    assert_eq!(moved.overworld.map_name, initial.overworld.map_name);
    assert_ne!(moved.overworld.tile, initial.overworld.tile);
    controller
        .press(GameButton::Start)
        .expect("open Start menu");
    assert!(
        controller
            .snapshot()
            .expect("menu snapshot")
            .ui
            .menu
            .is_some()
    );
    assert_presentation_fields_match_integrity_snapshot(&controller);
    controller.press(GameButton::B).expect("close Start menu");
    assert_presentation_fields_match_integrity_snapshot(&controller);

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let directory = env::temp_dir().join(format!(
        "geothite-controller-integrity-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir(&directory).expect("create save fixture directory");
    let save = directory.join("checkpoint.crystalsave");
    controller.save(&save).expect("validated save");
    let bytes = fs::read(&save).expect("saved checkpoint");

    let previous_balance = controller.shell.shell.session().state().blue_card_balance;
    controller
        .shell
        .shell
        .session_mut()
        .state_mut()
        .blue_card_balance = 31;
    assert!(controller.shell.shell.snapshot().is_err());
    assert!(controller.shell.shell.state_checksum().is_err());
    assert!(controller.save(&save).is_err());
    assert_eq!(fs::read(&save).expect("preserved checkpoint"), bytes);
    controller
        .shell
        .shell
        .session_mut()
        .state_mut()
        .blue_card_balance = previous_balance;
    assert_presentation_fields_match_integrity_snapshot(&controller);
    fs::remove_dir_all(directory).expect("remove save fixture directory");
}

#[test]
fn renderer_neutral_controller_exposes_retained_level_stats_and_returns_to_walking() {
    let pack = env::var_os("CRYSTAL_RENDER_TEST_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content-packs/text-tui.crystalpack")
        })
        .canonicalize()
        .expect("external controller pack");
    let root = AssetRoot::new(pack.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(pack).expect("verified pack"),
    )
    .expect("runtime");
    let mut controller =
        VisibleShellController::new_game(root, runtime, "CHRIS", None).expect("new game");
    controller.wait_frames(12).expect("settled bedroom");
    // Isolate the native retained result surface after the core battle ended.
    // No party, rewards, maps or authoritative progression are injected.
    controller.shell.battle_exp_tween = Some(crate::bevy_shell::VisibleBattleExpTween {
        trigger_message: "TOTODILE gained\n40 EXP. Points!".into(),
        started: true,
        pixels: 10,
        level: 8,
        target_pixels: 30,
        remaining_targets: std::collections::VecDeque::new(),
        frames_until_step: 9,
        steps_in_segment: 0,
    });
    controller
        .shell
        .battle_level_stats
        .push_back(crate::bevy_shell::VisibleBattleLevelStats {
            trigger_message: "TOTODILE grew to\nlevel 9!".into(),
            triggered: true,
            active: false,
            frames_before_input: 0,
            attack: 17,
            defense: 18,
            speed: 14,
            special_attack: 13,
            special_defense: 16,
        });
    controller
        .wait_frames(1)
        .expect("settle native input delay");
    assert!(
        controller.shell.battle_exp_tween.is_none(),
        "EXP animation did not finish through the shared clock"
    );
    let visible = controller.snapshot().expect("retained result snapshot");
    let text = visible
        .ui
        .text
        .as_ref()
        .expect("level stats must be visible");
    assert_eq!(text.label, "visible-shell:level-up-stats");
    assert!(
        text.asm_text
            .as_ref()
            .is_some_and(|text| { text.contains("ATTACK 17") && text.contains("SPCL.DEF 16") })
    );
    controller
        .press(GameButton::Right)
        .expect("direction while stats own input");
    assert_eq!(
        controller
            .snapshot()
            .expect("blocked movement")
            .overworld
            .tile,
        visible.overworld.tile
    );
    controller
        .press(GameButton::A)
        .expect("acknowledge native result surface");
    assert!(controller.shell.battle_level_stats.is_empty());
    for _ in 0..4 {
        controller
            .press(GameButton::Right)
            .expect("walk after results");
        if controller.snapshot().expect("walking").overworld.tile != visible.overworld.tile {
            break;
        }
    }
    assert_eq!(
        controller
            .snapshot()
            .expect("returned walking")
            .overworld
            .tile
            .x,
        visible.overworld.tile.x + 1
    );
}
