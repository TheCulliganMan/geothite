//! Append exact native blocks using unused glyph slots in a city-only tileset.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::{CompiledGamePack, CompiledTilesetExtension};

#[derive(Deserialize)]
struct Art {
    tileset: String,
    source_pixels_modified: bool,
    #[serde(default)]
    derived_blocks: Vec<Recipe>,
}

#[derive(Deserialize)]
struct Recipe {
    id: String,
    source_tileset: String,
    source_block: String,
    collision: Vec<String>,
    palette_override: Option<u8>,
    tile_row_order: Option<Vec<usize>>,
}

fn slot(glyph: u8, palette: u8, count: usize) -> Result<usize> {
    let physical = if palette & 8 == 0 {
        usize::from(glyph)
    } else {
        usize::from(glyph & 0x7f) + count / 2
    };
    ensure!(
        physical < count,
        "native glyph addresses missing graphics bank"
    );
    Ok(physical)
}

pub(super) fn install(
    base: &CompiledGamePack,
    source: &str,
    alias: &str,
    expected: &str,
    city: &str,
) -> Result<CompiledGamePack> {
    let art: Art = serde_json::from_str(source)?;
    ensure!(
        !art.source_pixels_modified && art.tileset == expected,
        "city native art source differs"
    );
    if art.derived_blocks.is_empty() {
        ensure!(
            alias == expected,
            "city art alias needs explicit native recipes"
        );
        return Ok(base.clone());
    }
    ensure!(
        alias.starts_with("mn_") && alias != expected,
        "city art needs a dedicated alias"
    );
    let files = &base.runtime_files;
    ensure!(
        !files.contains_key(&format!("gfx/tilesets/{expected}.pal")),
        "city art expects shared native palettes"
    );
    let mut definition = base
        .data
        .tilesets
        .get(expected)
        .context("city source art")?
        .clone();
    let mut metatiles = files
        .get(&format!("data/tilesets/{expected}_metatiles.bin"))
        .context("city native blocks")?
        .clone();
    let mut graphics = files
        .get(&format!("gfx/tilesets/{expected}.2bpp"))
        .context("city native graphics")?
        .clone();
    let count = graphics.len() / 16;
    ensure!(count == 192, "city source graphics extent differs");
    let referenced = metatiles.iter().copied().collect::<BTreeSet<_>>();
    let protected = referenced
        .iter()
        .map(|&glyph| {
            slot(
                glyph,
                *definition
                    .palette_map
                    .get(usize::from(glyph))
                    .context("native palette address")?,
                count,
            )
        })
        .collect::<Result<BTreeSet<_>>>()?;
    let palette_count = definition.palette_map.len();
    let mut available = (0..count)
        .filter(|physical| !protected.contains(physical))
        .filter_map(|physical| {
            let (glyph, bank) = if physical < count / 2 {
                (physical as u8, 0)
            } else {
                ((physical - count / 2) as u8 | 0x80, 8)
            };
            (!referenced.contains(&glyph) && usize::from(glyph) < palette_count)
                .then_some((glyph, physical, bank))
        });
    let mut matching = BTreeMap::new();
    for &glyph in &referenced {
        // These inherited destinations animate. Identical static imported
        // pixels must not accidentally become flowers or water at runtime.
        if matches!(glyph, 0x03 | 0x14) {
            continue;
        }
        let palette = definition.palette_map[usize::from(glyph)];
        let physical = slot(glyph, palette, count)?;
        matching.insert(
            (
                graphics[physical * 16..(physical + 1) * 16].to_vec(),
                palette & 7,
            ),
            glyph,
        );
    }
    for recipe in &art.derived_blocks {
        let destination = u16::from_str_radix(&recipe.id, 16)?;
        ensure!(
            usize::from(destination) == metatiles.len() / 16 && destination <= 255,
            "city native block recipes must append contiguously"
        );
        ensure!(
            recipe.tile_row_order.is_none(),
            "city native rows cannot be rearranged without a collision contract"
        );
        let source_id = usize::from(u16::from_str_radix(&recipe.source_block, 16)?);
        ensure!(
            !files.contains_key(&format!("gfx/tilesets/{}.pal", recipe.source_tileset)),
            "city import expects shared palette"
        );
        let source = base
            .data
            .tilesets
            .get(&recipe.source_tileset)
            .context("city imported native art")?;
        let collision = source
            .collision
            .get(&format!("{source_id:02x}"))
            .context("city imported collision")?;
        ensure!(
            *collision == recipe.collision,
            "city imported collision differs from native source"
        );
        ensure!(
            recipe.palette_override.is_none_or(|palette| palette < 8),
            "native palette override outside bank"
        );
        let source_blocks = files
            .get(&format!(
                "data/tilesets/{}_metatiles.bin",
                recipe.source_tileset
            ))
            .context("city imported blocks")?;
        let source_graphics = files
            .get(&format!("gfx/tilesets/{}.2bpp", recipe.source_tileset))
            .context("city imported glyphs")?;
        let glyphs = source_blocks
            .get(source_id * 16..(source_id + 1) * 16)
            .context("city imported block extent")?;
        for &glyph in glyphs {
            let palette = *source
                .palette_map
                .get(usize::from(glyph))
                .context("city imported palette")?;
            let physical = slot(glyph, palette, source_graphics.len() / 16)?;
            let pixels = &source_graphics[physical * 16..(physical + 1) * 16];
            let color = recipe.palette_override.unwrap_or(palette & 7);
            let key = (pixels.to_vec(), color);
            let destination = if let Some(&existing) = matching.get(&key) {
                existing
            } else {
                let (new_glyph, physical, bank) = available
                    .next()
                    .context("city imported art exhausts unused native glyph slots")?;
                graphics[physical * 16..(physical + 1) * 16].copy_from_slice(pixels);
                definition.palette_map[usize::from(new_glyph)] = color | bank;
                matching.insert(key, new_glyph);
                new_glyph
            };
            metatiles.push(destination);
        }
        ensure!(
            definition
                .collision
                .insert(format!("{destination:02x}"), collision.clone())
                .is_none(),
            "occupied city collision block"
        );
    }
    base.with_tileset_extension(CompiledTilesetExtension {
        manifest_id: format!("{}-expanded-city-art", city.to_ascii_lowercase()),
        tileset_id: alias.into(),
        behavior_source_tileset_id: Some(expected.into()),
        definition,
        metatiles,
        tile_graphics_png: crate::native_tile_png::from_2bpp(&graphics)?,
        tile_graphics_2bpp: graphics,
    })
}
