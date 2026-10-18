//! Hall of Fame's native rear course and connected room enclosure.
use super::*;
use bevy::prelude::UVec2;

fn index(frame: &VisualWorldFrame, g: &GridGeometry, x: i32, y: i32) -> Option<usize> {
    let (x, y) = (x - frame.grid_origin.x, y - frame.grid_origin.y);
    (x >= 0 && y >= 0 && (x as usize) < g.width && (y as usize) < g.height)
        .then(|| y as usize * g.width + x as usize)
}
fn wall_cell(x: i32, y: i32) -> bool {
    (0..20).contains(&x) && (y == 0 || (y == 1 && !matches!(x, 6 | 8..=11 | 13)))
}
fn complete(
    mesh: &TerrainMeshData,
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
) -> bool {
    if frame.map_id.as_ref() != "HallOfFame"
        || frame.source_map_size_core_tiles != Some(UVec2::new(10, 14))
        || cells.len() != g.width * g.height
        || mesh.authored_cells.len() != cells.len()
        || frame.boundary_warps.len() != 2
        || ![IVec2::new(4, 13), IVec2::new(5, 13)]
            .iter()
            .all(|p| frame.boundary_warps.contains(p))
    {
        return false;
    }
    let second = [
        83, 83, 83, 83, 83, 83, 40, 83, 44, 45, 45, 46, 83, 41, 83, 83, 83, 83, 83, 83,
    ];
    for y in 0..2 {
        for x in 0..20 {
            let Some(i) = index(frame, g, x, y) else {
                return false;
            };
            let s = &cells[i].source;
            let block = match x {
                0..=3 | 16..=19 => 0x33,
                4..=7 => 0x30,
                8..=11 => 0x31,
                _ => 0x32,
            };
            if s.tileset_id.as_ref() != "ice_path"
                || s.metatile_id != block
                || i32::from(s.subtile_column) != x % 4
                || i32::from(s.subtile_row) != y
                || s.tile_index != if y == 0 { 82 } else { second[x as usize] }
                || (wall_cell(x, y) && mesh.authored_cells[i].is_some())
            {
                return false;
            }
        }
    }
    // Independent native floor beneath the folded drawing, never machine art.
    for y in 2..4 {
        for x in 0..2 {
            let Some(i) = index(frame, g, x, y) else {
                return false;
            };
            let s = &cells[i].source;
            if s.tileset_id.as_ref() != "ice_path"
                || s.metatile_id != 0x33
                || i32::from(s.subtile_column) != x
                || i32::from(s.subtile_row) != y
                || s.tile_index != [[12, 13], [28, 29]][y as usize - 2][x as usize]
            {
                return false;
            }
        }
    }
    true
}

fn panel(frame: &VisualWorldFrame, g: &GridGeometry) -> SurfaceMeshData {
    let mut out = SurfaceMeshData::default();
    let (w, e, n, s) = (0., g.tile_width, -3. * g.tile_height / 8., 0.);
    for band in 0..2 {
        let i = index(frame, g, 0, band).unwrap();
        let uv = g.uv(i % g.width, i / g.width);
        let uv = [[uv.0, uv.3], [uv.1, uv.3], [uv.1, uv.2], [uv.0, uv.2]];
        let top = (32. - band as f32 * 16.) * g.tile_height / 8.;
        let bottom = top - 16. * g.tile_height / 8.;
        for (p, normal) in [
            (
                [[w, bottom, s], [e, bottom, s], [e, top, s], [w, top, s]],
                [0., 0., 1.],
            ),
            (
                [[e, bottom, n], [w, bottom, n], [w, top, n], [e, top, n]],
                [0., 0., -1.],
            ),
            (
                [[w, bottom, n], [w, bottom, s], [w, top, s], [w, top, n]],
                [-1., 0., 0.],
            ),
            (
                [[e, bottom, s], [e, bottom, n], [e, top, n], [e, top, s]],
                [1., 0., 0.],
            ),
        ] {
            append_quad(&mut out, p, normal, uv, TEXTURED_SHADE);
        }
    }
    let i = index(frame, g, 0, 0).unwrap();
    let uv = g.uv(i % g.width, i / g.width);
    let uv = [[uv.0, uv.2], [uv.0, uv.3], [uv.1, uv.3], [uv.1, uv.2]];
    let top = 32. * g.tile_height / 8.;
    append_quad(
        &mut out,
        [[w, top, n], [w, top, s], [e, top, s], [e, top, n]],
        [0., 1., 0.],
        uv,
        TEXTURED_SHADE,
    );
    append_quad(
        &mut out,
        [[w, 0., n], [e, 0., n], [e, 0., s], [w, 0., s]],
        [0., -1., 0.],
        uv,
        TEXTURED_SHADE,
    );
    out
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
) {
    if !complete(mesh, frame, cells, g) {
        return;
    }
    let template = panel(frame, g);
    let west = g.origin_x - frame.grid_origin.x as f32 * g.tile_width;
    let north = g.origin_z - frame.grid_origin.y as f32 * g.tile_height;
    let east = west + 20. * g.tile_width;
    let south = north + 28. * g.tile_height;
    let thickness = 3. * g.tile_height / 8.;
    // The machine begins at source y=1. Its rear wall stands at the map edge,
    // behind that body, rather than cutting through its northern half.
    for x in 0..20 {
        let first = mesh.textured.positions.len();
        mesh.textured.positions.extend(
            template
                .positions
                .iter()
                .map(|p| [p[0] + west + x as f32 * g.tile_width, p[1], p[2] + north]),
        );
        mesh.textured.normals.extend_from_slice(&template.normals);
        mesh.textured.uvs.extend_from_slice(&template.uvs);
        mesh.textured.colors.extend_from_slice(&template.colors);
        mesh.textured
            .indices
            .extend(template.indices.iter().map(|i| *i + first as u32));
        mesh.textured
            .cutaway_ranges
            .push(first..mesh.textured.positions.len());
        mesh.camera_wall_textured_ranges
            .push((first..mesh.textured.positions.len(), [0., 1., north]));
    }
    let bounds = [0., g.tile_width, -thickness, 0.];
    for (edge, at, from, to, step, plane) in [
        (
            Edge::West,
            west,
            north - thickness,
            south + thickness,
            g.tile_height,
            [1., 0., west],
        ),
        (
            Edge::East,
            east,
            north - thickness,
            south + thickness,
            g.tile_height,
            [-1., 0., -east],
        ),
        (
            Edge::South,
            south,
            west,
            west + 8. * g.tile_width,
            g.tile_width,
            [0., -1., -south],
        ),
        (
            Edge::South,
            south,
            west + 12. * g.tile_width,
            east,
            g.tile_width,
            [0., -1., -south],
        ),
    ] {
        let mut start = from;
        while start < to - 0.001 {
            let end = (start + step).min(to);
            let first = mesh.textured.positions.len();
            copy_section(&mut mesh.textured, &template, bounds, edge, at, start, end);
            mesh.camera_wall_textured_ranges
                .push((first..mesh.textured.positions.len(), plane));
            start = end;
        }
    }
}

pub(in crate::mesh) fn finish(
    mesh: &mut TerrainMeshData,
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
) {
    if !complete(mesh, frame, cells, g) {
        return;
    }
    // Remove only the old horizontal wall pixels. Machine geometry, vertical
    // room faces, collision and resolved floor support remain independent.
    for surface in [
        &mut mesh.textured,
        &mut mesh.solid,
        &mut mesh.animated_textured,
        &mut mesh.animated_solid,
    ] {
        surface.indices = surface
            .indices
            .chunks_exact(3)
            .filter(|tri| {
                if !tri
                    .iter()
                    .all(|&i| surface.positions[i as usize][1].abs() < 0.001)
                    || surface.normals[tri[0] as usize][1] < 0.99
                {
                    return true;
                }
                let c = tri
                    .iter()
                    .map(|&i| Vec3::from_array(surface.positions[i as usize]))
                    .sum::<Vec3>()
                    / 3.;
                !wall_cell(
                    frame.grid_origin.x + ((c.x - g.origin_x) / g.tile_width).floor() as i32,
                    frame.grid_origin.y + ((c.z - g.origin_z) / g.tile_height).floor() as i32,
                )
            })
            .flatten()
            .copied()
            .collect();
    }
    for y in 0..2 {
        for x in 0..20 {
            if !wall_cell(x, y) {
                continue;
            }
            let i = index(frame, g, x, y).unwrap();
            let floor = index(frame, g, x % 2, 2 + y).unwrap();
            append_top(
                &mut mesh.textured,
                g.bounds(i % g.width, i / g.width).into(),
                0.,
                g.uv(floor % g.width, floor / g.width),
            );
            mesh.authored_cells[i] = Some("hall:rear-wall");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (
        VisualWorldFrame,
        GridGeometry,
        Vec<VisualTile>,
        TerrainMeshData,
    ) {
        let path = std::path::PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap());
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let pack = loaded.pack();
        let data = &pack.data().maps["HallOfFame"];
        let art = &pack.runtime_files()["data/tilesets/ice_path_metatiles.bin"];
        let (w, h) = (
            data.attributes.width as usize * 4,
            data.attributes.height as usize * 4,
        );
        let tiles = (0..w * h)
            .map(|i| {
                let (x, y) = (i % w, i / w);
                let block = data.blocks[y / 4 * (w / 4) + x / 4];
                VisualTile {
                    column: x as u32,
                    row: y as u32,
                    texture: Default::default(),
                    priority: false,
                    animation_frames: None,
                    source: VisualTileSource {
                        tileset_id: "ice_path".into(),
                        metatile_id: block,
                        subtile_column: (x % 4) as u8,
                        subtile_row: (y % 4) as u8,
                        tile_index: art[block as usize * 16 + y % 4 * 4 + x % 4] as u16,
                    },
                }
            })
            .collect::<Vec<_>>();
        let mut frame = super::super::super::super::tests::frame(
            w as u32,
            h as u32,
            tiles.iter().map(|t| t.source.clone()).collect(),
        );
        frame.map_id = "HallOfFame".into();
        frame.source_map_size_core_tiles = Some(UVec2::new(10, 14));
        frame.boundary_warps = data
            .events
            .warps
            .iter()
            .map(|w| IVec2::new(i32::from(w.x), i32::from(w.y)))
            .collect();
        let g = GridGeometry {
            width: w,
            height: h,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mesh = TerrainMeshData {
            authored_cells: vec![None; w * h],
            footing_heights: vec![2.75; w * h],
            ..Default::default()
        };
        (frame, g, tiles, mesh)
    }

    #[test]
    fn external_hall_room_has_native_closed_walls_connected_sides_and_an_open_door() {
        let (frame, g, tiles, mut mesh) = fixture();
        let refs = tiles.iter().collect::<Vec<_>>();
        append(&mut mesh, &frame, &refs, &g);
        assert!(!mesh.textured.indices.is_empty());
        assert_eq!(mesh.footing_heights, vec![2.75; tiles.len()]);
        for tri in mesh.textured.indices.chunks_exact(3) {
            let [a, b, c] = [tri[0], tri[1], tri[2]]
                .map(|i| Vec3::from_array(mesh.textured.positions[i as usize]));
            let n = Vec3::from_array(mesh.textured.normals[tri[0] as usize]);
            assert!(
                (b - a).cross(c - a).dot(n) > 0.,
                "all closed panel faces point outwards"
            );
        }
        for (axis, at) in [(0, 0.), (0, 160.), (2, 0.)] {
            let mut spans = Vec::new();
            for (range, plane) in &mesh.camera_wall_textured_ranges {
                if plane[if axis == 0 { 0 } else { 1 }] == 0. || (plane[2].abs() - at).abs() > 0.001
                {
                    continue;
                }
                let ps = &mesh.textured.positions[range.clone()];
                assert!(
                    ps.iter()
                        .all(|p| plane[0] * p[0] + plane[1] * p[2] <= plane[2] + 0.001),
                    "walls stay outside the floor"
                );
                assert_eq!(ps.iter().map(|p| p[1]).fold(0., f32::max), 32.);
                let along = if axis == 0 { 2 } else { 0 };
                spans.push((
                    ps.iter().map(|p| p[along]).fold(f32::INFINITY, f32::min),
                    ps.iter()
                        .map(|p| p[along])
                        .fold(f32::NEG_INFINITY, f32::max),
                ));
            }
            spans.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut end = 0.;
            for (a, b) in spans {
                assert!(a <= end + 0.001, "a gap remains before {a}");
                end = end.max(b);
            }
            assert!(end >= if axis == 0 { 224. } else { 160. });
        }
        for (range, plane) in &mesh.camera_wall_textured_ranges {
            if plane[1] >= 0. {
                continue;
            }
            let ps = &mesh.textured.positions[range.clone()];
            assert!(
                ps.iter().all(|p| p[0] <= 64.) || ps.iter().all(|p| p[0] >= 96.),
                "Lance doorway must stay open"
            );
        }
        let mut front_area = 0.;
        for tri in mesh.textured.indices.chunks_exact(3) {
            if mesh.textured.normals[tri[0] as usize] != [0., 0., 1.] {
                continue;
            }
            let ps = [tri[0], tri[1], tri[2]]
                .map(|i| Vec3::from_array(mesh.textured.positions[i as usize]));
            if !ps.iter().all(|p| p.z == 0.) {
                continue;
            }
            front_area += (ps[1] - ps[0]).cross(ps[2] - ps[0]).length() * 0.5;
            assert!(
                tri.iter()
                    .any(|&i| mesh.textured.uvs[i as usize] != mesh.textured.uvs[tri[0] as usize]),
                "native drawing must not collapse to flat paint"
            );
        }
        assert_eq!(
            front_area,
            160. * 32.,
            "complete rear drawing stands behind the machine"
        );
        finish(&mut mesh, &frame, &refs, &g);
        assert_eq!(
            mesh.authored_cells
                .iter()
                .filter(|&&c| c == Some("hall:rear-wall"))
                .count(),
            34
        );
        assert_eq!(mesh.footing_heights, vec![2.75; tiles.len()]);
        for y in 0..2 {
            for x in [6, 8, 9, 10, 11, 13] {
                if y == 1 {
                    assert_eq!(
                        mesh.authored_cells[y * g.width + x],
                        None,
                        "machine drawing stays independent"
                    );
                }
            }
        }
    }

    #[test]
    fn external_hall_room_rejects_edited_fields_custom_claims_and_cropped_sources() {
        let (frame, g, tiles, mesh) = fixture();
        for i in 0..g.width * 2 {
            let mut edited = tiles.clone();
            edited[i].source.tile_index ^= 1;
            let refs = edited.iter().collect::<Vec<_>>();
            let mut result = TerrainMeshData {
                authored_cells: mesh.authored_cells.clone(),
                ..Default::default()
            };
            append(&mut result, &frame, &refs, &g);
            assert!(
                result.textured.positions.is_empty(),
                "edited native course cell {i} must reject the enclosure"
            );
        }
        let refs = tiles.iter().collect::<Vec<_>>();
        let mut custom = TerrainMeshData {
            authored_cells: mesh.authored_cells.clone(),
            ..Default::default()
        };
        custom.authored_cells[0] = Some("custom wall");
        append(&mut custom, &frame, &refs, &g);
        finish(&mut custom, &frame, &refs, &g);
        assert!(custom.textured.positions.is_empty());
        assert_eq!(custom.authored_cells[1], None);
        let mut cropped = frame.clone();
        cropped.grid_origin.y = 2;
        let cg = GridGeometry {
            height: g.height - 2,
            ..g
        };
        let refs = tiles[g.width * 2..].iter().collect::<Vec<_>>();
        let mut result = TerrainMeshData {
            authored_cells: vec![None; refs.len()],
            ..Default::default()
        };
        append(&mut result, &cropped, &refs, &cg);
        assert!(result.textured.positions.is_empty());
        let mut other = frame.clone();
        other.map_id = "IcePath1F".into();
        assert!(!complete(
            &mesh,
            &other,
            &tiles.iter().collect::<Vec<_>>(),
            &g
        ));
    }
    #[test]
    fn external_hall_room_folds_only_wall_pixels_and_preserves_machine_support() {
        let (frame, g, tiles, mut mesh) = fixture();
        let refs = tiles.iter().collect::<Vec<_>>();
        for i in 0..tiles.len() {
            append_top(
                &mut mesh.textured,
                g.bounds(i % g.width, i / g.width).into(),
                0.,
                g.uv(i % g.width, i / g.width),
            );
        }
        append(&mut mesh, &frame, &refs, &g);
        super::super::super::hall_machine::finish(&mut mesh, &frame, &refs, &g, None);
        let support = mesh.footing_heights.clone();
        let protected = mesh
            .textured
            .indices
            .chunks_exact(3)
            .filter(|tri| {
                tri.iter()
                    .any(|&i| mesh.textured.positions[i as usize][1] > 0.001)
            })
            .map(|tri| [tri[0], tri[1], tri[2]])
            .collect::<Vec<_>>();
        finish(&mut mesh, &frame, &refs, &g);
        assert_eq!(
            mesh.footing_heights, support,
            "folding a wall must not change stair or floor contact"
        );
        let retained = mesh
            .textured
            .indices
            .chunks_exact(3)
            .map(|tri| [tri[0], tri[1], tri[2]])
            .collect::<std::collections::HashSet<_>>();
        assert!(
            protected.iter().all(|tri| retained.contains(tri)),
            "machine and closed wall volumes must remain intact"
        );
        assert_eq!(
            mesh.authored_cells
                .iter()
                .filter(|&&c| c == Some("hall:recording-machine"))
                .count(),
            46
        );
        assert_eq!(
            mesh.authored_cells
                .iter()
                .filter(|&&c| c == Some("hall:rear-wall"))
                .count(),
            34
        );
        let mut ground = 0;
        for tri in mesh.textured.indices.chunks_exact(3) {
            if mesh.textured.normals[tri[0] as usize][1] < 0.99
                || !tri
                    .iter()
                    .all(|&i| mesh.textured.positions[i as usize][1].abs() < 0.001)
            {
                continue;
            }
            let c = tri
                .iter()
                .map(|&i| Vec3::from_array(mesh.textured.positions[i as usize]))
                .sum::<Vec3>()
                / 3.;
            let (x, y) = ((c.x / 8.).floor() as i32, (c.z / 8.).floor() as i32);
            if !wall_cell(x, y) {
                continue;
            }
            ground += 1;
            let uv = g.uv((x % 2) as usize, (2 + y) as usize);
            assert!(
                tri.iter().all(|&i| {
                    let p = mesh.textured.uvs[i as usize];
                    p[0] >= uv.0 && p[0] <= uv.1 && p[1] >= uv.2 && p[1] <= uv.3
                }),
                "folded art must leave native floor, not a second horizontal wall drawing"
            );
        }
        assert_eq!(
            ground, 68,
            "each folded source cell has exactly two floor triangles"
        );
        let count = mesh.textured.positions.len();
        finish(&mut mesh, &frame, &refs, &g);
        assert_eq!(
            mesh.textured.positions.len(),
            count,
            "a second finish must not duplicate wall underlay"
        );
    }
}
