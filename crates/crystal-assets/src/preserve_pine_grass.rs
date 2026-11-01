//! Encounter bends and a native dry return on the public preserve approach.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::authored_route_layout::Layout;
use crate::authored_route_surface::walking_map_distances;
use crate::{CompiledGamePack, MapModule};

const MAP: &str = "MnPreservePineEntry";
const MANIFEST: &str = "minnesota-preserve-pine-grass-v1";
const PATCHES: [(usize, usize, u16, u16); 9] = [
    (8, 1, 0x01, 0x03),
    (5, 4, 0x01, 0x03),
    (9, 9, 0x01, 0x03),
    (6, 11, 0x01, 0x03),
    (6, 12, 0x01, 0x03),
    (6, 13, 0x01, 0x03),
    (7, 13, 0x01, 0x03),
    (8, 11, 0x05, 0x56),
    (8, 12, 0x05, 0x01),
];

/// Adds real habitat to the public pine trail and a dry southbound ledge return.
///
/// Requires exact native before/after layouts. Seven floor cells become grass;
/// two pine cells become a native ledge and landing. The sign, public seams,
/// every existing reachable tile and all admission rules remain unchanged.
/// Only this map's blocks/mirror, a new R20-derived wild table and provenance change.
/// Native gameplay and visual acceptance remain required before selection.
///
/// # Errors
/// Rejects stale source, changed native art, undeclared cells, affected events,
/// shared block labels, unsafe movement or invalid packs. The parent is borrowed.
pub fn revise_preserve_pine_grass(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    ensure!(
        !base.report.manifests.iter().any(|name| name == MANIFEST),
        "Preserve Pine grass already installed"
    );
    let before_art: Value = serde_json::from_str(before_json)?;
    let after_art: Value = serde_json::from_str(after_json)?;
    for key in ["tileset", "derived_blocks", "source_pixels_modified"] {
        ensure!(
            before_art.get(key) == after_art.get(key),
            "Preserve Pine cannot replace native art recipes"
        );
    }
    let before = Layout::parse(before_json)?;
    let after = Layout::parse(after_json)?;
    for source in [&before, &after] {
        ensure!(
            source.id == MAP
                && source.dimensions_metatiles == [24, 16]
                && source.compiled_tileset == "johto",
            "current Preserve Pine source required"
        );
    }
    let old = base
        .data
        .maps
        .get(MAP)
        .context("installed Preserve Pine map")?;
    ensure!(
        old.attributes.tileset_name == "johto"
            && [old.attributes.width, old.attributes.height] == [24, 16]
            && old.attributes.border_block == 5,
        "current Preserve Pine extent and art required"
    );
    ensure!(
        old.script_block_changes.is_empty(),
        "unreviewed Preserve Pine block programs"
    );
    ensure!(
        before.compile()? == old.blocks,
        "Preserve Pine source preimage differs"
    );
    let label = old
        .attributes
        .blocks_label
        .as_ref()
        .context("native Preserve Pine block label")?;
    ensure!(
        base.data
            .maps
            .iter()
            .all(|(name, map)| name == MAP || map.attributes.blocks_label.as_ref() != Some(label)),
        "shared Preserve Pine block label"
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
        "Preserve Pine native block mirror differs"
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
            "Preserve Pine patch preimage/anchor differs at {x},{y}"
        );
        module.blocks[y * 24 + x] = replacement;
    }
    ensure!(
        after.compile()? == module.blocks,
        "only the declared reviewed Preserve Pine cells may change"
    );
    validate_movement(base, old, &module)?;
    let bytes = module
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Preserve Pine native block range")?;
    ensure!(
        !base.data.wild_encounters.contains_key(MAP),
        "Preserve Pine already has a wild table"
    );
    let mut encounters = base
        .data
        .wild_encounters
        .get("MnR20C01R00")
        .context("current nearby R20 encounter table")?
        .clone();
    ensure!(
        encounters.grass.is_some()
            && encounters.water.is_some()
            && encounters.grass_rates.is_some()
            && encounters.water_rate.is_some(),
        "complete adjacent regional encounter table required"
    );
    encounters.map_name = MAP.into();
    encounters.zones.clear();
    encounters.swarm_overrides.clear();
    let mut output = base.clone();
    output.data.wild_encounters.insert(MAP.into(), encounters);
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
            .context("native pine trail block")?;
        ensure!(
            actual.iter().map(String::as_str).eq(expected),
            "native pine trail collision changed"
        );
    }
    let occupied = old
        .objects
        .iter()
        .map(|v| [v.x, v.y])
        .collect::<BTreeSet<_>>();
    for start in [[16, 0], [17, 0], [16, 31], [17, 31]] {
        let before = walking_map_distances(pack, old, start, &occupied)?;
        let after = walking_map_distances(pack, revised, start, &occupied)?;
        ensure!(
            before.keys().all(|at| after.contains_key(at)),
            "pine trail loses existing access"
        );
        ensure!(
            after.contains_key(&[13, 12]),
            "notice approach must remain accessible"
        );
    }
    let down = walking_map_distances(pack, revised, [17, 21], &occupied)?;
    ensure!(
        down.get(&[17, 24]) == Some(&3),
        "native pine trail descent changed"
    );
    let mut no_lip = occupied;
    no_lip.extend([[16, 22], [17, 22]]);
    let up = walking_map_distances(pack, revised, [17, 24], &no_lip)?;
    ensure!(
        up.get(&[17, 21]) == Some(&15),
        "ordinary pine trail return changed"
    );
    Ok(())
}
