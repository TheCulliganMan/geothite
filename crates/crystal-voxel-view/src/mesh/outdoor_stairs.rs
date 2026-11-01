//! One shared stone stair asset for the authenticated modern Johto ridge glyph.
//! Replace its atlas faces after terrace fitting; retain the walking support.
use super::*;

const LABEL: &str = "outdoor/stone_stairs";

fn stair(s: &VisualTileSource, column: u8) -> bool {
    let phase = if s.tileset_id.as_ref() == "kanto" && s.metatile_id == 0x5c {
        column.saturating_sub(2)
    } else {
        column
    };
    s.subtile_row == 3
        && s.subtile_column == phase
        && ((s.tileset_id.as_ref() == "johto_modern"
            && matches!(s.metatile_id, 0x4b | 0x5a)
            && s.tile_index == 0x5b)
            || (s.tileset_id.as_ref() == "kanto"
                && matches!(s.metatile_id, 0x2f | 0x5c)
                && s.tile_index == 0x04))
}

fn paint(tile: &VisualTile, images: &TerrainImageSamples) -> Option<[f32; 3]> {
    let rgba = tile_rgba(images, tile).ok()?;
    if rgba.len() != 256 || rgba.chunks_exact(4).any(|p| p[3] != 255) {
        return None;
    }
    let colors = rgba
        .chunks_exact(4)
        .map(|p| [p[0], p[1], p[2]])
        .collect::<std::collections::BTreeSet<_>>();
    if !(2..=4).contains(&colors.len()) {
        return None;
    }
    let c = colors
        .into_iter()
        .max_by_key(|c| u32::from(c[0]) * 2126 + u32::from(c[1]) * 7152 + u32::from(c[2]) * 722)?;
    let rgb = c.map(|v| {
        let x = f32::from(v) / 255.;
        if x <= 0.04045 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        }
    });
    let mineral = rgb[0] * 0.2126 + rgb[1] * 0.7152 + rgb[2] * 0.0722;
    // The brightest sprite pixel includes the highlight, not the stone's
    // entire albedo. Retain its mineral tint without making every tread chalk.
    Some(std::array::from_fn(|i| (rgb[i]*0.20+mineral*0.80)*[0.62,0.60,0.56][i]))
}

fn slab(
    out: &mut SurfaceMeshData,
    b: [f32; 4],
    bottom: f32,
    top: [f32; 2],
    bevel: f32,
    color: [f32; 3],
) {
    let [w, e, n, s] = b;
    let rings = [(w, top[0]), (e, top[1])].map(|(x, y)| {
        [
            Vec3::new(x, bottom, n),
            Vec3::new(x, bottom, s),
            Vec3::new(x, y - bevel, s),
            Vec3::new(x, y, s - bevel),
            Vec3::new(x, y, n),
        ]
    });
    let center = rings.iter().flatten().copied().sum::<Vec3>() / 10.;
    let mut faces = vec![rings[0].to_vec(), rings[1].to_vec()];
    for i in 0..5 {
        let j = (i + 1) % 5;
        faces.push(vec![rings[0][i], rings[1][i], rings[1][j], rings[0][j]]);
    }
    for mut p in faces {
        let mut normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
        let middle = p.iter().copied().sum::<Vec3>() / p.len() as f32;
        if normal.dot(middle - center) < 0. {
            p.reverse();
            normal = -normal;
        }
        let shade = 0.72 + normal.dot(Vec3::new(-0.3, 0.85, 0.4).normalize()).max(0.) * 0.28;
        let base = out.positions.len() as u32;
        for point in &p {
            out.positions.push(point.to_array());
            out.normals.push(normal.to_array());
            out.colors
                .push([color[0] * shade, color[1] * shade, color[2] * shade, 1.]);
            out.uvs.push(crate::scenery_finish::SOURCE_PAINT_UV);
        }
        for i in 1..p.len() - 1 {
            out.indices
                .extend([base, base + i as u32, base + i as u32 + 1]);
        }
    }
}

fn append(out: &mut SurfaceMeshData, b: [f32; 4], h: [f32; 4], unit: f32, paint: [f32; 3]) {
    let [w, e, n, s] = b;
    let [nw, ne, se, sw] = h;
    let bottom = h.into_iter().fold(f32::INFINITY, f32::min) - unit * 0.03;
    let flat = (nw - sw).abs().max((ne - se).abs()) < unit * 0.005;
    for row in 0..4 {
        // Each tread meets the ramp at its uphill edge. Sampling the centre
        // left a half-riser gap under the upper landing's exposed edge.
        let t = row as f32 / 4.;
        let lift = if flat {
            unit * 0.045 * (1. - t)
        } else {
            0.
        };
        let heights = [nw + (sw - nw) * t + lift, ne + (se - ne) * t + lift];
        let za = n + (s - n) * row as f32 / 4.;
        let zb = n + (s - n) * (row + 1) as f32 / 4.;
        let split = w + (e - w) * if row % 2 == 0 { 0.48 } else { 0.58 };
        for (col, (a, b)) in [(w, split - unit * 0.006), (split + unit * 0.006, e)]
            .into_iter()
            .enumerate()
        {
            let y = |x: f32| heights[0] + (heights[1] - heights[0]) * (x - w) / (e - w);
            slab(
                out,
                [a, b, za, zb],
                bottom,
                [y(a), y(b)],
                unit * 0.018,
                paint.map(|v| v * (0.97 + ((row + col) % 3) as f32 * 0.015)),
            );
        }
    }
}

pub(super) fn finish(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    images: &TerrainImageSamples,
    origin: [i32;2],
) {
    // Most maps have no such drawing; do not scan their submitted geometry.
    if !cells.iter().any(|tile| stair(&tile.source, 2)) {
        return;
    }
    let count = g.width * g.height;
    if cells.len() != count
        || mesh.authored_cells.len() != count
        || g.tile_width <= 0.
        || g.tile_height <= 0.
    {
        return;
    }
    let mut faces = vec![Vec::new(); count];
    for (number, tri) in mesh.textured.indices.chunks_exact(3).enumerate() {
        let uv = [tri[0], tri[1], tri[2]].map(|i| mesh.textured.uvs[i as usize]);
        let Some(i) = new_bark::sampled_cell(&uv, g) else {
            continue;
        };
        if stair(&cells[i].source, 2) || stair(&cells[i].source, 3) {
            faces[i].push(number);
        }
    }
    let corners = |i: usize| -> Option<[f32; 4]> {
        if faces[i].len() != 2 {
            return None;
        }
        let [w, e, n, s] = {
            let (w, e, n, s) = g.bounds(i % g.width, i / g.width);
            [w, e, n, s]
        };
        let mut heights = [None; 4];
        let eps = g.tile_width.min(g.tile_height) * 0.0002;
        for &number in &faces[i] {
            for &index in &mesh.textured.indices[number * 3..number * 3 + 3] {
                let index = index as usize;
                let p = mesh.textured.positions[index];
                if mesh.textured.normals[index][1] < 0.25
                    || mesh
                        .textured
                        .cutaway_ranges
                        .iter()
                        .any(|r| r.contains(&index))
                {
                    return None;
                }
                let x = if (p[0] - w).abs() < eps {
                    0
                } else if (p[0] - e).abs() < eps {
                    1
                } else {
                    return None;
                };
                let z = if (p[2] - n).abs() < eps {
                    0
                } else if (p[2] - s).abs() < eps {
                    1
                } else {
                    return None;
                };
                let corner = match (x, z) {
                    (0, 0) => 0,
                    (1, 0) => 1,
                    (1, 1) => 2,
                    _ => 3,
                };
                if heights[corner].is_some_and(|y: f32| (y - p[1]).abs() > eps) {
                    return None;
                }
                heights[corner] = Some(p[1]);
            }
        }
        Some([heights[0]?, heights[1]?, heights[2]?, heights[3]?])
    };
    let mut plans = Vec::new();
    for i in 0..count {
        if i % g.width + 1 >= g.width
            || !stair(&cells[i].source, 2)
            || !stair(&cells[i + 1].source, 3)
            || cells[i].source.metatile_id != cells[i + 1].source.metatile_id
            || mesh.authored_cells[i].is_some()
            || mesh.authored_cells[i + 1].is_some()
        {
            continue;
        }
        let (Some(a), Some(b), Some(color)) = (corners(i), corners(i + 1), paint(cells[i], images))
        else {
            continue;
        };
        if paint(cells[i + 1], images) != Some(color)
            || (a[1] - b[0]).abs() > 0.002
            || (a[2] - b[3]).abs() > 0.002
        {
            continue;
        }
        let (w, _, n, s) = g.bounds(i % g.width, i / g.width);
        plans.push((
            i,
            [w, w + 2. * g.tile_width, n, s],
            [a[0], b[1], b[2], a[3]],
            color,
        ));
    }
    let mut removed = std::collections::HashSet::new();
    for (i, b, h, color) in plans {
        let first=mesh.solid.positions.len();
        append(
            &mut mesh.solid,
            b,
            h,
            g.tile_width.min(g.tile_height),
            color,
        );
        for (p,uv) in mesh.solid.positions[first..].iter().zip(&mut mesh.solid.uvs[first..]) {
            *uv=[88000.+(p[0]-g.origin_x+origin[0] as f32*g.tile_width)/8.,
                -20000.-(p[2]-g.origin_z+origin[1] as f32*g.tile_height)/8.];
        }
        removed.extend(faces[i].iter().chain(&faces[i + 1]).copied());
        mesh.authored_cells[i] = Some(LABEL);
        mesh.authored_cells[i + 1] = Some(LABEL);
    }
    if removed.is_empty() {
        return;
    }
    // Keep vertex addresses stable for existing cutaway links and surfaces.
    // The replaced atlas triangles are no longer submitted for drawing.
    mesh.textured.indices = mesh
        .textured
        .indices
        .chunks_exact(3)
        .enumerate()
        .filter(|(number, _)| !removed.contains(number))
        .flat_map(|(_, tri)| tri.iter().copied())
        .collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (
        TerrainMeshData,
        Vec<VisualTile>,
        GridGeometry,
        TerrainImageSamples,
    ) {
        let g = GridGeometry {
            width: 3,
            height: 1,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut assets = Assets::<Image>::default();
        let texture = assets.add(Image::default());
        let mut images = TerrainImageSamples::default();
        images.pixels.insert(
            texture.id(),
            TileImageSample::Rgba(
                (0..64)
                    .flat_map(|i| [[232, 224, 200, 255], [80, 72, 56, 255], [0, 0, 0, 255]][i % 3])
                    .collect(),
            ),
        );
        let tiles = (0..3)
            .map(|i| VisualTile {
                column: i,
                row: 0,
                source: VisualTileSource {
                    tileset_id: "johto_modern".into(),
                    metatile_id: 0x5a,
                    subtile_column: (i + 2) as u8,
                    subtile_row: 3,
                    tile_index: 0x5b,
                },
                texture: texture.clone(),
                animation_frames: None,
                priority: false,
            })
            .collect::<Vec<_>>();
        let mut mesh = TerrainMeshData {
            authored_cells: vec![None; 3],
            footing_heights: vec![3.; 3],
            ..Default::default()
        };
        for i in 0..3 {
            let (w, e, n, s) = g.bounds(i, 0);
            let start = mesh.textured.positions.len();
            append_top(&mut mesh.textured, [w, e, n, s], 0., g.uv(i, 0));
            for p in &mut mesh.textured.positions[start..] {
                p[1] = 6. - p[2] * 0.75;
            }
        }
        (mesh, tiles, g, images)
    }

    #[test]
    fn shared_stairs_replace_only_complete_native_faces_and_keep_walking_support() {
        let (mut mesh, tiles, g, images) = fixture();
        let support = mesh.footing_heights.clone();
        let old_positions = mesh.textured.positions.clone();
        finish(&mut mesh, &tiles.iter().collect::<Vec<_>>(), &g, &images, [0,0]);
        assert_eq!(mesh.authored_cells, vec![Some(LABEL), Some(LABEL), None]);
        assert_eq!(mesh.footing_heights, support);
        assert_eq!(mesh.textured.positions, old_positions);
        assert_eq!(mesh.textured.indices, vec![8, 9, 10, 8, 10, 11]);
        assert_eq!(mesh.solid.indices.len(), 128 * 3);
        assert_eq!(mesh.solid.positions.len(), mesh.solid.normals.len());
        assert_eq!(mesh.solid.positions.len(), mesh.solid.colors.len());
        assert!(
            mesh.solid
                .uvs
                .iter()
                .all(|uv| uv[0]>87000. && uv[0]<90000. && uv[1]< -10000.)
        );
        assert!(
            mesh.solid
                .positions
                .iter()
                .all(|p| p[0] >= 0. && p[0] <= 16. && p[2] >= 0. && p[2] <= 8.)
        );
        // A complete horizontal stone edge seals against the six-unit upper
        // landing. The old centre-sampled tread stopped below this edge.
        assert!(mesh.solid.positions.iter().any(|p|p[0]==0. && p[2]==0. && p[1]==6.));
        assert!(mesh.solid.positions.iter().any(|p|p[0]==16. && p[2]==0. && p[1]==6.));
        assert!(mesh.solid.positions.iter().all(|p|p[1]<=6.));
        // Every individual tread is a closed, outward-facing volume, including
        // its buried bottom. No paper risers or open stair undersides.
        for prism in mesh.solid.indices.chunks_exact(48) {
            let mut edges = std::collections::BTreeMap::new();
            let mut volume = 0.;
            for tri in prism.chunks_exact(3) {
                let points = tri
                    .iter()
                    .map(|&i| Vec3::from_array(mesh.solid.positions[i as usize]))
                    .collect::<Vec<_>>();
                assert!(
                    (points[1] - points[0])
                        .cross(points[2] - points[0])
                        .length()
                        > 0.0001
                );
                volume += points[0].dot(points[1].cross(points[2])) / 6.;
                for k in 0..3 {
                    let key = |p: Vec3| p.to_array().map(|v| (v * 10000.).round() as i32);
                    let mut edge = [key(points[k]), key(points[(k + 1) % 3])];
                    edge.sort();
                    *edges.entry(edge).or_insert(0) += 1;
                }
            }
            assert!(volume > 0.);
            assert!(edges.values().all(|&n| n == 2));
        }
    }

    #[test]
    fn stone_grain_remains_fixed_when_the_viewport_origin_moves() {
        let (mut a, tiles, g, images)=fixture();
        let mut b=a.clone();
        let delta=Vec3::new(-16.,0.,24.);
        for p in &mut b.textured.positions {*p=(Vec3::from_array(*p)+delta).to_array();}
        let shifted=GridGeometry{origin_x:g.origin_x+delta.x,origin_z:g.origin_z+delta.z,..g};
        let cells=tiles.iter().collect::<Vec<_>>();
        finish(&mut a,&cells,&g,&images,[17,9]);
        finish(&mut b,&cells,&shifted,&images,[17,9]);
        assert!(!a.solid.indices.is_empty());
        assert_eq!(a.solid.indices,b.solid.indices);
        assert_eq!(a.solid.uvs,b.solid.uvs);
        for (a,b) in a.solid.colors.iter().zip(&b.solid.colors) {
            assert!(a.iter().zip(b).all(|(a,b)|(a-b).abs()<0.0001));
        }
        assert!(a.solid.colors.iter().all(|c|c[0]<0.65 && c[1]<0.65 && c[2]<0.65));
        for (a,b) in a.solid.positions.iter().zip(&b.solid.positions) {
            assert!((Vec3::from_array(*a)+delta-Vec3::from_array(*b)).length()<0.0001);
        }
    }

    #[test]
    fn kanto_half_bank_stairs_use_the_shared_closed_treads() {
        for block in [0x2f, 0x5c] {
            let (mut mesh, mut tiles, g, images) = fixture();
            for tile in &mut tiles {
                tile.source.tileset_id = "kanto".into();
                tile.source.metatile_id = block;
                if block == 0x5c {
                    tile.source.subtile_column -= 2;
                }
                tile.source.tile_index = 0x04;
            }
            let support = mesh.footing_heights.clone();
            finish(&mut mesh, &tiles.iter().collect::<Vec<_>>(), &g, &images, [0,0]);
            assert_eq!(mesh.authored_cells, vec![Some(LABEL), Some(LABEL), None]);
            assert_eq!(mesh.footing_heights, support);
            assert_eq!(mesh.solid.indices.len(), 128 * 3);
            assert_eq!(mesh.textured.indices.len(), 6);
        }
    }

    #[test]
    fn partial_changed_claimed_or_other_family_drawings_are_left_intact() {
        for fault in 0..7 {
            let (mut mesh, mut tiles, g, mut images) = fixture();
            match fault {
                0 => tiles[1].source.tile_index = 0x5c,
                1 => tiles[0].source.tileset_id = "johto".into(),
                2 => tiles[0].source.metatile_id = 0x5d,
                3 => mesh.authored_cells[1] = Some("another/asset"),
                4 => {
                    mesh.textured.indices.drain(0..3);
                }
                5 => mesh.textured.cutaway_ranges.push(0..4),
                _ => images.pixels.clear(),
            }
            let indices = mesh.textured.indices.clone();
            finish(&mut mesh, &tiles.iter().collect::<Vec<_>>(), &g, &images, [0,0]);
            assert!(mesh.solid.indices.is_empty(), "fault {fault}");
            assert_eq!(mesh.textured.indices, indices, "fault {fault}");
        }
    }
}
