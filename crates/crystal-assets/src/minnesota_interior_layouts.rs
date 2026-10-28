//! Explicit native room compositions, with existing programs and doors retained.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde::Deserialize;
use serde_json::json;

use crate::authored_town_geometry::kind;
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Layouts {
    schema_version: u8,
    manifest: String,
    rooms: Vec<RoomLayout>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RoomLayout {
    map: String,
    intent: String,
    before_tileset: String,
    before_rows: Vec<String>,
    tileset: String,
    border_block: u8,
    rows: Vec<String>,
    arrival: [u16; 2],
    actor_approaches: Vec<Approach>,
    #[serde(default)]
    exhibits: Vec<Exhibit>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Approach {
    script: String,
    tile: [u16; 2],
    counter_tile: Option<[u16; 2]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Exhibit {
    id: String,
    tile: [u16; 2],
    approach: [u16; 2],
    lines: Vec<String>,
}

/// Replaces explicitly certified non-gym room terrain with native source art.
/// The contract must match every old row and the old tileset. Dimensions, actors,
/// warps and all original scripts stay intact. New read-only furniture details
/// append to the existing native background-event and script tables. Residents
/// may be approached directly or through one explicitly declared native counter.
/// No assets, species, rewards, main-story flags, recovery data or audio are
/// replaced.
///
/// # Errors
/// Returns an error for stale/duplicate contracts, unknown art, inaccessible
/// actors or exits, invalid counters, false stairs, open room boundaries,
/// occupied labels/events, furniture on walkable/service tiles, malformed text
/// or failed native verification. Inputs remain unchanged.
pub fn build_minnesota_interior_layout_modpack(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let plan: Layouts = serde_json::from_str(source).context("explicit interior layouts")?;
    ensure!(
        plan.schema_version == 1 && !plan.rooms.is_empty(),
        "interior layout schema/coverage"
    );
    ensure!(
        plan.manifest.starts_with("minnesota-interior-layout-")
            && !base.report.manifests.contains(&plan.manifest),
        "invalid/installed layout manifest"
    );
    let mut pack = base.clone();
    let mut rooms = BTreeSet::new();
    let mut labels = base
        .data
        .map_scripts
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let names = base
        .data
        .runtime_map_metadata
        .values()
        .map(|m| (m.constant.clone(), m.name.clone()))
        .collect::<BTreeMap<_, _>>();
    for room in &plan.rooms {
        ensure!(
            rooms.insert(&room.map) && !room.intent.trim().is_empty(),
            "duplicate/unnamed room design"
        );
        let original = base
            .data
            .maps
            .get(&room.map)
            .with_context(|| format!("missing interior {}", room.map))?;
        ensure!(
            room.map.starts_with("Mn")
                && !room.map.contains("Gym")
                && matches!(
                    original.attributes.environment.as_deref(),
                    Some("INDOOR" | "GATE")
                ),
            "layout is not an ordinary Minnesota interior: {}",
            room.map
        );
        let mut map = original.clone();
        let width = map.attributes.width;
        let height = map.attributes.height;
        ensure!(
            room.before_tileset == original.attributes.tileset_name
                && parse_rows(&room.before_rows, width, height)? == original.blocks,
            "stale room terrain preimage: {}",
            room.map
        );
        map.blocks = parse_rows(&room.rows, width, height)?;
        map.attributes.tileset_name = room.tileset.clone();
        map.attributes.border_block = room.border_block;
        let tileset = base
            .data
            .tilesets
            .get(&room.tileset)
            .context("installed native room tileset")?;
        ensure!(
            map.blocks
                .iter()
                .chain(std::iter::once(&u16::from(room.border_block)))
                .all(|b| tileset.collision.contains_key(&format!("{b:02x}"))),
            "unknown native room block"
        );
        validate(base, original, &map, room)
            .with_context(|| format!("room layout {}", room.map))?;
        let mut additions = authoring::blank(
            base,
            &room.map,
            &room.map,
            original
                .attributes
                .map_constant
                .as_deref()
                .context("room constant")?,
        )?;
        for exhibit in &room.exhibits {
            let text = format!("{}Text", exhibit.id);
            ensure!(
                labels.insert(exhibit.id.clone()) && labels.insert(text.clone()),
                "occupied furniture label"
            );
            ensure!(
                !exhibit.lines.is_empty() && exhibit.lines.iter().all(|s| !s.trim().is_empty()),
                "empty furniture text"
            );
            authoring::text(
                &mut additions,
                &text,
                &exhibit.lines.iter().map(String::as_str).collect::<Vec<_>>(),
            );
            additions.scripts.insert(
                exhibit.id.clone(),
                json!([authoring::command("jumptext", &[&text])]),
            );
            additions.events.bg_events.push(BackgroundEvent {
                x: exhibit.tile[0],
                y: exhibit.tile[1],
                event_type: "BGEVENT_READ".into(),
                script: exhibit.id.clone(),
            });
        }
        crate::authored_city_expansion_events::finish(&pack, &mut additions, &names)?;
        crate::authored_city_expansion::merge_new_content(&mut map, additions)?;
        authoring::refresh_event_section(&mut map);
        crate::authored_town_publication::replace(&mut pack, map)?;
    }
    pack.report.manifests.push(plan.manifest);
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn parse_rows(rows: &[String], width: u16, height: u16) -> Result<Vec<u16>> {
    ensure!(rows.len() == usize::from(height), "room row count differs");
    let mut blocks = Vec::new();
    for row in rows {
        let values = row
            .split_whitespace()
            .map(|value| u16::from_str_radix(value, 16))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(values.len() == usize::from(width), "room row width differs");
        blocks.extend(values);
    }
    Ok(blocks)
}

fn neighbors([x, y]: [u16; 2]) -> impl Iterator<Item = [u16; 2]> {
    [
        x.checked_sub(1).map(|x| [x, y]),
        x.checked_add(1).map(|x| [x, y]),
        y.checked_sub(1).map(|y| [x, y]),
        y.checked_add(1).map(|y| [x, y]),
    ]
    .into_iter()
    .flatten()
}

fn is_exit(kind: &str) -> bool {
    matches!(
        kind,
        "DOOR"
            | "LADDER"
            | "STAIRCASE"
            | "WARP_CARPET_DOWN"
            | "WARP_CARPET_LEFT"
            | "WARP_CARPET_RIGHT"
    )
}

fn validate(
    base: &CompiledGamePack,
    original: &MapModule,
    map: &MapModule,
    room: &RoomLayout,
) -> Result<()> {
    let width = map.attributes.width * 2;
    let height = map.attributes.height * 2;
    let bodies = map
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    let warps = map
        .events
        .warps
        .iter()
        .map(|w| [w.x, w.y])
        .collect::<BTreeSet<_>>();
    ensure!(
        kind(base, map, room.arrival)? == "FLOOR"
            && !bodies.contains(&room.arrival)
            && warps.contains(&room.arrival),
        "layout needs its real passive arrival"
    );
    for warp in &map.events.warps {
        let tile = [warp.x, warp.y];
        let before = kind(base, original, tile)?;
        let after = kind(base, map, tile)?;
        ensure!(
            (before == "FLOOR" && after == "FLOOR") || (is_exit(before) && is_exit(after)),
            "warp {} loses native arrival/activation at {tile:?}",
            warp.index
        );
    }
    let mut seen = BTreeSet::from([room.arrival]);
    let mut queue = VecDeque::from([room.arrival]);
    while let Some(tile) = queue.pop_front() {
        for q in neighbors(tile) {
            if q[0] >= width || q[1] >= height || bodies.contains(&q) || seen.contains(&q) {
                continue;
            }
            let k = kind(base, map, q)?;
            if k != "FLOOR" && !is_exit(k) {
                continue;
            }
            seen.insert(q);
            if !is_exit(k) {
                queue.push_back(q);
            }
        }
    }
    for y in 0..height {
        for x in 0..width {
            let tile = [x, y];
            let k = kind(base, map, tile)?;
            ensure!(
                !is_exit(k) || warps.contains(&tile),
                "unpaired native stair/exit at {tile:?}"
            );
            if x == 0 || y == 0 || x == width - 1 || y == height - 1 {
                ensure!(
                    k != "FLOOR" && (!is_exit(k) || warps.contains(&tile)),
                    "open room boundary at {tile:?}"
                );
            }
        }
    }
    for warp in &map.events.warps {
        ensure!(
            seen.contains(&[warp.x, warp.y]),
            "unreachable warp {}",
            warp.index
        );
    }
    let mut approaches = BTreeSet::new();
    let mut counters = BTreeSet::new();
    for approach in &room.actor_approaches {
        ensure!(
            approaches.insert(&approach.script),
            "duplicate actor approach"
        );
        let actor = map
            .objects
            .iter()
            .find(|o| o.script == approach.script)
            .context("unknown layout resident")?;
        ensure!(
            kind(base, map, [actor.x, actor.y])? == "FLOOR",
            "resident is inside furniture: {}",
            actor.script
        );
        let distance = actor.x.abs_diff(approach.tile[0]) + actor.y.abs_diff(approach.tile[1]);
        let reaches_actor = if let Some(counter) = approach.counter_tile {
            ensure!(
                distance == 2
                    && (actor.x == approach.tile[0] || actor.y == approach.tile[1])
                    && counter
                        == [
                            actor.x.min(approach.tile[0]) + actor.x.abs_diff(approach.tile[0]) / 2,
                            actor.y.min(approach.tile[1]) + actor.y.abs_diff(approach.tile[1]) / 2,
                        ]
                    && kind(base, map, counter)? == "COUNTER"
                    && counters.insert(counter)
                    && !bodies.contains(&counter)
                    && !warps.contains(&counter)
                    && !map.events.bg_events.iter().any(|e| [e.x, e.y] == counter),
                "invalid/occupied native counter approach: {}",
                actor.script
            );
            true
        } else {
            distance == 1
        };
        ensure!(
            seen.contains(&approach.tile)
                && kind(base, map, approach.tile)? == "FLOOR"
                && reaches_actor,
            "unreachable/invalid resident approach: {}",
            actor.script
        );
    }
    ensure!(
        approaches
            == map
                .objects
                .iter()
                .map(|o| &o.script)
                .collect::<BTreeSet<_>>(),
        "every existing resident needs an approach"
    );
    for event in &map.events.coord_events {
        ensure!(
            seen.contains(&[event.x, event.y]),
            "blocked original coordinate scene"
        );
    }
    for event in &map.events.bg_events {
        ensure!(
            neighbors([event.x, event.y]).any(|p| seen.contains(&p)),
            "blocked original background event"
        );
    }
    let mut details = map
        .events
        .bg_events
        .iter()
        .map(|e| [e.x, e.y])
        .collect::<BTreeSet<_>>();
    for exhibit in &room.exhibits {
        ensure!(
            details.insert(exhibit.tile)
                && !bodies.contains(&exhibit.tile)
                && !warps.contains(&exhibit.tile)
                && !counters.contains(&exhibit.tile),
            "furniture interaction overlaps event/actor"
        );
        ensure!(
            matches!(
                kind(base, map, exhibit.tile)?,
                "WALL" | "BOOKSHELF" | "COUNTER" | "MART_SHELF"
            ),
            "furniture detail must target solid furniture without replacing a service: {}",
            exhibit.id
        );
        ensure!(
            seen.contains(&exhibit.approach)
                && kind(base, map, exhibit.approach)? == "FLOOR"
                && exhibit.tile[0].abs_diff(exhibit.approach[0])
                    + exhibit.tile[1].abs_diff(exhibit.approach[1])
                    == 1,
            "unreachable furniture detail {}",
            exhibit.id
        );
    }
    Ok(())
}
