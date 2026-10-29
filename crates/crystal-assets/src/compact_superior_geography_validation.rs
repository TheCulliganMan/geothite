//! Reject retired northern geometry without freezing unrelated map content.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, bail, ensure};

use crate::CompiledGamePack;

/// Requires the reviewed compact, warm North Shore and inland Gunflint layout.
///
/// Checks the exact R11/R12 chunk footprints, Grand Marais/Cedar dimensions,
/// their critical reciprocal connections and the warm coast's non-ICE terrain.
/// All connected Minnesota outdoor footprints must have consistent placements and no
/// overlaps. Legitimate grass, trainer, fishing, interior and cave revisions
/// remain possible; this does not freeze their cells or claim visual acceptance.
/// Intended as a pre/postcondition for current Minnesota publishers. It does
/// not mutate the pack or migrate retired geometry.
///
/// # Errors
/// Rejects missing/extra retired chunks, changed canonical dimensions or seams,
/// a winter/ICE coast, missing terrain definitions, inconsistent placement
/// cycles, unsupported connection directions or overlapping outdoor maps.
pub fn validate_compact_superior_geography(pack: &CompiledGamePack) -> Result<()> {
    ensure!(
        !pack.data.maps.contains_key("MnMaraisInlandNeck"),
        "retired Grand Marais inland neck cannot be current geography"
    );
    let coast = [
        ("MnR11C00R00", [32, 24]),
        ("MnR11C00R01", [32, 24]),
        ("MnR11C00R02", [32, 24]),
        ("MnR11C00R03", [32, 24]),
        ("MnR11C01R00", [4, 24]),
        ("MnR11C01R01", [24, 24]),
        ("MnR11C01R02", [24, 24]),
        ("MnR11C01R03", [24, 24]),
    ];
    let inland = [("MnR12C00R00", [32, 24]), ("MnR12C00R01", [32, 8])];
    for (prefix, expected) in [("MnR11C", coast.as_slice()), ("MnR12C", inland.as_slice())] {
        let actual = pack
            .data
            .maps
            .keys()
            .filter_map(|name| {
                let suffix = name.strip_prefix(prefix)?.as_bytes();
                (suffix.len() == 5
                    && suffix[2] == b'R'
                    && [0, 1, 3, 4].iter().all(|&i| suffix[i].is_ascii_digit()))
                .then_some(name.as_str())
            })
            .collect::<BTreeSet<_>>();
        ensure!(
            actual == expected.iter().map(|(name, _)| *name).collect(),
            "retired or missing compact {prefix} chunks"
        );
    }
    for (name, size) in coast.into_iter().chain(inland).chain([
        ("MnGrandMarais", [20, 19]),
        ("MnMaraisCedarApproach", [20, 12]),
    ]) {
        let map = pack
            .data
            .maps
            .get(name)
            .context("canonical northern map missing")?;
        ensure!(
            [map.attributes.width, map.attributes.height] == size,
            "retired northern footprint: {name}"
        );
        if name.starts_with("MnR11C") || name == "MnMaraisCedarApproach" {
            ensure!(
                map.attributes.tileset_name == "mn_north_shore",
                "North Shore and its Cedar approach must retain their warm native tileset: {name}"
            );
            let definition = pack
                .data
                .tilesets
                .get("mn_north_shore")
                .context("warm coast tileset")?;
            for block in map
                .blocks
                .iter()
                .copied()
                .chain(std::iter::once(u16::from(map.attributes.border_block)))
                .chain(map.script_block_changes.iter().map(|row| row.block_id))
            {
                let permissions = definition
                    .collision
                    .get(&format!("{block:02x}"))
                    .context("coast collision block")?;
                ensure!(
                    permissions.iter().all(|kind| kind != "ICE"),
                    "North Shore cannot contain ICE: {name}"
                );
            }
        }
    }
    for (from, direction, to, offset) in [
        ("MnR11C00R00", "east", "MnR11C01R00", 0),
        ("MnR11C01R00", "west", "MnR11C00R00", 0),
        ("MnR11C01R00", "east", "MnGrandMarais", 0),
        ("MnGrandMarais", "west", "MnR11C01R00", 0),
        ("MnGrandMarais", "north", "MnMaraisCedarApproach", -4),
        ("MnMaraisCedarApproach", "south", "MnGrandMarais", 4),
        ("MnMaraisCedarApproach", "west", "MnR12C00R01", 4),
        ("MnR12C00R01", "east", "MnMaraisCedarApproach", -4),
        ("MnR12C00R00", "south", "MnR12C00R01", 0),
        ("MnR12C00R01", "north", "MnR12C00R00", 0),
    ] {
        let edges = &pack.data.maps[from].attributes.connections;
        ensure!(
            edges
                .iter()
                .filter(|edge| edge.direction == direction)
                .count()
                == 1
                && edges.iter().any(|edge| edge.direction == direction
                    && edge.target_map == to
                    && edge.offset == offset),
            "canonical northern connection differs: {from} {direction}"
        );
    }
    validate_outdoor_placements(pack)
}

fn validate_outdoor_placements(pack: &CompiledGamePack) -> Result<()> {
    let maps = pack
        .data
        .maps
        .iter()
        .filter(|(name, map)| {
            name.starts_with("Mn")
                && matches!(
                    map.attributes.environment.as_deref(),
                    Some("TOWN" | "ROUTE" | "OUTDOOR")
                )
        })
        .map(|(name, map)| (name.as_str(), map))
        .collect::<BTreeMap<_, _>>();
    let mut positions = BTreeMap::new();
    for &root in maps.keys() {
        if positions.contains_key(root) {
            continue;
        }
        positions.insert(root, (0_i64, 0_i64));
        let mut todo = VecDeque::from([root]);
        let mut component = Vec::new();
        while let Some(name) = todo.pop_front() {
            component.push(name);
            let (x, y) = positions[name];
            let a = &maps[name].attributes;
            for edge in &a.connections {
                let target_name = edge.target_map.as_str();
                let Some(target) = maps.get(target_name) else {
                    continue;
                };
                let b = &target.attributes;
                let offset = i64::from(edge.offset);
                let at = match edge.direction.as_str() {
                    "north" => (x + offset, y - i64::from(b.height)),
                    "south" => (x + offset, y + i64::from(a.height)),
                    "west" => (x - i64::from(b.width), y + offset),
                    "east" => (x + i64::from(a.width), y + offset),
                    _ => bail!("unsupported exterior connection direction"),
                };
                if let Some(previous) = positions.get(target_name) {
                    ensure!(
                        *previous == at,
                        "inconsistent outdoor placement: {name}/{target_name}"
                    );
                } else {
                    positions.insert(target_name, at);
                    todo.push_back(target_name);
                }
            }
        }
        for (index, &name) in component.iter().enumerate() {
            let (x, y) = positions[name];
            let a = &maps[name].attributes;
            for &other in &component[index + 1..] {
                let (tx, ty) = positions[other];
                let b = &maps[other].attributes;
                ensure!(
                    x + i64::from(a.width) <= tx
                        || tx + i64::from(b.width) <= x
                        || y + i64::from(a.height) <= ty
                        || ty + i64::from(b.height) <= y,
                    "outdoor footprints overlap: {name}/{other}"
                );
            }
        }
    }
    Ok(())
}
