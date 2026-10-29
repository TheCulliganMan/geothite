//! Local native-art imports: a service stair, intake cradle and actual outflow.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, CompiledTilesetExtension};

/// Repack native blocks into explicit equal VRAM banks before appending a block.
/// Legacy source sheets can use the renderer's documented out-of-bank alias;
/// typed extensions must resolve that alias into actual pixels at publication.
pub(super) fn import_block(
    pack: &mut CompiledGamePack,
    base: &str,
    target: &str,
    source: &str,
    block: u16,
) -> Result<u16> {
    let mut definition = pack
        .data
        .tilesets
        .get(base)
        .context("base tileset")?
        .clone();
    let original = pack
        .runtime_files
        .get(&format!("data/tilesets/{base}_metatiles.bin"))
        .context("base blocks")?;
    let count = original.len() / 16;
    let mut metatiles = Vec::new();
    let mut graphics = vec![0; 192 * 16];
    let mut glyphs = BTreeMap::new();
    definition.palette_map.fill(0);
    for (tileset, bytes) in [
        (base, original.as_slice()),
        (
            source,
            pack.runtime_files
                .get(&format!("data/tilesets/{source}_metatiles.bin"))
                .context("source blocks")?
                .get(usize::from(block) * 16..(usize::from(block) + 1) * 16)
                .context("native import block")?,
        ),
    ] {
        let native = &pack.data.tilesets[tileset];
        let pixels = pack
            .runtime_files
            .get(&format!("gfx/tilesets/{tileset}.2bpp"))
            .context("native glyphs")?;
        ensure!(pixels.len() % 32 == 0, "native two-bank graphics");
        for &glyph in bytes {
            let attribute = native.palette_map[usize::from(glyph)];
            let tile_count = pixels.len() / 16;
            let mut physical = usize::from(glyph);
            if attribute & 8 != 0 && usize::from(glyph & 127) + tile_count / 2 < tile_count {
                physical = usize::from(glyph & 127) + tile_count / 2;
            } else if attribute & 8 != 0 && usize::from(glyph & 127) + 128 < tile_count {
                physical = usize::from(glyph & 127) + 128;
            } else if physical >= tile_count {
                physical = if glyph >= 128 && usize::from(glyph - 128) < tile_count {
                    usize::from(glyph - 128)
                } else {
                    0
                };
            }
            let ink = pixels
                .get(physical * 16..(physical + 1) * 16)
                .context("resolved native glyph address")?;
            let key = (ink.to_vec(), attribute & 7);
            let logical = if let Some(&logical) = glyphs.get(&key) {
                logical
            } else {
                let next = glyphs.len();
                ensure!(next < 192, "native import glyph capacity");
                graphics[next * 16..(next + 1) * 16].copy_from_slice(ink);
                let logical = if next < 96 { next } else { next - 96 + 128 };
                definition.palette_map[logical] = (attribute & 7) | if next >= 96 { 8 } else { 0 };
                let logical = u8::try_from(logical)?;
                glyphs.insert(key, logical);
                logical
            };
            metatiles.push(logical);
        }
    }
    let id = u16::try_from(count)?;
    definition.collision.insert(
        format!("{id:02x}"),
        pack.data.tilesets[source]
            .collision
            .get(&format!("{block:02x}"))
            .context("native import collision")?
            .clone(),
    );
    let extension = CompiledTilesetExtension {
        manifest_id: format!("{target}-native-v1"),
        tileset_id: target.into(),
        behavior_source_tileset_id: Some(base.into()),
        definition,
        metatiles,
        tile_graphics_png: crate::native_tile_png::from_2bpp(&graphics)?,
        tile_graphics_2bpp: graphics,
    };
    crate::validate_compiled_tileset_extension(&extension)?;
    crate::insert_compiled_tileset_extension(pack, extension)?;
    // House interiors use a fixed native palette rather than the outdoor table.
    // Preserve that palette on the private extension as well as its pixels.
    if let Some(palette) = pack
        .runtime_files
        .get(&format!("gfx/tilesets/{base}.pal"))
        .cloned()
    {
        pack.runtime_files
            .insert(format!("gfx/tilesets/{target}.pal"), palette);
    }
    Ok(id)
}
