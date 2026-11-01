//! Append Bemidji's four exact timber metatiles without changing source assets.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, CompiledTilesetExtension, native_tile_png};

pub(crate) const TILESET: &str = "mn_bemidji";

fn physical(palette: u8, glyph: u8) -> usize {
    if palette & 8 == 0 {
        usize::from(glyph)
    } else {
        usize::from(glyph & 0x7f) + 96
    }
}

/// Retains every original used modern glyph/address/palette and appends Johto art.
pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    ensure!(
        !pack.data.tilesets.contains_key(TILESET),
        "Bemidji art occupied"
    );
    let mut definition = pack
        .data
        .tilesets
        .get("johto_modern")
        .context("modern source art")?
        .clone();
    let source = pack
        .data
        .tilesets
        .get("johto")
        .context("timber source art")?;
    ensure!(
        definition.palette_map.len() >= 224 && source.palette_map.len() >= 224,
        "native Bemidji source palette extent changed"
    );
    let files = pack.runtime_files();
    ensure!(
        !files.contains_key("gfx/tilesets/johto_modern.pal")
            && !files.contains_key("gfx/tilesets/johto.pal"),
        "Bemidji uses unchanged shared native palettes"
    );
    let mut blocks = files
        .get("data/tilesets/johto_modern_metatiles.bin")
        .context("modern blocks")?
        .clone();
    let mut graphics = files
        .get("gfx/tilesets/johto_modern.2bpp")
        .context("modern graphics")?
        .clone();
    let original_graphics = graphics.clone();
    let native_blocks = files
        .get("data/tilesets/johto_metatiles.bin")
        .context("timber blocks")?;
    let native_graphics = files
        .get("gfx/tilesets/johto.2bpp")
        .context("timber graphics")?;
    ensure!(
        blocks.len() == 128 * 16 && graphics.len() == 192 * 16 && native_graphics.len() == 192 * 16,
        "native Bemidji source art extent changed"
    );
    ensure!(
        blocks
            .iter()
            .all(|g| usize::from(*g) < definition.palette_map.len())
            && native_blocks
                .iter()
                .all(|g| usize::from(*g) < source.palette_map.len()),
        "native source glyph palette bounds"
    );
    let used: BTreeSet<u8> = blocks.iter().copied().collect();
    let protected: BTreeSet<usize> = used
        .iter()
        .map(|&glyph| physical(definition.palette_map[usize::from(glyph)], glyph))
        .collect();
    let mut destinations: Vec<(u8, usize)> = (0_u8..=255)
        .filter(|glyph| !used.contains(glyph))
        .filter_map(|glyph| {
            let slot = if glyph < 96 {
                usize::from(glyph)
            } else if (128..224).contains(&glyph) {
                usize::from(glyph - 128) + 96
            } else {
                return None;
            };
            (!protected.contains(&slot)).then_some((glyph, slot))
        })
        .collect();
    let mut imported = BTreeMap::new();
    for (index, block) in [0x22_usize, 0x23, 0x28, 0x29].into_iter().enumerate() {
        for &glyph in native_blocks
            .get(block * 16..(block + 1) * 16)
            .context("timber source block")?
        {
            let destination = if let Some(&destination) = imported.get(&glyph) {
                destination
            } else {
                let palette = source.palette_map[usize::from(glyph)];
                let slot = physical(palette, glyph);
                let pixels = native_graphics
                    .get(slot * 16..(slot + 1) * 16)
                    .context("timber source glyph")?;
                let reusable = used.iter().copied().find(|&candidate| {
                    let mapped = definition.palette_map[usize::from(candidate)];
                    let slot = physical(mapped, candidate);
                    mapped & 7 == palette & 7
                        && graphics.get(slot * 16..(slot + 1) * 16) == Some(pixels)
                });
                let destination = if let Some(candidate) = reusable {
                    candidate
                } else {
                    let (logical, slot) = destinations
                        .pop()
                        .context("unused native timber glyph capacity")?;
                    graphics[slot * 16..(slot + 1) * 16].copy_from_slice(pixels);
                    definition.palette_map[usize::from(logical)] =
                        (palette & 7) | if slot >= 96 { 8 } else { 0 };
                    logical
                };
                imported.insert(glyph, destination);
                destination
            };
            blocks.push(destination);
        }
        definition.collision.insert(
            format!("{:02x}", 0x80 + index),
            source
                .collision
                .get(&format!("{block:02x}"))
                .context("timber collision")?
                .clone(),
        );
    }
    for slot in protected {
        ensure!(
            graphics[slot * 16..(slot + 1) * 16] == original_graphics[slot * 16..(slot + 1) * 16],
            "modern glyph modified"
        );
    }
    let extension = CompiledTilesetExtension {
        manifest_id: "minnesota-bemidji-timber-art-v1".into(),
        tileset_id: TILESET.into(),
        behavior_source_tileset_id: Some("johto_modern".into()),
        definition,
        metatiles: blocks,
        tile_graphics_png: native_tile_png::from_2bpp(&graphics)?,
        tile_graphics_2bpp: graphics,
    };
    crate::validate_compiled_tileset_extension(&extension)?;
    crate::insert_compiled_tileset_extension(pack, extension)
}
