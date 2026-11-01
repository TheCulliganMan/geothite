//! Source-authentic blocked rail and paved crossing for Iron Range West.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, CompiledTilesetExtension, native_tile_png};

/// Adds native Johto Modern rail blocks to unchanged Johto and wooden-pier art.
///
/// The new `mn_iron_range` tileset appends original blocks 7e and 7f as 83 and
/// 84. Their native WALL/FLOOR collision remains authoritative. This imports
/// scenery only; it does not add a train, signal, timetable or rail controller.
/// Existing maps, source assets, field moves and audio remain unchanged.
///
/// # Errors
/// Returns an error for invalid or incompatible source content, exhausted unused
/// glyph slots, an existing target tileset or failed native pack validation.
pub fn build_iron_range_art_pack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.data().tilesets.contains_key("mn_iron_range"),
        "Iron Range art is already installed"
    );
    let pack = if base.data().tilesets.contains_key("mn_headwaters") {
        base.clone()
    } else {
        crate::headwaters_art::build_headwaters_art_pack(base)?
    };
    let files = pack.runtime_files();
    let mut definition = pack.data().tilesets["mn_headwaters"].clone();
    let source = pack
        .data()
        .tilesets
        .get("johto_modern")
        .context("native rail source")?;
    let mut blocks = files["data/tilesets/mn_headwaters_metatiles.bin"].clone();
    let mut graphics = files["gfx/tilesets/mn_headwaters.2bpp"].clone();
    let source_blocks = &files["data/tilesets/johto_modern_metatiles.bin"];
    let source_graphics = &files["gfx/tilesets/johto_modern.2bpp"];
    ensure!(blocks.len() == 131 * 16, "native pier extent changed");
    ensure!(
        !files.contains_key("gfx/tilesets/johto_modern.pal"),
        "rail import needs the shared native background palette"
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
        .filter(|index| !protected.contains(index))
        .filter_map(|index| {
            let (logical, bank) = if index < 96 {
                (index, 0)
            } else {
                ((index - 96) | 0x80, 8)
            };
            u8::try_from(logical).ok().map(|glyph| (glyph, index, bank))
        })
        .filter(|(glyph, _, _)| !used.contains(glyph));
    let mut relocated = BTreeMap::new();
    for (target_block, source_block, collision) in [(0x83, 0x7e, "WALL"), (0x84, 0x7f, "FLOOR")] {
        let source_collision = &source.collision[&format!("{source_block:02x}")];
        ensure!(
            source_collision.iter().all(|token| token == collision),
            "native rail collision changed"
        );
        for &glyph in source_blocks
            .get(source_block * 16..(source_block + 1) * 16)
            .context("native rail metatile")?
        {
            let target = if let Some(&target) = relocated.get(&glyph) {
                target
            } else {
                let palette = source.palette_map[usize::from(glyph)];
                let index = physical(glyph, palette);
                let pixels = source_graphics
                    .get(index * 16..(index + 1) * 16)
                    .context("native rail glyph")?;
                let existing = used.iter().copied().find(|&candidate| {
                    let own_palette = definition.palette_map[usize::from(candidate)];
                    let own_index = physical(candidate, own_palette);
                    own_palette & 7 == palette & 7
                        && graphics.get(own_index * 16..(own_index + 1) * 16) == Some(pixels)
                });
                let target = if let Some(existing) = existing {
                    existing
                } else {
                    let (target, index, bank) = free
                        .next()
                        .context("no unused native glyph slot for rail")?;
                    graphics[index * 16..(index + 1) * 16].copy_from_slice(pixels);
                    definition.palette_map[usize::from(target)] = (palette & 7) | bank;
                    target
                };
                relocated.insert(glyph, target);
                target
            };
            blocks.push(target);
        }
        definition
            .collision
            .insert(format!("{target_block:02x}"), source_collision.clone());
    }
    let png = native_tile_png::from_2bpp(&graphics)?;
    pack.with_tileset_extension(CompiledTilesetExtension {
        manifest_id: "minnesota-iron-range-art".into(),
        tileset_id: "mn_iron_range".into(),
        behavior_source_tileset_id: Some("johto".into()),
        definition,
        metatiles: blocks,
        tile_graphics_2bpp: graphics,
        tile_graphics_png: png,
    })
}
