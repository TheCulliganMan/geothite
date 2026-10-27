use std::fs;
use std::io::Read;
use std::path::PathBuf;

use anyhow::{Context, Result};
use crystal_assets::{MinnesotaTileCatalog, build_minnesota_tile_art_pack};
use flate2::read::ZlibDecoder;

fn inputs() -> Result<(MinnesotaTileCatalog, Vec<u8>)> {
    catalog_inputs("northwoods-v1")
}

fn catalog_inputs(name: &str) -> Result<(MinnesotaTileCatalog, Vec<u8>)> {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/tiles")
        .join(name);
    Ok((
        serde_json::from_slice(&fs::read(directory.join("catalog.json"))?)?,
        fs::read(directory.join("source.png"))?,
    ))
}

#[test]
fn campfire_preserves_cutout_alpha_and_separate_fire_and_stone_palettes() -> Result<()> {
    let (catalog, source) = catalog_inputs("campfire-v2")?;
    let frames = catalog.convert(&source)?;
    let item = &catalog.items[0];
    assert!(item.transparent_background);
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].indices[0], 0);
    let mut fire = false;
    let mut stone = false;
    for (n, &index) in frames[0].indices.iter().enumerate() {
        if index == 0 {
            continue;
        }
        let palette = item.palette_for_tile(n / 32 / 8 * 4 + n % 32 / 8);
        fire |= palette == 4 && (index == 1 || index == 2);
        stone |= palette == 0 && (index == 1 || index == 2);
    }
    assert!(
        fire && stone,
        "fire and gray stones must both retain visible highlights"
    );
    let preview = crystal_assets::minnesota_tile_item_preview(&frames[0], item, 1)?;
    assert_eq!(preview[25], 6, "cutout must be an RGBA PNG");
    Ok(())
}

#[test]
fn original_sheet_yields_six_distinct_unclipped_four_color_blocks() -> Result<()> {
    let (catalog, source) = inputs()?;
    let pixels = catalog.convert(&source)?;
    assert_eq!(pixels.len(), 6);
    for (offset, frame) in pixels.iter().enumerate() {
        assert_eq!(frame.indices.len(), 1024);
        for color in 0..4 {
            assert!(
                frame.indices.contains(&color),
                "item {offset} loses color {color}"
            );
        }
        assert!(
            pixels[..offset]
                .iter()
                .all(|other| other.indices != frame.indices)
        );
    }
    let mut damaged = source.clone();
    damaged[20] ^= 1;
    assert!(
        catalog.convert(&damaged).is_err(),
        "source drift must be rejected"
    );
    Ok(())
}

#[test]
fn rejects_invalid_crops_ids_and_collision_instead_of_publishing() -> Result<()> {
    let (mut catalog, source) = inputs()?;
    catalog.items[0].crop[0] = usize::MAX;
    assert!(catalog.convert(&source).is_err());
    let (mut catalog, source) = inputs()?;
    catalog.items[1].id = catalog.items[0].id.clone();
    assert!(catalog.convert(&source).is_err());
    let (mut catalog, source) = inputs()?;
    catalog.items[0].collision[0] = "WATER".into();
    assert!(catalog.convert(&source).is_err());
    Ok(())
}

#[test]
fn fair_wheel_has_four_connected_blocks_and_no_clipped_exterior() -> Result<()> {
    let (catalog, source) = catalog_inputs("state-fair-v1")?;
    let frames = catalog.convert(&source)?;
    assert_eq!(frames.len(), 7);
    for n in 0..64 {
        let pixel = |x: usize, y: usize| frames[y / 32 * 2 + x / 32].indices[y % 32 * 32 + x % 32];
        assert_eq!(pixel(n, 0), 0);
        assert_eq!(pixel(n, 63), 0);
        assert_eq!(pixel(0, n), 0);
        assert_eq!(pixel(63, n), 0);
    }
    assert!((0..32).any(|y| frames[0].indices[y * 32 + 31] != 0));
    assert!((0..32).any(|x| frames[0].indices[31 * 32 + x] != 0));
    for (item, frame) in catalog.items.iter().zip(&frames) {
        assert!(item.transparent_background);
        assert!(frame.indices.iter().filter(|index| **index != 0).count() > 64);
        let preview = crystal_assets::minnesota_tile_item_preview(frame, item, 1)?;
        assert_eq!(&preview[37..41], b"IDAT");
        let length = u32::from_be_bytes(preview[33..37].try_into()?) as usize;
        let mut decoded = Vec::new();
        ZlibDecoder::new(&preview[41..41 + length]).read_to_end(&mut decoded)?;
        for (n, &index) in frame.indices.iter().enumerate() {
            if index != 0 {
                let offset = n / 32 * 129 + 1 + n % 32 * 4;
                assert_eq!(
                    &decoded[offset..offset + 3],
                    &item.palette_foregrounds[&item.palette_for_tile(n / 32 / 8 * 4 + n % 32 / 8)]
                        [usize::from(index - 1)]
                );
            }
        }
    }
    Ok(())
}

#[test]
fn rejects_missing_nonreciprocal_shifted_and_nonadjacent_fair_seams() -> Result<()> {
    for fault in 0..5 {
        let (mut catalog, source) = catalog_inputs("state-fair-v1")?;
        match fault {
            0 => catalog.items[0]
                .neighbors
                .insert("E".into(), "missing".into()),
            1 => catalog.items[1].neighbors.remove("W"),
            2 => {
                catalog.items[0].offset[0] = 1;
                None
            }
            3 => {
                catalog.items[1].crop[0] += 1;
                None
            }
            _ => catalog.items[0]
                .neighbors
                .insert("NE".into(), "wheel_ne".into()),
        };
        assert!(catalog.convert(&source).is_err(), "seam fault {fault}");
    }
    Ok(())
}

#[test]
fn fair_display_area_sampling_recovers_source_contours_without_changing_the_grid() -> Result<()> {
    let (mut catalog, source) = catalog_inputs("state-fair-v1")?;
    let area = catalog.convert(&source)?;
    assert!(catalog.items[6].area_sampling);
    catalog.items[6].area_sampling = false;
    let center = catalog.convert(&source)?;
    let dark = |indices: &[u8]| indices.iter().filter(|index| **index == 3).count();
    assert!(dark(&area[6].indices) > dark(&center[6].indices));
    assert_eq!(area[6].indices.len(), center[6].indices.len());
    catalog.items[6].area_sampling = true;
    catalog.items[6].transparent_background = false;
    assert!(catalog.convert(&source).is_err());
    catalog.items[6].transparent_background = true;
    catalog.items[6].source_row_split = Some(640);
    assert!(catalog.convert(&source).is_err());
    Ok(())
}

#[test]
#[ignore = "requires CRYSTAL_TILE_TEST_PACK pointing to a completed external pack"]
fn extension_preserves_every_original_block_runtime_file_and_audio() -> Result<()> {
    let base = crystal_assets::read_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_TILE_TEST_PACK").context("CRYSTAL_TILE_TEST_PACK")?,
    )?;
    for name in ["northwoods-v1", "state-fair-v1"] {
        verify_extension_preservation(&base, name)?;
    }
    Ok(())
}

fn verify_extension_preservation(
    base: &crystal_assets::CompiledGamePack,
    name: &str,
) -> Result<()> {
    let (catalog, source) = catalog_inputs(name)?;
    let tileset = &catalog.tileset_id;
    let pixels = catalog.convert(&source)?;
    let pack = build_minnesota_tile_art_pack(base, &catalog, &source)?;
    let files = pack.runtime_files();
    for (name, original) in base.runtime_files() {
        assert_eq!(
            files.get(name),
            Some(original),
            "original file changed: {name}"
        );
    }
    assert_eq!(pack.data().maps, base.data().maps);
    assert_eq!(pack.data().audio, base.data().audio);
    let mut normalized = serde_json::to_value(pack.data())?;
    normalized["tilesets"]
        .as_object_mut()
        .context("tilesets")?
        .remove(tileset.as_str());
    for action in ["cut", "whirlpool"] {
        normalized["field_moves"][action]["replacements"]
            .as_object_mut()
            .context("field replacements")?
            .remove(tileset.as_str());
    }
    assert_eq!(
        normalized,
        serde_json::to_value(base.data())?,
        "unrelated game data changed"
    );
    let blocks = &files[&format!("data/tilesets/{tileset}_metatiles.bin")];
    let old_blocks = &files["data/tilesets/johto_metatiles.bin"];
    assert_eq!(&blocks[..old_blocks.len()], old_blocks);
    let old_definition = &base.data().tilesets["johto"];
    let new_definition = &pack.data().tilesets[tileset];
    let old_art = &files["gfx/tilesets/johto.2bpp"];
    let new_art = &files[&format!("gfx/tilesets/{tileset}.2bpp")];
    for glyph in old_blocks {
        let palette = old_definition.palette_map[usize::from(*glyph)];
        assert_eq!(new_definition.palette_map[usize::from(*glyph)], palette);
        let old_slot = if palette & 8 == 0 {
            usize::from(*glyph)
        } else {
            96 + usize::from(*glyph & 127)
        };
        let new_slot = if palette & 8 == 0 {
            usize::from(*glyph)
        } else {
            128 + usize::from(*glyph & 127)
        };
        assert_eq!(
            &old_art[old_slot * 16..old_slot * 16 + 16],
            &new_art[new_slot * 16..new_slot * 16 + 16]
        );
    }
    for (key, collision) in &old_definition.collision {
        assert_eq!(new_definition.collision.get(key), Some(collision));
    }
    // Decode every published bitplane independently, exercising bank selection
    // and metatile row order for both standalone and assembled props.
    for (offset, (item, frame)) in catalog.items.iter().zip(&pixels).enumerate() {
        let key = format!("{:02x}", 128 + offset);
        assert_eq!(new_definition.collision[&key], item.collision);
        for y in 0..32 {
            for x in 0..32 {
                let glyph = blocks[(128 + offset) * 16 + (y / 8) * 4 + x / 8];
                let palette = new_definition.palette_map[usize::from(glyph)];
                let tile_x = x / 8 * 8;
                let tile_y = y / 8 * 8;
                let empty = (tile_y..tile_y + 8).all(|row| {
                    (tile_x..tile_x + 8).all(|column| frame.indices[row * 32 + column] == 0)
                });
                if empty {
                    let ground = if item.palette_for_tile((y / 8) * 4 + x / 8) == 2 {
                        2
                    } else {
                        1
                    };
                    assert_eq!(glyph, old_blocks[ground * 16 + (y / 8) * 4 + x / 8]);
                    continue;
                }
                assert_eq!(palette & 7, item.palette_for_tile((y / 8) * 4 + x / 8));
                let slot = usize::from(glyph & 127) + usize::from(palette >> 3) * 128;
                let row = slot * 16 + (y % 8) * 2;
                let bit = 7 - x % 8;
                let pixel = ((new_art[row] >> bit) & 1) | (((new_art[row + 1] >> bit) & 1) << 1);
                assert_eq!(pixel, frame.indices[y * 32 + x]);
            }
        }
    }
    assert_eq!(
        pack.data().field_moves.cut.replacements.get(tileset),
        base.data().field_moves.cut.replacements.get("johto")
    );
    Ok(())
}
