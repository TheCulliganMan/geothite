//! Battle geometry measurements and one common, ratio-preserving scene layout.
//! Species dimensions come from the loaded pack; numbers here describe our art.
use bevy::prelude::*;

/// A single scene unit conversion, never a per-participant readability clamp.
pub(crate) const WORLD_UNITS_PER_METER: f32 = 2.65;

/// Match the pack's listed physical dimension to an authored body reference.
/// Most standing bodies use Y. Explicit exceptions exclude scenery/effects or
/// follow authored body length. These are original-art measurements, not a
/// duplicate catalogue of Pokédex dimensions.
pub(crate) fn reference_span(species: &str, min: Vec3, max: Vec3) -> f32 {
    let extent = max - min;
    let height_ratio = match species {
        // Ground to fitted dorsal coat; flames are included only in fit bounds.
        "CYNDAQUIL" => 0.716_729 / 0.91,
        // Capsule only; disturbed soil is not part of the animal's height.
        "DIGLETT" => 0.805_091 / 0.82,
        // Tail-to-face centerline of the current authored pose, excluding horns.
        "ONIX" => 1.857_489_3 / 1.72,
        "GYARADOS" => 1.270_513_3 / 0.82,
        "EKANS" => 2.485_483_5 / 1.18,
        "ARBOK" => 2.329_859_6 / 1.18,
        "DRATINI" | "DRAGONAIR" => 2.241_65 / 1.18,
        "STEELIX" => 2.249_835_3 / 1.18,
        // Longitudinal nose-to-tail extent, not fin/lure height.
        "MAGIKARP" | "GOLDEEN" | "SEAKING" | "REMORAID" | "CHINCHOU" | "LANTURN" | "DUNSPARCE"
        | "CATERPIE" | "WEEDLE" => {
            return extent.z;
        }
        _ => 1.0,
    };
    extent.y * height_ratio
}

pub(crate) fn model_scale(
    species: &str,
    pokedex_size_m: Option<f32>,
    min: Vec3,
    max: Vec3,
) -> Option<f32> {
    let meters = pokedex_size_m.filter(|v| v.is_finite() && *v > 0.0)?;
    let span = reference_span(species, min, max);
    (span.is_finite() && span > 0.0).then_some(WORLD_UNITS_PER_METER * meters / span)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn body_measurements_preserve_real_relative_sizes_and_uniform_proportions() {
        let c = model_scale(
            "CYNDAQUIL",
            Some(0.508),
            Vec3::ZERO,
            Vec3::new(0.65451, 0.91, 1.073912),
        )
        .unwrap();
        let s = model_scale(
            "SUDOWOODO",
            Some(1.1938),
            Vec3::ZERO,
            Vec3::new(0.770198, 0.82, 0.226966),
        )
        .unwrap();
        assert!((s * 0.82 / (c * 0.716729) - 2.35).abs() < 1e-5);
        assert!(c * 0.91 > c * 0.716729, "flames remain above measured body");
        let original = Vec3::new(0.65451, 0.91, 1.073912);
        let scaled = original * c;
        assert!((scaled.x / scaled.y - original.x / original.y).abs() < 1e-6);
    }
    #[test]
    fn serpent_dimension_measures_body_length_not_standing_height() {
        let scale = model_scale(
            "STEELIX",
            Some(9.1948),
            Vec3::ZERO,
            Vec3::new(0.766938, 1.18, 0.599248),
        )
        .unwrap();
        assert!((scale * 2.2498353 / WORLD_UNITS_PER_METER - 9.1948).abs() < 1e-5);
        assert!(scale * 1.18 / WORLD_UNITS_PER_METER < 5.0);
    }
    #[test]
    fn missing_or_invalid_pack_dimension_never_invents_a_species_size() {
        for value in [
            None,
            Some(0.0),
            Some(-1.0),
            Some(f32::NAN),
            Some(f32::INFINITY),
        ] {
            assert!(model_scale("CYNDAQUIL", value, Vec3::ZERO, Vec3::ONE).is_none());
        }
    }
}

pub(crate) const CAMERA_FOV: f32 = 0.58;
pub(crate) const SOURCE_PIXEL_WORLD: f32 = 0.045;
const DEFAULT_SEPARATION: f32 = 5.161_395;
const FLOOR_Y: f32 = 0.12;

/// The complete visual envelope is independent of its measurement reference.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BattleBody {
    pub min: Vec3,
    pub max: Vec3,
    /// Full animation envelope; neutral bounds still define grounding and size.
    pub visual_bounds: Option<(Vec3, Vec3)>,
    pub scale: Vec3,
    pub reference_height: f32,
    pub modeled: bool,
}
impl BattleBody {
    pub(crate) fn modeled(species: &str, min: Vec3, max: Vec3, scale: f32) -> Self {
        let height = (max - min).y;
        let reference_height = match species {
            "CYNDAQUIL" => height * (0.716_729 / 0.91),
            "DIGLETT" => height * (0.805_091 / 0.82),
            _ => height,
        };
        Self {
            min,
            max,
            visual_bounds: None,
            scale: Vec3::splat(scale),
            reference_height,
            modeled: true,
        }
    }
    pub(crate) fn source_card(aspect: f32) -> Self {
        Self {
            min: Vec3::new(-0.95, -0.95, -0.001),
            max: Vec3::new(0.95, 0.95, 0.001),
            visual_bounds: None,
            scale: Vec3::new(aspect, 1.0, 1.0),
            reference_height: 1.9,
            modeled: false,
        }
    }
    pub(crate) fn corners(self, pose: Transform) -> [Vec3; 8] {
        let (min, max) = self.visual_bounds.unwrap_or((self.min, self.max));
        std::array::from_fn(|i| {
            pose.transform_point(Vec3::new(
                if i & 1 == 0 { min.x } else { max.x },
                if i & 2 == 0 { min.y } else { max.y },
                if i & 4 == 0 { min.z } else { max.z },
            ))
        })
    }
}

/// Shared render-only projection for actors, source OAM, rows and hit anchors.
/// Created once from both neutral bodies; transient visibility and source
/// offsets do not reframe a held attack or resize an individual Pokémon.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct BattleSceneLayout {
    pub camera: Transform,
    pub far: f32,
    pub distance: f32,
    pub(crate) body_poses: [Option<Transform>; 2],
    pub(crate) origins: [Vec3; 2],
    pub(crate) hit_anchors: [Vec3; 2],
    pub(crate) arena_scale: f32,
    viewport: Vec2,
}
impl Default for BattleSceneLayout {
    fn default() -> Self {
        let origins = [Vec3::new(-2.1, FLOOR_Y, 1.5), Vec3::new(2.1, FLOOR_Y, -1.5)];
        let camera =
            Transform::from_xyz(7.8, 6.3, 11.6).looking_at(Vec3::new(0.0, 0.80, 0.0), Vec3::Y);
        Self {
            camera,
            far: 100.0,
            distance: camera.translation.distance(Vec3::new(0.0, 0.80, 0.0)),
            body_poses: [None, None],
            origins,
            hit_anchors: origins.map(|p| p + Vec3::Y),
            arena_scale: 1.0,
            viewport: Vec2::new(16.0, 9.0),
        }
    }
}
impl BattleSceneLayout {
    pub(crate) fn for_bodies(bodies: [Option<BattleBody>; 2], viewport: Vec2) -> Self {
        let mut layout = Self::default();
        let viewport = if viewport.is_finite() && viewport.min_element() > 0.0 {
            viewport
        } else {
            layout.viewport
        };
        layout.viewport = viewport;
        let axis = (layout.origins[1] - layout.origins[0]).normalize();
        let camera_yaw = Vec3::new(7.8, 0.0, 11.6).normalize();
        let rotations: [Quat; 2] = std::array::from_fn(|i| {
            if bodies[i].is_some_and(|body| !body.modeled) {
                return layout.camera.rotation;
            }
            let facing = (axis * if i == 0 { 1.0 } else { -1.0 })
                .lerp(camera_yaw, 0.40)
                .normalize();
            Quat::from_rotation_y(facing.x.atan2(facing.z))
        });
        let radii: [f32; 2] = std::array::from_fn(|i| {
            bodies[i].map_or(0.0, |body| {
                let pose = Transform {
                    rotation: rotations[i],
                    scale: body.scale,
                    ..default()
                };
                body.corners(pose)
                    .iter()
                    .map(|point| point.dot(axis).abs())
                    .fold(0.0, f32::max)
            })
        });
        let separation = DEFAULT_SEPARATION.max(radii[0] + radii[1] + 1.2);
        layout.arena_scale = separation / DEFAULT_SEPARATION;
        layout.origins = layout
            .origins
            .map(|p| Vec3::new(p.x * layout.arena_scale, p.y, p.z * layout.arena_scale));
        let mut points = Vec::with_capacity(16);
        for i in 0..2 {
            let Some(body) = bodies[i] else {
                continue;
            };
            let pose = Transform {
                translation: layout.origins[i]
                    - rotations[i] * (Vec3::Y * body.min.y * body.scale.y),
                rotation: rotations[i],
                scale: body.scale,
            };
            layout.body_poses[i] = Some(pose);
            let center = (body.min + body.max) * 0.5;
            layout.hit_anchors[i] = pose.transform_point(Vec3::new(
                center.x,
                body.min.y + body.reference_height * 0.5,
                center.z,
            ));
            points.extend(body.corners(pose));
        }
        if points.is_empty() {
            return layout;
        }
        let rotation = layout.camera.rotation;
        let camera_from_world = rotation.inverse();
        // Center in the fixed camera's axes, then solve every frustum half-plane
        // analytically. A sphere-fit would waste the narrow viewport's width.
        let mut lo = Vec3::splat(f32::INFINITY);
        let mut hi = Vec3::splat(f32::NEG_INFINITY);
        for point in &points {
            let p = camera_from_world * *point;
            lo = lo.min(p);
            hi = hi.max(p);
        }
        let target_view = (lo + hi) * 0.5;
        let target = rotation * target_view;
        let tangent = (CAMERA_FOV * 0.5).tan();
        let horizontal = tangent * (viewport.x / viewport.y) * 0.82;
        // Reserve the HP panels above and command window below. The slightly
        // asymmetric half-planes put every neutral corner in y=.23..75.
        let vertical_up = tangent * 0.54;
        let vertical_down = tangent * 0.50;
        let distance = points.iter().fold(layout.distance, |distance, point| {
            let p = camera_from_world * (*point - target);
            distance.max(
                p.z + (p.x.abs() / horizontal).max(if p.y >= 0.0 {
                    p.y / vertical_up
                } else {
                    -p.y / vertical_down
                }) + 0.15,
            )
        });
        layout.camera = Transform {
            translation: target + rotation * Vec3::Z * distance,
            rotation,
            ..default()
        };
        layout.distance = distance;
        layout.far = 100.0_f32.max(distance + (hi - lo).length() + 30.0);
        layout
    }
    pub(crate) fn origin(&self, index: usize) -> Vec3 {
        self.origins[index]
    }
    pub(crate) fn source_position(&self, center: Vec2) -> Vec3 {
        // The bridge publishes displayed LCD coordinates after OAM (8,16).
        let across = (center.x - 40.0) / 84.0;
        let baseline_y = 72.0 - across * 40.0;
        self.hit_anchors[0].lerp(self.hit_anchors[1], across)
            + Vec3::Y * (baseline_y - center.y) * SOURCE_PIXEL_WORLD
    }
    pub(crate) fn source_displacement(&self, offset: Vec2) -> Vec3 {
        let forward = (self.origins[1] - self.origins[0]).normalize();
        forward * offset.x * SOURCE_PIXEL_WORLD + Vec3::Y * offset.y * SOURCE_PIXEL_WORLD
    }
    pub(crate) fn source_pose(&self, center: Vec2, size: Vec2) -> Transform {
        Transform::from_translation(self.source_position(center))
            .with_rotation(self.camera.rotation)
            .with_scale((size * SOURCE_PIXEL_WORLD).extend(1.0))
    }
    /// One camera-facing similarity for the complete source effect canvas.
    /// Fit in physical pixels before normalizing UV: fitting directly in UV
    /// would squash circles on non-square viewports. A single global mapping
    /// preserves seams even when one source picture spans several OAM slots.
    pub(crate) fn source_projection(&self, viewport: Vec2) -> Mat3 {
        let viewport = viewport.max(Vec2::ONE);
        let source_start = Vec2::new(40.0, 72.0);
        let source_delta = Vec2::new(84.0, -40.0);
        let start = self.project_point(self.hit_anchors[0], viewport) * viewport;
        let end = self.project_point(self.hit_anchors[1], viewport) * viewport;
        let target_delta = end - start;
        let denominator = source_delta.length_squared();
        let a = source_delta.dot(target_delta) / denominator;
        let b = source_delta.perp_dot(target_delta) / denominator;
        let dx = Vec2::new(a, b);
        let dy = Vec2::new(-b, a);
        let origin = start - dx * source_start.x - dy * source_start.y;
        // This keeps both original battler anchors exact. Away from those
        // anchors, source paths retain their original shape rather than the
        // former world-plane shear/perspective distortion.
        Mat3::from_cols((dx / viewport).extend(0.0),
            (dy / viewport).extend(0.0), (origin / viewport).extend(1.0))
    }
    pub(crate) fn project_point(&self, point: Vec3, viewport: Vec2) -> Vec2 {
        let p = self
            .camera
            .compute_matrix()
            .inverse()
            .transform_point3(point);
        let tangent = (CAMERA_FOV * 0.5).tan();
        let aspect = viewport.x / viewport.y;
        Vec2::new(
            0.5 + p.x / (-p.z * 2.0 * tangent * aspect),
            0.5 - p.y / (-p.z * 2.0 * tangent),
        )
    }
}

#[cfg(test)]
mod framing_tests {
    use super::*;
    fn body(species: &str, meters: f32, min: Vec3, max: Vec3) -> BattleBody {
        BattleBody::modeled(
            species,
            min,
            max,
            model_scale(species, Some(meters), min, max).unwrap(),
        )
    }
    fn pair() -> [Option<BattleBody>; 2] {
        [
            Some(body(
                "ONIX",
                8.7884,
                Vec3::new(-0.388217, 0.0, -0.351834),
                Vec3::new(0.388217, 1.72, 0.351834),
            )),
            Some(body(
                "DIGLETT",
                0.2032,
                Vec3::new(-0.489595, 0.0, -0.418663),
                Vec3::new(0.489595, 0.82, 0.418663),
            )),
        ]
    }
    #[test]
    fn complete_giant_and_tiny_envelopes_fit_one_camera_in_both_aspects() {
        for viewport in [
            Vec2::new(1600.0, 900.0),
            Vec2::new(500.0, 1200.0),
            Vec2::new(1200.0, 500.0),
        ] {
            let bodies = pair();
            let layout = BattleSceneLayout::for_bodies(bodies, viewport);
            for i in 0..2 {
                let body = bodies[i].unwrap();
                let pose = layout.body_poses[i].unwrap();
                assert_eq!(
                    pose.scale, body.scale,
                    "camera fit must never resize a participant"
                );
                for point in body.corners(pose) {
                    let uv = layout.project_point(point, viewport);
                    assert!(
                        uv.x >= 0.085 && uv.x <= 0.915 && uv.y >= 0.225 && uv.y <= 0.755,
                        "{viewport:?} {uv:?}"
                    );
                }
            }
        }
    }
    #[test]
    fn source_endpoints_and_midpoints_follow_real_body_anchors() {
        let layout = BattleSceneLayout::for_bodies(pair(), Vec2::new(1200.0, 900.0));
        assert!(
            layout
                .source_position(Vec2::new(40.0, 72.0))
                .distance(layout.hit_anchors[0])
                < 1e-5
        );
        assert!(
            layout
                .source_position(Vec2::new(124.0, 32.0))
                .distance(layout.hit_anchors[1])
                < 1e-5
        );
        assert!(
            layout
                .source_position(Vec2::new(82.0, 52.0))
                .distance(layout.hit_anchors[0].lerp(layout.hit_anchors[1], 0.5))
                < 1e-5
        );
    }
    #[test]
    fn switching_size_or_viewport_changes_layout_but_repeating_does_not() {
        let original = BattleSceneLayout::for_bodies(pair(), Vec2::new(1600.0, 900.0));
        assert_eq!(
            original,
            BattleSceneLayout::for_bodies(pair(), Vec2::new(1600.0, 900.0))
        );
        assert_ne!(
            original,
            BattleSceneLayout::for_bodies(pair(), Vec2::new(500.0, 1200.0))
        );
        let mut switched = pair();
        switched.swap(0, 1);
        assert_ne!(
            original,
            BattleSceneLayout::for_bodies(switched, Vec2::new(1600.0, 900.0))
        );
    }
}

#[cfg(test)]
mod source_registration_tests {
    use super::*;
    #[test]
    fn similarity_matches_fitted_anchors_and_source_shapes_in_both_directions() {
        let min = Vec3::new(-0.388217, 0.0, -0.351834);
        let max = Vec3::new(0.388217, 1.72, 0.351834);
        let onix = BattleBody::modeled(
            "ONIX",
            min,
            max,
            model_scale("ONIX", Some(8.7884), min, max).unwrap(),
        );
        let min = Vec3::new(-0.489595, 0.0, -0.418663);
        let max = Vec3::new(0.489595, 0.82, 0.418663);
        let diglett = BattleBody::modeled(
            "DIGLETT",
            min,
            max,
            model_scale("DIGLETT", Some(0.2032), min, max).unwrap(),
        );
        for bodies in [[Some(onix), Some(diglett)], [Some(diglett), Some(onix)]] {
            for viewport in [Vec2::new(1600.0, 900.0), Vec2::new(500.0, 1200.0)] {
                let layout = BattleSceneLayout::for_bodies(bodies, viewport);
                let projection = layout.source_projection(viewport);
                let start = Vec2::new(40.0, 72.0);
                let end = Vec2::new(124.0, 32.0);
                let map = |pixel: Vec2| {
                    let h = projection * pixel.extend(1.0);
                    h.truncate() / h.z
                };
                assert!((map(start) - layout.project_point(layout.hit_anchors[0], viewport)).length() < 0.00001);
                assert!((map(end) - layout.project_point(layout.hit_anchors[1], viewport)).length() < 0.00001);
                for source in [
                    Vec2::ZERO,
                    Vec2::new(48.0, 88.0),
                    Vec2::new(90.0, 68.0),
                    Vec2::new(132.0, 48.0),
                    Vec2::new(160.0, 95.0),
                ] {
                    let homogeneous = projection * source.extend(1.0);
                    let projected = homogeneous.truncate() / homogeneous.z;
                    let dx = (map(source + Vec2::X * 8.0) - projected) * viewport;
                    let dy = (map(source + Vec2::Y * 8.0) - projected) * viewport;
                    assert!((dx.length() / dy.length() - 1.0).abs() < 0.00005);
                    assert!(dx.normalize().dot(dy.normalize()).abs() < 0.00005);
                    let inverse = projection.inverse() * projected.extend(1.0);
                    assert!((inverse.truncate() / inverse.z).abs_diff_eq(source, 0.005));
                }
            }
        }
    }
    #[test]
    fn cyndaquil_hit_anchor_is_on_body_below_flame_height() {
        let min = Vec3::new(-0.327255, 0.0, -0.536956);
        let max = Vec3::new(0.327255, 0.91, 0.536956);
        let body = BattleBody::modeled(
            "CYNDAQUIL",
            min,
            max,
            model_scale("CYNDAQUIL", Some(0.508), min, max).unwrap(),
        );
        let layout = BattleSceneLayout::for_bodies(
            [Some(body), Some(BattleBody::source_card(1.0))],
            Vec2::new(1600.0, 900.0),
        );
        let pose = layout.body_poses[0].unwrap();
        let local = pose
            .compute_matrix()
            .inverse()
            .transform_point3(layout.hit_anchors[0]);
        assert!((local.y - 0.716729 * 0.5).abs() < 1e-5);
        assert!(local.y < 0.716729 && local.y > 0.0);
    }
}
