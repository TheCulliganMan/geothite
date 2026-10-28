//! Native font chevrons painted on the supplied wooden wind-lane pixels.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

pub(crate) struct WindGlyph {
    pub pixels: [u8; 16],
    pub palette: u8,
}

/// Produces independent glyphs so opposing lanes cannot overwrite shared slats.
pub(crate) fn mark(
    pack: &CompiledGamePack,
    graphics: &[u8],
    palettes: &[u8],
    metatile: &[u8],
    wind: &str,
) -> Result<Vec<WindGlyph>> {
    ensure!(metatile.len() == 16, "one native wind metatile required");
    let right = match wind {
        "WALK_RIGHT" => true,
        "WALK_LEFT" => false,
        _ => anyhow::bail!("marked wind requires a horizontal direction"),
    };
    // The native character map assigns ▶ to ED, font tile ED - 80 = 6D.
    // Reflect those same source pixels for the matching west-pointing sign.
    let arrow = pack
        .runtime_files()
        .get("gfx/font/font.1bpp")
        .context("native arrow font")?
        .get(0x6d * 8..0x6e * 8)
        .context("native right-arrow glyph")?;
    metatile
        .iter()
        .enumerate()
        .map(|(index, tile)| {
            let palette = palettes[usize::from(*tile)];
            let physical = if palette & 8 == 0 {
                usize::from(*tile)
            } else {
                usize::from(*tile & 0x7f) + graphics.len() / 32
            };
            let mut pixels: [u8; 16] = graphics
                .get(physical * 16..(physical + 1) * 16)
                .context("native slat glyph")?
                .try_into()?;
            for y in 0..8 {
                for x in 0..8 {
                    let px = (index % 4 * 8 + x) % 16;
                    let py = (index / 4 * 8 + y) % 16;
                    if (4..12).contains(&px) && (4..12).contains(&py) {
                        let sx = if right { px - 4 } else { 11 - px };
                        let ink = arrow[py - 4] & (1 << (7 - sx)) != 0;
                        let mask = 1 << (7 - x);
                        for plane in 0..2 {
                            pixels[y * 2 + plane] =
                                (pixels[y * 2 + plane] & !mask) | if ink { mask } else { 0 };
                        }
                    }
                }
            }
            Ok(WindGlyph {
                pixels,
                palette: palette & 7,
            })
        })
        .collect()
}
