//! An art extension must preserve the supplied game's maps and source content.
use std::path::PathBuf;

use crystal_assets::{build_gunflint_art_pack, read_verified_compiled_game_pack};

#[test]
fn gunflint_art_preserves_source_content_and_forest_bank_addressing() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../content-packs/text-tui.crystalpack")
        .canonicalize()
        .expect("local source pack path");
    let base = read_verified_compiled_game_pack(path).expect("local source pack");
    let result = build_gunflint_art_pack(&base).expect("native forest extension");
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
        base.audio_manifest().expect("source audio")
    );
    for (path, bytes) in base.runtime_files() {
        assert_eq!(
            result.runtime_files().get(path),
            Some(bytes),
            "changed {path}"
        );
    }
    for (id, definition) in &base.data().tilesets {
        assert_eq!(
            result.data().tilesets.get(id),
            Some(definition),
            "changed {id}"
        );
    }
    let source = &base.data().tilesets["forest"];
    let derived = &result.data().tilesets["mn_gunflint"];
    let original = &base.runtime_files()["data/tilesets/forest_metatiles.bin"];
    let extended = &result.runtime_files()["data/tilesets/mn_gunflint_metatiles.bin"];
    let original_pixels = &base.runtime_files()["gfx/tilesets/forest.2bpp"];
    let extended_pixels = &result.runtime_files()["gfx/tilesets/mn_gunflint.2bpp"];
    assert_eq!(
        extended_pixels.len(),
        original_pixels.len(),
        "bank midpoint moved"
    );
    assert_eq!(&extended[..original.len()], original);
    assert_eq!(extended.len(), 58 * 16);
    assert_eq!(derived.collision.len(), 58);
    // Original art uses banked physical glyphs as well as its logical IDs.
    // Check the physically addressed bytes, not merely the metatile prefix.
    for &logical in original {
        let palette = source.palette_map[usize::from(logical)];
        assert_eq!(derived.palette_map[usize::from(logical)], palette);
        let physical = if palette & 8 != 0 {
            usize::from(logical & 127) + original_pixels.len() / 32
        } else {
            usize::from(logical)
        };
        assert_eq!(
            &extended_pixels[physical * 16..(physical + 1) * 16],
            &original_pixels[physical * 16..(physical + 1) * 16]
        );
    }
    for id in 0..40 {
        let key = format!("{id:02x}");
        assert_eq!(derived.collision[&key], source.collision[&key]);
    }
    assert!(
        build_gunflint_art_pack(&result).is_err(),
        "duplicate extension accepted"
    );
}
