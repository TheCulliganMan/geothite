//! Give the existing Route 9 southwest clearing a grassy approach and pickup pocket.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_route_layout::Layout;

/// Publishes three native grass metatiles, preserving the existing Cut gate.
///
/// # Errors
/// Rejects stale map/source/art, changed cache or lookout actors, unexpected
/// text, a shared block payload, or invalid native pack/geography.
pub fn revise_r09_southwest_grass(
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
        after_value.get("source_pixels_modified").is_none()
            || after_value["source_pixels_modified"] == false,
        "native pixels required"
    );
    let before: Layout = serde_json::from_value(before_value)?;
    let after: Layout = serde_json::from_value(after_value)?;
    let old = base
        .data
        .maps
        .get("MnR09C00R01")
        .context("upper bluff route")?;
    ensure!(
        before.schema_version == 2
            && after.schema_version == 2
            && before.id == old.id
            && after.id == old.id
            && before.dimensions_metatiles == [32, 12]
            && after.dimensions_metatiles == before.dimensions_metatiles
            && before.compiled_tileset == "mn_iron_range"
            && after.compiled_tileset == before.compiled_tileset
            && old.attributes.width == 32
            && old.attributes.height == 12
            && old.attributes.tileset_name == before.compiled_tileset,
        "native map identity/art/extent drift"
    );
    ensure!(
        before.compile()? == old.blocks,
        "complete upper-route preimage drift"
    );
    let changes = [(6, 3, 0x02, 0x03), (5, 6, 0x02, 0x03), (5, 7, 0x01, 0x03)];
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
        "only three declared grass cells may change"
    );
    let collision = &base.data.tilesets[&old.attributes.tileset_name].collision;
    for (block, expected) in [("01", "FLOOR"), ("02", "FLOOR"), ("03", "TALL_GRASS")] {
        ensure!(
            collision[block].iter().all(|v| v == expected),
            "native grass collision drift"
        );
    }
    ensure!(old.blocks[4 * 32 + 6] == 0x63, "preserve existing Cut gate");
    ensure!(
        old.objects
            .iter()
            .any(|o| o.script == "MN_R09_ITEM_04" && o.x == 11 && o.y == 15),
        "preserve pickup anchor"
    );
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
    const MANIFEST: &str = "minnesota-r09-southwest-grass-v1";
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
