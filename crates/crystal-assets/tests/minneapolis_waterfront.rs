//! Real external-pack bank/collision preservation, plus adversarial art contracts.
use std::path::PathBuf;

use crystal_assets::{CompiledGamePack, build_minneapolis_waterfront_pack};
use serde_json::Value;
use sha2::{Digest, Sha256};

const MAP: &str = "MnMinneapolis";
const TILESET: &str = "mn_minneapolis_city";
fn source() -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../planning/minnesota/revisions/minneapolis-waterfront-v01/source-v06/native.contract.json",
    )).expect("external reviewed Minneapolis waterfront contract")
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn base() -> CompiledGamePack {
    crystal_assets::read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_WATERFRONT_PARENT_PACK").expect("verified pine-knoll parent"),
    ))
    .expect("verified parent")
}
fn slot(glyph: u8, palette: u8, count: usize) -> usize {
    if palette & 8 == 0 {
        usize::from(glyph)
    } else {
        usize::from(glyph & 127) + count / 2
    }
}

#[test]
#[ignore = "requires CRYSTAL_WATERFRONT_PARENT_PACK and external source-v06/native.contract.json"]
fn waterfront_preserves_all_gameplay_and_native_logical_glyphs() {
    let base = base();
    let source = source();
    let output = build_minneapolis_waterfront_pack(&base, &source, &hash(source.as_bytes()))
        .expect("native waterfront art publication");
    assert_eq!(base.compiled_audio(), output.compiled_audio());
    assert_eq!(base.report(), output.report());
    let old = &base.data().maps[MAP];
    let new = &output.data().maps[MAP];
    assert_eq!(
        old.blocks
            .iter()
            .zip(&new.blocks)
            .filter(|(a, b)| a != b)
            .count(),
        16
    );
    let old_definition = &base.data().tilesets[TILESET];
    let new_definition = &output.data().tilesets[TILESET];
    assert_eq!(
        &new_definition.palette_map[..224],
        &old_definition.palette_map
    );
    for (&old, &new) in old.blocks.iter().zip(&new.blocks) {
        assert_eq!(
            old_definition.collision[&format!("{old:02x}")],
            new_definition.collision[&format!("{new:02x}")]
        );
    }
    let mut before = serde_json::to_value(base.data()).expect("original data");
    let mut after = serde_json::to_value(output.data()).expect("new data");
    for (table, key) in [
        ("maps", MAP),
        ("tilesets", TILESET),
        (
            "map_blocks",
            old.attributes
                .blocks_label
                .as_deref()
                .expect("native mirror"),
        ),
    ] {
        after[table][key] = before[table][key].take();
        before[table][key] = after[table][key].clone();
    }
    assert_eq!(
        before, after,
        "all rooms, programs, species, events, recovery and other maps exact"
    );
    let layout = &base.runtime_files()[&format!("data/tilesets/{TILESET}_metatiles.bin")];
    let before = &base.runtime_files()[&format!("gfx/tilesets/{TILESET}.2bpp")];
    let after = &output.runtime_files()[&format!("gfx/tilesets/{TILESET}.2bpp")];
    for &glyph in layout {
        let palette = old_definition.palette_map[usize::from(glyph)];
        let a = slot(glyph, palette, 192) * 16;
        let b = slot(glyph, palette, 256) * 16;
        assert_eq!(
            &before[a..a + 16],
            &after[b..b + 16],
            "logical glyph {glyph:02x}"
        );
    }
}

#[test]
#[ignore = "requires CRYSTAL_WATERFRONT_PARENT_PACK and external source-v06/native.contract.json"]
fn waterfront_rejects_collision_changes_occupied_glyphs_and_unreviewed_source() {
    let base = base();
    let source = source();
    assert!(build_minneapolis_waterfront_pack(&base, &source, &"0".repeat(64)).is_err());
    for (field, value) in [
        ("collision", Value::String("WATER".into())),
        ("target", Value::from(12)),
    ] {
        let mut source: Value = serde_json::from_str(&source).expect("source contract");
        if field == "collision" {
            source["blocks"][0]["collision"][0] = value;
        } else {
            source["glyph_copies"][0]["target"] = value;
        }
        let text = serde_json::to_string(&source).expect("adversarial contract");
        let error = build_minneapolis_waterfront_pack(&base, &text, &hash(text.as_bytes()))
            .expect_err("invalid native append accepted");
        assert!(
            !error.to_string().contains("missing field"),
            "schema rejection is not invariant proof"
        );
    }
}
