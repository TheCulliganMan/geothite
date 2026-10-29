//! Native park habitat on the lookout, north shore and picnic approaches.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::authored_route_layout::Layout;
use crate::authored_route_surface::walking_map_distances;
use crate::{CompiledGamePack, MapModule};

const MAP: &str = "MnMilleLacs";
const MANIFEST: &str = "minnesota-mille-lacs-grass-v1";
const PATCHES: [(usize, usize, u16, u16); 9] = [
    (7, 7, 1, 3),
    (6, 8, 1, 3),
    (16, 4, 1, 3),
    (16, 5, 1, 3),
    (16, 6, 2, 3),
    (5, 12, 2, 3),
    (6, 12, 1, 3),
    (7, 12, 2, 3),
    (8, 12, 2, 3),
];

/// Adds native grass to the existing park's three ordinary activity paths.
///
/// Requires exact before/after editor cells against the installed 24×20 park.
/// Nine existing floor blocks become native grass. Original walking, dock,
/// ledge, Surf-only reward, resident and shore-boundary contracts remain.
/// Only this map's blocks/encoded mirror, a manifest and identity change.
/// No output is written, and native gameplay/visual acceptance remains required.
///
/// # Errors
/// Rejects stale source/payloads, another map/extent/alias, shared labels,
/// changed art, undeclared cells, affected event/actor anchors, dynamic block
/// programs, incompatible native collision, unsafe movement or invalid packs.
/// The borrowed parent remains unchanged on success and failure.
pub fn revise_mille_lacs_grass(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    ensure!(
        !base.report.manifests.iter().any(|name| name == MANIFEST),
        "Mille Lacs grass already installed"
    );
    let before_art: Value = serde_json::from_str(before_json)?;
    let after_art: Value = serde_json::from_str(after_json)?;
    for key in ["tileset", "derived_blocks", "source_pixels_modified"] {
        ensure!(
            before_art.get(key) == after_art.get(key),
            "Mille Lacs cannot replace native art recipes"
        );
    }
    let before = Layout::parse(before_json)?;
    let after = Layout::parse(after_json)?;
    for source in [&before, &after] {
        ensure!(
            source.id == MAP
                && source.dimensions_metatiles == [24, 20]
                && source.compiled_tileset == "mn_headwaters",
            "current Mille Lacs source required"
        );
    }
    let old = base
        .data
        .maps
        .get(MAP)
        .context("installed Mille Lacs map")?;
    ensure!(
        old.attributes.tileset_name == "mn_headwaters"
            && [old.attributes.width, old.attributes.height] == [24, 20]
            && old.attributes.border_block == 5,
        "current Mille Lacs extent and art required"
    );
    ensure!(
        old.script_block_changes.is_empty(),
        "unreviewed Mille Lacs block programs"
    );
    ensure!(
        before.compile()? == old.blocks,
        "Mille Lacs source preimage differs"
    );
    let label = old
        .attributes
        .blocks_label
        .as_ref()
        .context("native Mille Lacs block label")?;
    ensure!(
        base.data
            .maps
            .iter()
            .all(|(name, map)| name == MAP || map.attributes.blocks_label.as_ref() != Some(label)),
        "shared Mille Lacs block label"
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
        "Mille Lacs native block mirror differs"
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
            !protected.contains(&(x, y)) && module.blocks[y * 24 + x] == expected,
            "Mille Lacs patch preimage/anchor differs at {x},{y}"
        );
        module.blocks[y * 24 + x] = replacement;
    }
    ensure!(
        after.compile()? == module.blocks,
        "only the nine reviewed Mille Lacs cells may change"
    );
    validate_movement(base, old, &module)?;
    let bytes = module
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Mille Lacs native block range")?;
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
        .get("mn_headwaters")
        .context("native Johto collision")?;
    for (block, expected) in [(1, ["FLOOR"; 4]), (3, ["TALL_GRASS"; 4]), (2, ["FLOOR"; 4])] {
        let actual = definition
            .collision
            .get(&format!("{block:02x}"))
            .context("Mille Lacs native block")?;
        ensure!(
            actual.iter().map(String::as_str).eq(expected),
            "Mille Lacs native collision recipe changed"
        );
    }
    let occupied = old
        .objects
        .iter()
        .map(|v| [v.x, v.y])
        .collect::<BTreeSet<_>>();
    for start in [[0, 12], [0, 13]] {
        let before = walking_map_distances(pack, old, start, &occupied)?;
        let after = walking_map_distances(pack, revised, start, &occupied)?;
        ensure!(
            before.keys().all(|at| after.contains_key(at)),
            "Mille Lacs loses existing walking access"
        );
    }
    crate::mille_lacs_park::validate_current_terrain(pack, revised)?;
    Ok(())
}
