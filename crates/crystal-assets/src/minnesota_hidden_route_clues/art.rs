use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};

use super::Cache;
use crate::CompiledGamePack;
use crate::northstar_trail_caches::{Landmark, install_art};

/// Append missing artwork, or verify the complete existing stamp on a sibling release.
pub(super) fn install(
    pack: &mut CompiledGamePack,
    tileset: &str,
    placed: u16,
    cache: &Cache,
    source: &[u8],
) -> Result<u16> {
    let [mut design]: [Landmark; 1] = serde_json::from_slice(source)?;
    ensure!(
        design.tileset == tileset
            && u16::from(design.block) == cache.to_block
            && u16::from(design.collision_from) == cache.from_block,
        "unexpected authored clue for {tileset}"
    );
    design.validate_rows()?;
    let path = format!("data/tilesets/{tileset}_metatiles.bin");
    let blocks = pack.runtime_files.get(&path).context("clue metatiles")?;
    ensure!(
        blocks.len().is_multiple_of(16),
        "incomplete metatile recipe"
    );
    let missing = placed == cache.from_block;
    let block = if missing {
        u8::try_from(blocks.len() / 16)?
    } else {
        u8::try_from(placed)?
    };
    design.block = block;
    if missing {
        install_art(
            pack,
            tileset,
            std::slice::from_ref(&design),
            cache.first_glyph,
        )?;
    }

    let definition = &pack.data.tilesets[tileset];
    let blocks = &pack.runtime_files[&path];
    let graphics = pack
        .runtime_files
        .get(&format!("gfx/tilesets/{tileset}.2bpp"))
        .context("clue graphics")?;
    let lookup = |glyph: u8| -> Result<[u8; 16]> {
        let palette = definition
            .palette_map
            .get(usize::from(glyph))
            .context("clue palette")?;
        let address = if palette & 8 == 0 {
            usize::from(glyph)
        } else {
            graphics.len() / 32 + usize::from(glyph & 127)
        };
        let start = address * 16;
        Ok(graphics
            .get(start..start + 16)
            .context("clue outside native graphics")?
            .try_into()?)
    };
    let recipe = |id: u8| -> Result<&[u8]> {
        let start = usize::from(id) * 16;
        blocks
            .get(start..start + 16)
            .context("clue outside metatile table")
    };
    let base = recipe(design.background)?;
    let result = recipe(block)?;
    ensure!(
        definition
            .collision
            .get(&format!("{block:02x}"))
            .context("clue collision")?
            == &definition.collision[&format!("{:02x}", design.collision_from)],
        "clue changed collision"
    );
    let animation_path = format!("data/tilesets/{tileset}_animations.json");
    let animated: BTreeMap<u8, serde_json::Value> = pack
        .runtime_files
        .get(&animation_path)
        .map(|bytes| serde_json::from_slice(bytes))
        .transpose()?
        .unwrap_or_default();
    let mut colors = BTreeMap::new();
    for (i, (&original, &destination)) in base.iter().zip(result).enumerate() {
        let Some(pixels) = design.painted_glyph(i, lookup(original)?) else {
            ensure!(
                original == destination,
                "clue background differs on {tileset}"
            );
            continue;
        };
        let donor = design.foreground.unwrap_or(original);
        let expected = cache
            .first_glyph
            .checked_add(u8::try_from(colors.len())?)
            .context("clue glyph overflow")?;
        ensure!(
            destination == expected
                && !animated.contains_key(&destination)
                && lookup(destination)? == pixels
                && definition.palette_map[usize::from(destination)]
                    == 8 | (definition.palette_map[usize::from(donor)] & 7),
            "clue pixels, animation or palette differ on {tileset} at {i}"
        );
        colors.insert(destination, donor);
    }
    let offered = colors.keys().copied().collect::<BTreeSet<_>>();
    ensure!(
        blocks
            .as_chunks::<16>()
            .0
            .iter()
            .enumerate()
            .all(|(i, recipe)| i == usize::from(block)
                || recipe.iter().all(|id| !offered.contains(id))),
        "clue glyphs are shared by other {tileset} metatiles"
    );

    // The underlying ground follows the clock on the thawed routes. A stamp needs
    // the same background color as its donor, including on already painted packs.
    let path = format!("data/tilesets/{tileset}_native_backgrounds.json");
    if let Some(bytes) = pack.runtime_files.get_mut(&path) {
        let mut backgrounds: BTreeMap<u8, u8> = serde_json::from_slice(bytes)?;
        let original = backgrounds.clone();
        for (dest, donor) in colors {
            if let Some(&value) = original.get(&donor) {
                backgrounds.insert(dest, value);
            } else {
                backgrounds.remove(&dest);
            }
        }
        if original != backgrounds {
            *bytes = serde_json::to_vec(&backgrounds)?;
        }
    }
    Ok(u16::from(block))
}
