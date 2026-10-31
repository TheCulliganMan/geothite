//! Restore the complete native Duluth mart facade.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_route_layout::Layout;

/// Restores the original native mart front and moves its paired doorway one tile.
///
/// # Errors
/// Rejects stale source, art, facade, door, collision, or compact geography.
pub fn revise_duluth_mart_facade(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let old = base.data.maps.get("MnDuluth").context("town")?;
    ensure!(
        old.attributes.width == 64
            && old.attributes.height == 46
            && old.attributes.tileset_name == "mn_duluth_expanded",
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
            && before.id == "MnDuluth"
            && after.id == before.id
            && before.dimensions_metatiles == [64, 46]
            && after.dimensions_metatiles == before.dimensions_metatiles
            && after.compiled_tileset == old.attributes.tileset_name,
        "layout identity/extent drift"
    );
    ensure!(
        before.compile()? == old.blocks,
        "complete town source preimage drift"
    );
    let mut expected = old.blocks.clone();
    ensure!(
        expected[19 * 64 + 39] == 0x10 && expected[19 * 64 + 40] == 0x17,
        "mart preimage drift"
    );
    expected[19 * 64 + 39] = 0x1a;
    expected[19 * 64 + 40] = 0x33;
    let mut map = old.clone();
    map.blocks = after.compile()?;
    ensure!(
        map.blocks == expected,
        "only two native mart front cells may change"
    );
    for (index, y) in [(7, 39), (8, 40)] {
        let warp = map
            .events
            .warps
            .iter_mut()
            .find(|w| w.index == index)
            .context("mart warp")?;
        ensure!(warp.x == 80 && warp.y == y, "mart warp preimage drift");
        warp.x = 79;
    }
    let mut rebound_commands = 0;
    for command in &mut map.map_event_section_commands {
        if command.command == "warp_event"
            && command.args.first().is_some_and(|v| v == "80")
            && command
                .args
                .get(1)
                .is_some_and(|v| matches!(v.as_str(), "39" | "40"))
        {
            ensure!(
                command.args.len() == 4
                    && command.args[0] == "80"
                    && matches!(command.args[1].as_str(), "39" | "40")
                    && ((command.args[1] == "39"
                        && command.args[2] == "MN_DULUTH_MART"
                        && command.args[3] == "3")
                        || (command.args[1] == "40"
                            && command.args[2] == "MN_DULUTH"
                            && command.args[3] == "8")),
                "raw mart warp preimage drift"
            );
            command.args[0] = "79".into();
            rebound_commands += 1;
        }
    }
    ensure!(rebound_commands == 2, "two raw mart warp records required");
    let reference = base
        .data
        .maps
        .get("AzaleaTown")
        .context("native mart reference")?;
    ensure!(
        reference.attributes.tileset_name == "johto_modern",
        "native reference art drift"
    );
    for dy in 0..2 {
        for dx in 0..2 {
            let block =
                reference.blocks[(1 + dy) * usize::from(reference.attributes.width) + 10 + dx];
            ensure!(
                map.blocks[(18 + dy) * 64 + 39 + dx] == block,
                "complete original mart assembly required"
            );
        }
    }
    ensure!(
        crate::authored_town_geometry::kind(base, &map, [79, 39])? == "DOOR"
            && crate::authored_town_geometry::kind(base, &map, [79, 40])? == "FLOOR",
        "existing mart anchors required"
    );
    for y in 0..92 {
        for x in 0..128 {
            let before = crate::authored_town_geometry::kind(base, old, [x, y])?;
            let after = crate::authored_town_geometry::kind(base, &map, [x, y])?;
            let expected = match [x, y] {
                [79, 39] => {
                    ensure!(before == "WALL", "new door preimage");
                    "DOOR"
                }
                [80, 39] => {
                    ensure!(before == "DOOR", "old door preimage");
                    "WALL"
                }
                _ => before,
            };
            ensure!(
                after == expected,
                "only relocated mart door collision may change"
            );
        }
    }
    const MANIFEST: &str = "minnesota-duluth-mart-facade-v1";
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
    let events_label = old
        .attributes
        .map_events_label
        .as_ref()
        .context("town event label")?;
    let raw_events = result
        .data
        .map_scripts
        .get_mut(events_label)
        .context("town raw event payload")?;
    let mut rebound_payloads = 0;
    for command in raw_events
        .as_array_mut()
        .context("raw event command array")?
    {
        if command["command"] == "warp_event"
            && command["args"][0] == "80"
            && matches!(command["args"][1].as_str(), Some("39" | "40"))
        {
            let args = command["args"]
                .as_array_mut()
                .context("raw warp arguments")?;
            ensure!(
                args.len() == 4
                    && args[0] == "80"
                    && matches!(args[1].as_str(), Some("39" | "40"))
                    && ((args[1] == "39" && args[2] == "MN_DULUTH_MART" && args[3] == "3")
                        || (args[1] == "40" && args[2] == "MN_DULUTH" && args[3] == "8")),
                "split raw mart warp preimage drift"
            );
            args[0] = "79".into();
            rebound_payloads += 1;
        }
    }
    ensure!(
        rebound_payloads == 2,
        "two split raw mart warp records required"
    );
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
