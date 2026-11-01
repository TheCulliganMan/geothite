use super::*;

fn slot() -> Rect {
    Rect::from_corners(FRONT_SLOT_MIN, FRONT_SLOT_MAX)
}

fn registration(opaque: Rect, view: Rect) -> CaptureActorRegistration {
    CaptureActorRegistration { source_opaque_rect: opaque, actor_view_rect: view }
}

fn project(source: Vec2, r: CaptureActorRegistration) -> Vec2 {
    r.actor_view_rect.min + (source - r.source_opaque_rect.min)
        / r.source_opaque_rect.size() * r.actor_view_rect.size()
}

fn unproject(uv: Vec2, r: CaptureActorRegistration) -> Vec2 {
    r.source_opaque_rect.min + (uv - r.actor_view_rect.min)
        / r.actor_view_rect.size() * r.source_opaque_rect.size()
}

fn close(actual: Vec2, expected: Vec2) {
    assert!(actual.abs_diff_eq(expected, 0.00004), "{actual:?} != {expected:?}");
}

fn pattern() -> Vec<[u8; 4]> {
    (0..56 * 56).map(|i| {
        let (x, y) = (i % 56, i / 56);
        [x as u8, y as u8, (x ^ (y * 3)) as u8,
            if (x + y * 2) % 11 == 0 { 0 } else if (2 * x + y) % 13 == 0 { 96 } else { 255 }]
    }).collect()
}

/// Independent forward raster reference: copy complete 8x8 tiles, including
/// arbitrary RGB under zero alpha. No coordinate math from the inverse helper.
fn reference_copy(source: &[[u8; 4]], axes: &[usize], left: usize, top: usize) -> Vec<[u8; 4]> {
    let mut out = vec![[0; 4]; 56 * 56];
    for (dy, sy) in axes.iter().copied().enumerate() {
        for (dx, sx) in axes.iter().copied().enumerate() {
            for y in 0..8 {
                for x in 0..8 {
                    out[(top + dy * 8 + y) * 56 + left + dx * 8 + x] =
                        source[(sy * 8 + y) * 56 + sx * 8 + x];
                }
            }
        }
    }
    out
}

#[test]
fn every_output_pixel_matches_intact_tile_copy_in_both_axes() {
    let source = pattern();
    let r = registration(slot(), Rect::new(0.2, 0.1, 0.8, 0.9));
    for (shown, axes, left, top) in [
        (7, vec![0, 1, 2, 3, 4, 5, 6], 0, 0),
        (5, vec![0, 1, 3, 5, 6], 8, 16),
        (3, vec![0, 3, 6], 16, 32),
    ] {
        let expected = reference_copy(&source, &axes, left, top);
        let mut transparent_selected = 0;
        let mut translucent_selected = 0;
        for y in 0..56 { for x in 0..56 {
            let destination = FRONT_SLOT_MIN + Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let source_uv = capture_actor_sample_uv(project(destination, r), r, shown);
            let actual = source_uv.map_or([0; 4], |uv| {
                let pixel = (unproject(uv, r) - FRONT_SLOT_MIN).floor().as_uvec2();
                let rgba = source[(pixel.y * 56 + pixel.x) as usize];
                transparent_selected += usize::from(rgba[3] == 0);
                translucent_selected += usize::from(rgba[3] == 96);
                rgba
            });
            assert_eq!(actual, expected[y * 56 + x], "shown={shown}, x={x}, y={y}");
        } }
        assert!(transparent_selected > 0 && translucent_selected > 0);
        assert_eq!(left * 2 + axes.len() * 8, 56);
        assert_eq!(top + axes.len() * 8, 56);
    }
}

#[test]
fn preserves_subpixel_detail_and_exact_seven_tile_identity() {
    let r = registration(slot(), Rect::new(0.125, 0.0625, 0.875, 0.9375));
    for shown in [7, 5, 3] {
        let axes: &[f32] = match shown {
            7 => &[0., 1., 2., 3., 4., 5., 6.],
            5 => &[0., 1., 3., 5., 6.],
            _ => &[0., 3., 6.],
        };
        let left = ((7 - shown) / 2 * 8) as f32;
        let top = ((7 - shown) * 8) as f32;
        for (dy, sy) in axes.iter().copied().enumerate() {
            for (dx, sx) in axes.iter().copied().enumerate() {
                for fraction in [Vec2::new(0.125, 0.375), Vec2::new(2.25, 5.625), Vec2::new(7.875, 7.75)] {
                    let destination = FRONT_SLOT_MIN + Vec2::new(left + dx as f32 * 8., top + dy as f32 * 8.) + fraction;
                    let expected = FRONT_SLOT_MIN + Vec2::new(sx, sy) * 8. + fraction;
                    assert_eq!(capture_front_source_pixel(destination, shown), Some(expected));
                    let uv = project(destination, r);
                    let actual = capture_actor_sample_uv(uv, r, shown).unwrap();
                    close(unproject(actual, r), expected);
                    if shown == 7 { assert_eq!(actual, uv); }
                }
            }
        }
    }
    let a = capture_front_source_pixel(Vec2::new(120.125, 32.25), 5).unwrap();
    let b = capture_front_source_pixel(Vec2::new(120.375, 32.5), 5).unwrap();
    assert_eq!(b - a, Vec2::new(0.25, 0.25), "mesh detail must not collapse to LCD texel centers");
}

#[test]
fn tile_edges_and_full_slot_anchor_are_half_open() {
    for (shown, left, top) in [(7, 0., 0.), (5, 8., 16.), (3, 16., 32.)] {
        let first = FRONT_SLOT_MIN + Vec2::new(left, top);
        assert_eq!(capture_front_source_pixel(first, shown), Some(FRONT_SLOT_MIN));
        let end = first + Vec2::splat(shown as f32 * 8.);
        assert_eq!(end.y, FRONT_SLOT_MAX.y);
        assert_eq!((first.x + end.x) * 0.5, 124.);
        assert!(capture_front_source_pixel(Vec2::new(end.x, end.y - 0.125), shown).is_none());
        assert!(capture_front_source_pixel(Vec2::new(end.x - 0.125, end.y), shown).is_none());
        assert!(capture_front_source_pixel(first - Vec2::new(0.125, 0.), shown).is_none());
        assert!(capture_front_source_pixel(first - Vec2::new(0., 0.125), shown).is_none());
        assert_eq!(capture_front_source_pixel(end - Vec2::splat(0.125), shown), Some(FRONT_SLOT_MAX - Vec2::splat(0.125)));
    }
    // Crossing destination column 1 -> 2 skips source column 2 in 5-tile mode.
    assert_eq!(capture_front_source_pixel(Vec2::new(119.875, 16.375), 5), Some(Vec2::new(111.875, 0.375)));
    assert_eq!(capture_front_source_pixel(Vec2::new(120., 16.375), 5), Some(Vec2::new(120., 0.375)));
    // Source Y has exactly the same discontinuity, not a vertical center crop.
    assert_eq!(capture_front_source_pixel(Vec2::new(104.375, 31.875), 5), Some(Vec2::new(96.375, 15.875)));
    assert_eq!(capture_front_source_pixel(Vec2::new(104.375, 32.), 5), Some(Vec2::new(96.375, 24.)));
}

#[test]
fn varied_registrations_sample_original_actor_without_scaling_or_cropping_it() {
    for opaque in [slot(), Rect::new(100., 4., 148., 56.), Rect::new(112., 8., 144., 48.), Rect::new(97., 1., 123., 41.)] {
        for view in [Rect::new(0.2, 0.1, 0.8, 0.9), Rect::new(0.42, 0.33, 0.61, 0.79), Rect::new(-0.1, 0.05, 0.55, 0.85)] {
            let r = registration(opaque, view);
            for shown in [7, 5, 3] {
                let mut tested = 0;
                for y in 0..56 { for x in 0..56 {
                    let destination = FRONT_SLOT_MIN + Vec2::new(x as f32 + 0.375, y as f32 + 0.625);
                    let uv = project(destination, r);
                    let expected = if shown == 7 {
                        Some(uv)
                    } else {
                        capture_front_source_pixel(destination, shown)
                            .map(|source| project(source, r))
                    }.filter(|source_uv| inside(*source_uv, Vec2::ZERO, Vec2::ONE))
                        .filter(|_| inside(uv, Vec2::ZERO, Vec2::ONE));
                    let actual = capture_actor_sample_uv(uv, r, shown);
                    match (actual, expected) {
                        (Some(a), Some(b)) => { close(a, b); tested += 1; },
                        (None, None) => {},
                        other => panic!("{other:?}; shown={shown}; destination={destination:?}; registration={r:?}"),
                    }
                } }
                assert!(tested > 0);
            }
        }
    }
}

#[test]
fn registration_is_not_a_mask_for_displacement_or_antialias_coverage() {
    let r = registration(Rect::new(112., 8., 144., 40.), Rect::new(0.4, 0.2, 0.6, 0.7));
    // Compact tile (1,1) comes from original (3,3). Destination y=40.5
    // lies below original opaque max y=40, yet remapped y=24.5 is valid.
    let destination = Vec2::new(120.5, 40.5);
    let uv = project(destination, r);
    assert!(uv.y > r.actor_view_rect.max.y);
    close(capture_actor_sample_uv(uv, r, 3).unwrap(), project(Vec2::new(120.5, 24.5), r));
    // A source sample just left of the projected vertex footprint can carry
    // antialias coverage, so the affine registration must not hard-clip it.
    let edge_source = Vec2::new(111.875, 24.5);
    let edge_uv = project(edge_source, r);
    assert!(edge_uv.x < r.actor_view_rect.min.x);
    let destination_uv = project(Vec2::new(119.875, 32.5), r);
    close(capture_actor_sample_uv(destination_uv, r, 5).unwrap(), edge_uv);
    assert_eq!(capture_actor_sample_uv(edge_uv, r, 7), Some(edge_uv));

    // Full7 also preserves fringe outside the registered full-slot coverage.
    // Any pixels there retain the isolated target's own color and alpha.
    let edge_r = registration(slot(), Rect::new(0.4, 0.2, 0.6, 0.7));
    let fringe = Vec2::new(0.3999, 0.5);
    assert!(unproject(fringe, edge_r).x < FRONT_SLOT_MIN.x);
    assert_eq!(capture_actor_sample_uv(fringe, edge_r, 7), Some(fringe));
    assert!(capture_actor_sample_uv(fringe, edge_r, 5).is_none());
}

#[test]
fn rejects_bad_dimensions_states_and_outside_coordinates() {
    let r = registration(slot(), Rect::new(0.2, 0.1, 0.8, 0.9));
    let uv = project(Vec2::new(124., 40.), r);
    for shown in [0, 1, 2, 4, 6, 8, u32::MAX] {
        assert!(capture_front_source_pixel(Vec2::new(124., 40.), shown).is_none());
        assert!(capture_actor_sample_uv(uv, r, shown).is_none());
    }
    for point in [Vec2::new(-1., 0.), Vec2::new(95.999, 40.), Vec2::new(152., 40.), Vec2::new(124., -0.01), Vec2::new(124., 56.), Vec2::splat(f32::NAN), Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)] {
        assert!(capture_front_source_pixel(point, 7).is_none());
    }
    for point in [Vec2::new(-0.01, 0.5), Vec2::new(0.5, -0.01), Vec2::new(1., 0.5), Vec2::new(0.5, 1.), Vec2::splat(f32::NAN), Vec2::splat(f32::INFINITY)] {
        assert!(capture_actor_sample_uv(point, r, 7).is_none());
    }
    // Raw fields are deliberate: Rect::from_corners normalizes inverted axes.
    for bad in [
        Rect { min: Vec2::ZERO, max: Vec2::ZERO },
        Rect { min: Vec2::ONE, max: Vec2::ZERO },
        Rect { min: Vec2::new(0., 0.), max: Vec2::new(1., 0.) },
        Rect { min: Vec2::splat(f32::NAN), max: Vec2::ONE },
        Rect { min: Vec2::ZERO, max: Vec2::splat(f32::INFINITY) },
        Rect { min: Vec2::splat(-f32::MAX), max: Vec2::splat(f32::MAX) },
    ] {
        assert!(capture_actor_sample_uv(uv, registration(slot(), bad), 7).is_none());
        assert!(capture_actor_sample_uv(uv, registration(bad, r.actor_view_rect), 7).is_none());
    }
    for bad in [Rect::new(95., 0., 152., 56.), Rect::new(96., -1., 152., 56.), Rect::new(96., 0., 153., 56.), Rect::new(96., 0., 152., 57.)] {
        assert!(capture_actor_sample_uv(uv, registration(bad, r.actor_view_rect), 7).is_none());
    }
}
