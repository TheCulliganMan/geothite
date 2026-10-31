//! Private native cave art preserving the ventilation callback's numeric floor.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::{CompiledGamePack, CompiledTilesetExtension};

pub(crate) const ALIAS: &str = "mn_iron_vent_rock";
const SOURCE: &str = "mn_iron_haulage_rock";

pub(crate) fn install(base: &CompiledGamePack, design: &Value) -> Result<CompiledGamePack> {
    ensure!(
        design["base_tileset"] == SOURCE
            && design["block_overrides"]
                == json!([{
                    "id":"07", "source_tileset":SOURCE, "source_block":"0d",
                    "collision":["FLOOR","FLOOR","FLOOR","FLOOR"],
                    "reason":"Retain original ventilation callback block7 as ordinary floor in the private alias."
                }]),
        "exact ventilation callback floor recipe required"
    );
    let mut definition = base
        .data
        .tilesets
        .get(SOURCE)
        .context("reviewed native mine art")?
        .clone();
    let mut metatiles = base
        .runtime_files
        .get(&format!("data/tilesets/{SOURCE}_metatiles.bin"))
        .context("native mine metatiles")?
        .clone();
    ensure!(
        metatiles.len() == 0x43 * 16
            && definition.collision["0d"].iter().all(|k| k == "FLOOR")
            && definition.collision["42"]
                .iter()
                .map(String::as_str)
                .eq(["FLOOR", "LADDER", "FLOOR", "FLOOR"]),
        "reviewed mine native floor and shaft quadrants required"
    );
    let floor: [u8; 16] = metatiles[13 * 16..14 * 16].try_into()?;
    metatiles[7 * 16..8 * 16].copy_from_slice(&floor);
    definition
        .collision
        .insert("07".into(), vec!["FLOOR".into(); 4]);
    base.with_tileset_extension(CompiledTilesetExtension {
        manifest_id: "minnesota-iron-ventilation-native-rock-v1".into(),
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
