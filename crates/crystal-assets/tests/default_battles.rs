use std::path::PathBuf;

use crystal_assets::{build_default_battle_modpack, read_verified_compiled_game_pack};

#[test]
fn default_battles_enable_trainers_and_wilds_preserving_content_and_audio() {
    let source = std::env::var_os("CRYSTAL_DEFAULT_BATTLE_BASE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content-packs/core-modular.browser.crystalpack")
        });
    let base = read_verified_compiled_game_pack(source.canonicalize().unwrap()).unwrap();
    let pack = build_default_battle_modpack(&base).unwrap();
    assert!(pack.data().competitive_battles);
    assert!(pack.data().wild_challenge.is_some());
    assert_eq!(pack.data().type_categories.moves.len(), 251);
    let mut unchanged = pack.data().clone();
    unchanged.competitive_battles = base.data().competitive_battles;
    unchanged.type_categories = base.data().type_categories.clone();
    unchanged.wild_challenge = base.data().wild_challenge;
    assert_eq!(&unchanged, base.data());
    assert_eq!(pack.runtime_files(), base.runtime_files());
    assert_eq!(pack.compiled_audio(), base.compiled_audio());
    // Native payloads can be gzip encoded. Compare those bytes above and the
    // declared PCM hashes, frame counts, and loop ranges here, without asking
    // the uncompressed-audio manifest helper to interpret gzip bytes as PCM.
    assert_eq!(pack.data().audio, base.data().audio);
    assert_eq!(
        build_default_battle_modpack(&pack)
            .unwrap()
            .identity()
            .unwrap(),
        pack.identity().unwrap()
    );
    let path = std::env::temp_dir().join(format!(
        "default-battles-test-{}.crystalpack",
        std::process::id()
    ));
    pack.write_preserving_storage(&path).unwrap();
    let loaded = read_verified_compiled_game_pack(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(loaded.data(), pack.data());
    assert_eq!(loaded.compiled_audio(), pack.compiled_audio());
    assert_eq!(loaded.runtime_files(), pack.runtime_files());
    assert_eq!(loaded.identity().unwrap(), pack.identity().unwrap());
}
