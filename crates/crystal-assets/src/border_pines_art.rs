//! Native forest and timber rails for the hand-authored Border Pines passage.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};

use crate::native_tile_png;
use crate::{CompiledGamePack, CompiledTilesetExtension};

/// Adds two source-authentic timber rail blocks to unchanged Gunflint forest art.
///
/// `mn_border_pines` retains all 58 forest/Gunflint blocks. Block `3a` imports
/// native Kanto `77`; `3b` reorders its four tile rows as `[2,3,0,1]` and uses
/// native Johto `75` collision. Only unused physical glyph slots are written;
/// existing palette/bank addresses, source maps, catalogs and audio are retained.
/// Native forest animation and Cut `0f` to `17` behavior are inherited.
///
/// # Errors
/// Returns an error for missing/incompatible source content, insufficient unused
/// glyph space, an existing target tileset or failed native pack validation.
pub fn build_border_pines_art_pack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.data().tilesets.contains_key("mn_border_pines"),
        "Border Pines art is already installed"
    );
    let pack = if base.data().tilesets.contains_key("mn_gunflint") {
        base.clone()
    } else {
        crate::build_gunflint_art_pack(base)?
    };
    let files = pack.runtime_files();
    let mut definition = pack
        .data()
        .tilesets
        .get("mn_gunflint")
        .context("Gunflint source art")?
        .clone();
    let source = pack
        .data()
        .tilesets
        .get("kanto")
        .context("native timber source")?;
    let south = source
        .collision
        .get("77")
        .context("native south rail collision")?;
    let north = pack
        .data()
        .tilesets
        .get("johto")
        .context("native north rail source")?
        .collision
        .get("75")
        .context("native north rail collision")?;
    ensure!(
        south == &["FLOOR", "FLOOR", "WALL", "WALL"],
        "native south rail collision changed"
    );
    ensure!(
        north == &["WALL", "WALL", "FLOOR", "FLOOR"],
        "native north rail collision changed"
    );
    let mut metatiles = files
        .get("data/tilesets/mn_gunflint_metatiles.bin")
        .context("forest metatiles")?
        .clone();
    let mut graphics = files
        .get("gfx/tilesets/mn_gunflint.2bpp")
        .context("forest pixels")?
        .clone();
    let source_blocks = files
        .get("data/tilesets/kanto_metatiles.bin")
        .context("native timber metatiles")?;
    let source_graphics = files
        .get("gfx/tilesets/kanto.2bpp")
        .context("native timber pixels")?;
    ensure!(
        metatiles.len() == 58 * 16 && graphics.len() == 192 * 16,
        "derived forest extent changed"
    );
    ensure!(
        !files.contains_key("gfx/tilesets/kanto.pal"),
        "timber import requires the shared native palette"
    );
    let physical = |glyph: u8, palette: u8| {
        if palette & 8 == 0 {
            usize::from(glyph)
        } else {
            usize::from(glyph & 0x7f) + 96
        }
    };
    let used = metatiles.iter().copied().collect::<BTreeSet<_>>();
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
    let native = source_blocks
        .get(0x77 * 16..0x78 * 16)
        .context("native timber block")?;
    let mut relocated = BTreeMap::new();
    for &glyph in native {
        if relocated.contains_key(&glyph) {
            continue;
        }
        let palette = source.palette_map[usize::from(glyph)];
        let index = physical(glyph, palette);
        let pixels = source_graphics
            .get(index * 16..(index + 1) * 16)
            .context("native timber glyph")?;
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
                .context("no unused native glyph slot for timber rail")?;
            graphics[index * 16..(index + 1) * 16].copy_from_slice(pixels);
            definition.palette_map[usize::from(target)] = (palette & 7) | bank;
            target
        };
        relocated.insert(glyph, target);
    }
    for order in [[0, 1, 2, 3], [2, 3, 0, 1]] {
        for row in order {
            for glyph in &native[row * 4..row * 4 + 4] {
                metatiles.push(*relocated.get(glyph).context("relocated timber glyph")?);
            }
        }
    }
    definition.collision.insert("3a".into(), south.clone());
    definition.collision.insert("3b".into(), north.clone());
    let image = native_tile_png::from_2bpp(&graphics)?;
    pack.with_tileset_extension(CompiledTilesetExtension {
        manifest_id: "minnesota-border-pines-art".into(),
        tileset_id: "mn_border_pines".into(),
        behavior_source_tileset_id: Some("forest".into()),
        definition,
        metatiles,
        tile_graphics_2bpp: graphics,
        tile_graphics_png: image,
    })
}
