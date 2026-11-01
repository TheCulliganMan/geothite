//! Reviewed grass bends and a native one-way return through the Northwoods.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::authored_route_layout::Layout;
use crate::authored_route_surface::walking_map_distances;
use crate::{CompiledGamePack, MapModule};

const MAP: &str = "MnBemidjiNorthwoods";
const MANIFEST: &str = "minnesota-bemidji-northwoods-grass-v1";
const PATCHES: [(usize, usize, u16, u16); 8] = [
    (4, 1, 1, 3),
    (4, 2, 1, 3),
    (5, 6, 1, 3),
    (5, 7, 1, 3),
    (2, 10, 1, 3),
    (2, 11, 1, 3),
    (3, 6, 5, 1),
    (3, 7, 5, 0x56),
];

/// Installs the reviewed Northwoods grass bends and southbound grove shortcut.
///
/// Requires exact before/after editor cells against the installed 8×14 route.
/// Six existing road blocks become grass; two interior forest blocks become
/// the ledge approach and native south-facing ledge. All existing walking
/// cells remain reachable from both ports. The local ordinary return takes
/// thirteen walking tiles, compared with the three-tile dry ledge descent.
/// Only this map's blocks/encoded mirror, a manifest and identity change.
/// No output is written, and native gameplay/visual acceptance remains required.
///
/// # Errors
/// Rejects stale source/payloads, another map/extent/alias, shared labels,
/// changed art, undeclared cells, affected event/actor anchors, dynamic block
/// programs, incompatible native collision, unsafe movement or invalid packs.
/// The borrowed parent remains unchanged on success and failure.
pub fn revise_bemidji_northwoods_grass(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    ensure!(
        !base.report.manifests.iter().any(|name| name == MANIFEST),
        "Northwoods grass already installed"
    );
    let before_art: Value = serde_json::from_str(before_json)?;
    let after_art: Value = serde_json::from_str(after_json)?;
    for key in ["tileset", "derived_blocks", "source_pixels_modified"] {
        ensure!(
            before_art.get(key) == after_art.get(key),
            "Northwoods cannot replace native art recipes"
        );
    }
    let before = Layout::parse(before_json)?;
    let after = Layout::parse(after_json)?;
    for source in [&before, &after] {
        ensure!(
            source.id == MAP
                && source.dimensions_metatiles == [8, 14]
                && source.compiled_tileset == "johto",
            "current Northwoods source required"
        );
    }
    let old = base
        .data
        .maps
        .get(MAP)
        .context("installed Northwoods map")?;
    ensure!(
        old.attributes.tileset_name == "johto"
            && [old.attributes.width, old.attributes.height] == [8, 14]
            && old.attributes.border_block == 5,
        "current Northwoods extent and art required"
    );
    ensure!(
        old.script_block_changes.is_empty(),
        "unreviewed Northwoods block programs"
    );
    ensure!(
        before.compile()? == old.blocks,
        "Northwoods source preimage differs"
    );
    let label = old
        .attributes
        .blocks_label
        .as_ref()
        .context("native Northwoods block label")?;
    ensure!(
        base.data
            .maps
            .iter()
            .all(|(name, map)| name == MAP || map.attributes.blocks_label.as_ref() != Some(label)),
        "shared Northwoods block label"
    );
    ensure!(
        crate::decode_base64_bytes(
            base.data
                .map_blocks
                .get(label)
                .context("native encoded blocks")?
        )?
        .into_iter()
        .map(u16::from)
        .eq(old.blocks.iter().copied()),
        "Northwoods native block mirror differs"
    );
    let mut protected = BTreeSet::new();
    protected.extend(
        old.objects
            .iter()
            .map(|v| (usize::from(v.x / 2), usize::from(v.y / 2))),
    );
    protected.extend(
        old.events
            .warps
            .iter()
            .map(|v| (usize::from(v.x / 2), usize::from(v.y / 2))),
    );
    protected.extend(
        old.events
            .bg_events
            .iter()
            .map(|v| (usize::from(v.x / 2), usize::from(v.y / 2))),
    );
    protected.extend(
        old.events
            .coord_events
            .iter()
            .map(|v| (usize::from(v.x / 2), usize::from(v.y / 2))),
    );
    let mut module = old.clone();
    for (x, y, expected, replacement) in PATCHES {
        ensure!(
            !protected.contains(&(x, y)) && module.blocks[y * 8 + x] == expected,
            "Northwoods patch preimage/anchor differs at {x},{y}"
        );
        module.blocks[y * 8 + x] = replacement;
    }
    ensure!(
        after.compile()? == module.blocks,
        "only the eight reviewed Northwoods cells may change"
    );
    validate_movement(base, old, &module)?;
    let bytes = module
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Northwoods native block range")?;
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

fn validate_movement(pack: &CompiledGamePack, old: &MapModule, revised: &MapModule) -> Result<()> {
    let definition = pack
        .data
        .tilesets
        .get("johto")
        .context("native Johto collision")?;
    for (block, expected) in [
        (1, ["FLOOR"; 4]),
        (3, ["TALL_GRASS"; 4]),
        (5, ["WALL"; 4]),
        (0x56, ["HOP_DOWN", "HOP_DOWN", "WALL", "WALL"]),
    ] {
        let actual = definition
            .collision
            .get(&format!("{block:02x}"))
            .context("Northwoods native block")?;
        ensure!(
            actual.iter().map(String::as_str).eq(expected),
            "Northwoods native collision recipe changed"
        );
    }
    let occupied = old
        .objects
        .iter()
        .map(|v| [v.x, v.y])
        .collect::<BTreeSet<_>>();
    for start in [[8, 0], [9, 0], [8, 27], [9, 27]] {
        let before = walking_map_distances(pack, old, start, &occupied)?;
        let after = walking_map_distances(pack, revised, start, &occupied)?;
        ensure!(
            before.keys().all(|at| after.contains_key(at)),
            "Northwoods loses existing walking access"
        );
    }
    let downhill = walking_map_distances(pack, revised, [7, 13], &occupied)?;
    ensure!(
        downhill.get(&[7, 16]) == Some(&3),
        "native Northwoods drop is not effective"
    );
    let mut ordinary = occupied;
    ordinary.extend([[6, 14], [7, 14]]);
    let uphill = walking_map_distances(pack, revised, [7, 16], &ordinary)?;
    ensure!(
        uphill.get(&[7, 13]) == Some(&13),
        "ordinary Northwoods return changed"
    );
    Ok(())
}
