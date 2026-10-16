//! Shared clerestory brick building for Kanto 20/54/21 + 37/{3a,7d}/7e.
//! The same source drawing serves Pallet, Celadon, Saffron and Fuchsia.
use super::*;
use super::kanto_craft::{block, mix};

#[derive(Clone, Copy)]
pub(super) struct Paint {
    roof: [[f32; 3]; 2],
    brick: [f32; 3],
    mortar: [f32; 3],
    glass: [f32; 3],
    pub(super) windows: [[f32; 3]; 10],
    door: [f32; 3],
    ink: [f32; 3],
    clerestory_only: bool,
}
fn linear(c: [u8; 3]) -> [f32; 3] {
    c.map(|v| {
        let x = f32::from(v) / 255.;
        if x <= 0.04045 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        }
    })
}
pub(super) fn paint(
    cells: &[&VisualTile],
    g: &GridGeometry,
    p: BuildingPlacement,
    images: &TerrainImageSamples,
) -> Option<Paint> {
    let sample = |x: usize, y: usize, tile: u16| {
        let cell = cells.get((p.row + y) * g.width + p.column + x)?;
        if cell.source.tileset_id.as_ref() != "kanto" || cell.source.tile_index != tile {
            return None;
        }
        let data = tile_rgba(images, cell).ok()?;
        (data.len() == 256 && data.chunks_exact(4).all(|p| p[3] == 255)).then_some(data)
    };
    let two = |rgba: &[u8]| -> Option<[[f32; 3]; 2]> {
        let colors = rgba
            .chunks_exact(4)
            .map(|p| [p[0], p[1], p[2]])
            .collect::<std::collections::BTreeSet<_>>();
        if colors.len() != 2 {
            return None;
        }
        let mut c = colors.into_iter().collect::<Vec<_>>();
        c.sort_by_key(|c| u32::from(c[0]) * 2126 + u32::from(c[1]) * 7152 + u32::from(c[2]) * 722);
        Some([linear(c[0]), linear(c[1])])
    };
    let pixel = |rgba: &[u8], x: usize, y: usize| {
        linear(
            rgba[(y * 8 + x) * 4..(y * 8 + x) * 4 + 3]
                .try_into()
                .unwrap(),
        )
    };
    let wall = two(sample(2, 5, 0x4b)?)?;
    let roof = two(sample(5, 1, 0x5e)?)?;
    let window = sample(2, 4, 0x0a)?;
    // 0a and32 share their drawn outline, but carry independent live palettes.
    // At night the source deliberately leaves selected upper panes unlit.
    let clerestory_only=cells.get((p.row+4)*g.width+p.column+4)?.source.metatile_id==0x7d;
    let panes=if clerestory_only {[
        (1,4,0x32),(2,4,0x0a),(3,4,0x0a),(4,4,0x0a),(5,4,0x32),
        (6,4,0x0a),(7,4,0x0a),(8,4,0x32),(9,4,0x0a),(10,4,0x32),
    ]} else {[
        (1, 4, 0x32),
        (2, 4, 0x0a),
        (3, 4, 0x0a),
        (4, 4, 0x32),
        (7, 4, 0x32),
        (8, 4, 0x32),
        (9, 4, 0x0a),
        (10, 4, 0x32),
        (6, 6, 0x0a),
        (7, 6, 0x0a),
    ]};
    let mut windows = [[0.; 3]; 10];
    for (i, (x, y, tile)) in panes
    .into_iter()
    .enumerate()
    {
        windows[i] = pixel(sample(x, y, tile)?, 3, 3);
    }
    Some(Paint {
        roof,
        brick: wall[0],
        mortar: wall[1],
        glass: if clerestory_only {windows[0]} else {pixel(sample(4, 6, 0x0b)?, 6, 5)},
        windows,
        ink: pixel(window, 0, 0),
        door: if clerestory_only {wall[0]} else {pixel(sample(4, 7, 0x1b)?, 5, 5)},
        clerestory_only,
    })
}
pub(super) fn append(out: &mut SurfaceMeshData, b: [f32; 4], unit: f32, door: Option<f32>, p: Paint) {
    let [w, e, n, s] = b;
    let front = s - unit * 0.72;
    let back = n + unit * 0.48;
    let stone = mix(p.roof[1], p.mortar, 0.28);
    let trim = mix(p.mortar, p.ink, 0.18);
    let mut box_at =
        |b: [f32; 6], bevel: f32, c: [f32; 3], kind: f32| block(out, b, bevel, c, kind, unit);
    box_at(
        [
            w + unit * 0.38,
            e - unit * 0.38,
            0.,
            unit * 0.42,
            back,
            front + unit * 0.12,
        ],
        unit * 0.06,
        stone,
        3.,
    );
    box_at(
        [
            w + unit * 0.45,
            e - unit * 0.45,
            unit * 0.40,
            unit * 5.20,
            back,
            front,
        ],
        unit * 0.02,
        p.mortar,
        3.,
    );
    // Running-bond brickwork on every exposed wall. The high window band
    // stays above the main brick field, as in the source clerestory facade.
    for side in 0..4 {
        let (a, z, span) = if side < 2 {
            (
                w + unit * 0.48,
                if side == 0 { front } else { back },
                e - w - unit * 0.96,
            )
        } else {
            (
                back,
                if side == 2 {
                    w + unit * 0.45
                } else {
                    e - unit * 0.45
                },
                front - back,
            )
        };
        let count = (span / (unit * 0.73)).ceil() as usize;
        let step = span / count as f32;
        for row in 0..7 {
            for col in 0..=count {
                let start = (col as f32 - if row % 2 == 0 { 0. } else { 0.5 }) * step;
                let l = a + start.max(0.) + unit * 0.012;
                let r = a + (start + step).min(span) - unit * 0.012;
                if r - l < unit * 0.05 {
                    continue;
                }
                let lo = unit * (0.53 + row as f32 * 0.43);
                let hi = lo + unit * 0.395;
                let color = p
                    .brick
                    .map(|v| v * (0.96 + ((row + col) % 5) as f32 * 0.018));
                let bounds = if side < 2 {
                    [l, r, lo, hi, z - unit * 0.035, z + unit * 0.035]
                } else {
                    [z - unit * 0.035, z + unit * 0.035, lo, hi, l, r]
                };
                box_at(bounds, unit * 0.018, color, 2.);
            }
        }
    }
    for x in [w + unit * 0.45, e - unit * 0.45] {
        for z in [back, front] {
            box_at(
                [
                    x - unit * 0.09,
                    x + unit * 0.09,
                    unit * 0.40,
                    unit * 5.24,
                    z - unit * 0.09,
                    z + unit * 0.09,
                ],
                unit * 0.025,
                trim,
                1.,
            );
        }
    }
    for y in [unit * 0.50, unit * 3.63, unit * 4.83, unit * 5.12] {
        box_at(
            [
                w + unit * 0.36,
                e - unit * 0.36,
                y,
                y + unit * 0.12,
                front - unit * 0.04,
                front + unit * 0.15,
            ],
            unit * 0.02,
            trim,
            1.,
        );
    }
    // Roof deck, physical slate flags and coping remain within the native plot.
    let roof = mix(p.roof[0], p.roof[1], 0.55);
    box_at(
        [
            w + unit * 0.08,
            e - unit * 0.08,
            unit * 5.22,
            unit * 5.42,
            n + unit * 0.10,
            s - unit * 0.20,
        ],
        unit * 0.04,
        roof,
        0.,
    );
    for row in 0..8 {
        for col in 0..12 {
            let l = w + unit * (0.30 + col as f32 * 0.95);
            let r = (l + unit * 0.93).min(e - unit * 0.30);
            let za = n + unit * (0.32 + row as f32 * 0.89);
            let zb = (za + unit * 0.87).min(s - unit * 0.42);
            if r > l && zb > za {
                box_at(
                    [l, r, unit * 5.41, unit * 5.47, za, zb],
                    unit * 0.025,
                    roof.map(|v| v * (0.98 + ((row * 3 + col) % 4) as f32 * 0.012)),
                    0.,
                );
            }
        }
    }
    for (a, z) in [
        (n + unit * 0.10, n + unit * 0.30),
        (s - unit * 0.40, s - unit * 0.20),
    ] {
        box_at(
            [
                w + unit * 0.08,
                e - unit * 0.08,
                unit * 5.40,
                unit * 5.70,
                a,
                z,
            ],
            unit * 0.045,
            stone,
            3.,
        );
    }
    for (a, z) in [
        (w + unit * 0.08, w + unit * 0.28),
        (e - unit * 0.28, e - unit * 0.08),
    ] {
        box_at(
            [
                a,
                z,
                unit * 5.40,
                unit * 5.70,
                n + unit * 0.28,
                s - unit * 0.38,
            ],
            unit * 0.045,
            stone,
            3.,
        );
    }
    drop(box_at);
    // Framed inset lights: all eight upper and both lower source panes.
    let mut window = |x: f32, y: f32, width: f32, height: f32, glass: [f32; 3]| {
        block(
            out,
            [
                x - width * 0.5 - unit * 0.055,
                x + width * 0.5 + unit * 0.055,
                y - unit * 0.055,
                y + height + unit * 0.055,
                front + unit * 0.025,
                front + unit * 0.15,
            ],
            unit * 0.02,
            p.ink,
            -1.,
            unit,
        );
        let light_first=out.positions.len();
        block(
            out,
            [
                x - width * 0.5,
                x + width * 0.5,
                y,
                y + height,
                front + unit * 0.15,
                front + unit * 0.17,
            ],
            unit * 0.008,
            glass.map(|v| v * 0.84),
            -1.,
            unit,
        );
        crate::city_lights::mark_window(out,light_first..out.positions.len(),light_first,true);
        for xx in [x - width * 0.5, x + width * 0.5] {
            block(
                out,
                [
                    xx - unit * 0.025,
                    xx + unit * 0.025,
                    y,
                    y + height,
                    front + unit * 0.17,
                    front + unit * 0.22,
                ],
                unit * 0.009,
                trim,
                1.,
                unit,
            );
        }
        block(
            out,
            [
                x - width * 0.5 - unit * 0.07,
                x + width * 0.5 + unit * 0.07,
                y - unit * 0.08,
                y - unit * 0.015,
                front + unit * 0.12,
                front + unit * 0.30,
            ],
            unit * 0.012,
            stone,
            3.,
            unit,
        );
        block(
            out,
            [
                x - width * 0.35,
                x + width * 0.15,
                y + height * 0.77,
                y + height * 0.80,
                front + unit * 0.171,
                front + unit * 0.18,
            ],
            0.,
            glass,
            -1.,
            unit,
        );
    };
    let upper: &[f32]=if p.clerestory_only {&[1.5,2.5,3.5,4.5,5.5,6.5,7.5,8.5,9.5,10.5]}
        else {&[1.5,2.5,3.5,4.5,7.5,8.5,9.5,10.5]};
    for (i, x) in upper.iter().copied()
        .into_iter()
        .enumerate()
    {
        window(
            w + unit * x,
            unit * 3.83,
            unit * 0.78,
            unit * 0.82,
            p.windows[i],
        );
    }
    for (i, x) in [6.5, 7.5].into_iter().enumerate().filter(|_|!p.clerestory_only) {
        window(
            w + unit * x,
            unit * 1.40,
            unit * 0.80,
            unit * 1.25,
            p.windows[8 + i],
        );
    }
    drop(window);
    // The 7d facade is an uninterrupted masonry wall below its ten lights.
    // It has no doorway, lower panes or entrance step in the source.
    let Some(door)=door else {return;};
    block(
        out,
        [
            door - unit * 0.98,
            door + unit * 0.98,
            unit * 0.38,
            unit * 3.31,
            front + unit * 0.10,
            front + unit * 0.30,
        ],
        unit * 0.045,
        trim,
        1.,
        unit,
    );
    block(
        out,
        [
            door - unit * 0.80,
            door + unit * 0.80,
            unit * 0.40,
            unit * 3.13,
            front + unit * 0.30,
            front + unit * 0.34,
        ],
        unit * 0.02,
        p.door,
        -1.,
        unit,
    );
    block(
        out,
        [
            door - unit * 0.61,
            door + unit * 0.61,
            unit * 1.85,
            unit * 2.91,
            front + unit * 0.34,
            front + unit * 0.36,
        ],
        unit * 0.015,
        p.ink,
        -1.,
        unit,
    );
    let light_first=out.positions.len();
    block(
        out,
        [
            door - unit * 0.51,
            door + unit * 0.51,
            unit * 1.95,
            unit * 2.81,
            front + unit * 0.36,
            front + unit * 0.38,
        ],
        unit * 0.012,
        p.glass,
        -1.,
        unit,
    );
    crate::city_lights::mark_window(out,light_first..out.positions.len(),light_first,true);
    block(
        out,
        [
            door + unit * 0.55,
            door + unit * 0.62,
            unit * 1.12,
            unit * 1.62,
            front + unit * 0.35,
            front + unit * 0.46,
        ],
        unit * 0.015,
        stone,
        -1.,
        unit,
    );
    block(
        out,
        [
            door - unit * 1.05,
            door + unit * 1.05,
            0.,
            unit * 0.28,
            front + unit * 0.18,
            s,
        ],
        unit * 0.035,
        stone,
        3.,
        unit,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    fn colors() -> Paint {
        Paint {
            roof: [[0.12; 3], [0.45; 3]],
            brick: [0.4, 0.22, 0.04],
            mortar: [0.8, 0.9, 0.8],
            glass: [0.7, 0.9, 0.7],
            windows: [[0.7, 0.9, 0.7]; 10],
            door: [0.4, 0.2, 0.03],
            ink: [0.04; 3],
            clerestory_only: false,
        }
    }
    #[test]
    fn doorless_brick_variant_has_ten_upper_lights_without_a_false_entrance() {
        let mut paint=colors();paint.clerestory_only=true;
        let mut mesh=SurfaceMeshData::default();
        append(&mut mesh,[0.,96.,0.,64.],8.,None,paint);
        let panes=mesh.uvs.iter().filter(|&&uv|crate::city_lights::source_window_tag(uv))
            .map(|uv|uv[0].to_bits()).collect::<std::collections::BTreeSet<_>>();
        assert_eq!(panes.len(),10);
        assert!(mesh.positions.iter().zip(&mesh.uvs).filter(|(_,uv)|crate::city_lights::source_window_tag(**uv))
            .all(|(p,_)|p[1]>3.8*8.));
        assert!(mesh.positions.iter().all(|p|p[1]>=3.4*8. || p[2]<(64.-8.*0.72)+8.*0.31));
        let mut changed=SurfaceMeshData::default();paint.windows[4]=[0.02,0.04,0.09];
        append(&mut changed,[0.,96.,0.,64.],8.,None,paint);
        assert_eq!(mesh.positions,changed.positions);assert_eq!(mesh.indices,changed.indices);
        assert_ne!(mesh.colors,changed.colors);
    }

    #[test]
    fn closed_office_parts_have_outward_winding_and_no_zero_area_faces() {
        for (bounds, kind, bevel) in [
            ([0., 2., 0., 3., 0., 1.], 1., 0.03),
            ([0., 2., 0., 0.1, 0., 0.01], -1., 0.),
            ([0., 2., 0., 1., 0., 0.07], 2., 0.018),
            ([0., 0.07, 0., 1., 0., 2.], 2., 0.018),
        ] {
            let mut m = SurfaceMeshData::default();
            block(&mut m, bounds, bevel, [0.5; 3], kind, 1.);
            let mut edges = std::collections::BTreeMap::new();
            let mut volume = 0.;
            for t in m.indices.chunks_exact(3) {
                let p = [t[0], t[1], t[2]].map(|i| Vec3::from_array(m.positions[i as usize]));
                assert!((p[1] - p[0]).cross(p[2] - p[0]).length() > 0.000001);
                volume += p[0].dot(p[1].cross(p[2])) / 6.;
                for i in 0..3 {
                    let mut e = [
                        p[i].to_array().map(|v| (v * 100000.).round() as i32),
                        p[(i + 1) % 3]
                            .to_array()
                            .map(|v| (v * 100000.).round() as i32),
                    ];
                    e.sort();
                    *edges.entry(e).or_insert(0) += 1;
                }
            }
            assert!(volume > 0.);
            assert!(edges.values().all(|&n| n == 2));
        }
    }
    #[test]
    fn corner_posts_do_not_bury_the_masonry_on_any_of_the_four_walls() {
        let mut m = SurfaceMeshData::default();
        append(&mut m, [0., 96., 0., 64.], 8., Some(40.), colors());
        for (axis, positive, along) in [
            (2, true, 2.8 * 8.),
            (2, false, 2.8 * 8.),
            (0, true, 3.404 * 8.),
            (0, false, 3.404 * 8.),
        ] {
            let query = bevy::prelude::Vec2::new(along, 2.03 * 8.);
            let mut hit = None::<(f32, usize)>;
            for tri in m.indices.chunks_exact(3) {
                let p = [tri[0], tri[1], tri[2]].map(|i| Vec3::from_array(m.positions[i as usize]));
                let project =
                    |p: Vec3| bevy::prelude::Vec2::new(if axis == 2 { p.x } else { p.z }, p.y);
                let q = p.map(project);
                let a = q[1] - q[0];
                let b = q[2] - q[0];
                let d = query - q[0];
                let area = a.perp_dot(b);
                if area.abs() < 0.000001 {
                    continue;
                }
                let u = d.perp_dot(b) / area;
                let v = a.perp_dot(d) / area;
                if u < 0. || v < 0. || u + v > 1. {
                    continue;
                }
                let depth = p[0][axis] * (1. - u - v) + p[1][axis] * u + p[2][axis] * v;
                if hit.is_none_or(|(old, _)| if positive { depth > old } else { depth < old }) {
                    hit = Some((depth, tri[0] as usize));
                }
            }
            let (_, index) = hit.expect("closed facade");
            assert_eq!(
                crate::scenery_finish::town_paint_kind(m.uvs[index]),
                Some(2),
                "wall axis{axis}, positive{positive}"
            );
        }
    }

    #[test]
    fn shared_office_stays_in_its_native_plot_and_has_bounded_geometry() {
        let mut m = SurfaceMeshData::default();
        append(&mut m, [24., 120., 40., 104.], 8., Some(64.), colors());
        assert!(
            m.indices.len() / 3 < 12000,
            "{} triangles",
            m.indices.len() / 3
        );
        assert!(m.indices.len() / 3 > 5000);
        assert!(m.positions.iter().all(|p| p.iter().all(|v| v.is_finite())
            && (24. ..=120.).contains(&p[0])
            && (40. ..=104.).contains(&p[2])
            && p[1] >= 0.));
        assert_eq!(m.positions.len(), m.colors.len());
        assert_eq!(m.positions.len(), m.uvs.len());
        assert!(
            m.normals
                .iter()
                .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.001)
        );
        let mut changed = SurfaceMeshData::default();
        let mut night = colors();
        night.windows[0] = [0.1, 0.03, 0.08];
        append(&mut changed, [24., 120., 40., 104.], 8., Some(64.), night);
        assert_eq!(m.positions, changed.positions);
        assert_eq!(m.indices, changed.indices);
        assert_ne!(m.colors, changed.colors);
    }
}
