//! One exact native waterfall block in unused Minneapolis glyph storage.

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

pub(super) fn waterfall(base: &CompiledGamePack, block: u16) -> Result<CompiledGamePack> {
    const TILESET: &str = "mn_minneapolis_city";
    ensure!(
        block == 0x99,
        "only the declared St Anthony Falls block is supported"
    );
    let files = &base.runtime_files;
    let definition = base
        .data
        .tilesets
        .get(TILESET)
        .context("installed Minneapolis art")?;
    let cave = base
        .data
        .tilesets
        .get("cave")
        .context("native waterfall art")?;
    let metatile_path = format!("data/tilesets/{TILESET}_metatiles.bin");
    let graphics_path = format!("gfx/tilesets/{TILESET}.2bpp");
    let mut blocks = files.get(&metatile_path).context("city metatiles")?.clone();
    let mut graphics = files
        .get(&graphics_path)
        .context("city native graphics")?
        .clone();
    ensure!(
        blocks.len() == usize::from(block) * 16
            && graphics.len() == 192 * 16
            && !definition.collision.contains_key("99")
            && !files.contains_key(&format!("gfx/tilesets/{TILESET}.pal"))
            && !files.contains_key("gfx/tilesets/cave.pal"),
        "installed Minneapolis native art preimage/extent differs"
    );
    let source = files
        .get("data/tilesets/cave_metatiles.bin")
        .and_then(|bytes| bytes.get(0x2c * 16..0x2d * 16))
        .context("native cave waterfall block")?;
    let source_tile = source[0];
    let collision = cave
        .collision
        .get("2c")
        .context("native waterfall collision")?;
    ensure!(
        source.iter().all(|tile| *tile == source_tile)
            && collision.iter().all(|kind| kind == "WATERFALL")
            && collision.len() == 4,
        "native waterfall recipe changed"
    );
    let source_palette = *cave
        .palette_map
        .get(usize::from(source_tile))
        .context("waterfall palette")?;
    ensure!(source_palette < 8, "waterfall source changed graphics bank");
    let pixels = files
        .get("gfx/tilesets/cave.2bpp")
        .and_then(|bytes| {
            bytes.get(usize::from(source_tile) * 16..(usize::from(source_tile) + 1) * 16)
        })
        .context("native waterfall glyph")?;
    let referenced = blocks.iter().copied().collect::<BTreeSet<_>>();
    let mut protected = BTreeSet::new();
    for &glyph in &referenced {
        let palette = *definition
            .palette_map
            .get(usize::from(glyph))
            .context("city glyph palette")?;
        let slot = if palette & 8 == 0 {
            usize::from(glyph)
        } else {
            usize::from(glyph & 0x7f) + 96
        };
        ensure!(slot < 192, "city glyph addresses a missing bank");
        protected.insert(slot);
    }
    let (glyph, slot, bank) = (0..192)
        .find_map(|slot| {
            let (glyph, bank) = if slot < 96 {
                (slot as u8, 0)
            } else {
                (((slot - 96) as u8) | 0x80, 8)
            };
            (!protected.contains(&slot)
                && !referenced.contains(&glyph)
                && usize::from(glyph) < definition.palette_map.len())
            .then_some((glyph, slot, bank))
        })
        .context("no unused native glyph for St Anthony Falls")?;
    graphics[slot * 16..(slot + 1) * 16].copy_from_slice(pixels);
    blocks.extend([glyph; 16]);
    let mut updated = definition.clone();
    updated.palette_map[usize::from(glyph)] = source_palette | bank;
    updated.collision.insert("99".into(), collision.clone());
    let mut pack = base.clone();
    pack.runtime_files.insert(metatile_path, blocks);
    pack.runtime_files.insert(graphics_path, graphics.clone());
    pack.runtime_files.insert(
        format!("gfx/tilesets/{TILESET}.png"),
        crate::native_tile_png::from_2bpp(&graphics)?,
    );
    pack.runtime_files.insert(
        format!("data/tilesets/{TILESET}.json"),
        serde_json::to_vec(&updated.collision)?,
    );
    pack.runtime_files.insert(
        format!("data/tilesets/{TILESET}_palette_map.json"),
        serde_json::to_vec(&updated.palette_map)?,
    );
    pack.data.tilesets.insert(TILESET.into(), updated);
    // The enclosing transaction derives identity and verifies after terrain
    // references the new block; it adds no new map, program or report entry.
    Ok(pack)
}
