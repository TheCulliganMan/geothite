//! A native stone staircase through Bemidji's south plateau face.
use crate::CompiledGamePack;
use crate::authored_route_layout::Layout;
use anyhow::{Context, Result, ensure};

/// Adds the requested two-way plateau stairs using original cave12 treads.
///
/// # Errors
/// Rejects stale complete layouts, changed native art, another stair site,
/// an occupied art alias, unrelated terrain changes, or invalid pack payloads.
pub fn revise_bemidji_plateau_stairs(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    const ALIAS: &str = "mn_bemidji_plateau_stairs";
    const MANIFEST: &str = "minnesota-bemidji-plateau-stairs-v1";
    let old = base.data.maps.get("MnBemidji").context("Bemidji")?;
    ensure!(
        old.attributes.width == 24
            && old.attributes.height == 24
            && old.attributes.tileset_name == "mn_bemidji",
        "plateau extent/art drift"
    );
    let before: Layout = serde_json::from_str(before_json)?;
    let after: Layout = serde_json::from_str(after_json)?;
    ensure!(
        before.id == "MnBemidji"
            && after.id == before.id
            && before.dimensions_metatiles == [24, 24]
            && after.dimensions_metatiles == [24, 24]
            && before.compiled_tileset == "mn_bemidji"
            && after.compiled_tileset == ALIAS,
        "plateau layout identity drift"
    );
    ensure!(
        before.compile()? == old.blocks,
        "complete plateau preimage drift"
    );
    let mut expected = old.blocks.clone();
    ensure!(
        expected[10 * 24 + 5] == 0x72
            && expected[9 * 24 + 20] == 0x1a
            && expected[9 * 24 + 21] == 0x33,
        "current plateau and repaired Mart required"
    );
    expected[10 * 24 + 5] = 0x84;
    ensure!(
        after.compile()? == expected,
        "only the plateau stair cell may change"
    );
    ensure!(
        !base.data.tilesets.contains_key(ALIAS)
            && !base.report.manifests.iter().any(|v| v == MANIFEST),
        "plateau stairs already installed"
    );
    let source = base
        .runtime_files
        .get("data/tilesets/mn_bemidji_metatiles.bin")
        .context("native Bemidji blocks")?;
    ensure!(
        source.len() == 132 * 16,
        "Bemidji native block extent drift"
    );
    let art = serde_json::json!({"tileset":"mn_bemidji", "source_pixels_modified":false,"derived_blocks":[{"id":"84","source_tileset":"cave","source_block":"12","collision":["FLOOR","FLOOR","WALL","FLOOR"]}]});
    let mut result = crate::authored_city_expansion_art::install(
        base,
        &serde_json::to_string(&art)?,
        ALIAS,
        "mn_bemidji",
        "minnesota-bemidji-plateau-stair-art-v1",
    )?;
    let raw = result
        .runtime_files
        .get_mut(&format!("data/tilesets/{ALIAS}_metatiles.bin"))
        .context("plateau stair blocks")?;
    raw[132 * 16..132 * 16 + 8].copy_from_slice(&source[0x72 * 16..0x72 * 16 + 8]);
    let mut map = old.clone();
    map.blocks = expected;
    map.attributes.tileset_name = ALIAS.into();
    for y in 0..48 {
        for x in 0..48 {
            let before = crate::authored_town_geometry::kind(base, old, [x, y])?;
            let after = crate::authored_town_geometry::kind(&result, &map, [x, y])?;
            if [x, y] == [11, 21] {
                ensure!(
                    before == "WALL" && after == "FLOOR",
                    "stair lane must open exactly one cliff quadrant"
                );
            } else {
                ensure!(before == after, "unrelated collision changed");
            }
        }
    }
    let label = old
        .attributes
        .blocks_label
        .as_ref()
        .context("plateau block label")?;
    ensure!(
        base.data
            .maps
            .values()
            .filter(|m| m.attributes.blocks_label.as_ref() == Some(label))
            .count()
            == 1,
        "shared plateau blocks"
    );
    let bytes = map
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    result
        .data
        .map_blocks
        .insert(label.clone(), crate::encode_base64_bytes(&bytes));
    result
        .data
        .map_attributes
        .insert(map.id.clone(), map.attributes.clone());
    result.data.maps.insert(map.id.clone(), map);
    result.report.manifests.push(MANIFEST.into());
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    crate::validate_compact_superior_geography(&result)?;
    Ok(result)
}
