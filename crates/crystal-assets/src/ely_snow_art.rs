//! Private Ely palette and an exact import of the installed snow-covered pines.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, CompiledTilesetExtension};

pub(crate) fn append(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    const TARGET: &str = "mn_ely_snow";
    let files = &base.runtime_files;
    let old = &files["data/tilesets/kanto_metatiles.bin"];
    let mut graphics = files["gfx/tilesets/kanto.2bpp"].clone();
    ensure!(
        old.len() == 131 * 16 && graphics.len() == 96 * 16 && old.iter().all(|glyph| *glyph < 96),
        "unused native Kanto slots required"
    );
    // At 128+ glyphs the native reader splits two equal VRAM banks.
    // Keep a complete 128-glyph bank0 so old Kanto addresses60..95 remain valid.
    graphics.resize(256 * 16, 0);
    let source = &files["gfx/tilesets/mn_winter_north_shore.2bpp"];
    let foregrounds: BTreeMap<u8, [[u8; 3]; 3]> =
        serde_json::from_slice(&files["data/tilesets/mn_winter_north_shore_foregrounds.json"])?;
    let mut own_foregrounds = BTreeMap::new();
    let mut definition = base.data.tilesets["kanto"].clone();
    for (target, original) in (96u8..102).zip(250u8..=255) {
        let to = usize::from(target) * 16;
        let from = usize::from(original) * 16;
        graphics[to..to + 16]
            .copy_from_slice(source.get(from..from + 16).context("native pine glyph")?);
        definition.palette_map[usize::from(target)] = 0;
        own_foregrounds.insert(
            target,
            *foregrounds.get(&original).context("native pine colors")?,
        );
    }
    let mut recipes = old.clone();
    recipes.extend([
        96, 97, 96, 97, 100, 101, 100, 101, 100, 101, 100, 101, 98, 99, 98, 99,
    ]);
    ensure!(
        definition
            .collision
            .insert("83".into(), vec!["WALL".into(); 4])
            .is_none(),
        "new pine slot occupied"
    );
    let mut output = base.with_tileset_extension(CompiledTilesetExtension {
        manifest_id: "minnesota-ely-shore-pass-v1-snow".into(),
        tileset_id: TARGET.into(),
        behavior_source_tileset_id: Some("kanto".into()),
        definition,
        metatiles: recipes,
        tile_graphics_png: crate::native_tile_png::from_2bpp(&graphics)?,
        tile_graphics_2bpp: graphics,
    })?;
    output.runtime_files.insert(
        format!("data/tilesets/{TARGET}_foregrounds.json"),
        serde_json::to_vec(&own_foregrounds)?,
    );
    let palette = std::str::from_utf8(&files["gfx/tilesets/bg_tiles.pal"])?;
    let day = palette
        .split("; day\n")
        .nth(1)
        .context("day bank")?
        .split("; nite")
        .next()
        .context("day colors")?;
    let mut rows = day.trim().lines().take(8).collect::<Vec<_>>();
    ensure!(rows.len() == 8, "complete native bank");
    rows[2] = " RGB 31,31,31, 27,29,31, 14,20,23, 07,07,07 ; snowy native green shapes";
    output.runtime_files.insert(
        format!("gfx/tilesets/{TARGET}.pal"),
        format!("{}\n", rows.join("\n")).into_bytes(),
    );
    Ok(output)
}
