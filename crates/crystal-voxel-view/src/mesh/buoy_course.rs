//! One source-owned floating course for Johto and Johto Modern.
//! Float paint comes from the existing atlas; its bed uses ordinary water.
use super::*;

pub(super) const LABEL: &str = "outdoor/buoy-course";

pub(super) fn source(s: &VisualTileSource) -> bool {
    matches!(s.tileset_id.as_ref(), "johto" | "johto_modern")
        && (0x30..=0x39).contains(&s.metatile_id)
        && s.tile_index == 0x58
        && s.subtile_column < 4
        && s.subtile_row < 4
        && matches!(shape_for_source(s),CellShape::PlaneAt {height} if height == crate::profile::WATER_HEIGHT)
}

// The dominant color is the water background. Keep only the largest connected
// foreground component: isolated wave marks must not become extra floats.
fn silhouette(pixels: &[u8]) -> Option<([usize; 4], [usize; 2])> {
    if pixels.len() != 256 {
        return None;
    }
    let mut counts = std::collections::BTreeMap::new();
    for p in pixels.chunks_exact(4) {
        let p: [u8; 4] = p.try_into().ok()?;
        if p[3] != 255 {
            return None;
        }
        *counts.entry(p).or_insert(0usize) += 1;
    }
    let background = counts.into_iter().max_by_key(|(_, count)| *count)?.0;
    let mut seen = [false; 64];
    let mut largest = Vec::new();
    for start in 0..64 {
        if seen[start] || pixels[start * 4..start * 4 + 4] == background {
            continue;
        }
        let mut component = Vec::new();
        let mut pending = vec![start];
        seen[start] = true;
        while let Some(i) = pending.pop() {
            component.push(i);
            let x = i % 8;
            let z = i / 8;
            for (nx, nz) in [
                (x as isize - 1, z as isize),
                (x as isize + 1, z as isize),
                (x as isize, z as isize - 1),
                (x as isize, z as isize + 1),
            ] {
                if !(0..8).contains(&nx) || !(0..8).contains(&nz) {
                    continue;
                }
                let next = nz as usize * 8 + nx as usize;
                if !seen[next] && pixels[next * 4..next * 4 + 4] != background {
                    seen[next] = true;
                    pending.push(next);
                }
            }
        }
        if component.len() > largest.len() {
            largest = component;
        }
    }
    if largest.len() < 4 || largest.len() > 48 {
        return None;
    }
    let b = [
        largest.iter().map(|i| i % 8).min()?,
        largest.iter().map(|i| i % 8 + 1).max()?,
        largest.iter().map(|i| i / 8).min()?,
        largest.iter().map(|i| i / 8 + 1).max()?,
    ];
    let luminance = |i: usize| -> u32 {
        let p = &pixels[i * 4..i * 4 + 4];
        u32::from(p[0]) * 3 + u32::from(p[1]) * 6 + u32::from(p[2])
    };
    let bright = *largest.iter().max_by_key(|i| luminance(**i))?;
    let mid = largest
        .iter()
        .copied()
        .filter(|i| luminance(*i) < luminance(bright))
        .max_by_key(|i| luminance(*i))
        .unwrap_or(bright);
    Some((b, [bright, mid]))
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    images: &TerrainImageSamples,
    reserved: &[bool],
) {
    let water = CellShape::Water.surface_height(g.tile_height);
    // Only a native, complete, unclaimed horizontal tile is replaceable.
    // Borrowed samples, cropped planes and custom live profiles retain their art.
    let mut planes = Vec::new();
    for (q, (p, uv)) in mesh
        .textured
        .positions
        .chunks_exact(4)
        .zip(mesh.textured.uvs.chunks_exact(4))
        .enumerate()
    {
        let Some(i) = new_bark::sampled_cell(uv, g) else {
            continue;
        };
        if !source(&cells[i].source)
            || reserved[i]
            || mesh.authored_cells.get(i).is_some_and(|c| c.is_some())
        {
            continue;
        }
        let (w, e, n, s) = g.bounds(i % g.width, i / g.width);
        let expected = [[w, water, s], [e, water, s], [e, water, n], [w, water, n]];
        if !p.iter().all(|v| expected.contains(v)) || expected.iter().any(|v| !p.contains(v)) {
            continue;
        }
        if mesh.textured.normals[q * 4..q * 4 + 4]
            .iter()
            .any(|n| n[1] < 0.999)
        {
            continue;
        }
        planes.push((q, i));
    }
    let mut consumed = std::collections::HashSet::new();
    for (q, i) in planes {
        if consumed.contains(&i) {
            continue;
        }
        let Some(donor) = cells.iter().position(|t| {
            t.source.tileset_id == cells[i].source.tileset_id
                && t.source.tile_index == 0x14
                && matches!(shape_for_source(&t.source), CellShape::Water)
        }) else {
            continue;
        };
        let Ok(pixels) = tile_rgba(images, cells[i]) else {
            continue;
        };
        let Some((b, paint)) = silhouette(pixels) else {
            continue;
        };
        let (u0, u1, v0, v1) = g.uv(i % g.width, i / g.width);
        let swatches = paint.map(|p| {
            [
                u0 + (u1 - u0) * (p % 8) as f32 / 8. + (u1 - u0) / 16.,
                v0 + (v1 - v0) * (p / 8) as f32 / 8. + (v1 - v0) / 16.,
            ]
        });
        let (w, _, n, _) = g.bounds(i % g.width, i / g.width);
        let center = Vec3::new(
            w + (b[0] + b[1]) as f32 / 16. * g.tile_width,
            water,
            n + (b[2] + b[3]) as f32 / 16. * g.tile_height,
        );
        let radius =
            ((b[1] - b[0]) as f32 * g.tile_width).min((b[3] - b[2]) as f32 * g.tile_height) / 16.
                * 0.90;
        float(&mut mesh.textured, center, radius, g.tile_height, swatches);
        let (a, b, c, d) = g.uv(donor % g.width, donor / g.width);
        mesh.textured.uvs[q * 4..q * 4 + 4].copy_from_slice(&[[a, d], [b, d], [b, c], [a, c]]);
        mesh.authored_cells[i] = Some(LABEL);
        consumed.insert(i);
    }
}

fn float(out: &mut SurfaceMeshData, center: Vec3, radius: f32, tile: f32, paint: [[f32; 2]; 2]) {
    let levels = [(-0.08, 0.70), (0.05, 1.0), (0.30, 0.30)];
    let rings: Vec<[Vec3; 12]> = levels
        .into_iter()
        .map(|(y, r)| {
            std::array::from_fn(|i| {
                let a = i as f32 * std::f32::consts::TAU / 12.;
                center + Vec3::new(a.cos() * radius * r, y * tile, a.sin() * radius * r)
            })
        })
        .collect();
    for row in 0..2 {
        for side in 0..12 {
            let next = (side + 1) % 12;
            let outward = (rings[row][side] - center).with_y(0.);
            let uv = paint[1];
            face(
                out,
                [rings[row][side], rings[row + 1][side], rings[row + 1][next]],
                outward,
                uv,
            );
            face(
                out,
                [rings[row][side], rings[row + 1][next], rings[row][next]],
                outward,
                uv,
            );
        }
    }
    for side in 0..12 {
        face(
            out,
            [
                rings[2][side],
                center + Vec3::Y * tile * 0.34,
                rings[2][(side + 1) % 12],
            ],
            Vec3::Y,
            paint[0],
        );
        face(
            out,
            [
                rings[0][side],
                center - Vec3::Y * tile * 0.08,
                rings[0][(side + 1) % 12],
            ],
            Vec3::NEG_Y,
            paint[1],
        );
    }
}

fn face(out: &mut SurfaceMeshData, mut p: [Vec3; 3], outward: Vec3, uv: [f32; 2]) {
    if (p[1] - p[0]).cross(p[2] - p[0]).dot(outward) < 0. {
        p.swap(1, 2);
    }
    let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize().to_array();
    // Reuse the native prop face lighting. The original middle swatch owns
    // the broad body; white is confined to the small crown above it.
    let n = Vec3::from_array(normal);
    let x = direction_shade(if n.x < 0. {
        Direction::West
    } else {
        Direction::East
    });
    let z = direction_shade(if n.z < 0. {
        Direction::North
    } else {
        Direction::South
    });
    let y = if n.y > 0. {
        1.
    } else {
        direction_shade(Direction::North)
    };
    let shade =
        (x * n.x.abs() + z * n.z.abs() + y * n.y.abs()) / (n.x.abs() + n.y.abs() + n.z.abs());
    let color = [shade, shade, shade, 1.];
    // Ground finishing stores faces as quads. Split the fan triangle into
    // three genuine planar quads so it remains compatible with that path.
    let center = (p[0] + p[1] + p[2]) / 3.;
    for i in 0..3 {
        append_quad(
            out,
            [
                p[i].to_array(),
                ((p[i] + p[(i + 1) % 3]) * 0.5).to_array(),
                center.to_array(),
                ((p[i] + p[(i + 2) % 3]) * 0.5).to_array(),
            ],
            normal,
            [uv; 4],
            color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pixels() -> Vec<u8> {
        let mut p = [20, 70, 120, 255].repeat(64);
        for z in 2..6 {
            for x in 2..6 {
                p[(z * 8 + x) * 4..(z * 8 + x + 1) * 4].copy_from_slice(&[170, 170, 170, 255]);
            }
        }
        p[(3 * 8 + 3) * 4..(3 * 8 + 4) * 4].copy_from_slice(&[255, 255, 255, 255]);
        p[0..4].copy_from_slice(&[255, 255, 255, 255]); // detached wave highlight
        p
    }
    #[test]
    fn silhouette_ignores_detached_waves_and_preserves_source_swatches() {
        let (bounds, paint) = silhouette(&pixels()).unwrap();
        assert_eq!(bounds, [2, 6, 2, 6]);
        assert_eq!(paint[0], 27);
        assert_ne!(paint[1], 0);
        assert!(silhouette(&[0, 0, 255, 255].repeat(64)).is_none());
        assert!(silhouette(&[0; 255]).is_none());
    }
    fn fixture() -> (
        Vec<VisualTile>,
        GridGeometry,
        TerrainImageSamples,
        TerrainMeshData,
    ) {
        let f = super::super::tests::frame(
            2,
            1,
            vec![
                super::super::tests::source_with_tile(0x34, 0, 0, 0x58),
                super::super::tests::source_with_tile(0x34, 1, 0, 0x14),
            ],
        );
        let g = GridGeometry {
            width: 2,
            height: 1,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut samples = TerrainImageSamples::default();
        samples
            .pixels
            .insert(f.tiles[0].texture.id(), TileImageSample::Rgba(pixels()));
        let mut mesh = TerrainMeshData {
            authored_cells: vec![None; 2],
            footing_heights: vec![0., -2.],
            ..Default::default()
        };
        append_top(&mut mesh.textured, [0., 8., 0., 8.], -2., g.uv(0, 0));
        append_top(&mut mesh.textured, [8., 16., 0., 8.], -2., g.uv(1, 0));
        (f.tiles, g, samples, mesh)
    }
    #[test]
    fn native_course_reuses_water_bed_and_source_paint_without_moving_footing() {
        let (tiles, g, samples, mut mesh) = fixture();
        let refs: Vec<_> = tiles.iter().collect();
        append(&mut mesh, &refs, &g, &samples, &[false; 2]);
        assert_eq!(mesh.footing_heights, vec![0., -2.]);
        assert_eq!(mesh.authored_cells, vec![Some(LABEL), None]);
        assert_eq!(new_bark::sampled_cell(&mesh.textured.uvs[..4], &g), Some(1));
        let float = &mesh.textured.positions[8..];
        assert!(!float.is_empty());
        assert!(
            float
                .iter()
                .all(|p| p[0] > 0. && p[0] < 8. && p[2] > 0. && p[2] < 8.)
        );
        assert!(float.iter().any(|p| p[1] < -2.));
        assert!(float.iter().any(|p| p[1] > 0.));
        assert!(
            mesh.textured.uvs[8..]
                .iter()
                .all(|uv| new_bark::sampled_cell(&[*uv], &g) == Some(0))
        );
        assert_eq!(mesh.textured.positions.len() % 4, 0);
        let before = mesh.textured.positions.len();
        append(&mut mesh, &refs, &g, &samples, &[false; 2]);
        assert_eq!(
            mesh.textured.positions.len(),
            before,
            "the same cell cannot be claimed twice"
        );
        new_bark::polish_world_surfaces(&mut mesh, "CherrygroveCity", &refs, &g, [0, 0]);
        assert!(mesh.solid.positions.contains(&[0., -2., 8.]));
        assert!(
            !mesh
                .textured
                .positions
                .chunks_exact(4)
                .any(|q| q.iter().all(|p| p[1] == -2.)),
            "old buoy and water planes must use the shared water finish"
        );
        assert_eq!(mesh.footing_heights, vec![0., -2.]);
        assert!(
            mesh.textured.positions.iter().any(|p| p[1] > 0.),
            "the float survives ground finishing"
        );
    }
    #[test]
    fn cropped_reserved_unknown_and_missing_art_retain_the_original_plane() {
        for kind in 0..5 {
            let (mut tiles, g, samples, mut mesh) = fixture();
            let mut reserved = [false; 2];
            match kind {
                0 => reserved[0] = true,
                1 => mesh.textured.positions[0][0] += 1.,
                2 => tiles[0].source.tile_index = 0x57,
                3 => tiles[1].source.tileset_id = std::sync::Arc::from("kanto"),
                _ => {}
            }
            let samples = if kind == 4 {
                TerrainImageSamples::default()
            } else {
                samples
            };
            let refs: Vec<_> = tiles.iter().collect();
            let original = mesh.textured.positions.clone();
            let uv = mesh.textured.uvs.clone();
            append(&mut mesh, &refs, &g, &samples, &reserved);
            assert_eq!(mesh.textured.positions, original);
            assert_eq!(mesh.textured.uvs, uv);
            assert_eq!(mesh.authored_cells, vec![None; 2]);
        }
    }
    #[test]
    fn float_is_closed_consistently_wound_and_non_degenerate() {
        let mut mesh = SurfaceMeshData::default();
        float(
            &mut mesh,
            Vec3::new(4., -2., 4.),
            2.,
            8.,
            [[0.1, 0.2], [0.3, 0.4]],
        );
        let key = |p: [f32; 3]| p.map(|v| (v * 10000.).round() as i32);
        let mut edges = std::collections::BTreeMap::new();
        let mut volume = 0.;
        for triangle in mesh.indices.chunks_exact(3) {
            let ids: [u32; 3] = triangle.try_into().unwrap();
            let p = ids.map(|i| Vec3::from_array(mesh.positions[i as usize]));
            assert!((p[1] - p[0]).cross(p[2] - p[0]).length_squared() > 1e-10);
            volume += p[0].dot(p[1].cross(p[2])) / 6.;
            for j in 0..3 {
                let a = key(p[j].to_array());
                let b = key(p[(j + 1) % 3].to_array());
                let (edge, direction) = if a < b { ((a, b), 1) } else { ((b, a), -1) };
                let e = edges.entry(edge).or_insert((0, 0));
                e.0 += 1;
                e.1 += direction;
            }
        }
        assert!(volume > 0.);
        assert!(edges.values().all(|e| *e == (2, 0)));
    }
}
