//! Fit the Headwaters survey ledge to its existing forest and legal landings.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_route_layout::Layout;

/// Closes six unusable ledge metatiles, retaining the four central hop lanes.
///
/// # Errors
/// Rejects stale full-route source, altered art, events, forest or landing geometry.
pub fn revise_r14_forest_bank(
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
            "art contract drift: {field}"
        );
    }
    ensure!(
        after_value["source_pixels_modified"] == false,
        "native pixels required"
    );
    let before: Layout = serde_json::from_value(before_value)?;
    let after: Layout = serde_json::from_value(after_value)?;
    ensure!(
        before.schema_version == 2
            && after.schema_version == 2
            && before.id == "MN_R14"
            && after.id == before.id
            && before.dimensions_metatiles == [48, 32]
            && after.dimensions_metatiles == before.dimensions_metatiles
            && before.compiled_tileset == "mn_headwaters"
            && after.compiled_tileset == before.compiled_tileset,
        "route identity drift"
    );
    let mut native = Vec::with_capacity(48 * 32);
    for y in 0..32 {
        for x in 0..48 {
            let name = format!("MnR14C{:02}R{:02}", x / 32, y / 24);
            let map = base.data.maps.get(&name).context("route chunk")?;
            ensure!(
                map.attributes.tileset_name == "mn_headwaters",
                "chunk art drift"
            );
            native.push(map.blocks[(y % 24) * usize::from(map.attributes.width) + x % 32]);
        }
    }
    ensure!(
        before.compile()? == native,
        "complete native route preimage drift"
    );
    let old = base
        .data
        .maps
        .get("MnR14C00R00")
        .context("survey bank chunk")?;
    ensure!(
        old.attributes.width == 32 && old.attributes.height == 24,
        "chunk extent drift"
    );
    let mut map = old.clone();
    for x in [10, 11, 12, 13, 16, 17] {
        ensure!(
            native[12 * 48 + x] == 0x57 && old.blocks[13 * 32 + x] == 0x05,
            "close only existing ledges with forest-blocked landings"
        );
        native[12 * 48 + x] = 0x05;
        map.blocks[12 * 32 + x] = 0x05;
    }
    ensure!(
        after.compile()? == native,
        "only six forest-bank cells may change"
    );
    for y in 0..48 {
        for x in 0..64 {
            let previous = crate::authored_town_geometry::kind(base, old, [x, y])?;
            let current = crate::authored_town_geometry::kind(base, &map, [x, y])?;
            if y == 24 && ((20..28).contains(&x) || (32..36).contains(&x)) {
                ensure!(
                    previous == "HOP_DOWN" && current == "WALL",
                    "close only unusable hop faces"
                );
                ensure!(
                    !old.objects.iter().any(|o| [o.x, o.y] == [x, y]),
                    "no actor may be closed"
                );
                ensure!(
                    !old.events
                        .warps
                        .iter()
                        .map(|e| [e.x, e.y])
                        .chain(old.events.coord_events.iter().map(|e| [e.x, e.y]))
                        .chain(old.events.bg_events.iter().map(|e| [e.x, e.y]))
                        .any(|at| at == [x, y]),
                    "no event may be closed"
                );
            } else {
                ensure!(previous == current, "all other collision must remain exact");
            }
        }
    }
    for x in 28..32 {
        for (y, kind) in [(23, "FLOOR"), (24, "HOP_DOWN"), (25, "WALL"), (26, "FLOOR")] {
            ensure!(
                crate::authored_town_geometry::kind(base, &map, [x, y])? == kind,
                "retain all four legal native hop lanes"
            );
        }
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
        "unique block payload required"
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
        .context("encoded native block payload")? = crate::encode_base64_bytes(&bytes);
    result
        .report
        .manifests
        .push("minnesota-r14-forest-bank-v1".into());
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
