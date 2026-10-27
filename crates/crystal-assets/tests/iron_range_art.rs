//! Native rail fidelity and preservation before Iron Range map publication.
use std::path::PathBuf;

use crystal_assets::{build_iron_range_art_pack, read_verified_compiled_game_pack};

#[test]
fn iron_range_rail_keeps_native_pixels_collision_and_existing_content() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../content-packs/text-tui.crystalpack")
        .canonicalize()
        .expect("external source pack");
    let base = read_verified_compiled_game_pack(path).expect("verified source");
    let result = build_iron_range_art_pack(&base).expect("native rail import");
    assert_eq!(result.data().maps, base.data().maps);
    assert_eq!(result.compiled_audio(), base.compiled_audio());
    for (path, bytes) in base.runtime_files() {
        assert_eq!(result.runtime_files().get(path), Some(bytes), "{path}");
    }
    let files = result.runtime_files();
    let original = &files["data/tilesets/mn_headwaters_metatiles.bin"];
    let blocks = &files["data/tilesets/mn_iron_range_metatiles.bin"];
    assert_eq!(blocks.len(), 133 * 16);
    assert_eq!(&blocks[..131 * 16], original);
    let glyph = |tileset: &str, glyph: u8| {
        let palette = result.data().tilesets[tileset].palette_map[usize::from(glyph)];
        let index = if palette & 8 == 0 {
            usize::from(glyph)
        } else {
            usize::from(glyph & 0x7f) + 96
        };
        let pixels = &files[&format!("gfx/tilesets/{tileset}.2bpp")];
        (&pixels[index * 16..(index + 1) * 16], palette & 7)
    };
    for &id in original {
        assert_eq!(glyph("mn_headwaters", id), glyph("mn_iron_range", id));
    }
    for (target, source, token) in [(0x83, 0x7e, "WALL"), (0x84, 0x7f, "FLOOR")] {
        for (&from, &to) in files["data/tilesets/johto_modern_metatiles.bin"]
            [source * 16..(source + 1) * 16]
            .iter()
            .zip(&blocks[target * 16..(target + 1) * 16])
        {
            assert_eq!(glyph("johto_modern", from), glyph("mn_iron_range", to));
        }
        assert!(
            result.data().tilesets["mn_iron_range"].collision[&format!("{target:02x}")]
                .iter()
                .all(|kind| kind == token)
        );
    }
    assert!(build_iron_range_art_pack(&result).is_err());
}
