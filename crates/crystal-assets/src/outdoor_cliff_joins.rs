//! Explicit repair of hop ledges spliced into Minnesota's tall rock faces.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::outdoor_cliff_corners::{self, CornerBoulder};
use crate::outdoor_trainer_sightlines::{self, TrainerTurn};

#[derive(Deserialize)]
struct Plan {
    modules: BTreeMap<String, String>,
    changes: Vec<Change>,
    #[serde(default)]
    boulder_moves: Vec<BoulderMove>,
    #[serde(default)]
    corner_boulders: Vec<CornerBoulder>,
    #[serde(default)]
    trainer_turns: Vec<TrainerTurn>,
}

#[derive(Deserialize)]
struct BoulderMove {
    map: String,
    tileset: String,
    from: [usize; 2],
    destination_before: String,
}

#[derive(Deserialize)]
struct Change {
    map: String,
    tileset: String,
    metatile: [usize; 2],
    before: String,
    after: String,
}

/// Repairs explicitly authored Minnesota cliff and low-bank joins.
///
/// Certifies complete map-module preimages before closing tall-cliff hops,
/// restoring Brainerd's vertical side wall, and lowering the two isolated banks.
/// Moves three North Shore descents to actual right corners using native36.
/// Moves the listed rim boulders one metatile north onto flat ground, restoring
/// the native front cliff below them without changing the boulder graphics.
/// A separate corner-only pass restores the native side/front turns and moves
/// rocks inward, omitting only the two reviewed narrow Iron Range shelf rocks.
/// A trainer-only pass changes guarded standing directions only when the full
/// new sight lane is walkable and unoccupied. All other actor fields remain exact.
/// Retains map attributes, events, scripts, doors, encounters, art and audio,
/// including optional Cut gates.
/// Updates each existing encoded terrain mirror and derives a fresh identity.
/// Borrows the parent and writes no files.
///
/// # Errors
/// Rejects invalid parents, stale or incomplete module guards, repeated cells,
/// non-Minnesota or indoor maps, changed source collisions, invalid coordinates,
/// shared terrain payloads, or failed final content verification.
pub fn revise_minnesota_outdoor_cliff_joins(
    base: &CompiledGamePack,
    plan_json: &str,
) -> Result<CompiledGamePack> {
    crate::validate_compact_superior_geography(base)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let plan: Plan = serde_json::from_str(plan_json).context("outdoor cliff join plan")?;
    // A later boulder-only pass preserves the installed cliff revision. Give
    // that additive stage its own provenance instead of repeating a manifest.
    let manifest = if !plan.trainer_turns.is_empty() {
        ensure!(
            plan.changes.is_empty()
                && plan.boulder_moves.is_empty()
                && plan.corner_boulders.is_empty(),
            "trainer sightlines must be a separate bounded revision"
        );
        "minnesota-outdoor-trainer-sightlines-v1"
    } else if !plan.corner_boulders.is_empty() {
        ensure!(
            plan.changes.is_empty() && plan.boulder_moves.is_empty(),
            "corner repairs must be a separate bounded revision"
        );
        "minnesota-outdoor-cliff-corners-v1"
    } else if plan.changes.is_empty() && !plan.boulder_moves.is_empty() {
        "minnesota-outdoor-boulder-setbacks-v1"
    } else {
        "minnesota-outdoor-cliff-joins-v1"
    };
    ensure!(
        !base.report.manifests.iter().any(|id| id == manifest),
        "outdoor terrain revision is already installed: {manifest}"
    );
    let names: BTreeSet<_> = plan
        .changes
        .iter()
        .map(|change| &change.map)
        .chain(plan.boulder_moves.iter().map(|movement| &movement.map))
        .chain(plan.corner_boulders.iter().map(|repair| &repair.map))
        .chain(plan.trainer_turns.iter().map(|turn| &turn.map))
        .collect();
    ensure!(
        !names.is_empty() && names.iter().copied().eq(plan.modules.keys()),
        "every changed map needs exactly one complete preimage guard"
    );
    for (name, expected) in &plan.modules {
        let map = base.data.maps.get(name).context("cliff exterior")?;
        ensure!(
            name.starts_with("Mn")
                && matches!(
                    map.attributes.environment.as_deref(),
                    Some("TOWN" | "ROUTE")
                ),
            "cliff join patch is restricted to Minnesota outdoors"
        );
        ensure!(
            format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(&serde_json::to_value(map)?)?)
            ) == *expected,
            "cliff map preimage differs: {name}"
        );
        let label = map
            .attributes
            .blocks_label
            .as_ref()
            .context("cliff terrain label")?;
        ensure!(
            base.data
                .maps
                .values()
                .filter(|map| map.attributes.blocks_label.as_ref() == Some(label))
                .count()
                == 1,
            "cliff terrain payload is shared"
        );
    }
    let mut result = base.clone();
    let mut cells = BTreeSet::new();
    for change in &plan.changes {
        let hop = change.after == "72" && matches!(change.before.as_str(), "56" | "57");
        let low_bank = matches!(
            (
                change.map.as_str(),
                change.tileset.as_str(),
                change.metatile,
                change.before.as_str(),
                change.after.as_str()
            ),
            ("MnR19C00R00", "johto", [16, 16], "72", "56")
                | ("MnR19C00R00", "johto", [18, 16], "72", "61")
                | ("MnR03C00R00", "mn_headwaters", [7, 10], "0a", "57")
        );
        let side_wall = change.map == "MnBrainerd"
            && change.tileset == "johto"
            && change.metatile == [12, 19]
            && change.before == "4d"
            && change.after == "69";
        let continuity = change.map == "MnBoundarySafariC01R00"
            && change.tileset == "mn_boundary_safari"
            && change.before == "02"
            && matches!(
                (change.metatile, change.after.as_str()),
                ([15, 10], "6f") | ([16, 10], "72")
            );
        let corner = change.tileset == "mn_north_shore"
            && matches!(
                (
                    change.map.as_str(),
                    change.metatile,
                    change.before.as_str(),
                    change.after.as_str()
                ),
                ("MnR11C00R01", [12, 0], "82", "69")
                    | ("MnR11C00R01", [12, 2], "6d", "83")
                    | ("MnR11C00R01", [18, 20], "01", "69")
                    | ("MnR11C00R01", [18, 21], "81", "83")
                    | ("MnR11C00R03", [19, 13], "81", "69")
                    | ("MnR11C00R03", [19, 14], "6d", "83")
            );
        ensure!(
            (hop || corner || low_bank || side_wall || continuity)
                && cells.insert((&change.map, change.metatile)),
            "cliff patch requires unique bounded source cells"
        );
        let map = result
            .data
            .maps
            .get_mut(&change.map)
            .context("cliff exterior")?;
        ensure!(
            map.attributes.tileset_name == change.tileset,
            "cliff tileset differs"
        );
        let collision = &base
            .data
            .tilesets
            .get(&change.tileset)
            .context("cliff art")?
            .collision;
        if hop {
            ensure!(
                collision.get(&change.before).is_some_and(|kinds| kinds
                    .iter()
                    .map(String::as_str)
                    .eq(["HOP_DOWN", "HOP_DOWN", "WALL", "WALL"]))
                    && collision.get("72").is_some_and(|kinds| kinds
                        .iter()
                        .map(String::as_str)
                        .eq(["FLOOR", "FLOOR", "WALL", "WALL"])),
                "cliff patch collision source differs"
            );
        } else if corner {
            ensure!(
                collision.get("83").is_some_and(|kinds| kinds
                    .iter()
                    .map(String::as_str)
                    .eq(["FLOOR", "WALL", "FLOOR", "WALL"])),
                "native right-corner36 art is not installed"
            );
        } else if continuity {
            let expected = if change.after == "6f" {
                ["FLOOR", "FLOOR", "FLOOR", "WALL"]
            } else {
                ["FLOOR", "FLOOR", "WALL", "WALL"]
            };
            ensure!(
                collision
                    .get(&change.after)
                    .is_some_and(|kinds| kinds.iter().map(String::as_str).eq(expected)),
                "Boundary Waters retaining wall join collision differs"
            );
        } else if side_wall {
            ensure!(
                collision.get("69").is_some_and(|kinds| kinds
                    .iter()
                    .map(String::as_str)
                    .eq(["FLOOR", "WALL", "FLOOR", "WALL",])),
                "Brainerd's right side wall must remain continuous"
            );
        } else {
            ensure!(
                collision.get(&change.after).is_some_and(|kinds| {
                    if change.after == "61" {
                        kinds.iter().all(|kind| kind == "HEADBUTT_TREE")
                    } else {
                        kinds
                            .iter()
                            .map(String::as_str)
                            .eq(["HOP_DOWN", "HOP_DOWN", "WALL", "WALL"])
                    }
                }),
                "low-bank repair must use a native ledge or closed pine end"
            );
        }
        let [x, y] = change.metatile;
        ensure!(
            x < usize::from(map.attributes.width) && y < usize::from(map.attributes.height),
            "cliff cell outside map"
        );
        let cell = map
            .blocks
            .get_mut(y * usize::from(map.attributes.width) + x)
            .context("cliff cell")?;
        ensure!(
            *cell == u16::from_str_radix(&change.before, 16)?,
            "cliff cell preimage differs"
        );
        *cell = u16::from_str_radix(&change.after, 16)?;
    }
    for movement in &plan.boulder_moves {
        let map = result
            .data
            .maps
            .get_mut(&movement.map)
            .context("boulder exterior")?;
        let [x, y] = movement.from;
        let width = usize::from(map.attributes.width);
        ensure!(
            map.attributes.tileset_name == movement.tileset
                && x < width
                && y > 0
                && y < usize::from(map.attributes.height)
                && cells.insert((&movement.map, [x, y]))
                && cells.insert((&movement.map, [x, y - 1])),
            "boulder relocation requires two unique in-bounds cells"
        );
        let collision = &base.data.tilesets[&movement.tileset].collision;
        ensure!(
            collision
                .get("0a")
                .is_some_and(|kinds| kinds.iter().all(|kind| kind == "WALL"))
                && collision
                    .get(&movement.destination_before)
                    .is_some_and(|kinds| kinds
                        .iter()
                        .all(|kind| matches!(kind.as_str(), "FLOOR" | "TALL_GRASS")))
                && collision.get("72").is_some_and(|kinds| kinds
                    .iter()
                    .map(String::as_str)
                    .eq(["FLOOR", "FLOOR", "WALL", "WALL"])),
            "boulders must move onto flat ground above a native cliff face"
        );
        ensure!(
            map.blocks[y * width + x] == 0x0a
                && map.blocks[(y - 1) * width + x]
                    == u16::from_str_radix(&movement.destination_before, 16)?
                && !map.objects.iter().any(|object| {
                    usize::from(object.x) / 2 == x && usize::from(object.y) / 2 == y - 1
                }),
            "boulder relocation source changed or destination is occupied"
        );
        map.blocks[y * width + x] = 0x72;
        map.blocks[(y - 1) * width + x] = 0x0a;
    }
    for repair in &plan.corner_boulders {
        ensure!(
            cells.insert((&repair.map, repair.metatile))
                && repair
                    .destination
                    .is_none_or(|at| cells.insert((&repair.map, at))),
            "corner repairs require unique source and destination cells"
        );
        outdoor_cliff_corners::apply(&mut result, repair)?;
    }
    let mut turned = BTreeSet::new();
    for turn in &plan.trainer_turns {
        ensure!(
            turned.insert((&turn.map, &turn.script)),
            "trainer sightlines require unique native objects"
        );
        outdoor_trainer_sightlines::apply(&mut result, turn)?;
    }
    for name in names {
        let map = result
            .data
            .maps
            .get(name)
            .context("repaired cliff exterior")?;
        let label = map
            .attributes
            .blocks_label
            .as_ref()
            .context("cliff terrain label")?;
        let bytes = map
            .blocks
            .iter()
            .copied()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        result
            .data
            .map_blocks
            .insert(label.clone(), crate::encode_base64_bytes(&bytes));
    }
    result.report.manifests.push(manifest.into());
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::validate_compact_superior_geography(&result)?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    Ok(result)
}
