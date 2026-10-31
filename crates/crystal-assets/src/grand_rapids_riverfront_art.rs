//! Native port paving with the original outdoor water animation retained.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::CompiledGamePack;

pub(super) const ALIAS: &str = "mn_grand_rapids_riverfront";

fn physical(glyph: u8, palette: u8, count: usize) -> usize {
    if palette & 8 == 0 {
        usize::from(glyph)
    } else {
        usize::from(glyph & 0x7f) + count / 2
    }
}

pub(super) fn install(base: &CompiledGamePack, source: &Value) -> Result<CompiledGamePack> {
    let recipes = json!([
        {"id":"80","source_tileset":"port","source_block":"2f","collision":["WATER","FLOOR","WATER","FLOOR"]},
        {"id":"81","source_tileset":"port","source_block":"2e","collision":["FLOOR","WATER","FLOOR","WATER"]},
        {"id":"82","source_tileset":"port","source_block":"30","collision":["FLOOR","FLOOR","WATER","WATER"]}
    ]);
    ensure!(
        source["derived_blocks"] == recipes
            && source["source_pixels_modified"] == false
            && source["tileset"] == "johto_modern"
            && source["compiled_tileset"] == ALIAS,
        "three exact native stone-pier recipes required"
    );
    let mut pack = crate::authored_city_expansion_art::install(
        base,
        &serde_json::to_string(source)?,
        ALIAS,
        "johto_modern",
        "minnesota-grand-rapids-riverfront-native-v1",
    )?;
    let original = &base.runtime_files["gfx/tilesets/johto_modern.2bpp"];
    let port = &base.runtime_files["gfx/tilesets/port.2bpp"];
    let palette = &base.data.tilesets["johto_modern"].palette_map;
    let port_palette = &base.data.tilesets["port"].palette_map;
    let water = physical(0x14, palette[0x14], original.len() / 16);
    let port_water = physical(0x14, port_palette[0x14], port.len() / 16);
    ensure!(
        palette[0x14] & 7 == port_palette[0x14] & 7
            && original[water * 16..water * 16 + 16] == port[port_water * 16..port_water * 16 + 16],
        "native port/outdoor water pixels and palette must agree"
    );
    let port_layout = &base.runtime_files["data/tilesets/port_metatiles.bin"];
    let path = format!("data/tilesets/{ALIAS}_metatiles.bin");
    let layout = pack
        .runtime_files
        .get_mut(&path)
        .context("private pier metatiles")?;
    // The generic importer keeps imported pixels static. These exact port
    // water glyphs must instead use the original animated outdoor destination.
    // Ground/stone glyphs and all inherited addresses remain unchanged.
    for (to, from) in [(0x80, 0x2f), (0x81, 0x2e), (0x82, 0x30)] {
        for index in 0..16 {
            if port_layout[from * 16 + index] == 0x14 {
                layout[to * 16 + index] = 0x14;
            }
        }
    }
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}
