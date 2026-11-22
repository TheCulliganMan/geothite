//! Reconcile idle checkpoints from the published Lakeside archive with Fair Patrols.
use anyhow::{Context, Result, ensure};
use crystal_assets::GameDataSet;
use crystal_core::save::SaveModpackIdentity;
use crystal_core::state::{GameState, OverworldMemory};
use crystal_core::world::collision::{
    PlayerTraversalState, is_permission_passable, permissions, sample_collision,
};
use crystal_core::world::map::{Direction, OverworldMapData, TilePosition};
use crystal_core::world::session::OverworldSession;
use crystal_core::world::{bridge, collision};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::{Compatibility, CompatibleSaveSource, parse_regional_compatibility};

const MARKER: &str = "northstar-fair-release-v1";
const PRESENTATION_MARKER: &str = "+northstar-fair-presentation-v1";
const FAIR_HASHES: [&str; 2] = [
    "18003b91597173f1d98937a0f6cc2fa82cfd2c5b3612c9803cd5f615acad2a73",
    "9fbfa371972f6b3cdbd9d0ca9689aa5a66ee43d1d17e7650982da88d4e57f58e",
];
const FAIR_CONTENT: [&str; 2] = [
    "d81cdb3d46bd72ffd22a2793c66f1d2f7ff2b8500674bca1ec68e1ee230a3078",
    "415294715e2f50c60b3696966441a98a7b869fde34f6a3acbf94b4ab64bb8b4a",
];
const SUFFIX: &str = "+realtime-clock+player-customization";
const LAKESIDE_RAW_HASH: &str = "e56ded3d55b8364e8756a4a02eb530fb74f52c03951c1c01d962441f7e2193d9";
const LAKESIDE_HOSTED_HASH: &str =
    "ad775429c71fc48aabbc8e1e2db40d3594a209dbc81e8b959fed1b73f5486426";
const DIRECTIONS: [Direction; 4] = [
    Direction::Down,
    Direction::Up,
    Direction::Left,
    Direction::Right,
];

fn on_traversal_layer(permission: u8, traversal: PlayerTraversalState) -> bool {
    traversal != PlayerTraversalState::Surf
        || collision::describe_collision(bridge::layer_permission(permission, traversal)).terrain
            == collision::Terrain::Water
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MapChange {
    blocks: BTreeSet<[u16; 2]>,
    prior_tiles: Vec<[u16; 3]>,
    actors: BTreeSet<u8>,
    triggers: BTreeSet<[u16; 2]>,
    removed_changes: BTreeSet<[u16; 3]>,
    replacements: Vec<[u16; 4]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Release {
    schema_version: u8,
    target_manifest: String,
    target_modpack_ids: [String; 2],
    sources: [CompatibleSaveSource; 2],
    inherited: serde_json::Value,
    layouts: BTreeMap<String, MapChange>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PresentationRelease {
    schema_version: u8,
    sources: [CompatibleSaveSource; 2],
    inherited: serde_json::Value,
}

pub(super) fn parse_presentation(bytes: &[u8], target: &str) -> Result<Compatibility> {
    let release: PresentationRelease = serde_json::from_slice(bytes)?;
    let [raw, hosted] = &release.sources;
    let target_base = target.strip_suffix(SUFFIX).unwrap_or(target);
    let parent = target_base
        .strip_suffix(PRESENTATION_MARKER)
        .context("invalid Fair presentation target")?;
    ensure!(
        release.schema_version == 10
            && parent == raw.modpack_id
            && hosted.modpack_id.strip_prefix(&raw.modpack_id) == Some(SUFFIX)
            && release.sources.iter().enumerate().all(|(i, source)| {
                source.modpack_hash == FAIR_HASHES[i] && source.content_hash == FAIR_CONTENT[i]
            }),
        "expected exact Fair presentation sources"
    );
    let inherited: Release = serde_json::from_value(release.inherited.clone())?;
    ensure!(
        inherited.target_modpack_ids == [raw.modpack_id.clone(), hosted.modpack_id.clone()],
        "mismatched inherited Fair save targets"
    );
    let fair_target = if target_base == target {
        &raw.modpack_id
    } else {
        &hosted.modpack_id
    };
    let mut policy = parse(&serde_json::to_vec(&release.inherited)?, fair_target)?;
    policy.unchanged_checkpoint_parents.extend(
        release
            .sources
            .iter()
            .map(|source| source.modpack_id.clone()),
    );
    policy.sources.splice(0..0, release.sources);
    Ok(policy)
}

pub(super) fn parse(bytes: &[u8], target: &str) -> Result<Compatibility> {
    let release: Release = serde_json::from_slice(bytes)?;
    let [raw, hosted] = &release.target_modpack_ids;
    ensure!(
        release.schema_version == 9
            && release.target_manifest == MARKER
            && raw.ends_with(&format!("+{MARKER}"))
            && raw.split('+').any(|s| s == "northstar-fair-patrols-v1")
            && hosted.strip_prefix(raw) == Some(SUFFIX)
            && (target == raw || target == hosted),
        "invalid Fair release target"
    );
    for id in &release.target_modpack_ids {
        SaveModpackIdentity::validate_id(id)?;
    }
    let [prior, live] = &release.sources;
    ensure!(
        prior.modpack_hash == LAKESIDE_RAW_HASH
            && prior.content_hash
                == "a9d128babd0de1a48bdd6caf35fcc505965002d596181c13a35f424dcac0f55a"
            && live.modpack_hash == LAKESIDE_HOSTED_HASH
            && live.content_hash
                == "70d8a66374dfeca5eed7a3a33fd6885b43aa9afaa0be300b60381ca7aacc99f4"
            && live.modpack_id.strip_prefix(&prior.modpack_id) == Some(SUFFIX),
        "expected exact public Lakeside pair"
    );
    let mut prior_policy = parse_regional_compatibility(
        &serde_json::to_vec(&release.inherited)?,
        &prior.modpack_id,
        8,
    )?;
    ensure!(
        release.layouts.contains_key("MnStateFairgrounds")
            && release.layouts.keys().all(|id| id.starts_with("Mn")),
        "invalid Fair layout set"
    );
    prior_policy
        .unchanged_checkpoint_parents
        .extend(release.sources.iter().map(|s| s.modpack_id.clone()));
    prior_policy.sources.splice(0..0, release.sources);
    prior_policy.fair_layouts = release.layouts;
    Ok(prior_policy)
}

pub(super) fn reconcile(
    state: &mut GameState,
    data: &GameDataSet,
    layouts: &BTreeMap<String, MapChange>,
    source: &CompatibleSaveSource,
) -> Result<()> {
    // The presentation release preserves Fair terrain and object positions.
    if FAIR_HASHES.contains(&source.modpack_hash.as_str()) {
        return Ok(());
    }
    for (map, change) in layouts {
        let room = data.map_module(map)?;
        if let Some(blocks) = state.map_block_overrides.get_mut(map) {
            for &[x, y, prior, next] in &change.replacements {
                if let Some(value) = blocks.get_mut(&(x, y))
                    && *value == prior
                {
                    *value = next;
                }
            }
            blocks.retain(|&(x, y), id| {
                let authored = room
                    .script_block_changes
                    .iter()
                    .any(|c| c.x / 2 == x && c.y / 2 == y && c.block_id == *id);
                authored
                    || (!change.blocks.contains(&[x, y])
                        && !change.removed_changes.contains(&[x, y, *id]))
            });
        }
        if let Some(memory) = state.map_object_overrides.get_mut(map) {
            for index in &change.actors {
                let actor = room
                    .objects
                    .get(
                        usize::from(*index)
                            .checked_sub(1)
                            .context("zero actor index")?,
                    )
                    .context("missing revised actor")?;
                if let Some(id) = &actor.object_identifier {
                    memory.objects.remove(id);
                }
            }
            // A fresh map entry recreates these transient sprite structs. Store
            // permanent show/hide receipts separately, exactly as normal entry does.
            let changed_landing = |at: TilePosition| {
                change.prior_tiles.iter().any(|&[x, y, _]| {
                    i32::from(x) == i32::from(at.x) && i32::from(y) == i32::from(at.y)
                })
            };
            memory.object_structs.structs.retain(|a| {
                !change.actors.contains(&a.map_object_index) && !changed_landing(a.live_tile)
            });
            memory.objects.retain(|_, a| {
                !change
                    .prior_tiles
                    .iter()
                    .any(|&[x, y, _]| x == a.x && y == a.y)
            });
        }
    }
    let OverworldMemory::Active {
        map_name,
        tile,
        mode,
        ..
    } = &state.overworld
    else {
        return Ok(());
    };
    let Some(change) = layouts.get(map_name) else {
        return Ok(());
    };
    let room = data.map_module(map_name)?;
    let native = OverworldMapData::from_attributes(map_name, &room.attributes, room.blocks.clone());
    let tiles = data.tileset_collision(&room.attributes.tileset_name)?;
    let old_terrain: BTreeMap<_, _> = change
        .prior_tiles
        .iter()
        .map(|&[x, y, t]| Ok(((i16::try_from(x)?, i16::try_from(y)?), u8::try_from(t)?)))
        .collect::<Result<_>>()?;
    let blocked_triggers: BTreeSet<_> = room
        .events
        .warps
        .iter()
        .map(|w| [w.x, w.y])
        .chain(room.events.coord_events.iter().map(|e| [e.x, e.y]))
        .collect();
    let mut session = OverworldSession::with_events_and_objects(
        native.clone(),
        room.events.clone(),
        room.objects.clone(),
        tiles.clone(),
        *tile,
    );
    data.apply_saved_overworld_overrides(&mut session, state)?;
    let occupied: BTreeSet<_> = session
        .occupied_tiles_checked()?
        .into_iter()
        .map(|a| (a.tile.x, a.tile.y))
        .collect();
    let mode = mode.traversal_state();
    let start = *tile;
    let original = [u16::try_from(tile.x)?, u16::try_from(tile.y)?];
    if !occupied.contains(&(tile.x, tile.y))
        && !change.triggers.contains(&original)
        && sample_collision(&session.map, &session.tileset, start).is_some_and(|s| {
            // Connected maps permit special border arrival tiles when restoring a save.
            // A changed interior quadrant must itself support the player's movement mode.
            on_traversal_layer(s.permission, mode)
                && (!old_terrain.contains_key(&(start.x, start.y))
                    || DIRECTIONS
                        .into_iter()
                        .any(|dir| is_permission_passable(s.permission, dir, mode)))
        })
        && data
            .overworld_session_for_traversal_with_state(
                map_name,
                *tile,
                state.frame_counter,
                mode,
                Some(state),
            )
            .is_ok()
    {
        return Ok(());
    }
    let old = |p: TilePosition| {
        old_terrain
            .get(&(p.x, p.y))
            .copied()
            .or_else(|| sample_collision(&native, &tiles, p).map(|sample| sample.permission))
    };
    let target_permission =
        |p| sample_collision(&session.map, &session.tileset, p).map(|sample| sample.permission);
    let clear_landing = |p: TilePosition| {
        !occupied.contains(&(p.x, p.y)) && !blocked_triggers.contains(&[p.x as u16, p.y as u16])
    };
    let landing = nearest_reachable_landing(start, mode, old, |p| {
        clear_landing(p)
            && target_permission(p).is_some_and(|permission| match mode {
                PlayerTraversalState::Walk => matches!(
                    permission,
                    permissions::FLOOR | permissions::TALL_GRASS | permissions::LONG_GRASS
                ),
                PlayerTraversalState::Surf => {
                    bridge::layer_permission(permission, mode) == permissions::WATER
                }
            })
    })
    .or_else(|| {
        // Both pockets were reachable with a field move. With the pocket gone,
        // the walker's old connected component has no surviving landing.
        if !matches!(
            source.modpack_hash.as_str(),
            LAKESIDE_RAW_HASH | LAKESIDE_HOSTED_HASH
        ) || mode != PlayerTraversalState::Walk
            || old_terrain.get(&(start.x, start.y)) != Some(&permissions::FLOOR)
            || target_permission(start) != Some(permissions::WALL)
        {
            return None;
        }
        let approach = match (map_name.as_str(), start.x, start.y) {
            ("MnR12Cave1F", 28..=29, 16..=17) => TilePosition::new(21, 16),
            ("MnR15C00R00", 49, 18) => TilePosition::new(47, 18),
            _ => return None,
        };
        [approach, approach.moved(Direction::Left)]
            .into_iter()
            .find(|&p| {
                clear_landing(p)
                    && old(p) == Some(permissions::FLOOR)
                    && target_permission(p) == Some(permissions::FLOOR)
            })
    })
    .with_context(|| format!("no reachable safe {mode:?} landing on {map_name}"))?;
    if let OverworldMemory::Active { tile, .. } = &mut state.overworld {
        *tile = landing;
    }
    Ok(())
}

fn nearest_reachable_landing(
    start: TilePosition,
    mode: PlayerTraversalState,
    old: impl Fn(TilePosition) -> Option<u8>,
    safe_landing: impl Fn(TilePosition) -> bool,
) -> Option<TilePosition> {
    let mut seen = BTreeSet::from([(start.x, start.y)]);
    let mut queue = VecDeque::from([start]);
    while let Some(p) = queue.pop_front() {
        let Some(source) = old(p) else { continue };
        if p != start && safe_landing(p) {
            return Some(p);
        }
        for dir in DIRECTIONS {
            let q = p.moved(dir);
            if seen.contains(&(q.x, q.y))
                || collision::is_direction_blocked_leaving(source, dir)
                || bridge::blocks_exit(source, dir, mode)
            {
                continue;
            }
            if old(q).is_some_and(|k| {
                on_traversal_layer(k, mode)
                    && is_permission_passable(k, dir, mode)
                    && !collision::is_warp_permission(k)
            }) {
                seen.insert((q.x, q.y));
                queue.push_back(q);
            }
        }
    }
    None
}

#[cfg(test)]
#[path = "fair_save_migration_tests.rs"]
mod tests;
