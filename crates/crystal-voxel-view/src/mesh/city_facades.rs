//! Coordinated, world-anchored treatments for Goldenrod's residential shells.
use super::*;
use crate::exterior_models::Model;
use crate::new_bark_models::{ModelKind, model};

fn variant(at: [i32; 2]) -> usize {
    let mut h = (at[0] as u32).wrapping_mul(0x9e3779b9) ^ (at[1] as u32).wrapping_mul(0x85ebca6b);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb352d);
    h ^= h >> 15;
    (h % 4) as usize
}

// Six closed faces, with key/fill shading matching the fitted building kit.
pub(super) fn block(mesh: &mut SurfaceMeshData, b: [f32; 6], color: [f32; 3]) {
    let [w, e, y, h, n, s] = b;
    let light = Vec3::new(-0.35, 0.85, 0.4).normalize();
    for (p, normal) in [
        ([[w, h, n], [w, h, s], [e, h, s], [e, h, n]], [0., 1., 0.]),
        ([[w, y, s], [w, y, n], [e, y, n], [e, y, s]], [0., -1., 0.]),
        ([[w, y, s], [e, y, s], [e, h, s], [w, h, s]], [0., 0., 1.]),
        ([[e, y, n], [w, y, n], [w, h, n], [e, h, n]], [0., 0., -1.]),
        ([[w, y, n], [w, y, s], [w, h, s], [w, h, n]], [-1., 0., 0.]),
        ([[e, y, s], [e, y, n], [e, h, n], [e, h, s]], [1., 0., 0.]),
    ] {
        let shade = 0.62 + 0.38 * Vec3::from_array(normal).dot(light).max(0.);
        append_solid_quad(
            mesh,
            p,
            normal,
            [color[0] * shade, color[1] * shade, color[2] * shade, 1.],
        );
    }
}

pub(super) fn finish(
    mesh: &mut SurfaceMeshData,
    first: usize,
    source: &Model,
    bounds: [f32; 4],
    ys: f32,
    door: Option<f32>,
    at: [i32; 2],
) {
    let style = variant(at);
    let wall = [
        [0.90, 0.80, 0.65],
        [0.69, 0.78, 0.68],
        [0.83, 0.67, 0.62],
        [0.72, 0.77, 0.80],
    ][style];
    let trim = [
        [0.97, 0.91, 0.79],
        [0.89, 0.91, 0.80],
        [0.96, 0.87, 0.75],
        [0.92, 0.91, 0.82],
    ][style];
    let roof: [f32; 3] = [
        [0.24, 0.38, 0.53],
        [0.29, 0.43, 0.43],
        [0.35, 0.38, 0.46],
        [0.29, 0.37, 0.51],
    ][style];
    let accent = [
        [0.47, 0.29, 0.20],
        [0.28, 0.40, 0.32],
        [0.43, 0.27, 0.25],
        [0.30, 0.37, 0.43],
    ][style];
    let [w, e, n, s] = bounds;
    let xs = (e - w) / (source.max[0] - source.min[0]);
    let zs = (s - n) / (source.max[2] - source.min[2]);
    for i in 0..source.surface.positions.len() {
        let color = source.surface.colors[i];
        let replacement = if color[..3] == [0.87, 0.79, 0.59] {
            Some(wall)
        } else if color[..3] == [0.98, 0.91, 0.73] {
            Some(trim)
        } else if color[..3] == [0.18, 0.34, 0.55] {
            Some(roof)
        } else if color[..3] == [0.31, 0.48, 0.69] {
            Some(roof.map(|c| (c + 0.16).min(1.)))
        } else if color[..3] == [0.11, 0.22, 0.37] {
            Some(roof.map(|c| c * 0.66))
        } else {
            None
        };
        if let Some(new) = replacement {
            for c in 0..3 {
                mesh.colors[first + i][c] *= new[c] / color[c];
            }
        }
        // A shallow stepped cornice changes the roof silhouette. All entrance
        // and window courses are below this authored upper-wall datum.
        let upper = source.surface.positions[i][1] - 2.1;
        if upper > 0. {
            let factor = [-0.20, 0.28, 0.10, -0.05][style];
            mesh.positions[first + i][1] += upper * ys * factor;
            let normal = Vec3::from_array(mesh.normals[first + i]);
            let next = (normal / Vec3::new(1., 1. + factor, 1.)).normalize();
            let light = Vec3::new(-0.35, 0.85, 0.4).normalize();
            let before = 0.62 + 0.38 * normal.dot(light).max(0.);
            let after = 0.62 + 0.38 * next.dot(light).max(0.);
            mesh.normals[first + i] = next.to_array();
            for c in 0..3 {
                mesh.colors[first + i][c] *= after / before;
            }
        }
        // Only the named kit's stone hatch and its louvers occupy this material/
        // height combination. The roof coping and the shell stay fixed.
        let p = source.surface.positions[i];
        if p[1] > 2.39 && (color[..3] == [0.62, 0.65, 0.54] || color[..3] == [0.32, 0.35, 0.31]) {
            mesh.positions[first + i][0] += [-0.7, 0.0, 0.2, -0.4][style] * xs;
            mesh.positions[first + i][2] += [-0.35, 0.2, 0.45, 0.35][style] * zs;
        }
    }
    let x = |v: f32| {
        if let (Some(a), Some(d)) = (source.anchor, door) {
            crate::exterior_models::fit_door_axis(
                v,
                [source.min[0], source.max[0]],
                [w, e],
                a[0],
                d,
            )
            .0
        } else {
            w + (v - source.min[0]) * xs
        }
    };
    let z = |v: f32| n + (v - source.min[2]) * zs;
    // Side windows retain the source shell. Decorative front details occupy
    // the two original outer window bays, leaving the central doorway clear.
    for center in [-1.12, 1.12] {
        if style == 1 || style == 3 {
            for side in [-1., 1.] {
                let cx = center + side * 0.48;
                block(
                    mesh,
                    [
                        x(cx - 0.075),
                        x(cx + 0.075),
                        0.83 * ys,
                        1.72 * ys,
                        z(1.31),
                        z(1.37),
                    ],
                    accent,
                );
                for level in [0.98, 1.16, 1.34, 1.52] {
                    block(
                        mesh,
                        [
                            x(cx - 0.08),
                            x(cx + 0.08),
                            level * ys,
                            (level + 0.018) * ys,
                            z(1.37),
                            z(1.39),
                        ],
                        trim,
                    );
                }
            }
        } else {
            let b = [x(center - 0.42), x(center + 0.42), z(1.30), z(1.53)];
            block(mesh, [b[0], b[1], 0.61 * ys, 0.79 * ys, b[2], b[3]], accent);
            block(
                mesh,
                [b[0], b[1], 0.785 * ys, 0.80 * ys, b[2], b[3]],
                [0.23, 0.28, 0.13],
            );
            let flowers = model(ModelKind::Flowers);
            flowers.append_fitted(
                mesh,
                b,
                0.8 * ys,
                ys * 0.17 / (flowers.max[1] - flowers.min[1]),
                None,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exterior_models::{Kind, model};
    #[test]
    fn treatments_vary_without_moving_doors_or_plot_edges() {
        let m = model(Kind::ModernHouse);
        let b = [0., 64., 0., 64.];
        let mut signatures = std::collections::HashSet::new();
        for at in [
            [0, 0],
            [4, 0],
            [8, 0],
            [12, 0],
            [16, 0],
            [20, 0],
            [24, 0],
            [28, 0],
        ] {
            let mut mesh = SurfaceMeshData::default();
            m.append_fitted(&mut mesh, b, 0., 16., Some(32.));
            let before = mesh.positions.clone();
            finish(&mut mesh, 0, m, b, 16., Some(32.), at);
            assert!(
                mesh.positions
                    .iter()
                    .all(|p| p[0] >= 0. && p[0] <= 64. && p[2] >= 0. && p[2] <= 64.)
            );
            for (i, p) in m.surface.positions.iter().enumerate() {
                if p[1] <= 2.1 {
                    assert_eq!(mesh.positions[i], before[i]);
                }
            }
            assert_eq!(mesh.indices[..m.surface.indices.len()], m.surface.indices);
            assert!(
                mesh.normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 1e-5)
            );
            signatures.insert((variant(at), mesh.positions.len()));
            let mut again = SurfaceMeshData::default();
            m.append_fitted(&mut again, b, 0., 16., Some(32.));
            finish(&mut again, 0, m, b, 16., Some(32.), at);
            assert_eq!(mesh, again);
        }
        assert_eq!(signatures.len(), 4);
    }
}
