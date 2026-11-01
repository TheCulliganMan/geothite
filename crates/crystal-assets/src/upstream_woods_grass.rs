//! A grassy eastbound loop and dry westward return on the upper Mississippi.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::authored_route_layout::Layout;
use crate::authored_route_surface::walking_map_distances;
use crate::{CompiledGamePack, MapModule};

const MAP: &str = "MnMississippiUpstreamWoodsC01R00";
const PREFIX: &str = "MnMississippiUpstreamWoods";
const MANIFEST: &str = "minnesota-upstream-woods-grass-v1";
const PATCHES: [(usize, usize, u16, u16); 5] = [
    (4, 9, 0x03, 0x4c),
    (5, 9, 0x03, 0x01),
    (5, 8, 0x02, 0x03),
    (7, 9, 0x02, 0x03),
    (7, 10, 0x02, 0x03),
];

/// Revises five cells on the northern Ether bank.
///
/// Requires exact full 64×68 source preimages for all six current river chunks.
/// The river, shores, actors, activities, wild tables and all previous walking access except the two new reverse-wall quadrants remain intact. Only one chunk's blocks/mirror and
/// publication identity/provenance change. The parent is borrowed unchanged.
/// Native controller and 2D acceptance remain required before selection.
///
/// # Errors
/// Rejects stale source, undeclared cells, altered native art/schema, changed
/// profiles, shared block labels, affected events, unsafe movement or invalid packs.
pub fn revise_upstream_woods_grass(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    ensure!(
        !base.report.manifests.iter().any(|name| name == MANIFEST),
        "Upstream Woods grass already installed"
    );
    let mut before_header: Value = serde_json::from_str(before_json)?;
    let mut after_header: Value = serde_json::from_str(after_json)?;
    for header in [&mut before_header, &mut after_header] {
        header
            .as_object_mut()
            .context("river source object")?
            .remove("layers");
    }
    ensure!(
        before_header == after_header,
        "only source terrain layers may change"
    );
    ensure!(
        before_header["shore_fitting"] == "explicit_source_edges",
        "native river source fitting required"
    );
    let before: Layout = serde_json::from_str(before_json)?;
    let after: Layout = serde_json::from_str(after_json)?;
    for layout in [&before, &after] {
        ensure!(
            layout.schema_version == 2
                && layout.id == PREFIX
                && layout.dimensions_metatiles == [64, 68]
                && layout.compiled_tileset == "johto",
            "complete current Upstream Woods source required"
        );
    }
    let raster = installed_raster(base)?;
    ensure!(
        before.compile()? == raster,
        "full river source preimage differs"
    );
    let old = base.data.maps.get(MAP).context("current northern bank")?;
    ensure!(
        old.script_block_changes.is_empty(),
        "unreviewed bank block programs"
    );
    ensure!(
        base.data
            .wild_encounters
            .get(MAP)
            .is_some_and(|w| w.grass.is_some()),
        "native bank encounters required"
    );
    let label = old
        .attributes
        .blocks_label
        .as_ref()
        .context("native bank block label")?;
    ensure!(
        base.data
            .maps
            .iter()
            .all(|(name, map)| name == MAP || map.attributes.blocks_label.as_ref() != Some(label)),
        "shared bank block label"
    );
    ensure!(
        crate::decode_base64_bytes(
            base.data
                .map_blocks
                .get(label)
                .context("native block mirror")?
        )?
        .into_iter()
        .map(u16::from)
        .eq(old.blocks.iter().copied()),
        "native bank mirror differs"
    );
    let protected = old
        .objects
        .iter()
        .map(|v| (usize::from(v.x / 2), usize::from(v.y / 2)))
        .chain(
            old.events
                .warps
                .iter()
                .map(|v| (usize::from(v.x / 2), usize::from(v.y / 2))),
        )
        .chain(
            old.events
                .bg_events
                .iter()
                .map(|v| (usize::from(v.x / 2), usize::from(v.y / 2))),
        )
        .chain(
            old.events
                .coord_events
                .iter()
                .map(|v| (usize::from(v.x / 2), usize::from(v.y / 2))),
        )
        .collect::<BTreeSet<_>>();
    let mut module = old.clone();
    let mut expected = raster;
    for (x, y, preimage, replacement) in PATCHES {
        ensure!(
            !protected.contains(&(x, y)) && module.blocks[y * 32 + x] == preimage,
            "bank patch preimage/anchor differs at {x},{y}"
        );
        module.blocks[y * 32 + x] = replacement;
        expected[y * 64 + x + 32] = replacement;
    }
    ensure!(
        after.compile()? == expected,
        "only the five reviewed bank cells may change"
    );
    validate_movement(base, old, &module)?;
    let bytes = module
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("native bank block range")?;
    let mut output = base.clone();
    output.data.maps.insert(MAP.into(), module);
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
    crate::validate_compact_superior_geography(&output)?;
    crate::verify_compiled_game_pack_for_runtime(&output)?;
    Ok(output)
}

fn installed_raster(pack: &CompiledGamePack) -> Result<Vec<u16>> {
    let mut raster = vec![0; 64 * 68];
    for row in 0..3 {
        for col in 0..2 {
            let name = format!("{PREFIX}C{col:02}R{row:02}");
            let map = pack.data.maps.get(&name).context("current river chunk")?;
            let height = if row == 2 { 20 } else { 24 };
            ensure!(
                [map.attributes.width, map.attributes.height] == [32, height]
                    && map.attributes.tileset_name == "johto"
                    && map.attributes.border_block == 5,
                "current river profile differs"
            );
            for y in 0..usize::from(height) {
                for x in 0..32 {
                    raster[(row * 24 + y) * 64 + col * 32 + x] = map.blocks[y * 32 + x];
                }
            }
        }
    }
    Ok(raster)
}

fn validate_movement(pack: &CompiledGamePack, old: &MapModule, revised: &MapModule) -> Result<()> {
    let definition = pack
        .data
        .tilesets
        .get("johto")
        .context("native Johto collision")?;
    for (block, expected) in [
        (1, ["FLOOR"; 4]),
        (2, ["FLOOR"; 4]),
        (3, ["TALL_GRASS"; 4]),
        (5, ["WALL"; 4]),
        (0x4c, ["WALL", "HOP_LEFT", "WALL", "HOP_LEFT"]),
    ] {
        let actual = definition
            .collision
            .get(&format!("{block:02x}"))
            .context("native bank collision block")?;
        ensure!(
            actual.iter().map(String::as_str).eq(expected),
            "native bank collision recipe changed"
        );
    }
    let occupied = old
        .objects
        .iter()
        .map(|v| [v.x, v.y])
        .collect::<BTreeSet<_>>();
    for start in [[7, 18], [7, 19], [10, 19], [20, 19]] {
        let before = walking_map_distances(pack, old, start, &occupied)?;
        let after = walking_map_distances(pack, revised, start, &occupied)?;
        ensure!(
            before
                .keys()
                .all(|at| after.contains_key(at) || matches!(*at, [8, 18 | 19])),
            "bank loses original walking access"
        );
    }
    for (y, distance) in [(18, 7), (19, 5)] {
        let west = walking_map_distances(pack, revised, [10, y], &occupied)?;
        ensure!(
            west.get(&[7, y]) == Some(&3),
            "native westward bank hop changed"
        );
        let mut no_lip = occupied.clone();
        no_lip.extend([[9, 18], [9, 19]]);
        let east = walking_map_distances(pack, revised, [7, y], &no_lip)?;
        ensure!(
            east.get(&[10, y]) == Some(&distance),
            "ordinary grassy eastbound bank return changed"
        );
    }
    Ok(())
}
