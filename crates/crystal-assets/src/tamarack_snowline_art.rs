//! Native glyph and palette composition for the bound Tamarack melt band.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::{CompiledGamePack, CompiledTilesetExtension};

type Inks = [[u8; 3]; 3];

#[derive(Deserialize)]
struct Recipes {
    duplicated_static_glyphs: BTreeMap<u8, u8>,
    cold_variants: BTreeMap<String, String>,
    melted_glyphs: BTreeMap<u8, u8>,
    melt_variants: BTreeMap<String, String>,
    melt_static_sources: BTreeMap<u8, u8>,
}

fn bytes(pack: &CompiledGamePack, path: &str) -> Result<Vec<u8>> {
    pack.runtime_files
        .get(path)
        .cloned()
        .with_context(|| format!("native source asset {path}"))
}

fn physical(glyph: u8, palette: u8) -> usize {
    if palette & 8 == 0 {
        usize::from(glyph)
    } else {
        usize::from(glyph & 127) + 128
    }
}

struct Glyphs {
    graphics: Vec<u8>,
    palettes: Vec<u8>,
    foregrounds: BTreeMap<u8, Inks>,
    backgrounds: BTreeMap<u8, u8>,
    protected: BTreeSet<usize>,
    written: BTreeSet<u8>,
}

impl Glyphs {
    fn duplicate(
        &mut self,
        source: u8,
        target: u8,
        melt: bool,
        grass: &BTreeSet<u8>,
    ) -> Result<()> {
        ensure!(
            !self.protected.contains(&usize::from(target)),
            "native glyph address is protected"
        );
        let palette = self.palettes[usize::from(source)];
        let address = physical(source, palette) * 16;
        let mut raw: [u8; 16] = self.graphics[address..address + 16]
            .try_into()
            .context("native glyph")?;
        let inks = self.foregrounds.get(&source).copied();
        if melt
            && !grass.contains(&source)
            && inks.is_some_and(|colors| colors[0].iter().all(|&value| value >= 230))
        {
            // Native white snow backing becomes native clock-lit green. Never
            // remap grass: every original priority hole remains at index zero.
            for row in 0..8 {
                raw[row * 2] &= raw[row * 2 + 1];
            }
        }
        let destination = usize::from(target) * 16;
        let target_palette = (palette & 7) | if target >= 128 { 8 } else { 0 };
        if self.written.contains(&target) {
            ensure!(
                self.graphics[destination..destination + 16] == raw
                    && self.palettes[usize::from(target)] == target_palette
                    && self.foregrounds.get(&target).copied() == inks,
                "shared native glyph recipe differs"
            );
        } else {
            ensure!(
                self.graphics[destination..destination + 16]
                    .iter()
                    .all(|&value| value == 0)
                    && !self.foregrounds.contains_key(&target)
                    && !self.backgrounds.contains_key(&target),
                "new glyph slot is occupied"
            );
            self.graphics[destination..destination + 16].copy_from_slice(&raw);
            self.palettes[usize::from(target)] = target_palette;
            if let Some(colors) = inks {
                self.foregrounds.insert(target, colors);
            }
            self.written.insert(target);
        }
        if melt {
            self.backgrounds.insert(target, 0);
        }
        Ok(())
    }
}

/// Installs a bound native cold/melt pair without changing existing art.
/// The caller must validate the complete source and the allowed alias names.
pub(crate) fn install_with_aliases(
    pack: &mut CompiledGamePack,
    specification: &Value,
    result: &Value,
    source: &str,
    full: &str,
    thaw: &str,
    manifest: &str,
) -> Result<()> {
    ensure!(
        specification["source"] == source
            && specification["target"] == full
            && specification["melt_white_ink_min"] == 230,
        "explicit bound native snowline recipe required"
    );
    crate::inland_winter_belt::install_source_bound_alias(pack, specification)?;
    let recipes: Recipes = serde_json::from_value(result.clone())?;
    let original = pack
        .data
        .tilesets
        .get(source)
        .context("native snowline source definition")?
        .clone();
    let mut definition = pack
        .data
        .tilesets
        .get(full)
        .context("full snow definition")?
        .clone();
    let original_layout = bytes(pack, &format!("data/tilesets/{source}_metatiles.bin"))?;
    let full_layout = bytes(pack, &format!("data/tilesets/{full}_metatiles.bin"))?;
    let animations: BTreeMap<u8, Value> = pack
        .runtime_files
        .get(&format!("data/tilesets/{source}_animations.json"))
        .map(|raw| serde_json::from_slice(raw))
        .transpose()?
        .unwrap_or_default();
    let animated = animations
        .keys()
        .copied()
        .chain([3, 0x14])
        .collect::<BTreeSet<_>>();
    let foregrounds: BTreeMap<u8, Inks> = serde_json::from_slice(&bytes(
        pack,
        &format!("data/tilesets/{full}_foregrounds.json"),
    )?)?;
    let original_fg: BTreeMap<u8, Inks> = pack
        .runtime_files
        .get(&format!("data/tilesets/{source}_foregrounds.json"))
        .map(|raw| serde_json::from_slice(raw))
        .transpose()?
        .unwrap_or_default();
    let full_backgrounds = animated
        .iter()
        .map(|&glyph| (glyph, original.palette_map[usize::from(glyph)] & 7))
        .collect::<BTreeMap<_, _>>();
    let original_used = original_layout
        .iter()
        .copied()
        .chain(animated.iter().copied())
        .collect::<BTreeSet<_>>();
    let backgrounds = original_used
        .iter()
        .map(|&glyph| {
            (
                glyph,
                if original_fg.contains_key(&glyph) {
                    0
                } else {
                    original.palette_map[usize::from(glyph)] & 7
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let used = original_used
        .iter()
        .copied()
        .chain(full_layout.iter().copied())
        .chain(foregrounds.keys().copied())
        .collect::<BTreeSet<_>>();
    let protected = used
        .iter()
        .flat_map(|&glyph| {
            [
                usize::from(glyph),
                physical(glyph, definition.palette_map[usize::from(glyph)]),
            ]
        })
        .collect();
    let mut art = Glyphs {
        graphics: bytes(pack, &format!("gfx/tilesets/{full}.2bpp"))?,
        palettes: definition.palette_map.clone(),
        foregrounds,
        backgrounds,
        protected,
        written: BTreeSet::new(),
    };
    ensure!(
        art.graphics.len() == 4096
            && art.palettes.len() == 256
            && original_layout.len() == 134 * 16
            && full_layout.len() == original_layout.len(),
        "complete native source extents required"
    );
    let placed: Vec<u8> = serde_json::from_value(specification["placed_blocks"].clone())?;
    let mut grass = BTreeSet::new();
    for block in placed {
        let collision = original
            .collision
            .get(&format!("{block:02x}"))
            .context("native habitat collision")?;
        ensure!(
            collision.len() == 4,
            "four native habitat quadrants required"
        );
        for index in 0..16 {
            let quadrant = (index / 8) * 2 + (index % 4) / 2;
            if collision[quadrant] == "TALL_GRASS" {
                grass.insert(
                    *full_layout
                        .get(usize::from(block) * 16 + index)
                        .context("native habitat glyph")?,
                );
            }
        }
    }
    for (&source, &target) in &recipes.duplicated_static_glyphs {
        art.duplicate(source, target, false, &grass)?;
    }
    for (&source, &target) in &recipes.melted_glyphs {
        art.duplicate(source, target, true, &grass)?;
    }
    let mut layout = original_layout.clone();
    for (variants, melt) in [
        (&recipes.cold_variants, false),
        (&recipes.melt_variants, true),
    ] {
        for (source, target) in variants {
            let source_block = usize::from(u8::from_str_radix(source, 16)?);
            let target_block = usize::from(u8::from_str_radix(target, 16)?);
            ensure!(
                layout.len() / 16 == target_block,
                "cold variants must append in reviewed order"
            );
            let parts = if melt {
                let cold = usize::from(u8::from_str_radix(
                    recipes
                        .cold_variants
                        .get(source)
                        .context("matching cold block")?,
                    16,
                )?);
                layout
                    .get(cold * 16..(cold + 1) * 16)
                    .context("cold metatile")?
                    .to_vec()
            } else {
                full_layout
                    .get(source_block * 16..(source_block + 1) * 16)
                    .context("full snow metatile")?
                    .to_vec()
            };
            for glyph in parts {
                let target_glyph = if !melt {
                    recipes
                        .duplicated_static_glyphs
                        .get(&glyph)
                        .copied()
                        .unwrap_or(glyph)
                } else if art.backgrounds.contains_key(&glyph) {
                    glyph
                } else if let Some(&original) = recipes.melt_static_sources.get(&glyph) {
                    original
                } else {
                    *recipes
                        .melted_glyphs
                        .get(&glyph)
                        .context("melt foreground glyph recipe")?
                };
                layout.push(target_glyph);
            }
            definition.collision.insert(
                target.clone(),
                original
                    .collision
                    .get(source)
                    .context("original cold collision")?
                    .clone(),
            );
        }
    }
    ensure!(
        layout[..original_layout.len()] == original_layout
            && original_used
                .iter()
                .all(|&glyph| art.palettes[usize::from(glyph)]
                    == original.palette_map[usize::from(glyph)]),
        "native glyph and metatile prefix changed"
    );
    definition.palette_map = art.palettes;
    let extension = CompiledTilesetExtension {
        manifest_id: manifest.into(),
        tileset_id: thaw.into(),
        behavior_source_tileset_id: Some(source.into()),
        definition,
        metatiles: layout,
        tile_graphics_png: crate::native_tile_png::from_2bpp(&art.graphics)?,
        tile_graphics_2bpp: art.graphics,
    };
    crate::validate_compiled_tileset_extension(&extension)?;
    crate::insert_compiled_tileset_extension(pack, extension)?;
    let palette = bytes(pack, &format!("gfx/tilesets/{full}.pal"))?;
    pack.runtime_files
        .insert(format!("gfx/tilesets/{thaw}.pal"), palette);
    pack.runtime_files.insert(
        format!("data/tilesets/{thaw}_foregrounds.json"),
        serde_json::to_vec(&art.foregrounds)?,
    );
    pack.runtime_files.insert(
        format!("data/tilesets/{thaw}_native_backgrounds.json"),
        serde_json::to_vec(&art.backgrounds)?,
    );
    pack.runtime_files.insert(
        format!("data/tilesets/{full}_native_backgrounds.json"),
        serde_json::to_vec(&full_backgrounds)?,
    );
    Ok(())
}
