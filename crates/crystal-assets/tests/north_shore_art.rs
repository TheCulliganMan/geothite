//! Bounded art preparation must preserve the supplied game's content.
use std::path::PathBuf;

use crystal_assets::{build_north_shore_art_pack, read_verified_compiled_game_pack};

#[test]
fn north_shore_art_adds_only_a_bounded_source_waterfall_tileset() {
    let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../content-packs/text-tui.crystalpack")
        .canonicalize()
        .expect("local source pack path");
    let base = read_verified_compiled_game_pack(base_path).expect("local source pack");
    let result = build_north_shore_art_pack(&base).expect("typed source-art extension");
    assert_eq!(result.data().maps, base.data().maps);
    assert_eq!(
        result.data().runtime_spawn_points,
        base.data().runtime_spawn_points
    );
    assert_eq!(result.data().pokemon, base.data().pokemon);
    assert_eq!(result.data().trainers, base.data().trainers);
    assert_eq!(result.data().marts, base.data().marts);
    assert_eq!(result.compiled_audio(), base.compiled_audio());
    assert_eq!(
        result.audio_manifest().expect("derived audio"),
        base.audio_manifest().expect("base audio")
    );
    for (path, bytes) in base.runtime_files() {
        assert_eq!(
            result.runtime_files().get(path),
            Some(bytes),
            "changed {path}"
        );
    }
    for (id, tileset) in &base.data().tilesets {
        assert_eq!(
            result.data().tilesets.get(id),
            Some(tileset),
            "changed {id}"
        );
    }
    let definition = &result.data().tilesets["mn_north_shore"];
    assert_eq!(
        definition.collision.len(),
        base.data().tilesets["johto"].collision.len() + 1
    );
    assert_eq!(
        definition.collision["80"],
        base.data().tilesets["cave"].collision["2c"]
    );
    let original = &base.runtime_files()["data/tilesets/johto_metatiles.bin"];
    let derived = &result.runtime_files()["data/tilesets/mn_north_shore_metatiles.bin"];
    assert_eq!(&derived[..original.len()], original);
    assert_eq!(derived.len(), original.len() + 16);
    assert!(
        build_north_shore_art_pack(&result).is_err(),
        "duplicate tileset accepted"
    );
}
