//! Native clock lighting for snow colors and golden harvest grass.

use std::collections::BTreeMap;

use anyhow::{Context, Result};
use crystal_assets::AssetRoot;

use super::{Palette, load_tileset_palette_bank};

pub(super) fn is_prairie(tileset: &str) -> bool {
    matches!(tileset, "mn_prairie_kanto" | "mn_prairie_headwaters")
}

pub(super) fn is_winter(tileset: &str) -> bool {
    matches!(
        tileset,
        "mn_winter_outdoor"
            | "mn_winter_north_shore"
            | "mn_outdoor_winter_stairs"
            | "mn_winter_gunflint"
            | "mn_ely_snow"
            | "mn_international_falls"
    )
}

// The native neutral background supplies the clock's light color. Relative to
// its day value, it tints authored colors without replacing their own hues.
// Day is an exact identity, and the shared gold/text bank remains untouched.
fn light_color(color: [u8; 3], light: [u8; 3], daylight: [u8; 3]) -> [u8; 3] {
    std::array::from_fn(|channel| {
        let reference = u16::from(daylight[channel]).max(1);
        let value =
            (u16::from(color[channel]) * u16::from(light[channel]) + reference / 2) / reference;
        value.min(255) as u8
    })
}

pub(super) fn light_prairie_bank(bank: &mut [Palette], day: &[Palette]) {
    let Some(light) = bank.first().map(|palette| palette[0]) else {
        return;
    };
    let Some(daylight) = day.first().map(|palette| palette[0]) else {
        return;
    };
    if let Some(gold) = bank.get_mut(7) {
        for color in gold {
            *color = light_color(*color, light, daylight);
        }
    }
}

/// Preserve private snow overrides while restoring native clock changes to
/// palette entries that were copied unchanged from the shared daylight bank.
pub(super) fn light_winter_bank(bank: &mut [Palette], day: &[Palette], current: &[Palette]) {
    let Some((daylight, light)) = day.first().zip(current.first()) else {
        return;
    };
    for ((palette, native_day), native_current) in bank.iter_mut().zip(day).zip(current) {
        for ((color, day_color), current_color) in
            palette.iter_mut().zip(native_day).zip(native_current)
        {
            *color = if *color == *day_color {
                *current_color
            } else {
                light_color(*color, light[0], daylight[0])
            };
        }
    }
}

/// Load authored foregrounds through the same lighting path for terrain,
/// animation frames and grass rustles. Emissive props retain their own colors.
pub(super) fn load_foregrounds(
    root: &AssetRoot,
    tileset: &str,
    bank: Option<&[Palette]>,
) -> Result<BTreeMap<usize, [[u8; 3]; 3]>> {
    let path = root
        .runtime_assets()
        .join(format!("data/tilesets/{tileset}_foregrounds.json"));
    let mut foregrounds: BTreeMap<usize, [[u8; 3]; 3]> = if crate::runtime_asset_exists(&path) {
        serde_json::from_str(&crate::read_runtime_asset_to_string(&path)?)
            .context("parse authored tile foreground colors")?
    } else {
        BTreeMap::new()
    };
    if is_winter(tileset) && !foregrounds.is_empty() {
        let day = load_tileset_palette_bank(root, tileset, "day")?
            .context("winter foregrounds require the native day palette")?;
        let light = bank
            .and_then(|palettes| palettes.first())
            .context("winter foregrounds require the current native palette")?[0];
        let daylight = day.first().context("native day palette is empty")?[0];
        for colors in foregrounds.values_mut() {
            for color in colors {
                *color = light_color(*color, light, daylight);
            }
        }
    }
    Ok(foregrounds)
}

#[cfg(test)]
#[path = "seasonal_palette_tests.rs"]
mod tests;
