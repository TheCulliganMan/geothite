//! Repair Grand Rapids' building assemblies using complete native facades.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_route_layout::Layout;

/// Repairs eleven depot/mart cells and validates the clinic, retaining all events.
///
/// # Errors
/// Rejects stale layouts, art, native facade reference, sign fixes or door anchors.
pub fn revise_grand_rapids_building_facades(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let old = base.data.maps.get("MnGrandRapids").context("town")?;
    ensure!(
        old.attributes.width == 24
            && old.attributes.height == 24
            && old.attributes.tileset_name == "johto_modern",
        "town extent/art drift"
    );
    let before_value: serde_json::Value = serde_json::from_str(before_json)?;
    let after_value: serde_json::Value = serde_json::from_str(after_json)?;
    for field in [
        "tileset",
        "compiled_tileset",
        "source_pixels_modified",
        "derived_blocks",
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
    ensure!(
        before_value["shore_fitting"] == "johto_8_neighbors"
            && after_value["shore_fitting"] == before_value["shore_fitting"],
        "town shore fitting drift"
    );
    let before: Layout = serde_json::from_value(before_value)?;
    let after: Layout = serde_json::from_value(after_value)?;
    ensure!(
        before.schema_version == 2
            && after.schema_version == 2
            && before.id == "MnGrandRapids"
            && after.id == before.id
            && before.dimensions_metatiles == [24, 24]
            && after.dimensions_metatiles == before.dimensions_metatiles
            && after.compiled_tileset == old.attributes.tileset_name,
        "layout identity/extent drift"
    );
    ensure!(
        before.compile()? == old.blocks,
        "complete town source preimage drift"
    );
    let blocks = after.compile()?;
    let mut expected = old.blocks.clone();
    for (x, y, from, to) in [
        (12, 5, 0x18, 0x02),
        (14, 5, 0x19, 0x1f),
        (15, 5, 0x02, 0x19),
        (12, 6, 0x20, 0x01),
        (14, 6, 0x20, 0x0f),
        (15, 6, 0x02, 0x11),
        (18, 6, 0x18, 0x02),
        (19, 6, 0x19, 0x18),
        (20, 6, 0x02, 0x19),
        (18, 7, 0x33, 0x02),
        (20, 7, 0x02, 0x33),
    ] {
        let i = y * 24 + x;
        ensure!(expected[i] == from, "facade preimage drift at {x},{y}");
        expected[i] = to;
    }
    ensure!(
        blocks == expected,
        "only eleven contracted facade cells may change"
    );
    let reference = base
        .data
        .maps
        .get("GoldenrodCity")
        .context("native reference")?;
    ensure!(
        reference.attributes.tileset_name == old.attributes.tileset_name,
        "reference art drift"
    );
    for (dy, row) in [[0x18, 0x1f, 0x19], [0x1a, 0x0f, 0x11]]
        .into_iter()
        .enumerate()
    {
        for (dx, block) in row.into_iter().enumerate() {
            ensure!(
                reference.blocks[(5 + dy) * usize::from(reference.attributes.width) + 4 + dx]
                    == block
                    && blocks[(5 + dy) * 24 + 13 + dx] == block,
                "complete authentic Goldenrod facade required"
            );
        }
    }
    let reference = base
        .data
        .maps
        .get("AzaleaTown")
        .context("native clinic/mart reference")?;
    ensure!(
        reference.attributes.tileset_name == old.attributes.tileset_name,
        "clinic/mart reference art drift"
    );
    for ([sx, sy], [tx, ty]) in [([7, 3], [4, 4]), ([10, 1], [19, 6])] {
        for dy in 0..2 {
            for dx in 0..2 {
                ensure!(
                    reference.blocks[(sy + dy) * usize::from(reference.attributes.width) + sx + dx]
                        == blocks[(ty + dy) * 24 + tx + dx],
                    "complete native clinic/mart required"
                );
            }
        }
    }
    for (id, at) in [
        ("MN_GR_WEST_SIGN", [14, 22]),
        ("MN_GR_EAST_SIGN", [38, 22]),
        ("MN_GR_DEPOT_SIGN", [22, 12]),
    ] {
        ensure!(
            old.events
                .bg_events
                .iter()
                .any(|e| e.script == id && [e.x, e.y] == at),
            "retain corrected sign: {id}"
        );
    }
    let mut map = old.clone();
    map.blocks = blocks;
    for (door, back) in [
        ([9, 11], [9, 12]),
        ([39, 15], [39, 16]),
        ([27, 13], [27, 14]),
    ] {
        ensure!(
            crate::authored_town_geometry::kind(base, &map, door)? == "DOOR"
                && crate::authored_town_geometry::kind(base, &map, back)? == "FLOOR",
            "all three existing door/return anchors required"
        );
    }
    for y in 0..48 {
        for x in 0..48 {
            ensure!(
                (crate::authored_town_geometry::kind(base, old, [x, y])? == "DOOR")
                    == (crate::authored_town_geometry::kind(base, &map, [x, y])? == "DOOR"),
                "no added or removed facade doors"
            );
        }
    }
    const MANIFEST: &str = "minnesota-grand-rapids-building-facades-v1";
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
