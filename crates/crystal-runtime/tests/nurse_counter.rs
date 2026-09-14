use crystal_assets::AssetRoot;
use crystal_core::world::map::{Direction, TilePosition};
use crystal_runtime::{CrystalRuntime, RuntimeGameShell};

fn runtime() -> (AssetRoot, CrystalRuntime) {
    let pack = std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external game pack");
    let root = AssetRoot::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(std::path::Path::new(&pack)).unwrap();
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).unwrap();
    (root, runtime)
}

#[test]
#[ignore = "requires the external Crystal pack"]
fn walking_from_center_entrance_loads_nurse() {
    let (_, runtime) = runtime();
    let mut session = runtime.data().overworld_session("VioletPokecenter1F", TilePosition::new(3,7), 0).unwrap();
    for _ in 0..8 {
        if session.player.tile.y == 3 { break; }
        session.step_checked(Direction::Up, Default::default()).unwrap();
    }
    assert_eq!(session.player.tile, TilePosition::new(3,3));
    let interaction = session.check_interaction_checked(1).unwrap();
    assert!(interaction.as_ref().is_some_and(|i| i.script == "VioletPokecenterNurse"), "{interaction:?}");
}

#[test]
#[ignore = "requires the external Crystal pack and recorded nurse save"]
fn saving_after_npc_activation_preserves_the_live_roster() {
    let (root, runtime) = runtime();
    let save = std::env::var_os("FLYGON_NURSE_TEST_SAVE").expect("recorded save");
    let mut shell = RuntimeGameShell::resume_from_save(root, runtime, std::path::Path::new(&save)).unwrap();
    let world = shell.session_mut().overworld_mut();
    // Traverse the viewport edge, which activates the nurse through the normal
    // roster mechanism. The original save omitted her despite showing her.
    let path = [4,5,6,7,6,5,4,3].map(|y| TilePosition::new(3,y));
    world.advance_object_struct_roster_along_player_path(&path).unwrap();
    world.player.facing = Direction::Up;
    shell.tick([]).unwrap();
    let before = shell.current_overworld_interaction_checked().unwrap().expect("live nurse interaction");
    assert_eq!(before.script, "VioletPokecenterNurse");
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("geothite-nurse-{stamp}.crystalsave"));
    shell.save(&path).unwrap();
    let persisted = shell.runtime().load_save(&path).unwrap();
    assert!(persisted.map_object_overrides["VioletPokecenter1F"].object_structs.structs.iter()
        .any(|s| s.map_object_index == 1), "The save itself must contain the nurse, independent of Continue recovery");
    shell.load(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert_eq!(shell.session().overworld().player.facing, Direction::Up);
    assert_eq!(shell.session().overworld().player.tile, TilePosition::new(3,3));
    let after = shell.current_overworld_interaction_checked().unwrap();
    assert!(after.as_ref().is_some_and(|i| i.script == before.script), "Live NPC lost across save/load: {after:?}");
}

#[test]
#[ignore = "requires the external Crystal pack and recorded nurse save"]
fn continue_activates_nurse_omitted_by_older_save() {
    let (root, runtime) = runtime();
    let save = std::env::var_os("FLYGON_NURSE_TEST_SAVE").expect("recorded save");
    let original = runtime.load_save(std::path::Path::new(&save)).unwrap();
    let prior_slots = original.map_object_overrides["VioletPokecenter1F"].object_structs.structs.clone();
    let mut shell = RuntimeGameShell::resume_from_save(root, runtime, std::path::Path::new(&save)).unwrap();
    shell.session_mut().overworld_mut().player.facing = Direction::Up;
    let interaction = shell.current_overworld_interaction_checked().unwrap();
    assert!(interaction.as_ref().is_some_and(|i| i.script == "VioletPokecenterNurse"), "{interaction:?}");
    let restored = shell.session().overworld().object_struct_roster_memory().unwrap();
    for prior in prior_slots {
        assert!(restored.structs.iter().any(|s| s.slot == prior.slot && s.map_object_index == prior.map_object_index),
            "Continue must retain existing NPC slot assignments");
    }
}

#[test]
#[ignore = "requires the external Crystal pack and recorded nurse save"]
fn continue_does_not_reactivate_explicitly_hidden_nurse() {
    let (root, runtime) = runtime();
    let source = std::env::var_os("FLYGON_NURSE_TEST_SAVE").expect("recorded save");
    let mut state = runtime.load_save(std::path::Path::new(&source)).unwrap();
    state.map_object_overrides.get_mut("VioletPokecenter1F").unwrap()
        .hidden_object_identifiers.insert("VIOLETPOKECENTER1F_NURSE".into());
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("geothite-hidden-nurse-{stamp}.crystalsave"));
    runtime.save_game(&path, state).unwrap();
    let mut shell = RuntimeGameShell::resume_from_save(root, runtime, &path).unwrap();
    std::fs::remove_file(path).unwrap();
    shell.session_mut().overworld_mut().player.facing = Direction::Up;
    assert!(shell.current_overworld_interaction_checked().unwrap().is_none());
}
