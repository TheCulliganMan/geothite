//! Preserve native shore quadrants while closing outward water with real reefs.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::{CompiledGamePack, CompiledTilesetExtension};

#[derive(Deserialize)]
struct Recipe {
    id: String,
    base: String,
    quadrants: Vec<Quadrant>,
}
#[derive(Deserialize)]
struct Quadrant {
    index: usize,
    source: String,
}

pub(crate) fn install(base: &CompiledGamePack, source: &Value) -> Result<CompiledGamePack> {
    ensure!(
        source["tileset"] == "mn_marais_shore",
        "bounded Marais art identity"
    );
    let recipes: Vec<Recipe> = serde_json::from_value(source["reef_recipes"].clone())?;
    ensure!(
        recipes.len() == 10,
        "ten declared native reef variants required"
    );
    let mut definition = base
        .data
        .tilesets
        .get("johto")
        .context("native Johto")?
        .clone();
    let original = base
        .runtime_files
        .get("data/tilesets/johto_metatiles.bin")
        .context("native Johto metatiles")?;
    ensure!(original.len() == 128 * 16, "original Johto block extent");
    let mut metatiles = original.clone();
    for (i, recipe) in recipes.iter().enumerate() {
        let id = u16::from_str_radix(&recipe.id, 16)?;
        let b = usize::from(u8::from_str_radix(&recipe.base, 16)?);
        ensure!(
            id == 128 + i as u16 && b < 128 && !recipe.quadrants.is_empty(),
            "sequential original reef recipe"
        );
        let mut block = original[b * 16..(b + 1) * 16].to_vec();
        let mut kinds = definition
            .collision
            .get(&recipe.base)
            .context("native base collision")?
            .clone();
        let mut used = [false; 4];
        for q in &recipe.quadrants {
            ensure!(
                q.index < 4 && !used[q.index] && kinds[q.index] == "WATER",
                "only distinct water quadrants may gain reefs"
            );
            used[q.index] = true;
            ensure!(
                matches!(q.source.as_str(), "34" | "36" | "39"),
                "original reef edge reference"
            );
            let reef = usize::from(u8::from_str_radix(&q.source, 16)?);
            ensure!(
                definition.collision[&q.source][q.index] == "BUOY",
                "native reef must actually block Surf"
            );
            for dy in 0..2 {
                for dx in 0..2 {
                    let glyph = (q.index / 2 * 2 + dy) * 4 + q.index % 2 * 2 + dx;
                    block[glyph] = original[reef * 16 + glyph];
                }
            }
            kinds[q.index] = "BUOY".into();
        }
        metatiles.extend(block);
        ensure!(
            definition
                .collision
                .insert(recipe.id.clone(), kinds)
                .is_none(),
            "unused reef address"
        );
    }
    // Every copied glyph already belongs to Johto and keeps its palette index.
    // No source pixel, original block, graphic byte or animation is changed.
    base.with_tileset_extension(CompiledTilesetExtension {
        manifest_id: "minnesota-marais-shore-art-v1".into(),
        tileset_id: "mn_marais_shore".into(),
        behavior_source_tileset_id: Some("johto".into()),
        definition,
        metatiles,
        tile_graphics_2bpp: base
            .runtime_files
            .get("gfx/tilesets/johto.2bpp")
            .context("Johto graphics")?
            .clone(),
        tile_graphics_png: base
            .runtime_files
            .get("gfx/tilesets/johto.png")
            .context("Johto PNG")?
            .clone(),
    })
}
