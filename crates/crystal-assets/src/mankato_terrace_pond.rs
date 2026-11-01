//! Tighten Mankato's services and fit a contained, fishable pond garden.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::authored_town_geometry;
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const MAP: &str = "MnMankato";
const PARENT: &str = "0513ba901b48e3f85c42a8ecad8b38711c31ca743484b422cfd054256618cf16";
const SOURCE: &str = "9d2b4bbc5aa30521eb1724a4a6e66d6f39ee86c716a547c48e3db00f9b2ec9ea";
const MARKER: &str = "minnesota-mankato-terrace-pond-v1";

#[derive(Deserialize)]
struct Proposal {
    schema_version: u8,
    map: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    native_module_sha256: String,
    source_pixels_modified: bool,
    before_rows: Vec<String>,
    rows: Vec<String>,
    building_moves: Vec<BuildingMove>,
    warp_moves: Vec<WarpMove>,
    sign_moves: Vec<SignMove>,
    public_door_returns: BTreeMap<String, [u16; 2]>,
    public_promenades: Vec<Promenade>,
    fishing_banks: Vec<FishingBank>,
    expected_changed_metatiles: usize,
    expected_water_quadrants: usize,
    expected_floor_quadrants: usize,
}

#[derive(Deserialize)]
struct BuildingMove {
    source_rect_metatiles: [u16; 4],
    destination_top_left_metatiles: [u16; 2],
}

#[derive(Deserialize)]
struct WarpMove {
    index: u16,
    from: [u16; 2],
    to: [u16; 2],
}

#[derive(Deserialize)]
struct SignMove {
    id: String,
    from: [u16; 2],
    to: [u16; 2],
}

#[derive(Deserialize)]
struct Promenade {
    id: String,
    rectangle_core: [u16; 4],
}

#[derive(Deserialize)]
struct FishingBank {
    stand: [u16; 2],
    water: [u16; 2],
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn rows(source: &[String]) -> Result<Vec<u16>> {
    ensure!(source.len() == 18, "eighteen Mankato rows required");
    let mut blocks = Vec::with_capacity(432);
    for row in source {
        let row = row
            .split_whitespace()
            .map(|word| u16::from_str_radix(word, 16))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(row.len() == 24, "twenty-four Mankato columns required");
        blocks.extend(row);
    }
    Ok(blocks)
}

fn geometry(pack: &CompiledGamePack, town: &MapModule, plan: &Proposal) -> Result<()> {
    ensure!(
        town.blocks[15 * 24 + 16..15 * 24 + 21] == [0x40, 0x41, 0x41, 0x41, 0x42],
        "Mankato south rail must use fitted native corners and straight spans"
    );
    let reached = authored_town_geometry::reachable(pack, town, [46, 18], false)?;
    let bodies = town
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    let gate_pocket = BTreeSet::from([[24, 0], [25, 0], [24, 1], [25, 1]]);
    let mut floor = 0;
    let mut water = 0;
    for y in 0..36 {
        for x in 0..48 {
            let at = [x, y];
            let kind = authored_town_geometry::kind(pack, town, at)?;
            ensure!(
                kind != "CUT_TREE" && kind != "ICE",
                "incidental Mankato gate"
            );
            if kind == "FLOOR" {
                floor += 1;
                ensure!(
                    reached.contains(&at) || bodies.contains(&at) || gate_pocket.contains(&at),
                    "Mankato floor disconnected: {at:?}"
                );
            }
            if kind == "WATER" {
                water += 1;
                ensure!(
                    (32..=41).contains(&x) && (24..=29).contains(&y),
                    "Mankato pond escaped its contained garden"
                );
            }
        }
    }
    ensure!(
        floor == plan.expected_floor_quadrants && water == plan.expected_water_quadrants,
        "reviewed Mankato terrain counts differ"
    );
    for object in &town.objects {
        let at = [object.x, object.y];
        ensure!(
            authored_town_geometry::kind(pack, town, at)? == "FLOOR",
            "Mankato actor lost its standing tile"
        );
    }
    for at in plan.public_door_returns.values() {
        ensure!(
            reached.contains(at),
            "Mankato service frontage disconnected"
        );
    }
    for spawn in pack
        .data
        .runtime_spawn_points
        .values()
        .filter(|s| s.map_name == MAP)
    {
        let at = [u16::try_from(spawn.tile_x)?, u16::try_from(spawn.tile_y)?];
        ensure!(
            reached.contains(&at),
            "Mankato native arrival spawn disconnected"
        );
    }
    for door in town.events.warps.iter().filter(|warp| warp.index % 2 == 1) {
        ensure!(
            authored_town_geometry::kind(pack, town, [door.x, door.y])? == "DOOR",
            "Mankato facade lost its native door quadrant"
        );
        let outside = town
            .events
            .warps
            .iter()
            .find(|w| w.index == door.index + 1)
            .context("paired Mankato exterior return")?;
        ensure!(
            outside.target_map_constant == "MN_MANKATO"
                && [outside.x, outside.y] == [door.x, door.y + 1],
            "Mankato exterior pairing changed"
        );
    }
    for sign in &town.events.bg_events {
        ensure!(
            authored_town_geometry::kind(pack, town, [sign.x, sign.y])? == "WALL"
                && [
                    [sign.x - 1, sign.y],
                    [sign.x + 1, sign.y],
                    [sign.x, sign.y - 1],
                    [sign.x, sign.y + 1]
                ]
                .iter()
                .any(|p| reached.contains(p)),
            "Mankato sign lost its solid post or reading approach"
        );
    }
    for promenade in &plan.public_promenades {
        let [x, y, width, height] = promenade.rectangle_core;
        for yy in y..y + height {
            for xx in x..x + width {
                ensure!(
                    reached.contains(&[xx, yy]),
                    "Mankato promenade blocked: {}",
                    promenade.id
                );
            }
        }
    }
    for bank in &plan.fishing_banks {
        ensure!(
            reached.contains(&bank.stand)
                && authored_town_geometry::kind(pack, town, bank.water)? == "WATER"
                && bank.stand[0].abs_diff(bank.water[0]) + bank.stand[1].abs_diff(bank.water[1])
                    == 1,
            "Mankato dry casting bank disconnected"
        );
    }
    Ok(())
}

/// Brings Mankato's clinic and Mart onto its upper court and pond-garden loop.
///
/// Preserves all actors, programs, room pairings, the existing ledge/return
/// rectangle, road mouths, attributes, fishing tables, art and audio. Adds a
/// contained pond and two public dry casting banks without granting a rod,
/// badge or field move. The result is unselected; saved terrain is not migrated.
///
/// # Errors
/// Rejects source/parent/preimage drift, incomplete facades, disconnected public
/// paths, unsafe anchors, moved protected terrain or unrelated content changes.
/// Borrowed inputs are unchanged on failure.
pub fn revise_mankato_terrace_pond(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        hash(source.as_bytes()) == SOURCE,
        "reviewed Mankato source differs"
    );
    let plan: Proposal = serde_json::from_str(source)?;
    ensure!(
        plan.schema_version == 1
            && plan.map == MAP
            && plan.tileset == "johto"
            && plan.dimensions_metatiles == [24, 18]
            && !plan.source_pixels_modified
            && base.identity.content_hash == PARENT
            && !base.report.manifests.iter().any(|m| m == MARKER),
        "complete current Mankato parent required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let original = base.data.maps.get(MAP).context("installed Mankato")?;
    ensure!(
        hash(&serde_json::to_vec(&serde_json::to_value(original)?)?) == plan.native_module_sha256
            && original.blocks == rows(&plan.before_rows)?
            && original.attributes.tileset_name == plan.tileset
            && [original.attributes.width, original.attributes.height] == [24, 18],
        "complete Mankato native preimage differs"
    );
    let mut town = original.clone();
    town.blocks = rows(&plan.rows)?;
    ensure!(
        original
            .blocks
            .iter()
            .zip(&town.blocks)
            .filter(|(a, b)| a != b)
            .count()
            == plan.expected_changed_metatiles,
        "reviewed Mankato terrain scope differs"
    );
    for y in 0..18 {
        for x in 0..24 {
            if x == 0
                || x == 23
                || y < 2
                || y == 17
                || ((4..13).contains(&x) && (8..12).contains(&y))
            {
                ensure!(
                    original.blocks[y * 24 + x] == town.blocks[y * 24 + x],
                    "Mankato road/gate/ledge/return preimage changed"
                );
            }
        }
    }
    for building in &plan.building_moves {
        let [x, y, width, height] = building.source_rect_metatiles;
        let [tx, ty] = building.destination_top_left_metatiles;
        for dy in 0..height {
            let from = usize::from((y + dy) * 24 + x);
            let to = usize::from((ty + dy) * 24 + tx);
            ensure!(
                original.blocks[from..from + usize::from(width)]
                    == town.blocks[to..to + usize::from(width)],
                "incomplete Mankato facade"
            );
        }
    }
    for change in &plan.warp_moves {
        let warp = town
            .events
            .warps
            .iter_mut()
            .find(|w| w.index == change.index)
            .context("reviewed Mankato warp")?;
        ensure!([warp.x, warp.y] == change.from, "stale Mankato warp");
        [warp.x, warp.y] = change.to;
    }
    for change in &plan.sign_moves {
        let sign = town
            .events
            .bg_events
            .iter_mut()
            .find(|s| s.script == change.id)
            .context("reviewed Mankato sign")?;
        ensure!([sign.x, sign.y] == change.from, "stale Mankato sign");
        [sign.x, sign.y] = change.to;
    }
    authoring::refresh_event_section(&mut town);
    geometry(base, &town, &plan)?;
    let mut child = base.clone();
    crate::authored_town_publication::replace(&mut child, town)?;
    child.report.manifests.push(MARKER.into());
    child.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        child.format_version,
        &child.data,
        &child.audio_manifest,
        &child.runtime_files,
        &child.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&child)?;
    crate::validate_compact_superior_geography(&child)?;
    let mut residual = child.clone();
    crate::authored_town_publication::replace(&mut residual, original.clone())?;
    residual.identity = base.identity.clone();
    residual.report = base.report.clone();
    ensure!(
        residual == *base,
        "unrelated Mankato revision content changed"
    );
    Ok(child)
}
