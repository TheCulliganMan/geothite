use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use crate::VisibleShellController;

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
