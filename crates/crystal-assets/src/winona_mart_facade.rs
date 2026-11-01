//! Restore the complete native Winona mart facade.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_route_layout::Layout;

/// Replaces the mixed wooden front with its original native mart component.
///
/// # Errors
/// Rejects stale source, art, facade, door, collision, or compact geography.
pub fn revise_winona_mart_facade(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let old = base.data.maps.get("MnWinona").context("town")?;
    ensure!(
        old.attributes.width == 16
            && old.attributes.height == 14
            && old.attributes.tileset_name == "johto",
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
            && before.id == "MnWinona"
            && after.id == before.id
            && before.dimensions_metatiles == [16, 14]
            && after.dimensions_metatiles == before.dimensions_metatiles
            && after.compiled_tileset == old.attributes.tileset_name,
        "layout identity/extent drift"
    );
    ensure!(
        before.compile()? == old.blocks,
        "complete town source preimage drift"
    );
    let mut expected = old.blocks.clone();
    ensure!(expected[11 * 16 + 3] == 0x16, "mart preimage drift");
    expected[11 * 16 + 3] = 0x1a;
    let mut map = old.clone();
    map.blocks = after.compile()?;
    ensure!(map.blocks == expected, "only one mart wall cell may change");
    let reference = base
        .data
        .maps
        .get("CherrygroveCity")
        .context("native mart reference")?;
    ensure!(
        reference.attributes.tileset_name == old.attributes.tileset_name,
        "native reference art drift"
    );
    for dy in 0..2 {
        for dx in 0..2 {
            let block = reference.blocks[dy * usize::from(reference.attributes.width) + 11 + dx];
            ensure!(
                map.blocks[(10 + dy) * 16 + 3 + dx] == block,
                "complete original mart assembly required"
            );
        }
    }
    ensure!(
        crate::authored_town_geometry::kind(base, &map, [7, 23])? == "DOOR"
            && crate::authored_town_geometry::kind(base, &map, [7, 24])? == "FLOOR",
        "existing mart anchors required"
    );
    for y in 0..28 {
        for x in 0..32 {
            ensure!(
                crate::authored_town_geometry::kind(base, old, [x, y])?
                    == crate::authored_town_geometry::kind(base, &map, [x, y])?,
                "all town collision must remain exact"
            );
        }
    }
    const MANIFEST: &str = "minnesota-winona-mart-facade-v1";
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
