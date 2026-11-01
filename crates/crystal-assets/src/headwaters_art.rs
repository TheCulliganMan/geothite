//! Native pier rails appended to an otherwise unchanged Johto tileset.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};

use crate::native_tile_png;
use crate::{CompiledGamePack, CompiledTilesetExtension};

pub(crate) fn build_headwaters_art_pack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    let files = base.runtime_files();
    let mut definition = base
        .data()
        .tilesets
        .get("johto")
        .context("Johto source")?
        .clone();
    let source = base
        .data()
        .tilesets
        .get("kanto")
        .context("native pier source")?;
    let mut metatiles = files
        .get("data/tilesets/johto_metatiles.bin")
        .context("Johto metatiles")?
        .clone();
    let native_blocks = files
        .get("data/tilesets/kanto_metatiles.bin")
        .context("Kanto metatiles")?;
    let mut graphics = files
        .get("gfx/tilesets/johto.2bpp")
        .context("Johto pixels")?
        .clone();
    let native_graphics = files
        .get("gfx/tilesets/kanto.2bpp")
        .context("Kanto pixels")?;
    ensure!(
        metatiles.len() == 128 * 16 && graphics.len() == 192 * 16,
        "Johto art extent changed"
    );
    ensure!(
        !files.contains_key("gfx/tilesets/johto.pal")
            && !files.contains_key("gfx/tilesets/kanto.pal"),
        "pier import requires the shared background palette"
    );
    let protected = metatiles
        .iter()
        .map(|&tile| {
            let palette = definition
                .palette_map
                .get(usize::from(tile))
                .context("original palette")?;
            Ok(if palette & 8 != 0 {
                usize::from(tile & 0x7f) + 96
            } else {
                usize::from(tile)
            })
        })
        .collect::<Result<BTreeSet<_>>>()?;
    // The shared ground and animated water keep their original Johto addresses.
    // Only the three wooden glyphs need previously unused physical slots.
    let mut relocated = BTreeMap::from([(0x39, 0x06), (0x14, 0x14)]);
    for (&from, &to) in &relocated {
        ensure!(
            source.palette_map[usize::from(from)] == definition.palette_map[usize::from(to)],
            "reused pier glyph palette changed"
        );
        ensure!(
            native_graphics[usize::from(from) * 16..(usize::from(from) + 1) * 16]
                == graphics[usize::from(to) * 16..(usize::from(to) + 1) * 16],
            "reused pier glyph pixels changed"
        );
    }
    for (from, to) in [(0x0e, 0x20), (0x31, 0x21), (0x55, 0x30)] {
        ensure!(
            !metatiles.contains(&to) && !protected.contains(&usize::from(to)),
            "pier destination glyph is in use"
        );
        let palette = *source
            .palette_map
            .get(usize::from(from))
            .context("pier palette")?;
        ensure!(palette & 8 == 0, "pier source bank changed");
        graphics[usize::from(to) * 16..(usize::from(to) + 1) * 16].copy_from_slice(
            native_graphics
                .get(usize::from(from) * 16..(usize::from(from) + 1) * 16)
                .context("pier glyph")?,
        );
        definition.palette_map[usize::from(to)] = palette;
        relocated.insert(from, to);
    }
    for (id, original, row_order) in [
        (0x80, 0x78, [0, 1, 2, 3]),
        (0x81, 0x77, [0, 1, 2, 3]),
        (0x82, 0x77, [2, 3, 0, 1]),
    ] {
        let block = native_blocks
            .get(original * 16..(original + 1) * 16)
            .context("native pier block")?;
        for row in row_order {
            for glyph in &block[row * 4..row * 4 + 4] {
                metatiles.push(*relocated.get(glyph).context("unexpected pier glyph")?);
            }
        }
        let collision = if id == 0x82 {
            definition
                .collision
                .get("75")
                .context("native north-edge collision")?
        } else {
            source
                .collision
                .get(&format!("{original:02x}"))
                .context("native pier collision")?
        }
        .clone();
        definition.collision.insert(format!("{id:02x}"), collision);
    }
    let image = native_tile_png::from_2bpp(&graphics)?;
    base.with_tileset_extension(CompiledTilesetExtension {
        manifest_id: "minnesota-headwaters-art".into(),
        tileset_id: "mn_headwaters".into(),
        behavior_source_tileset_id: Some("johto".into()),
        definition,
        metatiles,
        tile_graphics_2bpp: graphics,
        tile_graphics_png: image,
    })
}
