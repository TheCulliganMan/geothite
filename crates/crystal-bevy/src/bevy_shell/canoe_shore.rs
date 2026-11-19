//! Native player art during a single automatic launch or landing.
use super::visible_movement_progress_with_subframe;
use super::{
    BevyRuntimeShell, MovementMode, PLAYER_GENDER_FEMALE, RuntimeShellSnapshot, TilePosition,
};
use crystal_core::systems::field_moves::CANOE_ACTIVE_MEMORY;
use crystal_core::world::session::OverworldSnapshot;

pub(super) type PlayerArt = (&'static str, &'static str, u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShorePose {
    Walking,
    Paddling,
}

impl ShorePose {
    fn for_mode(mode: MovementMode, canoe: bool) -> Option<Self> {
        match mode {
            MovementMode::Normal => Some(Self::Walking),
            MovementMode::Surf if canoe => Some(Self::Paddling),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub(super) struct CanoeShoreStep {
    map: String,
    from: TilePosition,
    to: TilePosition,
    departure: ShorePose,
    arrival: ShorePose,
}

impl CanoeShoreStep {
    pub(super) fn between(
        before: &OverworldSnapshot,
        from_canoe: bool,
        after: &OverworldSnapshot,
        to_canoe: bool,
    ) -> Option<Self> {
        let departure = ShorePose::for_mode(before.mode, from_canoe)?;
        let arrival = ShorePose::for_mode(after.mode, to_canoe)?;
        if departure == arrival
            || before.map_name != after.map_name
            || !after.map_name.starts_with("Mn")
            || before.tile.x.abs_diff(after.tile.x) as u32
                + before.tile.y.abs_diff(after.tile.y) as u32
                != 1
        {
            return None;
        }
        Some(Self {
            map: after.map_name.clone(),
            from: before.tile,
            to: after.tile,
            departure,
            arrival,
        })
    }

    fn departure_pose(
        &self,
        scene: &OverworldSnapshot,
        from: Option<TilePosition>,
        frames_remaining: u8,
        total_frames: u8,
        subframe: f32,
    ) -> Option<ShorePose> {
        if from != Some(self.from) || scene.tile != self.to || scene.map_name != self.map {
            return None;
        }
        self.contact_on_departure(frames_remaining, total_frames, subframe)
            .then_some(self.departure)
    }

    pub(super) fn landing_still_afloat(&self, frames_remaining: u8, total_frames: u8) -> bool {
        self.departure == ShorePose::Paddling
            && self.contact_on_departure(frames_remaining, total_frames, 0.0)
    }

    fn contact_on_departure(&self, frames_remaining: u8, total_frames: u8, subframe: f32) -> bool {
        // Normal movement is tile-atomic. Use the bottom-center contact point
        // between the boots or under the keel. Some poses leave that pixel
        // clear. Its position stays fixed as the silhouette strides or paddles.
        let progress =
            visible_movement_progress_with_subframe(frames_remaining, total_frames, subframe);
        let distance = 16.0 * progress;
        let x = 8.5 + f32::from(self.to.x - self.from.x) * distance;
        let y = 15.5 + f32::from(self.to.y - self.from.y) * distance;
        (0.0..16.0).contains(&x) && (0.0..16.0).contains(&y)
    }
}

pub(super) fn player_art(
    snapshot: &RuntimeShellSnapshot,
    crossing: Option<(&BevyRuntimeShell, f32)>,
) -> PlayerArt {
    let mounted = snapshot
        .script_events
        .memory
        .contains_key(CANOE_ACTIVE_MEMORY);
    let pose = crossing.and_then(|(shell, subframe)| {
        let step = shell.player_canoe_shore_step.as_ref()?;
        if Some(step.arrival) != ShorePose::for_mode(snapshot.overworld.mode, mounted) {
            return None;
        }
        step.departure_pose(
            &snapshot.overworld,
            shell.player_walk_from,
            shell.player_walk_frame_ticks,
            shell.player_walk_total_ticks,
            subframe,
        )
    });
    let (mode, canoe) = match pose {
        Some(ShorePose::Walking) => (MovementMode::Normal, false),
        Some(ShorePose::Paddling) => (MovementMode::Surf, true),
        None => (snapshot.overworld.mode, mounted),
    };
    let female = snapshot.trainer.player_gender == PLAYER_GENDER_FEMALE;
    let palette = snapshot.trainer.player_palette_id;
    match mode {
        MovementMode::Normal | MovementMode::Skate if female => ("kris", "SPRITE_KRIS", palette),
        MovementMode::Normal | MovementMode::Skate => ("chris", "SPRITE_CHRIS", palette),
        MovementMode::Bike if female => ("kris_bike", "SPRITE_KRIS_BIKE", palette),
        MovementMode::Bike => ("chris_bike", "SPRITE_CHRIS_BIKE", palette),
        MovementMode::Surf if canoe && female => ("mn_canoe_kris", "SPRITE_KRIS_BIKE", palette),
        MovementMode::Surf if canoe => ("mn_canoe_chris", "SPRITE_CHRIS_BIKE", palette),
        MovementMode::Surf => ("surf", "SPRITE_SURF", 1),
        MovementMode::SurfPika => ("surfing_pikachu", "SPRITE_SURFING_PIKACHU", 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crystal_core::world::map::Direction;

    fn point(x: i16, mode: MovementMode) -> OverworldSnapshot {
        OverworldSnapshot {
            map_name: String::from("MnR18C00R00"),
            tile: TilePosition::new(x, 12),
            mode,
            facing: Direction::Right,
            frame: 0,
        }
    }

    #[test]
    fn canoe_shore_step_changes_at_the_visible_sole_or_keel_in_each_direction() {
        let land = point(17, MovementMode::Normal);
        for (facing, forward_tick, reverse_tick) in [
            (Direction::Down, 1, 8),
            (Direction::Up, 8, 1),
            (Direction::Right, 4, 5),
            (Direction::Left, 5, 4),
        ] {
            let mut water = point(17, MovementMode::Surf);
            water.tile = land.tile.moved(facing);
            for (from, start, to, end, first_new_pose, old_pose) in [
                (&land, false, &water, true, forward_tick, ShorePose::Walking),
                (
                    &water,
                    true,
                    &land,
                    false,
                    reverse_tick,
                    ShorePose::Paddling,
                ),
            ] {
                let step = CanoeShoreStep::between(from, start, to, end).expect("shore step");
                for elapsed in 0..=8 {
                    let expected = (elapsed < first_new_pose).then_some(old_pose);
                    assert_eq!(
                        step.departure_pose(to, Some(from.tile), 8 - elapsed, 8, 0.0),
                        expected,
                        "moving from {:?} to {:?} on tick {elapsed}",
                        from.tile,
                        to.tile
                    );
                }
                assert_eq!(step.departure_pose(to, None, 8, 8, 0.0), None);
            }

            let launch = CanoeShoreStep::between(&land, false, &water, true).expect("launch");
            let landing = CanoeShoreStep::between(&water, true, &land, false).expect("landing");
            // Compare each half-pixel on the same physical path in reverse,
            // including coordinates exactly on either tile's leading edge.
            for half_pixel in 0..=32 {
                let on_land_leaving = launch
                    .departure_pose(&water, Some(land.tile), 1, 1, half_pixel as f32 / 32.0)
                    .is_some();
                let on_water_returning = landing
                    .departure_pose(
                        &land,
                        Some(water.tile),
                        1,
                        1,
                        (32 - half_pixel) as f32 / 32.0,
                    )
                    .is_some();
                assert_ne!(on_land_leaving, on_water_returning);
            }
        }
    }

    #[test]
    fn ordinary_surf_other_travel_and_warps_have_no_canoe_shore_step() {
        let land = point(17, MovementMode::Normal);
        let water = point(18, MovementMode::Surf);
        for (mode, active) in [
            (MovementMode::Normal, false),
            (MovementMode::Surf, false),
            (MovementMode::SurfPika, false),
            (MovementMode::Skate, false),
            (MovementMode::Bike, false),
        ] {
            let other = point(18, mode);
            assert!(CanoeShoreStep::between(&land, false, &other, active).is_none());
            assert!(CanoeShoreStep::between(&other, active, &land, false).is_none());
        }
        assert!(
            CanoeShoreStep::between(&point(17, MovementMode::Surf), true, &water, true).is_none()
        );
        assert!(
            CanoeShoreStep::between(&land, false, &point(23, MovementMode::Surf), true).is_none()
        );
        let mut elsewhere = water;
        elsewhere.map_name = "MnMilleLacs".into();
        assert!(CanoeShoreStep::between(&land, false, &elsewhere, true).is_none());
    }
}
