//! Small explicit grassy recesses along the two existing wooded river reaches.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;

#[derive(Deserialize)]
struct Plan {
    schema_version: u8,
    maps: Vec<Map>,
}
#[derive(Deserialize)]
struct Map {
    id: String,
    expected_blocks_sha256: String,
    cells: Vec<Cell>,
}
#[derive(Deserialize)]
struct Cell {
    at: [u16; 2],
    before: u16,
    after: u16,
}

/// Adds the recorded bank recesses without changing any existing walking tile.
///
/// Only native solid canopy may become grass or Floor. Original water, launch
/// tiles, event anchors, map boundaries, actors, scripts, art and audio remain
/// exact. Per-map preimages allow composition after unrelated map revisions.
///
/// # Errors
/// Rejects stale river terrain, retired northern geometry, icy North Shore,
/// duplicate cells, edits outside the two reaches, or non-canopy replacements.
pub fn refine_mississippi_banks(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::river_content_lineage::validate(base)?;
    let plan: Plan = serde_json::from_str(source)?;
    ensure!(
        plan.schema_version == 1 && !plan.maps.is_empty() && plan.maps.len() <= 12,
        "bounded bank source required"
    );
    let mut pack = base.clone();
    let mut names = BTreeSet::new();
    let mut count = 0;
    let mut changed = Vec::new();
    for row in plan.maps {
        ensure!(
            names.insert(row.id.clone())
                && (row.id.starts_with("MnMississippiLowerWoodsC")
                    || row.id.starts_with("MnMississippiUpstreamWoodsC")),
            "distinct wooded river chunks required"
        );
        let old = base.data.maps.get(&row.id).context("river chunk")?;
        let bytes = old
            .blocks
            .iter()
            .copied()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(
            old.attributes.tileset_name == "johto"
                && format!("{:x}", Sha256::digest(bytes)) == row.expected_blocks_sha256,
            "river bank preimage differs: {}",
            row.id
        );
        let mut map = old.clone();
        let mut positions = BTreeSet::new();
        for cell in row.cells {
            let [x, y] = cell.at;
            ensure!(
                positions.insert(cell.at)
                    && x > 0
                    && y > 0
                    && x + 1 < old.attributes.width
                    && y + 1 < old.attributes.height,
                "bank recess must remain inside its native chunk"
            );
            let index = usize::from(y) * usize::from(old.attributes.width) + usize::from(x);
            ensure!(
                old.blocks[index] == cell.before && cell.before == 5 && matches!(cell.after, 1 | 3),
                "bank recess must replace solid native canopy only"
            );
            map.blocks[index] = cell.after;
            changed.push((row.id.clone(), cell.at));
            count += 1;
        }
        crate::authored_town_publication::replace(&mut pack, map)?;
    }
    ensure!(
        (1..=96).contains(&count),
        "bank revision exceeds reviewed extent"
    );
    validate_access(&pack, &changed)?;
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::river_content_lineage::validate(&pack)?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn origin(name: &str) -> [u16; 2] {
    let col = if name.contains("C00") { 0 } else { 64 };
    let row = if name.ends_with("R00") {
        0
    } else if name.ends_with("R01") {
        48
    } else {
        96
    };
    [col, row]
}

fn neighbors([x, y]: [u16; 2]) -> impl Iterator<Item = [u16; 2]> {
    [
        x.checked_sub(1).map(|v| [v, y]),
        x.checked_add(1).map(|v| [v, y]),
        y.checked_sub(1).map(|v| [x, v]),
        y.checked_add(1).map(|v| [x, v]),
    ]
    .into_iter()
    .flatten()
}

fn validate_access(pack: &CompiledGamePack, changed: &[(String, [u16; 2])]) -> Result<()> {
    for prefix in ["MnMississippiLowerWoodsC", "MnMississippiUpstreamWoodsC"] {
        let mut kinds = BTreeMap::new();
        let mut bodies = BTreeSet::new();
        for (name, map) in &pack.data.maps {
            if !name.starts_with(prefix) {
                continue;
            }
            let [ox, oy] = origin(name);
            for y in 0..map.attributes.height * 2 {
                for x in 0..map.attributes.width * 2 {
                    kinds.insert(
                        [x + ox, y + oy],
                        crate::authored_town_geometry::kind(pack, map, [x, y])?,
                    );
                }
            }
            bodies.extend(map.objects.iter().map(|o| [o.x + ox, o.y + oy]));
        }
        let mut reached = kinds
            .iter()
            .filter_map(|(&point, &kind)| {
                (kind == "FLOOR"
                    && !bodies.contains(&point)
                    && neighbors(point).any(|q| kinds.get(&q) == Some(&"WATER")))
                .then_some(point)
            })
            .collect::<BTreeSet<_>>();
        let mut queue = reached.iter().copied().collect::<VecDeque<_>>();
        while let Some(point) = queue.pop_front() {
            for next in neighbors(point) {
                if !bodies.contains(&next)
                    && matches!(kinds.get(&next), Some(&"FLOOR" | &"TALL_GRASS"))
                    && reached.insert(next)
                {
                    queue.push_back(next);
                }
            }
        }
        for (name, [x, y]) in changed.iter().filter(|(name, _)| name.starts_with(prefix)) {
            let [ox, oy] = origin(name);
            for dy in 0..2 {
                for dx in 0..2 {
                    ensure!(
                        reached.contains(&[ox + x * 2 + dx, oy + y * 2 + dy]),
                        "bank recess is unreachable from an existing dry Surf shore: {name} at [{x}, {y}]; install the completed river layouts before applying this refinement"
                    );
                }
            }
        }
    }
    Ok(())
}
