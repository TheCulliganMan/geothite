//! Read-only scenery proof from an external pret checkout. This is not a pack
//! exporter or gameplay verifier; no source catalogs or artwork are persisted.
use crate::GeneratedGrid;
use anyhow::{Context, Result, bail, ensure};
use crystal_assets::TilesetDefinition;
use image::RgbaImage;
use std::{collections::BTreeMap, path::Path};

pub struct SourceArtPreview {
    definition: TilesetDefinition,
    files: BTreeMap<String, Vec<u8>>,
}

impl SourceArtPreview {
    pub fn load(pret: &Path) -> Result<Self> {
        let mut definitions = BTreeMap::new();
        let mut files = BTreeMap::new();
        for id in ["johto_modern", "johto", "park", "lab", "cave", "ice_path"] {
            let collision =
                std::fs::read_to_string(pret.join(format!("data/tilesets/{id}_collision.asm")))?;
            let palette =
                std::fs::read_to_string(pret.join(format!("gfx/tilesets/{id}_palette_map.asm")))?;
            definitions.insert(
                id.into(),
                TilesetDefinition {
                    collision: parse_collisions(&collision)?,
                    palette_map: parse_tile_palettes(&palette)?,
                },
            );
            let key = format!("data/tilesets/{id}_metatiles.bin");
            files.insert(key.clone(), std::fs::read(pret.join(key))?);
            let png = std::fs::read(pret.join(format!("gfx/tilesets/{id}.png")))?;
            files.insert(format!("gfx/tilesets/{id}.2bpp"), png_to_2bpp(&png)?);
        }
        // Reuse production atlas construction, including its exact source-art
        // shape, layout, collision and free-slot checks.
        let extension = crate::custom_tileset::build_extension_from_parts(
            &definitions,
            &files,
            "source-art-proof".into(),
        )?;
        let id = crate::GENERATED_TILESET_ID;
        files.clear();
        files.insert(
            format!("data/tilesets/{id}_metatiles.bin"),
            extension.metatiles,
        );
        files.insert(
            format!("gfx/tilesets/{id}.png"),
            extension.tile_graphics_png,
        );
        files.insert(
            "gfx/tilesets/bg_tiles.pal".into(),
            std::fs::read(pret.join("gfx/tilesets/bg_tiles.pal"))?,
        );
        Ok(Self {
            definition: extension.definition,
            files,
        })
    }

    pub fn atlas(&self) -> Result<RgbaImage> {
        let count = self.files[&format!(
            "data/tilesets/{}_metatiles.bin",
            crate::GENERATED_TILESET_ID
        )]
            .len()
            / 16;
        let height = count.div_ceil(16);
        let blocks = (0..height * 16)
            .map(|i| if i < count { i as u16 } else { 2 })
            .collect::<Vec<_>>();
        crate::preview::render_blocks(
            16,
            height as u16,
            &blocks,
            crate::GENERATED_TILESET_ID,
            &self.definition,
            &self.files,
            "day",
        )
    }

    pub fn render(&self, grid: &GeneratedGrid, time: &str) -> Result<RgbaImage> {
        ensure!(
            matches!(time, "day" | "night" | "morn"),
            "unknown palette time"
        );
        crate::preview::render_blocks(
            grid.width,
            grid.height,
            &grid.crystal_blocks(),
            crate::GENERATED_TILESET_ID,
            &self.definition,
            &self.files,
            if time == "night" { "nite" } else { time },
        )
    }
}

fn parse_collisions(text: &str) -> Result<BTreeMap<String, Vec<String>>> {
    let mut result = BTreeMap::new();
    for raw in text.lines() {
        let line = raw.split(';').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        let args = line
            .strip_prefix("tilecoll ")
            .context("unsupported collision directive")?;
        let values = args
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>();
        ensure!(values.len() == 4, "collision needs four quadrants");
        result.insert(format!("{:02x}", result.len()), values);
    }
    Ok(result)
}

fn parse_tile_palettes(text: &str) -> Result<Vec<u8>> {
    let names = [
        "GRAY", "RED", "GREEN", "WATER", "YELLOW", "BROWN", "ROOF", "TEXT",
    ];
    let mut result = Vec::new();
    let mut repeat = 1;
    for raw in text.lines() {
        let line = raw.split(';').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        if let Some(count) = line.strip_prefix("rept ") {
            repeat = count.parse::<usize>()?;
            ensure!(repeat <= 256, "oversized palette repetition");
        } else if line == "endr" {
            repeat = 1;
        } else if line == "db $ff" {
            result.extend(std::iter::repeat_n(255, repeat * 2));
        } else if let Some(args) = line.strip_prefix("tilepal ") {
            let mut parts = args.split(',').map(str::trim);
            let bank = parts.next().context("missing tile bank")?.parse::<u8>()?;
            ensure!(bank < 2, "invalid VRAM bank");
            for name in parts {
                let palette = names
                    .iter()
                    .position(|p| *p == name)
                    .context("unknown tile palette")? as u8;
                result.push(palette | bank << 3);
            }
        } else {
            bail!("unsupported palette directive {line}");
        }
    }
    ensure!(
        (128..=256).contains(&result.len()),
        "invalid palette entry count {}",
        result.len()
    );
    result.resize(256, 255);
    Ok(result)
}

fn png_to_2bpp(bytes: &[u8]) -> Result<Vec<u8>> {
    let source = image::load_from_memory(bytes)?.to_rgba8();
    ensure!(
        source.width() % 8 == 0 && source.height() % 8 == 0,
        "unaligned source tiles"
    );
    let mut output = Vec::new();
    for ty in 0..source.height() / 8 {
        for tx in 0..source.width() / 8 {
            for y in 0..8 {
                let (mut low, mut high) = (0, 0);
                for x in 0..8 {
                    let p = source.get_pixel(tx * 8 + x, ty * 8 + y);
                    ensure!(
                        p[0] == p[1] && p[1] == p[2] && p[3] == 255,
                        "source tiles must be opaque grayscale"
                    );
                    let index = match p[0] {
                        255 => 0,
                        170 => 1,
                        85 => 2,
                        0 => 3,
                        _ => bail!("unexpected source shade {}", p[0]),
                    };
                    low |= (index & 1) << (7 - x);
                    high |= ((index >> 1) & 1) << (7 - x);
                }
                output.extend([low, high]);
            }
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn palette_padding_expands_packed_nibbles() {
        let row = "tilepal 0, GRAY, RED, GREEN, WATER, YELLOW, BROWN, ROOF, TEXT\n";
        let source = format!(
            "{}rept 16\ndb $ff\nendr\n{}",
            row.repeat(12),
            row.replace("tilepal 0", "tilepal 1").repeat(16)
        );
        let parsed = parse_tile_palettes(&source).unwrap();
        assert_eq!(&parsed[..8], &[0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(&parsed[96..128], &[255; 32]);
        assert_eq!(parsed[128], 8);
    }
}
