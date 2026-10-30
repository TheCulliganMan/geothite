//! Native boot text uses both the regular and extra Crystal font tiles.
use std::collections::HashMap;

use anyhow::{Context, Result};
use crystal_assets::AssetRoot;
use image::RgbaImage;

use super::{bitmap_font_char_map, name_entry_token_tile};

pub(super) trait BootFontSource {
    fn token_tile(&self, token: &str) -> Option<u8> {
        name_entry_token_tile(token)
    }

    fn glyph(&self, tile: u8) -> Result<(&RgbaImage, usize)>;
}

impl BootFontSource for RgbaImage {
    fn glyph(&self, tile: u8) -> Result<(&RgbaImage, usize)> {
        let index = tile
            .checked_sub(0x80)
            .context("glyph is outside font.png")?;
        Ok((self, usize::from(index)))
    }
}

pub(super) struct BootFont {
    normal: RgbaImage,
    extra: RgbaImage,
    glyphs: HashMap<char, u16>,
}

impl BootFont {
    pub(super) fn load(root: &AssetRoot) -> Result<Self> {
        let assets = root.runtime_assets();
        let normal = crate::open_runtime_image(assets.join("gfx/font/font.png"))
            .context("decode boot font PNG")?
            .to_rgba8();
        let extra = crate::open_runtime_image(assets.join("gfx/font/font_extra.png"))
            .context("decode boot extra font PNG")?
            .to_rgba8();
        anyhow::ensure!(
            normal.dimensions() == (128, 64),
            "invalid boot font dimensions"
        );
        anyhow::ensure!(
            extra.dimensions() == (128, 16),
            "invalid extra font dimensions"
        );
        Ok(Self {
            normal,
            extra,
            glyphs: bitmap_font_char_map(),
        })
    }
}

impl BootFontSource for BootFont {
    fn token_tile(&self, token: &str) -> Option<u8> {
        name_entry_token_tile(token).or_else(|| {
            let mut chars = token.chars();
            let ch = chars.next()?;
            if chars.next().is_some() {
                return None;
            }
            u8::try_from(*self.glyphs.get(&ch)?).ok()
        })
    }

    fn glyph(&self, tile: u8) -> Result<(&RgbaImage, usize)> {
        if (0x63..=0x78).contains(&tile) {
            return Ok((&self.extra, usize::from(tile - 0x60)));
        }
        self.normal.glyph(tile)
    }
}
