//! Complete original St Cloud clinic/Mart fronts with paired visible doorways.
use crate::CompiledGamePack;
use crate::authored_route_layout::Layout;
use anyhow::{Context, Result, ensure};

fn shifted_x(x: u16, y: u16) -> Option<u16> {
    match (x, y) {
        (6, 9 | 10) => Some(7),
        (14, 9 | 10) => Some(15),
        _ => None,
    }
}

/// Restores two native service assemblies and their active/passive doorway pairs.
///
/// The unused solid third columns become native forest. Existing streets,
/// river joins, stairs, art, actors and all other map content stay intact.
///
/// # Errors
/// Rejects stale complete layouts, changed art/extent, unrelated edits,
/// mismatched raw or parsed door records, and invalid compiled payloads.
pub fn revise_st_cloud_service_fronts(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let old = base.data.maps.get("MnStCloud").context("St Cloud")?;
    ensure!(
        old.attributes.width == 24
            && old.attributes.height == 18
            && old.attributes.tileset_name == "mn_outdoor_modern_stairs",
        "current St Cloud extent/art required"
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
            "art/fitting source drift: {field}"
        );
    }
    ensure!(
        after_value["source_pixels_modified"] == false,
        "original native pixels required"
    );
    let before: Layout = serde_json::from_value(before_value)?;
    let after: Layout = serde_json::from_value(after_value)?;
    ensure!(
        before.schema_version == 2
            && after.schema_version == 2
            && before.id == "MnStCloud"
            && after.id == before.id
            && before.dimensions_metatiles == [24, 18]
            && after.dimensions_metatiles == before.dimensions_metatiles
            && after.compiled_tileset == old.attributes.tileset_name,
        "service layout identity drift"
    );
    ensure!(
        before.compile()? == old.blocks,
        "complete town source preimage drift"
    );
    let mut expected = old.blocks.clone();
    for (x, y, from, to) in [
        (4, 3, 0x1f, 0x19),
        (5, 3, 0x19, 0x05),
        (3, 4, 0x17, 0x1a),
        (5, 4, 0x11, 0x05),
        (8, 3, 0x1f, 0x19),
        (9, 3, 0x19, 0x05),
        (7, 4, 0x17, 0x1a),
        (9, 4, 0x11, 0x05),
    ] {
        ensure!(
            expected[y * 24 + x] == from,
            "service facade preimage drift"
        );
        expected[y * 24 + x] = to;
    }
    let mut map = old.clone();
    map.blocks = after.compile()?;
    ensure!(
        map.blocks == expected,
        "only eight service facade cells may change"
    );
    let reference = base
        .data
        .maps
        .get("AzaleaTown")
        .context("original service reference")?;
    ensure!(
        reference.attributes.tileset_name == "johto_modern",
        "original service reference art drift"
    );
    for (destination, source) in [(3, (7, 3)), (7, (10, 1))] {
        for dy in 0..2 {
            for dx in 0..2 {
                ensure!(
                    map.blocks[(3 + dy) * 24 + destination + dx]
                        == reference.blocks[(source.1 + dy)
                            * usize::from(reference.attributes.width)
                            + source.0
                            + dx],
                    "complete original clinic/Mart assembly required"
                );
            }
        }
    }
    for (index, x, y) in [(1, 6, 9), (2, 6, 10), (3, 14, 9), (4, 14, 10)] {
        let warp = map
            .events
            .warps
            .iter_mut()
            .find(|w| w.index == index)
            .context("service door pair")?;
        ensure!(warp.x == x && warp.y == y, "service door preimage drift");
        warp.x = shifted_x(x, y).context("service door coordinate")?;
    }
    let mut count = 0;
    for command in &mut map.map_event_section_commands {
        if command.command != "warp_event" {
            continue;
        }
        ensure!(command.args.len() == 4, "cached warp shape");
        let x: u16 = command.args[0].parse()?;
        let y: u16 = command.args[1].parse()?;
        if let Some(x) = shifted_x(x, y) {
            command.args[0] = x.to_string();
            count += 1;
        }
    }
    ensure!(count == 4, "four cached service warp commands required");
    for y in 0..36 {
        for x in 0..48 {
            let before = crate::authored_town_geometry::kind(base, old, [x, y])?;
            let after = crate::authored_town_geometry::kind(base, &map, [x, y])?;
            let expected = match [x, y] {
                [6 | 14, 9] => {
                    ensure!(before == "DOOR", "old door collision");
                    "WALL"
                }
                [7 | 15, 9] => {
                    ensure!(before == "WALL", "new door collision");
                    "DOOR"
                }
                _ => before,
            };
            ensure!(
                after == expected,
                "only four service door quadrants may change collision"
            );
        }
    }
    const MANIFEST: &str = "minnesota-st-cloud-service-fronts-v1";
    ensure!(
        !base.report.manifests.iter().any(|v| v == MANIFEST),
        "service fronts already revised"
    );
    let label = old
        .attributes
        .blocks_label
        .as_ref()
        .context("town blocks")?;
    ensure!(
        base.data
            .maps
            .values()
            .filter(|m| m.attributes.blocks_label.as_ref() == Some(label))
            .count()
            == 1,
        "shared town blocks"
    );
    let mut result = base.clone();
    let events_label = old
        .attributes
        .map_events_label
        .as_ref()
        .context("town events label")?;
    let commands = result
        .data
        .map_scripts
        .get_mut(events_label)
        .and_then(serde_json::Value::as_array_mut)
        .context("split town event commands")?;
    let mut count = 0;
    for command in commands {
        if command["command"] != "warp_event" {
            continue;
        }
        let args = command["args"]
            .as_array_mut()
            .context("split warp arguments")?;
        ensure!(args.len() == 4, "split warp shape");
        let x = args[0].as_str().context("split warp x")?.parse()?;
        let y = args[1].as_str().context("split warp y")?.parse()?;
        if let Some(x) = shifted_x(x, y) {
            args[0] = x.to_string().into();
            count += 1;
        }
    }
    ensure!(count == 4, "four split service warp commands required");
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
