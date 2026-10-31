//! Private native cave art for the haulage floor's retained numeric scripts.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::CompiledGamePack;

pub(crate) const ALIAS: &str = "mn_iron_haulage_rock";

pub(crate) fn install(base: &CompiledGamePack, design: &Value) -> Result<CompiledGamePack> {
    ensure!(
        design["extended_graphics"] == true,
        "private expanded cave bank required"
    );
    let wall = vec!["WALL"; 4];
    let shutter_order = [2, 3, 2, 3, 6, 7, 6, 7, 10, 11, 10, 11, 14, 15, 14, 15];
    let quadrants: Vec<_> = [0, 3, 2, 1]
        .into_iter()
        .map(|q| {
            json!({
                "source_block":"0b", "source_quadrant":q
            })
        })
        .collect();
    ensure!(
        design["derived_blocks"]
            == json!([
                {"id":"40","source_tileset":"underground","source_block":"2b","collision":wall},
                {"id":"41","source_tileset":"facility","source_block":"16","tile_order":shutter_order,"collision":wall},
                {"id":"42","source_tileset":"cave","source_block":"0b","tile_quadrant_sources":quadrants,
                 "collision":["FLOOR","LADDER","FLOOR","FLOOR"]}
            ]),
        "exact native mine machine/shutter/ladder recipes required"
    );
    ensure!(
        design["block_overrides"]
            == json!([{
                "id":"0d","source_tileset":"cave","source_block":"02","collision":["FLOOR","FLOOR","FLOOR","FLOOR"],
                "reason":"Retained pressure scripts open block13; preserve that numeric floor meaning in the private alias."
            }]),
        "retain block13's script floor meaning"
    );
    let cave = &base.data.tilesets["cave"];
    ensure!(
        cave.collision["02"].iter().all(|k| k == "FLOOR")
            && cave.collision["0b"]
                .iter()
                .map(String::as_str)
                .eq(["FLOOR", "FLOOR", "FLOOR", "LADDER"])
            && base.runtime_files["data/tilesets/cave_metatiles.bin"].len() == 64 * 16,
        "native cave source collision/extent differs"
    );
    let art = json!({"tileset":"cave","source_pixels_modified":false,"extended_graphics":true,
    "derived_blocks":[
        {"id":"40","source_tileset":"underground","source_block":"2b","collision":wall},
        {"id":"41","source_tileset":"facility","source_block":"16","tile_order":shutter_order,"collision":wall},
        {"id":"42","source_tileset":"cave","source_block":"0b","collision":cave.collision["0b"]}
    ]});
    let mut pack = crate::authored_city_expansion_art::install(
        base,
        &art.to_string(),
        ALIAS,
        "cave",
        "minnesota-iron-haulage-native-rock-v1",
    )?;
    let path = format!("data/tilesets/{ALIAS}_metatiles.bin");
    let raw = pack
        .runtime_files
        .get_mut(&path)
        .context("private mine blocks")?;
    let floor: [u8; 16] = raw[2 * 16..3 * 16].try_into()?;
    raw[13 * 16..14 * 16].copy_from_slice(&floor);
    let ladder: [u8; 16] = raw[0x42 * 16..0x43 * 16].try_into()?;
    // Move complete native 16px quadrants, including their exact collision.
    // The original five active stairs use the upper-right quadrant. No glyph
    // is redrawn, mirrored, scaled or substituted with an outdoor climb.
    for (destination, source) in [0, 3, 2, 1].into_iter().enumerate() {
        for y in 0..2 {
            for x in 0..2 {
                let to = (destination / 2 * 2 + y) * 4 + destination % 2 * 2 + x;
                let from = (source / 2 * 2 + y) * 4 + source % 2 * 2 + x;
                raw[0x42 * 16 + to] = ladder[from];
            }
        }
    }
    let definition = pack
        .data
        .tilesets
        .get_mut(ALIAS)
        .context("private mine collision")?;
    definition
        .collision
        .insert("0d".into(), vec!["FLOOR".into(); 4]);
    definition.collision.insert(
        "42".into(),
        ["FLOOR", "LADDER", "FLOOR", "FLOOR"]
            .map(String::from)
            .to_vec(),
    );
    pack.runtime_files.insert(
        format!("data/tilesets/{ALIAS}.json"),
        serde_json::to_vec(&definition.collision)?,
    );
    // The caller replaces the map and derives the final pack identity.
    Ok(pack)
}
