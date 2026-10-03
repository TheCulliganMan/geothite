//! One camera-facing similarity for every source OAM pixel, independent of how
//! those pixels are partitioned into objects. The rectangle is raster coverage
//! only: the native overlay inverse-maps each fragment through this matrix.
use bevy::prelude::*;
use crate::BattleSceneLayout;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BattleSourceProjection {
    /// Original LCD pixels (Y down) to normalized viewport coordinates.
    pub source_to_view: Mat3,
    pub view_to_source: Mat3,
}

impl BattleSourceProjection {
    pub fn new(layout: &BattleSceneLayout, viewport: Vec2) -> Option<Self> {
        if !viewport.is_finite() || viewport.min_element() <= 0.0 {
            return None;
        }
        let source_to_view = layout.source_projection(viewport);
        if !source_to_view.is_finite() || source_to_view.determinant().abs() < 1e-8 {
            return None;
        }
        let view_to_source = source_to_view.inverse();
        view_to_source.is_finite().then_some(Self { source_to_view, view_to_source })
    }

    pub fn project(&self, pixel: Vec2) -> Option<Vec2> {
        divide(self.source_to_view * pixel.extend(1.0))
    }

    pub fn unproject(&self, uv: Vec2) -> Option<Vec2> {
        divide(self.view_to_source * uv.extend(1.0))
    }

    /// Bounds of the projected four corners, in normalized viewport units.
    /// Never use this rectangle as a stretched sprite's texture coordinates.
    pub fn coverage(&self, source: Rect) -> Option<Rect> {
        if !source.min.is_finite() || !source.max.is_finite()
            || source.size().min_element() <= 0.0 {
            return None;
        }
        let corners = [source.min, Vec2::new(source.max.x, source.min.y),
            source.max, Vec2::new(source.min.x, source.max.y)];
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        for corner in corners {
            // Positive depths at all vertices also exclude a projective pole
            // anywhere inside this convex source rectangle.
            let point = self.project(corner)?;
            min = min.min(point);
            max = max.max(point);
        }
        Some(Rect::from_corners(min, max))
    }

    /// Expand to physical pixel edges so separate slot coverage cannot drop a
    /// shared boundary through rounded mesh transforms. The shader still owns
    /// exact half-open source clipping and never stretches this coverage box.
    pub fn raster_coverage(&self, source: Rect, physical_size: UVec2) -> Option<Rect> {
        if physical_size.min_element() == 0 { return None; }
        let bounds = self.coverage(source)?;
        let size = physical_size.as_vec2();
        let min = (bounds.min * size).floor().max(Vec2::ZERO) / size;
        let max = (bounds.max * size).ceil().min(size) / size;
        (max.cmpgt(min).all()).then_some(Rect::from_corners(min, max))
    }
}

fn divide(point: Vec3) -> Option<Vec2> {
    (point.is_finite() && point.z > 0.00001).then(|| point.truncate() / point.z)
}

#[cfg(test)]
mod tests {
    use super::*;

    // CPU reference for the WGSL's global source-pixel lookup. Individual
    // object UVs cannot decide which LCD pixel owns a screen fragment.
    fn texel(projection: BattleSourceProjection, screen: Vec2, rect: Rect,
        crop: Rect, image: UVec2) -> Option<UVec2> {
        let source = projection.unproject(screen)?;
        if source.cmplt(rect.min).any() || source.cmpge(rect.max).any() { return None; }
        let center = source.floor() + Vec2::splat(0.5);
        let uv = crop.min + (center - rect.min) / rect.size() * crop.size();
        let pixel = (uv * image.as_vec2()).floor();
        if pixel.cmplt(Vec2::ZERO).any() || pixel.cmpge(image.as_vec2()).any() { return None; }
        Some(pixel.as_uvec2())
    }

    fn clipped(original: Rect) -> Option<(Rect, Rect)> {
        let min = original.min.max(Vec2::ZERO);
        let max = original.max.min(Vec2::new(160.0, 144.0));
        if max.cmple(min).any() { return None; }
        Some((Rect::from_corners(min, max), Rect::from_corners(
            (min - original.min) / original.size(), (max - original.min) / original.size())))
    }

    fn pattern(pixel: UVec2) -> Option<u32> {
        // Asymmetric, independently identifiable texels, including holes.
        ((pixel.x + 2 * pixel.y) % 7 != 0).then_some(1 + pixel.x + 24 * pixel.y)
    }

    fn layouts(viewport: Vec2) -> [BattleSceneLayout; 3] {
        use crate::battle_layout::BattleBody;
        let small = BattleBody::modeled("DIGLETT", Vec3::ZERO,
            Vec3::new(0.5, 0.8, 0.5), 0.7);
        let large = BattleBody::modeled("ONIX", Vec3::new(-0.4, 0.0, -0.35),
            Vec3::new(0.4, 1.72, 0.35), 7.0);
        [BattleSceneLayout::default(),
            BattleSceneLayout::for_bodies([Some(small), Some(large)], viewport),
            BattleSceneLayout::for_bodies([Some(large), Some(small)], viewport)]
    }

    #[test]
    fn projected_oam_partition_is_identical_for_all_pixels_and_clipped_edges() {
        for physical in [UVec2::new(800, 600), UVec2::new(1600, 900), UVec2::new(600, 900)] {
            for layout in layouts(physical.as_vec2()) {
                let projection = BattleSourceProjection::new(&layout, physical.as_vec2()).unwrap();
                for origin in [Vec2::new(112.0, 8.0), Vec2::new(29.0, 59.0),
                    Vec2::new(-5.0, -3.0), Vec2::new(149.0, 133.0)] {
                    let full = Rect::from_corners(origin, origin + Vec2::splat(24.0));
                    let (full_rect, full_crop) = clipped(full).unwrap();
                    let Some(coverage) = projection.raster_coverage(full_rect, physical) else {
                        // A narrow viewport can exclude the whole projected
                        // pattern. Its pieces must be equally absent.
                        for ty in 0..3 { for tx in 0..3 {
                            let offset = Vec2::new(tx as f32, ty as f32) * 8.0;
                            if let Some((rect, _)) = clipped(Rect::from_corners(origin + offset, origin + offset + Vec2::splat(8.0))) {
                                assert!(projection.raster_coverage(rect, physical).is_none());
                            }
                        } }
                        continue;
                    };
                    let mut covered = 0;
                    for y in (coverage.min.y * physical.y as f32) as u32..(coverage.max.y * physical.y as f32).ceil() as u32 {
                        for x in (coverage.min.x * physical.x as f32) as u32..(coverage.max.x * physical.x as f32).ceil() as u32 {
                            let screen = (Vec2::new(x as f32, y as f32) + Vec2::splat(0.5)) / physical.as_vec2();
                            let composite = texel(projection, screen, full_rect, full_crop, UVec2::splat(24)).and_then(pattern);
                            let mut pieces = None;
                            let mut owners = 0;
                            for ty in 0..3 { for tx in 0..3 {
                                let offset = Vec2::new(tx as f32, ty as f32) * 8.0;
                                let tile = Rect::from_corners(origin + offset, origin + offset + Vec2::splat(8.0));
                                let Some((rect, crop)) = clipped(tile) else { continue; };
                                if let Some(pixel) = texel(projection, screen, rect, crop, UVec2::splat(8)) {
                                    owners += 1;
                                    let bounds = projection.raster_coverage(rect, physical).unwrap();
                                    assert!(screen.cmpge(bounds.min).all() && screen.cmplt(bounds.max).all());
                                    pieces = pattern(pixel + UVec2::new(tx * 8, ty * 8));
                                }
                            } }
                            assert!(owners <= 1, "half-open source edges have one owner");
                            assert_eq!(composite, pieces, "partition changed screen pixel {x},{y} in {physical:?} at {origin:?}");
                            covered += usize::from(composite.is_some());
                        }
                    }
                    assert!(covered > 0);
                }
            }
        }
    }

    #[test]
    fn projected_oam_preserves_circle_aspect_and_anchor_trajectories() {
        for viewport in [Vec2::new(800.0, 600.0), Vec2::new(1600.0, 900.0),
            Vec2::new(600.0, 900.0)] {
            for layout in layouts(viewport) {
                let projection = BattleSourceProjection::new(&layout, viewport).unwrap();
                let physical = |p| projection.project(p).unwrap() * viewport;
                let source = [Vec2::new(40.0, 72.0), Vec2::new(124.0, 32.0)];
                let targets = layout.hit_anchors.map(|p| layout.project_point(p, viewport) * viewport);
                for side in 0..2 {
                    assert!((physical(source[side]) - targets[side]).length() < 0.0002);
                    // The same mapping preserves the original trajectory in
                    // either direction, with no side-specific sprite scale.
                    for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
                        assert!((physical(source[side].lerp(source[1-side], t))
                            - targets[side].lerp(targets[1-side], t)).length() < 0.0002);
                    }
                }
                for center in [Vec2::new(16.0, 120.0), source[0], source[1], Vec2::new(150.0, 8.0)] {
                    let x = physical(center + Vec2::X * 8.0) - physical(center);
                    let y = physical(center + Vec2::Y * 8.0) - physical(center);
                    assert!((x.length() / y.length() - 1.0).abs() < 0.00003);
                    assert!(x.normalize().dot(y.normalize()).abs() < 0.00003);
                    // Actual round source marks must remain round for every
                    // sampled angle, not just match an axis-aligned bbox.
                    for angle in 0..32 {
                        let radians = std::f32::consts::TAU * angle as f32 / 32.0;
                        let point = center + Vec2::from_angle(radians) * 8.0;
                        assert!(((physical(point) - physical(center)).length() / x.length() - 1.0).abs() < 0.00003);
                        assert!((projection.unproject(projection.project(point).unwrap()).unwrap() - point).length() < 0.0002);
                    }
                }
                // Corners, centers and texture sampling all share this same
                // global affine map; no independent object rectangle stretch.
                let origin = Vec2::new(112.0, 8.0);
                let a = physical(origin);
                let b = physical(origin + Vec2::splat(24.0));
                assert!((physical(origin + Vec2::splat(12.0)) - (a+b)*0.5).length() < 0.0002);
            }
        }
        let layout = BattleSceneLayout::default();
        assert!(BattleSourceProjection::new(&layout, Vec2::ZERO).is_none());
        assert!(BattleSourceProjection::new(&layout, Vec2::splat(f32::NAN)).is_none());
        let mut coincident = layout;
        coincident.hit_anchors[1] = coincident.hit_anchors[0];
        assert!(BattleSourceProjection::new(&coincident, Vec2::new(800.0,600.0)).is_none());
    }
}
