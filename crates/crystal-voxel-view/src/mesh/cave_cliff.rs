//! Joined rock geometry, rather than a color treatment on rectangular banks.
//! Every boundary uses the same map-space deformation, including corners and
//! changed-height seams. Walkable shelves at or below six pixels stay exact.
use super::*;
#[path = "cave_corners.rs"]
mod corners;
pub(super) use corners::RockCorners;

// Continuous planar mineral slabs. Adjacent lattice triangles share their
// edge samples, so fractures change the slope without tearing the surface.
pub(super) fn slab(x: f32, z: f32) -> f32 {
    let ix = x.floor();
    let iz = z.floor();
    let u = x - ix;
    let v = z - iz;
    let noise = new_bark::meadow_noise;
    let a = noise(ix + 83., iz - 29.);
    let b = noise(ix + 84., iz - 29.);
    let c = noise(ix + 83., iz - 28.);
    let d = noise(ix + 84., iz - 28.);
    if u + v <= 1. {
        a + (b - a) * u + (c - a) * v
    } else {
        d + (c - d) * (1. - u) + (b - d) * (1. - v)
    }
}

pub(super) fn rock_point(p: Vec3, g: &GridGeometry, origin: [i32; 2]) -> Vec3 {
    let y = p.y / g.tile_height * SOURCE_TILE_HEIGHT;
    let strength = ((y - crate::cave::CAVE_SHELF_HEIGHT) / 16.0).clamp(0.0, 1.0);
    if strength == 0.0 {
        return p;
    }
    let x = (p.x - g.origin_x) / g.tile_width + origin[0] as f32;
    let z = (p.z - g.origin_z) / g.tile_height + origin[1] as f32;
    let noise = new_bark::meadow_noise;
    // Joined beds have overhanging lips and recessed seams. The shared field
    // carries those formations through wall corners without attached props.
    let bed = (y + slab(x * 0.24, z * 0.24) * 3.2 + noise(x * 0.10 + 19., z * 0.10) * 1.5) / 7.5;
    // A thin recessed seam, a stepped lip, then a broad sloping slab face.
    // Linear sections replace the repeating rounded sinusoidal pillows.
    let course = bed.rem_euclid(1.0);
    let shoulder = ((course - 0.10) / 0.18).clamp(0., 1.);
    let slope = 1. - ((course - 0.46) / 0.54).clamp(0., 1.);
    let lip = shoulder * slope * 0.32 - 0.16;
    // Beds terminate inside larger broken formations. A positive minimum
    // made every course run across the room as a repeating horizontal stripe.
    let continuity = (0.10 + noise(x * 0.48 + y * 0.085, z * 0.48 + 31.) * 1.8)
        .clamp(0., 1.);
    let layer = lip * continuity;
    // Fractures span several source cells, creating broad broken rock faces
    // through the full height of the bank. Smaller, densely repeated facets
    // made the long Union Cave walls read as embossed rectangular masonry.
    // Keep the horizontal gradient bounded while widening the mineral planes.
    let fracture = slab(x * 0.32 + y * 0.09, z * 0.27 - y * 0.08) * 0.70;
    // Broad, unequal masses carry the outline above the trainer. Their lower
    // frequency bounds horizontal gradients while adding deeper wall relief.
    let crown = ((y - 22.) / 18.).clamp(0., 1.);
    let crest = slab(x * 0.24 + 57., z * 0.24 - 13.) * 1.35 * crown;
    // Narrow vertical joints are actual recessed geometry. Independent
    // fields cut both wall orientations and continue through their corners.
    let joint = |a: f32, b: f32| {
        let distance = noise(a * 0.88 + y * 0.009, b * 0.88).abs();
        (1. - distance / 0.24).clamp(0., 1.).powi(2) * 0.06
    };
    let cleft_x = joint(x + 19., z);
    let cleft_z = joint(z - 23., x);
    // Combined beds and fractures must retain their outward face order.
    // Excess lateral relief folded thin courses backward into visible spikes
    // when the foreground cutaway exposed their lower faces.
    let lateral = 0.60;
    p + Vec3::new(
        (noise(x * 0.25 + y * 0.035, z * 0.25) * 1.05 + layer + fracture - cleft_x) * g.tile_width * lateral,
        (noise(x * 0.32, z * 0.32) * 0.85 + slab(x * 0.7, z * 0.7) * 0.95 + crest) * g.tile_height,
        (noise(z * 0.25 - y * 0.035 + 31.0, x * 0.25) * 1.05 - layer - fracture - cleft_z)
            * g.tile_height * lateral,
    ) * strength
}

/// Mineral pigment is sampled in map space, independently of the camera and
/// directional light. The same sample paints joined caps and borrowed floors.
pub(super) fn pigment(
    base: [f32; 4],
    p: Vec3,
    g: &GridGeometry,
    origin: [i32; 2],
    frozen: bool,
) -> [f32; 4] {
    let x = (p.x - g.origin_x) / g.tile_width + origin[0] as f32;
    let z = (p.z - g.origin_z) / g.tile_height + origin[1] as f32;
    let y = p.y / g.tile_height;
    let noise = new_bark::meadow_noise;
    let mottling = noise(x * 0.36 + y * 0.06, z * 0.36) * 0.13;
    let grain = noise(x * 1.9 + y * 0.4 + 17., z * 1.9 - 11.) * 0.014;
    let inclusion = (noise(x * 0.8 + y * 0.13, z * 0.8 + 37.) * 2.).max(0.);
    let mineral = (0.09 - noise(x * 0.72 + y * 0.09, z * 0.72 - y * 0.08).abs()).max(0.) / 0.09;
    let tint = if frozen {
        [0.005, 0.022, 0.032]
    } else {
        [0.042, 0.020, -0.009]
    };
    let variation = mottling + grain;
    let result = [
        (base[0] + variation + tint[0] * mineral - inclusion * 0.016).clamp(0., 1.),
        (base[1] + variation + tint[1] * mineral - inclusion * 0.020).clamp(0., 1.),
        (base[2] + variation + tint[2] * mineral - inclusion * 0.023).clamp(0., 1.),
        base[3],
    ];
    if frozen && y < 1.2 {
        // Frozen mineral dust at the wall foot shares the adjacent floor's
        // cold palette. Feather the pigment; retain the exact wall geometry.
        let t = (1. - y / (0.80 + noise(x * 1.2 + 43., z * 1.2 - 17.) * 0.20)).clamp(0., 1.);
        let t = t * t * (3. - 2. * t) * 0.75;
        let c = Vec3::new(result[0], result[1], result[2]).lerp(Vec3::new(0.39, 0.51, 0.54), t);
        return c.extend(base[3]).to_array();
    }
    result
}

// A distinct positive UV domain keeps cave paint out of outdoor rim shaping
// and the live atlas/water sampler. Coordinates stay fixed in source-map space.
pub(super) fn rock_uv(p: Vec3, g: &GridGeometry, origin: [i32; 2], frozen: bool) -> [f32; 2] {
    [
        100000. + p.x - g.origin_x + origin[0] as f32 * g.tile_width,
        if frozen { 300000. } else { 200000. } + p.z - g.origin_z
            + origin[1] as f32 * g.tile_height,
    ]
}

pub(super) fn floor_uv(p: Vec3, g: &GridGeometry, origin: [i32; 2], frozen: bool) -> [f32; 2] {
    let mut uv = rock_uv(p, g, origin, frozen);
    uv[1] += 200000.;
    uv
}

/// Authenticated low rock formations use the same mineral coordinates, but their
/// complete silhouettes do not participate in enclosing-wall cutaways.
pub(super) fn stone_uv(p: Vec3, g: &GridGeometry, origin: [i32; 2], frozen: bool) -> [f32; 2] {
    let mut uv = rock_uv(p, g, origin, frozen);
    uv[1] += 400000.;
    uv
}

/// Join shallow faces before cutting their recesses. The weathering belongs
/// to the whole exposed bank, not a repeated bevel on every source tile.
pub(super) fn scarps(
    out: &mut SurfaceMeshData,
    mut faces: Vec<([[f32; 3]; 4], [f32; 3], [f32; 4])>,
    g: &GridGeometry,
    origin: [i32; 2],
) {
    let key = |p: [[f32; 3]; 4], n: [f32; 3]| {
        let axis = if n[0] == 0. { 0 } else { 2 };
        let plane_axis = 2 - axis;
        let lo = p.iter().map(|p| p[axis]).fold(f32::INFINITY, f32::min);
        let hi = p.iter().map(|p| p[axis]).fold(f32::NEG_INFINITY, f32::max);
        (axis, p[0][plane_axis], lo, hi)
    };
    faces.sort_by(|a, b| {
        let ka = key(a.0, a.1);
        let kb = key(b.0, b.1);
        ka.0.cmp(&kb.0)
            .then(a.1[0].total_cmp(&b.1[0]))
            .then(a.1[2].total_cmp(&b.1[2]))
            .then(ka.1.total_cmp(&kb.1))
            .then(ka.2.total_cmp(&kb.2))
    });
    let mut i = 0;
    while i < faces.len() {
        let (mut p, n, color) = faces[i];
        let (axis, plane, lo, mut hi) = key(p, n);
        i += 1;
        while i < faces.len() {
            let (next, nn, nc) = faces[i];
            let (na, np, nl, nh) = key(next, nn);
            if na != axis || np != plane || nn != n || nc != color || (nl - hi).abs() > 0.001 {
                break;
            }
            hi = nh;
            i += 1;
        }
        let old_hi = key(p, n).3;
        for v in &mut p {
            if v[axis] == old_hi {
                v[axis] = hi;
            }
        }
        let height = crate::cave::CAVE_SHELF_HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT;
        let first = out.positions.len();
        quad_with_point_step(
            out,
            p,
            color,
            g,
            origin,
            |v, g, origin| {
                let t = (v.y / height).clamp(0., 1.);
                if t == 0. || t == 1. {
                    return v;
                }
                // A broken shoulder slopes out from the exact walking lip,
                // replacing the sheer rectangular top edge. Its lower bed
                // remains recessed; floor, cap and true end joins stay exact.
                let bed = (t / 0.32).min((1. - t) / 0.28).clamp(0., 1.);
                let shoulder = if t >= 2. / 3. {
                    // A circular upper shoulder meets the unchanged cap
                    // tangentially, instead of ending a triangular bevel
                    // against the walking plane.
                    (1. - ((t - 2. / 3.) * 3.).powi(2)).max(0.).sqrt()
                } else {
                    let rise = ((t - 0.40) / (2. / 3. - 0.40)).clamp(0., 1.);
                    rise * rise * (3. - 2. * rise)
                };
                // A one-cell bank needs the same shoulder as a long bank.
                // Confine the join taper to half a tile and meet both the
                // exact end plane and the full profile with zero slope.
                let end = ((v[axis] - lo).min(hi - v[axis]) / (g.tile_width * 0.5)).clamp(0., 1.);
                let end = end * end * (3. - 2. * end);
                let x = (v.x - g.origin_x) / g.tile_width + origin[0] as f32;
                let z = (v.z - g.origin_z) / g.tile_height + origin[1] as f32;
                let chip = 0.70 + slab(x * 1.4, z * 1.4) * 0.55;
                let width = 0.29 + slab(x * 0.65 + 31., z * 0.65 - 17.) * 0.08;
                v + Vec3::from_array(n) * end * g.tile_width
                    * (shoulder * width - bed * chip * 0.12)
            },
            false,
            g.tile_height * 0.125,
        );
        for j in first..out.positions.len() {
            // Six-pixel banks cannot hide the trainer; keep their complete
            // recessed body alongside the cap during the tall-wall cutaway.
            out.uvs[j] = stone_uv(Vec3::from_array(out.positions[j]), g, origin, false);
        }
        // Sharp changes between the broken shoulder and recessed bed need
        // the actual facet normal, rather than a tangent sampled at its lip.
        for start in (first..out.positions.len()).step_by(3) {
            let points: [Vec3; 3] = std::array::from_fn(|i| Vec3::from_array(out.positions[start+i]));
            let normal = (points[1]-points[0]).cross(points[2]-points[0]).normalize();
            let shade = 0.60 + 0.40 * normal.dot(Vec3::new(-0.4,0.8,0.45).normalize()).max(0.);
            for i in 0..3 {
                out.normals[start+i] = normal.to_array();
                let pigment = pigment(color, points[i], g, origin, false);
                out.colors[start+i] = [pigment[0]*shade, pigment[1]*shade, pigment[2]*shade, pigment[3]];
            }
        }
    }
}

fn triangle(
    out: &mut SurfaceMeshData,
    p: [Vec3; 3],
    color: [f32; 4],
    g: &GridGeometry,
    origin: [i32; 2],
) {
    let cross = (p[1] - p[0]).cross(p[2] - p[0]);
    if cross.length_squared() < 1e-8 {
        return;
    }
    let normal = cross.normalize();
    let shade = 0.55 + 0.45 * normal.dot(Vec3::new(-0.4, 0.8, 0.45).normalize()).max(0.0);
    let base = out.positions.len() as u32;
    out.positions.extend(p.map(|v| v.to_array()));
    out.normals.extend([normal.to_array(); 3]);
    out.uvs.extend(p.map(|p| rock_uv(p, g, origin, false)));
    out.colors.extend(p.map(|p| {
        let color = pigment(color, p, g, origin, false);
        [
            color[0] * shade,
            color[1] * shade,
            color[2] * shade,
            color[3],
        ]
    }));
    out.indices.extend([base, base + 1, base + 2]);
}

pub(super) fn quad(
    out: &mut SurfaceMeshData,
    points: [[f32; 3]; 4],
    color: [f32; 4],
    g: &GridGeometry,
    origin: [i32; 2],
) {
    quad_with_point(out, points, color, g, origin, rock_point, false);
}

pub(super) fn quad_with_point<F: Fn(Vec3, &GridGeometry, [i32; 2]) -> Vec3>(
    out: &mut SurfaceMeshData,
    points: [[f32; 3]; 4],
    color: [f32; 4],
    g: &GridGeometry,
    origin: [i32; 2],
    point: F,
    frozen: bool,
) {
    quad_with_point_step(out, points, color, g, origin, point, frozen, g.tile_height * 0.25);
}

fn quad_with_point_step<F: Fn(Vec3, &GridGeometry, [i32; 2]) -> Vec3>(
    out: &mut SurfaceMeshData,
    points: [[f32; 3]; 4],
    color: [f32; 4],
    g: &GridGeometry,
    origin: [i32; 2],
    point: F,
    frozen: bool,
    step: f32,
) {
    let p = points.map(Vec3::from_array);
    let painted = p
        .iter()
        .any(|p| p.y >= 16. * g.tile_height / SOURCE_TILE_HEIGHT);
    // Low vertical closures are rock, including the triangular ends of a
    // stair ramp. Floor paint belongs only to the walking plane. Choose the
    // original face orientation so fractured normals cannot change material.
    let vertical = (p[1] - p[0]).cross(p[3] - p[0])
        .try_normalize()
        .or_else(|| (p[1] - p[0]).cross(p[2] - p[0]).try_normalize())
        .is_some_and(|n| n.y.abs() < 0.001);
    // Refine the curved cross-section while retaining the cap's existing
    // quarter-tile edge samples, regardless of the face's vertex order.
    let edge_step = |delta: Vec3| if delta.y.abs() > 0.001 {
        step
    } else {
        g.tile_height * 0.25
    };
    let nu = ((p[3] - p[0])
        .abs()
        .max_element()
        .max((p[2] - p[1]).abs().max_element())
        / edge_step(p[3] - p[0]))
        .ceil()
        .max(1.0) as usize;
    let nv = ((p[1] - p[0])
        .abs()
        .max_element()
        .max((p[2] - p[3]).abs().max_element())
        / edge_step(p[1] - p[0]))
        .ceil()
        .max(1.0) as usize;
    // Neighboring walls can start on different walking shelves. Sample their
    // common vertical edge at the same absolute heights, rather than splitting
    // each wall into an unrelated number of equal bands (which opens cracks).
    let parameters = |count: usize, from: f32, to: f32, vertical: bool| {
        if !vertical {
            return (0..=count)
                .map(|i| i as f32 / count as f32)
                .collect::<Vec<_>>();
        }
        let mut values = vec![0.0, 1.0];
        for course in (from.min(to) / step).floor() as i32..=(from.max(to) / step).ceil() as i32 {
            let t = (course as f32 * step - from) / (to - from);
            if t > 0.00001 && t < 0.99999 {
                values.push(t);
            }
        }
        values.sort_by(f32::total_cmp);
        values
    };
    let vertical_u = p[0].y == p[1].y && p[3].y == p[2].y && p[0].y != p[3].y;
    let vertical_v = p[0].y == p[3].y && p[1].y == p[2].y && p[0].y != p[1].y;
    let us = parameters(
        nu,
        p[0].y,
        p[3].y,
        vertical_u,
    );
    let vs = parameters(
        nv,
        p[0].y,
        p[1].y,
        vertical_v,
    );
    let original_at = |u: usize, v: usize| {
        let mut original = p[0].lerp(p[3], us[u]).lerp(p[1].lerp(p[2], us[u]), vs[v]);
        // Fractions reconstructed by lerp can differ by a few ulps when
        // adjoining walls start on different shelves. Use the absolute
        // course height for interior knots; authored endpoints stay exact.
        if (vertical_u && u > 0 && u + 1 < us.len())
            || (vertical_v && v > 0 && v + 1 < vs.len())
        {
            original.y = (original.y / step).round() * step;
        }
        original
    };
    let at = |u: usize, v: usize| point(original_at(u, v), g, origin);
    let normal_at = |u: usize, v: usize| {
        let original = original_at(u, v);
        // A ramp can terminate at the cap and collapse one quad edge to
        // a point. Use the opposite edge's tangent at that endpoint so the
        // surviving triangle still has a finite, outward surface normal.
        let tangent = |a: Vec3, b: Vec3, t: f32| {
            a.lerp(b, t)
                .try_normalize()
                .or_else(|| a.try_normalize())
                .or_else(|| b.try_normalize())
                .unwrap_or(Vec3::X)
        };
        let du = tangent(p[3] - p[0], p[2] - p[1], vs[v]);
        let dv = tangent(p[1] - p[0], p[2] - p[3], us[u]);
        let epsilon = g.tile_height * 0.01;
        let a =
            point(original + dv * epsilon, g, origin) - point(original - dv * epsilon, g, origin);
        let b =
            point(original + du * epsilon, g, origin) - point(original - du * epsilon, g, origin);
        a.cross(b).normalize()
    };
    for v in 0..vs.len() - 1 {
        for u in 0..us.len() - 1 {
            let q = [at(u, v), at(u, v + 1), at(u + 1, v + 1), at(u + 1, v)];
            let normals = [
                normal_at(u, v),
                normal_at(u, v + 1),
                normal_at(u + 1, v + 1),
                normal_at(u + 1, v),
            ];
            for ids in [[0, 1, 2], [0, 2, 3]] {
                let points = ids.map(|i| q[i]);
                let cross = (points[1] - points[0]).cross(points[2] - points[0]);
                if cross.length_squared() < 1e-8 {
                    continue;
                }
                let first = out.positions.len() as u32;
                out.positions.extend(points.map(|p| p.to_array()));
                out.normals.extend(ids.map(|i| normals[i].to_array()));
                out.uvs.extend(points.map(|p| {
                    if painted {
                        rock_uv(p, g, origin, frozen)
                    } else if vertical {
                        stone_uv(p, g, origin, frozen)
                    } else {
                        floor_uv(p, g, origin, frozen)
                    }
                }));
                out.colors.extend(ids.map(|i| {
                    let color = pigment(color, q[i], g, origin, frozen);
                    let shade = 0.60
                        + 0.40
                            * normals[i]
                                .dot(Vec3::new(-0.4, 0.8, 0.45).normalize())
                                .max(0.);
                    [
                        color[0] * shade,
                        color[1] * shade,
                        color[2] * shade,
                        color[3],
                    ]
                }));
                out.indices.extend([first, first + 1, first + 2]);
            }
        }
    }
}

pub(super) fn cap_triangle(
    out: &mut SurfaceMeshData,
    points: [[f32; 3]; 3],
    color: [f32; 4],
    g: &GridGeometry,
    origin: [i32; 2],
) {
    cap_triangle_with_point(out, points, color, g, origin, rock_point);
}

pub(super) fn cap_triangle_with_point<F: Fn(Vec3, &GridGeometry, [i32;2]) -> Vec3>(
    out: &mut SurfaceMeshData, points: [[f32;3];3], color: [f32;4], g: &GridGeometry, origin: [i32;2], point: F,
) {
    let p = points.map(Vec3::from_array);
    let n = ((p[1] - p[0])
        .abs()
        .max_element()
        .max((p[2] - p[0]).abs().max_element())
        / (g.tile_height * 0.25))
        .ceil()
        .max(1.0) as usize;
    let at = |u: usize, v: usize| {
        point(
            p[0] + (p[1] - p[0]) * (u as f32 / n as f32) + (p[2] - p[0]) * (v as f32 / n as f32),
            g,
            origin,
        )
    };
    for v in 0..n {
        for u in 0..n - v {
            triangle(
                out,
                [at(u, v), at(u + 1, v), at(u, v + 1)],
                color,
                g,
                origin,
            );
            if u + v + 1 < n {
                triangle(
                    out,
                    [at(u + 1, v), at(u + 1, v + 1), at(u, v + 1)],
                    color,
                    g,
                    origin,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cave_relief_caps_keep_upward_winding_across_large_map_coordinates() {
        let g = GridGeometry { width: 80, height: 144, tile_width: 8., tile_height: 8.,
            origin_x: -320., origin_z: -576. };
        for z in 0..g.height * 4 { for x in 0..g.width * 4 { for y in [16., 40., 48.] {
            let p = Vec3::new(g.origin_x + x as f32 * 2., y,
                g.origin_z + z as f32 * 2.);
            let a = rock_point(p, &g, [0,0]);
            let b = rock_point(p + Vec3::Z * 2., &g, [0,0]);
            let c = rock_point(p + (Vec3::X + Vec3::Z) * 2., &g, [0,0]);
            let d = rock_point(p + Vec3::X * 2., &g, [0,0]);
            for normal in [(b-a).cross(c-a), (c-a).cross(d-a)] {
                assert!(normal.y > 0.,
                    "rock cap folds at quarter-cell knot {x},{z}, height {y}: {normal:?}");
            }
        }}}
    }
    #[test]
    fn cave_relief_does_not_fold_vertical_faces_across_large_map_coordinates() {
        // Covers the combined authored source extent of all 32 stone caves
        // in the local pack, including Mount Mortar and Victory Road.
        let g = GridGeometry { width: 80, height: 144, tile_width: 8., tile_height: 8.,
            origin_x: -320., origin_z: -576. };
        for z in 0..g.height * 4 { for x in 0..g.width * 4 { for y in (6..48).step_by(2) {
            let p = Vec3::new(g.origin_x + x as f32 * 2., y as f32,
                g.origin_z + z as f32 * 2.);
            for (along, outward) in [(Vec3::X, Vec3::Z), (Vec3::Z, Vec3::NEG_X)] {
                let a = rock_point(p, &g, [0,0]);
                let b = rock_point(p + along * 2., &g, [0,0]);
                let c = rock_point(p + along * 2. + Vec3::Y * 2., &g, [0,0]);
                let d = rock_point(p + Vec3::Y * 2., &g, [0,0]);
                for normal in [(b-a).cross(c-a), (c-a).cross(d-a)] {
                    assert!(normal.dot(outward) > 0.,
                        "rock face folds at quarter-cell knot {x},{z}, height {y}: {normal:?}");
                }
            }
        }}}
    }
    #[test]
    fn cave_relief_diagonal_faces_keep_outward_winding_across_large_maps() {
        let g = GridGeometry { width: 80, height: 144, tile_width: 8., tile_height: 8.,
            origin_x: -320., origin_z: -576. };
        for z in 0..g.height*4 { for x in 0..g.width*4 { for y in (6..48).step_by(2) {
            let p=Vec3::new(g.origin_x+x as f32*2.,y as f32,g.origin_z+z as f32*2.);
            for along in [Vec3::X+Vec3::Z,Vec3::X-Vec3::Z] {
                let outward=along.cross(Vec3::Y);
                let a=rock_point(p,&g,[0,0]);
                let b=rock_point(p+along*2.,&g,[0,0]);
                let c=rock_point(p+along*2.+Vec3::Y*2.,&g,[0,0]);
                let d=rock_point(p+Vec3::Y*2.,&g,[0,0]);
                for normal in [(b-a).cross(c-a),(c-a).cross(d-a)] {
                    assert!(normal.dot(outward)>0.,
                        "diagonal rock folds at {x},{z}, height {y}, direction {along:?}: {normal:?}");
                }
            }
        }}}
    }
    fn grid() -> GridGeometry {
        GridGeometry {
            width: 2,
            height: 2,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        }
    }
    #[test]
    fn shallow_scarp_is_one_weathered_bank_with_exact_floor_cap_and_end_joins() {
        let g = grid();
        let color = [0.34, 0.345, 0.31, 1.];
        let face = |w: f32, e: f32| [[w, 0., 8.], [e, 0., 8.], [e, 6., 8.], [w, 6., 8.]];
        let mut joined = SurfaceMeshData::default();
        scarps(
            &mut joined,
            vec![
                (face(8., 16.), [0., 0., 1.], color),
                (face(0., 8.), [0., 0., 1.], color),
            ],
            &g,
            [10, 20],
        );
        let mut whole = SurfaceMeshData::default();
        scarps(
            &mut whole,
            vec![(face(0., 16.), [0., 0., 1.], color)],
            &g,
            [10, 20],
        );
        assert_eq!(joined.positions, whole.positions);
        assert_eq!(joined.normals, whole.normals);
        assert_eq!(joined.uvs, whole.uvs);
        assert!(joined.uvs.iter().all(|uv| (uv[1] / 100000.).round() == 6.));
        assert!(joined
            .positions
            .iter()
            .any(|p| p[0] == 8. && p[1] == 2. && p[2] < 7.5));
        assert!(joined.positions.iter().any(|p| p[0] == 8. && p[1] == 4. && p[2] > 9.));
        assert!(joined.positions.iter().any(|p| p[0] == 8. && p[1] == 5. && p[2] > 9.),
            "the upper circular shoulder must have its own sample before the exact cap");
        // The shoulder stays short of the lower cell's walking center.
        assert!(joined.positions.iter().all(|p| p[2] < 12.));
        for p in &joined.positions {
            assert!((0. ..=6.).contains(&p[1]));
            if p[1] == 0. || p[1] == 6. || p[0] == 0. || p[0] == 16. {
                assert_eq!(p[2], 8.);
            }
        }
        for ids in joined.indices.chunks_exact(3) {
            let p = ids
                .iter()
                .map(|&i| Vec3::from_array(joined.positions[i as usize]))
                .collect::<Vec<_>>();
            assert!((p[1] - p[0]).cross(p[2] - p[0]).z > 0.);
        }
        assert!(joined
            .normals
            .iter()
            .all(|n| Vec3::from_array(*n).is_finite() && n[2] > 0.));
    }
    #[test]
    fn short_cave_ledge_has_the_full_rounded_shoulder_and_exact_end_joins() {
        for scale in [1.,2.] {
            let g=GridGeometry {tile_width:8.*scale,tile_height:8.*scale,
                origin_x:-32.*scale,origin_z:48.*scale,..grid()};
            let face=|w:f32,e:f32| [[w*scale,0.,8.*scale],[e*scale,0.,8.*scale],
                [e*scale,6.*scale,8.*scale],[w*scale,6.*scale,8.*scale]];
            let build=|w,e| {let mut out=SurfaceMeshData::default();
                scarps(&mut out,vec![(face(w,e),[0.,0.,1.],[0.34,0.345,0.31,1.])],&g,[17,33]);out};
            let short=build(0.,8.);
            let long=build(-4.,12.);
            let section=|mesh:&SurfaceMeshData| {
                let mut points:Vec<_>=mesh.positions.iter().copied().filter(|p|p[0]==4.*scale).collect();
                points.sort_by(|a,b|a[1].total_cmp(&b[1]).then(a[2].total_cmp(&b[2])));
                points.dedup();points
            };
            assert_eq!(section(&short),section(&long),
                "short shelf sections must retain the long shelf's complete shoulder");
            for p in &short.positions {
                assert!(p[2]<12.*scale,"the lower walking center stays clear");
                if p[1]==0. || p[1]==6.*scale || p[0]==0. || p[0]==8.*scale {
                    assert_eq!(p[2],8.*scale,"every authored landing and end seam stays exact");
                }
            }
            for ids in short.indices.chunks_exact(3) {
                let [a,b,c]=[ids[0],ids[1],ids[2]].map(|i|Vec3::from_array(short.positions[i as usize]));
                assert!((b-a).cross(c-a).z>0.);
            }
        }
    }

    #[test]
    fn ramp_termination_emits_only_finite_outward_faces() {
        let g = GridGeometry {
            width: 1,
            height: 1,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut mesh = SurfaceMeshData::default();
        quad_with_point(
            &mut mesh,
            [[8., 6., 8.], [0., 0., 8.], [0., 6., 8.], [8., 6., 8.]],
            [0.34, 0.345, 0.31, 1.],
            &g,
            [0, 0],
            |p, _, _| p,
            false,
        );
        assert!(!mesh.indices.is_empty());
        assert!(mesh
            .positions
            .iter()
            .all(|p| Vec3::from_array(*p).is_finite()));
        assert!(mesh
            .normals
            .iter()
            .all(|n| Vec3::from_array(*n).is_finite() && n[2] < -0.999));
        for tri in mesh.indices.chunks_exact(3) {
            let p = [tri[0], tri[1], tri[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
            assert!((p[1] - p[0]).cross(p[2] - p[0]).z < 0.);
        }
        assert!(mesh.positions.iter().any(|p| *p == [8., 6., 8.]));
        assert!(mesh
            .uvs
            .iter()
            .all(|uv| uv[0] > 90000. && (uv[1] / 100000.).round() == 6.));
    }

    #[test]
    fn rock_caps_do_not_fold_in_the_horizontal_plane() {
        let g = grid();
        let step = g.tile_width * 0.125;
        for y in [32., 40., 48.] {
            for z in 0..128 {
                for x in 0..128 {
                    let p = Vec3::new(x as f32 * step, y, z as f32 * step);
                    let a = rock_point(p, &g, [10, 20]);
                    let b = rock_point(p + Vec3::Z * step, &g, [10, 20]);
                    let c = rock_point(p + Vec3::X * step, &g, [10, 20]);
                    assert!((b - a).cross(c - a).y > 0., "fold at {p:?}");
                }
            }
        }
    }

    #[test]
    fn tall_cave_crests_remain_closed_across_full_map_coordinates() {
        let g = grid();
        let step = g.tile_width * 0.5;
        for height in [32., 40., 48.] {
            for z in -32..160 {
                for x in -32..128 {
                    let p = Vec3::new(x as f32 * step, height, z as f32 * step);
                    let a = rock_point(p, &g, [0, 0]);
                    let b = rock_point(p + Vec3::Z * step, &g, [0, 0]);
                    let c = rock_point(p + Vec3::X * step, &g, [0, 0]);
                    assert!((b-a).cross(c-a).y > 0., "folded crest at {p:?}");
                    assert!(a.y > 20., "enclosing crown dropped near the walking floor at {p:?}");
                }
            }
        }
    }

    #[test]
    fn mineral_pigment_keeps_world_registration_and_original_alpha() {
        let g = grid();
        let mut shifted = g;
        shifted.origin_x = -16.;
        shifted.origin_z = 8.;
        for frozen in [false, true] {
            for x in 0..16 {
                let p = Vec3::new(x as f32 * 8., 24., 16.);
                let base = [0.34, 0.345, 0.31, 0.7];
                let a = pigment(base, p, &g, [10, 20], frozen);
                let b = pigment(
                    base,
                    p + Vec3::new(-16., 0., 8.),
                    &shifted,
                    [10, 20],
                    frozen,
                );
                assert_eq!(a, b);
                assert_eq!(
                    rock_uv(p, &g, [10, 20], frozen),
                    rock_uv(p + Vec3::new(-16., 0., 8.), &shifted, [10, 20], frozen)
                );
                assert_eq!(
                    floor_uv(p, &g, [10, 20], frozen),
                    floor_uv(p + Vec3::new(-16., 0., 8.), &shifted, [10, 20], frozen)
                );
                assert_eq!(
                    stone_uv(p, &g, [10, 20], frozen),
                    stone_uv(p + Vec3::new(-16., 0., 8.), &shifted, [10, 20], frozen)
                );
                assert_eq!(a[3], base[3]);
                assert!(a.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v)));
            }
        }
    }
    #[test]
    fn low_vertical_closures_use_rock_while_walking_caps_keep_floor_paint() {
        let g = grid();
        for frozen in [false, true] {
            for (points, category) in [
                ([[0., 0., 8.], [8., 0., 8.], [8., 6., 8.], [0., 6., 8.]], if frozen { 7. } else { 6. }),
                ([[0., 6., 0.], [0., 6., 8.], [8., 6., 8.], [8., 6., 0.]], if frozen { 5. } else { 4. }),
            ] {
                let mut mesh = SurfaceMeshData::default();
                quad_with_point(&mut mesh, points, [0.34, 0.345, 0.31, 1.], &g, [0, 0], |p, _, _| p, frozen);
                assert!(!mesh.indices.is_empty());
                assert!(mesh.uvs.iter().all(|uv| (uv[1] / 100000.).round() == category));
                for point in points {
                    assert!(mesh.positions.contains(&point));
                }
                assert!(mesh.positions.iter().all(|p| (0. ..=6.).contains(&p[1])));
            }
        }
    }
    #[test]
    fn rock_and_floor_paint_tags_remain_distinct_outside_the_source_map() {
        let g = grid();
        for frozen in [false, true] {
            for z in [-320., -8., 0., 8., 320.] {
                let p = Vec3::new(24., 0., z);
                let rock = rock_uv(p, &g, [0, 0], frozen);
                let floor = floor_uv(p, &g, [0, 0], frozen);
                let stone = stone_uv(p, &g, [0, 0], frozen);
                let rock_tag = if frozen { 3. } else { 2. };
                assert_eq!((rock[1] / 100000.).round(), rock_tag);
                assert_eq!((floor[1] / 100000.).round(), rock_tag + 2.);
                assert_eq!((stone[1] / 100000.).round(), rock_tag + 4.);
                assert_eq!(rock[0], floor[0]);
                assert_eq!(rock[1] - rock_tag * 100000., z);
                assert_eq!(floor[1] - (rock_tag + 2.) * 100000., z);
                assert_eq!(stone[1] - (rock_tag + 4.) * 100000., z);
            }
        }
    }
    #[test]
    fn cliff_has_real_relief_while_shelf_and_ground_are_exact() {
        let g = grid();
        for y in [0.0, 3.0, 6.0] {
            let p = Vec3::new(4.0, y, 5.0);
            assert_eq!(rock_point(p, &g, [10, 20]), p);
        }
        let mut out = SurfaceMeshData::default();
        quad(
            &mut out,
            [[0., 0., 0.], [0., 40., 0.], [8., 40., 0.], [8., 0., 0.]],
            [0.5; 4],
            &g,
            [10, 20],
        );
        assert!(out.positions.iter().any(|p| p[2].abs() > 0.5));
        assert!(out.uvs.iter().all(|uv| uv[0] > 90000. && uv[1] < 250000.));
        assert!(out.positions.iter().any(|p| p[1] > 40.5));
        assert!(out
            .normals
            .iter()
            .all(|n| (Vec3::from_array(*n).length() - 1.0).abs() < 1e-4));
    }
    #[test]
    fn joined_caps_and_walls_share_their_transformed_edge() {
        let g = grid();
        let mut cap = SurfaceMeshData::default();
        let mut wall = SurfaceMeshData::default();
        quad(
            &mut cap,
            [[0., 40., 0.], [0., 40., 8.], [8., 40., 8.], [8., 40., 0.]],
            [0.5; 4],
            &g,
            [10, 20],
        );
        quad(
            &mut wall,
            [[0., 0., 0.], [0., 40., 0.], [8., 40., 0.], [8., 0., 0.]],
            [0.5; 4],
            &g,
            [10, 20],
        );
        let mut corner_cap = SurfaceMeshData::default();
        cap_triangle(
            &mut corner_cap,
            [[0., 40., 0.], [8., 40., 0.], [0., 40., 8.]],
            [0.5; 4],
            &g,
            [10, 20],
        );
        for x in [0., 2., 4., 6., 8.] {
            let edge = rock_point(Vec3::new(x, 40., 0.), &g, [10, 20]).to_array();
            assert!(cap.positions.contains(&edge));
            assert!(wall.positions.contains(&edge));
            assert!(corner_cap.positions.contains(&edge));
        }
        let mut scrolled = g;
        scrolled.origin_x = -8.0;
        let a = rock_point(Vec3::new(4., 40., 3.), &g, [10, 20]);
        let b = rock_point(Vec3::new(-4., 40., 3.), &scrolled, [10, 20]);
        assert!((a - b - Vec3::X * 8.0).length() < 1e-5);
    }
    #[test]
    fn neighboring_wall_bases_use_the_same_vertical_rock_knots() {
        let g = grid();
        let mut a = SurfaceMeshData::default();
        let mut b = SurfaceMeshData::default();
        quad(
            &mut a,
            [[0., 0., 0.], [0., 40., 0.], [8., 40., 0.], [8., 0., 0.]],
            [0.5; 4],
            &g,
            [10, 20],
        );
        quad(
            &mut b,
            [[8., 6., 0.], [8., 40., 0.], [16., 40., 0.], [16., 6., 0.]],
            [0.5; 4],
            &g,
            [10, 20],
        );
        for y in (8..=40).step_by(4) {
            let p = rock_point(Vec3::new(8., y as f32, 0.), &g, [10, 20]).to_array();
            assert!(a.positions.contains(&p));
            assert!(b.positions.contains(&p));
        }
    }
}
