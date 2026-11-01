//! Inward geological slopes inside permanent native rock footprints.
use bevy::prelude::Vec2;
use crystal_render_api::{VisualSourceRegion, VisualTile};

use super::GridGeometry;
use super::{cave_cliff, outdoor_mountains};
use crate::profile::{CellShape, MOUNTAIN_CLIFF_HEIGHT, SOURCE_TILE_HEIGHT};

pub(super) struct Field {
    corners: Vec<Vec2>,
}

fn source_location(
    point: Vec2,
    origin: [i32; 2],
    regions: &[VisualSourceRegion],
) -> (Vec2, Option<&str>) {
    let p = point + Vec2::new(origin[0] as f32, origin[1] as f32);
    let owner = regions.iter().min_by(|a, b| {
        let distance = |r: &VisualSourceRegion| {
            let lo = r.origin.as_vec2();
            let hi = lo + r.size.as_vec2();
            p.distance_squared(p.clamp(lo, hi))
        };
        distance(a)
            .total_cmp(&distance(b))
            .then_with(|| a.map_id.cmp(&b.map_id))
    });
    (
        p - owner.map_or(Vec2::ZERO, |r| r.origin.as_vec2()),
        owner.map(|region| region.map_id.as_ref()),
    )
}

pub(super) fn height(
    point: Vec2,
    origin: [i32; 2],
    regions: &[VisualSourceRegion],
    distance: f32,
    tile: f32,
) -> f32 {
    let (source, map) = source_location(point, origin, regions);
    let bed = cave_cliff::slab(
        source.x * 0.15 + source.y * 0.035 + 17.,
        source.y * 0.17 - source.x * 0.04 - 31.,
    );
    let ridge = cave_cliff::slab(source.x * 0.095 - 43., source.y * 0.105 + 23.);
    let toe = 0.32 + bed * 0.12;
    let rise = distance * (1.40 + bed * 0.65);
    // Shape the complete body. Keeping a native two/four-tile shaft beneath
    // this field made even a low summit read as a box with a pointed lid.
    let body = (toe + rise.min(3.2 + ridge.abs() * 3.2 - toe)) * tile;
    if map == Some("Route36") {
        // The narrow ridge lost its native two-cell body in the shared
        // reshaping. Restore that rise inside the footprint, fading it into
        // the seated margin rather than lifting a vertical shaft or rim.
        let inward = (distance / 1.5).clamp(0., 1.);
        let retained_body = MOUNTAIN_CLIFF_HEIGHT / SOURCE_TILE_HEIGHT;
        body + retained_body * tile * inward * inward * (3. - 2. * inward)
    } else {
        body
    }
}

impl Field {
    pub(super) fn new(
        cells: &[&VisualTile],
        shapes: &[CellShape],
        g: &GridGeometry,
        origin: [i32; 2],
        regions: &[VisualSourceRegion],
    ) -> Self {
        let rock = cells
            .iter()
            .zip(shapes)
            .map(|(c, &s)| outdoor_mountains::permanent(&c.source, s))
            .collect::<Vec<_>>();
        let at = |x: i32, z: i32| -> Option<bool> {
            (x >= 0 && z >= 0 && x < g.width as i32 && z < g.height as i32)
                .then(|| rock[z as usize * g.width + x as usize])
        };
        let mut corners = vec![Vec2::ZERO; (g.width + 1) * (g.height + 1)];
        for z in 0..=g.height {
            for x in 0..=g.width {
                if ![[-1, -1], [0, -1], [-1, 0], [0, 0]]
                    .into_iter()
                    .any(|[dx, dz]| at(x as i32 + dx, z as i32 + dz) == Some(true))
                {
                    continue;
                }
                let p = Vec2::new(x as f32, z as f32);
                let (source, _) = source_location(p, origin, regions);
                // Broad linear mineral beds vary the slope along a run. The
                // field stays inside the rock; it never bulges onto the path.
                let width =
                    0.32 + cave_cliff::slab(source.x * 0.17 + 17., source.y * 0.17 - 31.) * 0.08;
                let mut cuts = [0.0_f32; 4];
                for cz in z as i32 - 4..=z as i32 + 4 {
                    for cx in x as i32 - 4..=x as i32 + 4 {
                        if at(cx, cz) != Some(true) {
                            continue;
                        }
                        for (side, dx, dz, a, b) in [
                            (
                                0,
                                -1,
                                0,
                                Vec2::new(cx as f32, cz as f32),
                                Vec2::new(cx as f32, cz as f32 + 1.),
                            ),
                            (
                                1,
                                1,
                                0,
                                Vec2::new(cx as f32 + 1., cz as f32),
                                Vec2::new(cx as f32 + 1., cz as f32 + 1.),
                            ),
                            (
                                2,
                                0,
                                -1,
                                Vec2::new(cx as f32, cz as f32),
                                Vec2::new(cx as f32 + 1., cz as f32),
                            ),
                            (
                                3,
                                0,
                                1,
                                Vec2::new(cx as f32, cz as f32 + 1.),
                                Vec2::new(cx as f32 + 1., cz as f32 + 1.),
                            ),
                        ] {
                            // A cropped source window is not a geological edge.
                            if at(cx + dx, cz + dz) != Some(false) {
                                continue;
                            }
                            let outward = Vec2::new(dx as f32, dz as f32);
                            if (p - a).dot(-outward) < -0.0001 {
                                continue;
                            }
                            let nearest = a
                                + (b - a)
                                    * ((p - a).dot(b - a) / (b - a).length_squared()).clamp(0., 1.);
                            let t = (1. - p.distance(nearest) / 4.).clamp(0., 1.);
                            cuts[side] = cuts[side].max(width * t * t * (3. - 2. * t));
                        }
                    }
                }
                corners[z * (g.width + 1) + x] = Vec2::new(cuts[0] - cuts[1], cuts[2] - cuts[3]);
            }
        }
        Self { corners }
    }

    pub(super) fn inset(&self, point: Vec2, g: &GridGeometry) -> Vec2 {
        if point.x < 0. || point.y < 0. || point.x > g.width as f32 || point.y > g.height as f32 {
            return Vec2::ZERO;
        }
        let x = (point.x.floor() as usize).min(g.width - 1);
        let z = (point.y.floor() as usize).min(g.height - 1);
        let i = z * (g.width + 1) + x;
        let (u, v) = (point.x - x as f32, point.y - z as f32);
        self.corners[i].lerp(self.corners[i + 1], u).lerp(
            self.corners[i + g.width + 1].lerp(self.corners[i + g.width + 2], u),
            v,
        )
    }
}

pub(super) fn slope(fraction: f32) -> f32 {
    // One planar flank avoids universal horizontal bends at fixed fractions
    // of every cliff's height. The existing lower mineral weathering remains.
    fraction
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::shape_for_source;
    use bevy::prelude::{IVec2, UVec2};
    use crystal_render_api::VisualWorldFrame;

    fn fixture() -> (VisualWorldFrame, GridGeometry) {
        let mut source = super::super::tests::source_with_tile(0x0a, 1, 1, 0x3c);
        source.tileset_id = "johto".into();
        let flat = super::super::tests::source_with_tile(1, 0, 0, 5);
        let mut sources = vec![flat; 144];
        for z in 2..10 {
            for x in 2..10 {
                sources[z * 12 + x] = source.clone();
            }
        }
        (
            super::super::tests::frame(12, 12, sources),
            GridGeometry {
                width: 12,
                height: 12,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: 0.,
                origin_z: 0.,
            },
        )
    }

    #[test]
    fn upper_slopes_recede_inside_the_body_without_crossing_or_collapsing_caps() {
        let (frame, g) = fixture();
        let cells = frame.tiles.iter().collect::<Vec<_>>();
        let shapes = cells
            .iter()
            .map(|c| shape_for_source(&c.source))
            .collect::<Vec<_>>();
        let field = Field::new(&cells, &shapes, &g, [0, 0], &[]);
        assert!((0.2..0.4).contains(&field.inset(Vec2::new(2., 6.), &g).x));
        assert!((-0.4..-0.2).contains(&field.inset(Vec2::new(10., 6.), &g).x));
        assert_eq!(slope(0.), 0.);
        assert!((slope(1.) - 1.).abs() < 0.00001);
        for z in 0..48 {
            for x in 0..48 {
                let p = Vec2::new(2. + x as f32 / 6., 2. + z as f32 / 6.);
                let d = Vec2::splat(1. / 6.);
                let q = [p, p + Vec2::new(0., d.y), p + d, p + Vec2::new(d.x, 0.)]
                    .map(|p| p + field.inset(p, &g));
                assert!(
                    q.iter()
                        .all(|p| p.cmpge(Vec2::splat(2.)).all() && p.cmple(Vec2::splat(10.)).all())
                );
                assert!((q[1] - q[0]).perp_dot(q[2] - q[0]) < -0.00001);
                assert!((q[2] - q[0]).perp_dot(q[3] - q[0]) < -0.00001);
            }
        }
    }

    #[test]
    fn whole_bodies_rise_inward_with_unequal_beds_and_a_moderate_height_bound() {
        let mut low = f32::INFINITY;
        let mut high = f32::NEG_INFINITY;
        for x in 0..=96 {
            let point = Vec2::new(x as f32, 23.);
            let profile = [0., 1., 2., 4., 6.].map(|d| height(point, [0, 0], &[], d, 8.));
            assert!(profile[0] > 0. && profile[0] < 8. * 0.4);
            assert!(profile.windows(2).all(|p| p[1] >= p[0]));
            assert!(profile[2] > profile[0] + 8.);
            assert!(profile[4] <= 8. * 4.8 + 0.001);
            low = low.min(profile[1]);
            high = high.max(profile[1]);
        }
        assert!(
            high - low > 8. * 0.15,
            "one long course must not have a uniform geological shoulder"
        );
    }

    #[test]
    fn source_owned_slopes_survive_connected_map_handoffs_and_paint_bases() {
        let (frame, g) = fixture();
        let cells = frame.tiles.iter().collect::<Vec<_>>();
        let shapes = cells
            .iter()
            .map(|c| shape_for_source(&c.source))
            .collect::<Vec<_>>();
        let field = Field::new(&cells, &shapes, &g, [0, 0], &[]);
        let shift = IVec2::new(-40, 8);
        let regions = [crystal_render_api::VisualSourceRegion {
            map_id: "Route44".into(),
            origin: shift,
            size: UVec2::splat(12),
        }];
        let connected = Field::new(&cells, &shapes, &g, shift.to_array(), &regions);
        assert_eq!(field.corners, connected.corners);
        for point in [Vec2::new(2., 6.), Vec2::new(6., 6.), Vec2::new(10., 8.)] {
            for distance in [0., 1., 2., 4., 6.] {
                assert_eq!(
                    height(point, [0, 0], &[], distance, 8.),
                    height(point, shift.to_array(), &regions, distance, 8.)
                );
            }
        }
    }

    #[test]
    fn route36_retains_a_rising_ridge_through_connected_views() {
        let shift = IVec2::new(-120, 0);
        let regions = [
            VisualSourceRegion {
                map_id: "Route36".into(),
                origin: shift,
                size: UVec2::new(120, 36),
            },
            VisualSourceRegion {
                map_id: "VioletCity".into(),
                origin: IVec2::ZERO,
                size: UVec2::new(80, 72),
            },
        ];
        let local_region = [crystal_render_api::VisualSourceRegion {
            origin: IVec2::ZERO,
            ..regions[0].clone()
        }];
        for tile in [8., 32.] {
            for x in 100..=116 {
                let point = Vec2::new(x as f32, 34.);
                let profile = [0., 0.5, 1., 2.]
                    .map(|distance| height(point, [0, 0], &local_region, distance, tile));
                assert!(profile[0] < tile * 0.4, "retain the low seated margin");
                assert!(profile.windows(2).all(|p| p[1] > p[0]));
                assert!(
                    profile[3] > tile * 4.,
                    "the narrow ridge needs mountain height"
                );
                for distance in [0., 0.5, 1., 2., 6.] {
                    assert_eq!(
                        height(point, [0, 0], &local_region, distance, tile),
                        height(point, shift.to_array(), &regions, distance, tile),
                        "Violet's connected view must retain Route 36's mountain"
                    );
                }
            }
            for distance in [0., 0.5, 1., 2., 6.] {
                let violet = Vec2::new(8., 34.);
                assert_eq!(
                    height(violet, [0, 0], &regions, distance, tile),
                    height(violet, [0, 0], &[], distance, tile),
                    "neighboring Violet scenery keeps its existing height"
                );
            }
        }
    }
}
