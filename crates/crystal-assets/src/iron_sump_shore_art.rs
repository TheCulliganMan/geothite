//! Exact native water/rock quadrants for the sump's closed southern bank.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::{CompiledGamePack, CompiledTilesetExtension};

pub(crate) const ALIAS: &str = "mn_iron_sump_rock";
const SOURCE: &str = "mn_iron_haulage_rock";

pub(crate) fn install(base: &CompiledGamePack, design: &Value) -> Result<CompiledGamePack> {
    let mut recipes = Vec::new();
    for (id, water) in [("43", "26"), ("44", "25")] {
        recipes.push(json!({"id":id,"source_tileset":SOURCE,"source_block":water,
            "tile_quadrant_sources":[
                {"source_block":water,"source_quadrant":0},
                {"source_block":water,"source_quadrant":1},
                {"source_block":"21","source_quadrant":2},
                {"source_block":"21","source_quadrant":3}],
            "collision":["WATER","WATER","WALL","WALL"]}));
    }
    ensure!(
        design["base_tileset"] == SOURCE && design["derived_blocks"] == json!(recipes),
        "exact native sump shore recipes required"
    );
    let mut definition = base
        .data
        .tilesets
        .get(SOURCE)
        .context("installed mine cave art")?
        .clone();
    let mut metatiles = base
        .runtime_files
        .get(&format!("data/tilesets/{SOURCE}_metatiles.bin"))
        .context("mine metatiles")?
        .clone();
    ensure!(
        metatiles.len() == 0x43 * 16
            && definition.collision["0d"].iter().all(|k| k == "FLOOR")
            && definition.collision["42"]
                .iter()
                .map(String::as_str)
                .eq(["FLOOR", "LADDER", "FLOOR", "FLOOR"]),
        "reviewed mine cave alias required"
    );
    for (id, water) in [(0x43_usize, 0x26_usize), (0x44, 0x25)] {
        let mut glyphs = [0_u8; 16];
        for quadrant in 0..4 {
            let source = if quadrant < 2 { water } else { 0x21 };
            let collision = &definition.collision[&format!("{source:02x}")][quadrant];
            ensure!(
                collision == if quadrant < 2 { "WATER" } else { "WALL" },
                "native shore quadrant collision differs"
            );
            for row in 0..2 {
                for column in 0..2 {
                    let at = (quadrant / 2 * 2 + row) * 4 + quadrant % 2 * 2 + column;
                    glyphs[at] = metatiles[source * 16 + at];
                }
            }
        }
        metatiles.extend(glyphs);
        ensure!(
            definition
                .collision
                .insert(
                    format!("{id:02x}"),
                    ["WATER", "WATER", "WALL", "WALL"]
                        .map(String::from)
                        .to_vec()
                )
                .is_none(),
            "occupied sump shore block"
        );
    }
    base.with_tileset_extension(CompiledTilesetExtension {
        manifest_id: "minnesota-iron-sump-native-shore-v1".into(),
        tileset_id: ALIAS.into(),
        behavior_source_tileset_id: Some("cave".into()),
        definition,
        metatiles,
        tile_graphics_png: base
            .runtime_files
            .get(&format!("gfx/tilesets/{SOURCE}.png"))
            .context("native mine PNG")?
            .clone(),
        tile_graphics_2bpp: base
            .runtime_files
            .get(&format!("gfx/tilesets/{SOURCE}.2bpp"))
            .context("native mine graphics")?
            .clone(),
    })
}
