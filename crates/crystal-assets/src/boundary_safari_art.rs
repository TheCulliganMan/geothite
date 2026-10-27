//! Import native wooden campsite seating without changing any existing art.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, CompiledTilesetExtension, native_tile_png};

/// Adds a Safari tileset with unchanged Johto/deck art and a native wooden bench.
///
/// The new `mn_boundary_safari` tileset appends Kanto block 08 as block 83.
/// Existing source maps, tilesets, field-move behavior and audio are preserved.
/// This art overlay does not implement Safari expedition or encounter rules.
///
/// # Errors
/// Returns an error for missing or incompatible source art, insufficient unused
/// glyph slots, an existing Safari tileset, or failed native pack validation.
pub fn build_boundary_safari_art_pack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.data().tilesets.contains_key("mn_boundary_safari"),
        "Safari art is already installed"
    );
    let pack = if base.data().tilesets.contains_key("mn_headwaters") {
        base.clone()
    } else {
        crate::headwaters_art::build_headwaters_art_pack(base)?
    };
    let files = pack.runtime_files();
    let mut definition = pack.data().tilesets["mn_headwaters"].clone();
    let source = pack.data().tilesets.get("kanto").context("bench source")?;
    let mut blocks = files["data/tilesets/mn_headwaters_metatiles.bin"].clone();
    let mut graphics = files["gfx/tilesets/mn_headwaters.2bpp"].clone();
    let source_blocks = &files["data/tilesets/kanto_metatiles.bin"];
    let source_graphics = &files["gfx/tilesets/kanto.2bpp"];
    ensure!(
        blocks.len() == 131 * 16,
        "native headwaters art extent changed"
    );
    ensure!(
        !files.contains_key("gfx/tilesets/kanto.pal"),
        "bench import needs the shared native background palette"
    );
    let physical = |glyph: u8, palette: u8| {
        if palette & 8 == 0 {
            usize::from(glyph)
        } else {
            usize::from(glyph & 0x7f) + 96
        }
    };
    let used = blocks.iter().copied().collect::<BTreeSet<_>>();
    let protected = used
        .iter()
        .map(|&glyph| physical(glyph, definition.palette_map[usize::from(glyph)]))
        .collect::<BTreeSet<_>>();
    let mut free = (0..graphics.len() / 16)
        .filter(|&index| !protected.contains(&index))
        .filter_map(|index| {
            // Each compact native VRAM bank contains 96 physical glyphs.
            let (logical, bank) = if index < 96 {
                (index, 0)
            } else {
                ((index - 96) | 0x80, 8)
            };
            u8::try_from(logical).ok().map(|glyph| (glyph, index, bank))
        })
        .filter(|(glyph, _, _)| !used.contains(glyph));
    let mut relocated = BTreeMap::new();
    let mut imported = Vec::new();
    for &glyph in source_blocks
        .get(8 * 16..9 * 16)
        .context("native bench block")?
    {
        let target = if let Some(&target) = relocated.get(&glyph) {
            target
        } else {
            let palette = source.palette_map[usize::from(glyph)];
            let index = physical(glyph, palette);
            let pixels = source_graphics
                .get(index * 16..(index + 1) * 16)
                .context("bench glyph pixels")?;
            let existing = used.iter().copied().find(|&candidate| {
                let native_palette = definition.palette_map[usize::from(candidate)];
                let native_index = physical(candidate, native_palette);
                palette & 7 == native_palette & 7
                    && graphics.get(native_index * 16..(native_index + 1) * 16) == Some(pixels)
            });
            let target = if let Some(existing) = existing {
                existing
            } else {
                let (target, physical_index, bank) = free
                    .next()
                    .context("no unused native slot for wooden bench")?;
                graphics[physical_index * 16..(physical_index + 1) * 16].copy_from_slice(pixels);
                definition.palette_map[usize::from(target)] = (palette & 7) | bank;
                target
            };
            relocated.insert(glyph, target);
            target
        };
        imported.push(target);
    }
    blocks.extend(imported);
    definition
        .collision
        .insert("83".into(), source.collision["08"].clone());
    let png = native_tile_png::from_2bpp(&graphics)?;
    pack.with_tileset_extension(CompiledTilesetExtension {
        manifest_id: "minnesota-boundary-safari-art".into(),
        tileset_id: "mn_boundary_safari".into(),
        behavior_source_tileset_id: Some("johto".into()),
        definition,
        metatiles: blocks,
        tile_graphics_2bpp: graphics,
        tile_graphics_png: png,
    })
}
