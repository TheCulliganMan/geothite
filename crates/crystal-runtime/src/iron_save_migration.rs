//! Local player relocations for the two revised Iron Workings galleries.

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_assets::GameDataSet;
use crystal_core::state::{GameState, OverworldMemory};
use crystal_core::world::collision::{PlayerTraversalState, permissions, sample_collision};
use crystal_core::world::map::TilePosition;
use crystal_core::world::session::object_tile_position_checked;
use serde::Deserialize;

pub(crate) const TARGET_MANIFEST: &str = "northstar-iron-workings-revision-v1";
const CHANGED_TILES: [(&str, [i16; 2], u8); 4] = [
    ("MnIronHaulage", [6, 8], permissions::HOP_DOWN),
    ("MnIronHaulage", [7, 8], permissions::HOP_DOWN),
    ("MnIronSump", [8, 6], permissions::WALL),
    ("MnIronSump", [9, 6], permissions::WALL),
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Relocation {
    map_id: String,
    from: [i16; 2],
    landings: Vec<[i16; 2]>,
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub(crate) struct IronRelocations(Vec<Relocation>);

impl IronRelocations {
    pub(crate) fn validate_metadata(&self) -> Result<()> {
        ensure!(
            self.0.len() == CHANGED_TILES.len(),
            "incomplete iron relocations"
        );
        let mut seen = BTreeSet::new();
        for row in &self.0 {
            ensure!(
                CHANGED_TILES
                    .iter()
                    .any(|&(map, tile, _)| map == row.map_id && tile == row.from)
                    && seen.insert((&row.map_id, row.from)),
                "iron relocation must name a distinct revised walking tile"
            );
            let mut unique = BTreeSet::new();
            ensure!(
                !row.landings.is_empty()
                    && row.landings.iter().all(|&at| {
                        let distance = u32::from(at[0].abs_diff(row.from[0]))
                            + u32::from(at[1].abs_diff(row.from[1]));
                        (1..=2).contains(&distance) && unique.insert(at)
                    }),
                "iron landings must be distinct nearby tiles"
            );
        }
        Ok(())
    }

    pub(crate) fn validate_landings(&self, data: &GameDataSet) -> Result<()> {
        for row in &self.0 {
            let session = data.overworld_session(&row.map_id, position(row.landings[0]), 0)?;
            let from = sample_collision(&session.map, &session.tileset, position(row.from));
            ensure!(
                CHANGED_TILES
                    .iter()
                    .any(|&(map, tile, permission)| map == row.map_id
                        && tile == row.from
                        && from.is_some_and(|s| s.permission == permission)),
                "iron relocation origin has unexpected target geometry"
            );
            let module = data.map_module(&row.map_id)?;
            for &at in &row.landings {
                let tile = position(at);
                ensure!(
                    sample_collision(&session.map, &session.tileset, tile)
                        .is_some_and(|s| s.permission == permissions::FLOOR)
                        && session
                            .objects
                            .iter()
                            .all(|o| object_tile_position_checked(o) != Some(tile))
                        && !module
                            .events
                            .warps
                            .iter()
                            .any(|w| [i32::from(w.x), i32::from(w.y)] == at.map(i32::from))
                        && !module
                            .events
                            .coord_events
                            .iter()
                            .any(|e| [i32::from(e.x), i32::from(e.y)] == at.map(i32::from)),
                    "iron relocation landing must be ordinary unoccupied floor away from triggers: {} {at:?}",
                    row.map_id
                );
            }
        }
        Ok(())
    }

    pub(crate) fn apply(&self, data: &GameDataSet, state: &mut GameState) -> Result<()> {
        let OverworldMemory::Active {
            map_name,
            tile,
            mode,
            ..
        } = &state.overworld
        else {
            anyhow::bail!("iron upgrade requires an overworld checkpoint");
        };
        let Some(row) = self
            .0
            .iter()
            .find(|r| r.map_id == *map_name && position(r.from) == *tile)
        else {
            return Ok(());
        };
        // Start the in-memory collision query on validated base floor. The saved
        // player's own tile can now be solid. Restore the real block and actor
        // snapshots into this temporary query without touching the checkpoint.
        let mut session =
            data.overworld_session(map_name, position(row.landings[0]), state.frame_counter)?;
        data.apply_saved_overworld_overrides(&mut session, state)?;
        let current = sample_collision(&session.map, &session.tileset, *tile);
        if current.is_some_and(|s| s.permission == permissions::FLOOR) {
            return Ok(());
        }
        ensure!(
            mode.traversal_state() == PlayerTraversalState::Walk,
            "iron relocation expects a saved walking tile"
        );
        let occupied = session.occupied_tiles_checked()?;
        let landing = row
            .landings
            .iter()
            .copied()
            .map(position)
            .find(|&at| {
                sample_collision(&session.map, &session.tileset, at)
                    .is_some_and(|s| s.permission == permissions::FLOOR)
                    && occupied.iter().all(|actor| actor.tile != at)
            })
            .context("all reviewed iron relocation landings are blocked")?;
        if let OverworldMemory::Active { tile, .. } = &mut state.overworld {
            *tile = landing;
        }
        Ok(())
    }
}

fn position([x, y]: [i16; 2]) -> TilePosition {
    TilePosition::new(x, y)
}
