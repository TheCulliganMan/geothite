use crystal_core::save::{
    SaveModpackIdentity, encode_save_game_bytes, erase_save_game, read_save_game_bytes_for_modpack,
    read_save_game_for_modpack, write_save_game_for_modpack,
};
use crystal_core::state::GameState;

#[test]
fn shared_save_preserves_progress_and_rejects_corruption_and_other_packs() {
    let identity =
        SaveModpackIdentity::from_compiled_pack_bytes("current-pack", b"current Rust pack")
            .unwrap();
    let hash = "01".repeat(32);
    let mut state = GameState::default();
    state.player_name = "NOVA".into();
    state.player_id = 23456;
    state.player_gender = 1;
    let path = std::env::temp_dir().join(format!(
        "geothite-sharing-{}.crystalsave",
        std::process::id()
    ));
    write_save_game_for_modpack(&path, state, &identity, &hash).unwrap();
    let save = read_save_game_for_modpack(&path, &identity, &hash).unwrap();
    erase_save_game(&path).unwrap();
    let bytes = encode_save_game_bytes(&save).unwrap();
    let imported =
        read_save_game_bytes_for_modpack(&bytes, "shared.crystalsave", &identity, &hash).unwrap();
    assert_eq!(imported.state(), save.state());
    let other =
        SaveModpackIdentity::from_compiled_pack_bytes("other-pack", b"other Rust pack").unwrap();
    assert!(read_save_game_bytes_for_modpack(&bytes, "shared.crystalsave", &other, &hash).is_err());
    let mut corrupt = bytes.clone();
    *corrupt.last_mut().unwrap() ^= 1;
    assert!(
        read_save_game_bytes_for_modpack(&corrupt, "shared.crystalsave", &identity, &hash).is_err()
    );
    assert!(
        read_save_game_bytes_for_modpack(
            &bytes[..bytes.len() / 2],
            "shared.crystalsave",
            &identity,
            &hash
        )
        .is_err()
    );
}
