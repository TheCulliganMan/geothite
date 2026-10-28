//! A private copy of the shelter's native lab pixels, with an unlit console.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, CompiledTilesetExtension, MapModule, native_tile_png};

pub(crate) const TILESET: &str = "mn_northstar_relay";

pub(crate) fn install(pack: &mut CompiledGamePack, room: &mut MapModule) -> Result<u16> {
    ensure!(
        room.attributes.tileset_name == "lab",
        "native lab shelter required"
    );
    let source = pack
        .data
        .tilesets
        .get("lab")
        .context("native lab definition")?;
    let files = pack.runtime_files();
    let source_blocks = files
        .get("data/tilesets/lab_metatiles.bin")
        .context("lab blocks")?;
    let source_graphics = files.get("gfx/tilesets/lab.2bpp").context("lab pixels")?;
    let mut definition = source.clone();
    definition.collision.clear();
    definition.palette_map.fill(0);
    let mut graphics = vec![0_u8; 192 * 16];
    let mut blocks = Vec::new();
    let mut glyphs = BTreeMap::new();
    let mut symbols = BTreeMap::new();
    let mut next = 0_usize;
    // Preserve every original collision quadrant, including both exit carpets.
    let needed = room.blocks.iter().copied().collect::<BTreeSet<_>>();
    for (original, off) in needed.into_iter().map(|id| (id, false)).chain([(8, true)]) {
        let id = u16::try_from(blocks.len() / 16)?;
        let key = format!("{original:02x}");
        for (quadrant, glyph) in source_blocks
            .get(usize::from(original) * 16..(usize::from(original) + 1) * 16)
            .context("lab block bounds")?
            .iter()
            .enumerate()
        {
            let attribute = source.palette_map[usize::from(*glyph)];
            let palette = attribute & 7;
            let logical = if let Some(existing) = glyphs.get(&(*glyph, palette, off)) {
                *existing
            } else {
                ensure!(next < 192, "relay pixels exceed native glyph capacity");
                let physical = if attribute & 8 == 0 {
                    usize::from(*glyph)
                } else {
                    usize::from(*glyph & 0x7f) + source_graphics.len() / 32
                };
                let pixels = source_graphics
                    .get(physical * 16..(physical + 1) * 16)
                    .context("lab glyph addressing")?;
                graphics[next * 16..(next + 1) * 16].copy_from_slice(pixels);
                if off {
                    // Turn off only the screen inside the original native
                    // bezel. The wall, case, keyboard and collision stay exact.
                    for y in 0..8 {
                        for x in 0..8 {
                            let px = quadrant % 4 * 8 + x;
                            let py = quadrant / 4 * 8 + y;
                            if (4..12).contains(&px) && (20..25).contains(&py) {
                                let mask = 1 << (7 - x);
                                graphics[next * 16 + y * 2] |= mask;
                                graphics[next * 16 + y * 2 + 1] |= mask;
                            }
                        }
                    }
                }
                let (logical, bank) = if next < 96 {
                    (next, 0)
                } else {
                    ((next - 96) | 0x80, 8)
                };
                definition.palette_map[logical] = palette | bank;
                let logical = u8::try_from(logical)?;
                glyphs.insert((*glyph, palette, off), logical);
                next += 1;
                logical
            };
            blocks.push(logical);
        }
        definition.collision.insert(
            format!("{id:02x}"),
            source
                .collision
                .get(&key)
                .context("native lab collision")?
                .clone(),
        );
        symbols.insert((original, off), id);
    }
    let off = symbols[&(8, true)];
    for id in &mut room.blocks {
        *id = symbols[&(*id, false)];
    }
    room.attributes.tileset_name = TILESET.into();
    let extension = CompiledTilesetExtension {
        manifest_id: "minnesota-northstar-relay-art-v1".into(),
        tileset_id: TILESET.into(),
        behavior_source_tileset_id: Some("lab".into()),
        definition,
        metatiles: blocks,
        tile_graphics_png: native_tile_png::from_2bpp(&graphics)?,
        tile_graphics_2bpp: graphics,
    };
    crate::validate_compiled_tileset_extension(&extension)?;
    crate::insert_compiled_tileset_extension(pack, extension)?;
    Ok(off)
}
