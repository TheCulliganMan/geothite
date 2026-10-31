//! Restore the complete native Brainerd mart facade.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_route_layout::Layout;

/// Replaces the tree-covered roof with its original native mart roof.
///
/// # Errors
/// Rejects stale source, art, facade, door, collision, or compact geography.
pub fn revise_brainerd_mart_facade(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let old = base.data.maps.get("MnBrainerd").context("town")?;
    ensure!(
        old.attributes.width == 32
            && old.attributes.height == 24
            && old.attributes.tileset_name == "mn_outdoor_johto_stairs",
        "town extent/art drift"
    );
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
        "native source pixels required"
    );
    let before: Layout = serde_json::from_value(before_value)?;
    let after: Layout = serde_json::from_value(after_value)?;
    ensure!(
        before.schema_version == 2
            && after.schema_version == 2
            && before.id == "MnBrainerd"
            && after.id == before.id
            && before.dimensions_metatiles == [32, 24]
            && after.dimensions_metatiles == before.dimensions_metatiles
            && after.compiled_tileset == old.attributes.tileset_name,
        "layout identity/extent drift"
    );
    ensure!(
        before.compile()? == old.blocks,
        "complete town source preimage drift"
    );
    let mut expected = old.blocks.clone();
    ensure!(expected[9 * 32 + 10] == 0x05, "mart preimage drift");
    expected[9 * 32 + 10] = 0x19;
    let mut map = old.clone();
    map.blocks = after.compile()?;
    ensure!(map.blocks == expected, "only one mart roof cell may change");
    let reference = base
        .data
        .maps
        .get("CherrygroveCity")
        .context("native mart reference")?;
    ensure!(
        reference.attributes.tileset_name == "johto",
        "native reference art drift"
    );
    for dy in 0..2 {
        for dx in 0..2 {
            let block = reference.blocks[dy * usize::from(reference.attributes.width) + 11 + dx];
            ensure!(
                map.blocks[(9 + dy) * 32 + 9 + dx] == block,
                "complete original mart assembly required"
            );
        }
    }
    ensure!(
        crate::authored_town_geometry::kind(base, &map, [19, 21])? == "DOOR"
            && crate::authored_town_geometry::kind(base, &map, [19, 22])? == "FLOOR",
        "existing mart anchors required"
    );
    for y in 0..48 {
        for x in 0..64 {
            ensure!(
                crate::authored_town_geometry::kind(base, old, [x, y])?
                    == crate::authored_town_geometry::kind(base, &map, [x, y])?,
                "all town collision must remain exact"
            );
        }
    }
    const MANIFEST: &str = "minnesota-brainerd-mart-facade-v1";
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "already revised"
    );
    let label = old
        .attributes
        .blocks_label
        .as_ref()
        .context("town block label")?;
    ensure!(
        base.data
            .maps
            .values()
            .filter(|m| m.attributes.blocks_label.as_ref() == Some(label))
            .count()
            == 1,
        "unique town block payload required"
    );
    let bytes = map
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut result = base.clone();
    result.data.maps.insert(map.id.clone(), map);
    *result
        .data
        .map_blocks
        .get_mut(label)
        .context("existing town block payload")? = crate::encode_base64_bytes(&bytes);
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
