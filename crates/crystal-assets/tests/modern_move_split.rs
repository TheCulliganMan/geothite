use crystal_assets::{build_modern_move_split_modpack, read_verified_compiled_game_pack};
use crystal_core::battle::damage::MoveCategory;

#[test]
fn modern_move_split_pack_roundtrips_and_preserves_real_species_stats() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../content-packs/core-modular.browser.crystalpack")
        .canonicalize()
        .unwrap();
    let base = read_verified_compiled_game_pack(path).unwrap();
    let pack = build_modern_move_split_modpack(&base).unwrap();
    assert_ne!(pack.identity().unwrap(), base.identity().unwrap());
    assert_eq!(pack.data().type_categories.moves.len(), 251);
    assert_eq!(
        pack.data().type_categories.moves["THUNDERPUNCH"],
        MoveCategory::Physical
    );
    assert_eq!(
        pack.data().type_categories.moves["SHADOW_BALL"],
        MoveCategory::Special
    );
    assert!(build_modern_move_split_modpack(&pack).is_err());
    assert_eq!(
        pack.identity().unwrap(),
        build_modern_move_split_modpack(&base)
            .unwrap()
            .identity()
            .unwrap()
    );
    let mut unchanged = pack.data().clone();
    unchanged.type_categories.moves.clear();
    assert_eq!(&unchanged, base.data());
    let output = std::env::temp_dir().join(format!(
        "modern-move-split-{}.crystalpack",
        std::process::id()
    ));
    pack.write_preserving_storage(&output).unwrap();
    let reloaded = read_verified_compiled_game_pack(&output).unwrap();
    std::fs::remove_file(output).unwrap();
    assert_eq!(pack.identity().unwrap(), reloaded.identity().unwrap());
    assert_eq!(pack.data(), reloaded.data());
}
