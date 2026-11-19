//! Install original four-shade 8×8 shoreline stones in unused native background tile slots.
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use super::super::{metatile_path, northstar_public_piers::address, source};
use crate::{CompiledGamePack, native_tile_png};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Ink {
    tileset: String,
    glyph: u8,
    palette_from: u8,
    snow_light_from: Option<u8>,
    bank: Option<u8>,
    rows: [String; 8],
}

fn inherit_colors<T: Copy + PartialEq + DeserializeOwned + Serialize>(
    raw: &mut Vec<u8>,
    tiles: &[Ink],
    blend: impl Fn(&Ink, Option<T>, &BTreeMap<u8, T>) -> Result<Option<T>>,
) -> Result<()> {
    let old: BTreeMap<u8, T> = serde_json::from_slice(raw)?;
    let mut revised = old.clone();
    for tile in tiles {
        if let Some(color) = blend(tile, old.get(&tile.palette_from).copied(), &old)? {
            revised.insert(tile.glyph, color);
        } else {
            revised.remove(&tile.glyph);
        }
    }
    if revised != old {
        *raw = serde_json::to_vec(&revised)?;
    }
    Ok(())
}

fn pixels(rows: &[String; 8]) -> Result<[u8; 16]> {
    let mut encoded = [0; 16];
    for (y, row) in rows.iter().enumerate() {
        ensure!(row.len() == 8, "expected an eight-pixel rock tile row");
        for (x, color) in row.bytes().enumerate() {
            ensure!((b'0'..=b'3').contains(&color), "invalid two-bit shade");
            let color = color - b'0';
            encoded[y * 2] |= (color & 1) << (7 - x);
            encoded[y * 2 + 1] |= (color >> 1) << (7 - x);
        }
    }
    Ok(encoded)
}

pub(super) fn install(
    pack: &mut CompiledGamePack,
    dir: &Path,
) -> Result<BTreeMap<String, BTreeSet<u8>>> {
    let mut banks = BTreeMap::<String, Vec<Ink>>::new();
    for section in super::SECTIONS {
        let name = format!("{section}-ink.json");
        if dir.join(&name).try_exists()? {
            for ink in source::<Vec<Ink>>(dir, &name)? {
                banks.entry(ink.tileset.clone()).or_default().push(ink);
            }
        }
    }
    let mut installed = BTreeMap::new();
    for (bank, tiles) in banks {
        ensure!(
            bank.starts_with("mn_"),
            "shoreline ink requires a Minnesota bank"
        );
        let layout = pack
            .runtime_files
            .get(&metatile_path(&bank))
            .context("missing native layout")?;
        let logical: BTreeSet<_> = layout.iter().copied().collect();
        let animation_path = format!("data/tilesets/{bank}_animations.json");
        let animations: BTreeMap<u8, serde_json::Value> = pack
            .runtime_files
            .get(&animation_path)
            .map(|data| serde_json::from_slice(data))
            .transpose()?
            .unwrap_or_default();
        let def = pack
            .data
            .tilesets
            .get_mut(&bank)
            .context("missing native tileset")?;
        let gfx = pack
            .runtime_files
            .get_mut(&format!("gfx/tilesets/{bank}.2bpp"))
            .context("missing native pixels")?;
        ensure!(gfx.len() % 16 == 0, "incomplete graphics bank");
        let count = gfx.len() / 16;
        let mut reserved = BTreeSet::new();
        for &id in &logical {
            let palette = def
                .palette_map
                .get(usize::from(id))
                .context("missing native tile palette")?;
            reserved.insert(address(id, *palette, count));
        }
        let mut added = BTreeSet::new();
        for tile in &tiles {
            let color = *def
                .palette_map
                .get(usize::from(tile.palette_from))
                .context("missing stone palette donor")?
                & 7;
            let vram = tile.bank.unwrap_or(1);
            ensure!(
                vram <= 1
                    && (vram == 1
                        || (bank == "mn_headwaters" && usize::from(tile.glyph) < count / 2)),
                "invalid native rock tile bank"
            );
            let palette = color | (vram << 3);
            let at = address(tile.glyph, palette, count);
            ensure!(
                (vram == 0 || tile.glyph >= 128)
                    && !matches!(tile.glyph, 0x03 | 0x14)
                    && !animations.contains_key(&tile.glyph)
                    && logical.contains(&tile.palette_from)
                    && !logical.contains(&tile.glyph)
                    && added.insert(tile.glyph)
                    && at < count
                    && reserved.insert(at),
                "occupied or invalid rock glyph {bank} {:02x}",
                tile.glyph
            );
            gfx[at * 16..at * 16 + 16].copy_from_slice(&pixels(&tile.rows)?);
            *def.palette_map
                .get_mut(usize::from(tile.glyph))
                .context("missing rock palette slot")? = palette;
        }
        let png = native_tile_png::from_2bpp(gfx)?;
        pack.runtime_files
            .insert(format!("gfx/tilesets/{bank}.png"), png);
        pack.runtime_files.insert(
            format!("data/tilesets/{bank}_palette_map.json"),
            serde_json::to_vec(&def.palette_map)?,
        );
        let key = format!("data/tilesets/{bank}_native_backgrounds.json");
        if let Some(raw) = pack.runtime_files.get_mut(&key) {
            inherit_colors(raw, &tiles, |_, color: Option<u8>, _| Ok(color))?;
        }
        let key = format!("data/tilesets/{bank}_foregrounds.json");
        let snow = tiles.iter().any(|tile| tile.snow_light_from.is_some());
        ensure!(
            !snow || pack.runtime_files.contains_key(&key),
            "missing snow foregrounds"
        );
        if let Some(raw) = pack.runtime_files.get_mut(&key) {
            inherit_colors(
                raw,
                &tiles,
                |tile, mut shades: Option<[[u8; 3]; 3]>, old| {
                    if let Some(id) = tile.snow_light_from {
                        ensure!(
                            logical.contains(&id),
                            "snow highlight donor is absent from {bank}"
                        );
                        shades.as_mut().context("missing slate foregrounds")?[0] =
                            old.get(&id).context("missing snow highlight")?[0];
                    }
                    Ok(shades)
                },
            )?;
        }
        installed.insert(bank, added);
    }
    Ok(installed)
}
