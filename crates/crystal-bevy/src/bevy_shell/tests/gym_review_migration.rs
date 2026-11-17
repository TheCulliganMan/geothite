//! Production-input save migration from an earned source-gym trainer battle.
use std::path::PathBuf;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_runtime::{CrystalRuntime, migrate_gym_review_save};

use super::{
    VisibleShellController, at, battle, controller, event, exit_and_reenter, finish, interact,
    load, point, press, walk_to,
};

#[test]
#[ignore = "requires CRYSTAL_GYM_MIGRATION_SOURCE and CRYSTAL_GYM_MIGRATION_TARGET"]
fn gym_review_migration_earned_trainer_save_continues_and_finishes() {
    let source_path =
        PathBuf::from(std::env::var_os("CRYSTAL_GYM_MIGRATION_SOURCE").expect("source pack"));
    let target_path =
        PathBuf::from(std::env::var_os("CRYSTAL_GYM_MIGRATION_TARGET").expect("target pack"));
    let root = AssetRoot::new(source_path.parent().expect("pack parent"));
    let source = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&source_path).expect("verified source"),
    )
    .expect("source runtime");
    let target = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&target_path).expect("verified target"),
    )
    .expect("target runtime");
    // The shared loader reads the authored historical gym source. Pack path is
    // the verified source; Mew100 and the opening prerequisite are fixtures.
    let (_, _, historical) = load();
    let mut old = controller(&root, &source, &historical["gyms"][0], false);
    interact(
        &mut old,
        &source,
        point(&historical["gyms"][0]["trainers"][0]["at"]),
    );
    battle(&mut old, false);
    finish(&mut old);
    assert!(event(&old, "EVENT_MN_GYM_1_TRAINER_1_DEFEATED"));
    assert!(!event(&old, "EVENT_MN_GYM_1_TRAINER_2_DEFEATED"));
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let old_path = std::env::temp_dir().join(format!(
        "gym-migration-old-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    let new_path = std::env::temp_dir().join(format!(
        "gym-migration-new-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    old.save(&old_path).expect("source save");
    let original_bytes = std::fs::read(&old_path).expect("source bytes");
    let saved = source
        .load_save(&old_path)
        .expect("source identity verification");
    let migrated = migrate_gym_review_save(&source, &target, &root, &saved)
        .expect("migrate earned source save");
    assert_eq!(migrated.flags, saved.flags);
    assert_eq!(migrated.storage, saved.storage);
    assert_eq!(migrated.bag, saved.bag);
    assert_eq!(migrated.badges, saved.badges);
    target
        .save_game(&new_path, migrated)
        .expect("write separate test save");
    assert_eq!(
        std::fs::read(&old_path).expect("source remains"),
        original_bytes
    );
    let mut c =
        VisibleShellController::load_save(root.clone(), target.clone(), new_path.clone(), None)
            .expect("target Continue");
    assert_eq!(at(&mut c), (6, 17));
    assert!(event(&c, "EVENT_MN_GYM_1_TRAINER_1_DEFEATED"));
    press(&mut c, GameButton::Start);
    assert!(
        super::super::visible_start_menu_entries(&c.shell).is_ok(),
        "Start remains responsive"
    );
    press(&mut c, GameButton::B);
    let revised: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../planning/minnesota/gyms/rochester-training-hall-v01.json"
    ))
    .expect("revised gym");
    let gym = &revised["gyms"][0];
    walk_to(&mut c, &target, (6, 16));
    interact(&mut c, &target, point(&gym["trainers"][1]["at"]));
    battle(&mut c, false);
    finish(&mut c);
    interact(&mut c, &target, point(&gym["leader_at"]));
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_1_DEFEATED"));
    assert!(c.shell.shell.session().state().badges.johto[0]);
    for carpet in gym["exit_tiles_core"].as_array().expect("both exits") {
        exit_and_reenter(&mut c, &target, gym, point(carpet));
    }
    std::fs::remove_file(old_path).expect("remove source fixture");
    std::fs::remove_file(new_path).expect("remove target fixture");
}
