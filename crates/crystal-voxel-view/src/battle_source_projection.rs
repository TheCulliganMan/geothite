//! One camera-facing similarity for every source OAM pixel, independent of how
//! those pixels are partitioned into objects. The rectangle is raster coverage
//! only: the native overlay inverse-maps each fragment through this matrix.
use crate::BattleSceneLayout;
use bevy::prelude::*;
use crystal_render_api::VisualBattleSourcePlacement;

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
        view_to_source.is_finite().then_some(Self {
            source_to_view,
            view_to_source,
        })
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
        if !source.min.is_finite() || !source.max.is_finite() || source.size().min_element() <= 0.0
        {
            return None;
        }
        let corners = [
            source.min,
            Vec2::new(source.max.x, source.min.y),
            source.max,
            Vec2::new(source.min.x, source.max.y),
        ];
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
        if physical_size.min_element() == 0 {
            return None;
        }
        let bounds = self.coverage(source)?;
        let size = physical_size.as_vec2();
        let min = (bounds.min * size).floor().max(Vec2::ZERO) / size;
        let max = (bounds.max * size).ceil().min(size) / size;
        (max.cmpgt(min).all()).then_some(Rect::from_corners(min, max))
    }
}

/// A neutral body registration. Source bounds and allocation remain in the
/// original LCD, while the view footprint is measured before animation offsets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BattleSourceBodyRegistration {
    /// Original opaque source pixels, not the padded sprite allocation.
    pub source: Rect,
    /// Original full source sprite allocation, used only for transport bounds.
    pub slot: Rect,
    /// Normalized viewport footprint of the neutral body.
    pub view: Rect,
}

/// Registrations are usable only with the camera and physical viewport that
/// produced them. Their absence leaves the original global projection intact.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct BattleSourceBodyRegistrations {
    pub bodies: [Option<BattleSourceBodyRegistration>; 2],
    pub viewport: Vec2,
    pub camera: Transform,
}

impl BattleSourceBodyRegistrations {
    /// Resolve only source-proven placement descriptors. Every pixel belonging
    /// to an independent assembly must use this same matrix and its inverse;
    /// clipping individual OAM objects must never change the supplied pivot.
    pub fn projection(
        &self,
        placement: VisualBattleSourcePlacement,
        layout: &BattleSceneLayout,
        viewport: Vec2,
    ) -> Option<BattleSourceProjection> {
        if !viewport.is_finite()
            || viewport.min_element() <= 0.0
            || self.viewport != viewport
            || self.camera != layout.camera
            || !self.camera.translation.is_finite()
            || !self.camera.rotation.is_finite()
            || !self.camera.rotation.is_normalized()
            || !self.camera.scale.is_finite()
        {
            return None;
        }
        let camera = self.camera.compute_matrix();
        let determinant = camera.determinant();
        if !camera.is_finite() || !determinant.is_finite() || determinant == 0.0 {
            return None;
        }
        match placement {
            VisualBattleSourcePlacement::BattlerLocal { side } => {
                self.bodies[side.index()]?.projection(viewport)
            }
            VisualBattleSourcePlacement::IndependentTransport { from, pivot, .. } => {
                if !pivot.is_finite() {
                    return None;
                }
                let attacker = self.bodies[from.index()]?;
                let target = self.bodies[from.opposite().index()]?;
                let start = attacker.projection(viewport)?;
                let end = target.projection(viewport)?;
                // Use the original allocation edges, not the opaque bounds or
                // cue progress. Inside either slot its body chart is exact.
                let (departure, arrival) = if attacker.slot.max.x < target.slot.min.x {
                    (attacker.slot.max.x, target.slot.min.x)
                } else if attacker.slot.min.x > target.slot.max.x {
                    (attacker.slot.min.x, target.slot.max.x)
                } else {
                    // Overlapping/touching slots have no nonzero travel gap.
                    return None;
                };
                let gap = arrival - departure;
                if !gap.is_finite() {
                    return None;
                }
                let weight = ((pivot.x - departure) / gap).clamp(0.0, 1.0);
                if !weight.is_finite() {
                    return None;
                }
                // Both charts are upright similarities in physical pixels,
                // so this convex blend keeps one positive isotropic scale.
                // Blending translations preserves distinct source contacts;
                // the pivot selects a chart, never a replacement destination.
                let source_to_view =
                    start.source_to_view * (1.0 - weight) + end.source_to_view * weight;
                body_projection(source_to_view)
            }
        }
    }
}

impl BattleSourceBodyRegistration {
    fn projection(self, viewport: Vec2) -> Option<BattleSourceProjection> {
        if ![self.source, self.slot, self.view]
            .into_iter()
            .all(valid_body_rect)
        {
            return None;
        }
        let scale = self.view.height() * viewport.y / self.source.height();
        if !scale.is_finite() || scale <= 0.0 {
            return None;
        }
        let source_bottom = Vec2::new(self.source.center().x, self.source.max.y);
        let view_bottom = Vec2::new(self.view.center().x, self.view.max.y) * viewport;
        let origin = view_bottom - source_bottom * scale;
        // LCD and viewport coordinates both point down. Positive axis scales
        // keep source screen-up upright without rotating or stretching anatomy.
        body_projection(Mat3::from_cols(
            Vec3::new(scale / viewport.x, 0.0, 0.0),
            Vec3::new(0.0, scale / viewport.y, 0.0),
            (origin / viewport).extend(1.0),
        ))
    }
}

fn valid_body_rect(rect: Rect) -> bool {
    rect.min.is_finite()
        && rect.max.is_finite()
        && rect.size().is_finite()
        && rect.size().min_element() > 0.0
}

fn body_projection(source_to_view: Mat3) -> Option<BattleSourceProjection> {
    let determinant = source_to_view.determinant();
    if !source_to_view.is_finite() || !determinant.is_finite() || determinant <= 0.0 {
        return None;
    }
    let view_to_source = source_to_view.inverse();
    view_to_source
        .is_finite()
        .then_some(BattleSourceProjection {
            source_to_view,
            view_to_source,
        })
}

fn divide(point: Vec3) -> Option<Vec2> {
    (point.is_finite() && point.z > 0.00001).then(|| point.truncate() / point.z)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crystal_render_api::{VisualBattleSide, VisualBattleSourceAssembly};

    // CPU reference for the WGSL's global source-pixel lookup. Individual
    // object UVs cannot decide which LCD pixel owns a screen fragment.
    fn texel(
        projection: BattleSourceProjection,
        screen: Vec2,
        rect: Rect,
        crop: Rect,
        image: UVec2,
    ) -> Option<UVec2> {
        let source = projection.unproject(screen)?;
        if source.cmplt(rect.min).any() || source.cmpge(rect.max).any() {
            return None;
        }
        let center = source.floor() + Vec2::splat(0.5);
        let uv = crop.min + (center - rect.min) / rect.size() * crop.size();
        let pixel = (uv * image.as_vec2()).floor();
        if pixel.cmplt(Vec2::ZERO).any() || pixel.cmpge(image.as_vec2()).any() {
            return None;
        }
        Some(pixel.as_uvec2())
    }

    fn clipped(original: Rect) -> Option<(Rect, Rect)> {
        let min = original.min.max(Vec2::ZERO);
        let max = original.max.min(Vec2::new(160.0, 144.0));
        if max.cmple(min).any() {
            return None;
        }
        Some((
            Rect::from_corners(min, max),
            Rect::from_corners(
                (min - original.min) / original.size(),
                (max - original.min) / original.size(),
            ),
        ))
    }

    fn pattern(pixel: UVec2) -> Option<u32> {
        // Asymmetric, independently identifiable texels, including holes.
        ((pixel.x + 2 * pixel.y) % 7 != 0).then_some(1 + pixel.x + 24 * pixel.y)
    }

    fn layouts(viewport: Vec2) -> [BattleSceneLayout; 3] {
        use crate::battle_layout::BattleBody;
        let small = BattleBody::modeled("DIGLETT", Vec3::ZERO, Vec3::new(0.5, 0.8, 0.5), 0.7);
        let large = BattleBody::modeled(
            "ONIX",
            Vec3::new(-0.4, 0.0, -0.35),
            Vec3::new(0.4, 1.72, 0.35),
            7.0,
        );
        [
            BattleSceneLayout::default(),
            BattleSceneLayout::for_bodies([Some(small), Some(large)], viewport),
            BattleSceneLayout::for_bodies([Some(large), Some(small)], viewport),
        ]
    }

    fn assert_oam_partition(projection: BattleSourceProjection, physical: UVec2, origin: Vec2) {
        let full = Rect::from_corners(origin, origin + Vec2::splat(24.0));
        let (full_rect, full_crop) = clipped(full).unwrap();
        let Some(coverage) = projection.raster_coverage(full_rect, physical) else {
            // A narrow viewport can exclude the whole projected
            // pattern. Its pieces must be equally absent.
            for ty in 0..3 {
                for tx in 0..3 {
                    let offset = Vec2::new(tx as f32, ty as f32) * 8.0;
                    if let Some((rect, _)) = clipped(Rect::from_corners(
                        origin + offset,
                        origin + offset + Vec2::splat(8.0),
                    )) {
                        assert!(projection.raster_coverage(rect, physical).is_none());
                    }
                }
            }
            return;
        };
        let mut covered = 0;
        for y in (coverage.min.y * physical.y as f32) as u32
            ..(coverage.max.y * physical.y as f32).ceil() as u32
        {
            for x in (coverage.min.x * physical.x as f32) as u32
                ..(coverage.max.x * physical.x as f32).ceil() as u32
            {
                let screen =
                    (Vec2::new(x as f32, y as f32) + Vec2::splat(0.5)) / physical.as_vec2();
                let composite = texel(projection, screen, full_rect, full_crop, UVec2::splat(24))
                    .and_then(pattern);
                let mut pieces = None;
                let mut owners = 0;
                for ty in 0..3 {
                    for tx in 0..3 {
                        let offset = Vec2::new(tx as f32, ty as f32) * 8.0;
                        let tile =
                            Rect::from_corners(origin + offset, origin + offset + Vec2::splat(8.0));
                        let Some((rect, crop)) = clipped(tile) else {
                            continue;
                        };
                        if let Some(pixel) = texel(projection, screen, rect, crop, UVec2::splat(8))
                        {
                            owners += 1;
                            let bounds = projection.raster_coverage(rect, physical).unwrap();
                            assert!(
                                screen.cmpge(bounds.min).all() && screen.cmplt(bounds.max).all()
                            );
                            pieces = pattern(pixel + UVec2::new(tx * 8, ty * 8));
                        }
                    }
                }
                assert!(owners <= 1, "half-open source edges have one owner");
                assert_eq!(
                    composite, pieces,
                    "partition changed screen pixel {x},{y} in {physical:?} at {origin:?}"
                );
                covered += usize::from(composite.is_some());
            }
        }
        assert!(covered > 0);
    }

    #[test]
    fn projected_oam_partition_is_identical_for_all_pixels_and_clipped_edges() {
        for physical in [
            UVec2::new(800, 600),
            UVec2::new(1600, 900),
            UVec2::new(600, 900),
        ] {
            for layout in layouts(physical.as_vec2()) {
                let projection = BattleSourceProjection::new(&layout, physical.as_vec2()).unwrap();
                for origin in [
                    Vec2::new(112.0, 8.0),
                    Vec2::new(29.0, 59.0),
                    Vec2::new(-5.0, -3.0),
                    Vec2::new(149.0, 133.0),
                ] {
                    assert_oam_partition(projection, physical, origin);
                }
            }
        }
    }

    fn body_registrations(
        layout: &BattleSceneLayout,
        viewport: Vec2,
    ) -> BattleSourceBodyRegistrations {
        // Different source proportions and world footprints make accidental
        // width-based scale or independent X/Y stretching observable.
        let source = [
            Rect::new(20.0, 57.0, 59.0, 100.0),
            Rect::new(100.0, 6.0, 146.0, 52.0),
        ];
        let slot = [
            Rect::new(8.0, 40.0, 72.0, 104.0),
            Rect::new(96.0, 0.0, 152.0, 56.0),
        ];
        let bodies = std::array::from_fn(|i| {
            let mut min = Vec2::splat(f32::INFINITY);
            let mut max = Vec2::splat(f32::NEG_INFINITY);
            for corner in 0..8 {
                let point = layout.origins[i]
                    + Vec3::new(
                        if corner & 1 == 0 { -0.35 } else { 0.55 },
                        if corner & 2 == 0 {
                            0.0
                        } else {
                            1.0 + i as f32 * 0.6
                        },
                        if corner & 4 == 0 { -0.25 } else { 0.25 },
                    );
                let view = layout.project_point(point, viewport);
                min = min.min(view);
                max = max.max(view);
            }
            Some(BattleSourceBodyRegistration {
                source: source[i],
                slot: slot[i],
                view: Rect::from_corners(min, max),
            })
        });
        BattleSourceBodyRegistrations {
            bodies,
            viewport,
            camera: layout.camera,
        }
    }

    fn local(side: VisualBattleSide) -> VisualBattleSourcePlacement {
        VisualBattleSourcePlacement::BattlerLocal { side }
    }

    fn transport(from: VisualBattleSide, pivot: Vec2) -> VisualBattleSourcePlacement {
        VisualBattleSourcePlacement::IndependentTransport {
            from,
            assembly: VisualBattleSourceAssembly {
                event_index: 17,
                spawn_frame: 9,
            },
            pivot,
        }
    }

    fn travel_edges(
        registrations: &BattleSourceBodyRegistrations,
        from: VisualBattleSide,
    ) -> (f32, f32) {
        let attacker = registrations.bodies[from.index()].unwrap();
        let target = registrations.bodies[from.opposite().index()].unwrap();
        match from {
            VisualBattleSide::Player => (attacker.slot.max.x, target.slot.min.x),
            VisualBattleSide::Enemy => (attacker.slot.min.x, target.slot.max.x),
        }
    }

    fn assert_upright_circle(projection: BattleSourceProjection, viewport: Vec2, center: Vec2) {
        let physical = |point| projection.project(point).unwrap() * viewport;
        let origin = physical(center);
        let x = physical(center + Vec2::X * 8.0) - origin;
        let y = physical(center + Vec2::Y * 8.0) - origin;
        assert!(x.x > 0.0 && y.y > 0.0);
        assert!(
            x.y.abs() < 0.0002 && y.x.abs() < 0.0002,
            "source screen-up must stay upright"
        );
        assert!((x.length() / y.length() - 1.0).abs() < 0.0001);
        assert!(x.normalize().dot(y.normalize()).abs() < 0.0001);
        for angle in 0..32 {
            let radians = std::f32::consts::TAU * angle as f32 / 32.0;
            let point = center + Vec2::from_angle(radians) * 8.0;
            assert!(((physical(point) - origin).length() / x.length() - 1.0).abs() < 0.0001);
            assert!(
                (projection
                    .unproject(projection.project(point).unwrap())
                    .unwrap()
                    - point)
                    .length()
                    < 0.0005
            );
        }
    }

    #[test]
    fn body_projection_registers_opaque_bottom_center_and_height_without_width_stretch() {
        for viewport in [
            Vec2::new(800.0, 600.0),
            Vec2::new(1600.0, 900.0),
            Vec2::new(600.0, 900.0),
        ] {
            for layout in layouts(viewport) {
                let registrations = body_registrations(&layout, viewport);
                for side in [VisualBattleSide::Player, VisualBattleSide::Enemy] {
                    let body = registrations.bodies[side.index()].unwrap();
                    let projection = registrations
                        .projection(local(side), &layout, viewport)
                        .unwrap();
                    let bottom = Vec2::new(body.source.center().x, body.source.max.y);
                    let top = Vec2::new(body.source.center().x, body.source.min.y);
                    let expected_bottom = Vec2::new(body.view.center().x, body.view.max.y);
                    let expected_top = Vec2::new(body.view.center().x, body.view.min.y);
                    assert!(
                        ((projection.project(bottom).unwrap() - expected_bottom) * viewport)
                            .length()
                            < 0.0002
                    );
                    assert!(
                        ((projection.project(top).unwrap() - expected_top) * viewport).length()
                            < 0.0002
                    );
                    let expected_scale = body.view.height() * viewport.y / body.source.height();
                    let dx = (projection.project(bottom + Vec2::X * 8.0).unwrap()
                        - projection.project(bottom).unwrap())
                        * viewport;
                    assert!((dx.x - 8.0 * expected_scale).abs() < 0.0002);
                    assert_upright_circle(projection, viewport, body.source.center());

                    let mut wider = registrations.clone();
                    let view = &mut wider.bodies[side.index()].as_mut().unwrap().view;
                    view.min.x -= 0.125;
                    view.max.x += 0.125;
                    let same = wider.projection(local(side), &layout, viewport).unwrap();
                    for point in [body.source.min, bottom, body.source.max] {
                        assert!(
                            ((same.project(point).unwrap() - projection.project(point).unwrap())
                                * viewport)
                                .length()
                                < 0.0002
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn transport_uses_current_source_slot_progress_and_preserves_both_endpoint_charts() {
        for viewport in [
            Vec2::new(800.0, 600.0),
            Vec2::new(1600.0, 900.0),
            Vec2::new(600.0, 900.0),
        ] {
            for layout in layouts(viewport) {
                let registrations = body_registrations(&layout, viewport);
                for from in [VisualBattleSide::Player, VisualBattleSide::Enemy] {
                    let (departure, arrival) = travel_edges(&registrations, from);
                    let start = registrations
                        .projection(local(from), &layout, viewport)
                        .unwrap();
                    let end = registrations
                        .projection(local(from.opposite()), &layout, viewport)
                        .unwrap();
                    for progress in [-0.5_f32, 0.0, 0.25, 0.5, 0.75, 1.0, 1.5] {
                        let pivot = Vec2::new(departure + (arrival - departure) * progress, 36.0);
                        let projection = registrations
                            .projection(transport(from, pivot), &layout, viewport)
                            .unwrap();
                        for offset in [Vec2::ZERO, Vec2::new(-11.0, -8.0), Vec2::new(12.0, 16.0)] {
                            let source = pivot + offset;
                            let expected = start
                                .project(source)
                                .unwrap()
                                .lerp(end.project(source).unwrap(), progress.clamp(0.0, 1.0));
                            assert!(
                                ((projection.project(source).unwrap() - expected) * viewport)
                                    .length()
                                    < 0.0003
                            );
                        }
                        assert_upright_circle(projection, viewport, pivot);
                        // Assembly identity and pivot Y do not introduce a
                        // second clock or alter the horizontal slot progress.
                        let same = registrations
                            .projection(
                                VisualBattleSourcePlacement::IndependentTransport {
                                    from,
                                    assembly: VisualBattleSourceAssembly {
                                        event_index: 29,
                                        spawn_frame: 25,
                                    },
                                    pivot: pivot + Vec2::Y * 19.0,
                                },
                                &layout,
                                viewport,
                            )
                            .unwrap();
                        assert_eq!(projection, same);
                    }
                }
            }
        }
    }

    #[test]
    fn transport_is_continuous_at_both_slot_boundaries_and_keeps_distinct_terminal_contacts() {
        for viewport in [
            Vec2::new(800.0, 600.0),
            Vec2::new(1600.0, 900.0),
            Vec2::new(600.0, 900.0),
        ] {
            for layout in layouts(viewport) {
                let registrations = body_registrations(&layout, viewport);
                for from in [VisualBattleSide::Player, VisualBattleSide::Enemy] {
                    let (departure, arrival) = travel_edges(&registrations, from);
                    for boundary in [departure, arrival] {
                        let physical = |x: f32, offset: Vec2| {
                            let pivot = Vec2::new(x, 34.0);
                            registrations
                                .projection(transport(from, pivot), &layout, viewport)
                                .unwrap()
                                .project(pivot + offset)
                                .unwrap()
                                * viewport
                        };
                        for offset in [Vec2::ZERO, Vec2::new(-12.0, -8.0), Vec2::new(12.0, 16.0)] {
                            let before = physical(boundary - 0.0001, offset);
                            let at = physical(boundary, offset);
                            let after = physical(boundary + 0.0001, offset);
                            assert!((before - at).length() < 0.03);
                            assert!((after - at).length() < 0.03);
                        }
                    }
                    let target = registrations.bodies[from.opposite().index()].unwrap();
                    let bottom = Vec2::new(target.source.center().x, target.source.max.y);
                    let contact = bottom + Vec2::new(7.0, -11.0);
                    let endpoint = registrations
                        .projection(transport(from, bottom), &layout, viewport)
                        .unwrap();
                    let other = registrations
                        .projection(transport(from, contact), &layout, viewport)
                        .unwrap();
                    let expected = registrations
                        .projection(local(from.opposite()), &layout, viewport)
                        .unwrap();
                    assert_eq!(endpoint, expected);
                    assert_eq!(other, expected);
                    let first = endpoint.project(bottom).unwrap() * viewport;
                    let second = other.project(contact).unwrap() * viewport;
                    let scale = target.view.height() * viewport.y / target.source.height();
                    assert!(((second - first) - (contact - bottom) * scale).length() < 0.0002);
                    assert!((second - first).length() > 0.0);
                    assert!(
                        (first - Vec2::new(target.view.center().x, target.view.max.y) * viewport)
                            .length()
                            < 0.0002
                    );
                }
            }
        }
    }

    #[test]
    fn body_and_transport_oam_partition_is_identical_for_all_pixels_and_clipped_edges() {
        for physical in [
            UVec2::new(800, 600),
            UVec2::new(1600, 900),
            UVec2::new(600, 900),
        ] {
            let viewport = physical.as_vec2();
            for layout in layouts(viewport) {
                let registrations = body_registrations(&layout, viewport);
                for from in [VisualBattleSide::Player, VisualBattleSide::Enemy] {
                    let (departure, arrival) = travel_edges(&registrations, from);
                    let placements = [
                        local(from),
                        transport(
                            from,
                            Vec2::new(departure - (arrival - departure) * 0.5, 36.0),
                        ),
                        transport(from, Vec2::new(departure, 36.0)),
                        transport(
                            from,
                            Vec2::new(departure + (arrival - departure) * 0.5, 36.0),
                        ),
                        transport(from, Vec2::new(arrival, 36.0)),
                        transport(from, Vec2::new(arrival + (arrival - departure) * 0.5, 36.0)),
                    ];
                    for placement in placements {
                        let projection = registrations
                            .projection(placement, &layout, viewport)
                            .unwrap();
                        for origin in [
                            Vec2::new(112.0, 8.0),
                            Vec2::new(29.0, 59.0),
                            Vec2::new(-5.0, -3.0),
                            Vec2::new(149.0, 133.0),
                        ] {
                            assert_oam_partition(projection, physical, origin);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn body_projection_rejects_missing_invalid_or_stale_registrations() {
        let viewport = Vec2::new(800.0, 600.0);
        let layout = BattleSceneLayout::default();
        let registrations = body_registrations(&layout, viewport);
        let player = local(VisualBattleSide::Player);
        let travel = transport(VisualBattleSide::Player, Vec2::new(84.0, 36.0));
        assert!(
            BattleSourceBodyRegistrations::default()
                .projection(player, &layout, viewport)
                .is_none()
        );
        for invalid in [
            Vec2::ZERO,
            Vec2::new(-800.0, 600.0),
            Vec2::splat(f32::NAN),
            Vec2::splat(f32::INFINITY),
            viewport * 2.0,
        ] {
            assert!(registrations.projection(player, &layout, invalid).is_none());
        }
        let mut changed_camera = layout.clone();
        changed_camera.camera.translation.x += 0.01;
        assert!(
            registrations
                .projection(player, &changed_camera, viewport)
                .is_none()
        );
        for camera in [
            Transform::from_scale(Vec3::ZERO),
            Transform::from_translation(Vec3::splat(f32::INFINITY)),
        ] {
            let mut invalid = registrations.clone();
            invalid.camera = camera;
            let mut current = layout.clone();
            current.camera = camera;
            assert!(invalid.projection(player, &current, viewport).is_none());
        }
        for index in 0..2 {
            let mut missing = registrations.clone();
            missing.bodies[index] = None;
            assert!(missing.projection(travel, &layout, viewport).is_none());
            assert_eq!(
                missing.projection(player, &layout, viewport).is_none(),
                index == 0
            );
        }
        for rect in [
            Rect {
                min: Vec2::ZERO,
                max: Vec2::new(0.0, 1.0),
            },
            Rect {
                min: Vec2::ZERO,
                max: Vec2::new(1.0, 0.0),
            },
            Rect {
                min: Vec2::ONE,
                max: Vec2::ZERO,
            },
            Rect {
                min: Vec2::splat(f32::NAN),
                max: Vec2::ONE,
            },
            Rect {
                min: Vec2::ZERO,
                max: Vec2::splat(f32::INFINITY),
            },
            Rect {
                min: Vec2::splat(-f32::MAX),
                max: Vec2::splat(f32::MAX),
            },
        ] {
            for field in 0..3 {
                let mut invalid = registrations.clone();
                let body = invalid.bodies[0].as_mut().unwrap();
                match field {
                    0 => body.source = rect,
                    1 => body.slot = rect,
                    _ => body.view = rect,
                }
                assert!(invalid.projection(player, &layout, viewport).is_none());
                assert!(invalid.projection(travel, &layout, viewport).is_none());
            }
        }
        for min_x in [71.0, 72.0] {
            let mut overlapping = registrations.clone();
            overlapping.bodies[1].as_mut().unwrap().slot.min.x = min_x;
            for from in [VisualBattleSide::Player, VisualBattleSide::Enemy] {
                assert!(
                    overlapping
                        .projection(transport(from, Vec2::new(84.0, 36.0)), &layout, viewport)
                        .is_none()
                );
            }
        }
        for pivot in [
            Vec2::new(f32::NAN, 36.0),
            Vec2::new(84.0, f32::NAN),
            Vec2::splat(f32::INFINITY),
        ] {
            assert!(
                registrations
                    .projection(
                        transport(VisualBattleSide::Player, pivot),
                        &layout,
                        viewport
                    )
                    .is_none()
            );
        }
    }

    #[test]
    fn projected_oam_preserves_circle_aspect_and_anchor_trajectories() {
        for viewport in [
            Vec2::new(800.0, 600.0),
            Vec2::new(1600.0, 900.0),
            Vec2::new(600.0, 900.0),
        ] {
            for layout in layouts(viewport) {
                let projection = BattleSourceProjection::new(&layout, viewport).unwrap();
                let physical = |p| projection.project(p).unwrap() * viewport;
                let source = [Vec2::new(40.0, 72.0), Vec2::new(124.0, 32.0)];
                let targets = layout
                    .hit_anchors
                    .map(|p| layout.project_point(p, viewport) * viewport);
                for side in 0..2 {
                    assert!((physical(source[side]) - targets[side]).length() < 0.0002);
                    // The same mapping preserves the original trajectory in
                    // either direction, with no side-specific sprite scale.
                    for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
                        assert!(
                            (physical(source[side].lerp(source[1 - side], t))
                                - targets[side].lerp(targets[1 - side], t))
                            .length()
                                < 0.0002
                        );
                    }
                }
                for center in [
                    Vec2::new(16.0, 120.0),
                    source[0],
                    source[1],
                    Vec2::new(150.0, 8.0),
                ] {
                    let x = physical(center + Vec2::X * 8.0) - physical(center);
                    let y = physical(center + Vec2::Y * 8.0) - physical(center);
                    assert!((x.length() / y.length() - 1.0).abs() < 0.00003);
                    assert!(x.normalize().dot(y.normalize()).abs() < 0.00003);
                    // Actual round source marks must remain round for every
                    // sampled angle, not just match an axis-aligned bbox.
                    for angle in 0..32 {
                        let radians = std::f32::consts::TAU * angle as f32 / 32.0;
                        let point = center + Vec2::from_angle(radians) * 8.0;
                        assert!(
                            ((physical(point) - physical(center)).length() / x.length() - 1.0)
                                .abs()
                                < 0.00003
                        );
                        assert!(
                            (projection
                                .unproject(projection.project(point).unwrap())
                                .unwrap()
                                - point)
                                .length()
                                < 0.0002
                        );
                    }
                }
                // Corners, centers and texture sampling all share this same
                // global affine map; no independent object rectangle stretch.
                let origin = Vec2::new(112.0, 8.0);
                let a = physical(origin);
                let b = physical(origin + Vec2::splat(24.0));
                assert!((physical(origin + Vec2::splat(12.0)) - (a + b) * 0.5).length() < 0.0002);
            }
        }
        let layout = BattleSceneLayout::default();
        assert!(BattleSourceProjection::new(&layout, Vec2::ZERO).is_none());
        assert!(BattleSourceProjection::new(&layout, Vec2::splat(f32::NAN)).is_none());
        let mut coincident = layout;
        coincident.hit_anchors[1] = coincident.hit_anchors[0];
        assert!(BattleSourceProjection::new(&coincident, Vec2::new(800.0, 600.0)).is_none());
    }
}
