//! Append one native snowy pine without altering any existing visible recipe.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

const SOURCE: &str = "mn_winter_north_shore";
const TARGET: &str = "mn_winter_gunflint";
type Foregrounds = BTreeMap<u8, [[u8; 3]; 3]>;

pub(super) fn append_pine(base: &CompiledGamePack, output: &mut CompiledGamePack) -> Result<()> {
    let recipe_path = |name: &str| format!("data/tilesets/{name}_metatiles.bin");
    let graphics_path = |name: &str| format!("gfx/tilesets/{name}.2bpp");
    let foreground_path = |name: &str| format!("data/tilesets/{name}_foregrounds.json");
    let old = base
        .data
        .tilesets
        .get(TARGET)
        .context("private Gunflint definition")?;
    let donor = base.data.tilesets.get(SOURCE).context("snowy pine donor")?;
    let old_recipe = base
        .runtime_files
        .get(&recipe_path(TARGET))
        .context("Gunflint recipes")?;
    let donor_recipes = base
        .runtime_files
        .get(&recipe_path(SOURCE))
        .context("snowy pine recipes")?;
    let recipe = donor_recipes
        .get(0x8b * 16..0x8c * 16)
        .context("accepted pine recipe8b")?;
    ensure!(
        old_recipe.len() == 0x49 * 16
            && old.collision.len() == 0x49
            && donor.collision["8b"].iter().all(|k| k == "WALL"),
        "private append or donor collision drift"
    );
    let mut definition = old.clone();
    let mut recipes = old_recipe.clone();
    let mut graphics = base
        .runtime_files
        .get(&graphics_path(TARGET))
        .context("Gunflint pixels")?
        .clone();
    let donor_pixels = base
        .runtime_files
        .get(&graphics_path(SOURCE))
        .context("snowy pine pixels")?;
    let mut foregrounds: Foregrounds = serde_json::from_slice(
        base.runtime_files
            .get(&foreground_path(TARGET))
            .context("Gunflint foregrounds")?,
    )?;
    let donor_foregrounds: Foregrounds = serde_json::from_slice(
        base.runtime_files
            .get(&foreground_path(SOURCE))
            .context("pine foregrounds")?,
    )?;
    ensure!(
        graphics.len() == 256 * 16 && donor_pixels.len() == 256 * 16,
        "private native glyph extent changed"
    );
    let glyphs = recipe.iter().copied().collect::<BTreeSet<_>>();
    ensure!(
        glyphs == BTreeSet::from([0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff]),
        "reviewed pine glyph recipe differs"
    );
    let mut replacements = BTreeMap::new();
    for (source, target) in glyphs.into_iter().zip(0x60u8..0x66) {
        ensure!(
            !old_recipe.contains(&target)
                && graphics[usize::from(target) * 16..(usize::from(target) + 1) * 16]
                    .iter()
                    .all(|b| *b == 0),
            "private pine glyph occupied"
        );
        let source_attribute = donor.palette_map[usize::from(source)];
        ensure!(source_attribute == 11, "reviewed pine palette differs");
        let source_slot = usize::from(source & 127) + 128;
        graphics[usize::from(target) * 16..(usize::from(target) + 1) * 16]
            .copy_from_slice(&donor_pixels[source_slot * 16..(source_slot + 1) * 16]);
        // Low-bank unused slots must not alias the old occupied high-bank art.
        definition.palette_map[usize::from(target)] = source_attribute & !8;
        foregrounds.insert(
            target,
            *donor_foregrounds
                .get(&source)
                .context("native pine colors")?,
        );
        replacements.insert(source, target);
    }
    recipes.extend(recipe.iter().map(|g| replacements[g]));
    ensure!(
        definition
            .collision
            .insert("49".into(), vec!["WALL".into(); 4])
            .is_none(),
        "private pine block occupied"
    );
    output
        .data
        .tilesets
        .insert(TARGET.into(), definition.clone());
    for (path, bytes) in [
        (recipe_path(TARGET), recipes),
        (graphics_path(TARGET), graphics.clone()),
        (
            format!("gfx/tilesets/{TARGET}.png"),
            crate::native_tile_png::from_2bpp(&graphics)?,
        ),
        (foreground_path(TARGET), serde_json::to_vec(&foregrounds)?),
        (
            format!("data/tilesets/{TARGET}_palette_map.json"),
            serde_json::to_vec(&definition.palette_map)?,
        ),
        (
            format!("data/tilesets/{TARGET}.json"),
            serde_json::to_vec(&definition.collision)?,
        ),
    ] {
        output.runtime_files.insert(path, bytes);
    }
    Ok(())
}

pub(super) fn paths() -> [String; 6] {
    [
        format!("data/tilesets/{TARGET}_metatiles.bin"),
        format!("gfx/tilesets/{TARGET}.2bpp"),
        format!("gfx/tilesets/{TARGET}.png"),
        format!("data/tilesets/{TARGET}_foregrounds.json"),
        format!("data/tilesets/{TARGET}_palette_map.json"),
        format!("data/tilesets/{TARGET}.json"),
    ]
}
