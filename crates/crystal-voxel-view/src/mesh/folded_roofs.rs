//! Folded paper details on the editable modern kit's blue weathering deck.
//! This material/normal datum belongs to the named source mesh, not map pixels.
use super::*;
use crate::exterior_models::Model;

pub(in crate::mesh) fn append(mesh: &mut SurfaceMeshData, first: usize, source: &Model, scale: f32, at: [i32; 2]) {
    let seed = (at[0] as u32).wrapping_mul(0x9e3779b9)
        ^ (at[1] as u32).wrapping_mul(0x85ebca6b);
    let style = ((seed ^ (seed >> 16)) % 4) as usize;
    let deck = |i: usize| {
        source.surface.colors[i][..3] == [0.18, 0.34, 0.55]
            && source.surface.normals[i][1] > 0.999
            && source.surface.positions[i][1] > 2.2
    };
    let Some(top) = (0..source.surface.positions.len())
        .filter(|&i| deck(i))
        .map(|i| source.surface.positions[i][1])
        .max_by(f32::total_cmp)
    else {
        return;
    };
    let ids: Vec<_> = (0..source.surface.positions.len())
        .filter(|&i| deck(i) && (source.surface.positions[i][1] - top).abs() < 0.0001)
        .collect();
    if ids.len() < 4 {
        return;
    }
    let mut b = [
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
    ];
    for &i in &ids {
        let p = mesh.positions[first + i];
        b[0] = b[0].min(p[0]);
        b[1] = b[1].max(p[0]);
        b[2] = b[2].min(p[2]);
        b[3] = b[3].max(p[2]);
    }
    let [w, e, n, s] = b;
    let y = mesh.positions[first + ids[0]][1];
    if e - w < scale || s - n < scale {
        return;
    }
    let color = mesh.colors[first + ids[0]];
    let light = Vec3::new(-0.35, 0.85, 0.4).normalize();
    let shade = 0.62 + 0.38 * light.y;
    let ink = [color[0] / shade, color[1] / shade, color[2] / shade];
    // Narrow closed folds run back to front; a shallow cross-course breaks up
    // the broad plane without painting a grid or replacing the existing slab.
    let count = ((e - w) / (scale * [0.36,0.48,0.42,0.52][style])).round().clamp(4., 24.) as usize;
    for i in 1..count {
        let x = w + (e - w) * i as f32 / count as f32;
        city_facades::block(
            mesh,
            [
                x - scale * 0.009,
                x + scale * 0.009,
                y + scale * 0.001,
                y + scale * 0.023,
                n,
                s,
            ],
            ink.map(|v| v * 1.12),
        );
    }
    for z in [n + (s - n) * [0.33,0.28,0.38,0.25][style], n + (s - n) * [0.66,0.61,0.71,0.64][style]] {
        city_facades::block(
            mesh,
            [
                w,
                e,
                y + scale * 0.001,
                y + scale * 0.006,
                z - scale * 0.005,
                z + scale * 0.005,
            ],
            ink.map(|v| v * 0.82),
        );
    }
    if style == 1 { return; }
    // A low clerestory sits opposite the kit's original service hatch. Its
    // triangular ends and two pitched panes read as folded card and glazing.
    let x0 = w + (e - w) * 0.16;
    let x1 = w + (e - w) * [0.46,0.46,0.38,0.42][style];
    let z0 = n + (s - n) * [0.22,0.22,0.40,0.55][style];
    let z1 = n + (s - n) * [0.45,0.45,0.61,0.76][style];
    let cy = y + scale * 0.08;
    let peak = y + scale * 0.22;
    let cz = (z0 + z1) * 0.5;
    city_facades::block(
        mesh,
        [x0, x1, y + scale * 0.002, cy, z0, z1],
        [0.68, 0.71, 0.66],
    );
    for (a, b) in [(z0, cz), (cz, z1)] {
        let (ya, yb) = if a == z0 { (cy, peak) } else { (peak, cy) };
        let normal = Vec3::new(0., b - a, ya - yb).normalize();
        let shade = 0.62 + 0.38 * normal.dot(light).max(0.);
        append_solid_quad(
            mesh,
            [[x0, ya, a], [x0, yb, b], [x1, yb, b], [x1, ya, a]],
            normal.to_array(),
            [0.42 * shade, 0.66 * shade, 0.69 * shade, 1.],
        );
    }
    for x in [x0, x1] {
        let normal = if x == x0 { [-1., 0., 0.] } else { [1., 0., 0.] };
        let base = mesh.positions.len() as u32;
        let points = if x == x0 {
            [[x, cy, z1], [x, peak, cz], [x, cy, z0]]
        } else {
            [[x, cy, z0], [x, peak, cz], [x, cy, z1]]
        };
        let shade = 0.62 + 0.38 * Vec3::from_array(normal).dot(light).max(0.);
        for p in points {
            mesh.positions.push(p);
            mesh.normals.push(normal);
            mesh.colors
                .push([0.68 * shade, 0.71 * shade, 0.66 * shade, 1.]);
            mesh.uvs.push([0., 0.]);
        }
        mesh.indices.extend([base, base + 1, base + 2]);
    }
    // Fine frame strips follow the glazing slopes rather than standing upright.
    for x in [x0, (x0 + x1) * 0.5, x1] {
        for (a, b, ya, yb) in [(z0, cz, cy, peak), (cz, z1, peak, cy)] {
            let normal = Vec3::new(0., b - a, ya - yb).normalize();
            let shade = 0.62 + 0.38 * normal.dot(light).max(0.);
            let dx = scale * 0.009;
            append_solid_quad(
                mesh,
                [
                    [x - dx, ya + scale * 0.006, a],
                    [x - dx, yb + scale * 0.006, b],
                    [x + dx, yb + scale * 0.006, b],
                    [x + dx, ya + scale * 0.006, a],
                ],
                normal.to_array(),
                [0.82 * shade, 0.85 * shade, 0.78 * shade, 1.],
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exterior_models::{Kind, model};
    #[test]
    fn roof_details_preserve_shell_and_stay_inside_its_footprint() {
        for kind in [
            Kind::ModernHouse,
            Kind::ModernGym,
            Kind::ModernDepartment,
            Kind::ModernMart,
        ] {
            let source = model(kind);
            let mut mesh = SurfaceMeshData::default();
            source.append_fitted(&mut mesh, [0., 80., 0., 64.], 0., 16., Some(40.));
            let original = mesh.clone();
            append(&mut mesh, 0, source, 16., [20, 12]);
            assert!(mesh.positions.len() > original.positions.len(), "{kind:?}");
            assert_eq!(
                mesh.positions[..original.positions.len()],
                original.positions
            );
            assert_eq!(mesh.colors[..original.colors.len()], original.colors);
            assert!(
                mesh.positions
                    .iter()
                    .all(|p| p.iter().all(|v| v.is_finite())
                        && p[0] >= 0.
                        && p[0] <= 80.
                        && p[2] >= 0.
                        && p[2] <= 64.)
            );
            assert!(
                mesh.normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 1e-5)
            );
            for triangle in mesh.indices[original.indices.len()..].chunks_exact(3) {
                let a = Vec3::from_array(mesh.positions[triangle[0] as usize]);
                let b = Vec3::from_array(mesh.positions[triangle[1] as usize]);
                let c = Vec3::from_array(mesh.positions[triangle[2] as usize]);
                let cross = (b-a).cross(c-a);
                assert!(cross.length_squared() > 1e-10);
                assert!(cross.dot(Vec3::from_array(mesh.normals[triangle[0] as usize])) > 0.);
            }
            let mut other = original.clone();
            append(&mut other, 0, source, 16., [24, 16]);
            assert_ne!(mesh, other);
            let mut again = original;
            append(&mut again, 0, source, 16., [20, 12]);
            assert_eq!(mesh, again);
        }
    }
}
