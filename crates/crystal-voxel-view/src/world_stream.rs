//! Sliding source-space coverage, independent of authored map boundaries.
use std::sync::Arc;

use bevy::prelude::{IVec2, Res, ResMut, Resource, Vec2};
use crystal_render_api::{VisualActorId, VisualWorldFrame};

use crate::{TerrainRevisionCache, VoxelViewSettings, footing, loading_trace, sky};

const WINDOW_STRIDE: f32 = 16.0;

#[cfg(test)]
#[path = "world_stream_fixture.rs"]
pub(crate) mod test_frames;

/// The next terrain window to prepare from the read-only content catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldStreamTarget {
    /// Coordinate basis for `tile`; this does not select a gameplay map.
    pub map_id: Arc<str>,
    /// Resident window center in gameplay cells, including connected-map acreage.
    pub tile: IVec2,
}

/// Progressive world-rendering demand and completed window promotions.
#[derive(Resource, Default)]
pub struct WorldStreamStatus {
    /// Read-only coverage requested around the player in every direction.
    pub target: Option<WorldStreamTarget>,
    /// Completed coverage windows adopted without a gameplay loading phase.
    pub promotions: u64,
    basis: Option<VisualWorldFrame>,
    origin: IVec2,
}

impl WorldStreamStatus {
    fn update(&mut self, frame: &VisualWorldFrame) {
        if !frame.active
            || !sky::outdoors(frame)
            || !frame.tile_size.is_finite()
            || frame.tile_size.cmple(Vec2::ZERO).any()
        {
            self.target = None;
            return;
        }
        if self
            .basis
            .as_ref()
            .is_none_or(|old| old.map_id != frame.map_id)
        {
            self.origin = self
                .basis
                .as_ref()
                .and_then(|old| old.connected_source_shift(frame))
                .and_then(|shift| {
                    Some(IVec2::new(
                        self.origin.x.checked_sub(shift.x)?,
                        self.origin.y.checked_sub(shift.y)?,
                    ))
                })
                .unwrap_or(IVec2::ZERO);
            // Only placement metadata participates in the coordinate proof.
            self.basis = Some(VisualWorldFrame {
                map_id: Arc::clone(&frame.map_id),
                source_regions: Arc::clone(&frame.source_regions),
                ..Default::default()
            });
        }
        let Some(player) = frame
            .actors
            .iter()
            .find(|actor| actor.id == VisualActorId::Player)
        else {
            self.target = None;
            return;
        };
        let source = frame.grid_origin.as_vec2()
            + frame.grid_size.as_vec2() * 0.5
            + (footing::actor_ground_point(frame, player) - frame.center) / frame.tile_size
                * Vec2::new(1.0, -1.0);
        if !source.is_finite() {
            self.target = None;
            return;
        }
        let global = source + self.origin.as_vec2();
        let center =
            (global / WINDOW_STRIDE).floor() * WINDOW_STRIDE + Vec2::splat(WINDOW_STRIDE * 0.5);
        self.target = Some(WorldStreamTarget {
            map_id: Arc::clone(&frame.map_id),
            tile: ((center - self.origin.as_vec2()) * 0.5).round().as_ivec2(),
        });
    }
}

pub(super) fn update_interest(
    frame: Res<VisualWorldFrame>,
    settings: Res<VoxelViewSettings>,
    resident: Res<TerrainRevisionCache>,
    mut stream: ResMut<WorldStreamStatus>,
) {
    if settings.enabled {
        if frame.active
            && sky::outdoors(&frame)
            && stream
                .basis
                .as_ref()
                .is_none_or(|old| old.map_id != frame.map_id)
        {
            let _trace = loading_trace::span("world.coverage_basis", || {
                serde_json::json!({
                    "map": frame.map_id.as_ref(),
                    "resident_map": resident.built_frame.as_ref().map(|built| built.map_id.as_ref()),
                    "resident_origin": resident.built_frame.as_ref().map(|built| built.grid_origin.to_array()),
                    "roots": ([resident.instances_root, resident.textured_entity, resident.solid_entity,
                        resident.animated_textured_entity, resident.animated_solid_entity]
                        .into_iter().flatten().map(|entity| entity.to_bits()).collect::<Vec<_>>()),
                    "window_promotions": stream.promotions,
                })
            });
        }
        stream.update(&frame);
    } else {
        stream.target = None;
    }
}

pub(super) fn needs_refresh(built: &VisualWorldFrame, live: &VisualWorldFrame) -> bool {
    let Some(shift) = built.connected_source_shift(live) else {
        return true;
    };
    let delta = live.grid_origin - built.grid_origin - shift;
    let allowance = crate::terrain_tracking::retention_allowance(live);
    // Promote inside the retained overlap, leaving room for pitched scenery
    // and worker variation before the old window's coverage is exhausted.
    delta.abs().cmpge((allowance / 2).max(IVec2::ONE)).any()
}

/// A replacement must move coverage toward the live camera. An overlapping
/// window left behind after promotion must never be promoted back again.
pub(super) fn advances_coverage(
    resident: &VisualWorldFrame,
    candidate: &VisualWorldFrame,
    live: &VisualWorldFrame,
) -> bool {
    let distance = |built: &VisualWorldFrame| {
        built.connected_source_shift(live).map(|shift| {
            (live.grid_origin - built.grid_origin - shift)
                .abs()
                .max_element()
        })
    };
    match (distance(resident), distance(candidate)) {
        (Some(old), Some(next)) => next < old,
        (None, Some(_)) => true,
        _ => false,
    }
}

pub(super) fn same_window(a: &VisualWorldFrame, b: &VisualWorldFrame) -> bool {
    a.grid_size == b.grid_size
        && a.tile_size == b.tile_size
        && a.viewport_size == b.viewport_size
        && a.connected_source_shift(b)
            .is_some_and(|shift| b.grid_origin - a.grid_origin == shift)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{Handle, UVec2};
    use crystal_render_api::{VisualActor, VisualSourceRegion, VisualTile, VisualTileSource};

    fn outdoor() -> VisualWorldFrame {
        VisualWorldFrame {
            active: true,
            map_id: "RouteA".into(),
            tile_size: Vec2::splat(8.0),
            grid_size: UVec2::splat(84),
            tiles: vec![VisualTile {
                column: 0,
                row: 0,
                texture: Handle::default(),
                priority: false,
                animation_frames: None,
                source: VisualTileSource {
                    tileset_id: "johto".into(),
                    metatile_id: 1,
                    tile_index: 1,
                    subtile_column: 0,
                    subtile_row: 0,
                },
            }],
            actors: vec![VisualActor {
                id: VisualActorId::Player,
                source_id: "chris".into(),
                model_palette: None,
                texture: Handle::default(),
                center: Vec2::ZERO,
                size: Vec2::splat(16.0),
                flip_x: false,
                above_priority: false,
                facing: Some(Vec2::X),
            }],
            source_regions: vec![
                VisualSourceRegion {
                    map_id: "RouteA".into(),
                    origin: IVec2::ZERO,
                    size: UVec2::splat(80),
                },
                VisualSourceRegion {
                    map_id: "RouteB".into(),
                    origin: IVec2::new(80, 0),
                    size: UVec2::splat(80),
                },
            ]
            .into(),
            ..Default::default()
        }
    }

    #[test]
    fn coverage_stays_centered_when_the_trainer_turns() {
        let frame = outdoor();
        let mut stream = WorldStreamStatus::default();
        stream.update(&frame);
        let target = stream.target.clone().expect("outdoor coverage");
        let source = frame.grid_size.as_vec2() * 0.5;
        assert!(
            (target.tile.as_vec2() * 2.0 - source)
                .abs()
                .cmple(Vec2::splat(WINDOW_STRIDE * 0.5))
                .all()
        );
        for facing in [Vec2::X, Vec2::Y, -Vec2::X, -Vec2::Y] {
            let mut turned = frame.clone();
            turned.actors[0].facing = Some(facing);
            stream.update(&turned);
            assert_eq!(stream.target.as_ref(), Some(&target));
        }
    }

    #[test]
    fn expanded_coverage_starts_replacement_before_its_outer_buffer_is_used() {
        let mut resident = outdoor();
        resident.viewport_size = Vec2::new(160., 144.);
        resident.grid_size = UVec2::new(216, 212);
        let allowance = crate::terrain_tracking::retention_allowance(&resident);
        assert_eq!(allowance, IVec2::splat(16));
        for direction in [IVec2::X, IVec2::Y, -IVec2::X, -IVec2::Y] {
            let mut live = resident.clone();
            live.grid_origin += direction * 7;
            assert!(!needs_refresh(&resident, &live));
            live.grid_origin += direction;
            assert!(needs_refresh(&resident, &live));
        }
        let halo = (resident.grid_size.as_ivec2() - IVec2::new(20, 18)) / 2;
        assert!(
            (halo - allowance - IVec2::splat(16))
                .cmpge(IVec2::splat(64))
                .all()
        );
    }

    #[test]
    fn a_logical_map_rebase_does_not_move_the_requested_world_window() {
        let before = outdoor();
        let mut after = before.clone();
        after.map_id = "RouteB".into();
        after.grid_origin.x -= 80;
        after.source_regions = vec![
            VisualSourceRegion {
                map_id: "RouteB".into(),
                origin: IVec2::ZERO,
                size: UVec2::splat(80),
            },
            VisualSourceRegion {
                map_id: "RouteA".into(),
                origin: IVec2::new(-80, 0),
                size: UVec2::splat(80),
            },
        ]
        .into();
        let mut stream = WorldStreamStatus::default();
        stream.update(&before);
        let old = stream.target.clone().expect("first window");
        stream.update(&after);
        let new = stream.target.expect("connected window");
        assert_eq!(new.tile + IVec2::new(40, 0), old.tile);
    }

    #[test]
    fn a_retained_old_window_cannot_reverse_a_coverage_promotion() {
        let (resident, _) = test_frames::connected_frames(IVec2::new(40, 0));
        let mut live = resident.clone();
        live.grid_origin.x += 4;
        let mut behind = resident.clone();
        behind.grid_origin.x -= 4;
        assert!(!advances_coverage(&resident, &behind, &live));
        assert!(advances_coverage(&resident, &live, &live));
        // Reversing travel makes the retained window useful again.
        assert!(advances_coverage(&live, &resident, &behind));
    }

    #[test]
    fn coverage_progress_and_window_identity_survive_coordinate_rebasing() {
        let (resident, mut live) = test_frames::connected_frames(IVec2::new(40, 0));
        assert!(same_window(&resident, &live));
        assert!(!advances_coverage(&resident, &live, &live));
        live.grid_origin.y += 4;
        assert!(!same_window(&resident, &live));
        assert!(advances_coverage(&resident, &live, &live));
    }
}
