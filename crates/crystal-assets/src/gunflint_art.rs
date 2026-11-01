//! Native forest art with a bounded set of Johto rock/crossing additions.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};

use crate::native_tile_png;
use crate::{CompiledGamePack, CompiledTilesetExtension};

// These IDs are the authoring layout's explicit palette, not inferred terrain.
const IMPORT_BLOCKS: [u8; 18] = [
    0x0a, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x06, 0x01, 0x03,
    0x79, 0x7a,
];

/// Adds Gunflint's derived forest art without changing maps, spawns or audio.
///
/// All forty original forest blocks retain their pixels and collision. Blocks
/// 0x28 through 0x39 use native Johto rocks, cave face, masonry crossing, ground,
/// grass and shore corners. Imported glyphs occupy unused slots in the existing
/// sheet; its dimensions and original bank addressing remain unchanged.
///
/// # Errors
/// Returns an error for missing/incompatible source art, an occupied glyph slot,
/// an existing derived tileset, or failed typed pack validation.
pub fn build_gunflint_art_pack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    let files = base.runtime_files();
    let mut definition = base
        .data()
        .tilesets
        .get("forest")
        .context("source forest tileset")?
        .clone();
    let johto = base
        .data()
        .tilesets
        .get("johto")
        .context("source Johto tileset")?;
    let mut metatiles = files
        .get("data/tilesets/forest_metatiles.bin")
        .context("forest metatiles")?
        .clone();
    let source_blocks = files
        .get("data/tilesets/johto_metatiles.bin")
        .context("Johto metatiles")?;
    let mut graphics = files
        .get("gfx/tilesets/forest.2bpp")
        .context("forest pixels")?
        .clone();
    let source_graphics = files
        .get("gfx/tilesets/johto.2bpp")
        .context("Johto pixels")?;
    ensure!(
        metatiles.len() == 40 * 16 && graphics.len() == 192 * 16,
        "forest source extent changed"
    );
    // The source collision table reserves 64 entries for forty art blocks.
    // Keep its forty visible definitions before appending the bounded imports.
    definition
        .collision
        .retain(|key, _| u16::from_str_radix(key, 16).is_ok_and(|id| id < 40));
    ensure!(
        !files.contains_key("gfx/tilesets/forest.pal")
            && !files.contains_key("gfx/tilesets/johto.pal"),
        "derived art expects the shared background palette"
    );
    let protected = metatiles
        .iter()
        .map(|&tile| {
            let bank = definition
                .palette_map
                .get(usize::from(tile))
                .context("forest source palette")?
                & 8;
            Ok(if bank != 0 {
                usize::from(tile & 0x7f) + 96
            } else {
                usize::from(tile)
            })
        })
        .collect::<Result<BTreeSet<_>>>()?;
    let mut glyphs = BTreeSet::new();
    for block in IMPORT_BLOCKS {
        glyphs.extend(
            source_blocks
                .get(usize::from(block) * 16..(usize::from(block) + 1) * 16)
                .context("import source block")?
                .iter()
                .copied(),
        );
    }
    ensure!(
        glyphs.len() <= 32,
        "bounded import no longer fits the free forest range"
    );
    let mut relocated = BTreeMap::new();
    for (index, source) in glyphs.into_iter().enumerate() {
        let destination = 0x40 + u8::try_from(index)?;
        ensure!(
            !metatiles.contains(&destination) && !protected.contains(&usize::from(destination)),
            "forest glyph slot {destination:02x} is referenced"
        );
        let palette = *johto
            .palette_map
            .get(usize::from(source))
            .context("import palette")?;
        ensure!(
            palette & 8 == 0,
            "import moved to an unsupported source bank"
        );
        let bytes = source_graphics
            .get(usize::from(source) * 16..(usize::from(source) + 1) * 16)
            .context("import glyph pixels")?;
        graphics[usize::from(destination) * 16..(usize::from(destination) + 1) * 16]
            .copy_from_slice(bytes);
        *definition
            .palette_map
            .get_mut(usize::from(destination))
            .context("derived glyph palette")? = palette;
        relocated.insert(source, destination);
    }
    for (index, source) in IMPORT_BLOCKS.into_iter().enumerate() {
        let block = 0x28 + index;
        for tile in &source_blocks[usize::from(source) * 16..(usize::from(source) + 1) * 16] {
            metatiles.push(*relocated.get(tile).context("relocated source glyph")?);
        }
        definition.collision.insert(
            format!("{block:02x}"),
            johto
                .collision
                .get(&format!("{source:02x}"))
                .context("import collision")?
                .clone(),
        );
    }
    let image = native_tile_png::from_2bpp(&graphics)?;
    base.with_tileset_extension(CompiledTilesetExtension {
        manifest_id: "minnesota-gunflint-art".into(),
        tileset_id: "mn_gunflint".into(),
        behavior_source_tileset_id: Some("forest".into()),
        definition,
        metatiles,
        tile_graphics_2bpp: graphics,
        tile_graphics_png: image,
    })
}
