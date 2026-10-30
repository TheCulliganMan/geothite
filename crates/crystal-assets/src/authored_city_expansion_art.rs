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
    extended_graphics: bool,
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
    tile_order: Option<[usize; 16]>,
}

/// Rearrange native glyphs only when every source and destination quadrant is solid.
pub(super) fn order_solid_glyphs(
    glyphs: &[u8],
    collision: &[String],
    order: &[usize; 16],
) -> Result<[u8; 16]> {
    ensure!(
        glyphs.len() == 16
            && collision.len() == 4
            && collision.iter().all(|kind| kind == "WALL")
            && order.iter().all(|&index| index < 16),
        "native glyph composition requires sixteen valid glyphs and four solid quadrants"
    );
    Ok(order.map(|index| glyphs[index]))
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
    let source_count = graphics.len() / 16;
    ensure!(
        matches!(source_count, 192 | 256),
        "city source graphics extent differs"
    );
    let inherited_foregrounds: BTreeMap<u8, [[u8; 3]; 3]> = files
        .get(&format!("data/tilesets/{expected}_foregrounds.json"))
        .map(|bytes| serde_json::from_slice(bytes))
        .transpose()
        .context("city base foreground colors")?
        .unwrap_or_default();
    let referenced = metatiles.iter().copied().collect::<BTreeSet<_>>();
    if art.extended_graphics {
        // Give each original bank 128 physical slots. Copy referenced source
        // glyphs to their new physical addresses without changing any IDs,
        // palette colors or pixels. Only the new alias receives the wider bank.
        let mut extended = vec![0; 256 * 16];
        let mut assigned = BTreeMap::new();
        for &glyph in &referenced {
            let palette = definition.palette_map[usize::from(glyph)];
            let from = slot(glyph, palette, source_count)?;
            let to = slot(glyph, palette, 256)?;
            let pixels = &graphics[from * 16..(from + 1) * 16];
            if let Some(previous) = assigned.insert(to, pixels) {
                ensure!(
                    previous == pixels,
                    "native glyph bank expansion aliases different pixels"
                );
            }
            extended[to * 16..(to + 1) * 16].copy_from_slice(pixels);
        }
        graphics = extended;
    }
    let count = graphics.len() / 16;
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
            (!referenced.contains(&glyph)
                && !inherited_foregrounds.contains_key(&glyph)
                && usize::from(glyph) < palette_count)
                .then_some((glyph, physical, bank))
        });
    let mut matching = BTreeMap::new();
    let mut foregrounds = inherited_foregrounds.clone();
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
                inherited_foregrounds.get(&glyph).copied(),
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
        let source_foregrounds: BTreeMap<u8, [[u8; 3]; 3]> = files
            .get(&format!(
                "data/tilesets/{}_foregrounds.json",
                recipe.source_tileset
            ))
            .map(|bytes| serde_json::from_slice(bytes))
            .transpose()
            .context("city imported foreground colors")?
            .unwrap_or_default();
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
        let glyphs = match &recipe.tile_order {
            Some(order) => order_solid_glyphs(glyphs, collision, order)?.to_vec(),
            None => glyphs.to_vec(),
        };
        for glyph in glyphs {
            let palette = *source
                .palette_map
                .get(usize::from(glyph))
                .context("city imported palette")?;
            let physical = slot(glyph, palette, source_graphics.len() / 16)?;
            let pixels = &source_graphics[physical * 16..(physical + 1) * 16];
            let color = recipe.palette_override.unwrap_or(palette & 7);
            let foreground = source_foregrounds.get(&glyph).copied();
            ensure!(
                foreground.is_none() || recipe.palette_override.is_none(),
                "private native foregrounds cannot be replaced by a palette override"
            );
            let key = (pixels.to_vec(), color, foreground);
            let destination = if let Some(&existing) = matching.get(&key) {
                existing
            } else {
                let (new_glyph, physical, bank) = available
                    .next()
                    .context("city imported art exhausts unused native glyph slots")?;
                graphics[physical * 16..(physical + 1) * 16].copy_from_slice(pixels);
                definition.palette_map[usize::from(new_glyph)] = color | bank;
                if let Some(colors) = foreground {
                    foregrounds.insert(new_glyph, colors);
                }
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
    let mut pack = base.with_tileset_extension(CompiledTilesetExtension {
        manifest_id: format!("{}-expanded-city-art", city.to_ascii_lowercase()),
        tileset_id: alias.into(),
        behavior_source_tileset_id: Some(expected.into()),
        definition,
        metatiles,
        tile_graphics_png: crate::native_tile_png::from_2bpp(&graphics)?,
        tile_graphics_2bpp: graphics,
    })?;
    if !foregrounds.is_empty() {
        let path = format!("data/tilesets/{alias}_foregrounds.json");
        ensure!(
            !pack.runtime_files.contains_key(&path),
            "occupied city foreground identity"
        );
        pack.runtime_files
            .insert(path, serde_json::to_vec(&foregrounds)?);
        pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
            pack.format_version,
            &pack.data,
            &pack.audio_manifest,
            &pack.runtime_files,
            &pack.report,
        )?;
        crate::verify_compiled_game_pack_for_runtime(&pack)?;
    }
    Ok(pack)
}

#[cfg(test)]
mod tests {
    use super::order_solid_glyphs;

    #[test]
    fn solid_composition_rejects_doors_and_invalid_glyph_addresses() {
        let glyphs = std::array::from_fn::<_, 16, _>(|index| index as u8);
        let mut order = std::array::from_fn(|index| index);
        let mut collision = vec!["WALL".to_owned(); 4];
        order[3] = 0;
        assert_eq!(
            order_solid_glyphs(&glyphs, &collision, &order).expect("solid wall composition")[3],
            0
        );
        collision[3] = "DOOR".into();
        assert!(order_solid_glyphs(&glyphs, &collision, &order).is_err());
        collision[3] = "WALL".into();
        order[3] = 16;
        assert!(order_solid_glyphs(&glyphs, &collision, &order).is_err());
    }
}
