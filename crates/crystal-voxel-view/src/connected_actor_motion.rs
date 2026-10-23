//! Carry global actor rigs and gait through proven changes of map coordinates.
use std::sync::Arc;

use bevy::prelude::{Commands, UVec2, Vec2};
use crystal_render_api::{VisualActorId, VisualSourceRegion, VisualWorldFrame};

use super::ModeledActors;

pub(super) struct Basis {
    map: Arc<str>,
    regions: Arc<[VisualSourceRegion]>,
    tile_size: Vec2,
    grid_size: UVec2,
}

impl Basis {
    fn capture(frame: &VisualWorldFrame) -> Self {
        Self {
            map: Arc::clone(&frame.map_id),
            regions: Arc::clone(&frame.source_regions),
            tile_size: frame.tile_size,
            grid_size: frame.grid_size,
        }
    }

    fn shift(&self, next: &VisualWorldFrame) -> Option<Vec2> {
        if self.tile_size != next.tile_size
            || self.grid_size != next.grid_size
            || !self.tile_size.is_finite()
            || self.tile_size.cmple(Vec2::ZERO).any()
        {
            return None;
        }
        // Only construct a proof frame at a map handoff. Ordinary updates
        // retain two Arc references, without cloning cells or empty atlases.
        let previous = VisualWorldFrame {
            map_id: Arc::clone(&self.map),
            source_regions: Arc::clone(&self.regions),
            ..Default::default()
        };
        let shift = previous.connected_source_shift(next)?;
        // Motion already removes viewport scrolling via world_foot(). With
        // the same grid and scale, only this source-map translation remains.
        Some(shift.as_vec2() * self.tile_size * Vec2::new(1.0, -1.0))
    }
}

fn persistent(id: VisualActorId) -> bool {
    matches!(
        id,
        VisualActorId::Player
            | VisualActorId::PartyFollower
            | VisualActorId::SurfMount
            | VisualActorId::RemotePlayer(_)
    )
}

pub(super) fn handoff(
    state: &mut ModeledActors,
    frame: &VisualWorldFrame,
    enabled: bool,
    commands: &mut Commands,
) {
    if !enabled || state.map_id != frame.map_id.as_ref() {
        let shift = enabled
            .then(|| {
                state
                    .motion_basis
                    .as_ref()
                    .and_then(|basis| basis.shift(frame))
            })
            .flatten();
        state.instances.retain(|id, instance| {
            if let Some(shift) = shift.filter(|_| persistent(*id)) {
                instance.motion.last_world_foot += shift;
                true
            } else {
                commands.entity(instance.entity).despawn();
                false
            }
        });
        state.props.retain(|id, instance| {
            if let Some(shift) = shift.filter(|_| persistent(*id)) {
                if let Some(skin) = &mut instance.skin {
                    skin.motion.last_world_foot += shift;
                }
                true
            } else {
                commands.entity(instance.entity).despawn();
                false
            }
        });
        state.map_id = frame.map_id.to_string();
    }
    state.motion_basis = enabled.then(|| Basis::capture(frame));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::IVec2;

    fn frame(map: &str, other: &str, offset: IVec2) -> VisualWorldFrame {
        VisualWorldFrame {
            map_id: map.into(),
            tile_size: Vec2::splat(8.0),
            grid_size: UVec2::splat(64),
            source_regions: vec![
                VisualSourceRegion {
                    map_id: map.into(),
                    origin: IVec2::ZERO,
                    size: UVec2::splat(40),
                },
                VisualSourceRegion {
                    map_id: other.into(),
                    origin: offset,
                    size: UVec2::splat(40),
                },
            ]
            .into(),
            ..Default::default()
        }
    }

    #[test]
    fn all_four_connected_bases_translate_gait_history_without_scroll() {
        for offset in [
            IVec2::new(40, 0),
            IVec2::new(-40, 0),
            IVec2::new(0, 40),
            IVec2::new(0, -40),
        ] {
            let before = frame("Route29", "CherrygroveCity", offset);
            let mut after = frame("CherrygroveCity", "Route29", -offset);
            after.grid_origin = IVec2::new(4, 8);
            after.center = Vec2::new(12.0, -56.0);
            assert_eq!(
                Basis::capture(&before).shift(&after),
                Some(-offset.as_vec2() * before.tile_size * Vec2::new(1.0, -1.0))
            );
        }
    }

    #[test]
    fn unproven_warps_and_changed_scale_discard_old_coordinate_history() {
        let before = frame("Route29", "CherrygroveCity", IVec2::new(-40, 0));
        let basis = Basis::capture(&before);
        let mut after = frame("CherrygroveCity", "Route29", IVec2::new(40, 0));
        after.source_regions = Arc::from([]);
        assert!(basis.shift(&after).is_none());
        let mut after = frame("CherrygroveCity", "Route29", IVec2::new(40, 0));
        after.tile_size *= 2.0;
        assert!(basis.shift(&after).is_none());
        assert!(
            !persistent(VisualActorId::Object(0)),
            "NPC indices are map-local identities"
        );
        assert!(!persistent(VisualActorId::Effect(
            crystal_render_api::VisualEffectId::GrassRustle
        )));
        assert!(persistent(VisualActorId::Player));
        assert!(persistent(VisualActorId::SurfMount));
    }

    #[test]
    fn translated_gait_matches_the_same_walk_without_a_map_handoff() {
        use super::super::Motion;
        let mut reference = Motion::new(Vec2::ZERO, Vec2::X);
        for tick in 1..=30 {
            reference.advance(Vec2::new(tick as f32, 0.0), Some(Vec2::X), 20.0, 1.0 / 60.0);
        }
        let mut rebased = reference;
        let shift = Vec2::new(-640.0, 320.0);
        rebased.last_world_foot += shift;
        reference.advance(Vec2::new(31.0, 0.0), Some(Vec2::X), 20.0, 1.0 / 60.0);
        rebased.advance(
            Vec2::new(31.0, 0.0) + shift,
            Some(Vec2::X),
            20.0,
            1.0 / 60.0,
        );
        assert_eq!(
            rebased.phase, reference.phase,
            "map coordinates must not restart the gait cycle"
        );
        assert_eq!(rebased.walk_weight, reference.walk_weight);
        assert_eq!(rebased.run_weight, reference.run_weight);
        assert_eq!(rebased.speed, reference.speed);
        assert_eq!(rebased.yaw, reference.yaw);
    }
}
