//! Derived forest art must keep the source animation pixels and schedules.
use std::path::PathBuf;

use bevy::asset::Assets;
use bevy::image::Image;
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_runtime::CrystalRuntime;

use super::super::load_tileset_art;

#[test]
#[ignore = "requires the external Gunflint art pack"]
fn gunflint_animations_keep_native_forest_frames_and_restless_tree_gate() {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_GUNFLINT_ART_TEST_PACK").expect("external Gunflint art pack"),
    );
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("art pack"),
    )
    .expect("art runtime");
    let root = runtime
        .materialize_runtime_files()
        .expect("temporary runtime mount");
    let mut images = Assets::<Image>::default();
    let mut load = |id: &str| {
        load_tileset_art(
            &root,
            id,
            "day",
            &runtime.data().tilesets[id].palette_map,
            &mut images,
        )
        .expect("native tileset")
    };
    let source = load("forest");
    let derived = load("mn_gunflint");
    assert_eq!(source.animated_tiles.len(), derived.animated_tiles.len());
    for tile in [0x03, 0x14, 0x0c, 0x0f] {
        let a = &derived.animated_tiles[&tile];
        let b = &source.animated_tiles[&tile];
        assert_eq!(a.frame_ticks, b.frame_ticks);
        assert_eq!(a.phase_offset, b.phase_offset);
        assert_eq!(a.advance_on_phase_offset, b.advance_on_phase_offset);
        assert_eq!(a.requires_forest_restless, b.requires_forest_restless);
        assert_eq!(a.cave_water_composite, b.cave_water_composite);
        assert_eq!(a.frames.len(), b.frames.len());
        for (actual, expected) in a.frames.iter().zip(b.frames.iter()) {
            assert_eq!(
                images.get(actual).expect("derived frame").data,
                images.get(expected).expect("source frame").data,
                "tile {tile:02x}"
            );
        }
    }
}
