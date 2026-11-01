//! Native grass on the green side of the existing Ely-to-coast pass.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::authored_route_layout::Layout;
use crate::authored_route_surface::walking_map_distances;
use crate::{CompiledGamePack, MapModule};

const MAP: &str = "MnElyShorePass";
const MANIFEST: &str = "minnesota-ely-shore-pass-grass-v1";
const PATCHES: [(usize, usize, u16, u16); 2] = [(4, 4, 2, 3), (5, 5, 1, 3)];

/// Installs grass on the existing green approach and coastal exit.
///
/// Requires exact before/after editor cells against the installed 7×12 pass.
/// Two existing green floor blocks become native grass. Every existing snowy
/// inland cell, climate boundary, actor, port and walking access stays exact.
/// Only this map's blocks/encoded mirror, a manifest and identity change.
/// No output is written, and native gameplay/visual acceptance remains required.
///
/// # Errors
/// Rejects stale source/payloads, another map/extent/alias, shared labels,
/// changed art, undeclared cells, affected event/actor anchors, dynamic block
/// programs, incompatible native collision, unsafe movement or invalid packs.
/// The borrowed parent remains unchanged on success and failure.
pub fn revise_ely_shore_pass_grass(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    ensure!(
        !base.report.manifests.iter().any(|name| name == MANIFEST),
        "Ely pass grass already installed"
    );
    let before_art: Value = serde_json::from_str(before_json)?;
    let after_art: Value = serde_json::from_str(after_json)?;
    for key in ["tileset", "derived_blocks", "source_pixels_modified"] {
        ensure!(
            before_art.get(key) == after_art.get(key),
            "Ely pass cannot replace native art recipes"
        );
    }
    let before = Layout::parse(before_json)?;
    let after = Layout::parse(after_json)?;
    for source in [&before, &after] {
        ensure!(
            source.id == MAP
                && source.dimensions_metatiles == [7, 12]
                && source.compiled_tileset == "mn_winter_north_shore",
            "current Ely pass source required"
        );
    }
    let old = base.data.maps.get(MAP).context("installed Ely pass map")?;
    ensure!(
        old.attributes.tileset_name == "mn_winter_north_shore"
            && [old.attributes.width, old.attributes.height] == [7, 12]
            && old.attributes.border_block == 0x8b,
        "current Ely pass extent and art required"
    );
    ensure!(
        old.script_block_changes.is_empty(),
        "unreviewed Ely pass block programs"
    );
    ensure!(
        before.compile()? == old.blocks,
        "Ely pass source preimage differs"
    );
    let label = old
        .attributes
        .blocks_label
        .as_ref()
        .context("native Ely pass block label")?;
    ensure!(
        base.data
            .maps
            .iter()
            .all(|(name, map)| name == MAP || map.attributes.blocks_label.as_ref() != Some(label)),
        "shared Ely pass block label"
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
        "Ely pass native block mirror differs"
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
            !protected.contains(&(x, y)) && module.blocks[y * 7 + x] == expected,
            "Ely pass patch preimage/anchor differs at {x},{y}"
        );
        module.blocks[y * 7 + x] = replacement;
    }
    ensure!(
        after.compile()? == module.blocks,
        "only the two reviewed Ely pass cells may change"
    );
    validate_movement(base, old, &module)?;
    let bytes = module
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Ely pass native block range")?;
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
        .get("mn_winter_north_shore")
        .context("native Johto collision")?;
    for (block, expected) in [(1, ["FLOOR"; 4]), (3, ["TALL_GRASS"; 4]), (2, ["FLOOR"; 4])] {
        let actual = definition
            .collision
            .get(&format!("{block:02x}"))
            .context("Ely pass native block")?;
        ensure!(
            actual.iter().map(String::as_str).eq(expected),
            "Ely pass native collision recipe changed"
        );
    }
    let occupied = old
        .objects
        .iter()
        .map(|v| [v.x, v.y])
        .collect::<BTreeSet<_>>();
    for start in [[0, 14], [0, 15], [13, 10], [13, 11]] {
        let before = walking_map_distances(pack, old, start, &occupied)?;
        let after = walking_map_distances(pack, revised, start, &occupied)?;
        ensure!(
            before.keys().all(|at| after.contains_key(at)),
            "Ely pass loses existing walking access"
        );
    }
    ensure!(
        old.blocks
            .iter()
            .zip(&revised.blocks)
            .all(|(a, b)| *a < 0x83 || a == b),
        "Ely inland winter cells changed"
    );
    Ok(())
}
