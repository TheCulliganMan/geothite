//! Continuous frozen shelf faces in place of repeated pebble masonry.
//! The source-complete adapter owns the footprint, stairs and support datum.
use super::*;
use crate::exterior_models::{Kind, model};

fn exposed_edges(id: u16) -> &'static [(usize, f32, f32)] {
    match id {
        0x04 => &[(0, -2., 2.), (3, -2., 2.)],
        0x05 => &[(0, -2., 2.)],
        0x06 => &[(0, -2., 2.), (1, -2., 2.)],
        0x0c => &[(2, -2., 2.), (3, -2., 2.)],
        0x0d => &[(2, -2., 2.)],
        0x0e => &[(2, -2., 2.), (1, -2., 2.)],
        0x10 | 0x12 => &[(2, -2., 0.), (1, 0., 2.)],
        0x11 | 0x3e => &[(2, 0., 2.), (3, -2., 0.)],
        0x3a => &[(2, -2., 0.), (1, 0., 2.)],
        _ => &[],
    }
}

#[derive(Clone, Copy)]
pub(super) struct ShelfEdge {
    side: usize,
    line: f32,
    start: f32,
    end: f32,
}

/// Build one contour from the already validated complete source formation.
/// Collinear courses merge before sculpting; a block join is not a new end.
pub(super) fn contour(blocks: &[(u16, [f32; 2])], g: &GridGeometry) -> Vec<ShelfEdge> {
    let mut edges = Vec::new();
    for &(id, center) in blocks {
        let x = center[0] / g.tile_width;
        let z = center[1] / g.tile_height;
        for &(side, start, end) in exposed_edges(id) {
            let (line, along) = match side {
                0 => (z - 2., -x),
                1 => (x + 2., -z),
                2 => (z + 2., x),
                _ => (x - 2., z),
            };
            edges.push(ShelfEdge {
                side,
                line,
                start: along + start,
                end: along + end,
            });
        }
    }
    edges.sort_by(|a, b| {
        a.side
            .cmp(&b.side)
            .then(a.line.total_cmp(&b.line))
            .then(a.start.total_cmp(&b.start))
    });
    let mut joined: Vec<ShelfEdge> = Vec::new();
    for edge in edges {
        if let Some(last) = joined.last_mut() {
            if last.side == edge.side
                && (last.line - edge.line).abs() < 1e-5
                && edge.start <= last.end + 1e-5
            {
                last.end = last.end.max(edge.end);
                continue;
            }
        }
        joined.push(edge);
    }
    joined
}

// Only authenticated outside edges move. The cap, bevel and wall all use
// this same field so sculpting cannot open a gap under the cap. Native walking
// heights and interior model joins remain exact.
fn shelf_point(
    p: Vec3,
    base: f32,
    edges: &[ShelfEdge],
    g: &GridGeometry,
    origin: [i32; 2],
) -> Vec3 {
    let local_x = p.x / g.tile_width;
    let local_z = p.z / g.tile_height;
    let x = (p.x - g.origin_x) / g.tile_width + origin[0] as f32;
    let z = (p.z - g.origin_z) / g.tile_height + origin[1] as f32;
    let height = ((p.y - base) / g.tile_height / 1.3).clamp(0., 1.);
    let rise = height * height * (3. - 2. * height);
    let mut result = p;
    for &ShelfEdge {
        side,
        line,
        start,
        end,
    } in edges
    {
        let (distance, along) = match side {
            0 => (local_z - line, -local_x),
            1 => (line - local_x, -local_z),
            2 => (line - local_z, local_x),
            _ => (local_x - line, local_z),
        };
        if distance < -0.001 || distance >= 0.65 || along < start || along > end {
            continue;
        }
        let t = (1. - distance / 0.65).clamp(0., 1.);
        // Only actual contour ends taper; merged model joins deform together.
        // The shared field also reaches neighboring caps at outside corners.
        let opening = ((along - start).min(end - along) / 0.4).clamp(0., 1.);
        let noise = new_bark::meadow_noise(x * 1.05, z * 1.05);
        let face=(p.y-base)/g.tile_height/1.50;
        let depth=face.clamp(0.,1.);
        let taper=(depth/0.20).min((1.-depth)/0.24).clamp(0.,1.);
        let fracture=cave_cliff::slab(x*1.1+face*1.50*0.16,z*1.1-face*1.50*0.12);
        // Body recesses use the same complete contour as caps. A separate
        // per-face displacement pulled orthogonal corner edges apart.
        let bed=(0.09+fracture.abs()*0.35)*taper;
        // Retreat the upper lip farther than the body below it. This widens
        // the existing shoulder horizontally, preserving every authored Y
        // value and keeping the retreat inside the border tile's center.
        let shoulder = (((p.y - base) / g.tile_height - 1.50) / 0.50).clamp(0., 1.);
        let body = 0.09 + noise.abs() * 0.12;
        let crown = 0.39 + cave_cliff::slab(x * 0.75 + 17., z * 0.75 - 11.) * 0.05;
        // A linear feather keeps the wider cap monotonic. Squaring this
        // weight can fold its first strip back over itself at the outer lip.
        // A quarter-circle profile meets the body vertically and the cap
        // horizontally instead of adding another hard planar chamfer.
        let round = 1. - (1. - shoulder * shoulder).max(0.).sqrt();
        let inset = ((body + (crown - body) * round) * rise + bed) * t * opening;
        result += [
            Vec3::Z * g.tile_height,
            Vec3::NEG_X * g.tile_width,
            Vec3::NEG_Z * g.tile_height,
            Vec3::X * g.tile_width,
        ][side]
            * inset;
    }
    result
}

fn body_color(p: Vec3, base: f32, g: &GridGeometry, origin: [i32; 2]) -> Vec3 {
    let x = (p.x - g.origin_x) / g.tile_width + origin[0] as f32;
    let z = (p.z - g.origin_z) / g.tile_height + origin[1] as f32;
    let height = (p.y - base) / g.tile_height;
    let bed = (height * 7. + new_bark::meadow_noise(x * 0.6, z * 0.6) * 3.).sin() * 0.025;
    let frost = (height / 1.50).min(1.).powi(3) * 0.16;
    Vec3::new(0.21, 0.40, 0.46) + Vec3::splat(bed + frost)
}

pub(super) fn append(
    out: &mut SurfaceMeshData,
    kind: Kind,
    id: u16,
    center: [f32; 2],
    base: f32,
    g: &GridGeometry,
    origin: [i32; 2],
    edges: &[ShelfEdge],
) {
    let unit = Vec3::new(g.tile_width, g.tile_height, g.tile_height);
    let offset = Vec3::new(center[0], base, center[1]);
    let m = model(kind);
    let light = Vec3::new(-0.35, 0.85, 0.4).normalize();
    // Keep the authored cap and the large planar stair faces. Remove the
    // repeated faceted pebbles and identical cap scratches from this kit.
    for tri in m.surface.indices.chunks_exact(3) {
        let p = [tri[0], tri[1], tri[2]].map(|i| Vec3::from_array(m.surface.positions[i as usize]));
        let c = m.surface.colors[tri[0] as usize];
        let snow = c[0] > 0.7;
        let step = c[0] > 0.5
            && c[0] < 0.7
            && p.iter().all(|p| p.y < 1.99)
            && (p[0].x == p[1].x && p[1].x == p[2].x
                || p[0].y == p[1].y && p[1].y == p[2].y
                || p[0].z == p[1].z && p[1].z == p[2].z);
        let stair_trim = c[0] < 0.3;
        if !snow && !step && !stair_trim {
            continue;
        }
        let emit = |out: &mut SurfaceMeshData, triangle: [Vec3; 3]| {
            let p = triangle.map(|p| {
                let p = p * unit + offset;
                if snow {
                    // Broaden only the authored shoulder. Walking caps
                    // and stair surfaces retain their exact elevations.
                    let mut p = p;
                    if p.y < base + 2. * g.tile_height {
                        let t = ((p.y - base) / g.tile_height - 1.78) / 0.22;
                        p.y = base + (1.50 + 0.50 * t) * g.tile_height;
                    }
                    shelf_point(p, base, edges, g, origin)
                } else {
                    p
                }
            });
            let cross = (p[1] - p[0]).cross(p[2] - p[0]);
            if cross.length_squared() < 1e-8 {
                return;
            }
            let normal = cross.normalize();
            let shade = 0.67 + 0.33 * normal.dot(light).max(0.);
            let first = out.positions.len() as u32;
            out.positions.extend(p.map(|p| p.to_array()));
            out.normals.extend([normal.to_array(); 3]);
            out.uvs.extend(p.map(|p| if snow {
                { let mut uv = cave_cliff::stone_uv(p,g,origin,true); uv[1] += 200000.; uv }
            } else { [0.;2] }));
            out.colors.extend(p.map(|p| {
                let x = (p.x - g.origin_x) / g.tile_width + origin[0] as f32;
                let z = (p.z - g.origin_z) / g.tile_height + origin[1] as f32;
                let grain = new_bark::meadow_noise(x * 0.6, z * 0.6) * 0.065;
                let crystal = new_bark::meadow_noise(x * 2.3, z * 2.3) * 0.015;
                let tint = if snow {
                    [0.54, 0.69, 0.70]
                } else {
                    [0.30, 0.49, 0.54]
                };
                let mut pigment = Vec3::from_array(tint) + Vec3::splat(grain + crystal);
                if snow {
                    let shoulder = (((p.y - base) / g.tile_height - 1.50) / 0.50).clamp(0., 1.);
                    let blend = shoulder * shoulder * (3. - 2. * shoulder);
                    pigment = body_color(p, base, g, origin).lerp(pigment, blend);
                }
                (pigment * shade).extend(1.).to_array()
            }));
            out.indices.extend([first, first + 1, first + 2]);
        };
        // Subdivide before sculpting so the silhouette follows the field
        // rather than merely moving the corners of a rectangular slab.
        let divisions = if snow { 16 } else { 1 };
        let at = |a: usize, b: usize| {
            p[0] + (p[1] - p[0]) * (a as f32 / divisions as f32)
                + (p[2] - p[0]) * (b as f32 / divisions as f32)
        };
        for a in 0..divisions {
            for b in 0..divisions - a {
                emit(out, [at(a, b), at(a + 1, b), at(a, b + 1)]);
                if a + b + 1 < divisions {
                    emit(out, [at(a + 1, b), at(a + 1, b + 1), at(a, b + 1)]);
                }
            }
        }
    }
    let sides = exposed_edges(id);
    for &(side, start, end) in sides {
        let at = |along: usize, row: usize| {
            let a = start + (end - start) * along as f32 / 16.;
            let y = 1.50 * row as f32 / 8.;
            let local = match side {
                0 => Vec3::new(-a, y, -2.),
                1 => Vec3::new(2., y, -a),
                2 => Vec3::new(a, y, 2.),
                _ => Vec3::new(-2., y, a),
            };
            shelf_point(local * unit + offset, base, edges, g, origin)
        };
        for row in 0..8 {
            for along in 0..16 {
                let p = [
                    at(along, row),
                    at(along + 1, row),
                    at(along + 1, row + 1),
                    at(along, row + 1),
                ];
                for ids in [[0, 1, 2], [0, 2, 3]] {
                    let q = ids.map(|i| p[i]);
                    let normal = (q[1] - q[0]).cross(q[2] - q[0]).normalize();
                    let shade = 0.67 + 0.33 * normal.dot(light).max(0.);
                    let first = out.positions.len() as u32;
                    out.positions.extend(q.map(|p| p.to_array()));
                    out.normals.extend([normal.to_array(); 3]);
                    out.uvs.extend(q.map(|p|cave_cliff::stone_uv(p,g,origin,true)));
                    out.colors.extend(q.map(|p| {
                        (body_color(p, base, g, origin) * shade).extend(1.).to_array()
                    }));
                    out.indices.extend([first, first + 1, first + 2]);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn orthogonal_shelf_faces_keep_one_closed_corner_through_the_recessed_body() {
        let g=GridGeometry {width:4,height:4,tile_width:8.,tile_height:8.,origin_x:-16.,origin_z:-16.};
        let edges=contour(&[(0x0e,[0.,0.])],&g);
        let mut out=SurfaceMeshData::default();
        append(&mut out,Kind::IceShelfSoutheast,0x0e,[0.,0.],0.,&g,[10,20],&edges);
        for row in 0..=8 {
            let y=1.50*row as f32/8.*8.;
            let corner=[16.,y,16.];
            // Both orthogonal body faces must retain this actual contour end.
            assert!(out.positions.iter().filter(|p| **p==corner).count()>=2,
                "split frozen corner at {y}");
        }
        let caps: Vec<_> = out.indices.chunks_exact(3).filter_map(|ids| {
            let p = [ids[0], ids[1], ids[2]].map(|i| Vec3::from_array(out.positions[i as usize]));
            p.iter().all(|p| p.y == 16.).then_some(p)
        }).collect();
        assert!(!caps.is_empty());
        assert!(caps.iter().all(|p| (p[1] - p[0]).cross(p[2] - p[0]).normalize().y > 0.9999),
            "widening the shoulder must not fold the walking cap");
        let walker = Vec3::new(12., 16., 12.);
        assert!(caps.iter().any(|p| {
            let area = (p[1] - p[0]).cross(p[2] - p[0]).y.abs();
            let parts = (p[0] - walker).cross(p[1] - walker).y.abs()
                + (p[1] - walker).cross(p[2] - walker).y.abs()
                + (p[2] - walker).cross(p[0] - walker).y.abs();
            (parts - area).abs() < 0.0001
        }), "the border tile's walking center must remain on the full-height cap");
    }
    #[test]
    fn sculpted_contour_flows_across_blocks_without_moving_stairs_or_datum() {
        let g = GridGeometry {
            width: 12,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: -16.,
            origin_z: -16.,
        };
        let edges = contour(
            &[
                (0x04, [0., -32.]),
                (0x05, [32., -32.]),
                (0x06, [64., -32.]),
                (0x0c, [0., 0.]),
                (0x0d, [32., 0.]),
                (0x0e, [64., 0.]),
            ],
            &g,
        );
        assert_eq!(edges.iter().filter(|e| e.side == 2).count(), 1);
        let seam = Vec3::new(16., 16., 16.);
        let sculpted = shelf_point(seam, 0., &edges, &g, [10, 20]);
        assert_eq!(sculpted.y, seam.y);
        assert!(sculpted.z < seam.z && sculpted.z > 12.);
        let shoulder_base = shelf_point(Vec3::new(16., 1.50 * 8., 16.), 0., &edges, &g, [10, 20]);
        assert_eq!(shoulder_base.y, 1.50 * 8.);
        assert!(shoulder_base.z - sculpted.z > 1., "upper lip must form a broad sloping shoulder");
        let near_body = shelf_point(Vec3::new(16., 13., 16.), 0., &edges, &g, [10, 20]);
        let near_cap = shelf_point(Vec3::new(16., 15., 16.), 0., &edges, &g, [10, 20]);
        assert!(near_cap.z - sculpted.z > (shoulder_base.z - near_body.z) * 2.,
            "the shoulder must curve toward the cap instead of forming a straight chamfer");
        // The southeast face reaches the north cap's shared edge through one
        // common field, not a per-model displacement with mismatched joins.
        let join = Vec3::new(80., 16., -16.);
        assert!(shelf_point(join, 0., &edges, &g, [10, 20]).x < join.x);
        let floor = Vec3::new(16., 0., 16.);
        assert_eq!(shelf_point(floor, 0., &edges, &g, [10, 20]), floor);
        let interior = Vec3::new(32., 16., 0.);
        assert_eq!(shelf_point(interior, 0., &edges, &g, [10, 20]), interior);
        let stair_edges = contour(&[(0x12, [0., 0.])], &g);
        let stair = Vec3::new(0., 16., 16.);
        assert_eq!(shelf_point(stair, 0., &stair_edges, &g, [10, 20]), stair);
    }

    #[test]
    fn frozen_shelf_keeps_the_stair_opening_and_world_registered_faces() {
        let g = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: -16.,
            origin_z: -16.,
        };
        let mut a = SurfaceMeshData::default();
        append(
            &mut a,
            Kind::IceShelfStairRight,
            0x12,
            [0., 0.],
            0.,
            &g,
            [10, 20],
            &contour(&[(0x12, [0., 0.])], &g),
        );
        // The source model has a small trim bevel at the stair's upper edge.
        // New wall relief must not introduce a cap across that opening.
        let authored = &model(Kind::IceShelfStairRight).surface;
        let on_authored_surface = |point: [f32; 3]| {
            let point = Vec3::from_array(point) / 8.;
            authored.indices.chunks_exact(3).any(|ids| {
                let [a, b, c] = [ids[0], ids[1], ids[2]]
                    .map(|i| Vec3::from_array(authored.positions[i as usize]));
                let ab = b - a;
                let ac = c - a;
                let ap = point - a;
                let normal = ab.cross(ac);
                if normal.length_squared() < 1e-8 || ap.dot(normal.normalize()).abs() > 1e-5 {
                    return false;
                }
                let aa = ab.dot(ab);
                let bb = ac.dot(ac);
                let cross = ab.dot(ac);
                let determinant = aa * bb - cross * cross;
                let u = (ap.dot(ab) * bb - ap.dot(ac) * cross) / determinant;
                let v = (ap.dot(ac) * aa - ap.dot(ab) * cross) / determinant;
                u >= -1e-5 && v >= -1e-5 && u + v <= 1.00001
            })
        };
        assert!(
            a.positions
                .iter()
                .filter(|p| p[0] > 0.001 && p[2] > 0.001 && p[1] > 15.99)
                .all(|p| on_authored_surface(*p))
        );
        assert!(a.positions.iter().all(|p| p.iter().all(|v| v.is_finite())));
        assert!(
            a.normals
                .iter()
                .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.0001)
        );
        let mut shifted = g;
        shifted.origin_x -= 8.;
        shifted.origin_z += 16.;
        let mut b = SurfaceMeshData::default();
        append(
            &mut b,
            Kind::IceShelfStairRight,
            0x12,
            [-8., 16.],
            0.,
            &shifted,
            [10, 20],
            &contour(&[(0x12, [-8., 16.])], &shifted),
        );
        let color_delta = a
            .colors
            .iter()
            .zip(&b.colors)
            .flat_map(|(p, q)| p.iter().zip(q).map(|(p, q)| (p - q).abs()))
            .fold(0_f32, f32::max);
        assert!(color_delta < 0.0001, "scrolling color delta {color_delta}");
        assert!(a.uvs.iter().any(|uv| uv[0]>90000. && (uv[1]/100000.).round()==9.));
        assert!(a.uvs.iter().any(|uv| uv[0]>90000. && (uv[1]/100000.).round()==7.));
        assert!(a.uvs.iter().any(|uv| *uv==[0.;2]), "stair material remains authored");
        assert!(a.uvs.iter().zip(&b.uvs).all(|(a,b)|
            (a[0]-b[0]).abs()<=0.0625 && (a[1]-b[1]).abs()<=0.0625));
        for (p, q) in a.positions.iter().zip(&b.positions) {
            assert!(
                (p[0] - q[0] - 8.).abs() < 0.00001
                    && (p[2] - q[2] + 16.).abs() < 0.00001
                    && p[1] == q[1]
            );
        }
    }
}
