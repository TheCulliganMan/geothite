//! One dynamically allocated St Anthony Falls glyph; native cave scroll only.

use std::collections::HashMap;

use anyhow::{Context, Result, ensure};
use bevy::asset::{Assets, Handle};
use bevy::image::Image;
use crystal_assets::AssetRoot;

use super::{TilesetAnimatedTile, resolve_tileset_tile_index, scrolled_tileset_animation};

const CITY_TILESET: &str = "mn_minneapolis_city";
const FALLS_BLOCK: usize = 0x99;
const BLOCK_BYTES: usize = 16;

/// Resolves the one repeated, previously unreferenced glyph in the exact append.
/// The former 153-block city has no waterfall and keeps its old animations.
pub(super) fn glyph(tileset: &str, metatiles: &[u8]) -> Result<Option<usize>> {
    if tileset != CITY_TILESET {
        return Ok(None);
    }
    let offset = FALLS_BLOCK * BLOCK_BYTES;
    if metatiles.len() == offset {
        return Ok(None);
    }
    ensure!(
        metatiles.len() >= offset + BLOCK_BYTES && metatiles.len().is_multiple_of(BLOCK_BYTES),
        "Minneapolis waterfall metatile extent differs"
    );
    let imported = usize::from(metatiles[offset]);
    ensure!(
        metatiles[offset..offset + BLOCK_BYTES]
            .iter()
            .all(|&value| usize::from(value) == imported)
            && !metatiles[..offset].contains(&(imported as u8)),
        "Minneapolis waterfall must use one newly allocated glyph"
    );
    // Static overlays can import additional native facade glyphs. Their pack
    // loader owns graphics/bank validation; this animation binding only needs
    // to keep the isolated waterfall glyph out of every later static recipe.
    ensure!(
        metatiles[offset + BLOCK_BYTES..]
            .iter()
            .all(|&glyph| usize::from(glyph) != imported),
        "static Minneapolis blocks must not reuse the waterfall glyph"
    );
    Ok(Some(imported))
}

/// Registers only the certified native cave waterfall copy. Invalid appends fail
/// art loading rather than assigning a waterfall schedule to unrelated pixels.
pub(super) fn register(
    root: &AssetRoot,
    tileset: &str,
    metatiles: &[u8],
    palette_map: &[u8],
    handles: &[Handle<Image>],
    images: &mut Assets<Image>,
    animated: &mut HashMap<usize, TilesetAnimatedTile>,
) -> Result<()> {
    let Some(imported) = glyph(tileset, metatiles)? else {
        return Ok(());
    };
    let source = handles
        .get(imported)
        .context("Minneapolis waterfall rendered glyph")?;
    let palette = *palette_map
        .get(imported)
        .context("Minneapolis waterfall palette")?;
    let assets = root.runtime_assets();
    // These are existing pack assets, not a second exported source catalog.
    // Metatiles are passed by the caller; never read its already-loaded file.
    let graphics = crate::read_runtime_asset(assets.join(format!("gfx/tilesets/{tileset}.2bpp")))?;
    ensure!(
        graphics.len() % 16 == 0,
        "Minneapolis glyph bytes are misaligned"
    );
    let slot = resolve_tileset_tile_index(graphics.len() / 16, imported, (palette >> 3) & 1);
    let imported_pixels = graphics
        .get(slot * 16..(slot + 1) * 16)
        .context("Minneapolis waterfall source pixels")?;
    let cave = crate::read_runtime_asset(assets.join("gfx/tilesets/cave.2bpp"))?;
    let native_pixels = cave
        .get(0x40 * 16..0x41 * 16)
        .context("native cave waterfall pixels")?;
    ensure!(
        imported_pixels == native_pixels,
        "Minneapolis waterfall differs from native cave glyph40"
    );
    ensure!(
        !animated.contains_key(&imported),
        "Minneapolis waterfall glyph already animated"
    );
    // ScrollTileDown on the cave glyph: 3 pixels, timer19, operation slot16.
    animated.insert(
        imported,
        scrolled_tileset_animation(source, false, 3, 19, 16, images)?,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::glyph;

    #[test]
    fn exact_append_resolves_its_glyph_and_legacy_is_unchanged() {
        let mut layout = vec![0; 0x99 * 16];
        assert_eq!(glyph("mn_minneapolis_city", &layout).expect("legacy"), None);
        layout.extend([0x81; 16]);
        assert_eq!(
            glyph("mn_minneapolis_city", &layout).expect("append"),
            Some(0x81)
        );
        layout[0x99 * 16..].fill(0xa7);
        assert_eq!(
            glyph("mn_minneapolis_city", &layout).expect("different allocation"),
            Some(0xa7)
        );
        assert_eq!(glyph("johto_modern", &layout).expect("unrelated"), None);
    }

    #[test]
    fn later_bridge_copies_keep_the_original_waterfall_allocation() {
        let mut layout = vec![0; 0x99 * 16];
        layout.extend([0xa7; 16]);
        layout.extend([0; 16]);
        assert_eq!(
            glyph("mn_minneapolis_city", &layout).expect("native bridge copy"),
            Some(0xa7)
        );
        let last = layout.len() - 1;
        layout[last] = 0xa7;
        assert!(
            glyph("mn_minneapolis_city", &layout).is_err(),
            "bridge must not reuse the waterfall glyph"
        );
    }

    #[test]
    fn composed_and_imported_static_facades_keep_the_waterfall_glyph_isolated() {
        let mut layout = vec![0; 0x99 * 16];
        layout[16..32].fill(7);
        layout.extend([0xa7; 16]);
        layout.extend([0, 7, 0, 7, 7, 0, 7, 0, 0, 7, 0, 7, 7, 0, 7, 0]);
        assert_eq!(
            glyph("mn_minneapolis_city", &layout).expect("composed native wall"),
            Some(0xa7)
        );
        let last = layout.len() - 1;
        layout[last] = 8;
        assert_eq!(
            glyph("mn_minneapolis_city", &layout).expect("new static native glyph"),
            Some(0xa7)
        );
        layout[last] = 0xa7;
        assert!(
            glyph("mn_minneapolis_city", &layout).is_err(),
            "a static facade must not acquire the waterfall's animation"
        );
    }

    #[test]
    fn malformed_or_reused_appends_are_rejected() {
        let mut layout = vec![0; 0x9a * 16];
        assert!(
            glyph("mn_minneapolis_city", &layout).is_err(),
            "reused original glyph"
        );
        layout[0x99 * 16..].fill(0xa7);
        layout[0x99 * 16 + 5] = 0xa8;
        assert!(
            glyph("mn_minneapolis_city", &layout).is_err(),
            "mixed recipe"
        );
        layout[0x99 * 16 + 5] = 0xa7;
        layout.pop();
        assert!(
            glyph("mn_minneapolis_city", &layout).is_err(),
            "partial recipe"
        );
        layout.extend([0xa7; 17]);
        assert!(
            glyph("mn_minneapolis_city", &layout).is_err(),
            "undeclared additional block"
        );
    }
}
