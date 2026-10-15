//! Ecruteak's source-selected wooden houses: cedar courses, pale infill and
//! narrow lattice openings. All features stay inside the existing source plot.
use super::*;

const WOOD: [f32; 3] = [0.30, 0.18, 0.075];
const EDGE: [f32; 3] = [0.43, 0.28, 0.12];
const ROOF: [f32; 3] = [0.17, 0.14, 0.085];

fn block(mesh: &mut SurfaceMeshData, b: [f32; 6], color: [f32; 3]) {
    super::city_facades::block(mesh, b, color);
}

// The source facades have a continuous timber base with narrow upright
// boards. Keep it below the windows and omit the authored doorway span.
fn timber_base(mesh: &mut SurfaceMeshData, left: f32, right: f32, front: f32, t: f32) {
    block(mesh, [left, right, t * 0.15, t * 0.88, front, front + t * 0.06], WOOD);
    let boards = ((right - left) / (t * 0.25)).ceil() as usize;
    for i in 0..boards {
        let x = left + (right - left) * i as f32 / boards as f32;
        block(mesh, [x, (x + t * 0.06).min(right), t * 0.15, t * 0.88,
            front + t * 0.06, front + t * 0.09], EDGE);
    }
    block(mesh, [left, right, t * 0.88, t * 0.99, front, front + t * 0.10], EDGE);
}

fn face(mesh: &mut SurfaceMeshData, p: &[[f32; 3]], color: [f32; 3]) {
    let normal = (Vec3::from_array(p[1]) - Vec3::from_array(p[0]))
        .cross(Vec3::from_array(p[2]) - Vec3::from_array(p[0]))
        .normalize();
    let shade = 0.62 + 0.38 * normal.dot(Vec3::new(-0.35, 0.85, 0.4).normalize()).max(0.);
    let first = mesh.positions.len() as u32;
    for &v in p {
        mesh.positions.push(v);
        mesh.normals.push(normal.to_array());
        mesh.uvs.push([0., 0.]);
        mesh.colors
            .push([color[0] * shade, color[1] * shade, color[2] * shade, 1.]);
    }
    for i in 1..p.len() - 1 {
        mesh.indices
            .extend([first, first + i as u32, first + i as u32 + 1]);
    }
}

pub(super) fn append(mesh: &mut SurfaceMeshData, b: [f32; 4], t: f32, door: Option<f32>) {
    let [w, e, n, s] = b;
    let (left, right, back, front) = (w + t * 0.28, e - t * 0.28, n + t * 0.38, s - t * 0.46);
    let (base, eave, ridge) = (t * 0.18, t * 2.7, t * 4.2);
    let middle = (back + front) * 0.5;
    for z in [back - t * 0.08, front] {
        block(
            mesh,
            [left, right, t * 2.40, t * 2.52, z, z + t * 0.08],
            WOOD,
        );
    }
    for x in [left - t * 0.08, right] {
        block(
            mesh,
            [x, x + t * 0.08, t * 2.40, t * 2.52, back, front],
            WOOD,
        );
    }
    block(
        mesh,
        [left, right, 0., base, back, front],
        [0.38, 0.36, 0.29],
    );
    block(
        mesh,
        [left, right, base, eave, back, front],
        [0.77, 0.79, 0.62],
    );
    // Close the gables independently of the overlapping roof courses.
    face(
        mesh,
        &[
            [left, eave, back],
            [left, eave, front],
            [left, ridge, middle],
        ],
        WOOD,
    );
    face(
        mesh,
        &[
            [right, eave, front],
            [right, eave, back],
            [right, ridge, middle],
        ],
        WOOD,
    );
    let (rn, rs) = (n + t * 0.10, s - t * 0.16);
    for (a, z) in [(rn, middle), (middle, rs)] {
        let height = |v: f32| {
            if v <= middle {
                eave + (ridge - eave) * (v - rn) / (middle - rn)
            } else {
                ridge - (ridge - eave) * (v - middle) / (rs - middle)
            }
        };
        for course in 0..10 {
            let za = a + (z - a) * course as f32 / 10.;
            let zb = a + (z - a) * (course + 1) as f32 / 10.;
            let ya = height(za);
            let yb = height(zb);
            let thick = t * 0.065;
            let wl = w + t * 0.06;
            let er = e - t * 0.06;
            let c = ROOF.map(|v| v * (0.94 + (course % 3) as f32 * 0.05));
            face(
                mesh,
                &[[wl, ya, za], [wl, yb, zb], [er, yb, zb], [er, ya, za]],
                c,
            );
            face(
                mesh,
                &[
                    [wl, ya - thick, za],
                    [er, ya - thick, za],
                    [er, yb - thick, zb],
                    [wl, yb - thick, zb],
                ],
                c,
            );
            face(
                mesh,
                &[
                    [wl, ya - thick, za],
                    [wl, yb - thick, zb],
                    [wl, yb, zb],
                    [wl, ya, za],
                ],
                EDGE,
            );
            face(
                mesh,
                &[
                    [er, yb - thick, zb],
                    [er, ya - thick, za],
                    [er, ya, za],
                    [er, yb, zb],
                ],
                EDGE,
            );
            face(
                mesh,
                &[
                    [er, ya - thick, za],
                    [wl, ya - thick, za],
                    [wl, ya, za],
                    [er, ya, za],
                ],
                EDGE,
            );
            face(
                mesh,
                &[
                    [wl, yb - thick, zb],
                    [er, yb - thick, zb],
                    [er, yb, zb],
                    [wl, yb, zb],
                ],
                EDGE,
            );
        }
    }
    block(
        mesh,
        [left, right, eave - t * 0.12, eave, front, front + t * 0.10],
        EDGE,
    );
    // Wood wainscot wraps every side; seams are geometry, rather than paint.
    for i in 0..28 {
        let xa = left + (right - left) * i as f32 / 28.;
        let xb = left + (right - left) * (i + 1) as f32 / 28. - t * 0.018;
        let color = EDGE.map(|v| v * (0.90 + (i % 4) as f32 * 0.055));
        for z in [back - t * 0.045, front] {
            block(mesh, [xa, xb, base, t * 0.90, z, z + t * 0.045], color);
        }
    }
    for i in 0..18 {
        let za = back + (front - back) * i as f32 / 18.;
        let zb = back + (front - back) * (i + 1) as f32 / 18. - t * 0.018;
        for x in [left - t * 0.045, right] {
            block(mesh, [x, x + t * 0.045, base, t * 0.90, za, zb], EDGE);
        }
    }
    for x in [left, right - t * 0.12] {
        for z in [back, front - t * 0.12] {
            block(mesh, [x, x + t * 0.12, base, eave, z, z + t * 0.12], WOOD);
        }
    }
    let dx = door.unwrap_or((w + e) * 0.5);
    let half = t * 0.45;
    let dz = front + t * 0.055;
    block(
        mesh,
        [dx - half, dx + half, base, t * 2.05, front, dz],
        WOOD,
    );
    block(
        mesh,
        [
            dx - half + t * 0.08,
            dx + half - t * 0.08,
            t * 0.34,
            t * 1.92,
            dz,
            dz + t * 0.015,
        ],
        [0.12, 0.11, 0.065],
    );
    for i in 0..5 {
        let x = dx - half + t * 0.12 + i as f32 * t * 0.16;
        block(
            mesh,
            [
                x,
                x + t * 0.035,
                t * 0.34,
                t * 1.92,
                dz + t * 0.02,
                dz + t * 0.05,
            ],
            EDGE,
        );
    }
    for y in [t * 0.8, t * 1.35, t * 1.88] {
        block(
            mesh,
            [
                dx - half,
                dx + half,
                y,
                y + t * 0.035,
                dz + t * 0.02,
                dz + t * 0.05,
            ],
            EDGE,
        );
    }
    for x in [
        left + (dx - half - left) * 0.46,
        dx + half + (right - dx - half) * 0.54,
    ] {
        block(
            mesh,
            [
                x - t * 0.18,
                x + t * 0.18,
                t * 1.55,
                t * 2.30,
                front,
                front + t * 0.07,
            ],
            WOOD,
        );
        for slit in [-0.10, 0.04] {
            block(
                mesh,
                [
                    x + t * slit,
                    x + t * (slit + 0.055),
                    t * 1.63,
                    t * 2.21,
                    front + t * 0.075,
                    front + t * 0.09,
                ],
                [0.69, 0.68, 0.45],
            );
        }
    }
    block(
        mesh,
        [dx - t * 0.62, dx + t * 0.62, 0., base, front, s],
        [0.48, 0.45, 0.34],
    );
}

// The town's shrine gate and ruined tower are separate source drawings.
// Keep their timber silhouettes distinct from the ordinary house and pagoda.
pub(super) fn append_gate(mesh: &mut SurfaceMeshData, b: [f32; 4], t: f32) {
    let [w, e, n, s] = b;
    let (left, right, back, front) = (w + t * 0.30, e - t * 0.30, n + t * 0.30, s - t * 0.35);
    let h = t * 3.6;
    let infill = [0.76, 0.77, 0.59];
    block(
        mesh,
        [left, right, 0., t * 0.15, back, front],
        [0.41, 0.35, 0.22],
    );
    block(
        mesh,
        [left, right, t * 0.15, h, back, back + t * 0.12],
        infill,
    );
    block(
        mesh,
        [left, right, t * 0.15, h, front - t * 0.12, front],
        infill,
    );
    block(
        mesh,
        [left, left + t * 0.12, t * 0.15, h, back, front],
        infill,
    );
    // The source entry is on the east side, with no invented south door.
    // The two east-side warp cells occupy the southern portion of the
    // source drawing; leave that full passage open beneath the lintel.
    let entry = n + t * 2.0;
    for (a, b) in [(back, entry)] {
        block(mesh, [right - t * 0.12, right, t * 0.15, h, a, b], infill);
    }
    block(
        mesh,
        [right - t * 0.12, right, t * 3.2, h, entry, front],
        WOOD,
    );
    for x in [
        left,
        left + (right - left) * 0.33,
        left + (right - left) * 0.66,
        right - t * 0.12,
    ] {
        block(
            mesh,
            [x, x + t * 0.12, 0., h, front, front + t * 0.06],
            WOOD,
        );
    }
    for x in [
        left + (right - left) * 0.20,
        left + (right - left) * 0.50,
        left + (right - left) * 0.80,
    ] {
        block(
            mesh,
            [
                x - t * 0.11,
                x + t * 0.11,
                t * 2.0,
                t * 2.85,
                front,
                front + t * 0.05,
            ],
            [0.08, 0.065, 0.03],
        );
        block(
            mesh,
            [
                x - t * 0.025,
                x + t * 0.025,
                t * 2.0,
                t * 2.85,
                front + t * 0.055,
                front + t * 0.07,
            ],
            EDGE,
        );
    }
    timber_base(mesh, left, right, front, t);
    low_roof(mesh, b, t, h, t * 4.0, false);
}

pub(super) fn append_temple(mesh: &mut SurfaceMeshData, b: [f32; 4], t: f32, door: Option<f32>) {
    let [w, e, n, s] = b;
    let (left, right, back, front) = (w + t * 0.30, e - t * 0.30, n + t * 0.30, s - t * 0.35);
    let h = t * 3.9;
    let dx = door
        .unwrap_or((w + e) * 0.5)
        .clamp(left + t * 1.1, right - t * 1.1);
    let infill = [0.76, 0.77, 0.59];
    block(
        mesh,
        [left, right, 0., t * 0.15, back, front],
        [0.41, 0.35, 0.22],
    );
    block(
        mesh,
        [left, right, t * 0.15, h, back, back + t * 0.12],
        infill,
    );
    for x in [left, right - t * 0.12] {
        block(mesh, [x, x + t * 0.12, t * 0.15, h, back, front], infill);
    }
    for (a, b) in [(left, dx - t), (dx + t, right)] {
        block(mesh, [a, b, t * 0.15, h, front - t * 0.12, front], infill);
    }
    block(
        mesh,
        [dx - t, dx + t, t * 3.5, h, front - t * 0.12, front],
        WOOD,
    );
    block(
        mesh,
        [
            dx - t,
            dx + t,
            t * 0.15,
            t * 3.5,
            front - t * 0.40,
            front - t * 0.30,
        ],
        [0.06, 0.045, 0.024],
    );
    for x in [left, right - t * 0.12, dx - t, dx + t - t * 0.12] {
        block(
            mesh,
            [x, x + t * 0.12, t * 0.15, h, front, front + t * 0.06],
            WOOD,
        );
    }
    for x in [w + t * 1.5, w + t * 7.2, e - t * 1.5] {
        if (x - dx).abs() < t * 1.2 {
            continue;
        }
        block(
            mesh,
            [
                x - t * 0.12,
                x + t * 0.12,
                t * 2.0,
                t * 3.1,
                front,
                front + t * 0.05,
            ],
            [0.08, 0.065, 0.03],
        );
        block(
            mesh,
            [
                x - t * 0.025,
                x + t * 0.025,
                t * 2.0,
                t * 3.1,
                front + t * 0.055,
                front + t * 0.07,
            ],
            EDGE,
        );
    }
    for bar in 0..6 {
        let x = dx - t * 0.84 + bar as f32 * t * 0.32;
        block(
            mesh,
            [
                x,
                x + t * 0.06,
                t * 0.22,
                t * 3.38,
                front - t * 0.28,
                front - t * 0.20,
            ],
            EDGE,
        );
    }
    timber_base(mesh, left, dx - t, front, t);
    timber_base(mesh, dx + t, right, front, t);
    low_roof(mesh, b, t, h, t * 4.35, false);
}

pub(super) fn append_burned(mesh: &mut SurfaceMeshData, b: [f32; 4], t: f32, door: Option<f32>) {
    let [w, e, n, s] = b;
    let (left, right, back, front) = (w + t * 0.30, e - t * 0.30, n + t * 0.30, s - t * 0.40);
    let h = t * 3.5;
    let ash = [0.055, 0.046, 0.033];
    let timber = [0.40, 0.28, 0.105];
    let dx = door
        .unwrap_or(w + (e - w) * 0.30)
        .clamp(left + t * 0.70, right - t * 0.70);
    block(
        mesh,
        [left, right, 0., t * 0.15, back, front],
        [0.28, 0.25, 0.18],
    );
    block(
        mesh,
        [left, right, t * 0.15, h, back, back + t * 0.12],
        timber,
    );
    block(
        mesh,
        [left, left + t * 0.12, t * 0.15, h, back, front],
        timber,
    );
    block(
        mesh,
        [right - t * 0.12, right, t * 0.15, h, back, front],
        timber,
    );
    for (a, b) in [(left, dx - t * 0.55), (dx + t * 0.55, right)] {
        block(mesh, [a, b, t * 0.15, h, front - t * 0.12, front], timber);
    }
    block(
        mesh,
        [
            dx - t * 0.55,
            dx + t * 0.55,
            t * 3.0,
            h,
            front - t * 0.12,
            front,
        ],
        timber,
    );
    block(
        mesh,
        [
            dx - t * 0.55,
            dx + t * 0.55,
            t * 0.15,
            t * 3.0,
            front - t * 0.40,
            front - t * 0.30,
        ],
        [0.025, 0.022, 0.016],
    );
    for i in 0..9 {
        let x = left + (right - left) * i as f32 / 8.;
        if (x - dx).abs() > t * 0.58 {
            block(
                mesh,
                [
                    x - t * 0.04,
                    x + t * 0.04,
                    t * 0.15,
                    h,
                    front,
                    front + t * 0.05,
                ],
                ash,
            );
        }
    }
    for x in [left + t * 0.70, right - t * 0.70] {
        block(
            mesh,
            [
                x - t * 0.13,
                x + t * 0.13,
                t * 1.6,
                t * 2.65,
                front,
                front + t * 0.06,
            ],
            [0.035, 0.031, 0.020],
        );
    }
    // Dark interior remains behind the missing roof courses, not white infill.
    block(
        mesh,
        [left, right, h - t * 0.22, h - t * 0.12, back, front],
        ash,
    );
    low_roof(mesh, b, t, h, t * 3.95, true);
}

fn low_roof(mesh: &mut SurfaceMeshData, b: [f32; 4], t: f32, eave: f32, ridge: f32, burned: bool) {
    let [w, e, n, s] = b;
    let (left, right, back, front) = (w + t * 0.10, e - t * 0.10, n + t * 0.10, s - t * 0.12);
    let middle = (back + front) * 0.5;
    let height =
        |z: f32| eave + (ridge - eave) * (1. - ((z - middle) / ((front - back) * 0.5)).abs());
    for side in 0..2 {
        for row in 0..8 {
            for col in 0..6 {
                if burned
                    && ((side == 0 && row == 3 && col == 2) || (side == 1 && row == 5 && col == 4))
                {
                    continue;
                }
                let start = if side == 0 { back } else { middle };
                let za = start + (front - back) * 0.5 * row as f32 / 8.;
                let zb = start + (front - back) * 0.5 * (row + 1) as f32 / 8.;
                let xa = left + (right - left) * col as f32 / 6.;
                let xb = left + (right - left) * (col + 1) as f32 / 6.;
                let (ya, yb) = (height(za), height(zb));
                let color = if burned { [0.055, 0.052, 0.041] } else { ROOF };
                let color = color.map(|v| v * (0.91 + ((row + col) % 3) as f32 * 0.055));
                let thick = t * 0.06;
                face(
                    mesh,
                    &[[xa, ya, za], [xa, yb, zb], [xb, yb, zb], [xb, ya, za]],
                    color,
                );
                face(
                    mesh,
                    &[
                        [xa, ya - thick, za],
                        [xb, ya - thick, za],
                        [xb, yb - thick, zb],
                        [xa, yb - thick, zb],
                    ],
                    color,
                );
                face(
                    mesh,
                    &[
                        [xa, ya - thick, za],
                        [xa, ya, za],
                        [xb, ya, za],
                        [xb, ya - thick, za],
                    ],
                    color,
                );
                face(
                    mesh,
                    &[
                        [xb, yb - thick, zb],
                        [xb, yb, zb],
                        [xa, yb, zb],
                        [xa, yb - thick, zb],
                    ],
                    color,
                );
                face(
                    mesh,
                    &[
                        [xa, yb - thick, zb],
                        [xa, yb, zb],
                        [xa, ya, za],
                        [xa, ya - thick, za],
                    ],
                    color,
                );
                face(
                    mesh,
                    &[
                        [xb, ya - thick, za],
                        [xb, ya, za],
                        [xb, yb, zb],
                        [xb, yb - thick, zb],
                    ],
                    color,
                );
            }
        }
    }
    for (x, reverse) in [(left, false), (right, true)] {
        let mut p = vec![
            [x, eave - t * 0.08, back],
            [x, eave - t * 0.08, front],
            [x, ridge, middle],
        ];
        if reverse {
            p.reverse();
        }
        face(mesh, &p, if burned { [0.06, 0.05, 0.03] } else { WOOD });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shrine_and_ruin_keep_source_plots_and_compact_closed_roofs() {
        for t in [8., 16.] {
            for kind in 0..3 {
                let b = if kind == 0 {
                    [-4. * t, 4. * t, -4. * t, 4. * t]
                } else {
                    [-6. * t, 6. * t, -3. * t, 3. * t]
                };
                let mut m = SurfaceMeshData::default();
                match kind {
                    0 => append_burned(&mut m, b, t, Some(-t)),
                    1 => append_gate(&mut m, b, t),
                    _ => append_temple(&mut m, b, t, Some(-t)),
                }
                assert!(m.positions.iter().all(|p| p[0] >= b[0]
                    && p[0] <= b[1]
                    && p[2] >= b[2]
                    && p[2] <= b[3]
                    && p[1] >= 0.
                    && p[1] <= t * 4.5));
                for tri in m.indices.chunks_exact(3) {
                    let p =
                        [tri[0], tri[1], tri[2]].map(|i| Vec3::from_array(m.positions[i as usize]));
                    assert!(
                        (p[1] - p[0])
                            .cross(p[2] - p[0])
                            .dot(Vec3::from_array(m.normals[tri[0] as usize]))
                            > 0.
                    );
                }
            }
        }
    }

    #[test]
    fn wooden_house_stays_inside_source_plot_with_outward_faces() {
        for t in [8., 16.] {
            let b = [-4. * t, 4. * t, -3. * t, 3. * t];
            let mut mesh = SurfaceMeshData::default();
            append(&mut mesh, b, t, Some(-t));
            assert!(mesh.positions.iter().all(|p| p[0] >= b[0]
                && p[0] <= b[1]
                && p[2] >= b[2]
                && p[2] <= b[3]
                && p[1] >= 0.));
            for tri in mesh.indices.chunks_exact(3) {
                let p =
                    [tri[0], tri[1], tri[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
                let n = Vec3::from_array(mesh.normals[tri[0] as usize]);
                assert!((p[1] - p[0]).cross(p[2] - p[0]).dot(n) > 0.);
            }
        }
    }
}
