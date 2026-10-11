//! Low frozen relief for verified shelf-edge paint outside complete plateau
//! models. This decorates authenticated edge paint; support remains the
//! producer's original datum. Ladders and unknown drawings stay live.
use super::*;

// Bits point toward the outer edge of the native four-cell shelf drawing.
const WEST: u8 = 1;
const EAST: u8 = 2;
const NORTH: u8 = 4;
const SOUTH: u8 = 8;
const THIN_WEST: u8 = 16;
const THIN_SOUTH: u8 = 32;
const THIN_EAST: u8 = 64;

fn edges(s: &VisualTileSource) -> Option<u8> {
    if s.tileset_id.as_ref() != "ice_path" || s.subtile_column >= 4 || s.subtile_row >= 4 {
        return None;
    }
    let x = s.subtile_column;
    let y = s.subtile_row;
    let t = s.tile_index;
    if s.metatile_id == 0x28 && x == 3 && t == 0x9c {
        return Some(THIN_EAST);
    }
    // These course drawings are blocked rock fronts, not a traversable flight.
    // Their four authored texels form the low lip beside the shelf opening.
    if matches!(s.metatile_id, 0x12 | 0x3e)
        && x < 2
        && y >= 2
        && t == [[0x80, 0x81], [0x90, 0x91]][usize::from(y - 2)][usize::from(x)]
    {
        return Some(SOUTH);
    }
    // The southeast quadrant of $3e closes the same low frozen course
    // as its southwest $80/$81/$90/$91 face. Leaving these exact corner
    // texels on the floor exposes the purple source drawing beside the ice.
    if s.metatile_id == 0x3e
        && x >= 2
        && y >= 2
        && t == [[0xb9, 0xba], [0xc9, 0xca]][usize::from(y - 2)][usize::from(x - 2)]
    {
        return Some(SOUTH);
    }
    if matches!(s.metatile_id, 0x12 | 0x39 | 0x3a) && y >= 2 {
        let start = if s.metatile_id == 0x3a { 0 } else { 2 };
        if (start..start + 2).contains(&x)
            && t == [[0xae, 0xaf], [0xbe, 0xbf]][usize::from(y - 2)][usize::from(x - start)]
        {
            return Some(SOUTH);
        }
    }
    if s.metatile_id == 0x3a
        && x >= 2
        && t == [0x9c, 0xac, 0xbc, 0xcc][usize::from(y)] + u16::from(x - 2)
    {
        return Some(EAST | if y >= 2 { SOUTH } else { 0 });
    }
    if s.metatile_id == 0x10
        && x >= 2
        && y >= 2
        && t == [[0xa0, 0xa1], [0xb0, 0xb1]][usize::from(y - 2)][usize::from(x - 2)]
    {
        return Some(EAST | SOUTH);
    }
    if s.metatile_id == 0x11
        && x < 2
        && y >= 2
        && t == [[0xa2, 0xa3], [0xb2, 0xb3]][usize::from(y - 2)][usize::from(x)]
    {
        return Some(WEST | SOUTH);
    }
    let pair = |a: u16, local: u8| t == a + u16::from(local % 2);
    let west = match s.metatile_id {
        0x04 => {
            x < 2
                && ((y == 0 && pair(0x88, x))
                    || (y == 1 && pair(0x98, x))
                    || (y >= 2 && pair(0xa8, x)))
        }
        0x08 => x < 2 && pair(0xa8, x),
        0x39 => x < 2 && pair([0x98, 0xa8, 0xb8, 0xc8][usize::from(y)], x),
        0x0c => {
            x < 2
                && ((y < 2 && pair(0xa8, x))
                    || (y == 2 && pair(0xb8, x))
                    || (y == 3 && pair(0xc8, x)))
        }
        _ => false,
    };
    let east = match s.metatile_id {
        0x06 => {
            x >= 2
                && ((y == 0 && pair(0x8c, x))
                    || (y == 1 && pair(0x9c, x))
                    || (y >= 2 && pair(0xac, x)))
        }
        0x0a => x >= 2 && pair(0xac, x),
        0x0e => {
            x >= 2
                && ((y < 2 && pair(0xac, x))
                    || (y == 2 && pair(0xbc, x))
                    || (y == 3 && pair(0xcc, x)))
        }
        _ => false,
    };
    let north = y == 0
        && match s.metatile_id {
            0x04 => (x < 2 && pair(0x88, x)) || (x >= 2 && pair(0x8a, x)),
            0x05 => pair(0x8a, x),
            0x06 => (x < 2 && pair(0x8a, x)) || (x >= 2 && pair(0x8c, x)),
            0x2a => x >= 2 && pair(0x8a, x),
            _ => false,
        };
    let south = y >= 2
        && match s.metatile_id {
            0x0c => west || (x >= 2 && t == if y == 2 { 0xba } else { 0xca }),
            0x0d => {
                if y == 2 {
                    t == if x == 1 { 0xbb } else { 0xba }
                } else {
                    t == if x == 1 { 0xcb } else { 0xca }
                }
            }
            0x0e => east || (x < 2 && pair(if y == 2 { 0xba } else { 0xca }, x)),
            _ => false,
        };
    let thin_west =
        s.metatile_id == 0x24 && x == 0 && t == [0x99, 0xa9, 0xa9, 0xc8][usize::from(y)];
    let thin_south = y == 3
        && match s.metatile_id {
            0x24 => thin_west || (x > 0 && t == 0xba),
            0x25 => t == 0xba,
            0x39 => west,
            _ => false,
        };
    let flags = u8::from(west) * WEST
        | u8::from(east) * EAST
        | u8::from(north) * NORTH
        | u8::from(south) * SOUTH
        | u8::from(thin_west) * THIN_WEST
        | u8::from(thin_south) * THIN_SOUTH;
    (flags != 0).then_some(flags)
}

fn relief(flags: u8, x: f32, y: f32) -> f32 {
    let mut relief: f32 = 0.0;
    if flags & WEST != 0 {
        relief = relief.max((2.0 - x) / 2.0);
    }
    if flags & EAST != 0 {
        relief = relief.max((x - 2.0) / 2.0);
    }
    if flags & NORTH != 0 {
        relief = relief.max(1.0 - y);
    }
    if flags & SOUTH != 0 {
        relief = relief.max((y - 2.0) / 2.0);
    }
    if flags & THIN_WEST != 0 {
        relief = relief.max(1.0 - x);
    }
    if flags & THIN_SOUTH != 0 {
        relief = relief.max(y - 3.0);
    }
    if flags & THIN_EAST != 0 {
        relief = relief.max(x - 3.0);
    }
    relief.clamp(0.0, 1.0)
}

pub(super) fn has_edge(source: &VisualTileSource) -> bool {
    edges(source).is_some()
}

type Neighborhood<'a> = [Option<&'a VisualTileSource>; 9];
fn weight(distance: f32) -> f32 {
    let t = distance.clamp(0., 1.);
    1. - t * t * (3. - 2. * t)
}

// Both sides of a join sample the same physical patches, including the four
// patches at a corner. Unrelated art and raised authored models are excluded.
fn joined_relief(neighbors: &Neighborhood<'_>, u: f32, v: f32) -> f32 {
    let mut sum = 0.;
    let mut total = 0.;
    for (i, source) in neighbors.iter().enumerate() {
        let Some(source) = source else {
            continue;
        };
        let Some(flags) = edges(source) else {
            continue;
        };
        let dx = (i % 3) as f32 - 1.;
        let dz = (i / 3) as f32 - 1.;
        let w = weight((u - dx - 0.5).abs()) * weight((v - dz - 0.5).abs());
        if w == 0. {
            continue;
        }
        let r = relief(
            flags,
            source.subtile_column as f32 + (u - dx).clamp(0., 1.),
            source.subtile_row as f32 + (v - dz).clamp(0., 1.),
        );
        sum += r * w;
        total += w;
    }
    if total > 0. { sum / total } else { 0. }
}

fn contour_push(neighbors: &Neighborhood<'_>, u: f32, v: f32) -> [f32; 2] {
    let occupied = |x: i32, z: i32| {
        (-1..=1).contains(&x)
            && (-1..=1).contains(&z)
            && neighbors[((z + 1) * 3 + x + 1) as usize].is_some_and(has_edge)
    };
    let mut influence = [0_f32; 4];
    for (i, s) in neighbors.iter().enumerate() {
        if !s.is_some_and(has_edge) {
            continue;
        }
        let x = (i % 3) as i32 - 1;
        let z = (i / 3) as i32 - 1;
        for (side, dx, dz, distance, along) in [
            (0, -1, 0, (u - x as f32).abs(), v - z as f32),
            (1, 0, 1, (v - z as f32 - 1.).abs(), u - x as f32),
            (2, 1, 0, (u - x as f32 - 1.).abs(), v - z as f32),
            (3, 0, -1, (v - z as f32).abs(), u - x as f32),
        ] {
            if occupied(x + dx, z + dz) || !(0.0..=1.0).contains(&along) {
                continue;
            }
            influence[side] = influence[side].max((1. - distance / 0.4).clamp(0., 1.).powi(2));
        }
    }
    [influence[0] - influence[2], influence[3] - influence[1]]
}

pub(super) fn append(
    out: &mut SurfaceMeshData,
    points: [[f32; 3]; 4],
    normal: [f32; 3],
    source: &VisualTileSource,
    g: &GridGeometry,
    origin: [i32; 2],
    neighbors: Neighborhood<'_>,
) -> bool {
    if normal[1] < 0.999 || points.iter().any(|p| p[1].abs() > 0.001) {
        return false;
    }
    let p = points[0];
    let expected = [
        p,
        [p[0], 0.0, p[2] + g.tile_height],
        [p[0] + g.tile_width, 0.0, p[2] + g.tile_height],
        [p[0] + g.tile_width, 0.0, p[2]],
    ];
    if points
        .iter()
        .zip(expected)
        .any(|(a, b)| Vec3::from_array(*a).distance(Vec3::from_array(b)) > 0.001)
    {
        return false;
    }
    let Some(_) = edges(source) else {
        return false;
    };
    let at = |u: f32, v: f32| {
        let a = Vec3::from_array(points[0]).lerp(Vec3::from_array(points[3]), u);
        let b = Vec3::from_array(points[1]).lerp(Vec3::from_array(points[2]), u);
        let mut p = a.lerp(b, v);
        let r = joined_relief(&neighbors, u, v);
        let map_x = (p.x - g.origin_x) / g.tile_width + origin[0] as f32;
        let map_z = (p.z - g.origin_z) / g.tile_height + origin[1] as f32;
        // A rounded bank replaces the straight triangular extrusion. The
        // continuous map field keeps both halves of every source join equal.
        // Relief stays inside the original maximum and footprint.
        let shoulder = r * r * (3. - 2. * r);
        let drift = (0.72
            + new_bark::meadow_noise(map_x * 0.55, map_z * 0.55) * 0.22
            + cave_cliff::slab(map_x * 1.6, map_z * 1.6).abs() * 0.32).clamp(0.52, 1.);
        p.y = shoulder * drift * g.tile_height * 0.5;
        // Shape only exposed edges. Shared source-cell seams keep identical
        // map samples; the untouched floor below closes the eroded footprint.
        let rise = (p.y / (g.tile_height * 0.35)).clamp(0., 1.);
        let erosion = (0.08
            + cave_cliff::slab(map_x * 0.9 + 31., map_z * 0.9).abs() * 0.18
            + new_bark::meadow_noise(map_x * 0.75 + 19., map_z * 0.75) * 0.04) * rise;
        let [dx, dz] = contour_push(&neighbors, u, v);
        p.x += dx * erosion * g.tile_width;
        p.z += dz * erosion * g.tile_height;

        p.to_array()
    };
    // Retreating the frozen shoulder must reveal stone, never empty space.
    super::ice_ground::append(out, points, false, g, origin);
    let color = |p: [f32; 3], n: [f32; 3]| {
        let light = 0.76 + n[1].max(0.0) * 0.24;
        let frost = (p[1] / (g.tile_height * 0.5)).clamp(0.0, 1.0);
        let map_x = (p[0] - g.origin_x) / g.tile_width + origin[0] as f32;
        let map_z = (p[2] - g.origin_z) / g.tile_height + origin[1] as f32;
        let grain = new_bark::meadow_noise(map_x * 1.6, map_z * 1.6) * 0.045;
        [
            (0.20 + frost * 0.24 + grain) * light,
            (0.38 + frost * 0.28 + grain) * light,
            (0.45 + frost * 0.29 + grain) * light,
            1.0,
        ]
    };
    let mut face = |p: [[f32; 3]; 4], normals: Option<[[f32; 3]; 4]>| {
        if p.iter().all(|p| p[1] <= 0.000001) {
            return;
        }

        let a = Vec3::from_array(p[0]);
        let b = Vec3::from_array(p[1]);
        let c = Vec3::from_array(p[2]);
        let d = Vec3::from_array(p[3]);
        let mut n = (b - a).cross(c - a);
        if n.length_squared() < 0.000001 {
            n = (c - a).cross(d - a);
        }
        let n = n.normalize_or_zero().to_array();
        if n == [0.0; 3] {
            return;
        }
        let ns = normals.unwrap_or([n; 4]);
        let first = out.positions.len() as u32;
        out.positions.extend(p);
        out.normals.extend(ns);
        out.uvs.extend(p.map(|p|cave_cliff::stone_uv(Vec3::from_array(p),g,origin,true)));
        out.colors.extend((0..4).map(|i| color(p[i], ns[i])));
        out.indices
            .extend([first, first + 1, first + 2, first, first + 2, first + 3]);
    };
    let normal_at = |u: f32, v: f32| {
        let epsilon = 0.01;
        let du = Vec3::from_array(at(u + epsilon, v)) - Vec3::from_array(at(u - epsilon, v));
        let dv = Vec3::from_array(at(u, v + epsilon)) - Vec3::from_array(at(u, v - epsilon));
        dv.cross(du).normalize().to_array()
    };
    const STEPS: usize = 8;
    let step = 1. / STEPS as f32;
    for y in 0..STEPS {
        for x in 0..STEPS {
            let u = x as f32 * step;
            let v = y as f32 * step;
            let coords = [(u, v), (u, v + step), (u + step, v + step), (u + step, v)];
            face(
                coords.map(|(u, v)| at(u, v)),
                Some(coords.map(|(u, v)| normal_at(u, v))),
            );
        }
    }
    let exposed_sides = [3, 7, 5, 1].map(|i| !neighbors[i].is_some_and(has_edge));
    let rim = [(0., 0.), (0., 1.), (1., 1.), (1., 0.)];
    for i in 0..4 {
        if !exposed_sides[i] {
            continue;
        }
        let (u0, v0) = rim[i];
        let (u1, v1) = rim[(i + 1) % 4];
        for segment in 0..STEPS {
            let at_edge = |t: f32| at(u0 + (u1 - u0) * t, v0 + (v1 - v0) * t);
            let a = at_edge(segment as f32 * step);
            let b = at_edge((segment + 1) as f32 * step);
            face([[a[0], 0.0, a[2]], [b[0], 0.0, b[2]], b, a], None);
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source(block: u16, tile: u16, x: u8, y: u8) -> VisualTileSource {
        VisualTileSource {
            tileset_id: "ice_path".into(),
            metatile_id: block,
            tile_index: tile,
            subtile_column: x,
            subtile_row: y,
        }
    }
    #[test]
    fn exit_block_rim_does_not_claim_the_threshold_or_ladder_art() {
        for x in 2..4 {
            let rim = source(0x2a,0x8a+u16::from(x-2),x,0);
            assert_eq!(edges(&rim),Some(NORTH));
            assert_eq!(shape_for_source(&rim).surface_height(8.),0.);
            for wrong in [source(0x2a,rim.tile_index,x,1),
                source(0x2a,rim.tile_index,x-2,0),
                source(0x07,rim.tile_index,x,0),
                source(0x2a,rim.tile_index+1,x,0)] {
                assert_eq!(edges(&wrong),None);
            }
        }
        assert_eq!(edges(&source(0x2a,0x42,0,0)),None);
        assert_eq!(edges(&source(0x2a,0x43,1,0)),None);
    }
    #[test]
    fn southeast_3e_corner_finishes_only_its_exact_frozen_course() {
        for y in 2..4 {
            for x in 2..4 {
                let t = [[0xb9, 0xba], [0xc9, 0xca]][(y - 2) as usize][(x - 2) as usize];
                let corner = source(0x3e, t, x, y);
                assert_eq!(edges(&corner), Some(SOUTH));
                let mut wrong = corner.clone();
                wrong.tile_index += 1;
                assert_eq!(edges(&wrong), None);
                wrong = corner.clone();
                wrong.metatile_id = 0;
                assert_eq!(edges(&wrong), None);
                assert_eq!(shape_for_source(&corner).surface_height(8.), 0.);
            }
        }
    }

    #[test]
    fn different_courses_share_a_four_patch_corner() {
        let patches = [
            source(0x39, 0xb9, 1, 2),
            source(0x39, 0xae, 2, 2),
            source(0x39, 0xc9, 1, 3),
            source(0x39, 0xbe, 2, 3),
        ];
        let mut heights = Vec::new();
        let mut offsets = Vec::new();
        for z in 0..2 {
            for x in 0..2 {
                let n = std::array::from_fn(|i| {
                    let px = x + (i % 3) as i32 - 1;
                    let pz = z + (i / 3) as i32 - 1;
                    (px >= 0 && pz >= 0 && px < 2 && pz < 2)
                        .then(|| &patches[(pz * 2 + px) as usize])
                });
                heights.push(joined_relief(&n, 1. - x as f32, 1. - z as f32));
                offsets.push(contour_push(&n, 1. - x as f32, 1. - z as f32));
            }
        }
        assert!(heights.iter().all(|h| (*h - heights[0]).abs() < 1e-6));
        assert!(heights[0] > 0. && heights[0] < 1.);
        assert!(offsets.iter().all(|o| *o == [0., 0.]));
        let mut n = [None; 9];
        n[4] = Some(&patches[0]);
        // Unknown neighbors cannot invent relief or displace a source join.
        assert_eq!(joined_relief(&n, 1., 0.5), 0.);
    }

    #[test]
    fn rim_requires_exact_edge_position_and_art() {
        assert_eq!(edges(&source(0x28, 0x9c, 3, 0)), Some(THIN_EAST));
        assert_eq!(edges(&source(0x28, 0x9c, 3, 3)), Some(THIN_EAST));
        assert_eq!(edges(&source(0x28, 0x9c, 2, 0)), None);
        assert_eq!(edges(&source(0x27, 0x9c, 3, 0)), None);
        assert_eq!(edges(&source(0x39, 0xae, 2, 2)), Some(SOUTH));
        assert_eq!(edges(&source(0x39, 0xbf, 3, 3)), Some(SOUTH));
        assert_eq!(edges(&source(0x39, 0xae, 0, 2)), None);
        assert_eq!(edges(&source(0x04, 0x88, 0, 0)), Some(WEST | NORTH));
        assert_eq!(edges(&source(0x08, 0xa9, 1, 3)), Some(WEST));
        assert_eq!(edges(&source(0x0a, 0xad, 3, 0)), Some(EAST));
        assert_eq!(
            edges(&source(0x24, 0xc8, 0, 3)),
            Some(THIN_WEST | THIN_SOUTH)
        );
        assert_eq!(edges(&source(0x39, 0xc9, 1, 3)), Some(WEST | THIN_SOUTH));
        assert_eq!(edges(&source(0x25, 0xba, 2, 3)), Some(THIN_SOUTH));
        assert_eq!(edges(&source(0x12, 0x80, 0, 2)), Some(SOUTH));
        assert_eq!(edges(&source(0x3e, 0x90, 0, 3)), Some(SOUTH));
        assert_eq!(edges(&source(0x10, 0xb1, 3, 3)), Some(EAST | SOUTH));
        assert_eq!(edges(&source(0x11, 0xa2, 0, 2)), Some(WEST | SOUTH));
        for s in [
            source(0x08, 0xa8, 2, 0),
            source(0x04, 0x89, 0, 0),
            source(0x12, 0x81, 0, 2),
            source(0x3e, 0x91, 0, 3),
            source(0x1f, 0xc6, 0, 0),
        ] {
            assert_eq!(edges(&s), None);
        }
    }
    #[test]
    fn adjoining_ice_rims_share_the_same_surface_seam() {
        let g = GridGeometry {
            width: 2,
            height: 1,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let p = [[0., 0., 0.], [0., 0., 8.], [8., 0., 8.], [8., 0., 0.]];
        for (ls, rs) in [
            (source(0x08, 0xa8, 0, 2), source(0x08, 0xa9, 1, 2)),
            (source(0x39, 0xb9, 1, 2), source(0x39, 0xae, 2, 2)),
        ] {
            let mut ln = [None; 9];
            ln[4] = Some(&ls);
            ln[5] = Some(&rs);
            let mut rn = [None; 9];
            rn[3] = Some(&ls);
            rn[4] = Some(&rs);
            let mut left = SurfaceMeshData::default();
            let mut right = SurfaceMeshData::default();
            assert!(append(&mut left, p, [0., 1., 0.], &ls, &g, [0, 0], ln));
            assert!(append(
                &mut right,
                p.map(|p| [p[0] + 8., p[1], p[2]]),
                [0., 1., 0.],
                &rs,
                &g,
                [0, 0],
                rn
            ));
            let seam = |mesh: &SurfaceMeshData| {
                let mut points: Vec<_> = mesh
                    .positions
                    .iter()
                    .filter(|p| p[0] == 8.)
                    .map(|p| (p[1].to_bits(), p[2].to_bits()))
                    .collect();
                points.sort_unstable();
                points.dedup();
                points
            };
            assert_eq!(seam(&left), seam(&right));
            let normals = |mesh: &SurfaceMeshData| {
                let mut samples: Vec<_> = mesh
                    .positions
                    .iter()
                    .zip(&mesh.normals)
                    .filter(|(p, n)| p[0] == 8. && p[1] > 0.001 && n[1] > 0.2)
                    .map(|(p, n)| (*p, *n))
                    .collect();
                samples.sort_by(|a, b| a.0[2].total_cmp(&b.0[2]).then(a.0[1].total_cmp(&b.0[1])));
                samples.dedup_by(|a, b| a.0 == b.0);
                samples
            };
            let ln = normals(&left);
            let rn = normals(&right);
            assert_eq!(ln.len(), rn.len());
            for (a, b) in ln.iter().zip(&rn) {
                assert!(Vec3::from_array(a.1).distance(Vec3::from_array(b.1)) < 0.001);
            }

            assert!(
                left.normals
                    .iter()
                    .chain(&right.normals)
                    .all(|n| (Vec3::from_array(*n).length() - 1.0).abs() < 0.001)
            );
        }
    }

    #[test]
    fn relief_stays_inside_source_cell_and_does_not_replace_raised_models() {
        let g = GridGeometry {
            width: 1,
            height: 1,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let p = [[0., 0., 0.], [0., 0., 8.], [8., 0., 8.], [8., 0., 0.]];
        let s = source(0x08, 0xa8, 0, 1);
        let mut mesh = SurfaceMeshData::default();
        assert!(append(
            &mut mesh,
            p,
            [0., 1., 0.],
            &s,
            &g,
            [0, 0],
            std::array::from_fn(|i| (i == 4).then_some(&s))
        ));
        assert!(mesh.positions.iter().zip(&mesh.uvs).filter(|(p,_)|p[1]>0.001)
            .all(|(_,uv)|uv[0]>90000. && (uv[1]/100000.).round()==7.));
        assert!(mesh.positions.iter().zip(&mesh.uvs).any(|(p,uv)|p[1]==0.
            && uv[0]>90000. && (uv[1]/100000.).round()==5.), "continuous stone backing");
        assert!(mesh.positions.iter().all(|p| (0.0..=8.0).contains(&p[0])
            && (0.0..=8.0).contains(&p[2])
            && (0.0..=4.0).contains(&p[1])));
        assert!(mesh.positions.iter().any(|p| p[0] == 0. && p[1] == 0.));
        assert!(mesh.positions.iter().all(|p| p[1] < 0.001 || p[0] > 0.));
        assert!(!append(
            &mut SurfaceMeshData::default(),
            p.map(|p| [p[0], 6., p[2]]),
            [0., 1., 0.],
            &s,
            &g,
            [0, 0],
            std::array::from_fn(|i| (i == 4).then_some(&s))
        ));
        let mut other = s;
        other.tileset_id = "HallOfFame".into();
        assert!(edges(&other).is_none());
    }
}
