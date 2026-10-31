//! Native mechanical blocks in a private mine palette and rock bank.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::CompiledGamePack;

pub(crate) const ALIAS: &str = "mn_iron_core_rock";
const BASE: &str = "mn_iron_haulage_rock";

pub(crate) fn install(base: &CompiledGamePack, design: &Value) -> Result<CompiledGamePack> {
    let wall = vec!["WALL"; 4];
    let rows = (0..3)
        .map(|row| json!({"source_tileset":"facility","source_block":"28","source_row":row}))
        .chain(std::iter::once(
            json!({"source_tileset":BASE,"source_block":"0d","source_row":3}),
        ))
        .collect::<Vec<_>>();
    let recipes = json!([
        {"id":"43","source_tileset":"facility","source_block":"13","collision":wall},
        {"id":"44","source_tileset":"facility","source_block":"12","collision":wall},
        {"id":"45","source_tileset":"facility","source_block":"16","collision":wall},
        {"id":"46","source_tileset":"facility","source_block":"28","collision":["WALL","WALL","FLOOR","FLOOR"],"tile_row_sources":rows}
    ]);
    ensure!(
        design["base_tileset"] == BASE
            && design["source_pixels_modified"] == false
            && design["derived_blocks"] == recipes
            && design["block_overrides"]
                == json!([
                    {"id":"12","source_tileset":"facility","source_block":"12","collision":wall,"reason":"Retain original numeric18 extended drilling frame in the private alias."},
                    {"id":"16","source_tileset":"facility","source_block":"16","collision":wall,"reason":"Retain original numeric22 retracted/shutdown frame in the private alias."}
                ]),
        "exact native drill-frame and control-landing recipes required"
    );
    let source = base.data.tilesets.get(BASE).context("accepted mine art")?;
    ensure!(
        source.collision["0d"].iter().all(|kind| kind == "FLOOR")
            && source.collision["42"]
                .iter()
                .map(String::as_str)
                .eq(["FLOOR", "LADDER", "FLOOR", "FLOOR"])
            && base.runtime_files[&format!("data/tilesets/{BASE}_metatiles.bin")].len()
                == 0x43 * 16,
        "accepted mine floor and shaft art required"
    );
    let art = json!({"tileset":BASE,"source_pixels_modified":false,"derived_blocks":recipes});
    let mut pack = crate::authored_city_expansion_art::install(
        base,
        &art.to_string(),
        ALIAS,
        BASE,
        "minnesota-iron-core-native-rock-v1",
    )?;
    let path = format!("data/tilesets/{ALIAS}_metatiles.bin");
    let raw = pack
        .runtime_files
        .get_mut(&path)
        .context("private drill metatiles")?;
    ensure!(
        raw.len() == 0x47 * 16,
        "four contiguous native machine imports required"
    );
    for (destination, from) in [(0x12, 0x44), (0x16, 0x45)] {
        let glyphs: [u8; 16] = raw[from * 16..(from + 1) * 16].try_into()?;
        raw[destination * 16..(destination + 1) * 16].copy_from_slice(&glyphs);
    }
    // The control's native table feet remain exact. Its lower 8px floor row
    // joins the mine floor, avoiding a lone checker stripe under the console.
    let landing: [u8; 4] = raw[13 * 16 + 12..14 * 16].try_into()?;
    raw[0x46 * 16 + 12..0x47 * 16].copy_from_slice(&landing);
    let definition = pack
        .data
        .tilesets
        .get_mut(ALIAS)
        .context("private machine collision")?;
    for id in ["12", "16"] {
        definition
            .collision
            .insert(id.into(), vec!["WALL".into(); 4]);
    }
    pack.runtime_files.insert(
        format!("data/tilesets/{ALIAS}.json"),
        serde_json::to_vec(&definition.collision)?,
    );
    // The room publisher derives the final identity after the map replacement.
    Ok(pack)
}
