//! Duluth's authored terrace joins using native stair shapes and palettes.

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_route_layout::Layout;

/// Joins existing outdoor landings to native cave steps and their palettes, and
/// applies the explicitly authored terrace and lower street layout.
///
/// Keeps native glyph bitplanes and collisions. Appends private blocks without
/// changing existing art addresses; the complete anchor contract retains rooms,
/// actors and regional connections. Does not write files or mutate the parent.
///
/// # Errors
/// Rejects terrain/anchor drift, unexpected recipes or rock palettes, shared
/// tileset users, unavailable glyph storage, or unsafe authored geometry.
pub fn revise_duluth_terrace_stairs(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    crate::validate_compact_superior_geography(base)?;
    let before: Layout = serde_json::from_str(before_json)?;
    let city = base.data.maps.get("MnDuluth").context("installed Duluth")?;
    ensure!(
        before.compile()? == city.blocks,
        "Duluth terrain preimage differs"
    );
    ensure!(
        base.data.maps.iter().all(|(name, map)| {
            map.attributes.tileset_name != "mn_duluth_expanded" || name == "MnDuluth"
        }),
        "Duluth private art is shared by another map"
    );
    let before_value: serde_json::Value = serde_json::from_str(before_json)?;
    let after_value: serde_json::Value = serde_json::from_str(after_json)?;
    let old = before_value["derived_blocks"]
        .as_array()
        .context("old native recipes")?;
    let new = after_value["derived_blocks"]
        .as_array()
        .context("new native recipes")?;
    ensure!(
        old.len() == 13 && new.len() == 17 && new[..13] == old[..],
        "Duluth must retain existing art and append three front flights and one north flight"
    );
    let cave = base.data.tilesets.get("cave").context("native cave art")?;
    for ((id, source), recipe) in [("8d", "12"), ("8e", "30"), ("8f", "36")]
        .into_iter()
        .zip(&new[13..])
    {
        let landing = if source == "36" { "69" } else { "72" };
        let rows = serde_json::json!([
            {"source_tileset": "mn_duluth_expanded", "source_block": landing, "source_row": 0},
            {"source_tileset": "mn_duluth_expanded", "source_block": landing, "source_row": 1},
            {"source_tileset": "cave", "source_block": source, "source_row": 2},
            {"source_tileset": "cave", "source_block": source, "source_row": 3}
        ]);
        ensure!(
            recipe["id"] == id
                && recipe["source_tileset"] == "cave"
                && recipe["source_block"] == source
                && recipe.get("palette_override").is_none()
                && recipe["tile_row_sources"] == rows
                && recipe["collision"]
                    == serde_json::to_value(
                        cave.collision
                            .get(source)
                            .context("native stair collision")?
                    )?,
            "Duluth outdoor stair recipe differs"
        );
    }
    let tileset = base
        .data
        .tilesets
        .get("mn_duluth_expanded")
        .context("Duluth art")?;
    let blocks = base
        .runtime_files
        .get("data/tilesets/mn_duluth_expanded_metatiles.bin")
        .context("Duluth native blocks")?;
    ensure!(
        blocks
            .get(0x69 * 16..0x6a * 16)
            .context("outdoor cliff69")?
            .iter()
            .all(|&glyph| tileset.palette_map[usize::from(glyph)] & 7 == 5),
        "Duluth cliff palette changed"
    );
    let mut art = crate::north_shore_art::append_native_cave_stairs(
        base,
        "mn_duluth_expanded",
        0x8d,
        &[0x12, 0x30, 0x36],
        "minnesota-duluth-terrace-stairs-v2",
    )?;
    let north = &new[16];
    ensure!(
        north["id"] == "90"
            && north["source_tileset"] == "cave"
            && north["source_block"] == "12"
            && north.get("palette_override").is_none()
            && north["collision"] == serde_json::json!(["WALL", "FLOOR", "FLOOR", "FLOOR"])
            && north["tile_row_sources"]
                == serde_json::json!([
                    {"source_tileset":"cave", "source_block":"12", "source_row":2},
                    {"source_tileset":"cave", "source_block":"12", "source_row":3},
                    {"source_tileset":"mn_duluth_expanded", "source_block":"70", "source_row":2},
                    {"source_tileset":"mn_duluth_expanded", "source_block":"70", "source_row":3}
                ]),
        "Duluth north flight must join native lower steps to the north-facing plateau landing"
    );
    let path = "data/tilesets/mn_duluth_expanded_metatiles.bin";
    let raw = art
        .runtime_files
        .get_mut(path)
        .context("Duluth stair art")?;
    ensure!(
        raw.len() == 0x90 * 16,
        "Duluth north stair slot is occupied"
    );
    let mut north_rows = raw[0x8d * 16 + 8..0x8e * 16].to_vec();
    north_rows.extend_from_slice(&raw[0x70 * 16 + 8..0x71 * 16]);
    raw.extend(north_rows);
    let definition = art
        .data
        .tilesets
        .get_mut("mn_duluth_expanded")
        .context("Duluth collision")?;
    ensure!(
        definition
            .collision
            .insert(
                "90".into(),
                ["WALL", "FLOOR", "FLOOR", "FLOOR"].map(String::from).into()
            )
            .is_none(),
        "Duluth north stair collision is occupied"
    );
    art.runtime_files.insert(
        "data/tilesets/mn_duluth_expanded.json".into(),
        serde_json::to_vec(&definition.collision)?,
    );
    art.report
        .manifests
        .push("minnesota-duluth-north-stair-v1".into());
    art.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        art.format_version,
        &art.data,
        &art.audio_manifest,
        &art.runtime_files,
        &art.report,
    )?;
    let revised = crate::recompose_authored_city(&art, after_json, contract_json)?;
    crate::validate_compact_superior_geography(&revised)?;
    Ok(revised)
}
