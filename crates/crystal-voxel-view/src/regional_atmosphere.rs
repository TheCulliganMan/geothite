//! Continuous outdoor lighting in authored source coordinates.
use bevy::color::ColorToComponents;
use bevy::prelude::{Color, Vec2, Vec3};
use crystal_render_api::{VisualActorId, VisualWorldFrame};

// Eight gameplay cells on either side of a connected edge. Source rectangles,
// rather than the active map label or elapsed time, own the transition.
const BLEND_RADIUS: f32 = 16.0;

#[derive(Clone, Copy)]
pub(super) struct RegionalAtmosphere {
    pub sun_tint: Color,
    pub haze_scale: f32,
    pub fog_start: f32,
    pub fog_end: f32,
}

fn for_map(map: &str) -> RegionalAtmosphere {
    let sun_tint = if matches!(map, "Route29" | "NewBarkTown") {
        Color::srgb(1.0, 0.94, 0.82)
    } else if crate::environment_fx::coastal_map(map) {
        Color::srgb(0.89, 0.96, 1.0)
    } else {
        Color::WHITE
    };
    let (haze_scale, fog_start, fog_end) = if map.starts_with("Route") {
        (0.12, 16.0, 40.0)
    } else {
        (1.0, 8.0, 22.0)
    };
    RegionalAtmosphere {
        sun_tint,
        haze_scale,
        fog_start,
        fog_end,
    }
}

fn source_point(frame: &VisualWorldFrame) -> Option<Vec2> {
    if !frame.tile_size.is_finite() || frame.tile_size.cmple(Vec2::ZERO).any() {
        return None;
    }
    let point = frame
        .actors
        .iter()
        .find(|actor| actor.id == VisualActorId::Player)
        .map_or(frame.view_center, |actor| {
            crate::footing::actor_ground_point(frame, actor)
        });
    let local = (point - frame.center) / frame.tile_size * Vec2::new(1.0, -1.0);
    let point = frame.grid_origin.as_vec2() + frame.grid_size.as_vec2() * 0.5 + local;
    point.is_finite().then_some(point)
}

pub(super) fn sample(frame: &VisualWorldFrame) -> Option<RegionalAtmosphere> {
    // Enclosed lighting and Ilex's authored night palette retain their own
    // treatment. Doors do not create spatial connections to another biome.
    if !crate::sky::outdoors(frame) || frame.map_id.as_ref() == "IlexForest" {
        return None;
    }
    let fallback = for_map(&frame.map_id);
    let Some(point) = source_point(frame) else {
        return Some(fallback);
    };
    Some(sample_at(frame, point).unwrap_or(fallback))
}

fn sample_at(frame: &VisualWorldFrame, point: Vec2) -> Option<RegionalAtmosphere> {
    let mut total = 0.0;
    let mut sun = Vec3::ZERO;
    let (mut haze, mut start, mut end) = (0.0, 0.0, 0.0);
    for region in frame.source_regions.iter() {
        if region.size.x == 0 || region.size.y == 0 {
            continue;
        }
        let min = region.origin.as_vec2();
        let max = min + region.size.as_vec2();
        let distance = point.distance(point.clamp(min, max));
        if distance >= BLEND_RADIUS {
            continue;
        }
        let t = 1.0 - distance / BLEND_RADIUS;
        let weight = t * t * (3.0 - 2.0 * t);
        let style = for_map(&region.map_id);
        sun += style.sun_tint.to_linear().to_vec4().truncate() * weight;
        haze += style.haze_scale * weight;
        start += style.fog_start * weight;
        end += style.fog_end * weight;
        total += weight;
    }
    if total <= 0.0 {
        return None;
    }
    sun /= total;
    Some(RegionalAtmosphere {
        sun_tint: Color::linear_rgb(sun.x, sun.y, sun.z),
        haze_scale: haze / total,
        fog_start: start / total,
        fog_end: end / total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{IVec2, UVec2};
    use crystal_render_api::VisualSourceRegion;

    fn frame(offset: IVec2) -> VisualWorldFrame {
        VisualWorldFrame {
            map_id: "Route29".into(),
            source_regions: vec![
                VisualSourceRegion {
                    map_id: "Route29".into(),
                    origin: IVec2::ZERO,
                    size: UVec2::splat(40),
                },
                VisualSourceRegion {
                    map_id: "CherrygroveCity".into(),
                    origin: offset,
                    size: UVec2::splat(40),
                },
            ]
            .into(),
            ..Default::default()
        }
    }

    fn assert_same(a: RegionalAtmosphere, b: RegionalAtmosphere) {
        assert!(
            a.sun_tint
                .to_linear()
                .to_vec4()
                .abs_diff_eq(b.sun_tint.to_linear().to_vec4(), 0.000001)
        );
        for (a, b) in [
            (a.haze_scale, b.haze_scale),
            (a.fog_start, b.fog_start),
            (a.fog_end, b.fog_end),
        ] {
            assert!(
                (a - b).abs() < 0.00001,
                "same source point must keep the same atmosphere: {a} vs {b}"
            );
        }
    }

    #[test]
    fn reciprocal_handoffs_preserve_lighting_and_fog_in_all_directions() {
        for offset in [
            IVec2::new(40, 0),
            IVec2::new(-40, 0),
            IVec2::new(0, 40),
            IVec2::new(0, -40),
        ] {
            let before = frame(offset);
            let mut after = before.clone();
            after.map_id = "CherrygroveCity".into();
            after.source_regions = before
                .source_regions
                .iter()
                .rev()
                .map(|region| VisualSourceRegion {
                    map_id: region.map_id.clone(),
                    origin: region.origin - offset,
                    size: region.size,
                })
                .collect::<Vec<_>>()
                .into();
            assert_eq!(before.connected_source_shift(&after), Some(-offset));
            let center = Vec2::splat(20.0) + offset.as_vec2() * 0.5;
            for fraction in [-1.25, -0.25, 0.0, 0.25, 1.25] {
                let point = center + offset.as_vec2().normalize() * fraction;
                assert_same(
                    sample_at(&before, point).expect("source biome"),
                    sample_at(&after, point - offset.as_vec2()).expect("destination biome"),
                );
            }
        }
    }

    #[test]
    fn authored_styles_remain_exact_away_from_the_connected_edge() {
        let frame = frame(IVec2::new(40, 0));
        assert_same(
            sample_at(&frame, Vec2::splat(20.0)).expect("route"),
            for_map("Route29"),
        );
        assert_same(
            sample_at(&frame, Vec2::new(60.0, 20.0)).expect("town"),
            for_map("CherrygroveCity"),
        );
    }

    #[test]
    fn fractional_walks_blend_without_a_map_label_step() {
        let frame = frame(IVec2::new(40, 0));
        let before = sample_at(&frame, Vec2::new(39.999, 20.0)).expect("before edge");
        let after = sample_at(&frame, Vec2::new(40.001, 20.0)).expect("after edge");
        assert_same(before, after);
        let middle = sample_at(&frame, Vec2::new(40.0, 20.0)).expect("edge");
        assert!((middle.haze_scale - 0.56).abs() < 0.000001);
        assert!((middle.fog_start - 12.0).abs() < 0.000001);
    }

    #[test]
    fn source_point_keeps_fractional_motion_and_rebased_grid_coordinates() {
        let mut frame = frame(IVec2::new(40, 0));
        frame.tile_size = Vec2::splat(32.0);
        frame.grid_size = UVec2::splat(64);
        frame.view_center = Vec2::new(7.75, 12.25) * frame.tile_size * Vec2::new(1.0, -1.0);
        let before = source_point(&frame).expect("source point");
        frame.grid_origin = IVec2::new(8, 12);
        frame.center = frame.grid_origin.as_vec2() * frame.tile_size * Vec2::new(1.0, -1.0);
        assert!(
            source_point(&frame)
                .expect("rebased point")
                .abs_diff_eq(before, 0.000001)
        );
        assert_eq!(before, Vec2::new(39.75, 44.25));
    }
}
