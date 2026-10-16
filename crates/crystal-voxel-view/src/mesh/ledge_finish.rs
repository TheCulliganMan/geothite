//! World-seeded fractured rock on source-verified jump banks and water edges.
//! Perimeters and support stay fixed; only the visible face gains detail.
use super::*;
use bevy::prelude::Vec3;

pub(in crate::mesh) const BANK_GROUND_UV: [f32; 2] = [86500., 0.];

/// One shoreline treatment for both outdoor families: continuous dry ground
/// above the existing varied, closed rocky ledge, without a paved cap.
pub(in crate::mesh) fn append_natural_bank(
    mesh: &mut SurfaceMeshData, p: [[f32;3];4], normal: [f32;3],
    g: &GridGeometry, origin: [i32;2], palette: [[f32;3];2],
) {
    let palette = palette.map(|color| {
        let light = Vec3::from_array(color).dot(Vec3::new(0.2126,0.7152,0.0722));
        std::array::from_fn(|i|color[i]*0.12+light*[1.06,1.02,0.94][i]*0.88)
    });
    if normal[1]>0.99 {
        append_quad_colors(mesh,p,normal,[BANK_GROUND_UV;4],
            [[palette[0][0],palette[0][1],palette[0][2],1.];4]);
        return;
    }
    let first = mesh.positions.len();
    append_shore_with_rim_lift(mesh,p,normal,g,origin,palette,true);
    let low = p.iter().map(|p|p[1]).fold(f32::INFINITY,f32::min);
    let high = p.iter().map(|p|p[1]).fold(f32::NEG_INFINITY,f32::max);
    for ((position,color),uv) in mesh.positions[first..].iter()
        .zip(&mut mesh.colors[first..]).zip(&mut mesh.uvs[first..]) {
        // The existing painted bedrock texture uses triplanar source-map
        // coordinates. A shore tag selects close-grained caps as well as walls.
        *uv = [88000. + (position[0]-g.origin_x + origin[0] as f32*g.tile_width)/8.,
            -20000. - (position[2]-g.origin_z + origin[1] as f32*g.tile_height)/8.];
        let dry = ((position[1]-low)/(high-low).max(0.001)).clamp(0.,1.);
        for c in &mut color[..3] { *c *= 0.54+dry*0.46; }
    }
}

pub(in crate::mesh) fn soil_color(x: f32, z: f32, fraction: f32) -> [f32; 3] {
    let grain = meadow_noise(x * 4.1 + 7., z * 4.1 - 11.) * 0.065;
    let strata = (fraction * 11.0 + meadow_noise(x * 1.9, z * 1.9) * 1.7).sin() * 0.018;
    [
        0.29 + grain + strata,
        0.19 + grain * 0.70 + strata,
        0.080 + grain * 0.35,
    ]
}

pub(in crate::mesh) fn append(
    mesh: &mut SurfaceMeshData,
    p: [[f32; 3]; 4],
    normal: [f32; 3],
    g: &GridGeometry,
    origin: [i32; 2],
) -> bool {
    if !append_rock(mesh, p, normal, g, origin, true) {
        return false;
    }
    append_rim_rocks(mesh, p, normal, g, origin);
    true
}

/// A town's replacement closed lip owns its rim stones. Keep only the
/// original relieved support here so no old rim fragments survive the swap.
pub(in crate::mesh) fn append_support_face(mesh: &mut SurfaceMeshData,
    p: [[f32;3];4], normal: [f32;3], g: &GridGeometry, origin: [i32;2]) -> bool {
    append_rock(mesh,p,normal,g,origin,true)
}

/// Bare rock at path/water contacts, without lawn plants.
pub(in crate::mesh) fn append_bare_rock(
    mesh: &mut SurfaceMeshData,
    p: [[f32; 3]; 4],
    normal: [f32; 3],
    g: &GridGeometry,
    origin: [i32; 2],
) -> bool {
    if !append_rock(mesh, p, normal, g, origin, false) {
        return false;
    }
    append_rim_rocks(mesh, p, normal, g, origin);
    true
}

fn append_rock(
    mesh: &mut SurfaceMeshData,
    p: [[f32; 3]; 4],
    normal: [f32; 3],
    g: &GridGeometry,
    origin: [i32; 2],
    vegetated: bool,
) -> bool {
    let n = Vec3::from_array(normal);
    if n.y.abs() > 0.001 || n.length_squared() < 0.99 {
        return false;
    }
    let p = p.map(Vec3::from_array);
    if p[1].y < p[0].y || p[2].y < p[3].y || (p[1].y == p[0].y && p[2].y == p[3].y) {
        return false;
    }
    let width = (p[3] - p[0]).length();
    let height = (p[1].y - p[0].y).max(p[2].y - p[3].y);
    if width < g.tile_width * 0.1 || height <= 0.0001 {
        return false;
    }
    // Unequal fracture intersections, rather than repeated stones or a noise
    // texture on a flat wall. Only interior samples move: shared lip, toe and
    // terminal seams retain the exact authored support and never grow spikes.
    let columns = ((width / g.tile_width * 4.).ceil() as usize).clamp(2, 32);
    let rows = ((height / g.tile_height * 4.).ceil() as usize).clamp(2, 24);
    let point = |column: usize, row: usize| {
        let u = column as f32 / columns as f32;
        let v = row as f32 / rows as f32;
        let q = p[0].lerp(p[3], u).lerp(p[1].lerp(p[2], u), v);
        let x = (q.x - g.origin_x) / g.tile_width + origin[0] as f32;
        let z = (q.z - g.origin_z) / g.tile_height + origin[1] as f32;
        let seed = |salt: f32| {
            meadow_noise(
                x * 3.73 + q.y / g.tile_height * 2.9 + salt,
                z * 4.17 - q.y / g.tile_height * 3.1 - salt * 1.7,
            )
        };
        let boundary = column == 0 || column == columns || row == 0 || row == rows;
        let ju = if boundary {
            u
        } else {
            u + seed(19.) * 0.62 / columns as f32
        };
        let jv = if boundary {
            v
        } else {
            v + seed(73.) * 0.62 / rows as f32
        };
        let relief = if boundary {
            0.
        } else {
            height.min(g.tile_width) * (0.08 + (seed(137.) + 0.5).clamp(0., 1.) * 0.12)
        };
        let position = p[0].lerp(p[3], ju).lerp(p[1].lerp(p[2], ju), jv) + n * relief;
        let grain = seed(211.) * 0.065;
        // A light broken mineral rim contrasts with grass; the darker wet toe
        // gives even a shallow water drop a readable land/water boundary.
        let shade = 0.72 + jv * 0.28;
        let stone = [0.43 + grain, 0.385 + grain * 0.85, 0.29 + grain * 0.65];
        let moss = if vegetated && jv > 0.94 { 0.22 } else { 0. };
        let grass = [0.26, 0.39, 0.085];
        let color = std::array::from_fn(|c| {
            if c == 3 {
                1.
            } else {
                (stone[c] * (1. - moss) + grass[c] * moss) * shade
            }
        });
        (position, color)
    };
    append_fractures(mesh, p, normal, columns, rows, point);

    true
}

/// Exposed shoreline shelves have actual broken mineral surfaces, including
/// their top plane. Perimeters stay at the source datum so neighboring water,
/// land and corner pieces remain sealed. No atlas sprites are sampled.
pub(in crate::mesh) fn append_shore(
    mesh: &mut SurfaceMeshData,
    positions: [[f32; 3]; 4],
    normal: [f32; 3],
    g: &GridGeometry,
    origin: [i32; 2],
    palette: [[f32; 3]; 2],
) {
    append_shore_with_rim_lift(mesh,positions,normal,g,origin,palette,false);
}

fn append_shore_with_rim_lift(
    mesh: &mut SurfaceMeshData, positions: [[f32;3];4], normal: [f32;3],
    g: &GridGeometry, origin: [i32;2], palette: [[f32;3];2], raised: bool,
) {
    let first = mesh.colors.len();
    if normal[1].abs() < 0.001 && append_rock(mesh, positions, normal, g, origin, false) {
        let rim = mesh.positions.len();
        append_rim_rocks(mesh,positions,normal,g,origin);
        if raised {
            let center = positions.map(Vec3::from_array).into_iter().sum::<Vec3>()/4.;
            let x = (center.x-g.origin_x)/g.tile_width+origin[0] as f32;
            let z = (center.z-g.origin_z)/g.tile_height+origin[1] as f32;
            let lift = g.tile_width.min(g.tile_height)
                * (0.04+(meadow_noise(x*2.7+17.,z*3.1-31.)+0.5)*0.08);
            // Translate each closed rock, retaining a buried base and its
            // original normals. The continuous support and water stay fixed.
            for p in &mut mesh.positions[rim..] { p[1] += lift; }
        }
        for color in &mut mesh.colors[first..] {
            let amount = ((color[0] - 0.20) / 0.27).clamp(0., 1.);
            for c in 0..3 {
                color[c] = palette[1][c] + (palette[0][c] - palette[1][c]) * amount;
            }
        }
        return;
    }
    let p = positions.map(Vec3::from_array);
    let n = Vec3::from_array(normal);
    let point = |column: usize, row: usize| {
        let u = column as f32 / 4.;
        let v = row as f32 / 4.;
        let q = p[0].lerp(p[3], u).lerp(p[1].lerp(p[2], u), v);
        let x = (q.x - g.origin_x) / g.tile_width + origin[0] as f32;
        let z = (q.z - g.origin_z) / g.tile_height + origin[1] as f32;
        let seed = meadow_noise(x * 3.7 + 47., z * 3.3 - 91.);
        let boundary = column == 0 || column == 4 || row == 0 || row == 4;
        let ju = if boundary { u } else { u + seed * 0.14 };
        let jv = if boundary {
            v
        } else {
            v + meadow_noise(x * 4.1 - 71., z * 2.9 + 31.) * 0.14
        };
        let relief = if boundary {
            0.
        } else {
            g.tile_width.min(g.tile_height) * (0.018 + (seed + 0.5) * 0.055)
        };
        let position = p[0].lerp(p[3], ju).lerp(p[1].lerp(p[2], ju), jv) + n * relief;
        let amount = (0.64 + seed * 0.55).clamp(0., 1.);
        let color = std::array::from_fn(|c| {
            if c == 3 {
                1.
            } else {
                palette[1][c] + (palette[0][c] - palette[1][c]) * amount
            }
        });
        (position, color)
    };
    append_fractures(mesh, p, normal, 4, 4, point);
}

/// Closed, unequal rock masses break the straight support lip into a rocky
/// silhouette. The terrain hull below still owns every tile and walking datum.
pub(in crate::mesh) fn append_rim_rocks(
    mesh: &mut SurfaceMeshData,
    positions: [[f32; 3]; 4],
    normal: [f32; 3],
    g: &GridGeometry,
    origin: [i32; 2],
) {
    if normal[1].abs() > 0.001 {
        return;
    }
    let mut p = positions.map(Vec3::from_array);
    if (p[3].y - p[0].y).abs() > (p[1].y - p[0].y).abs() {
        p.rotate_left(1);
    }
    if p[1].y <= p[0].y || p[2].y <= p[3].y {
        return;
    }
    let tangent = (p[2] - p[1]).with_y(0.).normalize_or_zero();
    let width = (p[2] - p[1]).with_y(0.).length();
    if width < g.tile_width * 0.1 {
        return;
    }
    let outward = Vec3::from_array(normal);
    let midpoint = (p[1] + p[2]) * 0.5;
    let x = (midpoint.x - g.origin_x) / g.tile_width + origin[0] as f32;
    let z = (midpoint.z - g.origin_z) / g.tile_height + origin[1] as f32;
    let seed =
        |salt: f32| (meadow_noise(x * 3.17 + salt, z * 2.71 - salt * 1.31) + 0.5).clamp(0., 1.);
    let count = ((width / g.tile_width) * (0.65 + seed(23.) * 1.55))
        .round()
        .max(1.) as usize;
    let slot = width / count as f32;
    for rock in 0..count {
        let salt = rock as f32 * 17.3;
        let along = slot * (rock as f32 + 0.30 + seed(salt + 41.) * 0.40);
        let rx = slot * (0.48 + seed(salt + 59.) * 0.25);
        let rz = g.tile_width * (0.19 + seed(salt + 83.) * 0.22);
        let ry = g.tile_height * (0.14 + seed(salt + 107.) * 0.14);
        let center = p[1].lerp(p[2], along / width)
            - outward * rz * (0.12 + seed(salt + 113.) * 0.85)
            - Vec3::Y * ry * (0.42 + seed(salt + 119.) * 0.18);
        let turn = seed(salt + 131.) * 0.8;
        let mut rings = [[Vec3::ZERO; 7]; 4];
        for (ring, (spread, height)) in [(0.68, -0.72), (1., -0.08), (0.76, 0.52), (0.36, 0.79)]
            .into_iter()
            .enumerate()
        {
            for side in 0..7 {
                let angle = turn
                    + side as f32 * std::f32::consts::TAU / 7.
                    + (seed(salt + side as f32 * 7.1 + 151.) - 0.5) * 0.22;
                let radius = spread * (0.82 + seed(salt + side as f32 * 11.7 + 179.) * 0.30);
                let along_offset = (angle.cos() * rx * radius).clamp(
                    -along - g.tile_width * 0.20,
                    width - along + g.tile_width * 0.20,
                );
                rings[ring][side] = center
                    + tangent * along_offset
                    + outward * angle.sin() * rz * radius
                    + Vec3::Y
                        * ry
                        * (height + (seed(salt + side as f32 * 5.3 + 197.) - 0.5) * 0.16);
            }
        }
        let pigment = 0.88 + seed(salt + 229.) * 0.24;
        let stone = [0.45 * pigment, 0.40 * pigment, 0.31 * pigment, 1.];
        let mut face = |mut q: [Vec3; 3]| {
            let mut n = (q[1] - q[0]).cross(q[2] - q[0]);
            if n.dot((q[0] + q[1] + q[2]) / 3. - center) < 0. {
                q.swap(1, 2);
                n = -n;
            }
            if n.length_squared() < 1e-10 {
                return;
            }
            let n = n.normalize();
            let shade = 0.80 + 0.20 * n.y.max(0.);
            let color = [stone[0] * shade, stone[1] * shade, stone[2] * shade, 1.];
            let start = mesh.positions.len() as u32;
            mesh.positions.extend(q.map(|p| p.to_array()));
            mesh.normals.extend([n.to_array(); 3]);
            mesh.uvs.extend([[0.; 2]; 3]);
            mesh.colors.extend([color; 3]);
            mesh.indices.extend([start, start + 1, start + 2]);
        };
        for ring in 0..3 {
            for side in 0..7 {
                let next = (side + 1) % 7;
                face([
                    rings[ring][side],
                    rings[ring + 1][side],
                    rings[ring + 1][next],
                ]);
                face([rings[ring][side], rings[ring + 1][next], rings[ring][next]]);
            }
        }
        // Caps complete each rock volume; no billboard planes or open cards.
        for side in 1..6 {
            face([rings[0][0], rings[0][side], rings[0][side + 1]]);
            face([rings[3][0], rings[3][side], rings[3][side + 1]]);
        }
    }
}

/// Keep encounter grass from hiding an authored drop. This only shortens
/// rendered blades close to its lip; encounter tiles and footing are untouched.
pub(in crate::mesh) fn clear_grass_at_ledges(
    mesh: &mut SurfaceMeshData,
    first: usize,
    base: f32,
    edges: &[crystal_render_api::VisualJumpLedge],
    g: &GridGeometry,
    origin: [i32; 2],
) {
    if edges.is_empty() {
        return;
    }
    // Most grass plots are far from a ledge. Reject them once per plot,
    // instead of measuring every blade vertex against every map edge.
    let mut low = bevy::prelude::Vec2::splat(f32::INFINITY);
    let mut high = bevy::prelude::Vec2::splat(f32::NEG_INFINITY);
    for p in &mesh.positions[first..] {
        let q = bevy::prelude::Vec2::new(
            (p[0] - g.origin_x) / g.tile_width + origin[0] as f32,
            (p[2] - g.origin_z) / g.tile_height + origin[1] as f32,
        );
        low = low.min(q);
        high = high.max(q);
    }
    let nearby: Vec<_> = edges
        .iter()
        .filter(|edge| {
            let a = edge.start.as_vec2().min(edge.end.as_vec2());
            let b = edge.start.as_vec2().max(edge.end.as_vec2());
            a.x <= high.x + 1.56
                && b.x >= low.x - 1.56
                && a.y <= high.y + 1.56
                && b.y >= low.y - 1.56
        })
        .collect();
    if nearby.is_empty() {
        return;
    }
    for i in first..mesh.positions.len() {
        let p = Vec3::from_array(mesh.positions[i]);
        let q = bevy::prelude::Vec2::new(
            (p.x - g.origin_x) / g.tile_width + origin[0] as f32,
            (p.z - g.origin_z) / g.tile_height + origin[1] as f32,
        );
        let distance = nearby
            .iter()
            .map(|edge| {
                let a = edge.start.as_vec2();
                let d = edge.end.as_vec2() - a;
                let t = ((q - a).dot(d) / d.length_squared().max(0.0001)).clamp(0., 1.);
                (q - a - d * t).length()
            })
            .fold(f32::INFINITY, f32::min);
        let t = ((distance - 0.15) / 1.4).clamp(0., 1.);
        let scale = 0.12 + 0.88 * t * t * (3. - 2. * t);
        if scale >= 1. {
            continue;
        }
        mesh.positions[i][1] = base + (p.y - base) * scale;
        let n = Vec3::from_array(mesh.normals[i]);
        mesh.normals[i] = Vec3::new(n.x, n.y / scale, n.z)
            .normalize_or_zero()
            .to_array();
    }
}

/// A narrow dry-side rock lip makes lawn/path shore drops visible from above.
/// Its entire perimeter meets the existing ground; only its interior rises.
pub(in crate::mesh) fn append_water_rim(
    mesh: &mut SurfaceMeshData,
    face: [[f32; 3]; 4],
    normal: [f32; 3],
    g: &GridGeometry,
    origin: [i32; 2],
) {
    let a = Vec3::from_array(face[1]);
    let b = Vec3::from_array(face[2]);
    let center = (a + b) * 0.5;
    let x = (center.x - g.origin_x) / g.tile_width + origin[0] as f32;
    let z = (center.z - g.origin_z) / g.tile_height + origin[1] as f32;
    let width = g.tile_width.min(g.tile_height)
        * (0.22 + meadow_noise(x * 1.3 + 29., z * 1.7 - 61.) * 0.14);
    let inward = -Vec3::from_array(normal) * width;
    append_shore(
        mesh,
        [a, a + inward, b + inward, b].map(|p| p.to_array()),
        [0., 1., 0.],
        g,
        origin,
        [[0.48, 0.43, 0.33], [0.29, 0.28, 0.22]],
    );
}

fn append_fractures(
    mesh: &mut SurfaceMeshData,
    p: [Vec3; 4],
    normal: [f32; 3],
    columns: usize,
    rows: usize,
    point: impl Fn(usize, usize) -> (Vec3, [f32; 4]) + Copy,
) {
    let mut points: Vec<_> = (0..=rows)
        .flat_map(|r| (0..=columns).map(move |c| point(c, r)))
        .collect();
    // Lerp at t=1 can round a rotated endpoint. Preserve native corners bit
    // for bit, including their source normals and connected support seams.
    points[0].0 = p[0];
    points[rows * (columns + 1)].0 = p[1];
    points[rows * (columns + 1) + columns].0 = p[2];
    points[columns].0 = p[3];
    for row in 0..rows {
        for column in 0..columns {
            let a = row * (columns + 1) + column;
            let b = a + columns + 1;
            // Follow the local fracture shape rather than stamp the same
            // diagonal across every bank and shoreline cell.
            let triangles = if points[a].0.distance_squared(points[b + 1].0)
                <= points[b].0.distance_squared(points[a + 1].0)
            {
                [[a, b, b + 1], [a, b + 1, a + 1]]
            } else {
                [[a, b, a + 1], [b, b + 1, a + 1]]
            };
            for triangle in triangles {
                let q = triangle.map(|i| points[i].0);
                let cross = (q[1] - q[0]).cross(q[2] - q[0]);
                if cross.length_squared() < 1e-10 {
                    continue;
                }
                let facet = cross.normalize();
                let start = mesh.positions.len() as u32;
                for (j, i) in triangle.into_iter().enumerate() {
                    mesh.positions.push(q[j].to_array());
                    mesh.normals.push(if p.contains(&q[j]) {
                        normal
                    } else {
                        facet.to_array()
                    });
                    mesh.uvs.push([0.; 2]);
                    mesh.colors.push(points[i].1);
                }
                mesh.indices.extend([start, start + 1, start + 2]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn natural_bank_rocks_keep_texture_fixed_when_the_scene_grid_scrolls() {
        let g=GridGeometry {width:2,height:2,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let p=[[8.,-2.,8.],[8.,0.,8.],[0.,0.,8.],[0.,-2.,8.]];
        let palette=[[0.6,0.4,0.04],[0.3,0.2,0.01]];
        let mut a=SurfaceMeshData::default();
        append_natural_bank(&mut a,p,[0.,0.,1.],&g,[7,11],palette);
        let shifted=GridGeometry {origin_x:16.,origin_z:-24.,..g};
        let mut b=SurfaceMeshData::default();
        append_natural_bank(&mut b,p.map(|p|[p[0]+16.,p[1],p[2]-24.]),
            [0.,0.,1.],&shifted,[7,11],palette);
        assert_eq!(a.uvs,b.uvs);
        assert!(a.colors.iter().zip(&b.colors).all(|(a,b)|
            a.iter().zip(b).all(|(a,b)|(a-b).abs()<0.0001)));
        assert!(a.uvs.iter().all(|uv|uv[0]>87000. && uv[0]<90000. && uv[1] < -10000.));
        assert!(p.iter().all(|p|a.positions.contains(p)),"the support corners remain sealed");
        assert!(a.positions.iter().any(|p|p[1]>0.7),"rocks must rise above the bank, not read as flat chips");
        assert!(a.positions.iter().all(|p|p[1]>=-3.6 && p[1]<=2.56));
        for (a,b) in a.positions.iter().zip(&b.positions) {
            assert!((a[0]+16.-b[0]).abs()<0.0001 && a[1]==b[1]
                && (a[2]-24.-b[2]).abs()<0.0001);
        }
    }
    #[test]
    fn soil_face_seals_perimeters_and_scrolls_with_the_source_bank() {
        let g = GridGeometry {
            width: 2,
            height: 2,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let p = [[8., 0., 8.], [8., 3., 8.], [0., 3., 8.], [0., 0., 8.]];
        let mut a = SurfaceMeshData::default();
        assert!(append_rock(&mut a, p, [0., 0., 1.], &g, [10, 9], true));
        assert!(p.iter().all(|p| a.positions.contains(p)));
        for corner in p {
            assert!(
                a.positions
                    .iter()
                    .zip(&a.normals)
                    .any(|(position, normal)| *position == corner && *normal == [0., 0., 1.]),
                "native support corners retain exact positions and normals"
            );
        }
        assert!(
            a.positions
                .iter()
                .all(|p| p[1] >= 0. && p[1] <= 4.5 && p[0] >= 0. && p[0] <= 8.)
        );
        assert!(
            a.positions.iter().any(|p| p[2] > 8.1),
            "rock must have physical relief rather than a flat painted face"
        );
        assert!(a.positions.iter().all(|p| p[2] >= 8. && p[2] <= 8.6));
        assert!(
            a.uvs.iter().all(|uv| *uv == [0.; 2]),
            "no sprite atlas on rock faces"
        );
        for ids in a.indices.chunks_exact(3) {
            let q = ids
                .iter()
                .map(|i| Vec3::from_array(a.positions[*i as usize]))
                .collect::<Vec<_>>();
            let n = Vec3::from_array(a.normals[ids[0] as usize]);
            assert!((q[1] - q[0]).cross(q[2] - q[0]).dot(n) > 0.);
        }
        let shifted_g = GridGeometry { origin_x: -8., ..g };
        let mut b = SurfaceMeshData::default();
        assert!(append_rock(
            &mut b,
            p.map(|p| [p[0] - 8., p[1], p[2]]),
            [0., 0., 1.],
            &shifted_g,
            [10, 9],
            true
        ));
        assert_eq!(a.indices, b.indices);
        for (a, b) in a.normals.iter().zip(&b.normals) {
            assert!(
                Vec3::from_array(*a).distance(Vec3::from_array(*b)) < 1e-5,
                "scrolling must not alter bank lighting"
            );
        }
        for (a, b) in a.colors.iter().zip(&b.colors) {
            assert!(a.iter().zip(b).all(|(a, b)| (*a - *b).abs() < 1e-6));
        }
        let mut floor = SurfaceMeshData::default();
        assert!(!append_rock(
            &mut floor,
            [[0., 0., 0.], [0., 0., 8.], [8., 0., 8.], [8., 0., 0.]],
            [0., 1., 0.],
            &g,
            [10, 9],
            true
        ));
        assert!(floor.positions.is_empty());
    }
}

#[cfg(test)]
mod directions {
    use super::*;
    #[test]
    fn each_cardinal_bank_has_outward_winding_and_keeps_its_support_corners() {
        let g = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: -16.,
            origin_z: -16.,
        };
        for turn in 0..4 {
            let rotate =
                bevy::prelude::Quat::from_rotation_y(turn as f32 * std::f32::consts::FRAC_PI_2);
            let p = [[8., 0., 8.], [8., 3., 8.], [0., 3., 8.], [0., 0., 8.]]
                .map(|p| (rotate * Vec3::from_array(p)).to_array());
            let n = (rotate * Vec3::Z).to_array();
            let mut mesh = SurfaceMeshData::default();
            assert!(append_rock(&mut mesh, p, n, &g, [0, 0], true));
            for corner in p {
                assert!(mesh.positions.iter().zip(&mesh.normals).any(|(q, m)| {
                    Vec3::from_array(*q).distance(Vec3::from_array(corner)) < 1e-5
                        && Vec3::from_array(*m).distance(Vec3::from_array(n)) < 1e-5
                }));
            }
            for ids in mesh.indices.chunks_exact(3) {
                let q = ids
                    .iter()
                    .map(|i| Vec3::from_array(mesh.positions[*i as usize]))
                    .collect::<Vec<_>>();
                let n = Vec3::from_array(mesh.normals[ids[0] as usize]);
                assert!((q[1] - q[0]).cross(q[2] - q[0]).dot(n) > 0.);
            }
        }
    }
}

#[cfg(test)]
mod rocky_variation {
    use super::*;

    #[test]
    fn seeded_fractures_vary_by_world_position_and_leave_every_seam_sealed() {
        let g = GridGeometry {
            width: 2,
            height: 2,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let p = [[8., 0., 8.], [8., 6., 8.], [0., 6., 8.], [0., 0., 8.]];
        let mut a = SurfaceMeshData::default();
        let mut b = SurfaceMeshData::default();
        let mut repeat = SurfaceMeshData::default();
        assert!(append_rock(&mut a, p, [0., 0., 1.], &g, [0, 0], true));
        assert!(append_rock(&mut b, p, [0., 0., 1.], &g, [11, 7], true));
        assert!(append_rock(&mut repeat, p, [0., 0., 1.], &g, [0, 0], true));
        assert_eq!(a.positions, repeat.positions);
        assert_eq!(a.colors, repeat.colors);
        assert_ne!(
            a.positions, b.positions,
            "neighboring banks must not stamp the same rock"
        );
        assert_ne!(a.colors, b.colors);
        assert!(
            a.normals.iter().any(|n| n[1].abs() > 0.1),
            "facets must catch light in 3D"
        );
        for q in &a.positions {
            if q[0] == 0. || q[0] == 8. || q[1] == 0. || q[1] == 6. {
                assert_eq!(q[2], 8., "all perimeter points must remain sealed");
            }
        }
    }

    #[test]
    fn ledge_clearance_only_shortens_nearby_grass_and_keeps_roots_and_other_geometry() {
        let g = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut mesh = SurfaceMeshData::default();
        mesh.positions = vec![[0., 24., 0.], [8., 0., 8.], [8., 24., 8.], [24., 24., 24.]];
        mesh.normals = vec![[0., 1., 0.]; 4];
        let before = mesh.positions.clone();
        let edge = crystal_render_api::VisualJumpLedge {
            start: bevy::prelude::IVec2::new(0, 1),
            end: bevy::prelude::IVec2::new(2, 1),
            descent: bevy::prelude::IVec2::new(0, 1),
        };
        clear_grass_at_ledges(&mut mesh, 1, 0., &[edge], &g, [0, 0]);
        assert_eq!(mesh.positions[0], before[0]);
        assert_eq!(mesh.positions[1], before[1], "grass roots stay planted");
        assert!(
            mesh.positions[2][1] < 4.,
            "the rock lip must be visible above nearby grass"
        );
        assert_eq!(
            mesh.positions[3], before[3],
            "distant grass retains its full height"
        );
    }

    #[test]
    fn rim_rocks_are_closed_unequal_volumes_with_visible_silhouettes() {
        let g = GridGeometry {
            width: 2,
            height: 2,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let p = [[8., 0., 8.], [8., 6., 8.], [0., 6., 8.], [0., 0., 8.]];
        let mut a = SurfaceMeshData::default();
        let mut b = SurfaceMeshData::default();
        append_rim_rocks(&mut a, p, [0., 0., 1.], &g, [0, 0]);
        append_rim_rocks(&mut b, p, [0., 0., 1.], &g, [11, 7]);
        assert!(!a.indices.is_empty());
        assert_ne!(
            a.positions, b.positions,
            "rocks must vary in shape, size and spacing"
        );
        assert!(
            a.positions.iter().any(|p| p[1] > 6.15),
            "rocks must break the straight lip silhouette"
        );
        assert!(
            a.positions.iter().any(|p| p[2] > 8.1),
            "rocks need visible three-dimensional depth"
        );
        assert!(
            a.positions
                .iter()
                .all(|p| (-1.6..=9.6).contains(&p[0]) && p[1] <= 8.)
        );
        assert!(a.uvs.iter().all(|uv| *uv == [0.; 2]));
        let mut edges = std::collections::HashMap::new();
        for tri in a.indices.chunks_exact(3) {
            let q = [tri[0], tri[1], tri[2]].map(|i| Vec3::from_array(a.positions[i as usize]));
            let normal = Vec3::from_array(a.normals[tri[0] as usize]);
            assert!((q[1] - q[0]).cross(q[2] - q[0]).dot(normal) > 0.);
            for j in 0..3 {
                let mut edge = [
                    q[j].to_array().map(f32::to_bits),
                    q[(j + 1) % 3].to_array().map(f32::to_bits),
                ];
                edge.sort();
                *edges.entry(edge).or_insert(0) += 1;
            }
        }
        assert!(
            edges.values().all(|&count| count == 2),
            "rock volumes must be closed, never cards"
        );
    }

    #[test]
    fn shoreline_shelf_and_water_face_are_solid_rock_with_bounded_relief() {
        let g = GridGeometry {
            width: 2,
            height: 2,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let palette = [[0.51, 0.43, 0.29], [0.29, 0.27, 0.20]];
        for (p, normal) in [
            (
                [[0., 0., 0.], [0., 0., 8.], [8., 0., 8.], [8., 0., 0.]],
                [0., 1., 0.],
            ),
            (
                [[8., -2., 8.], [8., 0., 8.], [0., 0., 8.], [0., -2., 8.]],
                [0., 0., 1.],
            ),
        ] {
            let mut mesh = SurfaceMeshData::default();
            append_shore(&mut mesh, p, normal, &g, [0, 0], palette);
            assert!(p.iter().all(|p| mesh.positions.contains(p)));
            assert!(mesh.uvs.iter().all(|uv| *uv == [0.; 2]));
            let n = Vec3::from_array(normal);
            let origin = Vec3::from_array(p[0]);
            assert!(
                mesh.positions
                    .iter()
                    .any(|q| (Vec3::from_array(*q) - origin).dot(n) > 0.05)
            );
            for q in &mesh.positions {
                let depth = (Vec3::from_array(*q) - origin).dot(n);
                assert!((-7.0..=3.4).contains(&depth));
            }
            for tri in mesh.indices.chunks_exact(3) {
                let [a, b, c] =
                    [tri[0], tri[1], tri[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
                assert!(
                    (b - a)
                        .cross(c - a)
                        .dot(Vec3::from_array(mesh.normals[tri[0] as usize]))
                        > 0.,
                    "each closed rock facet must have outward, non-collapsed winding"
                );
            }
        }
    }
}
