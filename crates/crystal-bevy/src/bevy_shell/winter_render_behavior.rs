//! Exact presentation aliases and terrain-owned colors for winter art variants.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_assets::AssetRoot;
use crystal_core::world::map::TilePosition;

use super::{Palette, load_tileset_palette_bank, tile_foreground};

/// Preserve only the animation behavior already owned by the cloned tileset.
/// Border Pines currently has no common animation registrations; do not alias it
/// to forest and accidentally add new water, flowers or restless-tree animation.
pub(super) fn animation_source(tileset: &str) -> &str {
    match tileset {
        "mn_winter_north_shore" => "mn_north_shore",
        "mn_winter_outdoor" => "mn_border_pines",
        "mn_winter_gunflint" => "mn_gunflint",
        _ => tileset,
    }
}

pub(super) fn has_north_shore_waterfall(tileset: &str) -> bool {
    matches!(tileset, "mn_north_shore" | "mn_winter_north_shore")
}

/// Cache exact winter palettes independently from native and prairie.
pub(super) fn grass_cache_style(tileset: &str) -> Option<&str> {
    match tileset {
        "mn_winter_outdoor" | "mn_winter_north_shore" | "mn_winter_gunflint" => Some(tileset),
        _ => None,
    }
}

fn grass_block(tileset: &str) -> Option<usize> {
    match tileset {
        "mn_winter_outdoor" | "mn_winter_gunflint" => Some(0x3d),
        "mn_winter_north_shore" => Some(0x84),
        _ => None,
    }
}

/// Select the palette at the actual rustle tile. Winter aliases also contain
/// unchanged green grass, whose rustle must retain the native shared cache.
pub(super) fn rustle_tileset(tileset: &str, block: Option<u16>) -> &str {
    if grass_block(tileset).is_some_and(|icy| block == u16::try_from(icy).ok()) {
        tileset
    } else {
        animation_source(tileset)
    }
}

/// Resolve the actual map block without wrapping an off-map coordinate into
/// another row. The presentation caller already owns this snapshot geometry.
pub(super) fn rustle_tileset_at<'a>(
    tileset: &'a str,
    width: u16,
    blocks: &[u16],
    point: TilePosition,
) -> &'a str {
    let block = usize::try_from(point.x)
        .ok()
        .zip(usize::try_from(point.y).ok())
        .filter(|(x, _)| *x / 2 < usize::from(width))
        .and_then(|(x, y)| {
            (y / 2)
                .checked_mul(usize::from(width))
                .and_then(|row| row.checked_add(x / 2))
        })
        .and_then(|index| blocks.get(index))
        .copied();
    rustle_tileset(tileset, block)
}

/// Resolve the icy grass colors from its actual metatile glyphs and authored
/// foreground metadata, using the same time-aware resolver as terrain loading.
/// No color values or alternate pixel catalog are embedded in this helper.
/// Missing or inconsistent winter metadata is a render error rather than green
/// fallback. Other tilesets return None without reading any runtime files.
pub(super) fn grass_palette(
    root: &AssetRoot,
    tileset: &str,
    time: &str,
) -> Result<Option<Palette>> {
    let Some(block) = grass_block(tileset) else {
        return Ok(None);
    };
    let assets = root.runtime_assets();
    let metatiles =
        crate::read_runtime_asset(assets.join(format!("data/tilesets/{tileset}_metatiles.bin")))?;
    let glyphs = metatiles
        .get(block * 16..(block + 1) * 16)
        .context("icy grass metatile is missing")?;
    let palette_map: Vec<u8> = serde_json::from_str(&crate::read_runtime_asset_to_string(
        assets.join(format!("data/tilesets/{tileset}_palette_map.json")),
    )?)
    .context("icy grass palette addresses")?;
    let foregrounds: BTreeMap<usize, [[u8; 3]; 3]> =
        serde_json::from_str(&crate::read_runtime_asset_to_string(
            assets.join(format!("data/tilesets/{tileset}_foregrounds.json")),
        )?)
        .context("icy grass authored foreground colors")?;
    let bank = load_tileset_palette_bank(root, tileset, time)?
        .context("icy grass has no native time palette bank")?;
    resolve_grass_palette(glyphs, &palette_map, &foregrounds, &bank).map(Some)
}

fn resolve_grass_palette(
    glyphs: &[u8],
    palette_map: &[u8],
    foregrounds: &BTreeMap<usize, [[u8; 3]; 3]>,
    bank: &[Palette],
) -> Result<Palette> {
    ensure!(
        glyphs.len() == 16,
        "icy grass requires one complete metatile"
    );
    let mut result = None;
    for &glyph in glyphs {
        let index = usize::from(glyph);
        ensure!(
            foregrounds.contains_key(&index),
            "icy grass glyph has no authored foreground"
        );
        let attribute = *palette_map
            .get(index)
            .context("icy grass glyph palette missing")?;
        let palette =
            tile_foreground::resolve(index, usize::from(attribute & 7), Some(bank), foregrounds)
                .context("icy grass palette cannot be resolved")?;
        if let Some(expected) = result {
            ensure!(
                expected == palette,
                "icy grass glyphs disagree on rustle colors"
            );
        } else {
            result = Some(palette);
        }
    }
    result.context("icy grass contains no glyphs")
}

#[cfg(test)]
#[path = "winter_render_behavior_tests.rs"]
mod tests;
