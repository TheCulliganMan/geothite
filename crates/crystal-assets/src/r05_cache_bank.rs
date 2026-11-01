//! Give the existing Route 5 lookout cache a grassy climb and short dry return.
use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::CompiledGamePack;
use crate::authored_route_layout::Layout;

/// Publishes eight native terrain cells and corrects the old terrace directions.
///
/// # Errors
/// Rejects stale map/source/art, changed cache or lookout actors, unexpected
/// text, a shared block payload, or invalid native pack/geography.
pub fn revise_r05_cache_bank(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let before_value: serde_json::Value = serde_json::from_str(before_json)?;
    let after_value: serde_json::Value = serde_json::from_str(after_json)?;
    for field in [
        "tileset",
        "compiled_tileset",
        "source_pixels_modified",
        "derived_blocks",
        "shore_fitting",
    ] {
        ensure!(
            before_value.get(field) == after_value.get(field),
            "art source drift: {field}"
        );
    }
    ensure!(
        after_value["source_pixels_modified"] == false,
        "native pixels required"
    );
    let before: Layout = serde_json::from_value(before_value)?;
    let after: Layout = serde_json::from_value(after_value)?;
    let old = base
        .data
        .maps
        .get("MnR05C00R00")
        .context("upper bluff route")?;
    ensure!(
        before.schema_version == 2
            && after.schema_version == 2
            && before.id == old.id
            && after.id == old.id
            && before.dimensions_metatiles == [32, 24]
            && after.dimensions_metatiles == before.dimensions_metatiles
            && before.compiled_tileset == "mn_outdoor_headwaters_stairs"
            && after.compiled_tileset == before.compiled_tileset
            && old.attributes.width == 32
            && old.attributes.height == 24
            && old.attributes.tileset_name == before.compiled_tileset,
        "native map identity/art/extent drift"
    );
    ensure!(
        before.compile()? == old.blocks,
        "complete upper-route preimage drift"
    );
    let changes = [
        (17, 9, 0x02, 0x03),
        (18, 9, 0x02, 0x03),
        (18, 10, 0x01, 0x03),
        (19, 10, 0x01, 0x57),
        (20, 10, 0x01, 0x05),
        (19, 11, 0x05, 0x01),
        (20, 11, 0x05, 0x01),
        (21, 11, 0x05, 0x01),
    ];
    let mut expected = old.blocks.clone();
    for (x, y, previous, next) in changes {
        ensure!(
            expected[y * 32 + x] == previous,
            "cache-bank cell preimage drift"
        );
        expected[y * 32 + x] = next;
    }
    let mut map = old.clone();
    map.blocks = after.compile()?;
    ensure!(
        map.blocks == expected,
        "only eight declared terrain cells may change"
    );
    ensure!(
        old.objects
            .iter()
            .any(|o| o.script == "MN_R05_ITEM_02" && o.x == 37 && o.y == 19)
            && old
                .objects
                .iter()
                .any(|o| o.script == "MN_R05_LOOKOUT" && o.x == 53 && o.y == 21),
        "preserve the existing cache and river lookout"
    );
    for (block, kinds) in [
        (0x01, ["FLOOR"; 4]),
        (0x03, ["TALL_GRASS"; 4]),
        (0x05, ["WALL"; 4]),
        (0x57, ["HOP_DOWN", "HOP_DOWN", "WALL", "WALL"]),
    ] {
        let collision = &base.data.tilesets[&old.attributes.tileset_name].collision;
        ensure!(
            collision[&format!("{block:02x}")]
                .iter()
                .map(String::as_str)
                .eq(kinds),
            "native block collision drift"
        );
    }
    for x in [38, 39] {
        for (y, kind) in [(19, "FLOOR"), (20, "HOP_DOWN"), (21, "WALL"), (22, "FLOOR")] {
            ensure!(
                crate::authored_town_geometry::kind(base, &map, [x, y])? == kind,
                "legal dry bank geometry required"
            );
        }
    }
    for at in [[40, 19], [40, 20], [40, 21], [41, 20], [41, 21]] {
        ensure!(
            crate::authored_town_geometry::kind(base, &map, at)? == "WALL",
            "close the dry lateral bypass"
        );
    }
    let label = old
        .attributes
        .blocks_label
        .as_ref()
        .context("native block label")?;
    ensure!(
        base.data
            .maps
            .values()
            .filter(|m| m.attributes.blocks_label.as_ref() == Some(label))
            .count()
            == 1,
        "unique native block payload required"
    );
    let bytes = map
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut pack = base.clone();
    pack.data.maps.insert(map.id.clone(), map);
    *pack
        .data
        .map_blocks
        .get_mut(label)
        .context("existing encoded block payload")? = crate::encode_base64_bytes(&bytes);
    let text_label = "MN_R05_RAMP_SIGNText";
    let old_text = json!([
        {"command":"text", "args":["\"CLIMB THE BENDS\""]},
        {"command":"line", "args":["\"DROP TO RETURN\""]},
        {"command":"done", "args":[]}
    ]);
    ensure!(
        base.data.map_scripts.get(text_label) == Some(&old_text),
        "terrace sign text preimage drift"
    );
    let mut sign_map = base
        .data
        .maps
        .get("MnR05C00R01")
        .context("terrace sign map")?
        .clone();
    ensure!(
        sign_map.scripts.get(text_label) == Some(&old_text),
        "local sign mirror drift"
    );
    let mut text = old_text;
    text[1]["args"][0] = json!("\"STAIRS BOTH WAYS\"");
    sign_map.scripts.insert(text_label.into(), text.clone());
    pack.data.map_scripts.insert(text_label.into(), text);
    sign_map.script_text_commands =
        crate::parse_script_text_commands(&sign_map.id, &sign_map.scripts)?;
    sign_map.script_text_bodies = crate::parse_script_text_bodies(&sign_map.id, &sign_map.scripts)?;
    pack.data.maps.insert(sign_map.id.clone(), sign_map);
    const MANIFEST: &str = "minnesota-r05-cache-grass-bank-v1";
    ensure!(
        !base.report.manifests.iter().any(|v| v == MANIFEST),
        "already revised"
    );
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}
