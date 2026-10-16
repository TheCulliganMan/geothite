//! Shared closed masonry, trim and ceramic primitives for Kanto exteriors.
use super::*;

pub(super) fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    std::array::from_fn(|i| a[i] * (1. - t) + b[i] * t)
}
pub(super) fn face(
    out: &mut SurfaceMeshData,
    mut p: Vec<Vec3>,
    center: Vec3,
    color: [f32; 3],
    kind: f32,
    unit: f32,
) {
    let mut n = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
    let midpoint = p.iter().copied().sum::<Vec3>() / p.len() as f32;
    if n.dot(midpoint - center) < 0. {
        p.reverse();
        n = -n;
    }
    let shade = 0.62 + 0.38 * n.dot(Vec3::new(-0.35, 0.85, 0.4).normalize()).max(0.);
    let base = out.positions.len() as u32;
    for v in &p {
        out.positions.push(v.to_array());
        out.normals.push(n.to_array());
        out.colors
            .push([color[0] * shade, color[1] * shade, color[2] * shade, 1.]);
        let (u, v) = if n.y.abs() > 0.5 {
            (v.x, v.z)
        } else if n.x.abs() > n.z.abs() {
            (v.z, v.y)
        } else {
            (v.x, v.y)
        };
        out.uvs.push(if kind >= 0. {
            [
                u / unit,
                crate::scenery_finish::TOWN_PAINT_BASE + kind * 32. + v / unit,
            ]
        } else {
            crate::scenery_finish::SOURCE_PAINT_UV
        });
    }
    for i in 1..p.len() - 1 {
        out.indices
            .extend([base, base + i as u32, base + i as u32 + 1]);
    }
}
// Closed bevelled boxes use a full inner body and a recessed eight-sided
// crown; physical arrises supply shape instead of drawn black outlines.
pub(super) fn block(
    out: &mut SurfaceMeshData,
    b: [f32; 6],
    bevel: f32,
    color: [f32; 3],
    kind: f32,
    unit: f32,
) {
    let [w, e, lo, hi, n, s] = b;
    let c = Vec3::new((w + e) * 0.5, (lo + hi) * 0.5, (n + s) * 0.5);
    let k = bevel
        .min((e - w) * 0.2)
        .min((s - n) * 0.2)
        .min((hi - lo) * 0.2);
    if kind == 2. || k == 0. {
        // Closed shallow brick frustum: four small arrises around its face,
        // twelve triangles per brick rather than beveling every buried edge.
        let x_axis = e - w < s - n;
        let rings = if x_axis {
            [
                vec![
                    Vec3::new(w, lo, n),
                    Vec3::new(w, hi, n),
                    Vec3::new(w, hi, s),
                    Vec3::new(w, lo, s),
                ],
                vec![
                    Vec3::new(e, lo + k, n + k),
                    Vec3::new(e, hi - k, n + k),
                    Vec3::new(e, hi - k, s - k),
                    Vec3::new(e, lo + k, s - k),
                ],
            ]
        } else {
            [
                vec![
                    Vec3::new(w, lo, n),
                    Vec3::new(w, hi, n),
                    Vec3::new(e, hi, n),
                    Vec3::new(e, lo, n),
                ],
                vec![
                    Vec3::new(w + k, lo + k, s),
                    Vec3::new(w + k, hi - k, s),
                    Vec3::new(e - k, hi - k, s),
                    Vec3::new(e - k, lo + k, s),
                ],
            ]
        };
        face(out, rings[0].clone(), c, color, kind, unit);
        face(out, rings[1].clone(), c, color, kind, unit);
        for i in 0..4 {
            let j = (i + 1) % 4;
            face(
                out,
                vec![rings[0][i], rings[0][j], rings[1][j], rings[1][i]],
                c,
                color,
                kind,
                unit,
            );
        }
        return;
    }
    let outline = [
        (w + k, n),
        (e - k, n),
        (e, n + k),
        (e, s - k),
        (e - k, s),
        (w + k, s),
        (w, s - k),
        (w, n + k),
    ];
    let rings = [(lo, 0.), (hi, k)].map(|(y, inset)| {
        outline.map(|(x, z)| {
            Vec3::new(
                if x < c.x { x + inset } else { x - inset },
                y,
                if z < c.z { z + inset } else { z - inset },
            )
        })
    });
    face(out, rings[0].to_vec(), c, color, kind, unit);
    face(out, rings[1].to_vec(), c, color, kind, unit);
    for i in 0..8 {
        let j = (i + 1) % 8;
        face(
            out,
            vec![rings[0][i], rings[0][j], rings[1][j], rings[1][i]],
            c,
            color,
            kind,
            unit,
        );
    }
}

pub(super) fn source_ink(tile: u16, x: usize, y: usize) -> usize {
    let rows = match tile {
        0x4b => [
            "00000000", "11111110", "11111110", "00000000", "11101111", "11101111", "00000000", "01111110",
        ],
        0x17 => [
            "22222222", "11111111", "11111111", "11111111", "11111111", "33333333", "22222222",
            "33333333",
        ],
        0x1a => [
            "11111111", "00000000", "00000000", "11111111", "22222222", "22222222", "33333333",
            "11111111",
        ],
        0x12 => [
            "21111112", "12111121", "11211211", "11122111", "11122111", "11211211", "12111121",
            "21111112",
        ],
        0x22 => [
            "11111111", "00000000", "00000000", "00000000", "11111111", "00000000", "00000000",
            "00000000",
        ],
        0x0a | 0x32 => [
            "33333333", "30000003", "30000003", "30000003", "30000003", "30000003", "31111113",
            "33333333",
        ],
        0x0b => [
            "21211111", "21333333", "21322222", "21323333", "21323000", "21323000", "21323000",
            "21323111",
        ],
        0x1b => [
            "20323333", "20321111", "20321222", "20321222", "20321111", "20322222", "20333333",
            "33111111",
        ],
        _ => unreachable!("only sampled material roles have ink"),
    };
    (rows[y].as_bytes()[x] - b'0') as usize
}
