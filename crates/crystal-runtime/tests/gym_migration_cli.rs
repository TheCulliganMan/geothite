#![cfg(feature = "test-fixtures")]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_runtime::{CrystalRuntime, RuntimeGameShell};

#[test]
#[ignore = "requires CRYSTAL_GYM_MIGRATION_SOURCE and CRYSTAL_GYM_MIGRATION_TARGET"]
fn command_migrates_one_save_without_changing_the_original() {
    let source_pack =
        PathBuf::from(std::env::var_os("CRYSTAL_GYM_MIGRATION_SOURCE").expect("source pack"));
    let target_pack =
        PathBuf::from(std::env::var_os("CRYSTAL_GYM_MIGRATION_TARGET").expect("target pack"));
    let root = AssetRoot::new(source_pack.parent().expect("pack directory"));
    let load = |path: &PathBuf| {
        CrystalRuntime::from_loaded_compiled_pack(
            &root,
            read_loaded_verified_compiled_game_pack(path).expect("verified pack"),
        )
        .expect("runtime")
    };
    let source = load(&source_pack);
    let target = load(&target_pack);
    let mut game = RuntimeGameShell::new_game_at_runtime_tile(
        root.clone(),
        source.clone(),
        source.title_new_game_spawn_identifier().expect("spawn"),
        "MnRochesterGym",
        6,
        17,
    )
    .expect("local gym fixture");
    game.session_mut().state_mut().money = 4321;
    let dir = std::env::temp_dir().join(format!("geothite-gym-cli-test-{}", std::process::id()));
    fs::create_dir(&dir).expect("fresh test directory");
    let old = dir.join("original.crystalsave");
    let new = dir.join("review.crystalsave");
    source
        .save_game(&old, game.session().state().clone())
        .expect("source fixture");
    let original = fs::read(&old).expect("original bytes");
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_migrate_gym_review_save"))
            .args([&source_pack, &target_pack, &old, &new])
            .output()
            .expect("run migration command")
    };
    let output = run();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read(&old).expect("original preserved"), original);
    let migrated = target.load_save(&new).expect("target save");
    assert_eq!(migrated.money, 4321);
    let continued =
        RuntimeGameShell::resume_from_save(root, target, &new).expect("resume target save");
    assert_eq!(continued.session().state().money, 4321);
    let bytes = fs::read(&new).expect("target bytes");
    assert!(!run().status.success());
    assert_eq!(fs::read(&new).expect("target preserved"), bytes);
    fs::remove_dir_all(dir).expect("remove test saves");
}
