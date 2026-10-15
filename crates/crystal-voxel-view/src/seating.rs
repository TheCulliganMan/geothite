//! Furniture animation is presentation state; authored movement remains authoritative.
use bevy::prelude::*;
use crystal_render_api::VisualWorldFrame;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Seat {
    pub rect: [f32; 4],
    pub height: f32,
    pub yaw: Option<f32>,
}
impl Seat {
    pub fn actor_target(
        self,
        frame: &VisualWorldFrame,
        foot: Vec2,
        facing: Option<Vec2>,
        ground: f32,
    ) -> Option<Vec3> {
        if let Some(target) = self.target(frame, foot, ground) {
            return Some(target);
        }
        let facing = facing.filter(|facing| facing.is_finite() && facing.length_squared() > 0.5)?;
        // A source gameplay tile is two render cells. Backed furniture may
        // block entry; a player resting directly in front can still sit on it
        // visually without changing authoritative position or collision.
        let forward = facing.normalize() * frame.tile_size * 2.;
        let mut offset = self.target(frame, foot + forward, ground)?;
        // Backed seats are approached from their open side, not through the
        // backrest. Round stools accept any approach direction.
        if self
            .yaw
            .is_some_and(|yaw| facing.dot(Vec2::new(-yaw.sin(), yaw.cos())) < 0.9)
        {
            return None;
        }
        offset += Vec3::new(forward.x, 0., -forward.y);
        Some(offset)
    }
    pub fn target(self, frame: &VisualWorldFrame, foot: Vec2, ground: f32) -> Option<Vec3> {
        let top_left = frame.center
            + Vec2::new(
                -frame.tile_size.x * frame.grid_size.x as f32 * 0.5,
                frame.tile_size.y * frame.grid_size.y as f32 * 0.5,
            );
        let point = (foot - top_left) / frame.tile_size * Vec2::new(1., -1.);
        let [x, y, width, depth] = self.rect;
        // Feet lie on the south edge of their source gameplay tile. Exclude
        // the bench's legs and neighboring ground from the seat footprint.
        if point.x < x || point.x >= x + width || point.y <= y || point.y > y + depth + 0.001 {
            return None;
        }
        let seat_x = if width > 2. {
            point.x.clamp(x + 0.75, x + width - 0.75)
        } else {
            x + width * 0.5
        };
        Some(Vec3::new(
            (seat_x - point.x) * frame.tile_size.x,
            self.height - ground,
            (y + depth * 0.5 - point.y) * frame.tile_size.y,
        ))
    }
}

#[derive(Default)]
pub(crate) struct Sitting {
    pub weight: f32,
    velocity: f32,
    pub offset: Vec3,
    pub height: f32,
    pub yaw: Option<f32>,
}
impl Sitting {
    pub fn advance(
        &mut self,
        target: Option<(Vec3, f32, Option<f32>)>,
        still: bool,
        dt: f32,
        actor_scale: f32,
    ) {
        let target = target.filter(|_| still);
        if let Some((offset, height, yaw)) = target {
            self.offset = offset;
            // The rig’s own height is already included in model_scale.
            self.height = height / actor_scale;
            self.yaw = yaw;
        }
        // Exact critically damped spring: frame-rate independent, with continuous
        // velocity even when walking interrupts the descent onto the cushion.
        let goal = if target.is_some() { 1. } else { 0. };
        let dt = dt.clamp(0., 0.1);
        let response = 20.;
        let displacement = self.weight - goal;
        let impulse = self.velocity + response * displacement;
        let decay = (-response * dt).exp();
        self.weight = goal + (displacement + impulse * dt) * decay;
        self.velocity = (self.velocity - response * impulse * dt) * decay;
        if (self.weight - goal).abs() < 0.0001 && self.velocity.abs() < 0.002 {
            self.weight = goal;
            self.velocity = 0.;
        }
    }
    pub fn eased_weight(&self) -> f32 {
        self.weight
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bench_seat_excludes_legs_and_tracks_camera_translation() {
        let mut frame = VisualWorldFrame::default();
        frame.grid_size = UVec2::new(4, 4);
        frame.tile_size = Vec2::splat(8.);
        frame.center = Vec2::new(16., -16.);
        let bench = Seat {
            rect: [0., 0., 4., 2.],
            height: 5.,
            yaw: Some(0.),
        };
        let foot = Vec2::new(24., -16.);
        let target = bench.target(&frame, foot, 0.).unwrap();
        assert_eq!(target, Vec3::new(0., 5., -8.));
        assert!(bench.target(&frame, Vec2::new(24., -24.), 0.).is_none());
        assert!(bench.target(&frame, Vec2::new(32., -16.), 0.).is_none());
        frame.center += Vec2::new(7., -11.);
        assert_eq!(
            bench.target(&frame, foot + Vec2::new(7., -11.), 0.),
            Some(target)
        );
    }
    #[test]
    fn blocked_bench_can_be_sat_on_from_its_front_without_changing_the_game_foot() {
        let mut frame = VisualWorldFrame::default();
        frame.grid_size = UVec2::new(4, 4);
        frame.tile_size = Vec2::splat(8.);
        frame.center = Vec2::new(16., -16.);
        let bench = Seat {
            rect: [0., 0., 4., 2.],
            height: 5.,
            yaw: Some(0.),
        };
        let foot = Vec2::new(24., -32.);
        assert!(bench.target(&frame, foot, 0.).is_none());
        assert_eq!(
            bench.actor_target(&frame, foot, Some(Vec2::Y), 0.),
            Some(Vec3::new(0., 5., -24.))
        );
        assert!(
            bench
                .actor_target(&frame, foot, Some(Vec2::NEG_Y), 0.)
                .is_none()
        );
        assert!(bench.actor_target(&frame, foot, None, 0.).is_none());
        assert!(
            bench
                .actor_target(&frame, foot - Vec2::Y * 16., Some(Vec2::Y), 0.)
                .is_none()
        );
        assert_eq!(foot, Vec2::new(24., -32.));
    }

    #[test]
    fn sitting_blends_in_and_stands_on_departure_without_resitting_during_motion() {
        let mut sitting = Sitting::default();
        let target = Some((Vec3::new(0., 5., -8.), 5., Some(0.)));
        for _ in 0..45 {
            sitting.advance(target, true, 1. / 60., 1.);
        }
        assert!((sitting.weight - 1.).abs() < 1e-5);
        assert_eq!(sitting.offset, target.unwrap().0);
        for _ in 0..45 {
            sitting.advance(target, false, 1. / 60., 1.);
        }
        assert!(sitting.weight < 1e-5);
        for _ in 0..60 {
            sitting.advance(None, true, 1. / 60., 1.);
        }
        assert_eq!(sitting.weight, 0.);
    }

    #[test]
    fn interrupted_sitting_preserves_velocity_and_matches_frame_cadence() {
        let target = Some((Vec3::Y, 1., None));
        let mut slow = Sitting::default();
        let mut fast = Sitting::default();
        for _ in 0..6 {
            slow.advance(target, true, 1. / 30., 1.);
        }
        for _ in 0..24 {
            fast.advance(target, true, 1. / 120., 1.);
        }
        assert!((slow.weight - fast.weight).abs() < 1e-6);
        let previous_velocity = fast.velocity;
        let previous_weight = fast.weight;
        fast.advance(None, false, 0.00001, 1.);
        assert!((fast.velocity - previous_velocity).abs() < 0.005);
        assert!((fast.weight - previous_weight).abs() < 0.0001);
        for _ in 0..120 {
            fast.advance(None, false, 1. / 120., 1.);
            assert!((0. ..=1.).contains(&fast.weight));
        }
        assert_eq!(fast.weight, 0.);
    }
}

/// The broad horizontal upward face is the cushion rather than a backrest
/// cap or a foot. Source meshes remain the sole authority for furniture size.
pub(crate) fn cushion_height(positions: &[[f32; 3]], indices: &[u32]) -> Option<f32> {
    let mut planes: std::collections::BTreeMap<i32, f32> = Default::default();
    for triangle in indices.chunks_exact(3) {
        let a = Vec3::from_array(positions[triangle[0] as usize]);
        let b = Vec3::from_array(positions[triangle[1] as usize]);
        let c = Vec3::from_array(positions[triangle[2] as usize]);
        let normal = (b - a).cross(c - a);
        if normal.y > 0. && normal.x.abs() + normal.z.abs() < normal.y * 0.001 {
            *planes.entry((a.y * 10000.).round() as i32).or_default() += normal.y;
        }
    }
    planes
        .into_iter()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(height, _)| height as f32 / 10000.)
}
