//! Encounter frostgrass on three existing Rainy River snow approaches.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::authored_route_layout::Layout;

const MAP: &str = "MnRainyRiverIcefields";
const ALIAS: &str = "mn_winter_outdoor";
const MANIFEST: &str = "minnesota-rainy-icefields-grass-v1";
const PATCHES: [(usize, usize); 9] = [
    (14, 18),
    (15, 18),
    (16, 18),
    (21, 10),
    (22, 10),
    (23, 10),
    (10, 2),
    (11, 2),
    (12, 2),
];

/// Replaces nine native snow metatiles with native encounter frostgrass.
///
/// Requires the complete current 32×24 source preimage. All ICE, forest,
/// stopping locations, actor/event footing, scripts, connections, encounters,
/// art and unrelated data remain unchanged. Snow and frostgrass both terminate
/// native sliding, so the movement graph is unchanged. This writes no file.
///
/// # Errors
/// Rejects source/profile/collision/preimage drift, affected actor/event anchors,
/// shared block labels, missing native encounter tables or repeated application.
pub fn revise_rainy_icefields_grass(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    ensure!(
        !base.report.manifests.iter().any(|s| s == MANIFEST),
        "Rainy grass already installed"
    );
    let mut a: Value = serde_json::from_str(before_json)?;
    let mut b: Value = serde_json::from_str(after_json)?;
    for v in [&mut a, &mut b] {
        v.as_object_mut().context("source object")?.remove("layers");
    }
    ensure!(
        a == b && a["shore_fitting"] == "explicit_source_edges",
        "only source layers may differ"
    );
    let before: Layout = serde_json::from_str(before_json)?;
    let after: Layout = serde_json::from_str(after_json)?;
    for source in [&before, &after] {
        ensure!(
            source.schema_version == 2
                && source.id == MAP
                && source.dimensions_metatiles == [32, 24]
                && source.compiled_tileset == ALIAS,
            "complete current Rainy source required"
        );
    }
    let old = base.data.maps.get(MAP).context("current Rainy map")?;
    ensure!(
        old.attributes.width == 32
            && old.attributes.height == 24
            && old.attributes.tileset_name == ALIAS
            && old.attributes.border_block == 0x3f
            && old.script_block_changes.is_empty(),
        "current native Rainy profile differs"
    );
    ensure!(
        before.compile()? == old.blocks,
        "entire source preimage differs"
    );
    ensure!(
        old.attributes.connections.len() == 1
            && old.attributes.connections[0].direction == "east"
            && old.attributes.connections[0].target_map == "MnR20C00R00"
            && old.attributes.connections[0].offset == 0,
        "Rainy R20 connection differs"
    );
    for (block, expected) in [
        (0x3c, "FLOOR"),
        (0x3d, "TALL_GRASS"),
        (0x3e, "ICE"),
        (0x3f, "WALL"),
    ] {
        ensure!(
            base.data.tilesets[ALIAS]
                .collision
                .get(&format!("{block:02x}"))
                .is_some_and(|c| c.iter().all(|k| k == expected) && c.len() == 4),
            "native winter collision differs"
        );
    }
    ensure!(
        base.data
            .wild_encounters
            .get(MAP)
            .is_some_and(|w| w.grass.is_some()),
        "native Rainy grass table required"
    );
    let label = old
        .attributes
        .blocks_label
        .as_ref()
        .context("native block label")?;
    ensure!(
        base.data
            .maps
            .iter()
            .all(|(n, m)| n == MAP || m.attributes.blocks_label.as_ref() != Some(label)),
        "shared block label"
    );
    ensure!(
        crate::decode_base64_bytes(base.data.map_blocks.get(label).context("block mirror")?)?
            .into_iter()
            .map(u16::from)
            .eq(old.blocks.iter().copied()),
        "old block mirror differs"
    );
    let protected = old
        .objects
        .iter()
        .map(|o| (usize::from(o.x / 2), usize::from(o.y / 2)))
        .chain(
            old.events
                .warps
                .iter()
                .map(|e| (usize::from(e.x / 2), usize::from(e.y / 2))),
        )
        .chain(
            old.events
                .bg_events
                .iter()
                .map(|e| (usize::from(e.x / 2), usize::from(e.y / 2))),
        )
        .chain(
            old.events
                .coord_events
                .iter()
                .map(|e| (usize::from(e.x / 2), usize::from(e.y / 2))),
        )
        .collect::<BTreeSet<_>>();
    let mut revised = old.clone();
    for (x, y) in PATCHES {
        ensure!(
            !protected.contains(&(x, y)) && revised.blocks[y * 32 + x] == 0x3c,
            "snow preimage or protected anchor differs at {x},{y}"
        );
        revised.blocks[y * 32 + x] = 0x3d;
    }
    ensure!(
        after.compile()? == revised.blocks,
        "only nine reviewed snow cells may change"
    );
    let bytes = revised
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut output = base.clone();
    output.data.maps.insert(MAP.into(), revised);
    output
        .data
        .map_blocks
        .insert(label.clone(), crate::encode_base64_bytes(&bytes));
    output.report.manifests.push(MANIFEST.into());
    output.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        output.format_version,
        &output.data,
        &output.audio_manifest,
        &output.runtime_files,
        &output.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&output)?;
    crate::validate_compact_superior_geography(&output)?;
    Ok(output)
}
